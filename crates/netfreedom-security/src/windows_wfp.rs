use anyhow::Result;
use std::net::SocketAddr;
#[cfg(target_os = "windows")]
use std::process::Command;
use tracing::info;

pub const WFP_RULE_PREFIX: &str = "NetFreedom_KillSwitch";

/// Generates the PowerShell script to configure Windows Advanced Firewall (WFP) rules for the kill switch.
pub fn generate_windows_firewall_script(
    _tun_interface: &str,
    endpoint: Option<SocketAddr>,
    allow_lan: bool,
) -> String {
    let mut script = String::new();

    // 1. Block all outbound by default on high priority rule
    script.push_str(&format!(
        "New-NetFirewallRule -DisplayName '{}_BlockAll' -Direction Outbound -Action Block -Profile Any;\n",
        WFP_RULE_PREFIX
    ));

    // 2. Allow Loopback
    script.push_str(&format!(
        "New-NetFirewallRule -DisplayName '{}_AllowLoopback' -Direction Outbound -Action Allow -RemoteAddress '127.0.0.1' -Profile Any;\n",
        WFP_RULE_PREFIX
    ));

    // 3. Allow DHCP
    script.push_str(&format!(
        "New-NetFirewallRule -DisplayName '{}_AllowDHCP' -Direction Outbound -Action Allow -Protocol UDP -LocalPort 68 -RemotePort 67 -Profile Any;\n",
        WFP_RULE_PREFIX
    ));

    // 4. Allow LAN if enabled
    if allow_lan {
        script.push_str(&format!(
            "New-NetFirewallRule -DisplayName '{}_AllowLAN' -Direction Outbound -Action Allow -RemoteAddress @('10.0.0.0/8','192.168.0.0/16','172.16.0.0/12') -Profile Any;\n",
            WFP_RULE_PREFIX
        ));
    }

    // 5. Allow VPN endpoint
    if let Some(ep) = endpoint {
        script.push_str(&format!(
            "New-NetFirewallRule -DisplayName '{}_AllowEndpoint' -Direction Outbound -Action Allow -RemoteAddress '{}' -Protocol UDP -RemotePort {} -Profile Any;\n",
            WFP_RULE_PREFIX, ep.ip(), ep.port()
        ));
    }

    script
}

/// Generates the PowerShell script to remove the NetFreedom firewall rules.
pub fn generate_windows_firewall_cleanup_script() -> String {
    format!(
        "Get-NetFirewallRule -DisplayName '{}*' | Remove-NetFirewallRule;\n",
        WFP_RULE_PREFIX
    )
}

/// Applies the kill switch rules on Windows.
pub fn apply_windows_wfp_killswitch(
    tun_interface: &str,
    endpoint: Option<SocketAddr>,
    allow_lan: bool,
) -> Result<()> {
    info!("Enforcing Windows Firewall (WFP) kill switch");
    let script = generate_windows_firewall_script(tun_interface, endpoint, allow_lan);

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to apply Windows Firewall kill switch"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("[Cross-Platform Emulation] Windows WFP Script generated: {}", script);
    }

    Ok(())
}

/// Removes the kill switch rules on Windows.
pub fn remove_windows_wfp_killswitch() -> Result<()> {
    info!("Removing Windows Firewall (WFP) kill switch");
    let script = generate_windows_firewall_cleanup_script();

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status();
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("[Cross-Platform Emulation] Windows WFP Cleanup Script generated: {}", script);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_firewall_script_generation() {
        let ep = "162.159.192.1:2408".parse().unwrap();
        let script = generate_windows_firewall_script("NetFreedomTUN", Some(ep), true);

        assert!(script.contains("New-NetFirewallRule"));
        assert!(script.contains("NetFreedom_KillSwitch_BlockAll"));
        assert!(script.contains("162.159.192.1"));
        assert!(script.contains("10.0.0.0/8"));

        let cleanup = generate_windows_firewall_cleanup_script();
        assert!(cleanup.contains("Remove-NetFirewallRule"));
    }
}
