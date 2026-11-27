//! `@grammar` and `@noun` rows: the genders and case forms a language has, and the nouns that
//! take them (`docs/catalog-format.md`, "Nouns and grammar").
//!
//! A `@grammar` row is `@grammar<TAB>genders<TAB>masculine feminine` or `…forms…accusative`; a
//! `@noun` row is `@noun<TAB>page<TAB>feminine: strona; accusative=stronę`.

use crate::limits::MAX_FORMS;

/// What a language declares, as lowercase names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grammar {
    pub genders: Vec<String>,
    pub forms: Vec<String>,
}

/// Which list a `@grammar` row declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Genders,
    Forms,
}

/// A noun's translation: its gender, its base form, and the forms it lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Noun {
    pub gender: Option<String>,
    pub base: String,
    /// `(form, text)` in the order written.
    pub forms: Vec<(String, String)>,
}

impl Noun {
    /// Whether the noun lists `form`.
    #[must_use]
    pub fn has_form(&self, form: &str) -> bool {
        self.forms.iter().any(|(name, _)| name == form)
    }
}

/// A name of a gender or a form: lowercase letters, digits and `-`, starting with a letter.
#[must_use]
pub fn is_name(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A `@grammar` row.
///
/// # Errors
///
/// Returns an error when the key is neither `genders` nor `forms`, or a name is not a lowercase
/// name, is repeated, or the row names more than the limit.
pub fn parse_grammar_row(source: &str, translation: &str) -> Result<(Key, Vec<String>), String> {
    let key = match source {
        "genders" => Key::Genders,
        "forms" => Key::Forms,
        other => {
            return Err(format!(
                "unknown @grammar key {other:?}; use `genders` or `forms`"
            ));
        }
    };
    let mut names: Vec<String> = Vec::new();
    for name in translation.split_ascii_whitespace() {
        if !is_name(name) {
            return Err(format!(
                "{name:?} is not a name: write lowercase letters, digits and `-`, starting with a letter"
            ));
        }
        if names.iter().any(|n| n == name) {
            return Err(format!("`{name}` is listed twice"));
        }
        names.push(name.to_owned());
    }
    if names.len() > MAX_FORMS {
        return Err(format!(
            "{} names; at most {MAX_FORMS} in a @grammar row",
            names.len()
        ));
    }
    if names.is_empty() {
        return Err("no names; leave the row out to declare none".into());
    }
    Ok((key, names))
}

/// A `@noun` row: the English noun and its translation.
///
/// # Errors
///
/// Returns an error when the source is not a noun as the app names it, or the translation is not
/// `[gender: ]base[; form=text]…` with every name lowercase, every text present and no form given
/// twice.
pub fn parse_noun(source: &str, translation: &str) -> Result<Noun, String> {
    if !is_noun_source(source) {
        return Err(format!(
            "noun {source:?}: write the English noun in lowercase letters, digits, single spaces and `-`"
        ));
    }
    let mut parts = translation.split(';');
    let first = parts.next().unwrap_or_default();
    let (gender, base) = match first.split_once(':') {
        Some((gender, base)) if is_name(gender) => (Some(gender.to_owned()), base),
        _ => (None, first),
    };
    let base = base.trim();
    if base.is_empty() {
        return Err(format!("noun `{source}` has no base form"));
    }
    let mut forms: Vec<(String, String)> = Vec::new();
    for part in parts {
        let Some((name, text)) = part.split_once('=') else {
            return Err(format!(
                "noun `{source}`: expected `form=text` after `;`, found {:?}",
                part.trim()
            ));
        };
        let (name, text) = (name.trim(), text.trim());
        if !is_name(name) {
            return Err(format!(
                "noun `{source}`: form name {name:?} must be lowercase letters, digits and `-`"
            ));
        }
        if text.is_empty() {
            return Err(format!("noun `{source}`: form `{name}` is empty"));
        }
        if forms.iter().any(|(n, _)| n == name) {
            return Err(format!("noun `{source}`: form `{name}` is given twice"));
        }
        if forms.len() >= MAX_FORMS {
            return Err(format!("noun `{source}`: at most {MAX_FORMS} forms"));
        }
        forms.push((name.to_owned(), text.to_owned()));
    }
    Ok(Noun {
        gender,
        base: base.to_owned(),
        forms,
    })
}

/// An English noun as the app names it: `page`, `background page`.
fn is_noun_source(source: &str) -> bool {
    !source.is_empty()
        && !source.starts_with(' ')
        && !source.ends_with(' ')
        && !source.contains("  ")
        && source
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == ' ' || c == '-')
}

/// A noun against what its language declares.
///
/// # Errors
///
/// Returns an error when the noun's gender or a form is not declared, or a language that
/// declares genders is given a noun with none.
pub fn check_noun(source: &str, noun: &Noun, grammar: &Grammar) -> Result<(), String> {
    match (&noun.gender, grammar.genders.is_empty()) {
        (Some(gender), true) => {
            return Err(format!(
                "noun `{source}` has the gender `{gender}`, but this locale declares no genders; add a `@grammar genders` row or drop the gender"
            ));
        }
        (Some(gender), false) if !grammar.genders.contains(gender) => {
            return Err(format!(
                "noun `{source}`: gender `{gender}` is not declared; this locale declares {}",
                grammar.genders.join(", ")
            ));
        }
        (None, false) => {
            return Err(format!(
                "noun `{source}` has no gender, but this locale declares genders ({}); write `gender: base form`",
                grammar.genders.join(" ")
            ));
        }
        _ => {}
    }
    for (form, _) in &noun.forms {
        if !grammar.forms.contains(form) {
            return Err(if grammar.forms.is_empty() {
                format!(
                    "noun `{source}`: form `{form}` is not declared; this locale declares no forms"
                )
            } else {
                format!(
                    "noun `{source}`: form `{form}` is not declared; this locale declares {}",
                    grammar.forms.join(", ")
                )
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grammar_row_lists_names() {
        assert_eq!(
            parse_grammar_row("genders", "masculine feminine neuter").unwrap(),
            (
                Key::Genders,
                vec!["masculine".into(), "feminine".into(), "neuter".into()]
            )
        );
        assert_eq!(
            parse_grammar_row("forms", " accusative  genitive ")
                .unwrap()
                .1,
            ["accusative", "genitive"]
        );
    }

    #[test]
    fn a_grammar_row_is_refused() {
        for (source, translation, words) in [
            ("gender", "masculine", "genders"),
            ("genders", "Masculine", "Masculine"),
            ("genders", "1st", "1st"),
            ("genders", "a a", "twice"),
            ("genders", "  ", "no names"),
            ("forms", "a b c d e f g h i j k l m n o p q", "at most 16"),
        ] {
            let e = parse_grammar_row(source, translation).unwrap_err();
            assert!(e.contains(words), "{source} {translation}: {e}");
        }
        let sixteen = "a b c d e f g h i j k l m n o p";
        assert!(parse_grammar_row("forms", sixteen).is_ok());
    }

    #[test]
    fn a_noun_has_a_gender_a_base_and_forms() {
        let n = parse_noun("page", "feminine: strona; accusative=stronę").unwrap();
        assert_eq!(n.gender.as_deref(), Some("feminine"));
        assert_eq!(n.base, "strona");
        assert_eq!(n.forms, [("accusative".to_owned(), "stronę".to_owned())]);
        assert!(n.has_form("accusative") && !n.has_form("genitive"));
        let n = parse_noun("page", "ページ").unwrap();
        assert_eq!(n.gender, None);
        assert_eq!(n.base, "ページ");
        let n = parse_noun("background page", "feminine:Hintergrundseite").unwrap();
        assert_eq!(n.base, "Hintergrundseite");
    }

    #[test]
    fn a_colon_that_is_not_a_gender_stays_in_the_base() {
        // A capital, a digit start or a space before the colon: not a gender.
        assert_eq!(parse_noun("page", "Seite: Haupt").unwrap().gender, None);
        assert_eq!(parse_noun("page", "1: x").unwrap().gender, None);
        assert_eq!(parse_noun("page", "die Seite: x").unwrap().gender, None);
    }

    #[test]
    fn a_noun_is_refused() {
        for (source, translation, words) in [
            ("Page", "x", "lowercase"),
            ("", "x", "lowercase"),
            ("a  b", "x", "single spaces"),
            (" a", "x", "lowercase"),
            ("page", "", "no base form"),
            ("page", "feminine: ", "no base form"),
            ("page", "x; accusative", "form=text"),
            ("page", "x; Accusative=y", "lowercase"),
            ("page", "x; accusative=", "`accusative` is empty"),
            ("page", "x; a=1; a=2", "twice"),
        ] {
            let e = parse_noun(source, translation).unwrap_err();
            assert!(e.contains(words), "{source:?} {translation:?}: {e}");
        }
        let many: String = (0..17).map(|k| format!("; f{k}=x")).collect();
        let e = parse_noun("page", &format!("x{many}")).unwrap_err();
        assert!(e.contains("at most 16"), "{e}");
    }

    #[test]
    fn a_noun_is_held_to_the_grammar() {
        let grammar = Grammar {
            genders: vec!["masculine".into(), "feminine".into()],
            forms: vec!["accusative".into()],
        };
        let ok = parse_noun("page", "feminine: Seite; accusative=Seite").unwrap();
        assert!(check_noun("page", &ok, &grammar).is_ok());
        let e =
            check_noun("page", &parse_noun("page", "neuter: x").unwrap(), &grammar).unwrap_err();
        assert!(
            e.contains("`neuter`") && e.contains("masculine, feminine"),
            "{e}"
        );
        let e = check_noun("page", &parse_noun("page", "x").unwrap(), &grammar).unwrap_err();
        assert!(e.contains("no gender"), "{e}");
        let e = check_noun(
            "page",
            &parse_noun("page", "feminine: x; dative=y").unwrap(),
            &grammar,
        )
        .unwrap_err();
        assert!(e.contains("`dative`") && e.contains("accusative"), "{e}");
        let none = Grammar::default();
        assert!(check_noun("page", &parse_noun("page", "x").unwrap(), &none).is_ok());
        let e = check_noun("page", &parse_noun("page", "feminine: x").unwrap(), &none).unwrap_err();
        assert!(e.contains("no genders"), "{e}");
        let e = check_noun(
            "page",
            &parse_noun("page", "x; accusative=y").unwrap(),
            &none,
        )
        .unwrap_err();
        assert!(e.contains("no forms"), "{e}");
    }

    #[test]
    fn names() {
        assert!(is_name("masculine") && is_name("a1-b"));
        assert!(!is_name("") && !is_name("1a") && !is_name("A") && !is_name("a_b"));
    }
}
