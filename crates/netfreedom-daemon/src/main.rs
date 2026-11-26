use anyhow::Result;
use netfreedom_daemon::orchestrator::NetFreedomOrchestrator;
use tokio::signal;
use tracing::{info, warn};

fn detect_default_gateway_and_iface() -> (String, String) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/net/route") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[1] == "00000000" {
                    let iface = parts[0].to_string();
                    if let Ok(gw_hex) = u32::from_str_radix(parts[2], 16) {
                        let bytes = gw_hex.to_le_bytes();
                        let ip = format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]);
                        return (iface, ip);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Select-Object -First 1).NextHop",
            ])
            .output();
        if let Ok(out) = output {
            let hop = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !hop.is_empty() {
                return ("Ethernet".to_string(), hop);
            }
        }
    }

    ("eth0".to_string(), "192.168.1.1".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    println!("============================================================");
    println!("        NetFreedom Daemon: Background Service Running       ");
    println!("============================================================");

    info!("Initializing NetFreedom background daemon");

    let tun_dev = "netfreedom0".to_string();
    let (physical_dev, gateway_ip) = detect_default_gateway_and_iface();
    info!("Detected active network: interface={}, gateway={}", physical_dev, gateway_ip);

    let orchestrator = std::sync::Arc::new(tokio::sync::Mutex::new(
        NetFreedomOrchestrator::new(tun_dev, gateway_ip, physical_dev),
    ));

    info!("Daemon initialized. Listening for client control and system events...");

    // Start background IPC server for CLI and GUI communication
    let orch_clone = orchestrator.clone();
    tokio::spawn(async move {
        let _ = netfreedom_daemon::start_ipc_listener(orch_clone).await;
    });

    // Wait for termination signal (Ctrl+C / SIGTERM / systemd stop)
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received termination signal (SIGINT). Shutting down cleanly...");
        }
    }

    let mut orch = orchestrator.lock().await;
    if let Err(e) = orch.disconnect().await {
        warn!("Error during daemon disconnect cleanup: {}", e);
    }

    info!("NetFreedom Daemon stopped. Routing and firewall restored.");
    Ok(())
}
