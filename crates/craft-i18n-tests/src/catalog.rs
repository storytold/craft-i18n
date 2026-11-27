//! Reading one catalog: `context<TAB>English source text<TAB>translation` rows, checked against
//! their source text. Format: `docs/catalog-format.md`.

use std::collections::{BTreeMap, HashSet};

use crate::grammar::{self, Grammar, Noun};
use crate::limits::{MAX_CATALOG_BYTES, MAX_NOUNS, MAX_ROW_BYTES, MAX_ROWS};
use crate::message::{self, is_selector_message};
use crate::placeholder::{self, Source};
use crate::select::{self, RowInfo, RowKind};
use crate::spec::LocaleSpec;

/// A catalog row that passed every check, with its columns unescaped.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Row {
    /// 1-based line number in the file.
    pub line: usize,
    pub context: String,
    pub source: String,
    pub translation: String,
}

/// The contexts the apps give a meaning to. Any other context starting with `@` is an error;
/// contexts without `@` are free labels.
pub const RESERVED: &[&str] = &["@plural", "@ordinal", "@id", "@msg", "@noun", "@grammar"];

/// What reading a catalog found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reading {
    /// The rows that pass.
    pub rows: Vec<Row>,
    /// One problem (`line N: …`) per row that doesn't, and the problems of the file as a whole.
    pub problems: Vec<String>,
    /// What does not fail the catalog (`line N: …`).
    pub warnings: Vec<String>,
}

/// Read a catalog of `spec`'s locale. Returns the rows that pass, and one problem (`line N: …`)
/// per row that doesn't; a bad row is left out.
#[must_use]
pub fn read_catalog(text: &str, spec: &LocaleSpec) -> (Vec<Row>, Vec<String>) {
    let reading = read_catalog_report(text, spec);
    (reading.rows, reading.problems)
}

/// [`read_catalog`], with its warnings.
#[must_use]
pub fn read_catalog_report(text: &str, spec: &LocaleSpec) -> Reading {
    let mut reading = Reading::default();
    if text.len() > MAX_CATALOG_BYTES {
        reading.problems.push(format!(
            "the file is {} bytes; the apps do not read a catalog over {MAX_CATALOG_BYTES} bytes",
            text.len()
        ));
        return reading;
    }
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut keys = HashSet::new();
    let mut counted = 0_usize;
    let mut nouns = 0_usize;
    for (i, line) in text.split('\n').enumerate() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let number = i.saturating_add(1);
        counted = counted.saturating_add(1);
        if counted > MAX_ROWS {
            reading.problems.push(format!(
                "line {number}: more than {MAX_ROWS} rows; the apps ignore the rest of the file"
            ));
            break;
        }
        if line.len() > MAX_ROW_BYTES {
            reading.problems.push(format!(
                "line {number}: the row is {} bytes; the apps skip a row over {MAX_ROW_BYTES} bytes",
                line.len()
            ));
            continue;
        }
        let row = parse_row(number, line, spec).and_then(|(row, warnings)| {
            if row.context == "@noun" {
                nouns = nouns.saturating_add(1);
                if nouns > MAX_NOUNS {
                    return Err(format!(
                        "more than {MAX_NOUNS} @noun rows; the apps ignore the rest"
                    ));
                }
            }
            if keys.insert((row.context.clone(), row.source.clone())) {
                Ok((row, warnings))
            } else {
                Err("defined twice (same context and source text)".into())
            }
        });
        match row {
            Ok((row, warnings)) => {
                reading
                    .warnings
                    .extend(warnings.into_iter().map(|w| format!("line {number}: {w}")));
                reading.rows.push(row);
            }
            Err(e) => reading.problems.push(format!("line {number}: {e}")),
        }
    }
    reading
}

/// What a catalog declares: its `@grammar` lists and its `@noun` rows. Read apart from the rest of
/// the checks, because the rows of a file are checked against what the file, its locale's shared
/// catalog and its base language declare together.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Declared {
    pub genders: Option<Vec<String>>,
    pub forms: Option<Vec<String>>,
    pub nouns: BTreeMap<String, Noun>,
}

impl Declared {
    /// What `chain` declares, the first of it winning: the genders, the forms and each noun.
    #[must_use]
    pub fn merge<'a>(
        chain: impl IntoIterator<Item = &'a Declared>,
    ) -> (Grammar, BTreeMap<String, Noun>) {
        let mut grammar = Grammar::default();
        let (mut have_genders, mut have_forms) = (false, false);
        let mut nouns: BTreeMap<String, Noun> = BTreeMap::new();
        for declared in chain {
            if let (false, Some(genders)) = (have_genders, &declared.genders) {
                grammar.genders.clone_from(genders);
                have_genders = true;
            }
            if let (false, Some(forms)) = (have_forms, &declared.forms) {
                grammar.forms.clone_from(forms);
                have_forms = true;
            }
            for (source, noun) in &declared.nouns {
                nouns.entry(source.clone()).or_insert_with(|| noun.clone());
            }
        }
        (grammar, nouns)
    }
}

/// The declarations of a catalog's text. Rows that do not parse are left out here; reading the
/// catalog reports them.
#[must_use]
pub fn declared(text: &str) -> Declared {
    let mut declared = Declared::default();
    if text.len() > MAX_CATALOG_BYTES {
        return declared;
    }
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for line in text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .take(MAX_ROWS)
    {
        let columns: Vec<&str> = line.splitn(4, '\t').collect();
        let [context, source, translation] = columns.as_slice() else {
            continue;
        };
        if line.len() > MAX_ROW_BYTES || !matches!(*context, "@grammar" | "@noun") {
            continue;
        }
        let (Ok(source), Ok(translation)) = (unescape(source), unescape(translation)) else {
            continue;
        };
        if *context == "@grammar" {
            match grammar::parse_grammar_row(&source, &translation) {
                Ok((grammar::Key::Genders, names)) => {
                    declared.genders.get_or_insert(names);
                }
                Ok((grammar::Key::Forms, names)) => {
                    declared.forms.get_or_insert(names);
                }
                Err(_) => {}
            }
        } else if declared.nouns.len() < MAX_NOUNS
            && let Ok(noun) = grammar::parse_noun(&source, &translation)
        {
            declared.nouns.entry(source).or_insert(noun);
        }
    }
    declared
}

fn parse_row(line: usize, text: &str, spec: &LocaleSpec) -> Result<(Row, Vec<String>), String> {
    let columns: Vec<&str> = text.split('\t').collect();
    let [context, source, translation] = columns.as_slice() else {
        return Err(format!("{} columns; expected 3", columns.len()));
    };
    if let Some(c) = text
        .chars()
        .find(|&c| c != '\t' && (c.is_control() || c == '\u{feff}'))
    {
        return Err(format!(
            "invisible character {c:?} in the row; write \\n or \\t, or delete it"
        ));
    }
    if source.is_empty() {
        return Err("empty source text".into());
    }
    let row = Row {
        line,
        context: unescape(context)?,
        source: unescape(source)?,
        translation: unescape(translation)?,
    };
    if row.translation.is_empty() {
        return Err("empty translation; leave the row out instead".into());
    }
    bidi_problem(&row.translation, spec.rtl)?;
    let mut warnings = Vec::new();
    match row.context.as_str() {
        // The source is a command id, not a template.
        "@id" => {}
        "@plural" => check_count(&row, Count::Plural, spec, &mut warnings)?,
        "@ordinal" => check_count(&row, Count::Ordinal, spec, &mut warnings)?,
        "@grammar" => {
            grammar::parse_grammar_row(&row.source, &row.translation)?;
        }
        "@noun" => {
            let noun = grammar::parse_noun(&row.source, &row.translation)?;
            grammar::check_noun(&row.source, &noun, &spec.grammar)?;
        }
        "@msg" => check_msg(&row)?,
        c if c.starts_with('@') && !RESERVED.contains(&c) => {
            return Err(format!(
                "unknown reserved context {c:?}; use one of {RESERVED:?}"
            ));
        }
        _ => check_plain(&row, spec, &mut warnings)?,
    }
    Ok((row, warnings))
}

/// A translation holds no bidirectional embedding, override or isolate control, and the marks
/// LRM, RLM and ALM only in a right-to-left locale: the app isolates the values it inserts.
fn bidi_problem(translation: &str, rtl: bool) -> Result<(), String> {
    for c in translation.chars() {
        let control = matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}');
        let mark = matches!(c, '\u{200E}' | '\u{200F}' | '\u{061C}');
        if control {
            return Err(format!(
                "bidirectional control U+{:04X} in the translation; the app isolates what it inserts, so write none",
                u32::from(c)
            ));
        }
        if mark && !rtl {
            return Err(format!(
                "U+{:04X} is a bidirectional mark; only the catalogs of right-to-left locales use them",
                u32::from(c)
            ));
        }
    }
    Ok(())
}

/// A plain row: `{name}` placeholders in the translation, or a selector message.
fn check_plain(row: &Row, spec: &LocaleSpec, warnings: &mut Vec<String>) -> Result<(), String> {
    let source = placeholder::parse_source(&row.source)?;
    let ellipsis = row.source.ends_with('…');
    if is_selector_message(&row.translation) {
        return check_selector(
            &row.translation,
            RowKind::Plain,
            &source,
            ellipsis,
            spec,
            warnings,
        );
    }
    let translated = placeholder::translation_names(&row.translation)?;
    if !same_names(&source, &translated) {
        return Err("placeholders differ from the source text".into());
    }
    if ellipsis != row.translation.ends_with('…') {
        return Err("trailing … differs from the source text".into());
    }
    Ok(())
}

fn same_names(source: &Source, translated: &[&str]) -> bool {
    source
        .names
        .iter()
        .map(String::as_str)
        .eq(translated.iter().copied())
}

/// An `@msg` row is checked like a plain one, and its source is a template.
fn check_msg(row: &Row) -> Result<(), String> {
    if is_selector_message(&row.translation) {
        return Err(
            "a selector message is not allowed in an @msg row: its placeholders are text that matched"
                .into(),
        );
    }
    if let Some(typed) = placeholder::scan(&row.source)
        .into_iter()
        .find(|p| p.annotation.is_some())
    {
        return Err(format!(
            "typed placeholder {{{} :{}}} is not allowed in an @msg row: a placeholder there is text that matched",
            typed.name,
            typed.annotation.unwrap_or_default()
        ));
    }
    let source = placeholder::parse_source(&row.source)?;
    let translated = placeholder::translation_names(&row.translation)?;
    if !same_names(&source, &translated) {
        return Err("placeholders differ from the source text".into());
    }
    if row.source.ends_with('…') != row.translation.ends_with('…') {
        return Err("trailing … differs from the source text".into());
    }
    check_template(&row.source)
}

/// An `@msg` source text with `{` is a template whose placeholders match any text. The apps skip
/// a template that can never match, so the string would silently stay English. Rules:
/// - `{` with no closing `}` (craft-launcher's `Template::new`, vectorcraft's `Template::parse`);
/// - two placeholders in a row, as where one ends and the next begins is unknowable (same two);
/// - no literal text, `{x}` alone, which would match every message (vectorcraft's
///   `Template::parse`; craft-launcher accepts it, and the stricter rule holds for both).
fn check_template(source: &str) -> Result<(), String> {
    let mut literal = 0;
    let mut after_placeholder = false;
    let mut rest = source;
    while let Some((before, after_open)) = rest.split_once('{') {
        let Some((_, after)) = after_open.split_once('}') else {
            return Err("@msg template has { with no closing }; it can never match".into());
        };
        if before.is_empty() && after_placeholder {
            return Err("@msg template has two placeholders in a row; it can never match".into());
        }
        literal += before.len();
        after_placeholder = true;
        rest = after;
    }
    if after_placeholder && literal + rest.len() == 0 {
        return Err("@msg template has no literal text; it would match any message".into());
    }
    Ok(())
}

/// A row chosen by a count or by a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Count {
    Plural,
    Ordinal,
}

impl Count {
    fn word(self) -> &'static str {
        match self {
            Count::Plural => "plural",
            Count::Ordinal => "ordinal",
        }
    }

    /// The English forms of the source, in CLDR order.
    fn source_forms(self) -> &'static str {
        match self {
            Count::Plural => "one|other",
            Count::Ordinal => "one|two|few|other",
        }
    }

    fn row_kind(self) -> RowKind {
        match self {
            Count::Plural => RowKind::Plural,
            Count::Ordinal => RowKind::Ordinal,
        }
    }

    /// How many forms the locale's translation lists.
    fn forms(self, spec: &LocaleSpec) -> usize {
        match self {
            Count::Plural => spec.plurals.cardinal.reached.len(),
            Count::Ordinal => spec.plurals.ordinal.reached.len(),
        }
    }
}

/// The English forms of an `@plural` or `@ordinal` source, and what they say of the placeholders
/// together.
fn count_source(source: &str, count: Count) -> Result<(Source, bool), String> {
    let word = count.word();
    let english: Vec<&str> = source.split('|').collect();
    if english.len() != count.source_forms().split('|').count() {
        return Err(format!(
            "{word} source text must be {}",
            count.source_forms()
        ));
    }
    let mut merged = Source::default();
    let mut first = true;
    for form in &english {
        if form.is_empty() {
            return Err(format!(
                "{word} source forms must be nonempty with the same placeholders"
            ));
        }
        let parsed = placeholder::parse_source(form)?;
        if first {
            merged = parsed;
            first = false;
            continue;
        }
        if parsed.names != merged.names {
            return Err(format!(
                "{word} source forms must be nonempty with the same placeholders"
            ));
        }
        for (name, ty) in parsed.types {
            if merged
                .types
                .insert(name.clone(), ty)
                .is_some_and(|earlier| earlier != ty)
            {
                return Err(format!("{word} source forms type {{{name}}} differently"));
            }
        }
    }
    let ellipsis = english.last().is_some_and(|form| form.ends_with('…'));
    Ok((merged, ellipsis))
}

/// An `@plural` or `@ordinal` row: the locale's forms in CLDR order, or a selector message.
fn check_count(
    row: &Row,
    count: Count,
    spec: &LocaleSpec,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let word = count.word();
    let (source, ellipsis) = count_source(&row.source, count)?;
    if is_selector_message(&row.translation) {
        return check_selector(
            &row.translation,
            count.row_kind(),
            &source,
            ellipsis,
            spec,
            warnings,
        );
    }
    let forms = count.forms(spec);
    let translated: Vec<&str> = row.translation.split('|').collect();
    if translated.len() != forms || translated.iter().any(|f| f.is_empty()) {
        return Err(format!("expected {forms} nonempty {word} forms"));
    }
    for form in translated {
        if !same_names(&source, &placeholder::translation_names(form)?) {
            return Err(format!(
                "{word} form placeholders differ from the source text"
            ));
        }
    }
    Ok(())
}

/// A translation that is a selector message.
fn check_selector(
    translation: &str,
    kind: RowKind,
    source: &Source,
    ellipsis: bool,
    spec: &LocaleSpec,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let message = message::parse(translation).map_err(|e| format!("selector message: {e}"))?;
    let outcome = select::check(
        &message,
        RowInfo {
            kind,
            source,
            ellipsis,
        },
        spec,
    );
    warnings.extend(outcome.warnings);
    if outcome.errors.is_empty() {
        Ok(())
    } else {
        Err(format!("selector message: {}", outcome.errors.join("; ")))
    }
}

fn unescape(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => return Err(format!("unknown escape \\{other}; use \\n, \\t or \\\\")),
            None => return Err("trailing backslash; use \\\\ for a backslash".into()),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fr() -> LocaleSpec {
        LocaleSpec::for_locale("fr").unwrap()
    }

    fn ja() -> LocaleSpec {
        LocaleSpec::for_locale("ja").unwrap()
    }

    #[test]
    fn valid_rows_pass() {
        let ok = "# c\n\n\tSave\tEnregistrer\nmenu\tOpen\tOuvrir\n@id\tfile.save\tEnregistrer\n@msg\tno file `{name}`\taucun fichier `{name}`\n@plural\t{n} page|{n} pages\t{n} page|{n} pages\n";
        let (rows, problems) = read_catalog(ok, &fr());
        assert_eq!(problems, Vec::<String>::new());
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].line, 3);
    }

    #[test]
    fn context_distinguishes_keys() {
        assert!(
            read_catalog("\tOpen\tOuvrir\nmenu\tOpen\tOuvert\n", &fr())
                .1
                .is_empty()
        );
    }

    #[test]
    fn each_bad_row_is_one_problem() {
        for (text, line) in [
            ("\tSave\n", 1),
            ("\tSave\tA\tB\n", 1),
            ("\tSave\tA\n\tSave\tB\n", 2),
            ("\tSave\t\n", 1),
            ("\t{n} files\t{count} fichiers\n", 1),
            ("\tOpen…\tOuvrir\n", 1),
            ("\tA\\qB\tA\n", 1),
            ("\tA\\\tA\n", 1),
            ("@nope\tA\tB\n", 1),
            ("@plural\tpage\tpage|pages\n", 1),
            ("@plural\t{n} page|pages\tx|y\n", 1),
            ("@plural\t{n} a|{n} b\t{n} x\n", 1),
            ("@plural\t{n} a|{n} b\t{n} x|y\n", 1),
            ("\tSave\tEnregistrer\r\r\n", 1),
            ("\tSave\tEn\rreg\n", 1),
            ("\tA\tB\n\u{feff}\tC\tD\n", 2),
            ("\t\tVide\n", 1),
            ("@msg\t{x}\t{x}\n", 1),
            ("@msg\tOpen {a}{b}\tOuvrir {a}{b}\n", 1),
            ("@msg\tOpen {a\tOuvrir {a\n", 1),
            ("@ordinal\t{n}st|{n}th\t{n}e|{n}e\n", 1),
            ("@ordinal\t{n}st|{n}nd|{n}rd|{n}th\t{n}e\n", 1),
            ("@grammar\tgender\tmasculine\n", 1),
            ("@noun\tPage\tx\n", 1),
            ("\tSave\tEnregistrer\u{202E}\n", 1),
            ("\tSave\tEnregistrer\u{200F}\n", 1),
            ("\t{n :integer}\t{n :integer}\n", 1),
            ("\t{n :wat}\t{n}\n", 1),
            ("\tx\t.match\n", 1),
        ] {
            let problems = read_catalog(text, &fr()).1;
            assert_eq!(problems.len(), 1, "{text:?}: {problems:?}");
            assert!(
                problems[0].starts_with(&format!("line {line}:")),
                "{text:?}: {problems:?}"
            );
        }
    }

    #[test]
    fn placeholders_may_reorder() {
        assert!(
            read_catalog("\t{a} to {b}\t{b} vers {a}\n", &fr())
                .1
                .is_empty()
        );
    }

    #[test]
    fn escapes_unescape() {
        let (rows, problems) = read_catalog("\tA\\nB\tC\\tD\\\\\n", &fr());
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(rows[0].source, "A\nB");
        assert_eq!(rows[0].translation, "C\tD\\");
    }

    #[test]
    fn one_form_locale() {
        assert!(
            read_catalog("@plural\t{n} a|{n} b\t{n} x\n", &ja())
                .1
                .is_empty()
        );
    }

    #[test]
    fn crlf_and_bom_are_ignored() {
        let (rows, problems) =
            read_catalog("\u{feff}\tSave\tEnregistrer\r\n\tUndo\tAnnuler\r\n", &fr());
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(rows[0].context, "");
        assert_eq!(rows[1].translation, "Annuler");
    }

    #[test]
    fn a_typed_source_takes_a_plain_translation() {
        let (rows, problems) = read_catalog("\tPage {n :integer}\tPage {n}\n", &fr());
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(rows[0].source, "Page {n :integer}");
    }

    #[test]
    fn a_typed_count_takes_plain_forms() {
        let typed = "@plural\t{n :integer} page|{n :integer} pages\t{n} page|{n} pages\n";
        assert_eq!(read_catalog(typed, &fr()).1, Vec::<String>::new());
        let ordinal = "@ordinal\t{n :integer}st|{n}nd|{n}rd|{n}th\t{n}re|{n}e\n";
        assert_eq!(read_catalog(ordinal, &fr()).1, Vec::<String>::new());
        // Two forms that type one count two ways, and a type in a translation, are refused.
        for bad in [
            "@plural\t{n :integer} page|{n :number} pages\t{n} page|{n} pages\n",
            "@plural\t{n} page|{n} pages\t{n :integer} page|{n} pages\n",
        ] {
            assert_eq!(read_catalog(bad, &fr()).1.len(), 1, "{bad}");
        }
    }

    #[test]
    fn the_marks_follow_the_locale() {
        let arabic = LocaleSpec::for_locale("ar").unwrap();
        assert!(read_catalog("\tSave\tحفظ\u{200F}\n", &arabic).1.is_empty());
        assert_eq!(read_catalog("\tSave\tحفظ\u{200F}\n", &fr()).1.len(), 1);
        // A control is no one's.
        assert_eq!(read_catalog("\tSave\tحفظ\u{2067}\n", &arabic).1.len(), 1);
    }

    #[test]
    fn a_file_over_the_limit_is_not_read() {
        let big = format!("# {}\n\tSave\tEnregistrer\n", "x".repeat(MAX_CATALOG_BYTES));
        let reading = read_catalog_report(&big, &fr());
        assert!(reading.rows.is_empty());
        assert_eq!(reading.problems.len(), 1);
        assert!(
            reading.problems[0].contains("2097152"),
            "{:?}",
            reading.problems
        );
    }

    #[test]
    fn declared_collects_grammar_and_nouns() {
        let text = "# c\n@grammar\tgenders\tmasculine feminine\n@grammar\tforms\taccusative\n@noun\tpage\tfeminine: page\n@noun\tbad\tx; =\n\tSave\tEnregistrer\n@grammar\tgenders\tother\n";
        let d = declared(text);
        assert_eq!(
            d.genders.as_deref(),
            Some(&["masculine".to_owned(), "feminine".to_owned()][..])
        );
        assert_eq!(d.forms.as_deref(), Some(&["accusative".to_owned()][..]));
        assert_eq!(d.nouns.len(), 1, "first rows win; a bad noun is left out");
        assert!(d.nouns.contains_key("page"));
    }

    #[test]
    fn declarations_merge_first_wins() {
        let own = declared("@grammar\tgenders\ta b\n@noun\tpage\tx: own\n");
        let shared = declared(
            "@grammar\tgenders\tc\n@grammar\tforms\tf\n@noun\tpage\ty: shared\n@noun\tlayer\ty: layer\n",
        );
        let (grammar, nouns) = Declared::merge([&own, &shared]);
        assert_eq!(grammar.genders, ["a", "b"]);
        assert_eq!(grammar.forms, ["f"]);
        assert_eq!(nouns["page"].base, "own");
        assert_eq!(nouns["layer"].base, "layer");
    }
}
