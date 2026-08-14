use serde_json::Value;
use std::collections::HashMap;

const RU_JSON: &str = include_str!("../../locales/ru.json");
const EN_JSON: &str = include_str!("../../locales/en.json");

/// Internationalization manager for loading locale dictionaries.
#[derive(Debug, Clone)]
pub struct I18n {
    lang: String,
    messages: HashMap<String, String>,
}

impl I18n {
    /// Loads the specified language dictionary ("ru" or "en"). Defaults to "ru".
    pub fn load(lang: &str) -> Self {
        let json_str = match lang.to_lowercase().as_str() {
            "en" => EN_JSON,
            _ => RU_JSON,
        };

        let mut messages = HashMap::new();
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(json_str) {
            for (k, v) in map {
                if let Value::String(s) = v {
                    messages.insert(k, s);
                }
            }
        }

        Self {
            lang: lang.to_string(),
            messages,
        }
    }

    /// Gets a localized string message by key.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.messages.get(key).map_or(key, String::as_str)
    }

    /// Gets active language code.
    pub fn lang(&self) -> &str {
        &self.lang
    }
}
