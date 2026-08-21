//! Terminal Session Proxy Manager — core library.
//!
//! A configurable CLI toolkit for managing SOCKS/HTTP proxy profiles in a
//! terminal session: switching profiles, benchmarking latency, diagnosing
//! connectivity, and generating the environment exports a shell can evaluate.
//!
//! The binary in `main.rs` is a thin dispatcher; everything it does lives here
//! so it can be exercised by tests without spawning a process.
//!
//! # Layout
//! - [`cli`] — the clap command tree, and nothing else.
//! - [`cmd`] — one module per subcommand, each owning its own output.
//! - [`config`] — `config.json` / `settings.json` loading and the i18n catalogue.
//! - [`proxy_env`] — the single definition of the proxy environment variables.
//! - [`shell_handoff`] — the files used to change the parent shell's environment.
//! - [`error`] — error variants callers may want to match on.

/// Command-line argument parsing and CLI definitions.
pub mod cli;
/// Command implementations for all CLI subcommands.
pub mod cmd;
/// Configuration and settings management (JSON mapping, i18n).
pub mod config;
/// Custom error types and `anyhow` wrappers.
pub mod error;
/// Shared construction of proxy environment variables.
pub mod proxy_env;
/// Files used to hand shell statements back to the parent shell.
pub mod shell_handoff;

pub use config::{AppConfig, AppSettings, I18n, Profile};
pub use error::ProxyError;
