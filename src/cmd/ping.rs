use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::time::{Duration, Instant};

/// Probes configured ping targets in parallel and displays connection latency.
pub async fn run_ping(config: &AppConfig, i18n: &I18n, timeout_ms: u64) -> Result<()> {
    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("http_proxy"))
        .ok();

    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("   ⚡  {}", i18n.t("ping_header").white().bold());
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );

    if let Some(ref p) = proxy_env {
        let name = config
            .active_profile()
            .map(|pr| pr.name.as_str())
            .unwrap_or("Active");
        println!(
            "{} {} ({})",
            i18n.t("active_socket"),
            p.yellow().bold(),
            name
        );
    } else {
        println!("{}", i18n.t("direct_mode").red().bold());
    }
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(i18n.t("spinner_ping").to_string());
    pb.enable_steady_tick(Duration::from_millis(80));

    let mut tasks = Vec::new();
    let timeout = Duration::from_millis(timeout_ms);

    for ep in config.ping_targets.clone() {
        let proxy_str = proxy_env.clone();
        tasks.push(tokio::spawn(async move {
            let mut builder = reqwest::Client::builder()
                .timeout(timeout)
                .connect_timeout(timeout);

            if let Some(ref p) = proxy_str {
                if let Ok(proxy) = reqwest::Proxy::all(p) {
                    builder = builder.proxy(proxy);
                }
            }

            let client = match builder.build() {
                Ok(c) => c,
                Err(_) => return (ep.name, None, 0),
            };

            let start = Instant::now();
            let res = client.get(&ep.url).send().await;
            let elapsed = start.elapsed().as_millis();

            match res {
                Ok(resp) => (ep.name, Some(resp.status().as_u16()), elapsed),
                Err(_) => (ep.name, None, 0),
            }
        }));
    }

    let results = join_all(tasks).await;
    pb.finish_and_clear();

    for (name, status, elapsed) in results.into_iter().flatten() {
        if let Some(code) = status {
            println!(
                "  • {:<18} — {} [HTTP {}]",
                name.white().bold(),
                format!("✅ OK ({} ms)", elapsed).green().bold(),
                code
            );
        } else {
            println!(
                "  • {:<18} — {}",
                name.white().bold(),
                i18n.t("error_timeout").red().bold()
            );
        }
    }

    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    Ok(())
}
