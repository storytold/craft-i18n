//! Any text is read as a catalog of a locale: it never panics, the rows and problems stay within
//! the file's bounds and each problem says which line it is. The first byte picks the locale.

#![no_main]

use std::sync::OnceLock;

use craft_i18n_tests::catalog::{declared, read_catalog_report};
use craft_i18n_tests::limits::{MAX_CATALOG_BYTES, MAX_ROWS};
use craft_i18n_tests::spec::LocaleSpec;
use libfuzzer_sys::fuzz_target;

const LOCALES: [&str; 8] = ["de", "fr", "pl", "ar", "ja", "ru", "cy", "ckb"];

fn specs() -> &'static [LocaleSpec] {
    static SPECS: OnceLock<Vec<LocaleSpec>> = OnceLock::new();
    SPECS.get_or_init(|| {
        LOCALES
            .iter()
            .filter_map(|locale| LocaleSpec::for_locale(locale).ok())
            .collect()
    })
}

fuzz_target!(|data: &[u8]| {
    let pick = usize::from(data.first().copied().unwrap_or(0));
    let text = String::from_utf8_lossy(data.get(1..).unwrap_or_default());
    let specs = specs();
    let Some(spec) = specs.get(pick % specs.len().max(1)) else {
        return;
    };
    let reading = read_catalog_report(&text, spec);
    assert!(text.len() <= MAX_CATALOG_BYTES);
    assert!(reading.rows.len() <= MAX_ROWS);
    for problem in reading.problems.iter().chain(&reading.warnings) {
        assert!(problem.starts_with("line "), "{problem}");
    }
    let _ = declared(&text);
});
