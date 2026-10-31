#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Python-runner contract of scripts/gen_docs.sh and scripts/gen_cli_docs.py.

Two regressions from the 0.14.0 release (#1810): a venv whose interpreter
exists but lacks ``jubarte_redlines`` must rebuild the package, and the
``python -m jubarte_redlines`` fallback must reach ``gen_cli_docs.py`` — its
``--runner`` repeats once per argv word (``--runner=WORD``), an argv list
argparse's ``nargs="+"`` cannot carry past a ``-m``. Each word reaches the
subprocess verbatim: no shlex split, no printf %q round-trip.

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

    def builds(self, tmp: Path, py: Path | None, pythonpath: str | None = None) -> bool:
        marker = tmp / "built"
        env = dict(os.environ, PY=str(py or tmp / "missing-python"))
        if pythonpath is not None:
            env["PYTHONPATH"] = pythonpath
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

    def test_a_package_that_raises_on_import_does_not_rebuild(self) -> None:
        # The guard probes presence, not importability: a package that exists
        # but raises in __init__ is installed, so the build must be skipped
        # and the runner left to surface the error (find_spec does not
        # execute the package it finds).
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            site = tmp / "site"
            pkg = site / "jubarte_redlines"
            pkg.mkdir(parents=True)
            (pkg / "__init__.py").write_text("raise RuntimeError('broken install')\n")
            self.assertFalse(
                self.builds(tmp, Path(sys.executable), pythonpath=str(site))
            )


class RunnerArgument(unittest.TestCase):
    """gen_docs.sh passes --runner to gen_cli_docs.py once per argv word."""

    @staticmethod
    def python_block() -> str:
        lines = script_lines()
        start = next(
            i for i, l in enumerate(lines)
            if l.startswith('if [ -x "$ROOT/jubarte-python/.venv/bin/jubarte-redlines" ]')
        )
        end = next(i for i, l in enumerate(lines) if "--marker cli-python" in l)
        return "\n".join(lines[start : end + 1])

    def runner_args(self, root: Path, console_script: bool, tmp: Path) -> list[str]:
        log = tmp / "argv.log"
        bindir = tmp / "bin"
        bindir.mkdir()
        python3 = bindir / "python3"
        python3.write_text('#!/bin/sh\nfor a in "$@"; do echo "$a"; done >> "$LOG"\n')
        python3.chmod(0o755)
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
        return argv[1 : argv.index("--display")]

    def test_fallback_module_runner_is_one_word_per_option(self) -> None:
        # With no console script, `python -m jubarte_redlines` is three words,
        # each its own --runner=WORD (argparse append; `-m` needs the `=` form).
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            root = tmp / "root"
            py = root / "jubarte-python" / ".venv" / "bin" / "python"
            args = self.runner_args(root, console_script=False, tmp=tmp)
        self.assertEqual(
            args,
            [f"--runner={py}", "--runner=-m", "--runner=jubarte_redlines"],
            f"--runner got {args!r} (argparse drops -m without the = form)",
        )

    def test_console_script_runner_is_one_word(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            root = tmp / "root"
            script = root / "jubarte-python" / ".venv" / "bin" / "jubarte-redlines"
            args = self.runner_args(root, console_script=True, tmp=tmp)
        self.assertEqual(args, [f"--runner={script}"], f"--runner got {args!r}")

    def test_runner_words_survive_a_root_with_a_tab_and_a_space(self) -> None:
        # printf %q quotes a control char as $'..\t..', which shlex.split
        # does not undo; --runner=WORD passes the path verbatim instead.
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            root = tmp / "tab\there dir"
            py = root / "jubarte-python" / ".venv" / "bin" / "python"
            args = self.runner_args(root, console_script=False, tmp=tmp)
        self.assertEqual(
            args,
            [f"--runner={py}", "--runner=-m", "--runner=jubarte_redlines"],
            f"--runner got {args!r}",
        )


class RunnerEndToEnd(unittest.TestCase):
    """gen_cli_docs.py runs a --runner argv built of --runner=WORD (#1810)."""

    @staticmethod
    def page(tmp: Path) -> Path:
        page = tmp / "python.md"
        page.write_text(
            "<!-- gen:cli-python:start -->\nold\n<!-- gen:cli-python:end -->\n"
        )
        return page

    @staticmethod
    def run_docs(
        runner: list[str], page: Path, **env: str
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(GEN_CLI_DOCS),
                *[f"--runner={word}" for word in runner],
                "--display", "jubarte-redlines",
                "--file", str(page),
                "--marker", "cli-python",
            ],
            env=dict(os.environ, **env),
            capture_output=True,
            text=True,
            timeout=20,
        )

    def test_dash_m_runner_reaches_the_cli(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            (tmp / "jubarte_redlines.py").write_text(
                "print('jubarte-redlines 0.15.0')\nprint('Read documents')\n"
            )
            page = self.page(tmp)
            result = self.run_docs(
                [sys.executable, "-m", "jubarte_redlines"],
                page,
                PYTHONPATH=str(tmp),
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("jubarte-redlines 0.15.0", page.read_text())

    def test_runner_path_with_a_tab_and_a_space_reaches_the_subprocess(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            script = tmp / "tab\there dir" / "jubarte"
            script.parent.mkdir()
            script.write_text("#!/bin/sh\nprintf 'jubarte-redlines 0.15.0\\n'\n")
            script.chmod(0o755)
            page = self.page(tmp)
            result = self.run_docs([str(script)], page)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("jubarte-redlines 0.15.0", page.read_text())

    def test_windows_style_path_reaches_the_subprocess_verbatim(self) -> None:
        # shlex.split eats the backslashes of an unquoted C:\.. path; argv
        # words do not (POSIX exec tolerates backslashes in a file name).
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            script = tmp / "C:\\Users\\Arthur\\jubarte.exe"
            script.write_text("#!/bin/sh\nprintf 'jubarte-redlines 0.15.0\\n'\n")
            script.chmod(0o755)
            page = self.page(tmp)
            result = self.run_docs([str(script)], page)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("jubarte-redlines 0.15.0", page.read_text())

    def test_one_word_runner_without_equals_still_parses(self) -> None:
        # Backward compatibility: `--runner PATH` (space-separated, one word)
        # stays accepted beside the repeatable --runner=WORD form.
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            script = tmp / "jubarte"
            script.write_text("#!/bin/sh\nprintf 'jubarte-redlines 0.15.0\\n'\n")
            script.chmod(0o755)
            page = self.page(tmp)
            result = subprocess.run(
                [
                    sys.executable,
                    str(GEN_CLI_DOCS),
                    "--runner", str(script),
                    "--display", "jubarte-redlines",
                    "--file", str(page),
                    "--marker", "cli-python",
                ],
                env=dict(os.environ),
                capture_output=True,
                text=True,
                timeout=20,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("jubarte-redlines 0.15.0", page.read_text())


if __name__ == "__main__":
    unittest.main()
