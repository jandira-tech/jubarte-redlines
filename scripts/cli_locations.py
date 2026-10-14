#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Write every id the `jubarte` CLI prints or takes to docs/locations.md.

Four sections:

1. Where the Rust source accepts an id (`scripts/cli_id_sources.py`, read
   from `src/`, not from `--help`): the clap fields whose value is an id and
   the functions that read them, every id parser with its doc and refusals,
   every function that calls a parser, and the call chains from each flag
   to its parsers.

The other three come from the binary itself (`target/debug/jubarte`, built
first):

2. The grammar `capabilities --json` advertises.
3. Every id the CLI prints for the committed fixtures below (the view, the
   YAML header, `read --tables`, `read --json`, `changes`, `comments`,
   `validate`),
   grouped by shape (digits become N), with where it was printed and what
   it names.
4. Each printed shape and a fixed list of other spellings fed back to
   `read -p`, `edit -p`, `add -p` and `accept --id`: what each picked (the id
   lines it printed, the changes the output lost or gained), or its exit code
   and first line of stderr.

On top, "Deprecated ids still in use": every place the CLI prints or accepts
a long id (`body:p:12`) or a retired 0.12 spelling (`#12`, `t0.r1.c2`).
Every run prints them on stderr until none is left; `--strict` exits 1.

The output depends only on the source and the fixtures (no date, no
commit, no temporary paths), so a change shows up as a diff:

    scripts/cli_locations.py           # rewrite docs/locations.md
    scripts/cli_locations.py --check   # exit 1 when the committed file drifted
    scripts/cli_locations.py --strict  # exit 1 while a deprecated id is in use
"""

from __future__ import annotations

import argparse
import concurrent.futures
import difflib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

import cli_id_sources

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "docs" / "locations.md"
BINARY = ROOT / "target" / "debug" / "jubarte"

# Small committed fixtures that between them hold every story jubarte names:
# headers and footers, footnotes and endnotes, tables (nested, in headers,
# in a text box), comments with a reply, revisions in several stories.
FIXTURES = [
    "tests/fixtures/agent-view/received.docx",
    "tests/fixtures/from-docx/critic/synthetic/hard-notes.docx",
    "tests/fixtures/from-docx/critic/synthetic/hard-tables.docx",
    "tests/fixtures/nested_table/hf_a.docx",
    "tests/fixtures/sections/multi_section.docx",
    "tests/fixtures/multibody/multibody.docx",
]

# Spellings probed besides the printed ones: the advertised forms, the
# retired ones, and the ones a user would plausibly try.
CANDIDATES = [
    "0",
    "5",
    "p0",
    "p01",
    "P0",
    "#5",
    "body.p0",
    "body:p:0",
    "p4-p7",
    "p17-",
    "-p3",
    "p0,p1",
    "header1",
    "header1.p0",
    "header1.p1",
    "header1:p:0",
    "footer2.p0",
    "footnotes",
    "footnotes.p0",
    "footnotes:p:0",
    "endnotes.p0",
    "endnotes.p0-p2",
    "comments.p0",
    "textbox1.p0",
    "t0",
    "t0.row1",
    "t0.row1.cell1",
    "t0.row1.cell1.p0",
    "t0.r1.c1",
    "header1.t0.row0.cell0",
    "t0.row0.cell0.t0.row0.cell0",
    "c5",
    "#c5",
    "c5.text",
    "c6.text",
    "c5.p0",
    "r0",
    "#0",
    "r1+2",
    "#1+2",
    "body.r0",
    "body:rev:0",
    "footnotes.r2",
    "endnotes.r4",
    "header1.r0",
]

# The fixtures the fixed spellings are tried on.
CANDIDATE_FIXTURES = FIXTURES[:2]

PROBE_TEXT = "PROBE"

# Long ids (`body:p:12`, `header1:rev:3`) and 0.12's retired spellings
# (`#12`, `#c5`, `t0.r1.c2`): each place the CLI still prints or accepts one
# is listed under "Deprecated ids", and every run warns until none is left.
LONG_ID = re.compile(r"(?<![\w.#])[A-Za-z][\w]*:(?:p|rev):\d+")
RETIRED_ID = re.compile(r"(?<![\w])#c?\d+(?:\+\d+)*|\bt\d+\.r\d+\.c\d+\b")


class Deprecated:
    """(bucket, where, example) for each long or retired id still in use."""

    def __init__(self) -> None:
        self.hits: dict[tuple[str, str], set[str]] = defaultdict(set)

    def scan(self, bucket: str, where: str, text: str) -> None:
        for token in LONG_ID.findall(text) + RETIRED_ID.findall(text):
            self.hits[bucket, where].add(token)


DEPRECATED = Deprecated()


TIMEOUT = 120


def run(binary: Path, args: list[str], cwd: Path) -> tuple[int, str, str]:
    # UTF-8 whatever the locale: the view prints em dashes, and a C-locale
    # host would otherwise fail to decode them.
    try:
        done = subprocess.run(
            [str(binary), *args],
            cwd=cwd,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=TIMEOUT,
        )
    except subprocess.TimeoutExpired as hung:
        # A timeout says how loaded the machine was, not what the CLI does:
        # stop rather than write it into the catalogue.
        command = " ".join([Path(binary).name, *args])
        raise RuntimeError(f"timed out after {TIMEOUT}s: {command}") from hung
    return done.returncode, done.stdout, done.stderr


def cell(text: str) -> str:
    return " ".join(text.split()).replace("|", "\\|")


def shape(token: str) -> str:
    return re.sub(r"\d+", "N", token)


# ---------------------------------------------------------------- section 3


class Seen:
    def __init__(self) -> None:
        self.tokens: dict[str, set[str]] = defaultdict(set)  # shape -> tokens
        self.where: dict[str, set[str]] = defaultdict(set)  # shape -> sources
        self.what: dict[str, set[str]] = defaultdict(set)  # shape -> kinds
        self.fixture: dict[str, set[str]] = defaultdict(set)  # token -> fixtures

    def add(self, token: str, source: str, fixture: str, kind: str | None = None) -> None:
        key = shape(token)
        self.tokens[key].add(token)
        self.where[key].add(source)
        if kind:
            self.what[key].add(kind)
        self.fixture[token].add(fixture)


def harvest(binary: Path, fixture: str, seen: Seen) -> None:
    name = Path(fixture).name
    _, view, _ = run(binary, ["read", fixture], ROOT)
    DEPRECATED.scan("printed in text", "read (view)", view)
    for line in view.splitlines():
        if match := re.match(r"\s*<!-- (\S+)", line):
            if match.group(1) != "page":
                seen.add(match.group(1), "view id line", name)
        for note in re.findall(r"\{>>(\S+)", line):
            seen.add(note, "view note", name)
        for reply in re.findall(r" re (c\d+):", line):
            seen.add(reply, "view note (reply to)", name)
        for story in re.findall(r"\bid: ([^,}\s]+)", line):
            seen.add(story, "view header", name)
    _, tables, _ = run(binary, ["read", fixture, "--tables"], ROOT)
    DEPRECATED.scan("printed in text", "read --tables", tables)
    for line in tables.splitlines():
        for token in re.findall(r"(?:^|\t)([^\t=\s]+)=", line):
            seen.add(token, "read --tables", name)
    _, changes, _ = run(binary, ["changes", fixture], ROOT)
    DEPRECATED.scan("printed in text", "changes", changes)
    for line in changes.splitlines():
        parts = line.split("\t")
        if len(parts) >= 5:
            seen.add(parts[0], "changes (id)", name)
            seen.add(parts[4], "changes (paragraph)", name)
    _, comments, _ = run(binary, ["comments", fixture], ROOT)
    DEPRECATED.scan("printed in text", "comments", comments)
    for line in comments.splitlines():
        parts = line.split("\t")
        if len(parts) >= 2 and re.fullmatch(r"\S+", parts[0]):
            seen.add(parts[0], "comments (id)", name)
            seen.add(parts[1], "comments (paragraph)", name)
        for reply in re.findall(r"reply to (\S+)", line):
            seen.add(reply, "comments (reply to)", name)
    _, findings, _ = run(binary, ["validate", fixture, "--rules", "word,a11y,style,structure"], ROOT)
    DEPRECATED.scan("printed in text", "validate", findings)
    for line in findings.splitlines():
        parts = line.strip().split("\t")
        if len(parts) >= 4:
            # The word and edits sets print `part#ref`; the audit sets the ref alone.
            ref = parts[2].rpartition("#")[2]
            if ref and "/" not in ref:
                seen.add(ref, "validate (ref)", name)
    code, snapshot, _ = run(binary, ["read", fixture, "--json"], ROOT)
    if code != 0:
        return
    data = json.loads(snapshot)
    DEPRECATED.scan("printed in JSON (kept by D7)", "read --json", snapshot)
    for para in data.get("paragraphs", []):
        kind = "body table cell paragraph" if para.get("in_table") else "body paragraph"
        seen.add(para["id"], "read --json paragraphs[].id", name, kind)
    for story in data.get("stories", []):
        seen.add(story["id"], "read --json stories[].id", name, f"{story['kind']} story")
        for para in story.get("paragraphs", []):
            kind = f"{story['kind']} " + ("table cell paragraph" if para.get("in_table") else "paragraph")
            seen.add(para["id"], "read --json stories[].paragraphs[].id", name, kind)
    for change in data.get("changes", []):
        kind = f"change ({change['kind']}, {change['target']})"
        seen.add(change["id"], "read --json changes[].id", name, kind)
        seen.add(change["ref"], "read --json changes[].ref", name, kind)
        if change.get("paragraph"):
            seen.add(change["paragraph"], "read --json changes[].paragraph", name)
    for comment in data.get("comments", []):
        seen.add(comment["ref"], "read --json comments[].ref", name, "comment")
        if comment.get("paragraph"):
            seen.add(comment["paragraph"], "read --json comments[].paragraph", name)


# ---------------------------------------------------------------- section 4


def probes(token: str, fixture: str, scratch: Path, n: int) -> list[list[str]]:
    # One output per task: the probes run in parallel.
    out = [str(scratch / f"{n}-{task}.docx") for task in ("edit", "add", "accept")]
    is_change = bool(re.fullmatch(r"(?:[a-z][a-z0-9]*\.)?r[\d+]+|#[\d+]+|\S+:rev:\d+", token))
    is_comment = bool(re.fullmatch(r"#?c\d+", token))
    edit_op = (
        ["--accept"] if is_change else ["--reply", PROBE_TEXT] if is_comment else ["--content", PROBE_TEXT]
    )
    return [
        ["read", fixture, f"--paragraphs={token}"],
        ["edit", fixture, f"--location={token}", *edit_op, "-o", out[0], "--force"],
        ["add", fixture, f"--location={token}", "--content", PROBE_TEXT, "-o", out[1], "--force"],
        ["accept", fixture, f"--id={token}", "-o", out[2], "--force"],
    ]


def change_ids(binary: Path, docx: str) -> list[str]:
    _, out, _ = run(binary, ["changes", docx], ROOT)
    return [line.split("\t")[0] for line in out.splitlines() if "\t" in line]


def listed(ids: list[str]) -> str:
    return " ".join(ids[:6]) + (f" +{len(ids) - 6} more" if len(ids) > 6 else "")


def verdict(
    binary: Path, args: list[str], scratch: Path, before: list[str]
) -> tuple[str, list[tuple[str, str]]]:
    """The exit code and what the option picked (or the first stderr line),
    and the deprecated ids the task printed."""
    task, token = args[0], args[2].split("=", 1)[1]
    code, out, err = run(binary, args, ROOT)
    out = out.replace(str(scratch), "$TMP").replace(str(ROOT) + "/", "")
    err = err.replace(str(scratch), "$TMP").replace(str(ROOT) + "/", "")
    flag = args[2].split("=", 1)[0]
    # An id the task only echoes back from its own argument is not a use.
    found = [
        (f"{task} {flag} ({stream})", t)
        for stream, text in (("stdout", out), ("stderr", err))
        for t in LONG_ID.findall(text) + RETIRED_ID.findall(text)
        if t not in token
    ]
    if code == 0:
        if LONG_ID.fullmatch(token) or RETIRED_ID.fullmatch(token):
            found.append((f"{task} {flag} accepts it", token))
        picked = [i for i in re.findall(r"<!-- (\S+)", out) if i != "page"]
        parts = [listed(picked)] if picked else []
        written = next((a for a in args if a.endswith(".docx") and "$" not in a and a != args[1]), None)
        if written and Path(written).exists():
            after = change_ids(binary, written)
            gone = [i for i in before if i not in after]
            new = [i for i in after if i not in before]
            if gone:
                parts.append("-" + " -".join(gone[:6]) + (" …" if len(gone) > 6 else ""))
            if new:
                parts.append("+" + " +".join(new[:6]) + (" …" if len(new) > 6 else ""))
        return f"0 → {'; '.join(parts) or '(nothing)'}", found
    first = next((line for line in err.splitlines() if line.strip()), "")
    at = re.search(r'"at":"([^"]+)"', err)
    where = f" [at {at.group(1)}]" if at else ""
    return f"{code}{where} {first[:200]}", found


def probe_table(binary: Path, pairs: list[tuple[str, str]], scratch: Path, tag: str) -> list[str]:
    jobs = [
        (token, fixture, args)
        for n, (token, fixture) in enumerate(pairs)
        for args in probes(token, fixture, scratch / tag, n)
    ]
    (scratch / tag).mkdir()
    before = {fixture: change_ids(binary, fixture) for _, fixture in pairs}
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(lambda job: verdict(binary, job[2], scratch, before[job[1]]), jobs))
    for _, found in results:
        for where, token in found:
            DEPRECATED.hits["accepted or printed by a task", where].add(token)
    lines = [
        "| Id | Fixture | read -p (picked) | edit -p (picked) | add -p (picked) | accept --id (picked) |",
        "|-|-|-|-|-|-|",
    ]
    for i in range(0, len(jobs), 4):
        token, fixture = jobs[i][0], jobs[i][1]
        row = [f"`{token}`", Path(fixture).name, *(cell(r) for r, _ in results[i : i + 4])]
        lines.append("| " + " | ".join(row) + " |")
    return lines


# ---------------------------------------------------------------- render


def render(binary: Path) -> str:
    # REUSE-IgnoreStart (the header written into the generated file, not this script's)
    lines = [
        "<!--",
        "SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC",
        "",
        "SPDX-License-Identifier: AGPL-3.0-only",
        "-->",
        # REUSE-IgnoreEnd
        "",
        "# jubarte ids and locations",
        "",
        "Generated by `scripts/cli_locations.py` from the binary and the committed fixtures; "
        "do not edit by hand. `scripts/cli_locations.py --check` fails when this file no "
        "longer matches the source.",
        "",
        *cli_id_sources.section(ROOT, cell),
    ]

    _, manifest, _ = run(binary, ["capabilities", "--json"], ROOT)
    ids = json.loads(manifest).get("ids", {})
    lines += ["", "## Advertised grammar (`capabilities --json`)", "", "| Kind | Spellings |", "|-|-|"]
    for kind in sorted(k for k in ids if k != "note"):
        lines.append(f"| {kind} | {', '.join(f'`{s}`' for s in ids[kind])} |")
    if "note" in ids:
        lines += ["", f"Note: {ids['note']}"]

    seen = Seen()
    for fixture in FIXTURES:
        harvest(binary, fixture, seen)
    lines += [
        "",
        "## Printed ids",
        "",
        "Fixtures: " + ", ".join(f"`{f}`" for f in FIXTURES) + ".",
        "",
        "| Shape | Examples | Names | Printed by |",
        "|-|-|-|-|",
    ]
    for key in sorted(seen.tokens):
        examples = sorted(seen.tokens[key], key=lambda t: (len(t), t))[:3]
        lines.append(
            f"| `{key}` | {', '.join(f'`{e}`' for e in examples)} | "
            f"{cell('; '.join(sorted(seen.what[key])))} | {cell('; '.join(sorted(seen.where[key])))} |"
        )

    with tempfile.TemporaryDirectory() as tmp:
        scratch = Path(tmp).resolve()
        printed = []
        for key in sorted(seen.tokens):
            token = sorted(seen.tokens[key], key=lambda t: (len(t), t))[0]
            fixture = next(f for f in FIXTURES if Path(f).name in seen.fixture[token])
            printed.append((token, fixture))
        lines += [
            "",
            "## Printed ids fed back",
            "",
            "One id of each printed shape, given back to the tasks that take an id "
            "(`--paragraphs=ID`, `--location=ID`, `--id=ID`). Exit 0 shows what the "
            "option picked: the id lines the task printed, then the changes the output "
            "lost (`-r0`) or gained (`+r9`) against `changes` on the input. Otherwise the "
            "exit code, the report's `at` and the first line of stderr.",
            "",
            *probe_table(binary, printed, scratch, "printed"),
        ]
        candidates = [(token, fixture) for fixture in CANDIDATE_FIXTURES for token in CANDIDATES]
        lines += [
            "",
            "## Other spellings",
            "",
            "Advertised, retired and plausible spellings, on two fixtures.",
            "",
            *probe_table(binary, candidates, scratch, "other"),
        ]
    section = [
        "## Deprecated ids still in use",
        "",
        "Long ids (`body:p:12`, `header1:rev:3`) and 0.12's retired spellings (`#12`, `#c5`, "
        "`t0.r1.c2`) that the CLI still prints or accepts. Every run of this script warns "
        "while this table has rows; `--strict` fails. Plan D7 keeps long ids in JSON, machine "
        "hints and error payloads, so those rows stay until D7 changes.",
        "",
        "| Bucket | Where | Examples |",
        "|-|-|-|",
    ]
    for (bucket, where), tokens in sorted(DEPRECATED.hits.items()):
        examples = sorted(tokens, key=lambda t: (len(t), t))
        shown = ", ".join(f"`{t}`" for t in examples[:4]) + (
            f" +{len(examples) - 4} more" if len(examples) > 4 else ""
        )
        section.append(f"| {bucket} | {cell(where)} | {shown} |")
    if not DEPRECATED.hits:
        section.append("| (none) | | |")
    intro_end = lines.index("## Where the source accepts an id")
    lines[intro_end:intro_end] = [*section, ""]
    return "\n".join(lines) + "\n"


def warn_deprecated() -> int:
    """Print the deprecated-id summary to stderr; the number of places."""
    places = len(DEPRECATED.hits)
    if places:
        print(
            f"cli_locations: warning: {places} place(s) still print or accept long or retired ids "
            '(docs/locations.md, "Deprecated ids still in use"):',
            file=sys.stderr,
        )
        for bucket, where in sorted(DEPRECATED.hits):
            print(f"  {bucket}: {where}", file=sys.stderr)
    return places


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="exit 1 when docs/locations.md drifted")
    parser.add_argument("--strict", action="store_true", help="exit 1 while any deprecated id is in use")
    args = parser.parse_args()
    cargo = shutil.which("cargo") or str(Path.home() / ".cargo" / "bin" / "cargo")
    subprocess.run([cargo, "build", "--quiet", "--bin", "jubarte"], cwd=ROOT, check=True)
    fresh = render(BINARY)
    if warn_deprecated() and args.strict:
        return 1
    if args.check:
        current = TARGET.read_text(encoding="utf-8") if TARGET.exists() else ""
        if current != fresh:
            sys.stdout.writelines(
                difflib.unified_diff(current.splitlines(True), fresh.splitlines(True), str(TARGET), "fresh")
            )
            print(
                f"{TARGET.relative_to(ROOT)} drifted from the CLI: run scripts/cli_locations.py",
                file=sys.stderr,
            )
            return 1
        return 0
    TARGET.write_text(fresh, encoding="utf-8")
    print(f"wrote {TARGET.relative_to(ROOT)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
