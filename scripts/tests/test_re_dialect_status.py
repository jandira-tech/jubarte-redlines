#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Integration checks: matching output never excuses a failed command."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ExitStatusTests(unittest.TestCase):
    def run_script(self, script):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "jubarte"
            binary.write_text("""#!/usr/bin/env python3
import os
from pathlib import Path
import sys
args = sys.argv[1:]
source = next((Path(a) for a in args if a.endswith(('.docx', '.md'))), None)
if 'edit' in args:
    if '--reject' in args:
        name = 'reject-substitution' if 'r4+5' in args else 'reject-nested-deletion'
    elif '--accept' in args:
        name = 'accept-substitution'
    else:
        anchor = args[args.index('--anchor') + 1]
        name = {'their': 'modify-inside-insertion', 'month.': 'add-inside-insertion'}.get(anchor)
        if name is None:
            name = 'modify-substitution' if source.stem == 'step2-tracked' else 'modify-plain-text'
    Path(args[args.index('-o') + 1]).write_text('edits/' + name + '.md')
    sys.exit(0)
if source.suffix == '.md':
    Path(args[args.index('-o') + 1]).touch()
    sys.exit(0)
expected = source.read_text() if source.name == 'out.docx' else source.stem + '.md'
text = (Path(os.environ['DIALECT_FIXTURES']) / expected).read_text()
if '-o' in args:
    Path(args[args.index('-o') + 1]).write_text(text)
else:
    print(text, end='')
sys.exit(int(os.environ['READ_EXIT']))
""")
            binary.chmod(0o755)
            return subprocess.run(
                ["bash", str(ROOT / "scripts" / script)],
                env={**os.environ, "JUBARTE": str(binary), "READ_EXIT": "7",
                     "DIALECT_FIXTURES": str(ROOT / "tests/fixtures/re-dialect")},
                capture_output=True, text=True, check=False,
            )

    def test_reads_require_success_as_well_as_matching_output(self):
        result = self.run_script("re-dialect-read.sh")
        self.assertEqual(result.returncode, 1, result.stdout)
        self.assertIn("read: 0 ok, 15 failed", result.stdout)

    def test_round_trip_reads_require_success_as_well_as_matching_output(self):
        result = self.run_script("re-dialect-edit.sh")
        self.assertIn("edit: 0 ok, 12 failed", result.stdout)
        self.assertEqual(result.returncode, 1)


if __name__ == "__main__":
    unittest.main()
