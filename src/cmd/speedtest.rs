use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::time::{Duration, Instant};

/// Measures real download throughput (MB/s) through configured active proxy.
pub async fn run_speedtest(_config: &AppConfig, _i18n: &I18n) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!(
        "   🚀  {}",
        "BANDWIDTH THROUGHPUT SPEED TEST".white().bold()
    );
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );

    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("http_proxy"))
        .ok();
    let test_url = "https://speed.cloudflare.com/__down?bytes=2097152";

    let pb = ProgressBar::new(2_097_152);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan.bold} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5));

    if let Some(ref p) = proxy_env {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            builder = builder.proxy(proxy);
        }
    }

    let client = builder.build()?;
    let start = Instant::now();

    let mut response = match client.get(test_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            pb.finish_and_clear();
            println!("{}", format!("❌ Speedtest failed: {}", e).red().bold());
            return Ok(());
        }
    };

    let mut downloaded = 0u64;
    while let Ok(Some(chunk)) = response.chunk().await {
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    let elapsed = start.elapsed().as_secs_f64();
    pb.finish_and_clear();

    if downloaded > 0 && elapsed > 0.0 {
        let speed_mbps = (downloaded as f64 * 8.0) / (elapsed * 1_000_000.0);
        let speed_mbs = (downloaded as f64) / (elapsed * 1_048_576.0);

        println!(
            "  • Payload Downloaded : {} MB",
            format!("{:.2}", downloaded as f64 / 1_048_576.0)
                .yellow()
                .bold()
        );
        println!(
            "  • Time Elapsed       : {} s",
            format!("{:.2}", elapsed).cyan().bold()
        );
        println!(
            "  • Download Speed     : {} ({})",
            format!("{:.2} MB/s", speed_mbs).green().bold(),
            format!("{:.2} Mbps", speed_mbps).bold()
        );
    } else {
        println!("{}", "❌ Unable to complete speed test.".red().bold());
    }

    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    Ok(())
}
