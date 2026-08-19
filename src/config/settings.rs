use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::app::APP_DIR;

/// Environment variable that overrides the resolved `settings.json` location.
pub const SETTINGS_PATH_ENV: &str = "TSPM_SETTINGS";

static SETTINGS_PATH_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Pins the `settings.json` path for the rest of the process.
///
/// Called once from `main` with the value of `--settings`.
pub fn set_settings_path_override(path: PathBuf) {
    let _ = SETTINGS_PATH_OVERRIDE.set(path);
}

fn default_lang() -> String {
    "ru".to_string()
}

/// Global preferences: where the profile config lives and which language to speak.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppSettings {
    /// Custom path to `config.json`; `None` uses the OS config directory.
    pub config_path: Option<String>,
    /// Language code (`"ru"` or `"en"`).
    pub lang: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_path: None,
            lang: default_lang(),
        }
    }
}

impl AppSettings {
    /// Resolves the filesystem path to `settings.json`.
    ///
    /// Precedence: `--settings` / `TSPM_SETTINGS`, then the OS config
    /// directory, then a `settings.json` in the working directory.
    ///
    /// The working-directory entry exists so the repository can be run from a
    /// checkout without touching the developer's real settings. It is
    /// deliberately last: `settings.json` is a common filename, and letting an
    /// arbitrary directory win over the user's own configuration would make
    /// the tool behave differently depending on where it was invoked.
    #[must_use]
    pub fn get_settings_path() -> PathBuf {
        if let Some(path) = SETTINGS_PATH_OVERRIDE.get() {
            return path.clone();
        }

        // Read directly rather than relying on clap's `env` binding: settings
        // are loaded to pick the language *before* the arguments are parsed.
        if let Some(path) = std::env::var_os(SETTINGS_PATH_ENV).filter(|p| !p.is_empty()) {
            return PathBuf::from(path);
        }

        let global_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_DIR)
            .join("settings.json");

        if global_path.exists() {
            return global_path;
        }

        let local_path = PathBuf::from("./settings.json");
        if local_path.exists() {
            return local_path;
        }

        global_path
    }

    /// Loads `settings.json`, falling back to defaults when it is absent.
    ///
    /// As with the profile config, a file that exists but cannot be parsed is
    /// reported and left alone rather than overwritten.
    #[must_use]
    pub fn load() -> Self {
        let path = Self::get_settings_path();

        match Self::load_from(&path) {
            Ok(Some(settings)) => settings,
            Ok(None) => {
                let defaults = Self::default();
                let _ = defaults.save();
                defaults
            }
            Err(err) => {
                eprintln!("warning: {err}");
                eprintln!(
                    "warning: using default settings; {} was left unchanged",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Reads a settings file, distinguishing "absent" from "present but broken".
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_from(path: &Path) -> Result<Option<Self>, crate::error::ProxyError> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let settings = serde_json::from_str(&content).map_err(|source| {
            crate::error::ProxyError::ConfigParse {
                path: path.to_path_buf(),
                source,
            }
        })?;

        Ok(Some(settings))
    }

    /// Saves the settings to the file system.
    ///
    /// # Errors
    /// Returns an error if the parent directory cannot be created or the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::get_settings_path();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_russian_with_no_custom_config_path() {
        let settings = AppSettings::default();

        assert_eq!(settings.lang, "ru");
        assert!(settings.config_path.is_none());
    }

    #[test]
    fn a_partial_settings_file_fills_in_the_missing_fields() {
        let settings: AppSettings = serde_json::from_str(r#"{"lang":"en"}"#).unwrap();

        assert_eq!(settings.lang, "en");
        assert!(settings.config_path.is_none());
    }

    #[test]
    fn an_empty_settings_file_is_valid_and_yields_defaults() {
        let settings: AppSettings = serde_json::from_str("{}").unwrap();

        assert_eq!(settings.lang, default_lang());
    }

    #[test]
    fn a_broken_settings_file_is_reported_and_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let original = r#"{"lang": }"#;
        fs::write(&path, original).unwrap();

        let err = AppSettings::load_from(&path).unwrap_err();

        assert!(err.to_string().contains("is not valid JSON"));
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn settings_round_trip_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let settings = AppSettings {
            config_path: Some("~/elsewhere.json".to_string()),
            lang: "en".to_string(),
        };
        fs::write(&path, serde_json::to_string_pretty(&settings).unwrap()).unwrap();

        let loaded = AppSettings::load_from(&path).unwrap().unwrap();

        assert_eq!(loaded.config_path.as_deref(), Some("~/elsewhere.json"));
        assert_eq!(loaded.lang, "en");
    }
}
