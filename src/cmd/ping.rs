use crate::config::AppConfig;
use anyhow::Result;
use colored::Colorize;
use futures::future::join_all;
use std::env;
use std::time::{Duration, Instant};

/// Probes configured ping targets in parallel and displays connection latency.
pub async fn run_ping(config: &AppConfig, timeout_ms: u64) -> Result<()> {
    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("http_proxy"))
        .ok();

    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!(
        "   ⚡  {}",
        "ЗАМЕР ЗАДЕРЖКИ (PING) СЕРВИСОВ ЧЕРЕЗ ПРОКСИ".white().bold()
    );
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
        println!("Активный сокет: {} ({})", p.yellow().bold(), name);
    } else {
        println!(
            "Режим: {}",
            "🔴 ПРЯМОЕ СОЕДИНЕНИЕ (Без прокси)".red().bold()
        );
    }
    println!();

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
                "❌ Ошибка / Таймаут".red().bold()
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
