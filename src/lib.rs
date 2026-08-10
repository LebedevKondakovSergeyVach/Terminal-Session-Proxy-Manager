//! Proxy CLI Core Library
//!
//! Universal, configurable CLI proxy management toolkit in Rust.

pub mod cmd;
pub mod config;
pub mod error;

pub use config::{AppConfig, AppSettings, I18n, Profile};
pub use error::ProxyError;
