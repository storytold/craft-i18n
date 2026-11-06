#!/usr/bin/env python3
"""Uncomment lines that have a translation added after the TAB.

Usage:
    python3 uncomment_translations.py tw
    python3 uncomment_translations.py tw common.tsv wordcraft.tsv pdfcraft.tsv
"""

import sys
from pathlib import Path


def uncomment_file(path):
    path = Path(path)
    lines = path.read_text(encoding='utf-8').splitlines(keepends=True)
    changed = 0
    new_lines = []
    for line in lines:
        # Skip header lines and blank lines
        stripped = line.lstrip()
        if not stripped.startswith('#'):
            new_lines.append(line)
            continue

        # Check if line is a commented catalog row with a translation
        # Pattern: #<context>\t<source>\t<translation>
        # We want to uncomment if there's a third column (translation) present
        # Remove # and check for tabs
        content_after_hash = line[1:] if line.startswith('#') else line
        # Count tabs in the content after #
        tabs = content_after_hash.count('\t')
        if tabs >= 2:
            # Has context, source, and translation
            # Uncomment it - remove the # but keep everything else
            # Preserve original indentation
            indent = line[:len(line) - len(line.lstrip())]
            new_line = indent + content_after_hash.lstrip()
            new_lines.append(new_line)
            changed += 1
        else:
            # Keep as comment
            new_lines.append(line)

    if changed > 0:
        path.write_text(''.join(new_lines), encoding='utf-8')
        print(f"{path}: uncommented {changed} lines")
    else:
        print(f"{path}: no changes")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python3 uncomment_translations.py <locale> [files...]")
        sys.exit(1)

    locale = sys.argv[1]
    locale_dir = Path('locales') / locale

    if not locale_dir.exists():
        print(f"Locale directory {locale_dir} not found")
        sys.exit(1)

    # If specific files are given, use them
    if len(sys.argv) > 2:
        files = sys.argv[2:]
    else:
        # Otherwise, process all .tsv files in the locale directory
        files = [f.name for f in locale_dir.glob('*.tsv')]

    for f in files:
        path = locale_dir / f
        if path.exists():
            uncomment_file(path)
        else:
            print(f"{path}: not found")
