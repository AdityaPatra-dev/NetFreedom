use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Measures round-trip time (RTT) to a remote host using a non-ICMP TCP handshake.
/// This works reliably on campus and enterprise networks that block ICMP ping.
pub async fn measure_tcp_handshake_latency(
    target: SocketAddr,
    timeout_duration: Duration,
) -> anyhow::Result<f64> {
    let start = Instant::now();
    let connect_future = TcpStream::connect(target);
    
    match timeout(timeout_duration, connect_future).await {
        Ok(Ok(_stream)) => {
            let elapsed = start.elapsed();
            let latency_ms = elapsed.as_secs_f64() * 1000.0;
            Ok(latency_ms)
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("TCP handshake failed: {}", e)),
        Err(_) => Err(anyhow::anyhow!("TCP handshake timed out after {:?}", timeout_duration)),
    }
}

/// Measures average TCP latency and jitter over a series of sample handshakes.
pub async fn probe_latency_and_jitter(
    target: SocketAddr,
    samples: usize,
) -> (Option<f64>, Option<f64>) {
    let mut measurements = Vec::with_capacity(samples);
    
    for _ in 0..samples {
        if let Ok(latency) = measure_tcp_handshake_latency(target, Duration::from_millis(1500)).await {
            measurements.push(latency);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    if measurements.is_empty() {
        return (None, None);
    }

    let avg = measurements.iter().sum::<f64>() / measurements.len() as f64;
    
    let variance = measurements
        .iter()
        .map(|val| (val - avg).powi(2))
        .sum::<f64>()
        / measurements.len() as f64;
    let jitter = variance.sqrt();

    (Some(avg), Some(jitter))
}
