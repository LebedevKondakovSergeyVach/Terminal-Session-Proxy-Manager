use crate::config::{AppConfig, Profile};
use anyhow::Result;
use colored::Colorize;
use futures::future::join_all;
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Displays all configured proxy profiles.
pub fn list_profiles(config: &AppConfig) {
    println!("{}", "📋 Список профилей в config.json:".cyan().bold());
    for (key, prof) in &config.profiles {
        let active_mark = if key == &config.active_profile {
            " (активный)".green().bold().to_string()
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
pub fn use_profile(config: &mut AppConfig, key: &str) -> Result<()> {
    if config.profiles.contains_key(key) {
        config.active_profile = key.to_string();
        config.save()?;
        if let Some(p) = config.active_profile() {
            println!("⚙️ Переключено на профиль: {} ({}:{})", p.name.green().bold(), p.host, p.port);
        }
    } else {
        eprintln!("{}", format!("❌ Профиль '{}' не найден в config.json!", key).red().bold());
    }
    Ok(())
}

/// Creates or updates a profile in configuration.
pub fn set_profile(
    config: &mut AppConfig,
    key: String,
    name: Option<String>,
    port: Option<u16>,
    host: Option<String>,
    protocol: String,
) -> Result<()> {
    let mut prof = config.profiles.get(&key).cloned().unwrap_or_else(|| Profile {
        name: key.clone(),
        host: "127.0.0.1".to_string(),
        port: 2080,
        protocol: "socks5".to_string(),
    });

    if let Some(n) = name { prof.name = n; }
    if let Some(p) = port { prof.port = p; }
    if let Some(h) = host { prof.host = h; }
    prof.protocol = protocol;

    config.profiles.insert(key.clone(), prof.clone());
    config.active_profile = key.clone();
    config.save()?;

    println!("⚙️ Профиль '{}' сохранен: {} ({}:{})", key, prof.name.green().bold(), prof.host, prof.port);
    Ok(())
}

/// Removes a profile from configuration by key.
pub fn remove_profile(config: &mut AppConfig, key: &str) -> Result<()> {
    if config.profiles.remove(key).is_some() {
        if config.active_profile == key {
            if let Some((first_key, _)) = config.profiles.iter().next() {
                config.active_profile = first_key.clone();
            }
        }
        config.save()?;
        println!("🗑️ Профиль '{}' удален", key);
    } else {
        eprintln!("{}", format!("❌ Профиль '{}' не найден!", key).red().bold());
    }
    Ok(())
}

/// Interactive console profile selector.
pub fn select_profile_interactive(config: &mut AppConfig) -> Result<()> {
    println!("{}", "==========================================================".cyan().bold());
    println!("      ⚡ {}", "ИНТЕРАКТИВНЫЙ ВЫБОР ПРОФИЛЯ ПРОКСИ".white().bold());
    println!("{}", "==========================================================".cyan().bold());

    let profile_keys: Vec<String> = config.profiles.keys().cloned().collect();
    if profile_keys.is_empty() {
        println!("{}", "❌ В конфиге нет доступных профилей!".red());
        return Ok(());
    }

    for (idx, key) in profile_keys.iter().enumerate() {
        let prof = &config.profiles[key];
        let is_active = key == &config.active_profile;
        let mark = if is_active { " [активный]".green().bold().to_string() } else { "".to_string() };
        println!(
            "  [{}] {:<12} — {} ({}:{}){}",
            (idx + 1).to_string().yellow().bold(),
            key.bold(),
            prof.name,
            prof.host,
            prof.port,
            mark
        );
    }
    println!("{}", "==========================================================".cyan().bold());
    print!("Выберите номер профиля (1-{}): ", profile_keys.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice >= 1 && choice <= profile_keys.len() {
            let selected_key = &profile_keys[choice - 1];
            return use_profile(config, selected_key);
        }
    }

    println!("{}", "❌ Некорректный выбор".red());
    Ok(())
}

/// Benchmarks all profiles against ping targets and returns (key, avg_ms, success_rate).
pub async fn benchmark_profiles(config: &AppConfig) -> Vec<(String, u128, f64)> {
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

        for r in task_results {
            if let Ok(Some(ms)) = r {
                total_ms += ms;
                success_count += 1;
            }
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

    results.sort_by_key(|r| r.1);
    results
}

/// Runs benchmark and displays ranking table.
pub async fn run_benchmark(config: &AppConfig) -> Result<()> {
    println!("{}", "==========================================================".cyan().bold());
    println!("   🚀  {}", "БЕНЧМАРК И ТЕСТ СКОРОСТИ ВСЕХ ПРОФИЛЕЙ".white().bold());
    println!("{}", "==========================================================".cyan().bold());
    println!("Выполняется измерение задержки через каждый профиль...");
    println!();

    let results = benchmark_profiles(config).await;

    println!("{:<14} {:<18} {:<12} {:<10}", "ПРОФИЛЬ", "НАЗВАНИЕ", "СР. ПИНГ", "ДОСТУПНОСТЬ");
    println!("----------------------------------------------------------");

    for (key, avg_ms, rate) in results {
        let prof = &config.profiles[&key];
        let ping_str = if avg_ms < 9999 {
            format!("{} ms", avg_ms).green().bold().to_string()
        } else {
            "Таймаут".red().bold().to_string()
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
    println!("{}", "==========================================================".cyan().bold());

    Ok(())
}

/// Finds the fastest profile and sets it as active.
pub async fn select_best_profile(config: &mut AppConfig) -> Result<()> {
    println!("⚡ Автоматический поиск и выбор самого быстрого прокси...");
    let results = benchmark_profiles(config).await;

    if let Some((best_key, avg_ms, rate)) = results.first() {
        if *avg_ms < 9999 && *rate > 0.0 {
            let key = best_key.clone();
            use_profile(config, &key)?;
            println!(
                "🏆 Самый быстрый профиль: {} (Пинг: {} ms, Успешность: {:.0}%)",
                key.yellow().bold(),
                avg_ms.to_string().green().bold(),
                rate
            );
            return Ok(());
        }
    }

    println!("{}", "❌ Все прокси-профили недоступны или превысили таймаут!".red().bold());
    Ok(())
}
