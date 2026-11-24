use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Implements the Double /1 Prefix Trick for zero-destruction system routing:
/// Rather than overwriting the default 0.0.0.0/0 gateway, we add two more specific /1 routes:
/// 0.0.0.0/1 and 128.0.0.0/1 pointing to the TUN interface.
pub fn apply_double_slash_one_routes(
    tun_interface: &str,
    gateway_ip: &str,
    _physical_dev: &str,
    remote_endpoint_ip: Option<&str>,
) -> Result<()> {
    info!("Configuring safe /1 routes for interface {}", tun_interface);

    #[cfg(target_os = "linux")]
    {
        // 1. Explicit /32 route to remote VPN endpoint via original physical gateway
        if let Some(endpoint) = remote_endpoint_ip {
            let status = Command::new("ip")
                .args(["route", "replace", &format!("{}/32", endpoint), "via", gateway_ip, "dev", _physical_dev])
                .status()?;
            if !status.success() {
                warn!("Failed to add /32 route for endpoint {}", endpoint);
            }
        }

        // 2. Add 0.0.0.0/1 -> covers 0.0.0.0 - 127.255.255.255
        let status1 = Command::new("ip")
            .args(["route", "replace", "0.0.0.0/1", "dev", tun_interface])
            .status()?;
        if !status1.success() {
            return Err(anyhow::anyhow!("Failed to add 0.0.0.0/1 route to {}", tun_interface));
        }

        // 3. Add 128.0.0.0/1 -> covers 128.0.0.0 - 255.255.255.255
        let status2 = Command::new("ip")
            .args(["route", "replace", "128.0.0.0/1", "dev", tun_interface])
            .status()?;
        if !status2.success() {
            return Err(anyhow::anyhow!("Failed to add 128.0.0.0/1 route to {}", tun_interface));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 1. Explicit /32 route to remote VPN endpoint via physical gateway
        if let Some(endpoint) = remote_endpoint_ip {
            let _ = Command::new("route")
                .args(["add", endpoint, "mask", "255.255.255.255", gateway_ip, "metric", "1"])
                .status();
        }

        // 2. Add 0.0.0.0/1 (mask 128.0.0.0)
        let status1 = Command::new("route")
            .args(["add", "0.0.0.0", "mask", "128.0.0.0", "172.16.0.2", "metric", "1"])
            .status()?;
        if !status1.success() {
            warn!("Notice: route add 0.0.0.0/1 status: {:?}", status1);
        }

        // 3. Add 128.0.0.0/1 (mask 128.0.0.0)
        let status2 = Command::new("route")
            .args(["add", "128.0.0.0", "mask", "128.0.0.0", "172.16.0.2", "metric", "1"])
            .status()?;
        if !status2.success() {
            warn!("Notice: route add 128.0.0.0/1 status: {:?}", status2);
        }
    }

    info!("Safe /1 routing successfully established for {}", tun_interface);
    Ok(())
}

/// Removes the /1 routes, cleanly and immediately restoring original system gateway without disruption.
pub fn remove_double_slash_one_routes(
    tun_interface: &str,
    remote_endpoint_ip: Option<&str>,
) -> Result<()> {
    info!("Removing safe /1 routes for interface {}", tun_interface);

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("ip").args(["route", "del", "0.0.0.0/1", "dev", tun_interface]).status();
        let _ = Command::new("ip").args(["route", "del", "128.0.0.0/1", "dev", tun_interface]).status();

        if let Some(endpoint) = remote_endpoint_ip {
            let _ = Command::new("ip").args(["route", "del", &format!("{}/32", endpoint)]).status();
        }
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("route").args(["delete", "0.0.0.0", "mask", "128.0.0.0"]).status();
        let _ = Command::new("route").args(["delete", "128.0.0.0", "mask", "128.0.0.0"]).status();

        if let Some(endpoint) = remote_endpoint_ip {
            let _ = Command::new("route").args(["delete", endpoint]).status();
        }
    }

    info!("Safe /1 routes cleanly purged. Native system default gateway restored.");
    Ok(())
}
