// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte audit`: accessibility, style and structure findings with the
//! paragraph id an agent can act on.
//!
//! Every rule reads the admitted package through [`crate::inspect`]'s read
//! model, so a finding's `location` is the same `body:p:N` (or story
//! paragraph id) that `jubarte inspect` prints and an edit plan targets.
//! When a finding has no paragraph (a document-wide setting), the location
//! is the part name.
//!
//! Two rules need a layout pass: `FONT_SUBSTITUTED` reads the renderer's font
//! resolutions, and `STALE_FIELD_CACHE` compares a cached `NUMPAGES` result
//! with the laid-out page count. The pass runs only when one of them is
//! selected and has something to read, and it lays out in
//! [`RevisionStyle::Word`] because the cached page count it is checked
//! against is Word's. [`AuditReport::layout`] says whether it ran.
//! [`audit_report_with`] takes the pass from the caller, or none, for builds
//! that leave the renderer out.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Serialize;

use crate::convert::{PdfOptions, RenderReport, RevisionStyle, docx_render_report};
use crate::inspect::{
    InspectError, Opened, body_paragraph_nodes, parse_part, project_paragraph,
    story_paragraph_nodes,
};
use crate::namespaces::{MC, W, WP};
use crate::xmllinq::{Dom, NodeId, XName};

/// Every rule: `(code, rule set, severity)`, in report order.
pub const RULES: &[(&str, &str, &str)] = &[
    ("HEADING_SKIP", "a11y", "warning"),
    ("IMAGE_NO_DESCR", "a11y", "error"),
    ("TABLE_NO_HEADER_ROW", "a11y", "warning"),
    ("MISSING_LANG", "a11y", "warning"),
    ("LITERAL_BULLET", "style", "warning"),
    ("EMPTY_SPACER_PARAGRAPH", "style", "info"),
    ("DIRECT_FORMATTING_OVERRIDES_STYLE", "style", "info"),
    ("STALE_FIELD_CACHE", "structure", "warning"),
    ("FONT_SUBSTITUTED", "structure", "info"),
];

/// The rule sets [`RULES`] groups codes into.
const RULE_SETS: &[&str] = &["a11y", "style", "structure"];

/// Characters typed as list markers instead of using Word numbering.
const LITERAL_MARKERS: &[char] = &['•', '◦', '▪', '-', '*', '·'];

/// One audit finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuditFinding {
    /// Rule code.
    pub code: String,
    /// `a11y`, `style` or `structure`.
    pub rule_set: String,
    /// `error`, `warning` or `info`.
    pub severity: String,
    /// `body:p:N`, a story paragraph id, or a part name.
    pub location: String,
    /// What was found, in one sentence.
    pub message: String,
}

/// Findings plus what ran to produce them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AuditReport {
    /// Findings, rule by rule in [`RULES`] order, each in document order.
    pub findings: Vec<AuditFinding>,
    /// The codes that ran.
    pub rules: Vec<String>,
    /// Whether a layout pass ran (`FONT_SUBSTITUTED`, or a numeric
    /// `NUMPAGES` cache for `STALE_FIELD_CACHE`).
    pub layout: bool,
}

/// Why an audit could not run.
#[derive(Debug)]
pub enum AuditError {
    /// The package could not be admitted or read.
    Inspect(InspectError),
    /// The layout pass failed.
    Layout(String),
    /// A requested rule is neither a rule code nor a rule set.
    UnknownRule(String),
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inspect(error) => write!(f, "{error}"),
            Self::Layout(error) => write!(f, "layout pass failed: {error}"),
            Self::UnknownRule(rule) => {
                let codes: Vec<&str> = RULES.iter().map(|rule| rule.0).collect();
                write!(
                    f,
                    "unknown audit rule {rule}; expected one of {} or {}",
                    RULE_SETS.join(", "),
                    codes.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for AuditError {}

impl From<InspectError> for AuditError {
    fn from(error: InspectError) -> Self {
        Self::Inspect(error)
    }
}

/// Audit `docx` with `rules` (rule sets or codes; empty selects every rule).
pub fn audit(docx: &[u8], rules: &[&str]) -> Result<Vec<AuditFinding>, AuditError> {
    Ok(audit_report(docx, rules)?.findings)
}

/// [`audit`] plus the codes that ran and whether a layout pass ran.
pub fn audit_report(docx: &[u8], rules: &[&str]) -> Result<AuditReport, AuditError> {
    let mut word_layout = |docx: &[u8]| {
        let options = PdfOptions {
            revisions: RevisionStyle::Word,
            ..PdfOptions::default()
        };
        docx_render_report(docx, options).map_err(|error| AuditError::Layout(error.to_string()))
    };
    audit_report_with(docx, rules, Some(&mut word_layout))
}

/// A layout pass [`audit_report_with`] calls at most once.
pub type LayoutPass<'a> = &'a mut dyn FnMut(&[u8]) -> Result<RenderReport, AuditError>;

/// [`audit_report`] with the layout pass supplied by the caller, so a build
/// without the renderer (the slim WASM build) can audit without linking it.
///
/// Without a pass (`None`), `FONT_SUBSTITUTED` is left out of the default
/// selection and asking for it is [`AuditError::Layout`];
/// `STALE_FIELD_CACHE` still reports empty TOC and `NUMPAGES` caches but
/// cannot compare a cached page count with a layout.
pub fn audit_report_with(
    docx: &[u8],
    rules: &[&str],
    mut layout: Option<LayoutPass<'_>>,
) -> Result<AuditReport, AuditError> {
    let selected = select(rules, layout.is_some())?;
    let opened = Opened::open(docx)?;
    let package = Package::read(opened)?;
    let mut findings = Vec::new();
    let mut laid_out = None;
    for &(code, _, _) in &selected {
        match code {
            "HEADING_SKIP" => package.heading_skip(&mut findings),
            "IMAGE_NO_DESCR" => package.image_no_descr(&mut findings),
            "TABLE_NO_HEADER_ROW" => package.table_no_header_row(&mut findings),
            "MISSING_LANG" => package.missing_lang(&mut findings),
            "LITERAL_BULLET" => package.literal_bullet(&mut findings),
            "EMPTY_SPACER_PARAGRAPH" => package.empty_spacers(&mut findings),
            "DIRECT_FORMATTING_OVERRIDES_STYLE" => package.direct_formatting(&mut findings),
            "STALE_FIELD_CACHE" => {
                let fields = package.fields();
                let numeric = fields
                    .iter()
                    .any(|field| field.keyword == "NUMPAGES" && field.number().is_some());
                let pages = match layout.as_deref_mut() {
                    Some(pass) if numeric => Some(lay_out(docx, pass, &mut laid_out)?.page_count),
                    _ => None,
                };
                stale_fields(&fields, pages, &mut findings);
            }
            _ => {
                let pass = layout
                    .as_deref_mut()
                    .expect("select keeps FONT_SUBSTITUTED only with a layout pass");
                let fonts = &lay_out(docx, pass, &mut laid_out)?.fonts;
                let mut seen = BTreeSet::new();
                for font in fonts.iter().filter(|font| font.substituted()) {
                    if seen.insert(font.requested.clone()) {
                        findings.push(finding(
                            code,
                            package.main.clone(),
                            format!(
                                "font \"{}\" is not installed here and is drawn with {} ({})",
                                font.requested,
                                font.physical,
                                font.step.as_str()
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(AuditReport {
        findings,
        rules: selected.iter().map(|rule| rule.0.to_string()).collect(),
        layout: laid_out.is_some(),
    })
}

/// The rules `requested` names, in [`RULES`] order. Without a layout pass
/// the default leaves out `FONT_SUBSTITUTED`, and naming it is an error.
fn select(
    requested: &[&str],
    can_lay_out: bool,
) -> Result<Vec<(&'static str, &'static str, &'static str)>, AuditError> {
    let names = |rule: &(&str, &str, &str), item: &str| rule.0 == item || rule.1 == item;
    let mut wanted = BTreeSet::new();
    for item in requested.iter().map(|item| item.trim()) {
        if !RULES.iter().any(|rule| names(rule, item)) {
            return Err(AuditError::UnknownRule(item.to_string()));
        }
        wanted.extend(
            RULES
                .iter()
                .filter(|rule| names(rule, item))
                .map(|rule| rule.0),
        );
    }
    if requested.is_empty() {
        wanted.extend(RULES.iter().map(|rule| rule.0));
        if !can_lay_out {
            wanted.remove("FONT_SUBSTITUTED");
        }
    } else if !can_lay_out && wanted.contains("FONT_SUBSTITUTED") {
        return Err(AuditError::Layout(
            "FONT_SUBSTITUTED needs a layout pass, which this build does not have".to_string(),
        ));
    }
    Ok(RULES
        .iter()
        .filter(|rule| wanted.contains(rule.0))
        .copied()
        .collect())
}

/// The layout report, computed by `pass` on first use.
fn lay_out<'a>(
    docx: &[u8],
    pass: &mut dyn FnMut(&[u8]) -> Result<RenderReport, AuditError>,
    slot: &'a mut Option<RenderReport>,
) -> Result<&'a RenderReport, AuditError> {
    if slot.is_none() {
        *slot = Some(pass(docx)?);
    }
    Ok(slot.as_ref().expect("layout report was just stored"))
}

fn finding(code: &str, location: String, message: String) -> AuditFinding {
    let &(code, rule_set, severity) = RULES
        .iter()
        .find(|rule| rule.0 == code)
        .expect("finding codes come from RULES");
    AuditFinding {
        code: code.to_string(),
        rule_set: rule_set.to_string(),
        severity: severity.to_string(),
        location,
        message,
    }
}

fn on(value: Option<&str>) -> bool {
    !matches!(value, Some("0" | "false" | "off"))
}

/// One parsed part with its addressable paragraphs.
struct Story {
    /// `body`, or the story id (`header1`, `footnotes`, ...).
    id: String,
    /// Part name, the location when no paragraph applies.
    part: String,
    dom: Dom,
    root: NodeId,
    paragraphs: Vec<NodeId>,
}

impl Story {
    /// The id of the indexed paragraph holding `node` (the nearest one,
    /// so an image in a text box reports the paragraph that anchors it).
    fn location(&self, node: NodeId) -> String {
        std::iter::once(node)
            .chain(self.dom.ancestors(node, Some(&W::p())))
            .find_map(|p| self.paragraphs.iter().position(|&q| q == p))
            .map_or_else(|| self.part.clone(), |index| self.paragraph_id(index))
    }

    fn paragraph_id(&self, index: usize) -> String {
        format!("{}:p:{index}", self.id)
    }
}

/// A style's facts the rules read.
#[derive(Default)]
struct StyleFacts {
    name: Option<String>,
    based_on: Option<String>,
    numbering: Option<bool>,
    font: Option<String>,
    size: Option<String>,
    lang: bool,
}

/// The package, read once.
struct Package {
    main: String,
    stories: Vec<Story>,
    styles_part: Option<String>,
    styles: BTreeMap<String, StyleFacts>,
    default_paragraph_style: String,
    defaults_font: Option<String>,
    defaults_size: Option<String>,
    defaults_lang: bool,
    theme_font_lang: bool,
}

impl Package {
    fn read(opened: Opened) -> Result<Self, AuditError> {
        let story_parts = opened.story_parts();
        let styles_part = opened.related("styles").into_iter().next();
        let settings_part = opened.related("settings").into_iter().next();
        let Opened {
            pkg,
            main,
            dom,
            body,
            ..
        } = opened;
        let mut stories = vec![Story {
            id: "body".to_string(),
            part: main.clone(),
            paragraphs: body_paragraph_nodes(&dom, body),
            dom,
            root: body,
        }];
        for (id, _, part) in story_parts {
            let (dom, _, root) = parse_part(&pkg, &part)?;
            stories.push(Story {
                id,
                paragraphs: story_paragraph_nodes(&dom, root),
                part,
                dom,
                root,
            });
        }
        let mut package = Self {
            main,
            stories,
            styles_part,
            styles: BTreeMap::new(),
            default_paragraph_style: "Normal".to_string(),
            defaults_font: None,
            defaults_size: None,
            defaults_lang: false,
            theme_font_lang: false,
        };
        if let Some(part) = package.styles_part.clone() {
            let (dom, _, root) = parse_part(&pkg, &part)?;
            package.read_styles(&dom, root);
        }
        if let Some(part) = settings_part {
            let (dom, _, root) = parse_part(&pkg, &part)?;
            package.theme_font_lang = dom
                .element(root, &W::name("themeFontLang"))
                .is_some_and(|lang| has_lang(&dom, lang));
        }
        Ok(package)
    }

    fn read_styles(&mut self, dom: &Dom, root: NodeId) {
        if let Some(defaults) = dom.element(root, &W::name("docDefaults")) {
            let rpr = dom
                .element(defaults, &W::name("rPrDefault"))
                .and_then(|default| dom.element(default, &W::r_pr()));
            if let Some(rpr) = rpr {
                (self.defaults_font, self.defaults_size) = run_font_and_size(dom, rpr);
            }
            self.defaults_lang = dom
                .descendants(defaults, Some(&W::name("lang")))
                .into_iter()
                .any(|lang| has_lang(dom, lang));
        }
        let mut marked_default = None;
        let mut named_normal = None;
        for style in dom.elements(root, Some(&W::name("style"))) {
            let Some(id) = dom.attribute(style, &W::name("styleId")) else {
                continue;
            };
            let paragraph = dom
                .attribute(style, &W::name("type"))
                .unwrap_or("paragraph")
                == "paragraph";
            if !paragraph {
                continue;
            }
            if dom
                .attribute(style, &W::name("default"))
                .is_some_and(|value| matches!(value, "1" | "true" | "on"))
            {
                marked_default.get_or_insert_with(|| id.to_string());
            }
            let child_val = |name: &str| {
                dom.element(style, &W::name(name))
                    .and_then(|node| dom.attribute(node, &W::val()))
                    .map(str::to_string)
            };
            let mut facts = StyleFacts {
                name: child_val("name"),
                based_on: child_val("basedOn"),
                numbering: dom
                    .element(style, &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::num_pr()))
                    .map(|num_pr| numbering_on(dom, num_pr).unwrap_or(true)),
                ..StyleFacts::default()
            };
            if let Some(rpr) = dom.element(style, &W::r_pr()) {
                (facts.font, facts.size) = run_font_and_size(dom, rpr);
                facts.lang = dom
                    .elements(rpr, Some(&W::name("lang")))
                    .into_iter()
                    .any(|lang| has_lang(dom, lang));
            }
            if facts
                .name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case("normal"))
            {
                named_normal.get_or_insert_with(|| id.to_string());
            }
            self.styles.insert(id.to_string(), facts);
        }
        // The style marked default; failing that the one named Normal
        // (LibreOffice writes it as `style0` without `w:default`).
        if let Some(id) = marked_default.or(named_normal) {
            self.default_paragraph_style = id;
        }
    }

    /// Paragraph styles from `id` up its `basedOn` chain (cycles cut).
    fn style_chain(&self, id: &str) -> Vec<&StyleFacts> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut next = Some(id.to_string());
        while let Some(id) = next {
            if !seen.insert(id.clone()) {
                break;
            }
            let Some(facts) = self.styles.get(&id) else {
                break;
            };
            chain.push(facts);
            next = facts.based_on.clone();
        }
        chain
    }

    fn paragraph_style<'a>(&self, dom: &'a Dom, p: NodeId) -> Option<&'a str> {
        dom.element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::p_style()))
            .and_then(|style| dom.attribute(style, &W::val()))
    }

    /// The heading level of a paragraph style: its `w:name` is
    /// `heading N` (how Word names them in every UI language), else its id
    /// is `HeadingN`.
    fn heading_level(&self, style: &str) -> Option<u32> {
        let from_name = self
            .styles
            .get(style)
            .and_then(|facts| facts.name.as_deref())
            .and_then(|name| level_after(name, "heading "));
        from_name.or_else(|| level_after(style, "heading"))
    }

    /// Direct `w:numPr` decides (a `numId` of 0 removes numbering);
    /// otherwise the style chain's.
    fn numbered(&self, dom: &Dom, p: NodeId) -> bool {
        let direct = dom
            .element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::num_pr()))
            .and_then(|num_pr| numbering_on(dom, num_pr));
        direct.unwrap_or_else(|| {
            self.paragraph_style(dom, p).is_some_and(|style| {
                self.style_chain(style)
                    .into_iter()
                    .find_map(|facts| facts.numbering)
                    .unwrap_or(false)
            })
        })
    }

    fn body(&self) -> &Story {
        &self.stories[0]
    }

    fn heading_skip(&self, findings: &mut Vec<AuditFinding>) {
        let body = self.body();
        let mut previous: Option<u32> = None;
        for (index, &p) in body.paragraphs.iter().enumerate() {
            let Some(level) = self
                .paragraph_style(&body.dom, p)
                .and_then(|style| self.heading_level(style))
            else {
                continue;
            };
            let message = match previous {
                None if level > 1 => {
                    Some(format!("the first heading is level {level}, not level 1"))
                }
                Some(before) if level > before + 1 => Some(format!(
                    "a level {level} heading follows a level {before} heading"
                )),
                _ => None,
            };
            if let Some(message) = message {
                findings.push(finding("HEADING_SKIP", body.paragraph_id(index), message));
            }
            previous = Some(level);
        }
    }

    fn image_no_descr(&self, findings: &mut Vec<AuditFinding>) {
        let descr = XName::get("descr", "");
        for story in &self.stories {
            let dom = &story.dom;
            for doc_pr in dom.descendants(story.root, Some(&WP::name("docPr"))) {
                if under_fallback(dom, doc_pr) {
                    continue;
                }
                let described = dom
                    .attribute(doc_pr, &descr)
                    .is_some_and(|text| !text.trim().is_empty());
                if described || decorative(dom, doc_pr) {
                    continue;
                }
                let name = dom
                    .attribute(doc_pr, &XName::get("name", ""))
                    .unwrap_or("unnamed");
                findings.push(finding(
                    "IMAGE_NO_DESCR",
                    story.location(doc_pr),
                    format!(
                        "drawing \"{name}\" has no alternative text and is not marked decorative"
                    ),
                ));
            }
        }
    }

    fn table_no_header_row(&self, findings: &mut Vec<AuditFinding>) {
        for story in &self.stories {
            let dom = &story.dom;
            for table in dom.descendants(story.root, Some(&W::tbl())) {
                let rows = dom.elements(table, Some(&W::tr()));
                if rows.len() < 2 {
                    continue;
                }
                let header = dom
                    .element(rows[0], &W::tr_pr())
                    .and_then(|tr_pr| dom.element(tr_pr, &W::name("tblHeader")))
                    .is_some_and(|header| on(dom.attribute(header, &W::val())));
                if header {
                    continue;
                }
                let location = dom
                    .descendants(table, Some(&W::p()))
                    .first()
                    .map_or_else(|| story.part.clone(), |&p| story.location(p));
                findings.push(finding(
                    "TABLE_NO_HEADER_ROW",
                    location,
                    format!(
                        "a {}-row table does not mark its first row as a header row",
                        rows.len()
                    ),
                ));
            }
        }
    }

    fn missing_lang(&self, findings: &mut Vec<AuditFinding>) {
        let normal = self
            .style_chain(&self.default_paragraph_style)
            .into_iter()
            .any(|facts| facts.lang);
        if self.defaults_lang || normal || self.theme_font_lang {
            return;
        }
        findings.push(finding(
            "MISSING_LANG",
            self.styles_part
                .clone()
                .unwrap_or_else(|| self.main.clone()),
            "no document language is set (docDefaults, the default paragraph style and \
             themeFontLang carry no w:lang)"
                .to_string(),
        ));
    }

    fn literal_bullet(&self, findings: &mut Vec<AuditFinding>) {
        for story in &self.stories {
            for (index, &p) in story.paragraphs.iter().enumerate() {
                let text = project_paragraph(&story.dom, p).text;
                let mut chars = text.trim_start_matches([' ', '\u{a0}']).chars();
                let (Some(marker), Some(gap)) = (chars.next(), chars.next()) else {
                    continue;
                };
                if !LITERAL_MARKERS.contains(&marker) || !matches!(gap, ' ' | '\t' | '\u{a0}') {
                    continue;
                }
                if self.numbered(&story.dom, p) {
                    continue;
                }
                findings.push(finding(
                    "LITERAL_BULLET",
                    story.paragraph_id(index),
                    format!(
                        "the paragraph starts with a typed \"{marker}\" instead of list numbering"
                    ),
                ));
            }
        }
    }

    fn empty_spacers(&self, findings: &mut Vec<AuditFinding>) {
        for story in &self.stories {
            let dom = &story.dom;
            let breaks_section = |p: NodeId| {
                dom.element(p, &W::p_pr())
                    .is_some_and(|ppr| dom.element(ppr, &W::sect_pr()).is_some())
            };
            let empty = |p: NodeId| {
                if !dom.ancestors(p, Some(&W::tc())).is_empty() {
                    return false;
                }
                let projection = project_paragraph(dom, p);
                projection.text.trim().is_empty()
                    && !projection.page_break
                    && projection.limitations.is_empty()
            };
            let mut index = 0;
            while index < story.paragraphs.len() {
                let first = story.paragraphs[index];
                if !empty(first) {
                    index += 1;
                    continue;
                }
                let mut end = index + 1;
                while end < story.paragraphs.len()
                    && dom.parent(story.paragraphs[end]) == dom.parent(first)
                    && empty(story.paragraphs[end])
                    && !breaks_section(story.paragraphs[end - 1])
                {
                    end += 1;
                }
                let count = end - index;
                let last = story.paragraphs[end - 1];
                let before_break = breaks_section(last)
                    || story.paragraphs.get(end).is_some_and(|&next| {
                        dom.parent(next) == dom.parent(first) && breaks_section(next)
                    });
                if count >= 2 && !before_break {
                    findings.push(finding(
                        "EMPTY_SPACER_PARAGRAPH",
                        story.paragraph_id(index),
                        format!(
                            "{count} consecutive empty paragraphs ({} to {}) are used as vertical space",
                            story.paragraph_id(index),
                            story.paragraph_id(end - 1)
                        ),
                    ));
                }
                index = end;
            }
        }
    }

    fn direct_formatting(&self, findings: &mut Vec<AuditFinding>) {
        let chain = self.style_chain(&self.default_paragraph_style);
        let font = chain
            .iter()
            .find_map(|facts| facts.font.clone())
            .or_else(|| self.defaults_font.clone());
        let size = chain
            .iter()
            .find_map(|facts| facts.size.clone())
            .or_else(|| self.defaults_size.clone());
        let body = self.body();
        let dom = &body.dom;
        let (mut runs, mut overrides) = (0usize, 0usize);
        for &p in &body.paragraphs {
            let default_style = self
                .paragraph_style(dom, p)
                .is_none_or(|style| style == self.default_paragraph_style);
            if !default_style {
                continue;
            }
            for run in dom.descendants(p, Some(&W::r())) {
                let own = dom.ancestors(run, Some(&W::p())).first() == Some(&p);
                if !own || dom.element(run, &W::t()).is_none() {
                    continue;
                }
                runs += 1;
                let Some(rpr) = dom.element(run, &W::r_pr()) else {
                    continue;
                };
                let (run_font, run_size) = run_font_and_size(dom, rpr);
                let font_differs = run_font.is_some() && run_font != font;
                let size_differs = run_size.is_some() && run_size != size;
                if font_differs || size_differs {
                    overrides += 1;
                }
            }
        }
        if runs > 0 && overrides * 10 > runs * 3 {
            findings.push(finding(
                "DIRECT_FORMATTING_OVERRIDES_STYLE",
                self.main.clone(),
                format!(
                    "{overrides} of {runs} runs in default-style paragraphs set a font or size \
                     that differs from the style"
                ),
            ));
        }
    }

    /// Every complex and simple field, story by story in document order.
    fn fields(&self) -> Vec<Field> {
        let mut out = Vec::new();
        for (story_index, story) in self.stories.iter().enumerate() {
            let dom = &story.dom;
            let mut open: Vec<Field> = Vec::new();
            for node in dom.descendants(story.root, None) {
                if under_fallback(dom, node) {
                    continue;
                }
                if dom.name_is(node, &W::fld_char()) {
                    match dom.attribute(node, &W::name("fldCharType")) {
                        Some("begin") => open.push(Field::new(story_index, node)),
                        Some("separate") => {
                            if let Some(field) = open.last_mut() {
                                field.separated = true;
                            }
                        }
                        Some("end") => {
                            if let Some(mut field) = open.pop() {
                                field.finish(self);
                                out.push(field);
                            }
                        }
                        _ => {}
                    }
                } else if dom.name_is(node, &W::name("instrText")) {
                    if let Some(field) = open.last_mut().filter(|field| !field.separated) {
                        field.instruction.push_str(&dom.value_str(node));
                    }
                } else if dom.name_is(node, &W::t()) {
                    let text = dom.value_str(node);
                    for field in open.iter_mut().filter(|field| field.separated) {
                        field.result.push_str(&text);
                    }
                } else if dom.name_is(node, &W::fld_simple()) {
                    let mut field = Field::new(story_index, node);
                    field.instruction = dom
                        .attribute(node, &W::name("instr"))
                        .unwrap_or_default()
                        .to_string();
                    field.separated = true;
                    field.result = dom
                        .descendants(node, Some(&W::t()))
                        .into_iter()
                        .map(|t| dom.value(t))
                        .collect();
                    field.finish(self);
                    out.push(field);
                }
            }
            for mut field in open.into_iter().rev() {
                field.finish(self);
                out.push(field);
            }
        }
        out
    }
}

/// One field's instruction and cached result.
struct Field {
    story: usize,
    node: NodeId,
    instruction: String,
    separated: bool,
    result: String,
    /// Upper-cased first word of the instruction, set by `finish`.
    keyword: String,
    location: String,
}

impl Field {
    fn new(story: usize, node: NodeId) -> Self {
        Self {
            story,
            node,
            instruction: String::new(),
            separated: false,
            result: String::new(),
            keyword: String::new(),
            location: String::new(),
        }
    }

    fn finish(&mut self, package: &Package) {
        self.keyword = self
            .instruction
            .split_whitespace()
            .next()
            .unwrap_or("-")
            .to_ascii_uppercase();
        self.location = package.stories[self.story].location(self.node);
    }

    fn cached(&self) -> bool {
        self.separated && !self.result.trim().is_empty()
    }

    fn number(&self) -> Option<usize> {
        self.cached()
            .then(|| self.result.trim().parse().ok())
            .flatten()
    }
}

fn stale_fields(fields: &[Field], pages: Option<usize>, findings: &mut Vec<AuditFinding>) {
    for field in fields {
        let message = match field.keyword.as_str() {
            "TOC" if !field.cached() => {
                Some("the table of contents has no cached entries; update fields".to_string())
            }
            "NUMPAGES" if !field.cached() => {
                Some("the NUMPAGES field has no cached page count; update fields".to_string())
            }
            "NUMPAGES" => match (field.number(), pages) {
                (Some(cached), Some(pages)) if cached != pages => Some(format!(
                    "the NUMPAGES field caches {cached} pages but the document lays out to {pages}"
                )),
                _ => None,
            },
            _ => None,
        };
        if let Some(message) = message {
            findings.push(finding(
                "STALE_FIELD_CACHE",
                field.location.clone(),
                message,
            ));
        }
    }
}

/// `rFonts` (`ascii`, else `hAnsi`) and `sz` of an `rPr`.
fn run_font_and_size(dom: &Dom, rpr: NodeId) -> (Option<String>, Option<String>) {
    let font = dom.element(rpr, &W::name("rFonts")).and_then(|fonts| {
        dom.attribute(fonts, &W::name("ascii"))
            .or_else(|| dom.attribute(fonts, &W::name("hAnsi")))
            .map(str::to_string)
    });
    let size = dom
        .element(rpr, &W::name("sz"))
        .and_then(|sz| dom.attribute(sz, &W::val()))
        .map(str::to_string);
    (font, size)
}

/// A `w:lang`-shaped element naming any language.
fn has_lang(dom: &Dom, lang: NodeId) -> bool {
    ["val", "eastAsia", "bidi"].iter().any(|attribute| {
        dom.attribute(lang, &W::name(attribute))
            .is_some_and(|value| !value.trim().is_empty())
    })
}

/// `Some(false)` when a `numPr` names `numId` 0 (numbering removed),
/// `Some(true)` for any other `numId`, `None` when it names none.
fn numbering_on(dom: &Dom, num_pr: NodeId) -> Option<bool> {
    dom.element(num_pr, &W::name("numId"))
        .and_then(|num_id| dom.attribute(num_id, &W::val()))
        .map(|id| id.trim() != "0")
}

/// `N` from `{prefix}N` (case-insensitive), for heading levels 1-9.
fn level_after(text: &str, prefix: &str) -> Option<u32> {
    let head = text.get(..prefix.len())?;
    if !head.eq_ignore_ascii_case(prefix) {
        return None;
    }
    text[prefix.len()..]
        .trim()
        .parse()
        .ok()
        .filter(|level| (1..=9).contains(level))
}

/// The Office 2019 decorative flag (`adec:decorative val="1"` in the
/// `docPr`'s extension list). Matched by local name: OOXML defines no
/// `w:decorative`.
fn decorative(dom: &Dom, doc_pr: NodeId) -> bool {
    dom.descendants(doc_pr, None).into_iter().any(|node| {
        dom.name(node)
            .is_some_and(|name| name.local_name() == "decorative")
            && matches!(
                dom.attribute(node, &XName::get("val", "")),
                Some("1" | "true")
            )
    })
}

/// Inside an `mc:Fallback`, which repeats its `mc:Choice` for old readers.
fn under_fallback(dom: &Dom, node: NodeId) -> bool {
    !dom.ancestors(node, Some(&MC::name("Fallback"))).is_empty()
}
