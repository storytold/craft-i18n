//! What the checker reports without failing: a noun that lacks a form a message asks for.

use std::path::Path;

use craft_i18n_tests::tree::{check_tree, check_tree_report};

const H: &str = "# Clean room: written from the plain meaning.\n";

fn tree(name: &str, common: &str, app: &str) -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("warnings-{name}"));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("locales/de");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("common.tsv"),
        format!("{H}@id\tlanguage.name\tDeutsch\n{common}"),
    )
    .unwrap();
    std::fs::write(dir.join("app.tsv"), format!("{H}{app}")).unwrap();
    root
}

const COMMON: &str = "@grammar\tforms\taccusative\n\
@grammar\tgenders\tmasculine feminine\n\
@noun\tpage\tfeminine: Seite\n\
@noun\tpicture\tmasculine: Bild; accusative=Bild\n";

const MESSAGE: &str = "\tDelete this {item :noun}?\t.input {$item :noun} .match $item feminine {{Diese {$item :noun form=accusative}}} * {{Diesen {$item :noun form=accusative}}}\n";

#[test]
fn a_noun_without_the_form_is_a_warning() {
    let root = tree("missing-form", COMMON, MESSAGE);
    let report = check_tree_report(&root);
    assert_eq!(report.errors, Vec::<String>::new());
    // One warning for the message, naming the noun that lacks the form once.
    assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
    let warning = &report.warnings[0];
    assert!(
        warning.starts_with("locales/de/app.tsv: line 2:"),
        "{warning}"
    );
    assert!(warning.contains("`accusative`"), "{warning}");
    assert!(warning.contains("`page`"), "{warning}");
    assert!(!warning.contains("`picture`"), "{warning}");
    // `check_tree` is the errors alone.
    assert_eq!(check_tree(&root), Vec::<String>::new());
}

#[test]
fn a_noun_with_every_form_is_no_warning() {
    let common = COMMON.replace("feminine: Seite", "feminine: Seite; accusative=Seite");
    let report = check_tree_report(&tree("all-forms", &common, MESSAGE));
    assert_eq!(report.errors, Vec::<String>::new());
    assert_eq!(report.warnings, Vec::<String>::new());
}

#[test]
fn the_nouns_named_are_capped() {
    let nouns: String = (0..20)
        .map(|k| format!("@noun\tthing {k}\tfeminine: Ding\n"))
        .collect();
    let common =
        format!("@grammar\tforms\taccusative\n@grammar\tgenders\tmasculine feminine\n{nouns}");
    let report = check_tree_report(&tree("capped", &common, MESSAGE));
    assert_eq!(report.errors, Vec::<String>::new());
    assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
    assert!(
        report.warnings[0].contains("and 15 more"),
        "{:?}",
        report.warnings
    );
}

#[test]
fn the_warning_report_prints() {
    // `cargo test warning_report -- --nocapture` over the repository's own catalogs; never fails.
    for warning in check_tree_report(&craft_i18n_tests::repo_root()).warnings {
        println!("{warning}");
    }
}
