// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Relationship ids in NON-main parts must resolve in that part's own `.rels`.
//!
//! The output package starts as the original (A). Parts whose content comes from
//! the revised document (B) — comments, footnotes, headers/footers, numbering
//! picture bullets — kept B's `r:id`/`r:embed` references but not B's per-part
//! `.rels`, so the ids pointed nowhere. Word refuses such a package outright
//! ("document loaded empty"): 20/451 English redlines and 2/744 harness redlines
//! (2026-09-26 Word-truth bench). Word's own redlines of the same pairs carry 0.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const PKG_REL_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

/// Minimal .docx: `doc_rels` = (id, type-suffix, target); `extra` = (part, content);
/// `overrides` = (part, content type).
fn build_docx(
    body: &str,
    doc_rels: &[(&str, &str, &str)],
    extra: &[(&str, &str)],
    overrides: &[(&str, &str)],
) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        let mut ct = String::from(
            r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
        );
        for (part, ty) in overrides {
            ct.push_str(&format!(
                r#"<Override PartName="/{part}" ContentType="{ty}"/>"#
            ));
        }
        ct.push_str("</Types>");
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(ct.as_bytes()).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(format!(r#"<?xml version="1.0"?><Relationships xmlns="{PKG_REL_NS}"><Relationship Id="rIdM" Type="{REL_NS}/officeDocument" Target="word/document.xml"/></Relationships>"#).as_bytes()).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(
            format!(
                r#"<w:document xmlns:w="{W_NS}" xmlns:r="{REL_NS}"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
            )
            .as_bytes(),
        )
        .unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        let mut r = format!(r#"<?xml version="1.0"?><Relationships xmlns="{PKG_REL_NS}">"#);
        for (id, ty, tg) in doc_rels {
            r.push_str(&format!(
                r#"<Relationship Id="{id}" Type="{REL_NS}/{ty}" Target="{tg}"/>"#
            ));
        }
        r.push_str("</Relationships>");
        z.write_all(r.as_bytes()).unwrap();
        for (name, content) in extra {
            z.start_file(*name, opt).unwrap();
            z.write_all(content.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf
}

fn external_rels(links: &[(&str, &str)]) -> String {
    let mut r = format!(r#"<?xml version="1.0"?><Relationships xmlns="{PKG_REL_NS}">"#);
    for (id, url) in links {
        r.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{REL_NS}/hyperlink" Target="{url}" TargetMode="External"/>"#
        ));
    }
    r.push_str("</Relationships>");
    r
}

fn hyperlink(id: &str, text: &str) -> String {
    format!(r#"<w:hyperlink r:id="{id}"><w:r><w:t>{text}</w:t></w:r></w:hyperlink>"#)
}

/// Every `r:*` relationship attribute in every XML part of `docx`, with the ids
/// that part's `.rels` defines: returns `part: [missing ids]` for each part whose
/// references do not all resolve.
fn dangling_refs(docx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut parts: HashMap<String, String> = HashMap::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut s = String::new();
        if f.read_to_string(&mut s).is_ok() {
            parts.insert(f.name().to_string(), s);
        }
    }
    let mut out = Vec::new();
    let mut names: Vec<&String> = parts.keys().collect();
    names.sort();
    for name in names {
        if !name.ends_with(".xml") || name.contains("_rels/") {
            continue;
        }
        let xml = &parts[name];
        let used: HashSet<String> = ["r:id=\"", "r:embed=\"", "r:link=\"", "r:pict=\""]
            .iter()
            .flat_map(|k| xml.split(k).skip(1).filter_map(|s| s.split('"').next()))
            .map(str::to_string)
            .collect();
        if used.is_empty() {
            continue;
        }
        let (dir, base) = name.rsplit_once('/').unwrap_or(("", name));
        let rels_name = format!("{dir}/_rels/{base}.rels");
        let have: HashSet<String> = parts
            .get(&rels_name)
            .map(|r| {
                r.split(" Id=\"")
                    .skip(1)
                    .filter_map(|s| s.split('"').next())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let mut missing: Vec<&String> = used.difference(&have).collect();
        if !missing.is_empty() {
            missing.sort();
            out.push(format!("{name}: {missing:?}"));
        }
    }
    out
}

fn read_part(docx: &[u8], name: &str) -> Option<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).ok()?;
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    Some(s)
}

#[test]
fn comments_carried_from_the_revised_document_keep_their_hyperlink_rels() {
    let a = build_docx(
        r#"<w:p><w:r><w:t>Shared text.</w:t></w:r></w:p>"#,
        &[],
        &[],
        &[],
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" xmlns:r="{REL_NS}"><w:comment w:id="0" w:author="Rev"><w:p>{}{}</w:p></w:comment></w:comments>"#,
        hyperlink("rId1", "first source"),
        hyperlink("rId2", "second source"),
    );
    let b = build_docx(
        r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>Shared text.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
        &[("rId9", "comments", "comments.xml")],
        &[
            ("word/comments.xml", &comments),
            (
                "word/_rels/comments.xml.rels",
                &external_rels(&[
                    ("rId1", "https://example.com/a"),
                    ("rId2", "https://example.com/b"),
                ]),
            ),
        ],
        &[(
            "word/comments.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        )],
    );

    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let comments_out = read_part(&out, "word/comments.xml").expect("comments carried");
    assert!(comments_out.contains("first source"), "{comments_out}");
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
    let rels = read_part(&out, "word/_rels/comments.xml.rels").expect("comments rels");
    assert!(
        rels.contains("https://example.com/a") && rels.contains("TargetMode=\"External\""),
        "{rels}"
    );
}

#[test]
fn footnotes_from_the_revised_document_keep_their_hyperlink_rels() {
    let a = build_docx(
        r#"<w:p><w:r><w:t>Body text.</w:t></w:r></w:p>"#,
        &[],
        &[],
        &[],
    );
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}" xmlns:r="{REL_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p>{}</w:p></w:footnote></w:footnotes>"#,
        hyperlink("rId1", "cited page"),
    );
    let b = build_docx(
        r#"<w:p><w:r><w:t>Body text.</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[("rId8", "footnotes", "footnotes.xml")],
        &[
            ("word/footnotes.xml", &footnotes),
            (
                "word/_rels/footnotes.xml.rels",
                &external_rels(&[("rId1", "https://example.com/cited")]),
            ),
        ],
        &[(
            "word/footnotes.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        )],
    );

    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let notes = read_part(&out, "word/footnotes.xml").expect("footnotes carried");
    assert!(notes.contains("cited page"), "{notes}");
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
}

/// Internal relationship targets of every `.rels` part that do not resolve to a
/// part in `docx`.
fn unresolved_targets(docx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let names: HashSet<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();
    let mut out = Vec::new();
    let mut rels: Vec<&String> = names.iter().filter(|n| n.ends_with(".rels")).collect();
    rels.sort();
    for rel in rels {
        let mut s = String::new();
        zip.by_name(rel).unwrap().read_to_string(&mut s).unwrap();
        let owner_dir = rel.split("_rels/").next().unwrap_or("");
        for row in s.split("<Relationship ").skip(1) {
            if row.contains("TargetMode=\"External\"") {
                continue;
            }
            let target = row
                .split("Target=\"")
                .nth(1)
                .and_then(|t| t.split('"').next());
            let Some(target) = target else { continue };
            let mut segs: Vec<&str> = Vec::new();
            let joined = match target.strip_prefix('/') {
                Some(abs) => abs.to_string(),
                None => format!("{owner_dir}{target}"),
            };
            for seg in joined.split('/') {
                match seg {
                    "" | "." => {}
                    ".." => {
                        segs.pop();
                    }
                    s => segs.push(s),
                }
            }
            let part = segs.join("/");
            if !names.contains(&part) {
                out.push(format!("{rel}: {target} -> {part}"));
            }
        }
    }
    out
}

#[test]
fn pictures_stored_outside_the_word_folder_keep_resolvable_targets() {
    // B stores its picture at the package root (`/media/image.bin`, as some
    // non-Word producers do); 3e30939b in the 2026-09-26 English redlines
    // shipped `Target="media/image.bin"` — resolved against word/, a part that
    // does not exist — and Word refused the package.
    let a = build_docx(
        r#"<w:p><w:r><w:t>Shared text.</w:t></w:r></w:p>"#,
        &[],
        &[],
        &[],
    );
    let picture = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="100" cy="100"/><wp:docPr id="1" name="P"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="1" name="P"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rId5"/></pic:blipFill><pic:spPr/></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;
    let b = build_docx(
        &format!(r#"<w:p><w:r><w:t>Shared text.</w:t></w:r></w:p>{picture}"#),
        &[("rId5", "image", "/media/image.bin")],
        &[("media/image.bin", "PNGDATA")],
        &[("media/image.bin", "image/png")],
    );

    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let doc = read_part(&out, "word/document.xml").unwrap();
    assert!(doc.contains("r:embed="), "{doc}");
    assert_eq!(unresolved_targets(&out), Vec::<String>::new());
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
}
