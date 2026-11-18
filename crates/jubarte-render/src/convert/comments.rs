// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comments: balloons, anchors, the end-of-document listing and their
//! painting. Also the document walk's entry point, `collect_blocks` (the
//! package to `Vec<Block>`, building the `WalkCtx` it passes to
//! `walk_container`), and the `changed_only` page filter,
//! `keep_changed_pages`.

use super::*;

/// Word's PDF export draws the balloon pane on every page of a document
/// with comments, tracked or not (docxide case63/case64, fixtures_500
/// 00b0c1ee), or with an inserted or deleted table cell, whose "Inserted
/// Cells" / "Deleted Cells" mark only a balloon can carry (English
/// b/57439183; live Word 2026-09-26: one w:cellIns or w:cellDel suffices,
/// a w:ins run inside a cell does not).
///
/// `balloons` is `word_balloon_comments`: in Word mode only the comments
/// Word draws a balloon for bring the pane.
pub(super) fn document_has_balloons(
    pkg: &PartFs,
    main: &str,
    balloons: Option<&HashSet<String>>,
) -> bool {
    pkg.part_string(main).is_some_and(|xml| {
        let comments = balloons.map_or_else(
            || w_revision_count(&xml, "<w:commentReference") > 0,
            |ids| !ids.is_empty(),
        );
        comments
            || ["<w:cellIns", "<w:cellDel"]
                .iter()
                .any(|tag| w_revision_count(&xml, tag) > 0)
    })
}

/// `RevisionStyle::Word` only: the comments Word's Save as PDF draws a
/// balloon for, or `None` in the other styles (every comment gets one).
/// Live Word 16.114, 2026-09-29 (`comment_balloons_0929`, 149 of 151 corpus
/// comment documents; docs/WORD_COMMENT_BALLOONS.md): a comment the body
/// references gets a balloon unless its `w:commentRangeEnd` is dead (at body
/// or cell level, or before any content in its paragraph); a reply
/// (`w15:paraIdParent`) shares its parent's fate. Dropping a comment is a
/// Word mistake, so the other styles keep them all.
pub(super) fn word_balloon_comments(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    body: NodeId,
) -> Option<HashSet<String>> {
    if !matches!(REVISIONS.with(std::cell::Cell::get), RevisionStyle::Word) {
        return None;
    }
    let mut dead = HashSet::new();
    for end in dom.descendants(body, Some(&W::comment_range_end())) {
        let Some(id) = attr_any(dom, end, "id") else {
            continue;
        };
        if !comment_range_end_is_live(dom, end) {
            dead.insert(id.to_string());
        }
    }
    let parents = comment_reply_parents(pkg, main);
    let alive = |id: &str| {
        let mut seen = HashSet::new();
        let mut at = id;
        loop {
            if dead.contains(at) {
                return false;
            }
            match parents.get(at) {
                Some(parent) if seen.insert(at) => at = parent,
                _ => return true,
            }
        }
    };
    Some(
        dom.descendants(body, Some(&W::comment_reference()))
            .into_iter()
            .filter_map(|r| attr_any(dom, r, "id"))
            .filter(|id| alive(id))
            .map(str::to_string)
            .collect(),
    )
}

/// A range end is live inside a `w:p` with content before it there: text or
/// deleted text, a drawing, picture, object, symbol, tab, or a comment
/// reference mark.
/// Its own range start before it in that paragraph counts too: an empty
/// range followed by its reference is a reference alone, with or without
/// an empty run between (Word 16, round 6 of the balloon probes,
/// 2026-10-03: 13 shapes, all with a balloon; corpus 6ef6726c28 and
/// 1672057675 explained, the 151-document survey exact at 151/151).
pub(super) fn comment_range_end_is_live(dom: &Dom, end: NodeId) -> bool {
    let mut up = dom.parent(end);
    while let Some(node) = up {
        if dom.name_is(node, &W::p()) {
            break;
        }
        if dom.name_is(node, &W::body()) || dom.name_is(node, &W::tc()) {
            return false;
        }
        up = dom.parent(node);
    }
    let Some(para) = up else {
        return false;
    };
    // An empty range, its own start right before it in the paragraph, is
    // a reference alone to Word: corpus 6ef6726c28's balloon.
    let id = attr_any(dom, end, "id");
    dom.descendants(para, None)
        .into_iter()
        .take_while(|&node| node != end)
        .any(|node| {
            if dom.name_is(node, &W::comment_range_start()) {
                return id.is_some() && attr_any(dom, node, "id") == id;
            }
            if dom.name_is(node, &W::t()) || dom.name_is(node, &W::del_text()) {
                return (0..dom.child_count(node))
                    .filter_map(|i| dom.text_value(dom.child_at(node, i)))
                    .any(|s| !s.is_empty());
            }
            [
                "drawing",
                "pict",
                "object",
                "sym",
                "tab",
                "commentReference",
            ]
            .iter()
            .any(|tag| dom.name_is(node, &W::name(tag)))
        })
}

/// Reply comment id → parent comment id, from `commentsExtended.xml`'s
/// `w15:paraIdParent` matched to each comment's last `w14:paraId`.
fn comment_reply_parents(pkg: &PartFs, main: &str) -> HashMap<String, String> {
    comments_extended(pkg, main).0
}

/// The comments `commentsExtended.xml` marks resolved (`w15:done="1"`),
/// by comment id.
fn comments_resolved(pkg: &PartFs, main: &str) -> HashSet<String> {
    comments_extended(pkg, main).1
}

/// `commentsExtended.xml` read against the comments part: each reply's
/// parent (by comment id), and the resolved comments.
fn comments_extended(pkg: &PartFs, main: &str) -> (HashMap<String, String>, HashSet<String>) {
    let (Some(comments), Some(extended)) = (
        part_xml_by_rel_kind(pkg, main, "comments"),
        part_xml_by_rel_kind(pkg, main, "commentsExtended"),
    ) else {
        return (HashMap::new(), HashSet::new());
    };
    let mut cdom = Dom::new();
    let cdoc = cdom.parse_xdocument(&comments);
    let mut by_para = HashMap::new();
    if let Some(root) = cdom.root(cdoc) {
        for comment in cdom.descendants(root, Some(&W::comment())) {
            let last = cdom
                .descendants(comment, Some(&W::p()))
                .last()
                .and_then(|&p| cdom.attribute(p, &W14::name("paraId")));
            if let (Some(para), Some(id)) = (last, attr_any(&cdom, comment, "id")) {
                by_para.insert(para.to_string(), id.to_string());
            }
        }
    }
    let mut edom = Dom::new();
    let edoc = edom.parse_xdocument(&extended);
    let mut out = HashMap::new();
    let mut resolved = HashSet::new();
    if let Some(root) = edom.root(edoc) {
        for ex in edom.descendants(root, Some(&W15::name("commentEx"))) {
            let kid = edom
                .attribute(ex, &W15::name("paraId"))
                .and_then(|p| by_para.get(p));
            let parent = edom
                .attribute(ex, &W15::name("paraIdParent"))
                .and_then(|p| by_para.get(p));
            if let (Some(kid), Some(parent)) = (kid, parent) {
                out.insert(kid.clone(), parent.clone());
            }
            if let Some(kid) = kid
                && matches!(edom.attribute(ex, &W15::name("done")), Some("1" | "true"))
            {
                resolved.insert(kid.clone());
            }
        }
    }
    (out, resolved)
}

fn w_revision_count(xml: &str, tag: &str) -> usize {
    xml.match_indices(tag)
        .filter(|(i, _)| {
            matches!(
                xml.as_bytes().get(i + tag.len()).copied(),
                Some(b' ' | b'>' | b'/')
            )
        })
        .count()
}

pub(super) fn collect_blocks(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    body: NodeId,
    sheet: &StyleSheet,
    fonts: &Fonts,
) -> Vec<Block> {
    let _ = fonts;
    let mut blocks = Vec::new();
    let mut numbering = load_numbering(pkg);
    let sects = live_sect_prs(dom, body);
    let mut comments = load_comments(pkg, main);
    number_comments(dom, body, &mut comments, &comment_reply_parents(pkg, main));
    if let Some(balloons) = word_balloon_comments(pkg, main, dom, body) {
        comments.retain(|id, _| balloons.contains(id));
    }
    let ctx = WalkCtx {
        pkg,
        main,
        sheet,
        sects: &sects,
        authors: RefCell::new(AuthorColors::default()),
        comments,
        open_comments: RefCell::new(Vec::new()),
    };
    let mut endnotes = EndnoteBag::load(pkg, main);
    walk_container(&ctx, dom, body, &mut numbering, &mut blocks, &mut endnotes);
    endnotes.emit_remaining(&ctx, &mut numbering, &mut blocks);
    number_endnote_refs(&mut blocks, endnote_num_format(pkg, dom, &sects));
    blocks
}

pub(super) struct WalkCtx<'a> {
    pub(super) pkg: &'a PartFs,
    pub(super) main: &'a str,
    pub(super) sheet: &'a StyleSheet,
    pub(super) sects: &'a [NodeId],
    pub(super) authors: RefCell<AuthorColors>,
    pub(super) comments: HashMap<String, CommentRec>,
    /// Comment ranges open between the blocks: a `w:commentRangeStart` at
    /// body level opens one for the paragraphs that follow.
    pub(super) open_comments: RefCell<Vec<String>>,
}

#[derive(Clone)]
pub(super) struct CommentRec {
    pub(super) author: String,
    pub(super) initials: String,
    pub(super) text: String,
    /// `w15:done` in `commentsExtended.xml`: Word paints the balloon faded.
    pub(super) resolved: bool,
    /// The number in the balloon label ("3", or "3R2" for a reply), set
    /// by `number_comments`; empty when the part was loaded without a body.
    pub(super) label: String,
}

pub(super) fn load_comments(pkg: &PartFs, main: &str) -> HashMap<String, CommentRec> {
    let Some(xml) = part_xml_by_rel_kind(pkg, main, "comments") else {
        return HashMap::new();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return HashMap::new();
    };
    let resolved = comments_resolved(pkg, main);
    let mut out = HashMap::new();
    for node in dom.descendants(root, Some(&W::comment())) {
        let Some(id) = attr_any(&dom, node, "id").map(str::to_string) else {
            continue;
        };
        let author = attr_any(&dom, node, "author").unwrap_or("").to_string();
        let initials = attr_any(&dom, node, "initials").unwrap_or("").to_string();
        let mut text = String::new();
        for t in dom.descendants(node, Some(&W::t())) {
            if let Some(s) = dom.text_value(t).or_else(|| {
                (0..dom.child_count(t)).find_map(|i| dom.text_value(dom.child_at(t, i)))
            }) {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(s);
            }
        }
        if text.is_empty() {
            text = element_text(&dom, node);
        }
        let done = resolved.contains(&id);
        out.insert(
            id,
            CommentRec {
                author,
                initials,
                text,
                resolved: done,
                label: String::new(),
            },
        );
    }
    out
}

/// The pale fill Word gives a balloon (and the commented range) for an
/// author's ink: its own table for the first ten palette colours (read off
/// its PDFs: 466 balloons of 152 corpus documents, `word_balloon_spec.py`),
/// a 16 % blend toward white for the rest.
pub(super) fn balloon_tint(color: [f32; 3]) -> [f32; 3] {
    const TABLE: [([u8; 3], [u8; 3]); 10] = [
        ([0xD1, 0x34, 0x38], [248, 220, 221]),
        ([0x00, 0x78, 0xD4], [213, 237, 255]),
        ([0x5C, 0x2E, 0x91], [234, 223, 244]),
        ([0x49, 0x82, 0x05], [236, 253, 215]),
        ([0xCC, 0x35, 0x95], [247, 221, 237]),
        ([0x4E, 0x6A, 0xED], [217, 223, 251]),
        ([0x6D, 0x57, 0x00], [255, 247, 213]),
        ([0xCF, 0x0F, 0x1F], [252, 216, 219]),
        ([0x39, 0x41, 0x46], [232, 235, 236]),
        ([0x0B, 0x6A, 0x0B], [217, 251, 217]),
    ];
    let rgb = color.map(|c| (c * 255.0).round() as u8);
    TABLE.iter().find(|(ink, _)| *ink == rgb).map_or_else(
        || lighten(color, 0.16),
        |(_, tint)| tint.map(|c| f32::from(c) / 255.0),
    )
}

/// `PdfOptions::changed_only`: the pages a body or note change bar marks,
/// the pages a comment is anchored on, and the first page of every section
/// on which a revised header or footer paints; the first page when none
/// qualifies. A continuous section starts no page, so its pages count with
/// the section before (Word starts no header there either). With a
/// different first page, a revised default header first paints on the
/// section's second page, and that page is the one kept.
pub(super) fn keep_changed_pages(pages: &mut Vec<Page>) {
    let mut first_painted: BTreeMap<u32, usize> = BTreeMap::new();
    for page in pages.iter().filter(|p| p.chrome_changed) {
        first_painted.entry(page.section).or_insert(page.number);
    }
    let keep = |p: &Page| {
        p.changed || !p.comments.is_empty() || first_painted.get(&p.section) == Some(&p.number)
    };
    if pages.iter().any(keep) {
        pages.retain(keep);
    } else {
        pages.truncate(1);
    }
}

/// `CommentPlacement::End`: a `[JR1]` superscript in the author's ink
/// where comments are anchored, carrying them, so the line reflows around
/// it as around Word's inline comment marks. A reply ("1R2") rides on its
/// thread's marker, and is listed under it.
pub(super) fn mark_comment_anchors(runs: &mut Vec<TextRun>) {
    if runs.iter().all(|r| r.comments.is_empty()) {
        return;
    }
    let marker = |notes: Vec<CommentNote>, host: &RunStyle| {
        let is_reply = |n: &CommentNote| n.label.contains('R');
        let shown: Vec<&CommentNote> = if notes.iter().all(is_reply) {
            notes.iter().collect()
        } else {
            notes.iter().filter(|n| !is_reply(n)).collect()
        };
        let text: String = shown
            .iter()
            .map(|n| format!("[{}{}]", n.initials, n.label))
            .collect();
        let mut style = host.clone();
        style.vert = VertAlign::Super;
        style.color = shown.first().map_or(style.color, |n| n.color);
        style.color_auto = false;
        style.underline = false;
        style.underline_double = false;
        style.underline_wave = false;
        style.strike = false;
        style.strike_double = false;
        style.highlight = None;
        style.highlight_marker = false;
        style.tint = None;
        style.tint_id = None;
        style.caps = false;
        style.small_caps = false;
        style.hidden = false;
        // An effect run Word leaves unpainted must not hide the marker.
        style.effect_skip = false;
        let mut run = TextRun::new(text, style);
        run.comment_marker = true;
        run.comments = notes
            .into_iter()
            .map(|note| CommentNote {
                after: true,
                ..note
            })
            .collect();
        run
    };
    let mut out = Vec::with_capacity(runs.len() + 2);
    for mut run in runs.drain(..) {
        let (after, before): (Vec<_>, Vec<_>) = std::mem::take(&mut run.comments)
            .into_iter()
            .partition(|n| n.after);
        if !before.is_empty() {
            out.push(marker(before, &run.style));
        }
        let host = run.style.clone();
        out.push(run);
        if !after.is_empty() {
            out.push(marker(after, &host));
        }
    }
    *runs = out;
}

/// `CommentPlacement::End`: the comments of the pages kept, taken off
/// them and listed on pages of their own after the last one, in reading
/// order: `[JR1] Author, page N` in the author's ink, then the comment's
/// text, in the document's default face. The listing's pages number on
/// from `laid`, the document's page count, so none takes the number of a
/// document page `PdfOptions::changed_only` dropped.
pub(super) fn list_comments_at_end(
    fonts: &Fonts,
    sheet: &StyleSheet,
    pages: &mut Vec<Page>,
    laid: usize,
) {
    const MARGIN: f32 = 72.0;
    const HEADING: f32 = 14.0;
    const SIZE: f32 = 10.0;
    const INDENT: f32 = 18.0;
    const PITCH: f32 = 1.25;
    const GAP: f32 = 8.0;
    let mut listed: Vec<(usize, PdfComment)> = Vec::new();
    for page in pages.iter_mut() {
        let mut notes = std::mem::take(&mut page.comments);
        // Reference labels retain document order across columns and replies.
        notes.sort_by_key(|note| label_order(&note.label));
        listed.extend(notes.into_iter().map(|note| (page.number, note)));
    }
    if listed.is_empty() {
        return;
    }
    // The last horizontal page's paper; else a vertical page's, which is
    // stored turned a quarter.
    let (width, height) = pages
        .iter()
        .rev()
        .find(|p| !p.vertical)
        .map(|p| (p.width, p.height))
        .or_else(|| pages.last().map(|p| (p.height, p.width)))
        .unwrap_or((612.0, 792.0));
    let mut regular = sheet.defaults.run.clone();
    regular.size = SIZE;
    regular.bold = false;
    regular.italic = false;
    let mut bold = regular.clone();
    bold.bold = true;
    let line_width = (width - 2.0 * MARGIN).max(1.0);
    let mut out = vec![Page::new(width, height)];
    let bold_face = ink_face(fonts, &bold, "Comments");
    let mut y = height - MARGIN - fonts.get(bold_face).ascent_pt(HEADING);
    let mut title = bold.clone();
    title.size = HEADING;
    title.color = [0.0; 3];
    paint_listing_line(
        fonts,
        out.last_mut().unwrap(),
        &[TextRun::new("Comments", title)],
        MARGIN,
        y,
    );
    y -= HEADING * PITCH + GAP;
    let pitch = SIZE * PITCH;
    for (number, note) in &listed {
        let label = format!("[{}{}]", note.initials, note.label);
        let by = if note.resolved {
            format!("{}, page {number} (resolved)", note.author)
        } else {
            format!("{}, page {number}", note.author)
        };
        let mut label_style = bold.clone();
        label_style.color = note.color;
        let mut author_style = regular.clone();
        author_style.color = [0.35; 3];
        let headings = wrap_listing_runs(
            fonts,
            &[
                TextRun::new(label, label_style),
                TextRun::new(by, author_style),
            ],
            line_width,
        );
        let mut contents_style = regular.clone();
        contents_style.color = if note.resolved { [0.5; 3] } else { [0.0; 3] };
        let lines = wrap_listing_runs(
            fonts,
            &[TextRun::new(&note.contents, contents_style)],
            (line_width - INDENT).max(1.0),
        );
        // Keep the entire heading and first content line together when
        // that group fits a page; very long headings can span pages. From
        // the first heading's baseline, `needed` reaches the first content
        // line's.
        let needed = headings.len() as f32 * pitch;
        let fresh_y = height - MARGIN - fonts.get(bold_face).ascent_pt(SIZE);
        if y - needed < MARGIN && fresh_y - needed >= MARGIN {
            out.push(Page::new(width, height));
            y = fresh_y;
        }
        for (heading, x) in headings
            .iter()
            .map(|line| (line, MARGIN))
            .chain(lines.iter().map(|line| (line, MARGIN + INDENT)))
        {
            if y < MARGIN {
                out.push(Page::new(width, height));
                y = fresh_y;
            }
            paint_listing_line(fonts, out.last_mut().unwrap(), heading, x, y);
            y -= pitch;
        }
        y -= GAP;
    }
    for (i, page) in out.iter_mut().enumerate() {
        page.number = laid + 1 + i;
    }
    pages.extend(out);
}

/// Contiguous text using the same script-aware face. Measurement and
/// painting share these pieces so a fallback never changes the wrap width.
pub(super) fn listing_faces<'a>(
    fonts: &Fonts,
    style: &RunStyle,
    text: &'a str,
) -> Vec<(FaceRef, &'a str)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut previous = None;
    for (at, ch) in text.char_indices() {
        let mut bytes = [0; 4];
        let face = if ch.is_whitespace() {
            // Neutral spaces share their script's shaping face, retaining
            // the ordering and kerning of a whole RTL phrase.
            previous.unwrap_or_else(|| ink_face(fonts, style, ch.encode_utf8(&mut bytes)))
        } else {
            ink_face(fonts, style, ch.encode_utf8(&mut bytes))
        };
        if previous.is_some_and(|before| before != face) {
            out.push((previous.unwrap(), &text[start..at]));
            start = at;
        }
        previous = Some(face);
    }
    if let Some(face) = previous {
        out.push((face, &text[start..]));
    }
    out
}

/// Width in the exact faces used by the comment listing painter.
fn listing_width(fonts: &Fonts, style: &RunStyle, text: &str) -> f32 {
    listing_faces(fonts, style, text)
        .iter()
        .map(|(face, piece)| advance(fonts, *face, piece, style.size))
        .sum()
}

/// Paint a listing line preserving label ink and author/body styles.
pub(super) fn paint_listing_line(
    fonts: &Fonts,
    page: &mut Page,
    runs: &[TextRun],
    mut x: f32,
    y: f32,
) {
    for run in runs {
        for (face, text) in listing_faces(fonts, &run.style, &run.text) {
            let glyphs = fonts
                .get(face)
                .shape_kern(text, run.style.size, true)
                .iter()
                .map(|(glyph, _)| *glyph)
                .collect();
            page.ops.push(Op::text(
                face,
                run.style.size,
                x,
                y,
                glyphs,
                run.style.color,
                text,
            ));
            x += advance(fonts, face, text, run.style.size);
        }
    }
}

/// Word wrapping for styled listing text, including unbroken tokens.
/// Existing loaded faces supply both the split decisions and final ink.
pub(super) fn wrap_listing_runs(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<Vec<TextRun>> {
    let append = |line: &mut Vec<TextRun>, text: &str, style: &RunStyle| {
        if text.is_empty() {
            return;
        }
        if let Some(last) = line.last_mut()
            && style_eq(&last.style, style)
        {
            last.text.push_str(text);
        } else {
            line.push(TextRun::new(text, style.clone()));
        }
    };
    let mut lines = Vec::new();
    let mut line = Vec::new();
    for run in runs {
        for (paragraph, text) in run.text.split('\n').enumerate() {
            if paragraph != 0 {
                lines.push(std::mem::take(&mut line));
            }
            for word in text.split_whitespace() {
                let mut pieces = Vec::new();
                let mut start = 0;
                let mut end = 0;
                for (at, ch) in word.char_indices() {
                    let next = at + ch.len_utf8();
                    if at > start && listing_width(fonts, &run.style, &word[start..next]) > width {
                        pieces.push(&word[start..at]);
                        start = at;
                    }
                    end = next;
                }
                pieces.push(&word[start..end]);
                for (index, piece) in pieces.iter().enumerate() {
                    let mut candidate = line.clone();
                    if !line.is_empty() && index == 0 {
                        append(&mut candidate, " ", &run.style);
                    }
                    append(&mut candidate, piece, &run.style);
                    // Measure the coalesced text, including neutral spaces,
                    // in exactly the same script faces as final painting.
                    let used: f32 = candidate
                        .iter()
                        .map(|part| listing_width(fonts, &part.style, &part.text))
                        .sum();
                    if !line.is_empty() && used > width {
                        lines.push(std::mem::take(&mut line));
                        append(&mut line, piece, &run.style);
                    } else {
                        line = candidate;
                    }
                }
            }
        }
    }
    lines.push(line);
    lines
}

/// A balloon label's place in reading order: `"3R2"` is `(3, 2)`, `"3"`
/// is `(3, 1)`; an unnumbered label sorts last.
fn label_order(label: &str) -> (usize, usize) {
    let (thread, reply) = label.split_once('R').unwrap_or((label, "1"));
    (
        thread.parse().unwrap_or(usize::MAX),
        reply.parse().unwrap_or(usize::MAX),
    )
}

/// Word's comment balloons, painted into the markup pane the way its Save
/// as PDF paints them (466 balloons of 152 corpus documents, 2026-10-03).
/// The chrome is measured in page units (the pane's `cm` scales the page
/// by `k`, so they divide by `k` here); the text in laid-out units:
///
/// - the box starts 16.67pt right of the pane's left edge and ends 3.4pt
///   short of its right edge, a rounded rectangle filled with the author's
///   tint and stroked 0.36pt in the author's ink;
/// - its top sits on the top of the commented line, or 0.72pt under the
///   balloon above when that one reaches lower;
/// - `"Commented [<initials><n>]: "` bold, then the comment's text, both at
///   the Balloon Text style's size (9pt unless the document's style says
///   otherwise), the label in that style's face (Times New Roman when the
///   document has none), the text in the document's default face; the
///   first line's top 2.67pt under the box top, lines 1.236 × size apart,
///   the box 4.31pt taller than its lines;
/// - a dotted 0.18pt connector in the author's ink runs from the end of
///   the commented text along its line's bottom to 3.5pt left of the pane,
///   then straight to the box's left edge 4.6pt under its top.
pub(super) fn paint_comment_balloons(fonts: &Fonts, sheet: &StyleSheet, pages: &mut [Page]) {
    const BOX_LEFT: f32 = 16.67;
    const BOX_RIGHT_GAP: f32 = 3.4;
    const INSET: f32 = 3.4;
    const STROKE: f32 = 0.358;
    const CONNECTOR: f32 = 0.179;
    const BRACKET_OFF: f32 = 0.09;
    const BRACKET_SERIF: f32 = 0.18;
    const DOT: f32 = 0.5376;
    const STACK_GAP: f32 = 0.717;
    const PANE_LEAD: f32 = 3.5;
    const ELBOW_DROP: f32 = 4.6;
    const CORNER: f32 = 2.4;
    const FIRST_LINE_TOP: f32 = 2.67;
    const BOX_PAD: f32 = 4.31;
    const PITCH: f32 = 1.236;
    // A resolved face is a requested font in the report: a document that
    // paints no balloon asks for neither the label's face nor the text's.
    if !pages
        .iter()
        .any(|p| p.markup_pane && !p.comments.is_empty() && !p.vertical)
    {
        return;
    }
    let balloon = sheet.by_id.get("BalloonText");
    let size = balloon.filter(|b| b.sets_size).map_or(9.0, |b| b.run.size);
    let label_family = balloon
        .filter(|b| b.sets_family)
        .map_or("Times New Roman", |b| b.run.family.as_str());
    let label_face = fonts.resolve(label_family, true, false);
    let text_face = fonts.resolve(&sheet.defaults.run.family, false, false);
    let ascent = fonts.get(label_face).ascent_pt(size);
    let pitch = PITCH * size;
    for page in pages.iter_mut() {
        if !page.markup_pane || page.comments.is_empty() || page.vertical {
            continue;
        }
        let Some(chrome) = pdf::markup_chrome(page.width, page.height, page.margin_r) else {
            continue;
        };
        let k = chrome.k;
        let pane_left = (chrome.gx - chrome.tx) / k;
        let pane_right = pane_left + chrome.gw / k;
        let box_x0 = pane_left + BOX_LEFT / k;
        let box_x1 = pane_right - BOX_RIGHT_GAP / k;
        let text_x = box_x0 + INSET / k;
        let text_w = box_x1 - INSET / k - text_x;
        if text_w < 20.0 {
            continue;
        }
        let mut notes: Vec<PdfComment> = page.comments.clone();
        // Top of the page first (y grows upward), left to right on a line.
        notes.sort_by(|a, b| {
            b.top
                .partial_cmp(&a.top)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });
        let mut ceiling = f32::MAX;
        let mut ops = Vec::new();
        for note in &notes {
            let label = format!("Commented [{}{}]: ", note.initials, note.label);
            let words: Vec<(FaceRef, String, f32)> = [
                (label_face, label.as_str()),
                (text_face, note.contents.as_str()),
            ]
            .into_iter()
            .flat_map(|(face, text)| {
                text.split_inclusive(' ')
                    .map(move |word| (face, word.to_string(), advance(fonts, face, word, size)))
            })
            .collect();
            // Greedy lines: a word that does not fit starts the next line;
            // the blank ending a line's last word carries no width.
            let mut lines: Vec<Vec<(FaceRef, String, f32)>> = vec![Vec::new()];
            let mut used = 0.0f32;
            for (face, word, adv) in words {
                let bare = if word.ends_with(' ') {
                    adv - advance(fonts, face, " ", size)
                } else {
                    adv
                };
                if used > 0.0 && used + bare > text_w {
                    lines.push(Vec::new());
                    used = 0.0;
                }
                used += adv;
                lines.last_mut().expect("a line").push((face, word, adv));
            }
            let y0 = note.top.min(ceiling);
            let y1 = y0 - (BOX_PAD + pitch * lines.len() as f32);
            ceiling = y1 - STACK_GAP / k;
            // A resolved comment is faded: the stroke takes the tint, the
            // fill 16 % of that, the text Word's BFBFBF grey (d2b26d3d09).
            let (ink, fill, text_ink) = if note.resolved {
                let ink = balloon_tint(note.color);
                (ink, lighten(ink, 0.16), [0.749; 3])
            } else {
                (note.color, balloon_tint(note.color), [0.0; 3])
            };
            let contour = rounded_box(box_x0, y1, box_x1, y0, CORNER / k);
            ops.push(Op::FillPath {
                contours: vec![contour.clone()],
                color: fill,
                even_odd: false,
            });
            ops.push(Op::StrokePath {
                subpaths: vec![(contour, true)],
                width: STROKE / k,
                color: ink,
            });
            let mut baseline = y0 - FIRST_LINE_TOP - ascent;
            for line in &lines {
                let mut x = text_x;
                for (face, word, adv) in line {
                    let shaped = fonts.get(*face).shape_kern(word, size, true);
                    ops.push(Op::Text {
                        face: *face,
                        size,
                        x,
                        y: baseline,
                        glyphs: shaped.iter().map(|(g, _)| *g).collect(),
                        color: text_ink,
                        text: word.clone(),
                        hscale: 1.0,
                        hidden: false,
                    });
                    x += adv;
                }
                baseline -= pitch;
            }
            // Word brackets the range in the author's ink: a connector-
            // width stroke down each end of the tinted text with 0.18pt
            // serifs pointing in (bb35ae41ba: "[" at 77.03 for the range
            // from 77.12; 08286ffbfb: "[" 54.63 and "]" 69.5 round
            // 54.72–69.24, the connector leaving from 69.32).
            let rects: Vec<&pdf::CommentTint> =
                page.tints.iter().filter(|t| t.id == note.id).collect();
            if let (Some(first), Some(last)) = (rects.first(), rects.last()) {
                let (off, serif, width) = (BRACKET_OFF / k, BRACKET_SERIF / k, CONNECTOR / k);
                let mut stroke = |x1: f32, y1: f32, x2: f32, y2: f32| {
                    ops.push(Op::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        width,
                        color: ink,
                    });
                };
                let x = first.x - off;
                stroke(x, first.y, x, first.y + first.h);
                stroke(x, first.y, x + serif, first.y);
                stroke(x, first.y + first.h, x + serif, first.y + first.h);
                let x = last.x + last.w + off;
                stroke(x, last.y, x, last.y + last.h);
                stroke(x, last.y, x - serif, last.y);
                stroke(x, last.y + last.h, x - serif, last.y + last.h);
            }
            let elbow = (pane_left - PANE_LEAD / k, note.bottom);
            let dot = DOT / k;
            dotted(
                &mut ops,
                (note.x, note.bottom),
                elbow,
                dot,
                CONNECTOR / k,
                ink,
            );
            dotted(
                &mut ops,
                elbow,
                (box_x0, y0 - ELBOW_DROP / k),
                dot,
                CONNECTOR / k,
                ink,
            );
        }
        page.ops.extend(ops);
    }
}

/// The advance of `text` in `face` at `size`.
pub(super) fn advance(fonts: &Fonts, face: FaceRef, text: &str, size: f32) -> f32 {
    fonts
        .get(face)
        .shape_kern(text, size, true)
        .iter()
        .map(|(_, a)| a)
        .sum()
}

/// A rectangle with its corners rounded by `r`, as a closed polyline.
fn rounded_box(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> Vec<(f32, f32)> {
    let (y0, y1) = (y0.min(y1), y0.max(y1));
    let r = r.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0).max(0.0);
    let arc = |cx: f32, cy: f32, from: f32, to: f32| -> Vec<(f32, f32)> {
        (0..=4)
            .map(|i| {
                let t = from + (to - from) * i as f32 / 4.0;
                (cx + r * t.cos(), cy + r * t.sin())
            })
            .collect()
    };
    use std::f32::consts::PI;
    let mut pts = Vec::with_capacity(20);
    pts.extend(arc(x0 + r, y0 + r, PI, 1.5 * PI));
    pts.extend(arc(x1 - r, y0 + r, 1.5 * PI, 2.0 * PI));
    pts.extend(arc(x1 - r, y1 - r, 0.0, 0.5 * PI));
    pts.extend(arc(x0 + r, y1 - r, 0.5 * PI, PI));
    pts
}

/// Square dots of side `width` every `2 * dot` along the segment, the
/// `[dot dot] 0 d` of Word's connector drawn without a dash pattern.
fn dotted(
    ops: &mut Vec<Op>,
    from: (f32, f32),
    to: (f32, f32),
    dot: f32,
    width: f32,
    color: [f32; 3],
) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 || dot <= 0.0 {
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let mut at = 0.0f32;
    while at < len {
        let seg = dot.min(len - at);
        let (sx, sy) = (from.0 + ux * at, from.1 + uy * at);
        let (ex, ey) = (sx + ux * seg, sy + uy * seg);
        ops.push(Op::Line {
            x1: sx,
            y1: sy,
            x2: ex,
            y2: ey,
            width,
            color,
        });
        at += 2.0 * dot;
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod dotted_branch_tests {
    use super::*;

    fn lines(from: (f32, f32), to: (f32, f32), dot: f32) -> usize {
        let mut ops = Vec::new();
        dotted(&mut ops, from, to, dot, 0.5, [0.0; 3]);
        ops.len()
    }

    #[test]
    fn a_dotted_connector_needs_length_and_a_dot() {
        assert_eq!(lines((0.0, 0.0), (0.0, 0.0), 1.0), 0);
        assert_eq!(lines((0.0, 0.0), (10.0, 0.0), 0.0), 0);
        assert!(lines((0.0, 0.0), (10.0, 0.0), 1.0) > 0);
    }
}
