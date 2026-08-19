use crate::cmd::profile::{benchmark_profiles, rule, spinner, use_profile};
use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::time::Duration;

/// Monitors the active proxy and falls back to the fastest reachable profile.
///
/// # Errors
/// Returns an error if the replacement profile cannot be saved.
pub async fn run_monitor(config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    rule();
    println!("   🛡️  {}", i18n.t("monitor_header").white().bold());
    rule();

    let pb = spinner(i18n.t("spinner_monitor"));
    let is_healthy = check_active_proxy(config).await;
    pb.finish_and_clear();

    if is_healthy {
        let name = config
            .active_profile()
            .map_or(i18n.t("unknown"), |p| p.name.as_str());
        println!(
            "  • {} {} ({})",
            i18n.t("monitor_status_label"),
            i18n.t("monitor_healthy").green().bold(),
            name.yellow()
        );
        rule();
        return Ok(());
    }

    println!(
        "  • {} {}",
        i18n.t("monitor_status_label"),
        i18n.t("monitor_unhealthy").red().bold()
    );
    println!("  • {}", i18n.t("monitor_healing"));

    let results = benchmark_profiles(config, i18n).await;
    if let Some(best) = results.first().filter(|r| r.is_reachable()) {
        let (key, avg_ms) = (best.key.clone(), best.avg_ms.unwrap_or_default());
        use_profile(config, i18n, &key)?;
        println!(
            "  • {}",
            i18n.format("monitor_healed", &[&key, &avg_ms.to_string()])
                .green()
                .bold()
        );
    } else {
        println!("  • {}", i18n.t("monitor_heal_failed").red().bold());
    }

    rule();
    Ok(())
}

/// Probes the configured health-check URL through the session's proxy.
///
/// Reads the proxy from the environment rather than the config because the
/// point is to test what the shell is *actually* using right now, which may
/// differ from the saved active profile.
async fn check_active_proxy(config: &AppConfig) -> bool {
    let Some(proxy_url) = env::var("ALL_PROXY")
        .or_else(|_| env::var("all_proxy"))
        .or_else(|_| env::var("http_proxy"))
        .ok()
        .filter(|p| !p.is_empty())
    else {
        return false;
    };

    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2));

    if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
        builder = builder.proxy(proxy);
    }

    let Ok(client) = builder.build() else {
        return false;
    };

    // Any non-server-error response proves the tunnel carried the request.
    // The previous check special-cased a 404 from an API root, which broke as
    // soon as that endpoint changed; `health_check_url` now defaults to a
    // dedicated 204 endpoint and is configurable.
    match client.get(&config.health_check_url).send().await {
        Ok(resp) => !resp.status().is_server_error(),
        Err(_) => false,
    }
}
