// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2023 Ismael Shakverdiev
//
// SPDX-License-Identifier: AGPL-3.0-only

//! OpenType MATH constants and vertical glyph stretch.
//!
//! Values are font design units. `fallback` supplies the TeX stand-ins
//! when a face has no MATH table. The stretch and constant reading follow
//! markdown2pdf's MATH reader (MIT, Ismael Shakverdiev, 2023).

/// The MATH constants this step reads. Font units, except the percents,
/// which are fractions of the base size (0.7 = 70%).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MathConstants {
    pub script_percent: f32,
    pub script_script_percent: f32,
    pub axis_height: f32,
    pub fraction_rule_thickness: f32,
    pub fraction_num_shift_up: f32,
    pub fraction_num_display_shift_up: f32,
    pub fraction_denom_shift_down: f32,
    pub fraction_denom_display_shift_down: f32,
    pub fraction_num_gap_min: f32,
    pub fraction_num_display_gap_min: f32,
    pub fraction_denom_gap_min: f32,
    pub fraction_denom_display_gap_min: f32,
    pub radical_vertical_gap: f32,
    pub radical_display_vertical_gap: f32,
    pub radical_rule_thickness: f32,
    pub radical_extra_ascender: f32,
    pub radical_kern_before_degree: f32,
    pub radical_kern_after_degree: f32,
    pub radical_degree_bottom_raise_percent: f32,
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
///
/// The face is parsed on each call. `ttf_parser::Face` borrows the
/// bytes, so the struct keeps the bytes and not the face.
pub(crate) struct MathFont {
    bytes: Vec<u8>,
    index: u32,
    upem: f32,
    has_math: bool,
    c: MathConstants,
}

impl MathFont {
    /// Face 0 of a standalone font. A face with no MATH table still
    /// returns `Some`: constants come from [`Self::fallback`].
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::from_collection(bytes, 0)
    }

    /// One face of a collection. Same contract as [`Self::from_bytes`].
    pub(crate) fn from_collection(bytes: &[u8], index: u32) -> Option<Self> {
        let face = ttf_parser::Face::parse(bytes, index).ok()?;
        let upem = f32::from(face.units_per_em());
        let (has_math, c) = match face.tables().math.and_then(|m| m.constants) {
            Some(k) => (true, read_constants(&k)),
            None => (false, fallback_constants(upem)),
        };
        Some(Self {
            bytes: bytes.to_vec(),
            index,
            upem,
            has_math,
            c,
        })
    }

    /// TeXbook Appendix G stand-ins, in font units of `upem`.
    /// `fallback(1000)` is axis 250 and rule thickness 40.
    pub(crate) fn fallback(upem: f32) -> Self {
        Self {
            bytes: Vec::new(),
            index: 0,
            upem,
            has_math: false,
            c: fallback_constants(upem),
        }
    }

    #[cfg(test)]
    pub(crate) fn has_math_table(&self) -> bool {
        self.has_math
    }

    pub(crate) fn units_per_em(&self) -> f32 {
        self.upem
    }

    pub(crate) fn constants(&self) -> MathConstants {
        self.c
    }

    /// Font units to points at `size_pt`.
    pub(crate) fn scale(&self, units: f32, size_pt: f32) -> f32 {
        if self.upem == 0.0 {
            return 0.0;
        }
        units * size_pt / self.upem
    }

    /// The name table's family. Tests check Cambria Math is the face
    /// they opened. Layout does not branch on the name.
    #[cfg(test)]
    pub(crate) fn family_name(&self) -> String {
        self.with_face(family_name).unwrap_or_default()
    }

    pub(crate) fn glyph_id(&self, ch: char) -> Option<u16> {
        self.with_face(|face| face.glyph_index(ch).map(|g| g.0))
            .flatten()
    }

    pub(crate) fn glyph(&self, gid: u16) -> Glyph {
        self.with_face(|face| glyph_metrics(face, gid))
            .unwrap_or(Glyph {
                advance: 0.0,
                y_min: 0.0,
                y_max: 0.0,
            })
    }

    /// A vertical form of `base` at least `target` font units tall
    /// (height + depth). Variants first, then an assembly, then the
    /// tallest variant, then `base` itself.
    pub(crate) fn stretch_vertical(&self, base: u16, target: f32) -> Stretch {
        // No MATH table means no variants and no assembly.
        if !self.has_math {
            return Stretch::Single(base);
        }
        self.with_face(|face| stretch_vertical(face, base, target))
            .unwrap_or(Stretch::Single(base))
    }

    fn with_face<T>(&self, f: impl FnOnce(&ttf_parser::Face<'_>) -> T) -> Option<T> {
        let face = ttf_parser::Face::parse(&self.bytes, self.index).ok()?;
        Some(f(&face))
    }
}

thread_local! {
    static CURRENT_MATH_FONT: std::cell::RefCell<Option<MathFont>> = const { std::cell::RefCell::new(None) };
}

/// Installs the conversion's math font, restoring the previous one on drop.
pub(crate) struct MathFontGuard(Option<MathFont>);

impl MathFontGuard {
    pub(crate) fn set(font: MathFont) -> Self {
        Self(CURRENT_MATH_FONT.with(|slot| slot.replace(Some(font))))
    }
}

impl Drop for MathFontGuard {
    fn drop(&mut self) {
        let previous = self.0.take();
        CURRENT_MATH_FONT.with(|slot| {
            slot.replace(previous);
        });
    }
}

/// The font installed for this conversion, or the TeX fallback.
pub(crate) fn with_current_math_font<T>(f: impl FnOnce(&MathFont) -> T) -> T {
    CURRENT_MATH_FONT.with(|slot| {
        let borrowed = slot.borrow();
        if let Some(font) = borrowed.as_ref() {
            f(font)
        } else {
            drop(borrowed);
            f(&MathFont::fallback(1000.0))
        }
    })
}

/// TeXbook Appendix G / plain TeX at 10pt, scaled into `upem`.
/// The cmsy axis is 2.5pt (0.25 em) and the cmex rule is 0.4pt (0.04 em).
/// Used only when the face has no MATH table.
fn fallback_constants(upem: f32) -> MathConstants {
    let u = |em: f32| em * upem;
    MathConstants {
        script_percent: 0.70,
        script_script_percent: 0.50,
        axis_height: u(0.25),
        fraction_rule_thickness: u(0.04),
        fraction_num_shift_up: u(0.394),
        fraction_num_display_shift_up: u(0.676),
        fraction_denom_shift_down: u(0.345),
        fraction_denom_display_shift_down: u(0.686),
        fraction_num_gap_min: u(0.10),
        fraction_num_display_gap_min: u(0.30),
        fraction_denom_gap_min: u(0.10),
        fraction_denom_display_gap_min: u(0.30),
        radical_vertical_gap: u(0.05),
        radical_display_vertical_gap: u(0.14),
        radical_rule_thickness: u(0.04),
        radical_extra_ascender: u(0.04),
        radical_kern_before_degree: u(0.05),
        radical_kern_after_degree: u(-0.10),
        radical_degree_bottom_raise_percent: 0.60,
    }
}

fn read_constants(k: &ttf_parser::math::Constants<'_>) -> MathConstants {
    let v = |m: ttf_parser::math::MathValue<'_>| f32::from(m.value);
    MathConstants {
        script_percent: f32::from(k.script_percent_scale_down()) / 100.0,
        script_script_percent: f32::from(k.script_script_percent_scale_down()) / 100.0,
        axis_height: v(k.axis_height()),
        fraction_rule_thickness: v(k.fraction_rule_thickness()),
        fraction_num_shift_up: v(k.fraction_numerator_shift_up()),
        fraction_num_display_shift_up: v(k.fraction_numerator_display_style_shift_up()),
        fraction_denom_shift_down: v(k.fraction_denominator_shift_down()),
        fraction_denom_display_shift_down: v(k.fraction_denominator_display_style_shift_down()),
        fraction_num_gap_min: v(k.fraction_numerator_gap_min()),
        fraction_num_display_gap_min: v(k.fraction_num_display_style_gap_min()),
        fraction_denom_gap_min: v(k.fraction_denominator_gap_min()),
        fraction_denom_display_gap_min: v(k.fraction_denom_display_style_gap_min()),
        radical_vertical_gap: v(k.radical_vertical_gap()),
        radical_display_vertical_gap: v(k.radical_display_style_vertical_gap()),
        radical_rule_thickness: v(k.radical_rule_thickness()),
        radical_extra_ascender: v(k.radical_extra_ascender()),
        radical_kern_before_degree: v(k.radical_kern_before_degree()),
        radical_kern_after_degree: v(k.radical_kern_after_degree()),
        radical_degree_bottom_raise_percent: f32::from(k.radical_degree_bottom_raise_percent())
            / 100.0,
    }
}

#[cfg(test)]
fn family_name(face: &ttf_parser::Face<'_>) -> String {
    face.names()
        .into_iter()
        .filter(|n| n.name_id == ttf_parser::name_id::FAMILY)
        .filter_map(|n| n.to_string())
        .find(|n| !n.is_empty())
        .unwrap_or_default()
}

fn glyph_metrics(face: &ttf_parser::Face<'_>, gid: u16) -> Glyph {
    let g = ttf_parser::GlyphId(gid);
    let advance = f32::from(face.glyph_hor_advance(g).unwrap_or(0));
    match face.glyph_bounding_box(g) {
        Some(r) => Glyph {
            advance,
            y_min: f32::from(r.y_min),
            y_max: f32::from(r.y_max),
        },
        None => Glyph {
            advance,
            y_min: 0.0,
            y_max: 0.0,
        },
    }
}

/// Variants until one is at least `target` tall, else the assembly,
/// else the tallest variant. Parts stay in MATH order (bottom to top).
fn stretch_vertical(face: &ttf_parser::Face<'_>, base: u16, target: f32) -> Stretch {
    let Some(variants) = face.tables().math.and_then(|m| m.variants) else {
        return Stretch::Single(base);
    };
    let Some(con) = variants
        .vertical_constructions
        .get(ttf_parser::GlyphId(base))
    else {
        return Stretch::Single(base);
    };
    for var in con.variants {
        if f32::from(var.advance_measurement) >= target {
            return Stretch::Single(var.variant_glyph.0);
        }
    }
    if let Some(asm) = con.assembly {
        let parts: Vec<AssemblyPart> = asm
            .parts
            .into_iter()
            .map(|p| AssemblyPart {
                gid: p.glyph_id.0,
                full_advance: f32::from(p.full_advance),
                extender: p.part_flags.extender(),
            })
            .collect();
        if !parts.is_empty() {
            return Stretch::Assembly {
                parts,
                overlap: f32::from(variants.min_connector_overlap),
            };
        }
    }
    if let Some(last) = con.variants.last() {
        return Stretch::Single(last.variant_glyph.0);
    }
    Stretch::Single(base)
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
