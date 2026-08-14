use crate::cli::EnvMode;
use crate::config::{AppConfig, I18n};

/// Prints shell environment export or unset statements for `eval`.
pub fn print_env_commands(mode: &EnvMode, config: &AppConfig, i18n: &I18n) {
    match mode {
        EnvMode::On => {
            if let (Some(http_url), Some(socks_url), Some(profile)) = (
                config.get_http_url(),
                config.get_socks_url(),
                config.active_profile(),
            ) {
                println!("export http_proxy=\"{}\";", http_url);
                println!("export https_proxy=\"{}\";", http_url);
                println!("export ALL_PROXY=\"{}\";", socks_url);
                println!(
                    "export GRADLE_OPTS=\"-Dhttp.proxyHost={} -Dhttp.proxyPort={} -Dhttps.proxyHost={} -Dhttps.proxyPort={}\";",
                    profile.host, profile.port, profile.host, profile.port
                );
                println!(
                    "export JAVA_TOOL_OPTIONS=\"-Dhttp.proxyHost={} -Dhttp.proxyPort={} -Dhttps.proxyHost={} -Dhttps.proxyPort={}\";",
                    profile.host, profile.port, profile.host, profile.port
                );
                let msg = i18n
                    .t("env_on_msg")
                    .replacen("{}", &profile.name, 1)
                    .replacen("{}", &profile.host, 1)
                    .replacen("{}", &profile.port.to_string(), 1);
                println!("echo \"{}\";", msg);
            }
        }
        EnvMode::Off => {
            println!("unset http_proxy HTTP_PROXY https_proxy HTTPS_PROXY ALL_PROXY all_proxy GRADLE_OPTS JAVA_TOOL_OPTIONS;");
            println!("echo \"{}\";", i18n.t("env_off_msg"));
        }
    }
}
