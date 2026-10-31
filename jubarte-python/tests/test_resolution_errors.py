# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``edit -p rN --accept`` resolved lines: one view, and only an engine refusal falls back.

The compiled ``_native`` extension is not required. When no extension file
sits beside the package, this module installs a stub on ``sys.modules``
before importing jubarte_redlines, enough for these two CLI paths. CI builds
the extension (``maturin develop``); these tests then use it, and still
monkeypatch ``resolution_text``.
"""

from __future__ import annotations

import importlib.machinery
import json
import shutil
import sys
import types
from pathlib import Path

import pytest

_SRC = Path(__file__).resolve().parents[1] / "python"
_FIXTURE = Path(__file__).resolve().parents[2] / "tests" / "fixtures" / "agent-view" / "received.docx"
_RESOLVED = 'accepted r0 in p3: "RESOLVED-SENTINEL"\n'

_REPORT = {
    "schema_version": 1,
    "ok": True,
    "source_sha256": "abc",
    "base_sha256": "def",
    "guarded": False,
    "author": "Jubarte",
    "date": "2026-10-11T00:00:00Z",
    "existing_revisions": "auto",
    "paragraphs": {"from": 1, "to": 1},
    "operations": [],
    "comments_added": 0,
    "revisions": {
        "inserted": 0,
        "deleted": 0,
        "moved": 0,
        "format_changed": 0,
        "total": 0,
        "replaced": 0,
    },
    "resolved_revisions": {"accepted": ["r0"], "rejected": []},
    "fields": [],
    "notes": [],
}


def _extension_present() -> bool:
    package = _SRC / "jubarte_redlines"
    return any((package / f"_native{suffix}").is_file() for suffix in importlib.machinery.EXTENSION_SUFFIXES)


def _stub_module() -> types.ModuleType:
    """The calls ``edit -p r0 --accept`` makes on the way to ``resolution_text``."""
    module = types.ModuleType("jubarte_redlines._native")

    class JubarteError(Exception):
        """Engine refusal, standing in for the extension's exception."""

    def wants_json(_arguments: list[str]) -> bool:
        return False

    def parse_cli_json(
        arguments: list[str], program: str = "jubarte-redlines", supported: list[str] | None = None
    ) -> str:
        del program, supported
        if not arguments or arguments[0] != "edit":
            raise AssertionError(arguments)
        file = arguments[1]
        output = None
        out_dir = None
        when = None
        index = 2
        while index < len(arguments):
            word = arguments[index]
            if word == "-p":
                index += 2
            elif word == "--accept":
                index += 1
            elif word == "-o":
                output = arguments[index + 1]
                index += 2
            elif word == "--out-dir":
                out_dir = arguments[index + 1]
                index += 2
            elif word == "--datetime":
                when = arguments[index + 1]
                index += 2
            else:
                raise AssertionError(word)
        args = {
            "file": file,
            "output": output,
            "out_dir": out_dir,
            "plan": None,
            "operations": [{"location": "r0", "accept": True}],
            "author": "Jubarte",
            "datetime": when,
            "existing_revisions": "auto",
            "editing_mode": False,
            "in_place": False,
            "force": False,
            "quiet": False,
            "dry_run": False,
            "consume": False,
            "pdf": False,
            "png": False,
            "keep_others": False,
            "author_given": False,
        }
        return json.dumps({"command": "edit", "args": args, "notes": []})

    def refused_input(_path: object, _data: bytes, _word: bool) -> None:
        return None

    def flag_plan(
        _verb: str,
        _operations: str,
        _docx: bytes,
        *,
        author: str,
        date: str | None = None,
        existing: str = "auto",
    ) -> tuple[str, list[str]]:
        plan = {
            "schema_version": 1,
            "author": author,
            "date": date,
            "existing_revisions": existing,
            "resolve_revisions": {"accept": ["r0"]},
            "operations": [],
        }
        return json.dumps(plan), []

    def edit_json(_docx: bytes, _plan_json: str) -> tuple[bool, bytes, bytes, str]:
        return True, b"PK\x03\x04clean", b"PK\x03\x04redline", json.dumps(_REPORT)

    def redline_diff_json(
        _docx: bytes, *, name: str, author: str, date: str, columns: int = 72, own_only: bool = False
    ) -> tuple[str, str]:
        del name, author, date, columns, own_only
        return "", "[]"

    def report_jsonl(_report_json: str) -> str:
        return '{"comments":{"added":0},"ev":"summary","status":"ok"}\n'

    def source_sha256(_data: bytes) -> str:
        return "0123456789abcdef"

    def resolution_text(_source: bytes, _resolved_json: str) -> str:
        return _RESOLVED

    def changed_view(_docx: bytes, _author: str, *, accepted: bool = False, source: str | None = None) -> str:
        del accepted
        return f"---\nsource: {source}\nfallback view\n"

    module.JubarteError = JubarteError
    module.__version__ = "0.0.0-stub"
    module.wants_json = wants_json
    module.parse_cli_json = parse_cli_json
    module.refused_input = refused_input
    module.flag_plan = flag_plan
    module.edit_json = edit_json
    module.redline_diff_json = redline_diff_json
    module.report_jsonl = report_jsonl
    module.source_sha256 = source_sha256
    module.resolution_text = resolution_text
    module.changed_view = changed_view
    # Imported by jubarte_redlines.__init__ even though these tests never call them.
    for name in ("accept_revisions", "compare_documents", "docx_to_pdf", "get_revisions_json", "reject_revisions"):
        setattr(module, name, lambda *args, **kwargs: (_ for _ in ()).throw(JubarteError("stub")))
    return module


def _install_import() -> bool:
    sys.path.insert(0, str(_SRC))
    if _extension_present():
        return False
    sys.modules["jubarte_redlines._native"] = _stub_module()
    return True


NATIVE_STUBBED = _install_import()

from jubarte_redlines import _native
from jubarte_redlines.__main__ import main


def _argv(tmp_path: Path, bundle: bool) -> list[str]:
    source = tmp_path / "received.docx"
    shutil.copy(_FIXTURE, source)
    args = ["edit", str(source), "-p", "r0", "--accept", "--datetime", "2026-10-11T00:00:00Z"]
    if bundle:
        args += ["--out-dir", str(tmp_path / "bundle")]
    else:
        args += ["-o", str(tmp_path / "out.docx")]
    return args


def _run(argv: list[str], capsys: pytest.CaptureFixture[str]) -> tuple[int, str, str]:
    try:
        code = main(argv)
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 1
    captured = capsys.readouterr()
    return code, captured.out, captured.err


def _wrote(err: str) -> list[str]:
    return [line for line in err.splitlines() if line.startswith("wrote ")]


@pytest.mark.parametrize("bundle", [False, True], ids=["single", "bundle"])
def test_engine_refusal_prints_the_fallback_view_once(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch, bundle: bool
) -> None:
    def refuse(_source: bytes, _resolved_json: str) -> str:
        raise _native.JubarteError("engine refused the resolution")

    monkeypatch.setattr(_native, "resolution_text", refuse)
    code, out, err = _run(_argv(tmp_path, bundle), capsys)
    compact = out.replace(" ", "")
    assert code == 0
    assert out.count("---\nsource:") == 1
    assert "RESOLVED-SENTINEL" not in out
    assert "engine refused the resolution" not in out
    assert "engine refused the resolution" not in err
    assert compact.count('"ev":"summary"') == (1 if bundle else 0)
    assert len(_wrote(err)) == 1


@pytest.mark.parametrize("bundle", [False, True], ids=["single", "bundle"])
@pytest.mark.parametrize(
    ("kind", "message"),
    [("runtime", "engine defect"), ("oserror", "disk failed")],
    ids=["runtime", "oserror"],
)
def test_resolution_defect_exits_with_no_view(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
    bundle: bool,
    kind: str,
    message: str,
) -> None:
    def boom(_source: bytes, _resolved_json: str) -> str:
        raise RuntimeError(message) if kind == "runtime" else OSError(message)

    monkeypatch.setattr(_native, "resolution_text", boom)
    code, out, err = _run(_argv(tmp_path, bundle), capsys)
    assert code == 1
    assert out == ""
    assert f"error: {message}" in err
    assert "---\nsource:" not in out


@pytest.mark.parametrize("bundle", [False, True], ids=["single", "bundle"])
def test_resolution_text_is_printed_once(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch, bundle: bool
) -> None:
    calls = {"n": 0}

    def ok(_source: bytes, _resolved_json: str) -> str:
        calls["n"] += 1
        return _RESOLVED

    monkeypatch.setattr(_native, "resolution_text", ok)
    code, out, err = _run(_argv(tmp_path, bundle), capsys)
    assert code == 0
    assert out == _RESOLVED
    assert out.count("RESOLVED-SENTINEL") == 1
    assert "---\nsource:" not in out
    assert calls["n"] == 1
    assert len(_wrote(err)) == 1
