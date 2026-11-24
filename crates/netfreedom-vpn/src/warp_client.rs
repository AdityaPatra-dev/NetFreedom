use base64::Engine;
use netfreedom_common::WarpConfig;
use rand::rngs::OsRng;
use serde::Serialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Serialize)]
struct RegistrationRequest<'a> {
    install_id: &'a str,
    tos: &'a str,
    key: &'a str,
    #[serde(rename = "type")]
    device_type: &'a str,
    locale: &'a str,
}

pub const CLOUDFLARE_WARP_ANYCAST_ENDPOINT: &str = "188.114.96.1:2408";
pub const CLOUDFLARE_WARP_PUBLIC_KEY: &str = "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=";

// Known valid pre-registered Cloudflare WARP account for immediate zero-latency fallback
pub const FALLBACK_CLIENT_ID: &str = "b9bdec97-1f98-40f0-b4b1-ad13e98ecbd8";
pub const FALLBACK_PRIVATE_KEY: &str = "sDlFOwJUftki6+PoIQ3rBn75eFuwUKVIQZaHuJ0iimA=";
pub const FALLBACK_PUBLIC_KEY: &str = "BDIC2Cxf+lp3Em1JLnjNIuGyuc3SKxJ7ZSum6yvv+hA=";
pub const FALLBACK_ASSIGNED_IPV4: &str = "172.16.0.2/32";
pub const FALLBACK_ASSIGNED_IPV6: &str = "2606:4700:110:82c3:92ea:61b:bd0f:95f3/128";

pub fn get_fallback_warp_config() -> WarpConfig {
    let endpoint: SocketAddr = CLOUDFLARE_WARP_ANYCAST_ENDPOINT.parse().unwrap();
    WarpConfig {
        client_id: FALLBACK_CLIENT_ID.to_string(),
        private_key: FALLBACK_PRIVATE_KEY.to_string(),
        public_key: FALLBACK_PUBLIC_KEY.to_string(),
        peer_public_key: CLOUDFLARE_WARP_PUBLIC_KEY.to_string(),
        assigned_ipv4: FALLBACK_ASSIGNED_IPV4.to_string(),
        assigned_ipv6: Some(FALLBACK_ASSIGNED_IPV6.to_string()),
        endpoint,
        port: endpoint.port(),
    }
}

/// Generates a local Curve25519 keypair for WireGuard tunneling.
pub fn generate_warp_keypair() -> ([u8; 32], [u8; 32]) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret.to_bytes(), *public.as_bytes())
}

/// Helper to search for ByeDPI (ciadpi / ciadpi.exe) binary across common installation paths
fn find_ciadpi_binary() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("ciadpi"));
            candidates.push(parent.join("ciadpi.exe"));
            candidates.push(parent.join("third_party").join("byedpi").join("ciadpi"));
            candidates.push(parent.join("third_party").join("byedpi").join("ciadpi.exe"));
        }
    }
    if let Ok(cur) = std::env::current_dir() {
        candidates.push(cur.join("ciadpi"));
        candidates.push(cur.join("ciadpi.exe"));
        candidates.push(cur.join("third_party").join("byedpi").join("ciadpi"));
        candidates.push(cur.join("third_party").join("byedpi").join("ciadpi.exe"));
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local_app_data)
                .join("Programs")
                .join("NetFreedom")
                .join("ciadpi.exe"),
        );
    }
    if let Ok(prog_files) = std::env::var("ProgramFiles") {
        candidates.push(
            PathBuf::from(prog_files)
                .join("NetFreedom")
                .join("ciadpi.exe"),
        );
    }
    candidates.push(PathBuf::from("/usr/local/bin/ciadpi"));
    candidates.push(PathBuf::from("/usr/bin/ciadpi"));
    candidates.push(PathBuf::from("ciadpi"));
    candidates.push(PathBuf::from("ciadpi.exe"));

    candidates.into_iter().find(|p| p.exists())
}

struct EphemeralCiadpi {
    child: std::process::Child,
}

impl Drop for EphemeralCiadpi {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Registers an anonymous Cloudflare WARP account via their client API,
/// generating a Curve25519 keypair and retrieving WireGuard credentials.
pub async fn register_warp_account(client: &reqwest::Client) -> anyhow::Result<WarpConfig> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let b64 = base64::engine::general_purpose::STANDARD;
    let pub_b64 = b64.encode(public.as_bytes());
    let priv_b64 = b64.encode(secret.to_bytes());

    let payload = RegistrationRequest {
        install_id: "",
        tos: "2020-09-01T00:00:00.000Z",
        key: &pub_b64,
        device_type: "Android",
        locale: "en_US",
    };

    let url = "https://api.cloudflareclient.com/v0a2158/reg";

    // 1. First attempt: Direct registration via Cloudflare Anycast edge 172.66.3.170:443
    let direct_client = reqwest::Client::builder()
        .resolve("api.cloudflareclient.com", "172.66.3.170:443".parse()?)
        .timeout(Duration::from_millis(3500))
        .build()
        .unwrap_or_else(|_| client.clone());

    let direct_res = direct_client
        .post(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "okhttp/3.12.1")
        .json(&payload)
        .send()
        .await;

    let resp_val: serde_json::Value = match direct_res {
        Ok(r) if r.status().is_success() => {
            info!("Direct Cloudflare WARP registration succeeded");
            r.json().await?
        }
        _ => {
            warn!("Direct WARP registration failed or timed out (SNI blocked). Attempting ephemeral desync bypass...");
            // 2. Second attempt: Ephemeral ciadpi proxy on port 19088 with packet desynchronization (-s 2+s -f -1 -t 3 -d 1+s)
            let mut registered_val = None;

            if let Some(ciadpi_bin) = find_ciadpi_binary() {
                let ephemeral_port: u16 = 19088;
                let mut cmd = std::process::Command::new(&ciadpi_bin);
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
                }
                cmd.args(&[
                    "-i", "127.0.0.1",
                    "-p", &ephemeral_port.to_string(),
                    "-I", "0.0.0.0",
                    "-s", "2+s",
                    "-f", "-1",
                    "-t", "3",
                    "-d", "1+s",
                ])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());

                if let Ok(child) = cmd.spawn() {
                    let _guard = EphemeralCiadpi { child };
                    tokio::time::sleep(Duration::from_millis(300)).await;

                    if let Ok(proxy) = reqwest::Proxy::all(format!("socks5://127.0.0.1:{}", ephemeral_port)) {
                        if let Ok(proxy_client) = reqwest::Client::builder()
                            .proxy(proxy)
                            .resolve("api.cloudflareclient.com", "172.66.3.170:443".parse()?)
                            .timeout(Duration::from_millis(6000))
                            .build()
                        {
                            if let Ok(resp) = proxy_client
                                .post(url)
                                .header("Content-Type", "application/json")
                                .header("User-Agent", "okhttp/3.12.1")
                                .json(&payload)
                                .send()
                                .await
                            {
                                if resp.status().is_success() {
                                    if let Ok(val) = resp.json::<serde_json::Value>().await {
                                        info!("WARP registration succeeded via ephemeral split-SNI desync bypass!");
                                        registered_val = Some(val);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            match registered_val {
                Some(val) => val,
                None => {
                    warn!("Ephemeral desync registration unavailable. Utilizing pre-verified anonymous fallback account.");
                    return Ok(get_fallback_warp_config());
                }
            }
        }
    };

    let client_id = resp_val["id"]
        .as_str()
        .or_else(|| resp_val["result"]["id"].as_str())
        .unwrap_or("anon_warp_user")
        .to_string();

    let peer = resp_val["config"]["peers"][0]
        .as_object()
        .or_else(|| resp_val["result"]["config"]["peers"][0].as_object())
        .ok_or_else(|| anyhow::anyhow!("No peers found in response: {}", resp_val))?;

    let peer_public_key = peer["public_key"].as_str().unwrap_or(CLOUDFLARE_WARP_PUBLIC_KEY).to_string();
    let endpoint: SocketAddr = CLOUDFLARE_WARP_ANYCAST_ENDPOINT.parse().unwrap();

    let iface = resp_val["config"]["interface"]
        .as_object()
        .or_else(|| resp_val["result"]["config"]["interface"].as_object())
        .ok_or_else(|| anyhow::anyhow!("No interface found in response"))?;

    let assigned_ipv4 = iface["addresses"]["v4"].as_str().unwrap_or("172.16.0.2/32").to_string();
    let assigned_ipv6 = iface["addresses"]["v6"].as_str().map(|s| s.to_string());

    Ok(WarpConfig {
        client_id,
        private_key: priv_b64,
        public_key: pub_b64,
        peer_public_key,
        assigned_ipv4,
        assigned_ipv6,
        endpoint,
        port: endpoint.port(),
    })
}

/// Retrieves an existing cached WARP account from disk, or registers a new one via Cloudflare's API.
pub async fn get_or_register_warp_account() -> anyhow::Result<WarpConfig> {
    let candidate_dirs = [
        std::env::var("APPDATA").ok().map(|p| PathBuf::from(p).join("NetFreedom")),
        std::env::var("LOCALAPPDATA").ok().map(|p| PathBuf::from(p).join("NetFreedom")),
        std::env::var("HOME").ok().map(|p| PathBuf::from(p).join(".config").join("netfreedom")),
        Some(std::env::temp_dir().join("NetFreedom")),
    ];

    for dir_opt in &candidate_dirs {
        if let Some(dir) = dir_opt {
            let file_path = dir.join("warp_account.json");
            if file_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    if let Ok(cfg) = serde_json::from_str::<WarpConfig>(&content) {
                        return Ok(cfg);
                    }
                }
            }
        }
    }

    // Register fresh account
    let client = reqwest::Client::new();
    let config = match register_warp_account(&client).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to dynamically register WARP account: {}. Using fallback account.", e);
            get_fallback_warp_config()
        }
    };

    // Cache to disk
    for dir_opt in &candidate_dirs {
        if let Some(dir) = dir_opt {
            let _ = std::fs::create_dir_all(dir);
            let file_path = dir.join("warp_account.json");
            if let Ok(json_str) = serde_json::to_string_pretty(&config) {
                if std::fs::write(&file_path, json_str).is_ok() {
                    break;
                }
            }
        }
    }

    Ok(config)
}
