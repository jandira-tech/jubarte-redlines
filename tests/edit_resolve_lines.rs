// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pure revision decisions are confirmed by full before/after view diffs.

mod common;

fn received_path() -> String {
    format!(
        "{}/tests/fixtures/agent-view/received.docx",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn edit_cli(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .args(args)
        .args(["--datetime", "2026-10-11T00:00:00Z"])
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn edit_accept_confirms_the_resolved_insertion() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r0", "--accept", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(stdout.starts_with("SUCCESS:\n~~~~diff\n"), "{stdout}");
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('-') && line.contains("{++quarterly++}")),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('+') && line.contains("including quarterly route-cost")),
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(stderr.starts_with("wrote out.docx (docx, "), "{stderr}");
    assert!(stdout.contains("+source: out.docx\n"), "{stdout}");
}

#[test]
fn edit_reject_prints_the_resolved_insertion_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r0", "--reject", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(stdout.starts_with("SUCCESS:\n~~~~diff\n"), "{stdout}");
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('-') && line.contains("{++quarterly++}")),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('+') && line.contains("including  route-cost")),
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(stderr.starts_with("wrote out.docx (docx, "), "{stderr}");
}

#[test]
fn edit_accept_confirms_a_substitution_pair() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r1+2", "--accept", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(stdout.starts_with("SUCCESS:\n~~~~diff\n"), "{stdout}");
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('-') && line.contains("{~~weekly~>monthly~~}")),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('+') && line.contains("and a monthly review call.")),
        "{stdout}"
    );
}

#[test]
fn edit_reject_prints_a_single_deletion_half() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r3", "--reject", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(stdout.starts_with("SUCCESS:\n~~~~diff\n"), "{stdout}");
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('-') && line.contains("{~~thirty~>forty-five~~}")),
        "{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('+') && line.contains("within thirty{++forty-five++}")),
        "{stdout}"
    );
}

#[test]
fn edit_accept_of_an_unknown_id_keeps_its_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r999", "--accept", "-o", "out.docx"],
    );
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        stdout.contains("UNKNOWN_CHANGE") && stdout.contains("no tracked change r999"),
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        stderr.contains("error: plan refused: UNKNOWN_CHANGE"),
        "{stderr}"
    );
}

#[test]
fn unchanged_text_between_resolved_changes_prevents_a_substitution() {
    let source = common::docx::docx(
        r#"<w:p><w:del w:id="1" w:author="A"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t> kept </w:t></w:r><w:ins w:id="2" w:author="A"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#,
    );
    let lines = jubarte::edit::resolution::resolution_lines(
        &source,
        &jubarte::edit::ResolvedRevisions {
            accepted: vec!["r1".into(), "r2".into()],
            rejected: vec![],
        },
    )
    .unwrap();
    assert_eq!(
        lines,
        [
            "accepted r1 in p0: \"{--old--}\" > \"\"",
            "accepted r2 in p0: \"{++new++}\" > \"new\"",
        ]
    );
}

mod resolution_lines_api {
    use super::common::docx::{Part, docx, docx_with};
    use jubarte::edit::ResolvedRevisions;
    use jubarte::edit::resolution::{resolution_lines, resolution_text};

    const DEL: &str =
        r#"<w:del w:id="1" w:author="A"><w:r><w:delText>old</w:delText></w:r></w:del>"#;
    const INS: &str = r#"<w:ins w:id="2" w:author="A"><w:r><w:t>new</w:t></w:r></w:ins>"#;

    fn resolved(accepted: &[&str], rejected: &[&str]) -> ResolvedRevisions {
        ResolvedRevisions {
            accepted: accepted.iter().map(|s| s.to_string()).collect(),
            rejected: rejected.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn lines(src: &[u8], accepted: &[&str], rejected: &[&str]) -> Vec<String> {
        resolution_lines(src, &resolved(accepted, rejected)).unwrap()
    }

    #[test]
    fn an_accepted_deletion_and_insertion_pair_into_one_substitution() {
        let src = docx(&format!("<w:p>{DEL}{INS}</w:p>"));
        assert_eq!(
            lines(&src, &["r1", "r2"], &[]),
            [r#"accepted r1+2 in p0: "{~~old~>new~~}" > "new""#]
        );
    }

    #[test]
    fn a_rejected_insertion_before_a_deletion_pairs_in_document_order() {
        let src = docx(&format!("<w:p>{INS}{DEL}</w:p>"));
        assert_eq!(
            lines(&src, &[], &["r1", "r2"]),
            [r#"rejected r2+1 in p0: "{~~old~>new~~}" > "old""#]
        );
    }

    #[test]
    fn mixed_decisions_never_pair() {
        let src = docx(&format!("<w:p>{DEL}{INS}</w:p>"));
        assert_eq!(
            lines(&src, &["r1"], &["r2"]),
            [
                r#"accepted r1 in p0: "{--old--}" > """#,
                r#"rejected r2 in p0: "{++new++}" > """#,
            ]
        );
    }

    #[test]
    fn hyperlink_and_content_control_wrappers_do_not_break_a_pair() {
        for wrapped in [
            format!(r#"<w:p><w:hyperlink w:anchor="x">{DEL}</w:hyperlink>{INS}</w:p>"#),
            format!("<w:p><w:sdt><w:sdtContent>{DEL}</w:sdtContent></w:sdt>{INS}</w:p>"),
        ] {
            assert_eq!(
                lines(&docx(&wrapped), &["r1", "r2"], &[]),
                [r#"accepted r1+2 in p0: "{~~old~>new~~}" > "new""#],
                "{wrapped}"
            );
        }
    }

    #[test]
    fn a_paragraph_mark_resolves_to_a_paragraph_break() {
        let mark = |kind: &str, id: &str| {
            docx(&format!(
                r#"<w:p><w:pPr><w:rPr><w:{kind} w:id="{id}" w:author="A"/></w:rPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p><w:p><w:r><w:t>two</w:t></w:r></w:p>"#
            ))
        };
        let ins = mark("ins", "5");
        assert_eq!(
            lines(&ins, &["r5"], &[]),
            [r#"accepted r5 in p0: "{++\n\n++}" > "\n\n""#]
        );
        assert_eq!(
            lines(&ins, &[], &["r5"]),
            [r#"rejected r5 in p0: "{++\n\n++}" > """#]
        );
        let del = mark("del", "6");
        assert_eq!(
            lines(&del, &["r6"], &[]),
            [r#"accepted r6 in p0: "{--\n\n--}" > """#]
        );
        assert_eq!(
            lines(&del, &[], &["r6"]),
            [r#"rejected r6 in p0: "{--\n\n--}" > "\n\n""#]
        );
    }

    #[test]
    fn a_property_change_shows_its_unchanged_text() {
        let run = docx(
            r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="7" w:author="A"><w:rPr/></w:rPrChange></w:rPr><w:t>bold</w:t></w:r></w:p>"#,
        );
        assert_eq!(
            lines(&run, &["r7"], &[]),
            [r#"accepted r7 in p0: "bold" > "bold""#]
        );
        let para = docx(
            r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="8" w:author="A"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>centered</w:t></w:r></w:p>"#,
        );
        assert_eq!(
            lines(&para, &[], &["r8"]),
            [r#"rejected r8 in p0: "centered" > "centered""#]
        );
    }

    #[test]
    fn a_move_resolves_each_half_in_its_own_paragraph() {
        let src = docx(
            r#"<w:p><w:moveFromRangeStart w:id="10" w:author="A" w:name="m1"/><w:moveFrom w:id="11" w:author="A"><w:r><w:t>moved</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="10"/></w:p><w:p><w:moveToRangeStart w:id="12" w:author="A" w:name="m1"/><w:moveTo w:id="13" w:author="A"><w:r><w:t>moved</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="12"/></w:p>"#,
        );
        assert_eq!(
            lines(&src, &["r11", "r13"], &[]),
            [
                r#"accepted r11 in p0: "{--moved--}" > """#,
                r#"accepted r13 in p1: "{++moved++}" > "moved""#,
            ]
        );
        assert_eq!(
            lines(&src, &[], &["r11", "r13"]),
            [
                r#"rejected r11 in p0: "{--moved--}" > "moved""#,
                r#"rejected r13 in p1: "{++moved++}" > """#,
            ]
        );
    }

    #[test]
    fn markup_delimiters_inside_revised_text_are_escaped() {
        let src = docx(
            r#"<w:p><w:ins w:id="3" w:author="A"><w:r><w:t>a {++b++} c~>d</w:t></w:r></w:ins></w:p>"#,
        );
        assert_eq!(
            lines(&src, &["r3"], &[]),
            [r#"accepted r3 in p0: "{++a {\\++b++\\} c~\\>d++}" > "a {++b++} c~>d""#]
        );
    }

    #[test]
    fn unresolved_revisions_are_left_out() {
        let src = docx(&format!("<w:p>{DEL}{INS}</w:p>"));
        assert_eq!(
            lines(&src, &["r1"], &[]),
            [r#"accepted r1 in p0: "{--old--}" > """#]
        );
    }

    #[test]
    fn a_header_pair_names_its_story() {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p>{DEL}{INS}</w:p></w:hdr>"#
        );
        let header = Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &xml,
        };
        let src = docx_with("<w:p><w:r><w:t>body</w:t></w:r></w:p>", &[header]);
        assert_eq!(
            lines(&src, &["header1.r1", "header1.r2"], &[]),
            [r#"accepted header1.r1+2 in header1: "{~~old~>new~~}" > "new""#]
        );
    }

    #[test]
    fn resolution_text_ends_each_line_with_a_newline() {
        let src = docx(&format!("<w:p>{DEL}{INS}</w:p>"));
        assert_eq!(
            resolution_text(&src, &resolved(&["r1", "r2"], &[])).unwrap(),
            "accepted r1+2 in p0: \"{~~old~>new~~}\" > \"new\"\n"
        );
    }

    #[test]
    fn a_non_package_input_is_an_error() {
        let err = resolution_lines(b"nope", &resolved(&["r1"], &[])).unwrap_err();
        assert!(err.to_string().contains("not a ZIP file"), "{err}");
    }
}
