use crate::config::{AppConfig, I18n};
use anyhow::Result;
use colored::Colorize;
use std::process::Command;

/// Manages global Git proxy settings (`http.proxy` and `https.proxy`).
pub fn handle_git_proxy(mode: &str, config: &AppConfig, i18n: &I18n) -> Result<()> {
    match mode.to_lowercase().as_str() {
        "on" => {
            if let (Some(socks_url), Some(profile)) =
                (config.get_socks_url(), config.active_profile())
            {
                let _ = Command::new("git")
                    .args(["config", "--global", "http.proxy", &socks_url])
                    .status();
                let _ = Command::new("git")
                    .args(["config", "--global", "https.proxy", &socks_url])
                    .status();
                println!(
                    "⚙️ Git global proxy SET to {} ({}:{})",
                    profile.name.green().bold(),
                    profile.host,
                    profile.port
                );
            } else {
                eprintln!("{}", i18n.t("proxy_load_failed").red().bold());
            }
        }
        "off" => {
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "http.proxy"])
                .status();
            let _ = Command::new("git")
                .args(["config", "--global", "--unset", "https.proxy"])
                .status();
            println!("{}", "🛑 Git global proxy UNSET".yellow().bold());
        }
        _ => {
            let http = get_git_config("http.proxy");
            let https = get_git_config("https.proxy");

            println!(
                "{}",
                "=========================================================="
                    .cyan()
                    .bold()
            );
            println!("   🐙  {}", "GIT GLOBAL PROXY CONFIGURATION".white().bold());
            println!(
                "{}",
                "=========================================================="
                    .cyan()
                    .bold()
            );
            println!(
                "   • http.proxy  = {}",
                http.unwrap_or_else(|| "<none>".to_string()).yellow()
            );
            println!(
                "   • https.proxy = {}",
                https.unwrap_or_else(|| "<none>".to_string()).yellow()
            );
            println!(
                "{}",
                "=========================================================="
                    .cyan()
                    .bold()
            );
        }
    }
    Ok(())
}

fn get_git_config(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", key])
        .output()
        .ok()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}
