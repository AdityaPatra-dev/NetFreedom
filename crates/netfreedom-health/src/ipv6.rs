use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Tests if IPv6 is actually reachable on the current network.
/// Many campus networks advertise IPv6 DNS without providing an upstream route,
/// causing 3-5s browser stalls.
pub async fn is_ipv6_functional() -> bool {
    // Cloudflare Public DNS IPv6: 2606:4700:4700::1111:443
    let target = SocketAddr::new(
        IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)),
        443,
    );

    match timeout(Duration::from_millis(1500), TcpStream::connect(target)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}
