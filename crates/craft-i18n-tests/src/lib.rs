//! Test-only crate: checks the TSV catalogs in `locales/` ([`tree::check_tree`]), so a malformed
//! row, a placeholder that differs from the English source text, a wrong plural form count, a
//! selector message that leaves a plural category without a variant or a noun with a gender its
//! language does not declare fails CI here, not in an app. The format is in
//! `docs/catalog-format.md`. [`packs`] checks the Language pack manifest and builds the release
//! assets (`docs/language-data-format.md`).
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod catalog;
pub mod grammar;
pub mod limits;
pub mod locale;
pub mod message;
pub mod packs;
pub mod placeholder;
pub mod plural;
pub mod select;
pub mod spec;
pub mod tree;

/// The repository root (where `locales/` lives).
#[must_use]
pub fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
