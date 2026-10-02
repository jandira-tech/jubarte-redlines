<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Provider Adoption Plan 2 of 4: S11 to S13 Implementation Plan

> **Execution:** inline, by one engineer, no subagents. Run every Cargo
> command from the repository root, one at a time, in the default `target/`
> (`AGENTS.md`). Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Baseline:** `main` at `b420d64` (2026-10-02); see plan 1
> ([2026-10-02-provider-adoption-1-s01-s10.md](2026-10-02-provider-adoption-1-s01-s10.md))
> for the verified facts, the gates and the license position. This plan
> repeats only what it needs.

**Goal:** Admit untrusted input on every entry point (S12), give agents a
visual page diff and page-range rendering (S11), and append documents
(S13), so that a provider's security reviewer, its render-and-verify loop,
and its `merge_docx_append.py` each have a jubarte answer.

**Architecture:** S12 is one function call (`admission::admit`) placed at
the four openings the library has (`compare_documents_impl`,
`changes::open`, `convert::with_pages`, `markdown::docx_to_markdown`), with
a typed opt-out carried in the existing settings/options structs, never a
process global. S11 reuses `convert::render` and the `image` crate already
in the tree. S13 reuses the comparer's relationship carrying
(`comparer::parts::carry_relationship`) and the Markdown writer's numbering
and notes part builders.

**Tech Stack:** as plan 1.

---

## Why a provider would take this

| Provider need (verified 2026-10-02) | Today | After this plan |
|---|---|---|
| Anthropic skill: "`find unpacked -type l -delete` # strip symlink entries — docx from external parties is untrusted" | an agent shell step, only on the edit path | `admit` on every entry point: unsafe part names, duplicates, encryption, zip budgets, XML depth, before anything inflates (`src/admission.rs` module docs); an `INPUT_LIMIT`/`UNSUPPORTED_PACKAGE` code instead of a crash or a hang |
| Older Anthropic skill: `defusedxml` | XML bombs handled by a Python wrapper | `quick-xml` checked admission with a depth budget, in Rust, no DTD |
| OpenAI Codex issue #38313: `render_docx.py` "cannot render a requested page range ... no `--first-page` / `--last-page`", no timeout | full DOCX to PDF to all-page PNG for one citation | `convert --png --pages 3-5` rasterizes only those pages from one layout pass |
| ChatGPT container `render_and_diff.py` (reported, not verified) and Z.ai's visual-judge subagent | pixel diff through LibreOffice | `jubarte diff-render A B`: per-page `changed_ratio`, bounding box, overlay PNG, pages present on one side reported |
| ChatGPT container `merge_docx_append.py` (reported, not verified) | python-docx body copy, relationships and numbering by hand | `jubarte append A B -o OUT`: relationships, styles by (type, name), numbering, notes carried; output validated |

**Why S12 is first.** The mapping document scores S12 highest
(2.65) and says why: "A security reviewer will test `compare_documents`
first." Today `admit` is called only from `inspect::Opened::open`
(`src/inspect.rs:290`); `compare_documents`, `accept_revisions`,
`reject_revisions`, `get_revisions`, `list_changes`, `docx_to_pdf`,
`render` and `docx_to_markdown` open the ZIP with `PartFs::open` and no
budget. The module doc says so in one line: "The redline comparer keeps its
historical tolerance." The tolerance that matters (missing settings or
theme, broken media relationships, `tests/m148_tolerated_inputs.rs`) is
about package **content**, which admission does not judge; admission judges
the **container**. Task 1's corpus sweep proves that the default budgets
refuse no fixture the comparer accepts today.

## Dependencies

| Task | Suggestion | Depends on | Unlocks |
|---|---|---|---|
| 1 | S12 admission everywhere | nothing | the security pitch; plan 4's S18 rides the same function |
| 2 | S11 visual diff and page ranges | nothing (uses `convert::render`) | Codex #38313; ChatGPT/Z.ai visual QA; plan 4's watermark test |
| 3 | S13 append | plan 1 Task 2 (`validate()` as the oracle) | `merge_docx_append.py` |

---

### Task 1: S12, input admission on every entry point

**Files:**
- Modify: `src/admission.rs` (`InputLimits::unbounded()`, `Admission` policy enum, `pub fn refusal_code(err: &dyn std::error::Error) -> Option<&'static str>` helper for bindings)
- Modify: `src/comparer/mod.rs:1165` (`WmlComparerSettings.admission: Admission`, `Default` = `Admission::Default`)
- Modify: `src/document_comparer.rs:6024-6180` (`compare_documents_impl` admits both inputs first; `accept_revisions`, `reject_revisions`, `get_revisions` admit; a new `*_with_admission` variant each for the opt-out)
- Modify: `src/changes.rs:230` (`open()` admits; `list_changes_with`, `accept_changes_with`, `reject_changes_with` take `Admission`)
- Modify: `src/convert/mod.rs:236-260,441` (`PdfOptions.admission: Admission`; `with_pages` admits; `ConvertError::Refused(AdmissionError)`)
- Modify: `src/markdown/mod.rs:147-200` (`MarkdownOptions.admission`; `docx_to_markdown` admits; `Source::Docx` paths in `unified.rs`/`redline.rs` go through the same)
- Modify: `src/debug.rs:2109,2168` (`Package::open` enforces only the byte and entry budgets; `debug` exists for broken files, so `INVALID_XML`/`UNSUPPORTED_PACKAGE` are reported, not refused; say so in the module doc)
- Modify: `src/capabilities.rs` (`limits.input.entry_points: Vec<String>`)
- Modify: `src/bin/jubarte.rs` (global `--lenient` flag: `Admission::Lenient`)
- Modify: `jubarte-python/src/lib.rs` (`lenient: bool = False` keyword on `compare_documents`, `accept_revisions`, `reject_revisions`, `get_revisions_json`, `list_changes_json`, `accept_changes`, `reject_changes`, `docx_to_pdf`, `docx_to_png`, `render`, `markdown`, `diff_json`), `python/jubarte_redlines/document.py` (`Document.from_bytes(data, *, lenient=False)` stores the policy and passes it to every native call), `_native.pyi`
- Modify: `jubarte-wasm/src/lib.rs` (optional trailing `limitsJson?: string` on `compareDocuments`, `acceptRevisions`, `rejectRevisions`, `getRevisions`, `listChanges`, `acceptChanges`, `rejectChanges`, `docxToPdf`; `"lenient"` or a JSON `InputLimits`)
- Test: `tests/admission_everywhere.rs`, `tests/admission_corpus_sweep.rs`, `jubarte-python/tests/test_admission.py`, `jubarte-wasm/npm-smoke.mjs`
- Docs: `README.md` security paragraph, skill "Dependencies", `CHANGELOG.md`, `docs/api/` note

Error mapping, decided up front: `compare_documents` keeps returning
`Result<Vec<u8>, OpcError>`; `OpcError` is `rdocx_opc`'s type and cannot
grow a variant, so a refusal is `OpcError::Io(io::Error::new(InvalidData,
"INPUT_LIMIT: 10001 ZIP entries; the limit is 10000"))`, exactly as
`invalid_content` (`src/document_comparer.rs:4644`) already does for
orphaned notes. The code is the message prefix up to the first colon;
`admission::refusal_code` parses it, and the Python and WASM wrappers put
the code in front of `JubarteError`/`JsValue` messages as they already do
for `EditError`. `ConvertError` and `ChangeError` are jubarte's own enums
and gain a `Refused(AdmissionError)` variant whose `Display` is
`{code}: {message}`.

```rust
/// How an entry point admits its input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Admission {
    /// [`InputLimits::default`] (the capabilities manifest's `limits.input`).
    #[default]
    Default,
    /// These limits.
    Limits(InputLimits),
    /// No budgets; part-name, duplicate, encryption and compression checks
    /// still apply. For a caller that owns the bytes end to end.
    Lenient,
}

impl InputLimits {
    /// Every budget at its type's maximum.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self { max_compressed_bytes: u64::MAX, max_entries: usize::MAX, max_part_bytes: u64::MAX, max_uncompressed_bytes: u64::MAX, max_xml_depth: usize::MAX }
    }
}

impl Admission {
    pub(crate) fn admit(self, bytes: &[u8]) -> Result<AdmittedPackage, AdmissionError> {
        match self {
            Self::Default => admit(bytes, InputLimits::default()),
            Self::Limits(l) => admit(bytes, l),
            Self::Lenient => admit(bytes, InputLimits::unbounded()),
        }
    }
}
```

`Lenient` still refuses unsafe names, duplicates and encryption on
purpose: nothing in the engine handles those, and a reviewer should find no
mode that lets them through.

- [ ] **Step 1: Write the failing tests**

```rust
// tests/admission_everywhere.rs
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every entry point that reads a package admits it first and refuses with
//! the admission code; the opt-out lets a caller that owns the bytes through.

use std::io::Write;

use jubarte::admission::{Admission, InputLimits};
use jubarte::changes::{ChangeFilter, accept_changes, list_changes, reject_changes};
use jubarte::comparer::WmlComparerSettings;
use jubarte::convert::{PdfOptions, RenderRequest, docx_to_pdf, docx_to_png, render};
use jubarte::document_comparer::{accept_revisions, compare_documents, compare_documents_with_settings, get_revisions, reject_revisions};
use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const CT: &str = r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
const RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
const DOC: &str = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>x</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"#;

/// A well-formed document padded with one entry more than the default
/// `max_entries` budget: cheap to build, refused by every budgeted path.
fn too_many_entries() -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default();
        for (name, body) in [("[Content_Types].xml", CT), ("_rels/.rels", RELS), ("word/document.xml", DOC)] {
            z.start_file(name, opts).unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        for i in 0..(InputLimits::default().max_entries - 2) {
            z.start_file(format!("word/media/pad{i}.bin"), opts).unwrap();
            z.write_all(b"0").unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn code_of(message: &str) -> &str {
    message.split(':').next().unwrap_or("")
}

#[test]
fn every_entry_point_refuses_with_input_limit() {
    let bomb = too_many_entries();
    let messages: Vec<String> = vec![
        compare_documents(&bomb, &bomb, "A").unwrap_err().to_string(),
        accept_revisions(&bomb).unwrap_err().to_string(),
        reject_revisions(&bomb).unwrap_err().to_string(),
        get_revisions(&bomb, &WmlComparerSettings::default()).unwrap_err().to_string(),
        list_changes(&bomb).unwrap_err().to_string(),
        accept_changes(&bomb, &ChangeFilter::default()).unwrap_err().to_string(),
        reject_changes(&bomb, &ChangeFilter::default()).unwrap_err().to_string(),
        docx_to_pdf(&bomb).unwrap_err().to_string(),
        docx_to_png(&bomb, PdfOptions::default(), 50.0).unwrap_err().to_string(),
        render(&bomb, PdfOptions::default(), RenderRequest::default()).unwrap_err().to_string(),
        docx_to_markdown(&bomb, &MarkdownOptions::default()).unwrap_err().to_string(),
    ];
    for m in &messages {
        assert_eq!(code_of(m), "INPUT_LIMIT", "{m}");
    }
}

#[test]
fn the_opt_out_lets_an_owned_package_through_but_never_an_encrypted_one() {
    let bomb = too_many_entries();
    let settings = WmlComparerSettings { admission: Admission::Lenient, ..WmlComparerSettings::default() };
    assert!(compare_documents_with_settings(&bomb, &bomb, &settings).is_ok());
    let pdf = jubarte::convert::docx_to_pdf_with(&bomb, PdfOptions { admission: Admission::Lenient, ..PdfOptions::default() });
    assert!(pdf.is_ok());
    // Still no path for an encrypted entry. The zip crate's encryption
    // writer is crate-private, so flip bit 0 of the first local header's
    // general-purpose flags (offset 6), as src/admission.rs's own test does.
    let mut encrypted = too_many_entries();
    encrypted[6] |= 1;
    let e = compare_documents_with_settings(&encrypted, &too_many_entries(), &settings).unwrap_err().to_string();
    assert_eq!(code_of(&e), "UNSUPPORTED_PACKAGE", "{e}");
}
```

```rust
// tests/admission_corpus_sweep.rs
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The default budgets admit every document the suite already compares:
//! admission judges the container, not the content the comparer tolerates.

use jubarte::admission::{InputLimits, admit};
use std::path::PathBuf;

fn docx_files(root: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(root)];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "docx") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_fixture_and_corpus_document_is_admitted_by_the_defaults() {
    let mut refused = Vec::new();
    let mut seen = 0;
    for root in ["tests/fixtures", "tests/corpus"] {
        for path in docx_files(root) {
            seen += 1;
            let bytes = std::fs::read(&path).unwrap();
            if let Err(e) = admit(&bytes, InputLimits::default()) {
                refused.push(format!("{}: {e}", path.display()));
            }
        }
    }
    assert!(seen > 500, "expected the 537 corpus documents plus fixtures, saw {seen}");
    // Known-bad probes live under tests/fixtures/invalid; list any here on
    // purpose, by name, when the sweep finds them.
    assert!(refused.is_empty(), "defaults refuse {} known-good documents:\n{}", refused.len(), refused.join("\n"));
}
```

- [ ] **Step 2: Run and watch them fail**

Run: `cargo test --all-features --test admission_everywhere --test admission_corpus_sweep`
Expected: `admission_everywhere` fails to compile on `Admission`; after the
type exists, `every_entry_point_refuses_with_input_limit` fails on the first
message (compare returns `Ok`). `admission_corpus_sweep` passes or names
the documents that the defaults would refuse; if any, raise the budget in
`InputLimits::default` (and the manifest test in `capabilities.rs`) rather
than exempt the entry point, and record the document in the test.

- [ ] **Step 3: Implement**

1. `admission.rs`: `Admission`, `unbounded`, `refusal_code`.
2. `compare_documents_impl`: first lines become
   ```rust
   settings.admission.admit(original).map_err(refused)?;
   settings.admission.admit(modified).map_err(refused)?;
   ```
   with `fn refused(e: AdmissionError) -> OpcError { OpcError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) }`. `accept_revisions`, `reject_revisions`, `get_revisions` admit with `Admission::Default` and gain `*_with_admission(docx, Admission)` siblings.
3. `changes::open` takes `Admission`; the three public functions use `Default` and gain `_with` siblings used by the edit transaction (which already admitted the bytes; pass `Lenient` there to avoid double work, with a comment saying why).
4. `convert`: `PdfOptions.admission` (keeps `Default` derive), `with_pages` admits first, `ConvertError::Refused`.
5. `markdown`: `MarkdownOptions.admission`; `docx_to_markdown` admits; the Word side of `patch_documents`/`redline` goes through `inspect::Opened` or `docx_to_markdown`, so it is covered; add an assertion to the test if a path is found that is not.
6. `debug::Package::open`: enforce `max_compressed_bytes` and `max_entries`
   only; document the exemption in the module doc ("debug reads files Word
   refuses; it bounds size and entry count and reports the rest").
7. `capabilities`: `limits.input.entry_points = ["compare", "revisions", "changes", "convert", "inspect", "text", "edit", "markdown"]`; update the unit test and `tests/agent_contracts.rs`.
8. CLI `--lenient` (global, documented: "skip the input budgets for a file you trust; names, duplicates and encryption are still refused").
9. Python and WASM as listed in Files. Python test:

```python
# jubarte-python/tests/test_admission.py
from io import BytesIO
from zipfile import ZipFile, ZipInfo

import pytest

import jubarte_redlines as jubarte
from test_document import make_document


def too_many_entries() -> bytes:
    base = make_document("x")
    out = BytesIO()
    with ZipFile(BytesIO(base)) as src, ZipFile(out, "w") as dst:
        for info in src.infolist():
            dst.writestr(info, src.read(info))
        for i in range(jubarte.capabilities()["limits"]["input"]["max_entries"]):
            dst.writestr(ZipInfo(f"word/media/pad{i}.bin"), b"0")
    return out.getvalue()


@pytest.mark.parametrize("call", [
    lambda d, o: d.compare(o, author="A"),
    lambda d, o: d.accept(),
    lambda d, o: d.reject(),
    lambda d, o: d.changes(),
    lambda d, o: d.revisions(),
    lambda d, o: d.to_pdf(),
    lambda d, o: d.to_png(dpi=40),
    lambda d, o: d.markdown(),
])
def test_every_document_method_refuses_a_bomb_with_the_code(call) -> None:
    bomb = jubarte.Document.from_bytes(too_many_entries())
    other = jubarte.Document.from_bytes(make_document("y"))
    with pytest.raises(jubarte.JubarteError, match=r"^INPUT_LIMIT"):
        call(bomb, other)


def test_lenient_is_per_document_not_global() -> None:
    bomb = jubarte.Document.from_bytes(too_many_entries(), lenient=True)
    assert bomb.to_pdf().startswith(b"%PDF")
    strict = jubarte.Document.from_bytes(too_many_entries())
    with pytest.raises(jubarte.JubarteError, match=r"^INPUT_LIMIT"):
        strict.to_pdf()
```

- [ ] **Step 4: Run tests and watch them pass**

Run: `cargo test --all-features` (whole suite: the comparer's goldens must
not change, and `m148_tolerated_inputs` must still pass), then the Python
suite, then `node jubarte-wasm/npm-smoke.mjs` after `jubarte-wasm/build-npm.sh`.

- [ ] **Step 5: Docs**

README "Security" paragraph: list the eight entry points and the five
codes; state what `--lenient` keeps refusing. Skill "Dependencies": "jubarte
reads the document in memory with size, entry, part and XML-depth budgets
(`jubarte capabilities --json`, `limits.input`); a refused file reports
`INPUT_LIMIT`, `DUPLICATE_PART`, `UNSUPPORTED_PACKAGE`, `INVALID_PACKAGE` or
`INVALID_XML` on stderr. No temp files, no subprocess, no network."
`CHANGELOG.md` Unreleased / Changed: "Every entry point admits its input;
`compare` no longer keeps its historical tolerance for unbounded ZIPs.
`--lenient` / `lenient=True` / `Admission::Lenient` restore it for a file
you own." `docs/api/README.md`: note the additive fields on
`WmlComparerSettings`, `PdfOptions`, `MarkdownOptions`.

- [ ] **Step 6: Commit**

```bash
git add src/admission.rs src/comparer/mod.rs src/document_comparer.rs src/changes.rs src/convert/mod.rs src/markdown src/debug.rs src/capabilities.rs src/bin/jubarte.rs tests/admission_everywhere.rs tests/admission_corpus_sweep.rs tests/agent_contracts.rs jubarte-python jubarte-wasm README.md skills/jubarte-documents/SKILL.md CHANGELOG.md docs/api/README.md
git commit -m "feat(admission): admit input on compare, revisions, changes, convert and markdown; typed opt-out, no process global"
```

Plan 4 Task 1 (S18, `LEGACY_DOC`) adds one branch to `admit()`; land it in
the same release so a `.doc` handed to any binding gets the same code.

**Evidence this task hands a provider:** `docs/adoption/security.md`: the
five codes, the budgets, a 20-line Python script a reviewer can run that
feeds a 10,001-entry ZIP, an encrypted ZIP, a `../` part name and a
300-deep XML to every `Document` method and prints the code each returns,
plus the dependency audit (`cargo deny check`, `unsafe_code = "deny"`,
and the honest note that the dependency tree is not audited for `unsafe`).

---

### Task 2: S11, visual page diff and page-range rendering

**Files:**
- Create: `src/convert/diff_render.rs`
- Modify: `src/convert/mod.rs:334-420` (`RenderRequest.pages: Option<Vec<usize>>`, `render` rasterizes only those; `pub use diff_render::{diff_render, DiffOptions, PageDiff, RenderDiff}`)
- Modify: `src/bin/jubarte.rs:195-246` (`Convert --pages SPEC`), `:136` (`Command::DiffRender`)
- Modify: `jubarte-python/src/lib.rs` (`render(... pages=None)`, `diff_render_json`), `document.py` (`Document.to_png(pages=)`, `Document.render(pages=)`, `jubarte_redlines.diff_render(a, b, dpi=) -> RenderDiff`), `models.py` (`PageDiff`, `RenderDiff`), `_native.pyi`, `__main__.py` (`diff-render`, `convert --pages`)
- Modify: `src/capabilities.rs` (`operations.diff_render`, `operations.page_ranges`)
- Test: `tests/convert_diff_render.rs`, `tests/convert_page_ranges.rs`, `jubarte-python/tests/test_diff_render.py`
- Docs: skill §3, CHANGELOG, `TODO.md` §7 cross-reference

Design:

```rust
/// Which pages of `a` and `b` differ, from one layout pass each at `dpi`.
pub struct DiffOptions { pub dpi: f32, pub pdf: PdfOptions, /// Paint changed pixels magenta over `b`'s page. pub overlay: bool }

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PageDiff {
    /// Zero-based page index.
    pub index: usize,
    /// Changed pixels over all pixels, 0.0 to 1.0. 1.0 when the page exists on one side only.
    pub changed_ratio: f32,
    /// `[x0, y0, x1, y1]` in pixels around every changed pixel; `None` when equal.
    pub bbox: Option<[u32; 4]>,
    /// `"a"` or `"b"` when the page exists on one side only; never skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_in: Option<&'static str>,
}

pub struct RenderDiff { pub pages: Vec<PageDiff>, pub a: Vec<Vec<u8>>, pub b: Vec<Vec<u8>>, pub overlays: Vec<Option<Vec<u8>>>, pub a_report: RenderReport, pub b_report: RenderReport }

pub fn diff_render(a: &[u8], b: &[u8], options: &DiffOptions) -> Result<RenderDiff, ConvertError>;
```

Pixels are compared exactly (the renderer is deterministic: "the same input
writes the same bytes", 0.9.3). Decoding the PNGs with `image` is simpler
than threading `tiny_skia::Pixmap`s out of `render`; do that first and
measure; switch to pixmaps only if a 100-page diff at 100 dpi takes more
than a few seconds.

Page ranges: `--pages 1-3,7` is 1-based in the CLI and the Python
`pages=` keyword (people count pages from one), zero-based inside
`RenderRequest.pages`. Layout always runs for the whole document; only the
raster is skipped. `TODO.md` §7 ("changed pages only") is the same feature
driven by the layout's revision marks instead of a user list; this task
leaves it open and notes that `RenderRequest.pages` is the hook it will
use.

- [ ] **Step 1: Failing tests**

```rust
// tests/convert_diff_render.rs
mod common;
use common::docx::{docx, para};
use jubarte::convert::{DiffOptions, PdfOptions, diff_render};

fn opts() -> DiffOptions {
    DiffOptions { dpi: 50.0, pdf: PdfOptions::default(), overlay: true }
}

#[test]
fn identical_inputs_change_nothing() {
    let a = docx(&para("Same text."));
    let d = diff_render(&a, &a, &opts()).unwrap();
    assert_eq!(d.pages.len(), 1);
    assert_eq!(d.pages[0].changed_ratio, 0.0);
    assert_eq!(d.pages[0].bbox, None);
    assert!(d.overlays[0].is_none());
}

#[test]
fn a_changed_word_changes_a_bounded_region() {
    let a = docx(&para("The fee is ten."));
    let b = docx(&para("The fee is twenty."));
    let d = diff_render(&a, &b, &opts()).unwrap();
    let p = &d.pages[0];
    assert!(p.changed_ratio > 0.0 && p.changed_ratio < 0.05, "{p:?}");
    let [x0, y0, x1, y1] = p.bbox.unwrap();
    assert!(x1 > x0 && y1 > y0 && y1 < 120, "the change sits on the first line: {p:?}");
    assert!(d.overlays[0].as_ref().unwrap().starts_with(b"\x89PNG"));
}

#[test]
fn a_page_present_on_one_side_is_reported_not_skipped() {
    let a = docx(&para("One page."));
    let b = docx(&(para("One page.") + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"# + &para("Second page.")));
    let d = diff_render(&a, &b, &opts()).unwrap();
    assert_eq!(d.pages.len(), 2);
    assert_eq!(d.pages[1].only_in, Some("b"));
    assert_eq!(d.pages[1].changed_ratio, 1.0);
}
```

```rust
// tests/convert_page_ranges.rs
mod common;
use common::docx::{docx, para};
use jubarte::convert::{PdfOptions, RenderRequest, render};

#[test]
fn only_the_requested_pages_are_rasterized_but_the_report_covers_all() {
    let three = docx(&(para("A") + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"# + &para("B") + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"# + &para("C")));
    let out = render(&three, PdfOptions::default(), RenderRequest { pdf: false, png_dpi: Some(40.0), pages: Some(vec![2]) }).unwrap();
    assert_eq!(out.report.page_count, 3);
    assert_eq!(out.pngs.len(), 1);
    assert_eq!(out.report.pages[2].text.trim(), "C");
}

#[test]
fn an_out_of_range_page_is_an_error_naming_the_count() {
    let one = docx(&para("A"));
    let e = render(&one, PdfOptions::default(), RenderRequest { pdf: false, png_dpi: Some(40.0), pages: Some(vec![5]) }).unwrap_err();
    assert!(e.to_string().contains("1 page"), "{e}");
}
```

- [ ] **Step 2: Run, expect compile failures on `DiffOptions` and `RenderRequest.pages`.**

- [ ] **Step 3: Implement** `RenderRequest.pages` (filter in the raster loop
of `render`; `ConvertError::PageOutOfRange { requested, page_count }`),
`diff_render` as designed, CLI `jubarte diff-render A B [--dpi 100]
[--out-dir D] [--json] [--no-overlay]` writing `a-page-NN.png`,
`b-page-NN.png`, `diff-page-NN.png` (only for changed pages) and
`diff.json`; exit 0 equal, 5 differences (so a CI step can gate on it).
`convert --pages 1-3,7` (parse with a small `fn parse_pages(spec) ->
Result<Vec<usize>, String>` unit-tested in `bin/jubarte.rs` tests, 1-based
to 0-based). Python and capabilities as listed.

- [ ] **Step 4: Pass.** Also run `cargo test --all-features --test convert_docx_to_png` (unchanged behaviour without `pages`).

- [ ] **Step 5: Docs**: skill §3 adds "`jubarte diff-render before.docx
after.docx --out-dir diff` writes only the pages that changed with the
change boxed, and `diff.json` with each page's `changed_ratio`; `jubarte
convert file.docx --png --pages 3-5` renders three pages from one layout
pass." CHANGELOG. `TODO.md` §7: add "builds on `RenderRequest.pages`".

- [ ] **Step 6: Commit** `feat(convert): diff-render page diff with overlays; --pages selects pages to rasterize`.

**Evidence this task hands a provider:** `docs/adoption/render.md`: the
Codex issue #38313 scenario (a 60-page document, one citation on page 41):
`render_docx.py` time and files vs `jubarte convert --png --pages 41`;
and a before/after edit with `diff-render` output, the three PNGs and
`diff.json`, so the "render, inspect, iterate" gate of the ChatGPT skill
has a one-command form.

---

### Task 3: S13, append documents

> **Status (2026-10-02, `d24e2f6`): shipped in #285, with deviations listed
> in that PR (`keep_sections` puts A's final `w:sectPr` at the join, since a
> paragraph's `w:sectPr` ends its own section; `append_numbering` takes a
> builder closure; no `validate()` call, because plan 1 Task 2 is not on
> `main`). Not done: comments are dropped, no output has been opened in
> Word, and the evidence below is unwritten. A validator sweep found one
> defect (B's list attributes in a numbering part A did not have). The
> follow-ups, with the sweep and a comment-carrying design, are `TODO.md`
> §9.**

**Files:**
- Create: `src/append.rs`
- Modify: `src/lib.rs`, `src/bin/jubarte.rs` (`Command::Append`), `src/capabilities.rs` (`operations.append`)
- Modify: `src/comparer/parts.rs:466` (`carry_relationship` is already `pub`; add `pub fn carry_part_relationships(dest, dest_part, src, src_part, dom, root)` that rewrites every `r:id`/`r:embed`/`r:link` under `root` through it)
- Modify: `src/markdown/package.rs:359` (make the abstractNum/num appending reusable: `pub(crate) fn append_numbering(package, main, abstracts_xml, nums_xml) -> (first_abstract, first_num)`)
- Bindings: `Document.append(other, *, section_break="next_page") -> Document`, WASM `appendDocuments(a, b, optionsJson)`
- Test: `tests/append_documents.rs`, `jubarte-python/tests/test_append.py`
- Docs: skill §4, CHANGELOG

Design:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionBreak { #[default] NextPage, Continuous, None }

pub struct AppendOptions {
    pub section_break: SectionBreak,
    /// Keep B's final section (page size, margins, headers, footers) as a
    /// section of its own; off, B's content takes A's last section.
    pub keep_sections: bool,
}

pub struct Appended { pub docx: Vec<u8>, /// `COMMENTS_DROPPED`, `CONTROL_FLATTENED`, ... pub warnings: Vec<String> }

pub fn append_documents(a: &[u8], b: &[u8], options: &AppendOptions) -> Result<Appended, AppendError>;
```

Steps inside: admit both (`Admission::Default`), strict-to-transitional
both, open `PartFs` for both, parse A's main part and B's main part; clone
B's body children (all but the final `w:sectPr`) into A's body before A's
final `w:sectPr` (`Dom::clone_subtree` exists, used by `finalize`); between
them, for `NextPage`, a paragraph with `w:br w:type="page"`; for
`Continuous`, nothing; with `keep_sections`, instead write B's final
`w:sectPr` into the `w:pPr` of a new empty paragraph at the join (an inner
section break) and carry its header and footer parts through
`carry_relationship` with a `fits` of `|t| t.ends_with("/header") ||
t.ends_with("/footer")`. Then:

1. Relationships: for every attribute in `S_RELATIONSHIP_ATTRIBUTE_NAMES`
   under the cloned subtree, `carry_relationship(dest=A, A.main, src=B,
   B.main, rid, |_| true)` and rewrite the attribute (images, hyperlinks,
   embedded objects); external targets verbatim.
2. Styles: collect `w:pStyle`, `w:rStyle`, `w:tblStyle`, `w:numStyleLink`,
   `w:styleLink` values under the cloned subtree; for each id not present in
   A's `styles.xml` by **name and type** (the pairing rule of commit
   `3863718`: custom names exactly, built-ins case-insensitively), copy B's
   `w:style` with its `w:basedOn`/`w:link`/`w:next` chain; when A has a
   style of the same name but a different id, rewrite the reference to A's
   id; when A has the same id for a different name, rename B's copy
   `{id}B` and rewrite the references.
3. Numbering: for every `w:numId` under the cloned subtree, copy B's
   `w:num` and its `w:abstractNum` with ids offset past A's maxima (reuse
   `max_attribute` and the "every `w:abstractNum` precedes every `w:num`"
   splice from `markdown::package::numbering`), rewrite `w:numId`.
4. Notes: for every `w:footnoteReference`/`w:endnoteReference`, copy the
   note with an id past A's maximum (A's notes part created with separators
   through `markdown::package::footnotes` when absent), rewrite the id.
5. Comments: not carried in this task; the cloned subtree's
   `commentRangeStart`/`End`/`commentReference` are removed and
   `COMMENTS_DROPPED` is warned (a later task can route them through
   `comparer::comments::install_parts_from`).
6. `comparer::fixups::fix_up_drawing_ids_in_package` (unique `wp:docPr`
   ids), then `validate()` (plan 1 Task 2) must be clean, else
   `AppendError::Invalid(findings)`.

- [x] **Step 1: Failing tests**

```rust
// tests/append_documents.rs
mod common;
use common::docx::{Part, R_NS, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::append::{AppendOptions, SectionBreak, append_documents};
use jubarte::inspect::{paragraphs, summary};

const PNG_1X1: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, b'I', b'H', b'D', b'R', 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, b'I', b'D', b'A', b'T', 0x78, 0x9C, 0x63, 0, 1, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82];

fn b_with_image() -> Vec<u8> {
    let drawing = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><wp:docPr id="1" name="Pic" descr="dot"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="dot.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdX0"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;
    // `docx_with` writes extras as text; a binary part needs the zip
    // directly. Build with docx_with for the rels/overrides, then replace
    // the media entry's bytes with PNG_1X1 through zip (helper in the test).
    let image_rel = format!("{R_NS}/image");
    let doc = docx_with(&(para("B says hello.") + drawing), &[Part { name: "word/media/dot.png", content_type: "image/png", rel_type: &image_rel, xml: "" }]);
    common::docx::replace_entry(&doc, "word/media/dot.png", PNG_1X1)
}

#[test]
fn text_follows_a_then_b_and_the_image_relationship_is_carried() {
    let a = docx(&para("A says hi."));
    let out = append_documents(&a, &b_with_image(), &AppendOptions::default()).unwrap();
    assert_word_valid_package(&out.docx);
    let texts: Vec<String> = paragraphs(&out.docx).unwrap().into_iter().map(|p| p.text).collect();
    assert_eq!(texts[0], "A says hi.");
    assert!(texts.iter().any(|t| t == "B says hello."), "{texts:?}");
    assert_eq!(summary(&out.docx).unwrap().images, 1);
    let rels = part_string(&out.docx, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains("/image"), "{rels}");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

#[test]
fn numbering_from_both_sides_keeps_distinct_ids() {
    let numbering = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:nsid w:val="0A0B0C0D"/><w:multiLevelType w:val="hybridMultilevel"/><w:tmpl w:val="0A0B0C0E"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/><w:lvlJc w:val="left"/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#;
    let listed = |text: &str| {
        let numbering_rel = format!("{R_NS}/numbering");
        docx_with(
            &format!(r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#),
            &[Part { name: "word/numbering.xml", content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml", rel_type: &numbering_rel, xml: numbering }],
        )
    };
    let out = append_documents(&listed("a item"), &listed("b item"), &AppendOptions { section_break: SectionBreak::Continuous, keep_sections: false }).unwrap();
    assert_word_valid_package(&out.docx);
    let xml = part_string(&out.docx, "word/numbering.xml").unwrap();
    assert_eq!(xml.matches("<w:num ").count(), 2);
    assert_eq!(xml.matches("<w:abstractNum ").count(), 2);
    let doc = part_string(&out.docx, "word/document.xml").unwrap();
    assert!(doc.contains(r#"<w:numId w:val="1"/>"#) && doc.contains(r#"<w:numId w:val="2"/>"#), "{doc}");
}

#[test]
fn b_comments_are_dropped_with_a_warning() {
    let a = docx(&para("A."));
    let b = jubarte::edit::apply_plan(&docx(&para("B.")), &jubarte::edit::EditPlan::from_json(r#"{"schema_version":1,"author":"X","operations":[{"kind":"comment","paragraph":"body:p:0","text":"note"}]}"#).unwrap()).unwrap().clean;
    let out = append_documents(&a, &b, &AppendOptions::default()).unwrap();
    assert_word_valid_package(&out.docx);
    assert_eq!(out.warnings, vec!["COMMENTS_DROPPED: 1 comment of B was not carried".to_string()]);
}
```

(`common::docx::replace_entry` is a 15-line helper to add to
`tests/common/docx.rs`: rewrite one ZIP entry's bytes.)

- [x] **Step 2: Run, expect `could not find append in jubarte`.**

- [x] **Step 3: Implement** as designed; CLI `jubarte append A B [C...] -o
OUT [--section-break next-page|continuous|none] [--keep-sections]`
(folds left: `append(append(A, B), C)`); Python `Document.append(other,
*, section_break="next_page", keep_sections=False) -> Document` with
warnings exposed on `Document.append_report` (or return a small
`Appended` dataclass; pick the dataclass); WASM `appendDocuments`.

- [x] **Step 4: Pass**, then `cargo test --all-features` once.

- [x] **Step 5: Docs** (skill §4: "`jubarte append a.docx b.docx -o
ab.docx` puts B after A on a new page; images, links, styles, lists and
notes come along; comments do not yet (warned)"); CHANGELOG.

- [x] **Step 6: Commit** `feat(append): append documents carrying relationships, styles, numbering and notes; validated output`.

**Evidence this task hands a provider:** `docs/adoption/append.md`: two
real-looking documents (letter + exhibit with an image and a list), the
python-docx body-copy recipe's result (dangling `rId`, duplicate `numId`)
beside `jubarte append`'s, both run through `jubarte validate`.

(2026-10-02: not written yet. A provider's reviewer will compare with
docxcompose, which already maps styles, renumbers lists and copies images
and footnotes, so the guide adds it as a third recipe; until `jubarte
validate` is on `main`, the outputs go through `tools/validate-docx` and
the Ring-1 checks. See `TODO.md` §9.)

---

## Counterarguments the plan does not hide

- **Admission changes a public default.** A caller feeding compare a
  70 MB `.docx` today gets a result; after Task 1 it gets `INPUT_LIMIT`
  unless it opts out. That is the point, and it is a minor-version change
  with a documented opt-out, not a silent one.
- **Pixel diffs are exact, so a one-pixel kerning change on every line
  looks like a rewritten page.** `changed_ratio` and `bbox` tell the agent
  how big the change is; a tolerance knob is deliberately absent until a
  user shows a document that needs one.
- **Append is not a merge.** Conflicting style definitions resolve by
  name in A's favour; comments are dropped with a warning. Both are stated
  in the output so an agent does not report success it cannot see.
