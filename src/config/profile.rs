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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_profile_deserialization_default_protocol() {
        let json_data = json!({
            "name": "Test",
            "host": "127.0.0.1",
            "port": 8080
        });
        
        let profile: Profile = serde_json::from_value(json_data).unwrap();
        assert_eq!(profile.name, "Test");
        assert_eq!(profile.host, "127.0.0.1");
        assert_eq!(profile.port, 8080);
        assert_eq!(profile.protocol, "socks5");
    }

    #[test]
    fn test_profile_deserialization_custom_protocol() {
        let json_data = json!({
            "name": "Test HTTP",
            "host": "192.168.1.1",
            "port": 3128,
            "protocol": "http"
        });
        
        let profile: Profile = serde_json::from_value(json_data).unwrap();
        assert_eq!(profile.protocol, "http");
    }
}
