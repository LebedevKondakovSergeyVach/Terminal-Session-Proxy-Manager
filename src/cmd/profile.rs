use crate::config::{AppConfig, I18n, Profile};
use crate::error::ProxyError;
use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

/// How long a single benchmark probe may take before it counts as unreachable.
const BENCHMARK_TIMEOUT: Duration = Duration::from_secs(3);

/// Outcome of benchmarking one profile against every configured ping target.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkResult {
    /// Profile key in the config's `profiles` map.
    pub key: String,
    /// Mean latency of the probes that succeeded, or `None` if none did.
    ///
    /// Modelled as an option rather than a large sentinel value: the previous
    /// `9999` sentinel was compared with `<` in three places, so a genuinely
    /// slow proxy and an unreachable one were indistinguishable.
    pub avg_ms: Option<u128>,
    /// Share of ping targets that responded, 0.0–100.0.
    pub success_rate: f64,
}

impl BenchmarkResult {
    /// Whether this profile answered at least one probe.
    #[must_use]
    pub fn is_reachable(&self) -> bool {
        self.avg_ms.is_some() && self.success_rate > 0.0
    }

    /// Sort key placing reachable profiles first, fastest to slowest.
    fn ranking(&self) -> u128 {
        self.avg_ms.unwrap_or(u128::MAX)
    }
}

/// Displays all configured proxy profiles.
pub fn list_profiles(config: &AppConfig, i18n: &I18n) {
    println!("{}", i18n.t("profile_list_header").cyan().bold());
    for (key, prof) in &config.profiles {
        let active_mark = if key == &config.active_profile {
            format!(" {}", i18n.t("active_tag"))
                .green()
                .bold()
                .to_string()
        } else {
            String::new()
        };
        println!(
            "  • {:<12} — {} ({}:{}){}",
            key.yellow().bold(),
            prof.name,
            prof.host,
            prof.port,
            active_mark
        );
    }
}

/// Switches the active profile to `key`.
///
/// # Errors
/// Returns [`ProxyError::ProfileNotFound`] if no such profile exists. This
/// propagates to a non-zero exit status, which `proxy profile use "$p" ||
/// fallback` depends on; printing the error and returning `Ok` — the previous
/// behaviour — made every such script silently take the success branch.
pub fn use_profile(config: &mut AppConfig, i18n: &I18n, key: &str) -> Result<()> {
    if !config.profiles.contains_key(key) {
        return Err(ProxyError::ProfileNotFound(key.to_string()))
            .context(i18n.format("profile_not_found", &[key]));
    }

    config.active_profile = key.to_string();
    config.save()?;

    if let Some(p) = config.active_profile() {
        println!(
            "{} {} ({}:{})",
            i18n.t("profile_switched"),
            p.name.green().bold(),
            p.host,
            p.port
        );
    }
    Ok(())
}

/// Creates or updates a profile in the configuration.
///
/// # Errors
/// Returns an error if the resulting profile is invalid or cannot be saved.
pub fn set_profile(
    config: &mut AppConfig,
    i18n: &I18n,
    key: String,
    name: Option<String>,
    port: Option<u16>,
    host: Option<String>,
    protocol: String,
) -> Result<()> {
    let mut prof = config
        .profiles
        .get(&key)
        .cloned()
        .unwrap_or_else(|| Profile {
            name: key.clone(),
            host: "127.0.0.1".to_string(),
            port: 1080,
            protocol: "socks5".to_string(),
        });

    if let Some(n) = name {
        prof.name = n;
    }
    if let Some(p) = port {
        prof.port = p;
    }
    if let Some(h) = host {
        prof.host = h;
    }
    prof.protocol = protocol;

    // Validate before writing: a profile that cannot form a proxy URL would
    // otherwise be persisted and break every later command that loads it.
    prof.validate(&key)?;

    config.profiles.insert(key.clone(), prof.clone());
    config.active_profile = key.clone();
    config.save()?;

    println!(
        "{} {} ({}:{})",
        i18n.format("profile_saved", &[&key]),
        prof.name.green().bold(),
        prof.host,
        prof.port
    );
    Ok(())
}

/// Removes a profile from the configuration by key.
///
/// # Errors
/// Returns [`ProxyError::ProfileNotFound`] if no such profile exists.
pub fn remove_profile(config: &mut AppConfig, i18n: &I18n, key: &str) -> Result<()> {
    if config.profiles.remove(key).is_none() {
        return Err(ProxyError::ProfileNotFound(key.to_string()))
            .context(i18n.format("profile_not_found", &[key]));
    }

    // Removing the active profile would leave `active_profile` dangling, so
    // adopt whichever profile remains first.
    if config.active_profile == key
        && let Some((first_key, _)) = config.profiles.iter().next()
    {
        config.active_profile = first_key.clone();
    }

    config.save()?;
    println!("{}", i18n.format("profile_removed", &[key]));
    Ok(())
}

/// Interactive console profile selector using arrow keys.
///
/// # Errors
/// Returns an error if the selected profile cannot be saved.
pub fn select_profile_interactive(config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    let profile_keys: Vec<String> = config.profiles.keys().cloned().collect();
    if profile_keys.is_empty() {
        println!("{}", i18n.t("no_profiles").red());
        return Ok(());
    }

    let mut items = Vec::new();
    let mut default_idx = 0;

    for (idx, key) in profile_keys.iter().enumerate() {
        let prof = &config.profiles[key];
        let is_active = key == &config.active_profile;
        if is_active {
            default_idx = idx;
        }
        let mark = if is_active {
            format!(" [{}]", i18n.t("active_tag"))
        } else {
            String::new()
        };
        items.push(format!(
            "{:<12} — {} ({}:{}){}",
            key, prof.name, prof.host, prof.port, mark
        ));
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(i18n.t("prompt_choice"))
        .default(default_idx)
        .items(&items)
        .interact_opt();

    match selection {
        Ok(Some(choice)) => use_profile(config, i18n, &profile_keys[choice])?,
        // The user pressed Esc; leaving the active profile alone is correct.
        Ok(None) => {}
        // No usable TTY (piped or non-interactive): fall back to printing the
        // list so the command is still informative rather than failing.
        Err(_) => list_profiles(config, i18n),
    }

    Ok(())
}

/// Benchmarks every profile against the configured ping targets.
///
/// Results are sorted fastest first, with unreachable profiles last.
pub async fn benchmark_profiles(config: &AppConfig, i18n: &I18n) -> Vec<BenchmarkResult> {
    let pb = spinner(i18n.t("spinner_benchmark"));

    let mut results = Vec::new();
    for (key, prof) in &config.profiles {
        let proxy_url = format!("{}://{}:{}", prof.protocol, prof.host, prof.port);

        let probes = config.ping_targets.iter().map(|ep| {
            let proxy_url = proxy_url.clone();
            let target_url = ep.url.clone();
            tokio::spawn(async move { probe(&proxy_url, &target_url).await })
        });

        let latencies: Vec<u128> = join_all(probes)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        let total_targets = config.ping_targets.len();
        let avg_ms = if latencies.is_empty() {
            None
        } else {
            Some(latencies.iter().sum::<u128>() / latencies.len() as u128)
        };
        let success_rate = if total_targets == 0 {
            0.0
        } else {
            (latencies.len() as f64 / total_targets as f64) * 100.0
        };

        results.push(BenchmarkResult {
            key: key.clone(),
            avg_ms,
            success_rate,
        });
    }

    pb.finish_and_clear();
    results.sort_by_key(BenchmarkResult::ranking);
    results
}

/// Times a single request to `target_url` through `proxy_url`.
async fn probe(proxy_url: &str, target_url: &str) -> Option<u128> {
    let mut builder = reqwest::Client::builder()
        .timeout(BENCHMARK_TIMEOUT)
        .connect_timeout(BENCHMARK_TIMEOUT);

    if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
        builder = builder.proxy(proxy);
    }

    let client = builder.build().ok()?;
    let start = Instant::now();
    client
        .get(target_url)
        .send()
        .await
        .ok()
        .map(|_| start.elapsed().as_millis())
}

/// Builds the shared spinner style used by the long-running probes.
pub(crate) fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Prints the shared section rule used by the report-style commands.
pub(crate) fn rule() {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
}

/// Runs the benchmark and displays a ranking table.
///
/// # Errors
/// Currently infallible; returns `Result` for symmetry with the other commands.
pub async fn run_benchmark(config: &AppConfig, i18n: &I18n) -> Result<()> {
    rule();
    println!("   🚀  {}", i18n.t("benchmark_header").white().bold());
    rule();

    let results = benchmark_profiles(config, i18n).await;

    println!(
        "{:<14} {:<18} {:<12} {:<10}",
        i18n.t("col_profile"),
        i18n.t("col_name"),
        i18n.t("col_ping"),
        i18n.t("col_availability")
    );
    println!("----------------------------------------------------------");

    for result in results {
        let Some(prof) = config.profiles.get(&result.key) else {
            continue;
        };
        let ping_str = match result.avg_ms {
            Some(ms) => format!("{ms} ms").green().bold().to_string(),
            None => i18n.t("timeout_label").red().bold().to_string(),
        };

        println!(
            "{:<14} {:<18} {:<12} {:<10}",
            result.key.yellow().bold(),
            prof.name,
            ping_str,
            format!("{:.0}%", result.success_rate).cyan()
        );
    }
    rule();

    Ok(())
}

/// Finds the fastest reachable profile and sets it as active.
///
/// # Errors
/// Returns an error if the winning profile cannot be saved.
pub async fn select_best_profile(config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    let results = benchmark_profiles(config, i18n).await;

    if let Some(best) = results.first().filter(|r| r.is_reachable()) {
        let (key, avg_ms, rate) = (best.key.clone(), best.avg_ms, best.success_rate);
        use_profile(config, i18n, &key)?;
        println!(
            "{} {} (Ping: {} ms, Rate: {:.0}%)",
            i18n.t("best_found"),
            key.yellow().bold(),
            avg_ms.unwrap_or_default().to_string().green().bold(),
            rate
        );
        return Ok(());
    }

    println!("{}", i18n.t("best_all_failed").red().bold());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(key: &str, avg_ms: Option<u128>, success_rate: f64) -> BenchmarkResult {
        BenchmarkResult {
            key: key.to_string(),
            avg_ms,
            success_rate,
        }
    }

    #[test]
    fn a_profile_that_answered_nothing_is_unreachable() {
        assert!(!result("a", None, 0.0).is_reachable());
    }

    #[test]
    fn a_zero_success_rate_is_unreachable_even_with_a_latency() {
        assert!(!result("a", Some(10), 0.0).is_reachable());
    }

    #[test]
    fn a_responding_profile_is_reachable() {
        assert!(result("a", Some(120), 60.0).is_reachable());
    }

    #[test]
    fn ranking_sorts_fastest_first_and_unreachable_last() {
        let mut results = [
            result("slow", Some(900), 100.0),
            result("dead", None, 0.0),
            result("fast", Some(40), 100.0),
        ];

        results.sort_by_key(BenchmarkResult::ranking);

        let order: Vec<&str> = results.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(order, ["fast", "slow", "dead"]);
    }

    #[test]
    fn a_very_slow_profile_still_outranks_an_unreachable_one() {
        // The old `9999` sentinel made these two indistinguishable.
        let mut results = [
            result("dead", None, 0.0),
            result("slow", Some(60_000), 20.0),
        ];

        results.sort_by_key(BenchmarkResult::ranking);

        assert_eq!(results[0].key, "slow");
        assert!(results[0].is_reachable());
    }
}
