use crate::config::AppConfig;
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::net::TcpStream;
use std::time::Duration;

/// Runs diagnostic tests on local sockets, session env vars, and HTTP endpoints.
pub async fn run_diagnose(config: &AppConfig) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!(
        "      🔍 {}",
        "РАСШИРЕННАЯ ДИАГНОСТИКА СЕТИ И ПРОКСИ".white().bold()
    );
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );

    let profile = config.active_profile();
    let name = profile.map(|p| p.name.as_str()).unwrap_or("Default");
    let host = profile.map(|p| p.host.as_str()).unwrap_or("127.0.0.1");
    let port = profile.map(|p| p.port).unwrap_or(2080);

    println!(
        "Активный профиль : {} ({})",
        name.yellow().bold(),
        config.active_profile
    );
    println!("Активный хост/порт: {}:{}", host, port);
    println!();

    print!("1. Проверка локального сокета ({}:{}): ", host, port);
    let socket_addr = format!("{}:{}", host, port);
    if let Ok(addr) = socket_addr.parse() {
        if TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok() {
            println!("{}", "✅ Порт открыт (клиент активен)".green().bold());
        } else {
            println!(
                "{}",
                format!("❌ Порт закрыт! Проверьте клиент {}", name)
                    .red()
                    .bold()
            );
        }
    } else {
        println!("{}", "❌ Некорректный адрес хоста/порта".red().bold());
    }

    println!();
    println!("2. Переменные окружения текущей сессии:");
    println!(
        "   • http_proxy  = {}",
        env::var("http_proxy").unwrap_or_else(|_| "<не задан>".to_string())
    );
    println!(
        "   • https_proxy = {}",
        env::var("https_proxy").unwrap_or_else(|_| "<не задан>".to_string())
    );
    println!(
        "   • ALL_PROXY   = {}",
        env::var("ALL_PROXY").unwrap_or_else(|_| "<не задан>".to_string())
    );
    println!(
        "   • GRADLE_OPTS = {}",
        env::var("GRADLE_OPTS").unwrap_or_else(|_| "<не задан>".to_string())
    );

    println!();
    println!("3. Доступность критичных эндпоинтов (таймаут 3с):");

    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("http_proxy"))
        .ok();
    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(3));
    if let Some(ref p) = proxy_env {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            builder = builder.proxy(proxy);
        }
    }
    let client = builder.build().unwrap_or_default();

    for ep in &config.diagnose_endpoints {
        print!("   • {} ({}): ", ep.name, ep.url);
        match client.get(&ep.url).send().await {
            Ok(resp) => println!(
                "{}",
                format!("✅ Доступен (HTTP {})", resp.status().as_u16())
                    .green()
                    .bold()
            ),
            Err(_) => println!("{}", "❌ Недоступен (таймаут/блокировка)".red().bold()),
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
