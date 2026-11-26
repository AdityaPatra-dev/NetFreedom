use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use netfreedom_common::OperatingMode;
use crate::tcp_ping::measure_tcp_handshake_latency;

/// Comprehensive probe report produced by the Adaptive Auto-Pilot analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkProbeReport {
    pub captive_portal: bool,
    pub tcp_handshake_ms: Option<f64>,
    pub udp_wireguard_open: bool,
    pub direct_internet_open: bool,
    pub sni_inspection_detected: bool,
    pub recommended_mode: OperatingMode,
    pub reason: String,
}

/// Probes UDP reachability by sending a DNS query packet to 1.1.1.1:53 or WireGuard handshake probe.
/// If UDP packets are silently dropped by campus / enterprise perimeter firewalls,
/// WireGuard (WARP Shield) cannot function and traffic will blackhole.
pub async fn probe_udp_reachability() -> bool {
    let target: SocketAddr = match "1.1.1.1:53".parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    let local_addr: SocketAddr = match "0.0.0.0:0".parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    let Ok(socket) = UdpSocket::bind(local_addr).await else {
        return false;
    };

    // Standard DNS query for cloudflare.com (A record)
    // ID: 0x4e46 ('NF'), Flags: 0x0100 (standard query, recursion desired), 1 question
    let query: [u8; 32] = [
        0x4e, 0x46, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        10, b'c', b'l', b'o', b'u', b'd', b'f', b'l', b'a', b'r', b'e',
        3, b'c', b'o', b'm', 0x00,
        0x00, 0x01, // Type A
        0x00, 0x01, // Class IN
    ];

    if socket.send_to(&query, target).await.is_err() {
        return false;
    }

    let mut buf = [0u8; 512];
    let rx_future = socket.recv_from(&mut buf);
    match timeout(Duration::from_millis(1500), rx_future).await {
        Ok(Ok((bytes_read, _from))) => bytes_read > 12,
        _ => false,
    }
}

/// Checks whether direct HTTP / SNI traffic is intercepted or blocked by deep packet inspection.
pub async fn probe_sni_dpi_filtering() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(2000))
        .build();

    match client {
        Ok(c) => {
            let res = c.get("http://connectivitycheck.gstatic.com/generate_204").send().await;
            match res {
                Ok(r) => r.status().as_u16() != 204,
                Err(_) => true,
            }
        }
        Err(_) => true,
    }
}

/// Executes the empirical 4-stage Auto-Pilot ladder:
/// 1. Check captive portal trapping.
/// 2. Measure direct TCP handshake latency.
/// 3. Test UDP Anycast reachability (WireGuard readiness).
/// 4. If UDP is blocked or SNI is censored, determine whether ByeDPI desync or Stealth Tunnel (TCP 443) is needed.
pub async fn evaluate_autopilot_mode() -> NetworkProbeReport {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(2000))
        .build()
        .unwrap_or_default();

    // Stage 1: Captive Portal Check
    let is_captive = crate::captive::detect_captive_portal(&client).await;

    // Stage 2: Direct TCP Handshake
    let target: SocketAddr = "1.1.1.1:443".parse().unwrap();
    let tcp_latency = measure_tcp_handshake_latency(target, Duration::from_millis(1500)).await.ok();

    // Stage 3: UDP Reachability
    let udp_open = probe_udp_reachability().await;

    // Stage 4: SNI Inspection Check
    let sni_blocked = probe_sni_dpi_filtering().await;

    let (recommended_mode, reason) = if is_captive {
        (
            OperatingMode::StealthTunnel,
            "Captive portal / strict perimeter detected. Using Stealth Tunnel (TCP 443) to encapsulate traffic.".to_string(),
        )
    } else if !udp_open {
        (
            OperatingMode::StealthTunnel,
            "UDP traffic is dropped by network firewall (WireGuard blocked). Using Stealth Tunnel (VLESS over TCP 443).".to_string(),
        )
    } else if sni_blocked {
        (
            OperatingMode::DpiDesync,
            "Deep Packet Inspection detected on SNI. Activating ByeDPI desynchronization to defeat middlebox filtering.".to_string(),
        )
    } else {
        (
            OperatingMode::WarpShield,
            "Direct UDP and TCP are healthy. Engaging Cloudflare WARP Privacy Shield for maximum throughput.".to_string(),
        )
    };

    NetworkProbeReport {
        captive_portal: is_captive,
        tcp_handshake_ms: tcp_latency,
        udp_wireguard_open: udp_open,
        direct_internet_open: tcp_latency.is_some() && !is_captive,
        sni_inspection_detected: sni_blocked,
        recommended_mode,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_probe_report_serialization() {
        let report = NetworkProbeReport {
            captive_portal: false,
            tcp_handshake_ms: Some(25.4),
            udp_wireguard_open: true,
            direct_internet_open: true,
            sni_inspection_detected: false,
            recommended_mode: OperatingMode::WarpShield,
            reason: "All clear".to_string(),
        };
        let serialized = serde_json::to_string(&report).expect("Serialize probe report");
        assert!(serialized.contains("warp_shield"));
        assert!(serialized.contains("25.4"));
    }
}
