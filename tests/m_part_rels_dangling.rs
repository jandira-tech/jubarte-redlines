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
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

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

/// An inline picture run whose blip embeds relationship `rid`.
fn picture(rid: &str) -> String {
    format!(
        r#"<w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="100" cy="100"/><wp:docPr id="1" name="P"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="1" name="P"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{rid}"/></pic:blipFill><pic:spPr/></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#
    )
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
    let b = build_docx(
        &format!(
            r#"<w:p><w:r><w:t>Shared text.</w:t></w:r></w:p><w:p>{}</w:p>"#,
            picture("rId5")
        ),
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

#[test]
fn footnote_pictures_from_the_revised_document_carry_one_image_part() {
    // Internal targets are copied along with the relationship; two pictures
    // sharing one id get one relationship and one image part, not two.
    let a = build_docx(
        r#"<w:p><w:r><w:t>Body text.</w:t></w:r></w:p>"#,
        &[],
        &[],
        &[],
    );
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}" xmlns:r="{REL_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p>{}{}</w:p></w:footnote></w:footnotes>"#,
        picture("rId3"),
        picture("rId3"),
    );
    let fn_rels = format!(
        r#"<?xml version="1.0"?><Relationships xmlns="{PKG_REL_NS}"><Relationship Id="rId3" Type="{REL_NS}/image" Target="media/fn.png"/></Relationships>"#
    );
    let b = build_docx(
        r#"<w:p><w:r><w:t>Body text.</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[("rId8", "footnotes", "footnotes.xml")],
        &[
            ("word/footnotes.xml", &footnotes),
            ("word/_rels/footnotes.xml.rels", &fn_rels),
            ("word/media/fn.png", "PNGDATA"),
        ],
        &[
            (
                "word/footnotes.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            ),
            ("word/media/fn.png", "image/png"),
        ],
    );

    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let notes = read_part(&out, "word/footnotes.xml").expect("footnotes carried");
    assert_eq!(notes.matches("r:embed=").count(), 2, "{notes}");
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
    assert_eq!(unresolved_targets(&out), Vec::<String>::new());
    let rels = read_part(&out, "word/_rels/footnotes.xml.rels").expect("footnotes rels");
    assert_eq!(rels.matches("/image\"").count(), 1, "{rels}");
    // Copied media get a fresh name; follow the relationship to it.
    let target = rels
        .split("Target=\"")
        .nth(1)
        .and_then(|t| t.split('"').next())
        .expect("image target");
    let image = format!("word/{target}");
    assert_eq!(
        read_part(&out, &image).as_deref(),
        Some("PNGDATA"),
        "{rels}"
    );
    let types = read_part(&out, "[Content_Types].xml").unwrap();
    assert!(
        types.contains(&format!("/{image}\" ContentType=\"image/png\"")),
        "{types}"
    );
}

const V_NS: &str = "urn:schemas-microsoft-com:vml";
const NUMBERING_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml";

/// A numbering part: optional picture bullet 0 drawn from relationship `rId1`,
/// and abstractNum 0 / num 1 whose first level is either that picture bullet
/// or a decimal number.
fn numbering(picture_bullet: bool) -> String {
    let (pic, lvl) = if picture_bullet {
        (
            format!(
                r#"<w:numPicBullet w:numPicBulletId="0"><w:pict><v:shape xmlns:v="{V_NS}" style="width:9pt;height:9pt"><v:imagedata r:id="rId1"/></v:shape></w:pict></w:numPicBullet>"#
            ),
            r#"<w:numFmt w:val="bullet"/><w:lvlText w:val=""/><w:lvlPicBulletId w:val="0"/>"#,
        )
    } else {
        (
            String::new(),
            r#"<w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/>"#,
        )
    };
    format!(
        r#"<w:numbering xmlns:w="{W_NS}" xmlns:r="{REL_NS}">{pic}<w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/>{lvl}<w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
    )
}

/// A document with a numbering part (+ its picture bullet's image when given).
fn listed_docx(body: &str, picture_bullet: Option<&str>) -> Vec<u8> {
    let numbering = numbering(picture_bullet.is_some());
    let rels = format!(
        r#"<?xml version="1.0"?><Relationships xmlns="{PKG_REL_NS}"><Relationship Id="rId1" Type="{REL_NS}/image" Target="media/image1.gif"/></Relationships>"#
    );
    let mut extra = vec![("word/numbering.xml", numbering.as_str())];
    let mut overrides = vec![("word/numbering.xml", NUMBERING_CT)];
    if let Some(gif) = picture_bullet {
        extra.push(("word/_rels/numbering.xml.rels", &rels));
        extra.push(("word/media/image1.gif", gif));
        overrides.push(("word/media/image1.gif", "image/gif"));
    }
    build_docx(
        body,
        &[("rId7", "numbering", "numbering.xml")],
        &extra,
        &overrides,
    )
}

fn list_item(text: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
    )
}

/// `lvlPicBulletId` values of the output numbering part that name no
/// `numPicBullet`, plus the picture each defined bullet draws (bullet id →
/// image bytes).
fn picture_bullets(docx: &[u8]) -> (Vec<String>, HashMap<String, String>) {
    let numbering = read_part(docx, "word/numbering.xml").expect("numbering part");
    let rels = read_part(docx, "word/_rels/numbering.xml.rels").unwrap_or_default();
    let attr = |s: &str, name: &str| {
        s.split(&format!("{name}=\""))
            .nth(1)
            .and_then(|t| t.split('"').next())
            .map(str::to_string)
    };
    let mut images = HashMap::new();
    for bullet in numbering.split("<w:numPicBullet ").skip(1) {
        let bullet = bullet.split("</w:numPicBullet>").next().unwrap();
        let id = attr(bullet, "w:numPicBulletId").expect("bullet id");
        let rid = attr(bullet, "r:id").expect("bullet image rId");
        let target = rels
            .split("<Relationship ")
            .find(|r| attr(r, "Id").as_deref() == Some(rid.as_str()))
            .and_then(|r| attr(r, "Target"))
            .expect("bullet image relationship");
        images.insert(
            id,
            read_part(docx, &format!("word/{target}")).expect("bullet image"),
        );
    }
    let undefined = numbering
        .split("<w:lvlPicBulletId ")
        .skip(1)
        .filter_map(|l| attr(l, "w:val"))
        .filter(|id| !images.contains_key(id))
        .collect();
    (undefined, images)
}

/// B's list draws a picture bullet the original never had. The merged
/// numbering copied B's abstractNum (lvlPicBulletId 0) without B's
/// `numPicBullet`, and Word refused the package ("document loaded empty";
/// italic_rstyle_combos × paragraph_indent_normal_styles). Word's redline
/// carries the bullet and its image.
#[test]
fn revised_picture_bullets_travel_with_their_list() {
    let a = listed_docx(&list_item("Shared item."), None);
    let b = listed_docx(
        &format!("{}{}", list_item("Shared item."), list_item("New item.")),
        Some("GIFB"),
    );
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let (undefined, images) = picture_bullets(&out);
    assert_eq!(undefined, Vec::<String>::new());
    assert_eq!(images.values().collect::<Vec<_>>(), vec!["GIFB"]);
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
}

/// Both documents define picture bullet 0 on relationship `rId1`, with
/// different images. B's bullet takes a fresh id and must keep drawing B's
/// image, not the original's same-id relationship.
#[test]
fn colliding_picture_bullets_keep_their_own_images() {
    let a = listed_docx(&list_item("Shared item."), Some("GIFA"));
    let mut b = listed_docx(
        &format!("{}{}", list_item("Shared item."), list_item("New item.")),
        Some("GIFB"),
    );
    // B's list differs from A's (a wider indent), so it is copied, not merged.
    let mut pkg = jubarte::opc::PartFs::open(&b).unwrap();
    let nb = pkg.part_string("word/numbering.xml").unwrap();
    pkg.set_part(
        "word/numbering.xml",
        nb.replace("w:left=\"720\"", "w:left=\"1080\"").into_bytes(),
    );
    b = pkg.to_zip().unwrap();
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let (undefined, images) = picture_bullets(&out);
    assert_eq!(undefined, Vec::<String>::new());
    assert_eq!(images.len(), 2);
    let mut dom = Dom::new();
    let numbering = read_part(&out, "word/numbering.xml").unwrap();
    let doc = dom.parse_xdocument(&numbering);
    let root = dom.root(doc).unwrap();
    let lists = dom.elements(root, Some(&W::name("abstractNum")));
    assert_eq!(lists.len(), 2);
    let mut revised_list = None;
    for list in lists {
        let levels = dom.elements(list, Some(&W::name("lvl")));
        assert_eq!(levels.len(), 1);
        for level in levels {
            let indent = dom.descendants(level, Some(&W::name("ind")))[0];
            let expected = match dom.attribute(indent, &W::name("left")) {
                Some("720") => "GIFA",
                Some("1080") => {
                    revised_list = dom.attribute(list, &W::name("abstractNumId"));
                    "GIFB"
                }
                other => panic!("unexpected list indent: {other:?}"),
            };
            let bullet = dom.element(level, &W::name("lvlPicBulletId")).unwrap();
            let id = dom.attribute(bullet, &W::val()).unwrap();
            assert_eq!(images.get(id).map(String::as_str), Some(expected));
        }
    }
    let revised_list = revised_list.expect("revised abstract list").to_string();
    let document = read_part(&out, "word/document.xml").unwrap();
    let doc = dom.parse_xdocument(&document);
    let paragraph = dom
        .descendants(doc, Some(&W::p()))
        .into_iter()
        .find(|&p| dom.value(p) == "New item.")
        .expect("revised paragraph");
    let ppr = dom.element(paragraph, &W::p_pr()).unwrap();
    let numpr = dom.element(ppr, &W::name("numPr")).unwrap();
    let num_id = dom.element(numpr, &W::name("numId")).unwrap();
    let num_id = dom.attribute(num_id, &W::val()).unwrap();
    let num = dom
        .elements(root, Some(&W::name("num")))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::name("numId")) == Some(num_id))
        .unwrap();
    let abstract_id = dom.element(num, &W::name("abstractNumId")).unwrap();
    assert_eq!(
        dom.attribute(abstract_id, &W::val()),
        Some(revised_list.as_str())
    );
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
}

#[test]
fn revised_picture_bullets_sharing_an_image_reuse_its_relationship() {
    let a = listed_docx(&list_item("Shared item."), Some("GIFA"));
    let b = listed_docx(&list_item("New item."), Some("GIFB"));
    let mut pkg = jubarte::opc::PartFs::open(&b).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/numbering.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let bullet = dom.element(root, &W::name("numPicBullet")).unwrap();
    let copy = dom.clone_subtree(bullet);
    dom.set_attribute_value(copy, &W::name("numPicBulletId"), Some("1"));
    dom.add_before_self(bullet, copy);
    let list = dom.element(root, &W::name("abstractNum")).unwrap();
    let level = dom.element(list, &W::name("lvl")).unwrap();
    let copy = dom.clone_subtree(level);
    dom.set_attribute_value(copy, &W::name("ilvl"), Some("1"));
    let pic = dom.element(copy, &W::name("lvlPicBulletId")).unwrap();
    dom.set_attribute_value(pic, &W::val(), Some("1"));
    dom.add_after_self(level, copy);
    pkg.set_part(
        "word/numbering.xml",
        dom.serialize_element(root).into_bytes(),
    );

    let out = compare_documents(&a, &pkg.to_zip().unwrap(), "Test").expect("compare ok");
    let (undefined, images) = picture_bullets(&out);
    assert!(undefined.is_empty());
    assert_eq!(images.values().filter(|image| *image == "GIFB").count(), 2);
    let pkg = jubarte::opc::PartFs::open(&out).unwrap();
    let rels = pkg.read_rels_for("word/numbering.xml").unwrap();
    assert_eq!(
        rels.items
            .iter()
            .filter(|r| r.rel_type.ends_with("/image"))
            .count(),
        2
    );
    assert_eq!(
        pkg.parts()
            .iter()
            .filter(|p| p.starts_with("word/media/"))
            .count(),
        2
    );
    assert_eq!(dangling_refs(&out), Vec::<String>::new());
    assert_eq!(unresolved_targets(&out), Vec::<String>::new());
}
