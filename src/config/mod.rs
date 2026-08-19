/// Core configuration structures and JSON loading.
pub mod app;
/// Internationalization (i18n) and localization support.
pub mod i18n;
/// Defines the structure of a proxy profile.
pub mod profile;
/// Global application settings (`settings.json`).
pub mod settings;

pub use app::{AppConfig, CONFIG_PATH_ENV, DiagnoseEndpoint, PingTarget, set_config_path_override};
pub use i18n::{I18n, SUPPORTED_LANGS};
pub use profile::{Profile, SUPPORTED_PROTOCOLS};
pub use settings::{AppSettings, SETTINGS_PATH_ENV, set_settings_path_override};
