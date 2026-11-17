// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Line wrapping: run pieces, CJK and bidi rules, hyphenation, tabs and
//! justification squeeze.

use super::*;

/// Two runs may merge into one painted run: the same character style
/// and the same `STYLEREF` field (or none), so `patch_stylerefs` still
/// finds both.
pub(super) fn same_style_refs(a: &TextRun, b: &TextRun) -> bool {
    let same_field = match (&a.styleref, &b.styleref) {
        (None, None) => true,
        (Some(x), Some(y)) => std::rc::Rc::ptr_eq(x, y),
        _ => false,
    };
    same_field && a.char_style == b.char_style
}

/// Body text in one style, for header `STYLEREF` (`Layout::note_style_hit`).
pub(super) struct StyleHit {
    pub(super) id: std::rc::Rc<str>,
    pub(super) serial: u32,
    /// First and last page the text is painted on.
    pub(super) pages: (usize, usize),
    pub(super) text: String,
    /// The last run painted belongs to this hit.
    pub(super) open: bool,
}

pub(super) const NUMPAGES_MARK: &str = "@@N@@";

pub(super) const CHAP_PAGE_MARK: &str = "@@P@@";

pub(super) fn chrome_measure_text(text: &str) -> &str {
    if text == NUMPAGES_MARK || text == CHAP_PAGE_MARK {
        "0"
    } else {
        text
    }
}

pub(super) fn patch_numpages(fonts: &Fonts, pages: &mut [Page]) {
    let total = pages.len().to_string();
    for page in pages {
        for op in &mut page.ops {
            if let Op::Text {
                face, glyphs, text, ..
            } = op
            {
                let mark = fonts.get(*face).glyphs(NUMPAGES_MARK);
                if *glyphs == mark {
                    *glyphs = fonts.get(*face).glyphs(&total);
                    *text = total.clone();
                }
            }
        }
    }
}

pub(super) fn default_run_style() -> RunStyle {
    RunStyle {
        family: "Calibri".into(),
        family_ea: None,
        family_cs: None,
        lang_ea: None,
        ea_theme_slot: None,
        hint: FontHint::Default,
        rtl: false,
        family_hansi: None,
        size: 11.0,
        size_cs: None,
        bold: false,
        italic: false,
        underline: false,
        underline_double: false,
        strike_double: false,
        underline_wave: false,
        strike: false,
        color: [0.0, 0.0, 0.0],
        color_auto: true,
        highlight: None,
        tint: None,
        tint_id: None,
        highlight_marker: false,
        track: 0.0,
        scale: 1.0,
        caps: false,
        small_caps: false,
        box_size: 0.0,
        bold_set: false,
        italic_set: false,
        hidden: false,
        auto_space_de_off: false,
        auto_space_dn_off: false,
        offset: 0.0,
        vert: VertAlign::Baseline,
        kern_half: 0,
        effect_skip: false,
        ideograph_words: false,
        punct_compress: false,
        fixed_advance: 0.0,
    }
}

pub(super) fn small_caps_pieces(text: &str, style: &RunStyle) -> Vec<(String, RunStyle)> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut small = false;
    let full = style.size;
    let reduced = full * 0.8;
    let flush = |buf: &mut String, small: bool, out: &mut Vec<(String, RunStyle)>| {
        if buf.is_empty() {
            return;
        }
        let mut st = style.clone();
        st.small_caps = false;
        st.size = if small { reduced } else { full };
        st.box_size = full;
        out.push((std::mem::take(buf), st));
    };
    for ch in text.chars() {
        let is_small = ch.is_lowercase();
        if !buf.is_empty() && is_small != small {
            flush(&mut buf, small, &mut out);
        }
        small = is_small;
        for u in ch.to_uppercase() {
            buf.push(u);
        }
    }
    flush(&mut buf, small, &mut out);
    out
}

pub(super) fn style_eq(a: &RunStyle, b: &RunStyle) -> bool {
    // Script slots decide the painted face per range (ECMA-376 17.3.2.26):
    // runs that differ there are different formats even with one Latin face.
    a.family == b.family
        && a.family_ea == b.family_ea
        && a.family_cs == b.family_cs
        && a.hint == b.hint
        && (a.size - b.size).abs() < f32::EPSILON
        && a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.underline_double == b.underline_double
        && a.underline_wave == b.underline_wave
        && a.tint == b.tint
        && a.strike == b.strike
        && a.color == b.color
        && a.highlight == b.highlight
        && (a.track - b.track).abs() < f32::EPSILON
        && (a.fixed_advance - b.fixed_advance).abs() < f32::EPSILON
        && (a.scale - b.scale).abs() < f32::EPSILON
        && a.caps == b.caps
        && (a.offset - b.offset).abs() < f32::EPSILON
        && a.vert == b.vert
}

/// Word wraps `https://…/en-us/…` after `-` (comments-lots appendix), never
/// after `/`: 0 of fixtures_500's 34 mid-URL breaks (001c1554 moves the
/// whole URL down). Not generic character-break (Test 7 / mini 57).
pub(super) fn url_wrap_pieces(tok: &str) -> Vec<&str> {
    if !tok.contains("://") {
        return vec![tok];
    }
    let scheme_end = tok.find("://").map_or(0, |i| i + 3);
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = scheme_end;
    while let Some(ch) = tok[i..].chars().next() {
        let n = ch.len_utf8();
        if ch == '-' {
            out.push(&tok[start..i + n]);
            start = i + n;
        }
        i += n;
    }
    if start < tok.len() {
        out.push(&tok[start..]);
    }
    if out.is_empty() { vec![tok] } else { out }
}

/// A cell paragraph ready to paint: its wrapped lines (each sized by its
/// own runs, `cell_line_metrics`) and which of them end in a `w:br`.
pub(super) type LaidCellPara = (Vec<Vec<TextRun>>, Vec<bool>);

/// One measured piece of a run inside a wrap unit: source run, text, width.
type WrapPiece<'r> = (&'r TextRun, &'r str, f32);

/// Whether a line may break right after `tok` of `run`: it ends on a
/// hyphen that follows a letter or digit, as `hyphen_wrap_pieces` splits
/// inside one. A run boundary there is a break too (b6dd0b6ae8: Word ends
/// a line on "-Karaman-" and starts the next with the following run's
/// "Kepenekci,").
pub(super) fn breaks_after(run: &TextRun, tok: &str) -> bool {
    let mut end = tok.chars().rev();
    !run.style.punct_compress
        && end.next() == Some('-')
        && end.next().is_some_and(char::is_alphanumeric)
}

/// Word's line may end after a hyphen-minus that follows a letter or digit
/// and precedes more text: `sham-vaccinated` → `sham-` | `vaccinated`,
/// unless the hyphen holds (`RunStyle::punct_compress`).
pub(super) fn hyphen_wrap_pieces(
    tok: &str,
    ideograph_breaks: bool,
    hyphen_breaks: bool,
) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut prev: Option<char> = None;
    for (i, ch) in tok.char_indices() {
        let end = i + ch.len_utf8();
        // Word breaks between ideographs as between words (docxide
        // taiwanese_education_fraud_ruling: every justified line holds 25
        // characters), never before closing punctuation or small kana and
        // never after opening punctuation.
        if let Some(p) = prev
            && ideograph_breaks
            && i > start
            && (is_cjk_break_char(p) || is_cjk_break_char(ch))
            && !cjk_no_line_start(ch)
            && !cjk_no_line_end(p)
        {
            out.push(&tok[start..i]);
            start = i;
        }
        if hyphen_breaks && ch == '-' && prev.is_some_and(char::is_alphanumeric) && end < tok.len()
        {
            out.push(&tok[start..end]);
            start = end;
        }
        prev = Some(ch);
    }
    out.push(&tok[start..]);
    out
}

/// Kinsoku: characters a line may not start with (closing punctuation,
/// small kana, the prolonged sound mark).
fn cjk_no_line_start(c: char) -> bool {
    matches!(
        c,
        '、' | '。'
            | '，'
            | '．'
            | '：'
            | '；'
            | '？'
            | '！'
            | '）'
            | '」'
            | '』'
            | '】'
            | '〉'
            | '》'
            | '〕'
            | '］'
            | '｝'
            | '・'
            | 'ー'
            | '々'
            | 'ぁ'
            | 'ぃ'
            | 'ぅ'
            | 'ぇ'
            | 'ぉ'
            | 'っ'
            | 'ゃ'
            | 'ゅ'
            | 'ょ'
            | 'ゎ'
            | 'ァ'
            | 'ィ'
            | 'ゥ'
            | 'ェ'
            | 'ォ'
            | 'ッ'
            | 'ャ'
            | 'ュ'
            | 'ョ'
            | 'ヮ'
            | ','
            | '.'
            | ':'
            | ';'
            | '?'
            | '!'
            | ')'
            | ']'
            | '}'
            | '%'
    )
}

/// Closing CJK punctuation Word lets hang past the right edge (overflowPunct).
/// Word's oracle on 0022e7ab: 、。，．」） hang; ！, ー and kana wrap instead.
fn cjk_hangs(c: char) -> bool {
    matches!(c, '、' | '。' | '，' | '．' | '」' | '）')
}

/// Width of the hanging punctuation that ends a wrap unit. The line's fit
/// test leaves it out: "予定です。" stays on one line with "。" past the edge.
fn hanging_punct_width(fonts: &Fonts, unit: &[WrapPiece<'_>]) -> f32 {
    let Some((run, tok, w)) = unit.last() else {
        return 0.0;
    };
    let body = tok.trim_end_matches(cjk_hangs);
    if body.len() == tok.len() {
        return 0.0;
    }
    let size = run.style.layout_size();
    let face = fonts.get(ink_face(fonts, &run.style, &run.text));
    let kept = face.width_pt_kern(body, size, run.style.kerns_at(size)) * run.style.hscale()
        + run.style.track * body.chars().count() as f32;
    (w - kept).max(0.0)
}

/// Kinsoku: characters a line may not end with (opening punctuation).
fn cjk_no_line_end(c: char) -> bool {
    matches!(
        c,
        '（' | '「' | '『' | '【' | '〈' | '《' | '〔' | '［' | '｛' | '(' | '[' | '{'
    )
}

/// Ideographic text breaks between characters, so a run boundary next to
/// one stays a break opportunity.
pub(super) fn is_cjk_break_char(ch: char) -> bool {
    matches!(u32::from(ch),
        0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF | 0x20000..=0x2FFFF)
}

/// A space a line breaks at and hangs past its edge. A no-break space
/// (U+00A0, figure U+2007, narrow U+202F) is a letter to the wrap: it
/// joins its neighbours, and at a line's end it takes room (Word 16 probe
/// d3a: d328fa3674's "tunách\u{a0}" wraps in its cell, "tunách " fits).
pub(super) fn is_wrap_space(c: char) -> bool {
    c.is_whitespace() && !matches!(c, '\u{a0}' | '\u{2007}' | '\u{202f}')
}

fn ws_tokens(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut prev: Option<bool> = None;
    for (i, ch) in s.char_indices() {
        let sp = is_wrap_space(ch);
        if let Some(p) = prev
            && p != sp
        {
            out.push(&s[start..i]);
            start = i;
        }
        prev = Some(sp);
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

/// Word list hanging: the first short numbering/bullet run sits in the
/// hanging gutter; the remaining runs wrap as the body at `w:ind/@w:left`.
pub(super) fn split_hanging_marker(
    runs: &[TextRun],
    hanging: bool,
) -> (Option<&TextRun>, &[TextRun]) {
    if !hanging || runs.is_empty() {
        return (None, runs);
    }
    // A typed label its own tab places is tabbed text: c73c128db4's
    // Defpara "<tab>(a)<tab>…" right-aligns "(a)" on its 1332-twip stop.
    if !runs[0].list_marker && runs[0].text.starts_with('\t') {
        return (None, runs);
    }
    if !is_list_marker_text(&runs[0].text, runs[0].list_marker) {
        return (None, runs);
    }
    (Some(&runs[0]), &runs[1..])
}

/// `numbered`: the run is the paragraph's own numbering marker. A typed
/// lone symbol hangs only as one (675bc160's typed "“" sets "“(aa)" at the
/// first-line indent in Word).
pub(super) fn is_list_marker_text(text: &str, numbered: bool) -> bool {
    let t = text.trim();
    // Cap at 8 chars: `Section 1.01` is Word-hanging on sd_2517 Título2,
    // but treating it as a marker packed 107→106pp (mini sechang −0.10).
    if t.is_empty() || t.chars().count() > 8 {
        return false;
    }
    // Any one-symbol bullet hangs like Word's: •, ●, ○, –, ■, ➢ …
    // (019f3137's "●" text sat on the bullet instead of the indent).
    let mut chars = t.chars();
    if let (Some(c), None) = (chars.next(), chars.next())
        && (c == 'o' || !c.is_alphanumeric())
    {
        return numbered;
    }
    if t.chars().any(|c| (c as u32) >= 0xF000) {
        return true;
    }
    matches!(t.chars().last(), Some('.' | ')'))
}

/// The spaces justification stretches: Word leaves those before the
/// line's last tab alone (000eb113's `3.1.<tab>Настоящий …`).
pub(super) fn inter_word_gaps(line: &[TextRun]) -> usize {
    let joined: String = line.iter().map(|r| r.text.as_str()).collect();
    let body = joined.trim_end_matches(is_wrap_space);
    let after_tab = body.rfind('\t').map_or(body, |at| &body[at..]);
    after_tab.chars().filter(|&c| c == ' ').count()
}

/// A right-to-left paragraph's line in paint order, left to right: the
/// Unicode bidi algorithm cut down to what Word's Persian and Hebrew lines
/// need. Letters of an RTL script are R, other letters L, digits EN, or
/// AN in a w:rtl run; a lone separator between two numbers joins them.
/// Neutrals between like directions take it, else the paragraph's RTL.
/// L, EN and AN sit a level above R, the line reverses, and the higher
/// runs keep their order: 00205272's rtl "صفحات(12-8)" paints "8", "-",
/// "12" left to right, the word right of them.
/// Trailing wrap space goes: it hangs off the line's end, unpainted.
/// An RTL piece keeps its logical text for HarfBuzz to reverse; a
/// neutral-only one is already visual. Their brackets mirror here:
/// rustybuzz draws "(" unmirrored where Word draws ")".
pub(super) fn bidi_visual_line(line: &[TextRun]) -> Vec<TextRun> {
    #[derive(Clone, Copy, PartialEq)]
    enum B {
        R,
        L,
        En,
        An,
        N,
    }
    let mut cs: Vec<(char, usize)> = Vec::new();
    for (ri, run) in line.iter().enumerate() {
        cs.extend(run.text.chars().map(|c| (c, ri)));
    }
    while cs.last().is_some_and(|&(c, _)| is_wrap_space(c)) {
        cs.pop();
    }
    let class = |(c, ri): (char, usize)| {
        if ('\u{0660}'..='\u{0669}').contains(&c) {
            B::An
        } else if c.is_ascii_digit() || ('\u{06F0}'..='\u{06F9}').contains(&c) {
            // Word reads a w:rtl run's digits as Arabic numbers.
            if line[ri].style.rtl { B::An } else { B::En }
        } else if c.is_alphabetic() && is_rtl_char(c) {
            B::R
        } else if c.is_alphabetic() {
            B::L
        } else {
            B::N
        }
    };
    let mut k: Vec<B> = cs.iter().map(|&c| class(c)).collect();
    let n = k.len();
    // A lone separator joins two numbers: `-` `+` and the common ones two
    // European numbers, only `,` `.` `:` `/` two Arabic ones.
    for i in 1..n.saturating_sub(1) {
        let (a, b) = (k[i - 1], k[i + 1]);
        if k[i] == B::N && a == b {
            let joins = match a {
                B::En => matches!(cs[i].0, '-' | '+' | ',' | '.' | ':' | '/'),
                B::An => matches!(cs[i].0, ',' | '.' | ':' | '/'),
                _ => false,
            };
            if joins {
                k[i] = a;
            }
        }
    }
    let strong = |b: B| if b == B::L { B::L } else { B::R };
    let neutral: Vec<bool> = k.iter().map(|b| *b == B::N).collect();
    let mut i = 0;
    while i < n {
        if k[i] != B::N {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && k[i] == B::N {
            i += 1;
        }
        let before = if start == 0 {
            B::R
        } else {
            strong(k[start - 1])
        };
        let after = if i == n { B::R } else { strong(k[i]) };
        let dir = if before == after { before } else { B::R };
        k[start..i].fill(dir);
    }
    let level: Vec<u8> = k.iter().map(|b| if *b == B::R { 1 } else { 2 }).collect();
    let mut order: Vec<usize> = (0..n).collect();
    let mut i = 0;
    while i < n {
        if level[order[i]] < 2 {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && level[order[i]] >= 2 {
            i += 1;
        }
        order[start..i].reverse();
    }
    order.reverse();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let first = order[i];
        let key = (cs[first].1, level[first], cs[first].0.is_whitespace());
        let start = i;
        while i < n && {
            let j = order[i];
            (cs[j].1, level[j], cs[j].0.is_whitespace()) == key
        } {
            i += 1;
        }
        let visual: Vec<usize> = order[start..i].to_vec();
        let text: String = if key.1 == 1 {
            let logical: String = visual.iter().rev().map(|&j| cs[j].0).collect();
            if shapes_rtl(&logical) {
                visual
                    .iter()
                    .rev()
                    .map(|&j| mirror_bracket(cs[j].0, neutral[j]))
                    .collect()
            } else {
                visual
                    .iter()
                    .map(|&j| mirror_bracket(cs[j].0, neutral[j]))
                    .collect()
            }
        } else {
            visual.iter().map(|&j| cs[j].0).collect()
        };
        out.push(line[key.0].with_text(text));
    }
    out
}

/// A neutral bracket in a right-to-left run faces the other way.
pub(super) fn mirror_bracket(c: char, neutral: bool) -> char {
    if !neutral {
        return c;
    }
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        other => other,
    }
}

/// A tab is not trailing space: it ends on its stop and fills the line up
/// to it (cb4f8b4a43's centred "Date:" and 14 tabs sit 3.6pt in, as Word
/// centres the 10080 twips they reach), so the scan stops at one.
pub(super) fn trailing_ws_pt(fonts: &Fonts, line: &[TextRun]) -> f32 {
    let mut extra = 0.0;
    for run in line.iter().rev() {
        let fid = fonts.resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = fonts.get(fid);
        let paint = run.style.layout_size();
        let trimmed = run
            .text
            .trim_end_matches(|c: char| is_wrap_space(c) && c != '\t');
        if trimmed.len() < run.text.len() {
            extra += face.width_pt(&run.text[trimmed.len()..], paint);
        }
        if !trimmed.is_empty() {
            break;
        }
    }
    extra
}

pub(super) fn peel_leading_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)> {
    let mut found = None;
    for (i, run) in runs.iter().enumerate() {
        if let Some(at) = run.text.find('\t') {
            found = Some((i, at));
            break;
        }
    }
    let (i, at) = found?;
    let mut head = runs[..i].to_vec();
    let mut lead = runs[i].clone();
    lead.text = runs[i].text[..=at].to_string();
    if !lead.text.is_empty() {
        head.push(lead);
    }
    let mut rest = Vec::new();
    let tail = &runs[i].text[at + 1..];
    if !tail.is_empty() {
        let mut after = runs[i].clone();
        after.text = tail.to_string();
        rest.push(after);
    }
    rest.extend(runs[i + 1..].iter().cloned());
    Some((head, rest))
}

pub(super) fn peel_trailing_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)> {
    let mut last = None;
    for (i, run) in runs.iter().enumerate() {
        if let Some(at) = run.text.rfind('\t') {
            last = Some((i, at));
        }
    }
    let (i, at) = last?;
    let mut prefix = runs[..i].to_vec();
    if at > 0 {
        let mut head = runs[i].clone();
        head.text = runs[i].text[..at].to_string();
        prefix.push(head);
    }
    let mut suffix = Vec::new();
    let tail = &runs[i].text[at..];
    if !tail.is_empty() {
        let mut rest = runs[i].clone();
        rest.text = tail.to_string();
        suffix.push(rest);
    }
    suffix.extend(runs[i + 1..].iter().cloned());
    Some((prefix, suffix))
}

pub(super) fn wrap_runs(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
) -> Vec<Vec<TextRun>> {
    wrap_runs_marked(fonts, runs, first_width, width, list).0
}

pub(super) fn wrap_runs_marked(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
) -> (Vec<Vec<TextRun>>, Vec<bool>) {
    wrap_runs_tabbed(fonts, runs, first_width, width, list, None)
}

/// A table cell's lines: a word wider than the cell breaks at the
/// character that reaches its edge (08c53c4f's "3127" / "1" in a year
/// column the 100% table cannot widen), as body lines do.
pub(super) fn wrap_cell_runs(
    fonts: &Fonts,
    para: &CellPara,
    first_width: f32,
    width: f32,
    narrow: (usize, f32),
) -> (Vec<Vec<TextRun>>, Vec<bool>) {
    wrap_runs_split(
        fonts,
        &para.runs,
        first_width,
        width,
        false,
        None,
        LineFit {
            squeeze: para.squeeze,
            char_break: !para.vertical,
            hang_spaces: para.hang_spaces,
            narrow,
        },
    )
}

/// How far a line's spaces may narrow to keep its last word.
#[derive(Clone, Copy, Default)]
pub(super) struct Squeeze {
    /// Share of the line's plain spaces the line may give up (0: none).
    pub(super) share: f32,
    /// The overflow must also stay within `SQUEEZE_WORD_SHARE` of the
    /// word and its space (`wrap_runs_segment`).
    pub(super) word_cap: bool,
    /// Only the spaces of a face Word narrows count (`narrows`).
    pub(super) face_bound: bool,
    /// Share of the line's measure the whole line may run over, every
    /// advance narrowing alike (`WP_SQUEEZE`; 0: none).
    pub(super) line_share: f32,
}

impl Squeeze {
    pub(super) const NONE: Self = Self {
        share: 0.0,
        word_cap: false,
        face_bound: false,
        line_share: 0.0,
    };

    /// Whether a space set in `family` counts toward the squeeze.
    fn narrows(self, family: &str) -> bool {
        !self.face_bound || compressed_space_face(family)
    }

    /// Whether a line past its measure paints its spaces narrower: one of
    /// its spaces must be one the squeeze counts.
    pub(super) fn narrows_line(self, line: &[TextRun]) -> bool {
        self.line_share > 0.0
            || self.share > 0.0
                && line
                    .iter()
                    .any(|run| run.text.contains(' ') && self.narrows(&run.style.family))
    }
}

/// Word 2013+ layout (compatibilityMode 15) keeps a justified line's last
/// word by narrowing its spaces, up to a quarter of their total width
/// (Word 16 probes 2026-10-03, bench `scripts/probe_justify*.py`: Calibri,
/// Times, Cambria, Arial, Tahoma, Verdana, 4 to 25 spaces, every boundary
/// between 0.244 and 0.251 of the spaces), and while the overflow is
/// within `SQUEEZE_WORD_SHARE` of the word.
pub(super) const JUSTIFY_SQUEEZE: Squeeze = Squeeze {
    share: 0.25,
    word_cap: true,
    face_bound: false,
    line_share: 0.0,
};

/// WordPerfect justification (`w:compat/w:wpJustification`, the Texas
/// statutes' compat set): a justified line keeps its last word while its
/// natural width is within 1.04 of the measure, whatever its spaces (3 or
/// 21) or the word's length, and paints every advance narrower alike.
/// Word 16 probes 2026-10-03 (bench `scripts/probe_wp_justify.py`, 42
/// documents): kept at 1.0385 (Courier New 10 and 9), 1.037 (Arial 12),
/// 1.038 (Times New Roman 12); wrapped at 1.042 (Arial), 1.043 (Times).
/// (The probe's Courier 14 line, once read as a wrap at 1.041, was 1.077:
/// the generator padded it, fixed in bench 802b99db4.) Courier New 12 on
/// 468pt holds 67 cells at 6.985 each; 041ec70002's lines hold 66 and 67
/// in Word's PDF.
pub(super) const WP_SQUEEZE: Squeeze = Squeeze {
    share: 0.0,
    word_cap: false,
    face_bound: false,
    line_share: 0.04,
};

/// The second bound of `JUSTIFY_SQUEEZE`: the overflow may be at most
/// this share of the last word plus one space. 23 Word boundaries at
/// 0.2pt steps (Calibri 11 and 14, Times 11 and 12, words of 10 to 38pt;
/// "ad" 11.05pt in Calibri 11 kept at 4.42, moved at 4.72; "laboris"
/// 38.2pt in Calibri 14 kept at 14.06, moved at 14.30) admit a·word +
/// b·space only for a from 0.3425 to 0.35 with b from 0.37 down to 0.29;
/// a constant or a font-size term fits none, and the earlier
/// (word + 2 spaces) / 3 breaks every one of the 23.
const SQUEEZE_WORD_SHARE: f32 = 0.345;

/// Before compatibility mode 15, `compressPunctuation` lets a line of any
/// alignment keep its last word on spaces up to a fifth narrower, however
/// short the word (Word 16 probes, Times 12 compat 14: 19.97% fits, 20.3%
/// moves the word; a 26.7pt word keeps a 17.2pt overflow).
pub(super) const PUNCT_SQUEEZE: Squeeze = Squeeze {
    share: 0.2,
    word_cap: false,
    face_bound: true,
    line_share: 0.0,
};

/// The faces whose spaces `compressPunctuation` narrows by a fifth. Word 16
/// narrows by face, not by a rule we know (probes at 12pt, compat 14): Times
/// New Roman 20%, Arial between 17.8 and 21.3%, Calibri not at all
/// (3f5209785f's Calibri lines move "Duke" whole), Georgia between 5 and
/// 10%, Courier New between 22 and 28%. Only the measured fifths squeeze.
fn compressed_space_face(family: &str) -> bool {
    ["Times New Roman", "Arial"]
        .iter()
        .any(|face| family.eq_ignore_ascii_case(face))
}

/// How a line makes room for its last word.
#[derive(Clone, Copy, Default)]
pub(super) struct LineFit {
    /// How far the line's spaces may narrow to keep the word
    /// (`WrapTabs::squeeze`).
    pub(super) squeeze: Squeeze,
    /// A word wider than the line breaks by character even without tab
    /// stops (table cells; body lines always pass their tabs).
    pub(super) char_break: bool,
    /// A space wider than the line hangs after the character before it,
    /// as every space does: compat-15 cells (Word probes c028 and n028,
    /// 2026-10-02: a legacy cell gives each such space its own line).
    pub(super) hang_spaces: bool,
    /// The first `.0` lines are `.1` points narrower: they run beside a
    /// cell float (`cell_side_float`).
    pub(super) narrow: (usize, f32),
}

/// Where a paragraph's lines start relative to the tab origin (the flow
/// left edge), so a tab can take its real jump while wrapping.
pub(super) struct WrapTabs<'a> {
    pub(super) stops: &'a [TabStop],
    pub(super) default_tab: f32,
    pub(super) first_start: f32,
    pub(super) start: f32,
    /// How far a line's spaces may narrow to keep one more word.
    pub(super) squeeze: Squeeze,
}

pub(super) fn wrap_runs_tabbed(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
    tabs: Option<&WrapTabs<'_>>,
) -> (Vec<Vec<TextRun>>, Vec<bool>) {
    let fit = LineFit {
        squeeze: tabs.map_or(Squeeze::NONE, |t| t.squeeze),
        char_break: false,
        hang_spaces: false,
        narrow: (0, 0.0),
    };
    wrap_runs_split(fonts, runs, first_width, width, list, tabs, fit)
}

fn wrap_runs_split(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
    tabs: Option<&WrapTabs<'_>>,
    fit: LineFit,
) -> (Vec<Vec<TextRun>>, Vec<bool>) {
    let mut segments: Vec<Vec<TextRun>> = vec![Vec::new()];
    // The run holding each break, per ended segment.
    let mut breaks: Vec<TextRun> = Vec::new();
    for run in runs {
        let mut parts = run.text.split('\n');
        if let Some(first) = parts.next()
            && (!first.is_empty() || run.strut)
        {
            segments
                .last_mut()
                .expect("segment")
                .push(run.with_text(first));
        }
        for part in parts {
            let mut marker = run.with_text("");
            marker.comments.clear();
            marker.pageref = None;
            marker.ref_name = None;
            marker.footnote_id = None;
            marker.ends_line = true;
            breaks.push(marker);
            segments.push(Vec::new());
            // A line opened by a break keeps the break's run even when
            // empty: its height is that run's font (0072d3b3's trailing
            // w:br line is Times 12, not the 11pt fallback).
            let mut piece = run.with_text(part);
            piece.comments.clear();
            piece.pageref = None;
            piece.ref_name = None;
            piece.ref_copy_text = false;
            piece.footnote_id = None;
            segments.last_mut().expect("segment").push(piece);
        }
    }
    let mut lines = Vec::new();
    let mut ends_br = Vec::new();
    for (i, seg) in segments.iter().enumerate() {
        let fw = if i == 0 { first_width } else { width };
        let seg_tabs = tabs.map(|t| WrapTabs {
            stops: t.stops,
            default_tab: t.default_tab,
            first_start: if i == 0 { t.first_start } else { t.start },
            start: t.start,
            squeeze: t.squeeze,
        });
        let seg_fit = LineFit {
            narrow: (fit.narrow.0.saturating_sub(lines.len()), fit.narrow.1),
            ..fit
        };
        let wrapped = wrap_runs_segment(
            fonts,
            seg,
            fw,
            width,
            list && i == 0,
            seg_tabs.as_ref(),
            seg_fit,
        );
        let more = i + 1 < segments.len();
        let n = wrapped.len();
        for (j, mut line) in wrapped.into_iter().enumerate() {
            if more
                && j + 1 == n
                && let Some(marker) = breaks.get(i)
            {
                line.push(marker.clone());
            }
            lines.push(line);
            ends_br.push(more && j + 1 == n);
        }
    }
    if lines.is_empty() {
        lines.push(vec![TextRun::new(String::new(), default_run_style())]);
        ends_br.push(false);
    }
    (lines, ends_br)
}

pub(super) fn wrap_runs_segment(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
    tabs: Option<&WrapTabs<'_>>,
    fit: LineFit,
) -> Vec<Vec<TextRun>> {
    let mut lines: Vec<Vec<TextRun>> = vec![Vec::new()];
    let mut x = 0.0;
    let mut line_i = 0usize;
    let line_limit = |i: usize| {
        (if i == 0 { first_width } else { width })
            - if i < fit.narrow.0 { fit.narrow.1 } else { 0.0 }
    };
    if list {
        let bullet = "• ";
        x = fonts.get(FaceId::CarlitoRegular).width_pt(bullet, 11.0);
        lines[0].push(TextRun::new(
            bullet,
            runs.first()
                .map_or_else(default_run_style, |r| r.style.clone()),
        ));
    }
    // Word breaks only at whitespace, after a hyphen, or inside a URL. A
    // run boundary inside a word is not a break (`birds` + bold `.`), so
    // pieces glue into units and a unit wraps whole (fixtures_500 014babb2).
    let mut units: Vec<(Vec<WrapPiece<'_>>, bool)> = Vec::new();
    let mut open = false;
    for run in runs {
        if run.math.is_some() || run.pic.is_some() {
            let w = run
                .math
                .as_ref()
                .map(|math| math.width)
                .or_else(|| run.pic.as_ref().map(|img| img.w.max(1.0)))
                .unwrap_or(0.0);
            units.push((vec![(run, run.text.as_str(), w)], false));
            open = false;
            continue;
        }
        if run.strut && run.text.is_empty() {
            units.push((vec![(run, "", 0.0)], false));
            open = false;
            continue;
        }
        let run_face = fonts.get(ink_face(fonts, &run.style, &run.text));
        for tok in ws_tokens(&run.text) {
            let url = url_wrap_pieces(tok);
            let pieces: Vec<&str> = if url.len() > 1 {
                url
            } else {
                hyphen_wrap_pieces(tok, !run.style.ideograph_words, !run.style.punct_compress)
            };
            let last = pieces.len().saturating_sub(1);
            for (i, tok) in pieces.into_iter().enumerate() {
                // Tabs jump at paint time; counting .notdef width packed wraps.
                let w = if tok.contains('\t') {
                    0.0
                } else {
                    let face = run_face;
                    let size = run.style.layout_size();
                    // w:spacing tracking widens every letter, as painted
                    // (00080142's +0.35pt Arial packed too many words).
                    if run.style.fixed_advance > 0.0 {
                        (run.style.fixed_advance * run.style.hscale() + run.style.track)
                            * tok.chars().count() as f32
                    } else {
                        face.width_pt_kern(tok, size, run.style.kerns_at(size)) * run.style.hscale()
                            + run.style.track * tok.chars().count() as f32
                    }
                };
                let is_space = tok.chars().all(is_wrap_space);
                let glue = open
                    && !is_space
                    && units.last().is_some_and(|(u, _)| {
                        let prev = u.last().and_then(|(_, t, _)| t.chars().last());
                        run.style.ideograph_words
                            || (!prev.is_some_and(is_cjk_break_char)
                                && !tok.chars().next().is_some_and(is_cjk_break_char))
                    });
                if glue && let Some((unit, _)) = units.last_mut() {
                    unit.push((run, tok, w));
                } else {
                    units.push((vec![(run, tok, w)], is_space));
                }
                open = !is_space && i == last && !breaks_after(run, tok);
            }
        }
    }
    let mut line_spaces = 0.0_f32;
    // One space of the last gap: the squeeze test weighs the moved word's room.
    let mut space_w = 0.0_f32;
    // The text after each unit up to the next tab: a centre or right stop
    // sets it back by half or all of it, as at paint time. Measured from
    // the stop alone, 5fb9cedf's centred title overflowed and wrapped.
    let tails: Vec<f32> = (0..units.len())
        .map(|i| {
            units[i + 1..]
                .iter()
                .take_while(|(u, _)| u.iter().all(|(_, tok, _)| !tok.contains('\t')))
                .map(|(u, _)| u.iter().map(|(_, _, w)| w).sum::<f32>())
                .sum()
        })
        .collect();
    for (ui, (unit, is_space)) in units.into_iter().enumerate() {
        let mut w: f32 = unit.iter().map(|(_, _, w)| w).sum();
        // A tab jumps to the next stop from where it stands (00996ee5's
        // leading tab took 35pt of the first line in Word). Also whether
        // its first tab has no stop of its own left and its default stop
        // lies past the line's right edge.
        let tab_unit = is_space && unit.iter().any(|(_, tok, _)| tok.contains('\t'));
        let tab_w = |t: &WrapTabs<'_>, line_i: usize, x: f32| -> (f32, bool) {
            let start = if line_i == 0 { t.first_start } else { t.start };
            let right = start + line_limit(line_i);
            let mut past = false;
            let mut first_tab = true;
            let mut pos = start + x;
            let char_w = |run: &TextRun, ch: char| {
                let fid = fonts.resolve(&run.style.family, run.style.bold, run.style.italic);
                fonts
                    .get(fid)
                    .width_pt(ch.encode_utf8(&mut [0; 4]), run.style.layout_size())
                    * run.style.hscale()
            };
            let chars: Vec<(&TextRun, char)> = unit
                .iter()
                .flat_map(|(run, tok, _)| tok.chars().map(move |ch| (*run, ch)))
                .collect();
            for (k, &(run, ch)) in chars.iter().enumerate() {
                {
                    if ch == '\t' {
                        let stop = next_tab_stop(pos, 0.0, t.stops, t.default_tab);
                        if first_tab {
                            first_tab = false;
                            past = stop.pos > right + 0.01
                                && !t
                                    .stops
                                    .iter()
                                    .any(|s| s.align != TabAlign::Bar && s.pos > pos + 0.01);
                        }
                        let rest = &chars[k + 1..];
                        let own: f32 = rest
                            .iter()
                            .take_while(|(_, c)| *c != '\t')
                            .map(|&(r, c)| char_w(r, c))
                            .sum();
                        let after = own
                            + if rest.iter().any(|(_, c)| *c == '\t') {
                                0.0
                            } else {
                                tails[ui]
                            };
                        pos = match stop.align {
                            TabAlign::Center => (stop.pos - after * 0.5).max(pos),
                            TabAlign::Right => (stop.pos - after).max(pos),
                            _ => stop.pos,
                        };
                    } else {
                        let fid =
                            fonts.resolve(&run.style.family, run.style.bold, run.style.italic);
                        let size = run.style.layout_size();
                        pos += fonts.get(fid).width_pt(ch.encode_utf8(&mut [0; 4]), size)
                            * run.style.hscale();
                    }
                }
            }
            (pos - start - x, past)
        };
        let mut tab_past = false;
        if let Some(t) = tabs
            && tab_unit
        {
            (w, tab_past) = tab_w(t, line_i, x);
        }
        let limit = line_limit(line_i);
        // Word squeezes only while the overflow is within
        // `SQUEEZE_WORD_SHARE` of the word and its space: d06f02170c's
        // "times" (Times 11, 23.8pt) runs 11.07pt over and moves although a
        // quarter of its 19 spaces is 13.1pt; Word's cap for it is 9.17.
        let overflow = x + w - limit;
        let squeezed = fit.squeeze.share > 0.0
            && overflow <= fit.squeeze.share * line_spaces
            && (!fit.squeeze.word_cap || overflow <= SQUEEZE_WORD_SHARE * (w + space_w))
            || fit.squeeze.line_share > 0.0 && overflow <= fit.squeeze.line_share * limit;
        let hang = hanging_punct_width(fonts, &unit);
        // A space hangs past the edge unless one space is wider than the
        // line: then each is a line of its own, as each character is
        // (2b479f55f8's 1.4pt column; Word stacks "r", " ", "c"), except
        // in a compat-15 cell (`LineFit::hang_spaces`). A run of fill-in
        // spaces wider than its cell still hangs (file_146).
        let chars: usize = unit.iter().map(|(_, tok, _)| tok.chars().count()).sum();
        let space_stacks = !fit.hang_spaces && w / chars.max(1) as f32 > limit;
        // A tab whose default stop is past the edge starts the next line
        // and resolves from its start (air_pollution_permit_form's six
        // dotted tabs are three lines in Word).
        let breaks = !is_space || space_stacks || tab_past;
        if breaks && x + w - hang > limit && x > 0.0 && !squeezed {
            lines.push(Vec::new());
            line_i += 1;
            x = 0.0;
            line_spaces = 0.0;
            if let Some(t) = tabs
                && tab_unit
            {
                w = tab_w(t, line_i, x).0;
            }
        }
        // A word wider than the whole line breaks at the character that
        // reaches the edge (001472bb). A
        // run that overshoots by a hair (002ed0b9's dotted fill-in lines,
        // one line in Word) is measurement noise, not a break.
        let limit = line_limit(line_i);
        // Body lines and cells: a table column autofits its longest word
        // (0129b302's "19.720.000") where it has room, so only a word
        // wider than its laid-out cell breaks; CJK text already breaks per
        // character where our fallback faces run wide (002c5410).
        let math_unit = unit
            .iter()
            .any(|(run, _, _)| run.math.is_some() || run.pic.is_some());
        let cjk = unit
            .iter()
            .any(|(run, tok, _)| !run.style.ideograph_words && tok.chars().any(is_cjk));
        let ideograph_word = unit
            .iter()
            .any(|(run, tok, _)| run.style.ideograph_words && tok.chars().any(is_cjk));
        if !math_unit
            && (tabs.is_some() || fit.char_break || ideograph_word)
            && !cjk
            && !is_space
            && w > limit * 1.02
            && limit > 0.0
        {
            // An ideograph word starts its own line before it splits
            // (005919f8: Word leaves "1." alone above the sentence).
            if x > 0.0 && ideograph_word {
                lines.push(Vec::new());
                line_i += 1;
                x = 0.0;
                line_spaces = 0.0;
            }
            for (run, tok, _) in unit {
                // Ideographs under a Latin eastAsia font paint in the
                // fallback face; measure them in it, as the unit was.
                let face = if ideograph_word {
                    fonts.get(ink_face(fonts, &run.style, tok))
                } else {
                    fonts.get(fonts.resolve(
                        paint_family(&run.style, tok),
                        run.style.bold,
                        run.style.italic,
                    ))
                };
                let size = run.style.layout_size();
                for ch in tok.chars() {
                    let piece = ch.to_string();
                    let cw = face.width_pt(&piece, size) * run.style.hscale();
                    let limit = line_limit(line_i);
                    // Closing punctuation stays with the character before
                    // it, unless the mark alone is wider than the line:
                    // a column narrower than a letter gives "’", ")", ","
                    // a line each (Word probes p0-p8/l5, 2026-10-02).
                    let starts_line =
                        cw > limit || (!cjk_no_line_start(ch) && !matches!(ch, '’' | '”'));
                    if x + cw > limit && x > 0.0 && starts_line {
                        lines.push(Vec::new());
                        line_i += 1;
                        x = 0.0;
                        line_spaces = 0.0;
                    }
                    x += cw;
                    if let Some(last) = lines.last_mut().and_then(|line| line.last_mut())
                        && style_eq(&last.style, &run.style)
                        && last.field == run.field
                        && last.comment_marker == run.comment_marker
                        && last.pageref.is_none()
                        && last.ref_name.is_none()
                        && last.footnote_id.is_none()
                        && run.pageref.is_none()
                        && run.ref_name.is_none()
                        && run.footnote_id.is_none()
                        && same_style_refs(last, run)
                    {
                        last.text.push(ch);
                    } else if let Some(line) = lines.last_mut() {
                        line.push(run.with_text(piece));
                    }
                }
            }
            continue;
        }
        if is_space && unit.iter().all(|(_, tok, _)| !tok.contains('\t')) {
            // Compressed punctuation narrows only the spaces of some faces
            // (`compressed_space_face`).
            line_spaces += unit
                .iter()
                .filter(|(run, _, _)| fit.squeeze.narrows(&run.style.family))
                .map(|(_, _, pw)| pw)
                .sum::<f32>();
            space_w = w / chars.max(1) as f32;
        }
        x += w;
        for (run, tok, _) in unit {
            if let Some(last) = lines.last_mut().and_then(|line| line.last_mut())
                && style_eq(&last.style, &run.style)
                && !run.strut
                && last.math.is_none()
                && run.math.is_none()
                && last.pic.is_none()
                && run.pic.is_none()
                // A run carrying comments stays its own run: its start is
                // where the balloon connectors anchor, and the merge would
                // drop its notes.
                && run.comments.is_empty()
                // A field result stays its own run: a PAGE field's is
                // repainted per page (d45aa3d5's "Page" + PAGE footer box).
                && last.field == run.field
                        && last.comment_marker == run.comment_marker
                && last.pageref.is_none()
                && run.pageref.is_none()
                && last.ref_name.is_none()
                && run.ref_name.is_none()
                && last.footnote_id.is_none()
                && run.footnote_id.is_none()
                && same_style_refs(last, run)
            {
                last.text.push_str(tok);
            } else if let Some(line) = lines.last_mut() {
                line.push(run.with_text(tok));
            }
        }
    }
    if lines.len() == 1 && lines[0].is_empty() {
        // An empty line keeps its own run's style (the break that opened
        // it), not the 11pt fallback (0072d3b3).
        let style = runs
            .first()
            .map_or_else(default_run_style, |r| r.style.clone());
        lines[0].push(TextRun::new(String::new(), style));
    }
    lines
}
