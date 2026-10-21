// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `insert_image`: a new paragraph holding an inline picture next to the
//! anchor paragraph. The picture's bytes come base64-encoded in the plan;
//! the media part, its content type and the relationship are added, and the
//! redline tracks the new paragraph as an insertion.

mod common;

use std::io::Cursor;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::{paragraphs, source_sha256, summary};

fn plan(source: &[u8], operations: &str) -> EditPlan {
    let json = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-10-02T12:00:00Z","operations":{operations}}}"#,
        source_sha256(source)
    );
    EditPlan::from_json(&json).unwrap()
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn encoded(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    image::RgbImage::from_pixel(width, height, image::Rgb([200, 30, 30]))
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();
    bytes
}

fn gif(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 200, 255]))
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Gif)
        .unwrap();
    bytes
}

/// The value of `attr="..."` in the first element named `element`.
fn attr<'x>(xml: &'x str, element: &str, attr: &str) -> &'x str {
    let at = xml
        .find(&format!("<{element} "))
        .unwrap_or_else(|| panic!("{element}"));
    let tag = &xml[at..at + xml[at..].find('>').unwrap()];
    let key = format!(" {attr}=\"");
    let start = tag.find(&key).unwrap_or_else(|| panic!("{attr} in {tag}")) + key.len();
    &tag[start..start + tag[start..].find('"').unwrap()]
}

#[test]
fn insert_image_adds_a_picture_paragraph_with_media_and_relationship() {
    let source = docx(&(para("Figure follows.") + &para("After the figure.")));
    let image = png(40, 20);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            &format!(
                r#"[{{"kind":"insert_image","paragraph":{{"index":0}},"position":"after","image_base64":"{}","content_type":"image/png","width_emu":2743200,"alt":"Diagram"}}]"#,
                encoded(&image)
            ),
        ),
    )
    .unwrap();
    assert_eq!(summary(&source).unwrap().images, 0);
    assert_eq!(summary(&result.clean).unwrap().images, 1);
    let all = paragraphs(&result.clean).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[1].text, "");
    assert!(all[1].limitations.contains(&"drawing".to_string()));
    assert_eq!(texts(&result.clean)[2], "After the figure.");
    let body = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(attr(&body, "wp:extent", "cx"), "2743200");
    assert_eq!(attr(&body, "wp:extent", "cy"), "1371600");
    assert_eq!(attr(&body, "wp:docPr", "descr"), "Diagram");
    let rid = attr(&body, "a:blip", "r:embed");
    let rels = part_string(&result.clean, "word/_rels/document.xml.rels").unwrap();
    let rel_at = rels.find(&format!("Id=\"{rid}\"")).expect("relationship");
    let rel = &rels[rels[..rel_at].rfind('<').unwrap()..rel_at + rels[rel_at..].find('>').unwrap()];
    assert!(rel.contains("/image\""), "{rel}");
    let target = attr(&format!("{rel}>"), "Relationship", "Target").to_string();
    let media = std::io::Read::bytes(
        zip::ZipArchive::new(Cursor::new(&result.clean))
            .unwrap()
            .by_name(&format!("word/{target}"))
            .unwrap(),
    )
    .collect::<Result<Vec<u8>, _>>()
    .unwrap();
    assert_eq!(media, image);
    assert!(target.ends_with(".png"), "{target}");
    let types = part_string(&result.clean, "[Content_Types].xml").unwrap();
    assert!(
        types.contains(r#"Extension="png""#) && types.contains("image/png"),
        "{types}"
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    assert_eq!(summary(&result.redline).unwrap().images, 1);
    let changes = list_changes(&result.redline).unwrap();
    assert!(
        changes.iter().any(|c| c.kind == ChangeKind::Insertion),
        "{changes:?}"
    );
    let redline_body = part_string(&result.redline, "word/document.xml").unwrap();
    let ins = redline_body.find("<w:ins ").expect("insertion");
    let ins_end = ins + redline_body[ins..].find("</w:ins>").unwrap();
    assert!(
        redline_body[ins..ins_end].contains("<w:drawing"),
        "{redline_body}"
    );
    assert_eq!(
        summary(&accept_revisions(&result.redline).unwrap())
            .unwrap()
            .images,
        1
    );
    let rejected = reject_revisions(&result.redline).unwrap();
    assert!(
        !part_string(&rejected, "word/document.xml")
            .unwrap()
            .contains("<w:drawing")
    );
    assert_eq!(texts(&rejected), texts(&source));
    let op = &result.report.operations[0];
    assert_eq!(
        (op.kind.as_str(), op.status.as_str()),
        ("insert_image", "ok")
    );
}

#[test]
fn insert_image_sizes_from_pixels_and_caps_the_width() {
    let source = docx(&para("Anchor."));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            &format!(
                r#"[{{"kind":"insert_image","paragraph":{{"index":0}},"position":"before","image_base64":"{}"}},
                    {{"kind":"insert_image","paragraph":{{"index":0}},"image_base64":"{}","alt":"Wide"}}]"#,
                encoded(&gif(40, 30)),
                encoded(&png(1000, 100))
            ),
        ),
    )
    .unwrap();
    let all = paragraphs(&result.clean).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[1].text, "Anchor.");
    let body = part_string(&result.clean, "word/document.xml").unwrap();
    let extents: Vec<(&str, &str)> = body
        .match_indices("<wp:extent ")
        .map(|(at, _)| {
            (
                attr(&body[at..], "wp:extent", "cx"),
                attr(&body[at..], "wp:extent", "cy"),
            )
        })
        .collect();
    // 40x30 px at 9525 EMU per pixel; 1000x100 px capped at 6.5in.
    assert_eq!(extents, [("381000", "285750"), ("5943600", "594360")]);
    let ids: Vec<&str> = body
        .match_indices("<wp:docPr ")
        .map(|(at, _)| attr(&body[at..], "wp:docPr", "id"))
        .collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
    let types = part_string(&result.clean, "[Content_Types].xml").unwrap();
    assert!(
        types.contains("image/gif") && types.contains("image/png"),
        "{types}"
    );
    assert_eq!(summary(&result.clean).unwrap().images, 2);
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn insert_image_refusals() {
    let source = docx(&para("Anchor."));
    let png = encoded(&png(4, 4));
    let not_an_image = encoded(b"this is plain text, not a picture");
    for (op, code) in [
        (
            format!(r#""image_base64":"{not_an_image}""#),
            "UNSUPPORTED_IMAGE",
        ),
        (
            format!(r#""image_base64":"{png}","content_type":"image/webp""#),
            "UNSUPPORTED_IMAGE",
        ),
        (
            format!(r#""image_base64":"{png}","content_type":"image/jpeg""#),
            "INVALID_EDIT",
        ),
        (
            r#""image_base64":"not base64!""#.to_string(),
            "INVALID_EDIT",
        ),
        (r#""image_base64":"""#.to_string(), "INVALID_EDIT"),
        (
            format!(r#""image_base64":"{png}","width_emu":0"#),
            "INVALID_EDIT",
        ),
        (
            format!(r#""image_base64":"{png}","width_emu":99999999"#),
            "INVALID_EDIT",
        ),
        (
            format!(r#""image_base64":"{png}","alt":"a\u0001b""#),
            "INVALID_EDIT",
        ),
    ] {
        let ops = format!(r#"[{{"kind":"insert_image","paragraph":{{"index":0}},{op}}}]"#);
        let err = apply_plan(&source, &plan(&source, &ops)).unwrap_err();
        assert_eq!(err.code, code, "{op}");
    }
    let json = r#"{"schema_version":1,"author":"a","operations":[{"kind":"insert_image","paragraph":{"index":0},"image_base64":"AAAA","height_emu":5}]}"#;
    assert_eq!(EditPlan::from_json(json).unwrap_err().code, "INVALID_PLAN");
}
