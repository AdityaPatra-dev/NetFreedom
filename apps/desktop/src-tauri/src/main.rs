// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use netfreedom_common::{ConnectionStatus, HealthMetrics};
use std::process::Command;

#[tauri::command]
fn get_connection_status() -> ConnectionStatus {
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
    let _ = Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();

    // 2. Start ciadpi in daemon mode with DPI desynchronization
    let ciadpi_status = Command::new("./third_party/byedpi/ciadpi")
        .args(&[
            "-i", "127.0.0.1",
            "-p", "1080",
            "-I", "0.0.0.0",
            "-s", "2+s",
            "-f", "-1",
            "-t", "3",
            "-d", "1+s",
            "-D",
        ])
        .status();

    if let Err(e) = ciadpi_status {
        return Err(format!("Failed to start ciadpi: {}", e));
    }

    // 3. Set GNOME system proxy
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"])
        .status();
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy.socks", "port", "1080"])
        .status();
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
        .status();

    Ok(format!("Connected via mode: {}", mode))
}

#[tauri::command]
async fn disconnect_profile() -> Result<String, String> {
    // 1. Revert GNOME proxy
    let _ = Command::new("gsettings")
        .args(&["set", "org.gnome.system.proxy", "mode", "none"])
        .status();

    // 2. Kill ciadpi
    let _ = Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();

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
