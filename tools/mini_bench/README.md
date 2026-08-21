<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# DOCX→PDF mini-bench (60 stems × 2 official tracks)

Working measurement set for raising jubarte ITT on both official
Word-oracle convert tracks:

- `docx_to_pdf_no_redline_docs` — `membership.json` / `membership.txt`
- `docx_to_pdf` (accepted Word redline + randomized redline) —
  `membership_redline.json` / `membership_redline.txt`

## Membership

Each file is 60 unique stems:

1. The 50 lowest official jubarte per-doc scores (ties broken by stem).
2. Every official score **> 90**, capped at 10 controls.
3. If fewer than 10 files score >90, fill with the next-highest official
   scores (`next_highest_fill`).

Pinned from the sibling `neurotic_docx_bench/results/` jubarte `per_doc`
reports. Rebuild with:

```bash
python3 tools/mini_bench/select.py --track docx_to_pdf_no_redline_docs
python3 tools/mini_bench/select.py --track docx_to_pdf
```

This goal's 90/90 gate is ITT mean ≥ 90 and ITT median ≥ 90 on each
60-stem sample (convert failures score 0), not the full 398/428-stem
tracks.

## Run

Rebuild `target/release/jubarte` first (default `target/`, one Cargo
process), then from `neurotic_docx_bench`:

```bash
uv run python ../jubarte-redlines/tools/mini_bench/run.py \
  --converter ../jubarte-redlines/target/release/jubarte

uv run python ../jubarte-redlines/tools/mini_bench/run.py \
  --track docx_to_pdf \
  --converter ../jubarte-redlines/target/release/jubarte
```

Each run converts with the shipped `jubarte convert` binary and scores
with the official Word-oracle pipeline (`score_folders_plain`, dpi 144).
Rasters are deleted after scoring. The log is append-only JSONL.
