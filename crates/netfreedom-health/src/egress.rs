use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EgressInfo {
    pub ip: Option<String>,
    pub loc: Option<String>,
    pub colo: Option<String>,
    pub warp_active: bool,
}

/// Queries Cloudflare trace to detect the active public IP, country, and WARP status.
pub async fn check_egress_info(client: &reqwest::Client) -> anyhow::Result<EgressInfo> {
    let resp = client
        .get("https://1.1.1.1/cdn-cgi/trace")
        .timeout(Duration::from_millis(3500))
        .send()
        .await?
        .text()
        .await?;

    let mut info = EgressInfo::default();

    for line in resp.lines() {
        if let Some((k, v)) = line.split_once('=') {
            match k {
                "ip" => info.ip = Some(v.to_string()),
                "loc" => info.loc = Some(v.to_string()),
                "colo" => info.colo = Some(v.to_string()),
                "warp" => info.warp_active = v == "on" || v == "plus",
                _ => {}
            }
        }
    }

    Ok(info)
}
