#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""First-ink-row and band-pitch on page 1 (planning/plan.md Step 1 diagnostics).

The algorithm is the one behind the report.md `1st line Δpx` / `pitch` columns:
at 150 DPI, first non-white row, and the median gap between ink bands.

    python3 scripts/page1_delta.py ref.pdf cand.pdf
"""

from __future__ import annotations

import argparse
import statistics
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class InkDelta:
    first_ink: int | None
    pitch_ref: float | None = None
    pitch_cand: float | None = None


def first_ink_row(mask: list[bool]) -> int | None:
    for index, ink in enumerate(mask):
        if ink:
            return index
    return None


def _band_starts(mask: list[bool]) -> list[int]:
    starts: list[int] = []
    prev = False
    for index, ink in enumerate(mask):
        if ink and not prev:
            starts.append(index)
        prev = ink
    return starts


def band_pitch(mask: list[bool]) -> float | None:
    starts = _band_starts(mask)
    if len(starts) < 3:
        return None
    gaps = [b - a for a, b in zip(starts, starts[1:])]
    return float(statistics.median(gaps))


def ink_delta(ref: list[bool], cand: list[bool]) -> InkDelta:
    r0 = first_ink_row(ref)
    c0 = first_ink_row(cand)
    first = None if r0 is None or c0 is None else c0 - r0
    return InkDelta(
        first_ink=first,
        pitch_ref=band_pitch(ref),
        pitch_cand=band_pitch(cand),
    )


def ppm_mask(ppm: Path) -> list[bool]:
    data = ppm.read_bytes()
    buf = data.split(b"\n")
    magic = buf[0]
    idx = 1
    while idx < len(buf) and buf[idx].startswith(b"#"):
        idx += 1
    dims = buf[idx].split()
    width, height = int(dims[0]), int(dims[1])
    idx += 1
    while idx < len(buf) and buf[idx].startswith(b"#"):
        idx += 1
    maxval = int(buf[idx])
    idx += 1
    raw = b"\n".join(buf[idx:])
    if magic == b"P6":
        nchan = 3
    elif magic == b"P5":
        nchan = 1
    else:
        raise ValueError(f"unsupported ppm {magic!r}")
    row_bytes = width * nchan
    mask = []
    for y in range(height):
        row = raw[y * row_bytes : (y + 1) * row_bytes]
        if nchan == 1:
            ink = any(b < maxval for b in row)
        else:
            ink = any(
                row[x] < maxval or row[x + 1] < maxval or row[x + 2] < maxval
                for x in range(0, len(row) - 2, 3)
            )
        mask.append(ink)
    return mask


def raster_page1(pdf: Path, scratch: Path, dpi: int = 150) -> list[bool]:
    out = scratch / (pdf.stem + ".ppm")
    subprocess.run(
        ["mutool", "draw", "-r", str(dpi), "-F", "ppm", "-o", str(out), str(pdf), "1"],
        check=True,
        capture_output=True,
    )
    # mutool may write stem1.ppm when given a %d-less path plus a page list.
    if not out.is_file():
        candidates = list(scratch.glob("*.ppm"))
        if not candidates:
            raise FileNotFoundError(f"mutool did not write a ppm for {pdf}")
        out = candidates[0]
    return ppm_mask(out)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Page-1 first-ink-row and band-pitch delta (candidate minus reference)."
    )
    parser.add_argument("reference", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--dpi", type=int, default=150)
    args = parser.parse_args(argv)
    with tempfile.TemporaryDirectory(prefix="page1_ref_") as rtmp, tempfile.TemporaryDirectory(
        prefix="page1_cand_"
    ) as ctmp:
        ref_mask = raster_page1(args.reference, Path(rtmp), args.dpi)
        cand_mask = raster_page1(args.candidate, Path(ctmp), args.dpi)
    delta = ink_delta(ref_mask, cand_mask)
    print(
        f"first_ink_delta_px\t{delta.first_ink}\n"
        f"pitch_ref_px\t{delta.pitch_ref}\n"
        f"pitch_cand_px\t{delta.pitch_cand}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
