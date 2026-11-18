// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Headers and footers: section selection, chrome parts, tables, watermarks
//! and alternate content.

use super::*;

/// Header/footer chrome from the first section's `sectPr` refs only.
/// Scanning every `word/header*.xml` / `word/footer*.xml` concatenates unused
/// leftover parts (sd_2517 ships 19 footers; only later sections reference them).
#[derive(Clone, Default)]
pub(super) struct HfChrome {
    pub(super) header: Vec<TextRun>,
    pub(super) footer: Vec<TextRun>,
    pub(super) header_align: Align,
    pub(super) footer_align: Align,
    pub(super) header_bottom: Option<([f32; 3], f32)>,
    pub(super) footer_top: Option<([f32; 3], f32)>,
    pub(super) watermark: Option<Watermark>,
    pub(super) header_rest: Option<ChromePart>,
    pub(super) footer_rest: Option<ChromePart>,
    pub(super) even_and_odd: bool,
    pub(super) header_even: Option<ChromePart>,
    pub(super) header_odd: Option<ChromePart>,
    pub(super) footer_even: Option<ChromePart>,
    pub(super) footer_odd: Option<ChromePart>,
    pub(super) header_images: Vec<LaidImage>,
    pub(super) footer_images: Vec<LaidImage>,
    pub(super) header_boxes: std::rc::Rc<Vec<LaidTextBox>>,
    pub(super) footer_boxes: std::rc::Rc<Vec<LaidTextBox>>,
    pub(super) header_tables: Vec<ChromeTable>,
    pub(super) footer_tables: Vec<ChromeTable>,
    pub(super) mirror_margins: bool,
    /// `w:evenAndOddHeaders`: change bars sit on the outside border,
    /// right on odd pages (live Word 2026-09-25).
    pub(super) rev_bars_facing: bool,
    pub(super) punct_squeeze: bool,
    /// `w:compat/w:wpJustification` (`Defaults::wp_justify`).
    pub(super) wp_justify: bool,
    /// `w:compat/w:ulTrailSpace` (xml leftover).
    pub(super) ul_trail_space: bool,
    /// `w:compat/w:doNotExpandShiftReturn` (xml leftover).
    pub(super) do_not_expand_shift_return: bool,
}

pub(super) fn first_section_hf(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    body: NodeId,
    sheet: &StyleSheet,
) -> HfChrome {
    let Some(sect) = live_sect_prs(dom, body).into_iter().next() else {
        return HfChrome {
            punct_squeeze: sheet.defaults.punct_squeeze,
            wp_justify: sheet.defaults.wp_justify,
            ul_trail_space: settings_ul_trail_space(pkg),
            do_not_expand_shift_return: settings_do_not_expand_shift_return(pkg),
            ..Default::default()
        };
    };
    let ParityHf {
        header,
        footer,
        even_and_odd,
        header_odd,
        footer_odd,
    } = section_parity_hf(pkg, main, dom, sect, sheet);
    HfChrome {
        header: header.start.runs,
        footer: footer.start.runs,
        header_align: header.start.align,
        footer_align: footer.start.align,
        header_bottom: header.start.border,
        footer_top: footer.start.border,
        watermark: header.start.watermark,
        header_rest: header.rest,
        footer_rest: footer.rest,
        even_and_odd,
        header_even: header.even,
        header_odd,
        footer_even: footer.even,
        footer_odd,
        header_images: header.start.images,
        footer_images: footer.start.images,
        header_boxes: header.start.boxes,
        footer_boxes: footer.start.boxes,
        header_tables: header.start.tables,
        footer_tables: footer.start.tables,
        mirror_margins: settings_mirror_margins(pkg),
        rev_bars_facing: settings_even_and_odd_headers(pkg),
        punct_squeeze: sheet.defaults.punct_squeeze,
        wp_justify: sheet.defaults.wp_justify,
        ul_trail_space: settings_ul_trail_space(pkg),
        do_not_expand_shift_return: settings_do_not_expand_shift_return(pkg),
    }
}

#[derive(Clone, Default)]
pub(super) struct ChromePart {
    pub(super) runs: Vec<TextRun>,
    pub(super) border: Option<([f32; 3], f32)>,
    pub(super) align: Align,
    pub(super) watermark: Option<Watermark>,
    pub(super) images: Vec<LaidImage>,
    pub(super) tables: Vec<ChromeTable>,
    /// Anchored text boxes and shapes (01838a08's footer text box).
    pub(super) boxes: std::rc::Rc<Vec<LaidTextBox>>,
}

fn empty_chrome() -> ChromePart {
    ChromePart {
        runs: Vec::new(),
        border: None,
        align: Align::Left,
        watermark: None,
        images: Vec::new(),
        tables: Vec::new(),
        boxes: std::rc::Rc::new(Vec::new()),
    }
}

fn sect_title_pg(dom: &Dom, sect: NodeId) -> bool {
    first_named(dom, sect, "titlePg").is_some_and(|n| !val_is_false(dom, Some(n)))
}

pub(super) struct PickedHf {
    pub(super) start: ChromePart,
    pub(super) rest: Option<ChromePart>,
    pub(super) even: Option<ChromePart>,
    pub(super) odd: ChromePart,
}

/// The relationship id of `sect`'s `local` reference of type `want`; a
/// type the section omits comes from the nearest earlier section that
/// names it (ECMA-376 17.10.5: priority 2b479f55f8's third section names
/// only a footer and still shows section 1's default header in Word).
/// Only the section's own children count: a `w:sectPrChange` record's
/// references are history (PR #247 review).
pub(super) fn inherited_ref_id(dom: &Dom, sect: NodeId, local: &str, want: &str) -> Option<String> {
    let own = |s: NodeId| {
        dom.elements(s, Some(&W::name(local)))
            .into_iter()
            .find(|&node| dom.attribute(node, &W::type_()).unwrap_or("default") == want)
            .and_then(|node| attr_any(dom, node, "id"))
            .map(str::to_string)
    };
    if let Some(id) = own(sect) {
        return Some(id);
    }
    let mut body = sect;
    while !dom.name_is(body, &W::body()) {
        body = dom.parent(body)?;
    }
    let sects = live_sect_prs(dom, body);
    let at = sects.iter().position(|&s| s == sect)?;
    sects[..at].iter().rev().find_map(|&s| own(s))
}

fn chrome_present(part: &ChromePart) -> bool {
    !part.runs.is_empty()
        || part.watermark.is_some()
        || !part.images.is_empty()
        || !part.tables.is_empty()
}

#[derive(Clone)]
pub(super) struct ChromeTable {
    /// The laid-out table (0005052e's Doküman/Revizyon header row). Its
    /// cells paint themselves; the border fields below are then unused.
    pub(super) block: Option<std::rc::Rc<Block>>,
    /// No text paragraph precedes it in the part.
    pub(super) before_text: bool,
    /// The empty paragraphs above it: each a line of its mark (0107980d's
    /// header opens with one before its logo table).
    pub(super) lead: Vec<TextRun>,
    /// How many of those paragraphs are also the part's own leading lines
    /// (`collect_hf_runs` keeps only bare lines).
    pub(super) lead_lines: usize,
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) color: [f32; 3],
    pub(super) width: f32,
    pub(super) top: bool,
    pub(super) bottom: bool,
    pub(super) left: bool,
    pub(super) right: bool,
}

pub(super) fn hf_table_width_pt(dom: &Dom, table: NodeId) -> f32 {
    match table_pref_width(dom, table) {
        TblWidth::Dxa(w) if w.is_finite() && w > 1.0 => w,
        _ => {
            let mut w = 0.0;
            if let Some(grid) = direct_named(dom, table, "tblGrid") {
                for col in dom.elements(grid, Some(&W::grid_col())) {
                    // A non-finite column is no width, same as an unparsable one.
                    w += attr_any(dom, col, "w")
                        .and_then(parse_finite_f32)
                        .map(twip)
                        .unwrap_or(0.0);
                }
            }
            if w.is_finite() && w > 1.0 { w } else { 144.0 }
        }
    }
}

/// Top-level header/footer tables, laid out like body tables. Word draws
/// 0005052e's "Doküman | Revizyon | Sayfa No" as one row; stacking the
/// cell paragraphs as header lines put them in a column and pushed the
/// body 60pt down.
pub(super) fn collect_hf_tables(
    pkg: &PartFs,
    path: &str,
    dom: &Dom,
    root: NodeId,
    sheet: &StyleSheet,
) -> Vec<ChromeTable> {
    let first_text = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .filter(|p| hf_node_is_top_level(dom, root, *p))
        .find(|p| {
            // The running band excludes frame/shape-owned text. Inspect the
            // same collected runs as collect_hf_runs rather than descending
            // blindly into a drawing's nested text box.
            if hf_para_is_shape_text(dom, *p) || hf_text_frame_key(dom, *p, sheet).is_some() {
                return false;
            }
            let spans_margins = dom
                .descendants(*p, Some(&W::name("ptab")))
                .into_iter()
                .any(|n| {
                    attr_any(dom, n, "relativeTo") == Some("margin")
                        && matches!(attr_any(dom, n, "alignment"), Some("right" | "center"))
                });
            let frame_align = dom
                .element(*p, &W::p_pr())
                .and_then(|ppr| first_named(dom, ppr, "framePr"))
                .and_then(|fp| attr_any(dom, fp, "xAlign"))
                .filter(|_| !spans_margins);
            if matches!(frame_align, Some("right" | "center")) {
                return false;
            }
            let (_, base) = para_base(dom, *p, sheet, None);
            let mut runs = Vec::new();
            collect_hf_rec(dom, *p, &base, sheet, &mut FieldScan::default(), &mut runs);
            runs.iter().any(|run| !run.text.trim().is_empty())
        })
        .map(|p| p.0);
    let mut out = Vec::new();
    let mut prev = 0;
    for tbl in dom.descendants(root, Some(&W::tbl())) {
        if !hf_node_is_top_level(dom, root, tbl) {
            continue;
        }
        let borders = table_pr(dom, tbl).and_then(|pr| parse_tbl_borders(dom, pr));
        let block = table_block(
            dom,
            tbl,
            sheet,
            &mut Numbering::default(),
            &mut AuthorColors::default(),
            &HashMap::new(),
            Some((pkg, path)),
        );
        let lead = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .filter(|p| p.0 > prev && p.0 < tbl.0 && hf_node_is_top_level(dom, root, *p))
            .map(|p| {
                let (pstyle, mut mark) = para_base(dom, p, sheet, None);
                if let Some(rpr) = dom
                    .element(p, &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                {
                    apply_mark_rpr(dom, rpr, &mut mark, sheet);
                }
                let mut run = TextRun::new(HF_LINE_BREAK, mark);
                run.hf_para = Some(std::rc::Rc::new(pstyle));
                run
            })
            .collect();
        let lead_lines = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .filter(|p| p.0 > prev && p.0 < tbl.0 && hf_node_is_top_level(dom, root, *p))
            .filter(|p| hf_para_is_bare_line(dom, root, *p))
            // Each w:br is one more line, as `collect_hf_runs` stacks them
            // (PR #247 review).
            .map(|p| {
                1 + dom
                    .descendants(p, Some(&W::name("br")))
                    .into_iter()
                    .filter(|br| attr_any(dom, *br, "type").is_none_or(|t| t == "textWrapping"))
                    .count()
            })
            .sum();
        prev = tbl.0;
        out.push(ChromeTable {
            block: Some(std::rc::Rc::new(block)),
            lead,
            lead_lines,
            before_text: first_text.is_none_or(|t| tbl.0 < t),
            w: hf_table_width_pt(dom, tbl),
            h: 0.0,
            color: borders.map_or([0.0; 3], |b| b.color),
            width: borders.map_or(0.0, |b| b.width),
            top: false,
            bottom: false,
            left: false,
            right: false,
        });
    }
    out
}

/// A direct child of the part (or of an sdt in it), not inside a table.
pub(super) fn hf_node_is_top_level(dom: &Dom, root: NodeId, node: NodeId) -> bool {
    let mut cur = dom.parent(node);
    while let Some(id) = cur {
        if id == root {
            return true;
        }
        if !(dom.name_is(id, &W::sdt()) || dom.name_is(id, &W::sdt_content())) {
            return false;
        }
        cur = dom.parent(id);
    }
    false
}

pub(super) fn pick_section_hf(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    local: &str,
    sheet: &StyleSheet,
) -> PickedHf {
    let default = sect_ref_chrome_of(pkg, main, dom, sect, local, sheet, "default");
    let first = sect_ref_chrome_of(pkg, main, dom, sect, local, sheet, "first");
    let even_raw = sect_ref_chrome_of(pkg, main, dom, sect, local, sheet, "even");
    // An explicit type="even" reference counts even when its part is
    // blank: even pages then show no header, not the default one.
    let even_explicit = inherited_ref_id(dom, sect, local, "even").is_some();
    let even = (settings_even_and_odd_headers(pkg) && (even_explicit || chrome_present(&even_raw)))
        .then_some(even_raw);
    // The first-page part shows only under titlePg: without it Word never
    // uses it, even when the section has no default part (English corpus
    // 54b21048's first-only header is on none of Word's 30 pages).
    let odd = default.clone();
    // titlePg gives page 1 the first-page part, blank when the section has
    // none: 0016811c hides its page number with an explicit blank one, and
    // 000105a2 (default reference only) has no header on page 1 in Word.
    if sect_title_pg(dom, sect) {
        let mut first = first;
        if first.watermark.is_none() {
            first.watermark = default.watermark.clone();
        }
        PickedHf {
            start: first,
            rest: Some(default),
            even,
            odd,
        }
    } else {
        PickedHf {
            start: default,
            rest: None,
            even,
            odd,
        }
    }
}

fn sect_ref_chrome_of(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    local: &str,
    sheet: &StyleSheet,
    want: &str,
) -> ChromePart {
    let rid = inherited_ref_id(dom, sect, local, want);
    let page = apply_sect_pr(dom, sect, &sheet.defaults.page);
    let text_w = page.width - page.margin_l - page.margin_r;
    let background = local == "headerReference" && page_background(dom, sect);
    let Some(rid) = rid else {
        // Word still lays out a header under a page background: an empty
        // Header paragraph and the background's Normal one (f7143477's
        // body starts at 63.6pt, below its 36pt margin). Word's latent
        // Header is single-spaced with nothing after.
        if !(background && want == "default") {
            return empty_chrome();
        }
        let header = if sheet.by_id.contains_key("Header") {
            r#"<w:pStyle w:val="Header"/>"#
        } else {
            r#"<w:spacing w:after="0" w:line="240" w:lineRule="auto"/>"#
        };
        let xml = format!(
            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:pPr>{header}</w:pPr></w:p><w:p/></w:hdr>"#
        );
        return chrome_part_xml(pkg, main, &xml, local, sheet, text_w);
    };
    load_chrome_part(pkg, main, &rid, local, sheet, text_w, background)
}

/// `w:document/w:background`: Word anchors the page colour's shape in the
/// header story, which then ends with one more empty Normal paragraph.
fn page_background(dom: &Dom, node: NodeId) -> bool {
    dom.ancestors(node, Some(&W::document()))
        .first()
        .is_some_and(|doc| dom.element(*doc, &W::background()).is_some())
}

fn load_chrome_part(
    pkg: &PartFs,
    main: &str,
    rid: &str,
    local: &str,
    sheet: &StyleSheet,
    text_w: f32,
    background: bool,
) -> ChromePart {
    let Some(path) = rel_target_path(pkg, main, rid) else {
        return empty_chrome();
    };
    let Some(bytes) = pkg.part_bytes(&path) else {
        return empty_chrome();
    };
    let mut xml = String::from_utf8_lossy(bytes).into_owned();
    // The page background's anchor paragraph: af0035cc's one-paragraph
    // header (Normal, after=10 at 1.15) runs two lines deep in Word and
    // pushes the body 14.8pt below its 72pt margin.
    if background && let Some(end) = xml.rfind("</w:hdr>") {
        xml.insert_str(end, "<w:p/>");
    }
    chrome_part_xml(pkg, &path, &xml, local, sheet, text_w)
}

/// A header or footer part laid out from its xml; `path` resolves its
/// relationships (pictures, text boxes).
pub(super) fn chrome_part_xml(
    pkg: &PartFs,
    path: &str,
    xml: &str,
    local: &str,
    sheet: &StyleSheet,
    text_w: f32,
) -> ChromePart {
    let mut part_dom = Dom::new();
    let doc = part_dom.parse_xdocument(xml);
    let Some(root) = part_dom.root(doc) else {
        return empty_chrome();
    };
    let runs = collect_hf_runs(&part_dom, root, sheet, text_w, Some((pkg, path)));
    let spliced: HashSet<u32> = runs
        .iter()
        .filter_map(|r| r.pic.as_ref().map(|p| p.src))
        .collect();
    let mut images = Vec::new();
    let mut boxes = Vec::new();
    let watermark = parse_header_watermark(&part_dom, root);
    let mut seen_text = false;
    let mut para_no = 0_u32;
    // The part's last top-level paragraph (not in a table or a text box).
    let last_para = part_dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .rfind(|p| {
            !hf_para_in_table(&part_dom, root, *p)
                && part_dom.ancestors(*p, Some(&W::txbx_content())).is_empty()
        });
    // Estimated top of each top-level paragraph below the part's top: one
    // line of its largest size (x1.15, its multiple) and max(after, before).
    let mut para_top = 0.0_f32;
    let mut last_after = 0.0_f32;
    let frame_ctx = WalkCtx {
        pkg,
        main: path,
        sheet,
        sects: &[],
        authors: RefCell::new(AuthorColors::default()),
        comments: HashMap::new(),
        open_comments: RefCell::new(Vec::new()),
    };
    let mut frame: Option<(String, Vec<NodeId>)> = None;
    for para in part_dom.descendants(root, Some(&W::p())) {
        if hf_para_is_shape_text(&part_dom, para) {
            continue;
        }
        let top_level = !hf_para_in_table(&part_dom, root, para);
        // Consecutive paragraphs of one page-anchored text frame float as
        // one box, like the body's (walk_container).
        let key = top_level
            .then(|| hf_text_frame_key(&part_dom, para, sheet))
            .flatten();
        if let Some((k, _)) = frame.as_ref()
            && key.as_deref() != Some(k.as_str())
        {
            let (_, paras) = frame.take().expect("a frame");
            boxes.extend(frame_box(
                &frame_ctx,
                &part_dom,
                &paras,
                &mut Numbering::default(),
            ));
        }
        if let Some(k) = key {
            frame.get_or_insert_with(|| (k, Vec::new())).1.push(para);
            continue;
        }
        let (pstyle, prun) = para_base(&part_dom, para, sheet, None);
        // Anchored text boxes of a top-level paragraph float over the page
        // like the body's (a watermark keeps its own path).
        // An inline one stands on the paragraph's line like a picture.
        let mut flow_boxes = Vec::new();
        if watermark.is_none() && top_level {
            let (flow, anchored): (Vec<_>, Vec<_>) = collect_textboxes_styled(
                Some((pkg, path)),
                &part_dom,
                para,
                &prun,
                &sheet.theme,
                Some(sheet),
                None,
            )
            .into_iter()
            .partition(|b| matches!(b.slot, ImageSlot::Flow));
            // Only DrawingML inline boxes: an inline VML text box beside a
            // logo takes no slot (0017dd5f's boxed page number), and must
            // not cost a DrawingML box beside it its pairing (PR #272).
            let flow: Vec<_> = flow.into_iter().filter(|b| !b.vml).collect();
            boxes.extend(anchored.into_iter().map(|mut b| {
                b.chrome_para_top = para_top;
                b
            }));
            // Each stands in its drawing's wp:effectExtent (dcda3ae's
            // note bar 0.75pt under its line top). A stroked or filled
            // shape without text is a box too (e0fe3a82eb's connector).
            let effects: Vec<([f32; 4], RunStyle)> = part_dom
                .descendants(para, Some(&WP::inline()))
                .into_iter()
                .filter(|inl| {
                    (!descendants_local(&part_dom, *inl, "txbx").is_empty()
                        || (!descendants_local(&part_dom, *inl, "wsp").is_empty()
                            && (shape_line_color(&part_dom, *inl, &sheet.theme).is_some()
                                || shape_fill_color(&part_dom, *inl, &sheet.theme).is_some())))
                        && !part_dom.ancestors(*inl, None).iter().any(|a| {
                            local_name_is(&part_dom, *a, "Fallback")
                                || local_name_is(&part_dom, *a, "txbxContent")
                        })
                })
                .map(|inl| {
                    let mut run = prun.clone();
                    if let Some(rpr) = part_dom
                        .ancestors(inl, Some(&W::r()))
                        .first()
                        .and_then(|r| part_dom.element(*r, &W::r_pr()))
                    {
                        apply_rpr(&part_dom, rpr, &mut run, &sheet.theme);
                    }
                    (inline_effect_pt(&part_dom, inl), run)
                })
                .collect();
            if effects.len() == flow.len() {
                flow_boxes = flow
                    .into_iter()
                    .zip(effects)
                    .map(|(b, (effect, run))| {
                        let flat = b.h < 0.5;
                        let mut img = text_box_image(b, effect);
                        if flat {
                            img.chrome_flat_run = Some(run);
                        }
                        img
                    })
                    .collect();
            }
        }
        let this_top = para_top;
        let last_text_after = last_after;
        if top_level {
            let size = part_dom
                .descendants(para, Some(&W::name("sz")))
                .into_iter()
                .filter(|n| part_dom.ancestors(*n, Some(&W::txbx_content())).is_empty())
                .filter_map(|n| attr_any(&part_dom, n, "val").and_then(parse_finite_f32))
                .map(|half| half / 2.0)
                .fold(prun.size, f32::max);
            let line = pstyle
                .line_exact
                .or(pstyle.line_at_least.map(|l| l.max(size * 1.15)))
                .unwrap_or(size * 1.15 * pstyle.line_mult.max(1.0));
            // A picture-only paragraph's line is its picture plus the
            // mark's descent (e0fe3a82eb's 75.75pt logo over its title).
            let pic = hf_inline_pic_h(&part_dom, para);
            let line = if pic > 0.0 && para_shown_text(&part_dom, para).trim().is_empty() {
                let face = fonts().get(fonts().resolve(&prun.family, prun.bold, prun.italic));
                line.max(pic + face.line_descent_pt(size))
            } else {
                line
            };
            // A paragraph's own space before sits below its anchor top,
            // like the body's `para_top` (010ec7df's box 5pt low otherwise).
            para_top += (pstyle.before - last_after).max(0.0) + line + pstyle.after;
            last_after = pstyle.after;
        }
        // Under the paragraph's text line, above its space after.
        let trail_top = para_top - pstyle.after;
        let jc = pstyle.align;
        let leading = (pstyle.line_exact.is_none()
            && pstyle.line_at_least.is_none()
            && pstyle.line_mult > 1.0)
            .then(|| {
                let mut mark = prun.clone();
                if let Some(rpr) = part_dom
                    .element(para, &W::p_pr())
                    .and_then(|ppr| part_dom.element(ppr, &W::r_pr()))
                {
                    apply_rpr(&part_dom, rpr, &mut mark, &sheet.theme);
                }
                (pstyle.line_mult - 1.0, mark)
            });
        let lead = !seen_text;
        let tab_after = !part_dom
            .descendants(para, Some(&W::name("tab")))
            .into_iter()
            .filter(|t| part_dom.ancestors(*t, Some(&W::name("tabs"))).is_empty())
            .collect::<Vec<_>>()
            .is_empty();
        let mark_style = || {
            let mut mark = prun.clone();
            if let Some(rpr) = part_dom
                .element(para, &W::p_pr())
                .and_then(|ppr| part_dom.element(ppr, &W::r_pr()))
            {
                apply_rpr(&part_dom, rpr, &mut mark, &sheet.theme);
            }
            mark
        };
        // Only a picture-only paragraph stands as its own line; a picture
        // sharing a line with text (000e002d's logo + tabbed title) stays
        // with the text.
        // An inline VML picture (no position:absolute) flows too (redlines
        // vs 001cc92b: A's deleted 45.8pt VML banner is the whole header).
        let vml_inline = part_dom
            .descendants(para, Some(&W::pict()))
            .into_iter()
            .any(|pict| {
                part_dom
                    .ancestors(pict, Some(&W::txbx_content()))
                    .is_empty()
                    && descendants_local(&part_dom, pict, "shape")
                        .iter()
                        .any(|sh| {
                            !descendants_local(&part_dom, *sh, "imagedata").is_empty()
                                && !attr_any(&part_dom, *sh, "style")
                                    .unwrap_or("")
                                    .replace(' ', "")
                                    .contains("position:absolute")
                        })
            });
        // Deleted text is text here: the redline paints it (0800162a66 vs
        // 000e3e7b's deleted "Akreditācijas ekspertu" under the banner).
        let no_text = para_shown_text(&part_dom, para).trim().is_empty();
        let trails = hf_pic_trails_line(&part_dom, para, sheet, text_w);
        let flow = (no_text || hf_pic_owns_line(&part_dom, para, sheet, text_w))
            && (vml_inline
                || part_dom
                    .descendants(para, Some(&WP::inline()))
                    .into_iter()
                    .any(|inl| part_dom.ancestors(inl, Some(&W::txbx_content())).is_empty()));
        // A picture paragraph with no text or bare line after it in the
        // part stands under the whole text band.
        let band_para = |p: NodeId| {
            !hf_para_is_shape_text(&part_dom, p)
                && !hf_para_in_table(&part_dom, root, p)
                && (!para_shown_text(&part_dom, p).trim().is_empty()
                    || hf_para_is_bare_line(&part_dom, root, p))
        };
        let under_text = !lead
            && no_text
            && top_level
            && local.starts_with("header")
            && !part_dom
                .descendants(root, Some(&W::p()))
                .into_iter()
                .any(|p| p.0 > para.0 && band_para(p));
        if !hf_para_in_table(&part_dom, root, para) && !no_text {
            seen_text = true;
        }
        // A top-level chrome table lays out the pictures its cells hold,
        // nested tables' included (0005052e / 0107980d painted the logo
        // twice); page-anchored ones stay here.
        let under_table = !hf_para_in_table(&part_dom, root, para)
            && part_dom
                .descendants(root, Some(&W::tbl()))
                .iter()
                .any(|t| t.0 < para.0 && hf_node_is_top_level(&part_dom, root, *t));
        let table_owned = hf_para_in_table(&part_dom, root, para)
            && part_dom
                .ancestors(para, Some(&W::tbl()))
                .iter()
                .any(|t| hf_node_is_top_level(&part_dom, root, *t));
        para_no += 1;
        // A page-anchored frame (w:framePr hAnchor/vAnchor="page") holds its
        // pictures at the frame's page position, out of the band (redlines
        // vs 000e3e7b whose A carries one: Word paints the banner at the
        // frame's 13.55pt, 27.8pt and starts the body at the top margin).
        let frame_at = part_dom
            .element(para, &W::p_pr())
            .and_then(|ppr| first_named(&part_dom, ppr, "framePr"))
            .filter(|fp| {
                attr_any(&part_dom, *fp, "hAnchor") == Some("page")
                    && attr_any(&part_dom, *fp, "vAnchor") == Some("page")
            })
            .map(|fp| {
                let at = |name: &str| {
                    attr_any(&part_dom, fp, name)
                        .and_then(parse_finite_f32)
                        .map_or(0.0, |tw| tw / 20.0)
                };
                (at("x"), at("y"), at("w"))
            });
        // Flow pictures fill rows; one past the measure opens the next row,
        // under the wrapped tab line when the first row's tab wrapped.
        let (mut cursor, mut row_h, mut drop) = (0.0_f32, 0.0_f32, 0.0_f32);
        let mut tab_wrapped = false;
        images.extend(
            collect_images(pkg, path, &part_dom, para, &|_| None)
                .into_iter()
                .chain(flow_boxes)
                .filter(|img| img.src == 0 || !spliced.contains(&img.src))
                .filter(|img| !(table_owned && cell_holds_image(img)))
                .map(|mut img| {
                    img.chrome_align = jc;
                    // A picture wrapped under its paragraph's text stands on
                    // the part's lines below it, not above its text.
                    img.chrome_lead = lead && !trails;
                    if trails {
                        img.chrome_above = trail_top;
                        // A flat shape stands on its line's baseline, its
                        // stroke centred half a point under it (e0fe3a82eb:
                        // Word's 2pt teal line at 140.9, 15.4pt under the
                        // title's baseline; the line's own baseline is
                        // 140.4).
                        if let Some(run) = &img.chrome_flat_run {
                            let face =
                                fonts().get(fonts().resolve(&run.family, run.bold, run.italic));
                            img.chrome_above += face.single_line_pt(run.size)
                                - face.line_descent_pt(run.size)
                                + 0.5
                                - img.h.max(1.0) * 0.5;
                        }
                    } else if under_text {
                        // A picture paragraph after the part's text stands on
                        // its own line under it, its space before past the
                        // last text paragraph's after (dcda3ae's note bar
                        // under its header title).
                        img.chrome_under_text = true;
                        img.chrome_above = (pstyle.before - last_text_after).max(0.0);
                    }
                    img.chrome_under_table = under_table;
                    img.chrome_para = para_no;
                    // A footer picture placed from its paragraph sits under
                    // the part's paragraphs above it (432bf1c280's logos on
                    // "January 2025", the footer's second line).
                    if top_level
                        && local.starts_with("footer")
                        && let ImageSlot::Float {
                            para_y: Some(py), ..
                        } = &mut img.slot
                    {
                        *py += this_top;
                    }
                    if let Some((fx, fy, fw)) = frame_at
                        && matches!(img.slot, ImageSlot::Flow)
                    {
                        // The paragraph's jc places it across the frame's
                        // width (0015dee2's centred logo in a 488pt frame).
                        let room = (fw - img.w).max(0.0);
                        let fx = fx
                            + match jc {
                                Align::Center => room / 2.0,
                                Align::Right => room,
                                Align::Left | Align::Justify => 0.0,
                            };
                        img.slot = ImageSlot::Float {
                            align: Align::Left,
                            page_x: Some(fx),
                            page_y: Some(fy),
                            col_x: None,
                            col_in_column: false,
                            para_y: None,
                            pct_x: None,
                            pct_y: None,
                            pct_w: None,
                            pct_h: None,
                            v_align: Align::Left,
                            wrap_square: false,
                            wrap_top_bottom: false,
                            wrap_polygon: false,
                            poly_bottom: 1.0,
                            dist_l: 0.0,
                            dist_r: 0.0,
                            dist_t: 0.0,
                            dist_b: 0.0,
                            effect_b: 0.0,
                            h_rel: RelFrame::Page,
                            v_rel: RelFrame::Page,
                            v_off: None,
                        };
                    }
                    // Text under the picture closes the paragraph and
                    // takes its space after (in the part's text height).
                    if last_para == Some(para) && no_text {
                        img.chrome_after = pstyle.after;
                    }
                    img.chrome_flow = flow && matches!(img.slot, ImageSlot::Flow);
                    if img.chrome_flow {
                        img.chrome_leading.clone_from(&leading);
                        if !no_text {
                            img.chrome_text_under = Some(mark_style());
                        }
                        let first_row = row_h <= 0.0 || cursor + img.w <= text_w + 0.5;
                        if cursor > 0.0 && cursor + img.w > text_w + 0.5 {
                            drop += row_h;
                            cursor = 0.0;
                            row_h = 0.0;
                        }
                        img.chrome_drop = drop;
                        if drop > 0.0 {
                            img.chrome_drop_tab = Some((mark_style(), tab_wrapped));
                        }
                        // The part's first paragraph keeps its explicit space
                        // before above its pictures, as it does above text
                        // (52a8b97a's Caption logo: live Word 6pt lower).
                        if para_no == 1 && lead && !pstyle.before_auto {
                            img.chrome_drop += pstyle.before;
                        }
                        // Text after the tab shares its line (0800162a66 vs
                        // 000e3e7b: the deleted "Akreditācijas ekspertu").
                        if first_row && drop <= 0.0 && tab_after && img.w >= text_w - 0.5 && no_text
                        {
                            img.chrome_tab_line = Some(mark_style());
                            tab_wrapped = true;
                        }
                        cursor += img.w;
                        row_h = row_h.max(img.h);
                    }
                    img
                }),
        );
    }
    if let Some((_, paras)) = frame.take() {
        boxes.extend(frame_box(
            &frame_ctx,
            &part_dom,
            &paras,
            &mut Numbering::default(),
        ));
    }
    if local.starts_with("header") {
        drop_flow_under_square_floats(&mut images, text_w);
    }
    let align = first_para_align(&part_dom, root);
    let edge = if local.starts_with("header") {
        "bottom"
    } else {
        "top"
    };
    ChromePart {
        runs,
        border: first_para_border(&part_dom, root, edge),
        align,
        watermark,
        images,
        tables: collect_hf_tables(pkg, path, &part_dom, root, sheet),
        boxes: std::rc::Rc::new(boxes),
    }
}

/// A header's flow picture wider than the room beside a square-wrapped
/// picture of its paragraph starts under that picture, its wrap distance
/// and effect extent included (0800162a66 vs 000e3e7b: Word sets the
/// 441.75pt banner under the 117.75pt logo, 315pt of room beside it).
fn drop_flow_under_square_floats(images: &mut [LaidImage], text_w: f32) {
    let floats: Vec<(u32, f32, f32, f32)> = images
        .iter()
        .filter(|img| !img.behind)
        .filter_map(|img| match img.slot {
            ImageSlot::Float {
                wrap_square: true,
                page_y: None,
                para_y: Some(top),
                dist_l,
                dist_r,
                dist_b,
                ..
            } => Some((
                img.chrome_para,
                top,
                top + img.h + dist_b,
                img.w + dist_l + dist_r,
            )),
            _ => None,
        })
        .collect();
    for img in images.iter_mut().filter(|img| img.chrome_flow) {
        for &(para, top, bottom, taken) in &floats {
            if para == img.chrome_para
                && img.chrome_drop >= top - 0.5
                && img.chrome_drop < bottom
                && img.w > text_w - taken + 0.5
            {
                img.chrome_drop = bottom;
            }
        }
    }
}

/// A paragraph's text as a redline shows it, deleted runs included (not
/// its text boxes').
pub(super) fn para_shown_text(dom: &Dom, para: NodeId) -> String {
    let mut out = para_own_text(dom, para);
    for t in dom.descendants(para, Some(&W::del_text())) {
        if dom.ancestors(t, Some(&W::txbx_content())).is_empty() {
            out.push_str(&element_text(dom, t));
        }
    }
    out
}

/// The names of the main document's other story parts (headers, footers,
/// footnotes, endnotes, comments), found by relationship type.
fn story_part_names(pkg: &PartFs, main: &str) -> Vec<String> {
    let Some(rels) = pkg.read_rels_for(main) else {
        return Vec::new();
    };
    rels.items
        .iter()
        .filter(|r| {
            r.target_mode.as_deref() != Some("External")
                && ["/header", "/footer", "/footnotes", "/endnotes", "/comments"]
                    .iter()
                    .any(|k| r.rel_type.ends_with(k))
        })
        .map(|r| pkg.resolve_rel_target(main, &r.target))
        .collect()
}

/// The main document's other story parts' xml ([`story_part_names`]).
pub(super) fn story_parts_xml(pkg: &PartFs, main: &str) -> Vec<String> {
    story_part_names(pkg, main)
        .iter()
        .filter_map(|part| pkg.part_string(part))
        .collect()
}

/// `w16se`, Word's emoji symbols (`w16se:symEx`, a font and a character).
const W16SE: &str = "http://schemas.microsoft.com/office/word/2015/wordml/symex";

/// A namespace whose Choice the renderer draws: Word's
/// ([`crate::markup_simplifier::word_understands_namespace`]) but `w16se`.
/// The renderer draws no `w16se:symEx`, so it takes the Fallback, which
/// holds the same character as text in the same font (#1876: `a1cfa10c32`'s
/// emoji).
pub(super) fn renderer_understands_namespace(uri: &str) -> bool {
    uri != W16SE && crate::markup_simplifier::word_understands_namespace(uri)
}

/// Put in place of each `mc:AlternateContent` that holds no drawing, in the
/// main part and its story parts, the branch the renderer draws: the first
/// `mc:Choice` whose `Requires` it understands
/// ([`renderer_understands_namespace`]), else the `mc:Fallback` (#1876). The walks that follow know nothing of the wrapper: they drew
/// both branches of a run-level one and nothing of a block-level one.
/// Drawing wrappers stay for `drawing::alternate_choice`. A part is rewritten only
/// when a wrapper went.
pub(super) fn resolve_text_alternates(pkg: &mut PartFs, main: &str) {
    let wrapper = MC::alternate_content();
    // Sections often share a header or footer part: each part once.
    let mut seen = std::collections::HashSet::new();
    let parts: Vec<String> = std::iter::once(main.to_string())
        .chain(story_part_names(pkg, main))
        .filter(|part| seen.insert(part.clone()))
        .collect();
    let wrappers = |dom: &Dom, root: NodeId| {
        let mut count = 0usize;
        dom.for_each_descendant_element(root, Some(&wrapper), |_| count += 1);
        count
    };
    for part in parts {
        let Some(xml) = pkg.part_string(&part) else {
            continue;
        };
        if !xml.contains("AlternateContent") {
            continue;
        }
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        let before = wrappers(&dom, root);
        crate::markup_simplifier::resolve_alternate_content_with(
            &mut dom,
            root,
            renderer_understands_namespace,
        );
        if wrappers(&dom, root) != before {
            pkg.set_part(&part, dom.serialize_document(doc).into_bytes());
        }
    }
}

pub(super) fn rel_target_path(pkg: &PartFs, source: &str, rid: &str) -> Option<String> {
    let rels = pkg.read_rels_for(source)?;
    let rel = rels.items.iter().find(|item| item.id == rid)?;
    Some(pkg.resolve_rel_target(source, &rel.target))
}

/// A main-document part found the OPC way, by relationship type
/// (`…/relationships/<kind>`, transitional or strict), falling back to
/// Word's conventional name. Other producers may name these parts freely.
pub(crate) fn main_rel_part(pkg: &PartFs, kind: &str, fallback: &str) -> String {
    let suffix = format!("/{kind}");
    pkg.main_document_part()
        .and_then(|main| {
            let rel = pkg.read_rels_for(&main)?.items.iter().find(|r| {
                r.rel_type.ends_with(&suffix) && r.target_mode.as_deref() != Some("External")
            })?;
            Some(pkg.resolve_rel_target(&main, &rel.target))
        })
        .unwrap_or_else(|| fallback.to_string())
}

pub(super) fn settings_part(pkg: &PartFs) -> String {
    main_rel_part(pkg, "settings", "word/settings.xml")
}

pub(super) fn parse_header_watermark(dom: &Dom, root: NodeId) -> Option<Watermark> {
    let mut marked = false;
    for gal in descendants_local(dom, root, "docPartGallery") {
        if attr_any(dom, gal, "val")
            .unwrap_or("")
            .contains("Watermark")
        {
            marked = true;
            break;
        }
    }
    if !marked {
        for pr in descendants_local(dom, root, "docPr") {
            if attr_any(dom, pr, "name")
                .unwrap_or("")
                .contains("WaterMark")
            {
                marked = true;
                break;
            }
        }
    }
    if !marked {
        return None;
    }
    let mut text = String::new();
    let mut size = 36.0;
    let mut color = [0.7529, 0.7529, 0.7529];
    for tnode in descendants_local(dom, root, "t") {
        let s = element_text(dom, tnode);
        if !s.trim().is_empty() {
            text = s;
            if let Some(run) = dom.parent(tnode).and_then(|r| {
                if dom.name_is(r, &W::r()) {
                    Some(r)
                } else {
                    dom.parent(r).filter(|p| dom.name_is(*p, &W::r()))
                }
            }) && let Some(rpr) = dom.element(run, &W::r_pr())
            {
                if let Some(sz) = first_named(dom, rpr, "sz")
                    .and_then(|n| attr_any(dom, n, "val"))
                    .and_then(parse_finite_f32)
                {
                    size = sz / 2.0;
                }
                if let Some(rgb) = first_named(dom, rpr, "color")
                    .and_then(|n| attr_any(dom, n, "val"))
                    .and_then(parse_hex_color)
                {
                    color = rgb;
                }
            }
            break;
        }
    }
    if text.trim().is_empty() {
        for shape in descendants_local(dom, root, "textpath") {
            if let Some(s) = attr_any(dom, shape, "string")
                && !s.trim().is_empty()
            {
                text = s.to_string();
                break;
            }
        }
    }
    if text.trim().is_empty() {
        return None;
    }
    let mut rotate_deg = 315.0;
    for shape in descendants_local(dom, root, "shape") {
        if let Some(style) = attr_any(dom, shape, "style")
            && let Some(rel) = style.find("rotation:")
        {
            let rest = &style[rel + 9..];
            let num: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if let Some(deg) = parse_finite_f32(&num) {
                rotate_deg = deg;
            }
        }
    }
    for xfrm in descendants_local(dom, root, "xfrm") {
        if let Some(rot) = attr_any(dom, xfrm, "rot")
            && let Some(emu) = parse_finite_f32(rot)
        {
            rotate_deg = emu / 60_000.0;
        }
    }
    Some(Watermark {
        text,
        size,
        color,
        rotate_deg,
    })
}

fn first_para_align(dom: &Dom, root: NodeId) -> Align {
    let Some(para) = dom.descendants(root, Some(&W::p())).into_iter().next() else {
        return Align::Left;
    };
    let Some(ppr) = dom.element(para, &W::p_pr()) else {
        return Align::Left;
    };
    let mut style = ParaStyle {
        fmt_rev: false,
        mark_run: None,
        hrule: None,
        align: Align::Left,
        after: 0.0,
        before: 0.0,
        before_auto: false,
        after_auto: false,
        before_spec: 0.0,
        after_spec: 0.0,
        bidi: false,
        widow_control: true,
        snap_to_grid: true,
        cell_grid: 0.0,
        line_mult: 1.0,
        line_exact: None,
        line_at_least: None,
        indent_left: 0.0,
        indent_right: 0.0,
        indent_first: 0.0,
        contextual: false,
        sum_spacing: false,
        auto_space_de_off: false,
        auto_space_dn_off: false,
        list_num: String::new(),
        style_id: String::new(),
        style_name: String::new(),
        border_top: None,
        border_left: None,
        border_bottom: None,
        border_right: None,
        tab_stops: Vec::new(),
        page_break_before: false,
        keep_next: false,
        keep_lines: false,
        outline_lvl: None,
        chap_num: None,
        fill: None,
        list_jc_right: false,
        empty_toc_field: false,
    };
    apply_ppr(dom, ppr, &mut style);
    style.align
}

/// How far a paragraph box's inner edges stand past its text on the left
/// and right: each side rule's space, plus Word's 1.44pt outset on a
/// four-edge box (`paint_pbdr`).
pub(super) fn pbdr_side_reach(style: &ParaStyle) -> (f32, f32) {
    let four_edge = style.border_top.is_some()
        && style.border_bottom.is_some()
        && style.border_left.is_some()
        && style.border_right.is_some();
    let quartz = if four_edge { 1.44 } else { 0.0 };
    let reach =
        |edge: Option<([f32; 3], f32, f32)>| edge.map_or(0.0, |(_, _, space)| space + quartz);
    (reach(style.border_left), reach(style.border_right))
}

/// Word groups consecutive paragraphs whose borders and indents match
/// into one box.
pub(super) fn same_pbdr(a: &ParaStyle, b: &ParaStyle) -> bool {
    let has = |p: &ParaStyle| {
        p.border_top.is_some()
            || p.border_bottom.is_some()
            || p.border_left.is_some()
            || p.border_right.is_some()
    };
    has(a)
        && a.border_top == b.border_top
        && a.border_bottom == b.border_bottom
        && a.border_left == b.border_left
        && a.border_right == b.border_right
        && a.indent_left == b.indent_left
        && a.indent_right == b.indent_right
}

pub(super) fn pbdr_edge(dom: &Dom, ppr: NodeId, edge: &str) -> Option<([f32; 3], f32, f32)> {
    let pbdr = first_named(dom, ppr, "pBdr")?;
    let el = first_named(dom, pbdr, edge)?;
    // val none/nil is no border (0036eb25's Normal lists all four).
    if matches!(attr_any(dom, el, "val"), Some("none" | "nil")) {
        return None;
    }
    let color = attr_any(dom, el, "color")
        .and_then(parse_hex_color)
        .unwrap_or([0.0, 0.0, 0.0]);
    let width = attr_any(dom, el, "sz")
        .and_then(parse_finite_f32)
        .map(|eighths| {
            let pt = eighths / 8.0;
            // file_146 / sample_iter2 heading pBdr sz=3 is Word 0.24pt
            // (1px @ 300dpi). (sz/8).max(0.4) painted 0.40pt on 45 rules.
            // sz=4 IntenseQuote / table hairlines stay 0.5pt.
            if pt < 0.5 { 0.24 } else { pt }
        })
        .unwrap_or(0.6);
    // Word ST_Pts; omitted space is 0 (ECMA-376 17.3.1.24).
    let space = attr_any(dom, el, "space")
        .and_then(parse_finite_f32)
        .unwrap_or(0.0);
    Some((color, width, space))
}

fn first_para_border(dom: &Dom, root: NodeId, edge: &str) -> Option<([f32; 3], f32)> {
    let para = dom.descendants(root, Some(&W::p())).into_iter().next()?;
    let ppr = dom.element(para, &W::p_pr())?;
    pbdr_edge(dom, ppr, edge).map(|(color, width, _)| (color, width))
}

pub(super) const HF_LINE_BREAK: &str = "\n";

pub(super) fn hf_para_is_shape_text(dom: &Dom, para: NodeId) -> bool {
    // Strict01 watermark Fallback parks "CONFIDENTIAL" in a txbx <w:p>
    // under wp:anchor. descendants(<w:hdr>, p) would collect it as a
    // 36pt header line. Word paints that as a rotated behind-doc mark.
    let mut cur = Some(para);
    while let Some(id) = cur {
        if dom.name_is(id, &W::drawing()) || dom.name_is(id, &W::pict()) {
            return true;
        }
        cur = dom.parent(id);
    }
    false
}
