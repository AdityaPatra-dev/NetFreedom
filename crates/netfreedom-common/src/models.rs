use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Primary operating mode of NetFreedom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    /// 1-Click zero-config WireGuard tunnel to Cloudflare Anycast edge
    WarpShield,
    /// Serverless DPI bypass using TCP/TLS manipulation (ByeDPI / Zapret)
    DpiDesync,
    /// Custom WireGuard profile (.conf, QR code, or manual keys)
    CustomVpn,
    /// Outbound SOCKS5 / Shadowsocks / HTTP CONNECT proxy
    ModularProxy,
    /// Hybrid chaining: ByeDPI Desync -> WireGuard/SOCKS5
    HybridStealth,
    /// Auto-Pilot: Automatically detect network and pick the optimal working mode
    AutoPilot,
}

/// Real-time connection state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Disconnected,
    ProbingNetwork,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

/// DPI Desynchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpiConfig {
    /// Split position string, e.g. "1+s" (split at SNI)
    pub split_pos: String,
    /// Low TTL for fake packets, e.g. 3
    pub fake_ttl: u8,
    /// Enable TCP segment reverse ordering
    pub disorder: bool,
    /// Enable fake TCP RST packets
    pub fake_rst: bool,
    /// Target listening port for local SOCKS5 proxy (default 1080)
    pub local_port: u16,
}

impl Default for DpiConfig {
    fn default() -> Self {
        Self {
            split_pos: "1+s".to_string(),
            fake_ttl: 3,
            disorder: true,
            fake_rst: false,
            local_port: 1080,
        }
    }
}

/// Cloudflare WARP identity & connection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpConfig {
    pub client_id: String,
    pub private_key: String,
    pub public_key: String,
    pub peer_public_key: String,
    pub assigned_ipv4: String,
    pub assigned_ipv6: Option<String>,
    pub endpoint: SocketAddr,
    pub port: u16,
}

/// Custom WireGuard profile parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub private_key: String,
    pub address: String,
    pub peer_public_key: String,
    pub preshared_key: Option<String>,
    pub endpoint: String,
    pub dns: Vec<String>,
    pub mtu: u16,
}

/// SOCKS5 / Shadowsocks / HTTP proxy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub server: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_socks5: bool,
}

/// Complete connection profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub mode: OperatingMode,
    pub dpi_config: Option<DpiConfig>,
    pub warp_config: Option<WarpConfig>,
    pub wireguard_config: Option<WireGuardConfig>,
    pub proxy_config: Option<ProxyConfig>,
    pub kill_switch_enabled: bool,
    pub allow_lan: bool,
    pub custom_dns: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Live real-time health and throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthMetrics {
    /// Latency in milliseconds measured via TCP SYN/ACK handshake (non-ICMP)
    pub tcp_latency_ms: Option<f64>,
    /// Jitter in milliseconds
    pub jitter_ms: Option<f64>,
    /// Current download speed in Bytes/sec
    pub download_bytes_sec: u64,
    /// Current upload speed in Bytes/sec
    pub upload_bytes_sec: u64,
    /// Total bytes downloaded in session
    pub total_rx_bytes: u64,
    /// Total bytes uploaded in session
    pub total_tx_bytes: u64,
    /// Public IP detected
    pub public_ip: Option<String>,
    /// Detected country code (e.g. "IN")
    pub country: Option<String>,
    /// Cloudflare colo code (e.g. "MAA")
    pub colo: Option<String>,
    /// Captive portal detected on current Wi-Fi
    pub captive_portal_detected: bool,
    /// IPv6 status on current network
    pub ipv6_working: bool,
}
