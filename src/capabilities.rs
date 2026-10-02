// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What this build can do, derived from the compiled feature set rather than
//! from documentation. Agents read it before choosing an operation:
//! `jubarte capabilities --json`, `jubarte_redlines.capabilities()`.

use serde::{Deserialize, Serialize};

/// Machine-readable capability manifest (`schema_version` 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Engine crate version.
    pub engine_version: String,
    /// `cli`, `python`, `rust`, ...: the surface reporting.
    pub runtime: String,
    /// Operation name to availability in this build.
    pub operations: Operations,
    /// Edit plan schema versions this build accepts.
    pub edit_plan_versions: Vec<u32>,
    /// Operation kinds an edit plan may contain.
    pub edit_operations: Vec<String>,
    /// Scope limits an agent must plan around.
    pub limits: Limits,
}

/// Availability per operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operations {
    /// Two documents into a Word tracked-changes document.
    pub compare: bool,
    /// Accept every revision.
    pub accept_revisions: bool,
    /// Reject every revision.
    pub reject_revisions: bool,
    /// List tracked changes by id and accept or reject a selection of them
    /// (also in edit plans, as `resolve_revisions`).
    #[serde(default)]
    pub selective_revisions: bool,
    /// List revision records.
    pub revision_records: bool,
    /// DOCX to PDF.
    pub pdf: bool,
    /// DOCX pages to PNG.
    pub png: bool,
    /// Body paragraph inspection.
    pub inspect_body: bool,
    /// Markdown projection with paragraph ids.
    pub markdown: bool,
    /// Edit plans (clean copy, redline, report).
    pub edit: bool,
    /// Comments authored by an edit plan.
    pub comments: bool,
    /// Markdown to DOCX, CriticMarkup as tracked changes and comments.
    #[serde(default)]
    pub markdown_to_docx: bool,
    /// Two Markdown documents as CriticMarkup, and Word or Markdown
    /// documents as a Word redline.
    #[serde(default)]
    pub markdown_diff: bool,
    /// The changes between two documents, Word or Markdown, as a patch of
    /// the changed paragraphs at their ids (`jubarte diff`); an applied edit
    /// plan carries its redline's.
    #[serde(default)]
    pub patch: bool,
    /// Which pages of two documents differ, pixel for pixel, with overlays
    /// of the changed region (`jubarte diff-render`).
    #[serde(default)]
    pub diff_render: bool,
    /// Rasterize selected pages after one layout pass of the whole
    /// document (`convert --pages`).
    #[serde(default)]
    pub page_ranges: bool,
    /// Comment threads: `jubarte comments` lists them, and edit plans reply
    /// to, resolve, edit and delete comments.
    #[serde(default)]
    pub comment_threads: bool,
    /// Edit plans accept `existing_revisions: "keep"`: another party's
    /// tracked changes stay, and the plan's edits become new revisions
    /// beside them.
    #[serde(default)]
    pub edit_keeps_revisions: bool,
    /// One document after another, carrying images, links, styles, lists
    /// and notes (`jubarte append`).
    #[serde(default)]
    pub append: bool,
    /// `inspect` lists the body's content controls and edit plans fill
    /// them (`fill_control`).
    #[serde(default)]
    pub content_controls: bool,
    /// Word-validity findings beyond the schema (`jubarte validate`),
    /// with the tracked-edit audit (`--original --author`).
    #[serde(default)]
    pub validate: bool,
    /// The repairable findings fixed (`jubarte validate --repair`).
    #[serde(default)]
    pub repair: bool,
}

/// Documented scope limits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    /// Stories `inspect` and `edit` address; text boxes are reported in
    /// `summary` but not editable.
    pub stories: Vec<String>,
    /// Inserted run text is plain: no tabs or line breaks inside runs.
    pub plain_text_runs: bool,
    /// Edits refuse ranges crossing fields, hyperlinks, content controls,
    /// revisions, tabs, breaks and symbols.
    pub refuses_opaque_ranges: bool,
    /// Legacy `.doc` input is not read.
    pub reads_legacy_doc: bool,
    /// Package budgets `inspect` and `edit` admit (larger input is refused
    /// with `INPUT_LIMIT`).
    #[serde(default)]
    pub input: InputBudget,
}

/// [`crate::admission::InputLimits`] as the manifest reports them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBudget {
    /// Size of the .docx file.
    pub max_compressed_bytes: u64,
    /// ZIP entries.
    pub max_entries: usize,
    /// Inflated size of any one part.
    pub max_part_bytes: u64,
    /// Inflated size of all parts together.
    pub max_uncompressed_bytes: u64,
    /// Element nesting in any XML part.
    pub max_xml_depth: usize,
}

impl From<crate::admission::InputLimits> for InputBudget {
    fn from(l: crate::admission::InputLimits) -> Self {
        Self {
            max_compressed_bytes: l.max_compressed_bytes,
            max_entries: l.max_entries,
            max_part_bytes: l.max_part_bytes,
            max_uncompressed_bytes: l.max_uncompressed_bytes,
            max_xml_depth: l.max_xml_depth,
        }
    }
}

/// The manifest for `runtime`.
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
        },
        edit_plan_versions: vec![crate::inspect::SCHEMA_VERSION],
        edit_operations: [
            "replace",
            "insert",
            "delete",
            "comment",
            "insert_paragraph",
            "delete_paragraph",
            "format_paragraph",
            "merge_paragraphs",
            "rewrite",
            "reply_comment",
            "resolve_comment",
            "edit_comment",
            "delete_comment",
            "insert_table",
            "list",
            "watermark",
            "fill_control",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect(),
        limits: Limits {
            stories: ["body", "header", "footer", "footnotes", "endnotes"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            plain_text_runs: true,
            refuses_opaque_ranges: true,
            reads_legacy_doc: false,
            input: crate::admission::InputLimits::default().into(),
        },
    }
}

/// [`capabilities`] as JSON.
#[must_use]
pub fn capabilities_json(runtime: &str) -> String {
    serde_json::to_string_pretty(&capabilities(runtime)).expect("manifest serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_reports_the_crate_version_and_every_edit_kind() {
        let c = capabilities("rust");
        assert_eq!(c.engine_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(c.runtime, "rust");
        assert_eq!(c.edit_plan_versions, [1]);
        assert_eq!(c.edit_operations.len(), 17);
        assert!(c.edit_operations.iter().any(|kind| kind == "rewrite"));
        assert!(
            c.edit_operations
                .iter()
                .any(|kind| kind == "delete_comment")
        );
        assert!(c.operations.comment_threads);
        assert_eq!(
            c.edit_operations.last().map(String::as_str),
            Some("fill_control")
        );
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
