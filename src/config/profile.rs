use serde::{Deserialize, Serialize};

fn default_protocol() -> String {
    "socks5".to_string()
}

/// Proxy connection profile configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    /// Human-readable display name for the proxy profile.
    pub name: String,
    /// Host address or IP (e.g. `127.0.0.1`).
    pub host: String,
    /// Port number (e.g. `2080`, `10808`).
    pub port: u16,
    /// Protocol schema (`socks5` or `http`).
    #[serde(default = "default_protocol")]
    pub protocol: String,
}
