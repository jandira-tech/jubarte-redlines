<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# DOCX→PDF mini-bench (60 stems)

Working measurement set for raising jubarte ITT on the official
`docx_to_pdf_no_redline_docs` Word-oracle track.

## Membership

Pinned in `membership.json` / `membership.txt`. **Do not change the set
until 20 mini-runs are logged.**

Selection from `neurotic_docx_bench/results/docx_to_pdf_no_redline.json`
(jubarte 0.7.0, ITT mean 66.25 / median 67.39, n=398):

1. The 50 lowest per-doc scores (ties broken by stem).
2. Every official score **> 90** (baseline has 6).
3. Fill the 10-control budget with the next-highest official scores
   (`next_highest_fill`). After a later full-track report has ≥10 files
   >90, replace those fillers with a random >90 sample.

Mini-bench mean/median is **not** the 90/90 stop. That gate is the full
398-stem official track.

## Run

Rebuild `target/release/jubarte` first, then from `neurotic_docx_bench`:

```bash
uv run python ../jubarte-redlines/tools/mini_bench/run.py \
  --converter ../jubarte-redlines/target/release/jubarte \
  --log ../jubarte-redlines/tools/mini_bench/runs.jsonl
```

Each run converts with the shipped `jubarte convert` binary and scores
with the official Word-oracle pipeline (`score_folders_plain`, dpi 144).
Ras ters are deleted after scoring. The log is append-only JSONL.

Do not start a full-track `docx_to_pdf_no_redline_docs` eval of a changed
converter until `runs.jsonl` contains 20 completed rows.
