// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration contract shared by selective settlement and convert's processor.
#![cfg(feature = "cli")]

mod common;

use common::docx::{Part, W_NS, docx_with_sect, para, part_string};
use jubarte::changes::ChangeFilter;
use jubarte::comments::list_comments;
use jubarte::xmllinq::Dom;
use std::process::Command;

const FLAG: &str = "--keep-nested-comment";

struct Case {
    name: &'static str,
    kind: &'static str,
    decision: &'static str,
    partial: bool,
    header: bool,
}

const CASES: &[Case] = &[
    Case {
        name: "insertion",
        kind: "ins",
        decision: "reject",
        partial: false,
        header: false,
    },
    Case {
        name: "deletion",
        kind: "del",
        decision: "accept",
        partial: false,
        header: false,
    },
    Case {
        name: "move-source",
        kind: "moveFrom",
        decision: "accept",
        partial: false,
        header: false,
    },
    Case {
        name: "move-destination",
        kind: "moveTo",
        decision: "reject",
        partial: false,
        header: false,
    },
    Case {
        name: "partial-overlap",
        kind: "del",
        decision: "accept",
        partial: true,
        header: false,
    },
    Case {
        name: "header",
        kind: "ins",
        decision: "reject",
        partial: false,
        header: true,
    },
];

fn fixture(case: &Case) -> Vec<u8> {
    let start = r#"<w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/>"#;
    let end = r#"<w:commentRangeEnd w:id="0"/><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="0"/></w:r><w:r><w:commentReference w:id="1"/></w:r>"#;
    let tag = if case.kind == "del" || case.kind == "moveFrom" {
        "delText"
    } else {
        "t"
    };
    let mut changed = format!(
        r#"<w:{kind} w:id="2" w:author="Carol Diaz" w:date="2026-10-19T12:00:00Z">{start}<w:r><w:{tag}>Gone.</w:{tag}></w:r>{end}</w:{kind}>"#,
        kind = case.kind,
        start = if case.partial { "" } else { start },
        end = if case.partial { "" } else { end }
    );
    if case.kind.starts_with("move") {
        let other = if case.kind == "moveFrom" {
            "moveTo"
        } else {
            "moveFrom"
        };
        let other_tag = if other == "moveFrom" { "delText" } else { "t" };
        changed = format!(
            r#"<w:{kind}RangeStart w:id="3" w:name="move"/>{changed}<w:{kind}RangeEnd w:id="3"/><w:{other}RangeStart w:id="4" w:name="move"/><w:{other} w:id="5" w:author="Carol Diaz" w:date="2026-10-19T12:00:00Z"><w:r><w:{other_tag}>Moved.</w:{other_tag}></w:r></w:{other}><w:{other}RangeEnd w:id="4"/>"#,
            kind = case.kind
        );
    }
    let story = format!(
        r#"{}<w:p>{left}<w:r><w:t>Before.</w:t></w:r>{changed}{outside}<w:r><w:t>After.</w:t></w:r></w:p>"#,
        para("Title"),
        left = if case.partial { start } else { "" },
        outside = if case.partial { end } else { "" }
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:comment w:id="0" w:author="Bob" w:initials="B" w:date="2026-10-19T12:00:00Z"><w:p w14:paraId="00000010"><w:r><w:t>Bob's reason.</w:t></w:r></w:p></w:comment><w:comment w:id="1" w:author="Alice" w:initials="A" w:date="2026-10-19T12:01:00Z"><w:p w14:paraId="00000011"><w:r><w:t>Alice agrees.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let extended = r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="00000010"/><w15:commentEx w15:paraId="00000011" w15:paraIdParent="00000010"/></w15:commentsEx>"#;
    let header = format!(r#"<w:hdr xmlns:w="{W_NS}">{story}</w:hdr>"#);
    let mut parts = vec![
        Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        },
        Part {
            name: "word/commentsExtended.xml",
            content_type: "application/vnd.ms-word.commentsExtended+xml",
            rel_type: "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
            xml: extended,
        },
    ];
    if case.header {
        parts.push(Part { name: "word/header1.xml", content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml", rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header", xml: &header });
    }
    docx_with_sect(
        if case.header {
            "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>"
        } else {
            &story
        },
        &parts,
        if case.header {
            r#"<w:headerReference w:type="default" r:id="rIdX2"/>"#
        } else {
            ""
        },
    )
}

/// Exact paragraph text and marker offsets/parents, independent of XML prefixes.
fn story_signature(bytes: &[u8], part: &str) -> serde_json::Value {
    let xml = part_string(bytes, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let mut paragraphs = Vec::new();
    for p in dom.descendants(root, Some(&jubarte::namespaces::W::p())) {
        let mut text = String::new();
        let mut markers = Vec::new();
        for n in dom.descendants(p, None) {
            let Some(name) = dom.name(n) else { continue };
            match name.local_name() {
                "t" | "delText" => text.push_str(&dom.value(n)),
                "commentRangeStart" | "commentRangeEnd" | "commentReference" => {
                    let parent = dom.parent(n).unwrap();
                    let container = if name.local_name() == "commentReference" {
                        dom.parent(parent).unwrap()
                    } else {
                        parent
                    };
                    markers.push(serde_json::json!([
                        name.local_name(),
                        dom.attribute(n, &jubarte::namespaces::W::id()),
                        text.len(),
                        dom.name(container).unwrap().local_name()
                    ]));
                }
                _ => {}
            }
        }
        paragraphs.push(serde_json::json!({"text":text,"markers":markers}));
    }
    serde_json::json!(paragraphs)
}

#[test]
fn routes_share_one_keep_table_and_default_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let mut missing = Vec::new();
    for case in CASES {
        let source = fixture(case);
        let input = dir.path().join(format!("{}.docx", case.name));
        std::fs::write(&input, &source).unwrap();
        if let Some(export) = std::env::var_os("JUBARTE_1279_WORD_DIR") {
            let inputs = std::path::PathBuf::from(export)
                .parent()
                .unwrap()
                .join("inputs");
            std::fs::create_dir_all(&inputs).unwrap();
            std::fs::write(inputs.join(format!("{}.docx", case.name)), &source).unwrap();
        }
        let part = if case.header {
            "word/header1.xml"
        } else {
            "word/document.xml"
        };
        for keep in [false, true] {
            let mut signatures = Vec::new();
            let mut threads = Vec::new();
            for route in [case.decision, "convert"] {
                let output = dir
                    .path()
                    .join(format!("{}-{route}-{keep}.docx", case.name));
                let mut command = Command::new(env!("CARGO_BIN_EXE_jubarte"));
                command.arg(route).arg(&input).arg("-o").arg(&output);
                if route == "convert" {
                    command.args(["--track-changes", case.decision]);
                }
                if keep {
                    command.arg(FLAG);
                }
                let result = command.output().unwrap();
                if !result.status.success() {
                    missing.push(format!(
                        "{} {route} keep={keep}: {}",
                        case.name,
                        String::from_utf8_lossy(&result.stderr)
                    ));
                    continue;
                }
                let bytes = std::fs::read(&output).unwrap();
                if !keep {
                    // These are the original, unchanged public byte facades.
                    let baseline = match (route, case.decision) {
                        ("convert", "accept") => {
                            jubarte::document_comparer::accept_revisions(&source).unwrap()
                        }
                        ("convert", _) => {
                            jubarte::document_comparer::reject_revisions(&source).unwrap()
                        }
                        (_, "accept") => {
                            jubarte::changes::accept_changes(&source, &ChangeFilter::default())
                                .unwrap()
                        }
                        _ => jubarte::changes::reject_changes(&source, &ChangeFilter::default())
                            .unwrap(),
                    };
                    assert_eq!(bytes, baseline, "{} {route}: default bytes", case.name);
                }
                let comments = list_comments(&bytes).unwrap();
                assert_eq!(
                    comments.len(),
                    if keep || case.partial { 2 } else { 0 },
                    "{} {route} keep={keep}",
                    case.name
                );
                if !comments.is_empty() {
                    assert_eq!(
                        (comments[0].id, comments[1].id, comments[1].parent),
                        (0, 1, Some(0))
                    );
                    assert_eq!(
                        comments[0].anchor_text,
                        if case.partial { "Before." } else { "" }
                    );
                    assert_eq!(
                        comments[0].before,
                        if case.partial { "" } else { "Before." }
                    );
                    assert_eq!(
                        comments[0].after,
                        if case.kind.starts_with("move") {
                            "Moved.After."
                        } else {
                            "After."
                        }
                    );
                    assert_eq!(
                        (comments[0].author.as_str(), comments[1].author.as_str()),
                        ("Bob", "Alice")
                    );
                }
                if keep && !case.partial {
                    let paragraph = if case.header { "header1.p1" } else { "p1" };
                    assert!(
                        String::from_utf8_lossy(&result.stderr).contains(&format!(
                            "kept c0 and its reply c1 on an empty anchor in {paragraph}"
                        )),
                        "{} {route}: {}",
                        case.name,
                        String::from_utf8_lossy(&result.stderr)
                    );
                    let signature = story_signature(&bytes, part);
                    assert_eq!(signature[0]["text"], "Title");
                    assert_eq!(
                        signature[1]["text"],
                        if case.kind.starts_with("move") {
                            "Before.Moved.After."
                        } else {
                            "Before.After."
                        }
                    );
                    let markers = signature[1]["markers"].as_array().unwrap();
                    assert_eq!(
                        markers
                            .iter()
                            .map(|m| m[0].as_str().unwrap())
                            .collect::<Vec<_>>(),
                        [
                            "commentRangeStart",
                            "commentRangeStart",
                            "commentRangeEnd",
                            "commentRangeEnd",
                            "commentReference",
                            "commentReference"
                        ]
                    );
                    assert_eq!(markers.len(), 6);
                    for marker in markers {
                        assert_eq!(marker[2], 7, "empty anchor at removed text");
                        assert_eq!(marker[3], "p", "plain untracked markers");
                    }
                }
                if let Some(export) = std::env::var_os("JUBARTE_1279_WORD_DIR") {
                    let export = std::path::PathBuf::from(export);
                    std::fs::create_dir_all(&export).unwrap();
                    if keep || (case.name == "insertion" && route == "reject") {
                        let name = format!(
                            "{}-{route}-{}.docx",
                            case.name,
                            if keep { "keep" } else { "control" }
                        );
                        std::fs::write(export.join(&name), &bytes).unwrap();
                        std::fs::write(export.join(format!("{name}.note")), format!("Open without repair; {}\n", if !keep { "Bob's comment and Alice's reply are absent." } else if case.partial { "Bob's comment and Alice's reply remain on Before. in p1." } else if case.header { "Bob's comment and Alice's reply are threaded at the empty anchor after Before. in header1.p1." } else { "Bob's comment and Alice's reply are threaded at the empty anchor after Before. in p1." })).unwrap();
                    }
                }
                signatures.push(story_signature(&bytes, part));
                threads.push(serde_json::to_value(comments).unwrap());
            }
            if signatures.len() == 2 {
                assert_eq!(
                    signatures[0], signatures[1],
                    "{} keep={keep}: paragraph texts and anchor positions",
                    case.name
                );
                assert_eq!(
                    threads[0], threads[1],
                    "{} keep={keep}: comments and replies",
                    case.name
                );
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing keep behavior:\n{}",
        missing.join("\n")
    );
}

#[test]
fn help_and_capabilities_offer_keep() {
    for task in ["accept", "reject", "convert"] {
        let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args([task, "--help"])
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&result.stdout).contains(FLAG),
            "{task} help must offer {FLAG}"
        );
    }
    for args in [vec!["capabilities"], vec!["capabilities", "--json"]] {
        let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&result.stdout).contains(FLAG),
            "capabilities must offer {FLAG}"
        );
    }
}

#[test]
fn convert_keep_requires_accept_or_reject() {
    for extra in [vec![], vec!["--track-changes", "all"]] {
        let mut args = vec!["convert", "missing.docx", "-o", "out.docx", FLAG];
        args.extend(extra);
        let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains("--track-changes"),
            "must name --track-changes: {error}"
        );
    }
}

#[test]
fn keep_receipt_uses_source_ids_and_published_anchor() {
    let mut pkg = jubarte::opc::PartFs::open(&fixture(&CASES[0])).unwrap();
    for part in ["word/document.xml", "word/comments.xml"] {
        let xml = pkg
            .part_string(part)
            .unwrap()
            .replace("w:id=\"0\"", "w:id=\"8\"")
            .replace("w:id=\"1\"", "w:id=\"9\"");
        pkg.set_part(part, xml.into_bytes());
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.docx");
    std::fs::write(&input, pkg.to_zip().unwrap()).unwrap();
    for route in ["reject", "convert"] {
        let output = dir.path().join(format!("{route}.docx"));
        let mut command = Command::new(env!("CARGO_BIN_EXE_jubarte"));
        command
            .arg(route)
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .arg(FLAG);
        if route == "convert" {
            command.args(["--track-changes", "reject"]);
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{route}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            String::from_utf8_lossy(&result.stderr)
                .contains("kept c8 and its reply c9 on an empty anchor in p1")
        );
        let comments = list_comments(&std::fs::read(&output).unwrap()).unwrap();
        assert_eq!(
            (comments[0].id, comments[1].id, comments[1].parent),
            (0, 1, Some(0))
        );
    }
}

#[test]
fn keep_honors_selection_and_leaves_other_revisions() {
    let mut pkg = jubarte::opc::PartFs::open(&fixture(&CASES[0])).unwrap();
    let xml = pkg.part_string("word/document.xml").unwrap().replace("<w:sectPr>", r#"<w:p><w:ins w:id="6" w:author="Bob"><w:r><w:t>Unselected.</w:t></w:r></w:ins></w:p><w:sectPr>"#);
    pkg.set_part("word/document.xml", xml.into_bytes());
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.docx");
    let output = dir.path().join("output.docx");
    std::fs::write(&input, pkg.to_zip().unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("reject")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .args([FLAG, "--author", "Carol Diaz"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = std::fs::read(output).unwrap();
    let changes = jubarte::changes::list_changes(&bytes).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].author.as_deref(), Some("Bob"));
    assert_eq!(list_comments(&bytes).unwrap().len(), 2);
}

#[test]
fn convert_keeps_before_each_output_format() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.docx");
    std::fs::write(&input, fixture(&CASES[0])).unwrap();
    for format in ["md", "json", "pdf", "png"] {
        let output = dir.path().join(format!("output.{format}"));
        let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .arg("convert")
            .arg(&input)
            .arg("-o")
            .arg(output)
            .args(["--track-changes", "reject", FLAG, "--no-page-markers"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{format}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            String::from_utf8_lossy(&result.stderr)
                .contains("kept c0 and its reply c1 on an empty anchor in p1"),
            "{format}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn markdown_input_keeps_removed_anchor_threads() {
    let source = fixture(&CASES[0]);
    let view = jubarte::markdown::read(
        &source,
        &jubarte::markdown::ReadOptions {
            page_markers: false,
            ..Default::default()
        },
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.md");
    std::fs::write(&input, view.markdown).unwrap();
    let mut missing = Vec::new();
    for format in ["docx", "md"] {
        let output = dir.path().join(format!("output.{format}"));
        let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .arg("convert")
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .args(["--track-changes", "reject", FLAG, "--no-page-markers"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let bytes = std::fs::read(output).unwrap();
        let bytes = if format == "md" {
            jubarte::markdown::markdown_to_docx(
                std::str::from_utf8(&bytes).unwrap(),
                &Default::default(),
            )
            .unwrap()
            .docx
        } else {
            bytes
        };
        let comments = list_comments(&bytes).unwrap();
        if comments.len() != 2 {
            missing.push(format!(
                "{format}: expected 2 comments, got {}",
                comments.len()
            ));
        } else {
            assert_eq!(comments[1].parent, Some(comments[0].id));
            assert_eq!(comments[0].anchor_text, "");
        }
    }
    assert!(
        missing.is_empty(),
        "Markdown conversion must honor the offered keep flag: {missing:?}"
    );
}
