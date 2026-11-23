/// Generates the YAML configuration for hev-socks5-tunnel.
pub fn generate_hev_config(
    tun_name: &str,
    socks5_port: u16,
    mtu: u16,
    ipv4_address: &str,
) -> String {
    format!(
        r#"tunnel:
  name: {tun_name}
  mtu: {mtu}
  ipv4: {ipv4_address}
  icmp: 'reply'

socks5:
  port: {socks5_port}
  address: 127.0.0.1
  udp: 'udp'
"#
    )
}
