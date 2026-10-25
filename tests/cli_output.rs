// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tasks 12 and 13: one writer for `-o`. Compare, accept, reject, scrub,
//! append, convert and read write what the output's extension names: a Word package, the agent
//! view (`.md`), a PDF, PNG pages or the snapshot (`.json`), and say so on
//! stderr, never on stdout.

mod common;

use common::docx::{docx, para};
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

const TRACKED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/agent-view/received.docx"
);

const DRAFT: &str = "# Terms\n\nPayment is due in {~~30~>45~~} days.\n";

fn jubarte(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// What a file of each kind starts with. A convert's Markdown of Markdown
/// is the document's own text, not a view with a header.
fn magic(kind: &str, view: bool) -> &'static [u8] {
    match kind {
        "docx" => b"PK\x03\x04",
        "md" if view => b"---\nsource: ",
        "md" => b"",
        "pdf" => b"%PDF",
        "json" => b"{\"schema_version\"",
        _ => unreachable!("{kind}"),
    }
}

#[test]
fn every_writing_task_writes_what_its_output_names() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(TRACKED, dir.path().join("t.docx")).unwrap();
    std::fs::write(dir.path().join("d.md"), DRAFT).unwrap();
    let tasks: [(&str, &[&str], &str, bool); 7] = [
        (
            "compare",
            &["compare", "t.docx", "t.docx", "-d", "1970-01-01T00:00:00Z"],
            "docx",
            true,
        ),
        ("accept", &["accept", "t.docx"], "docx", true),
        ("reject", &["reject", "t.docx"], "docx", true),
        ("scrub", &["scrub", "t.docx"], "docx", true),
        ("append", &["append", "t.docx", "t.docx"], "docx", true),
        (
            "convert-docx",
            &["convert", "t.docx", "--track-changes", "accept"],
            "pdf",
            true,
        ),
        ("convert-md", &["convert", "d.md"], "docx", false),
    ];
    for (name, args, unknown, view) in tasks {
        for ext in ["docx", "md", "pdf", "json", "tmp"] {
            let file = format!("{name}.{ext}");
            let out = jubarte(dir.path(), &[args, &["-o", &file]].concat());
            let stderr = text(&out.stderr);
            assert!(out.status.success(), "{file}: {stderr}");
            assert_eq!(text(&out.stdout), "", "{file}: stdout stays empty");
            let kind = if ext == "tmp" { unknown } else { ext };
            assert!(
                stderr.contains(&format!("wrote {file} ({kind}, ")),
                "{file}: {stderr}"
            );
            let bytes = std::fs::read(dir.path().join(&file)).unwrap();
            assert!(
                bytes.starts_with(magic(kind, view)),
                "{file} starts {:?}",
                text(&bytes[..bytes.len().min(24)])
            );
        }
    }
    // A convert's Markdown of Markdown is its text with the changes kept.
    let md = std::fs::read_to_string(dir.path().join("convert-md.md")).unwrap();
    assert_eq!(md, DRAFT);
    // A convert's Markdown of a Word document is the agent view: its
    // header names the document read.
    let md = std::fs::read_to_string(dir.path().join("convert-docx.md")).unwrap();
    assert!(md.starts_with("---\nsource: t.docx\n"), "{md}");
    // The view of a redline names the redline compare did not write.
    let md = std::fs::read_to_string(dir.path().join("compare.md")).unwrap();
    assert!(
        md.starts_with("---\nsource: t_v_t.docx (not written; -o keeps it)\n"),
        "{md}"
    );
    // The view of a task's result names the Word file it did not write.
    let md = std::fs::read_to_string(dir.path().join("accept.md")).unwrap();
    assert!(
        md.starts_with("---\nsource: accept.docx (not written)\n"),
        "{md}"
    );
}

/// Two Markdown documents written to a `.md` are CriticMarkup, which the
/// view's options do not shape.
#[test]
fn compare_writes_criticmarkup_for_two_markdown_documents() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.md"),
        "# Terms\n\nPayment is due in 30 days.\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.md"),
        "# Terms\n\nPayment is due in 45 days.\n",
    )
    .unwrap();
    let out = jubarte(
        dir.path(),
        &[
            "compare",
            "a.md",
            "b.md",
            "-d",
            "1970-01-01T00:00:00Z",
            "-o",
            "c.md",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("c.md")).unwrap(),
        DRAFT
    );
    // The binary reads Markdown by its bytes: names that do not say so
    // still give CriticMarkup.
    std::fs::copy(dir.path().join("a.md"), dir.path().join("a")).unwrap();
    std::fs::copy(dir.path().join("b.md"), dir.path().join("b.json")).unwrap();
    let out = jubarte(
        dir.path(),
        &[
            "compare",
            "a",
            "b.json",
            "-d",
            "1970-01-01T00:00:00Z",
            "-o",
            "n.md",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("n.md")).unwrap(),
        DRAFT
    );
    // A view option there is a usage error, even over an existing file.
    std::fs::write(dir.path().join("h.md"), "keep").unwrap();
    for args in [
        &["compare", "a.md", "b.md", "-o", "h.md", "--head", "1"][..],
        &["a", "b.json", "-o", "h.md", "--comments", "inline"],
    ] {
        let out = jubarte(dir.path(), args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(
            text(&out.stderr).contains("two Markdown documents write CriticMarkup"),
            "{}",
            text(&out.stderr)
        );
    }
    assert_eq!(
        std::fs::read_to_string(dir.path().join("h.md")).unwrap(),
        "keep"
    );
}

/// The Markdown-input options with two Word documents would do nothing.
#[test]
fn compare_refuses_markdown_options_for_two_word_documents() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(TRACKED, dir.path().join("t.docx")).unwrap();
    for flags in [
        &["--reference-doc", "missing.docx"][..],
        &["--critic"],
        &["--resource-path", "."],
    ] {
        let out = jubarte(
            dir.path(),
            &[&["compare", "t.docx", "t.docx", "-o", "r.docx"][..], flags].concat(),
        );
        assert_eq!(out.status.code(), Some(2), "{flags:?}");
        assert!(
            text(&out.stderr).contains("applies to Markdown inputs; both documents are Word"),
            "{}",
            text(&out.stderr)
        );
        assert!(!dir.path().join("r.docx").exists());
    }
}

#[test]
fn png_pages_and_quiet() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("d.md"), DRAFT).unwrap();
    let out = jubarte(
        dir.path(),
        &["convert", "d.md", "-o", "d.docx", "--author", "Legal"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let out = jubarte(
        dir.path(),
        &["accept", "d.docx", "-o", "r.png", "--dpi", "20"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout), "");
    assert!(
        text(&out.stderr).contains("wrote r-page-NN.png (png, 1 page, 20 dpi)"),
        "{}",
        text(&out.stderr)
    );
    assert!(dir.path().join("r-page-01.png").exists());
    for args in [
        &["accept", "d.docx", "-q", "-o", "q.docx"][..],
        &["reject", "d.docx", "-o", "q.pdf", "-q"],
        &["read", "d.docx", "-o", "q.json", "-q"],
        &["scrub", "d.docx", "-o", "q.md", "--quiet"],
        &[
            "compare",
            "d.docx",
            "d.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "-o",
            "q3.png",
            "-q",
        ],
        &[
            "d.docx",
            "d.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "-o",
            "q3.json",
            "-q",
        ],
        &["diff", "d.docx", "d.docx", "-o", "q2.patch", "-q"],
    ] {
        let out = jubarte(dir.path(), args);
        assert!(out.status.success(), "{args:?}: {}", text(&out.stderr));
        assert_eq!(
            (text(&out.stdout), text(&out.stderr)),
            (String::new(), String::new()),
            "{args:?}"
        );
    }
    // -q silences the report, never the write (ledger review of #444, L5).
    for name in [
        "q.docx",
        "q.pdf",
        "q.json",
        "q.md",
        "q3-page-01.png",
        "q3.json",
    ] {
        let len = std::fs::metadata(dir.path().join(name))
            .unwrap_or_else(|error| panic!("{name}: {error}"))
            .len();
        assert!(len > 0, "{name} is empty");
    }
    // The same document on both sides has no diff: the patch is empty.
    let patch = std::fs::read(dir.path().join("q2.patch")).unwrap();
    assert_eq!(text(&patch), "");
}

#[test]
fn read_writes_the_view_or_the_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(TRACKED, dir.path().join("t.docx")).unwrap();
    let out = jubarte(dir.path(), &["read", "t.docx", "-o", "v.md"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let view = std::fs::read_to_string(dir.path().join("v.md")).unwrap();
    assert!(view.starts_with("---\nsource: t.docx\n"), "{view}");
    let printed = jubarte(dir.path(), &["read", "t.docx"]);
    assert_eq!(view, text(&printed.stdout), "-o writes what read prints");
    // An extension read does not know writes the view too.
    let out = jubarte(dir.path(), &["read", "t.docx", "-o", "v.weird"]);
    assert!(
        text(&out.stderr).contains("wrote v.weird (md, "),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("v.weird")).unwrap(),
        view
    );
    let out = jubarte(dir.path(), &["read", "t.docx", "-o", "v.json", "-p", "p3"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let printed = jubarte(dir.path(), &["read", "t.docx", "--json", "-p", "p3"]);
    assert_eq!(
        std::fs::read(dir.path().join("v.json")).unwrap(),
        printed.stdout,
        "-o v.json writes what --json prints"
    );
    let out = jubarte(dir.path(), &["read", "t.docx", "-o", "v.docx"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = text(&out.stderr);
    assert!(
        stderr.contains("convert") && stderr.contains("Usage:"),
        "{stderr}"
    );
    assert!(!dir.path().join("v.docx").exists());
    // The one-document shorthand is read.
    let out = jubarte(dir.path(), &["t.docx", "-o", "h.md", "--head", "1"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        std::fs::read_to_string(dir.path().join("h.md"))
            .unwrap()
            .contains("range:"),
    );
    let out = jubarte(dir.path(), &["t.docx", "-o", "h.docx"]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stderr));
}

#[test]
fn the_render_options_go_with_a_pdf_or_png() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(TRACKED, dir.path().join("t.docx")).unwrap();
    for flag in [
        &["--changed-pages"][..],
        &["--pages", "1"],
        &["--dpi", "50"],
    ] {
        let out = jubarte(
            dir.path(),
            &[&["accept", "t.docx", "-o", "r.docx"][..], flag].concat(),
        );
        assert_eq!(out.status.code(), Some(2), "{flag:?}");
        assert!(
            text(&out.stderr).contains("applies to PDF or PNG output only"),
            "{}",
            text(&out.stderr)
        );
    }
    let out = jubarte(
        dir.path(),
        &["accept", "t.docx", "-o", "r.pdf", "--changed-pages"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let accepted = jubarte::changes::accept_changes(
        &std::fs::read(TRACKED).unwrap(),
        &jubarte::changes::ChangeFilter::default(),
    )
    .unwrap();
    let expected = jubarte::convert::render(
        &accepted,
        jubarte::convert::PdfOptions {
            compress: false,
            revisions: jubarte::convert::RevisionStyle::Conventional,
            comments: jubarte::convert::CommentPlacement::Margin,
            comment_notes: false,
            changed_only: true,
        },
        jubarte::convert::RenderRequest {
            pdf: true,
            png_dpi: None,
            pages: None,
        },
    )
    .unwrap();
    assert!(
        std::fs::read(dir.path().join("r.pdf")).unwrap() == expected.pdf.unwrap(),
        "accept -o r.pdf renders the accepted document"
    );
}

/// Use the existing Markdown importer only to construct a small, deterministic
/// tracked document. Assertions below describe its expected visible content.
fn tracked_draft() -> Vec<u8> {
    jubarte::markdown::markdown_to_docx(
        DRAFT,
        &jubarte::markdown::DocxOptions {
            author: "Legal".to_string(),
            date: "2026-10-01T00:00:00Z".to_string(),
            ..Default::default()
        },
    )
    .unwrap()
    .docx
}

#[test]
fn json_and_markdown_writers_serialize_the_resolved_document() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("draft.docx"), tracked_draft()).unwrap();
    for (task, expected, absent) in [
        (
            "accept",
            "Payment is due in 45 days.",
            "Payment is due in 30 days.",
        ),
        (
            "reject",
            "Payment is due in 30 days.",
            "Payment is due in 45 days.",
        ),
    ] {
        for ext in ["json", "md"] {
            let file = format!("{task}.{ext}");
            let result = jubarte(dir.path(), &[task, "draft.docx", "-o", &file]);
            assert!(
                result.status.success(),
                "{task}/{ext}: {}",
                text(&result.stderr)
            );
            assert!(result.stdout.is_empty());
            let bytes = std::fs::read(dir.path().join(&file)).unwrap();
            if ext == "json" {
                let snapshot: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(snapshot["changes"], serde_json::json!([]));
                let paragraphs: Vec<_> = snapshot["paragraphs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| p["text"].as_str().unwrap())
                    .collect();
                assert_eq!(paragraphs, ["Terms", expected]);
            } else {
                let view = text(&bytes);
                assert!(view.contains(expected), "{view}");
                assert!(!view.contains(absent), "{view}");
                assert!(!view.contains("{~~"), "{view}");
                assert!(
                    view.starts_with(&format!("---\nsource: {task}.docx (not written)\n")),
                    "{view}"
                );
            }
            assert!(!dir.path().join(format!("{task}.docx")).exists());
        }
    }
}

#[test]
fn convert_json_resolves_changes_and_uses_the_default_output_name() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("draft.docx"), tracked_draft()).unwrap();
    std::fs::write(dir.path().join("draft.md"), DRAFT).unwrap();
    for input in ["draft.docx", "draft.md"] {
        for (mode, expected) in [
            ("accept", "Payment is due in 45 days."),
            ("reject", "Payment is due in 30 days."),
        ] {
            let result = jubarte(
                dir.path(),
                &[
                    "convert",
                    input,
                    "--to",
                    "json",
                    "--track-changes",
                    mode,
                    "--force",
                ],
            );
            assert!(
                result.status.success(),
                "{input}/{mode}: {}",
                text(&result.stderr)
            );
            assert!(result.stdout.is_empty());
            let bytes = std::fs::read(dir.path().join("draft.json")).unwrap();
            let snapshot: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(snapshot["changes"], serde_json::json!([]));
            assert_eq!(snapshot["paragraphs"][1]["text"], expected);
            assert_eq!(
                text(&result.stderr),
                format!("wrote draft.json (json, {} bytes)\n", bytes.len())
            );
            assert_eq!(bytes.last(), Some(&b'\n'));
        }
    }
}

#[test]
fn writers_refuse_overwrite_and_force_replaces_the_output() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(&para("Keep this text"));
    std::fs::write(dir.path().join("in.docx"), &original).unwrap();
    for task in ["accept", "reject", "scrub", "append", "convert", "read"] {
        for ext in ["md", "json"] {
            let file = format!("{task}.{ext}");
            let path = dir.path().join(&file);
            std::fs::write(&path, b"existing output").unwrap();
            let mut args = vec![task, "in.docx"];
            if task == "append" {
                args.push("in.docx");
            }
            args.extend(["-o", &file, "-q"]);
            let refused = jubarte(dir.path(), &args);
            assert_eq!(
                refused.status.code(),
                Some(1),
                "{args:?}: {}",
                text(&refused.stderr)
            );
            assert!(refused.stdout.is_empty());
            assert!(text(&refused.stderr).contains("already exists (use --force to overwrite)"));
            assert!(!text(&refused.stderr).contains("wrote "));
            assert_eq!(std::fs::read(&path).unwrap(), b"existing output");
            args.push("--force");
            let written = jubarte(dir.path(), &args);
            assert!(
                written.status.success(),
                "{args:?}: {}",
                text(&written.stderr)
            );
            assert!(written.stdout.is_empty());
            assert!(written.stderr.is_empty(), "{}", text(&written.stderr));
            let contents = std::fs::read_to_string(&path).unwrap();
            assert!(
                contents.contains("Keep this text"),
                "{task}/{ext}: {contents}"
            );
            if ext == "json" {
                let snapshot: serde_json::Value = serde_json::from_str(&contents).unwrap();
                let texts: Vec<_> = snapshot["paragraphs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|p| p["text"].as_str())
                    .filter(|s| !s.is_empty())
                    .collect();
                assert_eq!(
                    texts,
                    vec!["Keep this text"; if task == "append" { 2 } else { 1 }]
                );
            }
            assert_eq!(std::fs::read(dir.path().join("in.docx")).unwrap(), original);
        }
    }
}

#[test]
fn writer_png_selection_keeps_page_numbers_and_checks_all_paths_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let page_break = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;
    let bytes = docx(&(para("First") + page_break + &para("Second") + page_break + &para("Third")));
    std::fs::write(dir.path().join("in.docx"), bytes).unwrap();
    std::fs::write(dir.path().join("out-page-03.png"), b"existing page").unwrap();
    let args = [
        "reject", "in.docx", "-o", "out.png", "--pages", "3,1,3", "--dpi", "20",
    ];
    let refused = jubarte(dir.path(), &args);
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused.stderr));
    assert!(
        text(&refused.stderr).contains("output 'out-page-03.png' already exists"),
        "{}",
        text(&refused.stderr)
    );
    assert!(!dir.path().join("out-page-01.png").exists());
    assert_eq!(
        std::fs::read(dir.path().join("out-page-03.png")).unwrap(),
        b"existing page"
    );
    let written = jubarte(dir.path(), &[&args[..], &["--force"]].concat());
    assert!(written.status.success(), "{}", text(&written.stderr));
    assert!(written.stdout.is_empty());
    assert_eq!(
        text(&written.stderr),
        "wrote out-page-NN.png (png, 2 pages, 20 dpi)\n"
    );
    for name in ["out-page-01.png", "out-page-03.png"] {
        let png = std::fs::read(dir.path().join(name)).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
    assert!(!dir.path().join("out-page-02.png").exists());
    assert!(!dir.path().join("out.png").exists());
}

#[test]
fn json_writer_keeps_unselected_changes_and_scrubbed_authors() {
    let dir = tempfile::tempdir().unwrap();
    let original = tracked_draft();
    std::fs::write(dir.path().join("draft.docx"), &original).unwrap();
    for (task, flags, author) in [
        ("accept", ["--author", "Other"], "Legal"),
        ("reject", ["--author", "Other"], "Legal"),
        ("scrub", ["--author-alias", "Anonymous"], "Anonymous"),
    ] {
        let output = format!("{task}.json");
        let args = [&[task, "draft.docx", "-o", &output][..], &flags].concat();
        let result = jubarte(dir.path(), &args);
        assert!(
            result.status.success(),
            "{args:?}: {}",
            text(&result.stderr)
        );
        let snapshot: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.path().join(output)).unwrap()).unwrap();
        let changes = snapshot["changes"].as_array().unwrap();
        assert_eq!(changes.len(), 2, "{task}: {snapshot}");
        assert!(
            changes.iter().all(|change| change["author"] == author),
            "{task}: {snapshot}"
        );
        assert_eq!(
            std::fs::read(dir.path().join("draft.docx")).unwrap(),
            original
        );
    }
}
