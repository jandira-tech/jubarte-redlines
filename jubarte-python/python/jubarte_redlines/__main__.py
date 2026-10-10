# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte-redlines`` (or ``python -m jubarte_redlines``): the ``jubarte``
binary's commands over the installed wheel, with the same names, flags, output
files and exit codes. ``uvx jubarte-redlines`` runs it without installing.

    uvx jubarte-redlines redline a.docx b.docx -o redline.docx
    python -m jubarte_redlines inspect letter.docx --json
    python -m jubarte_redlines read letter.docx
    python -m jubarte_redlines edit letter.docx --plan plan.json --out-dir review --pdf --png
    python -m jubarte_redlines convert letter.docx --png --dpi 150 --report pages.json
    python -m jubarte_redlines convert letter.docx --png --pages 3-5
    python -m jubarte_redlines diff-render before.docx after.docx --out-dir diff
    python -m jubarte_redlines convert draft.md --page a4 -o draft.docx
    python -m jubarte_redlines compare a.docx b.docx -o redline.docx --author Legal
    python -m jubarte_redlines accept redline.docx -o clean.docx
    python -m jubarte_redlines changes redline.docx --json
    python -m jubarte_redlines reject redline.docx -o out.docx --id body:rev:12
    python -m jubarte_redlines capabilities --json
    python -m jubarte_redlines validate redline.docx --original a.docx --author Legal

Exit codes: 0 success, 1 error (I/O, engine, existing output), 2 usage, 3 edit
plan refused (its per-operation report is on stdout; nothing was written), 5
``diff-render`` found a page that differs. ``validate`` exits 2 when it has
findings.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict
from collections.abc import Sequence
from pathlib import Path

from . import __version__, capabilities
from .document import Document, EditPlanError, diff_render
from .models import Finding, PdfOptions, RevisionStyle

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_USAGE = 2
EXIT_PLAN_REFUSED = 3
EXIT_PAGES_DIFFER = 5
EXIT_FINDINGS = 2


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
    if not doc.to_bytes().startswith(b"PK\x03\x04"):
        raise CliError(f"reading {path}: invalid DOCX (expected a ZIP package)")
    return doc


def _read_side(path: Path, force_kind: str | None) -> bytes | str:
    """CLI format metadata wins; unknown suffixes use the core ZIP sniff rule."""
    try:
        data = path.read_bytes()
    except OSError as exc:
        raise CliError(f"reading {path}: {exc}") from exc
    if data.startswith(OLE_MAGIC):
        raise CliError(f"{path} is a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password")
    kind = force_kind or ("docx" if data.startswith(b"PK\x03\x04") else "md")
    if kind == "docx":
        if not data.startswith(b"PK\x03\x04"):
            raise CliError(f"reading {path}: invalid DOCX (expected a ZIP package)")
        return data
    try:
        return data.decode("utf-8-sig")
    except UnicodeError as exc:
        raise CliError(f"reading {path}: invalid UTF-8 Markdown: {exc}") from exc


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
    default_revisions: RevisionStyle = "conventional"
    revisions: RevisionStyle = getattr(args, "revisions", default_revisions)
    palette = getattr(args, "revision_palette", None)
    compress = getattr(args, "compress", False)
    if revisions == "custom" and palette is None:
        raise CliError("--revisions custom needs --revision-palette")
    if revisions != "custom" and palette is not None:
        raise CliError("--revision-palette needs --revisions custom")
    return PdfOptions(
        compress=compress,
        revisions=revisions,
        revision_palette=palette,
        move_comments=getattr(args, "move_comments", False),
        changed_only=getattr(args, "changed_only", False),
    )


def _png_name(stem: str, index: int, count: int) -> str:
    width = max(2, len(str(count)))
    return f"{stem}-page-{index + 1:0{width}d}.png"


def _parse_pages(spec: str) -> list[int]:
    """``--pages`` as page numbers counted from 1, ascending and without
    repeats: ``1-3,7`` is ``[1, 2, 3, 7]``."""

    def page(text: str) -> int:
        text = text.strip()
        if not (text.isascii() and text.isdigit()):
            raise CliError(f"--pages '{spec}': '{text}' is not a page number")
        if int(text) == 0:
            raise CliError(f"--pages '{spec}': pages are counted from 1")
        return int(text)

    pages: set[int] = set()
    for item in spec.split(","):
        if not item.strip():
            raise CliError(f"--pages '{spec}': empty item")
        first, dash, last = item.partition("-")
        if not dash:
            pages.add(page(item))
            continue
        lo, hi = page(first), page(last)
        if lo > hi:
            raise CliError(f"--pages '{spec}': '{item.strip()}' runs backwards")
        pages.update(range(lo, hi + 1))
    return sorted(pages)


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


def _print_view(docx: bytes, options: dict, source: str) -> int:
    """The agent view of ``docx`` with the read options ``options``."""
    from . import _native

    markdown, warnings = _native.read_view(
        docx,
        track_changes=options.get("track_changes"),
        comments=options.get("comments", "inline") == "inline",
        dates=bool(options.get("dates")),
        page_markers=not options.get("no_page_markers"),
        paragraphs=options.get("paragraphs"),
        head=options.get("head"),
        tail=options.get("tail"),
        changed=bool(options.get("changed")),
        by=options.get("by"),
        source=source,
    )
    for warning in warnings:
        print(f"warning: {warning}", file=sys.stderr)
    sys.stdout.write(markdown)
    return EXIT_OK


def cmd_read(args: argparse.Namespace) -> int:
    return _print_view(_read(args.file).to_bytes(), vars(args), args.file.name)


_EDITING_KEEPS = (
    "--editing-mode needs a document without tracked changes; it has some, "
    "so pass --existing-revisions accept or reject"
)


def _default_out_dir(file: Path) -> Path:
    """``<dir>/<stem>.edit`` next to the source, as the binary."""
    return file.with_name(f"{file.stem}.edit")


def _edit_plan(args: argparse.Namespace, verb: str, source: bytes) -> tuple[str, list[str]]:
    """``--plan``'s text, or the plan the operation flags describe, and its notes."""
    from . import _native

    if getattr(args, "plan", None) is not None:
        try:
            return Path(args.plan).read_text(encoding="utf-8"), []
        except OSError as exc:
            raise CliError(f"reading {args.plan}: {exc}") from exc
    try:
        plan, notes = _native.flag_plan(
            verb,
            json.dumps(args.operations, ensure_ascii=False),
            source,
            author=args.author,
            date=args.datetime,
            existing=args.existing_revisions,
        )
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(EXIT_USAGE) from exc
    return plan, list(notes)


def cmd_edit(args: argparse.Namespace) -> int:
    return _run_edit(args, "edit")


def cmd_add(args: argparse.Namespace) -> int:
    return _run_edit(args, "add")


def _run_edit(args: argparse.Namespace, verb: str) -> int:
    from . import _native

    doc = _read(args.file)
    plan_text, notes = _edit_plan(args, verb, doc.to_bytes())
    editing = bool(args.editing_mode)
    try:
        parsed = json.loads(plan_text)
    except ValueError:
        parsed = None  # the engine reports it as INVALID_PLAN below
    if editing and isinstance(parsed, dict) and parsed.get("existing_revisions") == "keep":
        print(f"error: {_EDITING_KEEPS}", file=sys.stderr)
        raise SystemExit(EXIT_USAGE)
    dry_run = bool(getattr(args, "dry_run", False))
    pdf, png = bool(getattr(args, "pdf", False)), bool(getattr(args, "png", False))
    out_dir: Path = args.out_dir if args.out_dir is not None else _default_out_dir(args.file)
    if not dry_run:
        if out_dir.exists() and not args.force:
            raise CliError(f"output directory '{out_dir}' already exists (use --force to replace its files)")
        if out_dir.resolve() == args.file.resolve().parent:
            raise CliError("--out-dir must not be the input's own directory")
    try:
        if dry_run:
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
    outputs: list[tuple[str, bytes]] = [("clean.docx", result.clean.to_bytes())]
    if not editing:
        outputs += [
            ("redline.docx", result.redline.to_bytes()),
            ("patch.diff", result.diff.text.encode("utf-8")),
        ]
    if pdf or png:
        options = PdfOptions(compress=True, revisions=args.revisions, revision_palette=args.revision_palette)
        pages: dict[str, int] = {}
        starts: dict[str, list[str]] = {}
        rendered_docs = (("clean", result.clean),) if editing else (("redline", result.redline), ("clean", result.clean))
        for name, document in rendered_docs:
            rendered = document.render(pdf=pdf, png_dpi=args.dpi if png else None, options=options)
            pages[name] = rendered.report.page_count
            starts[name] = [(p.text.splitlines() or [""])[0][:60] for p in rendered.report.pages]
            if rendered.pdf is not None:
                outputs.append((f"{name}.pdf", rendered.pdf))
            for i, image in enumerate(rendered.pngs):
                outputs.append((_png_name(name, i, len(rendered.pngs)), image))
        lines.append(json.dumps({"ev": "render", "engine": f"jubarte {__version__}", "pages": pages, "page_starts": starts}, ensure_ascii=False))

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
    names = [name for name, _ in outputs] + ["report.jsonl"]
    print(f"wrote {out_dir} ({len(names)} files: {', '.join(names)})")
    for note in notes:
        print(f"note: {note}")
    for outcome in result.report.operations:
        if outcome.anchor_given is not None and outcome.anchor_read_as is not None:
            given, read_as = json.dumps(outcome.anchor_given, ensure_ascii=False), json.dumps(outcome.anchor_read_as, ensure_ascii=False)
            print(f"note: {outcome.id}: anchor {given} read as {read_as} (Markdown marks are not document text)")
    shown = out_dir / ("clean.docx" if editing else "redline.docx")
    # The files are written and report.jsonl says so: a view that cannot be
    # read back is a warning, not a failed edit.
    try:
        view = _native.changed_view(result.redline.to_bytes(), result.report.author, accepted=editing, source=str(shown))
    except _native.JubarteError as exc:
        print(f"warning: the changed blocks cannot be shown: {exc}", file=sys.stderr)
        return EXIT_OK
    sys.stdout.write(view)
    return EXIT_OK


def _from_markdown(args: argparse.Namespace) -> Document:
    """``args.file`` (Markdown) written as Word, its warnings on stderr."""
    import warnings

    from .document import from_markdown

    try:
        text = args.file.read_text(encoding="utf-8")
        reference = None if args.reference_doc is None else _read(args.reference_doc).to_bytes()
    except OSError as exc:
        raise CliError(f"reading {exc.filename}: {exc.strerror}") from exc
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        doc = from_markdown(
            text,
            reference=reference,
            page=args.page,
            author=args.author,
            date=args.date,
            critic=not args.no_critic,
            track_changes=args.track_changes,
        )
    for warning in caught:
        print(f"warning: {warning.message}", file=sys.stderr)
    return doc


def cmd_convert(args: argparse.Namespace) -> int:
    if args.file.suffix.lower() in (".md", ".markdown"):
        doc = _from_markdown(args)
        # Markdown goes to Word unless a PDF or PNG is asked for.
        wants_render = args.pdf or args.png or (args.to != "docx" and args.output is not None and args.output.suffix.lower() != ".docx")
        if not wants_render:
            for given, flag in ((args.move_comments, "--move-comments"), (args.changed_only, "--changed-only")):
                if given:
                    raise CliError(f"{flag} applies to PDF or PNG output only")
            docx_out = args.output or args.file.with_suffix(".docx")
            _ensure_writable(docx_out, args.force)
            _write(docx_out, doc.to_bytes())
            print(f"wrote {docx_out} ({len(doc.to_bytes())} bytes)")
            return EXIT_OK
    else:
        doc = _read(args.file)
        if args.track_changes == "accept":
            doc = doc.accept()
        elif args.track_changes == "reject":
            doc = doc.reject()
    output: Path = args.output or args.file.with_suffix(".pdf")
    want_pdf = args.pdf or not args.png
    for side, what in ((args.font_report, "--font-report"), (args.report, "--report")):
        if side is not None:
            if side.resolve() in (output.resolve(), args.file.resolve()):
                raise CliError(f"{what} '{side}' is the same file as the PDF output or the input")
            _ensure_writable(side, args.force)
    selected = None if args.pages is None else _parse_pages(args.pages)
    if selected is not None and not args.png:
        raise CliError("--pages selects PNG pages; add --png")
    if want_pdf:
        _ensure_writable(output, args.force)
    rendered = doc.render(pdf=want_pdf, png_dpi=args.dpi if args.png else None, options=_pdf_options(args), pages=selected)
    pages = rendered.report.page_count
    # The page count is known only now; check every PNG path before the first
    # write so a refused page leaves no partial bundle.
    indices = range(len(rendered.pngs)) if selected is None else [p - 1 for p in selected]
    png_paths = [output.parent / _png_name(output.stem, i, pages) for i in indices]
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


def cmd_diff_render(args: argparse.Namespace) -> int:
    a, b = _read(args.a), _read(args.b)
    diff = diff_render(a, b, dpi=args.dpi, overlay=not args.no_overlay and args.out_dir is not None)
    changed = [p for p in diff.pages if p.differs]
    summary = json.dumps(
        {
            "a": str(args.a),
            "b": str(args.b),
            "dpi": args.dpi,
            "a_pages": diff.a_report.page_count,
            "b_pages": diff.b_report.page_count,
            "changed": len(changed),
            "pages": [{k: v for k, v in asdict(p).items() if k != "only_in" or v is not None} for p in diff.pages],
        }
    )
    if args.out_dir is not None:
        out: Path = args.out_dir
        count = len(diff.pages)
        files: list[tuple[Path, bytes | str]] = []
        for page in changed:
            i = page.index
            for prefix, png in (
                ("a", diff.a[i] if i < len(diff.a) else None),
                ("b", diff.b[i] if i < len(diff.b) else None),
                ("diff", diff.overlays[i]),
            ):
                if png is not None:
                    files.append((out / _png_name(prefix, i, count), png))
        files.append((out / "diff.json", summary))
        # Check every path before the first write so a refused file leaves no
        # partial bundle.
        for path, _ in files:
            _ensure_writable(path, args.force)
        out.mkdir(parents=True, exist_ok=True)
        for path, data in files:
            _write(path, data)
    if args.json:
        print(summary)
    else:
        for page in changed:
            if page.only_in is not None:
                print(f"page {page.index + 1}: only in {page.only_in}")
            else:
                print(f"page {page.index + 1}: {page.changed_ratio * 100:.2f}% of pixels changed, box {list(page.bbox or (0, 0, 0, 0))}")
        total = len(diff.pages)
        wrote = f" (wrote {args.out_dir})" if args.out_dir is not None else ""
        print(f"{len(changed)} of {total} page{'' if total == 1 else 's'} differ{wrote}")
    return EXIT_PAGES_DIFFER if changed else EXIT_OK


def _markdown_needs_both(old: object, new: object, old_path: Path, new_path: Path) -> None:
    """Markdown output is CriticMarkup of two Markdown documents, as in the native CLI."""
    if not isinstance(old, str) or not isinstance(new, str):
        word = old_path if not isinstance(old, str) else new_path
        raise CliError(f"Markdown output needs both documents in Markdown ({word} is Word): "
                       "write a Word redline (-o FILE.docx) or a PDF (-o FILE.pdf) instead")


def cmd_compare(args: argparse.Namespace) -> int:
    from .document import diff

    original = _read_side(args.original, args.old_format)
    modified = _read_side(args.modified, args.new_format)
    default = args.original.with_name(f"{args.original.stem}_v_{args.modified.stem}.docx")
    view = getattr(args, "view", None)
    if args.output is None and view is not None:
        # The shorthand `A B` without -o prints the redline's agent view.
        from . import _native

        redline = _native.redline_documents(original, modified, author=args.author, date=args.date)
        return _print_view(redline, view, f"{default} (not written; -o keeps it)")
    output: Path = args.output or default
    if args.output_format == "md":
        _markdown_needs_both(original, modified, args.original, args.modified)
    _ensure_writable(output, args.force)
    from . import _native

    redline = (diff(original, modified, format="critic", author=args.author, date=args.date).text
               if args.output_format == "md" else
               _native.redline_documents(original, modified, author=args.author, date=args.date))
    _write(output, redline)
    if not args.quiet:
        print(f"wrote {output} ({len(redline)} bytes)")
    return EXIT_OK


def cmd_diff(args: argparse.Namespace) -> int:
    from . import _native
    from .document import diff, _decode_diff, _default_author
    from datetime import datetime, timezone

    view = args.format in ("github", "word", "normal", "context", "side-by-side")
    output = args.output
    old = _read_side(args.old, args.old_format)
    new = _read_side(args.new, args.new_format)
    both_markdown = isinstance(old, str) and isinstance(new, str)
    to = args.to or args.output_format or ("md" if both_markdown else "docx")
    if not view and output is None and to != "md":
        output = args.old.with_name(f"{args.old.stem}_v_{args.new.stem}.{'pdf' if to == 'png' else to}")
    if not view and output is not None and to == "md":
        _markdown_needs_both(old, new, args.old, args.new)
    # A PNG output's path only names its pages, which are checked once counted.
    if output is not None and (view or to != "png"):
        _ensure_writable(output, args.force)
    # Unified snapshots are pure: no git author lookup or wall-clock default.
    author = args.author if args.author is not None else ("Redline" if view else _default_author())
    date = args.date if args.date is not None else ("" if view else datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
    if view:
        text = _native.diff_view(old, new, format=args.format, old_name=str(args.old), new_name=str(args.new),
                                 context=args.context, accept_changes=args.accept_changes, full_lines=args.full_lines)
        if output is not None:
            _write(output, text)
            print(f"wrote {output} ({len(text.encode('utf-8'))} bytes)", file=sys.stderr)
        else:
            sys.stdout.write(text)
        return EXIT_OK
    result = _decode_diff(_native.diff_json(old, new, old_name=args.old.name, new_name=args.new.name,
                                            author=author, date=date, columns=args.columns, critic=args.format == "critic"))
    data: str | bytes = result.text
    if output is not None:
        if to == "md":
            data = diff(old, new, format="critic", author=author, date=date).text
        elif to in ("docx", "pdf", "png"):
            document = Document.from_bytes(_native.redline_documents(old, new, author=author, date=date))
            if to == "docx":
                data = document.to_bytes()
            elif to == "pdf":
                data = document.to_pdf(options=_pdf_options(args))
            else:
                rendered = document.render(pdf=False, png_dpi=96, options=_pdf_options(args))
                png_paths = [output.parent / _png_name(output.stem, i, len(rendered.pngs)) for i in range(len(rendered.pngs))]
                for path in png_paths:
                    _ensure_writable(path, args.force)
                for path, png in zip(png_paths, rendered.pngs):
                    _write(path, png)
                _diff_done(args, f"wrote {len(png_paths)} PNG page{'' if len(png_paths) == 1 else 's'}", result.text)
                return EXIT_OK
    if output is not None:
        _write(output, data)
        _diff_done(args, f"wrote {output} ({len(data.encode() if isinstance(data, str) else data)} bytes)", result.text)
    else:
        sys.stdout.write(result.text)
    return EXIT_OK


def _diff_done(args: argparse.Namespace, wrote: str, patch: str) -> None:
    """Report a written diff output as the native CLI does: the patch on stdout
    with the status on stderr, or, for ``--format critic``, only the status."""
    if args.format == "critic":
        print(wrote)
    else:
        print(wrote, file=sys.stderr)
        sys.stdout.write(patch)


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


def cmd_comments(args: argparse.Namespace) -> int:
    comments = _read(args.file).comments(author=args.author, latest=args.latest)
    for comment in comments:
        row = {key: value for key, value in asdict(comment).items() if value is not None}
        if args.json:
            print(json.dumps(row, ensure_ascii=False))
            continue
        text = json.dumps(comment.text[:60], ensure_ascii=False)
        anchor = json.dumps(comment.anchor_text[:40], ensure_ascii=False)
        thread = f"\treply to {comment.parent}" if comment.parent is not None else ""
        done = "\tresolved" if comment.done else ""
        print(f"{comment.id}\t{comment.paragraph or '-'}\t{comment.author}\t{text}\ton {anchor}{thread}{done}")
    if not args.json:
        print(f"{len(comments)} comment(s)")
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


def _print_findings(findings: Sequence[Finding], as_json: bool) -> None:
    for finding in findings:
        row = asdict(finding)
        if as_json:
            print(json.dumps(row, ensure_ascii=False))
            continue
        star = "*" if row["word_fatal"] else " "
        print(f"{star} {row['code']}\t{row['part']}#{row['path']}\t{row['message']}")


def cmd_validate(args: argparse.Namespace) -> int:
    doc = _read(args.file)
    findings: list[Finding] = []
    if args.repair is not None:
        _ensure_writable(args.repair, args.force)
        repaired = doc.repair()
        _write(args.repair, repaired.document.to_bytes())
        findings.extend(repaired.remaining)
        if not args.json:
            print(f"repaired {len(repaired.repaired)} finding(s) into {args.repair}")
    else:
        findings.extend(doc.validate())
    if args.original is not None:
        findings.extend(doc.audit_tracked(_read(args.original), author=args.author))
    _print_findings(findings, args.json)
    if not args.json:
        fatal = sum(1 for f in findings if asdict(f)["word_fatal"])
        print(f"{len(findings)} finding(s), {fatal} Word-fatal" if findings else "no findings")
    return EXIT_FINDINGS if findings else EXIT_OK


def cmd_capabilities(_args: argparse.Namespace) -> int:
    print(json.dumps(capabilities(), indent=2))
    return EXIT_OK


# -- parser -------------------------------------------------------------------


# Handlers own host I/O. Rust clap owns the grammar, defaults and help.
_HANDLERS = {
    "inspect": cmd_inspect, "read": cmd_read, "edit": cmd_edit, "add": cmd_add,
    "convert": cmd_convert, "compare": cmd_compare, "diff": cmd_diff,
    "revisions": cmd_revisions, "changes": cmd_changes, "comments": cmd_comments,
    "accept": cmd_accept, "reject": cmd_reject, "diff-render": cmd_diff_render,
    "validate": cmd_validate, "capabilities": cmd_capabilities,
}
_PATH_ARGUMENTS = {"file", "original", "modified", "old", "new", "output", "plan", "out_dir", "reference_doc", "report", "font_report", "repair", "a", "b"}


class SharedParser:
    """Compatibility facade: argparse.Namespace and SystemExit over Rust clap."""

    supported = tuple(_HANDLERS)

    def error(self, message: str) -> None:
        print(f"error: {message}", file=sys.stderr)
        raise SystemExit(2)

    def parse_args(self, argv: Sequence[str] | None = None) -> argparse.Namespace:
        from . import _native

        arguments = list(sys.argv[1:] if argv is None else argv)
        parsed = json.loads(_native.parse_cli_json(arguments, program="jubarte-redlines", supported=list(self.supported)))
        if "text" in parsed:
            stream = sys.stderr if parsed["stream"] == "stderr" else sys.stdout
            stream.write(parsed["text"])
            raise SystemExit(parsed["exit_code"])
        command, values = parsed["command"], parsed["args"]
        self._validate_host(command, values)
        for key in _PATH_ARGUMENTS:
            if values.get(key) is not None:
                values[key] = Path(values[key])
        if command in ("accept", "reject"):
            for plural, singular in (("ids", "id"), ("authors", "author"), ("kinds", "kind")):
                values[singular] = values.pop(plural, []) or None
        return argparse.Namespace(command=command, func=_HANDLERS[command], **values)

    def _validate_host(self, command: str, values: dict) -> None:
        # Capability checks only, never an alternate option grammar.
        if command in ("compare", "diff"):
            if values.get("mode", "word") != "word" or values.get("powertools_faithful"):
                self.error("--mode powertools is not supported by the Python CLI")
            if values.get("detail_threshold") is not None:
                self.error("--detail-threshold is not supported by the Python CLI")
            if values.get("no_paragraph_merge"):
                self.error("--no-paragraph-merge is not supported by the Python CLI")
        if command == "inspect" and values.get("tables"):
            self.error("--tables is not supported by the Python CLI")
        if command in ("convert", "diff"):
            for key in (("from",) if command == "convert" else ()) + ("resource_path", "timeout", "fail_on_substitution", "no_page_markers"):
                if values.get(key) not in (None, False):
                    self.error(f"--{key.replace('_', '-')} is not supported by the Python CLI")
        if command == "diff":
            for key in ("reference_doc", "critic"):
                if values.get(key) not in (None, False):
                    self.error(f"--{key.replace('_', '-')} is not supported by the Python diff CLI")
            context = values.get("context", 3)
            if type(context) is not int or not 0 <= context <= 2**32 - 1:
                self.error("--context must be in the u32 range (0..4294967295)")
        if command == "convert":
            to = values.get("to")
            extension = Path(values.get("output") or "").suffix.lower()
            if to == "md" or (to is None and extension in (".md", ".markdown", ".txt", ".mdown", ".mkd", ".mkdn")):
                self.error("--to md with page markers is not supported by the Python CLI; `read FILE` prints the agent text view")
            if (to == "docx" or (to is None and extension == ".docx")) and Path(values["file"]).suffix.lower() not in (".md", ".markdown"):
                self.error("--to docx requires Markdown input in the Python CLI")
            if to in ("pdf", "png"):
                values[to] = True
            elif to is None and extension == ".png":
                values["png"] = True


def build_parser() -> SharedParser:
    """Return the shared clap parser with the legacy parse_args interface."""
    return SharedParser()


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
