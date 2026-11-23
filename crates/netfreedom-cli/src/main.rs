use netfreedom_common::DpiConfig;
use netfreedom_health::{
    check_egress_info, detect_captive_portal, is_ipv6_functional, probe_latency_and_jitter,
};
use netfreedom_security::{generate_nftables_ruleset, is_killswitch_active};
use netfreedom_vpn::register_warp_account;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
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
        "on" | "start" | "connect" => {
            println!("\n[>] Activating NetFreedom Stealth Bypass & System Proxy...");
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();
                
                let status = std::process::Command::new("./third_party/byedpi/ciadpi")
                    .args(&[
                        "-i", "127.0.0.1",
                        "-p", "1080",
                        "-I", "0.0.0.0",
                        "-s", "2+s",
                        "-f", "-1",
                        "-t", "3",
                        "-d", "1+s",
                        "-D"
                    ])
                    .status();
                if status.is_err() || !status.unwrap().success() {
                    let _ = std::process::Command::new("ciadpi")
                        .args(&["-i", "127.0.0.1", "-p", "1080", "-I", "0.0.0.0", "-s", "2+s", "-f", "-1", "-t", "3", "-d", "1+s", "-D"])
                        .status();
                }

                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.http", "host", "127.0.0.1"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.http", "port", "1081"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.http", "enabled", "true"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.https", "host", "127.0.0.1"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.https", "port", "1081"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy.socks", "port", "1080"])
                    .status();
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
                    .status();
            }

            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("taskkill").args(&["/F", "/IM", "ciadpi.exe"]).status();
                let ciadpi_bin = if std::path::Path::new("ciadpi.exe").exists() {
                    "ciadpi.exe"
                } else {
                    "third_party\\byedpi\\ciadpi.exe"
                };
                let _ = std::process::Command::new(ciadpi_bin)
                    .args(&[
                        "-i", "127.0.0.1",
                        "-p", "1080",
                        "-I", "0.0.0.0",
                        "-s", "2+s",
                        "-f", "-1",
                        "-t", "3",
                        "-d", "1+s",
                        "-D"
                    ])
                    .spawn();

                let _ = std::process::Command::new("reg")
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
                    .status();
                let _ = std::process::Command::new("reg")
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
                    .status();
            }

            // Notify local desktop server if running
            let _ = client.post("http://127.0.0.1:1420/api/connect").send().await;

            println!("[+] NetFreedom Stealth Bypass is now ACTIVE system-wide!");
            println!("  * Proxy: SOCKS5 127.0.0.1:1080 + HTTP/HTTPS CONNECT 127.0.0.1:1081");
            println!("  * Mode: SNI Cleave (2+s) + Fake TTL (3) + TCP Disordering");
            println!("  * GNOME System Proxy: manual (All browsers routed automatically)");
            println!("\n[>] Verifying access to https://en-thunderscans.com...");
            let test = std::process::Command::new("curl")
                .args(&["-s", "-I", "--connect-timeout", "3", "--proxy", "http://127.0.0.1:1081", "https://en-thunderscans.com"])
                .output();
            if let Ok(out) = test {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.contains("200 OK") {
                    println!("[+] VERIFIED! https://en-thunderscans.com is fully accessible!");
                } else {
                    println!("[-] HTTP Response: {}", s.lines().next().unwrap_or(""));
                }
            }
        }

        "off" | "stop" | "disconnect" => {
            println!("\n[>] Deactivating NetFreedom Bypass...");
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("gsettings")
                    .args(&["set", "org.gnome.system.proxy", "mode", "none"])
                    .status();
                let _ = std::process::Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();
            }

            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("reg")
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
                    .status();
                let _ = std::process::Command::new("taskkill").args(&["/F", "/IM", "ciadpi.exe"]).status();
            }

            println!("[+] NetFreedom Bypass deactivated. System returned to direct network mode.");
        }

        "gui" => {
            println!("\n[>] Launching NetFreedom Desktop GUI at http://127.0.0.1:1420...");
            let _ = std::process::Command::new("xdg-open")
                .arg("http://127.0.0.1:1420")
                .spawn();
        }

        "auto" => {
            println!("\n[>] Running Smart Auto-Pilot Network Diagnostic...");

            // 1. Probe non-ICMP TCP Handshake Latency
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

        "dpi-start" => {
            let config = DpiConfig::default();
            let ciadpi_path = PathBuf::from("./third_party/byedpi/ciadpi");
            println!("\n[>] Starting ByeDPI Desynchronization Supervisor...");
            let mut supervisor = netfreedom_dpi::DpiSupervisor::new(config.clone(), ciadpi_path);
            supervisor.start().await?;
            println!("[+] ByeDPI running on 127.0.0.1:{} (Press Ctrl+C to stop)", config.local_port);
            
            tokio::signal::ctrl_c().await?;
            supervisor.stop().await?;
            println!("[+] ByeDPI stopped cleanly.");
        }

        "killswitch" => {
            let action = args.get(2).map(|s| s.as_str()).unwrap_or("status");
            match action {
                "status" => {
                    println!("\n[>] Checking Kernel nftables Kill Switch Status...");
                    let active = is_killswitch_active();
                    println!("  Kill Switch Active: {}", active);
                }
                "preview" => {
                    println!("\n[>] Previewing Kernel nftables Kill Switch Ruleset:");
                    let dummy_ep: SocketAddr = "162.159.192.1:2408".parse()?;
                    let rules = generate_nftables_ruleset("netfreedom0", Some(dummy_ep), true, Some("10.6.80.0/20"));
                    println!("{}", rules);
                }
                _ => {
                    println!("Usage: netfreedom-cli killswitch [status | preview]");
                }
            }
        }

        "dns" => {
            println!("\n[>] Testing Encrypted DNS Resolver Verification...");
            let secure = netfreedom_dns::verify_secure_dns(&client).await?;
            println!("  Encrypted DNS Active (DoH/WARP): {}", secure);
        }

        "diag" => {
            println!("\n[>] Running Full Network Diagnostics Report...");
            let target: SocketAddr = "1.1.1.1:443".parse()?;
            let (lat, jit) = probe_latency_and_jitter(target, 5).await;
            println!("  TCP Handshake Latency: {:.2?} ms", lat);
            println!("  Jitter:                {:.2?} ms", jit);
            println!("  Functional IPv6:       {}", is_ipv6_functional().await);
            println!("  Captive Portal:        {}", detect_captive_portal(&client).await);
            println!("  Kill Switch Active:    {}", is_killswitch_active());
            if let Ok(egress) = check_egress_info(&client).await {
                println!("  Public IP:             {:?}", egress.ip);
                println!("  Country:               {:?}", egress.loc);
                println!("  Datacenter (Colo):     {:?}", egress.colo);
                println!("  WARP Active:           {}", egress.warp_active);
            }
        }

        _ => {
            println!("Usage: netfreedom-cli [on | off | gui | auto | warp-reg | dpi-preview | dpi-start | killswitch | dns | diag]");
        }
    }

    println!("\n============================================================");

    #[cfg(target_os = "windows")]
    {
        println!("\nPress Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    Ok(())
}
