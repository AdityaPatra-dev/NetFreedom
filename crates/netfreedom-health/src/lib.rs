pub mod captive;
pub mod egress;
pub mod ipv6;
pub mod tcp_ping;
pub mod autopilot;

pub use captive::detect_captive_portal;
pub use egress::{check_egress_info, EgressInfo};
pub use ipv6::is_ipv6_functional;
pub use tcp_ping::{measure_tcp_handshake_latency, probe_latency_and_jitter};
pub use autopilot::{evaluate_autopilot_mode, probe_udp_reachability, NetworkProbeReport};
