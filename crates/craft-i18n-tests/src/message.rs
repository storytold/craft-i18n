//! The selector-message subset of Unicode MessageFormat 2 (`docs/catalog-format.md`, "Selector
//! messages"): declarations, one `.match`, quoted patterns.
//!
//! A message is input: [`parse`] never panics, reads each character once, nests nothing, and
//! refuses what passes the limits of [`crate::limits`]. It reports the first fault with the
//! character it stands at; the rules that need the row and the locale are in [`crate::select`].

use crate::limits::{
    MAX_DECLARATIONS, MAX_KEY_DIGITS, MAX_MESSAGE_BYTES, MAX_PARTS, MAX_SELECTORS, MAX_VARIANTS,
};
use crate::placeholder::{Type, is_identifier};

/// A parsed selector message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub declarations: Vec<Declaration>,
    pub body: Body,
}

/// What follows the declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    /// One pattern, no choice.
    Pattern(Pattern),
    /// `.match $a $b`, then its variants.
    Match {
        selectors: Vec<String>,
        variants: Vec<Variant>,
    },
}

/// A function of the subset: the type of value a variable is read as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Number,
    Integer,
    Measure,
    Range,
    Date,
    Noun,
}

impl Func {
    /// The name after the colon.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.to_type().name()
    }

    fn from_name(name: &str) -> Option<Func> {
        Some(match Type::from_name(name)? {
            Type::Number => Func::Number,
            Type::Integer => Func::Integer,
            Type::Measure => Func::Measure,
            Type::Range => Func::Range,
            Type::Date => Func::Date,
            Type::Noun => Func::Noun,
        })
    }

    fn to_type(self) -> Type {
        match self {
            Func::Number => Type::Number,
            Func::Integer => Type::Integer,
            Func::Measure => Type::Measure,
            Func::Range => Type::Range,
            Func::Date => Type::Date,
            Func::Noun => Type::Noun,
        }
    }
}

/// `select=plural` or `select=ordinal`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Select {
    Plural,
    Ordinal,
}

/// A function and its options: `:number select=ordinal`, `:noun form=genitive`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub func: Func,
    pub select: Option<Select>,
    pub form: Option<String>,
}

/// `.input` or `.local`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Input,
    Local,
}

/// `.input {$n :number}` (the name and the operand are the same variable) or
/// `.local $of = {$item :noun form=genitive}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub kind: Kind,
    /// The variable declared.
    pub name: String,
    /// The variable the call is made on.
    pub operand: String,
    pub call: Call,
}

/// One variant of a `.match`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    /// One key per selector.
    pub keys: Vec<Key>,
    pub pattern: Pattern,
}

/// A key of a variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// `*`.
    Catchall,
    /// A whole number: matches that value only.
    Exact(u32),
    /// A plural category or a gender.
    Name(String),
}

impl Key {
    /// The key as the catalog writes it.
    #[must_use]
    pub fn show(&self) -> String {
        match self {
            Key::Catchall => "*".to_owned(),
            Key::Exact(n) => n.to_string(),
            Key::Name(name) => name.clone(),
        }
    }
}

/// The text of a variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub parts: Vec<Part>,
}

/// A part of a pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Text(String),
    Placeholder { name: String, call: Option<Call> },
}

impl Pattern {
    /// Whether the pattern's last part is text ending in `…`.
    #[must_use]
    pub fn ends_with_ellipsis(&self) -> bool {
        matches!(self.parts.last(), Some(Part::Text(t)) if t.ends_with('…'))
    }
}

/// The first fault of a message, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The character it stands at, counted from 0.
    pub at: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at character {})", self.message, self.at)
    }
}

/// Whether a translation is a selector message: it begins with `.input`, `.local` or `.match`.
#[must_use]
pub fn is_selector_message(translation: &str) -> bool {
    [".input", ".local", ".match"]
        .iter()
        .any(|keyword| translation.starts_with(keyword))
}

/// Parse a selector message.
///
/// # Errors
///
/// Returns the first fault: a construct outside the subset, a limit passed, or text that is not
/// a message.
pub fn parse(text: &str) -> Result<Message, ParseError> {
    if text.len() > MAX_MESSAGE_BYTES {
        return Err(ParseError {
            at: 0,
            message: format!(
                "the message is {} bytes; at most {MAX_MESSAGE_BYTES} bytes",
                text.len()
            ),
        });
    }
    let mut parser = Parser {
        chars: text.chars().collect(),
        at: 0,
    };
    parser.message()
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

type Parsed<T> = Result<T, ParseError>;

/// Whether `c` separates tokens: space, tab, line feed, carriage return.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// `1 key`, `2 keys`.
fn counted(count: usize, noun: &str) -> String {
    format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
}

/// A name of a key, a form or an option's value: lowercase letters, digits and `-`.
fn is_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'
}

impl Parser {
    fn fail<T>(&self, message: impl Into<String>) -> Parsed<T> {
        Err(ParseError {
            at: self.at,
            message: message.into(),
        })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.at.saturating_add(offset)).copied()
    }

    fn bump(&mut self) {
        self.at = self.at.saturating_add(1);
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(is_space) {
            self.bump();
        }
    }

    /// What the parser stands at, for a message.
    fn found(&self) -> String {
        self.peek()
            .map_or_else(|| "the end of the message".to_owned(), |c| format!("`{c}`"))
    }

    fn starts_with(&self, text: &str) -> bool {
        text.chars()
            .enumerate()
            .all(|(k, c)| self.peek_at(k) == Some(c))
    }

    /// Whether the text at the parser is the keyword and then a space (or the end).
    fn at_keyword(&self, keyword: &str) -> bool {
        self.starts_with(keyword) && self.peek_at(keyword.chars().count()).is_none_or(is_space)
    }

    fn keyword(&mut self, keyword: &str) -> bool {
        let hit = self.at_keyword(keyword);
        if hit {
            self.at += keyword.chars().count();
        }
        hit
    }

    fn expect(&mut self, c: char, what: &str) -> Parsed<()> {
        if self.peek() == Some(c) {
            self.bump();
            Ok(())
        } else {
            self.fail(format!("expected {what}, found {}", self.found()))
        }
    }

    fn message(&mut self) -> Parsed<Message> {
        let mut declarations = Vec::new();
        loop {
            self.skip_space();
            let kind = if self.at_keyword(".input") {
                Kind::Input
            } else if self.at_keyword(".local") {
                Kind::Local
            } else {
                break;
            };
            if declarations.len() >= MAX_DECLARATIONS {
                return self.fail(format!("at most {MAX_DECLARATIONS} declarations"));
            }
            declarations.push(match kind {
                Kind::Input => self.input()?,
                Kind::Local => self.local()?,
            });
        }
        let body = if self.keyword(".match") {
            self.matcher()?
        } else if self.starts_with("{{") {
            Body::Pattern(self.pattern()?)
        } else if declarations.is_empty() {
            return self.fail(format!(
                "expected `.input`, `.local` or `.match`, found {}",
                self.found()
            ));
        } else {
            return self.fail(format!(
                "expected `.match` or a pattern `{{{{…}}}}`, found {}",
                self.found()
            ));
        };
        self.skip_space();
        if self.at_keyword(".match") {
            return self.fail("only one `.match` is allowed");
        }
        if self.at_keyword(".input") || self.at_keyword(".local") {
            return self.fail("declarations come before `.match`");
        }
        if self.peek().is_some() {
            return self.fail(format!("unexpected {} after the message", self.found()));
        }
        Ok(Message { declarations, body })
    }

    /// A variable, `$name`.
    fn variable(&mut self) -> Parsed<String> {
        if self.peek() != Some('$') {
            return self.fail(format!(
                "expected a variable `$name`, found {}",
                self.found()
            ));
        }
        self.bump();
        let start = self.at;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.bump();
        }
        let name: String = self
            .chars
            .get(start..self.at)
            .unwrap_or_default()
            .iter()
            .collect();
        if !is_identifier(&name) {
            return self
                .fail("a variable is `$` and letters, digits and `_`, not starting with a digit");
        }
        Ok(name)
    }

    /// A word of lowercase letters, digits and `-`.
    fn word(&mut self) -> String {
        let start = self.at;
        while self.peek().is_some_and(is_name_char) {
            self.bump();
        }
        self.chars
            .get(start..self.at)
            .unwrap_or_default()
            .iter()
            .collect()
    }

    /// `:function` and its options, up to (not including) the closing `}`.
    fn call(&mut self) -> Parsed<Call> {
        if self.peek() != Some(':') {
            return self.fail(format!(
                "a function is needed here: expected `:number`, `:integer`, `:measure`, `:range`, `:date` or `:noun`, found {}",
                self.found()
            ));
        }
        self.bump();
        let start = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
            self.bump();
        }
        let name: String = self
            .chars
            .get(start..self.at)
            .unwrap_or_default()
            .iter()
            .collect();
        let Some(func) = Func::from_name(&name) else {
            self.at = start.saturating_sub(1);
            return self.fail(format!(
                "unknown function `:{name}`; the subset has {}",
                Type::list()
            ));
        };
        let mut call = Call {
            func,
            select: None,
            form: None,
        };
        loop {
            let before = self.at;
            self.skip_space();
            if self.peek() == Some('}') {
                return Ok(call);
            }
            if self.at == before {
                return self.fail(format!(
                    "expected an option or `}}`, found {}",
                    self.found()
                ));
            }
            self.option(&mut call)?;
        }
    }

    fn option(&mut self, call: &mut Call) -> Parsed<()> {
        let start = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric()) {
            self.bump();
        }
        let name: String = self
            .chars
            .get(start..self.at)
            .unwrap_or_default()
            .iter()
            .collect();
        if name.is_empty() {
            return self.fail(format!(
                "expected an option or `}}`, found {}",
                self.found()
            ));
        }
        if name != "select" && name != "form" {
            self.at = start;
            return self.fail(format!(
                "unknown option `{name}`; the subset has `select` and `form`"
            ));
        }
        self.expect('=', "`=` after the option's name")?;
        let value = self.word();
        if value.is_empty() {
            return self.fail(format!("option `{name}` needs a value"));
        }
        if name == "select" {
            if call.select.is_some() {
                return self.fail("option `select` is given twice");
            }
            call.select = Some(match value.as_str() {
                "plural" => Select::Plural,
                "ordinal" => Select::Ordinal,
                other => {
                    return self.fail(format!(
                        "select={other} is not in the subset; use `select=plural` or `select=ordinal`"
                    ));
                }
            });
        } else {
            if call.form.is_some() {
                return self.fail("option `form` is given twice");
            }
            call.form = Some(value);
        }
        Ok(())
    }

    /// `.input {$n :number}`, after the keyword.
    fn input(&mut self) -> Parsed<Declaration> {
        self.keyword(".input");
        self.skip_space();
        self.expect('{', "`{` after `.input`")?;
        self.skip_space();
        let name = self.variable()?;
        self.skip_space();
        let call = self.call_or_missing(".input")?;
        self.expect('}', "`}`")?;
        Ok(Declaration {
            kind: Kind::Input,
            operand: name.clone(),
            name,
            call,
        })
    }

    /// `.local $of = {$item :noun form=genitive}`, after the keyword.
    fn local(&mut self) -> Parsed<Declaration> {
        self.keyword(".local");
        self.skip_space();
        let name = self.variable()?;
        self.skip_space();
        self.expect('=', "`=` after the variable of `.local`")?;
        self.skip_space();
        self.expect('{', "`{` after `=`")?;
        self.skip_space();
        let operand = self.variable()?;
        self.skip_space();
        let call = self.call_or_missing(".local")?;
        self.expect('}', "`}`")?;
        Ok(Declaration {
            kind: Kind::Local,
            name,
            operand,
            call,
        })
    }

    /// The call of a declaration; a declaration with none says so.
    fn call_or_missing(&mut self, keyword: &str) -> Parsed<Call> {
        if self.peek() == Some('}') {
            return self.fail(format!("`{keyword}` needs a function: `{{$name :number}}`"));
        }
        self.call()
    }

    /// `.match`'s selectors and variants, after the keyword.
    fn matcher(&mut self) -> Parsed<Body> {
        let mut selectors = Vec::new();
        loop {
            self.skip_space();
            if self.peek() != Some('$') {
                break;
            }
            if selectors.len() >= MAX_SELECTORS {
                return self.fail(format!("at most {MAX_SELECTORS} selectors"));
            }
            selectors.push(self.variable()?);
            if !self.peek().is_none_or(is_space) {
                break;
            }
        }
        if selectors.is_empty() {
            return self.fail(format!("`.match` needs a variable, found {}", self.found()));
        }
        let mut variants = Vec::new();
        loop {
            self.skip_space();
            if self.peek().is_none()
                || self.at_keyword(".match")
                || self.at_keyword(".input")
                || self.at_keyword(".local")
            {
                break;
            }
            if variants.len() >= MAX_VARIANTS {
                return self.fail(format!("at most {MAX_VARIANTS} variants"));
            }
            variants.push(self.variant(selectors.len())?);
        }
        if variants.is_empty() {
            return self.fail("`.match` needs at least one variant");
        }
        Ok(Body::Match {
            selectors,
            variants,
        })
    }

    /// `key … {{pattern}}`, with `count` keys.
    fn variant(&mut self, count: usize) -> Parsed<Variant> {
        let mut keys = Vec::new();
        loop {
            self.skip_space();
            if self.starts_with("{{") {
                break;
            }
            if self.peek().is_none() {
                return self.fail(
                    "no pattern after the keys of the last variant: expected `{{…}}`, found the end of the message",
                );
            }
            if keys.len() >= count {
                return self.fail(format!(
                    "{} for {}: a variant has one key per selector, then `{{{{…}}}}`",
                    counted(keys.len().saturating_add(1), "key"),
                    counted(count, "selector"),
                ));
            }
            keys.push(self.key()?);
        }
        if keys.len() != count {
            return self.fail(format!(
                "{} for {}: a variant has one key per selector",
                counted(keys.len(), "key"),
                counted(count, "selector"),
            ));
        }
        let pattern = self.pattern()?;
        Ok(Variant { keys, pattern })
    }

    fn key(&mut self) -> Parsed<Key> {
        let start = self.at;
        let key = match self.peek() {
            Some('*') => {
                self.bump();
                Key::Catchall
            }
            Some(c) if c.is_ascii_digit() => {
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.bump();
                }
                let digits: String = self
                    .chars
                    .get(start..self.at)
                    .unwrap_or_default()
                    .iter()
                    .collect();
                if digits.len() > MAX_KEY_DIGITS {
                    self.at = start;
                    return self.fail(format!("key `{digits}`: at most {MAX_KEY_DIGITS} digits"));
                }
                if digits.len() > 1 && digits.starts_with('0') {
                    self.at = start;
                    return self.fail(format!(
                        "key `{digits}`: a whole number has no leading zero"
                    ));
                }
                match digits.parse::<u32>() {
                    Ok(n) => Key::Exact(n),
                    Err(_) => {
                        self.at = start;
                        return self.fail(format!("key `{digits}` is not a whole number"));
                    }
                }
            }
            Some(c) if c.is_ascii_lowercase() => Key::Name(self.word()),
            _ => {
                return self.fail(format!(
                    "expected a key (a category, a gender, a whole number or `*`), found {}",
                    self.found()
                ));
            }
        };
        match self.peek() {
            None => Ok(key),
            Some(c) if is_space(c) || c == '{' => Ok(key),
            Some(_) => self.fail(format!("unexpected {} in a key", self.found())),
        }
    }

    /// `{{…}}`.
    fn pattern(&mut self) -> Parsed<Pattern> {
        self.expect('{', "`{{`")?;
        self.expect('{', "`{{`")?;
        let mut parts: Vec<Part> = Vec::new();
        let mut text = String::new();
        loop {
            match self.peek() {
                None => return self.fail("expected `}}` to close the pattern"),
                Some('\\') => {
                    self.bump();
                    match self.peek() {
                        Some(c @ ('{' | '}' | '|' | '\\')) => {
                            text.push(c);
                            self.bump();
                        }
                        Some(c) => {
                            return self.fail(format!(
                                "unknown escape `\\{c}`; the escapes are `\\{{`, `\\}}`, `\\|` and `\\\\`"
                            ));
                        }
                        None => return self.fail("a pattern ends in an unfinished escape `\\`"),
                    }
                }
                Some('{') => {
                    if !text.is_empty() {
                        self.push_part(&mut parts, Part::Text(std::mem::take(&mut text)))?;
                    }
                    let placeholder = self.placeholder()?;
                    self.push_part(&mut parts, placeholder)?;
                }
                Some('}') => {
                    if self.peek_at(1) == Some('}') {
                        self.bump();
                        self.bump();
                        break;
                    }
                    return self.fail("a `}` in text is written `\\}`");
                }
                Some(c) => {
                    text.push(c);
                    self.bump();
                }
            }
        }
        if !text.is_empty() {
            self.push_part(&mut parts, Part::Text(text))?;
        }
        if parts.is_empty() {
            return self.fail("empty pattern: a variant needs text or a placeholder");
        }
        Ok(Pattern { parts })
    }

    fn push_part(&self, parts: &mut Vec<Part>, part: Part) -> Parsed<()> {
        if parts.len() >= MAX_PARTS {
            return self.fail(format!("at most {MAX_PARTS} parts in a pattern"));
        }
        parts.push(part);
        Ok(())
    }

    /// `{$name}` or `{$name :function options}`.
    fn placeholder(&mut self) -> Parsed<Part> {
        self.bump();
        if matches!(self.peek(), Some('#' | '/')) {
            return self.fail("markup (`{#…}`, `{/…}`) is not in the subset");
        }
        self.skip_space();
        if self.peek() != Some('$') {
            return self.fail(format!(
                "expected a variable `{{$name}}` in a placeholder, found {}; text with a brace is written `\\{{`",
                self.found()
            ));
        }
        let name = self.variable()?;
        let before = self.at;
        self.skip_space();
        let call = if self.peek() == Some('}') {
            None
        } else if self.at == before {
            return self.fail(format!("expected `}}`, found {}", self.found()));
        } else {
            Some(self.call()?)
        };
        self.expect('}', "`}`")?;
        Ok(Part::Placeholder { name, call })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> Message {
        parse(text).unwrap_or_else(|e| panic!("{text:?}: {e}"))
    }

    fn err(text: &str) -> ParseError {
        match parse(text) {
            Ok(m) => panic!("{text:?} parsed: {m:?}"),
            Err(e) => e,
        }
    }

    #[test]
    fn a_match_parses() {
        let m = ok(".input {$n :number} .match $n one {{{$n} page}} 2 {{two}} * {{{$n} pages}}");
        assert_eq!(m.declarations.len(), 1);
        let Body::Match {
            selectors,
            variants,
        } = &m.body
        else {
            panic!("not a match");
        };
        assert_eq!(selectors, &["n"]);
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0].keys, [Key::Name("one".into())]);
        assert_eq!(variants[1].keys, [Key::Exact(2)]);
        assert_eq!(variants[2].keys, [Key::Catchall]);
        assert_eq!(
            variants[0].pattern.parts,
            [
                Part::Placeholder {
                    name: "n".into(),
                    call: None
                },
                Part::Text(" page".into())
            ]
        );
    }

    #[test]
    fn options_and_locals_parse() {
        let m = ok(
            ".input {$n :number select=ordinal} .local $g = {$item :noun form=genitive} .match $n * {{{$g}}}",
        );
        assert_eq!(m.declarations[0].call.select, Some(Select::Ordinal));
        assert_eq!(m.declarations[1].kind, Kind::Local);
        assert_eq!(m.declarations[1].operand, "item");
        assert_eq!(m.declarations[1].call.form.as_deref(), Some("genitive"));
    }

    #[test]
    fn a_pattern_alone_follows_declarations() {
        let m = ok(".local $y = {$x :integer}{{Hello {$y :integer}}}");
        assert!(matches!(m.body, Body::Pattern(_)));
    }

    #[test]
    fn whitespace_may_vary() {
        ok(".input\n{ $n\t:number }\r\n.match   $n\n*\n{{x}}");
        ok(".match $n * {{x}}");
    }

    #[test]
    fn escapes_and_braces_in_text() {
        let m = ok(r".match $n * {{a \{b\} \| \\ c}}");
        let Body::Match { variants, .. } = m.body else {
            panic!();
        };
        assert_eq!(
            variants[0].pattern.parts,
            [Part::Text("a {b} | \\ c".into())]
        );
        // A pattern may end right after a placeholder, and start with one.
        ok(".match $n * {{{$n}}}");
        ok(".match $n * {{{$n}{$n}}}");
    }

    #[test]
    fn selectors_and_variants_count_keys() {
        ok(".match $a $b 1 x {{y}} * * {{z}}");
        assert!(
            err(".match $a $b 1 {{y}}")
                .message
                .contains("1 key for 2 selectors")
        );
        assert!(
            err(".match $a 1 x {{y}}")
                .message
                .contains("2 keys for 1 selector")
        );
    }

    #[test]
    fn the_faults_name_the_construct() {
        for (text, words) in [
            ("", "expected `.input`"),
            ("hello", "expected `.input`"),
            (".input", "expected `{`"),
            (".input {$n}", "needs a function"),
            (".input {$n :wat}", "unknown function `:wat`"),
            (".input {n :number}", "a variable"),
            (".input {$n :number x=1}", "unknown option `x`"),
            (".input {$n :number select=exact}", "select=exact"),
            (".input {$n :number select=plural select=ordinal}", "twice"),
            (".input {$n :number form=a form=b}", "twice"),
            (".input {$n :number select}", "`=`"),
            (".input {$n :number select=}", "needs a value"),
            (".input {$n :number} {{x}} y", "unexpected `y`"),
            (".input {$n :number}", "expected `.match` or a pattern"),
            (".local $y {$x :number}", "`=`"),
            (".match", "needs a variable"),
            (".match $n", "at least one variant"),
            (".match $n * {{x}} .match $n * {{y}}", "only one `.match`"),
            (".match $n * {{x}} .input {$n :number}", "before `.match`"),
            (".match $n * {{x", "`}}`"),
            (".match $n * {{x}", "a `}` in text"),
            (".match $n * {{x}} z", "no pattern"),
            (".match $n * {{}}", "empty pattern"),
            (".match $n * {{{#b}x{/b}}}", "markup"),
            (".match $n * {{{x}}}", "expected a variable"),
            (".match $n * {{{$n x}}}", "a function is needed"),
            (".match $n * {{a } b}}", "`\\}`"),
            (r".match $n * {{\q}}", "unknown escape `\\q`"),
            (".match $n * {{\\", "unfinished escape"),
            (".match $n 01 {{x}}", "leading zero"),
            (".match $n 1234567890 {{x}}", "at most 9 digits"),
            (".match $n One {{x}}", "expected a key"),
            (".match $n a.b {{x}}", "in a key"),
            (".match $n $n $n $n * * * * {{x}}", "at most 3 selectors"),
        ] {
            let e = err(text);
            assert!(e.message.contains(words), "{text:?}: {e}");
            assert!(e.at <= text.chars().count(), "{text:?}: {e}");
        }
    }

    #[test]
    fn the_limits_are_counted() {
        let many = |variants: usize| -> String {
            let list: String = (0..variants).map(|k| format!(" {k} {{{{x}}}}")).collect();
            format!(".match $n{list}")
        };
        assert!(parse(&many(64)).is_ok());
        assert!(err(&many(65)).message.contains("at most 64 variants"));
        let decls = |count: usize| -> String {
            format!(
                "{}.match $n * {{{{x}}}}",
                ".input {$n :number} ".repeat(count)
            )
        };
        assert!(err(&decls(9)).message.contains("at most 8 declarations"));
        assert!(parse(&decls(8)).is_ok());
        let parts =
            |count: usize| -> String { format!(".match $n * {{{{{}}}}}", "{$n}".repeat(count)) };
        assert!(parse(&parts(64)).is_ok());
        assert!(err(&parts(65)).message.contains("at most 64 parts"));
        let long = format!(".match $n * {{{{{}}}}}", "x".repeat(MAX_MESSAGE_BYTES));
        assert!(err(&long).message.contains("at most 4096 bytes"));
    }

    #[test]
    fn is_selector_message_reads_the_start_only() {
        assert!(is_selector_message(".match $n * {{x}}"));
        assert!(is_selector_message(".input"));
        assert!(is_selector_message(".local"));
        assert!(!is_selector_message(" .match"));
        assert!(!is_selector_message("a .match"));
        assert!(!is_selector_message("{n} pages"));
    }

    #[test]
    fn an_ellipsis_ends_a_pattern() {
        let m = ok(".match $n * {{Open…}}");
        let Body::Match { variants, .. } = m.body else {
            panic!();
        };
        assert!(variants[0].pattern.ends_with_ellipsis());
        let m = ok(".match $n * {{…{$n}}}");
        let Body::Match { variants, .. } = m.body else {
            panic!();
        };
        assert!(
            !variants[0].pattern.ends_with_ellipsis(),
            "the last part is a placeholder"
        );
    }

    #[test]
    fn non_ascii_text_and_positions_are_characters() {
        let m = ok(".match $n * {{ünï…}}");
        assert!(matches!(m.body, Body::Match { .. }));
        // The fault stands at the sixth character, not the sixth byte.
        let e = err(".match ü");
        assert_eq!(e.at, 7);
    }
}
