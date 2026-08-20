use std::path::PathBuf;

use thiserror::Error;

/// Errors that callers may want to distinguish, as opposed to merely report.
///
/// Command handlers return these through `anyhow::Result`, which means `main`
/// can print them uniformly while still allowing a caller to match on a
/// specific variant. Anything that only ever gets shown to the user stays as a
/// plain `anyhow` error instead of earning a variant here.
#[derive(Error, Debug)]
pub enum ProxyError {
    /// Specified profile key was not found in `config.json`.
    ///
    /// Returned rather than merely printed so the process exits non-zero:
    /// scripts doing `proxy profile use "$p" || fallback` depend on it.
    #[error("profile '{0}' not found in configuration")]
    ProfileNotFound(String),

    /// A configuration file exists but could not be parsed.
    ///
    /// Carries the path because the tool resolves it from several sources and
    /// the user otherwise has no way to tell which file is broken.
    #[error("{path} is not valid JSON: {source}")]
    ConfigParse {
        /// The file that failed to parse.
        path: PathBuf,
        /// The underlying `serde_json` failure, including line and column.
        #[source]
        source: serde_json::Error,
    },

    /// A mutating command was asked to write over a file that never loaded.
    ///
    /// Falling back to defaults lets read-only commands keep working, but
    /// saving that fallback would replace the user's real content with
    /// built-in defaults — destroying exactly what the fallback was meant to
    /// protect.
    #[error(
        "refusing to overwrite {path}: it exists but could not be parsed, so saving would \
         replace your settings with defaults. Fix the JSON, or move the file aside, then retry."
    )]
    RefusingToOverwrite {
        /// The unparsable file that would have been clobbered.
        path: PathBuf,
    },

    /// A profile field would produce an unusable proxy URL.
    #[error("invalid profile '{key}': {reason}")]
    InvalidProfile {
        /// Profile key that failed validation.
        key: String,
        /// Human-readable explanation of what is wrong.
        reason: String,
    },

    /// Standard I/O operation error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing or serialization error without an associated path.
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP client or transport network error.
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// Path error for invalid configuration files.
    #[error("invalid configuration path: {0}")]
    InvalidPath(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_not_found_names_the_missing_key() {
        let err = ProxyError::ProfileNotFound("staging".to_string());

        assert_eq!(
            err.to_string(),
            "profile 'staging' not found in configuration"
        );
    }

    #[test]
    fn config_parse_error_names_the_offending_file() {
        let source = serde_json::from_str::<serde_json::Value>("{oops").unwrap_err();
        let err = ProxyError::ConfigParse {
            path: PathBuf::from("/tmp/config.json"),
            source,
        };

        assert!(
            err.to_string()
                .starts_with("/tmp/config.json is not valid JSON:")
        );
    }
}
