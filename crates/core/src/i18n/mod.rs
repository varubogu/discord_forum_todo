//! i18n abstractions. The actual Fluent-backed implementation lives in
//! `core_infra`. This module owns only the trait, the locale type, and helpers.

use std::borrow::Cow;

use unic_langid::LanguageIdentifier;

/// A locale identifier, e.g. `ja-JP` or `en-US`. The fallback is `en-US`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale(pub LanguageIdentifier);

impl Locale {
    pub fn fallback() -> Self {
        Self(unic_langid::langid!("en-US"))
    }
}

/// A single argument value passed to a translation.
#[derive(Debug, Clone)]
pub enum ArgValue<'a> {
    String(Cow<'a, str>),
    Number(f64),
}

impl<'a> From<&'a str> for ArgValue<'a> {
    fn from(s: &'a str) -> Self {
        Self::String(Cow::Borrowed(s))
    }
}

impl From<String> for ArgValue<'static> {
    fn from(s: String) -> Self {
        Self::String(Cow::Owned(s))
    }
}

impl From<f64> for ArgValue<'static> {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i64> for ArgValue<'static> {
    fn from(n: i64) -> Self {
        #[allow(clippy::cast_precision_loss)]
        Self::Number(n as f64)
    }
}

/// Slice of named arguments for a translation call.
pub type TArgs<'a> = &'a [(&'a str, ArgValue<'a>)];

/// Port: resolves message keys against a locale.
/// Production implementation (`FluentTranslator`) lives in `core_infra`.
pub trait Translator: Send + Sync {
    fn t(&self, locale: &Locale, key: &str, args: TArgs<'_>) -> String;
}

/// Locales supported by this bot.
pub const SUPPORTED_LOCALES: &[&str] = &["en-US", "ja-JP"];

/// Convenience macro for calling [`Translator::t`] with named arguments.
///
/// ```ignore
/// // No arguments
/// tr!(translator, &locale, "some-key")
/// // With arguments
/// tr!(translator, &locale, "some-key", role = role_str.as_str(), count = 3_i64)
/// ```
#[macro_export]
macro_rules! tr {
    ($t:expr, $l:expr, $key:expr) => {
        $t.t($l, $key, &[])
    };
    ($t:expr, $l:expr, $key:expr, $($n:ident = $v:expr),+ $(,)?) => {
        $t.t($l, $key, &[$(
            (::core::stringify!($n), $crate::i18n::ArgValue::from($v))
        ),+])
    };
}
