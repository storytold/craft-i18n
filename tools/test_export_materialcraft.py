"""Regression tests for the MaterialCraft initial catalog migration."""
import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import export_materialcraft as material
import export_texturecraft as shared


class MaterialCraftExportTests(unittest.TestCase):
    """Keep app isolation, shared overrides and read-only parity checking explicit."""

    def migrate(self, root, rows, check=False):
        """Supply a fixed source snapshot while exercising the real TSV migration."""
        with patch.object(shared, 'original_tables', return_value={'ar': rows}) as tables:
            with patch.object(shared, 'output', return_value='a' * 40), contextlib.redirect_stdout(io.StringIO()):
                material.migrate(Path('checkout'), 'fixed', root, check)
        tables.assert_called_once_with(Path('checkout'), 'a' * 40, material.LANGUAGES)

    def fixture(self, root):
        """Create separate shared and TextureCraft rows that must not be overwritten."""
        locale = root / 'locales/ar'
        locale.mkdir(parents=True)
        (locale/'common.tsv').write_text('\tSave\tShared save\n\tOpen\tShared open\n', encoding='utf-8')
        (locale/'texturecraft.tsv').write_text('\tLayers\tTexture only\n', encoding='utf-8')
        return locale

    def test_export_preserves_shared_differences_without_changing_other_apps(self):
        """Keep a differing shared translation as an app override; omit identical rows."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locale = self.fixture(root)
            before = {p.name: p.read_bytes() for p in locale.iterdir()}
            rows = {('', 'Save'): 'Shared save', ('', 'Open'): 'Material open', ('', 'Nodes'): 'عقد'}
            self.migrate(root, rows)
            own = shared.read(locale/'materialcraft.tsv')
            self.assertNotIn(('', 'Save'), own)
            self.assertEqual(own[('', 'Open')], 'Material open')
            self.assertEqual(shared.resolved(root, 'ar', 'materialcraft')[('', 'Nodes')], 'عقد')
            for name, content in before.items():
                self.assertEqual((locale/name).read_bytes(), content)
            self.assertIn('MaterialCraft', (locale/'materialcraft.tsv').read_text(encoding='utf-8'))

    def test_check_mode_does_not_write_or_reformat_files(self):
        """A parity check preserves file bytes and modification times exactly."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locale = self.fixture(root)
            rows = {('', 'Open'): 'Material open'}
            self.migrate(root, rows)
            before = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in locale.iterdir()}
            self.migrate(root, rows, check=True)
            self.assertEqual(before, {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in locale.iterdir()})

    def test_check_rejects_changed_translation(self):
        """Detect a value changed after export rather than silently accepting it."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locale = self.fixture(root)
            (locale/'materialcraft.tsv').write_text('\tNodes\tChanged\n', encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'translations changed'):
                self.migrate(root, {('', 'Nodes'): 'عقد'}, check=True)

    def test_check_rejects_missing_translation_without_creating_catalog(self):
        """Missing MaterialCraft rows cannot be satisfied by another app's table."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            locale = self.fixture(root)
            with self.assertRaisesRegex(ValueError, 'translations changed'):
                self.migrate(root, {('', 'Layers'): 'Texture only'}, check=True)
            self.assertFalse((locale/'materialcraft.tsv').exists())

    def test_actual_rust_extraction_reads_only_the_configured_locale(self):
        """Use rustc to decode the single Arabic table and its escaped Rust literals."""
        source = r'''pub const TABLE: &[(&str, &str)] = &[("Line\nTab\tSlash\\", "عقد")];'''
        real_output = shared.output

        def output(*args, cwd=None):
            """Replace Git's source with a fixture but keep the real compiler execution."""
            return source if args[0] == 'git' else real_output(*args, cwd=cwd)

        with patch.object(shared, 'output', side_effect=output):
            tables = shared.original_tables(Path('.'), 'fixed', material.LANGUAGES)
        self.assertEqual(tables, {'ar': {('', r'Line\nTab\tSlash\\'): 'عقد'}})


if __name__ == '__main__':
    unittest.main()
