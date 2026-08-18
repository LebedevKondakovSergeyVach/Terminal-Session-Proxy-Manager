use super::profile::Profile;
use super::settings::AppSettings;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Target web service configuration for latency ping testing.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingTarget {
    /// Service name displayed in ping results.
    pub name: String,
    /// HTTPS URL endpoint to probe.
    pub url: String,
}

/// Endpoint configuration for network diagnostics.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiagnoseEndpoint {
    /// Endpoint description.
    pub name: String,
    /// HTTPS URL to test accessibility.
    pub url: String,
}

/// Main application proxy configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    /// Currently active profile key in `profiles` map.
    pub active_profile: String,
    /// Whether proxy environment variables are active.
    pub enabled: bool,
    /// Map of profile keys to Profile objects.
    pub profiles: BTreeMap<String, Profile>,
    /// Web targets for latency testing.
    pub ping_targets: Vec<PingTarget>,
    /// Endpoints for network diagnostic checks.
    pub diagnose_endpoints: Vec<DiagnoseEndpoint>,
    /// List of IP/Geo JSON API endpoints.
    pub geo_apis: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                name: "Default Proxy".to_string(),
                host: "127.0.0.1".to_string(),
                port: 1080,
                protocol: "socks5".to_string(),
            },
        );
        profiles.insert(
            "custom".to_string(),
            Profile {
                name: "Custom HTTP Proxy".to_string(),
                host: "127.0.0.1".to_string(),
                port: 8080,
                protocol: "http".to_string(),
            },
        );

        let ping_targets = vec![
            PingTarget {
                name: "Google API".to_string(),
                url: "https://generativelanguage.googleapis.com".to_string(),
            },
            PingTarget {
                name: "GitHub".to_string(),
                url: "https://github.com".to_string(),
            },
            PingTarget {
                name: "OpenAI API".to_string(),
                url: "https://api.openai.com".to_string(),
            },
            PingTarget {
                name: "Anthropic API".to_string(),
                url: "https://api.anthropic.com".to_string(),
            },
            PingTarget {
                name: "Telegram API".to_string(),
                url: "https://api.telegram.org".to_string(),
            },
        ];

        let diagnose_endpoints = vec![
            DiagnoseEndpoint {
                name: "Gemini API".to_string(),
                url: "https://generativelanguage.googleapis.com".to_string(),
            },
            DiagnoseEndpoint {
                name: "Cloud Code".to_string(),
                url: "https://daily-cloudcode-pa.googleapis.com".to_string(),
            },
        ];

        let geo_apis = vec![
            "https://ifconfig.co/json".to_string(),
            "http://ip-api.com/json".to_string(),
        ];

        Self {
            active_profile: "throne".to_string(),
            enabled: false,
            profiles,
            ping_targets,
            diagnose_endpoints,
            geo_apis,
        }
    }
}

fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Ok(strip) = p.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(strip);
        }
    }
    p.to_path_buf()
}

impl AppConfig {
    /// Resolves the filesystem path to active `config.json`.
    pub fn get_config_path() -> PathBuf {
        let settings = AppSettings::load();
        if let Some(ref custom_path) = settings.config_path {
            if !custom_path.is_empty() {
                let expanded = expand_tilde(custom_path);
                if expanded.is_absolute() {
                    return expanded;
                }
                let settings_path = AppSettings::get_settings_path();
                if let Some(parent) = settings_path.parent() {
                    return parent.join(custom_path);
                }
                return expanded;
            }
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("terminal-session-proxy-manager")
            .join("config.json")
    }

    /// Loads `config.json` from resolved path or returns default configuration.
    pub fn load() -> Self {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    /// Saves the current configuration to the config file.
    ///
    /// # Errors
    /// Returns an error if the parent directory cannot be created or the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Returns a reference to the active `Profile` if present.
    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles.get(&self.active_profile)
    }

    /// Generates SOCKS5 URL string for active profile.
    pub fn get_socks_url(&self) -> Option<String> {
        self.active_profile()
            .map(|p| format!("{}://{}:{}", p.protocol, p.host, p.port))
    }

    /// Generates HTTP URL string for active profile.
    pub fn get_http_url(&self) -> Option<String> {
        self.active_profile()
            .map(|p| format!("http://{}:{}", p.host, p.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_config() {
        let config = AppConfig::default();
        assert_eq!(config.active_profile, "throne");
        assert!(!config.enabled);
        assert!(config.profiles.contains_key("throne"));
        assert!(config.profiles.contains_key("v2ray"));
        assert!(!config.ping_targets.is_empty());
    }

    #[test]
    fn test_active_profile_resolution() {
        let mut config = AppConfig::default();

        // Default is throne
        let profile = config.active_profile().unwrap();
        assert_eq!(profile.name, "Throne");

        // Switch to v2ray
        config.active_profile = "v2ray".to_string();
        let profile2 = config.active_profile().unwrap();
        assert_eq!(profile2.name, "v2rayN");
        assert_eq!(profile2.port, 10808);

        // Invalid profile
        config.active_profile = "nonexistent".to_string();
        assert!(config.active_profile().is_none());
    }

    #[test]
    fn test_url_generation() {
        let config = AppConfig::default();
        assert_eq!(config.get_socks_url().unwrap(), "socks5://127.0.0.1:2080");
        assert_eq!(config.get_http_url().unwrap(), "http://127.0.0.1:2080");
    }
}
