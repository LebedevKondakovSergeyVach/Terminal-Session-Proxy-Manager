use crate::cli::ExportFormat;
use crate::config::{AppConfig, I18n};
use anyhow::{Result, anyhow};

/// Exports the active proxy configuration for external tools.
///
/// # Errors
/// Returns an error if no profile is active, so a shell pipeline such as
/// `proxy export envfile > .env` fails loudly instead of truncating the file
/// to nothing.
pub fn export_config(format: &ExportFormat, config: &AppConfig, i18n: &I18n) -> Result<()> {
    let (Some(http), Some(proxy)) = (config.get_http_url(), config.get_socks_url()) else {
        return Err(anyhow!("{}", i18n.t("proxy_load_failed")));
    };

    match format {
        ExportFormat::Docker => {
            println!("--build-arg http_proxy=\"{http}\" --build-arg https_proxy=\"{http}\"");
        }
        ExportFormat::Curl => println!("-x {proxy}"),
        ExportFormat::Envfile => {
            println!("HTTP_PROXY={http}");
            println!("HTTPS_PROXY={http}");
            println!("ALL_PROXY={proxy}");
        }
    }
    Ok(())
}
