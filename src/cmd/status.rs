use crate::config::AppConfig;
use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Holds resolved network IP, proxy status, and physical location information.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusInfo {
    /// Friendly status string (`🟢 ВКЛЮЧЕН` or `🔴 НАПРЯМУЮ`).
    pub status: String,
    /// Profile key identifier.
    pub profile_key: String,
    /// Profile display name.
    pub profile_name: String,
    /// Active proxy URL string if enabled.
    pub active_proxy: Option<String>,
    /// Resolved external IPv4 address.
    pub ipv4: String,
    /// Resolved external IPv6 address.
    pub ipv6: String,
    /// Resolved physical location (e.g. `City, Country (ISO)`).
    pub location: String,
}

#[derive(Debug, Deserialize)]
struct GeoResponse {
    ip: Option<String>,
    query: Option<String>,
    city: Option<String>,
    region_name: Option<String>,
    #[serde(rename = "regionName")]
    region_name_alt: Option<String>,
    country: Option<String>,
    country_iso: Option<String>,
    #[serde(rename = "countryCode")]
    country_code_alt: Option<String>,
}

fn build_client(proxy_url: Option<&str>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2));

    if let Some(p) = proxy_url {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            builder = builder.proxy(proxy);
        }
    }
    builder.build().unwrap_or_default()
}

/// Resolves external IPv4, IPv6, and physical location from configured Geo APIs.
pub async fn get_status_info(config: &AppConfig) -> StatusInfo {
    let proxy_env = env::var("ALL_PROXY").or_else(|_| env::var("http_proxy")).ok();
    let client = build_client(proxy_env.as_deref());

    let mut ip_from_json = String::new();
    let mut location = String::from("недоступно");

    let mut resp_data: Option<GeoResponse> = None;
    for geo_url in &config.geo_apis {
        if let Ok(resp) = client.get(geo_url).send().await {
            if let Ok(data) = resp.json::<GeoResponse>().await {
                resp_data = Some(data);
                break;
            }
        }
    }

    if let Some(data) = resp_data {
        if let Some(ip) = data.ip.or(data.query) {
            ip_from_json = ip;
        }

        let city = data.city.unwrap_or_default();
        let region = data.region_name.or(data.region_name_alt).unwrap_or_default();
        let country = data.country.unwrap_or_default();
        let country_code = data.country_iso.or(data.country_code_alt).unwrap_or_default();

        let mut parts = Vec::new();
        if !city.is_empty() {
            parts.push(city);
        }
        if !region.is_empty() && !parts.contains(&region) {
            parts.push(region);
        }
        if !country.is_empty() {
            parts.push(country.clone());
        }

        if !parts.is_empty() {
            let mut loc_str = parts.join(", ");
            if !country_code.is_empty() && !loc_str.contains(&country_code) {
                loc_str.push_str(&format!(" ({})", country_code));
            }
            location = loc_str;
        } else if !country_code.is_empty() {
            location = country_code;
        } else {
            location = String::from("неизвестно");
        }
    }

    let mut ipv4 = String::new();
    let mut ipv6 = String::new();

    if ip_from_json.contains('.') {
        ipv4 = ip_from_json;
    } else if ip_from_json.contains(':') {
        ipv6 = ip_from_json;
    }

    if ipv4.is_empty() {
        if let Ok(resp) = client.get("https://api4.ipify.org").send().await {
            if let Ok(text) = resp.text().await {
                ipv4 = text.trim().to_string();
            }
        }
        if ipv4.is_empty() {
            ipv4 = String::from("недоступен");
        }
    }

    if ipv6.is_empty() {
        if let Ok(resp) = client.get("https://api6.ipify.org").send().await {
            if let Ok(text) = resp.text().await {
                ipv6 = text.trim().to_string();
            }
        }
        if ipv6.is_empty() {
            ipv6 = String::from("недоступен");
        }
    }

    let profile = config.active_profile();
    let profile_key = config.active_profile.clone();
    let profile_name = profile.map(|p| p.name.clone()).unwrap_or_else(|| "Default".to_string());

    let status = if proxy_env.is_some() {
        "🟢 ВКЛЮЧЕН".to_string()
    } else {
        "🔴 НАПРЯМУЮ (Прокси выключен)".to_string()
    };

    StatusInfo {
        status,
        profile_key,
        profile_name,
        active_proxy: proxy_env,
        ipv4,
        ipv6,
        location,
    }
}

/// Formats and prints network status either as human-readable text or formatted JSON.
pub async fn print_status(config: &AppConfig, as_json: bool) -> Result<()> {
    let info = get_status_info(config).await;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("----------------------------------------------------------");
        if let Some(ref p) = info.active_proxy {
            println!(
                "Статус:  {}",
                format!("🟢 ВКЛЮЧЕН [{} -> {}]", info.profile_name, p).green().bold()
            );
        } else {
            println!(
                "Статус:  {} (Профиль: {})",
                "🔴 НАПРЯМУЮ (Прокси выключен)".red().bold(),
                info.profile_name.yellow()
            );
        }
        println!("IPv4:    {}", info.ipv4.bold());
        println!("IPv6:    {}", info.ipv6.bold());
        println!("Локация: {}", info.location.cyan().bold());
        println!("----------------------------------------------------------");
    }

    Ok(())
}
