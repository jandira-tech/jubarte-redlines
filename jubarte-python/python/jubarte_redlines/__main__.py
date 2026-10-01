# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte-redlines`` (or ``python -m jubarte_redlines``): the ``jubarte``
binary's commands over the installed wheel, with the same names, flags, output
files and exit codes. ``uvx jubarte-redlines`` runs it without installing.

    uvx jubarte-redlines redline a.docx b.docx -o redline.docx
    python -m jubarte_redlines inspect letter.docx --json
    python -m jubarte_redlines text letter.docx
    python -m jubarte_redlines edit letter.docx --plan plan.json --out-dir review --pdf --png
    python -m jubarte_redlines convert letter.docx --png --dpi 150 --report pages.json
    python -m jubarte_redlines compare a.docx b.docx -o redline.docx --author Legal
    python -m jubarte_redlines accept redline.docx -o clean.docx
    python -m jubarte_redlines changes redline.docx --json
    python -m jubarte_redlines reject redline.docx -o out.docx --id body:rev:12
    python -m jubarte_redlines capabilities --json

Exit codes: 0 success, 1 error (I/O, engine, existing output), 2 usage, 3 edit
plan refused (its per-operation report is on stdout; nothing was written).
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict
from collections.abc import Sequence
from pathlib import Path

from . import __version__, capabilities
from .document import Document, EditPlanError
from .models import PdfOptions

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_PLAN_REFUSED = 3


class CliError(Exception):
    """A user-facing failure; the message is printed as ``error: ...``."""


# The first bytes of an OLE compound file: a Word 97-2003 .doc, or a
# password-encrypted document of any Word version.
OLE_MAGIC = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"


def _read(path: Path) -> Document:
    try:
        doc = Document.read(path)
    except OSError as exc:
        raise CliError(f"reading {path}: {exc}") from exc
    if doc.to_bytes().startswith(OLE_MAGIC):
        raise CliError(f"{path} is a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password")
    return doc


def _ensure_writable(path: Path, force: bool) -> None:
    if path.exists() and not force:
        raise CliError(f"output '{path}' already exists (use --force to overwrite)")


def _write(path: Path, data: bytes | str) -> None:
    try:
        if isinstance(data, str):
            path.write_text(data, encoding="utf-8")
        else:
            path.write_bytes(data)
    except OSError as exc:
        raise CliError(f"writing {path}: {exc}") from exc


def _pdf_options(args: argparse.Namespace) -> PdfOptions:
    revisions = getattr(args, "revisions", "conventional")
    palette = getattr(args, "revision_palette", None)
    compress = getattr(args, "compress", False)
    if revisions == "custom" and palette is None:
        raise CliError("--revisions custom needs --revision-palette")
    if revisions != "custom" and palette is not None:
        raise CliError("--revision-palette needs --revisions custom")
    return PdfOptions(compress=compress, revisions=revisions, revision_palette=palette)


def _png_name(stem: str, index: int, count: int) -> str:
    width = max(2, len(str(count)))
    return f"{stem}-page-{index + 1:0{width}d}.png"


# -- commands -----------------------------------------------------------------


def cmd_inspect(args: argparse.Namespace) -> int:
    doc = _read(args.file)
    if args.json:
        from . import _native

        print(_native.inspect_json(doc.to_bytes()))
        return EXIT_OK
    snap = doc.inspect()
    s = snap.summary
    print(f"sha256: {snap.source_sha256}")
    print(
        f"paragraphs: {s.paragraphs}  tables: {s.tables}  fields: {s.fields}  sections: {s.sections}  "
        f"comments: {s.comments}  revisions: {s.revisions}  footnotes: {s.footnotes}  endnotes: {s.endnotes}  "
        f"headers: {s.headers}  footers: {s.footers}  images: {s.images}  numbering: {str(s.list_numbering).lower()}  "
        f"track_changes: {str(s.track_changes).lower()}"
    )
    for p in snap.paragraphs:
        flags = [f for f in (p.style, "numbered" if p.numbered else None, "table" if p.in_table else None, "page-break" if p.page_break else None) if f]
        flags.extend(p.limitations)
        preview = p.text[:80] + ("…" if len(p.text) > 80 else "")
        print(f"{p.id}\t[{','.join(flags)}]\t{preview}")
    return EXIT_OK


def cmd_text(args: argparse.Namespace) -> int:
    sys.stdout.write(_read(args.file).markdown())
    return EXIT_OK


def cmd_edit(args: argparse.Namespace) -> int:
    doc = _read(args.file)
    try:
        plan_text = Path(args.plan).read_text(encoding="utf-8")
    except OSError as exc:
        raise CliError(f"reading {args.plan}: {exc}") from exc
    out_dir: Path = args.out_dir
    if not args.dry_run:
        if out_dir.exists() and not args.force:
            raise CliError(f"output directory '{out_dir}' already exists (use --force to replace its files)")
        if out_dir.resolve() == args.file.resolve().parent:
            raise CliError("--out-dir must not be the input's own directory")
    try:
        if args.dry_run:
            sys.stdout.write(doc.preview(plan_text).to_jsonl())
            return EXIT_OK
        result = doc.edit(plan_text)
    except EditPlanError as exc:
        for i, o in enumerate(exc.outcomes, start=1):
            row: dict[str, object] = {"ev": "op", "i": i, "id": o.id, "op": o.kind, "status": o.status, "matches": o.matches}
            for key, value in (("at", o.paragraph), ("ctx", o.context), ("code", o.code), ("message", o.message)):
                if value is not None:
                    row[key] = value
            print(json.dumps(row, ensure_ascii=False))
        print(json.dumps({"ev": "summary", "status": "failed", "code": exc.code, "operation": exc.operation, "message": exc.message}, ensure_ascii=False))
        print(f"error: plan refused: {exc}", file=sys.stderr)
        return EXIT_PLAN_REFUSED
    lines = result.report.to_jsonl().splitlines()
    summary = lines.pop()
    outputs: list[tuple[str, bytes]] = [
        ("clean.docx", result.clean.to_bytes()),
        ("redline.docx", result.redline.to_bytes()),
        ("patch.diff", result.diff.text.encode("utf-8")),
    ]
    if args.pdf or args.png:
        options = PdfOptions(compress=True, revisions=args.revisions, revision_palette=args.revision_palette)
        pages: dict[str, int] = {}
        starts: dict[str, list[str]] = {}
        for name, document in (("redline", result.redline), ("clean", result.clean)):
            rendered = document.render(pdf=args.pdf, png_dpi=args.dpi if args.png else None, options=options)
            pages[name] = rendered.report.page_count
            starts[name] = [(p.text.splitlines() or [""])[0][:60] for p in rendered.report.pages]
            if rendered.pdf is not None:
                outputs.append((f"{name}.pdf", rendered.pdf))
            for i, png in enumerate(rendered.pngs):
                outputs.append((_png_name(name, i, len(rendered.pngs)), png))
        lines.append(json.dumps({"ev": "render", "engine": f"jubarte {__version__}", "pages": pages, "page_starts": starts}, ensure_ascii=False))
    from . import _native

    out_dir.mkdir(parents=True, exist_ok=True)
    saved = []
    for name, data in outputs:
        _write(out_dir / name, data)
        saved.append({"f": name, "bytes": len(data), "sha256": _native.source_sha256(data)})
    lines.append(json.dumps({"ev": "save", "dir": str(out_dir), "outputs": saved}, ensure_ascii=False))
    lines.append(summary)
    _write(out_dir / "report.jsonl", "\n".join(lines) + "\n")
    if args.quiet:
        return EXIT_OK
    print(summary)
    print(f"wrote {out_dir} ({len(outputs) + 1} files: clean.docx, redline.docx, patch.diff, report.jsonl{', …' if len(outputs) > 3 else ''})")
    sys.stdout.write(result.diff.text)
    return EXIT_OK


def cmd_convert(args: argparse.Namespace) -> int:
    doc = _read(args.file)
    output: Path = args.output or args.file.with_suffix(".pdf")
    want_pdf = args.pdf or not args.png
    for side, what in ((args.font_report, "--font-report"), (args.report, "--report")):
        if side is not None:
            if side.resolve() in (output.resolve(), args.file.resolve()):
                raise CliError(f"{what} '{side}' is the same file as the PDF output or the input")
            _ensure_writable(side, args.force)
    if want_pdf:
        _ensure_writable(output, args.force)
    rendered = doc.render(pdf=want_pdf, png_dpi=args.dpi if args.png else None, options=_pdf_options(args))
    pages = rendered.report.page_count
    # The page count is known only now; check every PNG path before the first
    # write so a refused page leaves no partial bundle.
    png_paths = [output.parent / _png_name(output.stem, i, len(rendered.pngs)) for i in range(len(rendered.pngs))]
    for path in png_paths:
        _ensure_writable(path, args.force)
    if rendered.pdf is not None:
        _write(output, rendered.pdf)
        print(f"wrote {output} ({len(rendered.pdf)} bytes, {pages} page{'' if pages == 1 else 's'})")
    if args.png:
        for path, png in zip(png_paths, rendered.pngs):
            _write(path, png)
        print(f"wrote {len(rendered.pngs)} PNG page{'' if len(rendered.pngs) == 1 else 's'} ({output.parent / output.stem}-page-NN.png, {args.dpi} dpi)")
    if args.font_report is not None:
        _write(args.font_report, json.dumps([asdict(f) for f in rendered.report.fonts]))
    if args.report is not None:
        report = {"page_count": pages, "pages": [asdict(p) for p in rendered.report.pages], "fonts": [asdict(f) for f in rendered.report.fonts]}
        _write(args.report, json.dumps(report, ensure_ascii=False))
    return EXIT_OK


def cmd_compare(args: argparse.Namespace) -> int:
    original = _read(args.original)
    modified = _read(args.modified)
    output: Path = args.output or args.original.with_name(f"{args.original.stem}_v_{args.modified.stem}.docx")
    _ensure_writable(output, args.force)
    from . import _native

    redline = _native.compare_documents(original.to_bytes(), modified.to_bytes(), author=args.author, date=args.date)
    _write(output, redline)
    print(f"wrote {output} ({len(redline)} bytes)")
    return EXIT_OK


def cmd_revisions(args: argparse.Namespace) -> int:
    from . import get_revisions

    rows = get_revisions(_read(args.file).to_bytes())
    if args.json:
        for row in rows:
            print(json.dumps(row, ensure_ascii=False))
    else:
        for row in rows:
            print(f"{row['type']}\t{row.get('author') or '-'}\t{row.get('part')}\t{(row.get('text') or '')[:60]!r}")
        print(f"{len(rows)} revision(s)")
    return EXIT_OK


def cmd_changes(args: argparse.Namespace) -> int:
    for change in (changes := _read(args.file).changes()):
        row = {key: value for key, value in asdict(change).items() if value is not None}
        if args.json:
            print(json.dumps(row, ensure_ascii=False))
            continue
        preview = json.dumps(change.text[:60], ensure_ascii=False)
        inside = f"\tinside {change.inside}" if change.inside else ""
        print(f"{change.id}\t{change.kind}\t{change.target}\t{change.author or '-'}\t{preview}{inside}")
    if not args.json:
        print(f"{len(changes)} change(s)")
    return EXIT_OK


def _resolution(args: argparse.Namespace, accept: bool) -> int:
    doc = _read(args.file)
    _ensure_writable(args.output, args.force)
    selection = {"ids": args.id, "authors": args.author, "kinds": args.kind}
    result = doc.accept(**selection) if accept else doc.reject(**selection)
    _write(args.output, result.to_bytes())
    print(f"wrote {args.output} ({len(result.to_bytes())} bytes)")
    return EXIT_OK


def cmd_accept(args: argparse.Namespace) -> int:
    return _resolution(args, accept=True)


def cmd_reject(args: argparse.Namespace) -> int:
    return _resolution(args, accept=False)


def cmd_capabilities(_args: argparse.Namespace) -> int:
    print(json.dumps(capabilities(), indent=2))
    return EXIT_OK


# -- parser -------------------------------------------------------------------


def _add_revision_flags(p: argparse.ArgumentParser) -> None:
    p.add_argument("--revisions", choices=["conventional", "word", "custom"], default="conventional", help="how tracked changes are painted")
    p.add_argument("--revision-palette", metavar="SPEC", help="marks for --revisions custom, e.g. deleted=#AA0000:strike,...")


def _prog() -> str:
    """The name the user typed: the console script's, or ``python -m …``."""
    script = Path(sys.argv[0])
    if script.name in ("__main__.py", "-m", "-c", ""):
        return "python -m jubarte_redlines"
    return script.stem if script.suffix.lower() == ".exe" else script.name


def build_parser(prog: str | None = None) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog=prog or _prog(), description="DOCX compare, tracked editing, inspection and rendering (the jubarte engine).")
    parser.add_argument("--version", action="version", version=f"jubarte-redlines {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    p = sub.add_parser("inspect", help="paragraph ids, formatting spans, limitations and package facts")
    p.add_argument("file", type=Path)
    p.add_argument("--json", action="store_true", help="emit the JSON snapshot")
    p.set_defaults(func=cmd_inspect)

    p = sub.add_parser("text", help="Markdown with [body:p:N] ids, the coordinates an edit plan uses")
    p.add_argument("file", type=Path)
    p.set_defaults(func=cmd_text)

    p = sub.add_parser("edit", help="apply an edit plan: clean.docx, redline.docx, patch.diff, report.jsonl (+ PDF/PNG)")
    p.add_argument("file", type=Path)
    p.add_argument("--plan", type=Path, required=True, metavar="PLAN.json")
    p.add_argument("--out-dir", type=Path, required=True, metavar="DIR")
    p.add_argument("--dry-run", action="store_true", help="resolve and report only; write nothing")
    p.add_argument("--force", action="store_true", help="replace an existing output directory's files")
    p.add_argument("--pdf", action="store_true", help="also write redline.pdf and clean.pdf")
    p.add_argument("--png", action="store_true", help="also write redline-page-NN.png and clean-page-NN.png")
    p.add_argument("--dpi", type=float, default=96.0)
    p.add_argument("-q", "--quiet", action="store_true", help="print nothing on success (patch.diff and report.jsonl are still written)")
    _add_revision_flags(p)
    p.set_defaults(func=cmd_edit)

    p = sub.add_parser("convert", help="DOCX to PDF and/or PNG pages, with an optional page report")
    p.add_argument("file", type=Path)
    p.add_argument("-o", "--output", type=Path, help="PDF path [default: <stem>.pdf beside the input]")
    p.add_argument("--force", action="store_true")
    p.add_argument("--pdf", action="store_true", help="write the PDF (default when --png is absent)")
    p.add_argument("--png", action="store_true", help="rasterize pages to <stem>-page-NN.png")
    p.add_argument("--dpi", type=float, default=96.0)
    p.add_argument("--compress", action="store_true", help="deflate PDF streams")
    p.add_argument("--font-report", type=Path, metavar="FILE", help="JSON font-resolution report")
    p.add_argument("--report", type=Path, metavar="FILE", help="JSON page report ({page_count, pages, fonts})")
    _add_revision_flags(p)
    p.set_defaults(func=cmd_convert)

    p = sub.add_parser("compare", aliases=["redline"], help="two documents into a Word tracked-changes document")
    p.add_argument("original", type=Path)
    p.add_argument("modified", type=Path)
    p.add_argument("-o", "--output", type=Path, help="[default: <original>_v_<modified>.docx]")
    p.add_argument("--author", default="jubarte")
    p.add_argument("--date", help="ISO-8601 revision timestamp (default: fixed epoch)")
    p.add_argument("--force", action="store_true")
    p.set_defaults(func=cmd_compare)

    p = sub.add_parser("revisions", help="list tracked revisions")
    p.add_argument("file", type=Path)
    p.add_argument("--json", action="store_true", help="one JSON object per line")
    p.set_defaults(func=cmd_revisions)

    p = sub.add_parser("changes", help="list each tracked change with the id accept/reject --id and edit plans take")
    p.add_argument("file", type=Path)
    p.add_argument("--json", action="store_true", help="one JSON object per line")
    p.set_defaults(func=cmd_changes)

    for name, func, help_text in (("accept", cmd_accept, "accept tracked changes (all, or the ones selected)"), ("reject", cmd_reject, "reject tracked changes (all, or the ones selected)")):
        p = sub.add_parser(name, help=help_text)
        p.add_argument("file", type=Path)
        p.add_argument("-o", "--output", type=Path, required=True)
        p.add_argument("--force", action="store_true")
        p.add_argument("--id", action="append", metavar="ID", help="only this change (body:rev:12); repeatable")
        p.add_argument("--author", action="append", metavar="NAME", help="only changes by this author; repeatable")
        p.add_argument("--kind", action="append", choices=["insertion", "deletion", "move", "formatting"], help="only changes of this kind; repeatable")
        p.set_defaults(func=func)

    p = sub.add_parser("capabilities", help="what this build can do")
    p.add_argument("--json", action="store_true", help="(the output is JSON either way)")
    p.set_defaults(func=cmd_capabilities)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Run the CLI; returns the exit code (``SystemExit`` only for usage errors)."""
    args = build_parser().parse_args(argv)
    try:
        return int(args.func(args))
    except CliError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_ERROR
    except Exception as exc:  # engine errors (JubarteError) and I/O surprises
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_ERROR


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
