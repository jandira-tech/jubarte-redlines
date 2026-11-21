// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Tab stops, the theme, the style sheet, run and paragraph properties,
//! script font selection, colours and section page setup.

use super::*;

pub(super) fn twip(v: f32) -> f32 {
    v / 20.0
}

/// Word Save-as-PDF A4 is 595.2×841.92 pt for `pgSz` 11906×16838 twips
/// (plan.md Step 10 I). Linear twip/20 is 595.3×841.9.
fn word_pgsz_pt(raw: &str, pt: f32) -> f32 {
    let Some(twips) = parse_finite_f32(raw.trim()) else {
        return pt;
    };
    if (twips - 11906.0).abs() < 0.5 {
        595.2
    } else if (twips - 16838.0).abs() < 0.5 {
        841.92
    } else {
        pt
    }
}

/// Explicit `w:tabs` first, then Word's 720-twip (0.5in) factory grid.
/// `w:tab/@w:pos` is from the left margin (ECMA-376 17.3.1.38). Treating
/// it as page-edge made sd_2517's 2520-twip Sumrio left tab sit at 126pt,
/// already behind "lorem 1.01", so the first tab fired the 8640-twip
/// dot leader and the TOC fell a page behind Word (p3 9.2 vs 11-1).
pub(super) fn next_tab_stop(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> TabStop {
    for &stop in stops {
        // `bar` is a drawn rule, not a destination (ECMA-376 17.3.1.38).
        if stop.align == TabAlign::Bar {
            continue;
        }
        let abs = origin + stop.pos;
        if abs > x + 0.01 {
            return TabStop {
                pos: abs,
                align: stop.align,
                leader: stop.leader,
                numbering: stop.numbering,
            };
        }
    }
    let grid = if default_tab > 0.5 { default_tab } else { 36.0 };
    // A tab standing exactly on a default stop goes to the next one: 3 x
    // 35.4 divides to 2.9999 in f32 and sent the tab nowhere (0081ba58's
    // three tabs before its logo).
    let rel = (x - origin).max(0.0) + 0.01;
    TabStop {
        pos: origin + ((rel / grid).floor() + 1.0) * grid,
        align: TabAlign::Left,
        leader: TabLeader::None,
        numbering: false,
    }
}

/// How far a list label may pass the indent before its suffix tab moves
/// the text to the next stop: past rounding, not a real overrun.
pub(super) const LABEL_OVERRUN: f32 = 0.05;

pub(super) fn next_tab_x(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> f32 {
    next_tab_stop(x, origin, stops, default_tab).pos
}

/// Where a tab at `x` sends the text after it: the next stop, less the
/// following text's width (right), half of it (center) or its integer part
/// (decimal).
pub(super) fn tab_dest(
    x: f32,
    origin: f32,
    stops: &[TabStop],
    default_tab: f32,
    after_w: f32,
    decimal_w: f32,
) -> f32 {
    let stop = next_tab_stop(x, origin, stops, default_tab);
    match stop.align {
        TabAlign::Left | TabAlign::Bar => stop.pos,
        TabAlign::Right => (stop.pos - after_w).max(x),
        TabAlign::Center => (stop.pos - after_w * 0.5).max(x),
        TabAlign::Decimal => (stop.pos - decimal_w).max(x),
    }
}

/// A pPr's `w:tabs` on top of the inherited stops: a stop replaces the
/// one at its position and `val=clear` removes it, the rest stay
/// (ECMA-376 17.3.1.38; 010684299d's footer keeps its style's center and
/// right stops under the paragraph's own right stop).
pub(super) fn apply_tab_stops(stops: &mut Vec<TabStop>, dom: &Dom, ppr: NodeId) {
    let Some(tabs) = first_named(dom, ppr, "tabs") else {
        return;
    };
    let same = |a: f32, b: f32| (a - b).abs() < 0.05;
    for tab in dom.elements(tabs, Some(&W::name("tab"))) {
        if attr_any(dom, tab, "val") == Some("clear")
            && let Some(pos) = attr_any(dom, tab, "pos").and_then(parse_finite_f32)
        {
            stops.retain(|s| !same(s.pos, twip(pos)));
        }
    }
    for stop in parse_tab_stops(dom, ppr) {
        stops.retain(|s| !same(s.pos, stop.pos));
        stops.push(stop);
    }
    stops.sort_by(|a, b| a.pos.total_cmp(&b.pos));
}

/// `w:tabs/w:tab`. `val=num` is a numbering left stop (ECMA-376 17.3.1.38).
pub(super) fn parse_tab_stops(dom: &Dom, ppr: NodeId) -> Vec<TabStop> {
    let Some(tabs) = first_named(dom, ppr, "tabs") else {
        return Vec::new();
    };
    let mut stops = Vec::new();
    for tab in dom.elements(tabs, Some(&W::name("tab"))) {
        let val = attr_any(dom, tab, "val").unwrap_or("left");
        if val == "clear" {
            continue;
        }
        if let Some(pos) = attr_any(dom, tab, "pos").and_then(parse_finite_f32) {
            let align = match val {
                // ISO Strict ST_TabJc: end/start are LTR right/left
                // (Strict01 TOC val=end leader=dot pos=9350).
                "right" | "end" => TabAlign::Right,
                "center" => TabAlign::Center,
                "decimal" => TabAlign::Decimal,
                "bar" => TabAlign::Bar,
                _ => TabAlign::Left,
            };
            let leader = match attr_any(dom, tab, "leader").unwrap_or("") {
                "dot" | "middleDot" => TabLeader::Dot,
                "hyphen" => TabLeader::Hyphen,
                "underscore" => TabLeader::Underscore,
                "heavy" => TabLeader::Heavy,
                _ => TabLeader::None,
            };
            stops.push(TabStop {
                pos: twip(pos),
                align,
                leader,
                numbering: val == "num",
            });
        }
    }
    stops.sort_by(|a, b| {
        a.pos
            .partial_cmp(&b.pos)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    stops
}

pub(super) fn merge_tab_stops(into: &mut Vec<TabStop>, extra: &[TabStop]) {
    for &stop in extra {
        if !into.iter().any(|t| (t.pos - stop.pos).abs() < 0.5) {
            into.push(stop);
        }
    }
    into.sort_by(|a, b| {
        a.pos
            .partial_cmp(&b.pos)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// OOXML length: bare number is twips; `pt` / `in` / `cm` / `mm` are already units.
/// A non-finite operand, or a conversion that overflows (`1e38in`), is absent.
pub(super) fn parse_len(s: &str) -> Option<f32> {
    let t = s.trim();
    let points = if let Some(rest) = t.strip_suffix("pt") {
        parse_finite_f32(rest.trim())?
    } else if let Some(rest) = t.strip_suffix("in") {
        parse_finite_f32(rest.trim())? * 72.0
    } else if let Some(rest) = t.strip_suffix("cm") {
        parse_finite_f32(rest.trim())? * 72.0 / 2.54
    } else if let Some(rest) = t.strip_suffix("mm") {
        parse_finite_f32(rest.trim())? * 72.0 / 25.4
    } else {
        twip(parse_finite_f32(t)?)
    };
    points.is_finite().then_some(points)
}

pub(super) fn load_theme(pkg: &PartFs) -> ThemeFonts {
    let Some(xml) = pkg
        .part_string(&main_rel_part(pkg, "theme", "word/theme/theme1.xml"))
        .or_else(|| pkg.part_string("word/theme/theme2.xml"))
    else {
        return ThemeFonts::default();
    };
    parse_theme_xml(&xml)
}

pub(super) fn parse_theme_xml(xml: &str) -> ThemeFonts {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let Some(root) = dom.root(doc) else {
        return ThemeFonts::default();
    };
    let typeface = |parent_local: &str, child_local: &str| -> Option<String> {
        let parent = dom
            .descendants(root, Some(&A::name(parent_local)))
            .into_iter()
            .next()?;
        let face = dom
            .descendants(parent, Some(&A::name(child_local)))
            .into_iter()
            .next()?;
        // Stock Word themes write `typeface=""` for unused ea/cs slots.
        attr_any(&dom, face, "typeface")
            .filter(|t| !t.trim().is_empty())
            .map(str::to_string)
    };
    let mut colors = HashMap::new();
    if let Some(scheme) = dom
        .descendants(root, Some(&A::name("clrScheme")))
        .into_iter()
        .next()
    {
        for i in 0..dom.child_count(scheme) {
            let child = dom.child_at(scheme, i);
            let Some(slot) = dom.name(child).map(|n| n.local_name().to_string()) else {
                continue;
            };
            let hex = descendants_local(&dom, child, "srgbClr")
                .into_iter()
                .find_map(|n| attr_any(&dom, n, "val"))
                .or_else(|| {
                    descendants_local(&dom, child, "sysClr")
                        .into_iter()
                        .find_map(|n| attr_any(&dom, n, "lastClr"))
                });
            if let Some(hex) = hex
                && let Some(rgb) = parse_hex_color(hex)
            {
                colors.insert(slot, rgb);
            }
        }
    }
    let mut script_fonts = HashMap::new();
    collect_script_fonts(&dom, root, "majorFont", "major", &mut script_fonts);
    collect_script_fonts(&dom, root, "minorFont", "minor", &mut script_fonts);
    let probe = ThemeFonts {
        colors: colors.clone(),
        ..ThemeFonts::default()
    };
    let fmt = parse_fmt_scheme(&dom, root, &probe);
    ThemeFonts {
        major: typeface("majorFont", "latin"),
        minor: typeface("minorFont", "latin"),
        major_ea: typeface("majorFont", "ea"),
        minor_ea: typeface("minorFont", "ea"),
        major_cs: typeface("majorFont", "cs"),
        minor_cs: typeface("minorFont", "cs"),
        script_fonts,
        colors,
        fill_styles: fmt.fills,
        ln_widths: fmt.widths,
        ln_colors: fmt.lines,
    }
}

struct FmtScheme {
    fills: [Option<[f32; 3]>; 3],
    widths: [Option<f32>; 3],
    lines: [Option<[f32; 3]>; 3],
}

/// `a:fmtScheme` fillStyleLst / lnStyleLst (ECMA-376 20.1.4.1.18).
/// Direct children only; `phClr` stays `None` so fillRef/lnRef schemeClr
/// still substitutes (Office default).
fn parse_fmt_scheme(dom: &Dom, root: NodeId, probe: &ThemeFonts) -> FmtScheme {
    let mut fills = [None; 3];
    if let Some(lst) = dom
        .descendants(root, Some(&A::name("fillStyleLst")))
        .into_iter()
        .next()
    {
        let mut i = 0usize;
        for c in 0..dom.child_count(lst) {
            let child = dom.child_at(lst, c);
            if dom.name(child).is_none() {
                continue;
            }
            if i >= 3 {
                break;
            }
            if local_name_is(dom, child, "solidFill") {
                fills[i] = scheme_color(dom, child, probe);
            }
            i += 1;
        }
    }
    let mut widths = [None; 3];
    let mut lines = [None; 3];
    if let Some(lst) = dom
        .descendants(root, Some(&A::name("lnStyleLst")))
        .into_iter()
        .next()
    {
        let mut i = 0usize;
        for c in 0..dom.child_count(lst) {
            let child = dom.child_at(lst, c);
            if !local_name_is(dom, child, "ln") {
                continue;
            }
            if i >= 3 {
                break;
            }
            if let Some(w) = attr_any(dom, child, "w").and_then(parse_finite_f64)
                && let Some(pt) = narrow_finite_f32(w / 12700.0)
            {
                widths[i] = Some(pt.clamp(0.4, 4.0));
            }
            lines[i] = scheme_color(dom, child, probe);
            i += 1;
        }
    }
    FmtScheme {
        fills,
        widths,
        lines,
    }
}

fn collect_script_fonts(
    dom: &Dom,
    root: NodeId,
    parent_local: &str,
    bucket: &str,
    out: &mut HashMap<(String, String), String>,
) {
    let Some(parent) = dom
        .descendants(root, Some(&A::name(parent_local)))
        .into_iter()
        .next()
    else {
        return;
    };
    for font in descendants_local(dom, parent, "font") {
        let Some(script) = attr_any(dom, font, "script") else {
            continue;
        };
        let Some(face) = attr_any(dom, font, "typeface") else {
            continue;
        };
        out.insert((bucket.to_string(), script.to_string()), face.to_string());
    }
}

pub(super) fn load_stylesheet(pkg: &PartFs) -> StyleSheet {
    let theme = load_theme(pkg);
    let mut defaults = Defaults::word();
    defaults.legacy_compat = settings_compat_mode(pkg) < 15;
    defaults.run.punct_compress = defaults.legacy_compat
        && settings_character_spacing(pkg) != CharacterSpacing::DoNotCompress;
    defaults.punct_squeeze = settings_punct_squeeze(pkg);
    defaults.wp_justify = settings_flag(pkg, "wpJustification");
    defaults.para.sum_spacing = settings_flag(pkg, "doNotUseHTMLParagraphAutoSpacing");
    let mut raw: std::collections::HashMap<String, RawStyle> = std::collections::HashMap::new();
    let Some(xml) = pkg.part_string(&main_rel_part(pkg, "styles", "word/styles.xml")) else {
        // No styles part at all: Word 16 lays the package out on the same
        // stock defaults a styles part without pPrDefault / rPrDefault
        // takes below — after=160 line=278, Aptos 12pt kern 1pt (#1725).
        defaults.word_stock_para();
        defaults.word_stock_run();
        return StyleSheet {
            defaults,
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme,
            latent: true,
            default_table: false,
        };
    };
    let linked = settings_link_styles(pkg);
    let xml = if linked {
        link_template_styles(&xml)
    } else {
        xml
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return StyleSheet {
            defaults,
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme,
            latent: true,
            default_table: false,
        };
    };
    // styles.xml is present. An empty pPrDefault + empty Normal (the
    // sample_document / eigenpal family) means after=0, and with no `w:line`
    // anywhere Word is single-spaced (fixtures_500 003599e1 TNR 13 lines
    // 14.88pt apart). A docDefaults with no pPrDefault at all lays out at
    // 1.15 lines and 8pt after (PHPWord's 00046848 / 000312ea: 13.3pt
    // lines, 21.3pt paragraph steps, 34.3pt table rows).
    // docDefaults below can still set them when the file specifies them.
    let has = |name: &str| !dom.descendants(root, Some(&W::name(name))).is_empty();
    let bare_defaults = has("docDefaults") && !has("pPrDefault");
    if bare_defaults {
        defaults.para.after = 8.0;
    } else {
        defaults.para.after = 0.0;
        defaults.para.line_mult = 1.0;
    }
    // And the size: with no `w:sz` anywhere Word runs at the OOXML
    // default 10pt, not the new-document 11 (fixtures_500 003c9ddd).
    defaults.run.size = 10.0;
    // A missing pPrDefault / rPrDefault takes Word's built-in defaults:
    // after=160 line=278, Aptos 12pt kern 1pt. The oracle reproduces
    // fixtures_500 015beda9 (no docDefaults) line for line from exactly
    // those; `<w:docDefaults/>` lays out the same. Present but empty, they
    // mean the OOXML ones: single-spaced Times New Roman.
    if !has("pPrDefault") {
        defaults.word_stock_para();
    }
    if !has("rPrDefault") {
        defaults.word_stock_run();
    }
    let doc_defaults = dom
        .descendants(root, Some(&W::name("docDefaults")))
        .into_iter()
        .next();
    // An rPrDefault that names no face is the OOXML default, Times New
    // Roman, whatever theme part the package carries (theme faces reach a
    // run only through asciiTheme attributes): Word's PDFs of 0edc50c464,
    // cbb3bab843, 0a1badc333, 43432ba9ab and 6d510ca476 (with a theme) set
    // the body in Times New Roman 10, not the factory Calibri.
    if has("rPrDefault")
        && doc_defaults.is_some_and(|dd| dom.descendants(dd, Some(&W::name("rFonts"))).is_empty())
    {
        defaults.run.family = "Times New Roman".into();
        defaults.run.family_hansi = Some("Times New Roman".into());
        defaults.run.family_cs = Some("Times New Roman".into());
    }
    if let Some(dd) = doc_defaults {
        if let Some(rpr) = first_named(&dom, dd, "rPr") {
            apply_rpr(&dom, rpr, &mut defaults.run, &theme);
            // Word squeezes compressed punctuation only while the document
            // default names no fonts (Word 16 probes 2026-10-01: Times New
            // Roman on the runs squeezes; the same face, Arial or Calibri
            // in docDefaults' w:rFonts does not, 3f5209785f's "Duke" and
            // "vocational" among them; an empty docDefaults squeezes).
            if first_named(&dom, rpr, "rFonts").is_some() {
                defaults.punct_squeeze = false;
            }
        }
        if let Some(ppr) = first_named(&dom, dd, "pPr") {
            apply_ppr(&dom, ppr, &mut defaults.para);
        }
    }
    // docDefaults' own w:sz, the size a legacy table style falls back to.
    let doc_default_size = dom
        .descendants(root, Some(&W::name("docDefaults")))
        .into_iter()
        .next()
        .and_then(|dd| first_named(&dom, dd, "rPr"))
        .and_then(|rpr| first_named(&dom, rpr, "sz"))
        .and_then(|sz| attr_any(&dom, sz, "val"))
        .and_then(parse_finite_f32)
        .map(|half| half / 2.0);
    let mut tables = HashMap::new();
    let mut implicit_para: Option<String> = None;
    let mut style_names: HashMap<String, String> = HashMap::new();
    let mut not_para: std::collections::HashSet<String> = std::collections::HashSet::new();
    for style in dom.descendants(root, Some(&W::name("style"))) {
        let Some(sid) = dom.attribute(style, &W::style_id()) else {
            continue;
        };
        if let Some(nm) = first_named(&dom, style, "name").and_then(|n| dom.attribute(n, &W::val()))
        {
            style_names.insert(sid.to_string(), nm.to_string());
        }
        if attr_any(&dom, style, "type") == Some("table") {
            tables.insert(
                sid.to_string(),
                parse_tbl_style(&dom, style, &defaults, &theme),
            );
            continue;
        }
        if matches!(
            attr_any(&dom, style, "type"),
            Some("character" | "numbering")
        ) {
            not_para.insert(sid.to_string());
        }
        // Only the default *paragraph* style becomes doc defaults.
        // TableNormal / NoList also carry w:default="1" and would
        // overwrite Normal (comments-lots Aptos 10.5 → Calibri 11).
        if attr_any(&dom, style, "default") == Some("1")
            && matches!(attr_any(&dom, style, "type"), Some("paragraph") | None)
        {
            implicit_para = Some(sid.to_string());
        }
        let based = first_named(&dom, style, "basedOn")
            .and_then(|n| dom.attribute(n, &W::val()).map(str::to_string));
        let ppr = dom.element(style, &W::p_pr());
        let rpr = dom.element(style, &W::r_pr());
        // `w:link` (ECMA-376 17.7.4.6) only pairs a paragraph and a
        // character style in Word's UI; it imports no properties. Word
        // paints I_am_sharing Heading1 in the paragraph style's 1F4E79,
        // not linked Heading1Char's 365F91.
        raw.insert(sid.to_string(), RawStyle { based, ppr, rpr });
    }
    let mut by_id = HashMap::new();
    let ids: Vec<String> = raw.keys().cloned().collect();
    for id in ids {
        let (mut para, run) = resolve_named(&dom, &raw, &defaults, &theme, &id, 0);
        para.style_id = id.clone();
        if let Some(nm) = style_names.get(&id) {
            para.style_name = nm.clone();
        }
        let (num_id, ilvl) = resolve_num_pr(&dom, &raw, &id, 0);
        let chain_sets = |name: &str, para_side: bool| {
            let mut cur = Some(id.as_str());
            for _ in 0..12 {
                let Some(r) = cur.and_then(|c| raw.get(c)) else {
                    break;
                };
                let pr = if para_side { r.ppr } else { r.rpr };
                if pr.is_some_and(|pr| first_named(&dom, pr, name).is_some()) {
                    return true;
                }
                cur = r.based.as_deref();
            }
            false
        };
        // Only an rFonts naming a Latin slot sets the face: A0's
        // cs-only rFonts left Body Text's Arial (docxide czech_census).
        let sets_family = {
            let mut cur = Some(id.as_str());
            let mut hit = false;
            for _ in 0..12 {
                let Some(r) = cur.and_then(|c| raw.get(c)) else {
                    break;
                };
                if let Some(fonts) = r.rpr.and_then(|pr| first_named(&dom, pr, "rFonts"))
                    && ["ascii", "hAnsi", "asciiTheme", "hAnsiTheme"]
                        .iter()
                        .any(|n| attr_any(&dom, fonts, n).is_some())
                {
                    hit = true;
                    break;
                }
                cur = r.based.as_deref();
            }
            hit
        };
        let sets_size = chain_sets("sz", false);
        let sets_jc = chain_sets("jc", true);
        let hidden = {
            let mut cur = Some(id.as_str());
            let mut on = None;
            for _ in 0..12 {
                let Some(r) = cur.and_then(|c| raw.get(c)) else {
                    break;
                };
                if let Some(v) = r.rpr.and_then(|pr| first_named(&dom, pr, "vanish")) {
                    on = Some(!val_is_false(&dom, Some(v)));
                    break;
                }
                cur = r.based.as_deref();
            }
            on
        };
        let chain_ind = |names: &[&str]| {
            let mut cur = Some(id.as_str());
            for _ in 0..12 {
                let Some(r) = cur.and_then(|c| raw.get(c)) else {
                    break;
                };
                if let Some(ind) = r.ppr.and_then(|pr| first_named(&dom, pr, "ind"))
                    && names.iter().any(|n| attr_any(&dom, ind, n).is_some())
                {
                    return true;
                }
                cur = r.based.as_deref();
            }
            false
        };
        let chain_spacing = |names: &[&str]| {
            let mut cur = Some(id.as_str());
            for _ in 0..12 {
                let Some(r) = cur.and_then(|c| raw.get(c)) else {
                    break;
                };
                if let Some(sp) = r.ppr.and_then(|pr| first_named(&dom, pr, "spacing"))
                    && names.iter().any(|n| attr_any(&dom, sp, n).is_some())
                {
                    return true;
                }
                cur = r.based.as_deref();
            }
            false
        };
        let sets_spacing = [
            chain_spacing(&["after", "afterLines", "afterAutospacing"]),
            chain_spacing(&["before", "beforeLines", "beforeAutospacing"]),
            chain_spacing(&["line", "lineRule"]),
        ];
        let style_not_para = not_para.contains(&id);
        let sets_ind = (
            chain_ind(&["left", "start"]),
            chain_ind(&["hanging", "firstLine"]),
        );
        let mut chain = Vec::new();
        let mut cur = Some(id.as_str());
        for _ in 0..12 {
            let Some(r) = cur.and_then(|c| raw.get(c)) else {
                break;
            };
            chain.push(r.ppr);
            cur = r.based.as_deref();
        }
        let mut frame: Vec<(String, String)> = Vec::new();
        for fp in chain
            .into_iter()
            .rev()
            .flatten()
            .filter_map(|pr| first_named(&dom, pr, "framePr"))
        {
            merge_frame_attrs(&dom, fp, &mut frame);
        }
        by_id.insert(
            id,
            NamedStyle {
                para,
                run,
                num_id,
                ilvl,
                sets_size,
                sets_family,
                sets_ind,
                sets_jc,
                sets_spacing,
                not_para: style_not_para,
                frame,
                hidden,
            },
        );
    }
    let implicit_id = implicit_para.as_deref().unwrap_or("Normal");
    // What the default paragraph style's own spacing sets (not inherited).
    if let Some(sp) = dom
        .descendants(root, Some(&W::name("style")))
        .into_iter()
        .find(|st| dom.attribute(*st, &W::style_id()) == Some(implicit_id))
        .and_then(|st| dom.element(st, &W::p_pr()))
        .and_then(|ppr| first_named(&dom, ppr, "spacing"))
    {
        let sets = |names: &[&str]| names.iter().any(|n| attr_any(&dom, sp, n).is_some());
        defaults.normal_spacing = [
            sets(&["after", "afterAutospacing"]),
            sets(&["before", "beforeAutospacing"]),
            sets(&["line"]),
        ];
    }
    if let Some(named) = by_id.get(implicit_id) {
        // Word applies the default paragraph style (usually Normal) to
        // paras with no pStyle. sd_2517 Normal is after=0; docDefaults is 200.
        defaults.para = named.para.clone();
        defaults.run = named.run.clone();
        // The template's Normal is empty: its 12pt is a docDefault, which
        // a table style's size beats in cells (probe tsn c15l).
        defaults.normal_run = (named.sets_size && !linked, named.sets_family);
        defaults.normal_ind_jc = (named.sets_ind.0, named.sets_jc);
    }
    // Below compatibilityMode 15 an unstyled cell takes the table style's
    // size (its own w:sz, else docDefaults', else the OOXML 10pt) unless
    // that size is 10pt, where Normal's stays. Checked in Word, Normal
    // 12pt throughout: table style none + docDefaults 11/9pt -> 11/9pt;
    // no docDefaults sz -> 12pt; table style 14/9pt -> 14/9pt; table style
    // 10pt (00587c73) -> 12pt. Mode 15, or
    // overrideTableStyleFontSizeAndJustification (the I_am_sharing lock,
    // mode 14), keeps Normal's throughout. So does a Normal sized other
    // than 11 or 12pt (probe tsz_*_0930: 9, 10.5, 11.5 and 14pt stay under
    // a 14pt table style; LibreOffice's writerfilter has the same rule).
    let normal_gives_way = !defaults.normal_run.0
        || [11.0, 12.0]
            .iter()
            .any(|pt| (defaults.run.size - pt).abs() < 0.01);
    let legacy = settings_compat_mode(pkg) < 15
        && !settings_override_table_style_size(pkg)
        && normal_gives_way;
    for table in tables.values_mut() {
        let size = table.run_size.or(doc_default_size).unwrap_or(10.0);
        table.run_size = (legacy && (size - 10.0).abs() > 0.01).then_some(size);
    }
    for table in tables.values_mut().filter(|t| !t.sets_line) {
        table.para.line_mult = defaults.para.line_mult;
        table.para.line_exact = defaults.para.line_exact;
        table.para.line_at_least = defaults.para.line_at_least;
    }
    for table in tables.values_mut().filter(|t| !t.sets_space) {
        table.para.before = defaults.para.before;
        table.para.after = defaults.para.after;
    }
    let default_table = dom
        .descendants(root, Some(&W::name("style")))
        .into_iter()
        .any(|style| {
            attr_any(&dom, style, "type") == Some("table")
                && matches!(attr_any(&dom, style, "default"), Some("1" | "true" | "on"))
        });
    StyleSheet {
        defaults,
        by_id,
        tables,
        theme,
        latent: xml.contains("latentStyles"),
        default_table,
    }
}

/// Narrower than this beside a floating table, no text fits: Word moves
/// the line below the table.
pub(super) const MIN_SIDE_FLOAT_ROOM_PT: f32 = 18.0;

/// A text frame needs a wider gap: text goes under it through an inch or
/// less (live Word: 72pt under, 76pt beside).
pub(super) const MIN_SIDE_FRAME_ROOM_PT: f32 = 72.5;

fn is_auto_spacing(dom: &Dom, spacing: NodeId, name: &str) -> bool {
    attr_any(dom, spacing, name).is_some_and(|v| matches!(v, "1" | "true" | "on"))
}

pub(super) fn parse_tbl_style(
    dom: &Dom,
    style: NodeId,
    defaults: &Defaults,
    theme: &ThemeFonts,
) -> TblStyle {
    let mut para = defaults.para.clone();
    para.after = 0.0;
    para.before = 0.0;
    let mut sets_line = false;
    let mut sets_space = false;
    if let Some(ppr) = dom.element(style, &W::p_pr()) {
        apply_ppr(dom, ppr, &mut para);
        sets_line =
            first_named(dom, ppr, "spacing").is_some_and(|sp| attr_any(dom, sp, "line").is_some());
        sets_space = first_named(dom, ppr, "spacing").is_some_and(|sp| {
            ["before", "after", "beforeAutospacing", "afterAutospacing"]
                .iter()
                .any(|a| attr_any(dom, sp, a).is_some())
        });
        // HTML auto spacing resolves to 0 inside a cell (fixtures_500
        // 00319da4 Table Grid after=100 afterAutospacing=1: Word's cell
        // lines are one line apart), and table-style pPr only reaches cells.
        if let Some(sp) = first_named(dom, ppr, "spacing") {
            if is_auto_spacing(dom, sp, "beforeAutospacing") {
                para.before = 0.0;
            }
            if is_auto_spacing(dom, sp, "afterAutospacing") {
                para.after = 0.0;
            }
        }
    }
    let run_size = dom
        .element(style, &W::r_pr())
        .and_then(|rpr| first_named(dom, rpr, "sz"))
        .and_then(|sz| attr_any(dom, sz, "val"))
        .and_then(parse_finite_f32)
        .map(|half| half / 2.0);
    let run_family = dom
        .element(style, &W::r_pr())
        .filter(|rpr| first_named(dom, *rpr, "rFonts").is_some())
        .map(|rpr| {
            let mut run = defaults.run.clone();
            run.family.clear();
            apply_rpr(dom, rpr, &mut run, theme);
            run.family
        })
        .filter(|f| !f.is_empty());
    let sets_ind_jc = dom
        .element(style, &W::p_pr())
        .map_or((false, false), |ppr| {
            (
                first_named(dom, ppr, "ind").is_some_and(|ind| {
                    attr_any(dom, ind, "left")
                        .or_else(|| attr_any(dom, ind, "start"))
                        .is_some()
                }),
                first_named(dom, ppr, "jc").is_some(),
            )
        });
    let mut out = TblStyle {
        para,
        sets_line,
        sets_space,
        sets_ind_jc,
        run_size,
        own_size: run_size,
        run_family,
        cell_mar_lr: {
            let mar = dom
                .element(style, &W::tbl_pr())
                .and_then(|pr| first_named(dom, pr, "tblCellMar"));
            let edge = |name: &str| mar.and_then(|m| mar_side(dom, m, name));
            (edge("left"), edge("right"))
        },
        cell_mar_tb: {
            let mar = dom
                .element(style, &W::tbl_pr())
                .and_then(|pr| first_named(dom, pr, "tblCellMar"));
            let edge = |name: &str| {
                mar.and_then(|m| first_named(dom, m, name))
                    .and_then(|n| attr_any(dom, n, "w"))
                    .and_then(parse_len)
            };
            (edge("top"), edge("bottom"))
        },
        first_row_fill: None,
        band1_fill: None,
        band2_fill: None,
        first_row_bold: false,
        first_row_italic: false,
        first_col_bold: false,
        first_col_italic: false,
        first_col_fill: None,
        last_row_fill: None,
        last_col_fill: None,
        first_row_color: None,
        first_row_borders: None,
        first_col_borders: None,
        borders: first_named(dom, style, "tblPr").and_then(|pr| parse_tbl_borders(dom, pr)),
    };
    for pr in dom.descendants(style, Some(&W::name("tblStylePr"))) {
        let kind = attr_any(dom, pr, "type").unwrap_or("");
        let fill = style_pr_fill(dom, pr);
        // `<w:i w:val="0"/>` names italic off (5a6c's banner header).
        let on = |name| first_named(dom, pr, name).is_some_and(|n| !val_is_false(dom, Some(n)));
        let (bold, italic) = (on("b"), on("i"));
        match kind {
            "firstRow" => {
                out.first_row_fill = fill;
                out.first_row_bold = bold;
                out.first_row_italic = italic;
                out.first_row_color = style_pr_color(dom, pr);
                out.first_row_borders = parse_style_pr_tc_borders(dom, pr);
            }
            "band1Horz" => out.band1_fill = fill,
            "band2Horz" => out.band2_fill = fill,
            "firstCol" => {
                out.first_col_bold = bold;
                out.first_col_italic = italic;
                out.first_col_fill = fill;
                out.first_col_borders = parse_style_pr_tc_borders(dom, pr);
            }
            "lastRow" => out.last_row_fill = fill,
            "lastCol" => out.last_col_fill = fill,
            _ => {}
        }
    }
    out
}

fn style_pr_fill(dom: &Dom, pr: NodeId) -> Option<[f32; 3]> {
    shd_paint(dom, first_named(dom, pr, "shd")?)
}

fn style_pr_color(dom: &Dom, pr: NodeId) -> Option<[f32; 3]> {
    let el = first_named(dom, pr, "color")?;
    let val = attr_any(dom, el, "val")?;
    if val.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_hex_color(val)
}

fn border_el(dom: &Dom, borders: NodeId, local: &str) -> Option<NodeId> {
    first_named(dom, borders, local).or_else(|| match local {
        "left" => first_named(dom, borders, "start"),
        "right" => first_named(dom, borders, "end"),
        _ => None,
    })
}

pub(super) fn parse_border_edge(dom: &Dom, el: NodeId) -> Option<([f32; 3], f32)> {
    let val = attr_any(dom, el, "val").unwrap_or("single");
    if val == "nil" || val == "none" {
        return None;
    }
    let sz = attr_any(dom, el, "sz")
        .and_then(parse_finite_f32)
        .unwrap_or(8.0);
    if sz <= 0.0 {
        return None;
    }
    let color = attr_any(dom, el, "color")
        .and_then(parse_hex_color)
        .unwrap_or([0.0, 0.0, 0.0]);
    // A double border is three strokes of `sz` (two lines and their gap)
    // and the row makes room for all three (Word 16 probes, bench
    // `scripts/probe_table_top.py`, 2026-10-03: a double sz=6 top border
    // puts the first row's text 2.16-2.25 under its no-border place, a
    // single 0.72, a sz=24 single 2.88; a double insideH stacks 2.25 into
    // the next row's pitch). 4910ce2060's double-rimmed rows sat 1.5
    // high. The painter fills the room as one band.
    let strokes = if val == "double" { 3.0 } else { 1.0 };
    Some((color, (sz / 8.0 * strokes).max(0.24)))
}

pub(super) fn parse_tbl_borders(dom: &Dom, parent: NodeId) -> Option<TblBorders> {
    let borders = first_named(dom, parent, "tblBorders")?;
    let mut out = TblBorders {
        top: false,
        bottom: false,
        left: false,
        right: false,
        inside_h: false,
        inside_v: false,
        color: [0.0, 0.0, 0.0],
        width: 0.5,
        outer_width: 0.5,
    };
    let (mut outer, mut inner) = (None, None);
    for (local, flag) in [
        ("top", &mut out.top),
        ("bottom", &mut out.bottom),
        ("left", &mut out.left),
        ("right", &mut out.right),
        ("insideH", &mut out.inside_h),
        ("insideV", &mut out.inside_v),
    ] {
        let Some(el) = border_el(dom, borders, local) else {
            continue;
        };
        let Some((color, width)) = parse_border_edge(dom, el) else {
            continue;
        };
        *flag = true;
        out.color = color;
        if local.starts_with("inside") {
            inner = Some(width);
        } else {
            outer = Some(width);
        }
    }
    out.width = inner.or(outer).unwrap_or(0.5);
    out.outer_width = outer.or(inner).unwrap_or(0.5);
    // Present `w:tblBorders` is a real override, including all-none
    // (file_22 / sd_2517). Returning None here used to inherit TableGrid.
    Some(out)
}

/// Cell edges from a row's `tblPrEx/tblBorders`: outer edges on the
/// table's rim, insideH/insideV between cells.
pub(super) fn row_exception_cell_borders(
    b: TblBorders,
    first_row: bool,
    last_row: bool,
    first_col: bool,
    last_col: bool,
) -> CellBorders {
    let edge =
        |rim: bool, on: bool| on.then_some((b.color, if rim { b.outer_width } else { b.width }));
    CellBorders {
        top: edge(first_row, if first_row { b.top } else { b.inside_h }),
        bottom: edge(last_row, if last_row { b.bottom } else { b.inside_h }),
        left: edge(first_col, if first_col { b.left } else { b.inside_v }),
        right: edge(last_col, if last_col { b.right } else { b.inside_v }),
    }
}

/// A cell's `w:tcBorders` edge by edge (top, bottom, left, right): the
/// outer `None` is an edge the cell does not name, `Some(None)` one it
/// names as none.
type TcEdge = Option<Option<([f32; 3], f32)>>;

pub(super) fn parse_tc_border_spec(dom: &Dom, cell: NodeId) -> Option<[TcEdge; 4]> {
    let pr = cell_pr(dom, cell)?;
    let borders = direct_named(dom, pr, "tcBorders")?;
    let edge = |name: &str| border_el(dom, borders, name).map(|el| parse_border_edge(dom, el));
    Some([edge("top"), edge("bottom"), edge("left"), edge("right")])
}

fn parse_style_pr_tc_borders(dom: &Dom, pr: NodeId) -> Option<CellBorders> {
    let tc_pr = first_named(dom, pr, "tcPr")?;
    parse_tc_borders_el(dom, tc_pr)
}

fn parse_tc_borders_el(dom: &Dom, pr: NodeId) -> Option<CellBorders> {
    let borders = direct_named(dom, pr, "tcBorders")?;
    Some(CellBorders {
        top: border_el(dom, borders, "top").and_then(|el| parse_border_edge(dom, el)),
        bottom: border_el(dom, borders, "bottom").and_then(|el| parse_border_edge(dom, el)),
        left: border_el(dom, borders, "left").and_then(|el| parse_border_edge(dom, el)),
        right: border_el(dom, borders, "right").and_then(|el| parse_border_edge(dom, el)),
    })
}

fn resolve_named(
    dom: &Dom,
    raw: &std::collections::HashMap<String, RawStyle>,
    defaults: &Defaults,
    theme: &ThemeFonts,
    id: &str,
    depth: u8,
) -> (ParaStyle, RunStyle) {
    if depth > 12 {
        return (defaults.para.clone(), defaults.run.clone());
    }
    let Some(raw_style) = raw.get(id) else {
        return (defaults.para.clone(), defaults.run.clone());
    };
    let (mut para, mut run) = if let Some(base) = raw_style.based.as_deref() {
        resolve_named(dom, raw, defaults, theme, base, depth + 1)
    } else {
        (defaults.para.clone(), defaults.run.clone())
    };
    if let Some(node) = raw_style.ppr {
        apply_ppr(dom, node, &mut para);
    }
    if let Some(node) = raw_style.rpr {
        apply_rpr(dom, node, &mut run, theme);
    }
    (para, run)
}

pub(super) fn first_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    // w:pPrChange / rPrChange / tblPrChange hold the *previous* pPr.
    // file_146 live pPr is pBdr+spacing; ListParagraph hanging lives
    // only in pPrChange. Stealing that ghost indented Hello at 90.
    dom.find_descendant_element(node, Some(&W::name(local)), |cand| {
        !under_prior_change(dom, node, cand)
    })
}

/// The body's section properties in document order, without the old
/// copies a tracked change keeps (`w:sectPrChange`, `w:pPrChange`): a
/// redline's final sectPr holding its previous self is one section, not
/// two (730 of 965 Word comparisons gained a blank last page).
pub(super) fn live_sect_prs(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    dom.descendants(body, Some(&W::sect_pr()))
        .into_iter()
        .filter(|&s| !under_prior_change(dom, body, s))
        .collect()
}

pub(super) fn under_prior_change(dom: &Dom, root: NodeId, mut n: NodeId) -> bool {
    while n != root {
        let Some(parent) = dom.parent(n) else {
            break;
        };
        if parent == root {
            break;
        }
        if local_name_is(dom, parent, "pPrChange")
            || local_name_is(dom, parent, "rPrChange")
            || local_name_is(dom, parent, "tblPrChange")
            || local_name_is(dom, parent, "trPrChange")
            || local_name_is(dom, parent, "tcPrChange")
            || local_name_is(dom, parent, "sectPrChange")
            || local_name_is(dom, parent, "numPrChange")
        {
            return true;
        }
        n = parent;
    }
    false
}

pub(super) fn direct_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    let want = W::name(local);
    (0..dom.child_count(node)).find_map(|i| {
        let child = dom.child_at(node, i);
        dom.name_is(child, &want).then_some(child)
    })
}

pub(super) fn apply_rfonts(dom: &Dom, fonts: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    // Explicit ascii/hAnsi wins, except a Display-family cache sitting
    // next to a theme slot. I_am_sharing Heading1/2/Title store
    // ascii="Aptos Display" + asciiTheme=majorHAnsi; Word Quartz paints
    // the live theme major (Calibri-Bold), not the Aptos Display cache.
    // Theme is otherwise the fallback when Word stored a slot and no
    // family name (comments Heading1 is majorHAnsi → major latin;
    // comments body is minorHAnsi → Aptos).
    // A minor slot resolves to its theme face whatever that face is
    // (Cambria included; see the minor branch below). The old Aptos-only
    // gate (mini 90 / 396: file_2 / file_41 Cambria line boxes) is gone
    // now that line boxes come from face metrics.
    let east_asia = attr_any(dom, fonts, "eastAsia");
    let east_asia_slot = attr_any(dom, fonts, "eastAsiaTheme");
    if let Some(name) = east_asia {
        style.family_ea = Some(name.to_string());
        style.ea_theme_slot = None;
    } else if let Some(slot) = east_asia_slot {
        style.ea_theme_slot = Some(slot.to_string());
        if let Some(face) = theme_script_face(theme, slot) {
            style.family_ea = Some(face);
        }
    }
    let cs = attr_any(dom, fonts, "cs");
    let cs_slot = attr_any(dom, fonts, "cstheme").or_else(|| attr_any(dom, fonts, "csTheme"));
    if let Some(name) = cs {
        style.family_cs = Some(name.to_string());
    } else if let Some(face) = cs_slot.and_then(|slot| theme_script_face(theme, slot)) {
        style.family_cs = Some(face);
    }
    if let Some(hint) = attr_any(dom, fonts, "hint") {
        style.hint = match hint {
            "eastAsia" => FontHint::EastAsia,
            "cs" => FontHint::Complex,
            _ => FontHint::Default,
        };
    }
    let ascii_attr = attr_any(dom, fonts, "ascii");
    let hansi_attr = attr_any(dom, fonts, "hAnsi");
    let ascii_slot = attr_any(dom, fonts, "asciiTheme");
    // An hAnsi face with no ascii one of its own paints only the
    // characters past U+007F; the ascii ones keep the inherited face.
    if ascii_attr.is_none()
        && ascii_slot.is_none()
        && let Some(hansi) = hansi_attr
    {
        style.family_hansi =
            (!hansi.eq_ignore_ascii_case(&style.family)).then(|| hansi.to_string());
        return;
    }
    if let (Some(a), Some(h)) = (ascii_attr, hansi_attr) {
        style.family_hansi = (!h.eq_ignore_ascii_case(a)).then(|| h.to_string());
    } else if let Some(a) = ascii_attr {
        // An ascii face alone leaves an hAnsi face set below it in place
        // (015beda9: Normal's ascii Segoe UI over Word's built-in Aptos
        // defaults, and Word paints "©" in Aptos). With no hAnsi of its
        // own below, hAnsi follows the ascii face: 0012788b's docDefaults
        // ascii Times New Roman + hAnsiTheme paints Cyrillic in Times.
        style.family_hansi = style
            .family_hansi
            .take()
            .filter(|h| !h.eq_ignore_ascii_case(a));
    }
    // An asciiTheme slot names the ascii face itself; an explicit hAnsi
    // next to it keeps to the characters past U+007F (0fc80afa25: slot
    // minorHAnsi + hAnsi Arial, Word paints the Latin text in Calibri).
    let slot_hansi = if ascii_attr.is_none() && ascii_slot.is_some() {
        hansi_attr
    } else {
        None
    };
    let ascii = if slot_hansi.is_some() {
        None
    } else {
        ascii_attr.or(hansi_attr)
    };
    let slot = ascii_slot.or_else(|| attr_any(dom, fonts, "hAnsiTheme"));
    let display_cache = ascii.is_some_and(|name| name.to_ascii_lowercase().contains("display"));
    if let Some(ascii) = ascii
        && !(display_cache && slot.is_some())
    {
        style.family = ascii.to_string();
        return;
    }
    let Some(slot) = slot else {
        return;
    };
    let slot = slot.to_ascii_lowercase();
    // minorBidi / majorBidi name the complex-script face, not the latin.
    if slot.contains("bidi")
        && let Some(face) = theme_script_face(theme, &slot)
    {
        style.family = face;
    } else if slot.contains("major")
        && let Some(face) = theme.major.as_deref()
    {
        style.family = face.to_string();
    } else if slot.contains("minor")
        && let Some(face) = theme.minor.as_deref()
    {
        // Honour minorHAnsi for every theme face, not only Aptos. The
        // Aptos-only gate was a mini-set trade (file_2 / file_41 Cambria
        // line boxes). Those two will drop until the line-box PR; do not
        // restore this gate.
        style.family = face.to_string();
    } else if let Some(ascii) = ascii.or(slot_hansi) {
        style.family = ascii.to_string();
    }
    if let Some(hansi) = slot_hansi {
        style.family_hansi =
            (!hansi.eq_ignore_ascii_case(&style.family)).then(|| hansi.to_string());
    }
}

pub(super) fn lang_to_ooxml_script(lang: &str) -> Option<&'static str> {
    let lang = lang.to_ascii_lowercase();
    if lang.starts_with("ja") {
        Some("Jpan")
    } else if lang.starts_with("ko") {
        Some("Hang")
    } else if lang.starts_with("zh-hant")
        || lang.starts_with("zh-tw")
        || lang.starts_with("zh-hk")
        || lang.starts_with("zh-mo")
    {
        Some("Hant")
    } else if lang.starts_with("zh") {
        Some("Hans")
    } else {
        None
    }
}

/// Re-resolve a theme-slot East Asian face against the effective
/// `w:lang/@w:eastAsia` script (theme `a:font script=`), after any rPr
/// layer. Named `w:eastAsia` faces and styles without a theme slot keep
/// what they have.
pub(super) fn apply_theme_script_fonts(style: &mut RunStyle, theme: &ThemeFonts) {
    let Some(slot) = style.ea_theme_slot.as_deref() else {
        return;
    };
    let Some(lang) = style.lang_ea.as_deref() else {
        return;
    };
    let Some(script) = lang_to_ooxml_script(lang) else {
        return;
    };
    let bucket = if slot.to_ascii_lowercase().contains("major") {
        "major"
    } else {
        "minor"
    };
    if let Some(face) = theme
        .script_fonts
        .get(&(bucket.to_string(), script.to_string()))
    {
        style.family_ea = Some(face.clone());
    }
}

pub(super) fn theme_script_face(theme: &ThemeFonts, slot: &str) -> Option<String> {
    let slot = slot.to_ascii_lowercase();
    if slot.contains("majoreastasia") {
        theme.major_ea.clone()
    } else if slot.contains("minoreastasia") {
        theme.minor_ea.clone()
    } else if slot.contains("majorcs") || slot.contains("majorbidi") {
        theme_bidi_face(theme, "major")
    } else if slot.contains("minorcs") || slot.contains("minorbidi") {
        theme_bidi_face(theme, "minor")
    } else {
        None
    }
}

/// A theme's complex-script face; an empty `a:cs` falls to the Arab script
/// font of Word's default ar-SA bidi language (0033735c draws Arial).
fn theme_bidi_face(theme: &ThemeFonts, which: &str) -> Option<String> {
    let cs = if which == "major" {
        &theme.major_cs
    } else {
        &theme.minor_cs
    };
    cs.clone().or_else(|| {
        theme
            .script_fonts
            .get(&(which.to_string(), "Arab".to_string()))
            .cloned()
    })
}

/// Font family names a part's markup mentions (`w:rFonts` slots, theme
/// `typeface`s), deduplicated.
/// Families that paint non-East-Asian text: run `w:ascii` / `w:hAnsi` /
/// `w:cs` and the theme's `a:latin` faces (not its per-script list).
pub(super) fn latin_font_names(xml: &str) -> Vec<String> {
    font_attr_names(
        xml,
        &[
            "w:ascii=\"",
            "w:hAnsi=\"",
            "w:cs=\"",
            "<a:latin typeface=\"",
        ],
    )
}

/// The distinct names `xml` gives after each attribute opener in `keys`.
pub(super) fn font_attr_names(xml: &str, keys: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for key in keys {
        let mut rest = xml;
        while let Some(at) = rest.find(key) {
            rest = &rest[at + key.len()..];
            let Some(end) = rest.find('"') else {
                break;
            };
            let name = &rest[..end];
            if !name.is_empty() && !out.iter().any(|n| n == name) {
                out.push(name.to_string());
            }
            rest = &rest[end..];
        }
    }
    out
}

/// Whether a w:numFmt or w:pgNumType format in `xml` writes characters
/// that pass `test` (`ideographDigital`'s 一, `thaiLetters`' ก): labels
/// whose script the text itself never shows.
pub(super) fn number_formats_write(xml: &str, test: fn(char) -> bool) -> bool {
    ["numFmt", "pgNumType"].iter().any(|tag| {
        xml.match_indices(tag).any(|(at, _)| {
            let attrs = &xml[at..];
            let attrs = &attrs[..attrs.find('>').unwrap_or(attrs.len())];
            attrs
                .split(['"', '\''])
                .skip(1)
                .step_by(2)
                .map(parse_num_fmt)
                .any(|fmt| {
                    [1, 10, 100]
                        .into_iter()
                        .any(|n| format_num(fmt, n).chars().any(test))
                })
        })
    })
}

/// The characters an XML part names by numeric reference (`&#x3001;`,
/// `&#12289;`), which `str::chars` sees only as ASCII.
pub(super) fn char_refs(xml: &str) -> impl Iterator<Item = char> + '_ {
    xml.split("&#").skip(1).filter_map(|rest| {
        let (num, _) = rest.split_once(';')?;
        let code = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse().ok()?,
        };
        char::from_u32(code)
    })
}

pub(super) fn rfont_names(xml: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for key in [
        "w:ascii=\"",
        "w:hAnsi=\"",
        "w:eastAsia=\"",
        "w:cs=\"",
        "typeface=\"",
    ] {
        let mut rest = xml;
        while let Some(at) = rest.find(key) {
            rest = &rest[at + key.len()..];
            let Some(end) = rest.find('"') else {
                break;
            };
            let name = &rest[..end];
            if !name.is_empty() && !out.iter().any(|n| n == name) {
                out.push(name.to_string());
            }
            rest = &rest[end..];
        }
    }
    out
}

pub(super) fn is_thaana(c: char) -> bool {
    ('\u{0780}'..='\u{07BF}').contains(&c)
}

/// The face Word paints `text` in when its own face lacks the glyphs:
/// Word's Thaana or CJK face for those scripts, else none (Arial).
pub(super) fn script_glyph_fallback(
    fonts: &Fonts,
    style: &RunStyle,
    text: &str,
) -> Option<FaceRef> {
    let bold = style.bold;
    if text.chars().any(is_thaana) {
        return fonts.thaana_glyph_fallback(bold);
    }
    if text.chars().any(is_hangul) {
        return fonts.hangul_glyph_fallback(bold);
    }
    if text.chars().any(is_thai) {
        return fonts.script_fallback(font::THAI_FALLBACK, bold);
    }
    if text.chars().any(is_devanagari) {
        return fonts.script_fallback(font::DEVANAGARI_FALLBACK, bold);
    }
    fonts
        .cjk_glyph_fallback(bold, style.family_ea.as_deref())
        .filter(|_| text.chars().any(|c| is_cjk(c) || is_enclosed_alnum(c)))
}

/// ① to ⓿ and ⒇: a Latin face rarely has them, and Word paints them from
/// its East Asian face (Word 16 probe ench 1001: Calibri's "A⒇①B", with
/// no eastAsia face, sets ⒇① in MS Gothic, as a footer's
/// decimalEnclosedParen PAGE label; probe 33g's eastAsia Calibri sets
/// them in MS Mincho).
fn is_enclosed_alnum(c: char) -> bool {
    matches!(c, '\u{2460}'..='\u{24FF}')
}

/// A character that paints from the CJK fallback when its face has none.
pub(super) fn takes_cjk_fallback(c: char) -> bool {
    takes_east_asian_face(c) || is_enclosed_alnum(c)
}

pub(super) fn is_thai(c: char) -> bool {
    matches!(c, '\u{0E00}'..='\u{0E7F}')
}

pub(super) fn is_devanagari(c: char) -> bool {
    matches!(c, '\u{0900}'..='\u{097F}' | '\u{A8E0}'..='\u{A8FF}')
}

/// A character a script fallback paints when its face has none.
pub(super) fn takes_script_fallback(c: char) -> bool {
    is_rtl_char(c) || takes_cjk_fallback(c) || is_thai(c) || is_devanagari(c)
}

/// Hebrew, Arabic, Syriac, Thaana, NKo and their presentation forms.
pub(super) fn is_rtl_char(c: char) -> bool {
    matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
}

/// Whether HarfBuzz shapes `text` right to left: its first letter's
/// script decides, as `guess_segment_properties` does. The glyphs then
/// run from the last character to the first.
pub(super) fn shapes_rtl(text: &str) -> bool {
    text.chars()
        .find(|c| c.is_alphabetic())
        .is_some_and(is_rtl_char)
}

pub(super) fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{3000}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF00}'..='\u{FFEF}'
            | '\u{20000}'..='\u{2FA1F}'
    )
}

/// Hangul syllables and jamo outside `is_cjk`'s blocks (the compatibility
/// jamo U+3130..318F are inside it).
fn is_hangul(c: char) -> bool {
    // The compatibility jamo too: Word paints chosung's ㄱ in Malgun
    // Gothic (Word 16 probe scripts 1001), and YaHei has none.
    matches!(
        c,
        '\u{1100}'..='\u{11FF}'
            | '\u{3130}'..='\u{318F}'
            | '\u{A960}'..='\u{A97F}'
            | '\u{AC00}'..='\u{D7FF}'
    )
}

/// A character Word paints in the run's East Asian face: ideographs, kana
/// and Hangul (the korean_japanese_conference_form title's "발표" is 맑은
/// 고딕, its eastAsia face, not the Yu Mincho its ascii face names). Line
/// breaking and Latin/CJK spacing keep `is_cjk`.
pub(super) fn takes_east_asian_face(c: char) -> bool {
    is_cjk(c) || is_hangul(c)
}

pub(super) fn paint_family<'a>(style: &'a RunStyle, text: &str) -> &'a str {
    // w:hint="eastAsia" decides only characters either script may own
    // (curly quotes, dashes, symbols); Latin letters and digits keep the
    // ascii/hAnsi face (00d2ca27's hinted "Suppl 1." is Times New Roman).
    // Letters of any other script keep it too (006ad742's hinted Cyrillic
    // "гарантиране" is Arial in Word, not the East Asian face).
    if let Some(ea) = style.family_ea.as_deref()
        && (text.chars().any(takes_east_asian_face)
            || (style.hint == FontHint::EastAsia
                && !text
                    .chars()
                    .any(|c| c.is_alphanumeric() && !takes_east_asian_face(c))
                && !text.is_ascii()))
    {
        return ea;
    }
    // Right-to-left and Thaana letters are complex script whatever the
    // hint: Word draws them in the cs font (0003fc93's Faruma runs). A cs
    // hint claims only the marks either script may own: digits and Latin
    // letters keep the ascii face (00205272's "12" is Calibri, the "-"
    // beside it Arial).
    if let Some(cs) = style.family_cs.as_deref()
        && (text.chars().any(is_rtl_char)
            || (style.hint == FontHint::Complex
                && !text.chars().any(|c| c.is_ascii_alphanumeric())))
    {
        return cs;
    }
    &style.family
}

pub(super) fn apply_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    if let Some(sz) = first_named(dom, rpr, "sz")
        && let Some(val) = dom.attribute(sz, &W::val())
        && let Some(half) = parse_finite_f32(val)
    {
        style.size = half / 2.0;
    }
    if let Some(sz) = first_named(dom, rpr, "szCs")
        && let Some(val) = dom.attribute(sz, &W::val())
        && let Some(half) = parse_finite_f32(val)
    {
        style.size_cs = Some(half / 2.0);
    }
    // A w:lang without @w:eastAsia leaves the inherited East Asian
    // language alone (it only restates the Latin one).
    if let Some(ea) = first_named(dom, rpr, "lang").and_then(|l| attr_any(dom, l, "eastAsia")) {
        style.lang_ea = Some(ea.to_string());
    }
    if let Some(fonts) = first_named(dom, rpr, "rFonts") {
        apply_rfonts(dom, fonts, style, theme);
    }
    apply_theme_script_fonts(style, theme);
    if let Some(rtl) = first_named(dom, rpr, "rtl") {
        style.rtl = !val_is_false(dom, Some(rtl));
    }
    if let Some(vanish) = first_named(dom, rpr, "vanish") {
        style.hidden = !val_is_false(dom, Some(vanish));
    }
    if first_named(dom, rpr, "b").is_some() {
        style.bold = !val_is_false(dom, first_named(dom, rpr, "b"));
        style.bold_set = true;
    }
    if first_named(dom, rpr, "i").is_some() {
        style.italic = !val_is_false(dom, first_named(dom, rpr, "i"));
        style.italic_set = true;
    }
    if first_named(dom, rpr, "u").is_some() {
        let val = first_named(dom, rpr, "u").and_then(|n| dom.attribute(n, &W::val()));
        // A w:u with no w:val (0023298b's color-only <w:u/>) draws nothing.
        let off = val.is_none_or(|v| v == "none");
        style.underline = !off;
        style.underline_double = val.is_some_and(|v| v == "double" || v == "thick");
        style.underline_wave = val.is_some_and(|v| v == "wave" || v == "wavy");
    }
    let dstrike = first_named(dom, rpr, "dstrike").is_some_and(|n| !val_is_false(dom, Some(n)));
    style.strike =
        first_named(dom, rpr, "strike").is_some_and(|n| !val_is_false(dom, Some(n))) || dstrike;
    style.strike_double = dstrike;
    if let Some(val) = first_named(dom, rpr, "vertAlign").and_then(|n| attr_any(dom, n, "val")) {
        style.vert = match val {
            "superscript" => VertAlign::Super,
            "subscript" => VertAlign::Sub,
            _ => VertAlign::Baseline,
        };
    }
    if let Some(color) = first_named(dom, rpr, "color") {
        if let Some(val) = dom.attribute(color, &W::val())
            && val != "auto"
            && let Some(rgb) = parse_hex_color(val)
        {
            style.color = rgb;
            style.color_auto = false;
        } else if let Some(slot) = attr_any(dom, color, "themeColor")
            && let Some(mut rgb) = theme.slot_color(slot)
        {
            if let Some(shade) = attr_any(dom, color, "themeShade")
                && let Ok(n) = u8::from_str_radix(shade, 16)
            {
                let f = f32::from(n) / 255.0;
                rgb = [rgb[0] * f, rgb[1] * f, rgb[2] * f];
            }
            if let Some(tint) = attr_any(dom, color, "themeTint")
                && let Ok(n) = u8::from_str_radix(tint, 16)
            {
                let f = f32::from(n) / 255.0;
                rgb = [
                    rgb[0] + (1.0 - rgb[0]) * (1.0 - f),
                    rgb[1] + (1.0 - rgb[1]) * (1.0 - f),
                    rgb[2] + (1.0 - rgb[2]) * (1.0 - f),
                ];
            }
            style.color = rgb;
            style.color_auto = false;
        } else if dom.attribute(color, &W::val()) == Some("auto") {
            // "auto" is Word's automatic colour: it overrides an inherited
            // one (004b3b3d's runs under a red paragraph style are black).
            style.color = [0.0, 0.0, 0.0];
            style.color_auto = true;
        }
    } else {
        // Strict01 Online Video: w14:textFill accent5, no w:color.
        // Not w14:shadow extra copy (mini 350 ITT-neg).
        apply_w14_text_fill(dom, rpr, style, theme);
    }
    // Word p11 omits reflection and shadow+outline as body glyphs
    // (filled bars in the oracle). Shadow-only (mini 350) still
    // paints. textOutline+w:color with explicit sz (mini 371 Keyword
    // 12pt) still paints; factory-11pt outline+color (p#107 peach) is
    // extra vs Word slabs.
    let has_reflection = !descendants_local(dom, rpr, "reflection").is_empty();
    let has_shadow = !descendants_local(dom, rpr, "shadow").is_empty();
    // An outline with w14:noFill draws nothing (Pages-made "Cuerpo"
    // styles in 107 redlines): it is no outline.
    let has_outline = descendants_local(dom, rpr, "textOutline")
        .into_iter()
        .any(|o| descendants_local(dom, o, "noFill").is_empty());
    let has_sz = first_named(dom, rpr, "sz").is_some();
    let has_color_el = first_named(dom, rpr, "color").is_some();
    if has_reflection || (has_shadow && has_outline) || (has_outline && has_color_el && !has_sz) {
        style.effect_skip = true;
    }
    if let Some(val) = first_named(dom, rpr, "highlight").and_then(|n| attr_any(dom, n, "val")) {
        style.highlight = highlight_color(val);
        style.highlight_marker = style.highlight.is_some();
    }
    if let Some(sp) = first_named(dom, rpr, "spacing")
        && let Some(val) = attr_any(dom, sp, "val")
        && let Some(pt) = parse_len(val)
    {
        style.track = pt;
    }
    if let Some(w) = first_named(dom, rpr, "w")
        && let Some(val) = attr_any(dom, w, "val")
        // ST_TextScale is a whole number or, in strict and newer files,
        // a percentage string (docxide case24: "150%").
        && let Some(pct) = parse_finite_f32(val.trim().trim_end_matches('%'))
        && pct > 0.0
    {
        style.scale = pct / 100.0;
    }
    if first_named(dom, rpr, "caps").is_some() {
        style.caps = !val_is_false(dom, first_named(dom, rpr, "caps"));
    }
    if first_named(dom, rpr, "smallCaps").is_some()
        && !val_is_false(dom, first_named(dom, rpr, "smallCaps"))
    {
        // Word only shrinks lowercase (ECMA-376 17.3.2.33). file_34 /
        // uipriority store already-uppercase "SMALL CAPS TEXT"; size*=0.8
        // on the whole run shrank those to ~8.8pt.
        style.small_caps = true;
    }
    if let Some(pos) = first_named(dom, rpr, "position")
        && let Some(val) = attr_any(dom, pos, "val")
        && let Some(half) = parse_finite_f32(val)
    {
        style.offset = half / 2.0;
    }
    if let Some(kern) = first_named(dom, rpr, "kern")
        && let Some(val) = attr_any(dom, kern, "val")
        && let Ok(half) = val.parse::<u16>()
    {
        // ECMA 17.3.2.19: smallest size (half-points) that gets
        // automatic kerning (val=2 kerns everything from 1pt up).
        style.kern_half = half;
    }
    if style.highlight.is_none()
        && let Some(shd) = first_named(dom, rpr, "shd")
        && let Some(paint) = shd_paint(dom, shd)
    {
        style.highlight = Some(paint);
        style.highlight_marker = false;
    }
}

pub(super) fn apply_ppr(dom: &Dom, ppr: NodeId, style: &mut ParaStyle) {
    if let Some(jc) = first_named(dom, ppr, "jc")
        && let Some(val) = dom.attribute(jc, &W::val())
    {
        style.align = match val {
            "center" => Align::Center,
            "right" | "end" => Align::Right,
            "both" | "distribute" => Align::Justify,
            _ => Align::Left,
        };
    }
    if let Some(b) = direct_named(dom, ppr, "bidi") {
        style.bidi = !val_is_false(dom, Some(b));
    }
    if let Some(wc) = direct_named(dom, ppr, "widowControl") {
        style.widow_control = !val_is_false(dom, Some(wc));
    }
    if let Some(sp) = first_named(dom, ppr, "spacing") {
        // ISO Strict (Strict01) writes `8pt` / `12.95pt`. Bare numbers are twips.
        // Spacing merges per attribute (Word 16 probe asp0930): a w:after
        // under an inherited afterAutospacing="1" is only the fallback, and
        // only an explicit w:afterAutospacing turns auto on or off. HTML auto
        // spacing (fixtures_500 00a46590) is 14pt.
        if let Some(after) = attr_any(dom, sp, "after").and_then(parse_len) {
            style.after_spec = after;
            if !style.after_auto {
                style.after = after;
            }
        }
        if let Some(before) = attr_any(dom, sp, "before").and_then(parse_len) {
            style.before_spec = before;
            if !style.before_auto {
                style.before = before;
            }
        }
        // Under `doNotUseHTMLParagraphAutoSpacing` ("use fixed paragraph
        // spacing for the HTML auto setting") the auto flag is void and
        // the explicit value stands: f23fc5de2e's before=100 after=100
        // list rows pitch 23.8 in Word's PDF, the 13.8 line plus 5 and 5,
        // not the 14 and 14 the flags would give.
        if attr_any(dom, sp, "beforeAutospacing").is_some() {
            style.before_auto = is_auto_spacing(dom, sp, "beforeAutospacing") && !style.sum_spacing;
            style.before = if style.before_auto {
                14.0
            } else {
                style.before_spec
            };
        }
        if attr_any(dom, sp, "afterAutospacing").is_some() {
            style.after_auto = is_auto_spacing(dom, sp, "afterAutospacing") && !style.sum_spacing;
            style.after = if style.after_auto {
                14.0
            } else {
                style.after_spec
            };
        }
        let rule = attr_any(dom, sp, "lineRule").unwrap_or("auto");
        if let Some(line) = attr_any(dom, sp, "line") {
            // Each lineRule owns the whole line model: a later layer (child
            // style, direct pPr) must clear whatever an earlier one set.
            let unit = line.chars().any(|c| c.is_ascii_alphabetic());
            let bare = if unit { None } else { parse_finite_f32(line) };
            let spec = if unit {
                parse_len(line)
            } else {
                bare.map(twip)
            };
            if let Some(pt) = spec {
                style.line_exact = None;
                style.line_at_least = None;
                match rule {
                    "exact" => style.line_exact = Some(pt),
                    "atLeast" => {
                        style.line_at_least = Some(pt);
                        style.line_mult = 1.0;
                    }
                    // Unit-form auto (Strict01 `12.95pt`) keeps its 11pt
                    // heuristic; bare auto is 240ths of a line.
                    _ => {
                        style.line_mult = bare.map_or((pt / 11.0).max(0.8), |v| v / 240.0);
                    }
                }
            }
        }
    }
    // A listed edge replaces the inherited one, a none/nil edge clears it.
    let listed =
        |edge| first_named(dom, ppr, "pBdr").is_some_and(|b| first_named(dom, b, edge).is_some());
    if listed("top") {
        style.border_top = pbdr_edge(dom, ppr, "top");
    }
    if listed("left") {
        style.border_left = pbdr_edge(dom, ppr, "left");
    }
    if listed("bottom") {
        style.border_bottom = pbdr_edge(dom, ppr, "bottom");
    }
    if listed("right") {
        style.border_right = pbdr_edge(dom, ppr, "right");
    }
    if let Some(ind) = first_named(dom, ppr, "ind") {
        if let Some(left) = attr_any(dom, ind, "left")
            .or_else(|| attr_any(dom, ind, "start"))
            .and_then(parse_len)
        {
            style.indent_left = left;
        }
        if let Some(right) = attr_any(dom, ind, "right")
            .or_else(|| attr_any(dom, ind, "end"))
            .and_then(parse_len)
        {
            style.indent_right = right;
        }
        if let Some(first) = attr_any(dom, ind, "firstLine").and_then(parse_len) {
            style.indent_first = first;
        }
        // Hanging and firstLine are mutually exclusive in Word; hanging wins if both exist.
        if let Some(hanging) = attr_any(dom, ind, "hanging").and_then(parse_len) {
            style.indent_first = -hanging;
        }
    }
    if first_named(dom, ppr, "contextualSpacing").is_some() {
        style.contextual = !val_is_false(dom, first_named(dom, ppr, "contextualSpacing"));
    }
    if let Some(n) = first_named(dom, ppr, "autoSpaceDE") {
        style.auto_space_de_off = val_is_false(dom, Some(n));
    }
    if let Some(n) = first_named(dom, ppr, "autoSpaceDN") {
        style.auto_space_dn_off = val_is_false(dom, Some(n));
    }
    if first_named(dom, ppr, "tabs").is_some() {
        apply_tab_stops(&mut style.tab_stops, dom, ppr);
    }
    if first_named(dom, ppr, "pageBreakBefore").is_some() {
        style.page_break_before = !val_is_false(dom, first_named(dom, ppr, "pageBreakBefore"));
    }
    if first_named(dom, ppr, "snapToGrid").is_some() {
        style.snap_to_grid = !val_is_false(dom, first_named(dom, ppr, "snapToGrid"));
    }
    if first_named(dom, ppr, "keepNext").is_some() {
        style.keep_next = !val_is_false(dom, first_named(dom, ppr, "keepNext"));
    }
    if first_named(dom, ppr, "keepLines").is_some() {
        style.keep_lines = !val_is_false(dom, first_named(dom, ppr, "keepLines"));
    }
    if let Some(lvl) = first_named(dom, ppr, "outlineLvl")
        && let Some(v) = attr_any(dom, lvl, "val").and_then(|s| s.parse::<u32>().ok())
    {
        style.outline_lvl = Some(v);
    }
    if let Some(fill) = ppr_shd_fill(dom, ppr) {
        style.fill = Some(fill);
    }
}

fn ppr_shd_fill(dom: &Dom, ppr: NodeId) -> Option<[f32; 3]> {
    // Direct w:pPr/w:shd only. first_named would steal pPr/rPr/shd
    // (file_71 paragraph-mark green) and paint a content-wide band.
    let shd = direct_named(dom, ppr, "shd")?;
    if let Some(rgb) = shd_paint(dom, shd) {
        // White-on-white (image_out_of_folder / sd_2517) is a no-op.
        if rgb.iter().all(|c| *c > 0.98) {
            return None;
        }
        return Some(rgb);
    }
    attr_any(dom, shd, "themeFill").and_then(theme_slot_color)
}

pub(super) fn val_is_false(dom: &Dom, node: Option<NodeId>) -> bool {
    node.and_then(|n| dom.attribute(n, &W::val()))
        .is_some_and(|v| v == "0" || v == "false" || v == "off")
}

fn apply_w14_text_fill(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    let Some(fill) = descendants_local(dom, rpr, "textFill").into_iter().next() else {
        return;
    };
    if !descendants_local(dom, fill, "noFill").is_empty() {
        return;
    }
    let Some(slot) = descendants_local(dom, fill, "schemeClr")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
    else {
        return;
    };
    if let Some(rgb) = theme.slot_color(slot) {
        style.color = rgb;
        style.color_auto = false;
    }
}

pub(super) fn theme_slot_color(slot: &str) -> Option<[f32; 3]> {
    // Office / Word 2007–2016 default theme (dk2/accent1 match I_am_sharing).
    let hex = match slot {
        "dk1" | "tx1" | "text1" => "000000",
        "lt1" | "bg1" => "FFFFFF",
        "dk2" | "tx2" | "text2" => "1F497D",
        "lt2" | "bg2" => "EEECE1",
        "accent1" => "4F81BD",
        "accent2" => "C0504D",
        "accent3" => "9BBB59",
        "accent4" => "8064A2",
        "accent5" => "4BACC6",
        "accent6" => "F79646",
        "hlink" | "hyperlink" => "0000FF",
        "folHlink" | "followedHyperlink" => "800080",
        _ => return None,
    };
    parse_hex_color(hex)
}

fn highlight_color(val: &str) -> Option<[f32; 3]> {
    Some(match val {
        "yellow" => [1.0, 1.0, 0.0],
        "green" => [0.0, 1.0, 0.0],
        "cyan" => [0.0, 1.0, 1.0],
        "magenta" => [1.0, 0.0, 1.0],
        "blue" => [0.0, 0.0, 1.0],
        "red" => [1.0, 0.0, 0.0],
        "darkBlue" => [0.0, 0.0, 0.5],
        "darkCyan" => [0.0, 0.5, 0.5],
        "darkGreen" => [0.0, 0.5, 0.0],
        "darkMagenta" => [0.5, 0.0, 0.5],
        "darkRed" => [0.5, 0.0, 0.0],
        "darkYellow" => [0.5, 0.5, 0.0],
        "darkGray" => [0.5, 0.5, 0.5],
        "lightGray" => [0.75, 0.75, 0.75],
        "black" => [0.0, 0.0, 0.0],
        "none" => return None,
        _ => return None,
    })
}

pub(super) fn parse_hex_color(val: &str) -> Option<[f32; 3]> {
    let hex = val.trim();
    if hex.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(hex, 16).ok()?;
    Some([
        ((n >> 16) & 0xff) as f32 / 255.0,
        ((n >> 8) & 0xff) as f32 / 255.0,
        (n & 0xff) as f32 / 255.0,
    ])
}

pub(super) fn load_page_setup(dom: &Dom, body: NodeId, fallback: &PageSetup) -> PageSetup {
    let Some(sect) = live_sect_prs(dom, body).into_iter().next() else {
        return *fallback;
    };
    apply_sect_pr(dom, sect, fallback)
}

pub(super) fn apply_sect_pr(dom: &Dom, sect: NodeId, fallback: &PageSetup) -> PageSetup {
    let mut page = *fallback;
    // A vertical fallback is stored turned; restore its physical geometry
    // before this section's own values land.
    if page.vertical {
        turn_page(&mut page, false);
    }
    if let Some(sz) = first_named(dom, sect, "pgSz") {
        if let Some(raw) = attr_any(dom, sz, "w").and_then(|s| parse_len(s).map(|pt| (s, pt))) {
            page.width = word_pgsz_pt(raw.0, raw.1);
        }
        if let Some(raw) = attr_any(dom, sz, "h").and_then(|s| parse_len(s).map(|pt| (s, pt))) {
            page.height = word_pgsz_pt(raw.0, raw.1);
        }
        // w:orient only tells the printer: Word lays out the page at the
        // w and h written, even a "landscape" 396 x 612 (docxide
        // handels_messiah_biblical_analysis, live Word).
    }
    if let Some(mar) = first_named(dom, sect, "pgMar") {
        if let Some(v) = attr_any(dom, mar, "left").and_then(parse_len) {
            page.margin_l = v;
        }
        if let Some(v) = attr_any(dom, mar, "right").and_then(parse_len) {
            page.margin_r = v;
        }
        if let Some(v) = attr_any(dom, mar, "top").and_then(parse_len) {
            page.margin_t = v.abs();
            page.top_exact = v < 0.0;
        }
        if let Some(v) = attr_any(dom, mar, "bottom").and_then(parse_len) {
            page.margin_b = v.abs();
            page.bottom_exact = v < 0.0;
        }
        if let Some(v) = attr_any(dom, mar, "header").and_then(parse_len) {
            page.header = v;
        }
        if let Some(v) = attr_any(dom, mar, "footer").and_then(parse_len) {
            page.footer = v;
        }
        // The binding gutter widens the left margin (right with
        // rtlGutter, top with settings gutterAtTop): 007cf2e2's left=1418
        // gutter=567 text starts at 99.25.
        if let Some(g) = attr_any(dom, mar, "gutter")
            .and_then(parse_len)
            .filter(|g| *g > 0.0)
        {
            let rtl = first_named(dom, sect, "rtlGutter")
                .is_some_and(|n| attr_any(dom, n, "val").is_none_or(|v| v != "0" && v != "false"));
            if page.gutter_at_top {
                page.margin_t += g;
            } else if rtl {
                page.margin_r += g;
            } else {
                page.margin_l += g;
            }
        }
    }
    page.valign_center = first_named(dom, sect, "vAlign")
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v == "center" || v == "both");
    if let Some(num) = first_named(dom, sect, "pgNumType") {
        if let Some(start) = attr_any(dom, num, "start").and_then(|s| s.parse::<u32>().ok()) {
            // 0 is a start Word honours (3936a8fe56 numbers its title
            // page 0 and its third page 2).
            page.page_num_start = Some(start);
        }
        page.page_num_fmt = match attr_any(dom, num, "fmt").unwrap_or("") {
            "lowerRoman" => PageNumFmt::LowerRoman,
            "upperRoman" => PageNumFmt::UpperRoman,
            "lowerLetter" => PageNumFmt::LowerLetter,
            "upperLetter" => PageNumFmt::UpperLetter,
            "decimalZero" => PageNumFmt::DecimalZero,
            "cardinalText" => PageNumFmt::CardinalText,
            "ordinal" => PageNumFmt::Ordinal,
            "ordinalText" => PageNumFmt::OrdinalText,
            "hex" => PageNumFmt::Hex,
            "chicago" => PageNumFmt::Chicago,
            "ideographDigital" => PageNumFmt::IdeographDigital,
            "ideographTraditional" => PageNumFmt::IdeographTraditional,
            "ideographZodiac" => PageNumFmt::IdeographZodiac,
            "ideographZodiacTraditional" => PageNumFmt::IdeographZodiacTraditional,
            "ideographLegalTraditional" => PageNumFmt::IdeographLegalTraditional,
            "ideographEnclosedCircle" => PageNumFmt::IdeographEnclosedCircle,
            "japaneseCounting" => PageNumFmt::JapaneseCounting,
            "aiueo" => PageNumFmt::Aiueo,
            "iroha" => PageNumFmt::Iroha,
            "decimalFullWidth" => PageNumFmt::DecimalFullWidth,
            "ganada" => PageNumFmt::Ganada,
            "chosung" => PageNumFmt::Chosung,
            "decimalEnclosedCircle" => PageNumFmt::DecimalEnclosedCircle,
            "decimalEnclosedParen" => PageNumFmt::DecimalEnclosedParen,
            "decimalEnclosedFullstop" => PageNumFmt::DecimalEnclosedFullstop,
            "hebrew1" => PageNumFmt::Hebrew1,
            "hebrew2" => PageNumFmt::Hebrew2,
            "arabicAlpha" => PageNumFmt::ArabicAlpha,
            "arabicAbjad" => PageNumFmt::ArabicAbjad,
            "russianLower" => PageNumFmt::RussianLower,
            "russianUpper" => PageNumFmt::RussianUpper,
            "thaiNumbers" => PageNumFmt::ThaiNumbers,
            "thaiLetters" => PageNumFmt::ThaiLetters,
            "thaiCounting" => PageNumFmt::ThaiCounting,
            "hindiNumbers" => PageNumFmt::HindiNumbers,
            "hindiVowels" => PageNumFmt::HindiVowels,
            "hindiConsonants" => PageNumFmt::HindiConsonants,
            "hindiCounting" => PageNumFmt::HindiCounting,
            "koreanCounting" => PageNumFmt::KoreanCounting,
            "koreanDigital" => PageNumFmt::KoreanDigital,
            "koreanDigital2" => PageNumFmt::KoreanDigital2,
            "koreanLegal" => PageNumFmt::KoreanLegal,
            _ => PageNumFmt::Decimal,
        };
        if let Some(ch) = attr_any(dom, num, "chapStyle").and_then(|s| s.parse::<u32>().ok())
            && ch >= 1
        {
            page.chap_style = Some(ch);
        }
        page.chap_sep = match attr_any(dom, num, "chapSep").unwrap_or("hyphen") {
            "period" => ".",
            "colon" => ":",
            "emDash" => "—",
            "enDash" => "–",
            _ => "-",
        };
    }
    page.borders = PageBorders::default();
    if let Some(pb) = first_named(dom, sect, "pgBorders") {
        page.borders = parse_pg_borders(dom, pb);
    }
    if let Some(cols) = first_named(dom, sect, "cols") {
        page.col_count = attr_any(dom, cols, "num")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(1)
            .max(1);
        if let Some(sp) = attr_any(dom, cols, "space").and_then(parse_len) {
            page.col_space = sp;
        }
        page.col_sep = matches!(attr_any(dom, cols, "sep"), Some("1" | "true" | "on"));
        let equal = !matches!(
            attr_any(dom, cols, "equalWidth").unwrap_or("1"),
            "0" | "false" | "off"
        );
        if !equal {
            let mut n = 0u8;
            let mut widths = [0.0_f32; 4];
            let mut gaps = [0.0_f32; 4];
            for col in descendants_local(dom, cols, "col") {
                let i = n as usize;
                if i >= 4 {
                    break;
                }
                widths[i] = attr_any(dom, col, "w").and_then(parse_len).unwrap_or(0.0);
                gaps[i] = attr_any(dom, col, "space")
                    .and_then(parse_len)
                    .unwrap_or(page.col_space);
                n += 1;
            }
            if n > 0 {
                page.col_custom = n;
                page.col_count = n;
                page.col_w = widths;
                page.col_gap = gaps;
            }
        }
    }
    page.ln_count_by = 0;
    page.ln_start = 1;
    page.ln_distance = 0.0;
    page.ln_restart = 0;
    if let Some(ln) = first_named(dom, sect, "lnNumType") {
        page.ln_count_by = attr_any(dom, ln, "countBy")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);
        page.ln_start = attr_any(dom, ln, "start")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        page.ln_distance = attr_any(dom, ln, "distance")
            .and_then(parse_len)
            .unwrap_or(0.0);
        page.ln_restart = match attr_any(dom, ln, "restart").unwrap_or("newPage") {
            "newSection" => 1,
            "continuous" => 2,
            _ => 0,
        };
    }
    page.grid_pitch = 0.0;
    page.grid_char = 0.0;
    if let Some(grid) = first_named(dom, sect, "docGrid") {
        let ty = attr_any(dom, grid, "type").unwrap_or("default");
        if matches!(ty, "lines" | "linesAndChars") {
            page.grid_pitch = attr_any(dom, grid, "linePitch")
                .and_then(parse_len)
                .unwrap_or(0.0);
        }
        if matches!(ty, "snapToChars" | "linesAndChars") {
            // 4096ths of a point (ECMA-376 17.6.5), not twips: 0016d88a's
            // -4301 is Word's 10.5 → 9.45pt CJK pitch, not -215pt a glyph.
            page.grid_char = attr_any(dom, grid, "charSpace")
                .and_then(parse_finite_f32)
                .map_or(0.0, |v| v / 4096.0);
        }
    }
    let vertical = first_named(dom, sect, "textDirection")
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| matches!(v, "tbRl" | "tbRlV"));
    if vertical {
        turn_page(&mut page, true);
    }
    page
}

/// Turns a section's page a quarter for vertical text (`on`), or back.
/// Lines run down the physical page and stack right to left: the laid-out
/// page's left/right margins are the physical top/bottom, its top the
/// physical right and its bottom the physical left.
pub(super) fn turn_page(page: &mut PageSetup, on: bool) {
    std::mem::swap(&mut page.width, &mut page.height);
    let (l, r, t, b) = (page.margin_l, page.margin_r, page.margin_t, page.margin_b);
    if on {
        (page.margin_l, page.margin_r, page.margin_t, page.margin_b) = (t, b, r, l);
    } else {
        (page.margin_l, page.margin_r, page.margin_t, page.margin_b) = (b, t, l, r);
    }
    page.vertical = on;
}

/// Single-byte-width characters: ASCII and the half-width forms block.
pub(super) fn is_half_width(c: char) -> bool {
    c.is_ascii() || ('\u{FF61}'..='\u{FFDC}').contains(&c)
}

/// The docGrid pitch a paragraph's lines snap to: none for an exact line
/// (0016d88a's exact 10.6pt line stays 10.6 under a 14.3pt grid) or a
/// paragraph with snapToGrid off.
pub(super) fn para_grid_pitch(style: &ParaStyle, pitch: f32) -> f32 {
    if style.line_exact.is_some() || !style.snap_to_grid {
        0.0
    } else {
        pitch
    }
}

pub(super) fn snap_doc_grid(h: f32, pitch: f32) -> f32 {
    if pitch <= 0.5 {
        h
    } else {
        (h / pitch).ceil().max(1.0) * pitch
    }
}

pub(super) fn parse_pg_borders(dom: &Dom, pb: NodeId) -> PageBorders {
    // MS-OI29500 17.6.10: an omitted offsetFrom is `text`.
    let from_page = attr_any(dom, pb, "offsetFrom") == Some("page");
    let edge = |name: &str| -> Option<PageBorder> {
        let el = first_named(dom, pb, name)?;
        let val = attr_any(dom, el, "val").unwrap_or("single");
        if val == "nil" || val == "none" {
            return None;
        }
        let color = attr_any(dom, el, "color")
            .and_then(parse_hex_color)
            .unwrap_or([0.0, 0.0, 0.0]);
        let width = attr_any(dom, el, "sz")
            .and_then(parse_finite_f32)
            .map(|eighths| {
                let pt = eighths / 8.0;
                if pt < 0.5 { 0.24 } else { pt }
            })
            .unwrap_or(0.6);
        let space = attr_any(dom, el, "space")
            .and_then(parse_finite_f32)
            .unwrap_or(24.0);
        let line = match val {
            "double" => BorderLine::Double,
            "dotted" => BorderLine::Dotted,
            "dashed" | "dashSmallGap" | "dotDash" | "dotDotDash" => BorderLine::Dashed,
            "thinThickMediumGap" | "thickThinMediumGap" => BorderLine::MediumGap,
            _ => BorderLine::Single,
        };
        Some(PageBorder {
            color,
            width,
            space,
            line,
        })
    };
    PageBorders {
        top: edge("top"),
        left: edge("left"),
        bottom: edge("bottom"),
        right: edge("right"),
        from_page,
        display: match attr_any(dom, pb, "display") {
            Some("firstPage") => BorderDisplay::FirstPage,
            Some("notFirstPage") => BorderDisplay::NotFirstPage,
            _ => BorderDisplay::AllPages,
        },
        back: attr_any(dom, pb, "zOrder") == Some("back"),
    }
}
