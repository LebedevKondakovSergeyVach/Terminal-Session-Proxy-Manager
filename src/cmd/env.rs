use crate::config::AppConfig;

/// Prints shell environment export or unset statements for `eval`.
pub fn print_env_commands(mode: &str, config: &AppConfig) {
    match mode {
        "on" => {
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
                println!(
                    "echo \"🚀 Прокси [{}] ВКЛЮЧЕН для текущей вкладки [{}:{}]\";",
                    profile.name, profile.host, profile.port
                );
            }
        }
        "off" => {
            println!("unset http_proxy;");
            println!("unset https_proxy;");
            println!("unset ALL_PROXY;");
            println!("unset GRADLE_OPTS;");
            println!("unset JAVA_TOOL_OPTIONS;");
            println!("echo \"🛑 Прокси ВЫКЛЮЧЕН\";");
        }
        _ => {}
    }
}
