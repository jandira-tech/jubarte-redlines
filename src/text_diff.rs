// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Git/GitHub unified patches of complete document text snapshots.
//!
//! DOCX snapshots use the same paragraph, table, and revision-mark view as
//! `debug --check text`, without clipping or a hunk limit. These are text
//! patches for review; they do not patch the binary DOCX package.

use crate::markdown::Source;

/// Names in the patch headers and the number of unchanged context lines.
#[derive(Clone, Debug)]
pub struct UnifiedOptions {
    /// Original document label (Git's `a/` prefix is added automatically).
    pub old_name: String,
    /// Modified document label (Git's `b/` prefix is added automatically).
    pub new_name: String,
    /// Number of unchanged lines around each change; defaults to three.
    pub context: usize,
}

impl Default for UnifiedOptions {
    fn default() -> Self {
        Self {
            old_name: "old.docx".into(),
            new_name: "new.docx".into(),
            context: 3,
        }
    }
}

/// Complete textual snapshot, preserving existing tracked marks.
pub fn document_text(source: Source<'_>) -> Result<String, String> {
    match source {
        Source::Markdown(text) => Ok(text.to_string()),
        Source::Docx(bytes) => crate::debug::document_text(bytes),
    }
}

/// A standard unified patch; identical text produces an empty string.
pub fn diff_text(old: &str, new: &str, options: &UnifiedOptions) -> String {
    if old == new {
        return String::new();
    }
    let old_name = git_path("a", &options.old_name);
    let new_name = git_path("b", &options.new_name);
    let diff = similar::TextDiff::from_lines(old, new);
    let hunks = diff
        .unified_diff()
        .context_radius(
            options
                .context
                .min(old.lines().count().max(new.lines().count())),
        )
        .header(&old_name, &new_name)
        .to_string();
    format!("diff --git {old_name} {new_name}\n{hunks}")
}

// Git C-style path quoting keeps control characters out of patch headers.
fn git_path(side: &str, name: &str) -> String {
    let path = format!("{side}/{name}");
    if !path
        .chars()
        .any(|c| c.is_control() || matches!(c, '"' | '\\' | ' '))
    {
        return path;
    }
    let mut out = String::from("\"");
    for byte in path.bytes() {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\t' => out.push_str("\\t"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            0..=31 | 127..=255 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\{byte:03o}");
            }
            _ => out.push(char::from(byte)),
        }
    }
    out.push('"');
    out
}

/// Compare DOCX bytes or Markdown text without constructing a redline.
pub fn diff_documents(
    old: Source<'_>,
    new: Source<'_>,
    options: &UnifiedOptions,
) -> Result<String, String> {
    Ok(diff_text(
        &document_text(old)?,
        &document_text(new)?,
        options,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::{DocxOptions, markdown_to_docx};

    fn word(text: &str) -> Vec<u8> {
        markdown_to_docx(text, &DocxOptions::default())
            .unwrap()
            .docx
    }

    #[test]
    fn patch_has_git_headers_ranges_and_context() {
        let out = diff_text(
            "Intro\nThirty days\nSigned\n",
            "Intro\nSixty days\nSigned\n",
            &UnifiedOptions::default(),
        );
        assert_eq!(
            out,
            "diff --git a/old.docx b/new.docx\n--- a/old.docx\n+++ b/new.docx\n@@ -1,3 +1,3 @@\n Intro\n-Thirty days\n+Sixty days\n Signed\n"
        );
    }

    #[test]
    fn empty_ranges_and_missing_final_newlines_follow_unified_convention() {
        let options = UnifiedOptions {
            context: 0,
            ..Default::default()
        };
        let inserted = diff_text("", "Hello\n", &options);
        assert!(inserted.contains("@@ -0,0 +1 @@\n+Hello\n"), "{inserted}");
        let deleted = diff_text("Hello\n", "", &options);
        assert!(deleted.contains("@@ -1 +0,0 @@\n-Hello\n"), "{deleted}");
        let unterminated = diff_text("Old", "New", &options);
        assert_eq!(
            unterminated.matches("\\ No newline at end of file").count(),
            2
        );
    }

    #[test]
    fn identical_content_has_no_patch_even_with_different_names() {
        assert_eq!(
            diff_text("same\n", "same\n", &UnifiedOptions::default()),
            ""
        );
        assert_eq!(diff_text("", "", &UnifiedOptions::default()), "");
    }

    #[test]
    fn names_with_control_characters_are_git_quoted() {
        let options = UnifiedOptions {
            old_name: "a\t\".docx".into(),
            new_name: "b\n.docx".into(),
            context: 0,
        };
        let out = diff_text("a\n", "b\n", &options);
        assert!(
            out.starts_with("diff --git \"a/a\\t\\\".docx\" \"b/b\\n.docx\"\n"),
            "{out}"
        );
        assert!(out.contains("--- \"a/a\\t\\\".docx\"\n"));
    }

    #[test]
    fn every_change_and_long_unicode_line_is_preserved() {
        let old = (0..12)
            .map(|n| format!("old {n} {}\n", "é".repeat(300)))
            .collect::<String>();
        let new = old.replace("old ", "new ");
        let out = diff_text(&old, &new, &UnifiedOptions::default());
        assert_eq!(out.lines().filter(|l| l.starts_with("-old ")).count(), 12);
        assert!(out.contains(&"é".repeat(300)));
        assert!(!out.contains('…'));
    }

    #[test]
    fn word_snapshot_preserves_existing_marks_and_compares_them() {
        let old = word("Due in {~~30~>45~~} days.\n");
        let new = word("Due in {~~30~>60~~} days.\n");
        let snapshot = document_text(Source::Docx(&old)).unwrap();
        assert!(snapshot.contains("[-30-]{+45+}"), "{snapshot}");
        let out = diff_documents(
            Source::Docx(&old),
            Source::Docx(&new),
            &UnifiedOptions::default(),
        )
        .unwrap();
        assert!(out.contains("[-30-]{+45+}"), "{out}");
        assert!(out.contains("[-30-]{+60+}"), "{out}");
        assert_eq!(
            diff_documents(
                Source::Docx(&old),
                Source::Docx(&old),
                &UnifiedOptions::default()
            )
            .unwrap(),
            ""
        );
    }

    #[test]
    fn invalid_docx_is_refused() {
        assert!(document_text(Source::Docx(b"not a document")).is_err());
    }

    #[test]
    fn markdown_and_mixed_sources_share_the_same_formatter() {
        let old = word("Hello\n");
        let snapshot = document_text(Source::Docx(&old)).unwrap();
        assert_eq!(
            diff_documents(
                Source::Docx(&old),
                Source::Markdown(&snapshot),
                &UnifiedOptions::default()
            )
            .unwrap(),
            ""
        );
        assert!(
            diff_documents(
                Source::Markdown("Old\n"),
                Source::Markdown("New\n"),
                &UnifiedOptions::default()
            )
            .unwrap()
            .contains("-Old\n+New\n")
        );
    }
}
