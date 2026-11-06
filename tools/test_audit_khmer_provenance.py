"""CLI failures in the offline provenance audit are concise, without tracebacks."""

from contextlib import redirect_stderr
import io
import os
import unittest
from unittest import mock

import audit_khmer_provenance as audit


class AuditErrors(unittest.TestCase):
    def run_cli(self):
        argv = ["audit_khmer_provenance.py", "--sil", ".", "--aksara", ".",
                "--cldr-xml", "unused.xml", "--cldr-licence", "unused.txt",
                "--sealang-licence", "unused.txt"]
        error = io.StringIO()
        with mock.patch("sys.argv", argv), redirect_stderr(error):
            result = audit.main()
        self.assertEqual(result, 1)
        self.assertTrue(error.getvalue().startswith("Khmer provenance audit: "))
        self.assertNotIn("Traceback", error.getvalue())
        return error.getvalue()

    def test_missing_spec_key(self):
        with mock.patch.object(audit.json, "loads", return_value={"inputs": {}}):
            self.assertIn("sil", self.run_cli())

    def test_missing_git(self):
        # Exercise the actual subprocess failure, rather than raising it from a mock.
        with mock.patch.dict(os.environ, {"PATH": ""}):
            self.assertIn("git", self.run_cli())


if __name__ == "__main__":
    unittest.main()
