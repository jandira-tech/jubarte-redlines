# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The MCP server exposes the Document facade with path containment."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any

import pytest

pytest.importorskip("mcp")
import anyio
from jubarte_redlines import Document
from jubarte_redlines.mcp_server import INSTRUCTIONS, build_server, main
from mcp import Client
from test_document import make_document

TOOLS = {
    "docx_capabilities",
    "docx_text",
    "docx_inspect",
    "docx_edit",
    "docx_render",
    "docx_compare",
    "docx_changes",
    "docx_accept",
    "docx_reject",
    "docx_validate",
    "docx_comments",
    "docx_audit",
}
READ_ONLY = {
    "docx_capabilities",
    "docx_text",
    "docx_inspect",
    "docx_changes",
    "docx_validate",
    "docx_comments",
    "docx_audit",
}


def call(root: Path, *calls: tuple[str, dict[str, Any]]) -> list[Any]:
    """Run tool calls in order against one in-process server."""

    async def go() -> list[Any]:
        async with Client(build_server(root=root)) as client:
            return [await client.call_tool(name, args) for name, args in calls]

    return anyio.run(go)


def error_text(result: Any) -> str:
    assert result.is_error, result
    return result.content[0].text


def write_pair(root: Path) -> tuple[Path, Path]:
    original = root / "a.docx"
    modified = root / "b.docx"
    original.write_bytes(make_document("The fee is ten."))
    modified.write_bytes(make_document("The fee is twelve."))
    return original, modified


# -- contract --------------------------------------------------------------


def test_the_tool_set_is_the_contract(tmp_path: Path) -> None:
    async def go() -> Any:
        async with Client(build_server(root=tmp_path)) as client:
            return (await client.list_tools()).tools

    tools = anyio.run(go)
    assert {t.name for t in tools} == TOOLS
    for tool in tools:
        assert tool.description, tool.name
        assert len(tool.description) <= 2048, tool.name
        assert bool(tool.annotations and tool.annotations.read_only_hint) == (tool.name in READ_ONLY), tool.name


def test_instructions_fit_the_hosts_limits() -> None:
    assert len(INSTRUCTIONS) <= 2048
    head = INSTRUCTIONS[:512]
    assert "docx_text" in head and "root" in head


def test_capabilities_returns_the_manifest(tmp_path: Path) -> None:
    (result,) = call(tmp_path, ("docx_capabilities", {}))
    assert not result.is_error
    assert result.structured_content["runtime"] == "python"


# -- containment -----------------------------------------------------------


def test_text_reads_a_file_under_root_and_refuses_one_outside(tmp_path: Path) -> None:
    root = tmp_path / "root"
    root.mkdir()
    inside = root / "in.docx"
    inside.write_bytes(make_document("inside text"))
    outside = tmp_path / "outside.docx"
    outside.write_bytes(make_document("outside text"))

    ok, bad, dotted = call(
        root,
        ("docx_text", {"path": str(inside)}),
        ("docx_text", {"path": str(outside)}),
        ("docx_text", {"path": str(root / ".." / "outside.docx")}),
    )
    assert not ok.is_error and "[body:p:0] inside text" in ok.content[0].text
    assert "outside root" in error_text(bad)
    assert "outside root" in error_text(dotted)


def test_relative_paths_resolve_against_root_not_the_process_cwd(tmp_path: Path) -> None:
    (tmp_path / "rel.docx").write_bytes(make_document("relative text"))
    (ok,) = call(tmp_path, ("docx_text", {"path": "rel.docx"}))
    assert not ok.is_error and "relative text" in ok.content[0].text


def test_a_symlink_escaping_root_is_refused(tmp_path: Path) -> None:
    root = tmp_path / "root"
    root.mkdir()
    target = tmp_path / "secret.docx"
    target.write_bytes(make_document("secret"))
    (root / "link.docx").symlink_to(target)
    (bad,) = call(root, ("docx_text", {"path": str(root / "link.docx")}))
    assert "outside root" in error_text(bad)


def test_outputs_outside_root_are_refused(tmp_path: Path) -> None:
    root = tmp_path / "root"
    root.mkdir()
    original, modified = write_pair(root)
    (bad,) = call(
        root,
        (
            "docx_compare",
            {"original": str(original), "modified": str(modified), "out": str(tmp_path / "x.docx"), "author": "R"},
        ),
    )
    assert "outside root" in error_text(bad)
    assert not (tmp_path / "x.docx").exists()


def test_missing_and_corrupt_inputs_are_tool_errors(tmp_path: Path) -> None:
    (tmp_path / "junk.docx").write_bytes(b"not a zip")
    missing, junk = call(
        tmp_path,
        ("docx_text", {"path": "nope.docx"}),
        ("docx_inspect", {"path": "junk.docx"}),
    )
    assert "nope.docx" in error_text(missing)
    assert error_text(junk) != "Error executing tool docx_inspect"


# -- reading ---------------------------------------------------------------


def test_inspect_returns_the_engine_snapshot(tmp_path: Path) -> None:
    (tmp_path / "d.docx").write_bytes(make_document("Hello there."))
    (result,) = call(tmp_path, ("docx_inspect", {"path": "d.docx"}))
    assert not result.is_error
    snapshot = result.structured_content
    assert {"summary", "paragraphs"} <= set(snapshot)
    assert snapshot["paragraphs"][0]["id"] == "body:p:0"


def test_inspect_json_is_the_engine_json_unchanged(tmp_path: Path) -> None:
    doc = Document.from_bytes(make_document("Hello there."))
    data = json.loads(doc.inspect_json())
    assert data["paragraphs"][0]["id"] == doc.inspect().paragraphs[0].id


# -- edit --------------------------------------------------------------------

PLAN = {
    "schema_version": 1,
    "author": "Agent",
    "operations": [{"kind": "replace", "paragraph": "body:p:0", "find": "ten", "replacement": "twelve"}],
}


def test_edit_writes_outputs_under_out_dir_and_never_overwrites(tmp_path: Path) -> None:
    src = tmp_path / "c.docx"
    src.write_bytes(make_document("The fee is ten."))
    args = {"path": str(src), "plan": PLAN, "out_dir": str(tmp_path / "review")}
    first, second, third = call(
        tmp_path,
        ("docx_edit", args),
        ("docx_edit", args),
        ("docx_edit", {**args, "overwrite": True}),
    )
    assert not first.is_error
    paths = first.structured_content
    assert Path(paths["clean"]).exists() and Path(paths["redline"]).exists()
    assert Path(paths["patch"]).read_text()
    assert json.loads(Path(paths["report_path"]).read_text())["ok"] is True
    assert paths["report"]["ok"] is True
    assert "_json" not in paths["report"]
    assert paths["pages"] == []
    assert "overwrite" in error_text(second)
    assert not third.is_error


def test_edit_can_render_the_redline(tmp_path: Path) -> None:
    (tmp_path / "c.docx").write_bytes(make_document("The fee is ten."))
    (result,) = call(
        tmp_path,
        ("docx_edit", {"path": "c.docx", "plan": PLAN, "out_dir": "out", "pdf": True, "png_dpi": 24}),
    )
    assert not result.is_error, result.content
    pages = result.structured_content["pages"]
    assert pages and all(Path(p).read_bytes().startswith(b"\x89PNG") for p in pages)
    assert (tmp_path / "out" / "redline.pdf").read_bytes().startswith(b"%PDF")


def test_a_refused_plan_returns_the_engine_payload_and_writes_nothing(tmp_path: Path) -> None:
    (tmp_path / "c.docx").write_bytes(make_document("The fee is ten."))
    plan = {**PLAN, "operations": [{**PLAN["operations"][0], "find": "eleven"}]}
    (bad,) = call(tmp_path, ("docx_edit", {"path": "c.docx", "plan": plan, "out_dir": "out"}))
    text = error_text(bad)
    payload = json.loads(text[text.index("{") :])
    assert payload["code"] == "ANCHOR_NOT_FOUND"
    assert isinstance(payload["outcomes"], list)
    assert not (tmp_path / "out").exists()


def test_an_invalid_plan_is_a_tool_error(tmp_path: Path) -> None:
    (tmp_path / "c.docx").write_bytes(make_document("x"))
    (bad,) = call(tmp_path, ("docx_edit", {"path": "c.docx", "plan": {"nonsense": 1}, "out_dir": "out"}))
    assert error_text(bad) != "Error executing tool docx_edit"


# -- render ------------------------------------------------------------------


def test_render_writes_pages_and_optional_pdf(tmp_path: Path) -> None:
    (tmp_path / "d.docx").write_bytes(make_document("Page text."))
    first, again, picked = call(
        tmp_path,
        ("docx_render", {"path": "d.docx", "out_dir": "r", "dpi": 24, "pdf": True}),
        ("docx_render", {"path": "d.docx", "out_dir": "r", "dpi": 24}),
        ("docx_render", {"path": "d.docx", "out_dir": "r2", "dpi": 24, "pages": [2]}),
    )
    assert not first.is_error, first.content
    out = first.structured_content
    assert out["page_count"] == 1
    assert [Path(p).name for p in out["pages"]] == ["page-01.png"]
    assert Path(out["pdf"]).read_bytes().startswith(b"%PDF")
    assert "Page text." in out["text"][0]
    assert "overwrite" in error_text(again)
    assert picked.structured_content["pages"] == []


# -- compare, changes, accept, reject -----------------------------------------


def test_compare_then_list_accept_and_reject(tmp_path: Path) -> None:
    original, modified = write_pair(tmp_path)
    compared, changes, accepted, rejected, partial = call(
        tmp_path,
        (
            "docx_compare",
            {
                "original": str(original),
                "modified": str(modified),
                "out": "red.docx",
                "author": "Reviewer",
                "date": "2026-10-02T00:00:00Z",
            },
        ),
        ("docx_changes", {"path": "red.docx"}),
        ("docx_accept", {"path": "red.docx", "out": "acc.docx"}),
        ("docx_reject", {"path": "red.docx", "out": "rej.docx"}),
        ("docx_accept", {"path": "red.docx", "out": "part.docx", "kinds": ["insertion"], "authors": ["Reviewer"]}),
    )
    assert not compared.is_error, compared.content
    assert compared.structured_content["out"] == str(tmp_path / "red.docx")
    listed = compared.structured_content["changes"]
    assert listed and all(c["author"] == "Reviewer" for c in listed)
    assert changes.structured_content["result"] == listed
    assert "twelve" in Document.read(accepted.structured_content["out"]).markdown()
    assert "ten" in Document.read(rejected.structured_content["out"]).markdown()
    assert not partial.is_error
    remaining = Document.read(tmp_path / "part.docx").changes()
    assert remaining and all(c.kind != "insertion" for c in remaining)


def test_accept_by_id_and_refuse_overwrite(tmp_path: Path) -> None:
    original, modified = write_pair(tmp_path)
    red = Document.read(original).compare(Document.read(modified), author="R")
    (tmp_path / "red.docx").write_bytes(red.to_bytes())
    first_id = red.changes()[0].id
    ok, again, bad_date = call(
        tmp_path,
        ("docx_reject", {"path": "red.docx", "out": "o.docx", "ids": [first_id]}),
        ("docx_reject", {"path": "red.docx", "out": "o.docx", "ids": [first_id]}),
        (
            "docx_compare",
            {"original": str(original), "modified": str(modified), "out": "c.docx", "author": "R", "date": "never"},
        ),
    )
    assert not ok.is_error
    assert "overwrite" in error_text(again)
    assert "date" in error_text(bad_date)


# -- features from sibling branches ------------------------------------------


@pytest.mark.parametrize(
    ("tool", "args", "feature"),
    [
        ("docx_validate", {"path": "d.docx"}, "validate"),
        ("docx_comments", {"path": "d.docx"}, "comments"),
        ("docx_audit", {"path": "d.docx"}, "audit"),
    ],
)
def test_tools_whose_engine_feature_is_absent_say_so(tmp_path: Path, tool: str, args: dict, feature: str) -> None:
    (tmp_path / "d.docx").write_bytes(make_document("x"))
    if hasattr(Document, feature):
        pytest.skip(f"Document.{feature} exists in this build")
    (bad,) = call(tmp_path, (tool, args))
    assert f"lacks {feature}" in error_text(bad)


def test_docx_validate_runs_the_engine_and_audits_against_the_original(tmp_path: Path) -> None:
    write_pair(tmp_path)
    clean, audited, half = call(
        tmp_path,
        ("docx_validate", {"path": "a.docx"}),
        ("docx_validate", {"path": "b.docx", "original": "a.docx", "author": "Z"}),
        ("docx_validate", {"path": "b.docx", "original": "a.docx"}),
    )
    assert clean.structured_content["result"] == []
    codes = {f["code"] for f in audited.structured_content["result"]}
    assert "UNTRACKED_EDIT" in codes, audited.structured_content
    assert "author" in error_text(half)


def test_feature_tools_still_contain_paths(tmp_path: Path) -> None:
    (bad,) = call(tmp_path, ("docx_validate", {"path": "/etc/hostname"}))
    assert "outside root" in error_text(bad)


def test_feature_tools_delegate_when_the_engine_has_them(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """When a sibling branch lands the method, the tool returns its result."""
    (tmp_path / "d.docx").write_bytes(make_document("x"))
    (tmp_path / "o.docx").write_bytes(make_document("y"))
    seen: dict[str, Any] = {}

    def validate(self: Document) -> list[dict]:
        seen["validate"] = True
        return [{"code": "OK"}]

    def audit_tracked(self: Document, original: Document, *, author: str) -> list[dict]:
        seen["audit"] = (isinstance(original, Document), author)
        return [{"code": "AUDIT"}]

    monkeypatch.setattr(Document, "validate", validate, raising=False)
    monkeypatch.setattr(Document, "audit_tracked", audit_tracked, raising=False)
    monkeypatch.setattr(Document, "comments", lambda self: ({"id": "c1"},), raising=False)
    monkeypatch.setattr(Document, "audit", lambda self, rules=None: [{"rules": rules}], raising=False)
    v, c, a = call(
        tmp_path,
        ("docx_validate", {"path": "d.docx", "original": "o.docx", "author": "Z"}),
        ("docx_comments", {"path": "d.docx"}),
        ("docx_audit", {"path": "d.docx", "rules": {"r": 1}}),
    )
    assert v.structured_content["result"] == [{"code": "OK"}, {"code": "AUDIT"}]
    assert seen["validate"] is True
    assert seen["audit"] == (True, "Z")
    assert c.structured_content["result"] == [{"id": "c1"}]
    assert a.structured_content["result"] == [{"rules": {"r": 1}}]


# -- entry point -------------------------------------------------------------


def test_main_rejects_a_root_that_is_not_a_directory(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exc:
        main(["--root", str(tmp_path / "missing")])
    assert exc.value.code == 2
    assert "not a directory" in capsys.readouterr().err


def test_main_serves_stdio(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    seen: dict[str, Any] = {}

    class Fake:
        def run(self, transport: str = "stdio") -> None:
            seen["transport"] = transport

    monkeypatch.setattr("jubarte_redlines.mcp_server.build_server", lambda *, root: seen.setdefault("root", root) and Fake())
    assert main(["--root", str(tmp_path)]) == 0
    assert seen == {"root": tmp_path.resolve(), "transport": "stdio"}


def test_stdio_smoke_initialize_and_list_tools(tmp_path: Path) -> None:
    """The console entry point speaks MCP over stdio as a real subprocess."""
    messages = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}},
        },
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
    ]
    # Keep stdin open until the tools/list reply arrives, as a real client
    # does: closing it ends the stdio session, and a request still queued
    # then goes unanswered (about one run in ten when stdin closed at once).
    proc = subprocess.Popen(
        [sys.executable, "-m", "jubarte_redlines.mcp_server", "--root", str(tmp_path)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env={**os.environ, "PYTHONUNBUFFERED": "1"},
    )
    assert proc.stdin is not None and proc.stdout is not None
    # A server that never answers must fail the test, not hang it.
    watchdog = threading.Timer(60, proc.kill)
    watchdog.start()
    try:
        proc.stdin.write("".join(json.dumps(m) + "\n" for m in messages))
        proc.stdin.flush()
        listed = None
        for line in proc.stdout:
            reply = json.loads(line) if line.strip() else {}
            if reply.get("id") == 2:
                listed = reply
                break
        proc.stdin.close()
        assert proc.stderr is not None
        stderr = proc.stderr.read()
        proc.wait(timeout=60)
    finally:
        watchdog.cancel()
        if proc.poll() is None:
            proc.kill()
    assert listed is not None, f"no tools/list reply; stderr: {stderr}"
    assert {t["name"] for t in listed["result"]["tools"]} == TOOLS
    assert "Traceback" not in stderr


# -- hardening -----------------------------------------------------------------


@pytest.mark.parametrize("overwrite", [False, True])
def test_an_output_symlink_pointing_out_of_root_is_never_followed(tmp_path: Path, overwrite: bool) -> None:
    root = tmp_path / "root"
    (root / "out").mkdir(parents=True)
    (root / "c.docx").write_bytes(make_document("The fee is ten."))
    escaped = tmp_path / "escaped.docx"
    # Dangling: exists() is False, so only a resolve check can catch it.
    (root / "out" / "clean.docx").symlink_to(escaped)
    (bad,) = call(
        root,
        ("docx_edit", {"path": "c.docx", "plan": PLAN, "out_dir": "out", "overwrite": overwrite}),
    )
    assert "outside root" in error_text(bad)
    assert not escaped.exists()
    assert not (root / "out" / "redline.docx").exists()


def test_a_render_page_symlink_pointing_out_of_root_is_never_followed(tmp_path: Path) -> None:
    root = tmp_path / "root"
    (root / "r").mkdir(parents=True)
    (root / "d.docx").write_bytes(make_document("x"))
    escaped = tmp_path / "escaped.png"
    (root / "r" / "page-01.png").symlink_to(escaped)
    (bad,) = call(root, ("docx_render", {"path": "d.docx", "out_dir": "r", "dpi": 24}))
    assert "outside root" in error_text(bad)
    assert not escaped.exists()


def test_an_out_of_range_dpi_is_refused_by_the_engine_with_its_message(tmp_path: Path) -> None:
    (tmp_path / "d.docx").write_bytes(make_document("x"))
    render, edit = call(
        tmp_path,
        ("docx_render", {"path": "d.docx", "out_dir": "r", "dpi": 100000}),
        ("docx_edit", {"path": "d.docx", "plan": PLAN, "out_dir": "e", "png_dpi": 100000}),
    )
    assert "dpi" in error_text(render)
    assert "dpi" in error_text(edit) or "ANCHOR" in error_text(edit)
    assert not (tmp_path / "r").exists()


def test_edit_of_a_corrupt_package_is_a_tool_error(tmp_path: Path) -> None:
    (tmp_path / "junk.docx").write_bytes(b"not a zip")
    (bad,) = call(tmp_path, ("docx_edit", {"path": "junk.docx", "plan": PLAN, "out_dir": "out"}))
    assert "INVALID_PACKAGE" in error_text(bad)
    assert not (tmp_path / "out").exists()


def test_edit_can_render_pngs_without_a_pdf(tmp_path: Path) -> None:
    (tmp_path / "c.docx").write_bytes(make_document("The fee is ten."))
    (result,) = call(tmp_path, ("docx_edit", {"path": "c.docx", "plan": PLAN, "out_dir": "out", "png_dpi": 24}))
    assert not result.is_error, result.content
    assert result.structured_content["pages"]
    assert "pdf" not in result.structured_content
    assert not (tmp_path / "out" / "redline.pdf").exists()


def test_running_the_module_calls_main(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    import runpy

    # Run it fresh, as `python -m` does, instead of over the imported copy.
    monkeypatch.delitem(sys.modules, "jubarte_redlines.mcp_server")
    monkeypatch.setattr(sys, "argv", ["jubarte-mcp", "--root", str(tmp_path / "missing")])
    with pytest.raises(SystemExit) as exc:
        runpy.run_module("jubarte_redlines.mcp_server", run_name="__main__")
    assert exc.value.code == 2
