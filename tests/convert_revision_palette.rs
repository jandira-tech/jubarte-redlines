// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The revision option contract shared by the converter and its bindings.

use jubarte::convert::{MarkLines, PdfOptions, RevisionMark, RevisionPalette, RevisionStyle};

#[test]
fn default_options_use_the_documented_conventional_marks() {
    assert_eq!(PdfOptions::default().revisions, RevisionStyle::Conventional);
    let palette = RevisionPalette::CONVENTIONAL;
    for (actual, color, strike, underline) in [
        (
            palette.deleted,
            [255, 0, 0],
            MarkLines::Single,
            MarkLines::None,
        ),
        (
            palette.inserted,
            [0, 0, 255],
            MarkLines::None,
            MarkLines::Double,
        ),
        (
            palette.moved_from,
            [0, 128, 0],
            MarkLines::Single,
            MarkLines::None,
        ),
        (
            palette.moved_to,
            [0, 128, 0],
            MarkLines::None,
            MarkLines::Double,
        ),
    ] {
        assert_eq!(
            actual,
            RevisionMark {
                color,
                strike,
                underline
            }
        );
    }
}

#[test]
fn color_only_overrides_keep_lines_and_unspecified_kinds() {
    let palette = RevisionPalette::parse(" deleted = #aB09fF , inserted=#123456 ").unwrap();
    assert_eq!(
        palette,
        RevisionPalette {
            deleted: RevisionMark {
                color: [0xab, 0x09, 0xff],
                ..RevisionPalette::CONVENTIONAL.deleted
            },
            inserted: RevisionMark {
                color: [0x12, 0x34, 0x56],
                ..RevisionPalette::CONVENTIONAL.inserted
            },
            ..RevisionPalette::CONVENTIONAL
        }
    );
}

#[test]
fn explicit_lines_replace_the_conventional_lines() {
    for (line, strike, underline) in [
        ("plain", MarkLines::None, MarkLines::None),
        ("strike", MarkLines::Single, MarkLines::None),
        ("double-strike", MarkLines::Double, MarkLines::None),
        ("underline", MarkLines::None, MarkLines::Single),
        ("double-underline", MarkLines::None, MarkLines::Double),
        (
            "double-strike:underline",
            MarkLines::Double,
            MarkLines::Single,
        ),
    ] {
        for kind in ["deleted", "inserted", "moved-from", "moved-to"] {
            let spec = format!("{kind}=#123456:{line}");
            let palette = RevisionPalette::parse(&spec).unwrap();
            let mark = match kind {
                "deleted" => palette.deleted,
                "inserted" => palette.inserted,
                "moved-from" => palette.moved_from,
                _ => palette.moved_to,
            };
            assert_eq!(
                mark,
                RevisionMark {
                    color: [0x12, 0x34, 0x56],
                    strike,
                    underline
                },
                "{spec}"
            );
        }
    }
}

#[test]
fn aliases_address_the_same_revision_kinds() {
    for (alias, name) in [
        ("del", "deleted"),
        ("ins", "inserted"),
        ("move-from", "moved-from"),
        ("move-to", "moved-to"),
    ] {
        assert_eq!(
            RevisionPalette::parse(&format!("{alias}=#abcdef:plain")),
            RevisionPalette::parse(&format!("{name}=#abcdef:plain")),
            "{alias}"
        );
    }
}

#[test]
fn empty_entries_do_not_erase_other_marks() {
    assert_eq!(
        RevisionPalette::parse(" , , "),
        Ok(RevisionPalette::CONVENTIONAL)
    );
    assert_eq!(
        RevisionPalette::parse(",deleted=#000000:plain,,"),
        RevisionPalette::parse("deleted=#000000:plain")
    );
}

#[test]
fn invalid_palette_entries_return_actionable_errors() {
    for (spec, message) in [
        ("deleted", "expected kind=#RRGGBB"),
        ("unknown=#123456", "unknown revision kind"),
        ("=123456", "unknown revision kind"),
        ("deleted=", "colour must be #RRGGBB"),
        ("deleted=#12345", "colour must be #RRGGBB"),
        ("deleted=#1234567", "colour must be #RRGGBB"),
        ("deleted=#GG0000", "colour must be #RRGGBB"),
        ("deleted=#é0000", "colour must be #RRGGBB"),
        ("inserted=#123456:", "unknown line style"),
        ("inserted=#123456:zigzag", "unknown line style"),
        (
            "deleted=#000000:plain,inserted=red",
            "colour must be #RRGGBB",
        ),
    ] {
        let error = RevisionPalette::parse(spec).expect_err(spec);
        assert!(error.contains(message), "{spec}: {error}");
    }
}

#[test]
fn revision_choice_validates_mode_and_palette_together() {
    assert_eq!(
        RevisionStyle::from_choice("conventional", None),
        Ok(RevisionStyle::Conventional)
    );
    assert_eq!(
        RevisionStyle::from_choice("word", None),
        Ok(RevisionStyle::Word)
    );
    let spec = "moved-from=#010203:double-strike";
    assert_eq!(
        RevisionStyle::from_choice("custom", Some(spec)),
        Ok(RevisionStyle::Custom(RevisionPalette::parse(spec).unwrap()))
    );
    assert!(
        RevisionStyle::from_choice("custom", None)
            .unwrap_err()
            .contains("needs a revision palette")
    );
    for mode in ["conventional", "word"] {
        for palette in ["", spec] {
            assert!(
                RevisionStyle::from_choice(mode, Some(palette))
                    .unwrap_err()
                    .contains("needs revisions \"custom\"")
            );
        }
    }
    for mode in ["", "Word", " word ", "unknown"] {
        assert!(
            RevisionStyle::from_choice(mode, None)
                .unwrap_err()
                .contains("unknown revisions")
        );
    }
    assert_eq!(
        RevisionStyle::from_choice("custom", Some("deleted=red")),
        RevisionPalette::parse("deleted=red").map(RevisionStyle::Custom)
    );
}
