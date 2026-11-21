# Hebrew (`he`) localization: contributor and review guide

> **Status:** Proposed guidance for maintainer and Hebrew-contributor review. This document does **not** appoint a reviewer, override project policy, or claim that a translated catalog is available in a released app.
>
> Applies to Hebrew strings in `craft-i18n` and, where applicable before cutover, app-local Hebrew catalogs. The authoritative catalog contract remains [catalog-format.md](catalog-format.md); [GLOSSARY.md](../GLOSSARY.md) defines shared terms such as *correction*, *override*, *pin*, and *Reviewer*.

## Goal

Make Hebrew UI consistent, idiomatic, accessible, and verifiable across the Crafting Apps while retaining precise distinctions between different tools. Shared strings should be translated once where their **meaning** is shared; context-specific meanings belong in a context-qualified entry or an app override.

This is a **reviewable convention**, not a blanket replacement of existing translations. A different valid phrasing is a preference, not necessarily a correction; follow the repository's existing correction/override policy.

## Clean-room source and provenance

- Work from the **English source text**, the calling code, and the behavior of the Crafting App itself. Write original Hebrew translations. Do **not** consult, compare, import, or copy another product's localized UI or string tables.
- Check the actual UI context before settling ambiguous source words (for example, *Open* as a verb, as a state, or as a paragraph-spacing preset).
- Preserve source keys, command IDs, placeholder names, and public format contracts. Explain any intentional key/context change separately from a wording edit.
- Record AI assistance and the extent of native-speaker review in [ATTRIBUTION.md](../ATTRIBUTION.md); an automated check is not a human linguistic review.
- Respect app-specific licensing and contribution requirements. A successful PR review is distinct from an upstream merge or a released binary.

## Language and terminology

- Use natural, concise **modern Hebrew** for UI. Avoid unnecessary transliteration of established technical terms, but retain product and file-format names when they function as names.
- Prefer short menu labels and clear imperative actions. Distinguish actions from nouns and states; do not mechanically reuse the same Hebrew word whenever the same English spelling appears.
- Prefer gender-neutral wording where it reads naturally; otherwise use a consistent construction for the relevant control and its surrounding screen. Avoid phrasing that incorrectly implies a user's gender.
- Use unpointed Hebrew (without niqqud) in ordinary controls unless the string has a documented special purpose.
- Preserve intentional punctuation, capitalization-sensitive identifiers, key names, file extensions, and trailing Unicode ellipsis (`…`). Do not replace `…` with three periods in a catalog.
- Maintain identical meanings across apps via `locales/he/common.tsv`. For valid domain-specific preferences, use an app override rather than silently rewriting the shared meaning.

### Starter examples already present in the shared Hebrew catalog

These examples document existing entries, **not** a new mandatory translation for every context:

| English source | Existing shared Hebrew |
|---|---|
| Accessibility | נגישות |
| Add | הוספה |
| Align | יישור |
| Animation | הנפשה |
| Apply | החלה |
| Bold | מודגש |

For a button that initiates an action, a context-specific imperative may be clearer than the shared noun. That decision should be justified against the actual source control and represented using the existing context/override mechanism.

## Ambiguity and decision records

When a term is disputed, record the decision in the PR discussion or an agreed locale decision log with:

1. Exact English key **and context**, app, source path, and where it appears.
2. Proposed Hebrew text and any relevant alternative.
3. Meaning and user action in that context; a screenshot from the **Crafting App** if possible.
4. Whether this changes a shared string (a universal **correction**) or only one app (**override**).
5. Native-speaker review status and the maintainer's final decision.

Avoid reopening settled wording debates without new source context or usability evidence. Do not invent a binding terminology authority outside the project's maintainer/reviewer process.

## Right-to-left behavior is a separate acceptance gate

A valid catalog does not prove the application renders a usable Hebrew interface.

- Store translations in **logical reading order**. Do not reverse Hebrew strings to work around a renderer.
- Validate mixed-direction examples with Hebrew plus Latin words, file paths, URLs, email addresses, version numbers, dates, percentages, keyboard shortcuts, and placeholders.
- Paths, extensions, code snippets, shortcut key names, and opaque identifiers must remain exact. Directional isolation belongs in the rendering/runtime layer where supported; never silently insert directional controls into source keys or machine-readable identifiers.
- Check correct mirrored placement of layout-specific controls without mirroring semantic icons or changing command IDs. Preserve a usable LTR experience when Hebrew is not selected.
- Verify Hebrew glyph coverage and fallback fonts on the supported platforms; a successful compile alone does not verify glyphs.
- Check text truncation, focus/keyboard navigation, screen-reader labels, and dialog ordering, especially in save/open dialogs.
- Do not mark a language as *GUI verified* merely because it is in `locales/he/` or offered by a language picker.

## Catalog integrity and plurals

The source of truth for syntax and CI remains [catalog-format.md](catalog-format.md).

- Use UTF-8 TSV with exactly three columns: `context`, exact English source, and Hebrew translation.
- Preserve `{name}` placeholders and the trailing ellipsis; do not translate key names, command IDs, or placeholder tokens.
- Use `@plural` forms in the locale's **CLDR order**, with the exact number of forms required by the currently pinned runtime for Hebrew. Do not infer the count from English or substitute hard-coded plural categories.
- Resolve source homographs with an appropriate context key in the app and catalog, rather than forcing one translation to cover different meanings.
- Keep unused or not-yet-reviewed rows untranslated so the normal English fallback remains reliable; do not fill them with guesses solely to raise coverage statistics.
- Run the repository's normal validation before review:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
python3 -m unittest discover -s tools
```

## Evidence required for an app-level Hebrew claim

Report separately:

| State | Evidence |
|---|---|
| Extracted | Source English keys and pinned app revision |
| Translated | Valid, active `he` rows with a catalog revision |
| Reviewed | Specific native-language review of meaning and UI context |
| In PR | Public PR URL and exact head commit |
| Merged | Upstream merge commit |
| Embedded | App's `craft-i18n.rev` (or app-local source revision) includes the change |
| GUI verified | Actual app build, Hebrew selected, screenshots and RTL/mixed-text checks |
| Released | Public release version containing the integrated locale |

For at least one representative screen, exercise **open → edit → save → reopen** in the actual application, where the workflow exists, and retain a screenshot of Hebrew with mixed LTR data. Keep these tests independent of unit tests and compilation reports.

## Governance and maintenance proposal

The project's [GLOSSARY.md](../GLOSSARY.md) defines a **Reviewer per locale**, not a global translation owner. If maintainers agree, Hebrew contributors can nominate a fluent Reviewer and maintain this guide as a lightweight, community-owned reference for future `he` PRs. Reviewer rights and final acceptance remain with the upstream project.

A useful ongoing workflow is: English string changes → Hebrew gap report → small translation PR → contextual review → catalog CI → app integration smoke → merge → release-pin update. Track regressions and coverage **per app** so a large untranslated catalog does not hide behind a global percentage.

## Feedback requested

Maintainers and Hebrew users: please review the proposed wording conventions, the existing shared examples, the clean-room boundary, and the separate RTL/GUI acceptance gate. In particular, should this guide be linked from the main translation instructions, and what would be an appropriate process for nominating a Hebrew locale Reviewer?
