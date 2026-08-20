#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Score the pinned 60-stem mini-bench with shipped `jubarte convert`.

Must be launched from neurotic_docx_bench so the official Word-oracle
scorer (`docx_to_pdf.run_eval`) is on the path:

  uv run python ../jubarte-redlines/tools/mini_bench/run.py \\
    --converter ../jubarte-redlines/target/release/jubarte
"""

from __future__ import annotations

import argparse
import hashlib
import json
import statistics
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEFAULT_MEMBERSHIP = HERE / "membership.json"
DEFAULT_LOG = HERE / "runs.jsonl"


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _version(binary: Path) -> str | None:
    try:
        proc = subprocess.run(
            [str(binary), "--version"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    text = (proc.stdout or proc.stderr or "").strip()
    return text.splitlines()[0] if text else None


def _next_index(log_path: Path) -> int:
    if not log_path.is_file():
        return 1
    last = 0
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        row = json.loads(line)
        last = max(last, int(row.get("run") or 0))
    return last + 1


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--converter", type=Path, required=True)
    parser.add_argument("--membership", type=Path, default=DEFAULT_MEMBERSHIP)
    parser.add_argument("--log", type=Path, default=DEFAULT_LOG)
    parser.add_argument("--work-dir", type=Path, default=None)
    parser.add_argument("--json-out", type=Path, default=None)
    parser.add_argument("--run", type=int, default=None, help="run index (default: next)")
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--convert-workers", type=int, default=8)
    parser.add_argument("--dpi", type=int, default=144)
    args = parser.parse_args()

    converter = args.converter.expanduser().resolve()
    if not converter.is_file():
        raise SystemExit(f"converter not found: {converter}")

    membership = json.loads(args.membership.read_text(encoding="utf-8"))
    wanted = [row["stem"] for row in membership["stems"]]
    if len(wanted) != 60 or len(set(wanted)) != 60:
        raise SystemExit(f"membership must be 60 unique stems, got {len(wanted)}")

    from neurotic_docx_bench.docx_to_pdf import load_fixtures, run_eval

    fixtures = load_fixtures(track="docx_to_pdf_no_redline_docs")
    by_stem = {item.stem: item for item in fixtures}
    missing = [stem for stem in wanted if stem not in by_stem]
    if missing:
        raise SystemExit(f"stems not on official track: {missing[:5]}")
    items = [by_stem[stem] for stem in wanted]

    run_idx = args.run if args.run is not None else _next_index(args.log)
    work_dir = args.work_dir or (HERE / "work" / f"run_{run_idx:02d}")
    json_out = args.json_out or (work_dir / "mini_bench.json")

    report = run_eval(
        json_out,
        converter=converter,
        tools=("jubarte",),
        jobs=args.jobs,
        dpi=args.dpi,
        work_dir=work_dir,
        fixtures=items,
        resume=False,
        convert_workers=args.convert_workers,
        track="docx_to_pdf_no_redline_docs",
    )
    tool = report["tools"]["jubarte"]
    per_doc = tool["per_doc"]
    vals = [float(per_doc[stem]) for stem in wanted]
    row = {
        "run": run_idx,
        "generated_at": datetime.now(UTC).isoformat(),
        "track": report.get("track"),
        "converter": str(converter),
        "converter_sha256": _sha256(converter),
        "version": _version(converter) or tool.get("version"),
        "itt_n": len(wanted),
        "n_scored": int(tool.get("n_scored") or 0),
        "mean": round(statistics.mean(vals), 4),
        "median": round(statistics.median(vals), 4),
        "failures": int(tool.get("failures") or 0),
        "per_doc": {stem: per_doc[stem] for stem in wanted},
        "json_out": str(json_out),
    }
    args.log.parent.mkdir(parents=True, exist_ok=True)
    with args.log.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(row, sort_keys=True) + "\n")
    print(
        f"mini-run {run_idx}  n={row['itt_n']}  mean={row['mean']}  "
        f"median={row['median']}  fail={row['failures']}  → {args.log}",
        flush=True,
    )


if __name__ == "__main__":
    try:
        main()
    except ImportError as exc:
        sys.exit(f"run via `uv run` from neurotic_docx_bench: {exc}")
