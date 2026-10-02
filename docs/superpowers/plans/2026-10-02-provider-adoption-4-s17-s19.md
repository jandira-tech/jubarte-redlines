<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Provider Adoption Plan 4 of 4: S17 to S19 Implementation Plan

> **Execution:** inline, by one engineer, no subagents. Run every Cargo
> command from the repository root, one at a time, in the default `target/`
> (`AGENTS.md`). Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Baseline:** `main` at `b420d64` (2026-10-02); the verified facts, gates
> and license position are in plan 1
> ([2026-10-02-provider-adoption-1-s01-s10.md](2026-10-02-provider-adoption-1-s01-s10.md)).

**Goal:** Refuse legacy `.doc` with a stable code at every surface (S18),
author watermarks (S17), and edit documents that already carry another
party's tracked changes without flattening them (S19), so that the last
"keep LibreOffice for this" cases and the central legal-review case
(redlining on top of the counterparty's redline) have a jubarte answer.

**Architecture:** S18 is one branch in `admission::admit`. S17 is one
edit operation that writes Word's own VML watermark shape into every
default header. S19 adds a third policy, `existing_revisions: "keep"`, that
emits the plan's edits as tracked changes **directly** into a copy of the
source instead of deriving the redline from a compare, because the comparer
accepts both inputs' revisions before diffing (`compare_documents_impl`,
`docs/C4_preexisting_revisions_decision.md`) and would erase the other
party's changes. The compare contract (`accept(redline) ≡ B`) is untouched.

**Tech Stack:** as plan 1.

---

## Why a provider would take this

| Provider need (verified 2026-10-02 unless marked) | Today | After this plan |
|---|---|---|
| Anthropic skill: "Legacy `.doc` files must be converted first: `soffice.py --headless --convert-to docx`" | `jubarte` CLI already refuses OLE files with a save-as hint (`src/bin/jubarte.rs:1446`); Python and WASM see a ZIP error | `LEGACY_DOC` from every binding, with the same hint; `capabilities().limits.reads_legacy_doc` stays `false` and the skill keeps soffice for the conversion itself |
| ChatGPT container watermark tools (reported, not verified) | `convert` paints watermarks it finds; nothing authors one | `watermark` plan operation; the PNG shows it |
| Every provider's review workflow: the document you are asked to redline already has the other side's tracked changes | `existing_revisions: refuse` (default), `accept` or `reject` flatten first; the mapping document scores this row M = 0.65 | `existing_revisions: "keep"`: their changes stay theirs, yours are yours, one document |

The S19 row is the one that matters for legal review: Word users type
with Track Changes on top of a received redline every day, and no provider
tool reproduces that without hand-written XML.

## Dependencies

| Task | Suggestion | Depends on | Unlocks |
|---|---|---|---|
| 1 | S18 `LEGACY_DOC` | plan 2 Task 1 (admission on compare and convert; without it only `inspect`/`edit` get the code) | one error for every surface |
| 2 | S17 watermark | plan 1 Task 2 (`validate`), plan 2 Task 2 (`diff_render` proves it painted) | ChatGPT watermark tools |
| 3 | S19 keep | plan 1 Task 2 (`validate`), plan 1 Task 4 (`occurrence`), plan 1 Task 3 (comment threads coexist) | the review-on-review case |

---

### Task 1: S18, legacy `.doc` refused with `LEGACY_DOC`

**Files:**
- Modify: `src/admission.rs:60-115` (`AdmissionErrorKind::LegacyDocument`), `:148` (`admit` checks the OLE signature before `declared_entry_count`)
- Modify: `src/bin/jubarte.rs:1446-1460` (`read_document` keeps its early check but takes the message from `AdmissionError`, so the two cannot drift)
- Modify: `src/capabilities.rs` (document `reads_legacy_doc` next to the new code), skill "Dependencies"
- Test: unit test in `src/admission.rs`, `tests/admission_everywhere.rs` (plan 2) gains one case, `jubarte-python/tests/test_admission.py` gains one case
- Docs: CHANGELOG

- [ ] **Step 1: Failing test** (append to the `tests` module of `src/admission.rs`)

```rust
    #[test]
    fn an_ole_compound_file_is_legacy_doc_with_a_save_as_hint() {
        let mut doc = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec();
        doc.extend_from_slice(&[0u8; 512]);
        let e = admit(&doc, InputLimits::default()).unwrap_err();
        assert_eq!(e.kind, AdmissionErrorKind::LegacyDocument);
        assert_eq!(e.code(), "LEGACY_DOC");
        assert!(e.message.contains("save it as .docx"), "{}", e.message);
    }

    #[test]
    fn rtf_is_unsupported_not_a_zip_error() {
        let e = admit(b"{\\rtf1\\ansi hello}", InputLimits::default()).unwrap_err();
        assert_eq!(e.code(), "UNSUPPORTED_PACKAGE");
        assert!(e.message.contains("RTF"));
    }
```

And in `tests/admission_everywhere.rs`:

```rust
#[test]
fn legacy_doc_has_the_same_code_on_every_entry_point() {
    let mut doc = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec();
    doc.extend_from_slice(&[0u8; 512]);
    let other = too_many_entries(); // any bytes; the .doc is checked first
    for m in [
        compare_documents(&doc, &other, "A").unwrap_err().to_string(),
        accept_revisions(&doc).unwrap_err().to_string(),
        docx_to_pdf(&doc).unwrap_err().to_string(),
        list_changes(&doc).unwrap_err().to_string(),
        jubarte::inspect::inspect_json(&doc).unwrap_err().to_string(),
    ] {
        assert_eq!(code_of(&m), "LEGACY_DOC", "{m}");
    }
}
```

- [ ] **Step 2: Run, expect `no variant LegacyDocument`.**

- [ ] **Step 3: Implement**

```rust
    /// A Word 97-2003 `.doc`, or an encrypted document of any Word version:
    /// an OLE compound file, which only Word reads (`LEGACY_DOC`).
    LegacyDocument,
```

`code()` returns `"LEGACY_DOC"`. In `admit`, before the size check:

```rust
    const OLE_MAGIC: &[u8] = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1";
    if bytes.starts_with(OLE_MAGIC) {
        return Err(AdmissionError::new(
            K::LegacyDocument,
            "a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password",
        ));
    }
    if bytes.starts_with(b"{\\rtf") {
        return Err(AdmissionError::new(K::UnsupportedPackage, "an RTF file, not a .docx package"));
    }
```

The CLI's `read_document` calls `admit`'s signature check through a small
`admission::sniff(bytes) -> Result<(), AdmissionError>` (the two branches
above, extracted) so the message lives once. `Admission::Lenient` (plan 2)
still runs `sniff`.

- [ ] **Step 4: Pass**, including the existing
`a_legacy_doc_is_refused_with_a_save_as_hint` CLI test.

- [ ] **Step 5: Docs**: skill "Dependencies": "`.doc` is refused with
`LEGACY_DOC`; convert it with Word or `soffice --convert-to docx` first";
CHANGELOG. **Step 6: Commit** `feat(admission): LEGACY_DOC for OLE files, UNSUPPORTED_PACKAGE for RTF, one message for CLI and bindings`.

---

### Task 2: S17, watermark authoring

**Files:**
- Create: `src/edit/watermark.rs`
- Modify: `src/edit.rs` (`OperationKind::Watermark`), `check_operation_keys`, `apply`, `touched_stories` (headers are stories: the operation marks every default header as touched, creating one per section when absent)
- Modify: `src/capabilities.rs`, `tests/agent_contracts.rs`
- Bindings: `EditPlan.watermark(text, *, color="C0C0C0", diagonal=True, font="Calibri", id=None)`
- Test: `tests/edit_watermark.rs`
- Docs: skill §2, CHANGELOG

Wire: `{"kind":"watermark","text":"DRAFT","color":"C0C0C0","diagonal":true,"font":"Calibri"}`.
`text` 1 to 64 characters, plain; `color` six hex digits; `diagonal` false
means horizontal (`rotation` omitted). One watermark per document: a second
`watermark` in the same plan, or a header that already holds a
`PowerPlusWaterMarkObject` shape, is refused with `UNSUPPORTED_STRUCTURE`
("remove the existing watermark first").

Markup (what Word 2016 and later writes for Insert > Watermark; the shape
type is `_x0000_t136`, the shape id prefix `PowerPlusWaterMarkObject`):

```xml
<w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr><w:r><w:pict>
  <v:shapetype id="_x0000_t136" coordsize="21600,21600" o:spt="136" adj="10800" path="m@7,l@8,m@5,21600l@6,21600e">
    <v:formulas><v:f eqn="sum #0 0 10800"/><v:f eqn="prod #0 2 1"/><v:f eqn="sum 21600 0 @1"/><v:f eqn="sum 0 0 @2"/><v:f eqn="sum 21600 0 @3"/><v:f eqn="if @0 @3 0"/><v:f eqn="if @0 21600 @1"/><v:f eqn="if @0 0 @2"/><v:f eqn="if @0 @4 21600"/><v:f eqn="mid @5 @6"/><v:f eqn="mid @8 @5"/><v:f eqn="mid @7 @8"/><v:f eqn="mid @6 @7"/><v:f eqn="sum @6 0 @5"/></v:formulas>
    <v:path textpathok="t" o:connecttype="custom" o:connectlocs="@9,0;@10,10800;@11,21600;@12,10800" o:connectangles="270,180,90,0"/>
    <v:textpath on="t" fitshape="t"/>
    <v:handles><v:h position="#0,bottomRight" xrange="6629,14971"/></v:handles>
    <o:lock v:ext="edit" text="t" shapetype="t"/>
  </v:shapetype>
  <v:shape id="PowerPlusWaterMarkObject1" o:spid="_x0000_s2049" type="#_x0000_t136"
    style="position:absolute;margin-left:0;margin-top:0;width:527.85pt;height:131.95pt;rotation:315;z-index:-251656192;mso-position-horizontal:center;mso-position-horizontal-relative:margin;mso-position-vertical:center;mso-position-vertical-relative:margin"
    o:allowincell="f" fillcolor="#C0C0C0" stroked="f">
    <v:fill opacity=".5"/>
    <v:textpath style="font-family:&quot;Calibri&quot;;font-size:1pt" string="DRAFT"/>
  </v:shape>
</w:pict></w:r></w:p>
```

The header root must bind `xmlns:v="urn:schemas-microsoft-com:vml"`,
`xmlns:o="urn:schemas-microsoft-com:office:office"` and
`xmlns:w10="urn:schemas-microsoft-com:office:word"` (`namespaces.rs`
has `VML`; add `O` and `W10` constants). A section with no default header
gets `word/headerN.xml` (next free N), its content-type override, a
relationship from the main part and a `w:headerReference w:type="default"`
inserted first in that `w:sectPr` (schema order). `o:spid` increments from
`_x0000_s2049` per header. Width and height follow Word's defaults for a
Letter page; for A4 and other sizes scale the width to 0.9 of the text
width and keep the 4:1 ratio (Word's own numbers for A4 differ slightly;
record the difference in `docs/WORD_DIFFERENCES.md`).

The watermark is header content, not a revision: the clean copy and the
redline both carry it, and the redline shows no `w:ins` for it (compare
does not track header shapes; the report's `render` line is how an agent
sees it).

- [ ] **Step 1: Failing test**

```rust
// tests/edit_watermark.rs
mod common;
use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::convert::{DiffOptions, PdfOptions, diff_render};
use jubarte::edit::{EditPlan, apply_plan};

#[test]
fn a_watermark_lands_in_a_new_default_header_and_paints() {
    let source = docx(&para("Body text."));
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[{"kind":"watermark","text":"DRAFT"}]}"#).unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let header = part_string(&out.clean, "word/header1.xml").expect("a default header was created");
    assert!(header.contains(r#"<v:textpath style="font-family:&quot;Calibri&quot;;font-size:1pt" string="DRAFT"/>"#), "{header}");
    assert!(header.contains("PowerPlusWaterMarkObject1"));
    let doc = part_string(&out.clean, "word/document.xml").unwrap();
    assert!(doc.contains(r#"<w:headerReference w:type="default""#));
    // It paints: the page differs from the unmarked page by a diagonal band.
    let diff = diff_render(&source, &out.clean, &DiffOptions { dpi: 50.0, pdf: PdfOptions::default(), overlay: false }).unwrap();
    assert!(diff.pages[0].changed_ratio > 0.01, "{:?}", diff.pages[0]);
}

#[test]
fn a_second_watermark_is_refused() {
    let source = docx(&para("x"));
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[{"kind":"watermark","text":"DRAFT"},{"kind":"watermark","text":"COPY"}]}"#).unwrap();
    let e = apply_plan(&source, &plan).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(e.operation.as_deref(), Some("op-2"));
}
```

- [ ] **Step 2: Run, expect `unknown kind "watermark"`.** **Step 3:
Implement** as designed (header creation reuses the three-step pattern of
`ensure_factory_package_chrome`; `Opened::story_parts()` enumerates
existing headers). **Step 4: Pass**; also `cargo test --all-features --test
edit_stories` (header stories still edit after a watermark). **Step 5:
Docs** (skill §2: "`watermark` writes Word's own diagonal text watermark
into every default header; one per document"); CHANGELOG. **Step 6:
Commit** `feat(edit): watermark operation writing Word's VML text watermark into default headers`.

**Evidence this task hands a provider:** the PNG before and after, and the
`diff-render` overlay, in `docs/adoption/watermark.md`.

---

### Task 3: S19, `existing_revisions: "keep"`: edit on top of another party's redline

**Files:**
- Create: `src/edit/tracked.rs` (direct tracked-change emission)
- Modify: `src/edit.rs:105-116` (`ExistingRevisions::Keep`), `:1062-1130` (`Transaction::start` no longer flattens under `Keep`), `:2545` (`finish` produces the redline by emission under `Keep`), `EditReport` (`existing_revisions` is already echoed; `base_sha256 == source_sha256` under `Keep`)
- Modify: `src/capabilities.rs` (`operations.edit_keeps_revisions`), skill §1 and §2, `docs/C4_preexisting_revisions_decision.md` (a footnote: the edit side now has a keep policy; the compare contract is unchanged)
- Modify: `jubarte-python/python/jubarte_redlines/models.py:419-424` (`existing_revisions` accepts `"keep"`), `document.py` docstrings
- Test: `tests/edit_keep_revisions.rs`, `jubarte-python/tests/test_keep_revisions.py`
- Docs: CHANGELOG

Why not freeze-through-compare (the mapping document's sketch): the
comparer accepts both inputs' revisions when either side carries tracked
changes (`compare_documents_impl`: "Word-visual mode + either side already
carries track changes: accept both packages first"), and its atomizer has
no contract for elements renamed into `urn:jubarte:frozen-revision` (the
`FROZEN_NS` trick in `src/changes.rs:201` works only inside the revision
processor, which skips that namespace on purpose). Keeping the other
party's markup through compare would mean teaching the comparer a second
revision model, which `docs/C4_preexisting_revisions_decision.md` has
deferred since July as option C with "high" risk. Direct emission is what
Word itself does when you type with Track Changes on: new `w:ins`/`w:del`
with fresh ids and your author, around text that is not already inside a
revision. The edit engine already resolves every text operation into
`ScheduledEdit { start, end, replacement }` ranges on a paragraph
projection (`rewrite` included), already refuses ranges inside revisions
(`check_range`: "the text sits inside a hyperlink, field, content control
or revision"), and already has the run-splitting helpers
(`split_run_at`, `wrap_range`, `apply_text_edit`). The emitter is the
missing 300 lines, not a new engine.

Semantics under `keep`:

- `base` is the source as is (no accept, no reject); `base_sha256 ==
  source_sha256`; `resolve_revisions` still runs first when given.
- `clean` is the base with the plan's edits applied as plain text, the
  other party's revisions untouched (so "clean" means "my edits accepted,
  theirs still tracked").
- `redline` is the base with the plan's edits as revisions by the plan's
  author and date, ids from `max(w:id)+1` across every revision-bearing
  part, the other party's revisions byte-identical.
- Invariants, which are the tests: `accept_changes(redline, authors=[me])`
  has the text of `clean`; `reject_changes(redline, authors=[me])` has the
  text of the source; `list_changes(redline)` lists the other party's
  changes with their original ids and the plan's changes with new ids;
  `validate(redline)` is clean.
- Per operation: `replace`/`insert`/`delete`/`rewrite` emit `w:del`
  (`w:t` to `w:delText`) and `w:ins` runs at the resolved ranges; `whole`
  is implied (direct emission is one deletion then one insertion);
  `insert_paragraph` emits the paragraph with `w:ins` around every run and
  `w:ins` in `w:pPr/w:rPr` (inserted mark); `delete_paragraph` emits
  `w:del` around every run and `w:del` in the mark; `merge_paragraphs`
  emits `w:del` in the first paragraph's mark and a `w:ins` run for the
  separator; `format_paragraph` emits `w:pPrChange` holding the old
  `w:pPr`; `comment` and the thread operations are unchanged;
  `format_run` (plan 1 Task 8c) emits `w:rPrChange`; structural inserts
  (`insert_table`, `insert_image`, `insert_footnote`) emit `w:ins` on rows
  and runs; `settings`, `watermark`, `fill_control` (plan 3) are not
  revisions and apply to both copies.
- `patch.diff` and `EditResult.diff` are computed from the redline filtered
  to the plan's author and date (check that `markdown::patch_documents`
  takes the author; pass it).

- [ ] **Step 1: Failing tests**

```rust
// tests/edit_keep_revisions.rs
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `existing_revisions: "keep"`: the other party's tracked changes stay
//! theirs, the plan's become new revisions, and the four invariants hold.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, ChangeKind, accept_changes, list_changes, reject_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

const OTHER: &str = r#"<w:p><w:r><w:t xml:space="preserve">Payment within </w:t></w:r><w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins><w:del w:id="8" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>30</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>"#;

fn source() -> Vec<u8> {
    docx(&(OTHER.to_string() + &para("Governing law: Delaware.")))
}

fn texts(docx: &[u8]) -> Vec<String> {
    paragraphs(docx).unwrap().into_iter().map(|p| p.text).collect()
}

fn mine() -> ChangeFilter {
    ChangeFilter { authors: Some(vec!["Me".to_string()]), ..ChangeFilter::default() }
}

#[test]
fn keep_leaves_their_changes_and_adds_mine() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","date":"2026-10-02T00:00:00Z","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":"body:p:1","find":"Delaware","replacement":"New York"}]}"#).unwrap();
    let out = apply_plan(&source(), &plan).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    assert_eq!(out.report.base_sha256, out.report.source_sha256);

    // clean: my edit applied, theirs still tracked (visible text shows the accepted view of theirs).
    assert_eq!(texts(&out.clean), ["Payment within 45 days.", "Governing law: New York."]);
    assert_eq!(list_changes(&out.clean).unwrap().len(), 2, "their two changes are still tracked in the clean copy");

    // redline: theirs by id and author, mine new.
    let changes = list_changes(&out.redline).unwrap();
    let theirs: Vec<_> = changes.iter().filter(|c| c.author.as_deref() == Some("Other")).map(|c| c.id.as_str()).collect();
    assert_eq!(theirs, ["body:rev:7", "body:rev:8"]);
    let mine_kinds: Vec<_> = changes.iter().filter(|c| c.author.as_deref() == Some("Me")).map(|c| (c.kind, c.text.as_str())).collect();
    assert_eq!(mine_kinds, [(ChangeKind::Deletion, "Delaware"), (ChangeKind::Insertion, "New York")]);
    assert!(changes.iter().filter(|c| c.author.as_deref() == Some("Me")).all(|c| c.id != "body:rev:7" && c.id != "body:rev:8"));

    // The invariants.
    assert_eq!(texts(&accept_changes(&out.redline, &mine()).unwrap()), texts(&out.clean));
    assert_eq!(texts(&reject_changes(&out.redline, &mine()).unwrap()), texts(&source()));
    // Their markup is byte-identical in the redline.
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(red.contains(r#"<w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins>"#), "{red}");
}

#[test]
fn an_edit_inside_their_insertion_is_still_refused() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":"body:p:0","find":"45","replacement":"60"}]}"#).unwrap();
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert!(e.message.contains("revision"));
}

#[test]
fn paragraph_operations_emit_marks_under_keep() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[
        {"kind":"insert_paragraph","paragraph":"body:p:1","position":"after","runs":[{"text":"Venue: New York County."}]},
        {"kind":"delete_paragraph","paragraph":"body:p:1"},
        {"kind":"rewrite","paragraph":"body:p:0","text":"Payment within 45 days of invoice."}]}"#).unwrap();
    let out = apply_plan(&source(), &plan).unwrap();
    assert_word_valid_package(&out.redline);
    let changes = list_changes(&out.redline).unwrap();
    assert!(changes.iter().any(|c| c.author.as_deref() == Some("Me") && c.kind == ChangeKind::Insertion && c.target == "paragraph_mark"));
    assert!(changes.iter().any(|c| c.author.as_deref() == Some("Me") && c.kind == ChangeKind::Deletion && c.target == "paragraph_mark"));
    assert!(changes.iter().any(|c| c.author.as_deref() == Some("Me") && c.kind == ChangeKind::Insertion && c.text == " of invoice"));
    assert_eq!(texts(&accept_changes(&out.redline, &mine()).unwrap()), texts(&out.clean));
    assert_eq!(texts(&reject_changes(&out.redline, &mine()).unwrap()), texts(&source()));
}

#[test]
fn keep_without_existing_revisions_equals_the_compare_redline_in_text() {
    // On a source with no revisions, keep and the default produce the same
    // visible result, so an agent can always send keep.
    let plain = docx(&para("Fee is 10."));
    let keep = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[{"kind":"replace","paragraph":"body:p:0","find":"10","replacement":"12"}]}"#).unwrap();
    let default = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","operations":[{"kind":"replace","paragraph":"body:p:0","find":"10","replacement":"12"}]}"#).unwrap();
    let a = apply_plan(&plain, &keep).unwrap();
    let b = apply_plan(&plain, &default).unwrap();
    assert_eq!(texts(&a.clean), texts(&b.clean));
    assert_eq!(texts(&accept_changes(&a.redline, &mine()).unwrap()), texts(&accept_changes(&b.redline, &mine()).unwrap()));
}
```

- [ ] **Step 2: Run, expect `INVALID_PLAN` (`unknown variant keep`).**

- [ ] **Step 3: Implement**

1. `ExistingRevisions::Keep` with the doc comment "Leave them tracked; the
   plan's edits become new revisions beside them (direct emission; no
   compare)."
2. `Transaction::start`: `(true, ExistingRevisions::Keep) => (source.to_vec(), probe)`.
   `check_range` already refuses non-direct segments, which under `keep`
   is the "no edits inside their revisions" rule; add a `keep` note to its
   message.
3. `Transaction::finish` under `Keep`: after producing `clean` as today,
   re-open the base (`Opened::open(&self.base)`), and replay
   `self.resolved` through `tracked::emit(&mut opened, &self.plan, &self.resolved, author, date, next_id)` which, per `Resolved` arm:
   - text edits: for each `ScheduledEdit` in descending `start` order (so
     earlier offsets stay valid), `split_run_at` at `start` and `end`,
     move the runs in `[start, end)` into a new `w:del` element (rename
     each `w:t` to `w:delText`, keep `xml:space`), then build the
     replacement run (clone the run before the range's `w:rPr`, or the
     deleted run's) inside a new `w:ins` placed after the `w:del`;
     empty `replacement` emits no `w:ins`; empty range (`insert`) emits no
     `w:del`;
   - paragraph marks: `w:pPr/w:rPr/w:ins|w:del` elements with id, author,
     date (create `w:pPr`/`w:rPr` in schema position; `insert_ppr_child`
     and `insert_rpr_child` order them);
   - `format_paragraph`: clone the old `w:pPr` into `w:pPrChange` (last
     child of the new `w:pPr`, per schema), then apply the new properties;
   - ids: `next_id` starts at the maximum `w:id` across the revision
     carriers of every story part (the collector from `validate`'s
     `DUPLICATE_REVISION_ID` check, made `pub(crate)`), increments per
     element; `w:date` is the plan's date.
   Then write the comments part and the touched stories exactly as `finish`
   does for the clean copy, and return the pair.
4. `EditReport.existing_revisions` already echoes the policy; `base_sha256`
   equals `source_sha256` under `Keep`.
5. `revision_counts` (for the report's totals) uses `list_changes` on the
   redline filtered to the plan's author under `Keep`.
6. Python: `models.py` `__post_init__` allows `"keep"`; `document.py`
   docstring for `edit`.

- [ ] **Step 4: Pass** the new tests, then `cargo test --all-features --test edit_plan --test edit_stories --test edit_comment_threads` (unchanged policies), then the whole suite once.

- [ ] **Step 5: Docs**

Skill §1 gotcha: "If `summary.revisions > 0`: `\"existing_revisions\":
\"keep\"` leaves their changes tracked and adds yours beside them (what
Word does when you type on a received redline); `\"accept\"`/`\"reject\"`
flatten first; the default refuses. Under `keep` you cannot edit text
inside their insertions or deletions; resolve those changes first with
`resolve_revisions`." Skill §3: the accept-equals-clean check under `keep`
is `jubarte accept review/redline.docx --author Claude -o check.docx`.
`docs/C4_preexisting_revisions_decision.md`: a dated footnote that the
edit API now offers `keep` by direct emission, and that compare's
accept-first contract is unchanged (option A stands). CHANGELOG.

- [ ] **Step 6: Commit**

```bash
git add src/edit.rs src/edit/tracked.rs src/capabilities.rs tests/edit_keep_revisions.rs tests/agent_contracts.rs jubarte-python skills/jubarte-documents/SKILL.md docs/C4_preexisting_revisions_decision.md CHANGELOG.md
git commit -m "feat(edit): existing_revisions keep: new revisions emitted beside another party's, no flattening"
```

**Evidence this task hands a provider:** `examples/agents/review-on-review/`:
a two-party contract fixture with the counterparty's redline, one plan with
`keep`, the resulting redline opened in Word showing both authors' balloons
(screenshot), `jubarte changes --json` listing both authors, and the two
invariant commands (`accept --author Me` equals clean, `reject --author Me`
equals the source) with their `jubarte text` outputs. Anthropic's skill
section on tracked changes ("Wrap runs in `<w:ins>`/`<w:del>` with `w:id`,
`w:author`, `w:date` attributes ... The `<w:del/>` must come before the
rPr's other children; their order is schema-enforced") is the hand-written
version of exactly this emitter; the pitch is that the agent never writes
that XML again and `validate --original --author` still proves it.

---

## Counterarguments the plan does not hide

- **Direct emission is a second way to produce revisions.** The comparer's
  output and the emitter's output can differ in grouping (compare groups
  by word, the emitter by operation). The invariant tests pin what matters
  (accept/reject text, their ids untouched, validity); grouping is reported
  per operation in `report.jsonl` as today.
- **`keep` does not merge conflicting edits.** An edit inside their
  revision is refused, which is stricter than Word (Word lets you delete
  inside an insertion by another author). Say so; offer `resolve_revisions`.
- **Watermark markup is VML.** Word still writes VML for watermarks in
  2026; the converter already paints it. If Word changes, the test's
  `diff_render` assertion catches a silent no-paint.
- **`LEGACY_DOC` is a refusal, not a reader.** A `.doc` parser is a large
  surface; keeping soffice for that one conversion is the right trade, and
  the skill says so.
