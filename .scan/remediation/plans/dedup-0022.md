# Remediation plan — dedup-0022

Scan folder: `.scan/artifacts/deep_discovery/dedup/dedup-0022/`
Reducer: `dedup-0022`, claimed worker `3db18731-b2e5-4085-a323-3e4478915f09`
(prompt.md). Repository HEAD at preparation time: `3c6ad3fc` (clean tree).

## What this folder holds

`output/result.json` carries 59 aggregated findings. Fourteen of them carry
evidence refs contributed by this round's worker (`dedup0022-*`). Every one of
those fourteen already exists, by title, in the previous round's aggregate
(verified with jq against `dedup-0021/output/result.json`: all 14 titles
present; zero titles new in round 22). This round strengthened existing
findings with new attack framings and evidence locations; it did not create a
new root issue. Under the reducer's own remediation-subsumption rule, the root
fixes for those findings belong to the folders that first carried them
(`dedup-0002` … `dedup-0021`), and one fix per root cause covers this round's
added framings.

The table below records that triage. The one finding whose *decisive* evidence
— the vulnerable code span itself — is this round's own contribution is fixed
here, in full TDD red/green form, by the accompanying patch.

| # | Finding (title in result.json) | This round's contribution (`dedup0022-*` refs) | Decision |
|---|---|---|---|
| 31 | Converting legacy DOC text with a long numeric prefix causes quadratic work | `dedup0022-n12-escape-prefix` is the **only** evidence ref pointing at the vulnerable rescan itself (`src/legacy_doc.rs:575-598`); the two inherited refs name only the entry (`legacy_doc.rs:225-236`) and control sites | **Fixed in this patch** |
| 2 | Shallow malformed XML can bypass admission and overflow the DOM parser stack | `n9-xml-*`: a new `<a b=x></a>x>` shallow-sibling framing of the same tokenizer/depth split (`src/admission.rs:788-820`, `src/xmllinq/parse.rs:230-325,554-593,660-663`) | Not patched here: root fix (unify tolerant tokenization + construction depth limit) is owned by rounds ≤ 21, which carry the same finding |
| 3 | Reading or converting nested legacy DOC fields can monopolize the processing thread | `n8-field-*`: re-confirmed locations (`src/legacy_doc.rs:180-225,1591-1621`; CLI hint `src/bin/jubarte.rs:2735-2750,3120-3135`) | Not patched: O(1) field-frame counter + budgets owned by earlier rounds |
| 4 | Large Chicago numbering values can exhaust memory while reading or rendering a DOCX | `n2-marker-*`: a second static framing (`start=2147483647` → 1.5 GiB of `‡`) of the same unbounded repeat (`src/convert/mod.rs:6015-6019`) | Not patched: formatter bound owned by earlier rounds |
| 9 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | `n7-legacy-*`: re-confirmed FIB/piece/expansion sites (`src/legacy_doc.rs:907-925,1082-1113,1176-1227`) | Not patched: piece-expansion budgets owned by earlier rounds |
| 10 | ZIP footer and count interpretation mismatch bypasses entry-count admission | `n5-eocd-*`: a false-EOCD-in-comment framing of the same preindex/reader split (`src/admission.rs:399-407,444-455,590-602`) | Not patched: reader-authoritative EOCD owned by earlier rounds |
| 16 | npm plan recording follows a predictable staging symlink | `n10-node-*`: re-confirmed Node CLI staging sites (`jubarte-wasm/cli/bin/jubarte-redlines.mjs:408-427,442-445`) | Not patched: exclusive-creation staging owned by round 19, which carried the finding and its native-side counterpart |
| 18 | Repeated HTML chunk references bypass conversion memory budgets | `n3-import-*`: nesting variant cross-referenced to the dedicated depth finding (altchunk `543-551`, `src/convert/mod.rs:13224-13238`) | Not patched: cumulative expansion budgets owned by earlier rounds; the depth variant is finding "Rendering nested HTML tables bypasses the XML depth limit", also pre-existing |
| 21 | Public Strict-format probe inflates unbounded document headers | `n6-probe-*`: re-confirmed public export (`src/lib.rs:138-146`) and unbounded read (`src/strict_translation.rs:670-703,719-728`) | Not patched: bounded probe owned by earlier rounds |
| 26 | Unbounded document table spans exhaust memory during rendering | `n0-*`: re-confirmed span/skip/colspan sites (`src/convert/mod.rs:13601-13615,14211-14215,14340-14345`; `src/convert/altchunk.rs:469-490,558-567`) | Not patched: span/output budgets owned by earlier rounds |
| 27 | Repeated relationship IDs trigger quadratic repair work | `n11-rels-*`: re-confirmed repair sites (`src/opc/mod.rs:99-125,222-234,520-544`) | Not patched: HashSet-tracked repair owned by earlier rounds (a second folder patching the same function would conflict) |
| 28 | Unbounded list levels cause billions of iterations or allocations | `n1-level-*`: re-confirmed ilvl sites (`src/convert/mod.rs:5518-5532,5610-5627,6618-6621,12707-12718`) | Not patched: defined-key level resolution owned by earlier rounds |
| 32 | Ragged legacy DOC tables expand into an unbounded padded rectangle | `n13-padding-*`: re-confirmed padding sites plus the output path (`src/legacy_doc.rs:430-460,511-527,1623-1648`) | Not patched: generated-cell quota owned by rounds 18/21, which carried the finding |
| 42 | Privacy scrubbing can return identifying data in valid relocated DOCX parts | `n4-scrub-*`: re-confirmed filename-filtered selection (`src/scrub.rs:109-140,211-216,271-289,353-385`) and the sharing contracts (`src/cli.rs:1553-1557`; `jubarte-python/.../document.py:202-226`) | Not patched: relationship-based part discovery owned by earlier rounds |

If any of those earlier-round folders close without landing their root fix,
their plans — not this one — are the place to pick the work up; re-fixing the
same functions here would produce conflicting patches on identical hunks.

---

## The vulnerability fixed here (finding 31)

**Finding.** "Converting legacy DOC text with a long numeric prefix causes
quadratic work" (`result.json` findings[31]; severity medium; source findings
`5199dcfe-…:3` and `3db18731-…:12`). Root cause, in the finding's words: the
outer character loop "recomputes whether the entire prefix is ASCII digits at
every `.` or `)`", so a long digit run followed by many delimiters makes
`escape_line` rescan the digit prefix once per delimiter.

**Code.** `src/legacy_doc.rs:575-598` (`escape_line`, read at this HEAD):

```rust
'.' | ')' => {
    // `1.` or `1)` at the start of a line opens a list.
    let before = text.get(..index).unwrap_or("");
    !before.is_empty() && before.chars().all(|c| c.is_ascii_digit())
}
```

Every `.` or `)` re-reads the whole prefix from the start of the (trimmed)
line. The scan short-circuits at the first non-digit, so after the *first*
period, each later period still walks the entire digit run before hitting that
period. Input `D` digits followed by `P` delimiter pairs (`D + 2P` bytes)
costs `≈ D·P` character examinations: a ~120 KB line (`D = 60 000`,
`P = 30 000`, 119 999 bytes) performs exactly 1 800 029 999 character
examinations. `escape_line` runs for
every table cell and every paragraph line of `render_spans`
(`src/legacy_doc.rs:518,567`), reached from the public `read` →
`to_markdown`/`doc_to_markdown`/`doc_to_docx` path (`src/legacy_doc.rs:180-236`)
with attacker-authored Word 97-2003 bytes — a small crafted `.doc` monopolizes
the converting thread. The threat model lists "escaping" among the
independently validated resource issues (`.scan/threatmodel.md`, mitigation
section at line 1285, citing `src/legacy_doc.rs:575-598`).

**Remediation prescribed by the finding.** "Track a running all-ASCII-digits
prefix flag while iterating once … Preserve escaping behavior with linear
work."

### The failing test (red)

Tests live beside the behavior, in the existing `#[cfg(test)] mod tests` of
`src/legacy_doc.rs` (module at `src/legacy_doc.rs:1723-1725`), following the
repository's established pattern for quadratic-scan regressions: a
`#[cfg(test)] thread_local!` counter with a test-only inline increment and an
empty production twin. The counter must count the **elementary work** (each
character examined), not loop entries, so that a reintroduced prefix re-read
counts its characters and fails the linear bound — the shape of
`src/markdown/critic.rs:465-482` (`CLOSER_STEPS` / `count_closer_steps(steps)`,
whose own comment states "a scan that reintroduced the rest of the text per
opener would count that text and fail the bound") and of
`src/edit/anchor.rs:25-43` (`ANCHOR_BYTE_PROBES` / `probe_compare`, test
`v2_overlapping_search_probes_in_linear_time` at `src/edit/anchor.rs:711-739`,
bound `const C: usize = 4`). The same pattern exists in `src/edit.rs:1189-1192`
(`COMMENT_MARK_SCANS`) and `src/xmllinq/mod.rs:276-277` (`ATTR_NAME_PROBES`).
An earlier draft of this patch counted delimiter-arm entries once per
delimiter and asserted `probes <= delimiters`; that assertion holds for any
single-pass implementation, quadratic or not, so it gated nothing — the
counter was moved to the per-character pass.

1. `src/legacy_doc.rs`, test `list_prefix_scan_stays_linear` (new):
   - Builds `text = "7"×60 000 + "." + ("1."×29 999)` — 30 000 delimiters
     after a 60 000-digit run, 119 999 characters — and asserts the
     delimiter count it built.
   - Resets `LIST_PREFIX_STEPS`, calls `escape_line(&text)`, reads the
     counter back.
   - **Asserts `steps <= C × chars` with `C = 4`** (the pass that keeps the
     all-digits answer current examines each character exactly once, so the
     fixed code reports 119 999 steps against a bound of 479 996). The
     pre-fix algorithm, instrumented with the *same* counter at its rescan
     (one step per character the `all(…)` closure examines), reports
     1 800 029 999 and fails the bound by a factor of ~3 750 — that
     instrumented run is the red demonstration, with the same counter and
     the same test; without the fix it is also simply the denial of
     service, at 120 KB of input.
   - **Asserts the exact escaped output** — `"7"×60 000 + "\." + "1."×29 999`
     — pinning behavior on the pathological input itself: only the first
     delimiter follows a pure digit run.
2. `src/legacy_doc.rs`, test `escapes_list_delimiters_only_after_sole_digit_runs`
   (new): the equivalence contract between old and new decision, case by case:
   - `escape_line("12) not a list") == "12\\) not a list"` — a `)` after only
     digits is escaped;
   - `escape_line("1.2.") == "1\\.2."` — after the first escaped delimiter the
     next is literal (its prefix contains a `.`);
   - `escape_line("3.14)") == "3\\.14)"` — a `)` whose prefix contains a `.`
     is never escaped;
   - `escape_line(".") == "."` and `escape_line(").") == ")."` — a delimiter
     at the start of the line opens nothing, so nothing is escaped;
   - `escape_line("٣.") == "٣."` — only **ASCII** digits count
     (`char::is_ascii_digit`), an Arabic-Indic digit does not open a list.

   These pass before and after the change; they exist so the flag rewrite
   cannot drift from the rescanning decision it replaces.

### The minimal source change (green)

In `src/legacy_doc.rs`, `escape_line` only:

- Add, immediately above the function, the test-only counter
  `LIST_PREFIX_STEPS` and the `count_list_prefix_steps(steps)` twins
  (`#[cfg(test)]` adds `steps`, `#[cfg(not(test))]` empty and `#[inline]`),
  copying the `critic.rs`/`anchor.rs` layout including the
  `#[cfg_attr(all(coverage_nightly, test), coverage(off))]` attributes the
  repository puts on test-gated helpers.
- Replace the per-delimiter prefix rescan with a running flag, and count one
  step per character of the pass that maintains it:

```rust
let mut prefix_is_digits = true;
for (index, ch) in text.char_indices() {
    …
    '.' | ')' => {
        // `1.` or `1)` at the start of a line opens a list.
        prefix_is_digits && !at_start
    }
    …
    out.push(ch);
    // One character of the pass: keep the all-digits answer current.
    count_list_prefix_steps(1);
    if !ch.is_ascii_digit() {
        prefix_is_digits = false;
    }
}
```

Equivalence: the old decision escaped a delimiter iff its prefix was non-empty
and all its chars were `is_ascii_digit()`. `prefix_is_digits` after consuming
chars `0..index` is exactly the conjunction of `is_ascii_digit()` over that
prefix (`true` vacuously when the prefix is empty, which the `!at_start` guard
converts back to "no escape"), and it can only flip from `true` to `false`,
once a non-digit is seen. Every case in
`escapes_list_delimiters_only_after_sole_digit_runs` is decided identically by
both formulations. Work becomes one pass, `O(len)` — the rescan, the
`text.get(..index)` and the closure allocation disappear. No new indexing or
arithmetic is introduced, keeping the module's
`deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)` clean
(`src/legacy_doc.rs:8-11`; the increment inside the `#[cfg(test)]` twin and
the `C * chars` bound in the test are exempt, as `critic.rs:476` and
`anchor.rs:732-738` already are).

### The existing behaviour that must keep passing

Escaping output must be byte-identical; nothing else in the module changes.

- Unit tests in `src/legacy_doc.rs` `mod tests`:
  `escapes_markdown_significant_text` (`1. not a list` → `1\. not a list`,
  `src/legacy_doc.rs:1729-1737`), `tables_become_github_tables` (cells escape
  `|` through `escape_line`, `src/legacy_doc.rs:1768-1788`),
  `emphasis_markers_hug_the_text_and_lists_get_markers`
  (`src/legacy_doc.rs:1799-1842`), `line_breaks_become_hard_breaks`
  (`src/legacy_doc.rs:1791-1796`).
- Integration tests in `tests/legacy_doc_convert.rs` (fixture-driven, through
  `doc_to_markdown`/`doc_to_docx`):
  `markdown_escapes_what_markdown_would_read_as_syntax` — asserts
  `## 1\. Fees\n` and the `$`, `[`, `#`, `_`, `*` escapes
  (`tests/legacy_doc_convert.rs:148-161`) — this is the Word-fidelity pin for
  the changed decision; `the_docx_is_word_valid_and_keeps_the_text`
  (`tests/legacy_doc_convert.rs:164-172`, ooxml-valid package with the text
  intact); `bold_and_italic_runs_are_kept` (`:74-100`);
  `headings_lists_paragraphs_and_the_table_are_read_in_order` (`:52-72`).
- Word-fidelity note: the `.docx` produced by `doc_to_docx` is unchanged —
  `document_to_docx` round-trips through the same `blocks_to_markdown`
  (`src/legacy_doc.rs:242-251`), whose bytes the fix keeps identical.

### Verification (commands the applying engineer runs; NOT run here — cargo is
forbidden in this preparation step)

Equivalence was checked mechanically (Python, not cargo; re-verified by the
second reviewer): the old and new decisions were simulated over all 37,449
strings of length 0-5 over the alphabet `{'1', '9', '.', ')', 'a', '-', '٣',
' '}` — zero mismatches; every pinned case above produced its expected string
under both algorithms; and on the pathological test input (119,999
characters) the new algorithm takes exactly 119,999 steps (bound 4 × 119,999
= 479,996: pass) where the old rescan examines 1,800,029,999 characters
(simulated, and confirmed by exact arithmetic: `D + (P-1)·(D+1)`), a factor
of ~3,750 over the bound. `git apply --check` on the corrected patch from
the repository root reported clean application (run twice), and applying the
patch to a pristine copy reproduced the intended file byte for byte.

Per AGENTS.md, from the repository root, sequentially:

1. `cargo nextest run -E 'test(escapes_list_delimiters) + test(list_prefix_scan)'`
   — the two new tests (unit; in-file module).
2. `cargo nextest run -E 'test(legacy_doc)'` — the legacy unit and integration
   suites touched (`src/legacy_doc.rs`, `tests/legacy_doc_convert.rs`,
   `tests/legacy_doc.rs`).
3. `cargo fmt --check` and `cargo clippy --lib -- -D warnings` (the file also
   builds under `#[cfg(test)]`; the counter follows the `anchor.rs` shape, so
   no `#[allow]` is needed anywhere).
4. Smoke: `cargo run --quiet --bin jubarte -- --help`.

Red/green sequencing, should the patch be replayed test-first: apply the
counter hunks plus the new tests **and** instrument the old `'.' | ')'` arm
with the same counter, one step per character the rescan examines —
`before.chars().all(|c| { count_list_prefix_steps(1); c.is_ascii_digit() })`
— so `list_prefix_scan_stays_linear` fails with `steps = 1,800,029,999 >
479,996`; then apply the flag rewrite and the tests pass with
`steps = 119,999`. The counter, the test and the bound are the same in red
and in green; only the code under them changes.

## Functionality preservation

The patch changes no public API, no file format, no output bytes: `escape_line`
returns the same `String` for every input (argued case-by-case above and
pinned by the new equivalence test), only cheaper. The probe counter is
`#[cfg(test)]`-gated with an empty production twin, so production builds keep
zero runtime cost and the module's clippy `deny` attributes hold. No
`#[allow]` is introduced.
