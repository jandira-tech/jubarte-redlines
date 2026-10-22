# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Measure what docx -> md -> docx -> md loses, per dimension, for both Markdown dialects.

Dialects:
  view     `jubarte read` (the agent view: ids, YAML header) -> `convert X.md -o X.docx`
  convert  `jubarte convert X.docx -o X.md` -> `convert X.md -o X.docx`

For each document the original and the round-tripped docx are compared through
`read --json` (the snapshot), one dimension at a time, and the two Markdown files
(first and second pass) are compared for a fixed point. The output is a Markdown
table of the share of documents that keep each dimension, and, with --details,
the first difference per document and dimension. Deterministic: documents are
sorted per folder, every Nth of each folder is taken, and no date or absolute path reaches the
report.

  scripts/md_roundtrip.py tests/corpus --every 7
  scripts/md_roundtrip.py tests/fixtures/agent-view/received.docx --details
"""

import argparse
import concurrent.futures
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target" / "debug" / "jubarte"
DIALECTS = ("view", "convert")


class HarnessError(Exception):
    """The measurement itself failed (a timeout twice, a snapshot it cannot read): the run must
    fail, never drop the document from the shares."""


def run(args: list[str], cwd: Path) -> subprocess.CompletedProcess:
    # A timeout is load, not a property of the document: retry once, then fail the run.
    for _ in range(2):
        try:
            return subprocess.run(
                [str(BINARY), *args],
                cwd=cwd,
                capture_output=True,
                encoding="utf-8",
                errors="replace",
                timeout=300,
            )
        except subprocess.TimeoutExpired:
            continue
    raise HarnessError(f"jubarte {' '.join(args[:2])} timed out twice (300s)")


def snapshot(docx: Path, cwd: Path) -> dict | None:
    done = run(["read", str(docx), "--json"], cwd)
    if done.returncode != 0:
        return None
    return json.loads(done.stdout)


def markdown(dialect: str, docx: Path, out: Path, cwd: Path) -> bool:
    if dialect == "view":
        done = run(["read", str(docx), "--no-page-markers"], cwd)
        if done.returncode != 0:
            return False
        out.write_text(done.stdout, encoding="utf-8")
        return True
    return run(["convert", str(docx), "-o", str(out), "--force"], cwd).returncode == 0


def runs(paragraph: dict) -> list:
    return [
        tuple(r.get(k) for k in ("start", "end", "bold", "italic", "underline", "highlight"))
        for r in paragraph.get("runs", [])
        if r.get("bold") or r.get("italic") or r.get("underline") or r.get("highlight")
    ]


def dimensions(snap: dict) -> dict:
    """The snapshot cut into the dimensions a round trip should keep. Each name says exactly
    what it compares; every list keeps document order (a deletion before its insertion, header1
    before header2, a reply after its comment)."""
    body = snap["paragraphs"]
    stories = [(s["kind"], tuple(p["text"] for p in s["paragraphs"])) for s in snap["stories"]]
    comment_keys = ("author", "initials", "date", "text", "done", "anchor_text", "parent")
    change_keys = ("kind", "target", "paragraph", "author", "date", "text")
    return {
        "paragraph text": [p["text"] for p in body],
        "paragraph styles": [p["style"] for p in body],
        "numbered or not": [p["numbered"] for p in body],
        "b/i/u/highlight": [runs(p) for p in body],
        "page breaks": [p["page_break"] for p in body],
        "table cell text": [[[c["text"] for c in row] for row in t["rows"]] for t in snap["tables"]],
        "header/footer text": [s for s in stories if s[0] in ("header", "footer")],
        "note text": [s for s in stories if s[0] not in ("header", "footer")],
        "changes (kind, place, author, date, text)": [
            tuple(str(c.get(k)) for k in change_keys) for c in snap["changes"]
        ],
        "comments (author, date, text, thread)": [
            tuple(str(c.get(k)) for k in comment_keys) for c in snap["comments"]
        ],
    }


DIMENSIONS = list(dimensions({"paragraphs": [], "stories": [], "tables": [], "changes": [], "comments": []}))


def first_difference(a, b) -> str:
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            return f"{len(a)} items -> {len(b)}"
        for i, (x, y) in enumerate(zip(a, b, strict=True)):
            if x != y:
                return f"[{i}] {str(x)[:80]!r} -> {str(y)[:80]!r}"
        return "same lines; the difference is outside line content (a trailing newline?)"
    return f"{str(a)[:80]!r} -> {str(b)[:80]!r}"


SOURCE_LINES = re.compile(r"^(source|sha256): .*\n", re.M)


def measure(docx: Path) -> dict:
    """{dialect: {dimension: None (kept) or the first difference}} plus a fixed-point row."""
    try:
        return measure_one(docx)
    except (KeyError, TypeError, ValueError) as error:
        raise HarnessError(f"{docx}: {type(error).__name__}: {error}") from error


def measure_one(docx: Path) -> dict:
    result: dict[str, dict[str, str | None]] = {}
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp)
        # The source sits apart from the Markdown: a conversion anchored on a file beside the
        # Markdown (`source:`/`sha256:`) would keep everything trivially and measure nothing.
        (work / "source").mkdir()
        src = work / "source" / "src.docx"
        shutil.copyfile(docx, src)
        original = snapshot(src, work)
        if original is None:
            # A jubarte failure scores zero, never a skip (AGENTS.md).
            failed = {d: "read --json failed on the source" for d in [*DIMENSIONS, "md fixed point"]}
            return {dialect: dict(failed) for dialect in DIALECTS}
        want = dimensions(original)
        for dialect in DIALECTS:
            row: dict[str, str | None] = {}
            md1, docx2, md3 = work / f"{dialect}.1.md", work / f"{dialect}.2.docx", work / f"{dialect}.3.md"
            if not markdown(dialect, src, md1, work):
                result[dialect] = {d: "docx -> md failed" for d in [*DIMENSIONS, "md fixed point"]}
                continue
            if run(["convert", str(md1), "-o", str(docx2), "--force"], work).returncode != 0:
                result[dialect] = {d: "md -> docx failed" for d in [*DIMENSIONS, "md fixed point"]}
                continue
            back = snapshot(docx2, work)
            got = dimensions(back) if back else None
            for name in DIMENSIONS:
                if got is None:
                    row[name] = "read --json failed on the round trip"
                elif want[name] != got[name]:
                    row[name] = first_difference(want[name], got[name])
                else:
                    row[name] = None
            if markdown(dialect, docx2, md3, work):
                a = SOURCE_LINES.sub("", md1.read_text(encoding="utf-8"))
                b = SOURCE_LINES.sub("", md3.read_text(encoding="utf-8"))
                row["md fixed point"] = None if a == b else first_difference(a.splitlines(), b.splitlines())
            else:
                row["md fixed point"] = "docx -> md failed on the round trip"
            result[dialect] = row
    return result


def positive(text: str) -> int:
    """An --every of 1 or more."""
    value = int(text)
    if value < 1:
        raise argparse.ArgumentTypeError("--every takes 1 or more")
    return value


def documents(paths: list[str], every: int) -> list[Path]:
    """Every Nth document of each folder, so the sample does not depend on argument order."""
    found = set()
    for p in map(Path, paths):
        if p.is_dir():
            found |= set(sorted(f for f in p.rglob("*.docx") if not f.name.startswith("~$"))[::every])
        else:
            found.add(p)
    return sorted(found)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("paths", nargs="+", help="docx files or folders")
    parser.add_argument("--every", type=positive, default=1, help="take every Nth document (sorted)")
    parser.add_argument("--details", action="store_true", help="list the first difference per document")
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    args = parser.parse_args()
    cargo = shutil.which("cargo") or str(Path.home() / ".cargo" / "bin" / "cargo")
    subprocess.run([cargo, "build", "--quiet", "--bin", "jubarte"], cwd=ROOT, check=True)
    docs = documents(args.paths, args.every)
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        try:
            results = dict(zip(docs, pool.map(measure, docs), strict=True))
        except HarnessError as error:
            # The documents still queued would each run up to two timeouts.
            pool.shutdown(wait=False, cancel_futures=True)
            print(f"harness error: {error}", file=sys.stderr)
            return 1
    # Every document counts: a source jubarte cannot read scores zero on every dimension.
    measured = results
    print(f"# docx -> md -> docx -> md: {len(measured)} documents\n")
    print("Share of documents that keep each dimension.\n")
    print("| Dimension | " + " | ".join(DIALECTS) + " |")
    print("|-|" + "-|" * len(DIALECTS))
    for name in [*DIMENSIONS, "md fixed point"]:
        cells = []
        for dialect in DIALECTS:
            kept = sum(1 for r in measured.values() if r[dialect][name] is None)
            cells.append(f"{kept}/{len(measured)} ({100 * kept // max(len(measured), 1)}%)")
        print(f"| {name} | " + " | ".join(cells) + " |")
    if args.details:
        for dialect in DIALECTS:
            print(f"\n## {dialect}\n")
            reasons = Counter()
            for doc, r in sorted(measured.items()):
                for name, why in r[dialect].items():
                    if why is not None:
                        print(
                            f"- {doc.relative_to(ROOT) if doc.is_relative_to(ROOT) else doc.name} · {name}: {why}"
                        )
                        reasons[name] += 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
