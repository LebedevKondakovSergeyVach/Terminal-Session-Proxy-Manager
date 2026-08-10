use crate::config::AppSettings;
use anyhow::Result;
use colored::Colorize;

pub fn show_settings_path() {
    println!("📍 Файл настроек settings.json:");
    println!("{}", AppSettings::get_settings_path().display());
}

pub fn show_settings() -> Result<()> {
    let settings = AppSettings::load();
    println!("{}", serde_json::to_string_pretty(&settings)?);
    Ok(())
}

pub fn set_config_path(path: String) -> Result<()> {
    let mut settings = AppSettings::load();
    settings.config_path = Some(path.clone());
    settings.save()?;
    println!("⚙️ Установлен новый путь к конфигу в settings.json: {}", path.green().bold());
    Ok(())
}
