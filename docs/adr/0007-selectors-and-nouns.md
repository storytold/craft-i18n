---
status: proposed
---

# Rows that must choose their words: typed placeholders, `@ordinal`, selector messages, nouns

`@plural` rows choose a translation by a cardinal count, and only by that. A language needs more
than that to say an interface sentence correctly:

- **Ordinals.** "3rd place" is `3.` in German, `3e` in French, `第3` in Chinese; CLDR has its own
  categories for them, apart from the cardinal ones.
- **Words in place of the number.** Arabic says "no shapes", "one shape" and "two shapes" with
  words; the classic list requires `{n}` in every form.
- **Gender and case.** "Delete this page?" is "Diese Seite", "Diesen Verbinder", "Dieses Bild"; in
  Polish the noun also takes the accusative ("tę stronę"). A sentence about an object whose kind is
  a value (page, connector, picture) cannot be one string.
- **Ranges.** The plural word of "2–5 pages" follows a CLDR table from the categories of both ends.

Fluent and ICU MessageFormat 1 cover this. [ADR 0001](0001-tsv-catalogs-keyed-by-english.md) chose
TSV keyed by English over Fluent because the catalogs, the shared rows and the contributors are
this format's, and that still holds. So the format grows, in the places a translation needs it and
nowhere else.

## Decision

Rows stay `context<TAB>English<TAB>translation`, keyed by the English text. A row that needs
nothing more is read exactly as before. Added, all in
[`docs/catalog-format.md`](../catalog-format.md):

1. **Typed placeholders in the source.** `{n :integer}`, `{when :date}`, `{item :noun}`; types
   `number`, `integer`, `measure`, `range`, `date`, `noun`. The translation writes `{n}`. The type
   says how the app writes the value for the user's region and what a translation may choose by.
2. **`@ordinal` rows**, beside `@plural`: English `one|two|few|other`, the translation the
   locale's CLDR ordinal forms in CLDR order.
3. **Selector messages.** A translation that begins with `.input`, `.local` or `.match` is a
   message in a small subset of [Unicode MessageFormat 2](https://unicode.org/reports/tr35/tr35-messageFormat.html):
   declarations, one `.match` of one to three variables, quoted patterns, `{$name}` placeholders,
   exact whole-number keys, plural, ordinal and range categories, genders, `*`. No markup, no
   other functions, no other options. MF2 is Unicode's standard for exactly this, so a translator
   learns something documented and a later tool can read it. The subset is what a catalog needs;
   everything else of MF2 is refused by name.
4. **`@grammar` and `@noun` rows.** A language declares its genders and case forms once, in its
   shared catalog, and gives each noun the app names a gender and named forms. A language with
   neither declares nothing.
5. **Bidirectional text.** The app isolates the values it inserts; a translation holds no
   embedding, override or isolate control, and the marks LRM, RLM and ALM only in the catalogs of
   right-to-left languages.
6. **Bounds.** A file, a row, a message, the variants, selectors, declarations and parts of a
   message, the nouns and the forms have limits, written in the format document. The checker
   refuses what the runtime would skip, so the two cannot disagree about input.

The checker's rules are the format document's "What the checker requires of a selector message".
The one that matters most: a `*` stands for `other`, so **every category the whole numbers 0 to
1000 reach, other than `other`, has a variant of its own** (or an exact variant for each of its
numbers). A Polish message that forgets `many` is an error, not a silent wrong form. A variant
may leave out the number it chooses by only when its key is an exact number or a category that
holds exactly one whole number (Arabic `zero`, `one`, `two`; Polish `one`), so a translation can
say it in words and never hides a count.

## Alternatives

- **Fluent.** Kept out by ADR 0001.
- **ICU MessageFormat 1.** Nests, so a translator can write a message nobody can check; its
  `select` and `plural` have no standard form for ranges, nouns or forms.
- **A syntax of our own.** Read by no one else; MF2 costs nothing more.
- **More `@` contexts for each need** (`@gender`, `@case`). Each would be one more special form
  for translators, none could combine a count and a gender.

## Consequences

- **The runtime side is a separate change** to the `craft-i18n` crate in craft-libs: ordinal
  rules, the subset's evaluator, nouns, caps. Until an app's runtime has it, it skips a row with a
  context or a message it does not understand, logs it and shows English, as it does for any row
  it cannot read; nothing breaks.
- **The shared catalogs gain `@grammar` rows** for the languages whose agreement the apps need
  first. They are data, not strings to translate: `tools/weblate_bridge.py` does not offer them.
- **The checker reads ICU4X directly** for ordinal categories and plural ranges, which the runtime
  crate has no call for yet, and checks its cardinal categories against the runtime crate's form
  count for every locale here. ICU4X's compiled data lacks two locales the runtime has (Central
  Kurdish and Dhivehi, both `one` and `other`): for them the checker takes the runtime's count and
  is cautious about which numbers are `one`. When the runtime exposes its rules the checker takes
  them from it.
- **A range cannot show its two ends apart.** A `range` value is shown as `from–to`; a sentence
  that needs the ends in separate places types them as two numbers.
- **A selector message is checked like any row**, by file and line, in CI.
