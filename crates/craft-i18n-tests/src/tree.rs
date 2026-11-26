//! Checking every catalog under a checkout's `locales/`: `locales/<locale>/common.tsv` (shared
//! strings) and `locales/<locale>/<app>.tsv` (one app's strings and overrides).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use crate::catalog::{Declared, Reading, Row, declared, read_catalog_report};
use crate::locale::check_locale_name;
use crate::spec::LocaleSpec;

/// The shared catalog's file name.
pub const COMMON: &str = "common.tsv";

/// (context, source text) to translation: one catalog's rows.
type Rows = HashMap<(String, String), String>;

/// One locale's catalogs: the shared rows and each app's rows by app name (file name without
/// `.tsv`).
#[derive(Default)]
struct Locale {
    common: Rows,
    apps: BTreeMap<String, Rows>,
}

/// What the checker found under `locales/`: problems that fail the tree, and warnings that do not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// One entry of a locale's directory.
enum Entry {
    /// Not named like a catalog.
    Stray,
    /// A catalog that could not be read.
    Unreadable(String),
    /// A catalog, read; whether it has the clean-room line.
    Catalog(Reading, bool),
}

/// One locale directory, read.
struct Dir {
    name: String,
    /// What was wrong with the directory or its locale before any file was read.
    problems: Vec<String>,
    /// Whether its files were read: not a directory, a name that is no locale, or a directory that
    /// cannot be listed leave it out of every other check.
    readable: bool,
    /// The files, sorted, by name.
    files: Vec<(String, Entry)>,
}

/// Every problem under `root/locales`, each prefixed by its path relative to `root`. No
/// `locales/` directory means no locales, and no problems.
#[must_use]
pub fn check_tree(root: &Path) -> Vec<String> {
    check_tree_report(root).errors
}

/// [`check_tree`], with the warnings: what a message asks of a noun that the noun lacks.
#[must_use]
pub fn check_tree_report(root: &Path) -> Report {
    let mut report = Report::default();
    let dirs = match read_tree(root) {
        Ok(dirs) => dirs,
        Err(e) => {
            report.errors.push(format!("locales: {e}"));
            return report;
        }
    };
    let mut catalogs: BTreeMap<String, Locale> = BTreeMap::new();
    for dir in dirs {
        let locale = &dir.name;
        report.errors.extend(
            dir.problems
                .iter()
                .map(|p| format!("locales/{locale}: {p}")),
        );
        if !dir.readable {
            continue;
        }
        let shared: HashSet<Row> = dir
            .files
            .iter()
            .find(|(file, _)| file == COMMON)
            .and_then(|(_, entry)| match entry {
                Entry::Catalog(reading, _) => Some(reading.rows.iter().cloned().map(key).collect()),
                _ => None,
            })
            .unwrap_or_default();
        let loaded = catalogs.entry(locale.clone()).or_default();
        for (file, entry) in dir.files {
            let path = format!("locales/{locale}/{file}");
            let (reading, clean_room) = match entry {
                Entry::Stray => {
                    report.errors.push(format!(
                        "{path}: not a catalog; name it {COMMON} or <app>.tsv (lowercase, digits, -)"
                    ));
                    continue;
                }
                Entry::Unreadable(e) => {
                    report.errors.push(format!("{path}: {e}"));
                    continue;
                }
                Entry::Catalog(reading, clean_room) => (reading, clean_room),
            };
            if !clean_room {
                report.errors.push(format!(
                    "{path}: no \"# Clean room:\" line; copy the header from docs/catalog-format.md"
                ));
            }
            report
                .errors
                .extend(reading.problems.iter().map(|p| format!("{path}: {p}")));
            report
                .warnings
                .extend(reading.warnings.iter().map(|w| format!("{path}: {w}")));
            if file != COMMON {
                for row in reading
                    .rows
                    .iter()
                    .filter(|r| shared.contains(&key((*r).clone())))
                {
                    report.errors.push(format!(
                        "{path}: line {}: same translation as {COMMON}; remove the override",
                        row.line
                    ));
                }
            }
            let table: Rows = reading
                .rows
                .into_iter()
                .map(|r| ((r.context, r.source), r.translation))
                .collect();
            if file == COMMON {
                loaded.common = table;
            } else if let Some(app) = file.strip_suffix(".tsv") {
                loaded.apps.insert(app.to_owned(), table);
            }
        }
    }
    for (locale, loaded) in &catalogs {
        if !has_language_name(loaded) {
            report.errors.push(format!(
                "locales/{locale}: no `@id<TAB>language.name<TAB><name>` row in {COMMON} or an app catalog; the language picker would show the code"
            ));
        }
    }
    report.errors.extend(redundant_regional_rows(&catalogs));
    report
}

/// The directories under `root/locales`, read: each file once, against what it sees declared.
fn read_tree(root: &Path) -> Result<Vec<Dir>, String> {
    let names = entries(&root.join("locales")).map_err(|e| e.to_string())?;
    // First pass: what each catalog declares, so a file can be read against the declarations of
    // its own locale's shared catalog and of its base language.
    let mut declarations: BTreeMap<(String, String), Declared> = BTreeMap::new();
    for locale in &names {
        let dir = root.join("locales").join(locale);
        if !dir.is_dir() || check_locale_name(locale).is_err() {
            continue;
        }
        for file in entries(&dir).unwrap_or_default() {
            if is_catalog_name(&file)
                && let Ok(text) = std::fs::read_to_string(dir.join(&file))
            {
                declarations.insert((locale.clone(), file), declared(&text));
            }
        }
    }
    let known: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    let mut dirs = Vec::new();
    for locale in &names {
        let dir = root.join("locales").join(locale);
        let mut found = Dir {
            name: locale.clone(),
            problems: Vec::new(),
            readable: false,
            files: Vec::new(),
        };
        if !dir.is_dir() {
            found.problems.push("not a locale directory".into());
            dirs.push(found);
            continue;
        }
        if let Err(e) = check_locale_name(locale) {
            found.problems.push(e);
            dirs.push(found);
            continue;
        }
        let base = LocaleSpec::for_locale(locale).unwrap_or_else(|e| {
            found.problems.push(e);
            LocaleSpec::unknown(locale)
        });
        let files = match entries(&dir) {
            Ok(files) => files,
            Err(e) => {
                found.problems.push(e.to_string());
                dirs.push(found);
                continue;
            }
        };
        found.readable = true;
        for file in files {
            if !is_catalog_name(&file) {
                found.files.push((file, Entry::Stray));
                continue;
            }
            let entry = match std::fs::read_to_string(dir.join(&file)) {
                Ok(text) => {
                    let chain = declaration_chain(locale, &file, &known, &declarations);
                    let (grammar, nouns) = Declared::merge(chain);
                    let spec = base.clone().with_declared(grammar, nouns);
                    let clean_room = text.lines().any(|l| l.starts_with("# Clean room:"));
                    Entry::Catalog(read_catalog_report(&text, &spec), clean_room)
                }
                Err(e) => Entry::Unreadable(e.to_string()),
            };
            found.files.push((file, entry));
        }
        dirs.push(found);
    }
    Ok(dirs)
}

/// What a file sees declared, nearest first: itself, its locale's shared catalog, and the same two
/// of the locale it falls back to.
fn declaration_chain<'a>(
    locale: &str,
    file: &str,
    known: &BTreeSet<&str>,
    declarations: &'a BTreeMap<(String, String), Declared>,
) -> Vec<&'a Declared> {
    let mut places: Vec<(String, String)> = vec![(locale.to_owned(), file.to_owned())];
    if file != COMMON {
        places.push((locale.to_owned(), COMMON.to_owned()));
    }
    if let Some(parent) = base_name(locale).filter(|p| known.contains(p.as_str())) {
        if file != COMMON {
            places.push((parent.clone(), file.to_owned()));
        }
        places.push((parent, COMMON.to_owned()));
    }
    places.iter().filter_map(|p| declarations.get(p)).collect()
}

/// The locale `locale` falls back to: its name without a trailing region (two letters or three
/// digits). A script is not a region.
fn base_name(locale: &str) -> Option<String> {
    let (parent, region) = locale.rsplit_once('-')?;
    let is_region = (region.len() == 2 && region.bytes().all(|b| b.is_ascii_lowercase()))
        || (region.len() == 3 && region.bytes().all(|b| b.is_ascii_digit()));
    is_region.then(|| parent.to_owned())
}

/// The row the apps' language pickers read: `@id language.name`.
fn has_language_name(locale: &Locale) -> bool {
    let key = ("@id".to_owned(), "language.name".to_owned());
    locale.common.contains_key(&key) || locale.apps.values().any(|rows| rows.contains_key(&key))
}

/// The locale `locale` falls back to, as in craft-libs' `craft-i18n` (`I18n::new`): only a
/// trailing region subtag (two letters or three digits) is dropped, and only when that locale is
/// in the tree too. `pt-br` has no fallback without a `pt/`; `sr-latn`, `zh-hans` and `zh-hant`
/// never have one, since a script is not a region (`sr-latn` to `sr` would show Cyrillic).
fn base_locale<'a>(locale: &str, catalogs: &'a BTreeMap<String, Locale>) -> Option<&'a Locale> {
    catalogs.get(&base_name(locale)?)
}

/// The keys an app tries for a row, in order: its own, then (for a label context) the plain row
/// of the same source text, as `tc` does in craft-libs. `@id` and `@plural` rows have no plain
/// fallback.
fn lookup_keys(context: &str, source: &str) -> Vec<(String, String)> {
    let own = (context.to_owned(), source.to_owned());
    if context.is_empty() || context == "@id" || context == "@plural" {
        vec![own]
    } else {
        vec![own, (String::new(), source.to_owned())]
    }
}

/// A row of the regional locale hidden from a lookup: its file (`None` for `common.tsv`, else the
/// app) and key.
type Hidden<'a> = (Option<&'a str>, &'a (String, String));

/// What `app` shows for `keys` in `locale`, then in `base`, as craft-libs' `find` walks the
/// chain: per locale, the first key found, the app's rows over the shared ones. `hidden` is a row
/// of `locale` to leave out.
fn resolve<'a>(
    app: Option<&str>,
    keys: &[(String, String)],
    locale: &'a Locale,
    base: &'a Locale,
    hidden: Option<Hidden<'_>>,
) -> Option<&'a str> {
    let in_locale = |l: &'a Locale, hidden: Option<Hidden<'_>>| {
        keys.iter().find_map(|key| {
            let is_hidden = |file: Option<&str>| hidden == Some((file, key));
            let from_app = app
                .filter(|_| !is_hidden(app))
                .and_then(|a| l.apps.get(a))
                .and_then(|rows| rows.get(key));
            from_app
                .or_else(|| l.common.get(key).filter(|_| !is_hidden(None)))
                .map(String::as_str)
        })
    };
    in_locale(locale, hidden).or_else(|| in_locale(base, None))
}

/// Every regional row that removing would not change: for each app that shows it, the lookup
/// gives the same text through the base language, since the runtime falls back region to base
/// (see [`base_locale`]). Reported as `locales/<region>/<file>: "<context>" "<source>": …`.
fn redundant_regional_rows(catalogs: &BTreeMap<String, Locale>) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, locale) in catalogs {
        let Some(base) = base_locale(name, catalogs) else {
            continue;
        };
        let base_name = name.rsplit_once('-').map_or("", |(parent, _)| parent);
        let apps: Vec<&str> = locale
            .apps
            .keys()
            .chain(base.apps.keys())
            .map(String::as_str)
            .collect();
        let files = std::iter::once((None, &locale.common)).chain(
            locale
                .apps
                .iter()
                .map(|(app, rows)| (Some(app.as_str()), rows)),
        );
        for (file, rows) in files {
            // A shared row shows in every app (and in an app with no catalog of its own); an app
            // row only in its app.
            let shown_in: Vec<Option<&str>> = match file {
                Some(app) => vec![Some(app)],
                None => std::iter::once(None)
                    .chain(apps.iter().copied().map(Some))
                    .collect(),
            };
            for key in rows.keys() {
                let keys = lookup_keys(&key.0, &key.1);
                let redundant = shown_in.iter().all(|&app| {
                    let kept = resolve(app, &keys, locale, base, None);
                    kept.is_some() && kept == resolve(app, &keys, locale, base, Some((file, key)))
                });
                if redundant {
                    let file_name = file.map_or_else(|| COMMON.to_owned(), |a| format!("{a}.tsv"));
                    problems.push(format!(
                        "locales/{name}/{file_name}: {:?} {:?}: same translation as {base_name}; remove the row",
                        key.0, key.1
                    ));
                }
            }
        }
    }
    problems.sort();
    problems
}

/// One line per catalog, `<locale>/<file>: <rows>/<max>`, where `<max>` is the most rows any
/// locale's catalog of that name has. A report only: unreadable files count as empty.
#[must_use]
pub fn coverage(root: &Path) -> Vec<String> {
    let mut counts = Vec::new();
    for dir in read_tree(root).unwrap_or_default() {
        for (file, entry) in dir.files {
            if let Entry::Catalog(reading, _) = entry {
                counts.push((dir.name.clone(), file, reading.rows.len()));
            }
        }
    }
    let mut max: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, file, rows) in &counts {
        let m = max.entry(file).or_default();
        *m = (*m).max(*rows);
    }
    counts
        .iter()
        .map(|(locale, file, rows)| {
            let m = max.get(file.as_str()).copied().unwrap_or(0);
            format!("{locale}/{file}: {rows}/{m}")
        })
        .collect()
}

/// A row without its line number, to compare rows across files.
fn key(row: Row) -> Row {
    Row { line: 0, ..row }
}

/// `common.tsv`, or `<app>.tsv` with `<app>` in `[a-z0-9]+(-[a-z0-9]+)*`.
fn is_catalog_name(file: &str) -> bool {
    file.strip_suffix(".tsv").is_some_and(|app| {
        app.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
    })
}

/// The names in `dir`, sorted, without dotfiles. A missing `dir` has none.
fn entries(dir: &Path) -> std::io::Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if !name.starts_with('.') {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}
