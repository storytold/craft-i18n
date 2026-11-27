#!/usr/bin/env python3
"""Reproduce the source audit offline; never fetch or emit dictionary words.

See docs/khmer-source-audit.json for immutable inputs and docs/khmer-provenance.md
for interpretation. A match establishes content relationships, not copyright ownership.
"""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET


def verified(row, raw):
    if len(raw) != row["size"] or hashlib.sha256(raw).hexdigest() != row["sha256"]:
        raise ValueError("input does not match its pinned bytes: " + row["url"])
    return raw


def git_file(checkout, row):
    return verified(row, subprocess.check_output(
        ["git", "-C", str(checkout), "show", row["commit"] + ":" + row["path"]]
    ))


def january_counts(raw):
    """January convert-dictionary.js semantics: equal raw counts, no SIL transforms."""
    result = Counter()
    for line in raw.decode("utf-8").split("\n"):
        text = line.strip().strip("\ufeff").strip()
        if not text or text.startswith("#"):
            continue
        parts = text.split("\t")
        word = parts[0]
        if not re.search("[\u1780-\u17ff]", word):
            continue
        match = re.match(r"[+-]?[0-9]+", parts[1].lstrip()) if len(parts) > 1 else None
        count = (int(match.group()) if match else 1) or 1
        result[word] += count
    return result


def audit(args):
    spec = json.loads((Path(__file__).resolve().parents[1]
                       / "docs/khmer-source-audit.json").read_text(encoding="utf-8"))
    rows = spec["inputs"]
    sil = {name: git_file(args.sil, row) for name, row in rows["sil"].items()}
    original = git_file(args.sil, rows["original_places"])
    frequency = json.loads(git_file(args.aksara, rows["aksara_frequency"]))
    january = json.loads(git_file(args.aksara, rows["aksara_january_dictionary"]))
    git_file(args.aksara, rows["aksara_january_converter"])
    cldr = verified(rows["cldr28_xml"], args.cldr_xml.read_bytes())
    verified(rows["cldr28_licence"], args.cldr_licence.read_bytes())
    verified(rows["sealang_licence"], args.sealang_licence.read_bytes())
    verified(rows["spdx_unicode_licence"], (Path(__file__).resolve().parents[1]
             / "language-data/licences/Unicode-DFS-2015.txt").read_bytes())

    initial_lines = original.decode("utf-8-sig").splitlines()
    root = ET.fromstring(cldr)
    # The empty identity/language element accounts for the original file's blank line.
    names = [element.text or "" for tag in ("language", "script", "territory", "exemplarCity")
             for element in root.iter(tag)]
    combined = Counter()
    overlaps = {}
    union = set()
    for path in sorted(name for name in sil if name.startswith("src/")):
        raw = sil[path]
        # Literal overlap only: remove BOM/ZWSP and delimit the count, without coeng
        # reordering, spelling expansion or SIL's validity filter.
        words = {
            re.split("[,\t]", line.strip().replace("\u200b", "").replace("\ufeff", ""), maxsplit=1)[0].strip()
            for line in raw.decode("utf-8-sig").splitlines()
        } - {""}
        source = Path(path).stem
        overlaps[source] = {
            "literal_sil_words": len(words),
            "aksara_literal_word_overlap": len(words & frequency.keys()),
        }
        union.update(words)
        combined.update(january_counts(raw))
    reconstructed = dict(sorted(
        ((word, count) for word, count in combined.items() if count >= 2),
        key=lambda item: -item[1])[:10000])
    expected_january = {row["word"]: row["frequency"] for row in january}
    return {
        "cldr28_initial_places": {
            "sil_lines": len(initial_lines),
            "nonempty_names": sum(bool(line) for line in initial_lines),
            "unique_nonempty_names": len(set(initial_lines) - {""}),
            "cldr_extracted_lines": len(names),
            "exact_multiset_including_duplicates": Counter(initial_lines) == Counter(names),
        },
        "aksara_entries": len(frequency),
        "sil_source_overlaps": overlaps,
        "literal_sil_union": len(union),
        "aksara_literal_sil_overlap": len(union & frequency.keys()),
        "january_reproduction": {
            "committed_entries": len(expected_january),
            "reconstructed_entries": len(reconstructed),
            "exact_word_frequency_pairs": reconstructed == expected_january,
            "maximum_frequency": max(reconstructed.values()),
            "cutoff_frequency": min(reconstructed.values()),
        },
    }, spec["findings"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sil", type=Path, required=True, help="full khmerlbdict checkout")
    parser.add_argument("--aksara", type=Path, required=True, help="Aksara checkout")
    parser.add_argument("--cldr-xml", type=Path, required=True)
    parser.add_argument("--cldr-licence", type=Path, required=True)
    parser.add_argument("--sealang-licence", type=Path, required=True)
    args = parser.parse_args()
    try:
        findings, expected = audit(args)
        if findings != expected:
            raise ValueError("computed findings differ from the reviewed evidence record")
        print(json.dumps(findings, ensure_ascii=False, indent=2))
        return 0
    except (KeyError, ValueError, OSError, subprocess.CalledProcessError, ET.ParseError) as error:
        print("Khmer provenance audit: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
