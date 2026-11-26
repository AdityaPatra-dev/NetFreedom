use netfreedom_common::{StealthNodeConfig, WarpConfig};
use serde_json::json;

/// Converts a user-configured StealthNodeConfig into a Sing-Box VLESS outbound object.
pub fn node_to_vless_outbound(node: &StealthNodeConfig) -> serde_json::Value {
    json!({
        "type": "vless",
        "tag": "stealth-out",
        "server": node.server,
        "server_port": node.server_port,
        "uuid": node.uuid,
        "tls": {
            "enabled": true,
            "server_name": node.host,
            "utls": {
                "enabled": true,
                "fingerprint": "chrome"
            }
        },
        "transport": {
            "type": "ws",
            "path": node.path,
            "headers": {
                "Host": node.host
            }
        }
    })
}

/// Generates a valid Sing-Box JSON configuration file for Cloudflare WARP.
/// Supports both full-system TUN mode (via Wintun on Windows or tun on Linux)
/// and local Mixed (SOCKS5/HTTP) proxy inbound on 127.0.0.1.
pub fn generate_singbox_warp_config(warp: &WarpConfig, enable_tun: bool, socks_port: u16) -> String {
    let mut inbounds = Vec::new();

    if enable_tun {
        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": "NetFreedom",
            "inet4_address": "172.19.0.1/30",
            "auto_route": true,
            "strict_route": true,
            "stack": "system",
            "sniff": true
        }));
    }

    inbounds.push(json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": "127.0.0.1",
        "listen_port": socks_port
    }));

    let mut local_addresses = vec![warp.assigned_ipv4.clone()];
    if let Some(ref v6) = warp.assigned_ipv6 {
        local_addresses.push(v6.clone());
    }

    let config = json!({
        "log": {
            "level": "info",
            "timestamp": true
        },
        "dns": {
            "servers": [
                {
                    "tag": "cloudflare-dns",
                    "address": "1.1.1.1",
                    "detour": "warp-out"
                }
            ]
        },
        "inbounds": inbounds,
        "outbounds": [
            {
                "type": "wireguard",
                "tag": "warp-out",
                "server": warp.endpoint.ip().to_string(),
                "server_port": warp.endpoint.port(),
                "local_address": local_addresses,
                "private_key": warp.private_key,
                "peer_public_key": warp.peer_public_key,
                "reserved": [0, 0, 0],
                "mtu": 1280
            },
            {
                "type": "direct",
                "tag": "direct-out"
            },
            {
                "type": "dns",
                "tag": "dns-out"
            }
        ],
        "route": {
            "auto_detect_interface": true,
            "final": "warp-out",
            "rules": [
                {
                    "protocol": "dns",
                    "outbound": "dns-out"
                },
                {
                    "ip_is_private": true,
                    "outbound": "direct-out"
                }
            ]
        }
    });

    serde_json::to_string_pretty(&config).unwrap_or_default()
}

/// Generates a valid Sing-Box JSON configuration file for an Encrypted Stealth Tunnel over TCP Port 443.
/// This completely bypasses campus firewalls (such as Fortinet on KIIT) that drop outbound UDP,
/// by encapsulating all traffic inside TLS 1.3 on TCP 443.
pub fn generate_singbox_stealth_config(
    custom_node_opt: Option<serde_json::Value>,
    enable_tun: bool,
    socks_port: u16,
) -> String {
    let mut inbounds = Vec::new();

    if enable_tun {
        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": "NetFreedom",
            "inet4_address": "172.19.0.1/30",
            "auto_route": true,
            "strict_route": true,
            "stack": "system",
            "sniff": true
        }));
    }

    inbounds.push(json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": "127.0.0.1",
        "listen_port": socks_port
    }));

    // Outbound using user-specified stealth node or standard default
    let stealth_outbound = custom_node_opt.unwrap_or_else(|| {
        node_to_vless_outbound(&StealthNodeConfig::default())
    });

    let config = json!({
        "log": {
            "level": "info",
            "timestamp": true
        },
        "dns": {
            "servers": [
                {
                    "tag": "remote-dns",
                    "address": "tcp://1.1.1.1:53",
                    "detour": "stealth-out"
                }
            ]
        },
        "inbounds": inbounds,
        "outbounds": [
            stealth_outbound,
            {
                "type": "direct",
                "tag": "direct-out"
            },
            {
                "type": "dns",
                "tag": "dns-out"
            }
        ],
        "route": {
            "auto_detect_interface": true,
            "final": "stealth-out",
            "rules": [
                {
                    "protocol": "dns",
                    "outbound": "dns-out"
                },
                {
                    "ip_is_private": true,
                    "outbound": "direct-out"
                }
            ]
        }
    });

    serde_json::to_string_pretty(&config).unwrap_or_default()
}

