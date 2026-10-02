<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Provider Adoption Plan 3 of 4: S14 to S16 Implementation Plan

> **Execution:** inline, by one engineer, no subagents. Run every Cargo
> command from the repository root, one at a time, in the default `target/`
> (`AGENTS.md`). Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Baseline:** `main` at `b420d64` (2026-10-02); the verified facts, gates
> and license position are in plan 1
> ([2026-10-02-provider-adoption-1-s01-s10.md](2026-10-02-provider-adoption-1-s01-s10.md)).

**Goal:** Fill content controls (S14), audit documents for accessibility and
style defects (S15), and ship an MCP server plus a Gemini CLI extension
(S16), so that the ChatGPT container's `content_controls.py`,
`a11y_audit.py` and `style_lint`, Z.ai's `postcheck.py`, and Google's
`safe-docx` incumbent each have a jubarte answer.

**Architecture:** S14 and S15 are read-side additions to `inspect` plus one
new edit operation and one new report module; they reuse the DOM walk
`inspect::project_paragraph` already does. S16 is a thin Python module over
the existing `Document` facade, exposed through the MCP Python SDK 2.x
(`mcp.server.MCPServer`), installed as an optional extra, and declared to
Gemini CLI by a manifest at the repository root so the existing
`skills/jubarte-documents/SKILL.md` is picked up as the extension's skill.

**Tech Stack:** as plan 1, plus `mcp>=2.2,<3` (Python, MIT) for S16.

---

## Why a provider would take this

| Provider need (verified 2026-10-02 unless marked) | Today | After this plan |
|---|---|---|
| Google: Gemini CLI reads `.docx` as binary (issue #20298, reported in the mapping document); `UseJunior/safe-docx` is the MCP extension on geminicli.com/extensions | no jubarte MCP server | `gemini extensions install https://github.com/jandira-tech/jubarte-redlines`: MCP tools plus the skill, one install |
| Anthropic Claude Code and OpenAI Codex both load MCP servers from a config file | CLI only | the same `jubarte-mcp` server over stdio, with path containment |
| ChatGPT container `content_controls.py` (reported, not verified) | detection only in `inspect` (`limitations: content_control`) | `inspect` lists controls with tag, alias, kind and choices; `fill_control` writes them |
| ChatGPT container `a11y_audit.py`, `style_lint`/`style_normalize`; Z.ai `postcheck.py` (reported, not verified) | nothing | `jubarte audit FILE --json`: eight rules with codes, paragraph ids and severities |

## Dependencies

| Task | Suggestion | Depends on | Unlocks |
|---|---|---|---|
| 1 | S14 content controls | plan 1 Task 4 (`occurrence`) for the anchor form; plan 1 Task 2 (`validate`) | form filling |
| 2 | S15 audit | plan 1 Task 7 (`layout_facts`) for `STALE_FIELD_CACHE`; plan 1 Task 5 for `FONT_SUBSTITUTED` | QA gates |
| 3 | S16 MCP and Gemini | plan 1 Task 1 (Windows wheel; `uvx` on every OS); plan 2 Task 1 (admission: an MCP server is an untrusted-input surface) | Google adoption |

---

### Task 1: S14, content controls: list and fill

**Files:**
- Modify: `src/inspect.rs:108-135` (`Snapshot.controls: Vec<ContentControl>`), `:684` (the `sdt` arm records the control), new `fn collect_controls(dom, body) -> Vec<ContentControl>`
- Modify: `src/edit.rs` (`OperationKind::FillControl`), `check_operation_keys`, `resolve_one`, `apply`
- Modify: `src/capabilities.rs` (`operations.content_controls`, `edit_operations += "fill_control"`), `tests/agent_contracts.rs`
- Modify: `jubarte-python/python/jubarte_redlines/models.py` (`ContentControl`, `Snapshot.controls`, `EditPlan.fill_control`), `_native.pyi` unchanged (JSON carries it)
- Modify: `jubarte-wasm` nothing (JSON carries it)
- Test: `tests/edit_fill_control.rs`, `tests/inspect_controls.rs`, `jubarte-python/tests/test_controls.py`
- Docs: skill §1 and §2, CHANGELOG

Wire:

```rust
/// A content control (`w:sdt`) in the body, in document order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContentControl {
    /// `body:sdt:N`.
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// `text`, `rich_text`, `drop_down`, `combo_box`, `date`, `checkbox`,
    /// `picture`, `group`, `repeating`, `building_block`, `citation`,
    /// `bibliography`, `equation`, `unknown`.
    pub kind: String,
    /// Visible text of the control's content.
    pub text: String,
    /// Paragraphs the control spans (`body:p:N`), empty for a run-level control.
    pub paragraph_ids: Vec<String>,
    /// `w:lock` is `contentLocked` or `sdtContentLocked`.
    pub locked: bool,
    /// Drop-down and combo-box choices (`w:listItem/@w:value`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    /// Checkbox state (`w14:checked`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// `w:showingPlcHdr` is set: the text is placeholder text.
    pub placeholder: bool,
}
```

`fill_control` operation:

```json
{"kind":"fill_control","control":{"tag":"Name"},"text":"Ada Lovelace"}
{"kind":"fill_control","control":{"alias":"Country"},"choice":"Brazil"}
{"kind":"fill_control","control":"body:sdt:3","checked":true}
{"kind":"fill_control","control":{"tag":"Signed"},"date":"2026-10-02"}
```

Exactly one of `text`, `choice`, `checked`, `date` (`INVALID_EDIT`
otherwise). `control` is an id string or `{tag}`/`{alias}`; both must match
exactly one control (`AMBIGUOUS_ANCHOR` with the ids, `ANCHOR_NOT_FOUND`).
Effects, all inside `w:sdtContent`, `w:sdtPr` untouched except
`w:showingPlcHdr` removed: `text` replaces the content runs with one run
carrying the first existing run's `w:rPr` (a rich-text control with several
paragraphs keeps the first paragraph's `w:pPr`); `choice` must be one of
`choices` (`INVALID_EDIT` naming them) and writes the matching
`w:listItem/@w:displayText`; `checked` writes `w14:checked w14:val` and the
glyph run (`☒` U+2612 or `☐` U+2610 in `MS Gothic`, as Word does);
`date` writes `w:date w:fullDate="YYYY-MM-DDT00:00:00Z"` and the text in
the control's `w:dateFormat` (default `yyyy-MM-dd`). `locked` controls are
refused with `LOCKED_CONTROL`; `picture`, `group`, `repeating`,
`building_block` with `UNSUPPORTED_STRUCTURE`.

The redline: the fill is applied to the base's clean copy and the redline
comes from compare. Step 1's third test decides whether compare keeps the
`w:sdt` wrapper in the redline (`tests/m25_sanitize_sdt_pr.rs` shows the
comparer sanitizes `w:sdtPr`; whether the wrapper survives in the output is
not known). The test is written for the desired behaviour (wrapper kept,
change inside). If it fails, record the finding in `KNOWN_ISSUES.md`,
keep the clean copy as the deliverable, and leave the test `#[ignore]`d with
the issue number; do not weaken it.

- [ ] **Step 1: Failing tests**

```rust
// tests/edit_fill_control.rs
mod common;
use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::{inspect_json, paragraphs};

fn form() -> Vec<u8> {
    let name = r#"<w:p><w:r><w:t xml:space="preserve">Name: </w:t></w:r><w:sdt><w:sdtPr><w:alias w:val="Full name"/><w:tag w:val="Name"/><w:id w:val="101"/><w:showingPlcHdr/><w:text/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:rStyle w:val="PlaceholderText"/></w:rPr><w:t>Click here</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    let country = r#"<w:p><w:sdt><w:sdtPr><w:tag w:val="Country"/><w:id w:val="102"/><w:dropDownList><w:listItem w:displayText="Brazil" w:value="BR"/><w:listItem w:displayText="Chile" w:value="CL"/></w:dropDownList></w:sdtPr><w:sdtContent><w:r><w:t>Choose</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    let locked = r#"<w:p><w:sdt><w:sdtPr><w:tag w:val="Ref"/><w:id w:val="103"/><w:lock w:val="sdtContentLocked"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>FIXED</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    docx(&format!("{name}{country}{locked}"))
}

#[test]
fn inspect_lists_the_controls_with_tag_kind_and_choices() {
    let json: serde_json::Value = serde_json::from_str(&inspect_json(&form()).unwrap()).unwrap();
    let controls = json["controls"].as_array().unwrap();
    assert_eq!(controls.len(), 3);
    assert_eq!(controls[0]["tag"], "Name");
    assert_eq!(controls[0]["kind"], "text");
    assert_eq!(controls[0]["placeholder"], true);
    assert_eq!(controls[1]["kind"], "drop_down");
    assert_eq!(controls[1]["choices"], serde_json::json!(["BR", "CL"]));
    assert_eq!(controls[2]["locked"], true);
}

#[test]
fn fill_text_and_choice_keep_the_properties_and_are_valid() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"fill_control","control":{"tag":"Name"},"text":"Ada Lovelace"},
        {"kind":"fill_control","control":{"tag":"Country"},"choice":"BR"}]}"#).unwrap();
    let out = apply_plan(&form(), &plan).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let texts: Vec<String> = paragraphs(&out.clean).unwrap().into_iter().map(|p| p.text).collect();
    assert_eq!(texts[0], "Name: Ada Lovelace");
    assert_eq!(texts[1], "Brazil");
    let xml = part_string(&out.clean, "word/document.xml").unwrap();
    assert!(xml.contains(r#"<w:tag w:val="Name"/>"#) && !xml.contains("showingPlcHdr"), "{xml}");
    // Decision test: the redline keeps the control and tracks inside it.
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(red.contains(r#"<w:tag w:val="Name"/>"#), "the comparer dropped the sdt wrapper; see the task text");
}

#[test]
fn locked_controls_and_bad_choices_are_refused() {
    let locked = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[{"kind":"fill_control","control":{"tag":"Ref"},"text":"x"}]}"#).unwrap();
    assert_eq!(apply_plan(&form(), &locked).unwrap_err().code, "LOCKED_CONTROL");
    let bad = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[{"kind":"fill_control","control":{"tag":"Country"},"choice":"AR"}]}"#).unwrap();
    let e = apply_plan(&form(), &bad).unwrap_err();
    assert_eq!(e.code, "INVALID_EDIT");
    assert!(e.message.contains("BR") && e.message.contains("CL"));
}
```

- [ ] **Step 2: Run, expect `controls` missing from the JSON and `unknown kind "fill_control"`.**

- [ ] **Step 3: Implement** as designed; Python `EditPlan.fill_control(control, *, text=None, choice=None, checked=None, date=None, id=None)` and `Snapshot.controls`.

- [ ] **Step 4: Pass**; record the decision test's outcome in the commit message.

- [ ] **Step 5: Docs**: skill §1 ("`inspect` lists `controls`: tag, alias,
kind, choices, locked, placeholder"), §2 (`fill_control` with the four
value forms). CHANGELOG.

- [ ] **Step 6: Commit** `feat(edit): list and fill content controls (text, choice, checkbox, date); locked controls refused`.

**Evidence this task hands a provider:** `examples/agents/fill-form/`: a
form with six controls, the `inspect` listing, one plan, the filled clean
copy and its PNG.

---

### Task 2: S15, `jubarte audit`: accessibility, style and structure rules

**Files:**
- Create: `src/audit.rs`
- Modify: `src/lib.rs`, `src/bin/jubarte.rs` (`Command::Audit`), `src/capabilities.rs` (`operations.audit`, `audit_rules: Vec<String>`)
- Bindings: `Document.audit(rules=None) -> tuple[AuditFinding, ...]`, WASM `auditDocument`
- Test: `tests/audit_rules.rs` (one test per rule), `jubarte-python/tests/test_audit.py`
- Docs: skill §3, CHANGELOG

Wire:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuditFinding {
    /// Rule code.
    pub code: String,
    /// `a11y`, `style` or `structure`.
    pub rule_set: String,
    /// `error`, `warning` or `info`.
    pub severity: String,
    /// `body:p:N`, a story paragraph id, or a part name.
    pub location: String,
    pub message: String,
}
pub fn audit(docx: &[u8], rules: &[&str]) -> Result<Vec<AuditFinding>, AuditError>;
pub const RULES: &[(&str, &str, &str)]; // (code, rule_set, severity)
```

Rules, each with the fact it reads:

| Code | Set | Severity | Fires when |
|---|---|---|---|
| `HEADING_SKIP` | a11y | warning | a `Heading{n}` paragraph follows a `Heading{m}` with `n > m + 1`, or the first heading is not level 1 |
| `IMAGE_NO_DESCR` | a11y | error | `wp:docPr` has no non-empty `descr` (and no `w:decorative`) |
| `TABLE_NO_HEADER_ROW` | a11y | warning | a table with 2+ rows has no `w:tblHeader` on its first row |
| `MISSING_LANG` | a11y | warning | neither `w:docDefaults/w:rPr/w:lang` nor `Normal`'s `w:lang` nor `w:themeFontLang` is set |
| `LITERAL_BULLET` | style | warning | a non-numbered paragraph's text starts with `•`, `◦`, `▪`, `-`, `*` or `·` followed by a space |
| `EMPTY_SPACER_PARAGRAPH` | style | info | two or more consecutive empty paragraphs (not in tables, not before a section break) |
| `DIRECT_FORMATTING_OVERRIDES_STYLE` | style | info | more than 30% of runs in `Normal`-styled paragraphs carry a direct `w:rFonts` or `w:sz` differing from the style |
| `STALE_FIELD_CACHE` | structure | warning | a TOC field with no result, or a `NUMPAGES` result that is not `layout_facts().page_count` (plan 1 Task 7) |
| `FONT_SUBSTITUTED` | structure | info | `RenderReport.fonts` has a `substituted` entry (plan 1 Task 5) |

`jubarte audit FILE [--json] [--rules a11y,style,structure | CODE,...]`
exits 0 with no findings, 2 with any `error`, 0 otherwise (warnings do not
fail a CI step unless `--strict`). The two rules that need a layout pass
(`STALE_FIELD_CACHE`, `FONT_SUBSTITUTED`) run only when selected or when no
`--rules` is given, and the report says `layout: true` when they ran.

- [ ] **Step 1: Failing tests**

```rust
// tests/audit_rules.rs
mod common;
use common::docx::{docx, para};
use jubarte::audit::audit;

fn codes(docx: &[u8], rules: &[&str]) -> Vec<String> {
    audit(docx, rules).unwrap().into_iter().map(|f| f.code).collect()
}

#[test]
fn literal_bullet_and_heading_skip() {
    let body = String::new()
        + r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#
        + r#"<w:p><w:pPr><w:pStyle w:val="Heading3"/></w:pPr><w:r><w:t>Deep</w:t></w:r></w:p>"#
        + &para("• item");
    let found = audit(&docx(&body), &["a11y", "style"]).unwrap();
    assert!(found.iter().any(|f| f.code == "HEADING_SKIP" && f.location == "body:p:1"), "{found:?}");
    assert!(found.iter().any(|f| f.code == "LITERAL_BULLET" && f.location == "body:p:2"), "{found:?}");
}

#[test]
fn a_numbered_paragraph_is_not_a_literal_bullet() {
    let body = r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>- dash in a real list</w:t></w:r></w:p>"#;
    assert!(!codes(&docx(body), &["style"]).contains(&"LITERAL_BULLET".to_string()));
}

#[test]
fn table_without_header_row_and_image_without_descr() {
    let table = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>h</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>d</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let image = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><wp:docPr id="1" name="Pic"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"/></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;
    let found = codes(&docx(&format!("{table}{image}")), &["a11y"]);
    assert!(found.contains(&"TABLE_NO_HEADER_ROW".to_string()) && found.contains(&"IMAGE_NO_DESCR".to_string()), "{found:?}");
}

#[test]
fn spacers_and_missing_lang() {
    let body = para("a") + "<w:p/><w:p/>" + &para("b");
    let found = codes(&docx(&body), &["style", "a11y"]);
    assert!(found.contains(&"EMPTY_SPACER_PARAGRAPH".to_string()) && found.contains(&"MISSING_LANG".to_string()), "{found:?}");
}

#[test]
fn an_empty_toc_is_a_stale_cache() {
    let toc = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    assert!(codes(&docx(toc), &["STALE_FIELD_CACHE"]).contains(&"STALE_FIELD_CACHE".to_string()));
}
```

- [ ] **Step 2: Run, expect `could not find audit`.** **Step 3: Implement**
`src/audit.rs` over `inspect::Opened` (admitted) and the `Paragraph`
projections; `styles.xml` read once for `MISSING_LANG` and
`DIRECT_FORMATTING_OVERRIDES_STYLE`; CLI; bindings. **Step 4: Pass.**
**Step 5: Docs**: skill §3 adds "`jubarte audit file.docx --json` lists
heading skips, images without alt text, tables without a header row, literal
bullets, spacer paragraphs, stale TOC and page-count caches, and substituted
fonts; `--strict` makes warnings fail." **Step 6: Commit**
`feat(audit): accessibility, style and structure rules with paragraph locations`.

**Evidence this task hands a provider:** `docs/adoption/audit.md`: the
Anthropic skill's own footguns (literal `•`, A4 default, table widths) as
audit findings on a document docx-js produced with them, so the audit is
the regression test for their creation path.

---

### Task 3: S16, MCP server and Gemini CLI extension

**Files:**
- Create: `jubarte-python/python/jubarte_redlines/mcp_server.py`
- Modify: `jubarte-python/pyproject.toml` (`[project.optional-dependencies] mcp = ["mcp>=2.2,<3"]`; `[project.scripts] jubarte-mcp = "jubarte_redlines.mcp_server:main"`)
- Create: `jubarte-python/tests/test_mcp_server.py`
- Create: `gemini-extension.json` (repository root), `GEMINI.md`
- Modify: `scripts/bump-version.mjs` (sync `gemini-extension.json` version), `scripts/test_bump_version.py`
- Create: `scripts/test_gemini_extension.py` (manifest shape and version sync); add to `ci.yml` `convert-sweep-unit`
- Create: `docs/adoption/mcp.md` (Claude Code `.mcp.json`, Codex `config.toml`, Gemini install, security model)
- Modify: `README.md`, `CHANGELOG.md`, skill "Dependencies"

Facts: MCP Python SDK 2.2.0 (`pip install mcp`, Python 3.10+, MIT):
`from mcp.server import MCPServer`, `@mcp.tool()`, `ToolError` from
`mcp.server.mcpserver.exceptions`; `Client(server_instance)` connects
in-process for tests; `uv run mcp run server.py` serves stdio by default.
Gemini CLI extension reference (page dated 2026-05-14): a repository is
installable with `gemini extensions install <github url>`; the manifest is
`gemini-extension.json` with `name`, `version`, `description`,
`mcpServers` (`command`, `args`, `cwd`, `${extensionPath}`,
`${workspacePath}`), `contextFileName`; `skills/<name>/SKILL.md` in the
extension directory is loaded as a skill; environment variables are
sanitized unless declared in `settings`. Gemini CLI stopped serving
unpaid and Google One accounts on 2026-06-18 (banner on the same page);
enterprise and API-key users remain, which is the audience that matters
for a provider decision.

Security model, written before the code: the server is a file-system
tool. Every path argument must resolve under `--root` (default: the
current directory) or the tool raises `ToolError("path outside root")`; no
tool overwrites an existing file without `overwrite=true`; large outputs
(PNG pages, PDFs, documents) are written to files under `root` and their
paths returned, never base64 in the response; document text is returned
only by the tools whose job is to return it (`docx_text`, `docx_inspect`,
`docx_changes`, `docx_comments`); all engine calls go through `Document`,
which admits input (plan 2 Task 1). Nothing listens on a network port
unless the user runs the server with an HTTP transport themselves.

Tools (names are the contract; the docstring is the description the model
reads):

| Tool | Arguments | Returns |
|---|---|---|
| `docx_capabilities` | | the manifest dict |
| `docx_text` | `path` | Markdown with `[body:p:N]` ids |
| `docx_inspect` | `path` | the snapshot dict (`summary`, `paragraphs`, `stories`, `controls`) |
| `docx_edit` | `path`, `plan` (dict), `out_dir`, `pdf=False`, `png_dpi=None`, `overwrite=False` | `{clean, redline, patch, report}` paths and the report dict; a refusal returns `is_error` with the `EditPlanError` payload |
| `docx_render` | `path`, `out_dir`, `dpi=96`, `pages=None`, `pdf=False` | page PNG paths and the render report |
| `docx_compare` | `original`, `modified`, `out`, `author`, `date=None` | `out` and the change list |
| `docx_changes` | `path` | the change records |
| `docx_accept` / `docx_reject` | `path`, `out`, `ids=None`, `authors=None`, `kinds=None` | `out` |
| `docx_validate` | `path`, `original=None`, `author=None` | findings |
| `docx_comments` | `path` | comment records (plan 1 Task 3) |
| `docx_audit` | `path`, `rules=None` | audit findings (Task 2) |

- [ ] **Step 1: Failing test**

```python
# jubarte-python/tests/test_mcp_server.py
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""The MCP server exposes the Document facade with path containment."""
from __future__ import annotations

import json
from pathlib import Path

import anyio
import pytest

pytest.importorskip("mcp")
from mcp import Client  # noqa: E402
from mcp.types import TextContent  # noqa: E402

from jubarte_redlines.mcp_server import build_server  # noqa: E402
from test_document import make_document  # noqa: E402

TOOLS = {
    "docx_capabilities", "docx_text", "docx_inspect", "docx_edit", "docx_render",
    "docx_compare", "docx_changes", "docx_accept", "docx_reject", "docx_validate",
    "docx_comments", "docx_audit",
}


def run(coro):
    return anyio.run(lambda: coro)


def test_the_tool_set_is_the_contract(tmp_path: Path) -> None:
    async def go():
        async with Client(build_server(root=tmp_path)) as client:
            listed = await client.list_tools()
            return {t.name for t in listed.tools}
    assert run(go()) == TOOLS


def test_text_reads_a_file_under_root_and_refuses_one_outside(tmp_path: Path) -> None:
    inside = tmp_path / "in.docx"
    inside.write_bytes(make_document("inside text"))
    outside = tmp_path.parent / "outside.docx"
    outside.write_bytes(make_document("outside text"))

    async def go():
        async with Client(build_server(root=tmp_path)) as client:
            ok = await client.call_tool("docx_text", {"path": str(inside)})
            bad = await client.call_tool("docx_text", {"path": str(outside)})
            return ok, bad
    ok, bad = run(go())
    assert not ok.is_error and "[body:p:0] inside text" in ok.content[0].text
    assert bad.is_error and "outside root" in bad.content[0].text


def test_edit_writes_outputs_under_out_dir_and_never_overwrites(tmp_path: Path) -> None:
    src = tmp_path / "c.docx"
    src.write_bytes(make_document("The fee is ten."))
    plan = {"schema_version": 1, "author": "Agent", "operations": [
        {"kind": "replace", "paragraph": "body:p:0", "find": "ten", "replacement": "twelve"}]}

    async def go():
        async with Client(build_server(root=tmp_path)) as client:
            first = await client.call_tool("docx_edit", {"path": str(src), "plan": plan, "out_dir": str(tmp_path / "review")})
            second = await client.call_tool("docx_edit", {"path": str(src), "plan": plan, "out_dir": str(tmp_path / "review")})
            return first, second
    first, second = run(go())
    assert not first.is_error
    paths = first.structured_content
    assert Path(paths["clean"]).exists() and Path(paths["redline"]).exists()
    assert paths["report"]["ok"] is True
    assert second.is_error and "overwrite" in second.content[0].text
```

- [ ] **Step 2: Run** (`uv run --with maturin maturin develop --release &&
uv run --with pytest --with 'mcp>=2.2,<3' --with anyio pytest -q
tests/test_mcp_server.py`), expect `ModuleNotFoundError: jubarte_redlines.mcp_server`.

- [ ] **Step 3: Implement**

```python
# jubarte-python/python/jubarte_redlines/mcp_server.py
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""`jubarte-mcp`: the Document facade as MCP tools over stdio.

Paths are contained under ``--root``; outputs are files, not payloads.
Install with ``pip install 'jubarte-redlines[mcp]'`` or run with
``uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .``.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict
from pathlib import Path

from mcp.server import MCPServer
from mcp.server.mcpserver.exceptions import ToolError

from . import Document, EditPlanError, capabilities, read

INSTRUCTIONS = (
    "Read a .docx with docx_text (paragraph ids), edit it with docx_edit "
    "(a JSON plan of exact anchored operations; the output is a clean copy "
    "and a Word redline), verify with docx_render and docx_validate. "
    "Every path must be under the server's root."
)


def build_server(*, root: Path) -> MCPServer:
    root = root.resolve()
    mcp = MCPServer("jubarte", instructions=INSTRUCTIONS)

    def contained(path: str) -> Path:
        p = Path(path).expanduser().resolve()
        if not p.is_relative_to(root):
            raise ToolError(f"{path} is outside root {root}")
        return p

    def fresh(path: Path, overwrite: bool) -> Path:
        if path.exists() and not overwrite:
            raise ToolError(f"{path} exists; pass overwrite=true to replace it")
        path.parent.mkdir(parents=True, exist_ok=True)
        return path

    @mcp.tool()
    def docx_capabilities() -> dict:
        """What this engine build can do (operations, limits, edit kinds)."""
        return capabilities()

    @mcp.tool()
    def docx_text(path: str) -> str:
        """The document as Markdown with [body:p:N] paragraph ids, the coordinates an edit plan uses."""
        return read(contained(path)).markdown()

    @mcp.tool()
    def docx_inspect(path: str) -> dict:
        """Paragraphs with ids, styles, runs and limitations; summary; stories; content controls."""
        return json.loads(read(contained(path)).inspect_json())

    @mcp.tool()
    def docx_edit(path: str, plan: dict, out_dir: str, pdf: bool = False, png_dpi: float | None = None, overwrite: bool = False) -> dict:
        """Apply an edit plan: writes clean.docx, redline.docx, patch.diff and report.json under out_dir."""
        doc = read(contained(path))
        out = contained(out_dir)
        try:
            result = doc.edit(plan)
        except EditPlanError as e:
            raise ToolError(json.dumps({"code": e.code, "operation": e.operation, "message": str(e), "outcomes": [asdict(o) for o in e.outcomes]})) from e
        clean = fresh(out / "clean.docx", overwrite); clean.write_bytes(result.clean.to_bytes())
        redline = fresh(out / "redline.docx", overwrite); redline.write_bytes(result.redline.to_bytes())
        patch = fresh(out / "patch.diff", overwrite); patch.write_text(str(result.diff))
        report = fresh(out / "report.json", overwrite); report.write_text(json.dumps(asdict(result.report)))
        pages: list[str] = []
        if pdf or png_dpi:
            rendered = result.redline.render(pdf=pdf, png_dpi=png_dpi)
            if rendered.pdf is not None:
                p = fresh(out / "redline.pdf", overwrite); p.write_bytes(rendered.pdf)
            for i, png in enumerate(rendered.pngs, start=1):
                p = fresh(out / f"redline-page-{i:02d}.png", overwrite); p.write_bytes(png); pages.append(str(p))
        return {"clean": str(clean), "redline": str(redline), "patch": str(patch), "report": asdict(result.report), "pages": pages}

    # docx_render, docx_compare, docx_changes, docx_accept, docx_reject,
    # docx_validate, docx_comments, docx_audit follow the same shape:
    # contained() on every path, fresh() on every output, Document methods.
    return mcp


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="jubarte-mcp", description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="directory every path must sit under")
    args = parser.parse_args(argv)
    build_server(root=args.root).run()  # stdio
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

(`Document.inspect_json()` does not exist today; add it as a thin method
returning `_native.inspect_json(self._data)` so the tool returns the engine's
JSON unchanged. `asdict` on frozen dataclasses with tuples is fine;
`EditReport` holds tuples of dataclasses.)

- [ ] **Step 4: Pass** the three tests; then a stdio smoke:
`echo '{}' | timeout 5 uv run --with 'mcp>=2.2,<3' jubarte-mcp --root /tmp; echo $?` exits without a traceback.

- [ ] **Step 5: Gemini extension**

`gemini-extension.json` at the repository root:

```json
{
  "name": "jubarte-redlines",
  "version": "0.11.0",
  "description": "Read, edit as tracked changes, comment, compare, accept/reject and render Word .docx files with one engine; no LibreOffice, pandoc or Poppler.",
  "mcpServers": {
    "jubarte": {
      "command": "uvx",
      "args": ["--from", "jubarte-redlines[mcp]", "jubarte-mcp", "--root", "${workspacePath}"]
    }
  },
  "contextFileName": "GEMINI.md"
}
```

`GEMINI.md` (20 lines): what the tools are, "read before you edit", the
plan shape, and a pointer to `skills/jubarte-documents/SKILL.md`, which the
extension loads as the `jubarte-documents` skill because it sits in
`skills/`. `scripts/bump-version.mjs` learns the manifest's `version`;
`scripts/test_gemini_extension.py` asserts the manifest parses, `name`
matches `^[a-z0-9-]+$`, `version` equals `Cargo.toml`'s, and `mcpServers`
uses `command`+`args`. Install check on a machine with Gemini CLI:
`gemini extensions install https://github.com/jandira-tech/jubarte-redlines`
then `/extensions list` shows it; record the output in `docs/adoption/mcp.md`.
The gallery listing process on geminicli.com/extensions was not verified in
this plan; check it when submitting.

Claude Code and Codex snippets go in `docs/adoption/mcp.md`:

```json
{ "mcpServers": { "jubarte": { "command": "uvx", "args": ["--from", "jubarte-redlines[mcp]", "jubarte-mcp", "--root", "."] } } }
```

(Claude Code `.mcp.json` shape verified from Claude Code's own docs at
writing time; the Codex `config.toml` `[mcp_servers.jubarte]` shape is
from Codex's docs and must be re-checked when the page is written.)

- [ ] **Step 6: Docs and commit**

README: an "MCP" section with the one-line install per host.
CHANGELOG. Commit:

```bash
git add jubarte-python/python/jubarte_redlines/mcp_server.py jubarte-python/pyproject.toml jubarte-python/tests/test_mcp_server.py gemini-extension.json GEMINI.md scripts/bump-version.mjs scripts/test_bump_version.py scripts/test_gemini_extension.py .github/workflows/ci.yml docs/adoption/mcp.md README.md CHANGELOG.md
git commit -m "feat(mcp): jubarte-mcp server with path containment; Gemini CLI extension manifest"
```

**Evidence this task hands a provider:** `docs/adoption/mcp.md` with a
transcript of Gemini CLI reading a `.docx` through the extension (the exact
failure of issue #20298, then the fix), and the safe-docx comparison table:
tools offered, license (jubarte AGPL-3.0-only; safe-docx reported as
Apache-2.0 or MIT by different listings, unresolved), tracked changes
(safe-docx: not in its listing as read; jubarte: yes), rendering (safe-docx:
none listed; jubarte: PDF and PNG).

---

## Counterarguments the plan does not hide

- **safe-docx is permissively licensed and already in the gallery.** On
  license alone Google may keep it. The pitch is capability (tracked
  changes, rendering, validation) and admission, not license.
- **An MCP server is a new untrusted-input surface.** Hence root
  containment, no overwrite, no payload bloat, and plan 2's admission
  before this task.
- **Content-control redlines may flatten the control** (the decision test
  in Task 1). The clean copy is still correct; the redline's fidelity is
  what is at stake, and the test will say which.
- **Audit rules are heuristics.** Each one names a paragraph so a human can
  disagree quickly; none of them edits anything.
