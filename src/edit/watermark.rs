// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The `watermark` operation: Word's own text watermark (Insert > Watermark),
//! a VML WordArt shape (`_x0000_t136`, id `PowerPlusWaterMarkObjectN`) in a
//! `Watermarks` building-block content control, written into every default
//! header. A first section without a default header gets a new header part;
//! a later one inherits the previous section's header, as in Word. The shape
//! is header content, not a revision, so the comparer leaves it untracked.

use std::collections::BTreeSet;

use super::{EditError, EditOutcome, StoryPart, Transaction, check_text, err};
use crate::namespaces::{R, W};
use crate::xmllinq::{Dom, NodeId, XNamespace};

const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const VML_URI: &str = "urn:schemas-microsoft-com:vml";
const O_URI: &str = "urn:schemas-microsoft-com:office:office";
const W10_URI: &str = "urn:schemas-microsoft-com:office:word";
/// Shape ids Word gives text and picture watermarks.
const WATERMARK_IDS: [&str; 2] = ["PowerPlusWaterMarkObject", "WordPictureWatermark"];
/// Longest watermark text the operation takes.
const MAX_TEXT: usize = 64;
/// Word's shape size for a text watermark on a Letter page.
const LETTER: (f64, f64) = (527.85, 131.95);

/// A validated watermark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Spec {
    text: String,
    /// Six upper-case hex digits.
    color: String,
    diagonal: bool,
    font: String,
}

/// Check the operation's parameters; `None` takes Word's defaults
/// (`C0C0C0`, diagonal, Calibri).
pub(super) fn spec(
    text: &str,
    color: Option<&str>,
    diagonal: Option<bool>,
    font: Option<&str>,
) -> Result<Spec, String> {
    check_text(text)?;
    let count = text.chars().count();
    if count == 0 || count > MAX_TEXT {
        return Err(format!(
            "watermark text must be 1 to {MAX_TEXT} characters, got {count}"
        ));
    }
    let color = color.unwrap_or("C0C0C0");
    if color.len() != 6 || !color.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "watermark color must be six hex digits (C0C0C0), got {color:?}"
        ));
    }
    let font = font.unwrap_or("Calibri");
    check_text(font)?;
    if font.trim().is_empty() || font.contains(['"', ';', '&', '<', '>']) {
        return Err(format!("watermark font {font:?} is not a font name"));
    }
    Ok(Spec {
        text: text.to_string(),
        color: color.to_ascii_uppercase(),
        diagonal: diagonal.unwrap_or(true),
        font: font.to_string(),
    })
}

/// `12.30` -> `12.3`, `406.17` stays.
fn points(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Word's watermark paragraph for header number `n` (1-based) with a shape
/// of `size` points: the `Watermarks` content control around the header
/// paragraph holding the VML shape type and shape.
fn markup(spec: &Spec, n: usize, size: (f64, f64)) -> String {
    let rotation = if spec.diagonal { "rotation:315;" } else { "" };
    let (width, height) = (points(size.0), points(size.1));
    let spid = 2048 + n;
    let (font, text) = (escape(&spec.font), escape(&spec.text));
    let color = &spec.color;
    format!(
        concat!(
            r#"<w:sdt><w:sdtPr><w:id w:val="{spid}"/><w:docPartObj><w:docPartGallery w:val="Watermarks"/><w:docPartUnique/></w:docPartObj></w:sdtPr><w:sdtContent>"#,
            r#"<w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr><w:r><w:rPr><w:noProof/></w:rPr><w:pict>"#,
            r#"<v:shapetype id="_x0000_t136" coordsize="21600,21600" o:spt="136" adj="10800" path="m@7,l@8,m@5,21600l@6,21600e">"#,
            r#"<v:formulas><v:f eqn="sum #0 0 10800"/><v:f eqn="prod #0 2 1"/><v:f eqn="sum 21600 0 @1"/><v:f eqn="sum 0 0 @2"/><v:f eqn="sum 21600 0 @3"/><v:f eqn="if @0 @3 0"/><v:f eqn="if @0 21600 @1"/><v:f eqn="if @0 0 @2"/><v:f eqn="if @0 @4 21600"/><v:f eqn="mid @5 @6"/><v:f eqn="mid @8 @5"/><v:f eqn="mid @7 @8"/><v:f eqn="mid @6 @7"/><v:f eqn="sum @6 0 @5"/></v:formulas>"#,
            r#"<v:path textpathok="t" o:connecttype="custom" o:connectlocs="@9,0;@10,10800;@11,21600;@12,10800" o:connectangles="270,180,90,0"/>"#,
            r##"<v:textpath on="t" fitshape="t"/><v:handles><v:h position="#0,bottomRight" xrange="6629,14971"/></v:handles>"##,
            r#"<o:lock v:ext="edit" text="t" shapetype="t"/></v:shapetype>"#,
            r##"<v:shape id="PowerPlusWaterMarkObject{n}" o:spid="_x0000_s{spid}" type="#_x0000_t136" "##,
            r#"style="position:absolute;margin-left:0;margin-top:0;width:{width}pt;height:{height}pt;{rotation}z-index:-251656192;mso-position-horizontal:center;mso-position-horizontal-relative:margin;mso-position-vertical:center;mso-position-vertical-relative:margin" "#,
            r##"o:allowincell="f" fillcolor="#{color}" stroked="f"><v:fill opacity=".5"/>"##,
            r#"<v:textpath style="font-family:&quot;{font}&quot;;font-size:1pt" string="{text}"/>"#,
            r#"<w10:wrap anchorx="margin" anchory="margin"/></v:shape></w:pict></w:r></w:p></w:sdtContent></w:sdt>"#,
        ),
        spid = spid,
        n = n,
        width = width,
        height = height,
        rotation = rotation,
        color = color,
        font = font,
        text = text,
    )
}

/// The `w:hdr` start tag with every namespace the markup uses.
fn header_open() -> String {
    format!(
        r#"<w:hdr xmlns:w="{}" xmlns:r="{}" xmlns:v="{VML_URI}" xmlns:o="{O_URI}" xmlns:w10="{W10_URI}">"#,
        W::URI,
        R::URI
    )
}

fn twips(dom: &Dom, node: Option<NodeId>, attr: &str) -> Option<f64> {
    node.and_then(|n| dom.attribute(n, &W::name(attr)))
        .and_then(|v| v.parse::<f64>().ok())
}

/// The shape size for a section: Word's own numbers on a portrait Letter
/// page; elsewhere 0.9 of the text width at Word's 4:1 ratio.
fn shape_size(dom: &Dom, sect_pr: NodeId) -> (f64, f64) {
    let pg_sz = dom.element(sect_pr, &W::name("pgSz"));
    let pg_mar = dom.element(sect_pr, &W::name("pgMar"));
    let (Some(w), h) = (twips(dom, pg_sz, "w"), twips(dom, pg_sz, "h")) else {
        return LETTER;
    };
    if w == 12240.0 && h == Some(15840.0) {
        return LETTER;
    }
    let left = twips(dom, pg_mar, "left").unwrap_or(1440.0);
    let right = twips(dom, pg_mar, "right").unwrap_or(1440.0);
    let width = ((w - left - right) / 20.0 * 0.9).max(36.0);
    let width = (width * 100.0).round() / 100.0;
    (width, width / 4.0)
}

/// Whether a header already holds a text or picture watermark.
fn holds_watermark(dom: &Dom, root: NodeId) -> bool {
    let shape = XNamespace::get(VML_URI).name("shape");
    let named = |value: Option<&str>| {
        value.is_some_and(|v| WATERMARK_IDS.iter().any(|id| v.starts_with(id)))
    };
    dom.descendants(root, Some(&shape))
        .into_iter()
        .any(|s| named(dom.attribute(s, &crate::xmllinq::XName::get("id", ""))))
        || dom
            .descendants(root, None)
            .into_iter()
            .filter(|&n| dom.name(n).is_some_and(|q| q.local_name() == "docPr"))
            .any(|n| named(dom.attribute(n, &crate::xmllinq::XName::get("name", ""))))
}

impl Transaction<'_> {
    /// Resolve a `watermark` operation: check its parameters and that the
    /// document holds no watermark yet.
    pub(super) fn resolve_watermark(
        &self,
        id: &str,
        params: (&str, Option<&str>, Option<bool>, Option<&str>),
    ) -> Result<(Spec, EditOutcome), Box<(EditError, EditOutcome)>> {
        let outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let (text, color, diagonal, font) = params;
        let spec = match spec(text, color, diagonal, font) {
            Ok(spec) => spec,
            Err(msg) => return Err(fail("INVALID_PLAN", msg, outcome)),
        };
        let marked = self.watermark.is_some()
            || self.stories[1..]
                .iter()
                .filter(|s| self.opened.dom.name_is(s.root, &W::name("hdr")))
                .any(|s| holds_watermark(&self.opened.dom, s.root));
        if marked {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                "the document already has a watermark; remove the existing watermark first".into(),
                outcome,
            ));
        }
        let mut outcome = outcome;
        outcome.matches = 1;
        outcome.context = Some(format!("{{+watermark {}}}", spec.text));
        Ok((spec, outcome))
    }

    /// Every section's `w:sectPr` in document order (paragraph section
    /// breaks, then the body's own), leaving out tracked old properties.
    fn section_properties(&self) -> Vec<NodeId> {
        let dom = &self.opened.dom;
        let body = self.stories[0].root;
        dom.descendants(body, Some(&W::sect_pr()))
            .into_iter()
            .filter(|&s| {
                dom.parent(s)
                    .is_some_and(|p| p == body || dom.name_is(p, &W::p_pr()))
            })
            .collect()
    }

    /// The story index of the default header a section references, if any.
    fn default_header(&self, sect_pr: NodeId) -> Option<Option<usize>> {
        let dom = &self.opened.dom;
        let reference = dom
            .elements(sect_pr, Some(&W::name("headerReference")))
            .into_iter()
            .find(|&r| dom.attribute(r, &W::name("type")) == Some("default"))?;
        let rid = dom.attribute(reference, &R::name("id"))?;
        let main = &self.opened.main;
        let part = self
            .opened
            .pkg
            .read_rels_for(main)?
            .items
            .iter()
            .find(|rel| rel.id == rid)
            .map(|rel| self.opened.pkg.resolve_rel_target(main, &rel.target))?;
        Some(self.stories.iter().position(|s| s.part == part))
    }

    /// Write the resolved watermark into every default header; returns the
    /// header stories it changed (a new header part is written here).
    pub(super) fn apply_watermark(&mut self) -> Result<BTreeSet<usize>, EditError> {
        let mut touched = BTreeSet::new();
        let Some(spec) = self.watermark.clone() else {
            return Ok(touched);
        };
        let mut n = 0;
        for (index, sect_pr) in self.section_properties().into_iter().enumerate() {
            let size = shape_size(&self.opened.dom, sect_pr);
            match self.default_header(sect_pr) {
                Some(Some(story)) => {
                    if touched.insert(story) {
                        n += 1;
                        self.mark_header(story, &markup(&spec, n, size))?;
                    }
                }
                // A reference to a part the package lacks: nothing to mark.
                Some(None) => {}
                // A later section without its own header inherits the
                // previous one, which is already marked.
                None if index > 0 => {}
                None => {
                    n += 1;
                    self.new_header(sect_pr, &markup(&spec, n, size));
                }
            }
        }
        Ok(touched)
    }

    /// Put the watermark first in an existing header story.
    fn mark_header(&mut self, story: usize, markup: &str) -> Result<(), EditError> {
        let StoryPart { root, ref part, .. } = self.stories[story];
        let dom = &mut self.opened.dom;
        let fragment = dom.parse_xdocument(&format!("{}{markup}</w:hdr>", header_open()));
        let sdt = dom
            .root(fragment)
            .and_then(|r| dom.element(r, &W::sdt()))
            .ok_or_else(|| err("INVALID_DOCUMENT", None, format!("{part}: watermark")))?;
        dom.remove(sdt);
        dom.add_first(root, sdt);
        let xmlns = XNamespace::xmlns();
        for (prefix, uri) in [("v", VML_URI), ("o", O_URI), ("w10", W10_URI)] {
            if dom.attribute(root, &xmlns.name(prefix)).is_none() {
                dom.set_attribute_value(root, &xmlns.name(prefix), Some(uri));
            }
        }
        Ok(())
    }

    /// Create `headerN.xml` beside the main part (`word/` in every package
    /// Word writes) holding only the watermark, relate it to the main part
    /// and reference it first in the section's properties.
    fn new_header(&mut self, sect_pr: NodeId, markup: &str) {
        let dir = match self.opened.main.rsplit_once('/') {
            Some((dir, _)) => format!("{dir}/"),
            None => String::new(),
        };
        let parts: BTreeSet<String> = self.opened.pkg.parts().into_iter().collect();
        let n = (1..)
            .find(|n| !parts.contains(&format!("{dir}header{n}.xml")))
            .unwrap_or(1);
        let name = format!("header{n}.xml");
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\r\n{}{markup}</w:hdr>",
            header_open()
        );
        let pkg = &mut self.opened.pkg;
        pkg.set_part(&format!("{dir}{name}"), xml.into_bytes());
        pkg.add_content_type_override(&format!("/{dir}{name}"), HEADER_CT);
        let rid = pkg.add_document_relationship(&self.opened.main, HEADER_REL, &name);
        let dom = &mut self.opened.dom;
        let reference = dom.new_element(W::name("headerReference"));
        dom.set_attribute_value(reference, &W::name("type"), Some("default"));
        dom.set_attribute_value(reference, &R::name("id"), Some(&rid));
        dom.add_first(sect_pr, reference);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_word() {
        let s = spec("DRAFT", None, None, None).unwrap();
        assert_eq!(
            (s.color.as_str(), s.diagonal, s.font.as_str()),
            ("C0C0C0", true, "Calibri")
        );
    }

    #[test]
    fn points_drop_trailing_zeros() {
        assert_eq!(points(12.3), "12.3");
        assert_eq!(points(406.17), "406.17");
        assert_eq!(points(100.0), "100");
    }

    #[test]
    fn horizontal_markup_has_no_rotation() {
        let s = spec("X", Some("00ff00"), Some(false), None).unwrap();
        let m = markup(&s, 2, LETTER);
        assert!(!m.contains("rotation"));
        assert!(m.contains(r##"fillcolor="#00FF00""##));
        assert!(m.contains(r#"o:spid="_x0000_s2050""#));
    }
}
