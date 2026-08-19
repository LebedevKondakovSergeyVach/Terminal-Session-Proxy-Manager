use crate::cli::LangCode;
use crate::config::{AppSettings, I18n};
use anyhow::Result;
use colored::Colorize;

/// Displays current filesystem path to `settings.json`.
pub fn show_settings_path(i18n: &I18n) {
    println!("{}", i18n.t("settings_path_label"));
    println!("{}", AppSettings::get_settings_path().display());
}

/// Displays formatted JSON contents of `settings.json`.
///
/// # Errors
/// Returns an error if the settings cannot be serialised.
pub fn show_settings() -> Result<()> {
    let settings = AppSettings::load();
    println!("{}", serde_json::to_string_pretty(&settings)?);
    Ok(())
}

/// Updates the config path in `settings.json`.
///
/// # Errors
/// Returns an error if the settings file cannot be written.
pub fn set_config_path(path: String, i18n: &I18n) -> Result<()> {
    let mut settings = AppSettings::load();
    settings.config_path = Some(path.clone());
    settings.save()?;
    println!("{} {}", i18n.t("config_path_set"), path.green().bold());
    Ok(())
}

/// Updates the interface language in `settings.json`.
///
/// # Errors
/// Returns an error if the settings file cannot be written.
pub fn set_lang(lang: LangCode) -> Result<()> {
    let mut settings = AppSettings::load();
    let code = lang.as_str();
    settings.lang = code.to_string();
    settings.save()?;
    let i18n = I18n::load(code);
    println!("{} {}", i18n.t("lang_switched"), code.green().bold());
    Ok(())
}
