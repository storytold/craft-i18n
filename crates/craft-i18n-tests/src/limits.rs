//! The bounds a catalog is read within. The apps skip what passes them (the runtime crate
//! mirrors these numbers), so the checker refuses it: the two cannot disagree on what a hostile
//! or runaway file may cost. `docs/catalog-format.md` lists them as "Limits".

/// A catalog file, in bytes. A larger file is not read at all.
pub const MAX_CATALOG_BYTES: usize = 2 * 1024 * 1024;
/// Rows in a file. The rows after the last one counted are ignored.
pub const MAX_ROWS: usize = 20_000;
/// One row, in bytes, without its line ending. A longer row is skipped.
pub const MAX_ROW_BYTES: usize = 16 * 1024;
/// One selector message, in bytes.
pub const MAX_MESSAGE_BYTES: usize = 4 * 1024;
/// Variables a `.match` chooses by.
pub const MAX_SELECTORS: usize = 3;
/// Variants of a `.match`.
pub const MAX_VARIANTS: usize = 64;
/// `.input` and `.local` declarations of one message.
pub const MAX_DECLARATIONS: usize = 8;
/// Parts of one pattern: runs of text and placeholders.
pub const MAX_PARTS: usize = 64;
/// `@noun` rows in a file.
pub const MAX_NOUNS: usize = 2_000;
/// Forms of one noun, and names in one `@grammar` row.
pub const MAX_FORMS: usize = 16;
/// Digits of an exact number in a variant's key.
pub const MAX_KEY_DIGITS: usize = 9;
