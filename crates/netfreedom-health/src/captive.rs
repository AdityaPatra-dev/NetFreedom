use std::time::Duration;

/// Probes whether the current connection is trapped behind a captive portal.
pub async fn detect_captive_portal(client: &reqwest::Client) -> bool {
    let url = "http://connectivitycheck.gstatic.com/generate_204";
    match client
        .get(url)
        .timeout(Duration::from_millis(2500))
        .send()
        .await
    {
        Ok(resp) => {
            // A 204 No Content indicates an open internet connection.
            // A 200 OK or 302 Redirect indicates a captive login screen.
            resp.status().as_u16() != 204
        }
        Err(_) => false,
    }
}
