#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Diff-fitting step of the conejo job in .github/workflows/jules.yml.

Jules receives the PR diff, the changed-file list and the static-analysis
report inside one prompt environment variable, which Linux caps at 131,058
bytes of value. The step with `id: diff` must cut every part that grows with
the PR at a line boundary, write only valid UTF-8 (the runner turns each
invalid byte into a 3-byte U+FFFD, which would break the budget), and use a
random heredoc delimiter so diff content cannot end an output early. Each
test runs that step's `run:` script with bash in a throwaway git fixture
repository (a bare origin plus a clone: the base on `main`, the change on a
branch) and checks what it writes to `$GITHUB_OUTPUT`.

Run with PyYAML: `uv run --with pyyaml python3 scripts/test_jules_workflow.py`.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
WORKFLOW = HERE.parent / ".github" / "workflows" / "jules.yml"

WORKFLOW_DOC: dict = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
STEPS: list[dict] = WORKFLOW_DOC["jobs"]["conejo"]["steps"]


def step_by_id(step_id: str) -> dict:
    step = next((s for s in STEPS if s.get("id") == step_id), None)
    if step is None:
        raise AssertionError(f"the conejo job has no step with id '{step_id}'")
    return step


DIFF_STEP = step_by_id("diff")
JULES_STEP = step_by_id("jules")
RUN_SCRIPT: str = str(DIFF_STEP["run"])

# The skill files the workflow inlines, keyed by output name.
SKILLS_DIR = WORKFLOW.parent.parent.parent / "skills" / "jules-el-conejo"
LOAD_SKILL = re.compile(r"load_skill (\w+) (\S+\.md)")
SKILL_FILES: dict[str, str] = dict(LOAD_SKILL.findall(str(step_by_id("skills")["run"])))
SKILL_TEXT: dict[str, str] = {
    key: (SKILLS_DIR / file).read_text(encoding="utf-8") for key, file in SKILL_FILES.items()
}
SKILL_BYTES = sum(len(text.encode("utf-8")) for text in SKILL_TEXT.values())


def sh(cwd: Path, *cmd: str) -> str:
    # Drop GIT_DIR, GIT_INDEX_FILE and friends: a git hook exports them, and
    # inherited they would aim every fixture command at the enclosing repo.
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    return subprocess.run(cmd, cwd=cwd, env=env, check=True, capture_output=True, text=True).stdout


def fixture(root: Path, files: dict[str, bytes | str]) -> Path:
    """A repo whose origin/main is the base and HEAD adds `files` on a branch."""
    origin = root / "origin.git"
    work = root / "work"
    sh(root, "git", "init", "-q", "--bare", "-b", "main", str(origin))
    sh(root, "git", "clone", "-q", str(origin), str(work))

    def git(*args: str) -> None:
        sh(work, "git", "-c", "user.name=t", "-c", "user.email=t@t", *args)

    (work / "README.md").write_text("base\n", encoding="utf-8")
    git("add", ".")
    git("commit", "-q", "-m", "base")
    git("push", "-q", "origin", "main")
    git("checkout", "-q", "-b", "feature")
    for name, body in files.items():
        data = body if isinstance(body, bytes) else body.encode("utf-8")
        (work / name).write_bytes(data)
    git("add", ".")
    git("commit", "-q", "-m", "change")
    # The real skills, so the step budgets for the text the prompt inlines.
    skills = work / ".jules-el-conejo-skills" / "skills" / "jules-el-conejo"
    skills.mkdir(parents=True)
    for key, file in SKILL_FILES.items():
        (skills / file).write_text(SKILL_TEXT[key], encoding="utf-8")
    return work


def iconv_shim(root: Path, body: str) -> Path:
    """A directory whose `iconv` runs `body`, to put first on PATH."""
    shim = root / "shim"
    shim.mkdir()
    (shim / "iconv").write_text(f"#!/bin/sh\n{body}\n", encoding="utf-8")
    (shim / "iconv").chmod(0o755)
    return shim


def run_step_raw(cwd: Path, env: dict[str, str] | None = None) -> bytes:
    """Runs the step's script like the runner does; returns GITHUB_OUTPUT's bytes."""
    out = cwd.parent / "github_output"
    out.write_bytes(b"")
    step_env = {str(k): "" if "${{" in str(v) else str(v) for k, v in (DIFF_STEP.get("env") or {}).items()}
    subprocess.run(
        # GitHub's default shell for a run step: -e, no pipefail.
        ["bash", "-e", "-c", RUN_SCRIPT],
        cwd=cwd,
        env={
            "PATH": os.environ.get("PATH", ""),
            "GITHUB_OUTPUT": str(out),
            **step_env,
            "BASE_REF": "main",
            **(env or {}),
        },
        check=True,
        capture_output=True,
    )
    return out.read_bytes()


# The runner's $GITHUB_OUTPUT grammar: `key=value` and `key<<DELIM` heredocs.
HEREDOC = re.compile(r"^([A-Za-z_][A-Za-z0-9_-]*)<<(.+)$")
KEY_VALUE = re.compile(r"^([A-Za-z_][A-Za-z0-9_-]*)=(.*)$")


def run_step(cwd: Path, env: dict[str, str] | None = None) -> dict[str, str]:
    """Runs the step and parses GITHUB_OUTPUT the way the runner does."""
    lines = run_step_raw(cwd, env).decode("utf-8").split("\n")
    outputs: dict[str, str] = {}
    i = 0
    while i < len(lines):
        heredoc = HEREDOC.match(lines[i])
        if heredoc:
            key, delim = heredoc.groups()
            i += 1
            body: list[str] = []
            while i < len(lines) and lines[i] != delim:
                body.append(lines[i])
                i += 1
            if i >= len(lines):
                raise AssertionError(f"unterminated {key}")
            outputs[key] = "\n".join(body)
            i += 1
            continue
        kv = KEY_VALUE.match(lines[i])
        if kv:
            outputs[kv.group(1)] = kv.group(2)
        i += 1
    return outputs


# The prompt's two expression forms: a plain step output, and the truncated
# note, whose cut branch carries the whole-diff byte count.
EXPRESSION = re.compile(r"\$\{\{\s*steps\.(\w+)\.outputs\.(\w+)\s*\}\}")
TRUNCATED_NOTE = re.compile(
    r"\$\{\{\s*steps\.diff\.outputs\.truncated == 'true'"
    r" && format\('([^']*)', steps\.diff\.outputs\.size\) \|\| '([^']*)'\s*\}\}"
)
PLAIN_EXPRESSION = re.compile(r"\$\{\{\s*steps\.\w+\.outputs\.\w+\s*\}\}")
ANY_EXPRESSION = re.compile(r"\$\{\{.*?\}\}", re.DOTALL)
FORMAT_PREFIX = r"\$\{\{\s*steps\.diff\.outputs\.truncated == 'true' && format\("


def render(template: str, diff: dict[str, str], changed: str, report: str) -> str:
    """Substitutes every ${{ }} in the prompt with worst-case values, like the runner."""
    pr = {
        "number": "99999",
        "title": "🐇" * 256,
        "head_ref": "h" * 255,
        "base_ref": "b" * 255,
        "head_sha": "f" * 40,
    }

    def cut_or_whole(m: re.Match[str]) -> str:
        cut, whole = m.groups()
        return cut.replace("{0}", diff["size"]) if diff["truncated"] == "true" else whole

    def substitute(m: re.Match[str]) -> str:
        step, key = m.groups()
        if step == "pr" and key in pr:
            return pr[key]
        if step == "diff" and key in diff:
            return diff[key]
        if step == "skills" and key == "changed_files":
            return changed
        if step == "skills" and key in SKILL_TEXT:
            return SKILL_TEXT[key]
        if step == "static_analysis" and key == "report":
            return report
        raise AssertionError(f"unhandled expression {m.group(0)}")

    out = EXPRESSION.sub(substitute, TRUNCATED_NOTE.sub(cut_or_whole, template))
    # Substituted values may contain ${{ (the skills do); the runner does not
    # re-evaluate them. Check the template's own expressions instead.
    for expr in ANY_EXPRESSION.findall(template):
        known = PLAIN_EXPRESSION.fullmatch(expr) or re.match(FORMAT_PREFIX, expr)
        if not known:
            raise AssertionError(f"unrendered expression: {expr}")
    return out


def step_index(step_id: str) -> int:
    return next(i for i, s in enumerate(STEPS) if s.get("id") == step_id)


class ConejoDiffStep(unittest.TestCase):
    def test_exists_and_is_wired_into_the_jules_prompt(self) -> None:
        self.assertIn("diff", [s.get("id") for s in STEPS], "the conejo job has no diff step")
        prompt = str(JULES_STEP["with"]["prompt"])
        self.assertIn("${{ steps.diff.outputs.diff }}", prompt)
        self.assertIn("${{ steps.pr.outputs.base_ref }}", prompt)
        self.assertIn("git fetch --deepen", prompt)
        # The step runs before Jules and after the skills and the report exist.
        self.assertGreater(step_index("diff"), step_index("static_analysis"))
        self.assertLess(step_index("diff"), step_index("jules"))

    def test_sends_a_small_diff_whole(self) -> None:
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            work = fixture(Path(tmp), {"lib.rs": "fn main() {}\n"})
            outputs = run_step(work)
        self.assertEqual(outputs["truncated"], "false")
        self.assertIn("+fn main() {}", outputs["diff"])
        self.assertIn("diff --git a/lib.rs b/lib.rs", outputs["diff"])

    def test_cuts_a_large_diff_at_a_line_boundary_under_the_prompt_budget(self) -> None:
        big = "\n".join(f"line {i} {'x' * 40}" for i in range(20_000))
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            work = fixture(Path(tmp), {"big.txt": f"{big}\n"})
            outputs = run_step(work)
        self.assertEqual(outputs["truncated"], "true")
        size = len(outputs["diff"].encode("utf-8"))
        # Skills + diff must leave room under 131,058 for the rest.
        self.assertLess(size + SKILL_BYTES, 120_000)
        self.assertGreater(size, 10_000)
        # Every kept line is whole: the last one is a complete "+line N xxx..." line.
        last = outputs["diff"].rstrip().split("\n")[-1]
        self.assertRegex(last, r"^\+line \d+ x{40}$")

    def test_keeps_the_fully_rendered_prompt_under_the_env_limit_in_the_worst_case(self) -> None:
        # 600 long file names (codex's case) plus a big diff and a long report.
        files = {f"{i:06d}{'x' * 140}.txt": "y" * 200 + "\n" for i in range(600)}
        report = "\n".join(f"#### finding {i} {'r' * 150}" for i in range(700))
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            work = fixture(Path(tmp), files)
            changed = sh(work, "git", "diff", "--name-only", "origin/main...HEAD")
            outputs = run_step(work, {"CHANGED_FILES": changed, "STATIC_REPORT": report})
            prompt = render(str(JULES_STEP["with"]["prompt"]), outputs, changed, report)
        # JULES_PROMPT= (13 bytes) + value + NUL must fit in 131,072.
        self.assertLessEqual(len(prompt.encode("utf-8")), 131_058)
        self.assertIn("000000", prompt)

    def test_sends_only_valid_utf_8_so_the_runner_cannot_inflate_the_diff(self) -> None:
        latin1 = b"\xff\n" * 30_000
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            work = fixture(Path(tmp), {"latin1.txt": latin1})
            raw = run_step_raw(work)
        raw.decode("utf-8")  # strict: raises UnicodeDecodeError on any invalid byte

    def test_does_not_let_diff_content_end_the_output_early(self) -> None:
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            work = fixture(Path(tmp), {"evil.txt": "__JULES_DIFF_EOF__\ninjected=1\nEOF\n"})
            outputs = run_step(work)
        self.assertNotIn("injected", outputs)
        self.assertIn("+injected=1", outputs["diff"])

    def test_keeps_the_output_when_iconv_reports_invalid_input(self) -> None:
        # glibc's iconv -c still writes the output but exits 1 when the input
        # ends in a truncated sequence; the shim reproduces that on any host.
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            shim = iconv_shim(Path(tmp), f'"{shutil.which("iconv")}" "$@"; exit 1')
            work = fixture(Path(tmp), {"lib.rs": "fn main() {}\n"})
            outputs = run_step(work, {"PATH": f"{shim}:{os.environ['PATH']}"})
        self.assertIn("+fn main() {}", outputs["diff"])

    def test_fails_loudly_when_iconv_cannot_run(self) -> None:
        # Exit 127 is "command not found"; swallowing it would send Jules an
        # empty diff labelled complete.
        with tempfile.TemporaryDirectory(prefix="conejo-diff-") as tmp:
            shim = iconv_shim(Path(tmp), "exit 127")
            work = fixture(Path(tmp), {"lib.rs": "fn main() {}\n"})
            with self.assertRaises(subprocess.CalledProcessError):
                run_step_raw(work, {"PATH": f"{shim}:{os.environ['PATH']}"})


class FixtureIsolation(unittest.TestCase):
    def test_never_touches_the_enclosing_repository_when_git_dir_is_set(self) -> None:
        # A git hook exports GIT_DIR and GIT_INDEX_FILE for the repository being
        # committed to; fixture git commands must not follow them there.
        with tempfile.TemporaryDirectory(prefix="conejo-sentinel-") as tmp:
            sentinel = Path(tmp) / "sentinel"
            sh(Path(tmp), "git", "init", "-q", "-b", "trunk", str(sentinel))
            config = (sentinel / ".git" / "config").read_text(encoding="utf-8")
            saved = {k: os.environ.get(k) for k in ("GIT_DIR", "GIT_INDEX_FILE")}
            os.environ["GIT_DIR"] = str(sentinel / ".git")
            os.environ["GIT_INDEX_FILE"] = str(sentinel / ".git" / "index")
            try:
                try:
                    fx = Path(tmp) / "fx"
                    fx.mkdir()
                    fixture(fx, {"lib.rs": "fn main() {}\n"})
                except subprocess.CalledProcessError:
                    pass  # a leaking fixture can fail midway; the sentinel decides
            finally:
                for key, value in saved.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
            self.assertEqual((sentinel / ".git" / "config").read_text(encoding="utf-8"), config)
            self.assertEqual(list((sentinel / ".git" / "refs" / "heads").iterdir()), [])


if __name__ == "__main__":
    unittest.main()
