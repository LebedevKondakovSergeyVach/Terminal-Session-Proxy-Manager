use crate::config::{AppConfig, Profile};
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use url::Url;

/// Imports proxy profiles from a local JSON file or a remote HTTP URL.
pub async fn import_profiles(config: &mut AppConfig, source: &str) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("      📥 {}", "ИМПОРТ ПРОФИЛЕЙ ПРОКСИ".white().bold());
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("Источник: {}", source.yellow());
    println!();

    let content = if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        let resp = client.get(source).send().await?;
        resp.text().await?
    } else {
        let path = Path::new(source);
        if !path.exists() {
            return Err(anyhow!("Файл '{}' не найден!", source));
        }
        fs::read_to_string(path)?
    };

    let imported_profiles = parse_import_content(&content)?;
    if imported_profiles.is_empty() {
        println!(
            "{}",
            "❌ Не удалось распознать профили прокси в источнике."
                .red()
                .bold()
        );
        return Ok(());
    }

    let mut added_count = 0usize;
    for (key, profile) in imported_profiles {
        println!(
            "  • Добавлен профиль: {:<14} — {} ({}:{})",
            key.yellow().bold(),
            profile.name,
            profile.host,
            profile.port
        );
        config.profiles.insert(key, profile);
        added_count += 1;
    }

    config.save()?;
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!(
        "✅ Успешно импортировано профилей: {}",
        added_count.to_string().green().bold()
    );
    Ok(())
}

fn parse_import_content(content: &str) -> Result<BTreeMap<String, Profile>> {
    let mut map = BTreeMap::new();
    let trimmed = content.trim();

    if let Ok(full_cfg) = serde_json::from_str::<AppConfig>(trimmed) {
        return Ok(full_cfg.profiles);
    }

    if let Ok(profiles_map) = serde_json::from_str::<BTreeMap<String, Profile>>(trimmed) {
        return Ok(profiles_map);
    }

    if let Ok(profiles_vec) = serde_json::from_str::<Vec<Profile>>(trimmed) {
        for (idx, p) in profiles_vec.into_iter().enumerate() {
            let key = format!("import_{}", idx + 1);
            map.insert(key, p);
        }
        return Ok(map);
    }

    let mut count = 1;
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Ok(url) = Url::parse(line) {
            let scheme = url.scheme().to_lowercase();
            if scheme == "socks5" || scheme == "socks" || scheme == "http" || scheme == "https" {
                if let (Some(host), Some(port)) = (url.host_str(), url.port()) {
                    let proto = if scheme.starts_with("socks") {
                        "socks5"
                    } else {
                        "http"
                    };
                    let key = format!("{}_{}", scheme, count);
                    let name = format!("Imported {} {}", scheme.to_uppercase(), count);
                    map.insert(
                        key,
                        Profile {
                            name,
                            host: host.to_string(),
                            port,
                            protocol: proto.to_string(),
                        },
                    );
                    count += 1;
                }
            }
        }
    }

    Ok(map)
}
