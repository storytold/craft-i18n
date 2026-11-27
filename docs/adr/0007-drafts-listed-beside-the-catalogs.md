# Drafts are listed beside the catalogs, not marked in them

A Crafting App that has no catalogs yet may start from AI-assisted first catalogs, written from the
plain meaning of the English (the clean-room rule) and recorded in `ATTRIBUTION.md`. Their rows are
Drafts: the app shows them, but no Reviewer has accepted them. We list each catalog's Drafts in a
separate file that no app reads: `drafts/<locale>/<app>.tsv`, outside `locales/`, one row per Draft
with the catalog row's source text and context, the two columns that identify it. The Weblate converter marks those rows "Needs
editing", and a Reviewer's acceptance takes a row off the list. The TSV format, the craft-libs
parser and every app's runtime pin stay as they are.

## Considered options

- **No marker, only `ATTRIBUTION.md`**: rejected. Reviewers could not find which rows still need
  review.
- **A Draft flag on each TSV row**: rejected. It changes the catalog format (ADR 0001), the
  craft-libs parser and every app's runtime pin, for data the runtime never needs.
- **Drafts only as Weblate Suggestions, not in the catalogs**: rejected. The app would show English
  until a Reviewer exists for the locale, and many locales have none yet.

## Consequences

- The converter and the checker are to read the Draft lists (a later change; neither does yet);
  the checker will fail a list entry whose row is missing from its catalog.
- AI rows already in catalogs before this decision are not listed as Drafts.
