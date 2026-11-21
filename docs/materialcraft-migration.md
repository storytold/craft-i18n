# MaterialCraft initial catalog migration

Source: `materialcraft@b73facdc7b8b5ce102cd9e14d964058a15182dc0`.
This pre-cutover main revision offers English (source text) and Arabic. The Hebrew
catalog/font work in MaterialCraft #197/#198 was not merged at export time.

From the root of this repository, with Git, Python 3 and rustc available:

```sh
python tools/export_materialcraft.py ../materialcraft --ref b73facdc7b8b5ce102cd9e14d964058a15182dc0
python tools/export_materialcraft.py ../materialcraft --ref b73facdc7b8b5ce102cd9e14d964058a15182dc0 --check
python -m unittest discover -s tools
cargo test --workspace
```

The export compiles the original Rust table so escaped and raw strings are decoded
by Rust itself. Existing shared rows are reused only when the translation matches
exactly; differing wording stays in materialcraft.tsv. `--check` writes nothing and
compares every original translation with the resolved app/shared catalogs.
English needs no catalog, and no missing translations are invented.

This is the catalog prerequisite for storytold/materialcraft#210. MaterialCraft's
runtime cutover must pin this PR's actual upstream merge commit after acceptance,
record the translation source in its own ATTRIBUTION.md, and rerun parity against
current main. Include any locale work merged meanwhile before removing Rust tables.
The catalog PR does not close the application issue or change MaterialCraft code.
