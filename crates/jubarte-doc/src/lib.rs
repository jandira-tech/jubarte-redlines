// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-doc
//!
//! The document operations of the jubarte engine above compare: listing
//! and settling changes (`changes`), comment threads (`comments`,
//! `comment_settlement`), Word-validity checks and repair (`validate`),
//! the accessibility and structure audit (`audit`), metadata scrubbing
//! (`scrub`), appending documents (`append`), field updates (`fields`)
//! and the maintainer views (`debug`, `inspect`). The CriticMarkup
//! reader (`critic`) lives here too: it is pure text, and the markdown
//! layer above re-exports it.
//!
//! Layout needs the renderer: with the `render` feature off, the public
//! items that need layout (audit's `word_layout`, the fields update
//! API, the debug render check) do not exist.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A5 shims: the floors' and the compare product's modules under the
// names the moved code uses, so `crate::<module>::…` paths compile
// unchanged.
pub(crate) use jubarte_dom::ooxml;
pub use jubarte_dom::{ids, namespaces, util, xmllinq};
pub use jubarte_opc::{admission, opc, strict_translation};
pub(crate) use jubarte_opc::{
    builtin_styles, data_stores, markup_simplifier, numbering, validate_dom,
};
// The debug render check's table and border views; nothing else reads them.
#[cfg(feature = "render")]
pub(crate) use jubarte_opc::{borders, table_dom};
pub use jubarte_redlines::{comparer, document_comparer, revision_processor};

/// Append one document after another.
pub mod append;
/// Accessibility, style and structure findings.
pub mod audit;
/// Tracked changes one at a time: list, accept or reject a selection.
pub mod changes;
/// Thread-level receipts for revision settlement.
#[doc(hidden)]
pub mod comment_settlement;
/// Comment threads across the parts that hold them.
pub mod comments;
/// Maintainer triage views.
pub mod debug;
/// Refresh a table of contents and other fields (needs `render`).
#[cfg(feature = "render")]
pub mod fields;
/// Read-only paragraph/package views and the Markdown projection.
pub mod inspect;
/// Remove authors and metadata before sending.
pub mod scrub;
/// Check that Word will open a package, and repair it.
pub mod validate;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(crate) mod corpus {
    /// Every `.docx` under the repository's `tests/fixtures`, sorted, with
    /// its path relative to that folder.
    pub(crate) fn fixtures() -> Vec<(String, Vec<u8>)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
        let mut out = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("fixtures") {
                let path = entry.expect("entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "docx") {
                    // `/`-separated on every platform, so names match
                    // literals such as `f4/original.docx` on Windows too.
                    let name = path
                        .strip_prefix(&root)
                        .unwrap()
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join("/");
                    out.push((name, std::fs::read(&path).expect("read")));
                }
            }
        }
        out.sort();
        out
    }

    /// Every eighth fixture, in name order: a spread of real Word, strict,
    /// math and round-trip packages small enough for a unit test.
    pub(crate) fn sample() -> Vec<(String, Vec<u8>)> {
        fixtures().into_iter().step_by(8).collect()
    }

    fn all_checks() -> crate::debug::Options {
        use crate::debug::{Check, Options};
        Options {
            checks: vec![
                Check::Orphans,
                Check::Fields,
                Check::Bookmarks,
                Check::Package,
                Check::Structure,
                Check::Ids,
                Check::Styles,
                Check::Chains,
                Check::Elements,
                Check::Textbox,
                Check::Text,
                Check::Xml,
                Check::Runs,
                Check::Changes,
                Check::Tables,
                Check::StyleDefs,
                Check::Numbering,
            ],
            ..Options::default()
        }
    }

    /// A file compared with itself differs nowhere, in every report.
    #[test]
    fn every_report_finds_a_file_equal_to_itself() {
        let opts = all_checks();
        let files = sample();
        assert!(files.len() > 20, "{}", files.len());
        for (name, bytes) in &files {
            let diff = crate::debug::diff::diff(&[("a", bytes), ("b", bytes)], &Default::default())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(diff, "files: a · b\nno differences\n", "{name}");
            let report = crate::debug::report(bytes, Some(bytes), &opts)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            for line in report.lines() {
                assert!(
                    line.ends_with(" identical") || line.ends_with(": 0 differ"),
                    "{name}: {line}"
                );
            }
            let single =
                crate::debug::report(bytes, None, &opts).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!single.is_empty(), "{name}");
        }
    }

    /// Two different files: the three-way diff names all three, and a
    /// report of the pair runs every check.
    #[test]
    fn neighbouring_fixtures_compare_without_error() {
        let opts = all_checks();
        let files = sample();
        for pair in files.windows(2) {
            let ((a_name, a), (b_name, b)) = (&pair[0], &pair[1]);
            let three =
                crate::debug::diff::diff(&[("a", a), ("b", b), ("c", a)], &Default::default())
                    .unwrap_or_else(|e| panic!("{a_name} {b_name}: {e}"));
            assert!(
                three.starts_with("files: a · b · c\n"),
                "{a_name} {b_name}: {three}"
            );
            crate::debug::report(a, Some(b), &opts)
                .unwrap_or_else(|e| panic!("{a_name} {b_name}: {e}"));
        }
    }

    /// The readers agree on which fixtures are readable: a package the
    /// change list reads, the comment, inspect, audit and validate readers
    /// read too.
    #[test]
    fn the_readers_accept_every_readable_fixture() {
        for (name, bytes) in sample() {
            let changes = crate::changes::list_changes(&bytes);
            let readable = changes.is_ok();
            assert_eq!(
                crate::comments::list_comments(&bytes).is_ok(),
                readable,
                "{name}"
            );
            assert_eq!(crate::inspect::snapshot(&bytes).is_ok(), readable, "{name}");
            #[cfg(feature = "render")]
            assert_eq!(crate::audit::audit(&bytes, &[]).is_ok(), readable, "{name}");
            assert_eq!(
                crate::audit::audit_report_with(&bytes, &[], None).is_ok(),
                readable,
                "{name}"
            );
            assert_eq!(
                crate::validate::validate(&bytes).is_ok(),
                readable,
                "{name}"
            );
            if let Ok(changes) = changes {
                let logical = crate::changes::list_logical_changes_tolerant(&bytes).unwrap();
                assert!(logical.len() <= changes.len(), "{name}");
            }
        }
    }

    /// Every other fixture: its findings each print under their own code,
    /// keep one identity apiece, and a package with repairable findings
    /// repairs to one without them.
    #[test]
    fn validation_findings_print_and_repair_on_every_other_fixture() {
        for (name, bytes) in fixtures().into_iter().step_by(2) {
            let Ok(findings) = crate::validate::validate(&bytes) else {
                assert!(crate::changes::list_changes(&bytes).is_err(), "{name}");
                continue;
            };
            let text = crate::validate::findings_text(&findings);
            for f in &findings {
                assert!(text.contains(&f.code), "{name}: {}", f.code);
            }
            assert_eq!(
                crate::validate::identities(&bytes, &findings).len(),
                findings.len(),
                "{name}"
            );
            if !findings.iter().any(|f| f.repairable) {
                continue;
            }
            let repaired =
                crate::validate::repair(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_ne!(repaired.docx, bytes, "{name}");
            let after =
                crate::validate::validate(&repaired.docx).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!after.iter().any(|f| f.repairable), "{name}: {after:?}");
        }
    }

    /// Every third fixture with a main part: a full scrub leaves no comment
    /// and gives every revision the alias.
    #[test]
    fn a_full_scrub_leaves_no_comment_and_one_author() {
        for (name, bytes) in fixtures().into_iter().step_by(3) {
            let Ok(out) = crate::scrub::scrub(&bytes, &crate::scrub::ScrubOptions::default())
            else {
                continue;
            };
            assert!(
                crate::comments::list_comments(&out).unwrap().is_empty(),
                "{name}"
            );
            for change in crate::changes::list_changes(&out).unwrap() {
                assert!(
                    change.author.as_deref().is_none_or(|a| a == "Author"),
                    "{name}: {:?}",
                    change.author
                );
            }
        }
    }

    /// Every fourth fixture: the audit without a layout pass runs every
    /// rule it can, and a field update keeps the package readable.
    #[test]
    fn audits_and_field_updates_keep_packages_readable() {
        for (name, bytes) in fixtures().into_iter().step_by(4) {
            let readable = crate::changes::list_changes(&bytes).is_ok();
            let report = crate::audit::audit_report_with(&bytes, &[], None);
            assert_eq!(report.is_ok(), readable, "{name}");
            #[cfg(feature = "render")]
            if let Ok(updated) = crate::fields::update_fields(&bytes) {
                assert!(
                    crate::changes::list_changes(&updated.docx).is_ok(),
                    "{name}"
                );
                assert!(updated.page_count >= 1, "{name}");
            }
        }
    }

    const RICH: [&str; 2] = ["agent-view/received.docx", "f1/word-redline.docx"];

    fn rich() -> Vec<(String, Vec<u8>)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
        RICH.iter()
            .map(|name| {
                (
                    (*name).to_string(),
                    std::fs::read(root.join(name)).expect("fixture"),
                )
            })
            .collect()
    }

    /// `bytes` with every XML part under `word/` rewritten by `edit`.
    fn rewrite(bytes: &[u8], edit: impl Fn(&str, String) -> Option<String>) -> Vec<u8> {
        let mut pkg = crate::opc::PartFs::open(bytes).expect("open");
        for part in pkg.parts() {
            if !part.starts_with("word/") || !part.ends_with(".xml") {
                continue;
            }
            let Some(xml) = pkg.part_string(&part) else {
                continue;
            };
            if let Some(new) = edit(&part, xml) {
                pkg.set_part(&part, new.into_bytes());
            }
        }
        pkg.to_zip().expect("zip")
    }

    /// Comments and processing instructions between every two tags.
    fn with_noise(bytes: &[u8]) -> Vec<u8> {
        rewrite(bytes, |_, xml| {
            Some(xml.replace("><", "><!--n--><?jr n?><"))
        })
    }

    #[test]
    fn comments_and_instructions_between_tags_change_no_reading() {
        for (name, bytes) in rich() {
            let noisy = with_noise(&bytes);
            assert_ne!(noisy, bytes);
            let changes =
                |b: &[u8]| crate::changes::changes_text(&crate::changes::list_changes(b).unwrap());
            assert_eq!(changes(&noisy), changes(&bytes), "{name}");
            let logical = |b: &[u8]| {
                crate::changes::changes_text(&crate::changes::list_logical_changes(b).unwrap())
            };
            assert_eq!(logical(&noisy), logical(&bytes), "{name}");
            let comments = |b: &[u8]| {
                crate::comments::comments_text(&crate::comments::list_comments(b).unwrap())
            };
            assert_eq!(comments(&noisy), comments(&bytes), "{name}");
            let text = |b: &[u8]| crate::debug::document_text(b).unwrap();
            assert_eq!(text(&noisy), text(&bytes), "{name}");
            let md = |b: &[u8]| crate::inspect::markdown(b).unwrap();
            assert_eq!(md(&noisy), md(&bytes), "{name}");
        }
    }

    /// The same noise reaches every other reader: validation, the debug
    /// report, the audit, scrub, the field update and append all read the
    /// noisy package as they read the clean one.
    #[test]
    fn noise_between_tags_reaches_every_reader_unchanged() {
        let opts = all_checks();
        for (name, bytes) in rich() {
            let noisy = with_noise(&bytes);
            let codes = |b: &[u8]| {
                let mut c: Vec<String> = crate::validate::validate(b)
                    .unwrap()
                    .into_iter()
                    .map(|f| f.code)
                    .collect();
                c.sort();
                c
            };
            assert_eq!(codes(&noisy), codes(&bytes), "{name}");
            let report = crate::debug::report(&bytes, Some(&noisy), &opts).unwrap();
            for line in report.lines() {
                assert!(
                    line.ends_with(" identical") || line.ends_with(": 0 differ"),
                    "{name}: {line}"
                );
            }
            let audit = |b: &[u8]| {
                crate::audit::audit_report_with(b, &[], None)
                    .unwrap()
                    .findings
                    .len()
            };
            assert_eq!(audit(&noisy), audit(&bytes), "{name}");
            let scrubbed =
                crate::scrub::scrub(&noisy, &crate::scrub::ScrubOptions::default()).unwrap();
            assert!(
                crate::comments::list_comments(&scrubbed)
                    .unwrap()
                    .is_empty(),
                "{name}"
            );
            #[cfg(feature = "render")]
            {
                let fields = |b: &[u8]| crate::fields::update_fields(b).unwrap().fields.len();
                assert_eq!(fields(&noisy), fields(&bytes), "{name}");
            }
            let appended =
                crate::append::append_documents(&noisy, &noisy, &Default::default()).unwrap();
            let before = crate::changes::list_changes(&bytes).unwrap().len();
            assert!(
                crate::changes::list_changes(&appended.docx).unwrap().len() >= before,
                "{name}"
            );
        }
    }

    /// Every `word/` part reduced to a prolog and a comment: the readers that
    /// need a body refuse it alike, the change list finds nothing, and the
    /// debug views still print.
    #[test]
    fn parts_with_no_root_element_are_refused_or_read_as_empty() {
        let opts = all_checks();
        for (name, bytes) in rich() {
            let gone = rewrite(&bytes, |_, _| {
                Some("<?xml version=\"1.0\"?><!--gone-->".to_string())
            });
            assert!(
                crate::changes::list_changes(&gone).unwrap().is_empty(),
                "{name}"
            );
            let refusal = "word/document.xml: missing or incomplete XML root";
            let comments = crate::comments::list_comments(&gone)
                .unwrap_err()
                .to_string();
            assert!(comments.contains(refusal), "{name}: {comments}");
            assert!(crate::inspect::snapshot(&gone).is_err(), "{name}");
            let audit = crate::audit::audit_report_with(&gone, &[], None)
                .unwrap_err()
                .to_string();
            assert!(audit.contains(refusal), "{name}: {audit}");
            assert!(
                !crate::debug::report(&gone, None, &opts).unwrap().is_empty(),
                "{name}"
            );
            let diff =
                crate::debug::diff::diff(&[("a", &bytes), ("b", &gone)], &Default::default())
                    .unwrap();
            assert!(
                diff.starts_with("files: a · b\n") && diff.contains("word/document.xml"),
                "{name}"
            );
        }
    }

    /// Each Word redline beside the original it was made from.
    pub(crate) fn word_redlines() -> Vec<(String, Vec<u8>, Vec<u8>)> {
        let mut out = Vec::new();
        let all = fixtures();
        for (name, bytes) in &all {
            let original = if let Some(stem) = name.strip_suffix("_word_redline.docx") {
                format!("{stem}_a.docx")
            } else if let Some(dir) = name.strip_suffix("word-redline.docx") {
                format!("{dir}original.docx")
            } else {
                continue;
            };
            if let Some((_, before)) = all.iter().find(|(n, _)| *n == original) {
                out.push((name.clone(), before.clone(), bytes.clone()));
            }
        }
        out
    }

    fn codes(findings: Vec<crate::validate::Finding>) -> Vec<String> {
        findings.into_iter().map(|f| f.code).collect()
    }

    /// Word's own redlines, audited as edits of their originals: a document
    /// is no edit of itself, the comparison's author made every change, and
    /// anyone else finds them foreign.
    #[test]
    fn word_redlines_audit_as_their_authors_edits() {
        let pairs = word_redlines();
        assert!(pairs.len() >= 10, "{}", pairs.len());
        for (name, before, after) in pairs {
            let authors: std::collections::BTreeSet<String> = crate::changes::list_changes(&after)
                .unwrap()
                .into_iter()
                .filter_map(|c| c.author)
                .collect();
            assert!(
                crate::validate::audit_tracked(&before, &before, "Nobody")
                    .unwrap()
                    .is_empty(),
                "{name}"
            );
            let other = codes(crate::validate::audit_tracked(&before, &after, "Nobody").unwrap());
            assert!(
                other.iter().any(|c| c == "FOREIGN_AUTHOR"),
                "{name}: {other:?}"
            );
            assert!(
                other
                    .iter()
                    .all(|c| c == "FOREIGN_AUTHOR" || c == "UNTRACKED_EDIT"),
                "{name}: {other:?}"
            );
            if authors.len() == 1 && authors.contains("Comparison") {
                let own = crate::validate::audit_tracked(&before, &after, "Comparison").unwrap();
                assert!(own.is_empty(), "{name}: {own:?}");
                let rules = crate::validate::validate_with(
                    &after,
                    &["edits"],
                    Some((&before, "Comparison")),
                    None,
                )
                .unwrap();
                assert!(rules.is_empty(), "{name}: {rules:?}");
            }
        }
    }

    #[test]
    fn rule_selection_refuses_what_it_cannot_run() {
        let (_, before, after) = word_redlines().remove(0);
        let unknown =
            crate::validate::validate_with(&after, &["nonsense"], None, None).unwrap_err();
        assert!(unknown.to_string().contains("nonsense"), "{unknown}");
        let alone = crate::validate::validate_with(&after, &["edits"], None, None).unwrap_err();
        assert!(alone.to_string().contains("needs --author"), "{alone}");
        let audit =
            crate::validate::validate_with(&after, &["a11y"], Some((&before, "Nobody")), None)
                .unwrap();
        assert!(
            audit.iter().any(|f| f.code == "FOREIGN_AUTHOR"),
            "{audit:?}"
        );
    }

    /// The comment parts of `agent-view/received.docx` damaged one way at a
    /// time: (label, part, from, to, a finding validate reports more of for
    /// it). A comment without `w:author` gets no finding today, though the
    /// schema requires the attribute; that case asserts none.
    const COMMENT_DAMAGE: &[(&str, &str, &str, &str, &str)] = &[
        (
            "parent cycle",
            "word/commentsExtended.xml",
            "w15:paraId=\"11A5D0F2\" w15:done",
            "w15:paraId=\"11A5D0F2\" w15:paraIdParent=\"33767091\" w15:done",
            "COMMENT_PARENT_CYCLE",
        ),
        (
            "own parent",
            "word/commentsExtended.xml",
            "w15:paraId=\"214DA01E\" w15:done",
            "w15:paraId=\"214DA01E\" w15:paraIdParent=\"214DA01E\" w15:done",
            "COMMENT_PARENT_CYCLE",
        ),
        (
            "unknown parent",
            "word/commentsExtended.xml",
            "w15:paraIdParent=\"11A5D0F2\"",
            "w15:paraIdParent=\"DEADBEEF\"",
            "COMMENT_METADATA_MISMATCH",
        ),
        (
            "one durable id",
            "word/commentsIds.xml",
            "4500665B",
            "15907BBC",
            "COMMENT_METADATA_MISMATCH",
        ),
        (
            "bad durable id",
            "word/commentsIds.xml",
            "6C9BB659",
            "XYZ",
            "COMMENT_PARTS_INCONSISTENT",
        ),
        (
            "unknown para",
            "word/commentsIds.xml",
            "w16cid:paraId=\"214DA01E\"",
            "w16cid:paraId=\"0BADF00D\"",
            "COMMENT_METADATA_MISMATCH",
        ),
        (
            "idless start",
            "word/document.xml",
            "<w:commentRangeStart w:id=\"5\"/>",
            "<w:commentRangeStart/>",
            "COMMENT_RANGE_UNPAIRED",
        ),
        (
            "idless reference",
            "word/document.xml",
            "<w:commentReference w:id=\"11\"/>",
            "<w:commentReference/>",
            "COMMENT_UNANCHORED",
        ),
        (
            "unknown reference",
            "word/document.xml",
            "<w:commentReference w:id=\"6\"/>",
            "<w:commentReference w:id=\"99\"/>",
            "COMMENT_ANCHOR_ORPHAN",
        ),
        (
            "no end",
            "word/document.xml",
            "<w:commentRangeEnd w:id=\"11\"/>",
            "",
            "COMMENT_RANGE_UNPAIRED",
        ),
        (
            "no start",
            "word/document.xml",
            "<w:commentRangeStart w:id=\"6\"/>",
            "",
            "COMMENT_RANGE_UNPAIRED",
        ),
        (
            "no author",
            "word/comments.xml",
            "w:author=\"Ann Counsel\"",
            "",
            "",
        ),
        (
            "idless comment",
            "word/comments.xml",
            "<w:comment w:id=\"11\"",
            "<w:comment",
            "COMMENT_RANGE_ORPHAN",
        ),
    ];

    fn damaged(bytes: &[u8], part: &str, from: &str, to: &str) -> Vec<u8> {
        let mut pkg = crate::opc::PartFs::open(bytes).expect("open");
        let xml = pkg.part_string(part).expect(part);
        assert!(xml.contains(from), "{part} lacks {from}");
        pkg.set_part(part, xml.replace(from, to).into_bytes());
        pkg.to_zip().expect("zip")
    }

    /// Each damage to the comment parts is reported by validate, never
    /// invents a comment, and leaves every other reader working.
    #[test]
    fn damaged_comment_parts_are_reported_and_still_read() {
        let opts = all_checks();
        let (_, bytes) = rich().remove(0);
        let ids = |b: &[u8]| -> Vec<u32> {
            crate::comments::list_comments(b)
                .unwrap()
                .iter()
                .map(|c| c.id)
                .collect()
        };
        let original = ids(&bytes);
        assert_eq!(original, [5, 6, 11]);
        let baseline = codes(crate::validate::validate(&bytes).unwrap());
        let carry = crate::append::AppendOptions {
            comments: crate::append::AppendComments::Carry,
            ..Default::default()
        };
        for (label, part, from, to, code) in COMMENT_DAMAGE {
            let b = damaged(&bytes, part, from, to);
            assert!(ids(&b).iter().all(|id| original.contains(id)), "{label}");
            let found = codes(crate::validate::validate(&b).unwrap());
            let count = |list: &[String]| list.iter().filter(|c| c == code).count();
            if !code.is_empty() {
                assert!(count(&found) > count(&baseline), "{label}: {found:?}");
            }
            crate::debug::report(&b, None, &opts).unwrap_or_else(|e| panic!("{label}: {e}"));
            let scrubbed = crate::scrub::scrub(&b, &crate::scrub::ScrubOptions::default()).unwrap();
            assert!(ids(&scrubbed).is_empty(), "{label}");
            let appended = crate::append::append_documents(&b, &b, &carry).unwrap();
            crate::comments::list_comments(&appended.docx)
                .unwrap_or_else(|e| panic!("{label}: {e}"));
            assert!(!crate::inspect::markdown(&b).unwrap().is_empty(), "{label}");
        }
    }

    /// The diff's options narrow or widen what it prints of a real redline
    /// against its original.
    #[test]
    fn diff_options_narrow_and_widen_the_listing() {
        use crate::debug::diff::{DiffOptions, diff};
        let (_, before, after) = word_redlines()
            .into_iter()
            .find(|(n, ..)| n.starts_with("f4/"))
            .unwrap();
        let files = [("a", &before[..]), ("b", &after[..])];
        let plain = diff(&files, &DiffOptions::default()).unwrap();
        let raw = diff(
            &files,
            &DiffOptions {
                raw: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(raw.len() >= plain.len());
        let full = diff(
            &files,
            &DiffOptions {
                full: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(full.len() >= plain.len());
        let one = diff(
            &files,
            &DiffOptions {
                limit: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(one.len() <= plain.len());
        let styles = diff(
            &files,
            &DiffOptions {
                part: Some("styles".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!styles.contains("word/document.xml"), "{styles}");
        let normal = diff(
            &files,
            &DiffOptions {
                style: Some("normal".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(normal.starts_with("files: a · b"));
        let none = diff(
            &files,
            &DiffOptions {
                para_text: Some("\u{1}never".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(none.len() <= plain.len());
        assert!(diff(&files[..1], &DiffOptions::default()).is_err());
    }

    /// The report's own options: one part, a grep, context and whole lines.
    #[test]
    fn report_options_narrow_the_two_file_views() {
        use crate::debug::{Check, Options, report};
        let (_, before, after) = word_redlines()
            .into_iter()
            .find(|(n, ..)| n.starts_with("f4/"))
            .unwrap();
        let checks = vec![
            Check::Text,
            Check::Xml,
            Check::Runs,
            Check::Changes,
            Check::Tables,
            Check::Textbox,
        ];
        let wide = report(
            &before,
            Some(&after),
            &Options {
                checks: checks.clone(),
                context: 2,
                full_lines: true,
                ..Options::default()
            },
        )
        .unwrap();
        let narrow = report(
            &before,
            Some(&after),
            &Options {
                checks: checks.clone(),
                part: Some("document".into()),
                limit: 1,
                ..Options::default()
            },
        )
        .unwrap();
        assert!(!wide.is_empty() && !narrow.is_empty());
        let grep = report(
            &after,
            None,
            &Options {
                checks,
                grep: Some("the".into()),
                ..Options::default()
            },
        )
        .unwrap();
        assert!(!grep.is_empty());
        let nothing = report(
            &after,
            None,
            &Options {
                checks: vec![Check::Xml],
                part: Some("\u{1}none".into()),
                ..Options::default()
            },
        )
        .unwrap();
        assert!(!nothing.contains("<w:"), "{nothing}");
    }

    fn without_story_parts(bytes: &[u8]) -> Vec<u8> {
        let mut pkg = crate::opc::PartFs::open(bytes).expect("open");
        for part in pkg.parts() {
            let stem = part.trim_start_matches("word/");
            if ["header", "footer", "footnotes", "endnotes", "comments"]
                .iter()
                .any(|s| stem.starts_with(s))
            {
                pkg.remove_part(&part);
            }
        }
        pkg.to_zip().expect("zip")
    }

    /// Story parts gone but still related: validate names each missing
    /// target, repair fixes what it can and reports the rest exactly, and
    /// the readers that need no story part still read the body.
    #[test]
    fn related_story_parts_that_are_missing() {
        let opts = all_checks();
        for (name, bytes) in rich() {
            let b = without_story_parts(&bytes);
            let found = crate::validate::validate(&b).unwrap();
            let has = |code: &str| found.iter().any(|f| f.code == code);
            assert!(
                has("MISSING_REL_TARGET") && has("OVERRIDE_WITHOUT_PART"),
                "{name}"
            );
            let repaired = crate::validate::repair(&b).unwrap();
            assert!(!repaired.repaired.is_empty(), "{name}");
            assert!(
                repaired
                    .repaired
                    .iter()
                    .all(|r| found.iter().any(|f| f.code == r.code)),
                "{name}"
            );
            assert_eq!(
                crate::validate::validate(&repaired.docx).unwrap().len(),
                repaired.remaining.len(),
                "{name}"
            );
            assert!(
                !crate::changes::list_changes(&b).unwrap().is_empty(),
                "{name}"
            );
            assert!(
                crate::comments::list_comments(&b).unwrap().is_empty(),
                "{name}"
            );
            crate::debug::report(&b, Some(&bytes), &opts).unwrap();
            crate::scrub::scrub(&b, &crate::scrub::ScrubOptions::default()).unwrap();
            crate::append::append_documents(&bytes, &b, &Default::default()).unwrap();
            assert_eq!(
                crate::inspect::markdown(&b).is_ok(),
                crate::audit::audit_report_with(&b, &[], None).is_ok(),
                "{name}"
            );
        }
    }

    /// A comment paragraph id past Word's range, written alike in every
    /// comment part: validate reports it and repair renumbers it.
    #[test]
    fn paragraph_ids_past_words_range_are_renumbered() {
        let (_, bytes) = rich().remove(0);
        let high = rewrite(&bytes, |_, xml| Some(xml.replace("11A5D0F2", "91A5D0F2")));
        let found = codes(crate::validate::validate(&high).unwrap());
        let base = codes(crate::validate::validate(&bytes).unwrap());
        assert!(found.len() > base.len(), "{found:?}");
        let repaired = crate::validate::repair(&high).unwrap();
        assert!(!repaired.repaired.is_empty(), "{found:?}");
        let after = crate::comments::list_comments(&repaired.docx).unwrap();
        assert_eq!(after.len(), 3);
    }

    fn with_permission(bytes: &[u8]) -> Vec<u8> {
        rewrite(bytes, |part, xml| {
            (part == "word/document.xml").then(|| {
                xml.replacen(
                    "<w:body>",
                    "<w:body><w:permStart w:id=\"7\" w:edGrp=\"everyone\"/>",
                    1,
                )
                .replacen("<w:sectPr", "<w:permEnd w:id=\"7\"/><w:sectPr", 1)
            })
        })
    }

    /// An editing permission the original grants and the edit removes is
    /// reported; one the edit keeps is not.
    #[test]
    fn dropped_permission_ranges_are_reported() {
        let (name, before, after) = word_redlines()
            .into_iter()
            .find(|(n, ..)| n.contains("case28"))
            .unwrap();
        let dropped = |b: &[u8], a: &[u8]| {
            codes(crate::validate::audit_tracked(b, a, "Comparison").unwrap())
                .iter()
                .filter(|c| *c == "PERMISSION_RANGE_DROPPED")
                .count()
        };
        let granted = with_permission(&before);
        assert!(dropped(&granted, &after) > 0, "{name}");
        assert_eq!(dropped(&granted, &with_permission(&after)), 0, "{name}");
        assert_eq!(dropped(&before, &after), 0, "{name}");
    }

    /// A tracked insertion inside a comment belongs to that comment; one id
    /// in two comments belongs to neither.
    #[test]
    fn revisions_inside_comments_name_their_comment() {
        let (_, bytes) = rich().remove(0);
        let ins = |id: &str, text: &str| {
            format!(
                "<w:ins w:id=\"{id}\" w:author=\"R\" w:date=\"2026-01-01T00:00:00Z\"><w:r><w:t>{text}</w:t></w:r></w:ins>"
            )
        };
        let one = damaged(
            &bytes,
            "word/comments.xml",
            "<w:t>Disagree.</w:t></w:r>",
            &format!("<w:t>Disagree.</w:t></w:r>{}", ins("900", " Strongly.")),
        );
        let bodies = crate::changes::comment_bodies(&one).unwrap();
        assert_eq!(bodies.comments.len(), 3);
        assert_eq!(
            bodies.owners.values().flatten().collect::<Vec<_>>(),
            [&"6".to_string()]
        );
        let shared = damaged(
            &one,
            "word/comments.xml",
            "<w:t>Add a three-year survival period?</w:t></w:r>",
            &format!(
                "<w:t>Add a three-year survival period?</w:t></w:r>{}",
                ins("900", " Or two.")
            ),
        );
        let bodies = crate::changes::comment_bodies(&shared).unwrap();
        assert_eq!(bodies.owners.len(), 1);
        assert_eq!(bodies.owners.values().next(), Some(&None));
        let none = crate::changes::comment_bodies(&without_story_parts(&bytes)).unwrap();
        assert!(none.comments.is_empty() && none.owners.is_empty());
    }

    /// Body content inside a content control, a markup-compatibility choice
    /// and a run holding a foreign element is read like any other.
    #[test]
    fn wrapped_body_content_is_read_through_its_wrappers() {
        let (_, bytes) = rich().remove(0);
        let extra = concat!(
            "<w:sdt><w:sdtPr/><w:sdtContent><w:p><w:r><w:t>Inside a control</w:t></w:r></w:p></w:sdtContent></w:sdt>",
            "<w:p><mc:AlternateContent xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\">",
            "<mc:Choice Requires=\"w14\"><w:r><w:t>Chosen text</w:t></w:r></mc:Choice>",
            "<mc:Fallback><w:r><w:t>Fallback text</w:t></w:r></mc:Fallback></mc:AlternateContent></w:p>",
            "<w:p><w:r><x:foo xmlns:x=\"urn:x\"/><w:t>Foreign run</w:t></w:r></w:p>",
        );
        let wrapped = damaged(
            &bytes,
            "word/document.xml",
            "<w:body>",
            &format!("<w:body>{extra}"),
        );
        let md = crate::inspect::markdown(&wrapped).unwrap();
        for text in ["Inside a control", "Chosen text", "Foreign run"] {
            assert!(md.contains(text), "{text}: {md}");
        }
        assert!(!md.contains("Fallback text"), "{md}");
        let paragraphs = crate::inspect::paragraphs(&wrapped).unwrap().len();
        assert_eq!(
            paragraphs,
            crate::inspect::paragraphs(&bytes).unwrap().len() + 3
        );
    }

    /// Every ordered pair of five varied documents, appended with each
    /// option: A's changes all stay, B adds at most its own, and the result
    /// still validates.
    #[test]
    fn appending_varied_documents_keeps_the_first_whole() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
        let names = [
            "agent-view/received.docx",
            "f1/word-redline.docx",
            "nested_table/hf_b.docx",
            "f4/original.docx",
            "redline-inpi/modified-new-2.docx",
        ];
        let files: Vec<Vec<u8>> = names
            .iter()
            .map(|n| std::fs::read(root.join(n)).unwrap())
            .collect();
        let count = |b: &[u8]| crate::changes::list_changes(b).unwrap().len();
        let breaks = [
            crate::append::SectionBreak::NextPage,
            crate::append::SectionBreak::Continuous,
            crate::append::SectionBreak::None,
        ];
        for (i, a) in files.iter().enumerate() {
            for (j, b) in files.iter().enumerate() {
                if i == j {
                    continue;
                }
                let options = crate::append::AppendOptions {
                    section_break: breaks[(i + j) % 3],
                    keep_sections: (i + j) % 2 == 0,
                    comments: if i % 2 == 0 {
                        crate::append::AppendComments::Carry
                    } else {
                        crate::append::AppendComments::Drop
                    },
                };
                let label = format!("{} + {}", names[i], names[j]);
                let out = crate::append::append_documents(a, b, &options)
                    .unwrap_or_else(|e| panic!("{label}: {e}"));
                let joined = count(&out.docx);
                assert!(
                    joined >= count(a) && joined <= count(a) + count(b),
                    "{label}: {joined}"
                );
                crate::validate::validate(&out.docx).unwrap_or_else(|e| panic!("{label}: {e}"));
            }
        }
    }

    fn with_styles(bytes: &[u8], styles: &str) -> Vec<u8> {
        let xml = format!(
            "<?xml version=\"1.0\"?><w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">{styles}</w:styles>"
        );
        rewrite(bytes, |part, _| {
            (part == "word/styles.xml").then(|| xml.clone())
        })
    }

    /// With no language in the settings, the language of the style named
    /// Normal (no `w:default`, as LibreOffice writes it) is the document's:
    /// present, no finding; absent, MISSING_LANG. A style without an id is
    /// skipped, and defaults without run properties set no font.
    #[test]
    fn the_style_named_normal_carries_the_language() {
        let (_, bytes) = rich().remove(0);
        let no_settings_lang = rewrite(&bytes, |part, xml| {
            (part == "word/settings.xml").then(|| xml.replace("w:themeFontLang", "w:noFontLang"))
        });
        let normal = |lang: &str| {
            format!(
                "<w:docDefaults><w:pPrDefault/></w:docDefaults>\
                 <w:style w:type=\"paragraph\"><w:name w:val=\"Orphan\"/></w:style>\
                 <w:style w:type=\"paragraph\" w:styleId=\"style0\"><w:name w:val=\"Normal\"/><w:rPr>{lang}</w:rPr></w:style>"
            )
        };
        let codes = |styles: &str| {
            crate::audit::audit_report_with(&with_styles(&no_settings_lang, styles), &[], None)
                .unwrap()
                .findings
                .into_iter()
                .map(|f| f.code)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            codes(&normal("<w:lang w:val=\"en-US\"/>")),
            Vec::<String>::new()
        );
        assert_eq!(codes(&normal("")), ["MISSING_LANG"]);
        assert!(
            crate::audit::audit_report_with(&bytes, &[], None)
                .unwrap()
                .findings
                .is_empty()
        );
    }

    /// A content control with no content inside a paragraph adds no text.
    #[test]
    fn an_empty_inline_content_control_adds_no_text() {
        let (_, bytes) = rich().remove(0);
        let sdt = damaged(
            &bytes,
            "word/document.xml",
            "<w:body>",
            "<w:body><w:p><w:sdt><w:sdtPr/></w:sdt><w:r><w:t>After control</w:t></w:r></w:p>",
        );
        let md = crate::inspect::markdown(&sdt).unwrap();
        assert!(md.starts_with("[p0] After control\n"), "{md}");
    }
}
