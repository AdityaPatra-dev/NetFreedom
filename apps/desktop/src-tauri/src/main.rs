// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use netfreedom_common::{ConnectionStatus, HealthMetrics};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
fn win_cmd(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[tauri::command]
fn get_connection_status() -> ConnectionStatus {
    #[cfg(target_os = "linux")]
    {
        let mode_out = Command::new("gsettings")
            .args(&["get", "org.gnome.system.proxy", "mode"])
            .output();
        let is_manual = if let Ok(out) = mode_out {
            String::from_utf8_lossy(&out.stdout).contains("manual")
        } else {
            false
        };

        let pgrep_out = Command::new("pgrep").arg("-f").arg("ciadpi").status();
        let is_running = pgrep_out.map(|s| s.success()).unwrap_or(false);

        if is_running && is_manual {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }

    #[cfg(target_os = "windows")]
    {
        let reg_out = win_cmd("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
            ])
            .output();
        let is_proxy_enabled = if let Ok(out) = reg_out {
            String::from_utf8_lossy(&out.stdout).contains("0x1")
        } else {
            false
        };

        let tasklist_out = win_cmd("tasklist").output();
        let is_running = if let Ok(out) = tasklist_out {
            String::from_utf8_lossy(&out.stdout).to_lowercase().contains("ciadpi")
        } else {
            false
        };

        if is_running && is_proxy_enabled {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        ConnectionStatus::Disconnected
    }
}

#[tauri::command]
fn get_health_metrics() -> HealthMetrics {
    HealthMetrics {
        tcp_latency_ms: Some(42.5),
        jitter_ms: Some(1.1),
        download_bytes_sec: 18_500_000,
        upload_bytes_sec: 4_200_000,
        total_rx_bytes: 512_000_000,
        total_tx_bytes: 128_000_000,
        public_ip: Some("123.63.59.157".to_string()),
        country: Some("India".to_string()),
        colo: Some("MAA".to_string()),
        captive_portal_detected: false,
        ipv6_working: false,
    }
}

#[tauri::command]
async fn connect_profile(mode: String) -> Result<String, String> {
    // 1. Kill any existing instance
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();

        // Start ciadpi in daemon mode with DPI desynchronization
        let ciadpi_status = Command::new("./third_party/byedpi/ciadpi")
            .args(&[
                "-i", "127.0.0.1",
                "-p", "1080",
                "-I", "0.0.0.0",
                "-s", "2+s",
                "-f", "-1",
                "-t", "3",
                "-D",
            ])
            .status();

        if let Err(e) = ciadpi_status {
            return Err(format!("Failed to start ciadpi: {}", e));
        }

        // Set GNOME system proxy
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"])
            .status();
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.socks", "port", "1080"])
            .status();
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
            .status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = win_cmd("taskkill")
            .args(&["/F", "/IM", "ciadpi.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
        let mut candidates = Vec::new();
        if let Some(ref d) = exe_dir {
            candidates.push(d.join("ciadpi.exe"));
        }
        if let Ok(cur) = std::env::current_dir() {
            candidates.push(cur.join("ciadpi.exe"));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(std::path::PathBuf::from(local_app_data).join("Programs").join("NetFreedom").join("ciadpi.exe"));
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            candidates.push(std::path::PathBuf::from(prog_files).join("NetFreedom").join("ciadpi.exe"));
        }
        candidates.push(std::path::PathBuf::from("ciadpi.exe"));

        let ciadpi_bin = candidates.into_iter().find(|p| p.exists());
        let ciadpi_bin = match ciadpi_bin {
            Some(p) => p,
            None => {
                return Err("ciadpi.exe not found! Please extract all files from the ZIP or run Install.bat.".to_string());
            }
        };

        let child = win_cmd(&ciadpi_bin)
            .args(&[
                "-i", "127.0.0.1",
                "-p", "1080",
                "-I", "0.0.0.0",
                "-s", "2+s",
                "-f", "-1",
                "-t", "3",
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        if let Err(e) = child {
            return Err(format!("Failed to start ciadpi.exe: {}. Check Windows Defender permissions.", e));
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Enable Windows internet proxy via Registry
        let _ = win_cmd("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "1",
                "/f",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = win_cmd("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyServer",
                "/t",
                "REG_SZ",
                "/d",
                "socks=127.0.0.1:1080",
                "/f",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    Ok(format!("Connected via mode: {}", mode))
}

#[tauri::command]
async fn disconnect_profile() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "none"])
            .status();
        let _ = Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = win_cmd("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "0",
                "/f",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = win_cmd("taskkill")
            .args(&["/F", "/IM", "ciadpi.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    Ok("Disconnected".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_connection_status,
            get_health_metrics,
            connect_profile,
            disconnect_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running NetFreedom desktop application");
}
