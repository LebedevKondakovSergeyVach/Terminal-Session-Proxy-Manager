use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::config::{CONFIG_PATH_ENV, SETTINGS_PATH_ENV};

/// Top-level CLI configuration | Конфигурация CLI
#[derive(Parser, Debug)]
#[command(
    name = "terminal-session-proxy-manager",
    author = "LebedevKondakovSergeyVach",
    version = env!("CARGO_PKG_VERSION"),
    about = "Universal, configurable CLI proxy management toolkit in Rust | Универсальный инструментарий управления прокси на Rust",
    long_about = None
)]
pub struct Cli {
    /// Path to config.json for this run | Путь к config.json для текущего запуска
    #[arg(long, global = true, value_name = "PATH", env = CONFIG_PATH_ENV)]
    pub config_file: Option<PathBuf>,

    /// Path to settings.json for this run | Путь к settings.json для текущего запуска
    #[arg(long, global = true, value_name = "PATH", env = SETTINGS_PATH_ENV)]
    pub settings_file: Option<PathBuf>,

    /// Interface language for this run only | Язык интерфейса только для этого запуска
    #[arg(long, global = true, value_enum, env = "TSPM_LANG")]
    pub lang: Option<LangCode>,

    /// The subcommand to execute | Исполняемая подкоманда
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for Proxy CLI | Доступные подкоманды
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check network status, IPv4, IPv6, and physical location | Проверить статус сети, IPv4, IPv6 и геолокацию
    Status {
        /// Output in JSON format | Вывод в формате JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Generate environment variable export commands for shell (eval) | Сгенерировать команды экспорта переменных для shell (eval)
    Env {
        /// Mode: on or off | Режим: on или off
        #[arg(value_enum)]
        mode: EnvMode,
    },
    /// Manage global Git proxy settings (on, off, status) | Управление глобальными настройками прокси в Git
    Git {
        /// Mode: on, off, or status | Режим: on, off, или status
        #[arg(value_enum, default_value_t = GitMode::Status)]
        mode: GitMode,
    },
    /// Export proxy configuration to Docker, cURL, or .env formats | Экспорт настроек прокси в форматы Docker, cURL и .env
    Export {
        /// Format: docker, curl, or envfile | Формат: docker, curl, или envfile
        #[arg(value_enum, default_value_t = ExportFormat::Envfile)]
        format: ExportFormat,
    },
    /// Benchmark real download bandwidth throughput in MB/s | Замер реальной скорости загрузки в МБ/с
    Speedtest,
    /// Monitor active proxy health and auto-fallback on failure | Проверка здоровья прокси и авто-переключение при сбоях
    Monitor,
    /// Switch application interface language (ru, en) | Переключить язык интерфейса (ru, en)
    Lang {
        /// Language code: ru or en | Код языка: ru или en
        #[arg(value_enum)]
        code: LangCode,
    },
    /// Manage proxy profiles in config.json | Управление профилями прокси в config.json
    #[command(subcommand)]
    Profile(ProfileCommands),

    /// Manage shell-integration debug logging (on, off) | Управление отладочным логом shell-интеграции (on, off)
    Debug {
        /// Mode: on or off | Режим: on или off
        #[arg(value_enum)]
        mode: EnvMode,
    },

    /// Interactive proxy profile selector | Интерактивный выбор профиля прокси
    Switch,

    /// Interactive TUI Dashboard for monitoring and switching profiles | Интерактивный TUI дашборд для мониторинга и переключения
    Dash,

    /// Benchmark ping and availability of all profiles | Замерить пинг и скорость всех профилей
    Benchmark,

    /// Automatically select the fastest proxy with lowest latency | Автоматически выбрать самый быстрый прокси
    Best,

    /// Import proxy profiles from local JSON file or URL subscription | Импорт профилей из файла или URL
    Import {
        /// File path or URL link | Путь к файлу или URL
        source: String,
    },

    /// Probe latency to endpoints configured in config.json | Замер задержки (пинг) до эндпоинтов из config.json
    Ping {
        /// Timeout in milliseconds (default 4000) | Таймаут в мс (по умолчанию 4000)
        #[arg(short, long, default_value_t = 4000)]
        timeout: u64,
    },
    /// Extended diagnostics for local sockets and HTTP endpoints | Расширенная диагностика сети и сокетов
    Diagnose,
    /// Proxy indicator for Zsh prompt segment | Индикатор прокси для Zsh Prompt
    Prompt,
    /// Run single command through proxy without modifying current shell | Выполнить команду через прокси без изменения текущей сессии
    Run {
        /// Command and arguments | Команда и аргументы
        #[arg(required = true, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    /// Generate interactive shell initialization script (zsh/bash) | Генерация скрипта инициализации для shell (zsh/bash)
    Init {
        /// Shell type: zsh or bash | Тип shell: zsh или bash
        #[arg(value_enum)]
        shell: ShellType,
    },
    /// Generate auto-completion scripts (zsh, bash, fish, powershell) | Генерация файлов автодополнения (completions)
    Completions {
        /// Shell type | Тип shell
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Manage active config.json file | Управление файлом config.json
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Manage global settings.json file | Управление настройками settings.json
    #[command(subcommand)]
    Settings(SettingsCommands),
}

/// Commands for managing proxy profiles | Команды для управления профилями прокси
#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List all available profiles | Вывести список всех профилей
    List,
    /// Select profile interactively | Выбрать профиль интерактивно
    Select,
    /// Choose active profile by key (e.g. throne, v2ray) | Выбрать активный профиль по ключу
    Use {
        /// Profile key | Ключ профиля
        key: String,
    },
    /// Add or update profile | Добавить или обновить профиль
    Set {
        /// Profile key | Ключ профиля
        key: String,
        /// Display name | Название
        #[arg(short, long)]
        name: Option<String>,
        /// Port number | Номер порта
        #[arg(short, long)]
        port: Option<u16>,
        /// Host address | Адрес хоста
        ///
        /// Deliberately long-form only: `-h` belongs to `--help`.
        #[arg(long)]
        host: Option<String>,
        /// Protocol (socks5, http) | Протокол (socks5, http)
        #[arg(short = 't', long, default_value = "socks5")]
        protocol: String,
    },
    /// Import profiles from file or URL | Импорт профилей из файла или URL
    Import {
        /// File path or URL | Путь или URL
        source: String,
    },
    /// Remove profile by key | Удалить профиль по ключу
    Remove {
        /// Profile key | Ключ профиля
        key: String,
    },
    /// Benchmark speed of all profiles | Тест скорости всех профилей
    Benchmark,
    /// Choose fastest profile | Выбрать самый быстрый профиль
    Best,
}

/// Commands for managing config file location and contents | Команды управления расположением конфигурации
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show path to active config.json file | Показать путь к текущему config.json
    Path,
    /// Show current JSON configuration | Показать содержимое конфигурации в JSON
    Show,
}

/// Commands for application settings (global settings.json) | Команды управления глобальными настройками
#[derive(Subcommand, Debug)]
pub enum SettingsCommands {
    /// Show path to settings.json file | Показать путь к файлу settings.json
    Path,
    /// Show current settings.json contents | Показать содержимое settings.json
    Show,
    /// Set config path or interface language | Настроить путь к конфигу или язык
    Set {
        /// Config path | Путь к конфигурации
        #[arg(short, long)]
        config_path: Option<String>,
        /// Interface language (ru, en) | Язык интерфейса (ru, en)
        #[arg(short, long, value_enum)]
        lang: Option<LangCode>,
    },
}

/// Modes for generating shell environment variables | Режимы для генерации переменных окружения
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvMode {
    /// Enable proxy (export variables) | Включить (export)
    On,
    /// Disable proxy (unset variables) | Выключить (unset)
    Off,
}

/// Modes for Git global proxy configuration | Режимы для настройки прокси в Git
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitMode {
    /// Set Git proxy globally | Включить прокси для Git
    On,
    /// Unset Git proxy globally | Отключить прокси для Git
    Off,
    /// Display current Git proxy status | Показать статус прокси в Git
    Status,
}

/// Output formats for configuration export | Форматы для экспорта конфигурации
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    /// Output as Docker build arguments | В виде Docker build args
    Docker,
    /// Output as cURL proxy arguments | В виде аргументов cURL
    Curl,
    /// Output as `.env` file variables | В формате .env файла
    Envfile,
}

/// Available localization languages | Доступные языки локализации
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LangCode {
    /// Russian | Русский
    Ru,
    /// English | Английский
    En,
}

/// Supported shells for integrations | Поддерживаемые типы shell
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellType {
    /// Zsh shell | Оболочка Zsh
    Zsh,
    /// Bash shell | Оболочка Bash
    Bash,
}

impl LangCode {
    /// Returns the two-letter code stored in `settings.json`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ru => "ru",
            Self::En => "en",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_command_tree_is_internally_consistent() {
        // clap's own validation: duplicate arg ids, bad defaults, conflicting
        // shorts. Without this it only fires when a user runs the broken path.
        Cli::command().debug_assert();
    }

    #[test]
    fn global_overrides_are_accepted_before_and_after_the_subcommand() {
        for args in [
            vec!["tspm", "--config-file", "/tmp/c.json", "status"],
            vec!["tspm", "status", "--config-file", "/tmp/c.json"],
        ] {
            let cli = Cli::try_parse_from(args).unwrap();
            assert_eq!(cli.config_file.unwrap().to_str().unwrap(), "/tmp/c.json");
        }
    }

    #[test]
    fn an_unknown_language_is_rejected_rather_than_silently_defaulting() {
        // `settings set --lang de` used to fall through to Russian.
        assert!(Cli::try_parse_from(["tspm", "settings", "set", "--lang", "de"]).is_err());
        assert!(Cli::try_parse_from(["tspm", "settings", "set", "--lang", "en"]).is_ok());
    }

    #[test]
    fn run_requires_a_command_to_execute() {
        assert!(Cli::try_parse_from(["tspm", "run"]).is_err());
    }

    #[test]
    fn run_forwards_flags_to_the_child_instead_of_claiming_them() {
        // `run curl -sS <url>` used to fail with "unknown argument -s".
        let cli =
            Cli::try_parse_from(["tspm", "run", "curl", "-sS", "https://example.com"]).unwrap();

        let Commands::Run { cmd } = cli.command else {
            panic!("expected the run subcommand");
        };
        assert_eq!(cmd, ["curl", "-sS", "https://example.com"]);
    }

    #[test]
    fn run_still_accepts_the_explicit_double_dash_form() {
        let cli = Cli::try_parse_from(["tspm", "run", "--", "curl", "-sS"]).unwrap();

        let Commands::Run { cmd } = cli.command else {
            panic!("expected the run subcommand");
        };
        assert_eq!(cmd, ["curl", "-sS"]);
    }

    #[test]
    fn help_is_reachable_from_every_subcommand() {
        // Regression guard for `-h` having been bound to `--host`.
        for args in [
            vec!["tspm", "-h"],
            vec!["tspm", "profile", "set", "-h"],
            vec!["tspm", "status", "-h"],
        ] {
            let err = Cli::try_parse_from(&args).unwrap_err();
            assert_eq!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp,
                "{args:?} did not print help"
            );
        }
    }

    #[test]
    fn ping_has_a_default_timeout() {
        let cli = Cli::try_parse_from(["tspm", "ping"]).unwrap();

        let Commands::Ping { timeout } = cli.command else {
            panic!("expected the ping subcommand");
        };
        assert_eq!(timeout, 4000);
    }

    #[test]
    fn git_defaults_to_the_read_only_status_mode() {
        // Defaulting to `on` would mutate global git config on a bare `git`.
        let cli = Cli::try_parse_from(["tspm", "git"]).unwrap();

        let Commands::Git { mode } = cli.command else {
            panic!("expected the git subcommand");
        };
        assert!(matches!(mode, GitMode::Status));
    }

    #[test]
    fn lang_codes_map_to_their_stored_representation() {
        assert_eq!(LangCode::Ru.as_str(), "ru");
        assert_eq!(LangCode::En.as_str(), "en");
    }
}
