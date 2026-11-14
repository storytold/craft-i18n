//! Hungarian archive regressions: upstream bytes are data, never repair inputs.
use super::*;

/// A verified synthetic upstream entry, including the original replacement errors.
fn patterns(kind: Kind, bytes: &[u8]) -> Entry {
    let commit = "4f1df64b2e8e14751aa7c2205d170ca016071315";
    Entry {
        locale: "hu".into(),
        kind,
        url: format!(
            "https://raw.githubusercontent.com/LibreOffice/dictionaries/{commit}/hu_HU/hyph_hu_HU.dic"
        ),
        commit: commit.into(),
        sha256: sha256_hex(bytes),
        size: bytes.len() as u64,
        licence: "MPL-1.1".into(),
        class: Class::Embeddable,
        name: Some("Magyar".into()),
        note: None,
    }
}

/// Packing preserves broken records, line endings and source checksums at any pin.
#[test]
fn upstream_hungarian_records_are_exact_and_unmodified() {
    let source = b"UTF-8\r\ngic5csetek/cs=,3,1 ,1,9\r\nas5szon2y/sz=,2,1\r\n";
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../language-data/licences/MPL-1.1.txt");
    let texts = BTreeMap::from([("MPL-1.1".into(), std::fs::read(root).unwrap())]);
    for commit in [
        "4f1df64b2e8e14751aa7c2205d170ca016071315",
        "1111111111111111111111111111111111111111",
    ] {
        let mut nonstandard = patterns(Kind::HyphenationNonstandard, source);
        nonstandard.commit = commit.into();
        nonstandard.url = format!(
            "https://raw.githubusercontent.com/LibreOffice/dictionaries/{commit}/hu_HU/hyph_hu_HU.dic"
        );
        let standard_bytes = b"\\patterns{ba1na}\n";
        let standard = patterns(Kind::Hyphenation, standard_bytes);
        let manifest = Manifest {
            file: vec![nonstandard, standard],
        };
        let files: Vec<_> = manifest
            .file
            .iter()
            .zip([source.as_slice(), standard_bytes.as_slice()])
            .map(|(e, d)| (e, d.to_vec()))
            .collect();
        let bytes = archive(&manifest, "hu", &files, &texts, "test").unwrap();
        assert_eq!(
            bytes,
            archive(&manifest, "hu", &files, &texts, "test").unwrap()
        );
        let raw = zstd::decode_all(bytes.as_slice()).unwrap();
        let mut tar = tar::Archive::new(raw.as_slice());
        let mut unpacked = BTreeMap::new();
        for entry in tar.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path = entry.path().unwrap().to_string_lossy().into_owned();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            unpacked.insert(path, data);
        }
        assert_eq!(unpacked["hyphenation.nonstandard"], source);
        assert_eq!(unpacked["hyphenation.pat"], standard_bytes);
        assert!(unpacked.contains_key("LICENCES/MPL-1.1.txt"));
        assert!(!unpacked.contains_key("LICENCES/MODIFICATIONS-hu-hyphenation.txt"));
        let metadata: toml::Table =
            toml::from_str(std::str::from_utf8(&unpacked["pack.toml"]).unwrap()).unwrap();
        let record = metadata["source"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["file"].as_str() == Some("hyphenation.nonstandard"))
            .unwrap();
        assert_eq!(record["sha256"].as_str(), Some(sha256_hex(source).as_str()));
    }
}
