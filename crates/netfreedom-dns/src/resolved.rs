use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Configures systemd-resolved via resolvectl to force all system DNS queries
/// through the specified tunnel link with routing domain "~." (catch-all).
pub fn configure_link_dns(link: &str, dns_servers: &[&str]) -> Result<()> {
    info!("Configuring systemd-resolved for link {} with DNS {:?}", link, dns_servers);

    // 1. Set the DNS servers for the link
    let mut cmd_dns = Command::new("resolvectl");
    cmd_dns.arg("dns").arg(link);
    for server in dns_servers {
        cmd_dns.arg(server);
    }
    let status_dns = cmd_dns.status()?;
    if !status_dns.success() {
        return Err(anyhow::anyhow!("Failed to set DNS servers on link {}", link));
    }

    // 2. Set domain "~." so all DNS queries exclusively route through this link
    let status_domain = Command::new("resolvectl")
        .args(["domain", link, "~."])
        .status()?;
    if !status_domain.success() {
        warn!("Failed to set routing domain '~.' on link {}", link);
    }

    // 3. Set DNSDefaultRoute=yes
    let _ = Command::new("resolvectl")
        .args(["default-route", link, "yes"])
        .status();

    info!("Link {} DNS successfully configured with catch-all '~.'", link);
    Ok(())
}

/// Reverts the link configuration in systemd-resolved, restoring original DNS.
pub fn revert_link_dns(link: &str) -> Result<()> {
    info!("Reverting systemd-resolved configuration for link {}", link);
    let output = Command::new("resolvectl").args(["revert", link]).output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                warn!("resolvectl revert returned non-zero (link may already be deleted)");
            }
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to execute resolvectl revert: {}", e)),
    }
}
