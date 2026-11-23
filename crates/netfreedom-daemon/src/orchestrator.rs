use anyhow::Result;
use netfreedom_common::{
    ConnectionProfile, ConnectionStatus, DpiConfig, HealthMetrics, OperatingMode, WarpConfig,
};
use netfreedom_dpi::DpiSupervisor;
use std::path::PathBuf;
use tracing::info;

pub struct NetFreedomOrchestrator {
    pub status: ConnectionStatus,
    pub active_mode: Option<OperatingMode>,
    pub active_profile: Option<ConnectionProfile>,
    pub metrics: HealthMetrics,
    pub tun_interface: String,
    pub gateway_ip: String,
    pub physical_dev: String,
    dpi_supervisor: Option<DpiSupervisor>,
}

impl NetFreedomOrchestrator {
    pub fn new(tun_interface: String, gateway_ip: String, physical_dev: String) -> Self {
        Self {
            status: ConnectionStatus::Disconnected,
            active_mode: None,
            active_profile: None,
            metrics: HealthMetrics::default(),
            tun_interface,
            gateway_ip,
            physical_dev,
            dpi_supervisor: None,
        }
    }

    /// Connects using the ByeDPI serverless desynchronization mode.
    pub async fn connect_dpi(
        &mut self,
        config: DpiConfig,
        ciadpi_path: PathBuf,
        profile_name: String,
    ) -> Result<()> {
        info!("Initiating ByeDPI Desynchronization connection: {}", profile_name);
        self.status = ConnectionStatus::Connecting;

        // 1. Spawn ciadpi process
        let mut supervisor = DpiSupervisor::new(config.clone(), ciadpi_path);
        supervisor.start().await?;
        self.dpi_supervisor = Some(supervisor);

        self.active_mode = Some(OperatingMode::DpiDesync);
        self.status = ConnectionStatus::Connected;
        info!("Successfully connected in ByeDPI mode on 127.0.0.1:{}", config.local_port);

        Ok(())
    }

    /// Connects using the Cloudflare WARP Shield mode.
    pub async fn connect_warp(
        &mut self,
        warp_config: WarpConfig,
        profile_name: String,
        enable_killswitch: bool,
    ) -> Result<()> {
        info!("Initiating Cloudflare WARP Shield connection: {}", profile_name);
        self.status = ConnectionStatus::Connecting;

        let endpoint_ip = warp_config.endpoint.ip().to_string();

        // 1. Configure safe /1 routes (never deleting default 0.0.0.0/0)
        let _ = netfreedom_tun::apply_double_slash_one_routes(
            &self.tun_interface,
            &self.gateway_ip,
            &self.physical_dev,
            Some(&endpoint_ip),
        );

        // 2. Configure systemd-resolved DNS
        let _ = netfreedom_dns::configure_link_dns(&self.tun_interface, &["1.1.1.1", "1.0.0.1"]);

        // 3. Configure nftables kill switch if requested
        if enable_killswitch {
            let _ = netfreedom_security::apply_killswitch_rules(
                &self.tun_interface,
                Some(warp_config.endpoint),
                true,
                Some("10.6.80.0/20"),
            );
        }

        self.active_mode = Some(OperatingMode::WarpShield);
        self.status = ConnectionStatus::Connected;
        info!("Successfully connected to Cloudflare WARP Shield at {}", warp_config.endpoint);

        Ok(())
    }

    /// Cleanly disconnects and restores all system routing, DNS, and firewall settings.
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting NetFreedom and restoring original system state");

        // 1. Stop DPI supervisor if running
        if let Some(mut supervisor) = self.dpi_supervisor.take() {
            let _ = supervisor.stop().await;
        }

        // 2. Remove safe /1 routes
        let _ = netfreedom_tun::remove_double_slash_one_routes(&self.tun_interface, None);

        // 3. Revert systemd-resolved DNS
        let _ = netfreedom_dns::revert_link_dns(&self.tun_interface);

        // 4. Remove nftables kill switch
        let _ = netfreedom_security::remove_killswitch_rules();

        self.status = ConnectionStatus::Disconnected;
        self.active_mode = None;
        self.active_profile = None;

        info!("System state cleanly restored");
        Ok(())
    }
}
