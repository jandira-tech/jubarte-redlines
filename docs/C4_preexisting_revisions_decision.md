<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# C4 decision memo — pre-existing tracked changes (`suggesting_*`)

**Status:** **DEFERRED — waiting on Arthur**
**Date:** 2026-07-16
**Class:** C4-preexisting-revisions (`suggesting_*` fixtures)
**Plan task:** B4

## Problem

Word Compare keeps input `w:ins`/`w:del` as **history** on the redline. Jubarte
(and the PowerTools lineage) **accepts** revisions before diffing. That changes
what `accept(redline)` reconstructs and is the architectural root of the C4
score gap (word_based: ~8 fixtures, several mid-40s–80s).

Changing this is not a local fold/chrome fix — it alters the public compare
contract.

## Options (forensics-only estimates)

| Option | Behavior | Est. word_based lift | Risk |
|---|---|---:|---|
| **A. Keep accept-first + document** | Status quo; C4 remains open in ledger / KNOWN_ISSUES | 0 | None |
| **B. Carry A-side pending dels as history** (w14/w15-style stamps) | Surface some input dels without full merge | ~+0.3–0.8 mean | Medium — accept() reconstruction partial |
| **C. Full merge semantics** (Word-like keep history) | Input revisions survive as nested history | ~+1.0–2.0 mean if all 8 recover | **High** — changes `accept(redline)≡B` invariant |

Estimates are from forensics only (fixture scores under accept-first vs Word
oracle shape); not measured re-scores of implemented options.

## Recommendation until decision

- **Do not implement B or C** in this plan.
- Leave C4 members in `docs/bench_classes.md` with status **open / deferred**.
- Ratchet-1 and ship bar must be met **without** treating C4 as fixed.

## Decision needed

Arthur picks A / B / C (or a variant). Implementation follows the anti-overfitting
protocol only after that pick is written here.
