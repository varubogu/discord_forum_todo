use std::{borrow::Cow, collections::HashMap, path::Path};

use fluent_bundle::concurrent::FluentBundle as ConcurrentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

use core_lib::i18n::{ArgValue, Locale, TArgs, Translator};

/// Error returned by [`FluentTranslator::load`].
#[derive(Debug, thiserror::Error)]
pub enum TranslatorLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("FTL parse error in {path}: {errors}")]
    FtlParse { path: String, errors: String },
    #[error("FTL duplicate key in {path}: {errors}")]
    FtlBundle { path: String, errors: String },
}

/// Production [`Translator`] backed by fluent-rs.
///
/// Load once at startup via [`FluentTranslator::load`].
pub struct FluentTranslator {
    bundles: HashMap<LanguageIdentifier, ConcurrentBundle<FluentResource>>,
    fallback: LanguageIdentifier,
}

impl FluentTranslator {
    /// Walk `locales_dir/{locale}/*.ftl` and build one [`ConcurrentBundle`] per locale.
    ///
    /// Sub-directories whose names cannot be parsed as BCP-47 tags are silently
    /// skipped. Returns an error on any IO or FTL parse failure.
    pub fn load(locales_dir: &Path) -> Result<Self, TranslatorLoadError> {
        let mut bundles: HashMap<LanguageIdentifier, ConcurrentBundle<FluentResource>> =
            HashMap::new();
        let fallback: LanguageIdentifier = "en-US".parse().expect("hardcoded locale is valid");

        for entry in std::fs::read_dir(locales_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let locale_str = entry.file_name().to_string_lossy().to_string();
            let Ok(lang_id) = locale_str.parse::<LanguageIdentifier>() else {
                continue;
            };

            let mut bundle = ConcurrentBundle::new_concurrent(vec![lang_id.clone()]);
            for ftl in std::fs::read_dir(entry.path())? {
                let ftl = ftl?;
                let ftl_path = ftl.path();
                if ftl_path.extension().is_none_or(|e| e != "ftl") {
                    continue;
                }
                let src = std::fs::read_to_string(&ftl_path)?;
                let resource = FluentResource::try_new(src).map_err(|(_, errs)| {
                    TranslatorLoadError::FtlParse {
                        path: ftl_path.display().to_string(),
                        errors: format!("{errs:?}"),
                    }
                })?;
                bundle
                    .add_resource(resource)
                    .map_err(|errs| TranslatorLoadError::FtlBundle {
                        path: ftl_path.display().to_string(),
                        errors: format!("{errs:?}"),
                    })?;
            }
            bundles.insert(lang_id, bundle);
        }

        Ok(Self { bundles, fallback })
    }
}

impl Translator for FluentTranslator {
    fn t(&self, locale: &Locale, key: &str, args: TArgs<'_>) -> String {
        // Build FluentArgs from TArgs (clone strings to keep lifetimes simple).
        let fa: Option<FluentArgs<'_>> = if args.is_empty() {
            None
        } else {
            let mut fa = FluentArgs::new();
            for (name, val) in args {
                let fv: FluentValue<'_> = match val {
                    ArgValue::String(s) => FluentValue::String(Cow::Owned(s.as_ref().to_owned())),
                    ArgValue::Number(n) => FluentValue::Number((*n).into()),
                };
                fa.set(*name, fv);
            }
            Some(fa)
        };

        // Try requested locale, then fall back to en-US.
        let langs: [&LanguageIdentifier; 2] = [&locale.0, &self.fallback];
        for lang in langs {
            let Some(bundle) = self.bundles.get(lang) else {
                continue;
            };
            let Some(msg) = bundle.get_message(key) else {
                continue;
            };
            let Some(pattern) = msg.value() else {
                continue;
            };
            let mut errors = vec![];
            let result = bundle.format_pattern(pattern, fa.as_ref(), &mut errors);
            if !errors.is_empty() {
                tracing::warn!(key, ?errors, "fluent format errors");
            }
            return result.into_owned();
        }

        tracing::warn!(key, "missing translation key");
        key.to_string()
    }
}
