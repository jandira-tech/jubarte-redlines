// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Remove who touched a document before it goes out, and find where a text
//! still occurs in one.
//!
//! [`scrub`] works on four kinds of identifying data, each one optional:
//!
//! - **Author alias.** Every `w:author` in the `word/` parts (tracked
//!   changes, formatting changes, comments) becomes the alias, comment
//!   initials become the alias's initials, and `people.xml` keeps a single
//!   person, the alias, without presence information (e-mail, provider).
//! - **rsids.** Every `w:rsid*` attribute and the settings' `w:rsids` list:
//!   the edit-session ids that tie copies of a document to one another.
//! - **Document properties.** Core properties lose the creator, the last
//!   editor, the revision number and the created, modified and printed
//!   dates; extended properties lose the manager and company; the custom
//!   properties part goes with its relationship and content type. The
//!   title, subject and application stay.
//! - **Comments.** Every comment reference and range goes, then the comment
//!   parts and `people.xml` with them.
//!
//! Text, tracked changes and formatting stay. Revision dates stay.
//!
//! [`leaks`] is the check behind the `redact` plan operation: the parts of a
//! package where a text still occurs.

use std::collections::BTreeSet;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::markup_simplifier::remove_rsid_transform;
use crate::namespaces::{W, W15};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

const CORE: &str = "docProps/core.xml";
const APP: &str = "docProps/app.xml";
const CUSTOM: &str = "docProps/custom.xml";
const CUSTOM_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
const CP: &str = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
const DC: &str = "http://purl.org/dc/elements/1.1/";
const DCTERMS: &str = "http://purl.org/dc/terms/";
const EXTENDED: &str = "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";

/// What [`scrub`] removes. A field left out of the JSON form is off; the
/// [`Default`] turns everything on with the alias `Author`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrubOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Name every author takes: revisions, comments and `people.xml`.
    pub author_alias: Option<String>,
    #[serde(default)]
    /// Remove `w:rsid*` attributes and the settings' `w:rsids`.
    pub rsids: bool,
    #[serde(default)]
    /// Remove the people and dates from the document properties.
    pub docprops: bool,
    #[serde(default)]
    /// Remove every comment.
    pub comments: bool,
}

impl Default for ScrubOptions {
    fn default() -> Self {
        Self {
            author_alias: Some("Author".into()),
            rsids: true,
            docprops: true,
            comments: true,
        }
    }
}

/// Why [`scrub`] refused its input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrubError {
    /// The bytes are not an OPC package, or it cannot be written back.
    Package(String),
    /// The package has no main document, or the options are invalid.
    Invalid(String),
}

impl std::fmt::Display for ScrubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Package(message) => write!(f, "opening DOCX: {message}"),
            Self::Invalid(message) => write!(f, "cannot scrub: {message}"),
        }
    }
}

impl std::error::Error for ScrubError {}

/// Remove the identifying data `options` names from a `.docx`.
pub fn scrub(docx: &[u8], options: &ScrubOptions) -> Result<Vec<u8>, ScrubError> {
    let mut pkg = PartFs::open(docx).map_err(|e| ScrubError::Package(e.to_string()))?;
    if !pkg
        .main_document_part()
        .is_some_and(|main| pkg.part_bytes(&main).is_some())
    {
        return Err(ScrubError::Invalid(
            "the package has no main document part".into(),
        ));
    }
    if let Some(alias) = &options.author_alias
        && (alias.trim().is_empty() || alias.chars().any(char::is_control))
    {
        return Err(ScrubError::Invalid(
            "author_alias must be a nonempty name without control characters".into(),
        ));
    }
    let before = crate::validate::validate(docx).map_err(|e| ScrubError::Package(e.to_string()))?;
    if options.comments {
        remove_comments(&mut pkg);
    }
    if let Some(alias) = &options.author_alias {
        alias_authors(&mut pkg, alias);
    }
    if options.rsids {
        remove_rsids(&mut pkg);
    }
    if options.docprops {
        scrub_properties(&mut pkg);
    }
    let out = pkg
        .to_zip()
        .map_err(|e| ScrubError::Package(e.to_string()))?;
    let after = crate::validate::validate(&out).map_err(|e| ScrubError::Package(e.to_string()))?;
    let new = introduced(&before, &after);
    if !new.is_empty() {
        return Err(ScrubError::Invalid(format!(
            "scrubbing would introduce {}",
            new.join(", ")
        )));
    }
    Ok(out)
}

/// The findings `after` has more of than `before`, as `CODE in part`,
/// counted by code and part: removing elements shifts the paths of
/// findings that were already there.
fn introduced(
    before: &[crate::validate::Finding],
    after: &[crate::validate::Finding],
) -> Vec<String> {
    let had = finding_counts(before);
    finding_counts(after)
        .into_iter()
        .filter(|(key, n)| had.get(key).is_none_or(|h| n > h))
        .map(|((code, part), _)| format!("{code} in {part}"))
        .collect()
}

fn finding_counts(
    findings: &[crate::validate::Finding],
) -> std::collections::BTreeMap<(&str, &str), usize> {
    let mut counts = std::collections::BTreeMap::new();
    for f in findings {
        *counts
            .entry((f.code.as_str(), f.part.as_str()))
            .or_default() += 1;
    }
    counts
}

/// One parsed XML part, written back only when changed.
struct Part {
    name: String,
    dom: Dom,
    doc: NodeId,
    root: NodeId,
}

impl Part {
    fn load(pkg: &PartFs, name: &str) -> Option<Self> {
        let xml = pkg.part_string(name)?;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc)?;
        Some(Self {
            name: name.to_string(),
            dom,
            doc,
            root,
        })
    }

    fn store(self, pkg: &mut PartFs) {
        pkg.set_part(
            &self.name,
            self.dom.serialize_document(self.doc).into_bytes(),
        );
    }
}

/// The XML parts under `word/`.
fn word_parts(pkg: &PartFs) -> Vec<String> {
    pkg.parts()
        .into_iter()
        .filter(|p| p.starts_with("word/") && p.ends_with(".xml"))
        .collect()
}

/// Drop every comment reference and range, then the comment parts and
/// `people.xml` that no comment needs any more.
fn remove_comments(pkg: &mut PartFs) {
    let stories: Vec<String> = crate::revision_processor::revision_bearing_parts(pkg)
        .into_iter()
        .filter(|(_, is_styles)| !is_styles)
        .map(|(part, _)| part)
        .collect();
    for story in &stories {
        let Some(mut part) = Part::load(pkg, story) else {
            continue;
        };
        let references = part
            .dom
            .descendants(part.root, Some(&W::name("commentReference")));
        if references.is_empty() {
            continue;
        }
        for reference in references {
            let run = part
                .dom
                .parent(reference)
                .filter(|&r| part.dom.name_is(r, &W::r()));
            part.dom.remove(reference);
            // The reference's own run holds nothing else but properties.
            if let Some(run) = run
                && part
                    .dom
                    .elements(run, None)
                    .iter()
                    .all(|&k| part.dom.name_is(k, &W::r_pr()))
            {
                part.dom.remove(run);
            }
        }
        part.store(pkg);
    }
    crate::revision_processor::prune_orphan_comments(pkg, &stories);
}

/// The alias's initials: the first letter of each word, upper case.
fn initials(alias: &str) -> String {
    alias
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

/// Every author in the `word/` parts becomes `alias`; `people.xml` keeps
/// one person, the alias, without presence information.
fn alias_authors(pkg: &mut PartFs, alias: &str) {
    let author = W::author();
    let initials_name = W::name("initials");
    let short = initials(alias);
    for name in word_parts(pkg) {
        if name == "word/people.xml" {
            continue;
        }
        let Some(mut part) = Part::load(pkg, &name) else {
            continue;
        };
        let mut changed = false;
        for node in part.dom.descendants_and_self(part.root, None) {
            if part
                .dom
                .attribute(node, &author)
                .is_some_and(|a| a != alias)
            {
                part.dom.set_attribute_value(node, &author, Some(alias));
                changed = true;
            }
            if part
                .dom
                .attribute(node, &initials_name)
                .is_some_and(|i| i != short)
            {
                part.dom
                    .set_attribute_value(node, &initials_name, Some(&short));
                changed = true;
            }
        }
        if changed {
            part.store(pkg);
        }
    }
    let Some(mut people) = Part::load(pkg, "word/people.xml") else {
        return;
    };
    let person = W15::name("person");
    let w15_author = W15::name("author");
    let mut kept = false;
    for p in people.dom.descendants(people.root, Some(&person)) {
        if kept {
            people.dom.remove(p);
            continue;
        }
        kept = true;
        people.dom.set_attribute_value(p, &w15_author, Some(alias));
        for presence in people.dom.elements(p, Some(&W15::name("presenceInfo"))) {
            people.dom.remove(presence);
        }
    }
    people.store(pkg);
}

/// Strip `w:rsid*` attributes and `w:rsid` elements everywhere under
/// `word/`, and the settings' `w:rsids` list.
fn remove_rsids(pkg: &mut PartFs) {
    let rsids = W::name("rsids");
    for name in word_parts(pkg) {
        if !pkg.part_string(&name).is_some_and(|x| x.contains("rsid")) {
            continue;
        }
        let Some(mut part) = Part::load(pkg, &name) else {
            continue;
        };
        for list in part.dom.descendants(part.root, Some(&rsids)) {
            part.dom.remove(list);
        }
        remove_rsid_transform(&mut part.dom, part.root);
        part.store(pkg);
    }
}

/// Remove the elements named `names` that are children of `part`'s root.
fn remove_children(part: &mut Part, names: &[XName]) -> bool {
    let mut changed = false;
    for name in names {
        for node in part.dom.elements(part.root, Some(name)) {
            part.dom.remove(node);
            changed = true;
        }
    }
    changed
}

/// Drop the people and dates of the core and extended properties, and the
/// custom properties part.
fn scrub_properties(pkg: &mut PartFs) {
    if let Some(mut core) = Part::load(pkg, CORE) {
        let names = [
            XName::get("creator", DC),
            XName::get("lastModifiedBy", CP),
            XName::get("revision", CP),
            XName::get("lastPrinted", CP),
            XName::get("created", DCTERMS),
            XName::get("modified", DCTERMS),
        ];
        if remove_children(&mut core, &names) {
            core.store(pkg);
        }
    }
    if let Some(mut app) = Part::load(pkg, APP) {
        let names = [
            XName::get("Manager", EXTENDED),
            XName::get("Company", EXTENDED),
        ];
        if remove_children(&mut app, &names) {
            app.store(pkg);
        }
    }
    if pkg.part_bytes(CUSTOM).is_some() {
        pkg.remove_part(CUSTOM);
        pkg.remove_package_relationships_by_type(CUSTOM_REL);
        pkg.remove_content_type_override(&format!("/{CUSTOM}"));
    }
}

/// The parts of `docx` where `needle` still occurs, sorted: in a text node,
/// an attribute value that can hold text, an XML comment, the joined run
/// text of a paragraph (a text split across runs), or the raw bytes of a
/// part that is not XML (UTF-8 or UTF-16LE). An empty needle, or bytes that
/// are not a zip, occur nowhere.
///
/// The check fails closed: a short needle can also match an unrelated
/// attribute value. Attributes that only ever hold numbers (sizes, ids,
/// positions) are skipped when their value is a plain number.
pub fn leaks(docx: &[u8], needle: &str) -> Vec<String> {
    let mut found = BTreeSet::new();
    if needle.is_empty() {
        return Vec::new();
    }
    let Ok(mut zip) = zip::ZipArchive::new(std::io::Cursor::new(docx)) else {
        return Vec::new();
    };
    let utf16: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
    for i in 0..zip.len() {
        let Ok(mut entry) = zip.by_index(i) else {
            continue;
        };
        let name = entry.name().to_string();
        let mut bytes = Vec::new();
        if entry.read_to_end(&mut bytes).is_err() {
            continue;
        }
        let xml = name.ends_with(".xml") || name.ends_with(".rels");
        let hit = if xml {
            xml_holds(&String::from_utf8_lossy(&bytes), needle)
        } else {
            contains(&bytes, needle.as_bytes()) || contains(&bytes, &utf16)
        };
        if hit {
            found.insert(name);
        }
    }
    found.into_iter().collect()
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle)
}

/// Attributes that only ever hold a number or a hex id.
const NUMERIC_ATTRIBUTES: &[&str] = &[
    "id",
    "paraId",
    "textId",
    "durableId",
    "w",
    "h",
    "top",
    "bottom",
    "left",
    "right",
    "start",
    "end",
    "header",
    "footer",
    "gutter",
    "before",
    "after",
    "line",
    "firstLine",
    "hanging",
    "pos",
    "x",
    "y",
    "cx",
    "cy",
    "sz",
    "space",
    "num",
    "numId",
    "ilvl",
    "abstractNumId",
    "colFirst",
    "colLast",
    "percent",
];

fn is_number(value: &str) -> bool {
    let digits = value.strip_prefix('-').unwrap_or(value);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_hexdigit())
}

fn xml_holds(xml: &str, needle: &str) -> bool {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let Some(root) = dom.root(doc) else {
        // Not XML after all: read it as bytes.
        return xml.contains(needle);
    };
    for node in dom.descendant_nodes(doc) {
        if dom.is_text(node) || dom.is_comment(node) {
            if dom.text_value(node).is_some_and(|v| v.contains(needle)) {
                return true;
            }
        } else if dom.is_pi(node) {
            if dom.pi_data(node).is_some_and(|v| v.contains(needle)) {
                return true;
            }
        } else if dom.is_element(node) {
            for i in 0..dom.attr_count(node) {
                let (name, value) = dom.attr_at(node, i);
                if !value.contains(needle) || dom.is_namespace_declaration(name) {
                    continue;
                }
                if NUMERIC_ATTRIBUTES.contains(&name.local_name()) && is_number(value) {
                    continue;
                }
                return true;
            }
        }
    }
    // A text split across runs shows only in the paragraph's joined text.
    let texts = [W::t(), W::del_text(), W::name("instrText")];
    for p in dom.descendants_and_self(root, Some(&W::p())) {
        let joined: String = dom
            .descendants(p, None)
            .into_iter()
            .filter(|&n| texts.iter().any(|t| dom.name_is(n, t)))
            .map(|n| dom.value(n))
            .collect();
        if joined.contains(needle) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(code: &str, part: &str, path: &str) -> crate::validate::Finding {
        crate::validate::Finding {
            code: code.into(),
            part: part.into(),
            path: path.into(),
            message: String::new(),
            word_fatal: true,
            repairable: false,
        }
    }

    #[test]
    fn introduced_counts_findings_by_code_and_part_not_path() {
        let had = [finding("A", "word/document.xml", "w:body[0]/w:p[3]")];
        // The same finding at a shifted path is not new.
        let moved = [finding("A", "word/document.xml", "w:body[0]/w:p[1]")];
        assert!(introduced(&had, &moved).is_empty());
        // A second one of the code in the part is.
        let twice = [moved[0].clone(), finding("A", "word/document.xml", "x")];
        assert_eq!(introduced(&had, &twice), ["A in word/document.xml"]);
        // So is the code in another part, or another code.
        let other = [
            finding("A", "word/comments.xml", ""),
            finding("B", "word/document.xml", ""),
        ];
        assert_eq!(
            introduced(&had, &other),
            ["A in word/comments.xml", "B in word/document.xml"]
        );
        // Fewer findings is fine.
        assert!(introduced(&had, &[]).is_empty());
    }

    #[test]
    fn initials_take_the_first_letter_of_each_word() {
        assert_eq!(initials("Outside Counsel"), "OC");
        assert_eq!(initials("reviewer"), "R");
    }

    #[test]
    fn numbers_and_hex_ids_are_numbers() {
        assert!(is_number("1440") && is_number("-720") && is_number("00AB12CD"));
        assert!(!is_number("") && !is_number("12 345") && !is_number("Jane"));
    }

    #[test]
    fn a_needle_in_a_numeric_attribute_is_no_leak_but_in_a_text_attribute_it_is() {
        let w = W::URI;
        let size = format!(r#"<w:document xmlns:w="{w}"><w:pgSz w:w="12240"/></w:document>"#);
        assert!(!xml_holds(&size, "1224"));
        let alt = format!(r#"<w:document xmlns:w="{w}"><w:x w:val="12240"/></w:document>"#);
        assert!(xml_holds(&alt, "1224"));
    }

    #[test]
    fn element_names_are_no_leak() {
        let w = W::URI;
        let xml = format!(r#"<w:document xmlns:w="{w}"><w:body/></w:document>"#);
        assert!(!xml_holds(&xml, "body"));
        assert!(xml_holds(&format!("{xml}<!-- body -->"), "body"));
    }
}
