use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Implements the Double /1 Prefix Trick for zero-destruction system routing:
/// Rather than overwriting the default 0.0.0.0/0 gateway (which risks breaking network connectivity),
/// we add two more specific /1 routes: 0.0.0.0/1 and 128.0.0.0/1 pointing to the TUN interface.
pub fn apply_double_slash_one_routes(
    tun_interface: &str,
    gateway_ip: &str,
    physical_dev: &str,
    remote_endpoint_ip: Option<&str>,
) -> Result<()> {
    info!("Configuring safe /1 routes for interface {}", tun_interface);

    // 1. Explicit /32 route to remote VPN endpoint via original physical gateway
    if let Some(endpoint) = remote_endpoint_ip {
        let status = Command::new("ip")
            .args(["route", "replace", &format!("{}/32", endpoint), "via", gateway_ip, "dev", physical_dev])
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

    info!("Safe /1 routing successfully established for {}", tun_interface);
    Ok(())
}

/// Removes the /1 routes, cleanly and immediately restoring original system gateway without disruption.
pub fn remove_double_slash_one_routes(
    tun_interface: &str,
    remote_endpoint_ip: Option<&str>,
) -> Result<()> {
    info!("Removing /1 routes for interface {}", tun_interface);

    let _ = Command::new("ip").args(["route", "del", "0.0.0.0/1", "dev", tun_interface]).status();
    let _ = Command::new("ip").args(["route", "del", "128.0.0.0/1", "dev", tun_interface]).status();

    if let Some(endpoint) = remote_endpoint_ip {
        let _ = Command::new("ip").args(["route", "del", &format!("{}/32", endpoint)]).status();
    }

    info!("Routes cleaned up. Original system gateway intact.");
    Ok(())
}
