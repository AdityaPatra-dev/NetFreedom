use anyhow::Result;
use netfreedom_daemon::orchestrator::NetFreedomOrchestrator;
use tokio::signal;
use tracing::{info, warn};

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
    let gateway_ip = "10.6.80.1".to_string();
    let physical_dev = "wlp195s0".to_string();

    let mut orchestrator = NetFreedomOrchestrator::new(tun_dev, gateway_ip, physical_dev);

    info!("Daemon initialized. Listening for client control and system events...");

    // Wait for termination signal (Ctrl+C / SIGTERM / systemd stop)
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received termination signal (SIGINT). Shutting down cleanly...");
        }
    }

    if let Err(e) = orchestrator.disconnect().await {
        warn!("Error during daemon disconnect cleanup: {}", e);
    }

    info!("NetFreedom Daemon stopped. Routing and firewall restored.");
    Ok(())
}
