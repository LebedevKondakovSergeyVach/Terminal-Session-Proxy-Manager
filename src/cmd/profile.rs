use crate::config::{AppConfig, I18n, Profile};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

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
            "".to_string()
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

/// Switches active profile to specified key.
pub fn use_profile(config: &mut AppConfig, i18n: &I18n, key: &str) -> Result<()> {
    if config.profiles.contains_key(key) {
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
    } else {
        let err_msg = i18n.t("profile_not_found").replace("{}", key);
        eprintln!("{}", err_msg.red().bold());
    }
    Ok(())
}

/// Creates or updates a profile in configuration.
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
            port: 2080,
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

    config.profiles.insert(key.clone(), prof.clone());
    config.active_profile = key.clone();
    config.save()?;

    let msg = i18n.t("profile_saved").replace("{}", &key);
    println!(
        "{} {} ({}:{})",
        msg,
        prof.name.green().bold(),
        prof.host,
        prof.port
    );
    Ok(())
}

/// Removes a profile from configuration by key.
pub fn remove_profile(config: &mut AppConfig, i18n: &I18n, key: &str) -> Result<()> {
    if config.profiles.remove(key).is_some() {
        if config.active_profile == key {
            if let Some((first_key, _)) = config.profiles.iter().next() {
                config.active_profile = first_key.clone();
            }
        }
        config.save()?;
        let msg = i18n.t("profile_removed").replace("{}", key);
        println!("{}", msg);
    } else {
        let err_msg = i18n.t("profile_not_found").replace("{}", key);
        eprintln!("{}", err_msg.red().bold());
    }
    Ok(())
}

/// Interactive console profile selector using arrow keys.
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
            "".to_string()
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
        .interact_opt()?;

    if let Some(choice) = selection {
        let selected_key = &profile_keys[choice];
        use_profile(config, i18n, selected_key)?;
    }

    Ok(())
}

/// Benchmarks all profiles against ping targets and returns (key, avg_ms, success_rate).
pub async fn benchmark_profiles(config: &AppConfig, i18n: &I18n) -> Vec<(String, u128, f64)> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(i18n.t("spinner_benchmark").to_string());
    pb.enable_steady_tick(Duration::from_millis(80));

    let mut results = Vec::new();
    let timeout = Duration::from_millis(3000);

    for (key, prof) in &config.profiles {
        let proxy_url = format!("{}://{}:{}", prof.protocol, prof.host, prof.port);
        let mut tasks = Vec::new();

        for ep in &config.ping_targets {
            let p_url = proxy_url.clone();
            let target_url = ep.url.clone();

            tasks.push(tokio::spawn(async move {
                let mut builder = reqwest::Client::builder()
                    .timeout(timeout)
                    .connect_timeout(timeout);

                if let Ok(proxy) = reqwest::Proxy::all(&p_url) {
                    builder = builder.proxy(proxy);
                }

                let client = match builder.build() {
                    Ok(c) => c,
                    Err(_) => return None,
                };

                let start = Instant::now();
                if client.get(&target_url).send().await.is_ok() {
                    Some(start.elapsed().as_millis())
                } else {
                    None
                }
            }));
        }

        let task_results = join_all(tasks).await;
        let mut total_ms = 0u128;
        let mut success_count = 0usize;
        let total_targets = config.ping_targets.len();

        for ms in task_results.into_iter().flatten().flatten() {
            total_ms += ms;
            success_count += 1;
        }

        let avg_ms = if success_count > 0 {
            total_ms / (success_count as u128)
        } else {
            9999
        };

        let rate = if total_targets > 0 {
            (success_count as f64 / total_targets as f64) * 100.0
        } else {
            0.0
        };

        results.push((key.clone(), avg_ms, rate));
    }

    pb.finish_and_clear();
    results.sort_by_key(|r| r.1);
    results
}

/// Runs benchmark and displays ranking table.
pub async fn run_benchmark(config: &AppConfig, i18n: &I18n) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("   🚀  {}", i18n.t("benchmark_header").white().bold());
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );

    let results = benchmark_profiles(config, i18n).await;

    println!(
        "{:<14} {:<18} {:<12} {:<10}",
        i18n.t("col_profile"),
        i18n.t("col_name"),
        i18n.t("col_ping"),
        i18n.t("col_availability")
    );
    println!("----------------------------------------------------------");

    for (key, avg_ms, rate) in results {
        let prof = &config.profiles[&key];
        let ping_str = if avg_ms < 9999 {
            format!("{} ms", avg_ms).green().bold().to_string()
        } else {
            i18n.t("timeout_label").red().bold().to_string()
        };

        let rate_str = format!("{:.0}%", rate);
        println!(
            "{:<14} {:<18} {:<12} {:<10}",
            key.yellow().bold(),
            prof.name,
            ping_str,
            rate_str.cyan()
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

/// Finds the fastest profile and sets it as active.
pub async fn select_best_profile(config: &mut AppConfig, i18n: &I18n) -> Result<()> {
    let results = benchmark_profiles(config, i18n).await;

    if let Some((best_key, avg_ms, rate)) = results.first() {
        if *avg_ms < 9999 && *rate > 0.0 {
            let key = best_key.clone();
            use_profile(config, i18n, &key)?;
            println!(
                "{} {} (Ping: {} ms, Rate: {:.0}%)",
                i18n.t("best_found"),
                key.yellow().bold(),
                avg_ms.to_string().green().bold(),
                rate
            );
            return Ok(());
        }
    }

    println!("{}", i18n.t("best_all_failed").red().bold());
    Ok(())
}
