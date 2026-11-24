export type OperatingMode =
  | 'stealth_tunnel'
  | 'warp_shield'
  | 'dpi_desync'
  | 'custom_vpn'
  | 'modular_proxy'
  | 'hybrid_stealth'
  | 'auto_pilot';

export type ConnectionStatus =
  | 'disconnected'
  | 'probing_network'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'failed';

export interface HealthMetrics {
  tcp_latency_ms: number | null;
  jitter_ms: number | null;
  download_bytes_sec: number;
  upload_bytes_sec: number;
  total_rx_bytes: number;
  total_tx_bytes: number;
  public_ip: string | null;
  country: string | null;
  colo: string | null;
  captive_portal_detected: boolean;
  ipv6_working: boolean;
}

export interface DpiSettings {
  split_pos: string;
  fake_ttl: number;
  disorder: boolean;
  fake_rst: boolean;
  local_port: number;
}
