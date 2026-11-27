//! Catalogs are input. The selector-message parser, the placeholder scanner, the grammar rows and
//! the catalog reader take any text: none panics, each stays within `limits`, and what the checker
//! passes is what it claims (no whole number from 0 to 1000 falls to `*` unseen).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use craft_i18n_tests::catalog::{declared, read_catalog_report};
use craft_i18n_tests::grammar::{parse_grammar_row, parse_noun};
use craft_i18n_tests::limits::{
    MAX_DECLARATIONS, MAX_MESSAGE_BYTES, MAX_PARTS, MAX_ROWS, MAX_SELECTORS, MAX_VARIANTS,
};
use craft_i18n_tests::message::{self, Body, Key, Message, parse};
use craft_i18n_tests::placeholder::{self, parse_source, scan, translation_names};
use craft_i18n_tests::plural::{Category, REACH};
use craft_i18n_tests::select::{RowInfo, RowKind, check};
use craft_i18n_tests::spec::LocaleSpec;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    }
}

/// The locales the checks run under, each built once.
fn specs() -> &'static BTreeMap<&'static str, LocaleSpec> {
    static SPECS: OnceLock<BTreeMap<&'static str, LocaleSpec>> = OnceLock::new();
    SPECS.get_or_init(|| {
        [
            "de", "fr", "pl", "ar", "ja", "ru", "cy", "sl", "he", "lv", "ckb",
        ]
        .into_iter()
        .map(|locale| {
            let spec = LocaleSpec::for_locale(locale).unwrap();
            let grammar = craft_i18n_tests::grammar::Grammar {
                genders: vec!["masculine".into(), "feminine".into(), "neuter".into()],
                forms: vec!["accusative".into(), "genitive".into()],
            };
            (locale, spec.with_declared(grammar, BTreeMap::new()))
        })
        .collect()
    })
}

fn soup(tokens: &'static [&'static str], longest: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(tokens), 0..longest).prop_map(|v| v.concat())
}

/// Pieces of a selector message, most of them out of place.
const MESSAGE_TOKENS: &[&str] = &[
    ".input",
    ".local",
    ".match",
    " ",
    " ",
    "{",
    "}",
    "{{",
    "}}",
    "$n",
    "$item",
    "$x",
    ":number",
    ":integer",
    ":noun",
    ":range",
    ":measure",
    ":date",
    "select=ordinal",
    "select=plural",
    "form=accusative",
    "=",
    "*",
    "0",
    "1",
    "2",
    "one",
    "few",
    "many",
    "feminine",
    "\\",
    "\\{",
    "}}}",
    "…",
    "x",
    "|",
    "\n",
    "é",
    "#",
    "/",
    "{#b}",
    "999999999",
    "9999999999",
];

/// Pieces of a source or a translation: placeholders typed and not.
const TEXT_TOKENS: &[&str] = &[
    "{",
    "}",
    "{n}",
    "{n :number}",
    "{n :integer}",
    "{item :noun}",
    "{x :wat}",
    "{ :integer}",
    " :",
    "|",
    "…",
    "a",
    "é",
    " ",
    "\\",
    "\\n",
    "\u{202E}",
    "\u{200F}",
    ".match",
    ".input $n",
    "{$n}",
    "{{",
    "}}",
];

fn arbitrary_text() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<String>(),
        soup(TEXT_TOKENS, 40),
        soup(MESSAGE_TOKENS, 40)
    ]
}

fn assert_within_limits(message: &Message) {
    assert!(message.declarations.len() <= MAX_DECLARATIONS);
    let patterns: Vec<&message::Pattern> = match &message.body {
        Body::Pattern(pattern) => vec![pattern],
        Body::Match {
            selectors,
            variants,
        } => {
            assert!((1..=MAX_SELECTORS).contains(&selectors.len()));
            assert!((1..=MAX_VARIANTS).contains(&variants.len()));
            for variant in variants {
                assert_eq!(variant.keys.len(), selectors.len());
            }
            variants.iter().map(|v| &v.pattern).collect()
        }
    };
    for pattern in patterns {
        assert!((1..=MAX_PARTS).contains(&pattern.parts.len()));
    }
}

proptest! {
    #![proptest_config(config(2000))]

    #[test]
    fn parse_never_panics_and_stays_within_the_limits(text in arbitrary_text()) {
        let first = parse(&text);
        match &first {
            Ok(message) => assert_within_limits(message),
            Err(e) => prop_assert!(e.at <= text.chars().count().saturating_add(1)),
        }
        prop_assert_eq!(first, parse(&text), "parse is a function of its text");
    }

    #[test]
    fn a_message_of_any_size_is_refused_or_small(text in soup(MESSAGE_TOKENS, 4000)) {
        if let Ok(message) = parse(&text) {
            prop_assert!(text.len() <= MAX_MESSAGE_BYTES);
            assert_within_limits(&message);
        }
    }

    #[test]
    fn a_valid_message_with_one_edit_never_panics(
        which in 0..SEEDS.len(),
        at in 0..400_usize,
        edit in 0..3_u8,
        piece in prop::sample::select(MESSAGE_TOKENS),
    ) {
        let seed = SEEDS[which];
        let mut chars: Vec<char> = seed.chars().collect();
        let at = at % (chars.len() + 1);
        match edit {
            0 if at < chars.len() => { chars.remove(at); }
            1 => { for (k, c) in piece.chars().enumerate() { chars.insert(at + k, c); } }
            _ if at < chars.len() => { chars.truncate(at); }
            _ => {}
        }
        let text: String = chars.into_iter().collect();
        if let Ok(message) = parse(&text) {
            assert_within_limits(&message);
        }
    }

    #[test]
    fn placeholders_never_panic(text in arbitrary_text()) {
        for p in scan(&text) {
            prop_assert!(text.contains(p.name));
        }
        let _ = parse_source(&text);
        let _ = translation_names(&text);
        let _ = placeholder::is_identifier(&text);
    }

    #[test]
    fn grammar_rows_never_panic(a in arbitrary_text(), b in arbitrary_text()) {
        let _ = parse_grammar_row(&a, &b);
        let _ = parse_noun(&a, &b);
    }

    #[test]
    fn a_catalog_of_any_text_reads_within_its_bounds(
        lines in prop::collection::vec(
            (
                prop::sample::select(&["", "@plural", "@ordinal", "@id", "@msg", "@noun", "@grammar", "menu", "@x"][..]),
                arbitrary_text(),
                arbitrary_text(),
            ),
            0..30,
        ),
        locale in prop::sample::select(&["de", "fr", "ja", "ar", "pl"][..]),
    ) {
        let text: String = lines
            .iter()
            .map(|(context, source, translation)| {
                let flat = |s: &str| s.replace(['\t', '\n', '\r'], " ");
                format!("{context}\t{}\t{}\n", flat(source), flat(translation))
            })
            .collect();
        let spec = &specs()[locale];
        let reading = read_catalog_report(&text, spec);
        let total = text.lines().count();
        prop_assert!(reading.rows.len() <= total.min(MAX_ROWS));
        prop_assert!(reading.rows.len() + reading.problems.len() <= total);
        for row in &reading.rows {
            prop_assert!(row.line >= 1 && row.line <= total);
        }
        for problem in &reading.problems {
            prop_assert!(problem.starts_with("line "), "{problem}");
        }
        for warning in &reading.warnings {
            prop_assert!(warning.starts_with("line "), "{warning}");
        }
        let _ = declared(&text);
    }
}

/// Well-formed messages the edits start from.
const SEEDS: &[&str] = &[
    ".input {$n :number} .match $n zero {{none}} one {{one}} two {{two}} few {{{$n} few}} many {{{$n} many}} * {{{$n}}}",
    ".input {$n :number select=ordinal} .match $n one {{{$n}re}} * {{{$n}e}}",
    ".input {$item :noun} .match $item feminine {{Diese {$item :noun form=accusative}}} * {{Dieses {$item}}}",
    ".input {$item :noun} .local $of = {$item :noun form=genitive} {{Von {$of}}}",
    ".input {$n :integer} .input {$item :noun} .match $n $item 1 feminine {{a {$item}}} 1 * {{b {$item}}} * * {{{$n} {$item}}}",
    ".input {$span :range} .match $span one {{a {$span}}} * {{b {$span}…}}",
];

// --- what the checker passes is what it says --------------------------------------------------

/// Which variant of a one-selector number message a whole number selects: an exact key, else its
/// category's variant, else `*`. `None` for no variant (never, once `*` is required).
fn selected(message: &Message, rule: &craft_i18n_tests::plural::Rule, n: u32) -> Option<usize> {
    let Body::Match { variants, .. } = &message.body else {
        return None;
    };
    let category = Category::ALL
        .into_iter()
        .find(|c| rule.members(*c).contains(&n))?;
    variants
        .iter()
        .position(|v| v.keys.first() == Some(&Key::Exact(n)))
        .or_else(|| {
            variants
                .iter()
                .position(|v| v.keys.first() == Some(&Key::Name(category.name().to_owned())))
        })
        .or_else(|| {
            variants
                .iter()
                .position(|v| v.keys.first() == Some(&Key::Catchall))
        })
}

fn count_message(keys: &[String], with_n: &[bool]) -> String {
    let variants: String = keys
        .iter()
        .zip(with_n)
        .map(|(key, with_n)| {
            let body = if *with_n { "{$n} x" } else { "x" };
            format!(" {key} {{{{{body}}}}}")
        })
        .collect();
    format!(".input {{$n :number}} .match $n{variants} * {{{{{{$n}} x}}}}")
}

proptest! {
    #![proptest_config(config(600))]

    /// If the checker passes a count message, no whole number from 0 to 1000 that has a category
    /// other than `other` falls to `*`; and where it refuses for a missing category, one does.
    /// A variant without `{$n}` is chosen by at most one whole number: it says that number in words.
    #[test]
    fn the_checker_is_right_about_what_the_whole_numbers_select(
        locale in prop::sample::select(&["de", "fr", "pl", "ar", "ja", "ru", "cy", "sl", "he", "lv"][..]),
        categories in prop::collection::vec(
            prop::sample::select(&["zero", "one", "two", "few", "many"][..]), 0..5),
        exacts in prop::collection::vec(
            prop::sample::select(&["0", "1", "2", "3", "4", "5", "11", "12", "21", "100", "101", "1000"][..]), 0..8),
        with_n in prop::collection::vec(any::<bool>(), 13),
    ) {
        let spec = &specs()[locale];
        let mut keys: Vec<String> = Vec::new();
        for key in categories.iter().map(|c| (*c).to_owned()).chain(exacts.iter().map(|e| (*e).to_owned())) {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        let text = count_message(&keys, &with_n);
        let message = parse(&text).unwrap();
        let source = parse_source("{n} x").unwrap();
        let outcome = check(
            &message,
            RowInfo { kind: RowKind::Plural, source: &source, ellipsis: false },
            spec,
        );
        let rule = &spec.plurals.cardinal;
        let Body::Match { variants, .. } = &message.body else { unreachable!() };
        let catch_all = variants.len() - 1;
        // The numbers that reach a category other than `other` but select `*`.
        let leaking: Vec<u32> = (0..=REACH)
            .filter(|n| {
                let category = Category::ALL.into_iter().find(|c| rule.members(*c).contains(n));
                category != Some(Category::Other) && selected(&message, rule, *n) == Some(catch_all)
            })
            .collect();
        let unknown_key = outcome.errors.iter().any(|e| e.contains("is not a plural category"));
        if unknown_key {
            // A category this locale has no word for: refused whatever else holds.
            return Ok(());
        }
        let missing = outcome.errors.iter().any(|e| e.contains("no variant for"));
        prop_assert_eq!(missing, !leaking.is_empty(), "{} {:?} {:?}", text, outcome.errors, leaking);
        if outcome.errors.is_empty() {
            for (k, variant) in variants.iter().enumerate() {
                let lacks_n = !variant
                    .pattern
                    .parts
                    .iter()
                    .any(|p| matches!(p, message::Part::Placeholder { name, .. } if name == "n"));
                if lacks_n {
                    let chosen = (0..=REACH).filter(|n| selected(&message, rule, *n) == Some(k)).count();
                    prop_assert!(chosen <= 1, "variant {k} of {text} says no number but {chosen} numbers choose it");
                }
            }
        }
    }
}

// --- the checker over messages that parse ----------------------------------------------------

const FUNCS: &[&str] = &[
    ":number", ":integer", ":measure", ":range", ":date", ":noun",
];
const OPTIONS: &[&str] = &[
    "",
    "",
    " select=ordinal",
    " select=plural",
    " form=accusative",
    " form=genitive",
];
const VARS: &[&str] = &["n", "item", "span", "when", "name", "k"];
const KEYS: &[&str] = &[
    "*",
    "0",
    "1",
    "2",
    "one",
    "two",
    "few",
    "many",
    "zero",
    "other",
    "feminine",
    "masculine",
    "neuter",
];
const PIECES: &[&str] = &[
    "x",
    "…",
    "{$n}",
    "{$item :noun form=accusative}",
    "{$span}",
    " ",
    "{$when}",
    "{$a}",
    "{$name}",
    "{$n :integer}",
];
const SOURCES: &[&str] = &[
    "{n} a",
    "{n :integer} {item :noun}",
    "{span :range}",
    "{when :date} {n :number}",
    "Open {name}…",
    "{n :measure} {item :noun}",
    "{n :number} {span :range} {when :date} {item :noun} {name}",
];

fn declaration() -> impl Strategy<Value = String> {
    (
        any::<bool>(),
        prop::sample::select(VARS),
        prop::sample::select(&["a", "b", "n"][..]),
        prop::sample::select(FUNCS),
        prop::sample::select(OPTIONS),
    )
        .prop_map(|(input, operand, name, func, options)| {
            if input {
                format!(".input {{${operand} {func}{options}}} ")
            } else {
                format!(".local ${name} = {{${operand} {func}{options}}} ")
            }
        })
}

fn pattern() -> impl Strategy<Value = String> {
    soup(PIECES, 6).prop_map(|p| format!("{{{{{p}}}}}"))
}

fn structured() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(declaration(), 0..4),
        prop::collection::vec(prop::sample::select(VARS), 1..3),
        prop::collection::vec(prop::collection::vec(prop::sample::select(KEYS), 3), 1..6),
        prop::collection::vec(pattern(), 6),
        any::<bool>(),
    )
        .prop_map(|(declarations, selectors, variants, patterns, matched)| {
            let mut text: String = declarations.concat();
            if !matched {
                text.push_str(&patterns[0]);
                return text;
            }
            text.push_str(".match");
            for selector in &selectors {
                text.push_str(&format!(" ${selector}"));
            }
            for (keys, pattern) in variants.iter().zip(&patterns) {
                text.push(' ');
                for key in keys.iter().take(selectors.len()) {
                    text.push_str(&format!("{key} "));
                }
                text.push_str(pattern);
            }
            text
        })
}

/// Messages over `{n :integer} {item :noun}` that are declared and keyed as the checker asks, so
/// the generator reaches coverage, the placeholders each variant keeps and the ellipsis; German
/// has the keys.
fn well_formed() -> impl Strategy<Value = String> {
    let n_keys = prop::sample::select(&["*", "0", "1", "2", "one"][..]);
    let item_keys = prop::sample::select(&["*", "feminine", "masculine", "neuter"][..]);
    let piece = prop::sample::select(
        &[
            "{$n}",
            "{$item}",
            " x ",
            "{$item :noun form=accusative}",
            "…",
        ][..],
    );
    (
        prop::sample::select(&["n", "item", "both"][..]),
        prop::collection::vec(
            (n_keys, item_keys, prop::collection::vec(piece, 1..4)),
            0..5,
        ),
        prop::collection::vec(prop::sample::select(&["{$n}", "{$item}"][..]), 2..3),
    )
        .prop_map(|(selected, variants, last)| {
            let selectors = match selected {
                "n" => vec!["n"],
                "item" => vec!["item"],
                _ => vec!["n", "item"],
            };
            let mut text = String::from(".input {$n :integer} .input {$item :noun} .match");
            for selector in &selectors {
                text.push_str(&format!(" ${selector}"));
            }
            for (n_key, item_key, pieces) in &variants {
                text.push(' ');
                for selector in &selectors {
                    text.push_str(if *selector == "n" { n_key } else { item_key });
                    text.push(' ');
                }
                text.push_str(&format!("{{{{{}}}}}", pieces.concat()));
            }
            text.push(' ');
            text.push_str(&"* ".repeat(selectors.len()));
            text.push_str(&format!("{{{{{} {}}}}}", last[0], last[1]));
            text
        })
}

proptest! {
    #![proptest_config(config(1500))]

    #[test]
    fn check_never_panics_and_is_a_function_of_its_inputs(
        text in prop_oneof![structured(), soup(MESSAGE_TOKENS, 30), well_formed()],
        source in prop::sample::select(SOURCES),
        locale in prop::sample::select(&["de", "fr", "pl", "ar", "ja", "ru", "cy", "sl", "he", "lv", "ckb"][..]),
        kind in prop::sample::select(&[RowKind::Plain, RowKind::Plural, RowKind::Ordinal][..]),
    ) {
        let Ok(message) = parse(&text) else {
            return Ok(());
        };
        let parsed = parse_source(source).unwrap();
        let info = RowInfo { kind, source: &parsed, ellipsis: source.ends_with('…') };
        let spec = &specs()[locale];
        let first = check(&message, info, spec);
        prop_assert!(first.errors.len() <= 20, "{:?}", first.errors);
        prop_assert_eq!(first, check(&message, info, spec));
    }
}

#[test]
fn the_generators_reach_the_parser_and_the_checker() {
    // A sanity check on the strategies above: most structured messages parse, and the directed
    // ones get through the declarations and keys to the completeness stage, where some pass.
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let source = parse_source("{n :integer} {item :noun}").unwrap();
    let info = RowInfo {
        kind: RowKind::Plain,
        source: &source,
        ellipsis: false,
    };
    let (mut parsed, mut stage_three, mut passed) = (0, 0, 0);
    for _ in 0..400 {
        let text = structured().new_tree(&mut runner).unwrap().current();
        parsed += usize::from(parse(&text).is_ok());
    }
    for _ in 0..400 {
        let text = well_formed().new_tree(&mut runner).unwrap().current();
        let message = parse(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        let outcome = check(&message, info, &specs()["de"]);
        stage_three += usize::from(outcome.errors.iter().all(|e| {
            e.contains("no variant for") || e.contains("leaves out") || e.contains("trailing")
        }));
        passed += usize::from(outcome.errors.is_empty());
    }
    assert!(parsed >= 200, "{parsed} of 400 structured messages parse");
    assert!(
        stage_three >= 150,
        "{stage_three} of 400 directed messages reach stage three"
    );
    assert!(passed >= 5, "{passed} of 400 directed messages pass");
}
