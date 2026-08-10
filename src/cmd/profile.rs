use crate::config::{AppConfig, Profile};
use anyhow::Result;
use colored::Colorize;

/// Displays all configured proxy profiles.
pub fn list_profiles(config: &AppConfig) {
    println!("{}", "📋 Список профилей в config.json:".cyan().bold());
    for (key, prof) in &config.profiles {
        let active_mark = if key == &config.active_profile {
            " (активный)".green().bold().to_string()
        } else {
            "".to_string()
        };
        println!(
            "  • {:<12} — {} ({}:{}){}",
            key.yellow().bold(),
            prof.name,
            prof.host,
            prof.port,
            active_mark
        );
    }
}

/// Switches active profile to specified key.
pub fn use_profile(config: &mut AppConfig, key: &str) -> Result<()> {
    if config.profiles.contains_key(key) {
        config.active_profile = key.to_string();
        config.save()?;
        if let Some(p) = config.active_profile() {
            println!("⚙️ Переключено на профиль: {} ({}:{})", p.name.green().bold(), p.host, p.port);
        }
    } else {
        eprintln!("{}", format!("❌ Профиль '{}' не найден в config.json!", key).red().bold());
    }
    Ok(())
}

/// Creates or updates a profile in configuration.
pub fn set_profile(
    config: &mut AppConfig,
    key: String,
    name: Option<String>,
    port: Option<u16>,
    host: Option<String>,
    protocol: String,
) -> Result<()> {
    let mut prof = config.profiles.get(&key).cloned().unwrap_or_else(|| Profile {
        name: key.clone(),
        host: "127.0.0.1".to_string(),
        port: 2080,
        protocol: "socks5".to_string(),
    });

    if let Some(n) = name { prof.name = n; }
    if let Some(p) = port { prof.port = p; }
    if let Some(h) = host { prof.host = h; }
    prof.protocol = protocol;

    config.profiles.insert(key.clone(), prof.clone());
    config.active_profile = key.clone();
    config.save()?;

    println!("⚙️ Профиль '{}' сохранен: {} ({}:{})", key, prof.name.green().bold(), prof.host, prof.port);
    Ok(())
}

/// Removes a profile from configuration by key.
pub fn remove_profile(config: &mut AppConfig, key: &str) -> Result<()> {
    if config.profiles.remove(key).is_some() {
        if config.active_profile == key {
            if let Some((first_key, _)) = config.profiles.iter().next() {
                config.active_profile = first_key.clone();
            }
        }
        config.save()?;
        println!("🗑️ Профиль '{}' удален", key);
    } else {
        eprintln!("{}", format!("❌ Профиль '{}' не найден!", key).red().bold());
    }
    Ok(())
}
