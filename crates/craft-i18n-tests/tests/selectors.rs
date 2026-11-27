//! Ordinals, typed placeholders, selector messages, nouns and grammar: the worked examples of
//! German, Polish, Arabic, Japanese and French pass the checker, and each rule of
//! `docs/catalog-format.md` ("What the checker requires of a selector message") refuses what it
//! says, by file and line.

use std::path::{Path, PathBuf};

use craft_i18n_tests::tree::check_tree;

const H: &str = "# Clean room: written from the plain meaning.\n";
const SHAPES: &str = "{n} shape selected|{n} shapes selected";
const PAGES: &str = "{n} page|{n} pages";
const PLACE: &str = "Move to the {n}st place|Move to the {n}nd place|Move to the {n}rd place|Move to the {n}th place";
const DELETE: &str = "Delete this {item :noun}?";

/// A fresh tree under the test target directory holding `files` (path relative to the root).
fn fixture(name: &str, files: &[(String, String)]) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    for (path, text) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    root
}

fn row(context: &str, source: &str, translation: &str) -> String {
    format!("{context}\t{source}\t{translation}\n")
}

/// A tree of one locale: `common.tsv` (header, language name, `common`) and `app.tsv` (header,
/// `app`). Line 1 of each file is the header, line 2 of `common.tsv` the language name, so the
/// first row of `app.tsv` is line 2 and the first of `common` is line 3.
fn tree(test: &str, locale: &str, common: &str, app: &str) -> PathBuf {
    let name = row("@id", "language.name", "Nom");
    fixture(
        &format!("selectors-{test}"),
        &[
            (
                format!("locales/{locale}/common.tsv"),
                format!("{H}{name}{common}"),
            ),
            (format!("locales/{locale}/app.tsv"), format!("{H}{app}")),
        ],
    )
}

fn problems(test: &str, locale: &str, common: &str, app: &str) -> Vec<String> {
    check_tree(&tree(test, locale, common, app))
}

fn assert_clean(test: &str, locale: &str, common: &str, app: &str) {
    assert_eq!(
        problems(test, locale, common, app),
        Vec::<String>::new(),
        "{test}"
    );
}

/// The checker finds exactly one problem, at `where_` (`app.tsv: line 2:`), and it holds `words`.
fn assert_refused(test: &str, locale: &str, common: &str, app: &str, where_: &str, words: &str) {
    let found = problems(test, locale, common, app);
    assert_eq!(found.len(), 1, "{test}: {found:?}");
    let prefix = format!("locales/{locale}/{where_}");
    assert!(found[0].starts_with(&prefix), "{test}: {found:?}");
    assert!(
        found[0].contains(words),
        "{test}: {found:?} lacks {words:?}"
    );
}

// --- the worked examples -------------------------------------------------------------------

#[test]
fn german_passes() {
    let common = [
        row("@grammar", "genders", "masculine feminine neuter"),
        row("@noun", "page", "feminine: Seite"),
        row("@noun", "connector", "masculine: Verbinder"),
        row("@noun", "picture", "neuter: Bild"),
    ]
    .concat();
    let app = [
        row("@plural", SHAPES, "{n} Form ausgewählt|{n} Formen ausgewählt"),
        // `one` holds the whole number 1 only, so a variant may say it in words.
        row(
            "@plural",
            PAGES,
            ".input {$n :number} .match $n 0 {{Keine Seiten}} one {{Eine Seite}} * {{{$n} Seiten}}",
        ),
        row("@ordinal", PLACE, "An die {n}. Stelle verschieben"),
        row(
            "",
            DELETE,
            ".input {$item :noun} .match $item feminine {{Diese {$item} löschen?}} masculine {{Diesen {$item} löschen?}} * {{Dieses {$item} löschen?}}",
        ),
        row("", "Saved on {when :date}", "Gespeichert am {when}"),
        row("", "Page {n :integer} of {total :integer}", "Seite {n} von {total}"),
    ]
    .concat();
    assert_clean("de", "de", &common, &app);
}

#[test]
fn polish_passes() {
    let common = [
        row(
            "@grammar",
            "forms",
            "accusative genitive instrumental locative",
        ),
        row("@grammar", "genders", "masculine feminine neuter"),
        row("@noun", "page", "feminine: strona; accusative=stronę"),
        row("@noun", "connector", "masculine: łącznik"),
        row("@noun", "picture", "masculine: obraz"),
    ]
    .concat();
    let app = [
        row(
            "@plural",
            SHAPES,
            "Zaznaczono {n} kształt|Zaznaczono {n} kształty|Zaznaczono {n} kształtów",
        ),
        // Whole numbers reach `one`, `few` and `many`; `*` stands for `other`, reached by fractions.
        row(
            "@plural",
            PAGES,
            ".input {$n :number} .match $n one {{Jedna strona}} few {{{$n} strony}} many {{{$n} stron}} * {{{$n} strony}}",
        ),
        row("@ordinal", PLACE, "Przenieś na {n}. miejsce"),
        row(
            "",
            DELETE,
            ".input {$item :noun} .match $item feminine {{Usunąć tę {$item :noun form=accusative}?}} neuter {{Usunąć to {$item :noun form=accusative}?}} * {{Usunąć ten {$item :noun form=accusative}?}}",
        ),
    ]
    .concat();
    assert_clean("pl", "pl", &common, &app);
}

#[test]
fn arabic_passes() {
    let common = [
        row("@grammar", "forms", "definite"),
        row("@grammar", "genders", "masculine feminine"),
        row("@noun", "page", "feminine: صفحة; definite=الصفحة"),
        row("@noun", "connector", "masculine: موصل; definite=الموصل"),
        row("@noun", "picture", "feminine: صورة; definite=الصورة"),
    ]
    .concat();
    let app = [
        // All six categories; `zero`, `one` and `two` hold one whole number each and say it in words.
        row(
            "@plural",
            SHAPES,
            ".input {$n :number} .match $n zero {{لا أشكال محددة}} one {{شكل واحد محدد}} two {{شكلان محددان}} few {{{$n} أشكال محددة}} many {{{$n} شكلًا محددًا}} * {{{$n} شكل محدد}}",
        ),
        row("@ordinal", PLACE, "انقل إلى الموضع {n}"),
        row(
            "",
            DELETE,
            ".input {$item :noun} .match $item feminine {{حذف هذه {$item :noun form=definite}؟}} * {{حذف هذا {$item :noun form=definite}؟}}",
        ),
    ]
    .concat();
    assert_clean("ar", "ar", &common, &app);
}

#[test]
fn japanese_passes() {
    let common = [
        row("@noun", "page", "ページ"),
        row("@noun", "connector", "コネクタ"),
        row("@noun", "picture", "画像"),
    ]
    .concat();
    let app = [
        row("@plural", SHAPES, "{n} 個の図形を選択中"),
        row("@ordinal", PLACE, "{n} 番目に移動"),
        row("", DELETE, "この{item}を削除しますか？"),
    ]
    .concat();
    assert_clean("ja", "ja", &common, &app);
}

#[test]
fn french_passes() {
    let common = row("@grammar", "genders", "masculine feminine");
    let app = [
        row("@plural", PAGES, "{n} page|{n} pages"),
        // French `one` is 0 and 1, so it keeps the number; the ordinal `one` is 1 alone.
        row(
            "@plural",
            SHAPES,
            ".input {$n :number} .match $n one {{{$n} forme sélectionnée}} * {{{$n} formes sélectionnées}}",
        ),
        row(
            "@ordinal",
            PLACE,
            ".input {$n :number select=ordinal} .match $n one {{Déplacer à la {$n}re place}} * {{Déplacer à la {$n}e place}}",
        ),
        row(
            "",
            "Pages {span :range} selected",
            ".input {$span :range} .match $span one {{Page {$span} sélectionnée}} * {{Pages {$span} sélectionnées}}",
        ),
    ]
    .concat();
    assert_clean("fr", "fr", &common, &app);
}

#[test]
fn a_message_may_declare_a_form_with_local_and_use_it() {
    let common = [
        row("@grammar", "forms", "genitive"),
        row("@grammar", "genders", "masculine feminine"),
        row("@noun", "page", "feminine: Seite; genitive=Seite"),
    ]
    .concat();
    let app = row(
        "",
        "Rename {item :noun}",
        ".input {$item :noun} .local $of = {$item :noun form=genitive} {{Umbenennen: {$of}}}",
    );
    assert_clean("local", "de", &common, &app);
}

#[test]
fn two_selectors_choose_by_a_count_and_a_gender() {
    let common = [
        row("@grammar", "genders", "masculine feminine neuter"),
        row("@noun", "page", "feminine: Seite"),
    ]
    .concat();
    let app = row(
        "",
        "{n :integer} {item :noun} deleted",
        ".input {$n :integer} .input {$item :noun} .match $n $item 1 feminine {{Eine {$item} gelöscht}} 1 * {{Ein {$item} gelöscht}} * * {{{$n} {$item} gelöscht}}",
    );
    assert_clean("two-selectors", "de", &common, &app);
}

#[test]
fn plain_and_classic_rows_work_as_before() {
    let app = [
        row("", "Save", "Enregistrer"),
        row("@id", "file.save", "Enregistrer"),
        row("@msg", "Couldn't open {name}", "Impossible d'ouvrir {name}"),
        row("@plural", PAGES, "{n} page|{n} pages"),
        row("menu", "Open…", "Ouvrir…"),
    ]
    .concat();
    assert_clean("plain", "fr", "", &app);
}

// --- plural and ordinal categories ---------------------------------------------------------

#[test]
fn a_polish_message_without_many_is_refused() {
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{Jedna strona}} few {{{$n} strony}} * {{{$n} stron}}",
    );
    assert_refused("pl-no-many", "pl", "", &app, "app.tsv: line 2:", "many");
}

#[test]
fn every_category_the_numbers_reach_needs_a_variant_of_its_own() {
    // Arabic reaches zero, one, two, few and many as well as other.
    let app = row(
        "@plural",
        SHAPES,
        ".input {$n :number} .match $n one {{شكل واحد}} * {{{$n} شكل}}",
    );
    let found = problems("ar-missing", "ar", "", &app);
    assert_eq!(found.len(), 1, "{found:?}");
    for category in ["zero", "two", "few", "many"] {
        assert!(found[0].contains(category), "{found:?}");
    }
    assert!(!found[0].contains("`one`"), "{found:?}");
}

#[test]
fn a_category_only_fractions_reach_needs_no_variant() {
    // Polish `other` is `*`; French `many` (millions) is never reached by 0 to 1000.
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{{$n} page}} * {{{$n} pages}}",
    );
    assert_clean("fr-many", "fr", "", &app);
}

#[test]
fn a_key_must_be_a_category_the_locale_has() {
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{{$n} Seite}} few {{{$n} Seiten}} * {{{$n} Seiten}}",
    );
    assert_refused("de-few", "de", "", &app, "app.tsv: line 2:", "`few`");
}

#[test]
fn other_is_written_as_the_catch_all() {
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{{$n} Seite}} other {{{$n} Seiten}} * {{{$n} Seiten}}",
    );
    assert_refused("de-other", "de", "", &app, "app.tsv: line 2:", "`other`");
}

#[test]
fn ordinal_categories_are_the_ordinal_ones() {
    // French ordinals are one and other; `few` is English's.
    let app = row(
        "@ordinal",
        PLACE,
        ".input {$n :number select=ordinal} .match $n one {{{$n}re}} few {{{$n}e}} * {{{$n}e}}",
    );
    assert_refused("fr-few", "fr", "", &app, "app.tsv: line 2:", "`few`");
    // A cardinal selector in an ordinal row reads the position as a count.
    let app = row(
        "@ordinal",
        PLACE,
        ".input {$n :number} .match $n one {{{$n}re}} * {{{$n}e}}",
    );
    assert_refused(
        "fr-cardinal-in-ordinal",
        "fr",
        "",
        &app,
        "app.tsv: line 2:",
        "select=ordinal",
    );
    // And an ordinal selector in a plural row reads a count as a position.
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number select=ordinal} .match $n one {{{$n} page}} * {{{$n} pages}}",
    );
    assert_refused(
        "fr-ordinal-in-plural",
        "fr",
        "",
        &app,
        "app.tsv: line 2:",
        "select=ordinal",
    );
}

#[test]
fn german_ordinal_has_one_form() {
    let app = row("@ordinal", PLACE, "{n}. Stelle|{n}. Platz");
    assert_refused(
        "de-two-ordinals",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        "expected 1",
    );
    let app = row("@ordinal", PLACE, "An die {n}. Stelle verschieben");
    assert_clean("de-one-ordinal", "de", "", &app);
}

#[test]
fn english_ordinal_source_has_four_forms() {
    let app = row("@ordinal", "{n}st place|{n}th place", "{n}. Platz");
    assert_refused(
        "ordinal-source",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        "one|two|few|other",
    );
}

#[test]
fn a_classic_ordinal_form_keeps_the_placeholders() {
    let app = row("@ordinal", PLACE, "An die Stelle verschieben");
    assert_refused(
        "de-ordinal-ph",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        "placeholders",
    );
}

// --- the numbers a variant may leave out ---------------------------------------------------

#[test]
fn only_single_valued_categories_and_exact_keys_may_leave_out_the_number() {
    // German `other` holds many numbers: `{$n}` must show.
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{Eine Seite}} * {{Viele Seiten}}",
    );
    assert_refused("de-no-n", "de", "", &app, "app.tsv: line 2:", "{n}");
    // French `one` holds 0 and 1: not single-valued.
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{Une page}} * {{{$n} pages}}",
    );
    assert_refused("fr-one-no-n", "fr", "", &app, "app.tsv: line 2:", "{n}");
}

#[test]
fn an_exact_number_may_leave_out_the_number() {
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n 0 {{Keine Seite}} 1 {{Eine Seite}} * {{{$n} Seiten}}",
    );
    assert_clean("de-exact", "de", "", &app);
}

#[test]
fn another_placeholder_is_never_left_out() {
    let app = row(
        "",
        "{n :number} pages in {file}",
        ".input {$n :number} .match $n 1 {{Eine Seite}} * {{{$n} Seiten in {$file}}}",
    );
    assert_refused("de-file", "de", "", &app, "app.tsv: line 2:", "{file}");
}

// --- variables, functions and keys ---------------------------------------------------------

#[test]
fn a_variable_must_be_a_placeholder_of_the_source() {
    let app = row(
        "@plural",
        PAGES,
        ".input {$count :number} .match $count one {{Eine}} * {{{$n} Seiten}}",
    );
    assert_refused("de-count", "de", "", &app, "app.tsv: line 2:", "$count");
}

#[test]
fn text_and_dates_cannot_be_chosen_by() {
    let app = row(
        "",
        "Saved on {when :date}",
        ".input {$when :number} .match $when 1 {{Am {$when}}} * {{Am {$when}}}",
    );
    assert_refused("de-date", "de", "", &app, "app.tsv: line 2:", "$when");
    let app = row(
        "",
        "Open {name}",
        ".input {$name :number} .match $name 1 {{{$name}}} * {{{$name}}}",
    );
    assert_refused("de-text", "de", "", &app, "app.tsv: line 2:", "$name");
}

#[test]
fn a_selector_must_be_declared() {
    let app = row(
        "@plural",
        PAGES,
        ".match $n one {{Eine Seite}} * {{{$n} Seiten}}",
    );
    assert_refused(
        "de-undeclared",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        ".input",
    );
}

#[test]
fn the_function_must_fit_the_source_type() {
    let common = row("@grammar", "genders", "masculine feminine");
    let app = row(
        "",
        DELETE,
        ".input {$item :number} .match $item 1 {{Löschen {$item}}} * {{Löschen {$item}}}",
    );
    assert_refused("de-fn", "de", &common, &app, "app.tsv: line 2:", ":number");
}

#[test]
fn a_noun_is_chosen_by_declared_genders() {
    let common = [
        row("@grammar", "genders", "masculine feminine"),
        row("@noun", "page", "feminine: page"),
    ]
    .concat();
    let app = row(
        "",
        DELETE,
        ".input {$item :noun} .match $item neuter {{Ce {$item}}} * {{Cette {$item}}}",
    );
    assert_refused(
        "fr-neuter",
        "fr",
        &common,
        &app,
        "app.tsv: line 2:",
        "`neuter`",
    );
}

#[test]
fn a_form_must_be_declared() {
    let common = [
        row("@grammar", "genders", "masculine feminine"),
        row("@grammar", "forms", "accusative"),
    ]
    .concat();
    let app = row(
        "",
        DELETE,
        ".input {$item :noun} .match $item feminine {{Diese {$item :noun form=genitive}}} * {{Dieses {$item}}}",
    );
    assert_refused(
        "de-form",
        "de",
        &common,
        &app,
        "app.tsv: line 2:",
        "`genitive`",
    );
}

#[test]
fn exact_keys_are_numbers_not_ranges() {
    let app = row(
        "",
        "Pages {span :range} selected",
        ".input {$span :range} .match $span 1 {{Seite {$span}}} * {{Seiten {$span}}}",
    );
    assert_refused(
        "fr-range-exact",
        "fr",
        "",
        &app,
        "app.tsv: line 2:",
        "range",
    );
}

#[test]
fn a_range_needs_the_categories_its_ends_reach() {
    // Polish ranges reach few and many besides other.
    let app = row(
        "",
        "Pages {span :range} selected",
        ".input {$span :range} .match $span * {{Strony {$span}}}",
    );
    let found = problems("pl-range", "pl", "", &app);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("many"), "{found:?}");
}

// --- the grammar of a message --------------------------------------------------------------

#[test]
fn a_message_needs_a_catch_all() {
    let app = row(
        "@plural",
        PAGES,
        ".input {$n :number} .match $n one {{Eine Seite}} 2 {{{$n} Seiten}}",
    );
    assert_refused("de-catch-all", "de", "", &app, "app.tsv: line 2:", "`*`");
}

#[test]
fn a_message_refuses_what_the_subset_leaves_out() {
    // Japanese reaches only `other`, so each message below has one fault and no other.
    for (test, message, words) in [
        (
            "markup",
            ".input {$n :number} .match $n * {{{#b}{$n}{/b}}}",
            "markup",
        ),
        (
            "fn",
            ".input {$n :string} .match $n * {{{$n}}}",
            "unknown function",
        ),
        (
            "option",
            ".input {$n :number minimumFractionDigits=2} .match $n * {{{$n}}}",
            "option",
        ),
        (
            "four",
            ".input {$n :number} .match $n $n $n $n * * * * {{{$n}}}",
            "at most 3 selectors",
        ),
        (
            "two-match",
            ".input {$n :number} .match $n * {{{$n}}} .match $n * {{{$n}}}",
            "`.match`",
        ),
        ("unclosed", ".input {$n :number} .match $n * {{{$n}", "`}}`"),
        ("keys", ".input {$n :number} .match $n * * {{{$n}}}", "keys"),
        (
            "escape",
            ".input {$n :number} .match $n * {{\\q{$n}}}",
            "escape",
        ),
        ("empty", ".input {$n :number} .match $n * {{}}", "empty"),
        (
            "stray",
            ".input {$n :number} .match $n * {{{$n}}} x",
            "no pattern",
        ),
        (
            "lone-brace",
            ".input {$n :number} .match $n * {{a } b {$n}}}",
            "`}`",
        ),
        (
            "leading-zero",
            ".input {$n :number} .match $n 01 {{{$n}}} * {{{$n}}}",
            "01",
        ),
        (
            "duplicate",
            ".input {$n :number} .match $n 1 {{{$n}}} 1 {{{$n}}} * {{{$n}}}",
            "twice",
        ),
        (
            "declared-twice",
            ".input {$n :number} .input {$n :integer} .match $n * {{{$n}}}",
            "twice",
        ),
        (
            "used-before",
            ".local $m = {$n :number} .input {$n :number} .match $m * {{{$n}}}",
            "before",
        ),
        ("bare-input", ".input {$n} .match $n * {{{$n}}}", "function"),
        ("garbage", ".input", "expected"),
        ("keyword", ".match", "needs a variable"),
    ] {
        let app = row("@plural", PAGES, message);
        let found = problems(&format!("grammar-{test}"), "ja", "", &app);
        assert_eq!(found.len(), 1, "{test}: {found:?}");
        assert!(
            found[0].starts_with("locales/ja/app.tsv: line 2:"),
            "{test}: {found:?}"
        );
        assert!(
            found[0].contains(words),
            "{test}: {found:?} lacks {words:?}"
        );
    }
}

/// `.input {$n :number} .match $n <variants> * {{{$n}}}` over `PAGES` in Japanese, whose numbers
/// all reach `other`: the only limit that can trip is the one under test.
fn counted(variants: &str) -> String {
    row(
        "@plural",
        PAGES,
        &format!(".input {{$n :number}} .match $n{variants} * {{{{{{$n}}}}}}"),
    )
}

#[test]
fn the_variant_limit_is_counted() {
    let exact =
        |count: usize| -> String { (0..count).map(|k| format!(" {k} {{{{{k}}}}}")).collect() };
    // 63 exact variants and the catch-all are 64.
    assert_clean("variants-64", "ja", "", &counted(&exact(63)));
    assert_refused(
        "variants-65",
        "ja",
        "",
        &counted(&exact(64)),
        "app.tsv: line 2:",
        "at most 64 variants",
    );
}

#[test]
fn the_declaration_limit_is_counted() {
    let message = |locals: usize| -> String {
        let locals: String = (0..locals)
            .map(|k| format!(".local $m{k} = {{$n :number}} "))
            .collect();
        row(
            "@plural",
            PAGES,
            &format!(".input {{$n :number}} {locals}.match $n * {{{{{{$n}}}}}}"),
        )
    };
    // The `.input` and seven `.local` are 8.
    assert_clean("declarations-8", "ja", "", &message(7));
    assert_refused(
        "declarations-9",
        "ja",
        "",
        &message(8),
        "app.tsv: line 2:",
        "at most 8 declarations",
    );
}

#[test]
fn the_part_limit_is_counted() {
    let message = |placeholders: usize, tail: &str| -> String {
        let parts = "{$n}x".repeat(placeholders);
        row(
            "@plural",
            PAGES,
            &format!(".input {{$n :number}} .match $n * {{{{{parts}{tail}}}}}"),
        )
    };
    // 32 placeholders and 32 runs of text are 64 parts.
    assert_clean("parts-64", "ja", "", &message(32, ""));
    assert_refused(
        "parts-65",
        "ja",
        "",
        &message(32, "{$n}"),
        "app.tsv: line 2:",
        "at most 64 parts",
    );
}

#[test]
fn the_message_size_limit_is_counted() {
    let head = ".input {$n :number} .match $n * {{{$n} ";
    let message = |bytes: usize| -> String {
        let pad = bytes - head.len() - "}}".len();
        row("@plural", PAGES, &format!("{head}{}}}}}", "x".repeat(pad)))
    };
    assert_clean("bytes-4096", "ja", "", &message(4096));
    assert_refused(
        "bytes-4097",
        "ja",
        "",
        &message(4097),
        "app.tsv: line 2:",
        "at most 4096 bytes",
    );
}

#[test]
fn the_row_size_limit_is_counted() {
    // `\tSave\t` is 6 bytes: 16,384 in all passes, one more does not.
    assert_clean(
        "row-16384",
        "ja",
        "",
        &row("", "Save", &"x".repeat(16_384 - 6)),
    );
    assert_refused(
        "row-16385",
        "ja",
        "",
        &row("", "Save", &"x".repeat(16_385 - 6)),
        "app.tsv: line 2:",
        "16384",
    );
}

#[test]
fn the_row_count_limit_is_counted() {
    let rows =
        |count: usize| -> String { (0..count).map(|k| row("", &format!("S{k}"), "T")).collect() };
    assert_clean("rows-20000", "ja", "", &rows(20_000));
    let found = problems("rows-20001", "ja", "", &rows(20_001));
    assert_eq!(
        found.len(),
        1,
        "{:?}",
        found.iter().take(3).collect::<Vec<_>>()
    );
    assert!(
        found[0].starts_with("locales/ja/app.tsv: line 20002:"),
        "{found:?}"
    );
    assert!(found[0].contains("20000"), "{found:?}");
}

#[test]
fn a_catalog_over_two_mib_is_refused_whole() {
    // A file of `total` bytes: the header `tree` writes, and a comment as long as it takes.
    let file = |total: usize| -> String { format!("# {}\n", "x".repeat(total - H.len() - 3)) };
    let limit = 2 * 1024 * 1024;
    assert_clean("file-2mib", "ja", "", &file(limit));
    let found = problems("file-over", "ja", "", &file(limit + 1));
    assert_eq!(
        found.len(),
        1,
        "{:?}",
        found.iter().take(3).collect::<Vec<_>>()
    );
    assert!(found[0].starts_with("locales/ja/app.tsv:"), "{found:?}");
    assert!(found[0].contains("2097152"), "{found:?}");
}

// --- placeholders --------------------------------------------------------------------------

#[test]
fn a_typed_placeholder_belongs_to_the_source() {
    let app = row("", "Page {n :integer}", "Seite {n :integer}");
    assert_refused(
        "type-in-translation",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        ":integer",
    );
}

#[test]
fn a_placeholder_type_must_exist() {
    let app = row("", "Page {n :ordinal}", "Seite {n}");
    assert_refused(
        "type-unknown",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        ":ordinal",
    );
}

#[test]
fn a_typed_name_is_an_identifier() {
    let app = row("", "Page {the n :integer}", "Seite {the n}");
    assert_refused("type-name", "de", "", &app, "app.tsv: line 2:", "the n");
}

#[test]
fn typed_placeholders_stay_out_of_msg_and_id_rows() {
    let app = row(
        "@msg",
        "Couldn't open {name :integer}",
        "Impossible d'ouvrir {name}",
    );
    assert_refused("msg-typed", "fr", "", &app, "app.tsv: line 2:", "@msg");
}

#[test]
fn a_selector_message_stays_out_of_msg_rows() {
    let app = row(
        "@msg",
        "Couldn't open {name}",
        ".input {$name :number} .match $name * {{{$name}}}",
    );
    assert_refused("msg-selector", "fr", "", &app, "app.tsv: line 2:", "@msg");
}

#[test]
fn placeholders_must_match_in_a_plain_row_and_in_every_variant() {
    let app = row("", "Page {n :integer} of {total :integer}", "Seite {n}");
    assert_refused(
        "ph-differ",
        "de",
        "",
        &app,
        "app.tsv: line 2:",
        "placeholders differ",
    );
    let app = row(
        "",
        "Page {n :integer} of {total :integer}",
        ".input {$n :integer} .match $n 1 {{Seite {$n} von {$total}}} * {{Seite von {$total}}}",
    );
    assert_refused("ph-variant", "de", "", &app, "app.tsv: line 2:", "{n}");
}

#[test]
fn variants_agree_with_the_source_on_a_trailing_ellipsis() {
    let app = row(
        "",
        "Rename {n :integer} pages…",
        ".input {$n :integer} .match $n 1 {{Eine Seite umbenennen}} * {{{$n} Seiten umbenennen…}}",
    );
    assert_refused("ellipsis", "de", "", &app, "app.tsv: line 2:", "…");
}

// --- bidirectional controls ----------------------------------------------------------------

#[test]
fn embeddings_overrides_and_isolates_are_refused_everywhere() {
    for (test, control) in [
        ("lre", '\u{202A}'),
        ("rlo", '\u{202E}'),
        ("lri", '\u{2066}'),
        ("pdi", '\u{2069}'),
    ] {
        let app = row("", "Save", &format!("Sauver{control}"));
        assert_refused(test, "fr", "", &app, "app.tsv: line 2:", "bidirectional");
        let app = row("@id", "file.save", &format!("Sauver{control}"));
        assert_refused(
            &format!("id-{test}"),
            "fr",
            "",
            &app,
            "app.tsv: line 2:",
            "bidirectional",
        );
        let app = row(
            "@plural",
            PAGES,
            &format!(".input {{$n :number}} .match $n * {{{{{{$n}} p{control}}}}}"),
        );
        assert_refused(
            &format!("sel-{test}"),
            "fr",
            "",
            &app,
            "app.tsv: line 2:",
            "bidirectional",
        );
    }
}

#[test]
fn marks_belong_to_right_to_left_locales() {
    let app = row("", "Save", "Sauver\u{200E}");
    assert_refused("lrm-fr", "fr", "", &app, "app.tsv: line 2:", "U+200E");
    assert_clean("lrm-ar", "ar", "", &row("", "Save", "حفظ\u{200F}"));
    assert_clean("lrm-he", "he", "", &row("", "Save", "שמור\u{200E}"));
}

// --- nouns and grammar ---------------------------------------------------------------------

#[test]
fn a_noun_gender_is_one_the_locale_declares() {
    let common = row("@grammar", "genders", "masculine feminine");
    let app = row("@noun", "page", "neuter: page");
    assert_refused(
        "noun-gender",
        "fr",
        &common,
        &app,
        "app.tsv: line 2:",
        "`neuter`",
    );
}

#[test]
fn a_gendered_locale_gives_every_noun_a_gender() {
    let common = row("@grammar", "genders", "masculine feminine");
    let app = row("@noun", "page", "page");
    assert_refused(
        "noun-no-gender",
        "fr",
        &common,
        &app,
        "app.tsv: line 2:",
        "gender",
    );
}

#[test]
fn a_locale_without_genders_gives_no_noun_one() {
    let app = row("@noun", "page", "feminine: ページ");
    assert_refused(
        "noun-ja-gender",
        "ja",
        "",
        &app,
        "app.tsv: line 2:",
        "no genders",
    );
}

#[test]
fn a_noun_form_is_one_the_locale_declares() {
    let common = [
        row("@grammar", "genders", "masculine feminine"),
        row("@grammar", "forms", "accusative"),
    ]
    .concat();
    let app = row("@noun", "page", "feminine: Seite; dative=Seite");
    assert_refused(
        "noun-form",
        "de",
        &common,
        &app,
        "app.tsv: line 2:",
        "`dative`",
    );
}

#[test]
fn a_noun_is_well_formed() {
    let common = [
        row("@grammar", "genders", "masculine feminine"),
        row("@grammar", "forms", "accusative"),
    ]
    .concat();
    for (test, source, translation, words) in [
        ("empty-base", "page", "feminine: ", "base form"),
        (
            "empty-form",
            "page",
            "feminine: Seite; accusative=",
            "accusative",
        ),
        (
            "twice",
            "page",
            "feminine: Seite; accusative=a; accusative=b",
            "twice",
        ),
        ("source", "Page", "feminine: Seite", "noun"),
    ] {
        let app = row("@noun", source, translation);
        let found = problems(&format!("noun-{test}"), "de", &common, &app);
        assert_eq!(found.len(), 1, "{test}: {found:?}");
        assert!(
            found[0].starts_with("locales/de/app.tsv: line 2:"),
            "{test}: {found:?}"
        );
        assert!(found[0].contains(words), "{test}: {found:?}");
    }
}

#[test]
fn grammar_rows_are_well_formed() {
    for (test, source, translation, words) in [
        ("key", "gender", "masculine feminine", "genders"),
        ("name", "genders", "Masculine feminine", "Masculine"),
        ("twice", "genders", "masculine masculine", "twice"),
        (
            "many",
            "forms",
            "a b c d e f g h i j k l m n o p q",
            "at most 16",
        ),
    ] {
        let found = problems(
            &format!("grammar-row-{test}"),
            "de",
            "",
            &row("@grammar", source, translation),
        );
        assert_eq!(found.len(), 1, "{test}: {found:?}");
        assert!(
            found[0].starts_with("locales/de/app.tsv: line 2:"),
            "{test}: {found:?}"
        );
        assert!(found[0].contains(words), "{test}: {found:?}");
    }
}

#[test]
fn an_app_may_declare_its_own_genders_and_a_region_takes_its_base() {
    let name = row("@id", "language.name", "Nom");
    let mx_name = row("@id", "language.name", "Nom (Mexique)");
    let message = row(
        "",
        DELETE,
        ".input {$item :noun} .match $item feminine {{Eliminar esta {$item}}} * {{Eliminar este {$item}}}",
    );
    let files = [
        (
            "locales/es/common.tsv",
            format!(
                "{H}{name}{}",
                row("@grammar", "genders", "masculine feminine")
            ),
        ),
        ("locales/es-mx/common.tsv", format!("{H}{mx_name}")),
        (
            "locales/es-mx/app.tsv",
            format!("{H}{}{message}", row("@noun", "page", "feminine: página")),
        ),
    ]
    .map(|(path, text)| (path.to_owned(), text));
    assert_eq!(
        check_tree(&fixture("selectors-region", &files)),
        Vec::<String>::new()
    );

    // The app's own declaration replaces the shared one for that app: `neuter` is in the app's.
    let common = row("@grammar", "genders", "masculine feminine");
    let own = [
        row("@grammar", "genders", "masculine feminine neuter"),
        row("@noun", "picture", "neuter: Bild"),
    ]
    .concat();
    assert_clean("own-genders", "de", &common, &own);
    // And the shared one does not hold it.
    let shared = row("@noun", "picture", "neuter: Bild");
    assert_refused(
        "shared-genders",
        "de",
        &common,
        &shared,
        "app.tsv: line 2:",
        "`neuter`",
    );
}

// --- the format document -------------------------------------------------------------------

/// Every ```tsv locale=<code> block of `docs/catalog-format.md` is a catalog the checker passes.
#[test]
fn the_examples_of_the_format_document_pass() {
    let doc = std::fs::read_to_string(craft_i18n_tests::repo_root().join("docs/catalog-format.md"))
        .unwrap();
    let mut blocks = Vec::new();
    let mut open: Option<(String, String)> = None;
    for line in doc.lines() {
        match (&mut open, line.strip_prefix("```")) {
            (None, Some(info)) => {
                if let Some(locale) = info.strip_prefix("tsv locale=") {
                    open = Some((locale.to_owned(), String::new()));
                }
            }
            (Some(_), Some(_)) => blocks.extend(open.take()),
            (Some((_, text)), None) => {
                text.push_str(line);
                text.push('\n');
            }
            (None, None) => {}
        }
    }
    assert!(blocks.len() >= 13, "{} blocks", blocks.len());
    for (k, (locale, text)) in blocks.iter().enumerate() {
        let found = problems(&format!("doc-{k}-{locale}"), locale, "", text);
        assert_eq!(found, Vec::<String>::new(), "block {k} ({locale}):\n{text}");
    }
}
