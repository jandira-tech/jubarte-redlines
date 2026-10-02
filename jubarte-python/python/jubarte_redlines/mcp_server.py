# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte-mcp``: the Document facade as MCP tools over stdio.

Every path argument must resolve under ``--root`` (default: the current
directory); outputs are files under it, and their paths are returned, never
their bytes. No tool replaces an existing file unless ``overwrite`` is true.
Install with ``pip install 'jubarte-redlines[mcp]'`` or run with
``uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .``.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any

try:
    from mcp.server import MCPServer
    from mcp.server.mcpserver.exceptions import ToolError
    from mcp.types import ToolAnnotations
except ImportError as exc:  # pragma: no cover - exercised only without the extra
    raise ImportError(
        "jubarte-mcp needs the MCP SDK: pip install 'jubarte-redlines[mcp]'"
    ) from exc

from ._native import JubarteError
from .document import Document, EditPlanError, capabilities
from .models import CompareOptions

INSTRUCTIONS = (
    "jubarte reads and edits Word .docx files under the server root. "
    "Read first: docx_text gives Markdown with [body:p:N] paragraph ids. "
    "Edit with docx_edit: a JSON plan of exact, anchored operations; it writes "
    "a clean copy and a Word redline under out_dir and refuses the whole plan "
    "if any anchor fails. Check results with docx_render (PNG pages) and "
    "docx_changes. Every path must be under the server root; outputs never "
    "replace existing files unless overwrite is true.\n\n"
    "Other tools: docx_inspect (paragraph ids, styles, limitations), "
    "docx_compare (two files into a tracked-changes redline), docx_accept and "
    "docx_reject (resolve tracked changes by id, author or kind), "
    "docx_capabilities (what this build supports), docx_validate, "
    "docx_comments and docx_audit (available when the engine build has them)."
)

_READ = ToolAnnotations(readOnlyHint=True, destructiveHint=False, idempotentHint=True, openWorldHint=False)
_WRITE = ToolAnnotations(readOnlyHint=False, destructiveHint=False, idempotentHint=False, openWorldHint=False)


def _plain(value: Any) -> Any:
    """Dataclasses, tuples and lists as JSON-ready values."""
    if hasattr(value, "__dataclass_fields__"):
        return {k: v for k, v in asdict(value).items() if not k.startswith("_")}
    if isinstance(value, (list, tuple)):
        return [_plain(v) for v in value]
    return value


def build_server(*, root: Path) -> MCPServer:
    """An MCP server whose tools only touch files under ``root``."""
    root = Path(root).expanduser().resolve()
    mcp = MCPServer("jubarte", instructions=INSTRUCTIONS)

    def contained(path: str) -> Path:
        # resolve() follows symlinks, so a link pointing out of root is refused.
        p = (root / Path(path).expanduser()).resolve()
        if not p.is_relative_to(root):
            raise ToolError(f"{path} is outside root {root}")
        return p

    def load(path: str) -> Document:
        p = contained(path)
        try:
            return Document.read(p)
        except OSError as e:
            raise ToolError(f"cannot read {path}: {e.strerror or e}") from e

    def fresh(path: Path, overwrite: bool) -> Path:
        # An output may itself be a symlink (dangling ones report exists() False);
        # resolve it so a write can never follow a link out of root.
        target = path.resolve()
        if not target.is_relative_to(root):
            raise ToolError(f"{path} is outside root {root}")
        if path.exists() and not overwrite:
            raise ToolError(f"{path} exists; pass overwrite=true to replace it")
        return target

    def write(path: Path, data: bytes | str) -> str:
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(data, str):
            path.write_text(data, encoding="utf-8")
        else:
            path.write_bytes(data)
        return str(path)

    def engine(call: Any) -> Any:
        """Run an engine call, turning its errors into messages the model sees."""
        try:
            return call()
        except EditPlanError:
            raise  # docx_edit reports it with its code and outcomes
        except (JubarteError, ValueError, TypeError, OSError) as e:
            raise ToolError(str(e)) from e

    def missing(feature: str) -> ToolError:
        return ToolError(f"this engine build lacks {feature}; upgrade jubarte-redlines")

    @mcp.tool(annotations=_READ)
    def docx_capabilities() -> dict[str, Any]:
        """What this engine build can do: operations, edit kinds, limits, runtime."""
        return capabilities()

    @mcp.tool(annotations=_READ)
    def docx_text(path: str) -> str:
        """The document body as Markdown with a [body:p:N] id before each paragraph.

        Those ids are the coordinates an edit plan uses. Read before you edit.
        """
        doc = load(path)
        return engine(doc.markdown)

    @mcp.tool(annotations=_READ)
    def docx_inspect(path: str) -> dict[str, Any]:
        """The engine snapshot: summary, paragraphs with ids, styles, runs and limitations, stories."""
        doc = load(path)
        return json.loads(engine(doc.inspect_json))

    @mcp.tool(annotations=_WRITE)
    def docx_edit(
        path: str,
        plan: dict[str, Any],
        out_dir: str,
        pdf: bool = False,
        png_dpi: float | None = None,
        overwrite: bool = False,
    ) -> dict[str, Any]:
        """Apply an edit plan to path; write clean.docx, redline.docx, patch.diff and report.json under out_dir.

        plan is {"schema_version": 1, "author": ..., "operations": [...]}; each
        operation names a paragraph id from docx_text and exact text. The whole
        plan is refused, with nothing written, if any operation cannot resolve;
        the error carries the engine code and every operation's outcome. With
        pdf or png_dpi the redline is also rendered to redline.pdf and
        redline-page-NN.png. Existing files are kept unless overwrite is true.
        """
        doc = load(path)
        out = contained(out_dir)
        try:
            result = engine(lambda: doc.edit(plan))
        except EditPlanError as e:
            raise ToolError(
                json.dumps(
                    {
                        "code": e.code,
                        "operation": e.operation,
                        "message": e.message,
                        "outcomes": _plain(e.outcomes),
                    }
                )
            ) from e
        names = ["clean.docx", "redline.docx", "patch.diff", "report.json"]
        rendered = None
        if pdf or png_dpi:
            rendered = engine(lambda: result.redline.render(pdf=pdf, png_dpi=png_dpi))
            if rendered.pdf is not None:
                names.append("redline.pdf")
            names += [f"redline-page-{i:02d}.png" for i in range(1, len(rendered.pngs) + 1)]
        for name in names:  # check every output before writing any
            fresh(out / name, overwrite)
        report = json.loads(result.report._json) if result.report._json else _plain(result.report)
        written = {
            "clean": write(out / "clean.docx", result.clean.to_bytes()),
            "redline": write(out / "redline.docx", result.redline.to_bytes()),
            "patch": write(out / "patch.diff", str(result.diff)),
            "report_path": write(out / "report.json", json.dumps(report, indent=2)),
            "report": report,
            "pages": [],
        }
        if rendered is not None:
            if rendered.pdf is not None:
                written["pdf"] = write(out / "redline.pdf", rendered.pdf)
            written["pages"] = [
                write(out / f"redline-page-{i:02d}.png", png) for i, png in enumerate(rendered.pngs, start=1)
            ]
        return written

    @mcp.tool(annotations=_WRITE)
    def docx_render(
        path: str,
        out_dir: str,
        dpi: float = 96,
        pages: list[int] | None = None,
        pdf: bool = False,
        overwrite: bool = False,
    ) -> dict[str, Any]:
        """Render path to page-NN.png files (and render.pdf when pdf is true) under out_dir.

        pages selects 1-based pages to write (default all). Returns the file
        paths, the page count and the text painted on each page.
        """
        doc = load(path)
        out = contained(out_dir)
        rendered = engine(lambda: doc.render(pdf=pdf, png_dpi=dpi))
        wanted = [
            (i, png) for i, png in enumerate(rendered.pngs, start=1) if pages is None or i in pages
        ]
        targets = [out / f"page-{i:02d}.png" for i, _ in wanted]
        if rendered.pdf is not None:
            targets.append(out / "render.pdf")
        for target in targets:
            fresh(target, overwrite)
        result: dict[str, Any] = {
            "page_count": rendered.report.page_count,
            "pages": [write(out / f"page-{i:02d}.png", png) for i, png in wanted],
            "text": [p.text for p in rendered.report.pages],
            "fonts": _plain(rendered.report.fonts),
        }
        if rendered.pdf is not None:
            result["pdf"] = write(out / "render.pdf", rendered.pdf)
        return result

    @mcp.tool(annotations=_WRITE)
    def docx_compare(
        original: str,
        modified: str,
        out: str,
        author: str,
        date: str | None = None,
        overwrite: bool = False,
    ) -> dict[str, Any]:
        """Compare original with modified into a tracked-changes redline written to out.

        author signs every change; date is an ISO-8601 timestamp with an offset
        (default: the engine's fixed timestamp). Returns out and the change list.
        """
        a, b = load(original), load(modified)
        target = fresh(contained(out), overwrite)
        options = engine(lambda: CompareOptions(date=date))
        redline = engine(lambda: a.compare(b, author=author, options=options))
        return {"out": write(target, redline.to_bytes()), "changes": _plain(engine(redline.changes))}

    @mcp.tool(annotations=_READ)
    def docx_changes(path: str) -> list[dict[str, Any]]:
        """Every tracked change in path, with the id docx_accept and docx_reject select by."""
        doc = load(path)
        return _plain(engine(doc.changes))

    def resolve(
        action: str,
        path: str,
        out: str,
        ids: Sequence[str] | None,
        authors: Sequence[str] | None,
        kinds: Sequence[str] | None,
        overwrite: bool,
    ) -> dict[str, Any]:
        doc = load(path)
        target = fresh(contained(out), overwrite)
        method = getattr(doc, action)
        result = engine(lambda: method(ids=ids, authors=authors, kinds=kinds))
        return {"out": write(target, result.to_bytes()), "remaining": len(engine(result.changes))}

    @mcp.tool(annotations=_WRITE)
    def docx_accept(
        path: str,
        out: str,
        ids: list[str] | None = None,
        authors: list[str] | None = None,
        kinds: list[str] | None = None,
        overwrite: bool = False,
    ) -> dict[str, Any]:
        """Accept tracked changes in path and write the result to out.

        With no selection every change is accepted. ids (from docx_changes),
        authors and kinds (insertion, deletion, move, formatting) select the
        changes matching every list given; the rest stay tracked.
        """
        return resolve("accept", path, out, ids, authors, kinds, overwrite)

    @mcp.tool(annotations=_WRITE)
    def docx_reject(
        path: str,
        out: str,
        ids: list[str] | None = None,
        authors: list[str] | None = None,
        kinds: list[str] | None = None,
        overwrite: bool = False,
    ) -> dict[str, Any]:
        """Reject tracked changes in path and write the result to out (selection as docx_accept)."""
        return resolve("reject", path, out, ids, authors, kinds, overwrite)

    @mcp.tool(annotations=_READ)
    def docx_validate(path: str, original: str | None = None, author: str | None = None) -> list[dict[str, Any]]:
        """Findings that would make Word refuse or repair path; with original and author, also check that every edit against original is a tracked change by author."""
        doc = load(path)
        base = load(original) if original is not None else None
        if not hasattr(Document, "validate"):
            raise missing("validate")
        if (base is None) != (author is None):
            raise ToolError("original and author go together")
        findings = list(engine(doc.validate))
        if base is not None:
            findings.extend(engine(lambda: doc.audit_tracked(base, author=author)))
        return _plain(findings)

    @mcp.tool(annotations=_READ)
    def docx_comments(path: str) -> list[dict[str, Any]]:
        """Comment records in path: id, author, date, anchor and text."""
        doc = load(path)
        if not hasattr(Document, "comments"):
            raise missing("comments")
        return _plain(engine(doc.comments))

    @mcp.tool(annotations=_READ)
    def docx_audit(path: str, rules: dict[str, Any] | None = None) -> list[dict[str, Any]]:
        """Audit findings for path against rules (default: the engine's built-in rules)."""
        doc = load(path)
        if not hasattr(Document, "audit"):
            raise missing("audit")
        return _plain(engine(lambda: doc.audit(rules=rules)))

    return mcp


def main(argv: list[str] | None = None) -> int:
    """Serve the tools over stdio until the client closes the stream."""
    parser = argparse.ArgumentParser(prog="jubarte-mcp", description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="directory every path must sit under")
    args = parser.parse_args(argv)
    root = args.root.expanduser().resolve()
    if not root.is_dir():
        parser.error(f"--root {args.root} is not a directory")
    build_server(root=root).run(transport="stdio")
    return 0


if __name__ == "__main__":
    sys.exit(main())
