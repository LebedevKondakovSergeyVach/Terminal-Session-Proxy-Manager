#![warn(missing_docs)]
//! Proxy CLI main binary

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use colored::Colorize;
use std::env;
use std::process::Command;
use terminal_session_proxy_manager::cli::{
    Cli, Commands, ConfigCommands, ProfileCommands, SettingsCommands,
};
use terminal_session_proxy_manager::cmd::{
    completions, dash, diagnose, export_cmd, git_cmd, import_cmd, init, monitor, ping, profile,
    settings, speedtest, status,
};
use terminal_session_proxy_manager::config::{AppConfig, AppSettings, I18n};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{} {:?}", "Error:".red().bold(), err);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    if env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    let settings = AppSettings::load();
    let i18n = I18n::load(&settings.lang);
    let mut config = AppConfig::load();

    let mut raw_cmd = Cli::command();
    let about_str: &'static str = Box::leak(i18n.t("cmd_about").to_string().into_boxed_str());
    raw_cmd = raw_cmd.about(about_str);

    let sub_names = [
        "status",
        "env",
        "git",
        "export",
        "speedtest",
        "monitor",
        "lang",
        "profile",
        "switch",
        "dash",
        "benchmark",
        "best",
        "import",
        "ping",
        "diagnose",
        "prompt",
        "run",
        "init",
        "completions",
        "config",
        "settings",
        "debug",
    ];

    for name in sub_names {
        let key = format!("cmd_{name}");
        let about_sub: &'static str = Box::leak(i18n.t(&key).to_string().into_boxed_str());
        raw_cmd = raw_cmd.mut_subcommand(name, |s| s.about(about_sub));
    }

    let matches = raw_cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    match cli.command {
        Commands::Status { json } => {
            status::print_status(&config, &i18n, json).await?;
        }
        Commands::Env { mode } => {
            terminal_session_proxy_manager::cmd::env::print_env_commands(&mode, &config, &i18n);
        }
        Commands::Debug { mode } => {
            if let Some(mut path) = dirs::home_dir() {
                path.push(".terminal-session-proxy-manager-debug-enabled");
                match mode {
                    terminal_session_proxy_manager::cli::EnvMode::On => {
                        let _ = std::fs::File::create(path);
                        println!("{} Debug logging enabled", "✓".green());
                    }
                    terminal_session_proxy_manager::cli::EnvMode::Off => {
                        let _ = std::fs::remove_file(path);
                        println!("{} Debug logging disabled", "✓".green());
                    }
                }
            }
        }
        Commands::Git { mode } => {
            git_cmd::handle_git_proxy(&mode, &config, &i18n)?;
        }
        Commands::Export { format } => {
            export_cmd::export_config(&format, &config)?;
        }
        Commands::Speedtest => {
            speedtest::run_speedtest(&config, &i18n).await?;
        }
        Commands::Monitor => {
            monitor::run_monitor(&mut config, &i18n).await?;
        }
        Commands::Lang { code } => {
            settings::set_lang(&code)?;
        }
        Commands::Switch => {
            profile::select_profile_interactive(&mut config, &i18n)?;
        }
        Commands::Dash => {
            dash::run_dashboard(&mut config, &i18n).await?;
        }
        Commands::Benchmark => {
            profile::run_benchmark(&config, &i18n).await?;
        }
        Commands::Best => {
            profile::select_best_profile(&mut config, &i18n).await?;
        }
        Commands::Import { source } => {
            import_cmd::import_profiles(&mut config, &i18n, &source).await?;
        }
        Commands::Profile(profile_cmd) => match profile_cmd {
            ProfileCommands::List => {
                profile::list_profiles(&config, &i18n);
            }
            ProfileCommands::Select => {
                profile::select_profile_interactive(&mut config, &i18n)?;
            }
            ProfileCommands::Use { key } => {
                profile::use_profile(&mut config, &i18n, &key)?;
            }
            ProfileCommands::Set {
                key,
                name,
                port,
                host,
                protocol,
            } => {
                profile::set_profile(&mut config, &i18n, key, name, port, host, protocol)?;
            }
            ProfileCommands::Import { source } => {
                import_cmd::import_profiles(&mut config, &i18n, &source).await?;
            }
            ProfileCommands::Remove { key } => {
                profile::remove_profile(&mut config, &i18n, &key)?;
            }
            ProfileCommands::Benchmark => {
                profile::run_benchmark(&config, &i18n).await?;
            }
            ProfileCommands::Best => {
                profile::select_best_profile(&mut config, &i18n).await?;
            }
        },
        Commands::Ping { timeout } => {
            ping::run_ping(&config, &i18n, timeout).await?;
        }
        Commands::Diagnose => {
            diagnose::run_diagnose(&config, &i18n).await?;
        }
        Commands::Prompt => {
            if env::var("ALL_PROXY").is_ok() || env::var("http_proxy").is_ok() {
                if let Some(p) = config.active_profile() {
                    println!("\x1b[1;33m🌐 [{}:{}]\x1b[0m", p.name, p.port);
                }
            }
        }
        Commands::Run { cmd } => {
            if let (Some(http_url), Some(socks_url), Some(profile)) = (
                config.get_http_url(),
                config.get_socks_url(),
                config.active_profile(),
            ) {
                let mut child = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .env("http_proxy", &http_url)
                    .env("https_proxy", &http_url)
                    .env("ALL_PROXY", &socks_url)
                    .env(
                        "GRADLE_OPTS",
                        format!(
                            "-Dhttp.proxyHost={} -Dhttp.proxyPort={} -Dhttps.proxyHost={} -Dhttps.proxyPort={}",
                            profile.host, profile.port, profile.host, profile.port
                        ),
                    )
                    .env(
                        "JAVA_TOOL_OPTIONS",
                        format!(
                            "-Dhttp.proxyHost={} -Dhttp.proxyPort={} -Dhttps.proxyHost={} -Dhttps.proxyPort={}",
                            profile.host, profile.port, profile.host, profile.port
                        ),
                    )
                    .spawn()?;

                let status = child.wait()?;
                if !status.success() {
                    std::process::exit(status.code().unwrap_or(1));
                }
            } else {
                eprintln!("{}", i18n.t("proxy_load_failed").red().bold());
                std::process::exit(1);
            }
        }
        Commands::Init { shell } => {
            init::generate_shell_init::<Cli>(&shell);
        }
        Commands::Completions { shell } => {
            completions::generate_completions::<Cli>(shell)?;
        }
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Path => {
                println!("{}", i18n.t("config_active_path"));
                println!("{}", AppConfig::get_config_path().display());
            }
            ConfigCommands::Show => {
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
        },
        Commands::Settings(settings_cmd) => match settings_cmd {
            SettingsCommands::Path => {
                settings::show_settings_path(&i18n);
            }
            SettingsCommands::Show => {
                settings::show_settings()?;
            }
            SettingsCommands::Set { config_path, lang } => {
                if let Some(cp) = config_path {
                    settings::set_config_path(cp, &i18n)?;
                }
                if let Some(l) = lang {
                    let lang_code = match l.to_lowercase().as_str() {
                        "en" | "english" => terminal_session_proxy_manager::cli::LangCode::En,
                        _ => terminal_session_proxy_manager::cli::LangCode::Ru,
                    };
                    settings::set_lang(&lang_code)?;
                }
            }
        },
    }

    Ok(())
}
