// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use netfreedom_common::{ConnectionStatus, HealthMetrics, OperatingMode};

#[tauri::command]
fn get_connection_status() -> ConnectionStatus {
    ConnectionStatus::Disconnected
}

#[tauri::command]
fn get_health_metrics() -> HealthMetrics {
    HealthMetrics {
        tcp_latency_ms: Some(46.1),
        jitter_ms: Some(1.3),
        download_bytes_sec: 0,
        upload_bytes_sec: 0,
        total_rx_bytes: 0,
        total_tx_bytes: 0,
        public_ip: Some("123.63.59.157".to_string()),
        country: Some("India".to_string()),
        colo: Some("MAA".to_string()),
        captive_portal_detected: false,
        ipv6_working: false,
    }
}

#[tauri::command]
async fn connect_profile(mode: String) -> Result<String, String> {
    Ok(format!("Connecting to mode: {}", mode))
}

#[tauri::command]
async fn disconnect_profile() -> Result<String, String> {
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
