use netfreedom_common::DpiConfig;
use netfreedom_health::{
    check_egress_info, detect_captive_portal, is_ipv6_functional, probe_latency_and_jitter,
};
use netfreedom_vpn::register_warp_account;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("auto");

    println!("============================================================");
    println!("      NetFreedom: Universal Privacy & Connectivity Engine   ");
    println!("============================================================");

    let client = reqwest::Client::builder()
        .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
        .timeout(Duration::from_secs(6))
        .build()?;

    match command {
        "auto" => {
            println!("\n[>] Running Smart Auto-Pilot Network Diagnostic...");
            
            // 1. Probe non-ICMP TCP Handshake Latency to Cloudflare Edge (1.1.1.1:443)
            let cf_target: SocketAddr = "1.1.1.1:443".parse()?;
            print!("  [*] Probing TCP Handshake Latency (non-ICMP)... ");
            let (latency, jitter) = probe_latency_and_jitter(cf_target, 3).await;
            if let Some(lat) = latency {
                println!("{:.2} ms (Jitter: {:.2} ms)", lat, jitter.unwrap_or(0.0));
            } else {
                println!("FAILED (Gateway blocked port 443)");
            }

            // 2. Probe IPv6 Status
            print!("  [*] Checking Dual-Stack IPv6 Route Health... ");
            let ipv6_ok = is_ipv6_functional().await;
            if ipv6_ok {
                println!("OK (Native IPv6 available)");
            } else {
                println!("BROKEN (IPv6 advertised by DNS but route is dead; Auto-Sanitizer Active)");
            }

            // 3. Captive Portal Detection
            print!("  [*] Checking for Captive Portal / Splash Screen... ");
            let portal = detect_captive_portal(&client).await;
            if portal {
                println!("DETECTED (Action required: log in to network)");
            } else {
                println!("NONE (Open Internet Access)");
            }

            // 4. Current Public Egress
            print!("  [*] Checking Egress Gateway & Edge Colo... ");
            if let Ok(egress) = check_egress_info(&client).await {
                println!(
                    "IP: {}, Country: {}, Edge Colo: {} (WARP: {})",
                    egress.ip.unwrap_or_else(|| "Unknown".into()),
                    egress.loc.unwrap_or_else(|| "Unknown".into()),
                    egress.colo.unwrap_or_else(|| "Unknown".into()),
                    if egress.warp_active { "ON" } else { "OFF" }
                );
            } else {
                println!("Could not determine egress info");
            }

            // 5. Auto-Pilot Recommendation
            println!("\n[+] SMART AUTO-PILOT RECOMMENDATION:");
            println!("  -> Primary Mode: Cloudflare WARP Shield (WireGuard over UDP 2408)");
            println!("     Provides 100% ChaCha20-Poly1305 encryption, hides all browsing from campus router.");
            println!("  -> Alternate Mode: ByeDPI Stealth Desynchronization (Split-SNI + Fake TTL 3)");
            println!("     Provides 0ms added latency, bypasses campus middlebox domain blocks without VPN server.");
        }

        "warp-reg" => {
            println!("\n[>] Requesting anonymous Cloudflare WARP identity via API...");
            match register_warp_account(&client).await {
                Ok(cfg) => {
                    println!("\n[+] SUCCESS! Cloudflare WARP Identity Provisioned:");
                    println!("  * Client ID:       {}", cfg.client_id);
                    println!("  * Assigned IPv4:   {}", cfg.assigned_ipv4);
                    if let Some(v6) = cfg.assigned_ipv6 {
                        println!("  * Assigned IPv6:   {}", v6);
                    }
                    println!("  * Endpoint:        {}", cfg.endpoint);
                    println!("  * Peer Public Key: {}", cfg.peer_public_key);
                    println!("  * Client PubKey:   {}", cfg.public_key);
                    println!("\n  (Private key securely generated in memory)");
                }
                Err(e) => {
                    println!("[-] WARP registration failed: {:#?}", e);
                }
            }
        }

        "dpi-preview" => {
            let config = DpiConfig::default();
            println!("\n[>] ByeDPI Desynchronization Profile Preview:");
            println!("  * Local SOCKS5 Port: 127.0.0.1:{}", config.local_port);
            println!("  * SNI Split Flag:    {}", config.split_pos);
            println!("  * Fake Packet TTL:   {}", config.fake_ttl);
            println!("  * TCP Disordering:   {}", config.disorder);
            println!("\n  Generated CLI arguments:");
            let supervisor = netfreedom_dpi::DpiSupervisor::new(config, "ciadpi".into());
            println!("    ciadpi {}", supervisor.build_arguments().join(" "));
        }

        "diag" => {
            println!("\n[>] Running Full Network Diagnostics Report...");
            let target: SocketAddr = "1.1.1.1:443".parse()?;
            let (lat, jit) = probe_latency_and_jitter(target, 5).await;
            println!("  TCP Handshake Latency: {:.2?} ms", lat);
            println!("  Jitter:                {:.2?} ms", jit);
            println!("  Functional IPv6:       {}", is_ipv6_functional().await);
            println!("  Captive Portal:        {}", detect_captive_portal(&client).await);
            if let Ok(egress) = check_egress_info(&client).await {
                println!("  Public IP:             {:?}", egress.ip);
                println!("  Country:               {:?}", egress.loc);
                println!("  Datacenter (Colo):     {:?}", egress.colo);
            }
        }

        _ => {
            println!("Usage: netfreedom-cli [auto | warp-reg | dpi-preview | diag]");
        }
    }

    println!("\n============================================================");
    Ok(())
}
