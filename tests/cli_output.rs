// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tasks 12 and 13: one writer for `-o`. Compare, accept, reject, scrub,
//! append, convert and read write what the output's extension names: a Word package, the agent
//! view (`.md`), a PDF, PNG pages or the snapshot (`.json`), and say so on
//! stderr, never on stdout.

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

/// What a file of each kind starts with. A convert's Markdown is the
/// document's own text, not a view with a header.
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
        ("compare", &["compare", "t.docx", "t.docx"], "docx", true),
        ("accept", &["accept", "t.docx"], "docx", true),
        ("reject", &["reject", "t.docx"], "docx", true),
        ("scrub", &["scrub", "t.docx"], "docx", true),
        ("append", &["append", "t.docx", "t.docx"], "docx", true),
        (
            "convert-docx",
            &["convert", "t.docx", "--track-changes", "accept"],
            "pdf",
            false,
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
    // A convert's Markdown of a Word document is its text, with no header.
    let md = std::fs::read_to_string(dir.path().join("convert-docx.md")).unwrap();
    assert!(!md.starts_with("---\nsource:"), "{md}");
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
    let out = jubarte(dir.path(), &["compare", "a.md", "b.md", "-o", "c.md"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("c.md")).unwrap(),
        DRAFT
    );
    // The binary reads Markdown by its bytes: names that do not say so
    // still give CriticMarkup.
    std::fs::copy(dir.path().join("a.md"), dir.path().join("a")).unwrap();
    std::fs::copy(dir.path().join("b.md"), dir.path().join("b.json")).unwrap();
    let out = jubarte(dir.path(), &["compare", "a", "b.json", "-o", "n.md"]);
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
        &["compare", "d.docx", "d.docx", "-o", "q3.png", "-q"],
        &["d.docx", "d.docx", "-o", "q3.json", "-q"],
        &[
            "diff", "d.docx", "d.docx", "-t", "pdf", "-o", "q2.pdf", "-q",
        ],
    ] {
        let out = jubarte(dir.path(), args);
        assert!(out.status.success(), "{args:?}: {}", text(&out.stderr));
        assert_eq!(
            (text(&out.stdout), text(&out.stderr)),
            (String::new(), String::new()),
            "{args:?}"
        );
    }
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
