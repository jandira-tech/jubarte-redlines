#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
"""Regenerate a CLI reference block in a docs page from real ``--help`` output.

Runs a CLI, discovers its subcommands from the top-level help text, captures
``--help`` for the top level and every (nested) subcommand, and writes the
result between ``gen:<marker>:start`` / ``gen:<marker>:end`` markers in a
markdown file. Output is deterministic: no timestamps, no paths, no terminal
widths. Run through ``scripts/gen_docs.sh``; CI fails when the committed block
drifts from the real one.

Used for all three runners of the shared command set — the Rust ``jubarte``
binary (clap), the Python wheel's ``jubarte-redlines`` (argparse) and the npm
``jubarte-redlines`` CLI — so one page per surface can quote the exact flags
the installed runner accepts.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess

START = "<!-- gen:{marker}:start -->"
END = "<!-- gen:{marker}:end -->"

# Leading comma-separated identifiers: `redline, compare  description…`,
# `revisions     description…`. Stops at the two-or-more-space description gap
# or at any non-identifier character.
_NAME_LIST = re.compile(r"^([a-z][a-z0-9_-]*(?:,\s*[a-z][a-z0-9_-]*)*)\b")
_ARGPARSE_CHOICES = re.compile(r"\{([a-z0-9_ ,|-]+)\}")

# Help text is captured at one fixed width: argparse wraps its usage and
# option blocks to shutil.get_terminal_size(), which honors the COLUMNS
# environment variable, so an unsanitized environment would leak the invoking
# terminal's geometry into the docs page.
_HELP_COLUMNS = 80


def run_help(runner: list[str], args: list[str]) -> str:
    # Sanitized environment so CI and local runs are byte-identical:
    # COLUMNS pins argparse's wrapping width (see _HELP_COLUMNS), LC_ALL the
    # locale, NO_COLOR drops terminal color hints.
    env = dict(os.environ)
    env["COLUMNS"] = str(_HELP_COLUMNS)
    env["LC_ALL"] = "C"
    env["NO_COLOR"] = "1"
    proc = subprocess.run(
        [*runner, *args, "--help"],
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    if proc.returncode != 0:
        joined = " ".join([*runner, *args])
        raise SystemExit(
            f"error: `{joined} --help` exited {proc.returncode}: "
            f"{proc.stderr.strip()[:400]}"
        )
    return proc.stdout.rstrip("\n")


def discover_sections(text: str) -> list[str]:
    """Subcommand names from a ``Commands:``/``commands:`` help section."""
    names: list[str] = []
    in_section = False
    for line in text.splitlines():
        if line.strip() in ("Commands:", "commands:"):
            in_section = True
            continue
        if not in_section:
            continue
        if not line.strip() or not line.startswith("  "):
            break
        match = _NAME_LIST.match(line.strip())
        if not match:
            break
        for name in match.group(1).split(","):
            # clap auto-adds `help`, which is a meta-command and rejects --help.
            if name.strip() != "help":
                names.append(name.strip())
    return names


def usage_stanza(text: str) -> str | None:
    """The first ``usage:`` line plus its continuation lines.

    argparse wraps a long usage line (e.g. a wide ``{a,b,c}`` subcommand
    choices group) onto indented continuation lines, width-dependent, so the
    stanza has to be reassembled before it can be searched.
    """
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if line.strip().startswith("usage:"):
            stanza = [line]
            for continuation in lines[i + 1 :]:
                if not continuation.strip():
                    break
                stanza.append(continuation)
            return "\n".join(stanza)
    return None


def discover_argparse(text: str) -> list[str]:
    """Subcommand names from the top-level usage stanza's ``{a,b,c}`` group."""
    stanza = usage_stanza(text)
    if stanza is None:
        return []
    match = _ARGPARSE_CHOICES.search(stanza)
    if not match:
        return []
    return [choice.strip() for choice in match.group(1).split(",")]


def command_paths(runner: list[str], max_depth: int) -> list[list[str]]:
    """Depth-first subcommand paths, e.g. ``[["convert"], ["debug"], ["debug", "diff"]]``."""
    top = run_help(runner, [])
    discovered = discover_sections(top) or discover_argparse(top)
    paths: list[list[str]] = []
    stack = [[name] for name in reversed(discovered)]
    while stack:
        path = stack.pop()
        paths.append(path)
        if len(path) >= max_depth:
            continue
        # Nested discovery only for section-style help (clap/npm); argparse
        # usage lines also use {…} for option choices, which would misparse.
        if discover_sections(run_help(runner, path)):
            children = discover_sections(run_help(runner, path))
            stack.extend([*path, name] for name in reversed(children))
    return paths


def build_block(runner: list[str], display: str, max_depth: int) -> str:
    parts: list[str] = []

    def section(args: list[str]) -> None:
        title = " ".join([display, *args])
        parts.extend(["", f"#### `{title}`", "", "```text", f"$ {title} --help"])
        parts.append(run_help(runner, args))
        parts.append("```")

    section([])
    for path in command_paths(runner, max_depth):
        section(path)
    return "\n".join(parts).strip("\n")


def replace_block(text: str, marker: str, content: str) -> str:
    start = START.format(marker=marker)
    end = END.format(marker=marker)
    i = text.find(start)
    j = text.find(end)
    if i == -1 or j == -1 or j < i:
        raise SystemExit(
            f"error: {start} … {end} markers not found for marker {marker!r}"
        )
    return text[: i + len(start)] + "\n" + content + "\n" + text[j:]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--runner",
        nargs="+",
        required=True,
        help="argv prefix that launches the CLI (e.g. `target/debug/jubarte`)",
    )
    parser.add_argument("--display", required=True, help="command name to show")
    parser.add_argument("--file", required=True, help="markdown file to update")
    parser.add_argument("--marker", required=True, help="gen block marker id")
    parser.add_argument(
        "--max-depth",
        type=int,
        default=2,
        help="how deep to recurse into nested subcommands (default 2)",
    )
    args = parser.parse_args()

    block = build_block(args.runner, args.display, args.max_depth)
    text = open(args.file, encoding="utf-8").read()
    updated = replace_block(text, args.marker, block)
    if updated != text:
        open(args.file, "w", encoding="utf-8").write(updated)
        print(f"gen_cli_docs: updated {args.file} block {args.marker!r}")
    else:
        print(f"gen_cli_docs: {args.file} block {args.marker!r} already current")


if __name__ == "__main__":
    main()
