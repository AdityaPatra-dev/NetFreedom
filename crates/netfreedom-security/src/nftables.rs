use anyhow::Result;
use std::io::Write;
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use tracing::{info, warn};

pub const TABLE_NAME: &str = "netfreedom_killswitch";

/// Builds an nftables ruleset that drops all outgoing unencrypted traffic
/// while strictly allowing loopback, the virtual tunnel interface, and optionally LAN / VPN endpoint.
pub fn generate_nftables_ruleset(
    tun_interface: &str,
    endpoint: Option<SocketAddr>,
    allow_lan: bool,
    lan_subnet: Option<&str>,
) -> String {
    let mut rules = String::new();
    rules.push_str(&format!("table inet {} {{\n", TABLE_NAME));
    rules.push_str("    chain output {\n");
    rules.push_str("        type filter hook output priority -10; policy drop;\n");
    rules.push_str("        oifname \"lo\" accept\n");
    rules.push_str(&format!("        oifname \"{}\" accept\n", tun_interface));

    // Allow DHCP so the device doesn't lose its IP lease
    rules.push_str("        udp sport 68 udp dport 67 accept\n");

    // Allow LAN traffic if user selected LAN-permitted mode
    if allow_lan {
        let subnet = lan_subnet.unwrap_or("10.0.0.0/8");
        rules.push_str(&format!("        ip daddr {} accept\n", subnet));
        rules.push_str("        ip daddr 192.168.0.0/16 accept\n");
        rules.push_str("        ip daddr 172.16.0.0/12 accept\n");
    }

    // Allow connection to the VPN endpoint
    if let Some(ep) = endpoint {
        rules.push_str(&format!(
            "        ip daddr {} udp dport {} accept\n",
            ep.ip(),
            ep.port()
        ));
    }

    rules.push_str("    }\n");
    rules.push_str("}\n");
    rules
}

/// Applies the nftables kill switch ruleset to the Linux kernel.
pub fn apply_killswitch_rules(
    tun_interface: &str,
    endpoint: Option<SocketAddr>,
    allow_lan: bool,
    lan_subnet: Option<&str>,
) -> Result<()> {
    let ruleset = generate_nftables_ruleset(tun_interface, endpoint, allow_lan, lan_subnet);
    info!("Applying nftables killswitch ruleset");

    let mut child = Command::new("nft")
        .arg("-f")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(ruleset.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("nftables failed to apply ruleset: {}", err));
    }

    info!("Kill switch successfully activated");
    Ok(())
}

/// Removes the nftables kill switch table, restoring normal unrestricted egress.
pub fn remove_killswitch_rules() -> Result<()> {
    info!("Removing nftables killswitch ruleset");
    let output = Command::new("nft")
        .args(["delete", "table", "inet", TABLE_NAME])
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                warn!("nft delete table returned non-zero (table may already be absent)");
            }
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to execute nft command: {}", e)),
    }
}

/// Checks if the netfreedom_killswitch table exists in nftables.
pub fn is_killswitch_active() -> bool {
    let status = Command::new("nft")
        .args(["list", "table", "inet", TABLE_NAME])
        .output();

    matches!(status, Ok(out) if out.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nftables_ruleset_generation() {
        let endpoint = "162.159.192.1:2408".parse().unwrap();
        let rules = generate_nftables_ruleset("netfreedom0", Some(endpoint), true, Some("10.6.80.0/20"));

        assert!(rules.contains("table inet netfreedom_killswitch"));
        assert!(rules.contains("oifname \"netfreedom0\" accept"));
        assert!(rules.contains("ip daddr 10.6.80.0/20 accept"));
        assert!(rules.contains("ip daddr 162.159.192.1 udp dport 2408 accept"));
        assert!(rules.contains("udp sport 68 udp dport 67 accept"));
    }
}
