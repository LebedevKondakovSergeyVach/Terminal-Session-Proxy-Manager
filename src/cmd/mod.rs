/// Generates shell completion scripts.
pub mod completions;
/// Checks socket connectivity and diagnoses endpoints.
pub mod diagnose;
/// Manages shell environment variables (`http_proxy`, etc.).
pub mod env;
/// Exports configurations for external tools (Docker, cURL).
pub mod export_cmd;
/// Configures global Git proxy settings.
pub mod git_cmd;
/// Imports proxy profiles from URLs or files.
pub mod import_cmd;
/// Generates shell initialization scripts (Zsh, Bash).
pub mod init;
/// Runs a background loop to monitor active proxy health.
pub mod monitor;
/// Pings proxy profiles to measure latency.
pub mod ping;
/// Commands for manipulating and selecting proxy profiles.
pub mod profile;
/// Manages global `settings.json` (config path and language).
pub mod settings;
/// Measures real download throughput bandwidth.
pub mod speedtest;
/// Displays the current active proxy status and network information.
pub mod status;
