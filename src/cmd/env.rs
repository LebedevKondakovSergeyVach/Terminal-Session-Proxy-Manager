use crate::cli::EnvMode;
use crate::config::{AppConfig, I18n};
use crate::proxy_env::{export_statements, shell_quote, unset_statement};

/// Prints shell statements for `eval "$(... env on|off)"`.
///
/// Everything printed here is executed by the caller's shell, so every
/// interpolated value goes through [`shell_quote`] — including the status
/// message, which embeds a profile name straight from `config.json`.
pub fn print_env_commands(mode: &EnvMode, config: &AppConfig, i18n: &I18n) {
    match mode {
        EnvMode::On => {
            let Some(statements) = export_statements(config) else {
                // Print to stderr so the shell evaluates nothing at all rather
                // than a half-built environment.
                eprintln!("{}", i18n.t("proxy_load_failed"));
                return;
            };

            for statement in statements {
                println!("{statement}");
            }

            if let Some(profile) = config.active_profile() {
                let message = i18n.format(
                    "env_on_msg",
                    &[&profile.name, &profile.host, &profile.port.to_string()],
                );
                println!("echo {};", shell_quote(&message));
            }
        }
        EnvMode::Off => {
            println!("{}", unset_statement());
            println!("echo {};", shell_quote(i18n.t("env_off_msg")));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    fn config() -> AppConfig {
        let mut config = AppConfig::default();
        config.profiles.insert(
            "p".to_string(),
            Profile {
                name: "P".to_string(),
                host: "127.0.0.1".to_string(),
                port: 9050,
                protocol: "socks5".to_string(),
            },
        );
        config.active_profile = "p".to_string();
        config
    }

    #[test]
    fn on_exports_every_managed_variable_for_the_active_profile() {
        let statements = export_statements(&config()).unwrap();
        let script = statements.join("\n");

        assert!(script.contains("export http_proxy='http://127.0.0.1:9050';"));
        assert!(script.contains("export ALL_PROXY='socks5://127.0.0.1:9050';"));
        assert!(script.contains("JAVA_TOOL_OPTIONS"));
    }

    #[test]
    fn off_unsets_both_letter_cases() {
        let statement = unset_statement();

        assert!(statement.contains("http_proxy"));
        assert!(statement.contains("HTTP_PROXY"));
        assert!(statement.contains("all_proxy"));
        assert!(statement.contains("ALL_PROXY"));
    }
}
