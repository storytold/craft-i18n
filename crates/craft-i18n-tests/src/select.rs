//! The rules a parsed selector message is held to: its variables and functions fit the English
//! source, its keys are the locale's, every category the whole numbers reach has a variant, and
//! every variant keeps the source's placeholders (`docs/catalog-format.md`, "What the checker
//! requires of a selector message").
//!
//! The checks run in three stages and stop at the first that finds a fault, so one mistake is one
//! report and not a cascade: the declarations; the selectors, keys and the variables the patterns
//! use; then coverage, the placeholders each variant keeps and the trailing `…`.

use std::collections::{BTreeMap, BTreeSet};

use crate::message::{Body, Call, Func, Key, Kind, Message, Part, Pattern, Select, Variant};
use crate::placeholder::{Source, Type};
use crate::plural::{Category, Rule};
use crate::spec::LocaleSpec;

/// What kind of row the message translates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// A plain row: any context that does not start with `@`.
    Plain,
    /// An `@plural` row: `{n}` is a count.
    Plural,
    /// An `@ordinal` row: `{n}` is a position.
    Ordinal,
}

impl RowKind {
    fn is_count(self) -> bool {
        self != RowKind::Plain
    }

    fn context(self) -> &'static str {
        match self {
            RowKind::Plain => "plain",
            RowKind::Plural => "@plural",
            RowKind::Ordinal => "@ordinal",
        }
    }
}

/// The row a message translates, as far as the message's rules need it.
#[derive(Clone, Copy, Debug)]
pub struct RowInfo<'a> {
    pub kind: RowKind,
    pub source: &'a Source,
    /// Whether the English source ends in `…`.
    pub ellipsis: bool,
}

/// What a check found. Warnings do not fail a catalog.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// The most nouns a warning names.
const WARNED_NOUNS: usize = 5;

/// A declared variable: what it stands for in the source, and how it is read.
#[derive(Clone, Debug)]
struct Var {
    /// The placeholder of the source it comes from.
    root: String,
    call: Call,
    /// The source's type for the placeholder.
    ty: Type,
}

/// What a `.match` chooses by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    /// A number's category: cardinal or ordinal.
    Number(Select),
    Range,
    Noun,
}

struct Checker<'a> {
    row: RowInfo<'a>,
    spec: &'a LocaleSpec,
    env: BTreeMap<String, Var>,
    errors: Vec<String>,
}

/// Check a parsed message against its row and locale.
#[must_use]
pub fn check(message: &Message, row: RowInfo<'_>, spec: &LocaleSpec) -> Outcome {
    let mut checker = Checker {
        row,
        spec,
        env: BTreeMap::new(),
        errors: Vec::new(),
    };
    checker.declarations(message);
    if !checker.errors.is_empty() {
        return checker.finish(Vec::new());
    }
    let uses = checker.structure(message);
    if !checker.errors.is_empty() {
        return checker.finish(Vec::new());
    }
    checker.completeness(message, &uses);
    if !checker.errors.is_empty() {
        return checker.finish(Vec::new());
    }
    let warnings = checker.form_warnings(message);
    checker.finish(warnings)
}

/// `a`, `a and b`, `a, b and c`, each in backticks.
fn names(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("`{item}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn category_names(categories: &[Category]) -> String {
    names(
        &categories
            .iter()
            .map(|c| c.name().to_owned())
            .collect::<Vec<_>>(),
    )
}

fn without_other(categories: &[Category]) -> Vec<Category> {
    categories
        .iter()
        .copied()
        .filter(|c| *c != Category::Other)
        .collect()
}

impl Checker<'_> {
    fn finish(self, warnings: Vec<String>) -> Outcome {
        Outcome {
            errors: self.errors,
            warnings,
        }
    }

    fn error(&mut self, text: String) {
        if self.errors.len() < 20 && !self.errors.contains(&text) {
            self.errors.push(text);
        }
    }

    /// The type the source gives a placeholder; the count of a count row is a number.
    fn source_type(&self, name: &str) -> Option<Option<Type>> {
        if !self.row.source.has(name) {
            return None;
        }
        Some(match self.row.source.type_of(name) {
            Some(ty) => Some(ty),
            None if name == "n" && self.row.kind.is_count() => Some(Type::Number),
            None => None,
        })
    }

    /// Whether `func` can read a placeholder the source types `ty`, and its options suit `func`.
    fn fit(&self, name: &str, call: &Call, ty: Option<Type>) -> Result<Type, String> {
        let Some(ty) = ty else {
            return Err(format!(
                "`:{}` does not fit `${name}`: it is plain text in the source text, so type it there ({{{name} :{}}})",
                call.func.name(),
                call.func.name()
            ));
        };
        let fits = matches!(
            (call.func, ty),
            (Func::Number | Func::Integer, Type::Number | Type::Integer)
                | (Func::Measure, Type::Measure)
                | (Func::Range, Type::Range)
                | (Func::Date, Type::Date)
                | (Func::Noun, Type::Noun)
        );
        if !fits {
            return Err(format!(
                "`:{}` does not fit `${name}`: the source text types it as {}",
                call.func.name(),
                ty.name()
            ));
        }
        if call.select.is_some() && !matches!(call.func, Func::Number | Func::Integer) {
            return Err(format!(
                "option `select` does not apply to `:{}`; it is for `:number` and `:integer`",
                call.func.name()
            ));
        }
        if call.form.is_some() && call.func != Func::Noun {
            return Err(format!(
                "option `form` does not apply to `:{}`; it is for `:noun`",
                call.func.name()
            ));
        }
        Ok(ty)
    }

    /// The count of a count row is read the way its row says: a count by category, a position by
    /// ordinal category.
    fn count_select(&self, name: &str, call: &Call) -> Result<(), String> {
        if name != "n"
            || !self.row.kind.is_count()
            || !matches!(call.func, Func::Number | Func::Integer)
        {
            return Ok(());
        }
        let selected = call.select.unwrap_or(Select::Plural);
        match (self.row.kind, selected) {
            (RowKind::Plural, Select::Ordinal) => Err(format!(
                "in an {} row `$n` is a count: drop `select=ordinal`",
                self.row.kind.context()
            )),
            (RowKind::Ordinal, Select::Plural) => Err(format!(
                "in an {} row `$n` is a position: write `.input {{$n :number select=ordinal}}`",
                self.row.kind.context()
            )),
            _ => Ok(()),
        }
    }

    fn declarations(&mut self, message: &Message) {
        let mut used: BTreeSet<String> = BTreeSet::new();
        for declaration in &message.declarations {
            let name = &declaration.name;
            let root = match declaration.kind {
                Kind::Input => {
                    if !self.row.source.has(name) {
                        self.error(format!("`${name}` is not a placeholder of the source text"));
                        continue;
                    }
                    name.clone()
                }
                Kind::Local => {
                    if self.row.source.has(name) {
                        self.error(format!(
                            "`.local ${name}` takes the name of a placeholder of the source text; choose another"
                        ));
                        continue;
                    }
                    let operand = &declaration.operand;
                    let root = match self.env.get(operand) {
                        Some(var) => var.root.clone(),
                        None if self.row.source.has(operand) => operand.clone(),
                        None => {
                            self.error(format!(
                                "`${operand}` is not a placeholder of the source text or an earlier declaration"
                            ));
                            continue;
                        }
                    };
                    used.insert(operand.clone());
                    root
                }
            };
            if self.env.contains_key(name) {
                self.error(format!("`${name}` is declared twice"));
                continue;
            }
            if used.contains(name) {
                self.error(format!("`${name}` is used before its `.input` declaration"));
                continue;
            }
            let ty = self.source_type(&root).flatten();
            let checked = self
                .fit(&root, &declaration.call, ty)
                .and_then(|ty| self.count_select(&root, &declaration.call).map(|()| ty));
            match checked {
                Ok(ty) => {
                    self.env.insert(
                        name.clone(),
                        Var {
                            root,
                            call: declaration.call.clone(),
                            ty,
                        },
                    );
                    self.check_form(&declaration.call);
                }
                Err(e) => self.error(e),
            }
        }
    }

    /// A `form=` names a form the locale declares.
    fn check_form(&mut self, call: &Call) {
        let Some(form) = &call.form else {
            return;
        };
        if self.spec.grammar.forms.contains(form) {
            return;
        }
        let declared = if self.spec.grammar.forms.is_empty() {
            "this locale declares no forms".to_owned()
        } else {
            format!(
                "this locale declares {}",
                self.spec.grammar.forms.join(", ")
            )
        };
        self.error(format!("form `{form}` is not declared; {declared}"));
    }

    /// What a `.match` chooses by, for one selector.
    fn choice(var: &Var) -> Option<Choice> {
        match var.ty {
            Type::Number | Type::Integer | Type::Measure => {
                Some(Choice::Number(var.call.select.unwrap_or(Select::Plural)))
            }
            Type::Range => Some(Choice::Range),
            Type::Noun => Some(Choice::Noun),
            Type::Date => None,
        }
    }

    fn rule(&self, select: Select) -> &Rule {
        match select {
            Select::Plural => &self.spec.plurals.cardinal,
            Select::Ordinal => &self.spec.plurals.ordinal,
        }
    }

    /// Stage two: the selectors and keys, and the variables the patterns use. Returns the
    /// placeholders of the source each pattern uses, one set per pattern.
    fn structure(&mut self, message: &Message) -> Vec<BTreeSet<String>> {
        match &message.body {
            Body::Pattern(pattern) => vec![self.pattern_uses(pattern)],
            Body::Match {
                selectors,
                variants,
            } => {
                let choices = self.selectors(selectors);
                self.variants(&choices, variants);
                variants
                    .iter()
                    .map(|variant| self.pattern_uses(&variant.pattern))
                    .collect()
            }
        }
    }

    /// What each selector chooses by; a fault in one is reported and the selector is left out.
    fn selectors(&mut self, selectors: &[String]) -> Vec<Option<(Choice, Var)>> {
        selectors
            .iter()
            .map(|name| {
                let Some(var) = self.env.get(name).cloned() else {
                    if self.row.source.has(name) {
                        self.error(format!(
                            "`${name}` must be declared before `.match` chooses by it: `.input {{${name} :number}}`"
                        ));
                    } else {
                        self.error(format!("`${name}` is not a placeholder of the source text"));
                    }
                    return None;
                };
                let Some(choice) = Self::choice(&var) else {
                    self.error(format!(
                        "`${name}` is a date: a `.match` cannot choose by it"
                    ));
                    return None;
                };
                Some((choice, var))
            })
            .collect()
    }

    fn variants(&mut self, choices: &[Option<(Choice, Var)>], variants: &[Variant]) {
        let mut seen: Vec<&[Key]> = Vec::new();
        for variant in variants {
            for (key, choice) in variant.keys.iter().zip(choices) {
                if let Some((choice, var)) = choice {
                    self.check_key(key, *choice, var);
                }
            }
            if seen.contains(&variant.keys.as_slice()) {
                self.error(format!("variant `{}` is written twice", label(variant)));
            }
            seen.push(&variant.keys);
        }
        if !variants
            .iter()
            .any(|v| v.keys.iter().all(|k| *k == Key::Catchall))
        {
            self.error("a `.match` needs a variant whose keys are all `*`".to_owned());
        }
    }

    fn check_key(&mut self, key: &Key, choice: Choice, var: &Var) {
        let name = match key {
            Key::Catchall => return,
            Key::Exact(n) => {
                if !matches!(choice, Choice::Number(_)) {
                    let what = if choice == Choice::Range {
                        "a range"
                    } else {
                        "a noun"
                    };
                    self.error(format!(
                        "key `{n}`: an exact number chooses a number, and `${}` is {what}",
                        var.root
                    ));
                }
                return;
            }
            Key::Name(name) => name,
        };
        match choice {
            Choice::Number(select) => {
                let defined = self.rule(select).defined.clone();
                self.check_category(name, &defined, "plural");
            }
            Choice::Range => {
                let defined = self.spec.plurals.ranges.defined.clone();
                self.check_category(name, &defined, "range");
            }
            Choice::Noun => {
                if !self.spec.grammar.genders.contains(name) {
                    let declared = if self.spec.grammar.genders.is_empty() {
                        "this locale declares no genders: use `*`".to_owned()
                    } else {
                        format!(
                            "this locale declares {}",
                            self.spec.grammar.genders.join(", ")
                        )
                    };
                    self.error(format!("key `{name}` is not a gender; {declared}"));
                }
            }
        }
    }

    fn check_category(&mut self, name: &str, defined: &[Category], what: &str) {
        match Category::from_name(name) {
            Some(Category::Other) => {
                self.error("key `other`: write the category `other` as `*`".to_owned())
            }
            Some(c) if defined.contains(&c) => {}
            _ => {
                let has: Vec<String> = defined
                    .iter()
                    .filter(|c| **c != Category::Other)
                    .map(|c| c.name().to_owned())
                    .collect();
                let has = if has.is_empty() {
                    "only `other`, written `*`".to_owned()
                } else {
                    format!("{} and `other`, written `*`", names(&has))
                };
                self.error(format!(
                    "key `{name}` is not a {what} category of this locale, which has {has}"
                ));
            }
        }
    }

    /// The placeholders of the source a pattern uses; a variable that is neither a placeholder nor
    /// declared is reported.
    fn pattern_uses(&mut self, pattern: &Pattern) -> BTreeSet<String> {
        let mut roots = BTreeSet::new();
        for part in &pattern.parts {
            let Part::Placeholder { name, call } = part else {
                continue;
            };
            let root = match self.env.get(name) {
                Some(var) => var.root.clone(),
                None if self.row.source.has(name) => name.clone(),
                None => {
                    self.error(format!("`${name}` is not a placeholder of the source text"));
                    continue;
                }
            };
            if let Some(call) = call {
                let ty = self.source_type(&root).flatten();
                if let Err(e) = self.fit(&root, call, ty) {
                    self.error(e);
                }
                self.check_form(call);
            }
            roots.insert(root);
        }
        roots
    }

    /// Stage three: coverage, the placeholders each variant keeps, the trailing `…`.
    fn completeness(&mut self, message: &Message, uses: &[BTreeSet<String>]) {
        let Body::Match {
            selectors,
            variants,
        } = &message.body
        else {
            if let (Body::Pattern(pattern), Some(used)) = (&message.body, uses.first()) {
                self.keeps(&[], used, "the pattern", pattern);
            }
            return;
        };
        let choices: Vec<Option<(Choice, Var)>> = selectors
            .iter()
            .map(|s| {
                let var = self.env.get(s)?.clone();
                Some((Self::choice(&var)?, var))
            })
            .collect();
        for (position, choice) in choices.iter().enumerate() {
            if let Some((choice, _)) = choice {
                self.coverage(position, *choice, selectors, variants);
            }
        }
        for (variant, used) in variants.iter().zip(uses) {
            let exempt = self.exempt(&choices, variant);
            self.keeps(
                &exempt,
                used,
                &format!("variant `{}`", label(variant)),
                &variant.pattern,
            );
        }
    }

    /// Every category the whole numbers reach has a variant: its own, or exact numbers for every
    /// number that has it.
    fn coverage(
        &mut self,
        position: usize,
        choice: Choice,
        selectors: &[String],
        variants: &[Variant],
    ) {
        let key_at = |variant: &'_ Variant| variant.keys.get(position).cloned();
        let (required, what, rule) = match choice {
            Choice::Number(select) => {
                let rule = self.rule(select);
                (
                    without_other(&rule.reached),
                    "whole numbers from 0 to 1000",
                    Some(rule),
                )
            }
            Choice::Range => (
                without_other(&self.spec.plurals.ranges.reached),
                "ranges between whole numbers from 0 to 1000",
                None,
            ),
            Choice::Noun => return,
        };
        let covered = |category: Category| {
            let named = variants
                .iter()
                .any(|v| key_at(v) == Some(Key::Name(category.name().to_owned())));
            named
                || rule.is_some_and(|rule| {
                    let members = rule.members(category);
                    !members.is_empty()
                        && members
                            .iter()
                            .all(|m| variants.iter().any(|v| key_at(v) == Some(Key::Exact(*m))))
                })
        };
        let missing: Vec<Category> = required.into_iter().filter(|c| !covered(*c)).collect();
        if !missing.is_empty() {
            let selector = selectors.get(position).map_or("?", String::as_str);
            let noun = if missing.len() == 1 {
                "category"
            } else {
                "categories"
            };
            self.error(format!(
                "`${selector}`: no variant for the {noun} {}, which {what} reach in this locale; `*` stands for `other`",
                category_names(&missing)
            ));
        }
    }

    /// The placeholders of the source a variant may leave out: the number a selector chooses by,
    /// when the key is an exact number or a category that holds one whole number.
    fn exempt(&self, choices: &[Option<(Choice, Var)>], variant: &Variant) -> Vec<String> {
        let mut exempt = Vec::new();
        for (key, choice) in variant.keys.iter().zip(choices) {
            let Some((Choice::Number(select), var)) = choice else {
                continue;
            };
            let single = match key {
                Key::Exact(_) => true,
                Key::Name(name) => {
                    Category::from_name(name).is_some_and(|c| self.rule(*select).is_single(c))
                }
                Key::Catchall => false,
            };
            if single {
                exempt.push(var.root.clone());
            }
        }
        exempt
    }

    /// A pattern keeps every placeholder of the source (but those `exempt`) and agrees with the
    /// source on a trailing `…`.
    fn keeps(&mut self, exempt: &[String], used: &BTreeSet<String>, what: &str, pattern: &Pattern) {
        let missing: Vec<String> = self
            .row
            .source
            .distinct()
            .into_iter()
            .filter(|name| !used.contains(*name) && !exempt.iter().any(|e| e == name))
            .map(|name| format!("{{{name}}}"))
            .collect();
        if !missing.is_empty() {
            self.error(format!("{what} leaves out {}", missing.join(", ")));
        }
        if pattern.ends_with_ellipsis() != self.row.ellipsis {
            self.error(format!("trailing … of {what} differs from the source text"));
        }
    }

    /// One warning for each form the message asks for that some noun of the locale lacks.
    fn form_warnings(&self, message: &Message) -> Vec<String> {
        let mut forms: BTreeSet<String> = BTreeSet::new();
        let mut note = |call: &Call| {
            if let Some(form) = &call.form {
                forms.insert(form.clone());
            }
        };
        for declaration in &message.declarations {
            note(&declaration.call);
        }
        let patterns: Vec<&Pattern> = match &message.body {
            Body::Pattern(pattern) => vec![pattern],
            Body::Match { variants, .. } => variants.iter().map(|v| &v.pattern).collect(),
        };
        for pattern in patterns {
            for part in &pattern.parts {
                if let Part::Placeholder {
                    call: Some(call), ..
                } = part
                {
                    note(call);
                }
            }
        }
        let mut warnings = Vec::new();
        for form in forms {
            let lacking: Vec<String> = self
                .spec
                .nouns
                .iter()
                .filter(|(_, noun)| !noun.has_form(&form))
                .map(|(source, _)| source.clone())
                .collect();
            if lacking.is_empty() {
                continue;
            }
            let shown: Vec<String> = lacking.iter().take(WARNED_NOUNS).cloned().collect();
            let more = lacking.len().saturating_sub(shown.len());
            let tail = if more == 0 {
                String::new()
            } else {
                format!(" and {more} more")
            };
            let noun = if lacking.len() == 1 { "noun" } else { "nouns" };
            warnings.push(format!(
                "form `{form}`: {noun} {}{tail} {} no such form, so the base form is shown",
                names(&shown),
                if lacking.len() == 1 { "has" } else { "have" }
            ));
        }
        warnings
    }
}

/// A variant's keys, as the catalog writes them.
fn label(variant: &Variant) -> String {
    variant
        .keys
        .iter()
        .map(Key::show)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{Grammar, Noun};
    use crate::message::parse;
    use crate::placeholder::parse_source;

    fn spec(locale: &str, genders: &[&str], forms: &[&str]) -> LocaleSpec {
        LocaleSpec::for_locale(locale).unwrap().with_declared(
            Grammar {
                genders: genders.iter().map(|g| (*g).to_owned()).collect(),
                forms: forms.iter().map(|g| (*g).to_owned()).collect(),
            },
            BTreeMap::new(),
        )
    }

    fn run(kind: RowKind, source: &str, message: &str, spec: &LocaleSpec) -> Outcome {
        let src = parse_source(source).unwrap();
        let message = parse(message).unwrap();
        check(
            &message,
            RowInfo {
                kind,
                source: &src,
                ellipsis: source.ends_with('…'),
            },
            spec,
        )
    }

    fn one_error(outcome: &Outcome, words: &str) {
        assert_eq!(outcome.errors.len(), 1, "{:?}", outcome.errors);
        assert!(
            outcome.errors[0].contains(words),
            "{:?} lacks {words:?}",
            outcome.errors
        );
    }

    #[test]
    fn german_plural_with_an_exact_key_passes() {
        let de = spec("de", &[], &[]);
        let ok = run(
            RowKind::Plural,
            "{n} page",
            ".input {$n :number} .match $n 0 {{Keine}} one {{Eine}} * {{{$n} Seiten}}",
            &de,
        );
        assert_eq!(ok, Outcome::default());
    }

    #[test]
    fn missing_categories_are_named_together() {
        let ar = spec("ar", &[], &[]);
        let out = run(
            RowKind::Plural,
            "{n} shape",
            ".input {$n :number} .match $n one {{واحد}} * {{{$n}}}",
            &ar,
        );
        one_error(&out, "`zero`, `two`, `few`, `many`");
        assert!(!out.errors[0].contains("`one`,"));
    }

    #[test]
    fn exact_numbers_cover_a_category_all_of_whose_numbers_they_name() {
        let de = spec("de", &[], &[]);
        let out = run(
            RowKind::Plural,
            "{n} page",
            ".input {$n :number} .match $n 1 {{Eine}} * {{{$n} Seiten}}",
            &de,
        );
        assert_eq!(out, Outcome::default());
        // French `one` is 0 and 1: naming only 1 leaves 0 on `*`.
        let fr = spec("fr", &[], &[]);
        let out = run(
            RowKind::Plural,
            "{n} page",
            ".input {$n :number} .match $n 1 {{{$n} page}} * {{{$n} pages}}",
            &fr,
        );
        one_error(&out, "`one`");
        let out = run(
            RowKind::Plural,
            "{n} page",
            ".input {$n :number} .match $n 0 {{{$n} page}} 1 {{{$n} page}} * {{{$n} pages}}",
            &fr,
        );
        assert_eq!(out, Outcome::default());
    }

    #[test]
    fn a_selector_must_fit_and_be_declared() {
        let de = spec("de", &["masculine", "feminine"], &[]);
        let out = run(
            RowKind::Plain,
            "Open {name}",
            ".input {$name :number} .match $name * {{{$name}}}",
            &de,
        );
        one_error(&out, "plain text");
        let out = run(
            RowKind::Plain,
            "Saved {when :date}",
            ".input {$when :date} .match $when * {{{$when}}}",
            &de,
        );
        one_error(&out, "a date");
        let out = run(
            RowKind::Plain,
            "{n :integer} x",
            ".match $n * {{{$n}}}",
            &de,
        );
        one_error(&out, "must be declared");
        let out = run(
            RowKind::Plain,
            "{n :integer} x",
            ".input {$m :number} .match $m * {{{$m}}}",
            &de,
        );
        one_error(&out, "`$m` is not a placeholder");
    }

    #[test]
    fn declarations_are_ordered_and_unique() {
        let de = spec("de", &[], &[]);
        let src = "{n :integer} x";
        let out = run(
            RowKind::Plain,
            src,
            ".input {$n :number} .input {$n :integer} {{x {$n}}}",
            &de,
        );
        one_error(&out, "twice");
        let out = run(
            RowKind::Plain,
            src,
            ".local $m = {$n :number} .input {$n :number} {{x {$m}}}",
            &de,
        );
        one_error(&out, "before its `.input`");
        let out = run(
            RowKind::Plain,
            src,
            ".local $n = {$n :number} {{x {$n}}}",
            &de,
        );
        one_error(&out, "name of a placeholder");
        let out = run(
            RowKind::Plain,
            src,
            ".local $m = {$k :number} {{x {$n}}}",
            &de,
        );
        one_error(&out, "`$k`");
        // A local may take another local.
        let out = run(
            RowKind::Plain,
            src,
            ".local $m = {$n :number} .local $o = {$m :integer} {{{$o}}}",
            &de,
        );
        assert_eq!(out, Outcome::default());
    }

    #[test]
    fn count_rows_read_the_count_their_own_way() {
        let fr = spec("fr", &[], &[]);
        let out = run(
            RowKind::Ordinal,
            "{n}st|{n}nd|{n}rd|{n}th",
            ".input {$n :number} .match $n one {{{$n}re}} * {{{$n}e}}",
            &fr,
        );
        one_error(&out, "select=ordinal");
        let out = run(
            RowKind::Plural,
            "{n} a|{n} b",
            ".input {$n :number select=ordinal} .match $n * {{{$n}}}",
            &fr,
        );
        one_error(&out, "select=ordinal");
    }

    #[test]
    fn options_belong_to_their_functions() {
        let de = spec("de", &["masculine"], &["accusative"]);
        let src = "{item :noun} {n :integer}";
        let out = run(
            RowKind::Plain,
            src,
            ".input {$n :number form=accusative} {{{$n} {$item}}}",
            &de,
        );
        one_error(&out, "option `form`");
        let out = run(
            RowKind::Plain,
            src,
            ".input {$item :noun select=ordinal} {{{$n} {$item}}}",
            &de,
        );
        one_error(&out, "option `select`");
    }

    #[test]
    fn keys_are_the_locales_own() {
        let fr = spec("fr", &["masculine", "feminine"], &[]);
        let out = run(
            RowKind::Plural,
            "{n} a",
            ".input {$n :number} .match $n one {{{$n}}} few {{{$n}}} * {{{$n}}}",
            &fr,
        );
        one_error(&out, "`few`");
        let out = run(
            RowKind::Plural,
            "{n} a",
            ".input {$n :number} .match $n one {{{$n}}} other {{{$n}}} * {{{$n}}}",
            &fr,
        );
        one_error(&out, "`other`");
        let out = run(
            RowKind::Plain,
            "Delete {item :noun}",
            ".input {$item :noun} .match $item neuter {{x {$item}}} * {{y {$item}}}",
            &fr,
        );
        one_error(&out, "not a gender");
        let ja = spec("ja", &[], &[]);
        let out = run(
            RowKind::Plain,
            "Delete {item :noun}",
            ".input {$item :noun} .match $item feminine {{x {$item}}} * {{y {$item}}}",
            &ja,
        );
        one_error(&out, "declares no genders");
        let out = run(
            RowKind::Plain,
            "{s :range}",
            ".input {$s :range} .match $s 3 {{x {$s}}} * {{y {$s}}}",
            &fr,
        );
        one_error(&out, "a range");
        let out = run(
            RowKind::Plain,
            "Delete {item :noun}",
            ".input {$item :noun} .match $item 3 {{x {$item}}} * {{y {$item}}}",
            &fr,
        );
        one_error(&out, "a noun");
    }

    #[test]
    fn variants_are_unique_and_there_is_a_catch_all() {
        let ja = spec("ja", &[], &[]);
        let out = run(
            RowKind::Plural,
            "{n} a",
            ".input {$n :number} .match $n 1 {{{$n}}} 1 {{{$n}}} * {{{$n}}}",
            &ja,
        );
        one_error(&out, "written twice");
        let out = run(
            RowKind::Plural,
            "{n} a",
            ".input {$n :number} .match $n 1 {{{$n}}} 2 {{{$n}}}",
            &ja,
        );
        one_error(&out, "`*`");
        let out = run(
            RowKind::Plain,
            "{n :integer} {item :noun}",
            ".input {$n :integer} .input {$item :noun} .match $n $item 1 * {{{$item}}} 2 * {{{$n}}}",
            &spec("ja", &[], &[]),
        );
        one_error(&out, "`*`");
    }

    #[test]
    fn a_variant_keeps_the_placeholders_but_the_exempt_number() {
        let pl = spec("pl", &[], &[]);
        let ok = run(
            RowKind::Plural,
            "{n} a",
            ".input {$n :number} .match $n one {{jeden}} few {{{$n} b}} many {{{$n} c}} * {{{$n} d}}",
            &pl,
        );
        assert_eq!(ok, Outcome::default());
        let out = run(
            RowKind::Plural,
            "{n} a",
            ".input {$n :number} .match $n one {{jeden}} few {{trzy}} many {{{$n} c}} * {{{$n} d}}",
            &pl,
        );
        one_error(&out, "variant `few` leaves out {n}");
        // The exemption is the selector's own number only.
        let out = run(
            RowKind::Plain,
            "{n :integer} in {file}",
            ".input {$n :integer} .match $n 1 {{eins}} * {{{$n} in {$file}}}",
            &spec("de", &[], &[]),
        );
        one_error(&out, "variant `1` leaves out {file}");
    }

    #[test]
    fn a_range_is_chosen_by_the_categories_its_ends_reach() {
        let fr = spec("fr", &[], &[]);
        let out = run(
            RowKind::Plain,
            "{s :range}",
            ".input {$s :range} .match $s * {{{$s}}}",
            &fr,
        );
        one_error(&out, "`one`");
        let ok = run(
            RowKind::Plain,
            "{s :range}",
            ".input {$s :range} .match $s one {{a {$s}}} * {{b {$s}}}",
            &fr,
        );
        assert_eq!(ok, Outcome::default());
        let ja = spec("ja", &[], &[]);
        let ok = run(
            RowKind::Plain,
            "{s :range}",
            ".input {$s :range} .match $s * {{{$s}}}",
            &ja,
        );
        assert_eq!(ok, Outcome::default());
    }

    #[test]
    fn an_ellipsis_must_agree() {
        let ja = spec("ja", &[], &[]);
        let out = run(
            RowKind::Plain,
            "Open {n :integer}…",
            ".input {$n :integer} .match $n * {{{$n}}}",
            &ja,
        );
        one_error(&out, "trailing …");
        // The last part decides: text ending in … is one, a placeholder is not.
        let ok = run(
            RowKind::Plain,
            "Open {n :integer}…",
            ".input {$n :integer} .match $n * {{{$n}…}}",
            &ja,
        );
        assert_eq!(ok, Outcome::default());
        let out = run(
            RowKind::Plain,
            "Open {n :integer}…",
            ".input {$n :integer} .match $n * {{…{$n}}}",
            &ja,
        );
        one_error(&out, "trailing …");
        let out = run(
            RowKind::Plain,
            "Open {n :integer}",
            ".input {$n :integer} .match $n * {{{$n}…}}",
            &ja,
        );
        one_error(&out, "trailing …");
    }

    #[test]
    fn a_pattern_alone_keeps_every_placeholder() {
        let de = spec("de", &["feminine"], &["genitive"]);
        let ok = run(
            RowKind::Plain,
            "Rename {item :noun}",
            ".input {$item :noun} .local $of = {$item :noun form=genitive} {{Von {$of}}}",
            &de,
        );
        assert_eq!(ok, Outcome::default());
        let out = run(
            RowKind::Plain,
            "Rename {item :noun} {n :integer}",
            ".input {$item :noun} {{Von {$item}}}",
            &de,
        );
        one_error(&out, "the pattern leaves out {n}");
    }

    #[test]
    fn a_form_is_declared_and_a_missing_form_warns() {
        let mut de = spec("de", &["feminine"], &["accusative"]);
        let out = run(
            RowKind::Plain,
            "Rename {item :noun}",
            ".input {$item :noun} {{Von {$item :noun form=dative}}}",
            &de,
        );
        one_error(&out, "`dative`");
        de.nouns.insert(
            "page".into(),
            Noun {
                gender: Some("feminine".into()),
                base: "Seite".into(),
                forms: vec![],
            },
        );
        de.nouns.insert(
            "picture".into(),
            Noun {
                gender: Some("feminine".into()),
                base: "Bild".into(),
                forms: vec![("accusative".into(), "Bild".into())],
            },
        );
        let out = run(
            RowKind::Plain,
            "Rename {item :noun}",
            ".input {$item :noun} {{Von {$item :noun form=accusative}}}",
            &de,
        );
        assert_eq!(out.errors, Vec::<String>::new());
        assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
        assert!(out.warnings[0].contains("`page`") && !out.warnings[0].contains("`picture`"));
    }

    #[test]
    fn errors_stop_at_the_first_stage() {
        // A declaration fault hides the key and coverage faults behind it.
        let de = spec("de", &[], &[]);
        let out = run(
            RowKind::Plural,
            "{n} a",
            ".input {$k :number} .match $k few {{x}}",
            &de,
        );
        one_error(&out, "`$k` is not a placeholder");
    }
}
