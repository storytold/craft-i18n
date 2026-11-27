//! `{name}` placeholders, and the type the English source may give one: `{name :type}`.
//!
//! The translation of a row writes the plain `{name}`; the type belongs to the source (the table
//! in `docs/catalog-format.md`, "Placeholders"). A placeholder with no type is text.

use std::collections::BTreeMap;

/// What a typed placeholder's value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Number,
    Integer,
    Measure,
    Range,
    Date,
    Noun,
}

impl Type {
    /// Every type, as the catalog names it.
    pub const ALL: [Type; 6] = [
        Type::Number,
        Type::Integer,
        Type::Measure,
        Type::Range,
        Type::Date,
        Type::Noun,
    ];

    /// The name after the colon: `number`, `integer`, `measure`, `range`, `date`, `noun`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Type::Number => "number",
            Type::Integer => "integer",
            Type::Measure => "measure",
            Type::Range => "range",
            Type::Date => "date",
            Type::Noun => "noun",
        }
    }

    /// The type a name stands for.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Type> {
        Type::ALL.into_iter().find(|t| t.name() == name)
    }

    /// The names of every type, for a message.
    #[must_use]
    pub fn list() -> String {
        Type::ALL
            .iter()
            .map(|t| format!(":{}", t.name()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// One `{…}` of a text, split into its name and the annotation after ` :`, if it has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placeholder<'a> {
    pub name: &'a str,
    /// The word after ` :`, not yet looked up in [`Type`].
    pub annotation: Option<&'a str>,
}

/// The placeholders of `text` in order. A `{` with no `}` ends the scan. A placeholder is typed
/// when its text ends in a space, a colon and lowercase letters (`{n :integer}`); any other text
/// between the braces is the name, as before types existed.
#[must_use]
pub fn scan(text: &str) -> Vec<Placeholder<'_>> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some((_, after)) = rest.split_once('{') {
        let Some((inner, after)) = after.split_once('}') else {
            break;
        };
        found.push(split(inner));
        rest = after;
    }
    found
}

fn split(inner: &str) -> Placeholder<'_> {
    if let Some((name, annotation)) = inner.rsplit_once(" :")
        && !annotation.is_empty()
        && annotation.bytes().all(|b| b.is_ascii_lowercase())
    {
        return Placeholder {
            name,
            annotation: Some(annotation),
        };
    }
    Placeholder {
        name: inner,
        annotation: None,
    }
}

/// Whether `name` may be a typed placeholder's: letters, digits and `_`, not starting with a digit.
#[must_use]
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// What an English source text says of its placeholders.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Source {
    /// Every placeholder's name, sorted, with repeats.
    pub names: Vec<String>,
    /// The type of each typed placeholder.
    pub types: BTreeMap<String, Type>,
}

impl Source {
    /// The type of a placeholder: `None` for text, or for a name the source lacks.
    #[must_use]
    pub fn type_of(&self, name: &str) -> Option<Type> {
        self.types.get(name).copied()
    }

    /// Whether the source has a placeholder of that name.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    /// The names without repeats.
    #[must_use]
    pub fn distinct(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.names.iter().map(String::as_str).collect();
        names.dedup();
        names
    }
}

/// The placeholders of an English source text.
///
/// # Errors
///
/// Returns an error when a placeholder's type is unknown, its name is not an identifier, or one
/// name is given two types.
pub fn parse_source(text: &str) -> Result<Source, String> {
    let mut source = Source::default();
    for placeholder in scan(text) {
        source.names.push(placeholder.name.to_owned());
        let Some(annotation) = placeholder.annotation else {
            continue;
        };
        let Some(ty) = Type::from_name(annotation) else {
            return Err(format!(
                "unknown placeholder type :{annotation}; the types are {}",
                Type::list()
            ));
        };
        if !is_identifier(placeholder.name) {
            return Err(format!(
                "typed placeholder name {:?} must be letters, digits and `_`, not starting with a digit",
                placeholder.name
            ));
        }
        if let Some(earlier) = source.types.insert(placeholder.name.to_owned(), ty)
            && earlier != ty
        {
            return Err(format!(
                "placeholder {{{}}} is typed :{} and :{}",
                placeholder.name,
                earlier.name(),
                ty.name()
            ));
        }
    }
    source.names.sort_unstable();
    Ok(source)
}

/// The names of a translation's placeholders, sorted.
///
/// # Errors
///
/// Returns an error when a placeholder carries a type: the type belongs to the source.
pub fn translation_names(text: &str) -> Result<Vec<&str>, String> {
    let mut names = Vec::new();
    for placeholder in scan(text) {
        if let Some(annotation) = placeholder.annotation {
            return Err(format!(
                "type annotation :{annotation} on {{{}}} belongs in the source text; write {{{}}}",
                placeholder.name, placeholder.name
            ));
        }
        names.push(placeholder.name);
    }
    names.sort_unstable();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_finds_names_and_annotations() {
        let found = scan("{a} then {n :integer} and { b } {}");
        assert_eq!(
            found,
            [
                Placeholder {
                    name: "a",
                    annotation: None
                },
                Placeholder {
                    name: "n",
                    annotation: Some("integer")
                },
                Placeholder {
                    name: " b ",
                    annotation: None
                },
                Placeholder {
                    name: "",
                    annotation: None
                },
            ]
        );
    }

    #[test]
    fn an_unclosed_brace_ends_the_scan() {
        assert_eq!(scan("{a} {b").len(), 1);
        assert!(scan("{").is_empty());
        assert!(scan("").is_empty());
    }

    #[test]
    fn only_a_space_colon_word_is_an_annotation() {
        assert_eq!(scan("{a:b}")[0].annotation, None);
        assert_eq!(scan("{a :B}")[0].annotation, None);
        assert_eq!(scan("{a : b}")[0].annotation, None);
        assert_eq!(scan("{a :}")[0].annotation, None);
        assert_eq!(
            scan("{a :b :c}")[0],
            Placeholder {
                name: "a :b",
                annotation: Some("c")
            }
        );
    }

    #[test]
    fn source_types_names_and_errors() {
        let s = parse_source("{n :integer} of {total :integer} in {file}").unwrap();
        assert_eq!(s.names, ["file", "n", "total"]);
        assert_eq!(s.type_of("n"), Some(Type::Integer));
        assert_eq!(s.type_of("file"), None);
        assert!(s.has("file") && !s.has("path"));
        assert!(parse_source("{n :nope}").unwrap_err().contains(":nope"));
        assert!(
            parse_source("{the n :integer}")
                .unwrap_err()
                .contains("the n")
        );
        assert!(parse_source("{1n :integer}").is_err());
        assert!(
            parse_source("{n :integer} {n :number}")
                .unwrap_err()
                .contains("{n}")
        );
        assert!(parse_source("{n :integer} {n :integer}").is_ok());
    }

    #[test]
    fn a_translation_has_no_types() {
        assert_eq!(translation_names("{b} {a}").unwrap(), ["a", "b"]);
        assert!(
            translation_names("{a :integer}")
                .unwrap_err()
                .contains("belongs in the source")
        );
    }

    #[test]
    fn identifiers() {
        for ok in ["n", "_x", "item2", "A"] {
            assert!(is_identifier(ok), "{ok}");
        }
        for bad in ["", "2n", "a b", "a-b", "é", "$n"] {
            assert!(!is_identifier(bad), "{bad}");
        }
    }

    #[test]
    fn type_names_round_trip() {
        for t in Type::ALL {
            assert_eq!(Type::from_name(t.name()), Some(t));
        }
        assert_eq!(Type::from_name("text"), None);
    }
}
