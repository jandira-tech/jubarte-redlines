#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Convert the 76 docxide fixtures and/or the 398 corpus, score vs Word PDFs.

Sibling checkouts (not vendored):

    <T>/docxide-pdf/tests/fixtures/cases/*/input.docx + reference.pdf
    <T>/neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/

    python3 scripts/convert_sweep.py 76
    python3 scripts/convert_sweep.py 398
    python3 scripts/convert_sweep.py 76 --fast          # skip case13 (205 pp)
    python3 scripts/convert_sweep.py 76 --compare tools/convert_baseline_76.tsv

Rasters are deleted after scoring. A row drop > 1.0 Jaccard, a mean drop
> 0.2, or a convert failure is a regression (planning/plan.md ground rules).
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import statistics
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
JUBARTE = HERE.parent
T = JUBARTE.parent
DEFAULT_DOCXIDE = T / "docxide-pdf"
DEFAULT_CORPUS = (
    T
    / "neurotic_docx_bench"
    / "corpus"
    / "no_comments_pdf_was_generated_by_word"
)
DEFAULT_SCORER = (
    T
    / "neurotic_docx_bench"
    / "src"
    / "neurotic_docx_bench"
    / "utils"
    / "docxide-metrics"
    / "target"
    / "release"
    / "docxide-metrics"
)
DEFAULT_JUBARTE_BIN = JUBARTE / "target" / "release" / "jubarte"
FAST_SKIP = frozenset({"case13"})
FIXTURE_LIST = "docx_to_pdf_no_redline_fixtures.txt"


@dataclass(frozen=True)
class Job:
    stem: str
    docx: Path
    ref: Path


@dataclass(frozen=True)
class ScoreRow:
    stem: str
    jaccard: float
    ssim: float
    text_boundary: float


@dataclass
class Ratchet:
    ok: bool
    regressions: list[str] = field(default_factory=list)
    mean_delta: float = 0.0
    mean_now: float = 0.0
    mean_base: float = 0.0
    failed: list[str] = field(default_factory=list)


def discover_76(docxide_root: Path, *, fast: bool = False) -> list[Job]:
    cases = docxide_root / "tests" / "fixtures" / "cases"
    if not cases.is_dir():
        return []
    jobs: list[Job] = []
    for case_dir in sorted(cases.iterdir()):
        if not case_dir.is_dir():
            continue
        if fast and case_dir.name in FAST_SKIP:
            continue
        docx = case_dir / "input.docx"
        ref = case_dir / "reference.pdf"
        if docx.is_file() and ref.is_file():
            jobs.append(Job(stem=case_dir.name, docx=docx, ref=ref))
    return jobs


def discover_76_or_skip(
    docxide_root: Path, *, fast: bool = False
) -> tuple[list[Job], str]:
    cases = docxide_root / "tests" / "fixtures" / "cases"
    if not cases.is_dir():
        return [], f"missing {cases} (clone docxide-pdf next to jubarte-redlines)"
    return discover_76(docxide_root, fast=fast), ""


def _pool_dirs(kind: str, corpus_root: Path) -> tuple[Path, Path]:
    if kind == "source":
        return corpus_root / "docx_source", corpus_root / "pdf_source"
    if kind == "source_randomized":
        return (
            corpus_root / "docx_source_randomized",
            corpus_root / "pdf_source_randomized",
        )
    raise ValueError(f"unknown fixture kind {kind!r}")


def discover_398(corpus_root: Path) -> list[Job]:
    listing = corpus_root / FIXTURE_LIST
    if not listing.is_file():
        return []
    jobs: list[Job] = []
    for line in listing.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        kind, stem = line.split("\t", 1)
        docx_dir, pdf_dir = _pool_dirs(kind, corpus_root)
        docx = docx_dir / f"{stem}.docx"
        ref = pdf_dir / f"{stem}.pdf"
        if docx.is_file() and ref.is_file():
            jobs.append(Job(stem=f"{kind}__{stem}", docx=docx, ref=ref))
    return jobs


def discover_398_or_skip(corpus_root: Path) -> tuple[list[Job], str]:
    listing = corpus_root / FIXTURE_LIST
    if not listing.is_file():
        return [], f"missing {listing} (clone neurotic_docx_bench next to jubarte-redlines)"
    return discover_398(corpus_root), ""


def write_tsv(rows: list[ScoreRow], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["stem\tjaccard\tssim\ttext_boundary"]
    for row in rows:
        lines.append(
            f"{row.stem}\t{row.jaccard:.4f}\t{row.ssim:.4f}\t{row.text_boundary:.4f}"
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def read_tsv(path: Path) -> list[ScoreRow]:
    rows: list[ScoreRow] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#") or line.startswith("stem\t"):
            continue
        stem, jaccard, ssim, text_boundary = line.split("\t")
        rows.append(
            ScoreRow(
                stem=stem,
                jaccard=float(jaccard),
                ssim=float(ssim),
                text_boundary=float(text_boundary),
            )
        )
    return rows


def compare_to_baseline(
    now: list[ScoreRow],
    base: list[ScoreRow],
    failed: list[str] | None = None,
    max_drop: float = 1.0,
    max_mean_drop: float = 0.2,
) -> Ratchet:
    failed = list(failed or [])
    now_map = {r.stem: r for r in now}
    base_map = {r.stem: r for r in base}
    regressions: list[str] = []
    deltas: list[float] = []
    for stem, brow in base_map.items():
        nrow = now_map.get(stem)
        njac = 0.0 if nrow is None else nrow.jaccard
        delta = njac - brow.jaccard
        deltas.append(delta)
        if delta < -max_drop:
            regressions.append(stem)
    mean_now = statistics.mean(r.jaccard for r in now) if now else 0.0
    mean_base = statistics.mean(r.jaccard for r in base) if base else 0.0
    mean_delta = mean_now - mean_base
    ok = not regressions and mean_delta >= -max_mean_drop and not failed
    return Ratchet(
        ok=ok,
        regressions=regressions,
        mean_delta=mean_delta,
        mean_now=mean_now,
        mean_base=mean_base,
        failed=failed,
    )


def _pct(value: object) -> float:
    if value is None:
        return 0.0
    number = float(value)
    return number * 100.0 if number <= 1.0 else number


def convert_and_score(
    jobs: list[Job],
    *,
    jubarte: Path,
    scorer: Path,
    workers: int,
) -> tuple[list[ScoreRow], list[str]]:
    work = Path(tempfile.mkdtemp(prefix="convert_sweep_"))
    failed: list[str] = []
    scorer_jobs = []
    try:
        for job in jobs:
            out = work / f"{job.stem}.pdf"
            proc = subprocess.run(
                [str(jubarte), "convert", str(job.docx), "-o", str(out), "--force"],
                capture_output=True,
                text=True,
                check=False,
            )
            if proc.returncode != 0 or not out.is_file():
                failed.append(job.stem)
            scorer_jobs.append(
                {
                    "stem": job.stem,
                    "oracle": str(job.ref),
                    "candidate": str(out),
                }
            )
        jobs_path = work / "jobs.json"
        scores_path = work / "scores.json"
        scratch = work / "scratch"
        scratch.mkdir()
        jobs_path.write_text(json.dumps(scorer_jobs), encoding="utf-8")
        subprocess.run(
            [
                str(scorer),
                "--jobs",
                str(jobs_path),
                "--scratch",
                str(scratch),
                "--out",
                str(scores_path),
                "--workers",
                str(workers),
            ],
            check=True,
        )
        raw = json.loads(scores_path.read_text(encoding="utf-8"))
        raw_list = raw if isinstance(raw, list) else list(raw.values())
        by_stem = {s["stem"]: s for s in raw_list}
        rows = []
        for job in jobs:
            score = by_stem.get(job.stem, {})
            rows.append(
                ScoreRow(
                    stem=job.stem,
                    jaccard=_pct(score.get("jaccard")),
                    ssim=_pct(score.get("ssim")),
                    text_boundary=_pct(score.get("text_boundary")),
                )
            )
        return rows, failed
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Score jubarte convert against Word PDFs on the 76 and/or 398 sets."
    )
    parser.add_argument("set", choices=("76", "398", "both"))
    parser.add_argument("--fast", action="store_true", help="skip case13 (205 pages)")
    parser.add_argument("--jubarte", type=Path, default=DEFAULT_JUBARTE_BIN)
    parser.add_argument("--scorer", type=Path, default=DEFAULT_SCORER)
    parser.add_argument("--docxide", type=Path, default=DEFAULT_DOCXIDE)
    parser.add_argument("--corpus", type=Path, default=DEFAULT_CORPUS)
    parser.add_argument("--out", type=Path, help="write TSV here (default stdout + tools/)")
    parser.add_argument("--compare", type=Path, help="baseline TSV to ratchet against")
    parser.add_argument("--workers", type=int, default=max(1, (os.cpu_count() or 4) // 2))
    parser.add_argument(
        "--list-only",
        action="store_true",
        help="print stems and paths; do not convert",
    )
    args = parser.parse_args(argv)

    jobs: list[Job] = []
    if args.set in ("76", "both"):
        found, reason = discover_76_or_skip(args.docxide, fast=args.fast)
        if reason:
            print(reason, file=sys.stderr)
            if args.set == "76":
                return 2
        jobs.extend(found)
    if args.set in ("398", "both"):
        found, reason = discover_398_or_skip(args.corpus)
        if reason:
            print(reason, file=sys.stderr)
            if args.set == "398":
                return 2
        jobs.extend(found)

    print(f"{len(jobs)} jobs", file=sys.stderr)
    if args.list_only:
        for job in jobs:
            print(f"{job.stem}\t{job.docx}\t{job.ref}")
        return 0 if jobs else 2

    if not args.jubarte.is_file():
        print(f"missing jubarte binary: {args.jubarte} (cargo build --release)", file=sys.stderr)
        return 2
    if not args.scorer.is_file():
        print(f"missing scorer: {args.scorer}", file=sys.stderr)
        return 2

    rows, failed = convert_and_score(
        jobs, jubarte=args.jubarte, scorer=args.scorer, workers=args.workers
    )
    out = args.out
    if out is None and args.set != "both" and args.compare is None:
        out = JUBARTE / "tools" / f"convert_baseline_{args.set}.tsv"
    if out is not None:
        write_tsv(rows, out)
        print(f"wrote {out} ({len(rows)} rows)", file=sys.stderr)
    else:
        write_tsv(rows, Path("/dev/stdout"))

    if failed:
        print(f"convert failures: {failed}", file=sys.stderr)

    if args.compare:
        base = read_tsv(args.compare)
        ratchet = compare_to_baseline(rows, base, failed=failed)
        print(
            f"mean J: baseline {ratchet.mean_base:.2f} -> now {ratchet.mean_now:.2f} "
            f"({ratchet.mean_delta:+.2f}); regressions {len(ratchet.regressions)}; "
            f"failures {len(ratchet.failed)}",
            file=sys.stderr,
        )
        if not ratchet.ok:
            print(
                "RESULT: REGRESSION — name every dropped row in the commit message "
                "or fix it.",
                file=sys.stderr,
            )
            for stem in ratchet.regressions:
                print(f"  {stem}", file=sys.stderr)
            return 1
        print("RESULT: OK", file=sys.stderr)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
