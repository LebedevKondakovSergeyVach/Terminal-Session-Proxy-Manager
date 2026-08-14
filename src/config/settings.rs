use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Environment settings pointing to active configuration file and language preference.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    /// Custom path to `config.json`.
    pub config_path: Option<String>,
    /// Language code (e.g., "ru", "en"). Defaults to "ru".
    #[serde(default = "default_lang")]
    pub lang: String,
}

fn default_lang() -> String {
    "ru".to_string()
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
    pub fn get_settings_path() -> PathBuf {
        let global_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("proxy-cli")
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

    /// Loads `settings.json` or returns default settings if absent.
    pub fn load() -> Self {
        let path = Self::get_settings_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }
        }
        let default_settings = Self::default();
        let _ = default_settings.save();
        default_settings
    }

    /// Saves the settings to the file system.
    ///
    /// # Errors
    /// Returns an error if the parent directory cannot be created or the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::get_settings_path();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
