// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
fn package() -> PartFs {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "[Content_Types].xml",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(
        br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
    )
    .unwrap();
    PartFs::open(&zip.finish().unwrap().into_inner()).unwrap()
}
fn duotone_dom() -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(r#"<a:blip xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:duotone><a:srgbClr val="FF0000"/><a:srgbClr val="00FF00"/></a:duotone></a:blip>"#);
    let root = dom.root(doc).unwrap();
    (dom, root)
}
fn header(components: u8) -> Vec<u8> {
    // Deliberately incomplete encoded payload: a readable SOF advertises
    // geometry, but no Huffman tables or image scan follows it.
    vec![0xFF, 0xD8, 0xFF, 0xC0, 0, 8, 8, 0, 3, 0, 5, components]
}
fn assert_reserved(kind: &ImageKind) {
    assert!(matches!(kind, ImageKind::Reserve));
}

#[test]
fn encoded_jpeg_header_retains_literal_geometry_even_when_pixel_scan_is_unreadable() {
    for components in [1, 3, 4] {
        let bytes = header(components);
        let unchanged = bytes.clone();
        assert_eq!(jpeg_info(&bytes), Some((5, 3, components)));
        let Some(ImageKind::Jpeg {
            width,
            height,
            bytes: retained,
            components: actual,
        }) = decode_image(bytes.clone())
        else {
            panic!("readable JPEG geometry retains the encoded source")
        };
        assert_eq!((width, height, actual), (5, 3, components));
        assert_eq!(retained, unchanged);
        assert_eq!(bytes, unchanged);
    }
}

#[test]
fn undecodable_jpeg_effects_reserve_the_picture_instead_of_fabricating_pixel_data() {
    let bytes = header(3);
    let (dom, blip) = duotone_dom();
    let before = dom.serialize_element(blip);
    let pkg = package();
    assert_reserved(&washed_out(decode_image(bytes.clone()).unwrap()));
    assert_reserved(&soften_edges(
        decode_image(bytes.clone()).unwrap(),
        0.25,
        0.25,
    ));
    assert_reserved(&duotone_image(
        decode_image(bytes.clone()).unwrap(),
        &dom,
        blip,
        &pkg,
    ));
    assert_eq!(bytes, header(3));
    assert_eq!(dom.serialize_element(blip), before);
}

#[test]
fn incomplete_or_wrong_jpeg_magic_never_manufactures_geometry_or_image_data() {
    for bytes in [
        vec![0xFF, 0, 0, 0],
        vec![0xFF, 0xD8, 0, 0],
        vec![0xFF, 0xD8, 0xFF, 0xC0, 0, 8, 8, 0, 3, 0, 5],
        vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 255, 0, 0, 0, 0, 0, 0],
    ] {
        let before = bytes.clone();
        assert!(decode_image(bytes.clone()).is_none());
        assert_eq!(jpeg_info(&bytes), None);
        assert_eq!(bytes, before);
    }
}

fn black_jpeg() -> Vec<u8> {
    let image =
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(3, 3, image::Rgb([0, 0, 0])));
    let mut output = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Jpeg)
        .unwrap();
    output.into_inner()
}
fn assert_rgb(kind: ImageKind, expected_bytes: &[u8], expected_alpha: Option<&[u8]>) {
    let ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha,
    } = kind
    else {
        panic!("successful pixel effect retains an RGB image")
    };
    assert_eq!((width, height), (3, 3));
    assert_eq!(bytes, expected_bytes);
    assert_eq!(alpha.as_deref(), expected_alpha);
}

#[test]
fn real_memory_jpeg_effects_have_complete_literal_pixels_and_alpha() {
    let bytes = black_jpeg();
    let before = bytes.clone();
    assert_eq!(jpeg_info(&bytes), Some((3, 3, 3)));
    assert_rgb(
        washed_out(decode_image(bytes.clone()).unwrap()),
        &[207; 27],
        None,
    );
    assert_rgb(
        soften_edges(decode_image(bytes.clone()).unwrap(), 0.4, 0.4),
        &[0; 27],
        Some(&[106, 106, 106, 106, 255, 106, 106, 106, 106]),
    );
    let (dom, blip) = duotone_dom();
    let xml_before = dom.serialize_element(blip);
    let pkg = package();
    let red = [255, 0, 0].repeat(9);
    assert_rgb(
        duotone_image(decode_image(bytes.clone()).unwrap(), &dom, blip, &pkg),
        &red,
        None,
    );
    assert_eq!(bytes, before);
    assert_eq!(dom.serialize_element(blip), xml_before);
}
