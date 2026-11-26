//! Any text is parsed as a selector message; a message that parses stays within the limits and is
//! checked against several rows and locales, which never panics and reports a bounded number of
//! faults. The first byte of the input picks the row, the locale and the kind of row.

#![no_main]

use std::collections::BTreeMap;
use std::sync::OnceLock;

use craft_i18n_tests::grammar::Grammar;
use craft_i18n_tests::limits::{
    MAX_DECLARATIONS, MAX_MESSAGE_BYTES, MAX_PARTS, MAX_SELECTORS, MAX_VARIANTS,
};
use craft_i18n_tests::message::{Body, parse};
use craft_i18n_tests::placeholder::parse_source;
use craft_i18n_tests::select::{RowInfo, RowKind, check};
use craft_i18n_tests::spec::LocaleSpec;
use libfuzzer_sys::fuzz_target;

const LOCALES: [&str; 8] = ["de", "fr", "pl", "ar", "ja", "ru", "cy", "ckb"];
const SOURCES: [&str; 6] = [
    "{n} a",
    "{n :integer} {item :noun}",
    "{span :range}",
    "{when :date} {n :number}",
    "Open {name}…",
    "{n :measure} {item :noun}",
];

fn specs() -> &'static [LocaleSpec] {
    static SPECS: OnceLock<Vec<LocaleSpec>> = OnceLock::new();
    SPECS.get_or_init(|| {
        LOCALES
            .iter()
            .filter_map(|locale| LocaleSpec::for_locale(locale).ok())
            .map(|spec| {
                let grammar = Grammar {
                    genders: vec!["masculine".into(), "feminine".into(), "neuter".into()],
                    forms: vec!["accusative".into(), "genitive".into()],
                };
                spec.with_declared(grammar, BTreeMap::new())
            })
            .collect()
    })
}

fuzz_target!(|data: &[u8]| {
    let pick = usize::from(data.first().copied().unwrap_or(0));
    let text = String::from_utf8_lossy(data.get(1..).unwrap_or_default());
    let Ok(message) = parse(&text) else {
        return;
    };
    assert!(text.len() <= MAX_MESSAGE_BYTES);
    assert!(message.declarations.len() <= MAX_DECLARATIONS);
    match &message.body {
        Body::Pattern(pattern) => assert!(pattern.parts.len() <= MAX_PARTS),
        Body::Match {
            selectors,
            variants,
        } => {
            assert!(selectors.len() <= MAX_SELECTORS && variants.len() <= MAX_VARIANTS);
            for variant in variants {
                assert_eq!(variant.keys.len(), selectors.len());
                assert!(variant.pattern.parts.len() <= MAX_PARTS);
            }
        }
    }
    let specs = specs();
    let Some(spec) = specs.get(pick % specs.len().max(1)) else {
        return;
    };
    let source_text = SOURCES[(pick / 8) % SOURCES.len()];
    let kind = [RowKind::Plain, RowKind::Plural, RowKind::Ordinal][(pick / 48) % 3];
    let Ok(source) = parse_source(source_text) else {
        return;
    };
    let info = RowInfo {
        kind,
        source: &source,
        ellipsis: source_text.ends_with('…'),
    };
    let outcome = check(&message, info, spec);
    assert!(outcome.errors.len() <= 20);
    assert_eq!(outcome, check(&message, info, spec));
});
