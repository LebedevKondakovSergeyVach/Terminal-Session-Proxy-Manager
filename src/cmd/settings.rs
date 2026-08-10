use crate::config::{AppSettings, I18n};
use anyhow::Result;
use colored::Colorize;

/// Displays current filesystem path to `settings.json`.
pub fn show_settings_path() {
    println!("📍 Файл настроек settings.json:");
    println!("{}", AppSettings::get_settings_path().display());
}

/// Displays formatted JSON contents of `settings.json`.
pub fn show_settings() -> Result<()> {
    let settings = AppSettings::load();
    println!("{}", serde_json::to_string_pretty(&settings)?);
    Ok(())
}

/// Updates target `config_path` inside `settings.json`.
pub fn set_config_path(path: String) -> Result<()> {
    let mut settings = AppSettings::load();
    settings.config_path = Some(path.clone());
    settings.save()?;
    println!(
        "⚙️ Установлен новый путь к конфигу в settings.json: {}",
        path.green().bold()
    );
    Ok(())
}

/// Updates target `lang` inside `settings.json`.
pub fn set_lang(lang: String) -> Result<()> {
    let mut settings = AppSettings::load();
    let code = match lang.to_lowercase().as_str() {
        "en" | "english" => "en",
        _ => "ru",
    };
    settings.lang = code.to_string();
    settings.save()?;
    let i18n = I18n::load(code);
    println!("{} {}", i18n.t("lang_switched"), code.green().bold());
    Ok(())
}
