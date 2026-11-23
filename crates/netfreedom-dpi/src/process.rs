use anyhow::Result;
use netfreedom_common::DpiConfig;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{info, warn};

/// Manages the lifecycle of the underlying ByeDPI (ciadpi) desynchronization process.
pub struct DpiSupervisor {
    child: Option<Child>,
    config: DpiConfig,
    binary_path: PathBuf,
}

impl DpiSupervisor {
    pub fn new(config: DpiConfig, binary_path: PathBuf) -> Self {
        Self {
            child: None,
            config,
            binary_path,
        }
    }

    /// Generates CLI arguments for ciadpi based on DpiConfig.
    pub fn build_arguments(&self) -> Vec<String> {
        let mut args = vec![
            "-i".to_string(),
            "127.0.0.1".to_string(),
            "-p".to_string(),
            self.config.local_port.to_string(),
            "-s".to_string(),
            self.config.split_pos.clone(),
            "-f".to_string(),
            "-1".to_string(),
            "-t".to_string(),
            self.config.fake_ttl.to_string(),
        ];

        if self.config.disorder {
            args.push("-d".to_string());
            args.push("1+s".to_string());
        }

        if self.config.fake_rst {
            args.push("-r".to_string());
            args.push("1+s".to_string());
        }

        args.push("-F".to_string()); // TCP Fast Open

        args
    }

    /// Spawns the ByeDPI process.
    pub async fn start(&mut self) -> Result<()> {
        if self.child.is_some() {
            warn!("ByeDPI process already running");
            return Ok(());
        }

        let args = self.build_arguments();
        info!("Spawning ByeDPI with args: {:?}", args);

        let child = Command::new(&self.binary_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.child = Some(child);
        info!("ByeDPI process started on 127.0.0.1:{}", self.config.local_port);

        Ok(())
    }

    /// Stops the ByeDPI process cleanly.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            info!("Stopping ByeDPI process");
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        Ok(())
    }

    /// Checks if the process is currently running.
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Drop for DpiSupervisor {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}
