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

    let peer_public_key = peer["public_key"].as_str().unwrap_or("").to_string();
    let endpoint_str = peer["endpoint"]["v4"].as_str().unwrap_or("162.159.192.1:2408");
    let endpoint: SocketAddr = endpoint_str.parse().unwrap_or_else(|_| "162.159.192.1:2408".parse().unwrap());

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
