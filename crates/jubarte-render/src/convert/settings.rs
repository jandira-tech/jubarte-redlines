// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The settings part, the East Asian line-layout options it switches on, and
//! compatibility settings.

use super::*;

/// `word/settings.xml` parsed once per reader; `None` when the part is absent.
pub(super) fn settings_dom(pkg: &PartFs) -> Option<ParsedSettings> {
    let xml = pkg.part_string(&settings_part(pkg))?;
    settings_dom_xml(&xml)
}

/// A parsed settings part and its root element.
type ParsedSettings = Rc<(Dom, NodeId)>;

/// `xml` parsed, memoised per thread on the text: a conversion asks a dozen
/// settings questions of the same part, and parsing it each time was 11% of
/// a one-page conversion (samply, fixtures_500 0081ba58).
fn settings_dom_xml(xml: &str) -> Option<ParsedSettings> {
    thread_local! {
        static LAST: RefCell<Option<(String, ParsedSettings)>> = const { RefCell::new(None) };
    }
    if let Some(hit) = LAST.with_borrow(|last| {
        last.as_ref()
            .filter(|(text, _)| text == xml)
            .map(|(_, parsed)| Rc::clone(parsed))
    }) {
        return Some(hit);
    }
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let parsed = Rc::new((dom, root));
    LAST.set(Some((xml.to_string(), Rc::clone(&parsed))));
    Some(parsed)
}

/// ST_OnOff `w:<local>` under `w:settings` (direct or inside `w:compat`):
/// absent → off; present → on unless `w:val` is `0`/`false`/`off`
/// (ECMA-376 17.17.4). Matched as elements, so quoting style, attribute
/// order and look-alike names cannot flip it.
pub(super) fn settings_flag_xml(xml: &str, local: &str) -> bool {
    settings_dom_xml(xml).is_some_and(|s| {
        let (dom, root) = (&s.0, s.1);
        dom.descendants(root, Some(&W::name(local)))
            .into_iter()
            .next()
            .is_some_and(|n| !val_is_false(dom, Some(n)))
    })
}

pub(super) fn settings_flag(pkg: &PartFs, local: &str) -> bool {
    pkg.part_string(&settings_part(pkg))
        .is_some_and(|xml| settings_flag_xml(&xml, local))
}

/// `w:compatSetting name="compatibilityMode"`. Absent → 12 (Word 2007),
/// which uses the pre-2013 table-edge rule (plan xml 3.3).
/// From compatibility mode 15 an atLeast line keeps its own height on a
/// line grid: 0085209e / 0062780d's lines step 15.5 / 20.7pt on an 18pt
/// grid, while 002c5410 (mode 14) snaps them.
pub(super) fn at_least_off_grid(blocks: &mut [Block], compat_mode: u8) {
    if compat_mode < 15 {
        return;
    }
    for block in blocks {
        if let Block::Paragraph { style, .. } = block
            && style.line_at_least.is_some()
        {
            style.snap_to_grid = false;
        }
    }
}

/// Before compatibility mode 15 Word breaks ideographs between characters
/// only when the run's eastAsia font is an East Asian one. Under a Latin
/// one (005919f8: Calibri) a run of ideographs is a word: Word's oracle
/// moves it whole to the next line and splits it only at a line's end.
pub(super) fn mark_ideograph_words(blocks: &mut [Block], fonts: &Fonts, compat_mode: u8) {
    if compat_mode >= 15 {
        return;
    }
    let mark = |runs: &mut [TextRun]| {
        for run in runs {
            run.style.ideograph_words = run
                .style
                .family_ea
                .as_deref()
                .is_some_and(|f| fonts.family_is_latin_only(f));
        }
    };
    for block in blocks {
        match block {
            Block::Paragraph { runs, .. } => mark(runs),
            Block::Table { rows, .. } => {
                for cell in rows.iter_mut().flatten() {
                    for para in &mut cell.paras {
                        mark(&mut para.runs);
                    }
                }
            }
            _ => {}
        }
    }
}

/// `w:balanceSingleByteDoubleByteWidth` (Word 16 probes sp1–sp5, any
/// compatibility mode): a space or no-break space next to another, across
/// runs too, or between two East Asian characters advances the run's
/// eastAsia face's average character width (MS Mincho 0.5em); a lone space
/// between Latin letters keeps its own. A run with no eastAsia font, or an
/// absent one, takes its Latin face's average (TNR 0.401em, Arial 0.441em).
/// An absent name, or a Latin face other than the run's own (sp6: Arial
/// over Times New Roman), falls back to an installed East Asian face the
/// document runs at the same size anywhere, before or after it (sp5 e:
/// 4.81pt at 12pt, 9.0pt at 18pt beside an 18pt MS Mincho run), a serif
/// Latin face to a serif one (MS Mincho) and a sans one to a sans one (MS
/// Gothic; sp9: Arial beside 16pt MS Mincho keeps its own 7.07pt); a run
/// with no eastAsia font, or its ascii face there, never does (sp3 LN,
/// sp6 V1).
/// LibreOffice's tdf#88908 (`BalanceCjkSpaces`) also widens a lone space
/// at a run's edge; Word's PDF of ee79137dd5 keeps those at Calibri's width.
/// Each stretch becomes a run of its own carrying the width
/// (`RunStyle::fixed_advance`).
///
/// `default_ea` is the document default eastAsia face: a run that names
/// none inherits it, and an inherited Latin name (Word 16's stock Aptos,
/// #1725) is no eastAsia font at all — Word balances such a run on the
/// Latin average, where an authored absent name borrows a loaded one.
pub(super) fn balance_spaces(blocks: &mut [Block], fonts: &Fonts, default_ea: Option<&str>) {
    let mut loaded = Vec::new();
    for_each_block_runs(blocks, &mut |runs| {
        for run in runs.iter() {
            let size = run.style.layout_size();
            if let Some(face) = run
                .style
                .family_ea
                .as_deref()
                .filter(|f| fonts.family_is_east_asian(f))
                .and_then(|f| fonts.east_asia_face(f))
                && !loaded
                    .iter()
                    .any(|&(s, serif, _)| s == size && serif == face.serif)
            {
                loaded.push((size, face.serif, face.avg_char_width));
            }
        }
    });
    for_each_block_runs(blocks, &mut |runs| {
        balance_run_spaces(runs, fonts, &loaded, default_ea);
    });
}

/// Whether Word runs its East Asian layout code, which balances spaces:
/// `w:useFELayout`, or a default eastAsia face that is East Asian. Word 16
/// probes sp7/sp8 (2026-10-02): docDefaults naming MS Mincho balance, a
/// styles part with Times New Roman or no eastAsia face does not (6a0d's
/// AASB standard keeps its double spaces at 2.38pt), and no styles part
/// at all takes Word's own East Asian default and balances.
pub(super) fn east_asian_layout(pkg: &PartFs, sheet: &StyleSheet, fonts: &Fonts) -> bool {
    settings_flag(pkg, "useFELayout")
        || pkg
            .part_string(&main_rel_part(pkg, "styles", "word/styles.xml"))
            .is_none()
        || sheet
            .defaults
            .run
            .family_ea
            .as_deref()
            .is_some_and(|f| fonts.family_is_east_asian(f))
}

/// Calls `f` on each body paragraph's and table cell paragraph's runs,
/// nested tables included.
fn for_each_block_runs(blocks: &mut [Block], f: &mut dyn FnMut(&mut Vec<TextRun>)) {
    for block in blocks {
        match block {
            Block::Paragraph { runs, .. } => f(runs),
            Block::Table { rows, .. } => {
                for cell in rows.iter_mut().flatten() {
                    for para in &mut cell.paras {
                        f(&mut para.runs);
                    }
                    for nested in &mut cell.nested {
                        if let Some(block) = std::rc::Rc::get_mut(nested) {
                            for_each_block_runs(std::slice::from_mut(block), f);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn balance_run_spaces(
    runs: &mut Vec<TextRun>,
    fonts: &Fonts,
    loaded: &[(f32, bool, f32)],
    default_ea: Option<&str>,
) {
    let is_space = |c: char| matches!(c, ' ' | '\u{a0}');
    let flat: Vec<char> = runs.iter().flat_map(|r| r.text.chars()).collect();
    let marks: Vec<bool> = (0..flat.len())
        .map(|i| {
            let prev = i.checked_sub(1).map(|j| flat[j]);
            let next = flat.get(i + 1).copied();
            is_space(flat[i])
                && (prev.is_some_and(is_space)
                    || next.is_some_and(is_space)
                    || prev.is_some_and(is_cjk_break_char) && next.is_some_and(is_cjk_break_char))
        })
        .collect();
    split_fixed_advances(runs, &marks, |run| {
        let size = run.style.layout_size();
        let latin = fonts.get(ink_face(fonts, &run.style, " "));
        let em = match run.style.family_ea.as_deref() {
            Some(family) if family.eq_ignore_ascii_case(&run.style.family) => None,
            // The inherited default: a non-East-Asian one is Word's
            // unset eastAsia slot, which balances on the Latin average.
            Some(family) if default_ea == Some(family) && !fonts.family_is_east_asian(family) => {
                None
            }
            Some(family) => fonts
                .family_is_east_asian(family)
                .then(|| fonts.east_asia_avg_em(family))
                .flatten()
                .or_else(|| {
                    loaded
                        .iter()
                        .find(|&&(s, serif, _)| s == size && serif == latin.serif)
                        .map(|&(_, _, em)| em)
                }),
            None => None,
        }
        .unwrap_or(latin.avg_char_width);
        size * em
    });
}

/// Word 16 probes cp1–cp3 (2026-10-02, every compatibility mode and
/// `w:characterSpacingControl`): a full-width closing bracket before
/// another bracket, or an opening bracket before another opening one,
/// advances half an em ("」「", "）」", "「「" are 6pt at 12pt), across runs
/// too; a bracket beside 。、・！？ or a curly quote keeps its em.
pub(super) fn squeeze_bracket_pairs(blocks: &mut [Block]) {
    for_each_block_runs(blocks, &mut |runs| {
        let flat: Vec<char> = runs.iter().flat_map(|r| r.text.chars()).collect();
        let marks: Vec<bool> = (0..flat.len())
            .map(|i| {
                let next = flat.get(i + 1).copied();
                is_closing_bracket(flat[i])
                    && next.is_some_and(|n| is_closing_bracket(n) || is_opening_bracket(n))
                    || is_opening_bracket(flat[i]) && next.is_some_and(is_opening_bracket)
            })
            .collect();
        split_fixed_advances(runs, &marks, |run| run.style.layout_size() * 0.5);
    });
}

fn is_opening_bracket(c: char) -> bool {
    matches!(
        c,
        '\u{FF08}'
            | '\u{FF3B}'
            | '\u{FF5B}'
            | '\u{3008}'
            | '\u{300A}'
            | '\u{300C}'
            | '\u{300E}'
            | '\u{3010}'
            | '\u{3014}'
            | '\u{3016}'
            | '\u{3018}'
            | '\u{301A}'
    )
}

fn is_closing_bracket(c: char) -> bool {
    matches!(
        c,
        '\u{FF09}'
            | '\u{FF3D}'
            | '\u{FF5D}'
            | '\u{3009}'
            | '\u{300B}'
            | '\u{300D}'
            | '\u{300F}'
            | '\u{3011}'
            | '\u{3015}'
            | '\u{3017}'
            | '\u{3019}'
            | '\u{301B}'
    )
}

/// Splits each plain run of `runs` around the characters `marks` flags
/// (indexed over the runs' joined text), giving each flagged stretch the
/// fixed advance `width` measures for its run.
pub(super) fn split_fixed_advances(
    runs: &mut Vec<TextRun>,
    marks: &[bool],
    width: impl Fn(&TextRun) -> f32,
) {
    if !marks.contains(&true) {
        return;
    }
    let mut out = Vec::with_capacity(runs.len());
    let mut at = 0;
    for run in runs.drain(..) {
        let n = run.text.chars().count();
        let marks = &marks[at..at + n];
        at += n;
        let plain = run.field == FieldKind::None
            && run.comments.is_empty()
            && run.pageref.is_none()
            && run.ref_name.is_none()
            && run.rule.is_none()
            && run.footnote_id.is_none()
            && run.endnote_id.is_none()
            && !run.note_ref
            && !run.note_rule
            && !run.ends_line
            && !run.list_marker
            && run.checkbox.is_none()
            && !run.strut
            && run.hf_para.is_none();
        if !plain || !marks.contains(&true) {
            out.push(run);
            continue;
        }
        let fixed = width(&run);
        let chars: Vec<char> = run.text.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let mark = marks[start];
            let end = (start..chars.len())
                .find(|&k| marks[k] != mark)
                .unwrap_or(chars.len());
            let mut piece = run.clone();
            piece.text = chars[start..end].iter().collect();
            if mark {
                piece.style.fixed_advance = fixed;
            }
            out.push(piece);
            start = end;
        }
    }
    *runs = out;
}

/// `w:linkStyles` with no `w:attachedTemplate`: Word refreshes the styles
/// from Normal.dotm when it opens the file. A named template lives on the
/// author's machine (a7110391's `C:\Users\...\NESO ... .dotx`, a4168b8a's
/// `D:\CASA ... .dotx`); Word cannot load it and keeps the file's styles.
pub(super) fn settings_link_styles(pkg: &PartFs) -> bool {
    pkg.part_string(&settings_part(pkg)).is_some_and(|xml| {
        settings_dom_xml(&xml).is_some_and(|s| {
            let (dom, root) = (&s.0, s.1);
            dom.descendants(root, Some(&W::name("attachedTemplate")))
                .is_empty()
                && dom
                    .descendants(root, Some(&W::name("linkStyles")))
                    .into_iter()
                    .any(|n| !matches!(attr_any(dom, n, "val"), Some("0" | "false" | "off")))
        })
    })
}

/// The stock Normal.dotm's Normal as Word resolves it over the template's
/// docDefaults: theme minor font, 12pt, kern 1pt, after=160, line=278.
pub(super) const TEMPLATE_NORMAL: &str = "<w:name w:val=\"Normal\"/><w:qFormat/>\
    <w:pPr><w:spacing w:after=\"160\" w:line=\"278\" w:lineRule=\"auto\"/></w:pPr>\
    <w:rPr><w:rFonts w:asciiTheme=\"minorHAnsi\" w:eastAsiaTheme=\"minorHAnsi\" w:hAnsiTheme=\"minorHAnsi\" w:cstheme=\"minorBidi\"/>\
    <w:kern w:val=\"2\"/><w:sz w:val=\"24\"/><w:szCs w:val=\"24\"/>\
    <w:lang w:val=\"en-US\" w:eastAsia=\"en-US\" w:bidi=\"ar-SA\"/></w:rPr>";

/// styles.xml as Word sees it after `w:linkStyles` pulled in the stock
/// Normal.dotm (en a 9b100bdc: 12pt on 16pt lines, not the file's 11pt on
/// 259). Only the default paragraph style is replaced: a style off its
/// chain (12d245d664's "Default", no basedOn) keeps the file's own
/// docDefaults (Word 16 probe lnk l1, 2026-10-02: 10pt single either
/// way). The template's other styles (Default Paragraph Font, Normal
/// Table, No List) are Word's built-in ones already.
pub(super) fn link_template_styles(xml: &str) -> String {
    let mut out = xml.to_string();
    let mut from = 0;
    while let Some(at) = out[from..].find("<w:style ").map(|i| from + i) {
        let Some(tag_end) = out[at..].find('>').map(|i| at + i) else {
            break;
        };
        let tag = out[at..=tag_end].to_string();
        let normal = tag.contains("w:type=\"paragraph\"") && tag.contains("w:default=\"1\"");
        let end = if tag.ends_with("/>") {
            Some(tag_end + 1)
        } else {
            out[at..]
                .find("</w:style>")
                .map(|i| at + i + "</w:style>".len())
        };
        let Some(end) = end else {
            break;
        };
        if normal {
            let open = tag.trim_end_matches("/>").trim_end_matches('>');
            out.replace_range(at..end, &format!("{open}>{TEMPLATE_NORMAL}</w:style>"));
            return out;
        }
        from = end;
    }
    if let Some(close) = out.rfind("</w:styles>") {
        out.insert_str(
            close,
            &format!(
                "<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">{TEMPLATE_NORMAL}</w:style>"
            ),
        );
    }
    out
}

pub(super) fn settings_compat_mode(pkg: &PartFs) -> u8 {
    pkg.part_string(&settings_part(pkg))
        .map_or(12, |xml| settings_compat_mode_xml(&xml))
}

/// The setting is keyed by `name` *and* `uri`: another vendor's
/// `compatibilityMode` under its own URI is not Word's.
pub(super) const WORD_COMPAT_SETTING_URI: &str = "http://schemas.microsoft.com/office/word";

pub(super) fn settings_compat_mode_xml(xml: &str) -> u8 {
    let Some(s) = settings_dom_xml(xml) else {
        return 12;
    };
    let (dom, root) = (&s.0, s.1);
    for node in dom.descendants(root, Some(&W::name("compatSetting"))) {
        if attr_any(dom, node, "name") == Some("compatibilityMode")
            && attr_any(dom, node, "uri") == Some(WORD_COMPAT_SETTING_URI)
            && let Some(mode) = attr_any(dom, node, "val").and_then(|s| s.parse().ok())
        {
            return mode;
        }
    }
    12
}

/// Word's `overrideTableStyleFontSizeAndJustification` compat setting: the
/// default paragraph style's size and jc win over a table style's.
pub(super) fn settings_override_table_style_size(pkg: &PartFs) -> bool {
    pkg.part_string(&settings_part(pkg))
        .is_some_and(|xml| settings_override_table_style_size_xml(&xml))
}

pub(super) fn settings_override_table_style_size_xml(xml: &str) -> bool {
    let Some(s) = settings_dom_xml(xml) else {
        return false;
    };
    let (dom, root) = (&s.0, s.1);
    dom.descendants(root, Some(&W::name("compatSetting")))
        .into_iter()
        .any(|node| {
            attr_any(dom, node, "name") == Some("overrideTableStyleFontSizeAndJustification")
                && attr_any(dom, node, "uri") == Some(WORD_COMPAT_SETTING_URI)
                && matches!(attr_any(dom, node, "val"), Some("1" | "true" | "on"))
        })
}

/// `w:evenAndOddHeaders`: type=even header/footer on even page numbers.
pub(super) fn settings_even_and_odd_headers(pkg: &PartFs) -> bool {
    settings_flag(pkg, "evenAndOddHeaders")
}

/// `w:mirrorMargins`: swap pgMar left/right on even page numbers.
pub(super) fn settings_mirror_margins(pkg: &PartFs) -> bool {
    settings_flag(pkg, "mirrorMargins")
}

/// Before compatibility mode 15, `compressPunctuation` lets every line
/// narrow its spaces to keep its last word (`PUNCT_SQUEEZE`), unless the
/// document balances single- and double-byte widths under an East Asian
/// theme language: 496e2984f7 (balanceSingleByteDoubleByteWidth,
/// themeFontLang zh-CN) leaves Word's lines unsqueezed. Word 16 probes
/// (2026-10-01): zh-TW, ja-JP and ko-KR alike; en-US or either setting
/// alone squeezes; run languages do not count. A document default that
/// names fonts turns it off too (`load_stylesheet`).
pub(super) fn settings_punct_squeeze(pkg: &PartFs) -> bool {
    pkg.part_string(&settings_part(pkg))
        .is_some_and(|xml| settings_punct_squeeze_xml(&xml))
}

pub(super) fn settings_punct_squeeze_xml(xml: &str) -> bool {
    if settings_compat_mode_xml(xml) >= 15
        || settings_character_spacing_xml(xml) == CharacterSpacing::DoNotCompress
    {
        return false;
    }
    let east_asian_theme = settings_dom_xml(xml).is_some_and(|s| {
        let (dom, root) = (&s.0, s.1);
        dom.descendants(root, Some(&W::name("themeFontLang")))
            .into_iter()
            .next()
            .and_then(|n| attr_any(dom, n, "eastAsia"))
            .is_some_and(|lang| {
                let lang = lang.to_ascii_lowercase();
                ["zh", "ja", "ko"].iter().any(|p| lang.starts_with(p))
            })
    });
    !(east_asian_theme && settings_flag_xml(xml, "balanceSingleByteDoubleByteWidth"))
}

/// `w:compat/w:ulTrailSpace`: underline trailing spaces (ECMA-376 17.15.3.63).
/// Omitted → off. Present (default on) paints the pad Word otherwise skips.
pub(super) fn settings_ul_trail_space(pkg: &PartFs) -> bool {
    settings_flag(pkg, "ulTrailSpace")
}

/// `w:compat/w:doNotExpandShiftReturn` (ECMA-376 17.15.3.10).
/// Present: a justified line ending at `w:br` is not expanded.
pub(super) fn settings_do_not_expand_shift_return(pkg: &PartFs) -> bool {
    settings_flag(pkg, "doNotExpandShiftReturn")
}

/// `w:characterSpacingControl/@w:val` (ECMA-376 17.15.1.18).
pub(super) fn settings_character_spacing(pkg: &PartFs) -> CharacterSpacing {
    pkg.part_string(&settings_part(pkg))
        .map_or(CharacterSpacing::DoNotCompress, |xml| {
            settings_character_spacing_xml(&xml)
        })
}

pub(super) fn settings_character_spacing_xml(xml: &str) -> CharacterSpacing {
    let val = settings_dom_xml(xml).and_then(|s| {
        let (dom, root) = (&s.0, s.1);
        dom.descendants(root, Some(&W::name("characterSpacingControl")))
            .into_iter()
            .next()
            .and_then(|n| dom.attribute(n, &W::val()).map(str::to_string))
    });
    match val.as_deref() {
        Some("compressPunctuationAndJapaneseKana") => CharacterSpacing::CompressPunctuationAndKana,
        Some("compressPunctuation") => CharacterSpacing::CompressPunctuation,
        _ => CharacterSpacing::DoNotCompress,
    }
}

/// Word factory is 720 twips (0.5in). Strict01 writes `36pt`; mcdoc `420`.
pub(super) fn settings_default_tab_pt(pkg: &PartFs) -> Option<f32> {
    let s = settings_dom(pkg)?;
    let (dom, root) = (&s.0, s.1);
    let stop = first_named(dom, root, "defaultTabStop")?;
    attr_any(dom, stop, "val")
        .and_then(parse_len)
        .filter(|pt| *pt > 0.5)
}
