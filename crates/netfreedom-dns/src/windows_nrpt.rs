use anyhow::Result;
#[cfg(target_os = "windows")]
use std::process::Command;
use tracing::info;

pub const NRPT_RULE_NAME: &str = "NetFreedomDNS";

/// Generates the PowerShell script to enforce the Name Resolution Policy Table (NRPT) rule on Windows.
/// This prevents Smart Multi-Homed Name Resolution (SMHNR) from leaking DNS queries to physical adapters.
pub fn generate_nrpt_enable_script(dns_servers: &[&str]) -> String {
    let servers_formatted = dns_servers
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "Add-DnsClientNrptRule -Namespace '.' -NameServers @({}) -DisplayName '{}'",
        servers_formatted, NRPT_RULE_NAME
    )
}

/// Generates the PowerShell script to remove the NetFreedom NRPT rule.
pub fn generate_nrpt_disable_script() -> String {
    format!(
        "Get-DnsClientNrptRule | Where-Object {{ $_.DisplayName -eq '{}' }} | Remove-DnsClientNrptRule -Force",
        NRPT_RULE_NAME
    )
}

/// Applies the NRPT rule on a live Windows system via PowerShell.
pub fn apply_windows_nrpt(dns_servers: &[&str]) -> Result<()> {
    info!("Enforcing Windows NRPT policy rule for DNS servers: {:?}", dns_servers);
    let script = generate_nrpt_enable_script(dns_servers);

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to apply Windows NRPT DNS rule"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("[Cross-Platform Emulation] NRPT Script generated: {}", script);
    }

    Ok(())
}

/// Removes the NRPT rule on Windows, restoring standard DNS resolution.
pub fn remove_windows_nrpt() -> Result<()> {
    info!("Removing Windows NRPT policy rule");
    let script = generate_nrpt_disable_script();

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status();
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!("[Cross-Platform Emulation] NRPT Removal Script generated: {}", script);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nrpt_script_generation() {
        let script = generate_nrpt_enable_script(&["1.1.1.1", "1.0.0.1"]);
        assert!(script.contains("Add-DnsClientNrptRule"));
        assert!(script.contains("1.1.1.1"));
        assert!(script.contains(NRPT_RULE_NAME));

        let disable_script = generate_nrpt_disable_script();
        assert!(disable_script.contains("Remove-DnsClientNrptRule"));
        assert!(disable_script.contains(NRPT_RULE_NAME));
    }
}
