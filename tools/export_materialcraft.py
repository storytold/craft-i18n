#!/usr/bin/env python3
"""Migrate MaterialCraft's Rust translation tables (storytold/materialcraft#210).

python tools/export_materialcraft.py ../materialcraft --ref <source SHA>
python tools/export_materialcraft.py ../materialcraft --ref <source SHA> --check

Requires Git and rustc. Use a pre-cutover source commit and record the full SHA in
ATTRIBUTION.md. This initial export preserves existing translations; it does not
create translations or import unmerged locale work.
"""
import argparse
from pathlib import Path

from export_texturecraft import migrate as migrate_tables

ROOT = Path(__file__).resolve().parent.parent
LANGUAGES = {'ar': 'العربية'}


def migrate(checkout, revision, root, check=False):
    """Export Arabic to materialcraft.tsv or verify resolved translations without writes."""
    migrate_tables(checkout, revision, root, check, languages=LANGUAGES, app='materialcraft', name='MaterialCraft')


def main():
    """Read the source checkout, fixed revision and optional read-only check mode."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checkout', type=Path)
    parser.add_argument('--ref', required=True)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    migrate(args.checkout.resolve(), args.ref, ROOT, args.check)


if __name__ == '__main__':
    main()
