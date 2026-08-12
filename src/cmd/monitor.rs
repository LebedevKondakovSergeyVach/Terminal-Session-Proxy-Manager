use crate::cmd::profile::{benchmark_profiles, use_profile};
use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::time::Duration;

/// Monitors active proxy connection health and automatically fallbacks to fastest profile upon failure.
pub async fn run_monitor(config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!(
        "   🛡️  {}",
        "PROXY HEALTH MONITOR & AUTO-HEAL".white().bold()
    );
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message("Testing active proxy connection health...".to_string());
    pb.enable_steady_tick(Duration::from_millis(80));

    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("http_proxy"))
        .ok();
    let mut is_healthy = false;

    if let Some(ref p) = proxy_env {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .connect_timeout(Duration::from_secs(2));

        if let Ok(proxy) = reqwest::Proxy::all(p) {
            builder = builder.proxy(proxy);
        }

        if let Ok(client) = builder.build() {
            if let Ok(resp) = client
                .get("https://generativelanguage.googleapis.com")
                .send()
                .await
            {
                if resp.status().is_success() || resp.status().as_u16() == 404 {
                    is_healthy = true;
                }
            }
        }
    }

    pb.finish_and_clear();

    if is_healthy {
        let name = config
            .active_profile()
            .map(|pr| pr.name.as_str())
            .unwrap_or("Active");
        println!(
            "  • Status: {} ({})",
            "✅ Active proxy connection is HEALTHY".green().bold(),
            name.yellow()
        );
    } else {
        println!(
            "  • Status: {}",
            "❌ Active proxy failed or timed out!".red().bold()
        );
        println!("  • Action: Auto-healing by selecting fastest alternative profile...");

        let results = benchmark_profiles(config, i18n).await;
        if let Some((best_key, avg_ms, rate)) = results.first() {
            if *avg_ms < 9999 && *rate > 0.0 {
                let key = best_key.clone();
                use_profile(config, i18n, &key)?;
                println!(
                    "  • {}",
                    format!(
                        "🏆 Auto-healed: switched to profile '{}' (Ping: {} ms)",
                        key, avg_ms
                    )
                    .green()
                    .bold()
                );
                println!(
                    "{}",
                    "=========================================================="
                        .cyan()
                        .bold()
                );
                return Ok(());
            }
        }
        println!(
            "  • {}",
            "❌ Auto-heal failed: no alternative proxies are reachable."
                .red()
                .bold()
        );
    }

    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    Ok(())
}
