use serde_json::Value;
use std::collections::HashMap;

const RU_JSON: &str = include_str!("../../locales/ru.json");
const EN_JSON: &str = include_str!("../../locales/en.json");

/// Language codes the interface ships translations for.
pub const SUPPORTED_LANGS: &[&str] = &["ru", "en"];

/// Internationalization manager for loading locale dictionaries.
#[derive(Debug, Clone)]
pub struct I18n {
    lang: String,
    messages: HashMap<String, String>,
}

impl I18n {
    /// Loads the dictionary for `lang`, falling back to Russian.
    ///
    /// The locale files are embedded at compile time, so a translation can
    /// never be missing at runtime because of a bad install.
    #[must_use]
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

    /// Looks up a localized message, returning the key itself if it is missing.
    ///
    /// Returning the key keeps the program running, but it also means a
    /// forgotten translation surfaces as raw `snake_case` in the UI — see the
    /// locale coverage tests, which exist to catch that before a release does.
    #[must_use]
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.messages.get(key).map_or(key, String::as_str)
    }

    /// Looks up a message and substitutes `{}` placeholders left to right.
    ///
    /// Replaces the `.replacen("{}", a, 1).replacen("{}", b, 1)` chains that
    /// were repeated at call sites, where an argument in the wrong position
    /// produced a plausible-looking but wrong sentence.
    #[must_use]
    pub fn format(&self, key: &str, args: &[&str]) -> String {
        let mut out = self.t(key).to_string();
        for arg in args {
            out = out.replacen("{}", arg, 1);
        }
        out
    }

    /// Gets active language code.
    #[must_use]
    pub fn lang(&self) -> &str {
        &self.lang
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses a locale file the same way [`I18n::load`] does.
    fn keys(raw: &str) -> std::collections::BTreeSet<String> {
        let Ok(Value::Object(map)) = serde_json::from_str::<Value>(raw) else {
            panic!("locale file is not a JSON object");
        };
        map.into_iter()
            .filter(|(_, v)| matches!(v, Value::String(_)))
            .map(|(k, _)| k)
            .collect()
    }

    #[test]
    fn both_locales_are_valid_json_objects_of_strings() {
        assert!(!keys(RU_JSON).is_empty());
        assert!(!keys(EN_JSON).is_empty());
    }

    #[test]
    fn the_two_locales_define_exactly_the_same_keys() {
        // A key present in one file and absent from the other renders as a raw
        // `snake_case` identifier for half the userbase, which is how
        // `cmd_debug` shipped untranslated.
        let (ru, en) = (keys(RU_JSON), keys(EN_JSON));

        let only_ru: Vec<_> = ru.difference(&en).collect();
        let only_en: Vec<_> = en.difference(&ru).collect();

        assert!(only_ru.is_empty(), "missing from en.json: {only_ru:?}");
        assert!(only_en.is_empty(), "missing from ru.json: {only_en:?}");
    }

    #[test]
    fn no_translation_is_left_empty() {
        for (lang, raw) in [("ru", RU_JSON), ("en", EN_JSON)] {
            let i18n = I18n::load(lang);
            for key in keys(raw) {
                assert!(
                    !i18n.t(&key).trim().is_empty(),
                    "{lang}.json has an empty value for '{key}'"
                );
            }
        }
    }

    #[test]
    fn every_supported_lang_loads_a_non_empty_dictionary() {
        for lang in SUPPORTED_LANGS {
            assert!(!I18n::load(lang).messages.is_empty());
        }
    }

    #[test]
    fn an_unknown_language_falls_back_to_russian() {
        let fallback = I18n::load("kl");
        let russian = I18n::load("ru");

        assert_eq!(fallback.t("cmd_status"), russian.t("cmd_status"));
    }

    #[test]
    fn language_matching_ignores_case() {
        assert_eq!(
            I18n::load("EN").t("cmd_status"),
            I18n::load("en").t("cmd_status")
        );
    }

    #[test]
    fn a_missing_key_returns_the_key_itself() {
        assert_eq!(I18n::load("en").t("no_such_key"), "no_such_key");
    }

    #[test]
    fn format_substitutes_placeholders_in_order() {
        let i18n = I18n::load("en");
        assert_eq!(i18n.format("no_such_{}_{}", &["a", "b"]), "no_such_a_b");
    }

    #[test]
    fn format_leaves_surplus_placeholders_alone_rather_than_panicking() {
        let i18n = I18n::load("en");
        assert_eq!(i18n.format("{}_{}", &["only"]), "only_{}");
    }

    #[test]
    fn lang_reports_what_was_requested() {
        assert_eq!(I18n::load("en").lang(), "en");
    }
}
