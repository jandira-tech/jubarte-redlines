<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Known issues

Engine defects and unresolved design conflicts.

> **Re-checked 2026-10-01 against 0.10.1.** Open items below are the complete set.

## Open

### 6. Dependabot: glib 0.18.5 in the desktop app (RUSTSEC-2024-0429) — **OPEN, blocked upstream**

**Alert:** GitHub reports one moderate vulnerability, `glib` ≥0.15 <0.20
("Unsoundness in `Iterator` and `DoubleEndedIterator` impls for
`glib::VariantStrIter`") in `jubarte-app/src-tauri/Cargo.lock`.

**Why it stays open:** glib is only there through `gtk` 0.18, which Tauri 2
uses for its Linux webview. gtk-rs/gtk3-rs was archived in March 2024, so no
gtk release can ever move to glib ≥0.20. The alert clears only when Tauri
moves its Linux build to GTK4. The engine, CLI, Python and WASM packages do
not depend on glib. The desktop app compiles it for Linux only, and nothing
in it calls `VariantStrIter`.

**Action:** re-check on each Tauri upgrade. Dismissing the alert as
"tolerable risk" is the maintainer's call.

## Settled

Items 1–5 are the engine's settled history, one line each; the full story of
every one is in [CHANGELOG.md](CHANGELOG.md).

1. **MovedSource / `w:moveFrom` text kind** — settled 2026-07-16 (Word
   wins: `w:t` under `w:moveFrom`, `w:delText` under `w:del`); enforced by
   Ring 1. See [CHANGELOG.md](CHANGELOG.md).
2. **Multi-del boundary fold** — resolved 2026-09-28 against Word's own
   redlines, closing the positional-zip defect ([0.10.0] Fixed). See
   [CHANGELOG.md](CHANGELOG.md).
3. **Free-mesh double-consumption — one A-side atom claimed by two
   paragraphs** — fixed 2026-09-27, with the text round-trips found beside
   it ([0.9.3] Fixed). See [CHANGELOG.md](CHANGELOG.md).
4. **Internal `Unid` scratch shipped as an undeclared `w:Unid`
   attribute** — fixed by b7fedc78, closed 2026-09-28 ([0.9.3] Fixed). See
   [CHANGELOG.md](CHANGELOG.md).
5. **Ring 3: five corpus redlines Word refused to open** — fixed
   2026-09-05 (`w:instrText` under `w:del` became `w:delInstrText`,
   [0.9.0] Fixed). See [CHANGELOG.md](CHANGELOG.md).

Two older notes are settled as well: external hyperlinks keep their `r:id`
([0.1.0] Fixed), and clean adjacent tables are merged through the
accept/reject pipeline ([0.10.1] Fixed).
