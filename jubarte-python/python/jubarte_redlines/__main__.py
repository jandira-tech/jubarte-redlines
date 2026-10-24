# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""``jubarte-redlines`` (or ``python -m jubarte_redlines``): the ``jubarte``
binary's commands over the installed wheel, with the same names, flags, output
files and exit codes. ``uvx jubarte-redlines`` runs it without installing.

    uvx jubarte-redlines redline a.docx b.docx -o redline.docx
    python -m jubarte_redlines read letter.docx --json
    python -m jubarte_redlines read letter.docx
    python -m jubarte_redlines edit letter.docx --plan plan.json --out-dir review --pdf --png
    python -m jubarte_redlines convert letter.docx -t png --dpi 150 --report pages.json
    python -m jubarte_redlines convert letter.docx -t png --pages 3-5
    python -m jubarte_redlines visual-diff before.docx after.docx --out-dir diff
    python -m jubarte_redlines convert draft.md --page a4 -o draft.docx
    python -m jubarte_redlines compare a.docx b.docx -o redline.docx --author Legal
    python -m jubarte_redlines accept redline.docx -o clean.docx
    python -m jubarte_redlines changes redline.docx --json
    python -m jubarte_redlines reject redline.docx -o out.docx --id r12
    python -m jubarte_redlines capabilities --json
    python -m jubarte_redlines validate redline.docx --original a.docx --author Legal

Exit codes: 0 success, 1 error (I/O, engine, existing output), 2 usage, 3 edit
plan refused (its per-operation report is on stdout; nothing was written), 5
``visual-diff`` found a page that differs. ``validate`` exits 2 when it has
findings.
"""

from __future__ import annotations

import argparse
import json
import os
import stat
import sys
from dataclasses import asdict
from collections.abc import Sequence
from pathlib import Path
from typing import NoReturn

from . import __version__, capabilities
from .document import Document, EditPlanError, EditResult, diff_render
from .models import PdfOptions, RevisionStyle

EXIT_OK = 0
EXIT_ERROR = 1
EXIT_USAGE = 2
EXIT_PLAN_REFUSED = 3
EXIT_PAGES_DIFFER = 5
EXIT_FINDINGS = 2

# Whether the command line asks for JSON (`_native.wants_json`): an error
# then prints its JSON line on stdout too, as the binary's does (#1255).
_JSON_ERRORS = False


def _error(message: str, exit: int) -> None:
    """``error: MESSAGE`` on stderr, and under ``--json`` first its
    ``{"error":…}`` line on stdout: what the binary says for an error that
    exits with ``exit``."""
    if _JSON_ERRORS:
        from . import _native

        sys.stdout.write(_native.runtime_json_error(message, exit))
        sys.stdout.flush()
    print(f"error: {message}", file=sys.stderr)


class CliError(Exception):
    """A user-facing failure; the message is printed as ``error: ...``."""


def _os(exc: OSError) -> str:
    """An I/O error as Rust words it: ``No such file or directory (os error 2)``."""
    return f"{exc.strerror} (os error {exc.errno})" if exc.errno is not None and exc.strerror else str(exc)


def _read(path: Path) -> Document:
    try:
        doc = Document.read(path)
    except OSError as exc:
        raise CliError(f"reading {path}: {_os(exc)}") from exc
    from . import _native

    refused = _native.refused_input(path, doc.to_bytes(), True)
    if refused is not None:
        raise CliError(refused)
    return doc


def _read_side(path: Path, force_kind: str | None) -> bytes | str:
    """CLI format metadata wins; unknown suffixes use the core ZIP sniff rule."""
    try:
        data = path.read_bytes()
    except OSError as exc:
        raise CliError(f"reading {path}: {_os(exc)}") from exc
    from . import _native

    kind = force_kind or ("docx" if data.startswith(b"PK\x03\x04") else "md")
    refused = _native.refused_input(path, data, kind == "docx")
    if refused is not None:
        raise CliError(refused)
    if kind == "docx":
        return data
    try:
        return data.decode("utf-8-sig")
    except UnicodeError as exc:
        raise CliError(f"reading {path}: invalid UTF-8 Markdown: {exc}") from exc


def _named_kind(path: Path) -> str | None:
    """The format a file's name declares, asked of the shared parser; None
    when only its bytes can tell."""
    from . import _native

    parsed = json.loads(
        _native.parse_cli_json(["diff", "--", str(path), str(path)], program="jubarte-redlines", supported=["diff"])
    )
    return parsed["args"]["old_format"]


def _ensure_writable(path: Path, force: bool) -> None:
    if path.exists() and not force:
        raise CliError(f"output '{path}' already exists (use --force to overwrite)")


def _same_file(a: Path, b: Path) -> bool:
    """Whether two CLI paths name the same file. Two existing paths are
    compared by identity, which sees through hard links, symlinks and a
    volume that ignores case; a path not written yet by its resolved form."""
    try:
        return os.path.samefile(a, b)
    except OSError:
        return a.resolve() == b.resolve()


def _check_side_file(side: Path, flag: str, others: tuple[tuple[Path, str], ...], force: bool) -> None:
    """A side file (``--report``, ``--font-report``) is written apart from the
    output, after the input is read, so sharing a path with either would
    silently replace one with the other; ``--force`` never allows that."""
    for other, name in others:
        if _same_file(side, other):
            raise CliError(f"{flag} '{side}' is the same file as the {name}")
    _ensure_writable(side, force)


def _write(path: Path, data: bytes | str) -> int:
    """Writes ``data`` (text as UTF-8, newlines as given); returns the bytes written."""
    raw = data.encode("utf-8") if isinstance(data, str) else data
    try:
        path.write_bytes(raw)
    except OSError as exc:
        raise CliError(f"writing {path}: {_os(exc)}") from exc
    return len(raw)


def _replace(path: Path, data: bytes | str) -> int:
    """Writes ``data`` beside ``path`` and renames it over, as the binary
    publishes: an existing output — a hard or symbolic link among them —
    has its own directory entry replaced, never its target written
    through; a replaced regular file keeps its mode. Returns the bytes
    written."""
    raw = data.encode("utf-8") if isinstance(data, str) else data
    staged = path.with_name(f".{path.name}.jubarte-{os.getpid()}.tmp")
    try:
        with open(staged, "wb") as handle:
            handle.write(raw)
        try:
            replaced = os.stat(path, follow_symlinks=False).st_mode
        except OSError:
            replaced = 0
        if stat.S_ISREG(replaced):
            os.chmod(staged, replaced & 0o777)
        os.replace(staged, path)
    except OSError as exc:
        try:
            os.unlink(staged)
        except OSError:
            pass
        raise CliError(f"writing {path}: {_os(exc)}") from exc
    return len(raw)


def _refuse_alias_output(args: argparse.Namespace) -> None:
    """Refuses a ``read --tables -o`` that names its input — the same
    path, or the same file through a hard or symbolic link — before any
    work: ``--force`` authorizes replacing an output, never an input
    alias (#1698). Any other existing output is the binary's
    ``OUTPUT_EXISTS`` usage refusal."""
    output, file = args.output, args.file
    if _same_file(output, file):
        _error(
            f"OUTPUT_ALIAS: the output '{_spelled(args, 'output')}' is the same file as the input '{_spelled(args, 'file')}'",
            EXIT_USAGE,
        )
        raise SystemExit(EXIT_USAGE)
    if output.exists() and not args.force:
        _error(
            f"OUTPUT_EXISTS: output '{_spelled(args, 'output')}' already exists (use --force to overwrite)",
            EXIT_USAGE,
        )
        raise SystemExit(EXIT_USAGE)


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
        changed_only=getattr(args, "changed_pages", False),
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


def _print_view(docx: bytes, options: dict, source: str) -> int:
    """The agent view of ``docx`` with the read options ``options``."""
    sys.stdout.write(_view_text(docx, options, source))
    return EXIT_OK


def _view_text(docx: bytes, options: dict, source: str) -> str:
    """:func:`_print_view`'s Markdown, its warnings said on stderr."""
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
    return markdown


def _selection(options: dict) -> dict:
    """The read options that select paragraphs, and the text
    ``--track-changes`` settles, as ``read_json`` takes them."""
    return {
        "track_changes": options.get("track_changes"),
        "paragraphs": options.get("paragraphs"),
        "head": options.get("head"),
        "tail": options.get("tail"),
        "changed": bool(options.get("changed")),
        "by": options.get("by"),
    }


def _write_result(
    args: argparse.Namespace,
    data: bytes,
    *,
    source: str,
    view: dict | None = None,
    validity_file: str | None = None,
) -> None:
    """``-o`` in the format its extension names, as the binary's writer: the
    Word package, its agent view (.md, headed ``source``), its snapshot
    (.json) or its drawing (.pdf, .png). ``view`` is read's options; the
    defaults elsewhere. ``validity_file`` puts the input's validity in a
    snapshot's summary, as ``read --json`` prints it."""
    from . import _native

    output: Path = args.output
    kind = args.output_format or "docx"
    view = view or {}
    if kind in ("pdf", "png"):
        _render_to(args, Document(data, output.name), output)
        return
    _ensure_writable(output, args.force)
    if kind == "md":
        text: bytes | str = _view_text(data, view, source)
    elif kind == "json":
        text = _native.read_json(data, **_selection(view), validity_file=validity_file) + "\n"
    else:
        text = data
    _wrote(args, output, kind, _write(output, text))


def _not_written(args: argparse.Namespace) -> str:
    """The name a task's .md view gives the Word file it did not write: -o
    as typed, with .docx for its extension."""
    stem, _ = os.path.splitext(_spelled(args, "output"))
    return f"{stem}.docx (not written)"


def _for_edit_json(args: argparse.Namespace, docx: bytes) -> str | None:
    """``read --for-edit``'s JSON, from the flags the shared parser stored;
    ``None`` after its refusal was printed (exit 3)."""
    from . import _native

    try:
        return _native.for_edit_json(
            docx,
            paragraphs=args.paragraphs,
            head=args.head,
            tail=args.tail,
            find=args.find,
            regex=args.regex,
            ignore_case=bool(args.ignore_case),
            changed=bool(args.changed),
            by=args.by,
            context=args.context,
            limit=args.limit,
            max_chars=args.max_chars,
            all=bool(args.all),
            cursor=args.cursor,
            accept_change=args.accept_change or None,
            reject_change=args.reject_change or None,
            existing_revisions=args.existing_revisions,
            track=args.track,
            validity_file=_spelled(args, "file"),
        )
    except _native.JubarteError as exc:
        message = str(exc)
        usage = message.removeprefix("USAGE: ")
        if usage != message:
            _error(usage, EXIT_USAGE)
            raise SystemExit(EXIT_USAGE) from exc
        _error(message, EXIT_PLAN_REFUSED)
        return None


def _usage_exit(exc: Exception) -> NoReturn:
    """Exit 2 with the binary's usage wording for an engine refusal raised
    with its ``USAGE`` category (a selection the flags do not spell); any
    other error is raised unchanged, keeping the ordinary exit 1."""
    message = str(exc)
    stripped = message.removeprefix("USAGE: ")
    if stripped != message:
        _error(stripped, EXIT_USAGE)
        raise SystemExit(EXIT_USAGE) from exc
    raise exc


def cmd_read(args: argparse.Namespace) -> int:
    from . import _native

    try:
        return _read_task(args)
    except _native.JubarteError as exc:
        # A selection refusal carries its USAGE category; the binary exits
        # 2 for it, in the view, snapshot and tables paths alike.
        _usage_exit(exc)


def _read_task(args: argparse.Namespace) -> int:
    from . import _native

    if args.for_edit:
        # A JSON read: the result carries the input's findings; a file
        # leaves stdout free, so the notice stays, as at `read -o FILE.json`.
        result = _for_edit_json(args, _input(args, notice=args.output is not None))
        if result is None:
            return EXIT_PLAN_REFUSED
        if args.output is not None:
            _ensure_writable(args.output, args.force)
            _wrote(args, args.output, "json", _write(args.output, result))
            return EXIT_OK
        sys.stdout.write(result)
        return EXIT_OK
    if args.tables:
        # With -p, the grids of the tables it reaches, then its view; a
        # pick that reaches no table is a usage error. -o FILE writes them,
        # never over the input itself, and only over an existing file with
        # --force (#1698).
        if args.output is not None:
            _refuse_alias_output(args)
        markdown, warnings = _native.tables_view(
            _input(args),
            track_changes=args.track_changes,
            comments=args.comments == "inline",
            dates=bool(args.dates),
            page_markers=not args.no_page_markers,
            paragraphs=args.paragraphs,
            source=args.file.name,
        )
        for warning in warnings:
            print(f"warning: {warning}", file=sys.stderr)
        if args.output is not None:
            size = _replace(args.output, markdown)
            if not args.quiet:
                # The binary's line for a text output with no format of its own.
                sys.stdout.flush()
                print(f"wrote {_spelled(args, 'output')} (tables, {size} bytes)", file=sys.stderr)
            return EXIT_OK
        sys.stdout.write(markdown)
        return EXIT_OK
    if args.output is not None:
        _write_result(
            args,
            _input(args),
            source=args.file.name,
            view=vars(args),
            validity_file=_spelled(args, "file"),
        )
        return EXIT_OK
    if args.json:
        snapshot = _native.read_json(
            _input(args, notice=False),
            **_selection(vars(args)),
            source=args.file.name,
            validity_file=_spelled(args, "file"),
        )
        print(snapshot)
        return EXIT_OK
    return _print_view(_input(args), vars(args), args.file.name)


_DOCX_NEEDS_MARKDOWN = "--to docx requires Markdown input in the Python CLI"


def _edit_plan(args: argparse.Namespace, verb: str, source: bytes) -> tuple[str, list[str]]:
    """``--plan``'s text, or the plan the operation flags describe, and its
    notes; ``--rewrite-others-comments`` sets the plan's field."""
    plan, notes = _edit_plan_as_given(args, verb, source)
    if getattr(args, "rewrite_others_comments", False):
        try:
            parsed = json.loads(plan)
        except ValueError:
            return plan, notes  # reported as INVALID_PLAN when applied
        if isinstance(parsed, dict):
            parsed["rewrite_others_comments"] = True
            plan = json.dumps(parsed, ensure_ascii=False)
    return plan, notes


def _edit_plan_as_given(args: argparse.Namespace, verb: str, source: bytes) -> tuple[str, list[str]]:
    from . import _native

    if getattr(args, "plan", None) is not None:
        text = _read_plan_text(_spelled(args, "plan"))
        if text is None:
            raise CliError(f"reading {_spelled(args, 'plan')}: No such file or directory")
        return text, []
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
        _error(str(exc), EXIT_USAGE)
        raise SystemExit(EXIT_USAGE) from exc
    return plan, list(notes)


def cmd_edit(args: argparse.Namespace) -> int:
    return _run_edit(args, "edit")


def cmd_add(args: argparse.Namespace) -> int:
    return _run_edit(args, "add")


def _print_refusal(exc: EditPlanError) -> None:
    """A refused plan's per-operation report on stdout, the error on stderr."""
    for i, o in enumerate(exc.outcomes, start=1):
        row: dict[str, object] = {
            "ev": "op",
            "i": i,
            "id": o.id,
            "op": o.kind,
            "status": o.status,
            "matches": o.matches,
        }
        row.update(
            {
                key: value
                for key, value in (("at", o.paragraph), ("ctx", o.context), ("code", o.code), ("message", o.message))
                if value is not None
            }
        )
        print(_jsonl(row))
    print(
        _jsonl(
            {"ev": "summary", "status": "failed", "code": exc.code, "operation": exc.operation, "message": exc.message}
        )
    )
    sys.stdout.flush()
    _error(f"plan refused: {exc}", EXIT_PLAN_REFUSED)


def _jsonl(row: dict) -> str:
    """One JSON line as the binary prints it: compact, keys sorted."""
    return json.dumps(row, ensure_ascii=False, separators=(",", ":"), sort_keys=True)


def _spelled(args: argparse.Namespace, key: str) -> str:
    """FILE, --plan, OLD, NEW or -o as the command line spells it."""
    return getattr(args, "spelled", {}).get(key) or str(getattr(args, key))


def _input(args: argparse.Namespace, *, notice: bool = True) -> bytes:
    """FILE's bytes for ``read``, ``changes`` and ``comments``; unless ``-q``,
    one stderr line says how many validity findings the input has. A JSON
    read (``notice=False``) reports them in its output instead (#1309)."""
    from . import _native

    docx = _read(args.file).to_bytes()
    if notice and not getattr(args, "quiet", False):
        message = _native.input_notice(docx, _spelled(args, "file"))
        if message is not None:
            print(message, file=sys.stderr)
    return docx


def _wrote(args: argparse.Namespace, path: Path, format: str, size: int, pages: int | None = None) -> None:
    """The engine's ``wrote`` line on stderr, unless ``-q``; ``-o`` as typed."""
    from . import _native

    if getattr(args, "quiet", False):
        return
    shown = _spelled(args, "output") if getattr(args, "output", None) == path else str(path)
    sys.stdout.flush()  # stdout's lines first when both streams share a pipe
    print(_native.wrote_line(shown, format, size, pages), file=sys.stderr)


def _read_plan_text(plan: str) -> str | None:
    """PLAN.json's text, strictly UTF-8 as the binary reads it; ``None``
    when there is no such file."""
    try:
        with open(plan, "rb") as f:
            data = f.read()
    except FileNotFoundError:
        return None
    except OSError as exc:
        raise CliError(f"reading {plan}: {exc.strerror or exc}") from exc
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise CliError(f"reading {plan}: stream did not contain valid UTF-8") from exc


def _record(args: argparse.Namespace, verb: str, doc: Document) -> int:
    """``--plan PLAN.json`` with operation flags: record them into the plan,
    checked against FILE first; a refusal leaves PLAN.json as it was. The
    shared parser has refused the flags that do not go with ``--plan``."""
    from . import _native

    plan = _spelled(args, "plan")
    held = _read_plan_text(plan)
    dry_run = bool(getattr(args, "dry_run", False))
    options = {
        "author": args.author,
        "date": args.datetime,
        "existingRevisions": args.existing_revisions,
        "file": _spelled(args, "file"),
        "plan": plan,
        "defaultPlan": bool(getattr(args, "plan_default", False)),
        "dryRun": dry_run,
        "rewriteOthersComments": bool(getattr(args, "rewrite_others_comments", False)),
    }
    try:
        recorded = json.loads(
            _native.record_plan(
                verb, json.dumps(args.operations, ensure_ascii=False), doc.to_bytes(), json.dumps(options), held=held
            )
        )
    except _native.JubarteError as exc:
        _error(f"{plan}: {exc}", EXIT_PLAN_REFUSED)
        return EXIT_PLAN_REFUSED
    except ValueError as exc:
        _error(str(exc), EXIT_USAGE)
        raise SystemExit(EXIT_USAGE) from exc
    try:
        doc.preview(recorded["plan"])
    except EditPlanError as exc:
        _print_refusal(exc)
        return EXIT_PLAN_REFUSED
    if not args.quiet:
        for note in recorded["notes"]:
            print(f"note: {note}")
    if not dry_run:
        # Through a symbolic link to the file it names, written yet or not;
        # via a hidden partial file named for this process, removed if the
        # write fails; bytes as they are, so no platform newline translation.
        target = Path(os.path.realpath(plan))
        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
        try:
            partial.write_bytes(recorded["json"].encode("utf-8"))
            os.replace(partial, target)
        except OSError as exc:
            partial.unlink(missing_ok=True)
            raise CliError(f"writing {plan}: {exc}") from exc
    if not args.quiet:
        print(recorded["summary"])
    return EXIT_OK


def _run_edit(args: argparse.Namespace, verb: str) -> int:
    doc = _read(args.file)
    if getattr(args, "plan", None) is not None and args.operations:
        return _record(args, verb, doc)
    # What --consume deletes once the edit is written: the plan file as it
    # was read, never a link to it or an output written over it.
    consumed: bytes | None = None
    plan = _spelled(args, "plan") if getattr(args, "plan", None) is not None else ""
    if getattr(args, "consume", False) and not getattr(args, "dry_run", False):
        if os.path.islink(plan):
            _error(f"--consume deletes {plan}, a symbolic link; pass the file it names", EXIT_USAGE)
            raise SystemExit(EXIT_USAGE)
        try:
            with open(plan, "rb") as f:
                consumed = f.read()
        except OSError as exc:
            raise CliError(f"reading {plan}: {exc.strerror or exc}") from exc
    code = _apply_edit(args, verb, doc)
    if code == EXIT_OK and consumed is not None:
        try:
            with open(plan, "rb") as f:
                current: bytes | None = f.read()
        except OSError:
            current = None
        if current != consumed:
            raise CliError(
                f"the plan was applied, but {plan} was not deleted: it no longer holds the plan (an output was written over it?)"
            )
        try:
            os.remove(plan)
        except OSError as exc:
            raise CliError(f"the plan was applied, but {plan} was not deleted: {exc}") from exc
    return code


def _apply_edit(args: argparse.Namespace, verb: str, doc: Document) -> int:
    from . import _native

    plan_text, notes = _edit_plan(args, verb, doc.to_bytes())
    editing = bool(args.editing_mode)
    # Changes the plan itself accepts or rejects (-p r12 --accept) do not
    # count; a plan that does not parse is reported as INVALID_PLAN below.
    conflict = _native.editing_mode_conflict(plan_text, doc.to_bytes()) if editing else None
    if conflict:
        _error(str(conflict), EXIT_USAGE)
        raise SystemExit(EXIT_USAGE)
    dry_run = bool(getattr(args, "dry_run", False))
    pdf, png = bool(getattr(args, "pdf", False)), bool(getattr(args, "png", False))
    try:
        if dry_run:
            sys.stdout.write(doc.preview(plan_text).to_jsonl())
            return EXIT_OK
        # The plan first: a refused plan prints its whole report, whatever
        # the outputs (D15).
        result = doc.edit(plan_text)
    except EditPlanError as exc:
        _print_refusal(exc)
        return EXIT_PLAN_REFUSED
    # The shared parser resolved the target: ``output`` for the one
    # document (FILE_modified.docx by default), ``out_dir`` for the bundle.
    out_dir: Path | None = args.out_dir
    source_bytes = doc.to_bytes()
    parsed = json.loads(plan_text)
    if out_dir is None:
        return _write_edited(args, args.output, result, notes, source_bytes, parsed)
    if out_dir.exists() and not args.force:
        raise CliError(f"output directory '{out_dir}' already exists (use --force to replace its files)")
    if out_dir.resolve() == args.file.resolve().parent:
        raise CliError("--out-dir must not be the input's own directory")
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
        rendered_docs = (
            (("clean", result.clean),) if editing else (("redline", result.redline), ("clean", result.clean))
        )
        for name, document in rendered_docs:
            rendered = document.render(pdf=pdf, png_dpi=args.dpi if png else None, options=options)
            pages[name] = rendered.report.page_count
            starts[name] = [(p.text.splitlines() or [""])[0][:60] for p in rendered.report.pages]
            if rendered.pdf is not None:
                outputs.append((f"{name}.pdf", rendered.pdf))
            for i, image in enumerate(rendered.pngs):
                outputs.append((_png_name(name, i, len(rendered.pngs)), image))
        lines.append(
            _jsonl({"ev": "render", "engine": f"jubarte {__version__}", "pages": pages, "page_starts": starts})
        )

    out_dir.mkdir(parents=True, exist_ok=True)
    saved = []
    for name, data in outputs:
        _write(out_dir / name, data)
        saved.append({"f": name, "bytes": len(data), "sha256": _native.source_sha256(data)})
    lines.append(_jsonl({"ev": "save", "dir": str(out_dir), "outputs": saved}))
    lines.append(summary)
    _write(out_dir / "report.jsonl", "\n".join(lines) + "\n")
    if args.quiet:
        return EXIT_OK
    # A pure accept/reject prints the resolved lines on stdout, nothing
    # else; the bundle's report.jsonl still holds the summary.
    if _is_pure_resolution(parsed, result):
        try:
            resolved = result.report.resolved_revisions
            text = _native.resolution_text(
                source_bytes,
                json.dumps(
                    {"accepted": list(resolved.accepted), "rejected": list(resolved.rejected)}, ensure_ascii=False
                ),
            )
            names = [name for name, _ in outputs] + ["report.jsonl"]
            sys.stdout.flush()
            print(f"wrote {out_dir} ({len(names)} files: {', '.join(names)})", file=sys.stderr)
            for note in _edit_notes(notes, result):
                print(note, file=sys.stderr)
            sys.stdout.write(text)
            return EXIT_OK
        except Exception:
            pass
    print(summary)
    # The status lines on stderr (D6); the summary and the view are data.
    names = [name for name, _ in outputs] + ["report.jsonl"]
    sys.stdout.flush()
    print(f"wrote {out_dir} ({len(names)} files: {', '.join(names)})", file=sys.stderr)
    for note in _edit_notes(notes, result):
        print(note, file=sys.stderr)
    shown = out_dir / ("clean.docx" if editing else "redline.docx")
    _print_resolution_or_view(source_bytes, parsed, result, editing, shown)
    return EXIT_OK


def _edit_notes(notes: list[str], result: EditResult) -> list[str]:
    """The ``note:`` lines of an applied edit: the flags' notes, the plan's
    (a field read as another), then each anchor read without its Markdown
    marks."""
    lines = [f"note: {note}" for note in (*notes, *result.report.notes)]
    for outcome in result.report.operations:
        if outcome.anchor_given is not None and outcome.anchor_read_as is not None:
            given, read_as = (
                json.dumps(outcome.anchor_given, ensure_ascii=False),
                json.dumps(outcome.anchor_read_as, ensure_ascii=False),
            )
            lines.append(
                f"note: {outcome.id}: anchor {given} read as {read_as} (Markdown marks are not document text)"
            )
    return lines


def _is_pure_resolution(parsed: object, result: EditResult) -> bool:
    """A pure ``--accept``/``--reject`` with no content operation prints the
    resolved lines, not the view; ordinary edits keep the view."""
    if not isinstance(parsed, dict):
        return False
    if parsed.get("operations"):
        return False
    if not parsed.get("resolve_revisions"):
        return False
    resolved = result.report.resolved_revisions
    return bool(resolved.accepted or resolved.rejected)


def _print_resolution_or_view(source: bytes, parsed: object, result: EditResult, editing: bool, shown: Path) -> None:
    """The resolved lines for a pure accept/reject, or the changed view
    otherwise. Failures fall back to the view, as the binary does."""
    from . import _native

    if _is_pure_resolution(parsed, result):
        try:
            resolved = result.report.resolved_revisions
            text = _native.resolution_text(
                source,
                json.dumps(
                    {"accepted": list(resolved.accepted), "rejected": list(resolved.rejected)}, ensure_ascii=False
                ),
            )
            sys.stdout.write(text)
            return
        except Exception:
            pass
    _print_changed_view(result, editing, shown)


def _print_changed_view(result: EditResult, editing: bool, shown: Path) -> None:
    """The changed blocks' view of the written ``shown``. The files are
    written: a view that cannot be read back is a warning, not a failed edit."""
    from . import _native

    try:
        view = _native.changed_view(
            result.redline.to_bytes(), result.report.author, accepted=editing, source=str(shown)
        )
    except _native.JubarteError as exc:
        print(f"warning: the changed blocks cannot be shown: {exc}", file=sys.stderr)
        return
    sys.stdout.write(view)


def _write_edited(
    args: argparse.Namespace,
    path: Path,
    result: EditResult,
    notes: list[str],
    source: bytes | None = None,
    parsed: object = None,
) -> int:
    """``edit`` and ``add`` without --out-dir, as the binary: the redline (in
    editing mode, the result) as one .docx at ``path``, its PDF and PNG pages
    beside it, the ``wrote`` lines and notes on stderr, the changed blocks'
    view of that file on stdout (a pure accept/reject prints the resolved
    lines)."""
    editing = bool(args.editing_mode)
    chosen = result.clean if editing else result.redline
    document = chosen.to_bytes()
    outputs: list[tuple[Path, bytes]] = [(path, document)]
    pdf, png = bool(getattr(args, "pdf", False)), bool(getattr(args, "png", False))
    # FILE itself (X_modified.docx edited again, or -o FILE) exists, so it is
    # replaced only with --force (D15). The PNG names wait for the page count.
    for taken in [path] + ([path.with_suffix(".pdf")] if pdf else []):
        if taken.exists() and not args.force:
            _error(f"output '{taken}' already exists (use --force to overwrite)", EXIT_USAGE)
            raise SystemExit(EXIT_USAGE)
    pages, png_count = None, 0
    if pdf or png:
        # Render before writing, so a failed render writes nothing.
        options = PdfOptions(compress=True, revisions=args.revisions, revision_palette=args.revision_palette)
        rendered = chosen.render(pdf=pdf, png_dpi=args.dpi if png else None, options=options)
        if rendered.pdf is not None:
            outputs.append((path.with_suffix(".pdf"), rendered.pdf))
            pages = rendered.report.page_count
        png_count = len(rendered.pngs)
        for i, image in enumerate(rendered.pngs):
            outputs.append((path.with_name(_png_name(path.stem, i, png_count)), image))
    # The PNG names are known only now: nothing is written over a file.
    for taken, _ in outputs:
        if taken.exists() and not args.force:
            _error(f"output '{taken}' already exists (use --force to overwrite)", EXIT_USAGE)
            raise SystemExit(EXIT_USAGE)
    for output, data in outputs:
        _write(output, data)
    if args.quiet:
        return EXIT_OK
    print(f"wrote {path} (docx, {len(document)} bytes)", file=sys.stderr)
    if pages is not None:
        pdf_path, pdf_bytes = outputs[1]
        print(
            f"wrote {pdf_path} (pdf, {len(pdf_bytes)} bytes, {pages} page{'' if pages == 1 else 's'})", file=sys.stderr
        )
    if png_count:
        pattern = path.with_name(f"{path.stem}-page-NN.png")
        print(
            f"wrote {pattern} (png, {png_count} page{'' if png_count == 1 else 's'}, {args.dpi:g} dpi)",
            file=sys.stderr,
        )
    for note in _edit_notes(notes, result):
        print(note, file=sys.stderr)
    if source is not None:
        _print_resolution_or_view(source, parsed, result, editing, path)
    else:
        _print_changed_view(result, editing, path)
    return EXIT_OK


def _from_markdown(args: argparse.Namespace, text: str) -> Document:
    """``text``, read from ``args.file``, written as Word; its warnings on stderr."""
    import warnings

    from .document import from_markdown

    reference = None if args.reference_doc is None else _read(args.reference_doc).to_bytes()
    # What names no one: the flags, else the front matter, else the
    # reference's creator and creation time, else Redline and now.
    author, date = args.author, args.datetime
    if author is None or date is None:
        from datetime import datetime, timezone

        from . import _native

        stamp_author, stamp_date = _native.markdown_stamp(text, reference)
        author = author or stamp_author or "Redline"
        date = date or stamp_date or datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        doc = from_markdown(
            text,
            reference=reference,
            page=args.page,
            author=author,
            date=date,
            critic=not args.no_critic,
            track_changes=args.track_changes,
            tracking=None if args.tracking is None else args.tracking == "on",
            scholar=args.scholar,
        )
    for warning in caught:
        print(f"warning: {warning.message}", file=sys.stderr)
    return doc


def cmd_convert(args: argparse.Namespace) -> int:
    # The native rule: the name's format, else the bytes (a ZIP is Word).
    source = _read_side(args.file, _named_kind(args.file))
    if isinstance(source, str):
        if args.update_fields:
            raise CliError("--update-fields needs a Word document in")
        doc = _from_markdown(args, source)
        # Markdown goes to Word unless a PDF, PNG or snapshot is asked for.
        if args.output_format not in ("pdf", "png", "json"):
            for given, flag in (
                (args.move_comments, "--move-comments"),
                (args.changed_pages, "--changed-pages"),
                (args.pages is not None, "--pages"),
            ):
                if given:
                    raise CliError(f"{flag} applies to PDF or PNG output only")
            docx_out = args.output or args.file.with_suffix(".docx")
            _ensure_writable(docx_out, args.force)
            _wrote(args, docx_out, "docx", _write(docx_out, doc.to_bytes()))
            return EXIT_OK
    else:
        if args.output_format == "docx" and not args.update_fields:
            # A name that said nothing; its bytes are Word.
            _error(_DOCX_NEEDS_MARKDOWN, EXIT_USAGE)
            raise SystemExit(EXIT_USAGE)
        doc = Document(source, args.file.name)
        if args.track_changes == "accept":
            doc = doc.accept()
        elif args.track_changes == "reject":
            doc = doc.reject()
        if args.update_fields:
            return _write_updated_fields(args, doc)
    if args.output_format == "json":
        _write_result(args, doc.to_bytes(), source=args.file.name)
        return EXIT_OK
    _render_to(args, doc, args.output or args.file.with_suffix(".pdf"))
    return EXIT_OK


def _render_to(args: argparse.Namespace, doc: Document, output: Path) -> None:
    """Draw ``doc`` as ``output`` (a PDF), or as PNG pages named after it
    when ``-o`` names a .png; the render flags are the task's own."""
    png = args.output_format == "png"
    want_pdf = not png
    for side, flag in (
        (getattr(args, "font_report", None), "--font-report"),
        (getattr(args, "report", None), "--report"),
    ):
        if side is not None:
            named = "PNG output" if png else "PDF output"
            _check_side_file(side, flag, ((output, named), (args.file, "input")), args.force)
    selected = None if args.pages is None else _parse_pages(args.pages)
    if want_pdf:
        _ensure_writable(output, args.force)
    rendered = doc.render(pdf=want_pdf, png_dpi=args.dpi if png else None, options=_pdf_options(args), pages=selected)
    pages = rendered.report.page_count
    # Each PNG is named by its document page, padded to the document's page
    # count. The pages are known only now; check every PNG path before the
    # first write so a refused page leaves no partial bundle.
    numbered = rendered.report.pages[: len(rendered.pngs)]
    png_paths = [
        output.parent / _png_name(output.stem, p.number - 1, rendered.report.document_pages) for p in numbered
    ]
    for path in png_paths:
        _ensure_writable(path, args.force)
        # `-o r.png --report r-page-01.png` would write the page, then the
        # report over it.
        for side, flag in (
            (getattr(args, "font_report", None), "--font-report"),
            (getattr(args, "report", None), "--report"),
        ):
            if side is not None and _same_file(side, path):
                raise CliError(f"{flag} '{side}' is the same file as the PNG output")
    if rendered.pdf is not None:
        _write(output, rendered.pdf)
        _wrote(args, output, "pdf", len(rendered.pdf), pages)
    if png:
        from . import _native

        for path, image in zip(png_paths, rendered.pngs):
            _write(path, image)
        if not args.quiet:
            pattern = output.parent / f"{output.stem}-page-NN.png"
            print(_native.wrote_png_line(pattern, len(rendered.pngs), args.dpi), file=sys.stderr)
    if getattr(args, "font_report", None) is not None:
        _write(args.font_report, json.dumps([asdict(f) for f in rendered.report.fonts]))
    if getattr(args, "report", None) is not None:
        report = {
            "page_count": pages,
            "document_pages": rendered.report.document_pages,
            "pages": [asdict(p) for p in rendered.report.pages],
            "fonts": [asdict(f) for f in rendered.report.fonts],
        }
        _write(args.report, json.dumps(report, ensure_ascii=False))


def _write_updated_fields(args: argparse.Namespace, doc: Document) -> int:
    """`convert --update-fields`: the .docx with refreshed field results."""
    output: Path | None = args.output
    if output is None:
        # As the binary: Word from Word goes to a named file, never stdout.
        _error("--output is required to write Word from Word", EXIT_ERROR)
        return EXIT_ERROR
    if args.report is not None:
        _check_side_file(args.report, "--report", ((output, "Word output"), (args.file, "input")), args.force)
    _ensure_writable(output, args.force)
    updated = doc.update_fields()
    for field in updated.fields:
        print(
            f"{field.paragraph}\t{field.kind}\t{json.dumps(field.old, ensure_ascii=False)} -> {json.dumps(field.new, ensure_ascii=False)}"
        )
    print(f"{len(updated.fields)} field(s) written; {updated.page_count} page(s)", file=sys.stderr)
    if args.report is not None:
        _write(
            args.report,
            json.dumps(
                {"page_count": updated.page_count, "fields": [asdict(f) for f in updated.fields]}, ensure_ascii=False
            ),
        )
    data = updated.document.to_bytes()
    _write(output, data)
    _wrote(args, output, "docx", len(data))
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
                print(
                    f"page {page.index + 1}: {page.changed_ratio * 100:.2f}% of pixels changed, box {list(page.bbox or (0, 0, 0, 0))}"
                )
        total = len(diff.pages)
        wrote = f" (wrote {args.out_dir})" if args.out_dir is not None else ""
        print(f"{len(changed)} of {total} page{'' if total == 1 else 's'} differ{wrote}")
    return EXIT_PAGES_DIFFER if changed else EXIT_OK


def cmd_compare(args: argparse.Namespace) -> int:
    from .document import diff

    original = _read_side(args.original, args.old_format)
    modified = _read_side(args.modified, args.new_format)
    default = args.original.with_name(f"{args.original.stem}_v_{args.modified.stem}.docx")
    from . import _native

    if args.output is None:
        # Without -o, compare (and the shorthand `A B`) prints the
        # redline's agent view.
        redline = _native.redline_documents(original, modified, author=args.author, date=args.datetime)
        return _print_view(redline, args.view, f"{default} (not written; -o keeps it)")
    output: Path = args.output
    if args.output_format == "md" and isinstance(original, str) and isinstance(modified, str):
        # A .md of two Markdown documents is CriticMarkup; of anything else,
        # the redline's view.
        if args.view_given:
            _error("read options shape the view; two Markdown documents write CriticMarkup to a .md", EXIT_USAGE)
            raise SystemExit(EXIT_USAGE)
        _ensure_writable(output, args.force)
        critic = diff(original, modified, format="critic", author=args.author, date=args.datetime).text
        _wrote(args, output, "md", _write(output, critic))
        return EXIT_OK
    if args.output_format != "png":
        _ensure_writable(output, args.force)
    redline = _native.redline_documents(original, modified, author=args.author, date=args.datetime)
    _write_result(args, redline, source=f"{default} (not written; -o keeps it)", view=getattr(args, "view", None))
    return EXIT_OK


def cmd_diff(args: argparse.Namespace) -> int:
    """Text only, as in the native CLI: the patch, a text view, or (two
    Markdown documents) CriticMarkup, on stdout or in -o's file."""
    from . import _native
    from .document import _decode_diff

    output = args.output
    # An existing output is refused before any work.
    if output is not None:
        _ensure_writable(output, args.force)
    old = _read_side(args.old, args.old_format)
    new = _read_side(args.new, args.new_format)
    if args.format in ("github", "word", "normal", "context", "side-by-side"):
        text = _native.diff_view(
            old,
            new,
            format=args.format,
            old_name=_spelled(args, "old"),
            new_name=_spelled(args, "new"),
            context=args.context,
            accept_changes=args.accept_changes,
            full_lines=args.full_lines,
        )
    else:
        critic = args.format == "critic"
        if critic and not (isinstance(old, str) and isinstance(new, str)):
            raise CliError(
                "--format critic needs both documents in Markdown; "
                "`jubarte compare OLD NEW` prints their redline as the agent view"
            )
        # The shared parser dated the run (D17).
        text = _decode_diff(
            _native.diff_json(
                old,
                new,
                old_name=_spelled(args, "old"),
                new_name=_spelled(args, "new"),
                author=args.author,
                date=args.datetime,
                columns=args.columns,
                critic=critic,
            )
        ).text
    if output is None:
        sys.stdout.write(text)
        return EXIT_OK
    size = _write(output, text)
    if not args.quiet:
        print(f"wrote {_spelled(args, 'output')} (md, {size} bytes)", file=sys.stderr)
    return EXIT_OK


def _json_lines(payload: str) -> None:
    """Each object of the engine's JSON array on its own line, as the binary prints them."""
    for row in json.loads(payload):
        print(json.dumps(row, ensure_ascii=False, separators=(",", ":")))


def cmd_changes(args: argparse.Namespace) -> int:
    from . import _native

    docx = _input(args, notice=not args.json)
    authors = args.author or None
    if args.json:
        _json_lines(_native.list_changes_json(docx, authors=authors))
        print(_native.input_validity_line(docx, _spelled(args, "file")))
    else:
        sys.stdout.write(_native.changes_text(docx, authors=authors))
    return EXIT_OK


def cmd_comments(args: argparse.Namespace) -> int:
    from . import _native

    docx = _input(args, notice=not args.json)
    if args.json:
        _json_lines(_native.list_comments_json(docx, author=args.author, latest=args.latest))
        print(_native.input_validity_line(docx, _spelled(args, "file")))
    else:
        sys.stdout.write(_native.comments_text(docx, author=args.author, latest=args.latest))
    return EXIT_OK


def _resolution(args: argparse.Namespace, accept: bool) -> int:
    from . import _native

    doc = _read(args.file)
    _ensure_writable(args.output, args.force)
    selection = {"ids": args.id, "authors": args.author, "kinds": args.kind}
    try:
        result = doc.accept(**selection) if accept else doc.reject(**selection)
    except ValueError as exc:
        # An id that is no change id, or names two: a usage error, as the binary's.
        _error(str(exc), EXIT_USAGE)
        raise SystemExit(EXIT_USAGE) from exc
    except _native.JubarteError as exc:
        raise CliError(f"{'accept' if accept else 'reject'} failed: {exc}") from exc
    _write_result(args, result.to_bytes(), source=_not_written(args))
    return EXIT_OK


def cmd_accept(args: argparse.Namespace) -> int:
    return _resolution(args, accept=True)


def cmd_reject(args: argparse.Namespace) -> int:
    return _resolution(args, accept=False)


def cmd_validate(args: argparse.Namespace) -> int:
    from . import _native

    doc = _read(args.file)
    docx = doc.to_bytes()
    # An original that cannot be read stops the run before any repair.
    original = _read(args.original).to_bytes() if args.original is not None else None
    if args.repair is not None:
        _ensure_writable(args.repair, args.force)
        repaired = doc.repair()
        docx = repaired.document.to_bytes()
        size = _write(args.repair, docx)
        if not args.json:
            # A structural repair: the edits' history is a separate check.
            history = (
                f"history check against {args.original} follows"
                if args.original is not None
                else "history not checked (--original, --author)"
            )
            print(f"repaired {len(repaired.repaired)} finding(s), structure only; {history}")
        if not args.quiet:
            sys.stdout.flush()  # stdout's lines first when both streams share a pipe
            print(_native.wrote_line(_spelled(args, "repair"), "docx", size), file=sys.stderr)
    baseline = _read(args.baseline).to_bytes() if args.baseline is not None else None
    jsonl, text, failed = _native.validate_run(
        docx,
        rules=list(args.rules) or None,
        original=original,
        author=args.author,
        strict=args.strict,
        baseline=baseline,
    )
    sys.stdout.write(jsonl if args.json else text)
    return EXIT_FINDINGS if failed else EXIT_OK


def cmd_capabilities(args: argparse.Namespace) -> int:
    if args.json:
        print(
            json.dumps(
                capabilities(operation=args.operation, field=args.field, matrix=args.matrix),
                indent=2,
            )
        )
        return EXIT_OK
    from . import _native

    sys.stdout.write(_native.capabilities_text(args.operation, args.field, args.matrix))
    return EXIT_OK


# -- parser -------------------------------------------------------------------


# Handlers own host I/O. Rust clap owns the grammar, defaults and help.
_HANDLERS = {
    "read": cmd_read,
    "edit": cmd_edit,
    "add": cmd_add,
    "convert": cmd_convert,
    "compare": cmd_compare,
    "diff": cmd_diff,
    "changes": cmd_changes,
    "comments": cmd_comments,
    "accept": cmd_accept,
    "reject": cmd_reject,
    "visual-diff": cmd_diff_render,
    "validate": cmd_validate,
    "capabilities": cmd_capabilities,
}
_PATH_ARGUMENTS = {
    "file",
    "original",
    "baseline",
    "modified",
    "old",
    "new",
    "output",
    "plan",
    "out_dir",
    "reference_doc",
    "report",
    "font_report",
    "repair",
    "a",
    "b",
}


class SharedParser:
    """Compatibility facade: argparse.Namespace and SystemExit over Rust clap."""

    supported = tuple(_HANDLERS)

    def error(self, message: str) -> None:
        _error(message, EXIT_USAGE)
        raise SystemExit(2)

    def parse_args(self, argv: Sequence[str] | None = None) -> argparse.Namespace:
        from . import _native

        global _JSON_ERRORS
        arguments = list(sys.argv[1:] if argv is None else argv)
        _JSON_ERRORS = _native.wants_json(arguments)
        parsed = json.loads(
            _native.parse_cli_json(arguments, program="jubarte-redlines", supported=list(self.supported))
        )
        if "text" in parsed:
            for note in parsed.get("notes", ()):
                print(note, file=sys.stderr)
            if "json" in parsed:
                sys.stdout.write(parsed["json"])
                sys.stdout.flush()
            stream = sys.stderr if parsed["stream"] == "stderr" else sys.stdout
            stream.write(parsed["text"])
            raise SystemExit(parsed["exit_code"])
        for note in parsed.get("notes", ()):
            print(note, file=sys.stderr)
        command, values = parsed["command"], parsed["args"]
        self._validate_host(command, values)
        # FILE and --plan as typed: messages quote them, and Path() would
        # drop a "./" or a trailing "/" the other CLIs keep.
        values["spelled"] = {
            key: values[key]
            for key in ("file", "plan", "old", "new", "output", "repair")
            if values.get(key) is not None
        }
        for key in _PATH_ARGUMENTS:
            if values.get(key) is not None:
                values[key] = Path(values[key])
        if command in ("accept", "reject"):
            for plural, singular in (("ids", "id"), ("authors", "author"), ("kinds", "kind")):
                values[singular] = values.pop(plural, []) or None
        return argparse.Namespace(command=command, func=_HANDLERS[command], **values)

    def _validate_host(self, command: str, values: dict) -> None:
        # Capability checks only, never an alternate option grammar.
        if command == "compare":
            if values.get("mode", "word") != "word" or values.get("powertools_faithful"):
                self.error("--mode powertools is not supported by the Python CLI")
            if values.get("detail_threshold") is not None:
                self.error("--detail-threshold is not supported by the Python CLI")
            if values.get("no_paragraph_merge"):
                self.error("--no-paragraph-merge is not supported by the Python CLI")
        if command == "compare":
            for key in ("reference_doc", "critic", "resource_path"):
                if values.get(key) not in (None, False):
                    self.error(f"--{key.replace('_', '-')} is not supported by the Python CLI")
        if command == "convert":
            for key in ("from", "resource_path", "timeout", "fail_on_substitution", "no_page_markers"):
                if values.get(key) not in (None, False):
                    self.error(f"--{key.replace('_', '-')} is not supported by the Python CLI")
        if command == "diff":
            context = values.get("context", 3)
            if type(context) is not int or not 0 <= context <= 2**32 - 1:
                self.error("--context must be in the u32 range (0..4294967295)")
        if command == "convert":
            to = values.get("to")
            extension = Path(values.get("output") or "").suffix.lower()
            if to == "md" or (to is None and extension in (".md", ".markdown", ".txt", ".mdown", ".mkd", ".mkdn")):
                self.error(
                    "--to md with page markers is not supported by the Python CLI; `read FILE` prints the agent text view"
                )
            # A Word name fails before any read; a name that says nothing
            # is sniffed in cmd_convert.
            word_in = _named_kind(Path(values["file"])) == "docx"
            if (to == "docx" or (to is None and extension == ".docx")) and word_in and not values.get("update_fields"):
                self.error(_DOCX_NEEDS_MARKDOWN)
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
        _error(str(exc), EXIT_ERROR)
        return EXIT_ERROR
    except Exception as exc:  # engine errors (JubarteError) and I/O surprises
        _error(str(exc), EXIT_ERROR)
        return EXIT_ERROR


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
