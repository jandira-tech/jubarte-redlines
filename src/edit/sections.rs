// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `page_setup`: page size, orientation and margins of the last section or
//! of every section. Each section's `w:pgSz` and `w:pgMar` are rewritten in
//! `CT_SectPr` order, keeping what the plan does not change; a document
//! without a final `w:sectPr` gets one, starting from Word's default Letter
//! page with one-inch margins. The comparer records the old geometry in
//! `w:sectPrChange`.

use serde::{Deserialize, Serialize};

use crate::inspect::Opened;
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

use super::{EditError, EditOutcome, Transaction, err};

/// Which sections a [`super::OperationKind::PageSetup`] changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionScope {
    /// The document's last section (the body's final `w:sectPr`).
    #[default]
    Last,
    /// Every section.
    All,
}

/// A paper size, by name or in twentieths of a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PageSize {
    /// A named paper size, portrait.
    Named(Paper),
    /// A custom size.
    Custom(CustomPage),
}

/// Named paper sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Paper {
    /// US Letter, 8.5 x 11 inches.
    Letter,
    /// ISO A4, 210 x 297 mm.
    A4,
}

/// A custom page size in twentieths of a point (1440 per inch).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomPage {
    /// Page width.
    pub width_dxa: u32,
    /// Page height.
    pub height_dxa: u32,
}

/// Page orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    /// Taller than wide.
    Portrait,
    /// Wider than tall (`w:orient="landscape"`).
    Landscape,
}

/// Page margins in twentieths of a point; fields not given keep the
/// section's. Top and bottom may be negative, as in Word: text then ignores
/// the header or footer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Margins {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Top margin.
    pub top: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Right margin.
    pub right: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Bottom margin.
    pub bottom: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Left margin.
    pub left: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Distance from the page top to the header.
    pub header: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Distance from the page bottom to the footer.
    pub footer: Option<u32>,
}

/// The largest page side Word accepts: 22 inches.
const MAX_SIDE: u32 = 31_680;

/// A section's page geometry, all in twentieths of a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Geometry {
    width: u32,
    height: u32,
    landscape: bool,
    /// `paper code` of the source page, kept while the size stays.
    keep_code: bool,
    top: i64,
    right: i64,
    bottom: i64,
    left: i64,
    header: i64,
    footer: i64,
    gutter: i64,
}

impl Geometry {
    /// Word's default section: Letter, portrait, one-inch margins.
    const WORD_DEFAULT: Self = Self {
        width: 12_240,
        height: 15_840,
        landscape: false,
        keep_code: true,
        top: 1440,
        right: 1440,
        bottom: 1440,
        left: 1440,
        header: 720,
        footer: 720,
        gutter: 0,
    };

    /// The geometry `sect` sets, Word's default for what it leaves out.
    fn of(dom: &Dom, sect: Option<NodeId>) -> Self {
        let mut g = Self::WORD_DEFAULT;
        let Some(sect) = sect else { return g };
        let read = |el: NodeId, local: &str| {
            dom.attribute(el, &W::name(local))
                .and_then(|v| v.trim().parse::<i64>().ok())
        };
        if let Some(size) = dom.element(sect, &W::name("pgSz")) {
            let side = |v: Option<i64>| v.and_then(|v| u32::try_from(v).ok());
            g.width = side(read(size, "w")).unwrap_or(g.width);
            g.height = side(read(size, "h")).unwrap_or(g.height);
            g.landscape = dom.attribute(size, &W::name("orient")) == Some("landscape");
        }
        if let Some(margins) = dom.element(sect, &W::name("pgMar")) {
            for (local, slot) in [
                ("top", &mut g.top),
                ("right", &mut g.right),
                ("bottom", &mut g.bottom),
                ("left", &mut g.left),
                ("header", &mut g.header),
                ("footer", &mut g.footer),
                ("gutter", &mut g.gutter),
            ] {
                if let Some(v) = read(margins, local) {
                    *slot = v;
                }
            }
        }
        g
    }

    /// The geometry after the plan's changes, or why it is refused.
    fn changed(
        mut self,
        page: Option<PageSize>,
        orientation: Option<Orientation>,
        margins: Margins,
    ) -> Result<Self, String> {
        let custom = matches!(page, Some(PageSize::Custom(_)));
        if let Some(page) = page {
            let (w, h) = match page {
                PageSize::Named(Paper::Letter) => (12_240, 15_840),
                PageSize::Named(Paper::A4) => (11_906, 16_838),
                PageSize::Custom(CustomPage {
                    width_dxa,
                    height_dxa,
                }) => (width_dxa, height_dxa),
            };
            for side in [w, h] {
                if !(1..=MAX_SIDE).contains(&side) {
                    return Err(format!(
                        "page side {side} is outside 1..={MAX_SIDE} (22 inches)"
                    ));
                }
            }
            if (w, h) != (self.width, self.height) {
                self.keep_code = false;
            }
            (self.width, self.height) = (w, h);
        }
        let landscape = match orientation {
            Some(o) => o == Orientation::Landscape,
            None if custom => self.width > self.height,
            None => self.landscape,
        };
        if !custom || orientation.is_some() {
            let (short, long) = (self.width.min(self.height), self.width.max(self.height));
            (self.width, self.height) = if landscape {
                (long, short)
            } else {
                (short, long)
            };
        }
        self.landscape = landscape;
        let set = |slot: &mut i64, value: Option<i64>| {
            if let Some(value) = value {
                *slot = value;
            }
        };
        set(&mut self.top, margins.top.map(i64::from));
        set(&mut self.right, margins.right.map(i64::from));
        set(&mut self.bottom, margins.bottom.map(i64::from));
        set(&mut self.left, margins.left.map(i64::from));
        set(&mut self.header, margins.header.map(i64::from));
        set(&mut self.footer, margins.footer.map(i64::from));
        let limit = i64::from(MAX_SIDE);
        for (name, value) in [
            ("top", self.top),
            ("bottom", self.bottom),
            ("left", self.left),
            ("right", self.right),
            ("header", self.header),
            ("footer", self.footer),
        ] {
            if value.abs() > limit {
                return Err(format!(
                    "margin {name} {value} is outside -{limit}..={limit}"
                ));
            }
        }
        if self.left + self.right + self.gutter >= i64::from(self.width) {
            return Err(format!(
                "left and right margins ({} + {}) leave no text width on a page {} wide",
                self.left, self.right, self.width
            ));
        }
        if self.top.abs() + self.bottom.abs() >= i64::from(self.height) {
            return Err(format!(
                "top and bottom margins ({} + {}) leave no text height on a page {} high",
                self.top, self.bottom, self.height
            ));
        }
        Ok(self)
    }
}

/// `CT_SectPr` child order.
const SECT_ORDER: &[&str] = &[
    "headerReference",
    "footerReference",
    "footnotePr",
    "endnotePr",
    "type",
    "pgSz",
    "pgMar",
    "paperSrc",
    "pgBorders",
    "lnNumType",
    "pgNumType",
    "cols",
    "formProt",
    "vAlign",
    "noEndnote",
    "titlePg",
    "textDirection",
    "bidi",
    "rtlGutter",
    "docGrid",
    "printerSettings",
    "sectPrChange",
];

fn sect_rank(local: &str) -> usize {
    SECT_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(SECT_ORDER.len())
}

/// `sect`'s child `local`, created at its schema position when absent.
fn sect_child(dom: &mut Dom, sect: NodeId, local: &str) -> NodeId {
    if let Some(found) = dom.element(sect, &W::name(local)) {
        return found;
    }
    let child = dom.new_element(W::name(local));
    let rank = sect_rank(local);
    let after = dom.elements(sect, None).into_iter().rev().find(|&c| {
        dom.name(c)
            .is_some_and(|n| n.namespace_name() == W::URI && sect_rank(n.local_name()) <= rank)
    });
    match after {
        Some(after) => dom.add_after_self(after, child),
        None => dom.add_first(sect, child),
    }
    child
}

/// Write `geometry` into `sect`'s `w:pgSz` and `w:pgMar`.
pub(super) fn write_geometry(dom: &mut Dom, sect: NodeId, geometry: &Geometry) {
    let size = sect_child(dom, sect, "pgSz");
    dom.set_attribute_value(size, &W::name("w"), Some(&geometry.width.to_string()));
    dom.set_attribute_value(size, &W::name("h"), Some(&geometry.height.to_string()));
    dom.set_attribute_value(
        size,
        &W::name("orient"),
        geometry.landscape.then_some("landscape"),
    );
    if !geometry.keep_code {
        dom.set_attribute_value(size, &W::name("code"), None);
    }
    let margins = sect_child(dom, sect, "pgMar");
    for (local, value) in [
        ("top", geometry.top),
        ("right", geometry.right),
        ("bottom", geometry.bottom),
        ("left", geometry.left),
        ("header", geometry.header),
        ("footer", geometry.footer),
        ("gutter", geometry.gutter),
    ] {
        dom.set_attribute_value(margins, &W::name(local), Some(&value.to_string()));
    }
}

/// The sections a page setup changes: `None` stands for a final section the
/// body lacks, which apply creates.
pub(super) type Targets = Vec<(Option<NodeId>, Geometry)>;

impl Transaction<'_> {
    /// The body's section properties in document order, the final one last
    /// (`None` when the body has no final `w:sectPr`). Recorded old
    /// properties inside revision marks do not count.
    fn sections(&self) -> Vec<Option<NodeId>> {
        let dom = &self.opened.dom;
        let body = self.opened.body;
        let last = dom.element(body, &W::sect_pr());
        let mut out: Vec<Option<NodeId>> = mid_sections(dom, body).into_iter().map(Some).collect();
        out.push(last);
        out
    }

    /// Resolve a `page_setup`: each target section with its new geometry.
    pub(super) fn resolve_page_setup(
        &self,
        scope: SectionScope,
        page: Option<PageSize>,
        orientation: Option<Orientation>,
        margins: Margins,
        outcome: &mut EditOutcome,
    ) -> Result<Targets, (String, String)> {
        if page.is_none() && orientation.is_none() && margins == Margins::default() {
            return Err((
                "INVALID_EDIT".into(),
                "page_setup needs page, orientation or margins_dxa".into(),
            ));
        }
        let mut sections = self.sections();
        if scope == SectionScope::Last {
            sections = sections.split_off(sections.len() - 1);
        }
        outcome.matches = sections.len();
        let mut targets = Vec::new();
        for sect in sections {
            let geometry = Geometry::of(&self.opened.dom, sect)
                .changed(page, orientation, margins)
                .map_err(|m| ("INVALID_EDIT".to_string(), m))?;
            targets.push((sect, geometry));
        }
        Ok(targets)
    }

    /// Apply resolved page setups; a missing final `w:sectPr` is appended to
    /// the body.
    pub(super) fn apply_page_setup(&mut self, targets: &Targets) {
        for (sect, geometry) in targets {
            let sect = match sect {
                Some(sect) => *sect,
                None => match self.opened.dom.element(self.opened.body, &W::sect_pr()) {
                    Some(created) => created,
                    None => {
                        let created = self.opened.dom.new_element(W::sect_pr());
                        self.opened.dom.add(self.opened.body, created);
                        created
                    }
                },
            };
            write_geometry(&mut self.opened.dom, sect, geometry);
        }
    }
}

/// Mid-document `w:sectPr`s under `body`, in document order: the final one
/// and recorded old properties inside revision marks are left out.
fn mid_sections(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    let last = dom.element(body, &W::sect_pr());
    dom.descendants(body, Some(&W::sect_pr()))
        .into_iter()
        .filter(|&s| Some(s) != last)
        .filter(|&s| {
            dom.ancestors(s, None).into_iter().all(|a| {
                !dom.name(a)
                    .is_some_and(|n| matches!(n.local_name(), "sectPrChange" | "pPrChange"))
            })
        })
        .collect()
}

/// `w:pgSz` and `w:pgMar` of a section, attributes sorted, for comparison.
fn geometry_signature(dom: &Dom, sect: NodeId) -> Vec<(String, Vec<(String, String)>)> {
    ["pgSz", "pgMar"]
        .iter()
        .map(|local| {
            let mut attrs: Vec<(String, String)> = dom
                .element(sect, &W::name(local))
                .map(|el| {
                    dom.attributes(el)
                        .into_iter()
                        .map(|(name, value)| (name.clark(), value))
                        .collect()
                })
                .unwrap_or_default();
            attrs.sort();
            ((*local).to_string(), attrs)
        })
        .collect()
}

/// The comparer records old section properties for the final section only.
/// Give every mid-document section whose page size or margins changed a
/// `w:sectPrChange` holding the base's properties (without header and
/// footer references, which `CT_SectPrBase` leaves out), as Word does.
/// Sections pair up by order: no edit adds or removes a section break.
pub(super) fn record_mid_changes(
    redline: &[u8],
    base: &[u8],
    author: &str,
    date: &str,
) -> Result<Vec<u8>, EditError> {
    let open = |bytes| Opened::open(bytes).map_err(|e| err("COMPARE_FAILED", None, e.to_string()));
    let mut red = open(redline)?;
    let old = open(base)?;
    let red_sections = mid_sections(&red.dom, red.body);
    let old_sections = mid_sections(&old.dom, old.body);
    if red_sections.len() != old_sections.len() {
        return Ok(redline.to_vec());
    }
    let mut next_id = red
        .dom
        .descendants(red.body, None)
        .into_iter()
        .filter_map(|n| red.dom.attribute(n, &W::id())?.parse::<u64>().ok())
        .max()
        .map_or(1, |max| max + 1);
    let mut changed = false;
    for (&now, &was) in red_sections.iter().zip(&old_sections) {
        if red.dom.element(now, &W::name("sectPrChange")).is_some()
            || geometry_signature(&red.dom, now) == geometry_signature(&old.dom, was)
        {
            continue;
        }
        let record = red.dom.new_element(W::name("sectPrChange"));
        red.dom
            .set_attribute_value(record, &W::id(), Some(&next_id.to_string()));
        next_id += 1;
        red.dom
            .set_attribute_value(record, &W::author(), Some(author));
        red.dom.set_attribute_value(record, &W::date(), Some(date));
        let props = red.dom.new_element(W::sect_pr());
        for child in old.dom.elements(was, None) {
            let skip = old.dom.name(child).is_some_and(|n| {
                matches!(
                    n.local_name(),
                    "headerReference" | "footerReference" | "sectPrChange"
                )
            });
            if !skip {
                let xml = old.dom.serialize_element(child);
                let fragment = red.dom.parse_xdocument(&format!(
                    r#"<w:sectPr xmlns:w="{}">{xml}</w:sectPr>"#,
                    W::URI
                ));
                if let Some(wrapper) = red.dom.root(fragment) {
                    for copied in red.dom.elements(wrapper, None) {
                        red.dom.add(props, copied);
                    }
                }
            }
        }
        red.dom.add(record, props);
        red.dom.add(now, record);
        changed = true;
    }
    if !changed {
        return Ok(redline.to_vec());
    }
    let xml = red.dom.serialize_document(red.document);
    let main = red.main.clone();
    red.pkg.set_part(&main, xml.into_bytes());
    red.pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}

/// Whether two page setups change a section in common.
pub(super) fn targets_overlap(a: &Targets, b: &Targets) -> bool {
    a.iter().any(|(x, _)| b.iter().any(|(y, _)| x == y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letter() -> Geometry {
        Geometry::WORD_DEFAULT
    }

    #[test]
    fn named_pages_follow_the_orientation() {
        let a4 = letter()
            .changed(
                Some(PageSize::Named(Paper::A4)),
                Some(Orientation::Landscape),
                Margins::default(),
            )
            .unwrap();
        assert_eq!((a4.width, a4.height, a4.landscape), (16_838, 11_906, true));
        assert!(!a4.keep_code);
        let turned = a4
            .changed(
                Some(PageSize::Named(Paper::Letter)),
                None,
                Margins::default(),
            )
            .unwrap();
        assert_eq!(
            (turned.width, turned.height, turned.landscape),
            (15_840, 12_240, true)
        );
        let upright = turned
            .changed(None, Some(Orientation::Portrait), Margins::default())
            .unwrap();
        assert_eq!(
            (upright.width, upright.height, upright.landscape),
            (12_240, 15_840, false)
        );
    }

    #[test]
    fn custom_pages_keep_their_sides_unless_an_orientation_is_given() {
        let wide = PageSize::Custom(CustomPage {
            width_dxa: 16_000,
            height_dxa: 9_000,
        });
        let g = letter()
            .changed(Some(wide), None, Margins::default())
            .unwrap();
        assert_eq!((g.width, g.height, g.landscape), (16_000, 9_000, true));
        let g = letter()
            .changed(Some(wide), Some(Orientation::Portrait), Margins::default())
            .unwrap();
        assert_eq!((g.width, g.height, g.landscape), (9_000, 16_000, false));
    }

    #[test]
    fn margins_must_leave_text_room_and_stay_in_range() {
        let tight = Margins {
            left: Some(6000),
            right: Some(6240),
            ..Margins::default()
        };
        assert!(letter().changed(None, None, tight).is_err());
        let tall = Margins {
            top: Some(-8000),
            bottom: Some(7840),
            ..Margins::default()
        };
        assert!(letter().changed(None, None, tall).is_err());
        let huge = Margins {
            header: Some(40_000),
            ..Margins::default()
        };
        assert!(letter().changed(None, None, huge).is_err());
        let fine = Margins {
            top: Some(-720),
            ..Margins::default()
        };
        assert_eq!(letter().changed(None, None, fine).unwrap().top, -720);
        let zero = PageSize::Custom(CustomPage {
            width_dxa: 0,
            height_dxa: 100,
        });
        assert!(
            letter()
                .changed(Some(zero), None, Margins::default())
                .is_err()
        );
    }

    #[test]
    fn section_children_go_in_schema_order() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:sectPr xmlns:w="{}"><w:headerReference/><w:cols/><w:docGrid/></w:sectPr>"#,
            W::URI
        ));
        let sect = dom.root(doc).unwrap();
        write_geometry(&mut dom, sect, &letter());
        let names: Vec<String> = dom
            .elements(sect, None)
            .into_iter()
            .map(|c| dom.name(c).unwrap().local_name().to_string())
            .collect();
        assert_eq!(
            names,
            ["headerReference", "pgSz", "pgMar", "cols", "docGrid"]
        );
    }
}
