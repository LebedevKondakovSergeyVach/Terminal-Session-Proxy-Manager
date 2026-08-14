use crate::config::{AppConfig, I18n, Profile};
use anyhow::{anyhow, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use url::Url;

/// Imports proxy profiles from a local JSON file or a remote HTTP URL.
pub async fn import_profiles(config: &mut AppConfig, i18n: &I18n, source: &str) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("      📥 {}", i18n.t("import_header").white().bold());
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("{} {}", i18n.t("import_source"), source.yellow());
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(i18n.t("spinner_import").to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let content_res = if source.starts_with("http://") || source.starts_with("https://") {
        let client_res = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build();
        match client_res {
            Ok(client) => match client.get(source).send().await {
                Ok(resp) => resp.text().await.map_err(|e| anyhow!(e)),
                Err(e) => Err(anyhow!(e)),
            },
            Err(e) => Err(anyhow!(e)),
        }
    } else {
        let path = Path::new(source);
        if !path.exists() {
            let msg = i18n.t("import_file_not_found").replace("{}", source);
            Err(anyhow!(msg))
        } else {
            fs::read_to_string(path).map_err(|e| anyhow!(e))
        }
    };

    pb.finish_and_clear();

    let content = content_res?;

    let imported_profiles = parse_import_content(&content)?;
    if imported_profiles.is_empty() {
        println!("{}", i18n.t("import_no_profiles").red().bold());
        return Ok(());
    }

    let mut added_count = 0usize;
    for (key, profile) in imported_profiles {
        println!(
            "  {} {:<14} — {} ({}:{})",
            i18n.t("import_added"),
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
        "{} {}",
        i18n.t("import_success"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_import_full_config() {
        let json = r#"{
            "active_profile": "test",
            "profiles": {
                "test": {
                    "name": "Test Profile",
                    "host": "127.0.0.1",
                    "port": 1080,
                    "protocol": "socks5"
                }
            }
        }"#;
        let map = parse_import_content(json).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("test").unwrap().port, 1080);
    }

    #[test]
    fn test_parse_import_urls() {
        let urls = "socks5://192.168.1.1:1080\nhttp://10.0.0.1:8080\n# comment\ninvalid_url";
        let map = parse_import_content(urls).unwrap();
        assert_eq!(map.len(), 2);
        let p1 = map.get("socks5_1").unwrap();
        assert_eq!(p1.host, "192.168.1.1");
        assert_eq!(p1.port, 1080);
        let p2 = map.get("http_2").unwrap();
        assert_eq!(p2.host, "10.0.0.1");
        assert_eq!(p2.port, 8080);
    }

    #[test]
    fn test_parse_import_json_array() {
        let json = r#"[
            {
                "name": "Array Item 1",
                "host": "1.1.1.1",
                "port": 1234,
                "protocol": "http"
            }
        ]"#;
        let map = parse_import_content(json).unwrap();
        assert_eq!(map.len(), 1);
        let p = map.get("import_1").unwrap();
        assert_eq!(p.host, "1.1.1.1");
    }
}
