// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn raster_formats_preserve_dimensions_and_rgb_samples() {
    let pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    for format in [
        image::ImageFormat::Bmp,
        image::ImageFormat::Gif,
        image::ImageFormat::Tiff,
    ] {
        let img = image::RgbImage::from_raw(2, 2, pixels.clone()).unwrap();
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut encoded, format)
            .unwrap();
        let Some(ImageKind::Rgb {
            width,
            height,
            bytes,
            alpha,
        }) = decode_image(encoded.into_inner())
        else {
            panic!("{format:?} must decode as RGB");
        };
        assert_eq!((width, height), (2, 2), "{format:?}");
        assert_eq!(bytes, pixels, "{format:?}");
        assert!(alpha.is_none_or(|a| a == [255; 4]), "{format:?}");
    }
}

#[test]
fn transparent_gif_preserves_the_alpha_mask() {
    let img = image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 0, 0]).unwrap();
    let mut encoded = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut encoded, image::ImageFormat::Gif)
        .unwrap();
    let Some(ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha,
    }) = decode_image(encoded.into_inner())
    else {
        panic!("GIF must decode as RGB with alpha");
    };
    assert_eq!((width, height), (2, 1));
    assert_eq!(&bytes[..3], &[255, 0, 0]);
    assert_eq!(alpha, Some(vec![255, 0]));
}

#[test]
fn truncated_raster_headers_do_not_decode_as_images() {
    for bytes in [b"".as_slice(), b"BM", b"GIF89a", b"II*\0", b"not an image"] {
        assert!(decode_image(bytes.to_vec()).is_none(), "{bytes:?}");
    }
}

#[test]
fn washout_clamps_highlights_and_preserves_transparency() {
    let image = ImageKind::Rgb {
        width: 2,
        height: 1,
        bytes: vec![0, 100, 255, 50, 200, 128],
        alpha: Some(vec![0, 128]),
    };
    let ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha,
    } = washed_out(image)
    else {
        panic!("washed image remains RGB");
    };
    assert_eq!((width, height), (2, 1));
    assert_eq!(bytes, [207, 236, 255, 221, 255, 244]);
    assert_eq!(alpha, Some(vec![0, 128]));
}

#[test]
fn soft_edges_multiply_existing_alpha_without_changing_color() {
    let image = ImageKind::Rgb {
        width: 5,
        height: 5,
        bytes: vec![42; 75],
        alpha: Some(vec![128; 25]),
    };
    let ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha,
    } = soften_edges(image, 0.4, 0.4)
    else {
        panic!("softened image remains RGB");
    };
    assert_eq!((width, height), (5, 5));
    assert_eq!(bytes, vec![42; 75]);
    let alpha = alpha.unwrap();
    assert_eq!(
        alpha,
        vec![
            32, 32, 32, 32, 32, 32, 96, 96, 96, 32, 32, 96, 128, 96, 32, 32, 96, 96, 96, 32, 32,
            32, 32, 32, 32
        ]
    );
}

#[test]
fn soft_edges_create_a_mask_for_an_opaque_single_pixel() {
    let image = ImageKind::Rgb {
        width: 1,
        height: 1,
        bytes: vec![1, 2, 3],
        alpha: None,
    };
    let ImageKind::Rgb { bytes, alpha, .. } = soften_edges(image, 0.1, 0.1) else {
        panic!("softened image remains RGB");
    };
    assert_eq!(bytes, [1, 2, 3]);
    assert_eq!(alpha, Some(vec![128]));
}

#[test]
fn image_effects_keep_placeholder_kinds() {
    assert!(matches!(washed_out(ImageKind::Reserve), ImageKind::Reserve));
    assert!(matches!(washed_out(ImageKind::Broken), ImageKind::Broken));
    assert!(matches!(
        soften_edges(ImageKind::Reserve, 0.1, 0.1),
        ImageKind::Reserve
    ));
    assert!(matches!(
        soften_edges(ImageKind::Broken, 0.1, 0.1),
        ImageKind::Broken
    ));
}

fn paragraph_with_breaks(text: &str) -> Block {
    let defaults = Defaults::word();
    let mut style = defaults.para.clone();
    style.before = 12.0;
    style.after = 18.0;
    style.before_auto = true;
    style.after_auto = true;
    Block::Paragraph {
        runs: vec![TextRun::new(text, defaults.run)],
        style,
        list: true,
        images: Vec::new(),
        boxes: Vec::new(),
        bookmarks: vec!["bookmark".into()],
    }
}

#[test]
fn page_and_column_breaks_split_unicode_text_and_keep_paragraph_metadata() {
    let text = format!("Č前{PAGE_BREAK_MARK}中{COLUMN_BREAK_MARK}後");
    let (pieces, separators, trailing) =
        split_page_breaks(paragraph_with_breaks(&text), true, true, false);
    assert_eq!(separators, [false, true]);
    assert_eq!(trailing, None);
    assert_eq!(pieces.len(), 3);
    for (i, (piece, expected)) in pieces.iter().zip(["Č前", "中", "後"]).enumerate() {
        let Block::Paragraph {
            runs,
            style,
            list,
            bookmarks,
            ..
        } = piece
        else {
            panic!("paragraph piece");
        };
        assert_eq!(
            runs.iter().map(|r| r.text.as_str()).collect::<String>(),
            expected
        );
        assert_eq!(*list, i == 0);
        assert_eq!(bookmarks.len(), usize::from(i == 0));
        assert_eq!(style.before, if i == 0 { 12.0 } else { 0.0 });
        assert_eq!(style.after, if i == 2 { 18.0 } else { 0.0 });
        assert_eq!(style.before_auto, i == 0);
        assert_eq!(style.after_auto, i == 2);
    }
}

#[test]
fn trailing_breaks_do_not_create_empty_paragraphs() {
    for (mark, column) in [(PAGE_BREAK_MARK, false), (COLUMN_BREAK_MARK, true)] {
        let text = format!("Text{mark}   ");
        let (pieces, separators, trailing) =
            split_page_breaks(paragraph_with_breaks(&text), !column, column, false);
        assert_eq!(pieces.len(), 1);
        assert!(separators.is_empty());
        assert_eq!(trailing, Some(column));
    }
}

#[test]
fn a_leading_page_break_keeps_the_following_text() {
    let text = format!("{PAGE_BREAK_MARK}Next page");
    let (pieces, separators, trailing) =
        split_page_breaks(paragraph_with_breaks(&text), true, false, false);
    assert_eq!(pieces.len(), 2);
    assert_eq!(separators, [false]);
    assert_eq!(trailing, None);
    let Block::Paragraph { runs, .. } = &pieces[1] else {
        panic!("paragraph")
    };
    assert_eq!(runs[0].text, "Next page");
}

#[test]
fn failed_conversion_restores_the_previous_revision_style() {
    let previous = REVISIONS.with(|r| r.get());
    let result = docx_to_pdf_with(
        b"not a docx",
        PdfOptions {
            revisions: RevisionStyle::Word,
            ..PdfOptions::default()
        },
    );
    assert!(matches!(result, Err(ConvertError::OpenPackage(_))));
    assert_eq!(REVISIONS.with(|r| r.get()), previous);
}

fn percent_table(pref: Vec<PrefWidth>) -> TableGeom {
    TableGeom {
        row_min: Vec::new(),
        bottom_above: Vec::new(),
        row_exact: Vec::new(),
        row_cant_split: Vec::new(),
        pad_v: 0.0,
        width: TblWidth::Pct(1.0),
        unstyled: true,
        header_rows: 0,
        table_grid: false,
        tbl_ind: 0.0,
        mar_l: 0.0,
        pref,
        rules: [0.0; 3],
        grid_padded: false,
        rows_past_grid: false,
        fixed: true,
        float: None,
        keep_at_margin: false,
        rtl: false,
        pct_margins: 0.0,
        pct_rules: 0.0,
        content_autofit: false,
        cell_spacing: 0.0,
        style_id: None,
    }
}

#[test]
fn legacy_percent_tables_include_cell_margins_in_the_target_width() {
    let mut geom = percent_table(vec![PrefWidth::Pct(0.5), PrefWidth::Pct(0.5)]);
    assert_eq!(
        table_col_widths(&[50.0, 50.0], &geom, 200.0),
        [100.0, 100.0]
    );
    geom.pct_margins = 12.0;
    assert_eq!(
        table_col_widths(&[50.0, 50.0], &geom, 200.0),
        [106.0, 106.0]
    );
    geom.width = TblWidth::Pct(0.5);
    assert_eq!(table_col_widths(&[50.0, 50.0], &geom, 200.0), [53.0, 53.0]);
}

#[test]
fn percent_overflow_is_removed_from_the_last_columns_first() {
    let geom = percent_table(vec![PrefWidth::Pct(0.6), PrefWidth::Pct(0.6)]);
    let widths = table_col_widths(&[50.0, 50.0], &geom, 100.0);
    assert!((widths[0] - 60.0).abs() < 0.001);
    assert!((widths[1] - 40.0).abs() < 0.001);
    let geom = percent_table(vec![
        PrefWidth::Pct(0.99),
        PrefWidth::Pct(0.5),
        PrefWidth::Pct(0.5),
    ]);
    let widths = table_col_widths(&[33.0; 3], &geom, 100.0);
    for (actual, expected) in widths.iter().zip([98.0, 1.0, 1.0]) {
        assert!((actual - expected).abs() < 0.001, "{widths:?}");
    }
}

#[test]
fn an_autofit_percent_table_keeps_its_already_resolved_grid() {
    let mut geom = percent_table(vec![PrefWidth::Pct(0.48), PrefWidth::Pct(0.52)]);
    geom.fixed = false;
    assert_eq!(table_col_widths(&[50.0, 50.0], &geom, 100.0), [50.0, 50.0]);
    geom.pref = vec![PrefWidth::Pct(0.4), PrefWidth::Pct(0.6)];
    let widths = table_col_widths(&[50.0, 50.0], &geom, 100.0);
    assert!((widths[0] - 40.0).abs() < 0.001);
    assert!((widths[1] - 60.0).abs() < 0.001);
}

#[test]
fn letters_past_z_repeat_the_letter_as_word_does() {
    // Word 16 probes la1006 (lowerLetter from 26, upperLetter from 52)
    // and nf1006 (SEQ \* alphabetic), 2026-10-06: z) aa) bb) cc),
    // ZZ) AAA) BBB); 702 is twenty-seven z's.
    let lower: Vec<String> = [1, 26, 27, 28, 29, 53]
        .map(|n| alpha_label(n, false))
        .into();
    assert_eq!(lower, ["a", "z", "aa", "bb", "cc", "aaa"]);
    let upper: Vec<String> = [52, 53, 54].map(|n| alpha_label(n, true)).into();
    assert_eq!(upper, ["ZZ", "AAA", "BBB"]);
    assert_eq!(alpha_label(702, false), "z".repeat(27));
}

#[test]
fn list_letters_and_roman_numerals_wrap_as_word_does() {
    // Word 16 probe bn1006b (2026-10-06): lowerLetter counts to thirty
    // z's at 780, then 781 is "a" again and 100000 "ddddddd";
    // upperRoman reaches 32767, then 32768 is "I" and 100000 "MDCXCIX".
    assert_eq!(alpha_label(780, false), "z".repeat(30));
    assert_eq!(alpha_label(781, false), "a");
    assert_eq!(alpha_label(1561, false), "a");
    assert_eq!(alpha_label(100_000, false), "ddddddd");
    assert!(alpha_label(u32::MAX, true).len() <= 30);
    assert!(roman_label(32767, true).ends_with("MMDCCLXVII"));
    assert_eq!(roman_label(32768, true), "I");
    assert_eq!(roman_label(100_000, true), "MDCXCIX");
    assert!(roman_label(u32::MAX, false).len() < 64);
}

fn with_revision_style(style: RevisionStyle, test: impl FnOnce()) {
    struct Restore(RevisionStyle);
    impl Drop for Restore {
        fn drop(&mut self) {
            REVISIONS.with(|r| r.set(self.0));
        }
    }
    let _restore = Restore(REVISIONS.with(|r| r.replace(style)));
    test();
}

#[test]
fn custom_marks_replace_existing_run_decorations_for_each_revision_kind() {
    let palette = RevisionPalette::parse("deleted=#102030:plain,inserted=#405060:strike,moved-from=#708090:double-strike:underline,moved-to=#a0b0c0:double-underline").unwrap();
    with_revision_style(RevisionStyle::Custom(palette), || {
        for (kind, color, lines) in [
            (
                RevMark::Del,
                [0x10_u8, 0x20, 0x30],
                [false, false, false, false],
            ),
            (
                RevMark::Ins,
                [0x40, 0x50, 0x60],
                [true, false, false, false],
            ),
            (
                RevMark::MoveFrom,
                [0x70, 0x80, 0x90],
                [true, true, true, false],
            ),
            (
                RevMark::MoveTo,
                [0xa0, 0xb0, 0xc0],
                [false, false, true, true],
            ),
        ] {
            let mut style = Defaults::word().run;
            style.strike = true;
            style.strike_double = true;
            style.underline = true;
            style.underline_double = true;
            style.underline_wave = true;
            apply_rev(&mut style, kind, [1.0, 0.0, 0.0]);
            assert_eq!(style.color, color.map(|c| f32::from(c) / 255.0));
            assert_eq!(
                [
                    style.strike,
                    style.strike_double,
                    style.underline,
                    style.underline_double
                ],
                lines
            );
            assert!(!style.underline_wave);
        }
    });
}

#[test]
fn word_marks_use_author_ink_for_insertions_and_deletions() {
    with_revision_style(RevisionStyle::Word, || {
        for kind in [RevMark::Ins, RevMark::MoveTo] {
            let mut style = Defaults::word().run;
            apply_rev(&mut style, kind, [0.1, 0.2, 0.3]);
            assert_eq!(style.color, [0.1, 0.2, 0.3]);
            assert!(style.underline && !style.underline_double && !style.strike);
        }
        // Live Word: a second author's deletion is 0078D4 beside the
        // first author's D13438, so a deletion takes its author's ink.
        for kind in [RevMark::Del, RevMark::MoveFrom] {
            let mut style = Defaults::word().run;
            apply_rev(&mut style, kind, [0.1, 0.2, 0.3]);
            assert_eq!(style.color, [0.1, 0.2, 0.3]);
            assert!(style.strike && !style.strike_double && !style.underline);
        }
    });
}

#[test]
fn unmarked_text_keeps_its_own_decorations_in_every_revision_mode() {
    for mode in [
        RevisionStyle::Conventional,
        RevisionStyle::Word,
        RevisionStyle::Custom(RevisionPalette::CONVENTIONAL),
    ] {
        with_revision_style(mode, || {
            let mut style = Defaults::word().run;
            style.color = [0.2, 0.4, 0.6];
            style.underline_wave = true;
            style.strike_double = true;
            apply_rev(&mut style, RevMark::None, [1.0, 0.0, 0.0]);
            assert_eq!(style.color, [0.2, 0.4, 0.6]);
            assert!(style.underline_wave && style.strike_double);
            assert!(!style.underline && !style.strike);
        });
    }
}
