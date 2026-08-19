use crate::cmd::profile::rule;
use crate::config::{AppConfig, I18n};
use anyhow::{Result, anyhow};
use colored::Colorize;
use std::process::Command;

/// Manages the global Git proxy settings (`http.proxy` and `https.proxy`).
///
/// # Errors
/// Returns an error if `git` is missing, fails, or no profile is active.
pub fn handle_git_proxy(mode: &crate::cli::GitMode, config: &AppConfig, i18n: &I18n) -> Result<()> {
    use crate::cli::GitMode;

    match mode {
        GitMode::On => {
            let (Some(proxy_url), Some(profile)) =
                (config.get_socks_url(), config.active_profile())
            else {
                return Err(anyhow!("{}", i18n.t("proxy_load_failed")));
            };

            set_git_config("http.proxy", &proxy_url)?;
            set_git_config("https.proxy", &proxy_url)?;

            println!(
                "⚙️  {} {} ({}:{})",
                i18n.t("git_proxy_set"),
                profile.name.green().bold(),
                profile.host,
                profile.port
            );
        }
        GitMode::Off => {
            // `--unset` exits 5 when the key is already absent, which is the
            // desired end state rather than a failure.
            unset_git_config("http.proxy")?;
            unset_git_config("https.proxy")?;
            println!("🛑 {}", i18n.t("git_proxy_unset").yellow().bold());
        }
        GitMode::Status => {
            let none = i18n.t("none_label");
            rule();
            println!("   🐙  {}", i18n.t("git_header").white().bold());
            rule();
            for key in ["http.proxy", "https.proxy"] {
                println!(
                    "   • {key:<12} = {}",
                    get_git_config(key)
                        .unwrap_or_else(|| none.to_string())
                        .yellow()
                );
            }
            rule();
        }
    }
    Ok(())
}

/// Runs `git config --global <key> <value>`, surfacing failures.
///
/// The previous implementation discarded the exit status, so a missing `git`
/// binary still printed "Git global proxy SET".
fn set_git_config(key: &str, value: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["config", "--global", key, value])
        .status()
        .map_err(|e| anyhow!("could not run git: {e}"))?;

    if !status.success() {
        return Err(anyhow!("git config --global {key} failed with {status}"));
    }
    Ok(())
}

/// Runs `git config --global --unset <key>`, tolerating an already-unset key.
fn unset_git_config(key: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["config", "--global", "--unset", key])
        .status()
        .map_err(|e| anyhow!("could not run git: {e}"))?;

    // Exit code 5 means "the key was not set", which is what we wanted anyway.
    match status.code() {
        Some(0 | 5) => Ok(()),
        _ => Err(anyhow!(
            "git config --global --unset {key} failed with {status}"
        )),
    }
}

fn get_git_config(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", key])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}
