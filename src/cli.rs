use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Top-level CLI configuration | Конфигурация CLI
#[derive(Parser)]
#[command(
    name = "terminal-session-proxy-manager",
    author = "LebedevKondakovSergeyVach",
    version = env!("CARGO_PKG_VERSION"),
    about = "Universal, configurable CLI proxy management toolkit in Rust | Универсальный инструментарий управления прокси на Rust",
    long_about = None
)]
pub struct Cli {
    /// The subcommand to execute | Исполняемая подкоманда
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for Proxy CLI | Доступные подкоманды
#[derive(Subcommand)]
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

    /// Manage debug logging state (on, off) | Управление режимом отладки (on, off)
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
        #[arg(required = true, num_args = 1..)]
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
#[derive(Subcommand)]
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
        #[arg(short, long)]
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
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show path to active config.json file | Показать путь к текущему config.json
    Path,
    /// Show current JSON configuration | Показать содержимое конфигурации в JSON
    Show,
}

/// Commands for application settings (global settings.json) | Команды управления глобальными настройками
#[derive(Subcommand)]
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
        #[arg(short, long)]
        lang: Option<String>,
    },
}

/// Modes for generating shell environment variables | Режимы для генерации переменных окружения
#[derive(ValueEnum, Clone, Debug)]
pub enum EnvMode {
    /// Enable proxy (export variables) | Включить (export)
    On,
    /// Disable proxy (unset variables) | Выключить (unset)
    Off,
}

/// Modes for Git global proxy configuration | Режимы для настройки прокси в Git
#[derive(ValueEnum, Clone, Debug)]
pub enum GitMode {
    /// Set Git proxy globally | Включить прокси для Git
    On,
    /// Unset Git proxy globally | Отключить прокси для Git
    Off,
    /// Display current Git proxy status | Показать статус прокси в Git
    Status,
}

/// Output formats for configuration export | Форматы для экспорта конфигурации
#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    /// Output as Docker build arguments | В виде Docker build args
    Docker,
    /// Output as cURL proxy arguments | В виде аргументов cURL
    Curl,
    /// Output as `.env` file variables | В формате .env файла
    Envfile,
}

/// Available localization languages | Доступные языки локализации
#[derive(ValueEnum, Clone, Debug)]
pub enum LangCode {
    /// Russian | Русский
    Ru,
    /// English | Английский
    En,
}

/// Supported shells for integrations | Поддерживаемые типы shell
#[derive(ValueEnum, Clone, Debug)]
pub enum ShellType {
    /// Zsh shell | Оболочка Zsh
    Zsh,
    /// Bash shell | Оболочка Bash
    Bash,
}
