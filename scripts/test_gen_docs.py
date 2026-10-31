#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Python-runner contract of scripts/gen_docs.sh and scripts/gen_cli_docs.py.

Two regressions from the 0.14.0 release (#1810): a venv whose interpreter
exists but lacks ``jubarte_redlines`` must rebuild the package, and the
``python -m jubarte_redlines`` fallback must reach ``gen_cli_docs.py`` — its
``--runner`` is one shell command line, not an argv list argparse cannot
carry past a ``-m``.

No Rust build, uv or node required: the script's own lines are exercised
against stub executables.
"""

from __future__ import annotations

import os
import re
import shlex
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
GEN_DOCS = HERE / "gen_docs.sh"
GEN_CLI_DOCS = HERE / "gen_cli_docs.py"

STUB_INSTALLED = "#!/bin/sh\nexit 0\n"
STUB_NO_PACKAGE = '#!/bin/sh\n[ "$1" = "-c" ] && exit 1\nexit 0\n'


def script_lines() -> list[str]:
    return GEN_DOCS.read_text(encoding="utf-8").splitlines()


def run_bash(snippet: str, env: dict[str, str]) -> None:
    result = subprocess.run(
        ["bash", "-euo", "pipefail", "-c", snippet],
        env=env,
        capture_output=True,
        text=True,
        timeout=10,
    )
    if result.returncode != 0:
        raise AssertionError(f"bash snippet failed ({result.returncode}): {result.stderr}")


class PackageGuard(unittest.TestCase):
    """The build condition must test the package, not only the interpreter."""

    @staticmethod
    def guard() -> str:
        guards = [l for l in script_lines() if re.match(r'^if \[ ! -x "\$PY" \]', l)]
        if len(guards) != 1:
            raise AssertionError(f"expected one PY build guard, found {guards}")
        return guards[0]

    def builds(self, tmp: Path, py: Path | None) -> bool:
        marker = tmp / "built"
        env = dict(os.environ, PY=str(py or tmp / "missing-python"))
        run_bash(f"if {self.guard()[3:]} touch {shlex.quote(str(marker))}; fi", env)
        return marker.exists()

    def test_missing_interpreter_still_builds(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            self.assertTrue(self.builds(Path(d), None))

    def test_interpreter_without_the_package_builds(self) -> None:
        # A venv left by a failed `maturin develop`: python runs, the import
        # fails, so the package must be rebuilt instead of documented empty.
        with tempfile.TemporaryDirectory() as d:
            py = Path(d) / "python"
            py.write_text(STUB_NO_PACKAGE)
            py.chmod(0o755)
            self.assertTrue(self.builds(Path(d), py))

    def test_interpreter_with_the_package_skips_the_build(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            py = Path(d) / "python"
            py.write_text(STUB_INSTALLED)
            py.chmod(0o755)
            self.assertFalse(self.builds(Path(d), py))


class RunnerArgument(unittest.TestCase):
    """gen_docs.sh passes --runner as ONE shell command line to gen_cli_docs.py."""

    @staticmethod
    def python_block() -> str:
        lines = script_lines()
        start = next(
            i for i, l in enumerate(lines)
            if l.startswith('if [ -x "$ROOT/jubarte-python/.venv/bin/jubarte-redlines" ]')
        )
        end = next(i for i, l in enumerate(lines) if "--marker cli-python" in l)
        return "\n".join(lines[start : end + 1])

    def runner_args(self, console_script: bool) -> list[str]:
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            log = tmp / "argv.log"
            bindir = tmp / "bin"
            bindir.mkdir()
            python3 = bindir / "python3"
            python3.write_text('#!/bin/sh\nfor a in "$@"; do echo "$a"; done >> "$LOG"\n')
            python3.chmod(0o755)
            root = tmp / "root"
            venv_bin = root / "jubarte-python" / ".venv" / "bin"
            venv_bin.mkdir(parents=True)
            py = venv_bin / "python"
            py.write_text(STUB_INSTALLED)
            py.chmod(0o755)
            if console_script:
                (venv_bin / "jubarte-redlines").write_text(STUB_INSTALLED)
                (venv_bin / "jubarte-redlines").chmod(0o755)
            env = dict(
                os.environ,
                ROOT=str(root),
                PY=str(py),
                LOG=str(log),
                PATH=f"{bindir}:{os.environ['PATH']}",
            )
            run_bash(self.python_block(), env)
            argv = log.read_text().splitlines()
            i = argv.index("--runner")
            j = argv.index("--display")
            return argv[i + 1 : j]

    def test_fallback_module_runner_is_one_shell_command(self) -> None:
        # With no console script, `python -m jubarte_redlines` must arrive as
        # one --runner value: argparse nargs="+" ends the option at `-m`.
        args = self.runner_args(console_script=False)
        self.assertEqual(len(args), 1, f"--runner got {args!r} (argparse drops -m)")
        words = shlex.split(args[0])
        self.assertEqual(words[1:], ["-m", "jubarte_redlines"])

    def test_console_script_runner_is_one_shell_command(self) -> None:
        args = self.runner_args(console_script=True)
        self.assertEqual(len(args), 1, f"--runner got {args!r}")
        self.assertTrue(shlex.split(args[0])[0].endswith("jubarte-redlines"))


class RunnerEndToEnd(unittest.TestCase):
    """gen_cli_docs.py runs a --runner whose argv carries `-m` (#1810)."""

    def test_dash_m_runner_reaches_the_cli(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            (tmp / "jubarte_redlines.py").write_text(
                "print('jubarte-redlines 0.15.0')\nprint('Read documents')\n"
            )
            page = tmp / "python.md"
            page.write_text(
                "<!-- gen:cli-python:start -->\nold\n<!-- gen:cli-python:end -->\n"
            )
            result = subprocess.run(
                [
                    sys.executable,
                    str(GEN_CLI_DOCS),
                    "--runner",
                    f"{shlex.quote(sys.executable)} -m jubarte_redlines",
                    "--display", "jubarte-redlines",
                    "--file", str(page),
                    "--marker", "cli-python",
                ],
                env=dict(os.environ, PYTHONPATH=str(tmp)),
                capture_output=True,
                text=True,
                timeout=20,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("jubarte-redlines 0.15.0", page.read_text())


if __name__ == "__main__":
    unittest.main()
