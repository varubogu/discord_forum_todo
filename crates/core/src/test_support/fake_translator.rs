use crate::i18n::{Locale, Translator};

/// A trivial translator that echoes the key. Useful when a test only cares
/// that *some* output is produced.
#[derive(Default, Clone)]
pub struct FakeTranslator;

impl FakeTranslator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Translator for FakeTranslator {
    fn t(&self, _locale: &Locale, key: &str) -> String {
        key.to_string()
    }
}
