use crate::cli::ExportFormat;
use crate::config::AppConfig;
use anyhow::Result;

/// Exports proxy configuration for external build tools (Docker, cURL, .env).
pub fn export_config(format: &ExportFormat, config: &AppConfig) -> Result<()> {
    match format {
        ExportFormat::Docker => {
            if let Some(http) = config.get_http_url() {
                println!(
                    "--build-arg http_proxy=\"{}\" --build-arg https_proxy=\"{}\"",
                    http, http
                );
            }
        }
        ExportFormat::Curl => {
            if let Some(socks) = config.get_socks_url() {
                println!("-x {}", socks);
            }
        }
        ExportFormat::Envfile => {
            if let (Some(http), Some(socks)) = (config.get_http_url(), config.get_socks_url()) {
                println!("HTTP_PROXY={}", http);
                println!("HTTPS_PROXY={}", http);
                println!("ALL_PROXY={}", socks);
            }
        }
    }
    Ok(())
}
