// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Paragraph blocks: base formatting, horizontal rules, revision marks and
//! flow pictures.

use super::*;

pub(super) fn para_base(
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
    table_spacing: Option<&TableParaSpacing>,
) -> (ParaStyle, RunStyle) {
    let table_para = table_spacing.map(|t| t.para);
    let mut pstyle = sheet.defaults.para.clone();
    let mut rstyle = sheet.defaults.run.clone();
    if let Some(t) = table_para {
        // Table style pPr sits between docDefaults and the cell's
        // pStyle/pPr; an unstyled table (TableNormal, no pPr) keeps
        // Normal's spacing (0073da0a). Direct cell spacing still wins via
        // apply_ppr below (xml 3.3 ckpt 2).
        // Except what the default paragraph style sets itself (checked in
        // Word: Normal's own after=200 line=276 keep 24.96pt rows under
        // Table Grid's after=0 line=240; inherited from docDefaults, the
        // table style's 13.2pt win).
        let [own_after, own_before, own_line] = sheet.defaults.normal_spacing;
        if !own_after {
            pstyle.after = t.after;
        }
        if !own_before {
            pstyle.before = t.before;
        }
        if !own_line {
            pstyle.line_mult = t.line_mult;
            pstyle.line_exact = t.line_exact;
            pstyle.line_at_least = t.line_at_least;
        }
    }
    if let Some(t) = table_spacing {
        let (own_ind, own_jc) = sheet.defaults.normal_ind_jc;
        take_table_ind_jc(&mut pstyle, t, (own_ind, own_jc));
    }
    if let Some(ppr) = dom.element(para, &W::p_pr())
        && let Some(ps) = first_named(dom, ppr, "pStyle")
        && let Some(sid) = dom.attribute(ps, &W::val())
    {
        if sheet.by_id.get(sid).is_some_and(|named| named.not_para) {
            // A character style as pStyle (redlines vs 0004c94c's
            // "Hyperlink" header paragraph): Word keeps Normal, not the
            // document defaults under the character style.
        } else if let Some(named) = sheet.by_id.get(sid) {
            pstyle = named.para.clone();
            rstyle = named.run.clone();
            if let Some(t) = table_spacing {
                keep_table_spacing_unset_by_style(&mut pstyle, t, named.sets_spacing);
                take_table_ind_jc(&mut pstyle, t, (named.sets_ind.0, named.sets_jc));
            }
        } else {
            // Word still applies latent built-in heading spacing when the
            // style is referenced but omitted from styles.xml (the
            // heading_*_style_demo fixtures). Direct pPr below wins.
            if sheet.latent {
                apply_latent_ppr(sid, &mut pstyle, &mut rstyle, &sheet.theme);
            }
        }
        pstyle.style_id = sid.to_string();
    }
    if let Some(ppr) = dom.element(para, &W::p_pr()) {
        apply_ppr(dom, ppr, &mut pstyle);
    }
    // Spacing merges per attribute: a plain w:after/w:before does not
    // cancel the table style's auto spacing, only an explicit
    // w:afterAutospacing / w:beforeAutospacing does (00319da4's header
    // cells write after=100 under Table Grid's afterAutospacing=1; Word
    // keeps them one line apart, and gives the 5pt back only with
    // afterAutospacing="0").
    if let Some(t) = table_para {
        let spacing = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| first_named(dom, ppr, "spacing"));
        let sets = |name: &str| spacing.is_some_and(|sp| attr_any(dom, sp, name).is_some());
        if t.after_auto && !pstyle.after_auto && !sets("afterAutospacing") {
            pstyle.after = t.after;
            pstyle.after_auto = true;
        }
        if t.before_auto && !pstyle.before_auto && !sets("beforeAutospacing") {
            pstyle.before = t.before;
            pstyle.before_auto = true;
        }
    }
    if pstyle.bidi {
        mirror_bidi(&mut pstyle);
    }
    if let Some(align) = display_math_align(dom, para) {
        pstyle.align = align;
    }
    pstyle.fmt_rev = para_formatting_changed(dom, para)
        || para_mark_revised(dom, para)
        || para_object_revised(dom, para);
    rstyle.auto_space_de_off = pstyle.auto_space_de_off;
    rstyle.auto_space_dn_off = pstyle.auto_space_dn_off;
    (pstyle, rstyle)
}

/// The alignment of a display-equation paragraph: one that holds an
/// `m:oMathPara` and no text of its own. Word centres the equation (its
/// default `centerGroup`, or `center`) unless the `m:oMathParaPr/m:jc`
/// says `left` or `right`, whatever the paragraph's own `w:jc`.
pub(super) fn display_math_align(dom: &Dom, para: NodeId) -> Option<Align> {
    let math_para = M::name("oMathPara");
    let display = dom
        .descendants(para, Some(&math_para))
        .into_iter()
        .find(|&m| dom.ancestors(m, Some(&W::p())).first() == Some(&para))?;
    // Deleted text counts: a markup view paints it beside the equation.
    let own_text = [W::t(), W::del_text()].iter().any(|name| {
        dom.descendants(para, Some(name)).into_iter().any(|t| {
            dom.ancestors(t, Some(&math_para)).is_empty() && !element_text(dom, t).is_empty()
        })
    });
    if own_text {
        return None;
    }
    let jc = dom
        .element(display, &M::name("oMathParaPr"))
        .and_then(|pr| dom.element(pr, &M::name("jc")))
        .and_then(|jc| {
            dom.attribute(jc, &M::name("val"))
                .or_else(|| attr_any(dom, jc, "val"))
        });
    Some(match jc {
        Some("left") => Align::Left,
        Some("right") => Align::Right,
        _ => Align::Center,
    })
}

/// A right-to-left paragraph as the left-to-right one it paints like: its
/// start is the right margin, so jc left/right and ind left/right swap
/// (checked in Word: no jc and jc="left" end at the right margin,
/// jc="right" starts at the left one, ind left indents from the right).
fn mirror_bidi(style: &mut ParaStyle) {
    style.align = match style.align {
        Align::Left => Align::Right,
        Align::Right => Align::Left,
        other => other,
    };
    std::mem::swap(&mut style.indent_left, &mut style.indent_right);
}

/// Word 2007/365 latent built-ins when `styles.xml` has no definition.
/// Heading 3/4 keep after=0 (heading_3_center gap 34.6pt). Heading1
/// after=0 is ITT-neg (red_bold_heading 90→72) — size/bold/face only.
/// Title has no Cambria Light FaceId: paint theme major at 28pt.
fn apply_latent_ppr(style_id: &str, para: &mut ParaStyle, run: &mut RunStyle, theme: &ThemeFonts) {
    let aptos = theme
        .major
        .as_deref()
        .is_some_and(|s| s.to_ascii_lowercase().contains("aptos"));
    let major = theme.major.clone().unwrap_or_else(|| {
        if aptos {
            "Aptos Display".into()
        } else {
            "Cambria".into()
        }
    });
    match style_id {
        "Title" => {
            run.size = 28.0;
            run.family = major;
        }
        "Subtitle" => {
            run.italic = true;
            run.size = 12.0;
            run.family = major;
        }
        "Heading1" => {
            run.bold = true;
            run.size = if aptos { 20.0 } else { 14.0 };
            run.family = major;
        }
        "Heading2" => {
            run.bold = true;
            run.italic = true;
            run.size = 13.0;
            run.family = major;
        }
        "Heading3" => {
            para.before = 10.0;
            para.after = 0.0;
            run.bold = true;
            run.size = 12.0;
            run.family = major;
        }
        "Heading4" => {
            para.before = 10.0;
            para.after = 0.0;
            run.bold = true;
            run.italic = true;
            run.size = 11.0;
            run.family = major;
        }
        "Heading5" => {
            run.italic = true;
            run.size = 11.0;
            run.family = major;
        }
        "Heading6" => {
            run.size = 11.0;
            run.family = major;
        }
        _ => {}
    }
}

/// A VML horizontal line: Word paints it on its paragraph's baseline.
#[derive(Clone, Copy)]
pub(super) struct HRule {
    /// Share of the text width (`o:hrpct`/1000; 0 means all of it).
    pub(super) frac: f32,
    /// An explicit `width` under `o:hrpct="0"` (000014a9's 108pt rules).
    pub(super) width: Option<f32>,
    pub(super) h: f32,
    pub(super) color: [f32; 3],
    pub(super) align: Align,
}

/// A VML colour: `#rrggbb`, `#rgb` or one of the 16 named colours, any
/// ` [n]` scheme index after it ignored (`black [3213]`).
pub(super) fn vml_color(v: &str) -> Option<[f32; 3]> {
    let v = v.split_whitespace().next()?;
    if let Some(hex) = v.strip_prefix('#') {
        if hex.len() == 3 {
            let long: String = hex.chars().flat_map(|c| [c, c]).collect();
            return parse_hex_color(&long);
        }
        return parse_hex_color(hex);
    }
    let hex = match v.to_ascii_lowercase().as_str() {
        "black" => "000000",
        "white" => "FFFFFF",
        "gray" | "grey" => "808080",
        "silver" => "C0C0C0",
        "red" => "FF0000",
        "maroon" => "800000",
        "yellow" => "FFFF00",
        "olive" => "808000",
        "lime" => "00FF00",
        "green" => "008000",
        "aqua" => "00FFFF",
        "teal" => "008080",
        "blue" => "0000FF",
        "navy" => "000080",
        "fuchsia" => "FF00FF",
        "purple" => "800080",
        _ => return None,
    };
    parse_hex_color(hex)
}

/// An `o:` (urn:schemas-microsoft-com:office:office) attribute.
pub(super) fn o_attr<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str> {
    dom.attribute(
        node,
        &XName::get(local, "urn:schemas-microsoft-com:office:office"),
    )
}

/// A tracked formatting change on the paragraph or any of its runs
/// (`w:pPrChange`, `w:rPrChange`). Word draws the change bar beside such a
/// paragraph even when no text was inserted or deleted (pool p0016, p0033,
/// p0111, p0225).
fn para_formatting_changed(dom: &Dom, para: NodeId) -> bool {
    [W::p_pr_change(), W::r_pr_change()].iter().any(|n| {
        dom.find_descendant_element(para, Some(n), |cand| !under_prior_change(dom, para, cand))
            .is_some()
    })
}

/// The paragraph mark itself is inserted, deleted or moved (`pPr/rPr` holds
/// the revision): Word bars the paragraph even when it has no text
/// (9b22b88370's empty paragraphs between deleted tables).
fn para_mark_revised(dom: &Dom, para: NodeId) -> bool {
    dom.element(para, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.elements(rpr, None)
                .into_iter()
                .any(|c| rev_mark_of(dom, c).is_some())
        })
}

/// A picture, drawing or embedded object of the paragraph sits in a
/// tracked insertion, deletion or move: no text run carries that revision,
/// yet Word bars its line.
fn para_object_revised(dom: &Dom, para: NodeId) -> bool {
    ["drawing", "pict", "object"].into_iter().any(|name| {
        descendants_local(dom, para, name)
            .into_iter()
            .any(|object| {
                dom.ancestors(object, None)
                    .into_iter()
                    .take_while(|&a| a != para)
                    .any(|a| {
                        ["ins", "del", "moveTo", "moveFrom"]
                            .into_iter()
                            .any(|rev| local_name_is(dom, a, rev))
                    })
            })
    })
}

/// The paragraph's `v:rect o:hr="t"` (not a Fallback copy), if any.
pub(super) fn para_hrule(dom: &Dom, para: NodeId) -> Option<HRule> {
    let rect = descendants_local(dom, para, "rect").into_iter().find(|r| {
        o_attr(dom, *r, "hr") == Some("t")
            && !dom
                .ancestors(*r, None)
                .iter()
                .any(|a| local_name_is(dom, *a, "Fallback"))
    })?;
    let h = attr_any(dom, rect, "style")
        .and_then(|st| vml_style_pt(st, "height"))
        .unwrap_or(1.5);
    let pct = o_attr(dom, rect, "hrpct")
        .and_then(parse_finite_f32)
        .unwrap_or(0.0);
    let fill = attr_any(dom, rect, "fillcolor")
        .and_then(vml_color)
        .unwrap_or([0.627, 0.627, 0.627]);
    let width = attr_any(dom, rect, "style")
        .and_then(|st| vml_style_pt(st, "width"))
        .filter(|w| *w > 0.0 && pct <= 0.0);
    // Word shades a rule without o:hrnoshade as a bevel whose face is far
    // lighter than its fillcolor (fixtures_500 00b540dd: lumas 202-229
    // under #a0a0a0); half toward white stands for it.
    let color = if o_attr(dom, rect, "hrnoshade") == Some("t") {
        fill
    } else {
        fill.map(|c| (c + 1.0) / 2.0)
    };
    let align = match o_attr(dom, rect, "hralign") {
        Some("center") => Align::Center,
        Some("right") => Align::Right,
        _ => Align::Left,
    };
    Some(HRule {
        frac: if pct > 0.0 { pct / 1000.0 } else { 1.0 },
        width,
        h,
        color,
        align,
    })
}

/// Attach each spliced picture placeholder to its `LaidImage` and drop
/// that image from the paragraph, so the line breaker paints it once.
/// A placeholder whose blip was not collected (a group, a float, a text
/// box) is dropped rather than painted as a missing glyph.
pub(super) fn bind_flow_pictures(runs: &mut Vec<TextRun>, images: &mut Vec<LaidImage>) {
    if runs.iter().all(|r| r.pic_src.is_none()) {
        return;
    }
    let mut by_src: HashMap<u32, Rc<LaidImage>> = HashMap::new();
    for img in images.iter() {
        if img.src != 0 {
            by_src
                .entry(img.src)
                .or_insert_with(|| Rc::new(img.clone()));
        }
    }
    let mut used = HashSet::new();
    let mut kept = Vec::with_capacity(runs.len());
    for mut run in runs.drain(..) {
        if let Some(src) = run.pic_src.take() {
            if let Some(img) = by_src.get(&src) {
                run.pic = Some(Rc::clone(img));
                used.insert(src);
                kept.push(run);
            }
        } else {
            kept.push(run);
        }
    }
    *runs = kept;
    images.retain(|img| img.src == 0 || !used.contains(&img.src));
}

pub(super) fn paragraph_block(
    ctx: &WalkCtx<'_>,
    dom: &Dom,
    para: NodeId,
    in_table: bool,
    numbering: &mut Numbering,
) -> Block {
    let sheet = ctx.sheet;
    let (mut pstyle, rstyle) = para_base(dom, para, sheet, None);
    pstyle.hrule = para_hrule(dom, para);
    let (marker, num_id, ilvl) = list_marker(dom, para, sheet, numbering);
    // numId=0 over a numbered style removes the list and the style's list
    // indent with it (000ebd12 Förslagstext: ind 397/397 renders flush
    // left in Word). A direct w:ind still wins.
    if num_id == "0" {
        let ppr = dom.element(para, &W::p_pr());
        let style_numbered = ppr
            .and_then(|ppr| first_named(dom, ppr, "pStyle"))
            .and_then(|ps| dom.attribute(ps, &W::val()))
            .and_then(|sid| sheet.by_id.get(sid))
            .is_some_and(|named| named.num_id.as_deref().is_some_and(|id| id != "0"));
        // A direct w:ind lands on the cleared indent attribute by
        // attribute: 019d92d9's `left=270` alone starts flush at 270, the
        // style's hanging gone (live Word).
        if style_numbered {
            let direct_ind = ppr.and_then(|ppr| first_named(dom, ppr, "ind"));
            let twips = |names: &[&str]| {
                direct_ind.and_then(|ind| {
                    names
                        .iter()
                        .find_map(|n| attr_any(dom, ind, n))
                        .and_then(parse_finite_f32)
                        .map(|v| v / 20.0)
                })
            };
            pstyle.indent_left = twips(&["left", "start"]).unwrap_or(0.0);
            pstyle.indent_first = match (twips(&["hanging"]), twips(&["firstLine"])) {
                (Some(h), _) => -h,
                (None, Some(f)) => f,
                (None, None) => 0.0,
            };
        }
    }
    if !marker.is_empty() {
        pstyle.list_num.clone_from(&num_id);
    }
    let pic = numbering.pic_image(&num_id, ilvl);
    let pic_extent = numbering.pic_extent(&num_id, ilvl);
    if pstyle.outline_lvl.is_some() && !num_id.is_empty() {
        pstyle.chap_num = numbering.last_used(&num_id, ilvl).map(|n| n.to_string());
    }
    let mut runs = collect_runs_in(
        dom,
        para,
        &rstyle,
        &sheet.theme,
        Some(&sheet.by_id),
        &mut RunBag {
            authors: &mut ctx.authors.borrow_mut(),
            comments: &ctx.comments,
            in_table,
            toc: is_toc_style(&pstyle),
            open: Some(&ctx.open_comments),
            flow_pics: true,
        },
    );
    if marker.is_empty()
        && !num_id.is_empty()
        && let Some(lvl) = numbering.level(&num_id, ilvl)
        && numbering.linked_style(&num_id, ilvl).is_some()
    {
        apply_list_level(dom, para, sheet, Some(lvl), &rstyle, &mut pstyle);
    }
    if !marker.is_empty() {
        let lvl = numbering.level(&num_id, ilvl);
        let marker_style = apply_list_level(dom, para, sheet, lvl, &rstyle, &mut pstyle);
        if pic.is_none() {
            let mut mark = TextRun::new(marker, marker_style);
            mark.list_marker = true;
            runs.insert(0, mark);
        }
        if pic.is_none() {
            revise_list_label(
                &mut runs,
                label_rev(dom, para, sheet, &num_id, ilvl),
                numbering.old_label.take(),
                &mut ctx.authors.borrow_mut(),
            );
        }
    }
    let mut images = collect_images(ctx.pkg, ctx.main, dom, para, &|drawing| {
        picture_run_style(dom, drawing, &rstyle, &sheet.theme, &sheet.by_id)
    });
    if let Some(kind) = pic {
        // The bullet's own v:shape extent; the run size only when absent.
        let size = rstyle.size.max(8.0);
        let (w, h) = pic_extent.unwrap_or((size, size));
        images.insert(
            0,
            LaidImage {
                group_reserved: false,
                w,
                h,
                kind,
                slot: ImageSlot::Flow,
                behind: false,
                z: 0,
                crop: None,
                rotate_deg: 0.0,
                oval: false,
                chrome_align: Align::Left,
                chrome_lead: false,
                chrome_flow: false,
                chrome_under_table: false,
                chrome_under_text: false,
                inset: [0.0; 4],
                chrome_leading: None,
                chrome_tab_line: None,
                chrome_text_under: None,
                chrome_above: 0.0,
                chrome_drop: 0.0,
                chrome_drop_tab: None,
                chrome_para: 0,
                chrome_after: 0.0,
                tail_anchor: false,
                outline: None,
                gap_before: 0.0,
                lead_chars: 0,
                at_chars: 0,
                after_text: false,
                under: None,
                chrome_flat_run: None,
                src: 0,
            },
        );
    }
    bind_flow_pictures(&mut runs, &mut images);
    let boxes = collect_textboxes_styled(
        Some((ctx.pkg, ctx.main)),
        dom,
        para,
        &rstyle,
        &sheet.theme,
        Some(sheet),
        Some(numbering),
    );
    // Word paints empty TitlePage/DocumentTitle with the style's rPr
    // (Arial 18 / exact 20 / after 24). Factory Calibri 11 stretched
    // DocumentTitle→date to 88pt and dropped the cover's 18pt spaces.
    // Do not stamp Normal 11 (file_146 Inter→Cambria): that swapped a
    // 15.4pt Calibri em-box for 12.65 and collapsed 7pp→6.
    pstyle.empty_toc_field = para_is_empty_toc_field(dom, para);
    // An installed style face is safe to stamp: fixtures_500 014babb2
    // empty Normal (Times New Roman 12, double) lines are 27.6pt in Word,
    // not a factory Calibri 11 line (26.85).
    // The mark's own pPr/rPr sizes an empty line (fixtures_500 000b1b49:
    // TNR 10 marks under a Calibri 11 Normal are 17.25pt lines at 1.5).
    let mark_rpr = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .filter(|rpr| {
            ["sz", "rFonts", "rStyle"]
                .iter()
                .any(|name| first_named(dom, *rpr, name).is_some())
        });
    // A floating picture takes no line space: its paragraph is still a
    // line of its mark (0004c94c's emblem paragraph is Arial 9, 10.35pt).
    // A picture spliced into the runs is a line of its own too.
    let run_flow = runs.iter().any(|r| r.pic.is_some());
    let floats_only = !run_flow
        && images
            .iter()
            .all(|img| !matches!(img.slot, ImageSlot::Flow));
    // Floating text boxes take no line space either (00bf6b4c's org chart
    // hangs its boxes off empty 20pt paragraphs).
    let boxes_float = boxes.iter().all(|b| !matches!(b.slot, ImageSlot::Flow));
    if let Some(rpr) = mark_rpr {
        let mut mark = rstyle.clone();
        apply_mark_rpr(dom, rpr, &mut mark, sheet);
        pstyle.mark_run = Some(std::rc::Rc::new(mark));
    } else if (!floats_only
        || boxes
            .iter()
            .any(|b| matches!(b.slot, ImageSlot::Flow) && !b.diag_shapes.is_empty()))
        && runs
            .iter()
            .all(|r| r.pic.is_some() || r.text.trim().is_empty())
    {
        // An inline picture's line takes its multiple's extra from the
        // paragraph's own run style even when the mark sets nothing
        // (docxide case78: 1.15 over Arial 11 adds 1.9pt under each
        // picture), and so does an inline SmartArt diagram's.
        pstyle.mark_run = Some(std::rc::Rc::new(rstyle.clone()));
    }
    // A paragraph of plain spaces is a line of its mark, like an empty one
    // (checked in Word: 28pt spaces under a 12pt mark make a 12pt line,
    // 12pt spaces under a 28pt mark a 28pt one; 00195f87's 12pt spaces
    // under a 14pt mark stood 2.1pt short). An ideographic space counts
    // (live Word: a 48pt U+3000 under a 12pt mark is a 12pt line;
    // 0041d394's 48pt one kept Word's poster on one page). So do tabs,
    // deleted or not (Word 16 probes tab1001 t1/t2/t4/t7: an 11pt or 24pt
    // tab alone over an 8pt mark is an 8pt line; t899ef4's deleted tab
    // paragraph stood 3.8pt too tall).
    // A note separator's run sizes its own line (Word 16 probes 2026-10-03:
    // a 20pt `w:separator` run reserves 24.41, a 20pt mark over an 11pt
    // run 13.43), so it is no blank.
    let spaces_only = !runs.is_empty()
        && runs.iter().all(|r| {
            !r.list_marker
                && !r.note_rule
                && matches!(r.field, FieldKind::None)
                && r.text.chars().all(|c| matches!(c, ' ' | '\u{3000}' | '\t'))
        });
    if spaces_only && floats_only && boxes_float {
        let mut mark = rstyle.clone();
        if let Some(rpr) = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        {
            apply_mark_rpr(dom, rpr, &mut mark, sheet);
        }
        for run in &mut runs {
            run.style.family.clone_from(&mark.family);
            run.style.bold = mark.bold;
            run.style.italic = mark.italic;
            run.style.size = mark.size;
        }
    }
    if runs.is_empty() && floats_only && boxes_float {
        if let Some(rpr) = mark_rpr {
            let mut mark = rstyle.clone();
            apply_mark_rpr(dom, rpr, &mut mark, sheet);
            runs.push(TextRun::new(" ", mark));
        } else if pstyle.line_exact.is_some()
            || rstyle.size >= 14.0
            || Fonts::is_installed_family(&rstyle.family)
        {
            runs.push(TextRun::new(" ", rstyle));
        }
    }
    Block::Paragraph {
        runs,
        style: pstyle,
        list: false, // marker already prepended
        images,
        boxes,
        bookmarks: para_bookmark_names(dom, para),
    }
}

pub(super) fn para_bookmark_names(dom: &Dom, para: NodeId) -> Vec<String> {
    let want = W::bookmark_start();
    let mut names = Vec::new();
    collect_bookmark_names(dom, para, &want, &mut names);
    names
}

fn collect_bookmark_names(dom: &Dom, node: NodeId, want: &XName, out: &mut Vec<String>) {
    if dom.name_is(node, want)
        && let Some(name) = attr_any(dom, node, "name")
    {
        out.push(name.to_string());
    }
    for i in 0..dom.child_count(node) {
        collect_bookmark_names(dom, dom.child_at(node, i), want, out);
    }
}

/// Every bookmarked paragraph in document order, table cells (and their
/// nested tables) included: `(own names, blank-paragraph names, runs)`.
fn visit_bookmarked_paras<'b>(
    blocks: &'b [Block],
    visit: &mut impl FnMut(&'b [String], &'b [String], &'b [TextRun]),
) {
    for block in blocks {
        match block {
            Block::Paragraph {
                bookmarks, runs, ..
            } => visit(bookmarks, &[], runs),
            Block::Table { rows, .. } => {
                for cell in rows.iter().flatten() {
                    // Cell paragraphs and nested tables in document order.
                    for (pi, para) in cell.paras.iter().enumerate() {
                        for (nested, _) in cell
                            .nested
                            .iter()
                            .zip(&cell.nested_at)
                            .filter(|(_, at)| **at == pi)
                        {
                            visit_bookmarked_paras(std::slice::from_ref(nested), visit);
                        }
                        visit(&para.bookmarks, &para.blank_bookmarks, &para.runs);
                    }
                    for (nested, _) in cell
                        .nested
                        .iter()
                        .zip(&cell.nested_at)
                        .filter(|(_, at)| **at >= cell.paras.len())
                    {
                        visit_bookmarked_paras(std::slice::from_ref(nested), visit);
                    }
                }
            }
            Block::PageBreak { .. } | Block::ColumnBreak | Block::SectionCols { .. } => {}
        }
    }
}

pub(super) fn document_bookmark_names(blocks: &[Block]) -> HashSet<String> {
    let mut names = HashSet::new();
    visit_bookmarked_paras(blocks, &mut |own, blank, _| {
        names.extend(own.iter().chain(blank).cloned());
    });
    names
}

pub(super) fn document_bookmark_texts(blocks: &[Block]) -> HashMap<String, String> {
    let mut texts = HashMap::new();
    visit_bookmarked_paras(blocks, &mut |own, blank, runs| {
        for name in blank {
            texts.entry(name.clone()).or_insert_with(String::new);
        }
        if own.is_empty() {
            return;
        }
        let text: String = runs
            .iter()
            .filter(|r| !r.comment_marker && r.pageref.is_none() && r.ref_name.is_none())
            .map(|r| r.text.as_str())
            .collect();
        for name in own {
            texts.entry(name.clone()).or_insert_with(|| text.clone());
        }
    });
    texts
}
