use anyhow::Result;
#[cfg(target_os = "windows")]
use std::process::Command;
use tracing::info;

pub const WINDOWS_SERVICE_NAME: &str = "NetFreedomService";
pub const WINDOWS_NAMED_PIPE: &str = r"\\.\pipe\NetFreedomIPC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsServiceState {
    Stopped,
    StartPending,
    StopPending,
    Running,
}

/// Helper to generate commands for registering NetFreedom as a native Windows Service
/// running under NT AUTHORITY\LocalSystem for driver and WFP privileges.
pub struct WindowsServiceManager;

impl WindowsServiceManager {
    /// Generates the sc.exe command string to install the Windows Service.
    pub fn generate_install_command(exe_path: &str) -> String {
        format!(
            "sc.exe create {} binPath= \"{}\" start= auto DisplayName= \"NetFreedom Core Service\"",
            WINDOWS_SERVICE_NAME, exe_path
        )
    }

    /// Generates the sc.exe command string to delete the Windows Service.
    pub fn generate_uninstall_command() -> String {
        format!("sc.exe delete {}", WINDOWS_SERVICE_NAME)
    }

    /// Installs the service on a Windows system.
    pub fn install_service(exe_path: &str) -> Result<()> {
        info!("Registering Windows Service: {}", WINDOWS_SERVICE_NAME);

        #[cfg(target_os = "windows")]
        {
            let status = Command::new("sc.exe")
                .args(["create", WINDOWS_SERVICE_NAME, &format!("binPath= \"{}\"", exe_path), "start=", "auto", "DisplayName=", "NetFreedom Core Service"])
                .status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to register Windows Service"));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            info!("[Cross-Platform Emulation] Install command: {}", Self::generate_install_command(exe_path));
        }

        Ok(())
    }

    /// Uninstalls the service on a Windows system.
    pub fn uninstall_service() -> Result<()> {
        info!("Unregistering Windows Service: {}", WINDOWS_SERVICE_NAME);

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("sc.exe").args(["delete", WINDOWS_SERVICE_NAME]).status();
        }

        #[cfg(not(target_os = "windows"))]
        {
            info!("[Cross-Platform Emulation] Uninstall command: {}", Self::generate_uninstall_command());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_service_commands() {
        let install = WindowsServiceManager::generate_install_command(r"C:\Program Files\NetFreedom\netfreedom-daemon.exe");
        assert!(install.contains("sc.exe create NetFreedomService"));
        assert!(install.contains("C:\\Program Files\\NetFreedom\\netfreedom-daemon.exe"));
        assert!(install.contains("start= auto"));

        let uninstall = WindowsServiceManager::generate_uninstall_command();
        assert_eq!(uninstall, "sc.exe delete NetFreedomService");
    }
}
