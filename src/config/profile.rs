use serde::{Deserialize, Serialize};

fn default_protocol() -> String {
    "socks5".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}
