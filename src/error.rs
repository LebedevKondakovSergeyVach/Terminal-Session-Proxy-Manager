use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Profile '{0}' not found in configuration")]
    ProfileNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid configuration path: {0}")]
    InvalidPath(String),
}
