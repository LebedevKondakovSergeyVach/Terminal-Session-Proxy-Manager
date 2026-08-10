use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::net::TcpStream;
use std::time::Duration;

/// Runs diagnostic tests on local sockets, session env vars, and HTTP endpoints.
pub async fn run_diagnose(config: &AppConfig, i18n: &I18n) -> Result<()> {
    println!(
        "{}",
        "=========================================================="
            .cyan()
            .bold()
    );
    println!("      🔍 {}", i18n.t("diagnose_header").white().bold());
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
        "{} {} ({})",
        i18n.t("profile_label"),
        name.yellow().bold(),
        config.active_profile
    );
    println!("{} {}:{}", i18n.t("active_host_port"), host, port);
    println!();

    print!("{} ({}:{}): ", i18n.t("socket_check"), host, port);
    let socket_addr = format!("{}:{}", host, port);
    if let Ok(addr) = socket_addr.parse() {
        if TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok() {
            println!("{}", i18n.t("port_open").green().bold());
        } else {
            println!(
                "{}",
                format!("{} {}", i18n.t("port_closed"), name).red().bold()
            );
        }
    } else {
        println!("{}", i18n.t("invalid_address").red().bold());
    }

    println!();
    println!("{}", i18n.t("session_env_vars"));
    println!(
        "   • http_proxy  = {}",
        env::var("http_proxy").unwrap_or_else(|_| "<none>".to_string())
    );
    println!(
        "   • https_proxy = {}",
        env::var("https_proxy").unwrap_or_else(|_| "<none>".to_string())
    );
    println!(
        "   • ALL_PROXY   = {}",
        env::var("ALL_PROXY").unwrap_or_else(|_| "<none>".to_string())
    );
    println!(
        "   • GRADLE_OPTS = {}",
        env::var("GRADLE_OPTS").unwrap_or_else(|_| "<none>".to_string())
    );

    println!();
    println!("{}", i18n.t("critical_endpoints"));

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
            Ok(resp) => {
                let code_str = resp.status().as_u16().to_string();
                let msg = i18n.t("endpoint_accessible").replace("{}", &code_str);
                println!("{}", msg.green().bold());
            }
            Err(_) => println!("{}", i18n.t("endpoint_unreachable").red().bold()),
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
