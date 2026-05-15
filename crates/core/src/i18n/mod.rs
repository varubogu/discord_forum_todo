//! i18n abstractions. The actual Fluent-backed implementation lives in
//! `core_infra`. This module owns only the trait, the locale type, and helpers.

use unic_langid::LanguageIdentifier;

/// A locale identifier, e.g. `ja-JP` or `en-US`. The fallback is `en-US`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale(pub LanguageIdentifier);

impl Locale {
    pub fn fallback() -> Self {
        Self(unic_langid::langid!("en-US"))
    }
}

/// Port: resolves message keys against a locale. Full implementation in Phase 8.
pub trait Translator: Send + Sync {
    fn t(&self, locale: &Locale, key: &str) -> String;
}
