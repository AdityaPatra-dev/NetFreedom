use anyhow::Result;
use netfreedom_common::WireGuardConfig;

/// Parses a standard WireGuard (.conf) file into WireGuardConfig.
pub fn parse_wireguard_conf(content: &str) -> Result<WireGuardConfig> {
    let mut private_key = String::new();
    let mut address = String::new();
    let mut peer_public_key = String::new();
    let mut preshared_key = None;
    let mut endpoint = String::new();
    let mut dns = Vec::new();
    let mut mtu = 1420;

    let mut in_interface = false;
    let mut in_peer = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("[Interface]") {
            in_interface = true;
            in_peer = false;
            continue;
        } else if trimmed.eq_ignore_ascii_case("[Peer]") {
            in_interface = false;
            in_peer = true;
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim().to_lowercase();
            let val = val.trim();

            if in_interface {
                match key.as_str() {
                    "privatekey" => private_key = val.to_string(),
                    "address" => address = val.to_string(),
                    "dns" => dns = val.split(',').map(|s| s.trim().to_string()).collect(),
                    "mtu" => mtu = val.parse().unwrap_or(1420),
                    _ => {}
                }
            } else if in_peer {
                match key.as_str() {
                    "publickey" => peer_public_key = val.to_string(),
                    "presharedkey" => preshared_key = Some(val.to_string()),
                    "endpoint" => endpoint = val.to_string(),
                    _ => {}
                }
            }
        }
    }

    if private_key.is_empty() || peer_public_key.is_empty() || endpoint.is_empty() {
        return Err(anyhow::anyhow!("Invalid WireGuard configuration: missing required keys"));
    }

    Ok(WireGuardConfig {
        private_key,
        address,
        peer_public_key,
        preshared_key,
        endpoint,
        dns,
        mtu,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wireguard_conf() {
        let conf = r#"
[Interface]
PrivateKey = aaaaaa=
Address = 10.0.0.2/32
DNS = 1.1.1.1, 1.0.0.1
MTU = 1420

[Peer]
PublicKey = bbbbbb=
Endpoint = 162.159.192.1:2408
"#;
        let parsed = parse_wireguard_conf(conf).expect("parse failed");
        assert_eq!(parsed.private_key, "aaaaaa=");
        assert_eq!(parsed.peer_public_key, "bbbbbb=");
        assert_eq!(parsed.endpoint, "162.159.192.1:2408");
        assert_eq!(parsed.mtu, 1420);
        assert_eq!(parsed.dns.len(), 2);
    }
}
