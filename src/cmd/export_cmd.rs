use crate::config::AppConfig;
use anyhow::Result;

/// Exports proxy configuration for external build tools (Docker, cURL, .env).
pub fn export_config(format: &str, config: &AppConfig) -> Result<()> {
    match format.to_lowercase().as_str() {
        "docker" => {
            if let Some(http) = config.get_http_url() {
                println!(
                    "--build-arg http_proxy=\"{}\" --build-arg https_proxy=\"{}\"",
                    http, http
                );
            }
        }
        "curl" => {
            if let Some(socks) = config.get_socks_url() {
                println!("-x {}", socks);
            }
        }
        "envfile" | "env" => {
            if let (Some(http), Some(socks)) = (config.get_http_url(), config.get_socks_url()) {
                println!("HTTP_PROXY={}", http);
                println!("HTTPS_PROXY={}", http);
                println!("ALL_PROXY={}", socks);
            }
        }
        _ => {
            eprintln!("Unknown format. Supported formats: docker, curl, envfile");
        }
    }
    Ok(())
}
