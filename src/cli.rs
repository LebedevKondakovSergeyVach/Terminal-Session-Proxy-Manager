use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Top-level CLI configuration.
#[derive(Parser)]
#[command(
    name = "proxy-cli",
    author = "LebedevSergeyVach",
    version = env!("CARGO_PKG_VERSION"),
    about = "Universal, configurable CLI proxy management toolkit in Rust",
    long_about = None
)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for Proxy CLI.
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
        #[arg(value_enum)]
        mode: EnvMode,
    },
    /// Manage global Git proxy settings (on, off, status)
    Git {
        /// Mode: on, off, or status
        #[arg(value_enum, default_value_t = GitMode::Status)]
        mode: GitMode,
    },
    /// Export proxy configuration to Docker, cURL, or .env formats
    Export {
        /// Format: docker, curl, or envfile
        #[arg(value_enum, default_value_t = ExportFormat::Envfile)]
        format: ExportFormat,
    },
    /// Benchmark real download bandwidth throughput in MB/s
    Speedtest,
    /// Monitor active proxy health and auto-fallback on failure
    Monitor,
    /// Switch application interface language (ru, en)
    Lang {
        /// Language code: ru or en
        #[arg(value_enum)]
        code: LangCode,
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
        #[arg(value_enum)]
        shell: ShellType,
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

/// Commands for managing proxy profiles.
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

/// Commands for managing config file location and contents.
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show path to active config.json file
    Path,
    /// Show current JSON configuration
    Show,
}

/// Commands for application settings (global settings.json).
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

/// Modes for generating shell environment variables.
#[derive(ValueEnum, Clone, Debug)]
pub enum EnvMode {
    /// Enable proxy (export variables)
    On,
    /// Disable proxy (unset variables)
    Off,
}

/// Modes for Git global proxy configuration.
#[derive(ValueEnum, Clone, Debug)]
pub enum GitMode {
    /// Set Git proxy globally
    On,
    /// Unset Git proxy globally
    Off,
    /// Display current Git proxy status
    Status,
}

/// Output formats for configuration export.
#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    /// Output as Docker build arguments
    Docker,
    /// Output as cURL proxy arguments
    Curl,
    /// Output as `.env` file variables
    Envfile,
}

/// Available localization languages.
#[derive(ValueEnum, Clone, Debug)]
pub enum LangCode {
    /// Russian
    Ru,
    /// English
    En,
}

/// Supported shells for integrations.
#[derive(ValueEnum, Clone, Debug)]
pub enum ShellType {
    /// Zsh shell
    Zsh,
    /// Bash shell
    Bash,
}
