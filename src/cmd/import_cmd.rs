use crate::cmd::profile::{rule, spinner};
use crate::config::{AppConfig, I18n, Profile};
use anyhow::{Result, anyhow};
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use url::Url;

/// Imports proxy profiles from a local JSON file or a remote HTTP URL.
///
/// # Errors
/// Returns an error if the source cannot be read, parsed, or saved.
pub async fn import_profiles(config: &mut AppConfig, i18n: &I18n, source: &str) -> Result<()> {
    rule();
    println!("   📥 {}", i18n.t("import_header").white().bold());
    rule();
    println!("{} {}", i18n.t("import_source"), source.yellow());
    println!();

    let pb = spinner(i18n.t("spinner_import"));
    let content = fetch_source(source, i18n).await;
    pb.finish_and_clear();

    let imported = parse_import_content(&content?)?;
    if imported.is_empty() {
        println!("{}", i18n.t("import_no_profiles").red().bold());
        return Ok(());
    }

    let mut added = 0usize;
    let mut rejected = 0usize;
    for (key, profile) in imported {
        // Validate each entry individually: one malformed record in a
        // subscription should not discard the rest of the import.
        if let Err(err) = profile.validate(&key) {
            eprintln!("  {} {err}", i18n.t("import_skipped").red());
            rejected += 1;
            continue;
        }

        println!(
            "  {} {:<14} — {} ({}:{})",
            i18n.t("import_added"),
            key.yellow().bold(),
            profile.name,
            profile.host,
            profile.port
        );
        config.profiles.insert(key, profile);
        added += 1;
    }

    if added > 0 {
        config.save()?;
    }

    rule();
    println!(
        "{} {}",
        i18n.t("import_success"),
        added.to_string().green().bold()
    );
    if rejected > 0 {
        println!(
            "{} {}",
            i18n.t("import_rejected"),
            rejected.to_string().red().bold()
        );
    }
    Ok(())
}

/// Reads the import payload from a URL or a local path.
async fn fetch_source(source: &str, i18n: &I18n) -> Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        return Ok(client.get(source).send().await?.text().await?);
    }

    let path = Path::new(source);
    if !path.exists() {
        return Err(anyhow!(i18n.format("import_file_not_found", &[source])));
    }
    Ok(fs::read_to_string(path)?)
}

/// Parses an import payload in any of the supported shapes.
///
/// Tried in order: a full `config.json`, a bare profile map, a profile array,
/// then a plain list of proxy URLs.
fn parse_import_content(content: &str) -> Result<BTreeMap<String, Profile>> {
    let trimmed = content.trim();

    // `AppConfig` applies serde defaults to every field, so *any* JSON object
    // deserialises into it successfully — including `{"unrelated": 1}`, which
    // used to "import" the two built-in default profiles. Require the
    // `profiles` key to be genuinely present before taking this branch.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed)
        && value.get("profiles").is_some()
        && let Ok(full_cfg) = serde_json::from_str::<AppConfig>(trimmed)
    {
        return Ok(full_cfg.profiles);
    }

    if let Ok(profiles_map) = serde_json::from_str::<BTreeMap<String, Profile>>(trimmed)
        && !profiles_map.is_empty()
    {
        return Ok(profiles_map);
    }

    if let Ok(profiles_vec) = serde_json::from_str::<Vec<Profile>>(trimmed) {
        return Ok(profiles_vec
            .into_iter()
            .enumerate()
            .map(|(idx, p)| (format!("import_{}", idx + 1), p))
            .collect());
    }

    Ok(parse_proxy_urls(trimmed))
}

/// Parses a newline-separated list of proxy URLs, skipping blanks and comments.
fn parse_proxy_urls(content: &str) -> BTreeMap<String, Profile> {
    let mut map = BTreeMap::new();
    let mut count = 1;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Ok(url) = Url::parse(line) else { continue };
        let scheme = url.scheme().to_lowercase();
        if !matches!(
            scheme.as_str(),
            "socks" | "socks4" | "socks4a" | "socks5" | "socks5h" | "http" | "https"
        ) {
            continue;
        }

        let (Some(host), Some(port)) = (url.host_str(), url.port()) else {
            continue;
        };

        let protocol = if scheme.starts_with("socks") {
            "socks5"
        } else {
            "http"
        };

        map.insert(
            format!("{scheme}_{count}"),
            Profile {
                name: format!("Imported {} {count}", scheme.to_uppercase()),
                host: host.to_string(),
                port,
                protocol: protocol.to_string(),
            },
        );
        count += 1;
    }

    map
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

    #[test]
    fn an_unrelated_json_object_imports_nothing() {
        // Regression: every field of AppConfig has a serde default, so this
        // used to deserialise cleanly and "import" the built-in defaults.
        let map = parse_import_content(r#"{"unrelated": 1, "name": "x"}"#).unwrap();

        assert!(
            map.is_empty(),
            "invented profiles from unrelated JSON: {map:?}"
        );
    }

    #[test]
    fn an_empty_document_imports_nothing() {
        for input in ["", "   ", "{}", "[]"] {
            assert!(
                parse_import_content(input).unwrap().is_empty(),
                "{input:?} should import nothing"
            );
        }
    }

    #[test]
    fn a_bare_profile_map_is_accepted() {
        let json = r#"{"eu":{"name":"EU","host":"10.0.0.9","port":1080,"protocol":"socks5"}}"#;

        let map = parse_import_content(json).unwrap();

        assert_eq!(map.len(), 1);
        assert_eq!(map["eu"].host, "10.0.0.9");
    }

    #[test]
    fn every_supported_url_scheme_is_recognised() {
        let urls = "socks5h://a.example:1\nsocks4://b.example:2\nhttps://c.example:3";

        let map = parse_import_content(urls).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map["socks5h_1"].protocol, "socks5");
        assert_eq!(map["https_3"].protocol, "http");
    }

    #[test]
    fn a_url_without_an_explicit_port_is_skipped() {
        // Guessing a port would silently create a profile pointing nowhere.
        assert!(
            parse_import_content("socks5://example.com")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn unsupported_schemes_are_ignored() {
        let map = parse_import_content("ftp://a.example:21\nvmess://whatever:1").unwrap();

        assert!(map.is_empty());
    }

    #[test]
    fn imported_url_profiles_pass_validation() {
        let map = parse_import_content("socks5://192.168.1.1:1080").unwrap();

        for (key, profile) in &map {
            assert!(profile.validate(key).is_ok());
        }
    }
}
