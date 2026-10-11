// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Package ordering and single-relationship carryover, without the document comparer.

use std::io::{Cursor, Write};

use jubarte::comparer::parts::carry_relationship;
use jubarte::opc::PartFs;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const HYPERLINK: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";

fn package() -> PartFs {
    let mut z = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, text) in [
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
        ),
        ("word/z.xml", "<z/>"),
        ("word/a.xml", "<a/>"),
    ] {
        z.start_file(name, SimpleFileOptions::default()).unwrap();
        z.write_all(text.as_bytes()).unwrap();
    }
    PartFs::open(&z.finish().unwrap().into_inner()).unwrap()
}

fn names(pkg: &PartFs) -> Vec<String> {
    let zip = ZipArchive::new(Cursor::new(pkg.to_zip().unwrap())).unwrap();
    (0..zip.len())
        .map(|i| zip.name_for_index(i).unwrap().to_string())
        .collect()
}

#[test]
fn new_relationships_sort_with_new_parts_and_survive_reopening() {
    let mut pkg = package();
    pkg.set_part("word/b.xml", b"<b/>".to_vec());
    pkg.set_part("word/z.xml", b"<updated/>".to_vec());
    pkg.add_document_relationship("word/b.xml", IMAGE, "media/a.png");
    pkg.add_document_relationship("word/z.xml", IMAGE, "media/b.png");
    let expected = [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/z.xml",
        "word/a.xml",
        "word/_rels/b.xml.rels",
        "word/_rels/z.xml.rels",
        "word/b.xml",
    ];
    assert_eq!(names(&pkg), expected);
    let bytes = pkg.to_zip().unwrap();
    let reopened = PartFs::open(&bytes).unwrap();
    assert_eq!(names(&reopened), expected);
    assert_eq!(
        reopened.part_bytes("word/z.xml"),
        Some(b"<updated/>".as_slice())
    );
    assert_eq!(pkg.to_zip().unwrap(), bytes);
}

#[test]
fn mixed_internal_and_external_relationships_keep_unique_ids_after_reopening() {
    let mut pkg = package();
    assert_eq!(
        pkg.add_document_relationship_external(
            "word/a.xml",
            HYPERLINK,
            "https://example.test/?a=1&b=2"
        ),
        "rId1"
    );
    assert_eq!(
        pkg.add_document_relationship("/word/a.xml", IMAGE, "media/one.png"),
        "rId2"
    );
    let mut reopened = PartFs::open(&pkg.to_zip().unwrap()).unwrap();
    assert_eq!(
        reopened.add_document_relationship("word/a.xml", IMAGE, "media/two.png"),
        "rId3"
    );
    let rels = reopened.read_rels_for("word/a.xml").unwrap();
    assert_eq!(rels.items.len(), 3);
    let link = rels.items.iter().find(|r| r.id == "rId1").unwrap();
    assert_eq!(link.target_mode.as_deref(), Some("External"));
    assert_eq!(link.target, "https://example.test/?a=1&b=2");
    assert!(
        rels.items
            .iter()
            .filter(|r| r.id != "rId1")
            .all(|r| r.target_mode.is_none())
    );
}

#[test]
fn carrying_an_image_preserves_collision_bytes_and_resolves_from_a_nested_part() {
    let mut src = package();
    src.set_part("word/media/bullet.png", b"revised-image".to_vec());
    src.add_content_type_default("png", "image/png");
    let rid = src.add_document_relationship("word/numbering.xml", IMAGE, "media/bullet.png");
    let mut dest = package();
    dest.set_part("word/media/bullet.png", b"original-image".to_vec());
    let carried = carry_relationship(
        &mut dest,
        "word/charts/chart1.xml",
        &src,
        "word/numbering.xml",
        &rid,
        |ty| ty == IMAGE,
    )
    .unwrap();
    let rel = dest
        .read_rels_for("word/charts/chart1.xml")
        .unwrap()
        .items
        .iter()
        .find(|r| r.id == carried)
        .unwrap();
    assert_eq!(rel.rel_type, IMAGE);
    assert!(rel.target_mode.is_none());
    let copied = dest.resolve_rel_target("word/charts/chart1.xml", &rel.target);
    assert_ne!(copied, "word/media/bullet.png");
    assert_eq!(dest.part_bytes(&copied), Some(b"revised-image".as_slice()));
    assert_eq!(dest.content_type_for(&copied).as_deref(), Some("image/png"));
    assert_eq!(
        dest.part_bytes("word/media/bullet.png"),
        Some(b"original-image".as_slice())
    );
}

#[test]
fn external_targets_are_copied_verbatim_without_creating_a_part() {
    let mut src = package();
    let target = "https://example.test/a?q=1&lang=pt#section";
    let rid = src.add_document_relationship_external("word/a.xml", HYPERLINK, target);
    let mut dest = package();
    let before = dest.parts();
    let id = carry_relationship(&mut dest, "word/z.xml", &src, "word/a.xml", &rid, |ty| {
        ty == HYPERLINK
    })
    .unwrap();
    assert_eq!(id, "rId1");
    assert_eq!(dest.parts(), before);
    let reopened = PartFs::open(&dest.to_zip().unwrap()).unwrap();
    let row = &reopened.read_rels_for("word/z.xml").unwrap().items[0];
    assert_eq!(row.target, target);
    assert_eq!(row.target_mode.as_deref(), Some("External"));
}

#[test]
fn missing_relationship_wrong_type_and_missing_target_leave_destination_untouched() {
    let mut src = package();
    let rid = src.add_document_relationship("word/a.xml", IMAGE, "media/missing.png");
    for (part, id, wanted) in [
        ("word/z.xml", rid.as_str(), IMAGE),
        ("word/a.xml", "rId999", IMAGE),
        ("word/a.xml", rid.as_str(), HYPERLINK),
        ("word/a.xml", rid.as_str(), IMAGE),
    ] {
        let mut dest = package();
        let before = dest.to_zip().unwrap();
        assert_eq!(
            carry_relationship(&mut dest, "word/z.xml", &src, part, id, |ty| ty == wanted),
            None
        );
        assert_eq!(dest.to_zip().unwrap(), before, "{part}: {id}: {wanted}");
        assert!(dest.read_rels_for("word/z.xml").is_none());
    }
}

#[test]
fn carrying_images_outside_word_media_deduplicates_bytes_without_overwriting_collisions() {
    use image::ImageEncoder as _;
    let mut owned = Vec::new();
    image::codecs::png::PngEncoder::new(&mut owned)
        .write_image(&[17, 31, 47], 1, 1, image::ExtendedColorType::Rgb8)
        .unwrap();
    let mut collision = Vec::new();
    image::codecs::png::PngEncoder::new(&mut collision)
        .write_image(&[61, 79, 97], 1, 1, image::ExtendedColorType::Rgb8)
        .unwrap();
    let digest = jubarte::inspect::source_sha256(&owned);
    let canonical = format!("word/media/P{digest}.png");
    for source_part in ["story.xml", "word/a.xml", "word/stories/a.xml"] {
        for target in [
            "picture.PNG",
            "assets/picture.png",
            "word/assets/picture.PnG",
        ] {
            for collide in [false, true] {
                let mut src = package();
                src.set_part(source_part, b"<source/>".to_vec());
                src.set_part(target, owned.clone());
                src.add_content_type_override(target, "image/png");
                let relative = jubarte::opc::relative_rel_target(source_part, target);
                let rid = src.add_document_relationship(source_part, IMAGE, &relative);
                let source_before = src.to_zip().unwrap();
                let mut dest = package();
                if collide {
                    dest.set_part(&canonical, collision.clone());
                }
                let destination_part = "word/stories/destination.xml";
                dest.set_part(destination_part, b"<destination/>".to_vec());
                let carried = carry_relationship(
                    &mut dest,
                    destination_part,
                    &src,
                    source_part,
                    &rid,
                    |kind| kind == IMAGE,
                )
                .unwrap();
                let resolved = if collide {
                    format!("word/media/P{digest}_1.png")
                } else {
                    canonical.clone()
                };
                let rels = dest.read_rels_for(destination_part).unwrap();
                let row = rels.items.iter().find(|row| row.id == carried).unwrap();
                assert_eq!(row.rel_type, IMAGE);
                assert_eq!(row.target_mode, None);
                assert_eq!(row.target, format!("/{resolved}"));
                assert_eq!(
                    dest.resolve_rel_target(destination_part, &row.target),
                    resolved
                );
                assert_eq!(dest.part_bytes(&resolved), Some(owned.as_slice()));
                assert_eq!(
                    dest.content_type_for(&resolved).as_deref(),
                    Some("image/png")
                );
                if collide {
                    assert_eq!(dest.part_bytes(&canonical), Some(collision.as_slice()));
                }
                let parts_before_second = dest.parts();
                let second = carry_relationship(
                    &mut dest,
                    destination_part,
                    &src,
                    source_part,
                    &rid,
                    |kind| kind == IMAGE,
                )
                .unwrap();
                assert_eq!(second, carried);
                assert_eq!(dest.parts(), parts_before_second);
                assert_eq!(src.to_zip().unwrap(), source_before);
                let reopened = PartFs::open(&dest.to_zip().unwrap()).unwrap();
                assert_eq!(reopened.part_bytes(&resolved), Some(owned.as_slice()));
                assert_eq!(
                    reopened.part_bytes(destination_part),
                    Some(b"<destination/>".as_slice())
                );
            }
        }
    }
}
