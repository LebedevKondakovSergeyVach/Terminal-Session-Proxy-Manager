use serde::{Deserialize, Serialize};

use crate::error::ProxyError;

fn default_protocol() -> String {
    "socks5".to_string()
}

/// Proxy schemes this tool will hand to the HTTP client.
///
/// Anything outside this list either fails inside `reqwest` at request time —
/// long after the user typed the command — or, worse, is accepted and silently
/// ignored. Rejecting it at the point of entry keeps the error next to the typo.
pub const SUPPORTED_PROTOCOLS: &[&str] =
    &["http", "https", "socks4", "socks4a", "socks5", "socks5h"];

/// Proxy connection profile configuration.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Human-readable display name for the proxy profile.
    pub name: String,
    /// Host address or IP (e.g. `127.0.0.1`).
    pub host: String,
    /// Port number (e.g. `1080`, `8080`).
    pub port: u16,
    /// Protocol schema (`socks5`, `http`, ...).
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

impl Profile {
    /// Checks that this profile can produce a usable proxy URL.
    ///
    /// Run before a profile is written to disk, whether it came from
    /// `profile set` or from an imported subscription, so a malformed entry
    /// cannot wedge every later command that reads the config.
    ///
    /// # Errors
    /// Returns [`ProxyError::InvalidProfile`] describing the first problem found.
    pub fn validate(&self, key: &str) -> Result<(), ProxyError> {
        let invalid = |reason: String| ProxyError::InvalidProfile {
            key: key.to_string(),
            reason,
        };

        if self.host.trim().is_empty() {
            return Err(invalid("host is empty".to_string()));
        }

        // Allowlist rather than denylist: hostnames, IPv4 literals and
        // bracketed IPv6 literals need nothing outside this set, and anything
        // else is either a typo or an injection attempt.
        if let Some(bad) = self.host.chars().find(|c| {
            !(c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':' | '[' | ']'))
        }) {
            return Err(invalid(format!(
                "host contains an illegal character {bad:?}"
            )));
        }

        if self.port == 0 {
            return Err(invalid("port 0 is not a listening port".to_string()));
        }

        if !SUPPORTED_PROTOCOLS.contains(&self.protocol.to_lowercase().as_str()) {
            return Err(invalid(format!(
                "protocol '{}' is not one of: {}",
                self.protocol,
                SUPPORTED_PROTOCOLS.join(", ")
            )));
        }

        if self.name.chars().any(char::is_control) {
            return Err(invalid("name contains a control character".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid() -> Profile {
        Profile {
            name: "Test".to_string(),
            host: "127.0.0.1".to_string(),
            port: 1080,
            protocol: "socks5".to_string(),
        }
    }

    #[test]
    fn test_profile_deserialization_default_protocol() {
        let json_data = json!({
            "name": "Test",
            "host": "127.0.0.1",
            "port": 8080
        });

        let profile: Profile = serde_json::from_value(json_data).unwrap();
        assert_eq!(profile.name, "Test");
        assert_eq!(profile.host, "127.0.0.1");
        assert_eq!(profile.port, 8080);
        assert_eq!(profile.protocol, "socks5");
    }

    #[test]
    fn test_profile_deserialization_custom_protocol() {
        let json_data = json!({
            "name": "Test HTTP",
            "host": "192.168.1.1",
            "port": 3128,
            "protocol": "http"
        });

        let profile: Profile = serde_json::from_value(json_data).unwrap();
        assert_eq!(profile.protocol, "http");
    }

    #[test]
    fn a_well_formed_profile_validates() {
        assert!(valid().validate("k").is_ok());
    }

    #[test]
    fn every_supported_protocol_is_accepted() {
        for protocol in SUPPORTED_PROTOCOLS {
            let profile = Profile {
                protocol: (*protocol).to_string(),
                ..valid()
            };

            assert!(
                profile.validate("k").is_ok(),
                "{protocol} is advertised as supported but rejected"
            );
        }
    }

    #[test]
    fn protocol_matching_ignores_case() {
        let profile = Profile {
            protocol: "SOCKS5".to_string(),
            ..valid()
        };

        assert!(profile.validate("k").is_ok());
    }

    #[test]
    fn an_unknown_protocol_is_rejected_with_the_supported_list() {
        let profile = Profile {
            protocol: "sockz".to_string(),
            ..valid()
        };

        let message = profile.validate("k").unwrap_err().to_string();

        assert!(message.contains("sockz"));
        assert!(message.contains("socks5"));
    }

    #[test]
    fn an_empty_host_is_rejected() {
        let profile = Profile {
            host: "  ".to_string(),
            ..valid()
        };

        assert!(
            profile
                .validate("k")
                .unwrap_err()
                .to_string()
                .contains("host is empty")
        );
    }

    #[test]
    fn port_zero_is_rejected() {
        let profile = Profile { port: 0, ..valid() };

        assert!(
            profile
                .validate("k")
                .unwrap_err()
                .to_string()
                .contains("port 0")
        );
    }

    #[test]
    fn a_host_carrying_shell_or_url_syntax_is_rejected() {
        for host in [
            "a b",
            "a;b",
            "user@host",
            "a/b",
            "a\nb",
            "$(id)",
            "a`id`b",
            "a|b",
        ] {
            let profile = Profile {
                host: host.to_string(),
                ..valid()
            };

            assert!(
                profile.validate("k").is_err(),
                "host {host:?} should not be accepted"
            );
        }
    }

    #[test]
    fn realistic_hosts_are_not_caught_by_the_allowlist() {
        for host in [
            "127.0.0.1",
            "proxy.example.com",
            "my-proxy_01",
            "::1",
            "[::1]",
        ] {
            let profile = Profile {
                host: host.to_string(),
                ..valid()
            };

            assert!(
                profile.validate("k").is_ok(),
                "host {host:?} should be accepted"
            );
        }
    }

    #[test]
    fn the_error_names_the_profile_key_so_the_user_can_find_it() {
        let profile = Profile { port: 0, ..valid() };

        assert!(
            profile
                .validate("staging-eu")
                .unwrap_err()
                .to_string()
                .contains("staging-eu")
        );
    }
}
