mod config;
mod diagnose;
mod env_cmd;
mod init;
mod ping;
mod status;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{AppConfig, AppSettings, Profile};
use colored::Colorize;
use std::env;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "proxy-cli",
    author = "Antigravity Developer",
    version = "0.2.0",
    about = "Universal, configurable CLI proxy management toolkit in Rust",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Проверить статус сети, IPv4, IPv6 и геолокацию
    Status {
        /// Вывод информации в формате JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Сгенерировать команды экспорта переменных для shell (eval)
    Env {
        /// Режим: on или off
        mode: String,
    },
    /// Управление профилями прокси в config.json
    #[command(subcommand)]
    Profile(ProfileCommands),

    /// Замер задержки (пинг) до сервисов из config.json
    Ping {
        /// Таймаут ожидания в миллисекундах (по умолчанию 4000)
        #[arg(short, long, default_value_t = 4000)]
        timeout: u64,
    },
    /// Расширенная диагностика локальных сокетов и эндпоинтов
    Diagnose,
    /// Индикатор прокси для строки приглашения Zsh Prompt
    Prompt,
    /// Выполнить команду через прокси без изменения окружения текущей сессии
    Run {
        /// Команда и аргументы для выполнения
        #[arg(required = true, num_args = 1..)]
        cmd: Vec<String>,
    },
    /// Генерация интерактивного скрипта инициализации для shell (zsh/bash)
    Init {
        /// Тип оболочки: zsh или bash
        shell: String,
    },
    /// Управление файлом конфигурации config.json
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Управление глобальными настройками (settings.json)
    #[command(subcommand)]
    Settings(SettingsCommands),
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// Показать список всех доступных профилей
    List,
    /// Выбрать активный профиль по ключу (например: throne, v2ray)
    Use {
        /// Ключ профиля из config.json
        key: String,
    },
    /// Добавить или обновить профиль
    Set {
        /// Ключ профиля (например: throne, v2ray, custom)
        key: String,
        /// Отображаемое имя (например: Throne, v2rayN)
        #[arg(short, long)]
        name: Option<String>,
        /// Номер порта (например: 2080, 10808)
        #[arg(short, long)]
        port: Option<u16>,
        /// Хост (по умолчанию 127.0.0.1)
        #[arg(short, long)]
        host: Option<String>,
        /// Протокол (socks5, http)
        #[arg(short = 't', long, default_value = "socks5")]
        protocol: String,
    },
    /// Удалить профиль по ключу
    Remove {
        /// Ключ профиля
        key: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Показать точный путь к целевому файлу config.json
    Path,
    /// Вывести текущую конфигурацию JSON
    Show,
}

#[derive(Subcommand)]
enum SettingsCommands {
    /// Показать путь к файлу settings.json
    Path,
    /// Показать текущие настройки settings.json
    Show,
    /// Задать путь к целевому config.json
    Set {
        /// Путь к конфигу (например: ./configs/config.throne-v2ray.json)
        #[arg(short, long)]
        config_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = AppConfig::load();

    match cli.command {
        Commands::Status { json } => {
            status::print_status(&config, json).await?;
        }
        Commands::Env { mode } => {
            env_cmd::print_env_commands(&mode, &config);
        }
        Commands::Profile(profile_cmd) => match profile_cmd {
            ProfileCommands::List => {
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
            ProfileCommands::Use { key } => {
                if config.profiles.contains_key(&key) {
                    config.active_profile = key.clone();
                    config.save()?;
                    if let Some(p) = config.active_profile() {
                        println!("⚙️ Переключено на профиль: {} ({}:{})", p.name.green().bold(), p.host, p.port);
                    }
                } else {
                    eprintln!("{}", format!("❌ Профиль '{}' не найден в config.json!", key).red().bold());
                }
            }
            ProfileCommands::Set { key, name, port, host, protocol } => {
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
            }
            ProfileCommands::Remove { key } => {
                if config.profiles.remove(&key).is_some() {
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
            }
        },
        Commands::Ping { timeout } => {
            ping::run_ping(&config, timeout).await?;
        }
        Commands::Diagnose => {
            diagnose::run_diagnose(&config).await?;
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
                eprintln!("{}", "❌ Не удалось загрузить настройки прокси!".red().bold());
            }
        }
        Commands::Init { shell } => {
            init::generate_shell_init(&shell);
        }
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Path => {
                println!("📍 Файл активной конфигурации:");
                println!("{}", AppConfig::get_config_path().display());
            }
            ConfigCommands::Show => {
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
        },
        Commands::Settings(settings_cmd) => match settings_cmd {
            SettingsCommands::Path => {
                println!("📍 Файл настроек settings.json:");
                println!("{}", AppSettings::get_settings_path().display());
            }
            SettingsCommands::Show => {
                let settings = AppSettings::load();
                println!("{}", serde_json::to_string_pretty(&settings)?);
            }
            SettingsCommands::Set { config_path } => {
                let mut settings = AppSettings::load();
                settings.config_path = Some(config_path.clone());
                settings.save()?;
                println!("⚙️ Установлен новый путь к конфигу в settings.json: {}", config_path.green().bold());
            }
        },
    }

    Ok(())
}
