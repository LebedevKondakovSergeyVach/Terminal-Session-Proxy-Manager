#![warn(missing_docs)]
//! Proxy CLI Core Library
//!
//! Universal, configurable CLI proxy management toolkit in Rust.

/// Command-line argument parsing and CLI definitions.
pub mod cli;
/// Command implementations for all CLI subcommands.
pub mod cmd;
/// Configuration and settings management (JSON mapping, i18n).
pub mod config;
/// Custom error types and `anyhow` wrappers.
pub mod error;

pub use config::{AppConfig, AppSettings, I18n, Profile};
pub use error::ProxyError;
