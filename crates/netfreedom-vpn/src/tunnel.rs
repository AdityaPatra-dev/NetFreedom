use anyhow::Result;
use boringtun::noise::{Tunn, TunnResult};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

pub struct WireGuardTunnel {
    is_running: Arc<AtomicBool>,
}

impl WireGuardTunnel {
    /// Launches a non-blocking BoringTun WireGuard async loop.
    /// Packets received from `tun_rx` are encrypted and sent via UDP to `endpoint`.
    /// Packets received from UDP are decrypted and sent to `tun_tx`.
    pub async fn start(
        private_key: [u8; 32],
        peer_public_key: [u8; 32],
        preshared_key: Option<[u8; 32]>,
        endpoint: SocketAddr,
        mut tun_rx: mpsc::Receiver<Vec<u8>>,
        tun_tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<Self> {
        info!("Starting BoringTun WireGuard async engine to endpoint: {}", endpoint);

        let tunn = Tunn::new(
            private_key.into(),
            peer_public_key.into(),
            preshared_key,
            Some(25), // persistent keepalive every 25 seconds
            0,        // session index
            None,     // rate limiter
        );

        let tunn = Arc::new(Mutex::new(tunn));
        let udp = UdpSocket::bind("0.0.0.0:0").await?;
        udp.connect(endpoint).await?;
        let udp = Arc::new(udp);

        let is_running = Arc::new(AtomicBool::new(true));

        // 1. Task: Outbound (TUN -> WireGuard Encapsulate -> UDP)
        let is_running_out = is_running.clone();
        let tunn_out = tunn.clone();
        let udp_out = udp.clone();
        tokio::spawn(async move {
            let mut send_buf = vec![0u8; 2048];
            while is_running_out.load(Ordering::Relaxed) {
                match tokio::time::timeout(Duration::from_millis(500), tun_rx.recv()).await {
                    Ok(Some(raw_packet)) => {
                        let mut t = tunn_out.lock().await;
                        match t.encapsulate(&raw_packet, &mut send_buf) {
                            TunnResult::WriteToNetwork(bytes) => {
                                let _ = udp_out.send(bytes).await;
                            }
                            TunnResult::Done => {}
                            TunnResult::Err(e) => {
                                debug!("BoringTun encapsulate warning: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => break, // Channel closed
                    Err(_) => {}       // Timeout, loop to check is_running
                }
            }
            debug!("BoringTun outbound worker stopped.");
        });

        // 2. Task: Inbound (UDP -> WireGuard Decapsulate -> TUN)
        let is_running_in = is_running.clone();
        let tunn_in = tunn.clone();
        let udp_in = udp.clone();
        let tun_tx_in = tun_tx.clone();
        tokio::spawn(async move {
            let mut recv_buf = vec![0u8; 2048];
            let mut decaps_buf = vec![0u8; 2048];
            while is_running_in.load(Ordering::Relaxed) {
                match tokio::time::timeout(Duration::from_millis(500), udp_in.recv(&mut recv_buf)).await {
                    Ok(Ok(n)) if n > 0 => {
                        let mut t = tunn_in.lock().await;
                        match t.decapsulate(None, &recv_buf[..n], &mut decaps_buf) {
                            TunnResult::WriteToTunnelV4(packet, _) | TunnResult::WriteToTunnelV6(packet, _) => {
                                let _ = tun_tx_in.send(packet.to_vec()).await;
                            }
                            TunnResult::WriteToNetwork(packet) => {
                                // Handshake response or keepalive
                                let _ = udp_in.send(packet).await;
                            }
                            TunnResult::Done => {}
                            TunnResult::Err(e) => {
                                debug!("BoringTun decapsulate error: {:?}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        warn!("UDP receive error in WireGuard tunnel: {}", e);
                    }
                    Err(_) => {} // Timeout, loop to check is_running
                    _ => {}
                }
            }
            debug!("BoringTun inbound worker stopped.");
        });

        // 3. Task: Handshake and Keepalive Timers
        let is_running_timer = is_running.clone();
        let tunn_timer = tunn.clone();
        let udp_timer = udp.clone();
        tokio::spawn(async move {
            let mut timer_buf = vec![0u8; 2048];
            while is_running_timer.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(250)).await;
                let mut t = tunn_timer.lock().await;
                match t.update_timers(&mut timer_buf) {
                    TunnResult::WriteToNetwork(packet) => {
                        let _ = udp_timer.send(packet).await;
                    }
                    TunnResult::Done => {}
                    TunnResult::Err(e) => {
                        debug!("Timer update note: {:?}", e);
                    }
                    _ => {}
                }
            }
            debug!("BoringTun timer worker stopped.");
        });

        // Trigger initial handshake immediately
        {
            let mut t = tunn.lock().await;
            let mut initial_buf = vec![0u8; 2048];
            match t.format_handshake_initiation(&mut initial_buf, false) {
                TunnResult::WriteToNetwork(handshake) => {
                    info!("Sending initial WireGuard handshake to {}", endpoint);
                    let _ = udp.send(handshake).await;
                }
                _ => {}
            }
        }

        info!("WireGuard tunnel active and established.");
        Ok(Self { is_running })
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

impl Drop for WireGuardTunnel {
    fn drop(&mut self) {
        self.stop();
    }
}
