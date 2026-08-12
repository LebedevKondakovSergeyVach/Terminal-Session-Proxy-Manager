use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use proxy_cli::cmd::*;
use proxy_cli::config::{AppConfig, AppSettings, I18n};
use std::env;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "proxy-cli",
    author = "LebedevSergeyVach",
    version = "1.0.0",
    about = "Universal, configurable CLI proxy management toolkit in Rust",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check network status, IPv4, IPv6, and physical location
    Status {
        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },
    /// Generate environment variable export commands for shell (eval)
    Env {
        /// Mode: on or off
        mode: String,
    },
    /// Manage global Git proxy settings (on, off, status)
    Git {
        /// Mode: on, off, or status
        #[arg(default_value = "status")]
        mode: String,
    },
    /// Export proxy configuration to Docker, cURL, or .env formats
    Export {
        /// Format: docker, curl, or envfile
        #[arg(default_value = "envfile")]
        format: String,
    },
    /// Benchmark real download bandwidth throughput in MB/s
    Speedtest,
    /// Monitor active proxy health and auto-fallback on failure
    Monitor,
    /// Switch application interface language (ru, en)
    Lang {
        /// Language code: ru or en
        code: String,
    },
    /// Manage proxy profiles in config.json
    #[command(subcommand)]
    Profile(ProfileCommands),

    /// Interactive proxy profile selector
    Switch,

    /// Benchmark ping and availability of all profiles
    Benchmark,

    /// Automatically select the fastest proxy with lowest latency
    Best,

    /// Import proxy profiles from local JSON file or URL subscription
    Import {
        /// File path or URL link
        source: String,
    },

    /// Probe latency to endpoints configured in config.json
    Ping {
        /// Timeout in milliseconds (default 4000)
        #[arg(short, long, default_value_t = 4000)]
        timeout: u64,
    },
    /// Extended diagnostics for local sockets and HTTP endpoints
    Diagnose,
    /// Proxy indicator for Zsh prompt segment
    Prompt,
    /// Run single command through proxy without modifying current shell
    Run {
        /// Command and arguments
        #[arg(required = true, num_args = 1..)]
        cmd: Vec<String>,
    },
    /// Generate interactive shell initialization script (zsh/bash)
    Init {
        /// Shell type: zsh or bash
        shell: String,
    },
    /// Generate auto-completion scripts (zsh, bash, fish, powershell)
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Manage active config.json file
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Manage global settings.json file
    #[command(subcommand)]
    Settings(SettingsCommands),
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// List all available profiles
    List,
    /// Select profile interactively
    Select,
    /// Choose active profile by key (e.g. throne, v2ray)
    Use {
        /// Profile key
        key: String,
    },
    /// Add or update profile
    Set {
        /// Profile key
        key: String,
        /// Display name
        #[arg(short, long)]
        name: Option<String>,
        /// Port number
        #[arg(short, long)]
        port: Option<u16>,
        /// Host address
        #[arg(short, long)]
        host: Option<String>,
        /// Protocol (socks5, http)
        #[arg(short = 't', long, default_value = "socks5")]
        protocol: String,
    },
    /// Import profiles from file or URL
    Import {
        /// File path or URL
        source: String,
    },
    /// Remove profile by key
    Remove {
        /// Profile key
        key: String,
    },
    /// Benchmark speed of all profiles
    Benchmark,
    /// Choose fastest profile
    Best,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show path to active config.json file
    Path,
    /// Show current JSON configuration
    Show,
}

#[derive(Subcommand)]
pub enum SettingsCommands {
    /// Show path to settings.json file
    Path,
    /// Show current settings.json contents
    Show,
    /// Set config path or interface language
    Set {
        /// Config path
        #[arg(short, long)]
        config_path: Option<String>,
        /// Interface language (ru, en)
        #[arg(short, long)]
        lang: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
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
    ];

    for name in sub_names {
        let key = format!("cmd_{}", name);
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
            proxy_cli::cmd::env::print_env_commands(&mode, &config, &i18n);
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
            settings::set_lang(code)?;
        }
        Commands::Switch => {
            profile::select_profile_interactive(&mut config, &i18n)?;
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

                child.wait()?;
            } else {
                eprintln!("{}", i18n.t("proxy_load_failed").red().bold());
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
                    settings::set_lang(l)?;
                }
            }
        },
    }

    Ok(())
}
