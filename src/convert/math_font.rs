// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! OpenType MATH constants and vertical glyph stretch.
//!
//! Values are font design units. `fallback` supplies the TeX stand-ins
//! when a face has no MATH table. This stub returns zeros so the tests
//! fail before the reader exists.

/// The MATH constants this step reads. Font units, except the two
/// percents, which are fractions of the base size (0.7 = 70%).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MathConstants {
    pub axis_height: f32,
    pub fraction_rule_thickness: f32,
    pub radical_vertical_gap: f32,
}

/// One glyph's design-space box. `y` grows up from the baseline.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Glyph {
    pub advance: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl Glyph {
    pub(crate) fn height(self) -> f32 {
        self.y_max.max(0.0)
    }

    pub(crate) fn depth(self) -> f32 {
        (-self.y_min).max(0.0)
    }
}

/// One piece of a vertical assembly. Parts stay in MATH order
/// (bottom to top).
#[derive(Debug, Clone, Copy)]
pub(crate) struct AssemblyPart {
    pub gid: u16,
    pub full_advance: f32,
    pub extender: bool,
}

/// A stretchy glyph, either one variant or a stacked assembly.
#[derive(Debug, Clone)]
pub(crate) enum Stretch {
    Single(u16),
    Assembly {
        parts: Vec<AssemblyPart>,
        overlap: f32,
    },
}

/// Font bytes plus the constants laid out against them.
pub(crate) struct MathFont {
    upem: f32,
    has_math: bool,
    c: MathConstants,
    family: String,
}

impl MathFont {
    /// Face 0 of a standalone font. A face with no MATH table still
    /// returns `Some`: constants come from [`Self::fallback`].
    pub(crate) fn from_bytes(_bytes: &[u8]) -> Option<Self> {
        Some(Self::zeros(1000.0))
    }

    /// One face of a collection. Same contract as [`Self::from_bytes`].
    pub(crate) fn from_collection(_bytes: &[u8], _index: u32) -> Option<Self> {
        Some(Self::zeros(1000.0))
    }

    /// TeXbook Appendix G stand-ins, in font units of `upem`.
    /// `fallback(1000)` is axis 250 and rule thickness 40.
    pub(crate) fn fallback(upem: f32) -> Self {
        let _ = upem;
        Self::zeros(upem)
    }

    fn zeros(upem: f32) -> Self {
        Self {
            upem,
            has_math: false,
            c: MathConstants {
                axis_height: 0.0,
                fraction_rule_thickness: 0.0,
                radical_vertical_gap: 0.0,
            },
            family: String::new(),
        }
    }

    pub(crate) fn has_math_table(&self) -> bool {
        self.has_math
    }

    pub(crate) fn units_per_em(&self) -> f32 {
        self.upem
    }

    pub(crate) fn constants(&self) -> MathConstants {
        self.c
    }

    pub(crate) fn family_name(&self) -> &str {
        &self.family
    }

    pub(crate) fn glyph_id(&self, _ch: char) -> Option<u16> {
        Some(1)
    }

    pub(crate) fn glyph(&self, _gid: u16) -> Glyph {
        Glyph {
            advance: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        }
    }

    /// A vertical form of `base` at least `target` font units tall
    /// (height + depth). The stub returns `base` unchanged.
    pub(crate) fn stretch_vertical(&self, base: u16, _target: f32) -> Stretch {
        Stretch::Single(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STIX: &[u8] = include_bytes!("../../tests/fixtures/fonts/STIXTwoMath.otf");

    fn stix() -> MathFont {
        MathFont::from_bytes(STIX).expect("STIX Two Math parses")
    }

    #[test]
    fn math_font_stix_reads_axis_rule_and_radical_gap() {
        let font = stix();
        assert!(
            font.has_math_table(),
            "STIX Two Math has a MATH table; the reader returned none"
        );
        let axis = font.constants().axis_height / font.units_per_em();
        assert!(
            (0.15..0.40).contains(&axis),
            "axis {axis} em is outside 0.15..0.40 (reader returned zeros)"
        );
        assert!(
            font.constants().fraction_rule_thickness > 0.0,
            "fraction rule thickness is 0"
        );
        assert!(
            font.constants().radical_vertical_gap > 0.0,
            "radical vertical gap is 0"
        );
    }

    fn stretched_is_taller(font: &MathFont, ch: char) {
        let gid = font
            .glyph_id(ch)
            .unwrap_or_else(|| panic!("{ch:?} has no glyph"));
        let own = font.glyph(gid);
        let h = own.height() + own.depth();
        let stretched = font.stretch_vertical(gid, h * 3.0);
        match stretched {
            Stretch::Single(variant) => {
                let g = font.glyph(variant);
                let sh = g.height() + g.depth();
                assert!(
                    sh > h,
                    "{ch:?} variant {variant} is {sh} tall, base is {h}; it did not grow"
                );
            }
            Stretch::Assembly { parts, .. } => {
                assert!(
                    parts.len() >= 2,
                    "{ch:?} assembly has {} parts",
                    parts.len()
                );
            }
        }
    }

    #[test]
    fn math_font_radical_sign_stretches_to_three_times_its_height() {
        stretched_is_taller(&stix(), '\u{221A}');
    }

    #[test]
    fn math_font_parenthesis_stretches_to_three_times_its_height() {
        stretched_is_taller(&stix(), '(');
    }

    #[test]
    fn math_font_fallback_uses_tex_constants_and_does_not_stretch() {
        let font = MathFont::fallback(1000.0);
        assert!(!font.has_math_table());
        assert_eq!(font.constants().axis_height, 250.0);
        assert_eq!(font.constants().fraction_rule_thickness, 40.0);
        match font.stretch_vertical(42, 10_000.0) {
            Stretch::Single(gid) => assert_eq!(gid, 42),
            Stretch::Assembly { .. } => panic!("a font with no MATH table cannot assemble"),
        }
    }

    #[test]
    fn math_font_cambria_math_is_face_one_when_word_is_installed() {
        let path = "/Applications/Microsoft Word.app/Contents/Resources/DFonts/Cambria.ttc";
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("skip: {path} is absent");
            return;
        };
        let font = MathFont::from_collection(&bytes, 1).expect("Cambria.ttc face 1 parses");
        assert!(
            font.has_math_table(),
            "Cambria Math (face 1) has a MATH table"
        );
        assert_eq!(font.family_name(), "Cambria Math");
    }
}
