use thiserror::Error;

/// Custom error types for Proxy CLI operations.
#[derive(Error, Debug)]
pub enum ProxyError {
    /// Specified profile key was not found in `config.json`.
    #[error("Profile '{0}' not found in configuration")]
    ProfileNotFound(String),

    /// Standard I/O operation error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing or serialization error.
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP client or transport network error.
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// Path error for invalid configuration files.
    #[error("Invalid configuration path: {0}")]
    InvalidPath(String),
}
