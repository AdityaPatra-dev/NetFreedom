use anyhow::Result;
use std::time::Duration;

/// Probes whether DNS is currently resolving via Cloudflare Encrypted DNS (1.1.1.1).
/// Queries 1.1.1.1/help API which returns a JSON payload showing whether DoH is active.
pub async fn verify_secure_dns(client: &reqwest::Client) -> Result<bool> {
    let url = "https://1.1.1.1/help";
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .timeout(Duration::from_millis(3500))
        .send()
        .await?;

    if resp.status().is_success() {
        let text = resp.text().await?;
        // If Cloudflare returns that DoH / WARP DNS is active
        Ok(text.contains("\"isDoH\":true") || text.contains("\"isWarp\":true") || text.contains("1.1.1.1"))
    } else {
        Ok(false)
    }
}
