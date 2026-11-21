# Thai (th) catalogs for the Crafting Apps

This is a fork of [storytold/craft-i18n](https://github.com/storytold/craft-i18n) whose `th/` branch
adds Thai for **WordCraft** and the **ArtCraft Launcher**, offered upstream as a pull request.

- `main` — the upstream `main` this branch is based on (`b5b2805`), untouched.
- `th-wordcraft` — one commit, `th: WordCraft and ArtCraft Launcher catalogs`:
  - `locales/th/wordcraft.tsv` (new, 57 rows)
  - `locales/th/craft-launcher.tsv` (new, 87 rows)
  - `ATTRIBUTION.md` (the `th` row gains the two catalogs and their provenance)

The rows are partial on purpose: a row left out shows in English, and more follow in smaller
pull requests. Every row was written from the plain meaning of the English source text, using the
Thai PdfCraft and PhotoCraft catalogs in this repository as the vocabulary reference.

Checks run for the change (`cargo test -p craft-i18n-tests --test locales --test
wordcraft_feature_names`): `17 passed` and `3 passed`.

To send it upstream:

1. Open <https://github.com/storytold/craft-i18n/compare/main...rachatasawasdikit-collab:th-wordcraft>
2. Title: `th: WordCraft and ArtCraft Launcher catalogs`
3. Paste the description from the local notes, and ask for `th` to be added to WordCraft's
   `LANGUAGES` in `crates/ui-egui/src/i18n/mod.rs`.

Translations are contributed under the repository's licence (MIT OR Apache-2.0).
