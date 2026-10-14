// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! OMML bar fractions, radicals, and stretchy delimiters as a box.
//!
//! The stub returns an empty box so the geometry tests fail first.

use super::math_font::MathFont;

/// One positioned glyph. `y` grows up from the box baseline.
/// `text` is the searchable character; empty on an assembly extender.
#[derive(Debug, Clone)]
pub(crate) struct MathGlyph {
    pub text: String,
    pub gid: u16,
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

/// A horizontal rule. `y_top` is its top edge, up from the baseline.
#[derive(Debug, Clone)]
pub(crate) struct MathRule {
    pub x: f32,
    pub y_top: f32,
    pub w: f32,
    pub thickness: f32,
}

/// A laid-out equation: width, ascent, descent, glyphs, and rules.
#[derive(Debug, Clone, Default)]
pub(crate) struct MathBox {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub glyphs: Vec<MathGlyph>,
    pub rules: Vec<MathRule>,
}

/// Lay out the first `m:oMath` child of `omml`. `display` selects the
/// display MATH constants (an `m:oMathPara` ancestor in the document).
pub(crate) fn layout_omml(font: &MathFont, size: f32, display: bool, omml: &str) -> MathBox {
    let _ = (font, size, display, omml);
    MathBox::default()
}

#[cfg(test)]
mod tests {
    use super::super::math_font::MathFont;
    use super::*;

    const SIZE: f32 = 12.0;
    const STIX: &[u8] = include_bytes!("../../tests/fixtures/fonts/STIXTwoMath.otf");

    const BAR: &str = r#"<m:f><m:num><m:r><m:t>a</m:t></m:r></m:num><m:den><m:r><m:t>b</m:t></m:r></m:den></m:f>"#;
    const RAD: &str = r#"<m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad>"#;
    const DEG: &str = r#"<m:rad><m:deg><m:r><m:t>3</m:t></m:r></m:deg><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad>"#;

    fn stix() -> MathFont {
        MathFont::from_bytes(STIX).expect("STIX Two Math")
    }

    fn lay(font: &MathFont, display: bool, omml: &str) -> MathBox {
        layout_omml(font, SIZE, display, omml)
    }

    /// Mathematical italic, the character Word puts in the text layer.
    fn italic(c: char) -> char {
        match c {
            'h' => '\u{210E}',
            'a'..='z' => char::from_u32(0x1D44E + (c as u32 - 'a' as u32)).unwrap(),
            'A'..='Z' => char::from_u32(0x1D434 + (c as u32 - 'A' as u32)).unwrap(),
            _ => c,
        }
    }

    fn glyph<'a>(b: &'a MathBox, ascii: char) -> &'a MathGlyph {
        let want = italic(ascii);
        b.glyphs
            .iter()
            .find(|g| g.text.contains(want))
            .unwrap_or_else(|| panic!("no {want:?} in {b:?}; layout returned an empty box"))
    }

    fn radical<'a>(b: &'a MathBox) -> &'a MathGlyph {
        b.glyphs
            .iter()
            .find(|g| g.text.contains('\u{221A}'))
            .unwrap_or_else(|| panic!("no radical sign in {b:?}; layout returned an empty box"))
    }

    fn opening(b: &MathBox) -> &MathGlyph {
        b.glyphs
            .iter()
            .find(|g| g.text.contains('('))
            .expect("opening delimiter")
    }

    fn span(font: &MathFont, g: &MathGlyph) -> (f32, f32) {
        let m = font.glyph(g.gid);
        let top = g.y + font.scale(m.height(), g.size);
        let bot = g.y - font.scale(m.depth(), g.size);
        (top, bot)
    }

    #[test]
    fn math_layout_bar_fraction_puts_the_rule_on_the_axis() {
        let font = stix();
        let b = lay(&font, true, BAR);
        let rule = b.rules.first().expect("bar fraction has a rule");
        let axis = font.scale(font.constants().axis_height, SIZE);
        let center = rule.y_top - rule.thickness / 2.0;
        assert!(
            (center - axis).abs() < 0.05,
            "rule center {center} is not the axis {axis}"
        );
        let num = glyph(&b, 'a');
        let den = glyph(&b, 'b');
        assert!(
            num.y > rule.y_top,
            "numerator baseline {} is not above the rule {}",
            num.y,
            rule.y_top
        );
        assert!(
            den.y < rule.y_top - rule.thickness,
            "denominator baseline {} is not below the rule",
            den.y
        );
        let piece = |g: &MathGlyph| font.scale(font.glyph(g.gid).advance, g.size);
        assert!(
            rule.w + 0.01 >= piece(num),
            "rule narrower than the numerator"
        );
        assert!(
            rule.w + 0.01 >= piece(den),
            "rule narrower than the denominator"
        );
        assert!(
            (num.size - SIZE).abs() < 0.05,
            "display numerator stays at the base size, got {}",
            num.size
        );
    }

    #[test]
    fn math_layout_inline_fraction_shrinks_numerator_and_denominator() {
        let font = stix();
        let b = lay(&font, false, BAR);
        let num = glyph(&b, 'a');
        let want = SIZE * font.constants().script_percent;
        assert!(
            (num.size - want).abs() < 0.05,
            "inline numerator {} is not script size {want}",
            num.size
        );
    }

    #[test]
    fn math_layout_radical_rule_clears_the_body() {
        let font = stix();
        let b = lay(&font, false, RAD);
        let sign = radical(&b);
        let body = glyph(&b, 'x');
        assert!(sign.x < body.x, "radical sign is not left of the body");
        let rule = b.rules.first().expect("radical has a rule");
        let (body_top, _) = span(&font, body);
        let gap = font.scale(font.constants().radical_vertical_gap, SIZE);
        let bottom = rule.y_top - rule.thickness;
        assert!(
            bottom + 0.05 >= body_top + gap,
            "rule bottom {bottom} is under body top {body_top} plus gap {gap}"
        );
    }

    #[test]
    fn math_layout_radical_degree_sits_above_and_left() {
        let font = stix();
        let b = lay(&font, false, DEG);
        let sign = radical(&b);
        let body = glyph(&b, 'x');
        let deg = b
            .glyphs
            .iter()
            .find(|g| g.text.contains('3'))
            .expect("degree");
        assert!(deg.x < sign.x, "degree is not left of the radical");
        assert!(deg.y > body.y, "degree is not above the body baseline");
    }

    #[test]
    fn math_layout_delimiters_grow_to_cover_a_fraction() {
        let font = stix();
        let inner = lay(&font, true, BAR);
        let wrapped = format!("<m:d><m:e>{BAR}</m:e></m:d>");
        let short = lay(
            &font,
            true,
            r#"<m:d><m:e><m:r><m:t>x</m:t></m:r></m:e></m:d>"#,
        );
        let tall = lay(&font, true, &wrapped);
        let short_span = {
            let (top, bot) = span(&font, opening(&short));
            top - bot
        };
        let tall_g = opening(&tall);
        let (top, bot) = span(&font, tall_g);
        assert!(
            top - bot > short_span + 0.5,
            "fence around a fraction ({}) is not taller than the fence around x ({short_span})",
            top - bot
        );
        assert!(
            top + 0.05 >= inner.ascent,
            "fence top {top} does not cover the fraction ascent {}",
            inner.ascent
        );
        assert!(
            bot - 0.05 <= -inner.descent,
            "fence bottom {bot} does not cover the fraction descent {}",
            inner.descent
        );
    }

    #[test]
    fn math_layout_nests_a_fraction_in_a_radical_in_brackets() {
        let font = stix();
        let omml = format!(
            r#"<m:d><m:e><m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e>{BAR}</m:e></m:rad></m:e></m:d>"#
        );
        let b = lay(&font, true, &omml);
        assert!(
            b.rules.len() >= 2,
            "nesting needs the fraction bar and the radical rule, got {}",
            b.rules.len()
        );
        let sign = radical(&b);
        let num = glyph(&b, 'a');
        let open = b
            .glyphs
            .iter()
            .find(|g| g.text.contains('('))
            .expect("opening bracket");
        let close = b
            .glyphs
            .iter()
            .find(|g| g.text.contains(')'))
            .expect("closing bracket");
        assert!(sign.x < num.x, "radical is not left of the fraction");
        assert!(
            open.x < sign.x,
            "opening bracket is not outside the radical"
        );
        assert!(
            close.x > num.x,
            "closing bracket is not outside the fraction"
        );
        assert!(b.width.is_finite() && b.ascent.is_finite() && b.descent.is_finite());
        assert!(b.glyphs.iter().all(|g| g.x.is_finite() && g.y.is_finite()));
        assert!(b.rules.iter().all(|r| r.x.is_finite() && r.w.is_finite()));
    }

    #[test]
    fn math_layout_superscript_inside_a_radical_uses_todays_script_shift() {
        let font = stix();
        let omml = r#"<m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e><m:sSup><m:e><m:r><m:t>x</m:t></m:r></m:e><m:sup><m:r><m:t>2</m:t></m:r></m:sup></m:sSup></m:e></m:rad>"#;
        let b = lay(&font, true, omml);
        let base = glyph(&b, 'x');
        let sup = b
            .glyphs
            .iter()
            .find(|g| g.text.contains('2'))
            .expect("superscript");
        assert!(
            (sup.size - SIZE * 0.65).abs() < 0.05,
            "script size {} is not 0.65 of the base",
            sup.size
        );
        assert!(
            (sup.y - base.y - SIZE * 0.35).abs() < 0.05,
            "script shift {} is not +0.35 em",
            sup.y - base.y
        );
    }

    #[test]
    fn math_layout_fallback_font_still_stacks_a_bar_fraction() {
        let font = MathFont::fallback(1000.0);
        let b = lay(&font, true, BAR);
        let rule = b.rules.first().expect("fallback fraction has a rule");
        let num = glyph(&b, 'a');
        let den = glyph(&b, 'b');
        assert!(
            num.y > rule.y_top,
            "fallback numerator is not above the bar"
        );
        assert!(
            den.y < rule.y_top - rule.thickness,
            "fallback denominator is not below the bar"
        );
    }
}
