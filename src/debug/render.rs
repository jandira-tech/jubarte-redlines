// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte debug FILE --check render`: what each story part should put on
//! the page, for diagnosing a low score against Word's PDF. Every story part
//! is listed (body, headers, footers, notes, comments; text boxes inside
//! them), not only `word/document.xml`, and a `(layout)` entry adds what
//! jubarte's own layout pass makes of the package: pages and the face each
//! requested font resolved to.

use std::collections::{BTreeMap, HashMap};

use super::{
    Package, TEXT_PARTS, attr, clip, decode_xml, local, percent_decode, rels_of, resolve_target,
    story_roles_all,
};
use crate::xmllinq::{Dom, NodeId};

/// Sample text kept per aggregated line.
const SAMPLE: usize = 40;
/// Longest first-row text a table line keeps.
const FIRST_ROW: usize = 60;
/// Deepest `basedOn` chain followed.
const CHAIN: usize = 16;

/// Part name → (lines, paragraph or page count), the `(layout)` entry last.
pub(super) fn render_parts(
    pkg: &Package,
    part: Option<&str>,
) -> BTreeMap<String, (Vec<String>, usize)> {
    let rels = main_rels(pkg);
    let styles = rels
        .iter()
        .find(|r| r.kind == "styles")
        .and_then(|r| parse(pkg, &r.part))
        .map(|(dom, root)| StyleBook::new(&dom, root))
        .unwrap_or_default();
    let roles = story_roles_all(pkg);
    let mut map = BTreeMap::new();
    for e in &pkg.entries {
        if part.is_some_and(|p| !e.name.contains(p)) || !e.name.ends_with(".xml") {
            continue;
        }
        let Some((dom, root)) = parse(pkg, &e.name) else {
            continue;
        };
        if !TEXT_PARTS.contains(&local(&dom, root).as_str()) {
            continue;
        }
        let mut lines: Vec<String> = roles
            .get(&e.name)
            .into_iter()
            .flatten()
            .map(|r| format!("  shown as {r}"))
            .collect();
        let paras = part_lines(&dom, root, &styles, &rels, &mut lines);
        map.insert(e.name.clone(), (lines, paras));
    }
    if part.is_none_or(|p| "(layout)".contains(p)) {
        map.insert("(layout)".to_string(), layout_lines(pkg, &rels));
    }
    map
}

/// A relationship of the main document part: its type's last segment
/// (`styles`, `theme`, `header`, …), id and target part.
struct Rel {
    kind: String,
    id: String,
    part: String,
}

/// The main document part's relationships, found through the package's own
/// `officeDocument` relationship (the part need not be `word/document.xml`).
fn main_rels(pkg: &Package) -> Vec<Rel> {
    let rels_in = |rels: &str| -> Vec<Rel> {
        let Some((dom, root)) = parse(pkg, rels) else {
            return Vec::new();
        };
        dom.elements(root, None)
            .into_iter()
            .filter(|&r| attr(&dom, r, "TargetMode") != "External")
            .map(|r| Rel {
                kind: attr(&dom, r, "Type")
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                id: attr(&dom, r, "Id"),
                part: resolve_target(rels, &percent_decode(&attr(&dom, r, "Target"))),
            })
            .collect()
    };
    let Some(main) = rels_in("_rels/.rels")
        .into_iter()
        .find(|r| r.kind == "officeDocument")
    else {
        return Vec::new();
    };
    rels_in(&rels_of(&main.part))
}

fn parse(pkg: &Package, name: &str) -> Option<(Dom, NodeId)> {
    let e = pkg.entries.iter().find(|e| e.name == name)?;
    let xml = decode_xml(&e.data)?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    Some((dom, root))
}

/// What a style contributes to the page, its `basedOn` parent and name.
#[derive(Default)]
struct StyleInfo {
    name: String,
    based: String,
    /// The style's own paragraph fill: `Some(None)` for a nil `shd`, which
    /// stops its base's.
    para_shd: Option<Option<String>>,
    /// The style's own run colour, highlight and shading: `None` when it
    /// sets none (its base's shows), `Some(None)` when it resets them
    /// (`auto`, `none`, a nil `shd`).
    color: Option<Option<String>>,
    highlight: Option<Option<String>>,
    run_shd: Option<Option<String>>,
    /// The style's own `w:vanish`: `Some(true)` hides, `Some(false)` turns
    /// a base's off.
    vanish: Option<bool>,
    /// The style's own `w:framePr` attributes, `name=value`.
    frame: Vec<(String, String)>,
    /// The face each slot of the style's own `w:rFonts` asks for.
    fonts: Vec<(&'static str, String)>,
    /// Table-wide and conditional shading: `whole` (`tblPr`), `cell`
    /// (`tcPr`), or the `tblStylePr` type; `None` for a nil `shd`, which
    /// resets the base's.
    table_shd: Vec<(String, Option<String>)>,
}

#[derive(Default)]
struct StyleBook {
    by_id: HashMap<String, StyleInfo>,
    /// What `docDefaults` paint: the layer under every style.
    defaults: StyleInfo,
    /// `docDefaults` run fonts, `name=value` sorted.
    default_fonts: Option<String>,
    /// The `w:default="1"` paragraph style: an unstyled paragraph's.
    default_para: String,
}

impl StyleBook {
    fn new(dom: &Dom, root: NodeId) -> Self {
        let mut book = StyleBook::default();
        for s in dom.elements(root, None) {
            match local(dom, s).as_str() {
                "docDefaults" => {
                    let rpr = descendant(dom, s, "rPrDefault").and_then(|d| child(dom, d, "rPr"));
                    book.defaults = StyleInfo {
                        name: "docDefaults".to_string(),
                        para_shd: descendant(dom, s, "pPrDefault")
                            .and_then(|d| child(dom, d, "pPr"))
                            .and_then(|p| child(dom, p, "shd"))
                            .map(|x| shd(dom, x)),
                        color: rpr.and_then(|r| color(dom, r)),
                        highlight: rpr.and_then(|r| highlight(dom, r)),
                        run_shd: rpr.and_then(|r| run_shd(dom, r)),
                        vanish: rpr
                            .and_then(|r| child(dom, r, "vanish").map(|_| on(dom, r, "vanish"))),
                        ..StyleInfo::default()
                    };
                    book.default_fonts = descendant(dom, s, "rFonts").map(|f| {
                        let mut items: Vec<String> = dom
                            .attributes(f)
                            .into_iter()
                            .map(|(k, v)| format!("{}={v}", k.local_name()))
                            .collect();
                        items.sort();
                        items.join(" ")
                    });
                }
                "style" => {
                    let val = |name: &str| child(dom, s, name).map(|c| attr(dom, c, "val"));
                    let rpr = child(dom, s, "rPr");
                    let mut info = StyleInfo {
                        name: val("name").unwrap_or_default(),
                        based: val("basedOn").unwrap_or_default(),
                        para_shd: child(dom, s, "pPr")
                            .and_then(|p| child(dom, p, "shd"))
                            .map(|x| shd(dom, x)),
                        color: rpr.and_then(|r| color(dom, r)),
                        highlight: rpr.and_then(|r| highlight(dom, r)),
                        run_shd: rpr.and_then(|r| run_shd(dom, r)),
                        vanish: rpr
                            .and_then(|r| child(dom, r, "vanish").map(|_| on(dom, r, "vanish"))),
                        frame: child(dom, s, "pPr")
                            .and_then(|p| child(dom, p, "framePr"))
                            .map(|f| attr_pairs(dom, f))
                            .unwrap_or_default(),
                        fonts: rpr
                            .and_then(|r| child(dom, r, "rFonts"))
                            .map(|f| font_slots(dom, f))
                            .unwrap_or_default(),
                        table_shd: Vec::new(),
                    };
                    let mut table = |label: String, holder: NodeId| {
                        for (pr, what) in [("tblPr", "whole"), ("tcPr", "cell")] {
                            if let Some(fill) = child(dom, holder, pr)
                                .and_then(|p| child(dom, p, "shd"))
                                .map(|x| shd(dom, x))
                            {
                                let key = if label.is_empty() {
                                    what.to_string()
                                } else {
                                    label.clone()
                                };
                                info.table_shd.push((key, fill));
                            }
                        }
                    };
                    table(String::new(), s);
                    for cond in kids(dom, s, "tblStylePr") {
                        table(attr(dom, cond, "type"), cond);
                    }
                    let id = attr(dom, s, "styleId");
                    if attr(dom, s, "type") == "paragraph"
                        && matches!(attr(dom, s, "default").as_str(), "1" | "true" | "on")
                    {
                        book.default_para.clone_from(&id);
                    }
                    book.by_id.insert(id, info);
                }
                _ => {}
            }
        }
        book
    }

    fn name(&self, id: &str) -> String {
        self.by_id
            .get(id)
            .map(|s| s.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.to_string())
    }

    /// `id`'s table shading down its `basedOn` chain: a base's conditions
    /// stay unless the derived style sets the same one; a nil `shd` resets
    /// it.
    fn table_shd(&self, id: &str) -> Vec<(String, String)> {
        let mut chain = Vec::new();
        let mut id = id.to_string();
        for _ in 0..CHAIN {
            let Some(s) = self.by_id.get(&id) else {
                break;
            };
            chain.push(s);
            if s.based.is_empty() {
                break;
            }
            id = s.based.clone();
        }
        let mut out: Vec<(String, Option<String>)> = Vec::new();
        for s in chain.into_iter().rev() {
            for (key, fill) in &s.table_shd {
                match out.iter_mut().find(|(k, _)| k == key) {
                    Some(slot) => slot.1.clone_from(fill),
                    None => out.push((key.clone(), fill.clone())),
                }
            }
        }
        out.into_iter()
            .filter_map(|(k, fill)| Some((k, fill?)))
            .collect()
    }

    /// The faces `id`'s `basedOn` chain asks for, slot by slot, each with
    /// the name of the style that sets it (a derived style's slot over its
    /// base's, as the converter overlays them).
    fn font_slots(&self, id: &str) -> Vec<(&'static str, String, String)> {
        let mut chain = Vec::new();
        let mut cur = id.to_string();
        for _ in 0..CHAIN {
            let Some(s) = self.by_id.get(&cur) else {
                break;
            };
            chain.push((s, self.name(&cur)));
            if s.based.is_empty() {
                break;
            }
            cur = s.based.clone();
        }
        let mut out: Vec<(&'static str, String, String)> = Vec::new();
        for (s, name) in chain.into_iter().rev() {
            for (slot, face) in &s.fonts {
                out.retain(|(k, _, _)| k != slot);
                out.push((slot, face.clone(), name.clone()));
            }
        }
        out
    }

    /// `id`'s frame attributes down its `basedOn` chain, a derived
    /// style's value over its base's.
    fn frame(&self, id: &str) -> Vec<(String, String)> {
        let mut chain = Vec::new();
        let mut id = id.to_string();
        for _ in 0..CHAIN {
            let Some(s) = self.by_id.get(&id) else {
                break;
            };
            chain.push(s);
            if s.based.is_empty() {
                break;
            }
            id = s.based.clone();
        }
        let mut out: Vec<(String, String)> = Vec::new();
        for s in chain.into_iter().rev() {
            merge_pairs(&mut out, &s.frame);
        }
        out
    }

    /// The paragraph style `id` names, or the default one.
    fn para_style(&self, id: String) -> String {
        if id.is_empty() {
            self.default_para.clone()
        } else {
            id
        }
    }

    /// The first value `f` finds walking `id`'s `basedOn` chain, with the
    /// name of the style that holds it.
    fn find<T>(&self, id: &str, f: impl Fn(&StyleInfo) -> Option<T>) -> Option<(T, String)> {
        let mut id = id.to_string();
        for _ in 0..CHAIN {
            let s = self.by_id.get(&id)?;
            if let Some(v) = f(s) {
                return Some((v, self.name(&id)));
            }
            if s.based.is_empty() {
                return None;
            }
            id = s.based.clone();
        }
        None
    }
}

fn kids(dom: &Dom, n: NodeId, name: &str) -> Vec<NodeId> {
    dom.elements(n, None)
        .into_iter()
        .filter(|&c| local(dom, c) == name)
        .collect()
}

fn child(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId> {
    dom.elements(n, None)
        .into_iter()
        .find(|&c| local(dom, c) == name)
}

fn descendant(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId> {
    dom.descendants(n, None)
        .into_iter()
        .find(|&c| local(dom, c) == name)
}

/// The nearest ancestor of `n` named `name`.
fn nearest(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId> {
    dom.ancestors(n, None)
        .into_iter()
        .find(|&a| local(dom, a) == name)
}

/// A `w:shd`'s visible fill: `fill` (or `theme:…`), the pattern's colour
/// after `/` when the pattern is not clear. `None` for no fill at all.
fn shd(dom: &Dom, x: NodeId) -> Option<String> {
    let (val, fill, theme, color) = (
        attr(dom, x, "val"),
        attr(dom, x, "fill"),
        attr(dom, x, "themeFill"),
        attr(dom, x, "color"),
    );
    if val == "nil" {
        return None;
    }
    if val == "solid" {
        return Some(if color.is_empty() || color == "auto" {
            "solid:auto".to_string()
        } else {
            color
        });
    }
    let base = if !theme.is_empty() {
        format!("theme:{theme}")
    } else if fill.is_empty() || fill == "auto" {
        String::new()
    } else {
        fill
    };
    let pattern = (!matches!(val.as_str(), "" | "clear")).then(|| format!("/{val}:{color}"));
    match (base.is_empty(), pattern) {
        (true, None) => None,
        (_, p) => Some(format!("{base}{}", p.unwrap_or_default())),
    }
}

/// `rPr`'s text colour: the value, or `theme:…` for a theme colour;
/// `Some(None)` for `auto`, which resets an inherited one.
fn color(dom: &Dom, rpr: NodeId) -> Option<Option<String>> {
    let c = child(dom, rpr, "color")?;
    let theme = attr(dom, c, "themeColor");
    if !theme.is_empty() {
        // Tint and shade change the ink the converter paints.
        let mut out = format!("theme:{theme}");
        for (name, label) in [("themeTint", "tint"), ("themeShade", "shade")] {
            let v = attr(dom, c, name);
            if !v.is_empty() {
                out.push_str(&format!("/{label}:{v}"));
            }
        }
        return Some(Some(out));
    }
    let val = attr(dom, c, "val");
    Some((!val.is_empty() && val != "auto").then_some(val))
}

/// `rPr`'s highlight; `Some(None)` for `none`, a reset.
fn highlight(dom: &Dom, rpr: NodeId) -> Option<Option<String>> {
    let val = attr(dom, child(dom, rpr, "highlight")?, "val");
    Some((!val.is_empty() && val != "none").then_some(val))
}

/// `rPr`'s shading; `Some(None)` for one that paints nothing (nil), a reset.
fn run_shd(dom: &Dom, rpr: NodeId) -> Option<Option<String>> {
    child(dom, rpr, "shd").map(|x| shd(dom, x))
}

/// A toggle property that is on (`w:val` absent, `1`, `true` or `on`).
fn on(dom: &Dom, pr: NodeId, name: &str) -> bool {
    child(dom, pr, name)
        .is_some_and(|c| !matches!(attr(dom, c, "val").as_str(), "0" | "false" | "off"))
}

/// Text `n` paints in its own paragraph: `w:t` and `w:delText` whose
/// nearest paragraph is `para` (a text box's paragraphs are their own).
fn own_text(dom: &Dom, n: NodeId, para: NodeId) -> String {
    let mut s = String::new();
    for t in dom.descendants(n, None) {
        if matches!(local(dom, t).as_str(), "t" | "delText") && nearest(dom, t, "p") == Some(para) {
            for c in dom.nodes(t) {
                s.push_str(dom.text_value(c).unwrap_or_default());
            }
        }
    }
    s
}

/// Counts with the first sample seen, by line key.
#[derive(Default)]
struct Tally(BTreeMap<String, (usize, String)>);

impl Tally {
    fn add(&mut self, key: String, sample: &str) {
        let e = self.0.entry(key).or_default();
        if e.0 == 0 {
            e.1 = clip(sample, SAMPLE);
        }
        e.0 += 1;
    }

    fn lines(self, out: &mut Vec<String>) {
        for (key, (n, sample)) in self.0 {
            if sample.is_empty() {
                out.push(format!("  {key} ×{n}"));
            } else {
                out.push(format!("  {key} ×{n} \"{sample}\""));
            }
        }
    }
}

/// One story part's lines: sections, tables, frames and drawings in
/// document order, then the tallies. Returns the paragraph count.
fn part_lines(
    dom: &Dom,
    root: NodeId,
    styles: &StyleBook,
    rels: &[Rel],
    out: &mut Vec<String>,
) -> usize {
    let mut tally = Tally::default();
    let mut paras = 0;
    let mut tables = 0;
    let mut sections = 0;
    // Field codes being collected, innermost last: (code, recorded).
    let mut fields: Vec<(String, bool)> = Vec::new();
    for n in dom.descendants(root, None) {
        // Word renders an mc:AlternateContent's Choice; its Fallback is a
        // copy of the same content.
        if nearest(dom, n, "Fallback").is_some() {
            continue;
        }
        match local(dom, n).as_str() {
            "p" => {
                paras += 1;
                paragraph(dom, n, styles, &mut tally, out);
            }
            "r" => run(dom, n, styles, &mut tally),
            "tbl" => {
                tables += 1;
                out.push(table(dom, n, tables, styles));
            }
            // A `w:sectPrChange`'s old `w:sectPr` is history.
            "sectPr" if nearest(dom, n, "sectPrChange").is_none() => {
                sections += 1;
                out.push(section(dom, n, sections, rels));
            }
            "anchor" => out.push(anchor(dom, n)),
            "inline" => tally.add("inline".to_string(), ""),
            "pict" | "object" => tally.add(format!("vml {}", local(dom, n)), ""),
            "AlternateContent" => tally.add("AlternateContent".to_string(), ""),
            "fldSimple" => {
                let code = attr(dom, n, "instr");
                if let Some(w) = code.split_whitespace().next() {
                    tally.add(format!("field {}", w.to_uppercase()), "");
                }
            }
            "fldChar" => match attr(dom, n, "fldCharType").as_str() {
                "begin" => fields.push((String::new(), false)),
                kind => {
                    if let Some(top) = fields.last_mut()
                        && !top.1
                    {
                        top.1 = true;
                        if let Some(w) = top.0.split_whitespace().next() {
                            tally.add(format!("field {}", w.to_uppercase()), "");
                        }
                    }
                    if kind == "end" {
                        fields.pop();
                    }
                }
            },
            "instrText" | "delInstrText" => {
                if let Some(top) = fields.last_mut() {
                    for c in dom.nodes(n) {
                        top.0.push_str(dom.text_value(c).unwrap_or_default());
                    }
                }
            }
            _ => {}
        }
    }
    tally.lines(out);
    paras
}

/// A paragraph's style, shading, frame and the order of its revisions.
fn paragraph(dom: &Dom, p: NodeId, styles: &StyleBook, tally: &mut Tally, out: &mut Vec<String>) {
    let text = own_text(dom, p, p);
    let ppr = child(dom, p, "pPr");
    let named = ppr
        .and_then(|x| child(dom, x, "pStyle"))
        .map(|s| attr(dom, s, "val"))
        .unwrap_or_default();
    if !named.is_empty() {
        tally.add(format!("pstyle \"{}\"", styles.name(&named)), "");
    }
    let pstyle = styles.para_style(named);
    match ppr.and_then(|x| child(dom, x, "shd")).map(|x| shd(dom, x)) {
        Some(Some(fill)) => tally.add(format!("para-shd {fill}"), &text),
        Some(None) => {}
        None => match styles.find(&pstyle, |s| s.para_shd.clone()) {
            Some((Some(fill), name)) => tally.add(format!("para-shd {fill} via \"{name}\""), &text),
            Some((None, _)) => {}
            None => {
                if let Some(Some(fill)) = &styles.defaults.para_shd {
                    tally.add(format!("para-shd {fill} via \"docDefaults\""), &text);
                }
            }
        },
    }
    // The style chain's frame under the paragraph's own attributes, as
    // the converter merges them.
    let mut frame = styles.frame(&pstyle);
    if let Some(own) = ppr.and_then(|x| child(dom, x, "framePr")) {
        merge_pairs(&mut frame, &attr_pairs(dom, own));
    }
    if !frame.is_empty() {
        let mut items: Vec<String> = frame.iter().map(|(k, v)| format!("{k}={v}")).collect();
        items.sort();
        out.push(format!(
            "  frame({}) \"{}\"",
            items.join(","),
            clip(&text, SAMPLE)
        ));
    }
    // Revision containers side by side, a plain run between them breaking
    // the pair.
    let mut order: Vec<char> = Vec::new();
    revision_order(dom, p, &mut order);
    for &[a, b] in order.array_windows() {
        match (a, b) {
            ('D', 'I') => tally.add("revisions del→ins".to_string(), &text),
            ('I', 'D') => tally.add("revisions ins→del".to_string(), &text),
            _ => {}
        }
    }
}

/// The revision containers under `n` in document order, looking through
/// hyperlinks, smart tags, content controls and customXml (their runs are
/// the paragraph's own).
fn revision_order(dom: &Dom, n: NodeId, order: &mut Vec<char>) {
    for c in dom.elements(n, None) {
        let k = match local(dom, c).as_str() {
            "del" | "moveFrom" => 'D',
            "ins" | "moveTo" => 'I',
            "fldSimple" => '-',
            "r" if paints(dom, c) => '-',
            "hyperlink" | "smartTag" | "sdt" | "sdtContent" | "customXml" => {
                revision_order(dom, c, order);
                continue;
            }
            _ => continue,
        };
        if order.last() != Some(&k) {
            order.push(k);
        }
    }
}

/// A run that puts something on the page: text, a tab or break, a symbol,
/// a field character or a drawing (anything but its properties, a
/// rendered-page-break hint, an empty `w:t` or a comment reference).
fn paints(dom: &Dom, r: NodeId) -> bool {
    dom.elements(r, None)
        .into_iter()
        .any(|c| match local(dom, c).as_str() {
            "rPr" | "lastRenderedPageBreak" | "commentReference" => false,
            "t" | "delText" => dom
                .nodes(c)
                .into_iter()
                .any(|t| !dom.text_value(t).unwrap_or_default().is_empty()),
            _ => true,
        })
}

/// A run's colour, highlight, shading, hidden state and requested font,
/// direct or from its character or paragraph style, over `docDefaults`.
/// The leading `w:rPr` blocks apply in order, each its character style and
/// then its own properties, as the converter folds them (an `rPr` after the
/// run's content is ignored).
fn run(dom: &Dom, r: NodeId, styles: &StyleBook, tally: &mut Tally) {
    let Some(para) = nearest(dom, r, "p") else {
        return;
    };
    let text = own_text(dom, r, para);
    let rprs = crate::convert::leading_rprs(dom, r);
    let style_of = |pr: Option<NodeId>, name: &str| {
        pr.and_then(|x| child(dom, x, name))
            .map(|s| attr(dom, s, "val"))
            .unwrap_or_default()
    };
    let pstyle = styles.para_style(style_of(child(dom, para, "pPr"), "pStyle"));
    // The value that reaches the page and the style it comes through
    // (`None` for direct formatting).
    type Layer = Option<(String, Option<String>)>;
    let fold = |get: &dyn Fn(NodeId) -> Option<Option<String>>,
                from_style: &dyn Fn(&StyleInfo) -> Option<Option<String>>|
     -> Layer {
        let mut cur: Layer = match styles.find(&pstyle, from_style) {
            Some((v, name)) => v.map(|v| (v, Some(name))),
            None => from_style(&styles.defaults)
                .flatten()
                .map(|v| (v, Some(styles.defaults.name.clone()))),
        };
        for &rpr in &rprs {
            let rstyle = style_of(Some(rpr), "rStyle");
            if let Some((v, name)) = styles.find(&rstyle, from_style) {
                cur = v.map(|v| (v, Some(name)));
            }
            if let Some(v) = get(rpr) {
                cur = v.map(|v| (v, None));
            }
        }
        cur
    };
    let mut add = |label: &str, layer: Layer| match layer {
        Some((v, None)) => tally.add(format!("{label} {v}"), &text),
        Some((v, Some(name))) => tally.add(format!("{label} {v} via \"{name}\""), &text),
        None => {}
    };
    add("color", fold(&|x| color(dom, x), &|s| s.color.clone()));
    add(
        "highlight",
        fold(&|x| highlight(dom, x), &|s| s.highlight.clone()),
    );
    add(
        "run-shd",
        fold(&|x| run_shd(dom, x), &|s| s.run_shd.clone()),
    );
    // Hidden the same way, a style's `vanish` standing in for a value.
    let as_value = |v: bool| v.then(|| "vanish".to_string());
    if let Some((_, via)) = fold(
        &|x| child(dom, x, "vanish").map(|_| as_value(on(dom, x, "vanish"))),
        &|s| s.vanish.map(as_value),
    ) {
        match via {
            None => tally.add("vanish".to_string(), &text),
            Some(name) => tally.add(format!("vanish via \"{name}\""), &text),
        }
    }
    // Fonts slot by slot: the paragraph style's chain, then each block's
    // character style and its own `w:rFonts`.
    let mut slots: Vec<(&'static str, String, Option<String>)> = styles
        .font_slots(&pstyle)
        .into_iter()
        .map(|(k, f, n)| (k, f, Some(n)))
        .collect();
    let overlay = |slots: &mut Vec<(&'static str, String, Option<String>)>,
                   new: Vec<(&'static str, String, Option<String>)>| {
        for (k, f, n) in new {
            slots.retain(|(s, _, _)| *s != k);
            slots.push((k, f, n));
        }
    };
    for &rpr in &rprs {
        let rstyle = style_of(Some(rpr), "rStyle");
        if !rstyle.is_empty() {
            let from = styles
                .font_slots(&rstyle)
                .into_iter()
                .map(|(k, f, n)| (k, f, Some(n)))
                .collect();
            overlay(&mut slots, from);
        }
        if let Some(f) = child(dom, rpr, "rFonts") {
            let direct = font_slots(dom, f)
                .into_iter()
                .map(|(k, face)| (k, face, None))
                .collect();
            overlay(&mut slots, direct);
        }
    }
    let mut seen: Vec<(String, Option<String>)> = Vec::new();
    for (_, face, via) in slots {
        if seen.iter().any(|(f, v)| *f == face && *v == via) {
            continue;
        }
        match &via {
            None => tally.add(format!("rfonts \"{face}\""), ""),
            Some(name) => tally.add(format!("rfonts \"{face}\" via \"{name}\""), ""),
        }
        seen.push((face, via));
    }
}

/// Each slot's face in `w:rFonts` (a theme font where the slot names
/// none): a run can paint Latin, East Asian and complex-script text in
/// different faces.
fn font_slots(dom: &Dom, f: NodeId) -> Vec<(&'static str, String)> {
    let mut faces: Vec<(&'static str, String)> = Vec::new();
    for (slot, theme) in [
        ("ascii", "asciiTheme"),
        ("hAnsi", "hAnsiTheme"),
        ("eastAsia", "eastAsiaTheme"),
        ("cs", "cstheme"),
    ] {
        let (face, theme) = (attr(dom, f, slot), attr(dom, f, theme));
        let face = if !face.is_empty() {
            face
        } else if !theme.is_empty() {
            format!("theme:{theme}")
        } else {
            continue;
        };
        faces.push((slot, face));
    }
    faces
}

/// An element's attributes as `(local name, value)`.
fn attr_pairs(dom: &Dom, n: NodeId) -> Vec<(String, String)> {
    dom.attributes(n)
        .into_iter()
        .map(|(k, v)| (k.local_name().to_string(), v.to_string()))
        .collect()
}

/// `over`'s values replace or join `base`'s.
fn merge_pairs(base: &mut Vec<(String, String)>, over: &[(String, String)]) {
    for (k, v) in over {
        match base.iter_mut().find(|(bk, _)| bk == k) {
            Some(slot) => slot.1.clone_from(v),
            None => base.push((k.clone(), v.clone())),
        }
    }
}

/// `table N RxC style=… float(…) shd=… cells-shd[…] style-shd[…] "first row"`.
fn table(dom: &Dom, t: NodeId, n: usize, styles: &StyleBook) -> String {
    let tblpr = child(dom, t, "tblPr");
    // customXml-wrapped rows are the table's own, as the converter lays
    // them out.
    let rows: Vec<NodeId> = dom
        .elements(t, None)
        .into_iter()
        .flat_map(|c| match local(dom, c).as_str() {
            "tr" => vec![c],
            "customXml" => kids(dom, c, "tr"),
            _ => Vec::new(),
        })
        .collect();
    // The columns the converter lays out: the grid, or the widest row
    // (its spans, grid before/after and, with a live `w:cellDel`, the
    // "Deleted Cells" column Word's All Markup appends).
    let grid = child(dom, t, "tblGrid").map_or(0, |g| kids(dom, g, "gridCol").len());
    let widest = rows
        .iter()
        .map(|&row| {
            let trpr = child(dom, row, "trPr");
            let skip = |name: &str| {
                trpr.and_then(|p| child(dom, p, name))
                    .and_then(|g| attr(dom, g, "val").parse::<usize>().ok())
                    .unwrap_or(0)
            };
            let cells = crate::convert::wrapped_children(dom, row, "tc");
            let spans: usize = cells
                .iter()
                .map(|&tc| {
                    child(dom, tc, "tcPr")
                        .and_then(|p| child(dom, p, "gridSpan"))
                        .and_then(|g| attr(dom, g, "val").parse::<usize>().ok())
                        .unwrap_or(1)
                        .max(1)
                })
                .sum();
            let stamp = cells
                .iter()
                .any(|&tc| crate::convert::cell_is_deleted(dom, tc));
            skip("gridBefore") + spans + skip("gridAfter") + usize::from(stamp)
        })
        .max()
        .unwrap_or(0);
    let cols = grid.max(widest);
    let mut s = format!("  table {n} {}x{cols}", rows.len());
    if nearest(dom, t, "tbl").is_some() {
        s.push_str(" nested");
    }
    if nearest(dom, t, "txbxContent").is_some() {
        s.push_str(" in-textbox");
    }
    let style = tblpr
        .and_then(|x| child(dom, x, "tblStyle"))
        .map(|x| attr(dom, x, "val"))
        .unwrap_or_default();
    if !style.is_empty() {
        s.push_str(&format!(" style=\"{}\"", styles.name(&style)));
    }
    if let Some(float) = tblpr.and_then(|x| child(dom, x, "tblpPr")) {
        let mut items: Vec<String> = dom
            .attributes(float)
            .into_iter()
            .map(|(k, v)| format!("{}={v}", k.local_name()))
            .collect();
        items.sort();
        s.push_str(&format!(" float({})", items.join(",")));
    }
    if let Some(fill) = tblpr
        .and_then(|x| child(dom, x, "shd"))
        .and_then(|x| shd(dom, x))
    {
        s.push_str(&format!(" shd={fill}"));
    }
    let mut cells: BTreeMap<String, usize> = BTreeMap::new();
    for tc in dom.descendants(t, None) {
        if local(dom, tc) != "tc" || nearest(dom, tc, "tbl") != Some(t) {
            continue;
        }
        if let Some(fill) = child(dom, tc, "tcPr")
            .and_then(|x| child(dom, x, "shd"))
            .and_then(|x| shd(dom, x))
        {
            *cells.entry(fill).or_default() += 1;
        }
    }
    if !cells.is_empty() {
        let items: Vec<String> = cells.iter().map(|(f, k)| format!("{f}×{k}")).collect();
        s.push_str(&format!(" cells-shd[{}]", items.join(",")));
    }
    // The conditions the table's look turns off paint nothing (the
    // converter's own reading of `w:tblLook`).
    let look = crate::convert::table_look(dom, t);
    let conds: Vec<(String, String)> = styles
        .table_shd(&style)
        .into_iter()
        .filter(|(k, _)| match k.as_str() {
            "firstRow" => look.first_row,
            "firstCol" => look.first_col,
            "band1Horz" | "band2Horz" => !look.no_h_band,
            _ => true,
        })
        .collect();
    if !conds.is_empty() {
        let items: Vec<String> = conds.iter().map(|(k, f)| format!("{k}={f}")).collect();
        s.push_str(&format!(" style-shd[{}]", items.join(",")));
    }
    if let Some(first) = rows.first() {
        let texts: Vec<String> = crate::convert::wrapped_children(dom, *first, "tc")
            .into_iter()
            .map(|tc| {
                kids(dom, tc, "p")
                    .into_iter()
                    .map(|p| own_text(dom, p, p))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        s.push_str(&format!(" \"{}\"", clip(&texts.join(" | "), FIRST_ROW)));
    }
    s
}

/// `section N page=WxH margins=t,r,b,l header=… footer=… cols=… TYPE-KIND=part`.
fn section(dom: &Dom, sect: NodeId, n: usize, rels: &[Rel]) -> String {
    let mut s = format!("  section {n}");
    if let Some(sz) = child(dom, sect, "pgSz") {
        s.push_str(&format!(
            " page={}x{}",
            attr(dom, sz, "w"),
            attr(dom, sz, "h")
        ));
        let orient = attr(dom, sz, "orient");
        if !orient.is_empty() {
            s.push_str(&format!(" {orient}"));
        }
    }
    if let Some(m) = child(dom, sect, "pgMar") {
        let a = |k: &str| attr(dom, m, k);
        s.push_str(&format!(
            " margins={},{},{},{} header={} footer={}",
            a("top"),
            a("right"),
            a("bottom"),
            a("left"),
            a("header"),
            a("footer")
        ));
    }
    if let Some(c) = child(dom, sect, "cols") {
        // Unequal columns: each width and the gap after it.
        let custom: Vec<String> = kids(dom, c, "col")
            .into_iter()
            .map(|col| {
                let gap = attr(dom, col, "space");
                if gap.is_empty() {
                    attr(dom, col, "w")
                } else {
                    format!("{}+{gap}", attr(dom, col, "w"))
                }
            })
            .collect();
        let unequal = matches!(attr(dom, c, "equalWidth").as_str(), "0" | "false");
        let num = attr(dom, c, "num");
        if unequal && !custom.is_empty() {
            s.push_str(&format!(" cols={}[{}]", custom.len(), custom.join(",")));
        } else if !num.is_empty() && num != "1" {
            s.push_str(&format!(" cols={num}"));
        }
    }
    if let Some(t) = child(dom, sect, "type") {
        s.push_str(&format!(" type={}", attr(dom, t, "val")));
    }
    if on(dom, sect, "titlePg") {
        s.push_str(" titlePg");
    }
    for r in dom.elements(sect, None) {
        let kind = match local(dom, r).as_str() {
            "headerReference" => "header",
            "footerReference" => "footer",
            _ => continue,
        };
        let ty = attr(dom, r, "type");
        let ty = if ty.is_empty() {
            "default".to_string()
        } else {
            ty
        };
        let id = attr(dom, r, "id");
        let part = rels
            .iter()
            .find(|x| x.id == id)
            .map_or_else(|| format!("{id}?"), |x| x.part.clone());
        s.push_str(&format!(" {ty}-{kind}={part}"));
    }
    s
}

/// `anchor wrap H=from:offset V=from:offset CXxCY [behind] "name"`.
fn anchor(dom: &Dom, a: NodeId) -> String {
    let wrap = dom
        .elements(a, None)
        .into_iter()
        .map(|c| local(dom, c))
        .find(|l| l.starts_with("wrap"))
        .unwrap_or_default();
    let pos = |name: &str| {
        child(dom, a, name).map_or_else(String::new, |p| {
            let at = dom
                .elements(p, None)
                .into_iter()
                .map(|c| {
                    let v: String = dom
                        .nodes(c)
                        .into_iter()
                        .filter_map(|t| dom.text_value(t).map(str::to_string))
                        .collect();
                    if local(dom, c) == "align" {
                        v
                    } else {
                        v.trim().to_string()
                    }
                })
                .next()
                .unwrap_or_default();
            format!("{}:{at}", attr(dom, p, "relativeFrom"))
        })
    };
    let size = child(dom, a, "extent")
        .map(|e| format!("{}x{}", attr(dom, e, "cx"), attr(dom, e, "cy")))
        .unwrap_or_default();
    let behind = if matches!(attr(dom, a, "behindDoc").as_str(), "1" | "true") {
        " behind"
    } else {
        ""
    };
    let name = child(dom, a, "docPr")
        .map(|d| attr(dom, d, "name"))
        .unwrap_or_default();
    format!(
        "  anchor {wrap} H={} V={} {size}{behind} \"{name}\"",
        pos("positionH"),
        pos("positionV")
    )
}

/// `(layout)`: jubarte's page count and font resolution (Word revision
/// style, as the bench renders it), then the fonts the package asks for:
/// `docDefaults`, the theme and `fontTable.xml`.
fn layout_lines(pkg: &Package, rels: &[Rel]) -> (Vec<String>, usize) {
    let mut out = Vec::new();
    let options = crate::convert::PdfOptions {
        revisions: crate::convert::RevisionStyle::Word,
        ..Default::default()
    };
    let pages = match crate::convert::docx_render_report(&pkg.raw, options) {
        Ok(report) => {
            let mut fonts: Vec<String> = report
                .fonts
                .iter()
                .map(|f| {
                    let style = match (f.bold, f.italic) {
                        (false, false) => "regular",
                        (true, false) => "bold",
                        (false, true) => "italic",
                        (true, true) => "bold-italic",
                    };
                    format!(
                        "  font \"{}\" {style} → {} {}{}",
                        f.requested,
                        f.physical,
                        f.step,
                        if f.synthetic { " synthetic" } else { "" }
                    )
                })
                .collect();
            fonts.sort();
            out.push(format!("  pages {}", report.page_count));
            out.extend(fonts);
            report.page_count
        }
        Err(e) => {
            out.push(format!("  layout failed: {e}"));
            0
        }
    };
    if let Some(d) = rels
        .iter()
        .find(|r| r.kind == "styles")
        .and_then(|r| parse(pkg, &r.part))
        .and_then(|(dom, root)| StyleBook::new(&dom, root).default_fonts)
    {
        out.push(format!("  docDefaults rfonts {d}"));
    }
    if let Some((dom, root)) = rels
        .iter()
        .find(|r| r.kind == "theme")
        .and_then(|r| parse(pkg, &r.part))
    {
        for (slot, label) in [("majorFont", "major"), ("minorFont", "minor")] {
            let face = descendant(&dom, root, slot)
                .and_then(|f| child(&dom, f, "latin"))
                .map(|l| attr(&dom, l, "typeface"));
            if let Some(face) = face {
                out.push(format!("  theme {label} latin=\"{face}\""));
            }
        }
    }
    if let Some((dom, root)) = rels
        .iter()
        .find(|r| r.kind == "fontTable")
        .and_then(|r| parse(pkg, &r.part))
    {
        for f in kids(&dom, root, "font") {
            let mut s = format!("  fontTable \"{}\"", attr(&dom, f, "name"));
            if let Some(alt) = child(&dom, f, "altName") {
                s.push_str(&format!(" alt=\"{}\"", attr(&dom, alt, "val")));
            }
            let embedded: Vec<String> = dom
                .elements(f, None)
                .into_iter()
                .map(|c| local(&dom, c))
                .filter_map(|l| l.strip_prefix("embed").map(str::to_lowercase))
                .collect();
            if !embedded.is_empty() {
                s.push_str(&format!(" embedded({})", embedded.join(",")));
            }
            out.push(s);
        }
    }
    (out, pages)
}
