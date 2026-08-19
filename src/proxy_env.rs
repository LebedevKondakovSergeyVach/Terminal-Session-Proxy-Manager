//! Single source of truth for the proxy environment variables this tool manages.
//!
//! Three call sites need the exact same variable set: `env on` (printed for
//! `eval`), `run` (applied to a child process), and the TUI dashboard (written
//! to the hand-off file the shell wrapper evaluates). Before this module they
//! each built the list independently and had already drifted apart — the
//! dashboard exported only `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` and silently
//! dropped the JVM options that `env on` sets.

use crate::config::AppConfig;

/// Every environment variable this tool sets, in the order it sets them.
///
/// `env off` unsets exactly this list, so adding a variable here automatically
/// keeps the on/off pair symmetrical.
pub const MANAGED_ENV_VARS: &[&str] = &[
    "http_proxy",
    "HTTP_PROXY",
    "https_proxy",
    "HTTPS_PROXY",
    "all_proxy",
    "ALL_PROXY",
    "GRADLE_OPTS",
    "JAVA_TOOL_OPTIONS",
];

/// Builds the proxy environment variables for the active profile.
///
/// Returns `None` when no profile is active, which callers surface as a
/// "profile could not be loaded" error rather than silently exporting nothing.
///
/// Both letter cases are emitted deliberately: curl and most Unix tooling read
/// the lowercase names, while Go, Java and many CI images read the uppercase
/// ones. Setting only one case is the most common cause of "the proxy works in
/// curl but not in my build tool".
#[must_use]
pub fn proxy_env_vars(config: &AppConfig) -> Option<Vec<(&'static str, String)>> {
    let profile = config.active_profile()?;
    let http_url = config.get_http_url()?;
    let socks_url = config.get_socks_url()?;

    let jvm_opts = format!(
        "-Dhttp.proxyHost={host} -Dhttp.proxyPort={port} -Dhttps.proxyHost={host} -Dhttps.proxyPort={port}",
        host = profile.host,
        port = profile.port
    );

    Some(vec![
        ("http_proxy", http_url.clone()),
        ("HTTP_PROXY", http_url.clone()),
        ("https_proxy", http_url.clone()),
        ("HTTPS_PROXY", http_url),
        ("all_proxy", socks_url.clone()),
        ("ALL_PROXY", socks_url),
        ("GRADLE_OPTS", jvm_opts.clone()),
        ("JAVA_TOOL_OPTIONS", jvm_opts),
    ])
}

/// Wraps a value in POSIX single quotes so a shell reads it as one literal.
///
/// This output is consumed by `eval "$(... env on)"`, so an unquoted value from
/// `config.json` would be executed as shell code. A profile host of
/// `x"; touch /tmp/pwned; "` is enough to demonstrate it. Single-quoting with
/// the standard `'\''` escape makes every byte inert, including `$`, backticks,
/// semicolons and newlines.
#[must_use]
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// Renders the `export NAME='value';` lines for the active profile.
#[must_use]
pub fn export_statements(config: &AppConfig) -> Option<Vec<String>> {
    Some(
        proxy_env_vars(config)?
            .into_iter()
            .map(|(name, value)| format!("export {name}={};", shell_quote(&value)))
            .collect(),
    )
}

/// Renders the single `unset ...;` line that clears every managed variable.
#[must_use]
pub fn unset_statement() -> String {
    format!("unset {};", MANAGED_ENV_VARS.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    fn config_with_host(host: &str) -> AppConfig {
        let mut config = AppConfig::default();
        config.profiles.insert(
            "t".to_string(),
            Profile {
                name: "T".to_string(),
                host: host.to_string(),
                port: 1080,
                protocol: "socks5".to_string(),
            },
        );
        config.active_profile = "t".to_string();
        config
    }

    #[test]
    fn exports_both_letter_cases_for_every_proxy_variable() {
        let vars = proxy_env_vars(&config_with_host("127.0.0.1")).unwrap();
        let names: Vec<&str> = vars.iter().map(|(n, _)| *n).collect();

        assert_eq!(names, MANAGED_ENV_VARS);
    }

    #[test]
    fn jvm_options_carry_host_and_port_for_both_schemes() {
        let vars = proxy_env_vars(&config_with_host("10.0.0.1")).unwrap();
        let gradle = &vars.iter().find(|(n, _)| *n == "GRADLE_OPTS").unwrap().1;

        assert_eq!(
            gradle,
            "-Dhttp.proxyHost=10.0.0.1 -Dhttp.proxyPort=1080 \
             -Dhttps.proxyHost=10.0.0.1 -Dhttps.proxyPort=1080"
        );
    }

    #[test]
    fn missing_active_profile_yields_none_instead_of_empty_exports() {
        let config = AppConfig {
            active_profile: "does-not-exist".to_string(),
            ..AppConfig::default()
        };

        assert!(proxy_env_vars(&config).is_none());
        assert!(export_statements(&config).is_none());
    }

    #[test]
    fn unset_clears_exactly_what_export_sets() {
        let statement = unset_statement();

        for name in MANAGED_ENV_VARS {
            assert!(
                statement.contains(name),
                "`env off` would leave {name} set in the user's shell"
            );
        }
    }

    #[test]
    fn an_embedded_quote_is_escaped_rather_than_ending_the_literal() {
        // The generated script is fed to `eval`, so a value that closes its own
        // quote would be executed as shell code instead of stored.
        assert_eq!(shell_quote("a'b"), r"'a'\''b'");
    }

    #[test]
    fn a_hostile_host_stays_inert_when_a_real_shell_evaluates_the_export() {
        // Asserting on the escaped string alone can pass while a shell still
        // splits it, so run the generated line through /bin/sh and read back
        // what the variable actually ended up holding.
        let hostile = "x'; echo PWNED; echo '";
        let line = export_statements(&config_with_host(hostile))
            .unwrap()
            .into_iter()
            .find(|s| s.starts_with("export http_proxy="))
            .unwrap();

        let output = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(format!(r#"{line} printf %s "$http_proxy""#))
            .output()
            .unwrap();
        // Exact equality is the whole assertion: had the payload executed, the
        // shell's own `echo PWNED` output would be prepended to this capture
        // and the variable would hold a truncated value.
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("http://{hostile}:1080")
        );
    }

    #[test]
    fn quoting_leaves_ordinary_values_readable() {
        assert_eq!(
            shell_quote("socks5://127.0.0.1:1080"),
            "'socks5://127.0.0.1:1080'"
        );
    }
}
