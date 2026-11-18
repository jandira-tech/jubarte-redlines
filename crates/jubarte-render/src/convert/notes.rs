// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Section boundaries, footnotes, endnotes and per-section header/footer
//! selection.

use super::*;

pub(super) fn next_sect_pr(sects: &[NodeId], current: NodeId) -> Option<NodeId> {
    let mut seen = false;
    for &s in sects {
        if seen {
            return Some(s);
        }
        if s == current {
            seen = true;
        }
    }
    None
}

pub(super) fn is_final_sect(sects: &[NodeId], sect: NodeId) -> bool {
    sects.last() == Some(&sect)
}

pub(super) fn is_cover_pages_sdt(dom: &Dom, sdt: NodeId) -> bool {
    let Some(pr) = dom.element(sdt, &W::sdt_pr()) else {
        return false;
    };
    first_named(dom, pr, "docPartGallery")
        .and_then(|n| dom.attribute(n, &W::val()))
        .is_some_and(|v| v.eq_ignore_ascii_case("Cover Pages"))
}

/// The part `main` relates to by type `…/kind`, then by a type or target
/// merely containing `kind`. The exact type goes first: a target check
/// alone took `commentsExtended.xml` for the comments part when its
/// relationship came first (with_comments_clean/37c6c62345).
pub(super) fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String> {
    if let Some(rels) = pkg.read_rels_for(main) {
        let exact = format!("/{kind}");
        let by_type = rels.items.iter().filter(|i| i.rel_type.ends_with(&exact));
        let loose = rels
            .items
            .iter()
            .filter(|i| i.rel_type.ends_with(kind) || i.target.contains(kind));
        for item in by_type.chain(loose) {
            let path = pkg.resolve_rel_target(main, &item.target);
            if let Some(xml) = pkg.part_string(&path) {
                return Some(xml);
            }
        }
    }
    pkg.part_string(&format!("word/{kind}.xml"))
}

/// Separator / continuation notes are chrome, not note text. Since plan
/// Step 7, `paint_page_footnotes` draws the separator itself: a
/// `FOOTNOTE_SEP_PT` rule from `margin_l` to `margin_l + FOOTNOTE_SEP_W`
/// at `sep_y`, above the reserved note block (mini 619-622's content-
/// driven 144x0.72 rule stays retired).
fn note_is_structural(dom: &Dom, note: NodeId) -> bool {
    matches!(
        attr_any(dom, note, "type"),
        Some("separator" | "continuationSeparator" | "continuationNotice")
    )
}

/// Endnotes collected during the body walk so `endnotePr/@w:pos=sectEnd`
/// can flush at the section boundary (plan.md Step 10 F). `docEnd` (the
/// default when pos is omitted) still emits after the last body block.
pub(super) struct EndnoteBag {
    ndom: Dom,
    pub(super) by_id: HashMap<String, NodeId>,
    /// The `w:type="separator"` note, laid out over each run of notes.
    separator: Option<NodeId>,
    pub(super) pending: Vec<String>,
    seen: HashSet<String>,
}

impl EndnoteBag {
    pub(super) fn load(pkg: &PartFs, main: &str) -> Self {
        let mut bag = Self {
            ndom: Dom::new(),
            by_id: HashMap::new(),
            separator: None,
            pending: Vec::new(),
            seen: HashSet::new(),
        };
        let Some(xml) = part_xml_by_rel_kind(pkg, main, "endnotes") else {
            return bag;
        };
        let doc = bag.ndom.parse_xdocument(&xml);
        let Some(root) = bag.ndom.root(doc) else {
            return bag;
        };
        for note in bag.ndom.descendants(root, Some(&W::endnote())) {
            if attr_any(&bag.ndom, note, "type") == Some("separator") {
                bag.separator = Some(note);
            }
            if note_is_structural(&bag.ndom, note) {
                continue;
            }
            if let Some(id) = attr_any(&bag.ndom, note, "id") {
                bag.by_id.insert(id.to_string(), note);
            }
        }
        bag
    }

    pub(super) fn observe_para(&mut self, dom: &Dom, para: NodeId) {
        for n in dom.descendants(para, Some(&W::name("endnoteReference"))) {
            let Some(id) = attr_any(dom, n, "id") else {
                continue;
            };
            if self.seen.insert(id.to_string()) {
                self.pending.push(id.to_string());
            }
        }
    }

    pub(super) fn flush_if_sect_end(
        &mut self,
        ctx: &WalkCtx<'_>,
        body_dom: &Dom,
        sect: NodeId,
        numbering: &mut Numbering,
        blocks: &mut Vec<Block>,
    ) {
        if endnote_pos_is_sect_end(body_dom, sect) {
            self.emit(ctx, numbering, blocks);
        }
    }

    pub(super) fn emit_remaining(
        &mut self,
        ctx: &WalkCtx<'_>,
        numbering: &mut Numbering,
        blocks: &mut Vec<Block>,
    ) {
        self.emit(ctx, numbering, blocks);
    }

    fn emit(&mut self, ctx: &WalkCtx<'_>, numbering: &mut Numbering, blocks: &mut Vec<Block>) {
        let ids = std::mem::take(&mut self.pending);
        if let Some(sep) = self.separator
            && ids.iter().any(|id| self.by_id.contains_key(id))
        {
            push_endnote_blocks(ctx, &self.ndom, sep, numbering, blocks);
        }
        for id in ids {
            let Some(&note) = self.by_id.get(&id) else {
                continue;
            };
            // Word `docEnd` continues on the last body page when the notes
            // fit (Strict01 p13 is SmartArt + endnote). A hard break here
            // turned that into 14 pages once the diagram reserved flow.
            push_endnote_blocks(ctx, &self.ndom, note, numbering, blocks);
        }
    }
}

fn endnote_pos_is_sect_end(dom: &Dom, sect: NodeId) -> bool {
    first_named(dom, sect, "endnotePr")
        .and_then(|pr| first_named(dom, pr, "pos"))
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v == "sectEnd")
}

pub(super) fn push_endnote_blocks(
    ctx: &WalkCtx<'_>,
    ndom: &Dom,
    note: NodeId,
    numbering: &mut Numbering,
    blocks: &mut Vec<Block>,
) {
    let first = blocks.len();
    for i in 0..ndom.child_count(note) {
        let child = ndom.child_at(note, i);
        if ndom.name_is(child, &W::p()) {
            blocks.push(without_page_marks(paragraph_block(
                ctx, ndom, child, false, numbering,
            )));
        } else if ndom.name_is(child, &W::tbl()) {
            let block = table_block(
                ndom,
                child,
                ctx.sheet,
                numbering,
                &mut ctx.authors.borrow_mut(),
                &ctx.comments,
                None,
            );
            if !block_is_blank(&block) {
                blocks.push(block);
            }
        }
    }
    // The note's own mark carries its id, so it reads as its reference.
    let id = attr_any(ndom, note, "id").unwrap_or_default();
    visit_runs_mut(&mut blocks[first..], |run| {
        if run.endnote_id.as_deref() == Some("") {
            run.endnote_id = Some(id.to_string());
        }
    });
}

/// Endnote numbering from the last section's `w:endnotePr`, else the
/// settings part's: format and start. Word's default is i, ii, iii
/// (9134397db6, cf977017dd).
pub(super) fn endnote_num_format(pkg: &PartFs, dom: &Dom, sects: &[NodeId]) -> (NumFmt, u32) {
    let read = |dom: &Dom, pr: NodeId| {
        let val = |name: &str| first_named(dom, pr, name).and_then(|n| attr_any(dom, n, "val"));
        (
            val("numFmt").map(parse_num_fmt),
            val("numStart").and_then(|v| v.parse::<u32>().ok()),
        )
    };
    let (mut fmt, mut start) = sects
        .last()
        .and_then(|&sect| first_named(dom, sect, "endnotePr"))
        .map_or((None, None), |pr| read(dom, pr));
    if (fmt.is_none() || start.is_none())
        && let Some(settings) = settings_dom(pkg)
    {
        let (sdom, root) = (&settings.0, settings.1);
        if let Some(pr) = sdom
            .descendants(root, Some(&W::name("endnotePr")))
            .into_iter()
            .next()
        {
            let (f, n) = read(sdom, pr);
            fmt = fmt.or(f);
            start = start.or(n);
        }
    }
    (fmt.unwrap_or(NumFmt::LowerRoman), start.unwrap_or(1))
}

/// Labels each endnote mark by its note's place in reference order.
pub(super) fn number_endnote_refs(blocks: &mut [Block], (fmt, start): (NumFmt, u32)) {
    let mut labels: HashMap<String, String> = HashMap::new();
    visit_runs_mut(blocks, |run| {
        let Some(id) = run.endnote_id.as_deref().filter(|id| !id.is_empty()) else {
            return;
        };
        let next = start + labels.len() as u32;
        let label = labels
            .entry(id.to_string())
            .or_insert_with(|| format_num(fmt, next));
        run.text.clone_from(label);
    });
}

/// The footnotes part's notes by id, and the separator note's paragraphs
/// (Word's default one when the part declares none).
pub(super) fn load_footnotes(
    pkg: &PartFs,
    main: &str,
    sheet: &StyleSheet,
) -> (HashMap<String, Vec<FootnotePara>>, Vec<FootnotePara>) {
    let Some(xml) = part_xml_by_rel_kind(pkg, main, "footnotes") else {
        return (HashMap::new(), Vec::new());
    };
    let mut ndom = Dom::new();
    let doc = ndom.parse_xdocument(&xml);
    let Some(root) = ndom.root(doc) else {
        return (HashMap::new(), Vec::new());
    };
    let comments = load_comments(pkg, main);
    let sects: Vec<NodeId> = Vec::new();
    let ctx = WalkCtx {
        pkg,
        main,
        sheet,
        sects: &sects,
        authors: RefCell::new(AuthorColors::default()),
        comments,
        open_comments: RefCell::new(Vec::new()),
    };
    let mut numbering = load_numbering(pkg);
    let mut out = HashMap::new();
    let mut separator = Vec::new();
    for note in ndom.descendants(root, Some(&W::footnote())) {
        let is_separator = attr_any(&ndom, note, "type") == Some("separator");
        if note_is_structural(&ndom, note) && !is_separator {
            continue;
        }
        let id = attr_any(&ndom, note, "id").unwrap_or("").to_string();
        if id.is_empty() && !is_separator {
            continue;
        }
        let mut paras = Vec::new();
        for i in 0..ndom.child_count(note) {
            let child = ndom.child_at(note, i);
            if ndom.name_is(child, &W::p()) {
                let block =
                    without_page_marks(paragraph_block(&ctx, &ndom, child, false, &mut numbering));
                if let Block::Paragraph { runs, style, .. } = block {
                    paras.push(FootnotePara { runs, style });
                }
            }
        }
        if is_separator {
            separator = paras;
        } else if !paras.is_empty() {
            out.insert(id, paras);
        }
    }
    if separator.is_empty() {
        separator.push(default_separator_para(sheet));
    }
    (out, separator)
}

/// Word's separator when the part declares none: one `w:separator` run
/// of the default font on a single line with no spacing (probe s8 ends
/// where the declared Calibri 11 separator does).
fn default_separator_para(sheet: &StyleSheet) -> FootnotePara {
    let mut run = TextRun::new(String::new(), sheet.defaults.run.clone());
    run.strut = true;
    run.note_rule = true;
    let mut style = sheet.defaults.para.clone();
    style.before = 0.0;
    style.after = 0.0;
    style.line_mult = 1.0;
    style.line_exact = None;
    style.line_at_least = None;
    FootnotePara {
        runs: vec![run],
        style,
    }
}

pub(super) fn number_footnote_refs(blocks: &mut [Block]) -> HashMap<String, String> {
    let mut display = HashMap::new();
    let mut n = 1u32;
    visit_runs_mut(blocks, |run| {
        let Some(id) = run.footnote_id.as_deref() else {
            return;
        };
        let label = display.entry(id.to_string()).or_insert_with(|| {
            let s = n.to_string();
            n += 1;
            s
        });
        run.text.clone_from(label);
        run.style.vert = VertAlign::Super;
    });
    display
}

pub(super) fn visit_runs_mut(blocks: &mut [Block], mut f: impl FnMut(&mut TextRun)) {
    visit_runs_mut_inner(blocks, &mut f);
}

fn visit_runs_mut_inner(blocks: &mut [Block], f: &mut impl FnMut(&mut TextRun)) {
    for block in blocks {
        match block {
            Block::Paragraph { runs, .. } => {
                for run in runs {
                    f(run);
                }
            }
            Block::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        for para in &mut cell.paras {
                            for run in &mut para.runs {
                                f(run);
                            }
                        }
                        // Nested tables are shared only by a split row
                        // during layout; before it each one is unique.
                        for nested in &mut cell.nested {
                            if let Some(block) = std::rc::Rc::get_mut(nested) {
                                visit_runs_mut_inner(std::slice::from_mut(block), f);
                            }
                        }
                    }
                }
            }
            Block::PageBreak { .. } | Block::ColumnBreak | Block::SectionCols { .. } => {}
        }
    }
}

pub(super) fn section_chrome(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    sheet: &StyleSheet,
) -> SectionChrome {
    let ParityHf {
        header,
        footer,
        even_and_odd,
        header_odd,
        footer_odd,
    } = section_parity_hf(pkg, main, dom, sect, sheet);
    SectionChrome {
        page: apply_sect_pr(dom, sect, &sheet.defaults.page),
        header: header.start.runs,
        footer: footer.start.runs,
        header_align: header.start.align,
        footer_align: footer.start.align,
        header_bottom: header.start.border,
        footer_top: footer.start.border,
        watermark: header.start.watermark,
        header_explicit: sect_has_ref(dom, sect, "headerReference"),
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
        punct_squeeze: sheet.defaults.punct_squeeze,
        ul_trail_space: settings_ul_trail_space(pkg),
        do_not_expand_shift_return: settings_do_not_expand_shift_return(pkg),
    }
}

/// A section's picked header/footer plus the even/odd split both
/// `section_chrome` and `first_section_hf` derive from them: the odd part
/// only exists when an even part does.
pub(super) struct ParityHf {
    pub(super) header: PickedHf,
    pub(super) footer: PickedHf,
    pub(super) even_and_odd: bool,
    pub(super) header_odd: Option<ChromePart>,
    pub(super) footer_odd: Option<ChromePart>,
}

pub(super) fn section_parity_hf(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    sheet: &StyleSheet,
) -> ParityHf {
    let mut header = pick_section_hf(pkg, main, dom, sect, "headerReference", sheet);
    let mut footer = pick_section_hf(pkg, main, dom, sect, "footerReference", sheet);
    let header_odd = header
        .even
        .is_some()
        .then(|| std::mem::take(&mut header.odd));
    let footer_odd = footer
        .even
        .is_some()
        .then(|| std::mem::take(&mut footer.odd));
    ParityHf {
        header,
        footer,
        even_and_odd: settings_even_and_odd_headers(pkg),
        header_odd,
        footer_odd,
    }
}

fn sect_has_ref(dom: &Dom, sect: NodeId, local: &str) -> bool {
    !dom.elements(sect, Some(&W::name(local))).is_empty()
}
