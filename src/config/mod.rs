/// Core configuration structures and JSON loading.
pub mod app;
/// Internationalization (i18n) and localization support.
pub mod i18n;
/// Defines the structure of a proxy profile.
pub mod profile;
/// Global application settings (`settings.json`).
pub mod settings;

pub use app::{AppConfig, DiagnoseEndpoint, PingTarget};
pub use i18n::I18n;
pub use profile::Profile;
pub use settings::AppSettings;
