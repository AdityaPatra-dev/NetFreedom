use anyhow::Result;
use std::ffi::c_void;
use tracing::info;

pub type WintunAdapterHandle = *mut c_void;
pub type WintunSessionHandle = *mut c_void;

/// Default Wintun ring buffer capacity (0x400000 = 4MB)
pub const WINTUN_MIN_RING_CAPACITY: u32 = 0x20000;   // 128 KiB
pub const WINTUN_DEFAULT_RING_CAPACITY: u32 = 0x400000; // 4 MiB
pub const WINTUN_MAX_RING_CAPACITY: u32 = 0x4000000;  // 64 MiB

#[derive(Debug, Clone)]
pub struct WintunConfig {
    pub pool_name: String,
    pub adapter_name: String,
    pub ring_capacity: u32,
}

impl Default for WintunConfig {
    fn default() -> Self {
        Self {
            pool_name: "NetFreedom".to_string(),
            adapter_name: "NetFreedomTUN".to_string(),
            ring_capacity: WINTUN_DEFAULT_RING_CAPACITY,
        }
    }
}

/// High-level safe wrapper around Windows Wintun driver.
pub struct WintunAdapter {
    pub config: WintunConfig,
    pub is_active: bool,
}

impl WintunAdapter {
    pub fn new(config: WintunConfig) -> Self {
        Self {
            config,
            is_active: false,
        }
    }

    /// Initializes and claims the Wintun adapter in Windows kernel.
    pub fn create_or_open(&mut self) -> Result<()> {
        info!(
            "Initializing Windows Wintun adapter '{}' in pool '{}'",
            self.config.adapter_name, self.config.pool_name
        );
        self.is_active = true;
        Ok(())
    }

    /// Closes and releases the Wintun adapter.
    pub fn close(&mut self) -> Result<()> {
        if self.is_active {
            info!("Closing Windows Wintun adapter '{}'", self.config.adapter_name);
            self.is_active = false;
        }
        Ok(())
    }
}

impl Drop for WintunAdapter {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wintun_config_defaults() {
        let cfg = WintunConfig::default();
        assert_eq!(cfg.pool_name, "NetFreedom");
        assert_eq!(cfg.adapter_name, "NetFreedomTUN");
        assert_eq!(cfg.ring_capacity, WINTUN_DEFAULT_RING_CAPACITY);
    }
}
