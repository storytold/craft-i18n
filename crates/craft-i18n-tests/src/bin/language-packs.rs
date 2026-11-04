//! `language-packs`: check, fetch and build the Language packs (`docs/language-data-format.md`).
//!
//! ```text
//! language-packs check
//! language-packs fetch [--cache DIR]
//! language-packs stage --locale LOCALE --out DIR [--cache DIR] [--no-download]
//! language-packs build --out DIR --release NAME --version SHA [--base-url URL]
//!                      [--only nl,fr,pl] [--cache DIR] [--no-download]
//! ```
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use craft_i18n_tests::packs;

const USAGE: &str = "usage:
  language-packs check
  language-packs fetch [--cache DIR]
  language-packs stage --locale LOCALE --out DIR [--cache DIR] [--no-download]
  language-packs build --out DIR --release NAME --version SHA [--base-url URL]
                       [--only nl,fr,pl] [--cache DIR] [--no-download]

--base-url defaults to https://github.com/storytold/craft-i18n/releases/download/<release>/;
--cache to $CRAFT_I18N_CACHE, else target/language-data-cache.";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("language-packs: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `--name value` pairs and bare flags after the subcommand.
struct Args(Vec<String>);

impl Args {
    fn value(&self, name: &str) -> Result<Option<String>, String> {
        match self.0.iter().position(|a| a == name) {
            None => Ok(None),
            Some(i) => self
                .0
                .get(i + 1)
                .filter(|v| !v.starts_with("--"))
                .cloned()
                .map(Some)
                .ok_or_else(|| format!("{name} needs a value")),
        }
    }

    fn required(&self, name: &str) -> Result<String, String> {
        self.value(name)?
            .ok_or_else(|| format!("{name} is required\n{USAGE}"))
    }

    fn flag(&self, name: &str) -> bool {
        self.0.iter().any(|a| a == name)
    }
}

fn run(argv: Vec<String>) -> Result<(), String> {
    let Some((cmd, rest)) = argv.split_first() else {
        return Err(USAGE.into());
    };
    let args = Args(rest.to_vec());
    let repo = packs::load_repo()?;
    let (manifest, checks, texts) = (&repo.manifest, &repo.checks, &repo.texts);
    let cache = args
        .value("--cache")?
        .map_or_else(packs::default_cache, PathBuf::from);
    match cmd.as_str() {
        "check" => {
            let errors = packs::check(manifest, checks, texts);
            if errors.is_empty() {
                println!(
                    "{} entries, {} locales: ok",
                    manifest.file.len(),
                    manifest.locales().len()
                );
                Ok(())
            } else {
                Err(errors.join("\n"))
            }
        }
        "fetch" => {
            let mut failed = Vec::new();
            for e in &manifest.file {
                match packs::fetch(e, &cache, true) {
                    Ok(_) => println!("ok {} {:?} {}", e.locale, e.kind, e.size),
                    Err(err) => failed.push(err),
                }
            }
            if failed.is_empty() {
                Ok(())
            } else {
                Err(failed.join("\n"))
            }
        }
        "stage" => {
            let locale = args.required("--locale")?;
            let out = PathBuf::from(args.required("--out")?);
            if !manifest.locales().contains(locale.as_str())
                || manifest.class(&locale) != packs::Class::Embeddable
            {
                return Err("stage requires a known Embeddable locale (ADR 0004); Download-only data cannot feed an embedding build".into());
            }
            let errors = packs::check(manifest, checks, texts);
            if !errors.is_empty() {
                return Err(errors.join("\n"));
            }
            // Verify every source before writing a staged directory, including hidden packs.
            let mut files = manifest
                .entries(&locale)
                .map(|e| {
                    packs::fetch(e, &cache, !args.flag("--no-download"))
                        .map(|bytes| (e.pack_path(), bytes))
                })
                .collect::<Result<Vec<_>, _>>()?;
            for id in packs::data_licence_ids(manifest, &locale) {
                if let Some(bytes) = texts.get(&id) {
                    files.push((format!("LICENCES/{id}.txt"), bytes.clone()));
                }
            }
            if let Some(notes) = packs::licence_notes(manifest, &locale) {
                files.push(("LICENCES/NOTICES.txt".into(), notes.into_bytes()));
            }
            write_staged_files(&out, files)
        }

        "build" => {
            let release = args.required("--release")?;
            let mut base_url = args.value("--base-url")?.unwrap_or_else(|| {
                format!("https://github.com/storytold/craft-i18n/releases/download/{release}/")
            });
            if !base_url.ends_with('/') {
                base_url.push('/');
            }
            let opts = packs::BuildOptions {
                cache,
                out: PathBuf::from(args.required("--out")?),
                release,
                version: args.required("--version")?,
                base_url,
                only: args
                    .value("--only")?
                    .map(|s| s.split(',').map(str::to_owned).collect())
                    .unwrap_or_default(),
                download: !args.flag("--no-download"),
            };
            let errors = packs::check(manifest, checks, texts);
            if !errors.is_empty() {
                return Err(errors.join("\n"));
            }
            let index = packs::build(manifest, checks, texts, &opts)?;
            for p in &index.packs {
                println!("{} {} bytes {}", p.locale, p.size, p.class);
            }
            println!(
                "{} packs and index.json in {}",
                index.packs.len(),
                opts.out.display()
            );
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

/// Require a fresh staging root: existing files and directory symlinks must never
/// redirect or overwrite data. The names have passed manifest validation above.
fn write_staged_files(out: &Path, files: Vec<(String, Vec<u8>)>) -> Result<(), String> {
    std::fs::create_dir(out)
        .map_err(|e| format!("stage needs a new output directory {}: {e}", out.display()))?;
    let result = (|| {
        for (name, bytes) in files {
            write_staged_file(out, &name, &bytes)?;
        }
        Ok(())
    })();
    result.map_err(|e: String| {
        format!(
            "staging failed: {e}; remove the incomplete output directory {} before retrying",
            out.display()
        )
    })
}

/// Names come from the checked manifest. Refuse existing file symlinks and a licence-directory
/// symlink, including when this is called after the fresh root has been created.
fn write_staged_file(out: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = out.join(name);
    if let Some(parent) = path.parent().filter(|parent| *parent != out) {
        match std::fs::create_dir(parent) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                let meta = std::fs::symlink_metadata(parent).map_err(|e| e.to_string())?;
                if !meta.is_dir() {
                    return Err(format!("{} is not a plain directory", parent.display()));
                }
            }
            Err(err) => return Err(err.to_string()),
        }
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{run, write_staged_file, write_staged_files};
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{error::Error, fs};

    fn temp_root(label: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        Ok(std::env::temp_dir().join(format!(
            "craft-stage-{label}-{}-{unique}",
            std::process::id()
        )))
    }

    #[test]
    fn staging_requires_a_new_directory_and_preserves_existing_files() -> Result<(), Box<dyn Error>>
    {
        let root = temp_root("existing")?;
        fs::create_dir(&root)?;
        fs::write(root.join("keep.txt"), b"unchanged")?;
        assert!(
            write_staged_files(&root, vec![("keep.txt".into(), b"overwrite".to_vec())]).is_err()
        );
        assert_eq!(fs::read(root.join("keep.txt"))?, b"unchanged");
        let out = root.join("fresh");
        write_staged_files(&out, vec![("source.txt".into(), "ខ្មែរ".as_bytes().to_vec())])?;
        assert_eq!(fs::read_to_string(out.join("source.txt"))?, "ខ្មែរ");
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn staging_failure_explains_how_to_retry() -> Result<(), Box<dyn Error>> {
        let root = temp_root("partial")?;
        let result = write_staged_files(
            &root,
            vec![
                ("same".into(), b"first".to_vec()),
                ("same".into(), b"second".to_vec()),
            ],
        );
        let error = result
            .err()
            .ok_or("duplicate staging output unexpectedly succeeded")?;
        assert!(error.contains("remove the incomplete output directory"));
        assert_eq!(fs::read(root.join("same"))?, b"first");
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn staging_rejects_every_non_embeddable_locale_before_writing() -> Result<(), Box<dyn Error>> {
        let repo = craft_i18n_tests::packs::load_repo()?;
        let root = temp_root("restricted")?;
        for locale in repo.manifest.locales() {
            if repo.manifest.class(locale) == craft_i18n_tests::packs::Class::Embeddable {
                continue;
            }
            let error = run(vec![
                "stage".into(),
                "--locale".into(),
                locale.into(),
                "--out".into(),
                root.to_string_lossy().into_owned(),
            ])
            .err()
            .ok_or("restricted pack was staged")?;
            assert!(error.contains("Embeddable locale"), "{locale}: {error}");
            assert!(!root.exists());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn staging_rejects_existing_file_and_licence_symlinks() -> Result<(), Box<dyn Error>> {
        let root = temp_root("symlink")?;
        fs::create_dir(&root)?;
        let target = root.join("outside");
        fs::create_dir(&target)?;
        fs::write(target.join("keep.txt"), b"unchanged")?;
        for (index, name) in ["km_frequency_dictionary.json", "LICENCES"]
            .iter()
            .enumerate()
        {
            let out = root.join(index.to_string());
            fs::create_dir(&out)?;
            symlink(
                if index == 0 {
                    target.join("keep.txt")
                } else {
                    target.clone()
                },
                out.join(name),
            )?;
            // Exercise the writer directly, after root creation, rather than failing at
            // write_staged_files' initial create_dir guard.
            let path = if index == 0 {
                (*name).to_owned()
            } else {
                format!("{name}/new.txt")
            };
            let result = write_staged_file(&out, &path, b"replacement");
            assert!(result.is_err());
            assert_eq!(fs::read(target.join("keep.txt"))?, b"unchanged");
            assert!(!target.join("new.txt").exists());
        }
        let out = root.join("fresh");
        write_staged_files(&out, vec![("LICENCES/LICENSE".into(), b"notice".to_vec())])?;
        assert_eq!(fs::read(out.join("LICENCES/LICENSE"))?, b"notice");
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
