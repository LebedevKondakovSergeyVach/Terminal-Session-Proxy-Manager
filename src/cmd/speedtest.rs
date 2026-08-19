use crate::cmd::profile::rule;
use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::time::{Duration, Instant};

/// Bytes assumed for the progress bar when the server sends no `Content-Length`.
const ASSUMED_PAYLOAD_BYTES: u64 = 2 * 1024 * 1024;

/// Measures real download throughput through the session's proxy.
///
/// # Errors
/// Returns an error if the HTTP client cannot be constructed.
pub async fn run_speedtest(config: &AppConfig, i18n: &I18n) -> Result<()> {
    rule();
    println!("   🚀  {}", i18n.t("speedtest_header").white().bold());
    rule();

    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("all_proxy"))
        .or_else(|_| env::var("http_proxy"))
        .ok()
        .filter(|p| !p.is_empty());

    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(5));

    if let Some(ref p) = proxy_env
        && let Ok(proxy) = reqwest::Proxy::all(p)
    {
        builder = builder.proxy(proxy);
    }

    let client = builder.build()?;
    let start = Instant::now();

    let mut response = match client.get(&config.speedtest_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            println!(
                "{}",
                i18n.format("speedtest_failed", &[&e.to_string()])
                    .red()
                    .bold()
            );
            rule();
            return Ok(());
        }
    };

    // Size the bar from the response rather than assuming the URL's query
    // string still says `bytes=2097152` — `speedtest_url` is user-configurable.
    let total_bytes = response.content_length().unwrap_or(ASSUMED_PAYLOAD_BYTES);
    let pb = ProgressBar::new(total_bytes);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan.bold} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );

    let mut downloaded = 0u64;
    while let Ok(Some(chunk)) = response.chunk().await {
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    let elapsed = start.elapsed().as_secs_f64();
    pb.finish_and_clear();

    if downloaded == 0 || elapsed <= 0.0 {
        println!("{}", i18n.t("speedtest_incomplete").red().bold());
        rule();
        return Ok(());
    }

    #[allow(clippy::cast_precision_loss)]
    let bytes = downloaded as f64;
    let speed_mbps = (bytes * 8.0) / (elapsed * 1_000_000.0);
    let speed_mbs = bytes / (elapsed * 1_048_576.0);

    println!(
        "  • {:<22}: {} MB",
        i18n.t("speedtest_payload"),
        format!("{:.2}", bytes / 1_048_576.0).yellow().bold()
    );
    println!(
        "  • {:<22}: {} s",
        i18n.t("speedtest_elapsed"),
        format!("{elapsed:.2}").cyan().bold()
    );
    println!(
        "  • {:<22}: {} ({})",
        i18n.t("speedtest_speed"),
        format!("{speed_mbs:.2} MB/s").green().bold(),
        format!("{speed_mbps:.2} Mbps").bold()
    );
    rule();

    Ok(())
}
