use crate::models::{ConnectionProfile, ConnectionStatus, HealthMetrics, OperatingMode};
use serde::{Deserialize, Serialize};

/// IPC Requests sent from GUI/CLI client to Daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcRequest {
    Connect { profile: ConnectionProfile },
    Disconnect,
    GetStatus,
    GetMetrics,
    SetKillSwitch { enabled: bool, allow_lan: bool },
    RunDiagnostics,
    AutoPilotConnect,
}

/// IPC Responses returned from Daemon to GUI/CLI client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcResponse {
    Status {
        status: ConnectionStatus,
        active_mode: Option<OperatingMode>,
        profile_name: Option<String>,
    },
    Metrics(HealthMetrics),
    DiagnosticsResult {
        tcp_latency_ms: Option<f64>,
        ipv6_working: bool,
        warp_reachable: bool,
        dns_leak_detected: bool,
        public_ip: Option<String>,
    },
    Success,
    Error(String),
}
