use std::path::Path;

use core_infra::FluentTranslator;
use core_lib::i18n::{ArgValue, Locale, Translator};
use core_lib::tr;
use unic_langid::LanguageIdentifier;

fn load_translator() -> FluentTranslator {
    let locales = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("locales");
    FluentTranslator::load(&locales).expect("failed to load locales")
}

fn locale(tag: &str) -> Locale {
    Locale(tag.parse::<LanguageIdentifier>().unwrap())
}

#[test]
fn known_key_en_us() {
    let t = load_translator();
    let result = tr!(t, &locale("en-US"), "active-tasks-header");
    assert_eq!(result, "Active tasks");
}

#[test]
fn known_key_ja_jp() {
    let t = load_translator();
    let result = tr!(t, &locale("ja-JP"), "active-tasks-header");
    assert_eq!(result, "アクティブなタスク");
}

#[test]
fn fallback_unknown_locale_returns_en_us() {
    let t = load_translator();
    // fr-FR is not in locales/ — should fall back to en-US.
    let result = tr!(t, &locale("fr-FR"), "active-tasks-header");
    assert_eq!(result, "Active tasks");
}

#[test]
fn missing_key_returns_key_string() {
    let t = load_translator();
    let result = tr!(t, &locale("en-US"), "no-such-key-xyz");
    assert_eq!(result, "no-such-key-xyz");
}

#[test]
fn args_substitution_notification() {
    let t = load_translator();
    let result = t.t(
        &locale("en-US"),
        "notification-stale-task",
        &[
            ("role", ArgValue::from("123456")),
            ("title", ArgValue::from("Fix login bug")),
            ("thread", ArgValue::from("789012")),
        ],
    );
    assert!(result.contains("123456"), "should contain role id");
    assert!(
        result.contains("Fix login bug"),
        "should contain task title"
    );
    assert!(result.contains("789012"), "should contain thread id");
}
