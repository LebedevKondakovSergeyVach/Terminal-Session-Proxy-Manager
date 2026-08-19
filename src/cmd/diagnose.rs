use crate::cmd::profile::{rule, spinner};
use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::net::TcpStream;
use std::time::Duration;

/// Runs diagnostic tests on local sockets, session env vars, and HTTP endpoints.
pub async fn run_diagnose(config: &AppConfig, i18n: &I18n) -> Result<()> {
    rule();
    println!("   🔍 {}", i18n.t("diagnose_header").white().bold());
    rule();

    let pb = spinner(i18n.t("spinner_diagnose"));

    let profile = config.active_profile();
    let name = profile.map(|p| p.name.as_str()).unwrap_or("Default");
    let host = profile.map(|p| p.host.as_str()).unwrap_or("127.0.0.1");
    let port = profile.map(|p| p.port).unwrap_or(2080);

    let socket_addr = format!("{host}:{port}");
    let socket_ok = if let Ok(addr) = socket_addr.parse() {
        TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok()
    } else {
        false
    };

    let proxy_env = env::var("ALL_PROXY")
        .or_else(|_| env::var("all_proxy"))
        .or_else(|_| env::var("http_proxy"))
        .ok()
        .filter(|p| !p.is_empty());
    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(3));
    if let Some(ref p) = proxy_env {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            builder = builder.proxy(proxy);
        }
    }
    let client = builder.build().unwrap_or_default();

    let mut endpoint_results = Vec::new();
    for ep in &config.diagnose_endpoints {
        let res = client.get(&ep.url).send().await;
        endpoint_results.push((
            ep.name.clone(),
            ep.url.clone(),
            res.map(|r| r.status().as_u16()).ok(),
        ));
    }

    pb.finish_and_clear();

    println!(
        "{} {} ({})",
        i18n.t("profile_label"),
        name.yellow().bold(),
        config.active_profile
    );
    println!("{} {}:{}", i18n.t("active_host_port"), host, port);
    println!();

    print!("{} ({}:{}): ", i18n.t("socket_check"), host, port);
    if socket_ok {
        println!("{}", i18n.t("port_open").green().bold());
    } else {
        println!(
            "{}",
            format!("{} {}", i18n.t("port_closed"), name).red().bold()
        );
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

    for (ep_name, ep_url, status_code) in endpoint_results {
        print!("   • {ep_name} ({ep_url}): ");
        if let Some(code) = status_code {
            let code_str = code.to_string();
            let msg = i18n.t("endpoint_accessible").replace("{}", &code_str);
            println!("{}", msg.green().bold());
        } else {
            println!("{}", i18n.t("endpoint_unreachable").red().bold());
        }
    }

    rule();
    Ok(())
}
