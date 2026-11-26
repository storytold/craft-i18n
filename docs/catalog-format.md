# Catalog format

A catalog is one locale's translations of a set of strings: `locales/<locale>/common.tsv` for
shared strings, `locales/<locale>/<app>.tsv` for one app's strings and its overrides of shared
ones. CI (`cargo test --workspace`) checks every rule below.

## Rows

One string per line, three columns separated by a TAB:

```
context <TAB> English source text <TAB> translation
```

- **Source text** is the English exactly as the app writes it (`tl!("Save")`). It is the key:
  English is never translated, so there are no English catalogs.
- **Context** is usually empty. A plain word (`menu`, `preview`, `cameraRaw`) is a label that tells
  apart two meanings of the same English; the app falls back to the plain row. Contexts starting
  with `@` mean something to the apps, and only these exist:
  - `@plural`: a string chosen by a count (below).
  - `@ordinal`: a string chosen by a position, 1st, 2nd, 3rd (below).
  - `@id`: the source is a command id, not English; the translation is the command's label.
  - `@msg`: a longer message (errors, status text); checked like a plain row. With a `{name}`
    placeholder it is a template that matches any text there, and CI rejects one the apps could
    never match: a `{` with no `}`, two placeholders in a row (`{a}{b}`), or no literal text at
    all (`{x}` alone). The apps skip such a row and show English.
  - `@grammar`, `@noun`: the genders and case forms a language has, and the nouns that take them
    (below).
- **Escapes** inside a column: `\n`, `\t`, `\\`. No others.
- `{name}` **placeholders** must appear in both the source text and the translation, in any order
  (the English may type them, below).
- A trailing `…` in one needs one in the other.
- An empty translation is an error: leave the row out and the app shows English.
- Each (context, source text) appears once per catalog.
- Lines starting with `#` and blank lines are ignored. Windows line endings and a byte-order mark
  are fine.
- **Bounds.** A row of more than 16 KiB, a file of more than 2 MiB and rows past the 20,000th of a
  file are errors, because the apps skip them (see Limits).

An app that predates a context or a construct below skips the row, logs it and shows English; it
never fails. Nothing here changes how a row that uses only the contexts `@plural`, `@id`, `@msg`
and the plain ones is read.

## Placeholders

A `{name}` is filled in by the app when the string is shown. In the English source a placeholder
may carry a **type**, `{name :type}`, which says what the value is. The translation always writes
the plain `{name}`: the type belongs to the source, and one in a translation is an error. A
placeholder with no type is text. The type tells the checker what a translation may choose its
words by, and tells the app how to write the value: numbers, ranges and dates as the user's
region writes them.

| Type | The value | Shown as | A translation may select on |
|---|---|---|---|
| (none) | text | as it is | nothing |
| `number` | a number, with or without decimals | the region's number | its plural or ordinal category, or an exact whole number |
| `integer` | a whole number | the region's number | the same |
| `measure` | a number with a unit | the region's number, then the unit the app supplies | the category of its number, as `number` |
| `range` | two numbers, from and to | `from–to` in the region's digits | the plural category of the range (CLDR plural ranges) |
| `date` | a date | the region's date | nothing |
| `noun` | a noun the app names, with its forms (below) | its base form, or the one `form=` names | its gender |

A typed placeholder's name is letters, digits and `_`, starting with a letter or `_` (an untyped
one may be any text, as before). An `@msg` row takes no typed placeholder: in it a placeholder
is text that matched, not a value. The values the app inserts are **isolated**
for bidirectional text by the app (see Bidirectional text), never by the catalog.

```tsv locale=de
	Delete this {item :noun}?	Dieses {item} löschen?
	Page {n :integer} of {total :integer}	Seite {n} von {total}
	Saved on {when :date}	Gespeichert am {when}
```

## Language name

Each `common.tsv` names its language in that language, for the apps' language pickers:

```
@id	language.name	Français
```

Without it, a picker shows the locale code, so CI requires the row in every locale, in its
`common.tsv` or an app's catalog.

## Plurals

An `@plural` row's source text is `one|other` English forms, both with the same placeholders. Its
translation lists the locale's plural forms, `|`-separated, in CLDR order (`zero`, `one`, `two`,
`few`, `many`, `other`, leaving out those the language doesn't use). The number of forms is the
CLDR cardinal categories whole numbers 0 to 1000 reach: 1 for Japanese, 2 for German and French,
3 for Polish and Russian, 6 for Arabic. The count is `{n}`.

```tsv locale=de
@plural	{n} page|{n} pages	{n} Seite|{n} Seiten
```

```tsv locale=pl
@plural	{n} page|{n} pages	{n} strona|{n} strony|{n} stron
```

```tsv locale=ja
@plural	{n} page|{n} pages	{n} ページ
```

The classic list cannot leave `{n}` out of a form, nor tell Arabic's `zero`, `one` and `two` from
each other in words. A translation that must (Arabic "no pages", "one page", "two pages") is a
selector message instead (below).

## Ordinals

An `@ordinal` row is chosen by a position (1st, 2nd, 3rd), the way `@plural` is chosen by a count.
Its source text lists the four English ordinal forms in CLDR order, `one|two|few|other` (1st, 2nd,
3rd, 4th; 11th, 12th and 13th are `other`), all with the same placeholders, the position `{n}`
among them. Its translation lists the locale's ordinal forms the same way: the CLDR ordinal
categories whole numbers 0 to 1000 reach. That is one form for German, Polish, Arabic, Japanese,
Spanish and Chinese, two for French (`one` for 1, `other` for the rest) and four for English.

```tsv locale=de
@ordinal	Move to the {n}st place|Move to the {n}nd place|Move to the {n}rd place|Move to the {n}th place	An die {n}. Stelle verschieben
```

```tsv locale=ja
@ordinal	Move to the {n}st place|Move to the {n}nd place|Move to the {n}rd place|Move to the {n}th place	{n} 番目に移動
```

A position that is not a whole number takes `other`.

## Ranges

A source placeholder typed `range` holds two numbers. What a plural word agrees with in a range
is not simply either end: CLDR gives each language a table from the categories of the two ends to
the category of the range, and where the table is silent the range takes its end's category. A
translation selects on the range's category with `.match` (below), and shows the range as
`{$span}`. The categories it needs a variant for are those a range between whole numbers from 0
to 1000, running upward, can have in the locale.

```tsv locale=fr
	Pages {span :range} selected	.input {$span :range} .match $span one {{Page {$span} sélectionnée}} * {{Pages {$span} sélectionnées}}
```

## Selector messages

A translation chooses its words by a count, a position, a range or the gender of a noun when it
is a **selector message**: a small subset of Unicode MessageFormat 2 (MF2). A translation is one
when it begins with `.input`, `.local` or `.match` (so a plain translation cannot begin with one
of those words). Selector messages are allowed in the translation of a plain row (any context
that does not start with `@`), of `@plural` and of `@ordinal` rows; not in `@id`, `@msg`,
`@noun` and `@grammar` rows.

```
message      = [ declaration *( ws declaration ) ws ] ( match / pattern )
declaration  = input / local
input        = ".input" ws "{" ws variable ws function *( ws option ) ws "}"
local        = ".local" ws variable ws "=" ws "{" ws variable ws function *( ws option ) ws "}"
match        = ".match" 1*3( ws variable ) 1*64( ws keys ws pattern )
keys         = key *( ws key )              ; as many keys as selectors
key          = "*" / whole-number / name    ; a category, a gender, an exact number or the catch-all
pattern      = "{{" *( text / escape / placeholder ) "}}"
placeholder  = "{" ws variable [ ws function *( ws option ) ] ws "}"
function     = ":number" / ":integer" / ":measure" / ":range" / ":date" / ":noun"
option       = "select=" ( "plural" / "ordinal" )       ; on :number, :integer and :measure
             / "form=" name                             ; on :noun
variable     = "$" ( ALPHA / "_" ) *( ALPHA / DIGIT / "_" )
name         = lowercase *( lowercase / DIGIT / "-" )
whole-number = "0" / ( "1"-"9" ) *8DIGIT
escape       = "\{" / "\}" / "\|" / "\\"
ws           = 1*( SP / TAB / LF / CR )
```

`text` is any other character. Everything else of MF2 (markup, other functions, other options,
more than one `.match`, quoted literals, number literals as operands) is not in the subset; the
checker refuses it by name. The TSV escapes apply first, so a literal backslash in a message is
`\\\\` in the file and `\{` is `\\{`.

- `.input {$n :number}` says how `$n`, a placeholder of the source text, is read. `.local $g =
  {$item :noun form=genitive}` names a second reading of a placeholder, to use in patterns. A
  variable is declared once and not used before its declaration.
- `.match $n` chooses among the variants by the value of `$n`, which must be declared with a
  function that suits its type in the source (table above). A variant is `keys {{pattern}}`:
  one key per selector. An exact whole number matches that value only; a category (`one`, `few`,
  …) matches the values whose category it is; a gender matches nouns of that gender; `*` matches
  anything. Of the variants that match, the one that matches best wins, comparing the selectors
  left to right: an exact number beats a category, and a category or gender beats `*`.
- There is always a variant whose keys are all `*`. `other` is never written as a key: `*` is
  the `other` category, and also catches any category a message does not name.
- `select=ordinal` reads a number as a position; the default is `select=plural`.
- In a pattern, `{$n}` writes the value as its source type says, `{$item :noun form=accusative}`
  writes a noun in one of its forms, and `{$span}` writes a range.

```tsv locale=ar
@plural	{n} shape selected|{n} shapes selected	.input {$n :number} .match $n zero {{لا أشكال محددة}} one {{شكل واحد محدد}} two {{شكلان محددان}} few {{{$n} أشكال محددة}} many {{{$n} شكلًا محددًا}} * {{{$n} شكل محدد}}
```

```tsv locale=fr
@ordinal	Move to the {n}st place|Move to the {n}nd place|Move to the {n}rd place|Move to the {n}th place	.input {$n :number select=ordinal} .match $n one {{Déplacer à la {$n}re place}} * {{Déplacer à la {$n}e place}}
```

### What the checker requires of a selector message

Each rule names the row by file and line.

1. The message parses under the grammar above, within the limits below.
2. Every variable is a placeholder of the source text, or a `.local` of the message. A variable
   a `.match` chooses by is declared with a function that fits the source: `:number` or
   `:integer` for a `number` or `integer` placeholder and for `{n}` of an `@plural` or
   `@ordinal` row; `:measure` for a `measure`; `:range` for a `range`; `:noun` for a `noun`
   (`:date` for a `date`, which cannot be chosen by). Text with no type fits no function. The
   option `select` belongs to `:number` and `:integer` only, and `form` to `:noun`; in an
   `@plural` row `{n}` is chosen by count, in an `@ordinal` row by `select=ordinal`.
3. Keys: for a number or measure, the locale's cardinal categories (`select=ordinal`: its
   ordinal ones) or exact whole numbers; for a range, the categories a range of the locale can
   have; for a noun, the genders the locale declares (below). **Every category the locale
   reaches with the whole numbers 0 to 1000, other than `other`, has a variant of its own**
   (for a range, every category its upward ranges reach between such numbers), or else an exact
   variant for each whole number from 0 to 1000 that has it (German `one` is only 1): `*`
   stands for `other`, so a Polish message that forgets `many` is an error and not a silent
   wrong form. Nothing is required for genders: `*` may stand for one of them.
4. Every placeholder of the source text appears in every variant, except that a variant may leave
   out the number it chooses by when its key is an exact number or a category that holds exactly
   one whole number from 0 to 1000 in that locale (Arabic `zero`, `one` and `two`; Polish `one`;
   French ordinal `one`), so a translation can say "one page" in words.
5. A classic `@plural` or `@ordinal` translation keeps today's rule: the locale's forms, in CLDR
   order.
6. A noun's `form=` names a form the locale declares. A noun that lacks the form shows its base
   form, which CI reports as a warning, not an error.
7. Every variant agrees with the source text on a trailing `…`.
8. No translation, selector or not, holds a bidirectional embedding, override or isolate control.

## Nouns and grammar

A language that makes words agree with a noun's gender, or puts a noun in a case, says so once, in
its shared catalog, and gives the nouns the app names.

- **`@grammar` rows** declare what the language has, as lowercase names separated by spaces:
  `@grammar<TAB>genders<TAB>masculine feminine neuter` and `@grammar<TAB>forms<TAB>accusative
  genitive`. A language with no grammatical gender declares no genders, and one with no case
  forms declares no forms. The app's catalog may declare its own, which then replace the shared
  ones for that app; a regional locale takes its base language's.
- **`@noun` rows** give a noun's translation, gender and named forms. The source is the English
  noun as the app names it (`page`, `background page`; lowercase letters, digits, spaces and `-`).
  The translation is `gender: base form; form=text; form=text`: the gender and the forms are
  optional, a form the noun does not list is its base form. A language that declares genders
  gives every noun one; a language that declares none gives none.

```tsv locale=de
@grammar	genders	masculine feminine neuter
@noun	connector	masculine: Verbinder
@noun	page	feminine: Seite
@noun	picture	neuter: Bild
	Delete this {item :noun}?	.input {$item :noun} .match $item feminine {{Diese {$item} löschen?}} masculine {{Diesen {$item} löschen?}} * {{Dieses {$item} löschen?}}
```

```tsv locale=pl
@grammar	forms	accusative genitive instrumental locative
@grammar	genders	masculine feminine neuter
@noun	connector	masculine: łącznik
@noun	page	feminine: strona; accusative=stronę
@noun	picture	masculine: obraz
	Delete this {item :noun}?	.input {$item :noun} .match $item feminine {{Usunąć tę {$item :noun form=accusative}?}} neuter {{Usunąć to {$item :noun form=accusative}?}} * {{Usunąć ten {$item :noun form=accusative}?}}
```

```tsv locale=ar
@grammar	forms	definite
@grammar	genders	masculine feminine
@noun	connector	masculine: موصل; definite=الموصل
@noun	page	feminine: صفحة; definite=الصفحة
@noun	picture	feminine: صورة; definite=الصورة
	Delete this {item :noun}?	.input {$item :noun} .match $item feminine {{حذف هذه {$item :noun form=definite}؟}} * {{حذف هذا {$item :noun form=definite}؟}}
```

```tsv locale=ja
@noun	connector	コネクタ
@noun	page	ページ
@noun	picture	画像
	Delete this {item :noun}?	この{item}を削除しますか？
```

A language with no gender needs no selector: a plain row takes the noun's base form, as the
Japanese row shows.

## Bidirectional text

The app, not the catalog, keeps a right-to-left value from reordering the sentence around it: it
puts each value it inserts in a bidirectional isolate when the wording or the value has a
right-to-left letter. A translation therefore carries no embedding, override or isolate control
(U+202A to U+202E, U+2066 to U+2069), in any row. The marks LRM, RLM and ALM (U+200E, U+200F,
U+061C) are allowed in the catalogs of right-to-left languages only.

## Limits

The apps skip what passes these, and CI refuses it, so the two cannot disagree.

| What | Limit |
|---|---|
| A catalog file | 2 MiB |
| Rows of a file | 20,000 |
| A row | 16 KiB |
| A selector message | 4 KiB |
| Selectors of a `.match` | 3 |
| Variants of a `.match` | 64 |
| Declarations of a message | 8 |
| Parts (text runs and placeholders) of a pattern | 64 |
| `@noun` rows of a file | 2,000 |
| Forms of a noun, names of a `@grammar` row | 16 |

## Locales and files

- Locale directories are lowercase BCP 47: `fr`, `pt-br`, `zh-hans`, `sr-latn`.
- Files are `common.tsv` or `<app>.tsv`, with `<app>` in lowercase letters, digits and `-`.
- Files starting with `.` are ignored; anything else is an error.
- An app row with the same context, source text **and** translation as `common.tsv` is an error:
  that override changes nothing.
- A row of a regional locale (`es-mx`) that gives the same translation as its base language
  (`es`) is an error: the app falls back from region to base, so the row changes nothing. The base
  is compared as the app sees it, its app row, else its shared one. This applies where the app
  really falls back: a locale whose last subtag is a region (two letters or three digits) and
  whose base language has a directory here. Script variants (`sr-latn`, `zh-hans`, `zh-hant`) never
  fall back, since `sr-latn` to `sr` would show Cyrillic, and `pt-br` falls back only if `pt/` exists.
- Partial catalogs are welcome. `cargo test coverage_report -- --nocapture` prints each catalog's
  row count against the largest catalog of the same name; `cargo test warning_report --
  --nocapture` prints the checker's warnings.

## What an app shows

For `pt-br`, an app looks up: `pt-br/<app>.tsv` → `pt-br/common.tsv` → `pt/<app>.tsv` →
`pt/common.tsv` → the English source text. A missing translation never errors.

## Header

Every catalog starts with this header (CI requires the `# Clean room:` line):

```
# <Language> (<locale>) interface catalog for <every Crafting App | AppName>.
# Columns, separated by a TAB:  context <TAB> English source text <TAB> translation
#   context  usually empty. `@plural`, `@ordinal`, `@id`, `@msg`, `@noun` and `@grammar` mean
#            something to the apps; any other word is a label that tells apart two meanings of
#            the same English.
#   escapes  \n  \t  \\  inside a column. `{name}` placeholders must appear in both columns (in any
#            order), and a trailing "…" in one needs one in the other. The English may type a
#            placeholder, `{n :number}`; the translation writes `{n}`.
#   @plural  English `one|other`; the translation lists this language's plural forms, `|`-separated.
#   @ordinal English `one|two|few|other`; the translation lists this language's ordinal forms.
#   .match   a translation that must choose its words by a count, a position or a noun's gender may
#            be a selector message (docs/catalog-format.md). @grammar and @noun rows declare the
#            genders and case forms a language has, and the nouns that take them.
#   Lines starting with # and blank lines are ignored. A missing entry shows in English.
#
# Clean room: every entry was written from the plain meaning of the English text in ordinary
# computing vocabulary. No other product's localisation, string table or translated UI was
# consulted or copied; add entries the same way.
```
