use super::profile::Profile;
use super::settings::AppSettings;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Environment variable that overrides the resolved `config.json` location.
pub const CONFIG_PATH_ENV: &str = "TSPM_CONFIG";

/// Directory created under the OS config directory.
pub(crate) const APP_DIR: &str = "terminal-session-proxy-manager";

static CONFIG_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Pins the `config.json` path for the rest of the process.
///
/// Called once from `main` with the value of `--config`. Later calls are
/// ignored, so a command handler cannot silently retarget the config file
/// halfway through a run.
pub fn set_config_path_override(path: PathBuf) {
    let _ = CONFIG_PATH_OVERRIDE.set(path);
}

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
///
/// Every field carries a serde default, so a config written by an older
/// version keeps loading after new fields are added, and unknown fields left
/// behind by a newer version are ignored rather than fatal.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    /// Currently active profile key in `profiles` map.
    pub active_profile: String,
    /// Map of profile keys to Profile objects.
    pub profiles: BTreeMap<String, Profile>,
    /// Web targets for latency testing.
    pub ping_targets: Vec<PingTarget>,
    /// Endpoints for network diagnostic checks.
    pub diagnose_endpoints: Vec<DiagnoseEndpoint>,
    /// List of IP/Geo JSON API endpoints, tried in order until one answers.
    pub geo_apis: Vec<String>,
    /// Plain-text endpoint returning the external IPv4 address.
    pub ipv4_api: String,
    /// Plain-text endpoint returning the external IPv6 address.
    pub ipv6_api: String,
    /// URL probed by `monitor` to decide whether the active proxy still works.
    ///
    /// Should be a cheap endpoint that answers `204 No Content`; anything in
    /// the 2xx/3xx range counts as healthy.
    pub health_check_url: String,
    /// URL downloaded by `speedtest` to measure throughput.
    pub speedtest_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        // Deliberately generic loopback examples. Shipping a real host or an
        // unusual port here would leak whoever's setup it was copied from, and
        // would silently point new users at a proxy that is not theirs.
        let profiles = BTreeMap::from([
            (
                "default".to_string(),
                Profile {
                    name: "Default Proxy".to_string(),
                    host: "127.0.0.1".to_string(),
                    port: 1080,
                    protocol: "socks5".to_string(),
                },
            ),
            (
                "custom".to_string(),
                Profile {
                    name: "Custom HTTP Proxy".to_string(),
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    protocol: "http".to_string(),
                },
            ),
        ]);

        let ping_targets = [
            ("Google", "https://www.google.com"),
            ("GitHub", "https://github.com"),
            ("Cloudflare", "https://1.1.1.1"),
        ]
        .map(|(name, url)| PingTarget {
            name: name.to_string(),
            url: url.to_string(),
        })
        .to_vec();

        let diagnose_endpoints = [
            ("GitHub API", "https://api.github.com"),
            ("Cloudflare DNS", "https://cloudflare-dns.com"),
        ]
        .map(|(name, url)| DiagnoseEndpoint {
            name: name.to_string(),
            url: url.to_string(),
        })
        .to_vec();

        Self {
            active_profile: "default".to_string(),
            profiles,
            ping_targets,
            diagnose_endpoints,
            geo_apis: vec![
                "https://ifconfig.co/json".to_string(),
                "http://ip-api.com/json".to_string(),
            ],
            ipv4_api: "https://api4.ipify.org".to_string(),
            ipv6_api: "https://api6.ipify.org".to_string(),
            // Returns 204 with an empty body: the cheapest unambiguous
            // "the tunnel carries real traffic" signal available.
            health_check_url: "https://www.gstatic.com/generate_204".to_string(),
            speedtest_url: "https://speed.cloudflare.com/__down?bytes=2097152".to_string(),
        }
    }
}

fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Ok(stripped) = p.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    p.to_path_buf()
}

/// Resolves the config path from its inputs, with no I/O and no globals.
///
/// Split out from [`AppConfig::get_config_path`] so the precedence rules can be
/// tested directly instead of through the environment.
fn resolve_config_path(
    override_path: Option<&Path>,
    settings_config_path: Option<&str>,
    settings_dir: Option<&Path>,
    default_dir: &Path,
) -> PathBuf {
    if let Some(path) = override_path {
        return expand_tilde(path);
    }

    if let Some(custom) = settings_config_path.filter(|p| !p.is_empty()) {
        let expanded = expand_tilde(custom);
        if expanded.is_absolute() {
            return expanded;
        }
        // A relative `config_path` is relative to settings.json, not to the
        // caller's working directory, so the same settings file behaves the
        // same no matter where the binary is invoked from.
        if let Some(parent) = settings_dir {
            return parent.join(custom);
        }
        return expanded;
    }

    default_dir.join(APP_DIR).join("config.json")
}

impl AppConfig {
    /// Resolves the filesystem path to the active `config.json`.
    ///
    /// Precedence: `--config` / `TSPM_CONFIG`, then `config_path` from
    /// `settings.json`, then the OS config directory.
    #[must_use]
    pub fn get_config_path() -> PathBuf {
        let settings = AppSettings::load();
        let settings_path = AppSettings::get_settings_path();

        // As with settings, the environment is read directly so the override
        // applies during the pre-parse phase too.
        let env_override = std::env::var_os(CONFIG_PATH_ENV)
            .filter(|p| !p.is_empty())
            .map(PathBuf::from);
        let override_path = CONFIG_PATH_OVERRIDE.get().cloned().or(env_override);

        resolve_config_path(
            override_path.as_deref(),
            settings.config_path.as_deref(),
            settings_path.parent(),
            &dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")),
        )
    }

    /// Loads `config.json`, falling back to defaults when it is absent.
    ///
    /// A file that exists but cannot be read or parsed is reported on stderr
    /// and left untouched. Overwriting it with defaults — which is what this
    /// used to do — silently destroyed every profile the user had over a single
    /// stray comma.
    #[must_use]
    pub fn load() -> Self {
        let path = Self::get_config_path();

        match Self::load_from(&path) {
            Ok(Some(config)) => config,
            Ok(None) => {
                // First run: materialise a config so the user has something to
                // edit. Failure here is not fatal; defaults still work.
                let default_config = Self::default();
                if let Err(err) = default_config.save() {
                    eprintln!("warning: could not create {}: {err}", path.display());
                }
                default_config
            }
            Err(err) => {
                eprintln!("warning: {err}");
                eprintln!(
                    "warning: falling back to built-in defaults; {} was left unchanged",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Reads a config file, distinguishing "absent" from "present but broken".
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_from(path: &Path) -> Result<Option<Self>, crate::error::ProxyError> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content).map_err(|source| {
            crate::error::ProxyError::ConfigParse {
                path: path.to_path_buf(),
                source,
            }
        })?;

        Ok(Some(config))
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
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Returns a reference to the active `Profile` if present.
    #[must_use]
    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles.get(&self.active_profile)
    }

    /// Generates the proxy URL for the active profile, using its own protocol.
    #[must_use]
    pub fn get_socks_url(&self) -> Option<String> {
        self.active_profile()
            .map(|p| format!("{}://{}:{}", p.protocol, p.host, p.port))
    }

    /// Generates the HTTP-scheme proxy URL for the active profile.
    #[must_use]
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
        assert_eq!(config.active_profile, "default");
        assert!(config.profiles.contains_key("default"));
        assert!(config.profiles.contains_key("custom"));
        assert!(!config.ping_targets.is_empty());
    }

    #[test]
    fn default_profiles_never_point_outside_loopback() {
        // Guards the project's "no private data in defaults" rule: a real host
        // leaking in here would ship someone's personal proxy to every user.
        for (key, profile) in &AppConfig::default().profiles {
            assert_eq!(
                profile.host, "127.0.0.1",
                "default profile '{key}' points at a non-loopback host"
            );
        }
    }

    #[test]
    fn test_active_profile_resolution() {
        let mut config = AppConfig::default();

        let profile = config.active_profile().unwrap();
        assert_eq!(profile.name, "Default Proxy");

        config.active_profile = "custom".to_string();
        let profile2 = config.active_profile().unwrap();
        assert_eq!(profile2.name, "Custom HTTP Proxy");
        assert_eq!(profile2.port, 8080);

        config.active_profile = "nonexistent".to_string();
        assert!(config.active_profile().is_none());
    }

    #[test]
    fn test_url_generation() {
        let config = AppConfig::default();
        assert_eq!(config.get_socks_url().unwrap(), "socks5://127.0.0.1:1080");
        assert_eq!(config.get_http_url().unwrap(), "http://127.0.0.1:1080");
    }

    #[test]
    fn proxy_url_honours_the_profile_protocol() {
        let config = AppConfig {
            active_profile: "custom".to_string(),
            ..AppConfig::default()
        };

        assert_eq!(config.get_socks_url().unwrap(), "http://127.0.0.1:8080");
    }

    #[test]
    fn a_config_missing_new_fields_still_loads_with_defaults() {
        // Simulates a config.json written by an older release.
        let legacy = r#"{"active_profile":"a","enabled":true,
            "profiles":{"a":{"name":"A","host":"127.0.0.1","port":1}}}"#;

        let config: AppConfig = serde_json::from_str(legacy).unwrap();

        assert_eq!(config.active_profile, "a");
        assert_eq!(config.ipv4_api, AppConfig::default().ipv4_api);
        assert!(!config.health_check_url.is_empty());
    }

    #[test]
    fn a_broken_config_is_reported_and_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let original = "{ this is not json";
        fs::write(&path, original).unwrap();

        let err = AppConfig::load_from(&path).unwrap_err();

        assert!(err.to_string().contains("is not valid JSON"));
        // The critical half of the assertion: the user's file survived.
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn an_absent_config_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();

        let loaded = AppConfig::load_from(&dir.path().join("nope.json")).unwrap();

        assert!(loaded.is_none());
    }

    #[test]
    fn a_saved_config_round_trips_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let config = AppConfig::default();
        fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let loaded = AppConfig::load_from(&path).unwrap().unwrap();

        assert_eq!(loaded.active_profile, config.active_profile);
        assert_eq!(loaded.profiles.len(), config.profiles.len());
        assert_eq!(loaded.speedtest_url, config.speedtest_url);
    }

    #[test]
    fn explicit_override_beats_every_other_source() {
        let path = resolve_config_path(
            Some(Path::new("/explicit/config.json")),
            Some("/from/settings.json"),
            Some(Path::new("/settings/dir")),
            Path::new("/default"),
        );

        assert_eq!(path, PathBuf::from("/explicit/config.json"));
    }

    #[test]
    fn a_relative_settings_path_resolves_against_the_settings_file() {
        // Not against the current working directory, which would make the tool
        // behave differently depending on where it was invoked.
        let path = resolve_config_path(
            None,
            Some("profiles/config.json"),
            Some(Path::new("/home/u/.config/app")),
            Path::new("/default"),
        );

        assert_eq!(
            path,
            PathBuf::from("/home/u/.config/app/profiles/config.json")
        );
    }

    #[test]
    fn an_empty_settings_path_falls_through_to_the_os_directory() {
        let path = resolve_config_path(None, Some(""), None, Path::new("/default"));

        assert_eq!(
            path,
            PathBuf::from("/default").join(APP_DIR).join("config.json")
        );
    }

    #[test]
    fn tilde_in_a_configured_path_expands_to_the_home_directory() {
        let home = dirs::home_dir().unwrap();

        let path = resolve_config_path(None, Some("~/custom.json"), None, Path::new("/default"));

        assert_eq!(path, home.join("custom.json"));
    }
}

#[cfg(test)]
mod shipped_defaults {
    use super::*;

    /// `configs/config.default.json` is documentation users copy from, so it
    /// must not drift away from what the binary actually writes on first run.
    /// Regenerate with `cargo run -- --config-file /dev/null config show`.
    #[test]
    fn the_shipped_default_config_matches_the_code() {
        let shipped: AppConfig =
            serde_json::from_str(include_str!("../../configs/config.default.json"))
                .expect("configs/config.default.json is not valid JSON");
        let actual = AppConfig::default();

        assert_eq!(
            serde_json::to_value(&shipped).unwrap(),
            serde_json::to_value(&actual).unwrap(),
            "configs/config.default.json is out of date"
        );
    }
}
