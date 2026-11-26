//! What the checker knows of one locale before it reads a row: its plural rules, whether it is
//! written right to left, and what its catalogs declare (`@grammar`, `@noun`).

use std::collections::BTreeMap;

use crate::grammar::{Grammar, Noun};
use crate::locale::plural_forms;
use crate::plural::{Category, Plurals, plurals};

/// The languages written right to left. Only they may carry the marks LRM, RLM and ALM.
const RTL: &[&str] = &["ar", "ckb", "dv", "fa", "he", "ps", "sd", "ug", "ur", "yi"];

/// Whether the locale's language is written right to left.
#[must_use]
pub fn is_rtl(locale: &str) -> bool {
    let language = locale.split('-').next().unwrap_or_default();
    RTL.contains(&language)
}

/// One locale, as a catalog of it is read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleSpec {
    pub locale: String,
    pub plurals: Plurals,
    pub rtl: bool,
    /// What the file being read sees declared: its own `@grammar` rows, else the locale's shared
    /// ones, else its base language's.
    pub grammar: Grammar,
    /// The nouns the file sees, by English noun: its own, else the shared ones, else the base
    /// language's. Only the warnings about missing forms use them.
    pub nouns: BTreeMap<String, Noun>,
}

impl LocaleSpec {
    /// The spec of a locale with nothing declared.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime crate has no plural rules for `locale`, or its form count
    /// and the categories read here disagree.
    pub fn for_locale(locale: &str) -> Result<LocaleSpec, String> {
        let forms = plural_forms(locale)?;
        let mut plurals = plurals(locale)?;
        if plurals.cardinal.reached.len() != forms {
            // The runtime crate carries every CLDR locale; ICU4X's compiled data leaves a few out
            // and answers `other` for them. Two forms can only be `one` and `other`.
            if forms == 2 && plurals.cardinal.defined == [Category::Other] {
                plurals = Plurals::one_and_other();
            } else {
                return Err(format!(
                    "{locale:?}: the runtime crate takes {forms} plural forms but CLDR's cardinal categories {:?} reached by 0 to 1000 are {}",
                    plurals
                        .cardinal
                        .reached
                        .iter()
                        .map(|c| c.name())
                        .collect::<Vec<_>>(),
                    plurals.cardinal.reached.len()
                ));
            }
        }
        Ok(LocaleSpec {
            locale: locale.to_owned(),
            plurals,
            rtl: is_rtl(locale),
            grammar: Grammar::default(),
            nouns: BTreeMap::new(),
        })
    }

    /// The stand-in for a locale the runtime has no rules for: one plural form.
    #[must_use]
    pub fn unknown(locale: &str) -> LocaleSpec {
        LocaleSpec {
            locale: locale.to_owned(),
            plurals: Plurals::only_other(),
            rtl: is_rtl(locale),
            grammar: Grammar::default(),
            nouns: BTreeMap::new(),
        }
    }

    /// The same locale with what a file sees declared.
    #[must_use]
    pub fn with_declared(mut self, grammar: Grammar, nouns: BTreeMap<String, Noun>) -> LocaleSpec {
        self.grammar = grammar;
        self.nouns = nouns;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_to_left_languages() {
        for rtl in ["ar", "he", "fa", "ur", "ckb", "dv", "ar-eg"] {
            assert!(is_rtl(rtl), "{rtl}");
        }
        for ltr in ["fr", "pt-br", "zh-hans", "sr-latn", "ja", "", "hi"] {
            assert!(!is_rtl(ltr), "{ltr}");
        }
    }

    #[test]
    fn a_locale_agrees_with_the_runtime_crate() {
        for locale in [
            "ja", "de", "fr", "pl", "ar", "ru", "pt-br", "pt-pt", "sl", "cy", "he",
        ] {
            let spec = LocaleSpec::for_locale(locale).unwrap();
            assert_eq!(
                Ok(spec.plurals.cardinal.reached.len()),
                plural_forms(locale),
                "{locale}"
            );
        }
    }

    #[test]
    fn locales_icu_lacks_take_the_cautious_two_forms() {
        for locale in ["ckb", "dv"] {
            let spec = LocaleSpec::for_locale(locale).unwrap();
            assert_eq!(spec.plurals, Plurals::one_and_other(), "{locale}");
        }
    }

    #[test]
    fn a_locale_without_rules_is_an_error() {
        assert!(LocaleSpec::for_locale("tlh").is_err());
        assert!(LocaleSpec::for_locale("not a locale").is_err());
        assert_eq!(LocaleSpec::unknown("tlh").plurals.cardinal.reached.len(), 1);
    }
}
