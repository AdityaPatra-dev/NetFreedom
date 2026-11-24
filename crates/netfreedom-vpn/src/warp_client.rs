use base64::Engine;
use netfreedom_common::WarpConfig;
use rand::rngs::OsRng;
use serde::Serialize;
use std::net::SocketAddr;
use std::time::Duration;
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

/// Generates a local Curve25519 keypair for WireGuard tunneling.
pub fn generate_warp_keypair() -> ([u8; 32], [u8; 32]) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret.to_bytes(), *public.as_bytes())
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

    // Note: On campus / enterprise networks where 162.159.0.0/16 is blackholed,
    // we use a dedicated client that resolves to Cloudflare Anycast edge 172.66.3.170:443.
    let resolved_client = reqwest::Client::builder()
        .resolve("api.cloudflareclient.com", "172.66.3.170:443".parse()?)
        .timeout(Duration::from_millis(6000))
        .build()
        .unwrap_or_else(|_| client.clone());

    // Try registering directly first; if middlebox blocks SNI, fallback via local ByeDPI proxy
    let url = "https://api.cloudflareclient.com/v0a2158/reg";

    let resp_res = resolved_client
        .post(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "okhttp/3.12.1")
        .json(&payload)
        .send()
        .await;

    let resp_val: serde_json::Value = match resp_res {
        Ok(r) if r.status().is_success() => r.json().await?,
        _ => {
            // Fallback: route registration through local ByeDPI proxy if available on 1080
            let proxy_client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all("socks5://127.0.0.1:1080").unwrap())
                .resolve("api.cloudflareclient.com", "172.66.3.170:443".parse()?)
                .timeout(Duration::from_millis(6000))
                .build()?;

            proxy_client
                .post(url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "okhttp/3.12.1")
                .json(&payload)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?
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
    // Default to the campus-accessible Anycast edge 188.114.96.1:2408 to defeat 162.159.0.0/16 blackholes
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
        std::env::var("APPDATA").ok().map(|p| std::path::PathBuf::from(p).join("NetFreedom")),
        std::env::var("LOCALAPPDATA").ok().map(|p| std::path::PathBuf::from(p).join("NetFreedom")),
        std::env::var("HOME").ok().map(|p| std::path::PathBuf::from(p).join(".config").join("netfreedom")),
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
    let config = register_warp_account(&client).await?;

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

