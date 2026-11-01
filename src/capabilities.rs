// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What this build can do. Agents read it before choosing an operation:
//! `jubarte capabilities --json`, `jubarte_redlines.capabilities()`.
//!
//! Every operation is compiled into the library unconditionally: the crate's
//! features (`cli`, `fast-alloc`, `self-update`, `perf-profile`) gate the
//! binary, its allocator, updating and profiling, not an operation, so there
//! is no feature to derive `operations` from and each is reported `true`. A
//! wrapper that compiles an operation out overrides that field in its own
//! manifest, as jubarte-wasm does for `pdf` (its `pdf` feature) and `png`
//! (never). The rest is read from the modules that implement it: the v1 plan
//! version and the story kinds come from `crate::inspect`, and whether a
//! runtime dispatches PlanV2 from its own surface, so the manifest cannot
//! drift from what a host can actually run.

mod discovery;
pub use discovery::{discovery, discovery_text};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Machine-readable capability manifest reported by
/// `jubarte capabilities --json` and `capabilities::capabilities_json`.
/// `schema_version` is `1`; the schema is intentionally narrow so an agent
/// can detect an old build without parsing every field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Manifest schema version; `1` for this build.
    pub schema_version: u32,
    /// Engine crate version, as `CARGO_PKG_VERSION`.
    pub engine_version: String,
    /// The surface reporting (`cli`, `python`, `rust`, `wasm`, …); the
    /// value a host passes to [`capabilities`].
    pub runtime: String,
    /// Operation name to availability in this build.
    pub operations: Operations,
    /// Edit plan schema versions `runtime` dispatches: `1`, plus `2`
    /// where the runtime routes plans through [`crate::edit::prepare_v2`]
    /// (the CLI and a Rust embedder; the Python and WASM bindings have
    /// no PlanV2 entry point yet).
    pub edit_plan_versions: Vec<u32>,
    /// Operation kinds an edit plan may contain
    /// ([`crate::edit::OperationKind`] variants, in a stable order).
    pub edit_operations: Vec<String>,
    /// Scope limits an agent must plan around (story kinds, opaque ranges,
    /// legacy `.doc` support, and the package budgets).
    pub limits: Limits,
    /// Audit codes `validate --rules` accepts, in
    /// [`crate::audit::RULES`] order.
    #[serde(default)]
    pub audit_rules: Vec<String>,
    /// Rule sets `validate --rules` accepts, with `word` (the default)
    /// first (`crate::validate::RULE_SETS`).
    #[serde(default)]
    pub validate_rules: Vec<String>,
    /// The file extensions each task's `-o` writes, indexed by task name
    /// ([`OUTPUT_FORMATS`]; `diff` writes text under `.md`, `.markdown`,
    /// `.txt`, `.patch`, `.diff`).
    #[serde(default)]
    pub formats: BTreeMap<String, Vec<String>>,
    /// The id spellings each kind of target takes, as sample tokens plus a
    /// shell-safety note ([`crate::ids`]).
    #[serde(default)]
    pub ids: Ids,
    /// Native CLI options offered while choosing revision settlement;
    /// other surfaces omit the entry (#1279).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub settlement_options: BTreeMap<String, String>,
}

/// The id spellings the CLI, plans and selectors take, by kind
/// ([`crate::ids`]).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ids {
    /// Sample change-id spellings (`r12`, `r1+2`, `header1.r1`, …)
    /// used by `accept` / `reject --id` and `edit -p … --accept` /
    /// `--reject`.
    pub change: Vec<String>,
    /// Sample comment-id spellings (`c27`) used by `edit` / `add
    /// -p` and an edit plan's `comment_id`.
    pub comment: Vec<String>,
    /// The text a comment covers (`edit -p`, a plan's `paragraph`).
    #[serde(default)]
    pub comment_text: Vec<String>,
    /// Paragraphs (`edit`/`add -p`, a plan's anchors); `read -p` takes only
    /// `pN`, `pN-pM` and `tN`.
    pub paragraph: Vec<String>,
    /// Tables: `read -p tN` selects one; `edit`/`add -p` address a cell as
    /// `tN.rowR.cellC`.
    pub table: Vec<String>,
    /// Content controls (a plan's `fill_control`): `ccN` for a block
    /// control, `pN.ccK` for one in a paragraph.
    #[serde(default)]
    pub control: Vec<String>,
    /// How ids print.
    pub note: String,
}

/// Per-operation availability in this build. Every operation is compiled
/// unconditionally; features do not gate an operation, so each field is
/// `true` in the default build. A wrapper that compiles an operation out
/// (for example jubarte-wasm drops `pdf` and `png`) overrides the
/// matching field in its own manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operations {
    /// Two documents into a Word tracked-changes document
    /// ([`crate::document_comparer::compare_documents`]).
    pub compare: bool,
    /// Accept every revision
    /// ([`crate::document_comparer::accept_revisions`]).
    pub accept_revisions: bool,
    /// Reject every revision
    /// ([`crate::document_comparer::reject_revisions`]).
    pub reject_revisions: bool,
    /// List tracked changes by id and accept or reject a selection of them
    /// ([`crate::changes`]); also the edit-plan `resolve_revisions` entry.
    #[serde(default)]
    pub selective_revisions: bool,
    /// List revision records.
    pub revision_records: bool,
    /// DOCX to PDF ([`crate::convert::docx_to_pdf`]).
    pub pdf: bool,
    /// DOCX pages to PNG ([`crate::convert::docx_to_png`]).
    pub png: bool,
    /// Body paragraph inspection ([`crate::inspect::paragraphs`],
    /// [`crate::inspect::summary`]).
    pub inspect_body: bool,
    /// Markdown projection with paragraph ids ([`crate::inspect::markdown`]).
    pub markdown: bool,
    /// Edit plans (clean copy, redline, report; [`crate::edit::apply_plan`]).
    pub edit: bool,
    /// Comments authored by an edit plan (`comment`, `reply_comment`,
    /// `resolve_comment`, `edit_comment`, `delete_comment`).
    pub comments: bool,
    /// Markdown → DOCX, CriticMarkup as tracked changes and comments
    /// ([`crate::markdown::markdown_to_docx`]).
    #[serde(default)]
    pub markdown_to_docx: bool,
    /// Two Markdown documents as CriticMarkup, or Word / Markdown as a
    /// Word redline ([`crate::markdown::diff_markdown`],
    /// [`crate::markdown::redline`]).
    #[serde(default)]
    pub markdown_diff: bool,
    /// The changes between two documents, Word or Markdown, as a patch
    /// over the changed paragraphs at their ids
    /// ([`crate::markdown::diff_markdown`] at the patch level; `jubarte
    /// diff`).
    #[serde(default)]
    pub patch: bool,
    /// Which pages of two documents differ, pixel for pixel, with overlays
    /// of the changed region ([`crate::convert::diff_render`]; `jubarte
    /// visual-diff`).
    #[serde(default)]
    pub diff_render: bool,
    /// Rasterize selected pages after one layout pass of the whole
    /// document (`convert --pages`).
    #[serde(default)]
    pub page_ranges: bool,
    /// Comment threads: `jubarte comments` lists them, and edit plans
    /// reply, resolve, edit and delete comments.
    #[serde(default)]
    pub comment_threads: bool,
    /// Edit plans accept `existing_revisions: "keep"`: another party's
    /// tracked changes stay, and the plan's edits become new revisions
    /// beside them.
    #[serde(default)]
    pub edit_keeps_revisions: bool,
    /// One document after another, carrying images, links, styles, lists
    /// and notes ([`crate::append::append_documents`]; `jubarte append`).
    #[serde(default)]
    pub append: bool,
    /// `read --json` lists the body's content controls and edit plans fill
    /// them (`fill_control`).
    #[serde(default)]
    pub content_controls: bool,
    /// Word-validity findings and the tracked-edit audit
    /// ([`crate::validate::validate`]; `jubarte validate` with
    /// `--original` / `--author`).
    #[serde(default)]
    pub validate: bool,
    /// Repairable findings fixed ([`crate::validate::repair`]; `jubarte
    /// validate --repair`).
    #[serde(default)]
    pub repair: bool,
    /// Refresh `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and `TOC` results from
    /// jubarte's layout ([`crate::fields::update_fields`]; `jubarte fields
    /// update`, the edit-plan `update_fields`).
    #[serde(default)]
    pub fields: bool,
    /// Remove authors, rsids, document properties and comments
    /// ([`crate::scrub::scrub`]; `jubarte scrub`); edit plans take `redact`.
    #[serde(default)]
    pub scrub: bool,
    /// Accessibility, style and structure findings by paragraph id
    /// ([`crate::audit::audit_report`]; `jubarte validate --rules
    /// a11y,style,structure`).
    #[serde(default)]
    pub audit: bool,
    /// Edit plans run with `--skip-failed`
    /// ([`crate::edit::PrepareOptions::skip_failed`]): the operations that
    /// pass are applied, and the failed ones are written as a plan of
    /// their own.
    #[serde(default)]
    pub edit_skip_failed: bool,
}

impl Operations {
    /// Each operation and whether this build has it, in declaration order
    /// (the same order as the field list). Used by [`capabilities_text`].
    pub fn entries(&self) -> Vec<(&'static str, bool)> {
        let Self {
            compare,
            accept_revisions,
            reject_revisions,
            selective_revisions,
            revision_records,
            pdf,
            png,
            inspect_body,
            markdown,
            edit,
            comments,
            markdown_to_docx,
            markdown_diff,
            patch,
            diff_render,
            page_ranges,
            comment_threads,
            edit_keeps_revisions,
            append,
            content_controls,
            validate,
            repair,
            fields,
            scrub,
            audit,
            edit_skip_failed,
        } = self;
        vec![
            ("compare", *compare),
            ("accept_revisions", *accept_revisions),
            ("reject_revisions", *reject_revisions),
            ("selective_revisions", *selective_revisions),
            ("revision_records", *revision_records),
            ("pdf", *pdf),
            ("png", *png),
            ("inspect_body", *inspect_body),
            ("markdown", *markdown),
            ("edit", *edit),
            ("comments", *comments),
            ("markdown_to_docx", *markdown_to_docx),
            ("markdown_diff", *markdown_diff),
            ("patch", *patch),
            ("diff_render", *diff_render),
            ("page_ranges", *page_ranges),
            ("comment_threads", *comment_threads),
            ("edit_keeps_revisions", *edit_keeps_revisions),
            ("append", *append),
            ("content_controls", *content_controls),
            ("validate", *validate),
            ("repair", *repair),
            ("fields", *fields),
            ("scrub", *scrub),
            ("audit", *audit),
            ("edit_skip_failed", *edit_skip_failed),
        ]
    }
}

/// Documented scope limits an agent must plan around.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    /// Story kinds `read` and `edit` address: `body`, then the `kind` of
    /// each entry of the snapshot's `stories` (`header`, `footer`, `footnotes`,
    /// `endnotes`). A selector names a story by its id, the part's file stem
    /// (`header1`, `{"story": "footnotes", "index": 0}`), which
    /// `read --json` lists for the document at hand. Comments are counted in
    /// `summary` but are not stories; text boxes are not stories either, and
    /// their owner paragraph carries the `text_box_omitted` limitation.
    pub stories: Vec<String>,
    /// `false`: an inserted run is plain text, so a `\t` or `\n` writes a
    /// tab or line break; other control characters and the symbol
    /// placeholder U+FFFC are refused.
    pub plain_text_runs: bool,
    /// `true`: edit ranges crossing fields, hyperlinks, content controls,
    /// revisions and symbols are refused. A range over a tab, a line
    /// break or a non-breaking hyphen removes it; a page or column break
    /// stays. A comment edits nothing and may cover any of them; its
    /// anchor text is the paragraph as it reads, so it cannot start or
    /// end in deleted text.
    pub refuses_opaque_ranges: bool,
    /// `false`: legacy `.doc` input is not read. An OLE compound file (a
    /// Word 97-2003 `.doc`, or an encrypted document of any Word version)
    /// is refused with `LEGACY_DOC` on every entry point, and RTF with
    /// `UNSUPPORTED_PACKAGE`.
    pub reads_legacy_doc: bool,
    /// Package budgets [`crate::admission::admit`] enforces (`read` and
    /// `edit` use these defaults; the redline comparer uses the roomier
    /// `compare`). Larger input is refused with `INPUT_LIMIT`.
    #[serde(default)]
    pub input: InputBudget,
}

/// Package budgets as the manifest reports them, in the same units
/// [`crate::admission::InputLimits`] uses.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBudget {
    /// Maximum compressed (.docx) size in bytes.
    pub max_compressed_bytes: u64,
    /// Maximum number of ZIP entries the package may contain.
    pub max_entries: usize,
    /// Maximum inflated size of any one part, in bytes.
    pub max_part_bytes: u64,
    /// Maximum total inflated size of all parts, in bytes.
    pub max_uncompressed_bytes: u64,
    /// Maximum element nesting depth in any XML part.
    pub max_xml_depth: usize,
    /// Maximum XML element start tags across all the package's parts
    /// (#2014). `None` in a manifest written before 0.17.0, whose engine
    /// had no element limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_xml_elements: Option<u64>,
    /// Maximum characters of text plus elements in one compared side
    /// (#2024). `None` in a manifest written before 0.17.0, whose comparer
    /// had no such limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_story_units: Option<u64>,
}

impl From<crate::admission::InputLimits> for InputBudget {
    /// Converts an [`crate::admission::InputLimits`] budget into the
    /// manifest-facing [`InputBudget`] (same fields, different struct).
    fn from(l: crate::admission::InputLimits) -> Self {
        Self {
            max_compressed_bytes: l.max_compressed_bytes,
            max_entries: l.max_entries,
            max_part_bytes: l.max_part_bytes,
            max_uncompressed_bytes: l.max_uncompressed_bytes,
            max_xml_depth: l.max_xml_depth,
            max_xml_elements: Some(l.max_xml_elements),
            max_story_units: Some(l.max_story_units),
        }
    }
}

/// The extensions each task's `-o` writes, by task name: Word, the agent
/// view (`md`), a rendered PDF, PNG pages and the JSON snapshot. `diff -o`
/// writes text under any of `DIFF_EXTENSIONS`. The CLI's `-o` rule reads this
/// table, so its refusals and its help agree.
pub const OUTPUT_FORMATS: &[(&str, &[&str])] = &[
    ("compare", &["docx", "md", "pdf", "png", "json"]),
    ("convert", &["docx", "md", "pdf", "png", "json"]),
    ("read", &["md", "json"]),
    ("diff", &["md"]),
    ("accept", &["docx", "md", "pdf", "png", "json"]),
    ("reject", &["docx", "md", "pdf", "png", "json"]),
    ("append", &["docx", "md", "pdf", "png", "json"]),
    ("scrub", &["docx", "md", "pdf", "png", "json"]),
    ("edit", &["docx"]),
    ("add", &["docx"]),
];

/// File extensions `diff -o` writes as text output: Markdown, plain text,
/// and the two patch file extensions (`patch` and `diff`). Replaces the
/// `md` row of [`OUTPUT_FORMATS`] for the `diff` task.
pub(crate) const DIFF_EXTENSIONS: &[&str] = &["md", "markdown", "txt", "patch", "diff"];

/// One-line note describing how ids print, used as [`Ids::note`] in the
/// manifest. Records the body vs. story prefix and the shell-safety rule.
pub(crate) const IDS_NOTE: &str = "body ids print without a story (r12); every other story prints \
     {story}.r{id} (header1.r1, footnotes.r3); rows and cells sit under their table \
     (t0.row1.cell2); every id is safe unquoted in bash, csh and zsh";

/// Whether `runtime` dispatches PlanV2 end to end. The CLI runs `edit
/// apply --plan` and inline edits through [`crate::edit::prepare_v2`];
/// `edit template` generates a PlanV2 that `edit apply --plan` then runs.
/// A Rust embedder calls `prepare_v2` directly. The Python and WASM bindings
/// expose no PlanV2 entry point yet, so their hosts stay on plan version 1.
///
/// Runtime names form a closed set: `cli`, `rust`, `python`, `wasm`, used
/// by `src/bin/jubarte.rs`, the library tests, `jubarte-python/src/lib.rs`
/// and `jubarte-wasm/src/lib.rs`, respectively. Any other name gets plan
/// version 1 only.
fn dispatches_v2(runtime: &str) -> bool {
    matches!(runtime, "cli" | "rust")
}

/// Whether `runtime` runs an edit plan with `--skip-failed`: the CLI and a
/// Rust embedder ([`crate::edit::PrepareOptions::skip_failed`]). The Python
/// and WASM bindings take no such option.
fn skips_failed(runtime: &str) -> bool {
    matches!(runtime, "cli" | "rust")
}

/// The plan versions `runtime` dispatches, ascending: the manifest's
/// `edit_plan_versions` and the focused matrix agree on it.
fn dispatched_plan_versions(runtime: &str) -> Vec<u32> {
    if dispatches_v2(runtime) {
        vec![crate::inspect::SCHEMA_VERSION, 2]
    } else {
        vec![crate::inspect::SCHEMA_VERSION]
    }
}

/// Builds a [`Capabilities`] manifest for `runtime` (typically `cli`,
/// `python`, `rust`, `wasm`). Every operation in the default build is
/// `true`, but what a runtime has no entry point for (`edit_skip_failed`,
/// the plan versions); a wrapper overrides fields in its own manifest when
/// it compiles an operation out.
#[must_use]
pub fn capabilities(runtime: &str) -> Capabilities {
    Capabilities {
        schema_version: 1,
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        runtime: runtime.to_string(),
        operations: Operations {
            compare: true,
            accept_revisions: true,
            reject_revisions: true,
            selective_revisions: true,
            revision_records: true,
            pdf: true,
            png: true,
            inspect_body: true,
            markdown: true,
            edit: true,
            comments: true,
            markdown_to_docx: true,
            markdown_diff: true,
            patch: true,
            diff_render: true,
            page_ranges: true,
            comment_threads: true,
            edit_keeps_revisions: true,
            append: true,
            content_controls: true,
            validate: true,
            repair: true,
            fields: true,
            scrub: true,
            audit: true,
            edit_skip_failed: skips_failed(runtime),
        },
        edit_plan_versions: dispatched_plan_versions(runtime),
        edit_operations: crate::edit::dispatched_operation_kinds(),
        limits: Limits {
            stories: std::iter::once(crate::inspect::BODY_STORY)
                .chain(crate::inspect::STORY_KINDS.iter().copied())
                .map(str::to_string)
                .collect(),
            plain_text_runs: false,
            refuses_opaque_ranges: true,
            reads_legacy_doc: false,
            input: crate::admission::InputLimits::default().into(),
        },
        audit_rules: crate::audit::RULES
            .iter()
            .map(|rule| rule.0.to_string())
            .collect(),
        validate_rules: crate::validate::RULE_SETS
            .iter()
            .map(|set| set.to_string())
            .collect(),
        formats: OUTPUT_FORMATS
            .iter()
            .map(|(task, names)| {
                let names = if *task == "diff" {
                    DIFF_EXTENSIONS
                } else {
                    names
                };
                let extensions = names.iter().map(|name| format!(".{name}")).collect();
                ((*task).to_string(), extensions)
            })
            .collect(),
        ids: Ids {
            change: strings(crate::ids::CHANGE_SPELLINGS),
            comment: strings(crate::ids::COMMENT_SPELLINGS),
            comment_text: strings(crate::ids::COMMENT_TEXT_SPELLINGS),
            paragraph: strings(crate::ids::PARAGRAPH_SPELLINGS),
            table: strings(crate::ids::TABLE_SPELLINGS),
            control: strings(crate::ids::CONTROL_SPELLINGS),
            note: IDS_NOTE.to_string(),
        },
        settlement_options: if runtime == "cli" {
            [("--keep-nested-comment".into(), "accept / reject; convert with --track-changes accept|reject: keep wholly removed comment threads on empty anchors (default: remove)".into())].into()
        } else {
            BTreeMap::new()
        },
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// Renders a [`Capabilities`] as text, one line per field: an `engine:`
/// header, `NAME: yes|no` per operation, plan versions and operations,
/// stories, limits, validate rules, audit codes, `-o` formats and id
/// spellings, ending with `run with --json for the manifest`. The format
/// `jubarte capabilities` prints without `--json`.
#[must_use]
pub fn capabilities_text(c: &Capabilities) -> String {
    let yes = |b: bool| if b { "yes" } else { "no" };
    let mut out = format!("engine: jubarte {} ({})\n", c.engine_version, c.runtime);
    for (name, has) in c.operations.entries() {
        out.push_str(&format!("{name}: {}\n", yes(has)));
    }
    let versions: Vec<String> = c.edit_plan_versions.iter().map(u32::to_string).collect();
    out.push_str(&format!("edit plan versions: {}\n", versions.join(", ")));
    out.push_str(&format!(
        "edit operations: {}\n",
        c.edit_operations.join(", ")
    ));
    out.push_str(&format!("stories: {}\n", c.limits.stories.join(", ")));
    let input = &c.limits.input;
    out.push_str(&format!(
        "limits: plain text runs {}, refuses opaque ranges {}, reads legacy .doc {}, \
         {} bytes compressed, {} entries, {} bytes per part, {} bytes uncompressed, \
         XML depth {}, {}, {}\n",
        yes(c.limits.plain_text_runs),
        yes(c.limits.refuses_opaque_ranges),
        yes(c.limits.reads_legacy_doc),
        input.max_compressed_bytes,
        input.max_entries,
        input.max_part_bytes,
        input.max_uncompressed_bytes,
        input.max_xml_depth,
        input.max_xml_elements.map_or_else(
            || "no XML element limit".to_string(),
            |n| format!("{n} XML elements")
        ),
        input.max_story_units.map_or_else(
            || "no story unit limit".to_string(),
            |n| format!("{n} story units per compared side")
        )
    ));
    let rules: Vec<String> = c
        .validate_rules
        .iter()
        .map(|rule| {
            if rule == "word" {
                format!("{rule} (default)")
            } else {
                rule.clone()
            }
        })
        .collect();
    out.push_str(&format!("validate rules: {}\n", rules.join(", ")));
    out.push_str(&format!("audit codes: {}\n", c.audit_rules.join(", ")));
    let formats: Vec<String> = OUTPUT_FORMATS
        .iter()
        .filter_map(|(task, _)| {
            let extensions = c.formats.get(*task)?;
            Some(format!("{task} {}", extensions.join(" ")))
        })
        .collect();
    out.push_str(&format!("formats: {}\n", formats.join("; ")));
    out.push_str(&format!(
        "ids: revisions {}; comments {}; comment text {}; paragraphs {}; tables {}; controls {}\n",
        c.ids.change.join(", "),
        c.ids.comment.join(", "),
        c.ids.comment_text.join(", "),
        c.ids.paragraph.join(", "),
        c.ids.table.join(", "),
        c.ids.control.join(", ")
    ));
    out.push_str(&format!("ids note: {}\n", c.ids.note));
    for (flag, meaning) in &c.settlement_options {
        out.push_str(&format!("{flag}: {meaning}\n"));
    }
    out.push_str("run with --json for the manifest\n");
    out
}

/// The manifest serialized as pretty-printed JSON; the body
/// `jubarte capabilities --json` prints and the form a host stores.
#[must_use]
pub fn capabilities_json(runtime: &str) -> String {
    serde_json::to_string_pretty(&capabilities(runtime)).expect("manifest serializes")
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    /// A manifest from before the element budget (#2014) reads as having
    /// none, not as a budget of 0; today's carries the default's.
    #[test]
    fn an_older_manifest_has_no_element_limit() {
        let older: InputBudget = serde_json::from_str(
            r#"{"max_compressed_bytes":1,"max_entries":2,"max_part_bytes":3,"max_uncompressed_bytes":4,"max_xml_depth":5}"#,
        )
        .unwrap();
        assert_eq!(older.max_xml_elements, None);
        assert_eq!(older.max_story_units, None);
        assert!(
            !serde_json::to_string(&older)
                .unwrap()
                .contains("max_xml_elements")
        );
        let today = InputBudget::from(crate::admission::InputLimits::default());
        assert_eq!(today.max_xml_elements, Some(2_000_000));
        assert_eq!(today.max_story_units, Some(2_000_000));
    }

    #[test]
    fn manifest_reports_the_crate_version_and_every_edit_kind() {
        let c = capabilities("rust");
        assert_eq!(c.engine_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(c.runtime, "rust");
        assert_eq!(c.edit_plan_versions, [1, 2]);
        assert_eq!(c.edit_operations.len(), 26);
        assert!(c.operations.fields);
        for op in ["insert_toc", "redact", "settings", "strike", "restore"] {
            assert!(c.edit_operations.iter().any(|o| o == op), "{op}");
        }
        assert!(c.edit_operations.iter().any(|kind| kind == "rewrite"));
        assert!(
            c.edit_operations
                .iter()
                .any(|kind| kind == "delete_comment")
        );
        assert!(c.operations.comment_threads);
        assert!(c.edit_operations.iter().any(|kind| kind == "fill_control"));
        assert!(c.operations.content_controls);
        assert!(c.operations.markdown_to_docx && c.operations.markdown_diff);
        let json: serde_json::Value = serde_json::from_str(&capabilities_json("cli")).unwrap();
        assert_eq!(json["runtime"], "cli");
        assert_eq!(json["operations"]["png"], true);
        assert_eq!(json["operations"]["comment_threads"], true);
        assert_eq!(json["operations"]["selective_revisions"], true);
        assert_eq!(json["operations"]["diff_render"], true);
        assert_eq!(json["operations"]["page_ranges"], true);
        assert_eq!(json["operations"]["edit_keeps_revisions"], true);
        assert_eq!(json["operations"]["scrub"], true);
        assert_eq!(json["operations"]["append"], true);
        assert_eq!(json["limits"]["reads_legacy_doc"], false);
        assert_eq!(json["limits"]["input"]["max_entries"], 10_000);
        assert_eq!(json["limits"]["input"]["max_xml_depth"], 256);
        let back: Capabilities = serde_json::from_value(json).unwrap();
        assert_eq!(
            back.limits.stories,
            ["body", "header", "footer", "footnotes", "endnotes"]
        );
        assert!(back.operations.validate && back.operations.repair);
        assert_eq!(back.limits.input.max_part_bytes, 64 * 1024 * 1024);
    }
}
