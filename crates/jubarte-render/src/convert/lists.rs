// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! List numbering: the numbering part, level markers and list labels.

use super::*;

#[derive(Clone)]
pub(super) struct NumLevel {
    pub(super) fmt: NumFmt,
    pub(super) text: String,
    pub(super) start: u32,
    pub(super) left: f32,
    pub(super) hanging: f32,
    /// lvl rPr ascii/hAnsi (ListBullet is Symbol).
    pub(super) family: String,
    /// `w:suff=nothing` (sd_2517 Título1 `Article %2`). Default is a
    /// trailing space so existing markers stay `1. `.
    pub(super) suff_nothing: bool,
    /// `w:lvlJc=right`: marker right edge at hanging end (body start).
    pub(super) jc_right: bool,
    /// The level's `w:pPr/w:jc`: the paragraph's alignment when its numPr
    /// is direct (8aea3634's centred heading 1 items sit left).
    pub(super) jc: Option<Align>,
    /// Numbering `w:tabs` (`val=num` is a left stop at hanging indent).
    pub(super) tab_stops: Vec<TabStop>,
    /// lvl `w:rPr/w:sz` in pt. None = inherit the paragraph run.
    pub(super) size: Option<f32>,
    /// lvl `w:rPr/w:u` other than `val=none`.
    pub(super) underline: bool,
    pub(super) bold: bool,
    pub(super) italic: bool,
}

#[derive(Default, Clone)]
pub(super) struct Numbering {
    pub(super) instances: HashMap<String, String>,
    pub(super) levels: HashMap<String, HashMap<u32, NumLevel>>,
    pub(super) counters: HashMap<(String, u32), u32>,
    /// `w:num/w:lvlOverride/w:startOverride` (or nested `lvl/start`).
    pub(super) starts: HashMap<(String, u32), u32>,
    /// `w:lvlRestart/@w:val` keyed by (abstractNumId, ilvl). Missing is
    /// Word's default (restart after any shallower level). 0 = never.
    pub(super) restarts: HashMap<(String, u32), u32>,
    /// `w:isLgl` on a level: every `%n` slot paints as Arabic.
    pub(super) is_lgl: HashSet<(String, u32)>,
    /// `(numId, ilvl)` → isLgl of a `w:lvlOverride/w:lvl` replacement.
    pub(super) lgl_override: HashMap<(String, u32), bool>,
    /// `w:numPicBullet/@w:numPicBulletId` → image bytes.
    pub(super) pic_bytes: HashMap<u32, Vec<u8>>,
    /// `w:numPicBullet` `v:shape` width/height (pt), when given.
    pub(super) pic_extent: HashMap<u32, (f32, f32)>,
    /// `(abstractNumId, ilvl)` → `lvlPicBulletId`.
    pub(super) lvl_pic: HashMap<(String, u32), u32>,
    /// `w:lvl/w:pStyle` keyed by (abstractNumId, ilvl): the style the
    /// level belongs to.
    pub(super) lvl_pstyle: HashMap<(String, u32), String>,
    /// The original document's lists, forked at the first revised
    /// paragraph mark: they skip inserted marks as these skip deleted ones.
    orig: Option<Box<Numbering>>,
    /// The original number of the paragraph just numbered, when it differs
    /// from its revised one (Word shows "3.1.").
    pub(super) old_label: Option<String>,
}

impl Numbering {
    /// The style `(num_id, ilvl)`'s level is linked to, if any.
    pub(super) fn linked_style(&self, num_id: &str, ilvl: u32) -> Option<&str> {
        let abs = self.instances.get(num_id)?;
        let resolved = self.resolve_ilvl(abs, ilvl);
        self.lvl_pstyle
            .get(&(abs.clone(), resolved))
            .map(String::as_str)
    }

    pub(super) fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32 {
        let Some(lvls) = self.levels.get(abs) else {
            return ilvl;
        };
        if lvls.contains_key(&ilvl) {
            return ilvl;
        }
        // Word ignores a requested ilvl the abstract never defined
        // (potpourri / file_170 ListNumber only has ilvl=0). Continue
        // the parent level so Sift is "3." at the same hanging as
        // Preheat, not a synthesized nested "1." at +18pt.
        (0..ilvl)
            .rev()
            .find(|i| lvls.contains_key(i))
            .unwrap_or(ilvl)
    }

    pub(super) fn level(&self, num_id: &str, ilvl: u32) -> Option<&NumLevel> {
        let abs = self.instances.get(num_id)?;
        let resolved = self.resolve_ilvl(abs, ilvl);
        self.levels.get(abs)?.get(&resolved)
    }

    /// Missing numbered levels inherit the parent's fmt and add 360
    /// twips of indent per step when a deeper lvl is actually defined
    /// later. Callers that should follow Word's "undefined ilvl
    /// continues the parent" rule must `resolve_ilvl` first.
    pub(super) fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel> {
        if let Some(lvl) = self.levels.get(abs).and_then(|m| m.get(&ilvl)).cloned() {
            return Some(lvl);
        }
        let parent_ilvl = (0..ilvl)
            .rev()
            .find(|&i| self.levels.get(abs).is_some_and(|m| m.contains_key(&i)))?;
        let parent = self.levels.get(abs)?.get(&parent_ilvl)?.clone();
        if parent.fmt == NumFmt::Bullet {
            return None;
        }
        let step = (ilvl - parent_ilvl) as f32;
        let synth = NumLevel {
            fmt: NumFmt::Decimal,
            text: format!("%{}.", ilvl + 1),
            start: 1,
            left: parent.left + twip(360.0) * step,
            hanging: if parent.hanging > 0.0 {
                parent.hanging
            } else {
                twip(360.0)
            },
            family: parent.family,
            suff_nothing: parent.suff_nothing,
            jc_right: parent.jc_right,
            jc: parent.jc,
            tab_stops: parent.tab_stops,
            size: parent.size,
            underline: parent.underline,
            bold: parent.bold,
            italic: parent.italic,
        };
        self.levels
            .entry(abs.to_string())
            .or_default()
            .insert(ilvl, synth.clone());
        Some(synth)
    }

    pub(super) fn next_marker(&mut self, num_id: &str, ilvl: u32) -> String {
        let Some(abs) = self.instances.get(num_id).cloned() else {
            return String::new();
        };
        let resolved = self.resolve_ilvl(&abs, ilvl);
        let Some(lvl) = self.ensure_level(&abs, resolved) else {
            return String::new();
        };
        // Incrementing level `resolved` restarts each deeper level L unless
        // its lvlRestart says otherwise: 0 never restarts; a valid 1-based
        // n (1..=L) restarts only when a level at or above n-1 increments;
        // anything else is the default (restart on any shallower level).
        self.counters.retain(|(id, level), _| {
            if id != num_id || *level <= resolved {
                return true;
            }
            match self.restarts.get(&(abs.clone(), *level)) {
                Some(0) => true,
                Some(&n) if (1..=*level).contains(&n) => resolved + 1 > n,
                _ => false,
            }
        });
        // A deeper item ahead of any item of a shallower level starts that
        // level at its start value and uses it up: ed36b607e8's list (lvl 0
        // start 3) opens with 3.1, 3.2, 3.3 and Word numbers the first
        // lvl-0 paragraph after them 4, the next 5; the engine gave it 3.
        for level in 0..resolved {
            if self.counters.contains_key(&(num_id.to_string(), level)) {
                continue;
            }
            let first = self
                .starts
                .get(&(num_id.to_string(), level))
                .copied()
                .or_else(|| {
                    self.levels
                        .get(&abs)
                        .and_then(|m| m.get(&level))
                        .map(|l| l.start)
                })
                .unwrap_or(1)
                .max(1);
            self.counters
                .insert((num_id.to_string(), level), first.saturating_add(1));
        }
        let start = self
            .starts
            .get(&(num_id.to_string(), resolved))
            .copied()
            .unwrap_or(lvl.start)
            .max(1);
        let cur = *self
            .counters
            .entry((num_id.to_string(), resolved))
            .or_insert(start);
        self.counters
            .insert((num_id.to_string(), resolved), cur.saturating_add(1));
        self.render(&abs, num_id, resolved, &lvl, cur)
    }

    /// Word numbers a revised list twice (probe lbl0930): the revised
    /// document skips deleted paragraph marks, the original one inserted
    /// marks. A deleted mark shows its original number, an inserted one its
    /// revised number; any other paragraph its revised number, with the
    /// original kept in `old_label` when the two differ.
    fn next_revised(&mut self, num_id: &str, ilvl: u32, mark: RevMark) -> String {
        self.old_label = None;
        match mark {
            RevMark::Del | RevMark::MoveFrom => self.original().next_marker(num_id, ilvl),
            RevMark::Ins | RevMark::MoveTo => {
                self.original();
                self.next_marker(num_id, ilvl)
            }
            RevMark::None => {
                let label = self.next_marker(num_id, ilvl);
                if let Some(orig) = self.orig.as_deref_mut() {
                    let old = orig.next_marker(num_id, ilvl);
                    self.old_label = (old != label).then_some(old);
                }
                label
            }
        }
    }

    /// The original document's lists; until the first revised mark they
    /// count as these do.
    fn original(&mut self) -> &mut Numbering {
        let orig = match self.orig.take() {
            Some(orig) => orig,
            None => Box::new(self.clone()),
        };
        self.orig.insert(orig)
    }

    pub(super) fn last_used(&self, num_id: &str, ilvl: u32) -> Option<u32> {
        self.counters
            .get(&(num_id.to_string(), ilvl))
            .map(|v| v.saturating_sub(1).max(1))
    }

    /// The picture bullet's own extent (pt), from its `v:shape` style.
    pub(super) fn pic_extent(&self, num_id: &str, ilvl: u32) -> Option<(f32, f32)> {
        let abs = self.instances.get(num_id)?;
        let resolved = self.resolve_ilvl(abs, ilvl);
        let pic_id = self.lvl_pic.get(&(abs.clone(), resolved))?;
        self.pic_extent.get(pic_id).copied()
    }

    pub(super) fn pic_image(&self, num_id: &str, ilvl: u32) -> Option<ImageKind> {
        let abs = self.instances.get(num_id)?;
        let resolved = self.resolve_ilvl(abs, ilvl);
        let pic_id = *self.lvl_pic.get(&(abs.clone(), resolved))?;
        let bytes = self.pic_bytes.get(&pic_id)?.clone();
        decode_image(bytes)
    }

    pub(super) fn render(
        &self,
        abs: &str,
        num_id: &str,
        ilvl: u32,
        lvl: &NumLevel,
        this: u32,
    ) -> String {
        if names_deeper_level(&lvl.text, ilvl) {
            return String::new();
        }
        if lvl.fmt == NumFmt::Bullet {
            // Word ListBullet is U+F0B7 in Symbol (cmap has F0B7/00B7, not
            // U+2022). Mapping PUA→U+2022 painted Aptos 0x95 and skipped
            // SymbolMT (empty glyphs). Keep the PUA on Symbol so Quartz
            // • matches comments-lots; hanging indent already places it.
            if lvl.family.to_ascii_lowercase().contains("symbol") {
                // Word paints PUA • at 72 then an Arial space at ~77.5
                // before the hanging body at 90 (potpourri ListBullet).
                // Keep the PUA (mini 108 U+00B7 was ITT-wrong); append
                // the gutter space non-Symbol bullets already have.
                let t = lvl.text.trim();
                let mark = if t.is_empty() { "\u{F0B7}" } else { t };
                return if mark.ends_with(' ') {
                    mark.to_string()
                } else {
                    format!("{mark} ")
                };
            }
            return bullet_glyph(&lvl.text);
        }
        let mut out = lvl.text.clone();
        for i in 0..=ilvl {
            let token = format!("%{}", i + 1);
            if !out.contains(&token) {
                continue;
            }
            let val = if i == ilvl {
                this
            } else {
                self.counters
                    .get(&(num_id.to_string(), i))
                    .map(|v| (*v).saturating_sub(1).max(1))
                    // An unused parent slot shows this instance's
                    // startOverride before the abstract start.
                    .or_else(|| self.starts.get(&(num_id.to_string(), i)).copied())
                    .or_else(|| {
                        self.levels
                            .get(abs)
                            .and_then(|m| m.get(&i))
                            .map(|l| l.start.max(1))
                    })
                    .unwrap_or(1)
            };
            let fmt = self
                .levels
                .get(abs)
                .and_then(|m| m.get(&i))
                .map_or(lvl.fmt, |l| l.fmt);
            // Word `Section 1.01`: decimalZero lvlText uses decimal for
            // parent slots, not the parent's cardinalText (`Article One`).
            // w:isLgl on this level forces every slot to Arabic (I.1 → 1.1).
            let lgl = self
                .lgl_override
                .get(&(num_id.to_string(), ilvl))
                .copied()
                .unwrap_or_else(|| self.is_lgl.contains(&(abs.to_string(), ilvl)));
            let fmt = if lgl || (lvl.fmt == NumFmt::DecimalZero && i != ilvl) {
                NumFmt::Decimal
            } else {
                fmt
            };
            out = out.replace(&token, &format_num(fmt, val));
        }
        if !lvl.suff_nothing && !out.ends_with(' ') && !out.ends_with('\t') {
            // Default `w:suff` is tab, with or without a numbering stop:
            // 0020e409's "1." goes to the paragraph's 851-twip stop, and a
            // space there broke 000eb113 / 0004c94c's lines early (fixtures_500
            // 22 up, 0 down; the old sd_2517 space rule predates them).
            out.push('\t');
        }
        out
    }
}

pub(super) fn bullet_glyph(raw: &str) -> String {
    // Word ListBullet is U+F0B7 + Symbol (cmap U+00B7). Mapping PUA to
    // U+00B7 (mini 108) put the real bullet at x=72, but ITT dropped the
    // comments-lots family ~0.008–0.016 and potpourri only +0.006. Keep
    // U+2022; Symbol paints a missing WinAnsi 0x95.
    let t = raw.trim();
    // Other symbol-font code points (003329b5's Symbol U+F02A asterisk)
    // paint through the marker font's own symbol cmap, as Word draws them.
    if t.is_empty() || t == "\u{F0B7}" {
        return "• ".into();
    }
    format!("{t} ")
}

pub(super) fn load_numbering(pkg: &PartFs) -> Numbering {
    let numbering_part = main_rel_part(pkg, "numbering", "word/numbering.xml");
    let Some(xml) = pkg.part_string(&numbering_part) else {
        return Numbering::default();
    };
    parse_numbering_xml(&xml, |rid| resolve_media(pkg, &numbering_part, rid))
}

/// `word/numbering.xml`; `media` resolves a picture-bullet r:id.
pub(super) fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Numbering {
    let mut numbering = Numbering::default();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let Some(root) = dom.root(doc) else {
        return numbering;
    };
    for bullet in dom.descendants(root, Some(&W::name("numPicBullet"))) {
        let Some(id) = attr_any(&dom, bullet, "numPicBulletId").and_then(|s| s.parse().ok()) else {
            continue;
        };
        let rid = descendants_local(&dom, bullet, "imagedata")
            .into_iter()
            .find_map(|n| attr_any(&dom, n, "id"))
            .or_else(|| {
                descendants_local(&dom, bullet, "blip")
                    .into_iter()
                    .find_map(|n| attr_any(&dom, n, "embed"))
            });
        if let Some(rid) = rid
            && let Some(bytes) = media(rid)
        {
            numbering.pic_bytes.insert(id, bytes);
        }
        if let Some(ext) = descendants_local(&dom, bullet, "shape")
            .into_iter()
            .find_map(|sh| attr_any(&dom, sh, "style"))
            .and_then(|st| Some((vml_style_pt(st, "width")?, vml_style_pt(st, "height")?)))
            .filter(|(w, h)| *w > 0.0 && *h > 0.0)
        {
            numbering.pic_extent.insert(id, ext);
        }
    }
    let mut style_links: HashMap<String, String> = HashMap::new();
    // An abstract that is only a `w:numStyleLink` borrows the levels of the
    // abstract carrying the matching `w:styleLink` (edbbb194's "Judgments").
    let mut linked_to: HashMap<String, String> = HashMap::new();
    for abs in dom.descendants(root, Some(&W::name("abstractNum"))) {
        let Some(aid) = attr_any(&dom, abs, "abstractNumId") else {
            continue;
        };
        if let Some(name) =
            first_named(&dom, abs, "styleLink").and_then(|n| dom.attribute(n, &W::val()))
        {
            style_links.insert(name.to_string(), aid.to_string());
        }
        if let Some(name) =
            first_named(&dom, abs, "numStyleLink").and_then(|n| dom.attribute(n, &W::val()))
        {
            linked_to.insert(aid.to_string(), name.to_string());
        }
        let mut lvls = HashMap::new();
        for lvl in dom.descendants(abs, Some(&W::name("lvl"))) {
            let ilvl = attr_any(&dom, lvl, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            if let Some(v) = first_named(&dom, lvl, "lvlRestart")
                .and_then(|n| attr_any(&dom, n, "val"))
                .and_then(|s| s.parse().ok())
            {
                numbering.restarts.insert((aid.to_string(), ilvl), v);
            }
            if first_named(&dom, lvl, "isLgl").is_some_and(|n| !val_is_false(&dom, Some(n))) {
                numbering.is_lgl.insert((aid.to_string(), ilvl));
            }
            if let Some(ps) =
                first_named(&dom, lvl, "pStyle").and_then(|n| attr_any(&dom, n, "val"))
            {
                numbering
                    .lvl_pstyle
                    .insert((aid.to_string(), ilvl), ps.to_string());
            }
            if let Some(pic) = first_named(&dom, lvl, "lvlPicBulletId")
                .and_then(|n| attr_any(&dom, n, "val"))
                .and_then(|s| s.parse().ok())
            {
                numbering.lvl_pic.insert((aid.to_string(), ilvl), pic);
            }
            lvls.insert(ilvl, parse_num_level(&dom, lvl));
        }
        numbering.levels.insert(aid.to_string(), lvls);
    }
    for num in dom.descendants(root, Some(&W::name("num"))) {
        let Some(nid) = attr_any(&dom, num, "numId") else {
            continue;
        };
        let aid = first_named(&dom, num, "abstractNumId")
            .and_then(|n| dom.attribute(n, &W::val()))
            .map(str::to_string)
            .or_else(|| {
                first_named(&dom, num, "numStyleLink")
                    .and_then(|n| dom.attribute(n, &W::val()))
                    .and_then(|name| style_links.get(name).cloned())
            })
            .map(|aid| {
                linked_to
                    .get(&aid)
                    .and_then(|name| style_links.get(name))
                    .cloned()
                    .unwrap_or(aid)
            });
        let Some(aid) = aid else {
            continue;
        };
        // Full `lvlOverride/w:lvl` replacements give this num its own level
        // table (014caa99 "PART %2" over the abstract's "%2.").
        let overrides: Vec<(u32, NumLevel)> = dom
            .descendants(num, Some(&W::name("lvlOverride")))
            .into_iter()
            .filter_map(|ov| {
                let ilvl = attr_any(&dom, ov, "ilvl")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                first_named(&dom, ov, "lvl").map(|lvl| (ilvl, parse_num_level(&dom, lvl)))
            })
            .collect();
        let aid = if overrides.is_empty() {
            aid
        } else {
            let own = format!("{aid}@{nid}");
            let mut lvls = numbering.levels.get(&aid).cloned().unwrap_or_default();
            lvls.extend(overrides);
            numbering.levels.insert(own.clone(), lvls);
            let copy = |set: &mut HashMap<(String, u32), u32>| {
                let rows: Vec<_> = set
                    .iter()
                    .filter(|((a, _), _)| *a == aid)
                    .map(|((_, l), v)| (*l, *v))
                    .collect();
                for (l, v) in rows {
                    set.insert((own.clone(), l), v);
                }
            };
            copy(&mut numbering.restarts);
            copy(&mut numbering.lvl_pic);
            let lgl: Vec<u32> = numbering
                .is_lgl
                .iter()
                .filter(|(a, _)| *a == aid)
                .map(|(_, l)| *l)
                .collect();
            for l in lgl {
                numbering.is_lgl.insert((own.clone(), l));
            }
            own
        };
        numbering.instances.insert(nid.to_string(), aid);
        for ov in dom.descendants(num, Some(&W::name("lvlOverride"))) {
            let ilvl = attr_any(&dom, ov, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let start = first_named(&dom, ov, "startOverride")
                .and_then(|n| attr_any(&dom, n, "val"))
                .and_then(|s| s.parse().ok())
                .or_else(|| {
                    first_named(&dom, ov, "lvl")
                        .and_then(|lvl| first_named(&dom, lvl, "start"))
                        .and_then(|n| attr_any(&dom, n, "val"))
                        .and_then(|s| s.parse().ok())
                });
            if let Some(start) = start {
                numbering.starts.insert((nid.to_string(), ilvl), start);
            }
            // A full lvlOverride/w:lvl replaces the abstract level for this
            // instance, isLgl included (absent there means off).
            if let Some(lvl) = first_named(&dom, ov, "lvl") {
                let lgl =
                    first_named(&dom, lvl, "isLgl").is_some_and(|n| !val_is_false(&dom, Some(n)));
                numbering.lgl_override.insert((nid.to_string(), ilvl), lgl);
            }
        }
    }
    numbering
}

/// One `w:lvl` (an abstract level or a `w:lvlOverride` replacement).
pub(super) fn parse_num_level(dom: &Dom, lvl: NodeId) -> NumLevel {
    let fmt = first_named(dom, lvl, "numFmt")
        .and_then(|n| dom.attribute(n, &W::val()))
        .unwrap_or("decimal");
    let text = first_named(dom, lvl, "lvlText")
        .and_then(|n| dom.attribute(n, &W::val()))
        .unwrap_or("%1.")
        .to_string();
    let start = first_named(dom, lvl, "start")
        .and_then(|n| dom.attribute(n, &W::val()))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let (left, hanging) = lvl_indent(dom, lvl);
    let family = lvl_marker_family(dom, lvl, fmt, &text);
    let (size, underline, bold, italic) = lvl_marker_rpr(dom, lvl);
    let suff_nothing = first_named(dom, lvl, "suff")
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v.eq_ignore_ascii_case("nothing"));
    let tab_stops = first_named(dom, lvl, "pPr")
        .map(|ppr| parse_tab_stops(dom, ppr))
        .unwrap_or_default();
    NumLevel {
        fmt: parse_num_fmt(fmt),
        text,
        start,
        left,
        hanging,
        family,
        suff_nothing,
        jc_right: first_named(dom, lvl, "lvlJc")
            .and_then(|n| attr_any(dom, n, "val"))
            .is_some_and(|v| v.eq_ignore_ascii_case("right") || v.eq_ignore_ascii_case("end")),
        jc: first_named(dom, lvl, "pPr")
            .and_then(|ppr| first_named(dom, ppr, "jc"))
            .and_then(|n| attr_any(dom, n, "val"))
            .map(|v| match v {
                "center" => Align::Center,
                "right" | "end" => Align::Right,
                "both" | "distribute" => Align::Justify,
                _ => Align::Left,
            }),
        tab_stops,
        size,
        underline,
        bold,
        italic,
    }
}

/// The level's own marker face. `hAnsi` names the face of non-ASCII
/// characters only: a marker that is all ASCII ("1.", "(a)", "IV") under an
/// `hAnsi`-only level keeps the face the paragraph gives it (2386218b: a
/// level with only hAnsi="Arial Unicode MS" still painted "1." in Times).
pub(super) fn lvl_marker_family(dom: &Dom, lvl: NodeId, fmt: &str, text: &str) -> String {
    let Some(rpr) = first_named(dom, lvl, "rPr") else {
        return String::new();
    };
    let Some(fonts) = first_named(dom, rpr, "rFonts") else {
        return String::new();
    };
    if let Some(ascii) = attr_any(dom, fonts, "ascii") {
        return ascii.to_string();
    }
    let ascii_number = matches!(
        fmt,
        "decimal"
            | "decimalZero"
            | "upperRoman"
            | "lowerRoman"
            | "upperLetter"
            | "lowerLetter"
            | "ordinal"
            | "cardinalText"
            | "ordinalText"
            | "none"
            | "bullet"
    );
    let mut literal = text.split('%').enumerate().flat_map(|(i, part)| {
        // After a `%`, the level digit is a placeholder, not marker text.
        let skip = usize::from(i > 0 && part.starts_with(|c: char| c.is_ascii_digit()));
        part.chars().skip(skip)
    });
    if ascii_number && literal.all(|c| c.is_ascii()) {
        return String::new();
    }
    attr_any(dom, fonts, "hAnsi").unwrap_or("").to_string()
}

fn lvl_marker_rpr(dom: &Dom, lvl: NodeId) -> (Option<f32>, bool, bool, bool) {
    let Some(rpr) = first_named(dom, lvl, "rPr") else {
        return (None, false, false, false);
    };
    let size = first_named(dom, rpr, "sz")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(parse_finite_f32)
        .map(|half| half / 2.0);
    let underline = first_named(dom, rpr, "u").is_some_and(|n| {
        !val_is_false(dom, Some(n))
            && attr_any(dom, n, "val").is_none_or(|v| !v.eq_ignore_ascii_case("none"))
    });
    let bold = first_named(dom, rpr, "b").is_some_and(|n| !val_is_false(dom, Some(n)));
    let italic = first_named(dom, rpr, "i").is_some_and(|n| !val_is_false(dom, Some(n)));
    (size, underline, bold, italic)
}

fn lvl_indent(dom: &Dom, lvl: NodeId) -> (f32, f32) {
    let Some(ppr) = first_named(dom, lvl, "pPr") else {
        return (0.0, 0.0);
    };
    let Some(ind) = first_named(dom, ppr, "ind") else {
        return (0.0, 0.0);
    };
    let left = attr_any(dom, ind, "left")
        .or_else(|| attr_any(dom, ind, "start"))
        .and_then(parse_len)
        .unwrap_or(0.0);
    let hanging = attr_any(dom, ind, "hanging")
        .and_then(parse_len)
        .unwrap_or(0.0);
    (left, hanging)
}

pub(super) fn resolve_num_pr(
    dom: &Dom,
    raw: &std::collections::HashMap<String, RawStyle>,
    id: &str,
    depth: u8,
) -> (Option<String>, u32) {
    if depth > 12 {
        return (None, 0);
    }
    let Some(raw_style) = raw.get(id) else {
        return (None, 0);
    };
    if let Some(ppr) = raw_style.ppr {
        let (nid, ilvl) = num_pr(dom, ppr);
        if nid.is_some() {
            return (nid, ilvl);
        }
    }
    if let Some(base) = raw_style.based.as_deref() {
        return resolve_num_pr(dom, raw, base, depth + 1);
    }
    (None, 0)
}

fn num_pr(dom: &Dom, ppr: NodeId) -> (Option<String>, u32) {
    let Some(num) = first_named(dom, ppr, "numPr") else {
        return (None, 0);
    };
    let ilvl = first_named(dom, num, "ilvl")
        .and_then(|n| dom.attribute(n, &W::val()))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let num_id = first_named(dom, num, "numId")
        .and_then(|n| dom.attribute(n, &W::val()))
        .map(str::to_string);
    (num_id, ilvl)
}

/// Apply a paragraph mark's `w:rPr` to `style` as Word does a run's: its
/// character style first, then its own properties (Word 16 probe
/// rsty0930: tb27bda's Font Style12 sets the labels in Times; its Font
/// Style11 marks make its empty lines Times lines, 13.8pt not Verdana's
/// 14.58).
pub(super) fn apply_mark_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, sheet: &StyleSheet) {
    if let Some(named) = first_named(dom, rpr, "rStyle")
        .and_then(|n| dom.attribute(n, &W::val()))
        .and_then(|sid| sheet.by_id.get(sid))
    {
        apply_named_char_style(style, named);
    }
    apply_rpr(dom, rpr, style, &sheet.theme);
}

/// A numbered paragraph's marker style — its mark's pPr/rPr, then the
/// level's rPr — with the level's indent and tabs applied to `pstyle`.
/// Body and cell paragraphs share it (00297360's cell "1." is 10pt and
/// hangs at the level's 18.15pt like a body item).
pub(super) fn apply_list_level(
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
    lvl: Option<&NumLevel>,
    rstyle: &RunStyle,
    pstyle: &mut ParaStyle,
) -> RunStyle {
    // Word styles the number from the paragraph mark's run (pPr/rPr),
    // then the level's rPr (019f3137: sz=20 marks under an 11pt
    // default keep 10pt bullets and 10pt lines).
    let mut marker_style = rstyle.clone();
    if let Some(rpr) = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
    {
        apply_mark_rpr(dom, rpr, &mut marker_style, sheet);
    }
    if let Some(lvl) = lvl {
        if !lvl.family.is_empty() {
            marker_style.family = lvl.family.clone();
        }
        if let Some(sz) = lvl.size {
            marker_style.size = sz;
        }
        if lvl.underline {
            marker_style.underline = true;
        }
        if lvl.bold {
            marker_style.bold = true;
        }
        if lvl.italic {
            marker_style.italic = true;
        }
        // Numbering lvl pPr/ind overrides the paragraph style (ListParagraph
        // start=720 vs Strict01 ilvl start=18pt/36pt). Direct pPr/ind wins
        // attribute by attribute: a direct `left` alone keeps the level's
        // hanging (00194caa's "1." hangs from left=426).
        let direct_ind = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| first_named(dom, ppr, "ind"));
        let direct_has = |names: &[&str]| {
            direct_ind.is_some_and(|ind| names.iter().any(|n| attr_any(dom, ind, n).is_some()))
        };
        // A numPr the paragraph only inherits from its style yields to that
        // style's own w:ind (live Word: Bulleted 270/270 over a 360/360
        // level puts text at 270; a direct numPr takes the level's 360).
        let direct_num = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| first_named(dom, ppr, "numPr"))
            .and_then(|np| first_named(dom, np, "numId"))
            .is_some();
        let (style_left, style_first) = if direct_num {
            (false, false)
        } else {
            dom.element(para, &W::p_pr())
                .and_then(|ppr| first_named(dom, ppr, "pStyle"))
                .and_then(|ps| dom.attribute(ps, &W::val()))
                .and_then(|sid| sheet.by_id.get(sid))
                .map_or((false, false), |named| named.sets_ind)
        };
        // A negative level indent counts too: 66cfa52b0c's bullets at
        // left=-131 sit 6.55pt into the margin in Word, not at the List
        // Paragraph's 36pt.
        if lvl.left != 0.0 && !style_left && !direct_has(&["left", "start"]) {
            pstyle.indent_left = lvl.left;
        }
        if lvl.hanging > 0.0 && !style_first && !direct_has(&["hanging", "firstLine"]) {
            pstyle.indent_first = -lvl.hanging;
        }
        // Like its w:ind, a direct numPr's level jc beats the paragraph
        // style's; a direct w:jc still wins.
        let direct_jc = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| first_named(dom, ppr, "jc"))
            .is_some();
        if let Some(jc) = lvl.jc
            && direct_num
            && !direct_jc
        {
            pstyle.align = jc;
        }
        pstyle.list_jc_right = lvl.jc_right;
        merge_tab_stops(&mut pstyle.tab_stops, &lvl.tab_stops);
    }
    marker_style
}

pub(super) fn list_marker(
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
    numbering: &mut Numbering,
) -> (String, String, u32) {
    let from_para = dom.element(para, &W::p_pr()).map(|ppr| num_pr(dom, ppr));
    let style_id = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| first_named(dom, ppr, "pStyle"))
        .and_then(|ps| dom.attribute(ps, &W::val()));
    let from_style = style_id
        .and_then(|sid| sheet.by_id.get(sid))
        .map(|named| (named.num_id.clone(), named.ilvl));
    let (num_id, ilvl) = match (from_para, from_style) {
        (Some((Some(id), ilvl)), _) => (id, ilvl),
        (_, Some((Some(id), ilvl))) => {
            // A style numbered through a level linked to another style
            // gets that level's indent but no number and no count
            // (docxide case73, live Word: P Heading B under a list whose
            // level 0 is P Heading A's).
            if numbering
                .linked_style(&id, ilvl)
                .is_some_and(|linked| Some(linked) != style_id)
            {
                return (String::new(), id, ilvl);
            }
            (id, ilvl)
        }
        _ => {
            // Numbering a tracked change took away still counts in the
            // original lists.
            if let Some(Some((id, ilvl))) = prior_list(dom, para, sheet) {
                numbering.original().next_marker(&id, ilvl);
            }
            return (String::new(), String::new(), 0);
        }
    };
    let mut mark = para_mark_rev(dom, para).map_or(RevMark::None, |(mark, _)| mark);
    // A tracked change that put the paragraph in this list: the original
    // counts it in its old list, if any, and shows no old number (en r
    // 00f467c010's "2)" was unnumbered before, Word draws no "1)2)").
    if mark == RevMark::None
        && let Some(prior) = prior_list(dom, para, sheet)
        && prior.as_ref() != Some(&(num_id.clone(), ilvl))
    {
        if let Some((id, old_ilvl)) = prior {
            numbering.original().next_marker(&id, old_ilvl);
        }
        mark = RevMark::Ins;
    }
    (numbering.next_revised(&num_id, ilvl, mark), num_id, ilvl)
}

/// The revision a list label carries: its paragraph mark's, else an
/// insertion by the `w:pPrChange` that put the paragraph in this list (Word
/// 16 probe chg0930: the "2." a change numbered is inked and underlined).
pub(super) fn label_rev<'a>(
    dom: &'a Dom,
    para: NodeId,
    sheet: &StyleSheet,
    num_id: &str,
    ilvl: u32,
) -> Option<(RevMark, &'a str)> {
    para_mark_rev(dom, para).or_else(|| {
        let prior = prior_list(dom, para, sheet)?;
        if prior.is_some_and(|(id, old)| id == num_id && old == ilvl) {
            return None;
        }
        let change = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))?;
        Some((RevMark::Ins, attr_any(dom, change, "author").unwrap_or("")))
    })
}

/// The list a tracked formatting change (`w:pPrChange`) says the paragraph
/// was in before: `None` without one, `Some(None)` when its old properties
/// number nothing.
fn prior_list(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<Option<(String, u32)>> {
    let old = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))
        .and_then(|change| dom.element(change, &W::p_pr()))?;
    let direct = match num_pr(dom, old) {
        (Some(id), ilvl) => Some((id, ilvl)),
        (None, _) => first_named(dom, old, "pStyle")
            .and_then(|ps| dom.attribute(ps, &W::val()))
            .and_then(|sid| sheet.by_id.get(sid))
            .and_then(|named| Some((named.num_id.clone()?, named.ilvl))),
    };
    Some(direct.filter(|(id, _)| id != "0"))
}

/// The revision on a paragraph's mark (`pPr/rPr`) and its author.
pub(super) fn para_mark_rev(dom: &Dom, para: NodeId) -> Option<(RevMark, &str)> {
    let rpr = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))?;
    dom.elements(rpr, None).into_iter().find_map(|c| {
        let mark = rev_mark_of(dom, c)?;
        Some((mark, attr_any(dom, c, "author").unwrap_or("")))
    })
}

/// The author Word charges a list's renumbering to: the next ink after the
/// document's own authors (probe lbl0930: msi's 394146, then 0B6A0B).
const RENUMBER_AUTHOR: &str = "\u{1}renumbered";

/// Word draws a list label with its paragraph mark's revision, not its
/// text's (probe lbl0930): an inserted mark inks and underlines "3." and
/// its tab over unrevised text, a deleted one strikes it, and inserted
/// text under a plain mark leaves the label alone. A renumbered paragraph
/// shows its old number plain, then the new one inked and underlined in
/// the renumbering's own colour ("1.2."); only `RevisionStyle::Word`
/// copies that, our own marks keep the revised number alone.
pub(super) fn revise_list_label(
    runs: &mut Vec<TextRun>,
    mark: Option<(RevMark, &str)>,
    old: Option<String>,
    authors: &mut AuthorColors,
) {
    let Some(label) = runs.first_mut().filter(|r| r.list_marker) else {
        return;
    };
    if let Some((mark, author)) = mark {
        apply_rev(&mut label.style, mark, authors.color(author));
        label.rev = true;
        return;
    }
    let Some(old) =
        old.filter(|_| matches!(REVISIONS.with(std::cell::Cell::get), RevisionStyle::Word))
    else {
        return;
    };
    let plain = label.with_text(old.trim_end_matches(['\t', ' ']));
    apply_rev(
        &mut label.style,
        RevMark::Ins,
        authors.color(RENUMBER_AUTHOR),
    );
    label.rev = true;
    runs.insert(0, plain);
}
