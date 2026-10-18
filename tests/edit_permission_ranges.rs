// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Public edit regressions for permission ranges and textless paragraphs.
mod common;

use common::docx::{Part, docx, docx_with_sect, para, part_string, run};
use jubarte::edit::{EditPlan, EditResult, apply_plan};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const START: &str = r#"<w:permStart w:id="5" w:edGrp="everyone"/>"#;
const END: &str = r#"<w:permEnd w:id="5"/>"#;
const TEXT: &str = "The Provider shall perform the Services. The Customer shall pay.";

fn probe() -> Vec<u8> {
    docx(&format!(
        "{}<w:p>{START}{}{END}</w:p>{}",
        para("Alice."),
        run(TEXT, false, false, None),
        para("Next clause.")
    ))
}

fn edit(source: &[u8], p: &str, find: &str, replacement: &str) -> EditResult {
    let plan = EditPlan::from_json(&format!(r#"{{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{{"kind":"replace","paragraph":"{p}","find":"{find}","replacement":"{replacement}"}}]}}"#)).unwrap();
    apply_plan(source, &plan).unwrap()
}

// Owner paragraph, text before and after each endpoint, and every attribute.
// Read serialized packages; never inspect the edit's in-memory DOM.
type MarkerLocation = (usize, String, String, Vec<(String, String)>);

fn markers(bytes: &[u8], part: &str, family: &str) -> Vec<MarkerLocation> {
    let xml = part_string(bytes, part).unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let ps = dom.descendants(root, Some(&W::p()));
    let mut out = Vec::new();
    for n in dom.descendants(root, None) {
        if ![format!("{family}Start"), format!("{family}End")]
            .iter()
            .any(|name| dom.name_is(n, &W::name(name)))
        {
            continue;
        }
        let owner = dom
            .ancestors(n, Some(&W::p()))
            .first()
            .copied()
            .unwrap_or(root);
        let mut seen = false;
        let (mut before, mut after) = (String::new(), String::new());
        for child in dom.descendant_nodes(owner) {
            if child == n {
                seen = true;
            } else if dom.name_is(child, &W::t()) {
                if seen {
                    after.push_str(&dom.value(child));
                } else {
                    before.push_str(&dom.value(child));
                }
            }
        }
        let mut attrs: Vec<_> = dom
            .attributes(n)
            .into_iter()
            .map(|(k, v)| (k.local_name().to_string(), v))
            .collect();
        attrs.sort();
        out.push((
            ps.iter().position(|&p| p == owner).unwrap_or(ps.len()),
            before,
            after,
            attrs,
        ));
    }
    out
}

fn assert_permissions(
    source: &[u8],
    result: &EditResult,
    part: &str,
    replacement: Option<(&str, &str)>,
) {
    let mut expected = markers(source, part, "perm");
    assert!(!expected.is_empty());
    if let Some((find, replacement)) = replacement {
        for (_, before, after, _) in &mut expected {
            *before = before.replace(find, replacement);
            *after = after.replace(find, replacement);
        }
    }
    for bytes in [&result.redline, &result.clean] {
        assert_eq!(
            markers(bytes, part, "perm"),
            expected,
            "permission endpoints/attributes changed in {part}"
        );
    }
}

fn save(name: &str, source: &[u8], output: &[u8]) {
    if let Some(dir) = std::env::var_os("JUBARTE_1160_OUTPUT_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{name}-source.docx")), source).unwrap();
        std::fs::write(dir.join(format!("{name}.docx")), output).unwrap();
    }
}

#[test]
fn probe_replace_inside_permission() {
    let source = probe();
    let result = edit(&source, "p1", "shall pay", "shall pay promptly");
    assert_permissions(
        &source,
        &result,
        "word/document.xml",
        Some(("shall pay", "shall pay promptly")),
    );
    save("probe", &source, &result.redline);
}
#[test]
fn probe_replace_outside_permission() {
    let source = probe();
    assert_permissions(
        &source,
        &edit(&source, "p2", "Next", "Following"),
        "word/document.xml",
        None,
    );
}
#[test]
fn probe_flag_edit_preserves_permission() {
    let source = probe();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), &source).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "edit",
            "source.docx",
            "-p",
            "p1",
            "--anchor",
            "shall pay",
            "--content",
            "shall pay promptly",
            "--author",
            "Alice",
            "-o",
            "out.docx",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let output = std::fs::read(dir.path().join("out.docx")).unwrap();
    let actual = markers(&output, "word/document.xml", "perm");
    assert_eq!(actual.len(), 2, "{actual:?}");
    assert_eq!(
        actual,
        markers(
            &edit(&source, "p1", "shall pay", "shall pay promptly").redline,
            "word/document.xml",
            "perm"
        )
    );
}
#[test]
fn permission_spanning_paragraphs_edit_inside() {
    let source = docx(&format!(
        "<w:p>{START}{}</w:p><w:p>{}{END}</w:p>{}",
        run("Alice shall pay.", false, false, None),
        run("Bob shall deliver.", false, false, None),
        para("Next clause.")
    ));
    assert_permissions(
        &source,
        &edit(&source, "p0", "shall pay", "shall pay promptly"),
        "word/document.xml",
        Some(("shall pay", "shall pay promptly")),
    );
}
#[test]
fn permission_spanning_paragraphs_edit_outside() {
    let source = docx(&format!(
        "<w:p>{START}{}</w:p><w:p>{}{END}</w:p>{}",
        run("Alice shall pay.", false, false, None),
        run("Bob shall deliver.", false, false, None),
        para("Next clause.")
    ));
    assert_permissions(
        &source,
        &edit(&source, "p2", "Next", "Following"),
        "word/document.xml",
        None,
    );
}
#[test]
fn permission_table_attributes() {
    let start = r#"<w:permStart w:id="5" w:edGrp="everyone" w:ed="Alice" w:colFirst="0" w:colLast="0" w:displacedByCustomXml="next"/>"#;
    let end = r#"<w:permEnd w:id="5" w:displacedByCustomXml="prev"/>"#;
    let source = docx(&format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid><w:tr><w:tc><w:tcPr/><w:p>{start}{}{end}</w:p></w:tc></w:tr></w:tbl>{}"#,
        run(TEXT, false, false, None),
        para("Next clause.")
    ));
    let result = edit(&source, "p0", "shall pay", "shall pay promptly");
    assert_permissions(
        &source,
        &result,
        "word/document.xml",
        Some(("shall pay", "shall pay promptly")),
    );
    save("table-cell", &source, &result.redline);
}

fn story_source(kind: &str) -> (Vec<u8>, String) {
    let (root, name, reference) = match kind {
        "header" => (
            "hdr",
            "header1",
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        ),
        "footer" => (
            "ftr",
            "footer1",
            r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
        ),
        "footnotes" => ("footnotes", "footnotes", ""),
        "endnotes" => ("endnotes", "endnotes", ""),
        _ => unreachable!(),
    };
    let p = format!("<w:p>{START}{}{END}</w:p>", run(TEXT, false, false, None));
    let content = if kind.ends_with("notes") {
        format!(
            r#"<w:{} w:id="1">{p}</w:{}>"#,
            kind.trim_end_matches('s'),
            kind.trim_end_matches('s')
        )
    } else {
        p
    };
    let xml = format!(r#"<w:{root} xmlns:w="{}">{content}</w:{root}>"#, W::URI);
    let part = format!("word/{name}.xml");
    let ct = format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml");
    let rel = format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}");
    let body = if kind.ends_with("notes") {
        format!(
            r#"<w:p><w:r><w:t>Next clause.</w:t></w:r><w:r><w:{}Reference w:id="1"/></w:r></w:p>"#,
            kind.trim_end_matches('s')
        )
    } else {
        para("Next clause.")
    };
    (
        docx_with_sect(
            &body,
            &[Part {
                name: &part,
                content_type: &ct,
                rel_type: &rel,
                xml: &xml,
            }],
            reference,
        ),
        part,
    )
}
#[test]
fn permission_header() {
    let (source, part) = story_source("header");
    assert_permissions(
        &source,
        &edit(&source, "header1.p0", "shall pay", "shall pay promptly"),
        &part,
        Some(("shall pay", "shall pay promptly")),
    );
}
#[test]
fn permission_footer() {
    let (source, part) = story_source("footer");
    assert_permissions(
        &source,
        &edit(&source, "footer1.p0", "shall pay", "shall pay promptly"),
        &part,
        Some(("shall pay", "shall pay promptly")),
    );
}
#[test]
fn permission_footnote() {
    let (source, part) = story_source("footnotes");
    assert_permissions(
        &source,
        &edit(&source, "footnotes.p0", "shall pay", "shall pay promptly"),
        &part,
        Some(("shall pay", "shall pay promptly")),
    );
}
#[test]
fn permission_endnote() {
    let (source, part) = story_source("endnotes");
    assert_permissions(
        &source,
        &edit(&source, "endnotes.p0", "shall pay", "shall pay promptly"),
        &part,
        Some(("shall pay", "shall pay promptly")),
    );
}
#[test]
fn empty_paragraph_bookmark_stays() {
    let source = docx(&format!(
        r#"<w:p><w:pPr><w:keepNext/></w:pPr><w:bookmarkStart w:id="7" w:name="Clause"/></w:p>{}<w:bookmarkEnd w:id="7"/>{}"#,
        para("Bookmarked clause."),
        para("Next clause.")
    ));
    let result = edit(&source, "p2", "Next", "Following");
    for bytes in [&result.redline, &result.clean] {
        let actual = markers(bytes, "word/document.xml", "bookmark");
        assert_eq!(
            actual[0].0, 0,
            "bookmark start left the empty paragraph: {actual:?}"
        );
        let xml = part_string(bytes, "word/document.xml").unwrap();
        assert!(xml.contains("</w:pPr><w:bookmarkStart"), "{xml}");
    }
    save("empty-bookmark", &source, &result.redline);
}
#[test]
fn empty_paragraph_permission_pair_stays_in_source_order() {
    let source = docx(&format!(
        r#"<w:p><w:pPr><w:keepNext/></w:pPr>{START}<w:bookmarkStart w:id="7" w:name="Clause"/>{END}<w:bookmarkEnd w:id="7"/></w:p>{}{}"#,
        para("Bob."),
        para("Next clause.")
    ));
    let result = edit(&source, "p2", "Next", "Following");
    assert_permissions(&source, &result, "word/document.xml", None);
    for bytes in [&result.redline, &result.clean] {
        let xml = part_string(bytes, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let p = dom.descendants(root, Some(&W::p()))[0];
        let names: Vec<_> = dom
            .elements(p, None)
            .iter()
            .map(|&n| dom.name(n).unwrap().local_name().to_string())
            .collect();
        assert_eq!(
            names,
            [
                "pPr",
                "permStart",
                "bookmarkStart",
                "permEnd",
                "bookmarkEnd"
            ]
        );
    }
}
#[test]
fn validate_edits_reports_removed_permission_markers() {
    let source = probe();
    let xml = part_string(&source, "word/document.xml")
        .unwrap()
        .replace(START, "")
        .replace(END, "");
    let output = common::docx::replace_entry(&source, "word/document.xml", xml.as_bytes());
    let findings = jubarte::validate::audit_tracked(&source, &output, "Alice").unwrap();
    assert!(
        findings.iter().any(|f| f.code == "PERMISSION_RANGE_DROPPED"
            && f.set == "edits"
            && !f.word_fatal
            && f.part == "word/document.xml"
            && f.message.contains('5')),
        "{findings:?}"
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), &source).unwrap();
    std::fs::write(dir.path().join("output.docx"), output).unwrap();
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "validate",
            "output.docx",
            "--rules",
            "edits",
            "--original",
            "source.docx",
            "--author",
            "Alice",
        ])
        .output()
        .unwrap();
    assert_eq!(cli.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&cli.stdout).contains("PERMISSION_RANGE_DROPPED"));
    assert!(
        jubarte::validate::audit_tracked(&source, &source, "Alice")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn permission_follows_bookmark_when_all_its_text_is_deleted() {
    let source = docx(&format!(
        r#"<w:p>{START}<w:bookmarkStart w:id="7" w:name="Clause"/>{}<w:bookmarkEnd w:id="7"/>{END}</w:p>{}"#,
        run("Alice shall pay.", false, false, None),
        para("Next clause.")
    ));
    let plan=EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{"kind":"delete_paragraph","paragraph":"p0"}]}"#).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    save("deleted-range", &source, &result.redline);
    let permissions = markers(&result.redline, "word/document.xml", "perm");
    let bookmarks = markers(&result.redline, "word/document.xml", "bookmark");
    assert_eq!(
        permissions.len(),
        bookmarks.len(),
        "deleted range diverges from bookmark carry-over"
    );
    for (p, b) in permissions.iter().zip(bookmarks) {
        assert_eq!((&p.0, &p.1, &p.2), (&b.0, &b.1, &b.2));
    }
}
#[test]
fn permission_leaves_foreign_content_control() {
    let source = docx(&format!(
        r#"<w:p>{START}<w:sdt><w:sdtPr/><w:sdtContent>{}</w:sdtContent></w:sdt>{END}</w:p>{}"#,
        run("Alice shall pay.", false, false, None),
        para("Next clause.")
    ));
    let result = edit(&source, "p1", "Next", "Following");
    assert_permissions(&source, &result, "word/document.xml", None);
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(
        xml.find("permStart").unwrap() < xml.find("<w:sdt>").unwrap(),
        "{xml}"
    );
    assert!(
        xml.find("permEnd").unwrap() > xml.find("</w:sdt>").unwrap(),
        "{xml}"
    );
}
#[test]
fn empty_paragraph_mapping_survives_inserted_paragraph() {
    let source = docx(&format!(
        r#"{}<w:p/><w:p><w:bookmarkStart w:id="7" w:name="Clause"/>{START}{END}<w:bookmarkEnd w:id="7"/></w:p>{}{}"#,
        para("Alice."),
        para("Bob."),
        para("Next clause.")
    ));
    let plan=EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{"kind":"insert_paragraph","paragraph":"p0","position":"after","runs":[{"text":"Carol Diaz."}]}]}"#).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    for bytes in [&result.clean, &result.redline] {
        for family in ["bookmark", "perm"] {
            let locations = markers(bytes, "word/document.xml", family);
            assert_eq!(locations.len(), 2);
            assert!(
                locations.iter().all(|m| m.0 == 3),
                "{family}: {locations:?}"
            );
        }
    }
}
#[test]
fn permission_inside_existing_content_control_stays_inside() {
    let source = docx(&format!(
        r#"<w:p><w:sdt><w:sdtPr/><w:sdtContent>{START}{}{END}</w:sdtContent></w:sdt></w:p>{}"#,
        run("Alice shall pay.", false, false, None),
        para("Next clause.")
    ));
    let result = edit(&source, "p1", "Next", "Following");
    assert_permissions(&source, &result, "word/document.xml", None);
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(
        xml.find("permStart").unwrap() > xml.find("<w:sdtContent>").unwrap(),
        "{xml}"
    );
    assert!(
        xml.find("permEnd").unwrap() < xml.find("</w:sdtContent>").unwrap(),
        "{xml}"
    );
}

#[test]
fn empty_paragraph_mapping_survives_removed_identical_predecessor() {
    let source = docx(&format!(
        r#"<w:p/><w:p><w:bookmarkStart w:id="7" w:name="Clause"/>{START}{END}<w:bookmarkEnd w:id="7"/></w:p>{}"#,
        para("Next clause.")
    ));
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{"kind":"delete_paragraph","paragraph":"p0"}]}"#).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    let actual = markers(&result.redline, "word/document.xml", "perm");
    assert_eq!(actual.len(), 2);
    // Accept the redline to disregard the deleted predecessor's paragraph mark.
    let accepted = jubarte::document_comparer::accept_revisions(&result.redline).unwrap();
    for family in ["bookmark", "perm"] {
        let actual = markers(&accepted, "word/document.xml", family);
        assert!(actual.iter().all(|m| m.0 == 0), "{family}: {actual:?}");
    }
}

#[test]
fn empty_paragraph_does_not_restore_go_back() {
    let source = docx(&format!(
        r#"<w:p><w:bookmarkStart w:id="8" w:name="_GoBack"/><w:bookmarkEnd w:id="8"/><w:bookmarkStart w:id="7" w:name="Clause"/>{START}{END}<w:bookmarkEnd w:id="7"/></w:p>{}"#,
        para("Next clause.")
    ));
    let plan=EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{"kind":"replace","paragraph":"p1","find":"Next","replacement":"Following"}]}"#).unwrap();
    let result = apply_plan(&source, &plan);
    assert!(
        result.is_ok(),
        "empty paragraph restoration invalidated the bookmark pair: {result:?}"
    );
    let output = result.unwrap().redline;
    let actual = markers(&output, "word/document.xml", "bookmark");
    assert_eq!(actual.len(), 2, "_GoBack endpoint resurrected: {actual:?}");
}

fn delete_paragraph(source: &[u8], paragraph: &str) -> EditResult {
    let plan = EditPlan::from_json(&format!(r#"{{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{{"kind":"delete_paragraph","paragraph":"{paragraph}"}}]}}"#)).unwrap();
    apply_plan(source, &plan).unwrap()
}

fn s3_source(bookmark: bool) -> Vec<u8> {
    let (start, end) = if bookmark {
        (
            r#"<w:bookmarkStart w:id="5" w:name="Clause"/>"#,
            r#"<w:bookmarkEnd w:id="5"/>"#,
        )
    } else {
        (START, END)
    };
    docx(&format!(
        "<w:p>{}{start}{}</w:p><w:p>{}{end}{}</w:p>{}",
        run("Intro ", false, false, None),
        run("text.", false, false, None),
        run("Gone", false, false, None),
        run(" words.", false, false, None),
        para("Last.")
    ))
}

fn deleted_end_stays_at_source(bookmark: bool) {
    let source = s3_source(bookmark);
    let result = delete_paragraph(&source, "p1");
    let family = if bookmark { "bookmark" } else { "perm" };
    save(
        if bookmark {
            "s3-bookmark"
        } else {
            "s3-permission"
        },
        &source,
        &result.redline,
    );
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let ends = dom.descendants(root, Some(&W::name(&format!("{family}End"))));
    assert_eq!(ends.len(), 1, "{xml}");
    let end = ends[0];
    assert!(
        !dom.ancestors(end, Some(&W::del())).is_empty(),
        "end must stay in deletion: {xml}"
    );
    let deletion = dom.ancestors(end, Some(&W::del()))[0];
    let before: String = dom
        .descendant_nodes(deletion)
        .into_iter()
        .take_while(|&n| n != end)
        .filter(|&n| dom.name_is(n, &W::del_text()))
        .map(|n| dom.value(n))
        .collect();
    assert_eq!(before, "Gone", "{xml}");
    let rejected = jubarte::document_comparer::reject_revisions(&result.redline).unwrap();
    let actual: Vec<_> = markers(&rejected, "word/document.xml", family)
        .into_iter()
        .map(|m| (m.0, m.1, m.2))
        .collect();
    let expected: Vec<_> = markers(&source, "word/document.xml", family)
        .into_iter()
        .map(|m| (m.0, m.1, m.2))
        .collect();
    assert_eq!(actual, expected, "Reject must restore source places");
}

#[test]
fn deleted_permission_end_stays_at_source_offset() {
    deleted_end_stays_at_source(false);
}
#[test]
fn deleted_bookmark_end_stays_at_source_offset() {
    deleted_end_stays_at_source(true);
}

#[test]
fn deleted_permission_redline_has_no_dropped_range_finding() {
    let source = docx(&format!(
        "<w:p>{}{START}{}</w:p><w:p>{END}</w:p>{}",
        run("Clause ", false, false, None),
        run("one text.", false, false, None),
        para("Next.")
    ));
    let result = delete_paragraph(&source, "p0");
    save("f1", &source, &result.redline);
    let findings = jubarte::validate::audit_tracked(&source, &result.redline, "Alice").unwrap();
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "PERMISSION_RANGE_DROPPED"),
        "{findings:?}"
    );
}

#[test]
fn surviving_textbox_permission_end_is_not_duplicated() {
    let source = docx(&format!(
        r#"<w:p>{START}{}<w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:p>{}{END}</w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>{}"#,
        run("Alice.", false, false, None),
        run("Bob.", false, false, None),
        para("Next.")
    ));
    let result = edit(&source, "p1", "Next", "Following");
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert_eq!(xml.matches("<w:permEnd").count(), 1, "{xml}");
}

fn header_with(body: &str) -> Vec<u8> {
    let (source, part) = story_source("header");
    let xml = format!(r#"<w:hdr xmlns:w="{}">{body}</w:hdr>"#, W::URI);
    common::docx::replace_entry(&source, &part, xml.as_bytes())
}

#[test]
fn empty_header_bookmark_never_restores_orphan() {
    let source = header_with(&format!(
        r#"<w:p><w:bookmarkStart w:id="7" w:name="Clause"/></w:p><w:p>{}<w:bookmarkEnd w:id="7"/></w:p>"#,
        run("Bob.", false, false, None)
    ));
    let result = edit(&source, "header1.p1", "Bob", "Carol Diaz");
    let actual = markers(&result.redline, "word/header1.xml", "bookmark");
    assert_eq!(actual.len() % 2, 0, "orphan restored: {actual:?}");
}

#[test]
fn repeated_permission_id_keeps_both_positions() {
    let source = docx(&format!(
        "<w:p>{START}{}{END}</w:p><w:p>{START}{}{END}</w:p>{}",
        run("Alice.", false, false, None),
        run("Bob.", false, false, None),
        para("Next.")
    ));
    let result = edit(&source, "p2", "Next", "Following");
    assert_permissions(&source, &result, "word/document.xml", None);
}

#[test]
fn textless_header_permission_range_survives() {
    let source = header_with(&format!("{START}<w:p/>{END}"));
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{"kind":"format_paragraph","paragraph":"header1.p0","alignment":"center"}]}"#).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    assert_permissions(&source, &result, "word/header1.xml", None);
}

#[test]
fn repeated_empty_ranges_keep_positions_and_bookmark_names() {
    let source = docx(&format!(
        r#"<w:p>{START}{END}<w:bookmarkStart w:id="7" w:name="Clause"/><w:bookmarkEnd w:id="7"/></w:p><w:p>{START}{END}<w:bookmarkStart w:id="8" w:name="Clause"/><w:bookmarkEnd w:id="8"/></w:p>{}"#,
        para("Next.")
    ));
    let result = edit(&source, "p2", "Next", "Following");
    assert_permissions(&source, &result, "word/document.xml", None);
    let actual = markers(&result.redline, "word/document.xml", "bookmark");
    assert_eq!(
        actual.iter().map(|m| m.0).collect::<Vec<_>>(),
        [0, 0, 1, 1],
        "{actual:?}"
    );
}

#[test]
fn deleted_permission_start_accepts_at_next_text() {
    let source = docx(&format!(
        "<w:p>{}{START}{}</w:p><w:p>{}{END}</w:p>{}",
        run("Clause ", false, false, None),
        run("one text.", false, false, None),
        run("Next.", false, false, None),
        para("Last.")
    ));
    let result = delete_paragraph(&source, "p0");
    save("s2", &source, &result.redline);
    let accepted = jubarte::document_comparer::accept_revisions(&result.redline).unwrap();
    let actual = markers(&accepted, "word/document.xml", "perm");
    assert_eq!(actual.len(), 2, "{actual:?}");
    assert_eq!((actual[0].1.as_str(), actual[0].2.as_str()), ("", "Next."));
    assert_eq!((actual[1].1.as_str(), actual[1].2.as_str()), ("Next.", ""));
}

#[test]
fn deleted_permission_f1_accepts_without_either_endpoint() {
    let source = docx(&format!(
        "<w:p>{}{START}{}</w:p><w:p>{END}</w:p>{}",
        run("Clause ", false, false, None),
        run("one text.", false, false, None),
        para("Next.")
    ));
    let result = delete_paragraph(&source, "p0");
    let accepted = jubarte::document_comparer::accept_revisions(&result.redline).unwrap();
    assert!(
        markers(&accepted, "word/document.xml", "perm").is_empty(),
        "{}",
        part_string(&accepted, "word/document.xml").unwrap()
    );
}

#[test]
fn bookmark_endpoint_in_deleted_text_is_not_a_revision_crossing() {
    let source = docx(
        r#"<w:p><w:r><w:t>Intro </w:t></w:r><w:bookmarkStart w:id="5" w:name="Clause"/><w:r><w:t>text.</w:t></w:r></w:p><w:p><w:del w:id="1" w:author="Alice"><w:r><w:delText>Gone</w:delText></w:r><w:bookmarkEnd w:id="5"/><w:r><w:delText> words.</w:delText></w:r></w:del></w:p>"#,
    );
    let findings = jubarte::validate::validate(&source).unwrap();
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "BOOKMARK_CROSSES_REVISION"),
        "{findings:?}"
    );
}
