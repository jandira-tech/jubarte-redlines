# Plan: use the Open XML SDK schema data as an oracle (minimal hand-rolling)

Status: PLANNED — nothing implemented yet. Written 2026-07-13.

## Goal

Every piece of OOXML schema knowledge that jubarte hand-rolls (element
ordering tables, namespace lists, validity rules) should be either
(a) mechanically cross-checked against the Open XML SDK's machine-readable
schema data, or (b) explicitly documented as an intentional PowerTools
divergence. New schema knowledge should be read from the data, never
transcribed from ECMA PDFs by hand.

## Non-goal (read this first)

**Do NOT replace the runtime ordering tables with schema-generated ones.**
The tables in `src/comparer/finalize.rs` are verbatim ports of PowerTools'
`PtOpenXmlUtil.cs` rank tables, quirks included (the `w14:wShadow` /
`wTextOutline` / … entries never match real markup; real markup uses
`w14:shadow` etc. — kept because the C# has them). Our oracle is
reconstruction parity with PowerTools + real-Word acceptance, not ECMA.
Regenerating the tables from schema particles would silently change output
bytes against the goldens. Known genuine divergence already found:

- `rPr` (CT_ParaRPr): PowerTools ranks `moveFrom`=5, `moveTo`=7 **before**
  `ins`=10, `del`=20; the schema particle orders `ins, del, moveFrom, moveTo`.
  Comparer semantics, keep PowerTools' order.

Same reason the engine stays on the untyped `xmllinq` DOM: PowerTools works
on XDocument, and a lossless redliner must carry unknown/extension markup
that typed structs would drop.

## The data (source of truth)

Lives in the enclosing ooxmlsdk checkout (upstream: dotnet/Open-XML-SDK
`data/`, MIT, auto-generated — see its NOTICE README; never hand-edit):

- `../data/schemas/schemas_openxmlformats_org_wordprocessingml_2006_main.json`
  — 726 WML types. Shape: `{TargetNamespace, Types: [...], Enums: [...]}`.
  Each type: `Name` (`"w:CT_PPr/w:pPr"` — complex type + qualified tag),
  `Attributes` (QName, type, `Validators`: required/range/length/version),
  `Children`, and **`Particle`** — nested `{Kind: Sequence|Choice|All|Group,
  Items: [...]}` encoding canonical child order.
- `../data/namespaces.json` — prefix↔URI table (duplicates our
  `src/namespaces.rs`).
- `../data/parts/*.json` — part/relationship metadata.

Feasibility verified 2026-07-13: `w:CT_PPr/w:pPr`'s flattened particle
matches our `pPr` hand table 1:1 (36 entries, same order); `CT_TblBorders`
matches the `tblBorders` table exactly (`top, left, start, bottom, right,
end, insideH, insideV`).

## Inventory of hand-rolled schema knowledge (audit targets)

| Where | What | Plan |
|---|---|---|
| `src/comparer/finalize.rs:3067` (`TBLPR_ORDER`) and `:3100-3260` (`rank()`: pPr 36, rPr ~43 + 5 w14 quirks, tcPr 14, tcBorders 10, tblBorders 8, pBdr 6) | child-order ranks, PowerTools-verbatim | cross-check (W1), never regenerate |
| `src/comparer/produce.rs:960` (`TBLPR_CHILD_ORDER`) | tblPr order again (subset) | cross-check (W1) + assert consistent with `TBLPR_ORDER` |
| `src/namespaces.rs` (140 lines) | ns constants | cross-check against `namespaces.json` (W1, cheap); not worth generating |
| `src/strict_translation.rs` (734 lines) | strict→transitional mapping | NOT covered by the JSON in usable form; stays hand-rolled, covered by W2 validator sweep instead |

## Workstreams

### W1 — Schema-consistency test for the hand tables (highest value/effort ratio)

Turns ~140 hand-typed entries into oracle-checked data with zero runtime
change.

1. Vendor the WML main schema JSON into `tests/data/wml_main_schema.json`
   (~1.5 MB). Record upstream provenance (ooxmlsdk repo commit / SDK
   version) in a sibling `tests/data/README.md` with the MIT attribution.
2. `tests/schema_consistency.rs` (dev-only, `serde_json` as dev-dep):
   - Flatten each container's `Particle` depth-first into an ordered name
     list (Sequence items in order; **Choice items have no defined relative
     order — treat members of one Choice as an unordered group**, compare
     only across groups).
   - Export the rank tables from the crate for the test (e.g. a
     `#[doc(hidden)] pub mod order_tables` or `#[cfg(test)]`-visible fn in
     finalize; keep the tables as the single source, don't duplicate).
   - Assert: for every pair of names present in BOTH the hand table and the
     schema list, relative order agrees — EXCEPT pairs on an explicit
     `DIVERGENCE_WHITELIST`, each entry commented with the PowerTools
     source line and the reason (start with rPr moveFrom/moveTo-before-
     ins/del; expect a few more — discover them by running the test red
     first, then whitelist deliberately, one by one, reading
     PtOpenXmlUtil.cs each time).
   - Assert: every hand-table name not in the schema list is on a
     `QUIRK_WHITELIST` (the w14 `wShadow`-family entries).
   - Assert: `TBLPR_CHILD_ORDER` (produce.rs) is order-consistent with
     `TBLPR_ORDER` (finalize.rs).
   - Bonus: assert `src/namespaces.rs` URIs appear in `namespaces.json`.
3. Red-green: write the assertions first, watch which pairs fail, verify
   each against PtOpenXmlUtil.cs before whitelisting. A failure that is NOT
   in the C# is a genuine typo in our port — fix the table (and re-run the
   golden suite before believing the fix is behavior-neutral).

Acceptance: test green; whitelist entries each cite a C# line; goldens
unchanged (`cargo test` full suite).

### W2 — Checked-in .NET OpenXmlValidator sweep (the real schema oracle)

History: the 2026-07-03 validity campaign found **146/166 benchmark
redlines had OpenXmlValidator schema errors while Word's own redlines
validate clean**; that sweep drove PR #75's fixes — but the tool was ad-hoc
and never checked in. Recreate it durably:

1. `tools/validate-cs/` — minimal console csproj on
   `DocumentFormat.OpenXml`, runs `OpenXmlValidator` (choose
   `FileFormatVersions` = Office2019 or Microsoft365; document the choice —
   it changes which extension elements are "valid") over a file or dir,
   prints one line per error (`file<TAB>part<TAB>path<TAB>description`),
   exit code = error count clamped.
2. `tools/validate-cs.sh` wrapper modeled on `tools/gen-goldens-cs.sh`:
   cd into the project dir so a `global.json` SDK pin applies (see pitfall
   below), accept a corpus dir.
3. Wire an opt-in test/script that runs it over `tests/goldens/**/*.docx`
   plus freshly-produced outputs; assert zero NEW errors vs a checked-in
   baseline file (some corpora have pre-existing quirks, e.g. the known
   math one — baseline, don't chase).
4. Local-only (needs dotnet), like the Docxodus goldens; CI skips when
   `dotnet` is absent.

Acceptance: one command validates a corpus; baseline file committed;
README section documenting the flow.

### W3 — Rust-side particle checking for CI (optional, after W1)

**Trap, verified 2026-07-13: the crates.io `ooxmlsdk` 0.11.0 `validators`
feature generates ZERO `validate_into` impls for the WML schema module**
(the 800 KB generated `schemas_openxmlformats_org_wordprocessingml_2006_main.rs`
has none; the feature ships only the trait + attribute-validator helpers).
Same in the local fork. Do not plan around it doing schema validation.

If CI needs a dotnet-free ordering check: reuse W1's vendored JSON to walk
produced `document.xml` and verify child order of the property containers
(pPr/rPr/tblPr/tcPr/…) against particles adjusted by the same whitelists.
~100 lines on quick-xml, test-only. Defer until W2 shows CI actually
misses regressions without it — the real validator is strictly stronger.

### W4 — Schema-driven data for future features

When accept/reject or formatting-merge work needs enum values, attribute
defaults, or Office-version gating: read them from `Enums` / `Validators`
in the vendored JSON at dev time and generate a committed Rust table with
a provenance comment (script in `tools/`), instead of transcribing ECMA.
No work now; this is a standing rule.

## What to test against (oracle ranking)

1. **Real Word open probe** — `../parity/scripts/word-open-probe.sh`
   (opens the FULL file in actual Word). Ultimate authority. Stage files in
   `../parity/_scratch`, **never `/tmp`** (Word sandboxing).
   Necessary AND not implied by #2: strict01_strikethrough validated with
   0 errors yet still trips Word's repair dialog — validator-clean is not
   sufficient.
2. **.NET OpenXmlValidator** (W2) — schema authority; catches child-order
   errors ("unexpected child element pStyle") the Rust stack can't today.
3. **PowerTools C# goldens** — `tools/gen-goldens-cs.sh` (Docxodus
   checkout, local-only). Byte-parity oracle for behavior neutrality: any
   change touching the ordering tables must leave goldens identical.
4. **ooxmlsdk 0.11 parse round-trip** — already in integration tests
   (`WordprocessingDocument::new`); catches structural breakage, not order.
5. **Vendored schema JSON** (W1) — table-consistency oracle.
6. **LibreOffice — explicitly NOT an oracle.** Its leniency masked all 146
   validator-error files in the campaign.

## Pitfalls

- **Parity first.** Any table "fix" W1 uncovers is a behavior change;
  verify against PtOpenXmlUtil.cs, then re-run goldens + a Word probe
  before merging. The stable-sort semantics matter too: unknown names rank
  999 and keep relative order (C# stable `OrderBy`) — a consistency test
  must not tempt anyone into sorting unknowns.
- **`validators` feature mirage** (see W3): the flag exists, the WML impls
  don't. Re-verify on each ooxmlsdk upgrade — if upstream ever generates
  them, W3 gets much cheaper.
- **The JSON is .NET-SDK metadata, not raw ECMA**: transitional +
  Microsoft extensions, version-gated. `Name` is `"w:CT_X/w:tag"` — split
  on `/` and strip the prefix for local-name comparison. Particles nest
  (`Sequence` containing `Sequence`/`Choice`/`Group`); flatten carefully
  and treat Choice members as unordered (tcBorders' `start`/`left` and
  `end`/`right` aliases are the likely tripwire).
- **Word acceptance ≠ validator-clean** (both directions): Word repairs
  some validator-clean files (strict01 sdt case, KNOWN_ISSUES) and opens
  some invalid ones. Keep both oracles.
- **dotnet SDK pin**: under SDK 10, C# 14 first-class-span conversions make
  `MemoryExtensions.Reverse` (in-place, void) shadow LINQ `Reverse()` —
  documented in `tools/gen-goldens-cs.sh`. Pin SDK 8 via `global.json` and
  run dotnet with cwd inside the pinned dir. Applies to W2's tool too.
- **`--detail-threshold=0.15`** must be passed to the Docxodus redline CLI
  when generating goldens (its Program.cs defaults to 0; library + Rust
  default is 0.15). Silent mismatch otherwise.
- **crates.io hygiene**: the vendored JSON must stay out of the published
  package. `Cargo.toml` uses an `include` whitelist that already excludes
  `tests/` — after adding files run `cargo package --list` and confirm.
- **License**: data is MIT (dotnet/Open-XML-SDK) vendored into an AGPL
  crate — fine, but keep the attribution README next to the JSON and
  record the exact upstream commit so it can be refreshed.
- **grep on single-line XML counts lines, not matches**: use
  `grep -o … | wc -l`, never `grep -c`. (This footgun produced two wrong
  conclusions in one past session.)
- **Corpus sweeps make big accidental commits**: a past checkpoint commit
  swept ~68 MB of fixtures into a branch. Keep W2's corpus outputs under
  an ignored dir (`_scratch/`) and check `git status` before committing.

## Suggested order

W1 (one sitting, pure test code) → W2 (needs dotnet locally) → re-evaluate
W3 → W4 as features demand.
