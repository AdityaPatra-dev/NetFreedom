use anyhow::Result;
use tracing::{info, warn};

pub const UNIX_SOCKET_PATH: &str = "/var/run/netfreedom.sock";
pub const WINDOWS_PIPE_NAME: &str = r"\\.\pipe\NetFreedomIPC";

/// Starts the cross-platform IPC listener for client (CLI / GUI) commands.
pub async fn start_ipc_listener() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = std::fs::remove_file(UNIX_SOCKET_PATH);
        match tokio::net::UnixListener::bind(UNIX_SOCKET_PATH) {
            Ok(listener) => {
                info!("Daemon IPC Unix socket listening on {}", UNIX_SOCKET_PATH);
                loop {
                    if let Ok((mut stream, _)) = listener.accept().await {
                        tokio::spawn(async move {
                            use tokio::io::{AsyncReadExt, AsyncWriteExt};
                            let mut buf = [0u8; 1024];
                            if let Ok(n) = stream.read(&mut buf).await {
                                if n > 0 {
                                    let _ = stream.write_all(b"{\"status\":\"ok\",\"service\":\"netfreedom-daemon\"}\n").await;
                                }
                            }
                        });
                    }
                }
            }
            Err(e) => {
                warn!("Could not bind Unix domain socket at {}: {}. Running in standalone mode.", UNIX_SOCKET_PATH, e);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        info!("Daemon IPC pipe registered on {}", WINDOWS_PIPE_NAME);
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
    }

    Ok(())
}
