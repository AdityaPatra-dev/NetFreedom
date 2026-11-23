use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetFreedomError {
    #[error("Network I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("DPI bypass error: {0}")]
    Dpi(String),

    #[error("VPN / Tunnel error: {0}")]
    Vpn(String),

    #[error("Registration error: {0}")]
    Registration(String),

    #[error("Health check failure: {0}")]
    HealthCheck(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Platform permission or system error: {0}")]
    Platform(String),
}

pub type Result<T> = std::result::Result<T, NetFreedomError>;
