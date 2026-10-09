// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-validity findings beyond the schema: what makes Word refuse or
//! repair a package, as data.
//!
//! [`validate`] runs the Ring-1 invariants (the checks the test suite has
//! gated every produced package on) and the five `jubarte debug` triage
//! checks, and returns one [`Finding`] per problem with a stable `code`.
//! [`repair`] fixes the findings that have a deterministic fix and reports
//! what it could not fix. [`audit_tracked`] is `validate.py --original
//! --author` without an XSD: every visible-text difference between an
//! original and an edited document must sit in a revision by one author.
//!
//! XSD validation stays with `tools/validate-docx` (OpenXmlValidator); the
//! question answered here is "will Word open it", which a schema does not
//! answer.
//!
//! Codes, with whether Word refuses or repairs the file for them
//! (`word_fatal`) and whether [`repair`] fixes them (`repairable`):
//!
//! | Code | `word_fatal` | `repairable` |
//! |---|---|---|
//! | `MC_UNBOUND_PREFIX` | yes | when the prefix is a conventional one |
//! | `MISSING_CONTENT_TYPE`, `OVERRIDE_WITHOUT_PART`, `MALFORMED_XML` | yes | no |
//! | `DANGLING_RELATIONSHIP` | yes | yes: the attribute is dropped |
//! | `DUPLICATE_RID` | yes | an identical repeat is kept once on open; two relationships under one Id are refused |
//! | `MISSING_REL_TARGET` | yes | no |
//! | `PICTURE_BULLET_UNDEFINED` | yes | no |
//! | `DUPLICATE_REVISION_ID`, `DUPLICATE_DOCPR_ID` | no (leads) | yes: renumbered |
//! | `PARA_ID_OUT_OF_RANGE` | yes | yes: renumbered |
//! | `TEXT_INSIDE_DELETION`, `DELTEXT_OUTSIDE_DELETION`, `MOVEFROM_WITH_DELTEXT` | yes | yes: `w:t` and `w:delText` swapped |
//! | `INSTR_TEXT_INSIDE_DELETION` | no (not probed in Word) | yes |
//! | `BOOKMARK_IN_SINGLE_VALUE_CONTROL` | yes | yes: the bookmark is dropped |
//! | `COMMENT_ANCHOR_ORPHAN` | yes | yes: the anchor is dropped |
//! | `COMMENT_WITHOUT_ANCHOR`, `COMMENT_PARTS_INCONSISTENT`, `COMMENT_PARENT_CYCLE` | yes | no |
//! | `FIELD_UNBALANCED`, `FIELD_SPLIT_DELETION` | yes | no |
//! | `FIELD_CODE_STATE` | no | no |
//! | `EMPTY_FIELD_CODE`, `ROW_WITHOUT_CELL`, `NESTED_SAME_REVISION`, `SECTPR_NOT_LAST` | yes | no |
//! | `CELL_WITHOUT_PARAGRAPH` | yes | yes: an empty paragraph is appended |
//! | `NOTE_REFERENCE_DANGLING` | yes | no |
//! | `COMMENT_RANGE_UNPAIRED`, `BARE_RUN_IN_DELETED_TEXTBOX`, `BOOKMARK_*` | no | no |
//! | `UNTRACKED_EDIT`, `FOREIGN_AUTHOR` | n/a | no |

use std::collections::{HashMap, HashSet};
use std::fmt;

use quick_xml::Reader;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};

use crate::admission::{AdmissionError, InputLimits};
use crate::changes::{ChangeError, ChangeFilter, list_changes, reject_changes};
use crate::inspect::InspectError;
use crate::namespaces::{MC, R, W, W14};
use crate::opc::PartFs;
use crate::xmllinq::serialize::{well_known_namespace, well_known_prefix};
use crate::xmllinq::{Dom, NodeId, XName};

/// One thing wrong with a package. `path` is the element chain inside
/// `part` (`w:body[0]/w:p[3]/w:r[2]`, each name indexed among its
/// same-named siblings, root excluded), stable across runs; empty for a
/// package-level finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable code (`TEXT_INSIDE_DELETION`, `MC_UNBOUND_PREFIX`, ...).
    pub code: String,
    /// Package part the finding sits in.
    pub part: String,
    /// Element chain inside the part, or empty for package-level findings.
    pub path: String,
    /// Human-readable detail.
    pub message: String,
    /// True when Word refuses or repairs the file for this (Ring-1 and
    /// Ring-3 evidence); false for leads (`DUPLICATE_DOCPR_ID`).
    pub word_fatal: bool,
    /// True when [`repair`] fixes it deterministically.
    pub repairable: bool,
}

impl Finding {
    /// A finding with `word_fatal` and `repairable` taken from the code's
    /// table ([`traits`]).
    pub fn new(
        code: &str,
        part: &str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let (word_fatal, repairable) = traits(code);
        Finding {
            code: code.to_string(),
            part: part.to_string(),
            path: path.into(),
            message: message.into(),
            word_fatal,
            repairable,
        }
    }
}

/// `(word_fatal, repairable)` for a code; an unknown code is a lead with
/// no repair.
pub fn traits(code: &str) -> (bool, bool) {
    match code {
        "MC_UNBOUND_PREFIX"
        | "DANGLING_RELATIONSHIP"
        | "PARA_ID_OUT_OF_RANGE"
        | "TEXT_INSIDE_DELETION"
        | "DELTEXT_OUTSIDE_DELETION"
        | "MOVEFROM_WITH_DELTEXT"
        | "BOOKMARK_IN_SINGLE_VALUE_CONTROL"
        | "COMMENT_ANCHOR_ORPHAN"
        | "CELL_WITHOUT_PARAGRAPH"
        // Only an identical repeat is reported (two relationships under one
        // Id are refused on open), and opening already keeps it once.
        | "DUPLICATE_RID" => (true, true),
        "MISSING_CONTENT_TYPE"
        | "OVERRIDE_WITHOUT_PART"
        | "MALFORMED_XML"
        | "MISSING_REL_TARGET"
        | "PICTURE_BULLET_UNDEFINED"
        | "COMMENT_WITHOUT_ANCHOR"
        | "COMMENT_PARTS_INCONSISTENT"
        | "COMMENT_PARENT_CYCLE"
        | "FIELD_UNBALANCED"
        | "FIELD_SPLIT_DELETION"
        | "EMPTY_FIELD_CODE"
        | "ROW_WITHOUT_CELL"
        | "NESTED_SAME_REVISION"
        | "SECTPR_NOT_LAST"
        | "NOTE_REFERENCE_DANGLING" => (true, false),
        "DUPLICATE_REVISION_ID" | "DUPLICATE_DOCPR_ID" | "INSTR_TEXT_INSIDE_DELETION" => {
            (false, true)
        }
        _ => (false, false),
    }
}

/// Codes the Ring-1 checks own; the triage checks' findings under these
/// codes are the same problems seen twice and are left out of [`validate`].
const RING1_CODES: [&str; 18] = [
    "MC_UNBOUND_PREFIX",
    "MISSING_CONTENT_TYPE",
    "MALFORMED_XML",
    "DANGLING_RELATIONSHIP",
    "DUPLICATE_RID",
    "MISSING_REL_TARGET",
    "PICTURE_BULLET_UNDEFINED",
    "DUPLICATE_REVISION_ID",
    "DUPLICATE_DOCPR_ID",
    "PARA_ID_OUT_OF_RANGE",
    "TEXT_INSIDE_DELETION",
    "DELTEXT_OUTSIDE_DELETION",
    "MOVEFROM_WITH_DELTEXT",
    "BOOKMARK_IN_SINGLE_VALUE_CONTROL",
    "COMMENT_ANCHOR_ORPHAN",
    "COMMENT_WITHOUT_ANCHOR",
    "COMMENT_PARTS_INCONSISTENT",
    "COMMENT_PARENT_CYCLE",
];

/// Why a package could not be validated at all (a finding is never an
/// error).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidateError {
    /// Refused before parsing by [`crate::admission`].
    Refused(AdmissionError),
    /// The package could not be read or written.
    Package(String),
    /// The tracked changes could not be listed or rejected.
    Changes(ChangeError),
    /// The paragraphs could not be read.
    Inspect(InspectError),
}

impl fmt::Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(e) => write!(f, "DOCX refused: {e}"),
            Self::Package(m) => write!(f, "invalid package: {m}"),
            Self::Changes(e) => write!(f, "{e}"),
            Self::Inspect(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ValidateError {}

impl From<AdmissionError> for ValidateError {
    fn from(e: AdmissionError) -> Self {
        Self::Refused(e)
    }
}

impl From<ChangeError> for ValidateError {
    fn from(e: ChangeError) -> Self {
        Self::Changes(e)
    }
}

impl From<InspectError> for ValidateError {
    fn from(e: InspectError) -> Self {
        Self::Inspect(e)
    }
}

impl From<crate::opc::OpcError> for ValidateError {
    fn from(e: crate::opc::OpcError) -> Self {
        Self::Package(e.to_string())
    }
}

/// The package with every repairable finding fixed, the findings it fixed
/// and the ones it could not. Untouched parts keep their bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repaired {
    /// The repaired package.
    pub docx: Vec<u8>,
    /// Findings of the input that the output no longer has.
    pub repaired: Vec<Finding>,
    /// Findings the output still has.
    pub remaining: Vec<Finding>,
}

/// Findings in document order, sorted by part and path; an empty vector is
/// a pass.
///
/// # Errors
///
/// [`ValidateError`] when the package is refused by admission or cannot be
/// read; a problem Word would have with a readable package is a finding.
pub fn validate(docx: &[u8]) -> Result<Vec<Finding>, ValidateError> {
    crate::admission::admit(docx, InputLimits::default())?;
    let pkg = PartFs::open(docx)?;
    let mut out = ring1(&pkg);
    let triage = crate::debug::findings(docx).map_err(ValidateError::Package)?;
    out.extend(
        triage
            .into_iter()
            .filter(|f| !RING1_CODES.contains(&f.code.as_str())),
    );
    out.sort_by(|a, b| (&a.part, &a.path).cmp(&(&b.part, &b.path)));
    Ok(out)
}

/// The Ring-1 invariants alone, on an opened package: no admission, no
/// triage checks. [`validate`] is the full check.
pub fn ring1(pkg: &PartFs) -> Vec<Finding> {
    let mut out = Vec::new();
    check_content_types_and_xml(pkg, &mut out);
    check_relationship_integrity(pkg, &mut out);
    check_picture_bullets(pkg, &mut out);
    check_revision_and_drawing_ids(pkg, &mut out);
    check_para_text_id_bounds(pkg, &mut out);
    check_del_text_under_del(pkg, &mut out);
    check_deleted_text_has_deletion(pkg, &mut out);
    check_bookmarks_outside_single_value_controls(pkg, &mut out);
    check_comment_graph(pkg, &mut out);
    check_namespace_qname_contexts(pkg, &mut out);
    out
}

/// `w:body[0]/w:p[3]/w:r[2]`: conventional prefix and local name with the
/// index among same-named siblings, root excluded.
fn element_path(dom: &Dom, node: NodeId) -> String {
    let mut parts = Vec::new();
    let mut cur = Some(node);
    while let Some(n) = cur {
        let Some(parent) = dom.parent(n) else { break };
        if dom.parent(parent).is_none() {
            break;
        }
        let Some(name) = dom.name(n) else { break };
        let index = dom
            .elements(parent, Some(&name))
            .iter()
            .position(|&e| e == n)
            .unwrap_or(0);
        let prefix = well_known_prefix(name.namespace_name())
            .map(|p| format!("{p}:"))
            .unwrap_or_default();
        parts.push(format!("{prefix}{}[{index}]", name.local_name()));
        cur = Some(parent);
    }
    parts.reverse();
    parts.join("/")
}

fn parse_part(pkg: &PartFs, name: &str) -> Option<(Dom, NodeId)> {
    let xml = pkg.part_string(name)?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    Some((dom, root))
}

fn xml_parts(pkg: &PartFs) -> Vec<String> {
    pkg.parts()
        .into_iter()
        .filter(|name| name.ends_with(".xml"))
        .collect()
}

// ── Ring-1 checks ─────────────────────────────────────────────────────────

/// Every prefix an MC prefix list names (`mc:Choice/@Requires`,
/// `mc:Ignorable`, …) is bound where it is named, in every XML part: Word
/// rejects a header whose `mc:Choice Requires="wps"` names an unbound `wps`.
fn check_namespace_qname_contexts(pkg: &PartFs, out: &mut Vec<Finding>) {
    for part in xml_parts(pkg) {
        check_namespace_qname_context(pkg, &part, out);
    }
}

fn check_content_types_and_xml(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in pkg.parts() {
        // Every part should have a content type (default or override).
        if pkg.content_type_for(&name).is_none() && name != "[Content_Types].xml" {
            out.push(Finding::new(
                "MISSING_CONTENT_TYPE",
                &name,
                "",
                format!("part '{name}' has no content type"),
            ));
        }
        // XML-ish parts must parse.
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let Some(xml) = pkg.part_string(&name) else {
                out.push(Finding::new(
                    "MALFORMED_XML",
                    &name,
                    "",
                    format!("part '{name}' unreadable as string"),
                ));
                continue;
            };
            let mut reader = Reader::from_str(&xml);
            reader.config_mut().trim_text(false);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(_) => {}
                    Err(e) => {
                        out.push(Finding::new(
                            "MALFORMED_XML",
                            &name,
                            "",
                            format!("part '{name}' is not well-formed XML: {e}"),
                        ));
                        break;
                    }
                }
                buf.clear();
            }
        }
    }
}

/// Every `r:id` / `r:embed` / `r:link` in a part resolves in that part's `.rels`;
/// no duplicate rIds; no dangling internal targets.
fn check_relationship_integrity(pkg: &PartFs, out: &mut Vec<Finding>) {
    // Opening kept one copy of each identical repeat; the source still
    // repeats it, which Word refuses (docxide case8).
    for (part, id) in pkg.repaired_duplicate_ids() {
        let owner = if part.is_empty() {
            "_rels/.rels"
        } else {
            part.as_str()
        };
        out.push(Finding::new(
            "DUPLICATE_RID",
            owner,
            "",
            format!("duplicate rId '{id}' in relationships of '{owner}' (identical; one kept)"),
        ));
    }
    for name in pkg.parts() {
        if !name.ends_with(".xml") || name.ends_with(".rels") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let rels = pkg.read_rels_for(&name);
        let mut ids: HashSet<String> = HashSet::new();
        let mut targets: HashMap<String, (String, bool)> = HashMap::new();
        if let Some(r) = rels {
            let mut seen_ids = HashSet::new();
            for item in &r.items {
                if !seen_ids.insert(item.id.clone()) {
                    out.push(Finding::new(
                        "DUPLICATE_RID",
                        &name,
                        "",
                        format!("duplicate rId '{}' in relationships of '{name}'", item.id),
                    ));
                }
                ids.insert(item.id.clone());
                let external = item.target_mode.as_deref() == Some("External");
                targets.insert(item.id.clone(), (item.target.clone(), external));
            }
        }
        // Scan for r:id / r:embed / r:link attributes (namespace-agnostic local).
        for attr in ["r:id=\"", "r:embed=\"", "r:link=\""] {
            let mut rest = xml.as_str();
            while let Some(i) = rest.find(attr) {
                let after = &rest[i + attr.len()..];
                if let Some(end) = after.find('"') {
                    let rid = &after[..end];
                    if rid.is_empty() {
                        rest = &after[end + 1..];
                        continue;
                    }
                    if !ids.contains(rid) {
                        out.push(Finding::new(
                            "DANGLING_RELATIONSHIP",
                            &name,
                            "",
                            format!("dangling relationship id '{rid}' referenced from '{name}'"),
                        ));
                    } else if let Some((target, external)) = targets.get(rid)
                        && !external
                    {
                        let resolved = pkg.resolve_rel_target(&name, target);
                        if pkg.part_bytes(&resolved).is_none()
                            && pkg.part_bytes(target.trim_start_matches('/')).is_none()
                        {
                            // External-looking absolute targets without External mode
                            // are still flagged only when the target is clearly a package
                            // path that is missing. Skip http(s) and mailto.
                            let t = target.as_str();
                            if !t.starts_with("http://")
                                && !t.starts_with("https://")
                                && !t.starts_with("mailto:")
                            {
                                out.push(Finding::new(
                                    "MISSING_REL_TARGET",
                                    &name,
                                    "",
                                    format!(
                                        "relationship '{rid}' on '{name}' targets missing part '{target}' (resolved '{resolved}')"
                                    ),
                                ));
                            }
                        }
                    }
                    rest = &after[end + 1..];
                } else {
                    break;
                }
            }
        }
    }
}

/// Every `w:lvlPicBulletId` names a `w:numPicBullet` of the same numbering
/// part: Word refuses the whole package otherwise ("document loaded empty").
fn check_picture_bullets(pkg: &PartFs, out: &mut Vec<Finding>) {
    const PART: &str = "word/numbering.xml";
    let Some((dom, root)) = parse_part(pkg, PART) else {
        return;
    };
    let defined: HashSet<&str> = dom
        .elements(root, Some(&W::name("numPicBullet")))
        .into_iter()
        .filter_map(|b| dom.attribute(b, &W::name("numPicBulletId")))
        .collect();
    for pic in dom.descendants(root, Some(&W::name("lvlPicBulletId"))) {
        if let Some(id) = dom.attribute(pic, &W::val())
            && !defined.contains(id)
        {
            out.push(Finding::new(
                "PICTURE_BULLET_UNDEFINED",
                PART,
                element_path(&dom, pic),
                format!("lvlPicBulletId '{id}' names no numPicBullet in word/numbering.xml"),
            ));
        }
    }
}

fn check_revision_and_drawing_ids(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &name) else {
            continue;
        };
        let mut rev_ids: HashSet<String> = HashSet::new();
        let mut docpr_ids: HashSet<String> = HashSet::new();
        collect_ids(&dom, root, &mut rev_ids, &mut docpr_ids, out, &name);
    }
}

/// Revision elements whose `w:id` must be unique within a part.
const UNIQUE_REVISION_IDS: [&str; 6] = [
    "ins",
    "del",
    "moveFrom",
    "moveTo",
    "moveFromRangeStart",
    "moveToRangeStart",
];

fn collect_ids(
    dom: &Dom,
    root: NodeId,
    rev_ids: &mut HashSet<String>,
    docpr_ids: &mut HashSet<String>,
    out: &mut Vec<Finding>,
    part: &str,
) {
    for e in dom.descendants(root, None) {
        let Some(name) = dom.name(e) else {
            continue;
        };
        let local = name.local_name();
        // comment* share one id space (start/end/ref/entry); ins/del/move*
        // share another and are the ones enforced here.
        if UNIQUE_REVISION_IDS.contains(&local)
            && let Some(id) = dom.attribute(e, &W::name("id"))
            && !rev_ids.insert(format!("rev:{id}"))
        {
            out.push(Finding::new(
                "DUPLICATE_REVISION_ID",
                part,
                element_path(dom, e),
                format!("duplicate w:id '{id}' on revision markup in '{part}' ({local}:{id})"),
            ));
        }
        if local == "docPr" {
            let wp_id = XName::get(
                "id",
                "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing",
            );
            let id = dom.attribute(e, &wp_id).map(str::to_string).or_else(|| {
                dom.attributes(e)
                    .into_iter()
                    .find(|(n, _)| n.local_name() == "id")
                    .map(|(_, v)| v)
            });
            if let Some(id) = id
                && !docpr_ids.insert(id.clone())
            {
                out.push(Finding::new(
                    "DUPLICATE_DOCPR_ID",
                    part,
                    element_path(dom, e),
                    format!("duplicate wp:docPr id '{id}' in '{part}'"),
                ));
            }
        }
    }
}

/// `(attribute, value)` pairs of `w14:paraId` / `w14:textId` values at or
/// above Word's `0x80000000` bound, in `xml`.
/// Every `w14:paraId` / `w14:textId` value in a part, parsed.
fn para_id_values(xml: &str) -> Vec<u32> {
    let mut found = Vec::new();
    for attr in ["w14:paraId=\"", "w14:textId=\""] {
        let mut rest = xml;
        while let Some(i) = rest.find(attr) {
            let after = &rest[i + attr.len()..];
            let Some(end) = after.find('"') else { break };
            if let Ok(n) = u32::from_str_radix(&after[..end], 16) {
                found.push(n);
            }
            rest = &after[end + 1..];
        }
    }
    found
}

fn out_of_range_para_ids(xml: &str) -> Vec<(&'static str, String)> {
    let mut found = Vec::new();
    for attr in ["w14:paraId=\"", "w14:textId=\""] {
        let mut rest = xml;
        while let Some(i) = rest.find(attr) {
            let after = &rest[i + attr.len()..];
            let Some(end) = after.find('"') else { break };
            let val = &after[..end];
            if let Ok(n) = u32::from_str_radix(val, 16)
                && (n >= 0x8000_0000 || n == 0)
            {
                found.push((attr, val.to_string()));
            }
            rest = &after[end + 1..];
        }
    }
    found
}

fn check_para_text_id_bounds(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in xml_parts(pkg) {
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        for (attr, val) in out_of_range_para_ids(&xml) {
            out.push(Finding::new(
                "PARA_ID_OUT_OF_RANGE",
                &name,
                "",
                format!(
                    "{attr} value '{val}' outside Word's range 1..0x7FFFFFFF (>= 0x80000000 or zero) in '{name}' (id-paraid-overflow)"
                ),
            ));
        }
    }
}

/// `w:del` must carry `w:delText` (never `w:t`).
/// `w:moveFrom` must carry `w:t` (never `w:delText`) — Word-required contract
/// settled by Ring-3 probe 2026-07-16 (delText-under-moveFrom failed open).
fn check_del_text_under_del(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &name) else {
            continue;
        };
        for del in dom.descendants(root, Some(&W::del())) {
            for t in dom.descendants(del, Some(&W::t())) {
                if ancestor_has(&dom, t, del, "ins") {
                    continue;
                }
                out.push(Finding::new(
                    "TEXT_INSIDE_DELETION",
                    &name,
                    element_path(&dom, t),
                    format!("w:t under w:del in '{name}' (must be w:delText)"),
                ));
            }
        }
        let move_from = W::name("moveFrom");
        for mf in dom.descendants(root, Some(&move_from)) {
            for dt in dom.descendants(mf, Some(&W::name("delText"))) {
                if ancestor_has(&dom, dt, mf, "ins") {
                    continue;
                }
                out.push(Finding::new(
                    "MOVEFROM_WITH_DELTEXT",
                    &name,
                    element_path(&dom, dt),
                    format!("w:delText under w:moveFrom in '{name}' (Word requires w:t)"),
                ));
            }
        }
    }
}

/// True when a `w:del` or `w:moveFrom` covers `node` in its own story (a
/// text box's `w:txbxContent` is a separate story).
fn covered_by_deletion(dom: &Dom, node: NodeId) -> bool {
    let mut cur = dom.parent(node);
    while let Some(p) = cur {
        match dom.name(p).as_ref().map(|n| n.local_name()) {
            Some("del" | "moveFrom") => return true,
            Some("txbxContent") => return false,
            _ => cur = dom.parent(p),
        }
    }
    false
}

/// `w:delText` / `w:delInstrText` must sit under a `w:del` (or `w:moveFrom`)
/// in its OWN story. A text box's `w:txbxContent` is a separate story: the
/// `w:del` around its anchor run does not cover it. Word refuses a deleted text
/// box whose field code is `w:delInstrText` in a run no deletion wraps (en
/// 30ff840c/bb113e88); Word's own redline wraps the whole field in a `w:del`
/// inside the text box. The OpenXmlValidator passes the file.
fn check_deleted_text_has_deletion(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &name) else {
            continue;
        };
        for kind in ["delText", "delInstrText"] {
            for t in dom.descendants(root, Some(&W::name(kind))) {
                if !covered_by_deletion(&dom, t) {
                    out.push(Finding::new(
                        "DELTEXT_OUTSIDE_DELETION",
                        &name,
                        element_path(&dom, t),
                        format!(
                            "w:{kind} in '{name}' has no w:del in its own story (text boxes are separate stories)"
                        ),
                    ));
                }
            }
        }
    }
}

/// Content controls whose content is a single value: Word keeps no bookmark
/// in them.
const SINGLE_VALUE_CONTROLS: [&str; 6] = [
    "text",
    "dropDownList",
    "comboBox",
    "date",
    "picture",
    "checkbox",
];

/// `(bookmark mark, kind, control)` for every bookmark start or end inside a
/// single-value content control.
fn bookmarks_in_single_value_controls(
    dom: &Dom,
    root: NodeId,
) -> Vec<(NodeId, &'static str, String)> {
    let mut found = Vec::new();
    for kind in ["bookmarkStart", "bookmarkEnd"] {
        for m in dom.descendants(root, Some(&W::name(kind))) {
            for sdt in dom.ancestors(m, Some(&W::name("sdt"))) {
                let control = dom
                    .element(sdt, &W::name("sdtPr"))
                    .into_iter()
                    .flat_map(|pr| dom.elements(pr, None))
                    .filter_map(|c| dom.name(c).map(|n| n.local_name().to_string()))
                    .find(|c| SINGLE_VALUE_CONTROLS.contains(&c.as_str()));
                if let Some(control) = control {
                    found.push((m, kind, control));
                }
            }
        }
    }
    found
}

/// No bookmark start or end in a plain-text, dropdown, combo box, date,
/// picture or checkbox content control. Jubarte carried B's bookmarks into a
/// data-bound title control (en 7b649361) and a dropdown cell (en 57c181da);
/// Word refused both files and opened each once those bookmarks were
/// dropped. No bookmark in the 1000 English sources sits in such a control.
/// The OpenXmlValidator passes the files.
fn check_bookmarks_outside_single_value_controls(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in xml_parts(pkg) {
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        if !xml.contains("bookmark") || !xml.contains("sdtContent") {
            continue;
        }
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        for (m, kind, control) in bookmarks_in_single_value_controls(&dom, root) {
            out.push(Finding::new(
                "BOOKMARK_IN_SINGLE_VALUE_CONTROL",
                &name,
                element_path(&dom, m),
                format!(
                    "w:{kind} in '{name}' sits in a {control} content control (Word keeps no bookmark there)"
                ),
            ));
        }
    }
}

fn ancestor_has(dom: &Dom, node: NodeId, stop: NodeId, local: &str) -> bool {
    let mut cur = dom.parent(node);
    while let Some(p) = cur {
        if p == stop {
            break;
        }
        if let Some(n) = dom.name(p)
            && n.local_name() == local
        {
            return true;
        }
        cur = dom.parent(p);
    }
    false
}

const COMMENT_FAMILY: [(&str, &str, &str); 4] = [
    (
        "word/comments.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
    ),
    (
        "word/commentsExtended.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
        "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
    ),
    (
        "word/commentsIds.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml",
        "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
    ),
    (
        "word/commentsExtensible.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml",
        "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
    ),
];

const COMMENTS_PART: &str = "word/comments.xml";

fn main_part(pkg: &PartFs) -> Option<String> {
    pkg.main_document_part().or_else(|| {
        pkg.part_bytes("word/document.xml")
            .map(|_| "word/document.xml".to_string())
    })
}

/// A finding of the comment family: `code` on `part`, no path.
fn comment_finding(out: &mut Vec<Finding>, code: &str, part: &str, message: String) {
    out.push(Finding::new(code, part, "", message));
}

fn check_comment_graph(pkg: &PartFs, out: &mut Vec<Finding>) {
    let Some(main) = main_part(pkg) else {
        return;
    };
    let Some((main_dom, main_root)) = parse_part(pkg, &main) else {
        return;
    };
    let starts = comment_id_counts(&main_dom, main_root, "commentRangeStart");
    let ends = comment_id_counts(&main_dom, main_root, "commentRangeEnd");
    let references = comment_id_counts(&main_dom, main_root, "commentReference");

    let Some((comments_dom, comments_root)) = parse_part(pkg, COMMENTS_PART) else {
        for (kind, counts) in [
            ("commentRangeStart", &starts),
            ("commentRangeEnd", &ends),
            ("commentReference", &references),
        ] {
            for id in counts.keys() {
                comment_finding(
                    out,
                    "COMMENT_ANCHOR_ORPHAN",
                    &main,
                    format!("{kind} id '{id}' has no entry in word/comments.xml"),
                );
            }
        }
        for (part, _, _) in &COMMENT_FAMILY[1..] {
            if pkg.part_bytes(part).is_some() {
                comment_finding(
                    out,
                    "COMMENT_PARTS_INCONSISTENT",
                    part,
                    format!("'{part}' exists without word/comments.xml"),
                );
            }
        }
        return;
    };

    let definitions = comment_id_counts(&comments_dom, comments_root, "comment");
    for (kind, counts, part) in [
        ("comment definition", &definitions, COMMENTS_PART),
        ("commentRangeStart", &starts, main.as_str()),
        ("commentRangeEnd", &ends, main.as_str()),
        ("commentReference", &references, main.as_str()),
    ] {
        for (id, count) in counts {
            if *count != 1 {
                comment_finding(
                    out,
                    "COMMENT_PARTS_INCONSISTENT",
                    part,
                    format!("{kind} id '{id}' occurs {count} times"),
                );
            }
        }
    }
    let definition_ids: HashSet<String> = definitions.keys().cloned().collect();
    for (kind, counts) in [
        ("commentRangeStart", &starts),
        ("commentRangeEnd", &ends),
        ("commentReference", &references),
    ] {
        let ids: HashSet<String> = counts.keys().cloned().collect();
        for id in definition_ids.difference(&ids) {
            comment_finding(
                out,
                "COMMENT_WITHOUT_ANCHOR",
                COMMENTS_PART,
                format!("comment definition id '{id}' has no matching {kind}"),
            );
        }
        for id in ids.difference(&definition_ids) {
            comment_finding(
                out,
                "COMMENT_ANCHOR_ORPHAN",
                &main,
                format!("{kind} id '{id}' has no entry in word/comments.xml"),
            );
        }
    }

    check_comment_family_packaging(pkg, &main, out);

    let aux_present = COMMENT_FAMILY[1..]
        .iter()
        .any(|(part, _, _)| pkg.part_bytes(part).is_some());
    let mut all_para_ids = HashSet::new();
    let mut last_para_ids = HashSet::new();
    for comment in comments_dom.elements(comments_root, Some(&W::name("comment"))) {
        let comment_id = comments_dom
            .attribute(comment, &W::id())
            .unwrap_or("<missing>");
        let mut comment_para_ids = Vec::new();
        for paragraph in comments_dom.descendants(comment, Some(&W::p())) {
            let Some(para_id) = comments_dom.attribute(paragraph, &W14::name("paraId")) else {
                continue;
            };
            let key = para_id.to_ascii_uppercase();
            check_hex_id("paraId", para_id, true, COMMENTS_PART, out);
            if !all_para_ids.insert(key.clone()) {
                comment_finding(
                    out,
                    "COMMENT_PARTS_INCONSISTENT",
                    COMMENTS_PART,
                    format!("duplicate comment paraId '{para_id}' in word/comments.xml"),
                );
            }
            comment_para_ids.push(key);
        }
        if aux_present && comment_para_ids.is_empty() {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                COMMENTS_PART,
                format!("comment id '{comment_id}' has no w14:paraId for its auxiliary metadata"),
            );
        }
        if let Some(last) = comment_para_ids.last() {
            last_para_ids.insert(last.clone());
        }
    }

    let extended_para_ids = check_comments_extended(pkg, &last_para_ids, out);
    let durable_ids = check_comments_ids(pkg, &last_para_ids, out);
    check_comments_extensible(pkg, durable_ids.as_ref(), out);
    if let Some(extended) = extended_para_ids {
        for parent in extended.parents.values() {
            if !extended.keys.contains(parent) {
                comment_finding(
                    out,
                    "COMMENT_PARTS_INCONSISTENT",
                    "word/commentsExtended.xml",
                    format!("commentsExtended paraIdParent '{parent}' does not resolve"),
                );
            }
        }
        check_parent_cycles(&extended.parents, out);
    }
}

fn comment_id_counts(dom: &Dom, root: NodeId, local: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for element in dom.descendants(root, Some(&W::name(local))) {
        if let Some(id) = dom.attribute(element, &W::id()) {
            *counts.entry(id.to_string()).or_default() += 1;
        }
    }
    counts
}

fn check_comment_family_packaging(pkg: &PartFs, main: &str, out: &mut Vec<Finding>) {
    let rels = pkg.read_rels_for(main);
    for (part, content_type, relationship_type) in COMMENT_FAMILY {
        let part_present = pkg.part_bytes(part).is_some();
        let matching: Vec<_> = rels
            .into_iter()
            .flat_map(|relationships| &relationships.items)
            .filter(|relationship| relationship.rel_type == relationship_type)
            .collect();
        if part_present {
            if pkg.content_type_for(part).as_deref() != Some(content_type) {
                comment_finding(
                    out,
                    "COMMENT_PARTS_INCONSISTENT",
                    part,
                    format!("'{part}' has the wrong content type (expected '{content_type}')"),
                );
            }
            if matching.len() != 1 {
                comment_finding(
                    out,
                    "COMMENT_PARTS_INCONSISTENT",
                    main,
                    format!(
                        "'{main}' needs exactly one relationship to '{part}', found {}",
                        matching.len()
                    ),
                );
            }
            for relationship in matching {
                let resolved = pkg
                    .resolve_rel_target(main, &relationship.target)
                    .trim_start_matches('/')
                    .to_string();
                if relationship.target_mode.as_deref() == Some("External") || resolved != part {
                    comment_finding(
                        out,
                        "COMMENT_PARTS_INCONSISTENT",
                        main,
                        format!(
                            "comment relationship '{}' on '{main}' resolves to '{}' instead of '{part}'",
                            relationship.id, relationship.target
                        ),
                    );
                }
            }
        } else if !matching.is_empty() {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                main,
                format!("'{main}' has a relationship for missing comment part '{part}'"),
            );
        }
    }
}

fn check_hex_id(
    label: &str,
    value: &str,
    word_para_bound: bool,
    part: &str,
    out: &mut Vec<Finding>,
) {
    let parsed = if value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        u32::from_str_radix(value, 16).ok()
    } else {
        None
    };
    let Some(parsed) = parsed else {
        comment_finding(
            out,
            "COMMENT_PARTS_INCONSISTENT",
            part,
            format!("{label} '{value}' is not an 8-digit hexadecimal id"),
        );
        return;
    };
    if word_para_bound && parsed >= 0x8000_0000 {
        comment_finding(
            out,
            "COMMENT_PARTS_INCONSISTENT",
            part,
            format!("{label} '{value}' is outside Word's paraId range"),
        );
    }
}

struct ExtendedGraph {
    keys: HashSet<String>,
    parents: HashMap<String, String>,
}

fn check_comments_extended(
    pkg: &PartFs,
    last_para_ids: &HashSet<String>,
    out: &mut Vec<Finding>,
) -> Option<ExtendedGraph> {
    const PART: &str = "word/commentsExtended.xml";
    let (dom, root) = parse_part(pkg, PART)?;
    let mut keys = HashSet::new();
    let mut parents = HashMap::new();
    for entry in dom.elements(root, None) {
        let Some(para_id) = attribute_by_local(&dom, entry, "paraId") else {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                "commentsExtended entry has no paraId".to_string(),
            );
            continue;
        };
        let key = para_id.to_ascii_uppercase();
        check_hex_id("commentsExtended paraId", para_id, true, PART, out);
        if !keys.insert(key.clone()) {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                format!("duplicate commentsExtended paraId '{para_id}'"),
            );
        }
        if let Some(parent) = attribute_by_local(&dom, entry, "paraIdParent") {
            let parent = parent.to_ascii_uppercase();
            check_hex_id("commentsExtended paraIdParent", &parent, true, PART, out);
            if parent == key {
                comment_finding(
                    out,
                    "COMMENT_PARENT_CYCLE",
                    PART,
                    format!("commentsExtended paraId '{key}' is its own parent"),
                );
            }
            parents.insert(key, parent);
        }
    }
    check_exact_key_set("commentsExtended paraId", &keys, last_para_ids, PART, out);
    Some(ExtendedGraph { keys, parents })
}

fn check_comments_ids(
    pkg: &PartFs,
    last_para_ids: &HashSet<String>,
    out: &mut Vec<Finding>,
) -> Option<HashSet<String>> {
    const PART: &str = "word/commentsIds.xml";
    let (dom, root) = parse_part(pkg, PART)?;
    let mut para_ids = HashSet::new();
    let mut durable_ids = HashSet::new();
    for entry in dom.elements(root, None) {
        let Some(para_id) = attribute_by_local(&dom, entry, "paraId") else {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                "commentsIds entry has no paraId".to_string(),
            );
            continue;
        };
        let para_key = para_id.to_ascii_uppercase();
        check_hex_id("commentsIds paraId", para_id, true, PART, out);
        if !para_ids.insert(para_key) {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                format!("duplicate commentsIds paraId '{para_id}'"),
            );
        }
        let Some(durable_id) = attribute_by_local(&dom, entry, "durableId") else {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                format!("commentsIds paraId '{para_id}' has no durableId"),
            );
            continue;
        };
        let durable_key = durable_id.to_ascii_uppercase();
        check_hex_id("commentsIds durableId", durable_id, false, PART, out);
        if !durable_ids.insert(durable_key) {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                format!("duplicate commentsIds durableId '{durable_id}'"),
            );
        }
    }
    check_exact_key_set("commentsIds paraId", &para_ids, last_para_ids, PART, out);
    Some(durable_ids)
}

fn check_comments_extensible(
    pkg: &PartFs,
    expected_durable_ids: Option<&HashSet<String>>,
    out: &mut Vec<Finding>,
) {
    const PART: &str = "word/commentsExtensible.xml";
    let Some((dom, root)) = parse_part(pkg, PART) else {
        return;
    };
    let mut durable_ids = HashSet::new();
    for entry in dom.elements(root, None) {
        let Some(durable_id) = attribute_by_local(&dom, entry, "durableId") else {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                "commentsExtensible entry has no durableId".to_string(),
            );
            continue;
        };
        let key = durable_id.to_ascii_uppercase();
        check_hex_id("commentsExtensible durableId", durable_id, false, PART, out);
        if !durable_ids.insert(key) {
            comment_finding(
                out,
                "COMMENT_PARTS_INCONSISTENT",
                PART,
                format!("duplicate commentsExtensible durableId '{durable_id}'"),
            );
        }
    }
    match expected_durable_ids {
        Some(expected) => check_exact_key_set(
            "commentsExtensible durableId",
            &durable_ids,
            expected,
            PART,
            out,
        ),
        None => comment_finding(
            out,
            "COMMENT_PARTS_INCONSISTENT",
            PART,
            "commentsExtensible exists without commentsIds".to_string(),
        ),
    }
}

fn attribute_by_local<'a>(dom: &'a Dom, element: NodeId, local: &str) -> Option<&'a str> {
    for index in 0..dom.attr_count(element) {
        let (name, value) = dom.attr_at(element, index);
        if name.local_name() == local {
            return Some(value);
        }
    }
    None
}

fn check_exact_key_set(
    label: &str,
    actual: &HashSet<String>,
    expected: &HashSet<String>,
    part: &str,
    out: &mut Vec<Finding>,
) {
    for key in expected.difference(actual) {
        comment_finding(
            out,
            "COMMENT_PARTS_INCONSISTENT",
            part,
            format!("{label} is missing '{key}'"),
        );
    }
    for key in actual.difference(expected) {
        comment_finding(
            out,
            "COMMENT_PARTS_INCONSISTENT",
            part,
            format!("{label} '{key}' has no matching comment paragraph"),
        );
    }
}

fn check_parent_cycles(parents: &HashMap<String, String>, out: &mut Vec<Finding>) {
    for start in parents.keys() {
        let mut seen = HashSet::new();
        let mut current = start;
        while let Some(parent) = parents.get(current) {
            if !seen.insert(current.clone()) {
                comment_finding(
                    out,
                    "COMMENT_PARENT_CYCLE",
                    "word/commentsExtended.xml",
                    format!("commentsExtended paraIdParent cycle contains '{current}'"),
                );
                break;
            }
            current = parent;
        }
    }
}

/// An unqualified `Requires` is a prefix list only on `mc:Choice`; on any
/// other element (a custom XML part) it is application data.
fn is_namespace_qname_list(element: &XName, name: &XName) -> bool {
    (name.namespace_name().is_empty()
        && name.local_name() == "Requires"
        && element.namespace_name() == MC::URI
        && element.local_name() == "Choice")
        || (name.namespace_name() == MC::URI
            && matches!(
                name.local_name(),
                "Ignorable"
                    | "PreserveAttributes"
                    | "PreserveElements"
                    | "ProcessContent"
                    | "MustUnderstand"
            ))
}

fn check_namespace_qname_context(pkg: &PartFs, part: &str, out: &mut Vec<Finding>) {
    let Some((dom, root)) = parse_part(pkg, part) else {
        return;
    };
    for element in dom.descendants_and_self(root, None) {
        let Some(element_name) = dom.name(element) else {
            continue;
        };
        for (name, value) in dom.attributes(element) {
            if !is_namespace_qname_list(&element_name, &name) {
                continue;
            }
            for token in value.split_whitespace() {
                let prefix = token.split_once(':').map_or(token, |(prefix, _)| prefix);
                if prefix != "xml" && namespace_in_scope(&dom, element, prefix).is_none() {
                    let mut finding = Finding::new(
                        "MC_UNBOUND_PREFIX",
                        part,
                        element_path(&dom, element),
                        format!(
                            "unresolved namespace prefix '{prefix}' in {}='{}' in '{part}'",
                            name.local_name(),
                            value
                        ),
                    );
                    finding.repairable = well_known_namespace(prefix).is_some();
                    out.push(finding);
                }
            }
        }
    }
}

fn namespace_in_scope<'a>(dom: &'a Dom, element: NodeId, prefix: &str) -> Option<&'a str> {
    let mut current = Some(element);
    while let Some(node) = current {
        for index in 0..dom.attr_count(node) {
            let (name, value) = dom.attr_at(node, index);
            if dom.is_namespace_declaration(name) && name.local_name() == prefix {
                return Some(value);
            }
        }
        current = dom.parent(node);
    }
    None
}

// ── Repair ────────────────────────────────────────────────────────────────

/// The package with every repairable finding fixed, in a fixed order:
/// unbound MC prefixes bound on the part root, `w:t`/`w:delText` swapped,
/// bookmarks dropped from single-value controls, dangling relationship
/// attributes dropped, duplicate drawing and revision ids renumbered,
/// paragraph ids brought into Word's range, cells given a last paragraph,
/// orphan comment anchors dropped. The result is validated again to fill
/// `remaining`. A package with no repairable finding comes back as its
/// own bytes.
///
/// # Errors
///
/// As [`validate`], plus a package that cannot be written back.
pub fn repair(docx: &[u8]) -> Result<Repaired, ValidateError> {
    let findings = validate(docx)?;
    // Nothing to repair: hand the bytes back untouched rather than a
    // rewritten zip, so "nothing changed" is visible as equal bytes.
    if !findings.iter().any(|f| f.repairable) {
        return Ok(Repaired {
            docx: docx.to_vec(),
            repaired: Vec::new(),
            remaining: findings,
        });
    }
    let has = |code: &str| findings.iter().any(|f| f.code == code);
    let parts_with = |codes: &[&str]| -> Vec<String> {
        let mut parts: Vec<String> = findings
            .iter()
            .filter(|f| codes.contains(&f.code.as_str()))
            .map(|f| f.part.clone())
            .collect();
        parts.sort();
        parts.dedup();
        parts
    };
    let bytes = if has("DUPLICATE_DOCPR_ID") {
        crate::comparer::fixups::fix_up_drawing_ids_in_package(docx)?
    } else {
        docx.to_vec()
    };
    let mut pkg = PartFs::open(&bytes)?;
    for part in parts_with(&["MC_UNBOUND_PREFIX"]) {
        edit_part(&mut pkg, &part, |dom, root| {
            crate::comparer::finalize::bind_compatibility_prefixes(dom, root);
        });
    }
    for part in parts_with(&[
        "TEXT_INSIDE_DELETION",
        "DELTEXT_OUTSIDE_DELETION",
        "MOVEFROM_WITH_DELTEXT",
        "INSTR_TEXT_INSIDE_DELETION",
    ]) {
        edit_part(&mut pkg, &part, fix_deleted_text);
    }
    for part in parts_with(&["BOOKMARK_IN_SINGLE_VALUE_CONTROL"]) {
        edit_part(&mut pkg, &part, drop_bookmarks_in_single_value_controls);
    }
    for part in parts_with(&["DANGLING_RELATIONSHIP"]) {
        let ids: HashSet<String> = pkg
            .read_rels_for(&part)
            .map(|r| r.items.iter().map(|i| i.id.clone()).collect())
            .unwrap_or_default();
        edit_part(&mut pkg, &part, |dom, root| {
            drop_dangling_relationship_attributes(dom, root, &ids);
        });
    }
    for part in parts_with(&["DUPLICATE_REVISION_ID"]) {
        edit_part(&mut pkg, &part, renumber_duplicate_revision_ids);
    }
    if has("PARA_ID_OUT_OF_RANGE") {
        renumber_out_of_range_para_ids(&mut pkg);
    }
    for part in parts_with(&["CELL_WITHOUT_PARAGRAPH"]) {
        edit_part(&mut pkg, &part, end_cells_with_a_paragraph);
    }
    if has("COMMENT_ANCHOR_ORPHAN")
        && let Some(main) = main_part(&pkg)
    {
        let defined: HashSet<String> = parse_part(&pkg, COMMENTS_PART)
            .map(|(dom, root)| {
                comment_id_counts(&dom, root, "comment")
                    .into_keys()
                    .collect()
            })
            .unwrap_or_default();
        edit_part(&mut pkg, &main, |dom, root| {
            drop_orphan_comment_anchors(dom, root, &defined);
        });
    }
    let docx = pkg.to_zip()?;
    let remaining = validate(&docx)?;
    let same =
        |a: &Finding, b: &Finding| a.code == b.code && a.part == b.part && a.message == b.message;
    let repaired = findings
        .into_iter()
        .filter(|f| f.repairable && !remaining.iter().any(|r| same(f, r)))
        .collect();
    Ok(Repaired {
        docx,
        repaired,
        remaining,
    })
}

/// Parse `part`, let `f` change it, write it back.
fn edit_part(pkg: &mut PartFs, part: &str, f: impl FnOnce(&mut Dom, NodeId)) {
    let Some(xml) = pkg.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    f(&mut dom, root);
    pkg.set_part(part, dom.serialize_document(doc).into_bytes());
}

/// `w:t` and `w:instrText` under a deletion become `w:delText` and
/// `w:delInstrText`; `w:delText` under a move source, and deleted text no
/// deletion covers, become live text again.
fn fix_deleted_text(dom: &mut Dom, root: NodeId) {
    let mut renames: Vec<(NodeId, XName)> = Vec::new();
    for del in dom.descendants(root, Some(&W::del())) {
        for (live, deleted) in [("t", "delText"), ("instrText", "delInstrText")] {
            for t in dom.descendants(del, Some(&W::name(live))) {
                if !ancestor_has(dom, t, del, "ins") {
                    renames.push((t, W::name(deleted)));
                }
            }
        }
    }
    for mf in dom.descendants(root, Some(&W::name("moveFrom"))) {
        for dt in dom.descendants(mf, Some(&W::name("delText"))) {
            if !ancestor_has(dom, dt, mf, "ins") {
                renames.push((dt, W::t()));
            }
        }
    }
    for (node, name) in renames {
        dom.set_name(node, name);
    }
    let mut revive: Vec<(NodeId, XName)> = Vec::new();
    for (deleted, live) in [("delText", "t"), ("delInstrText", "instrText")] {
        for t in dom.descendants(root, Some(&W::name(deleted))) {
            if !covered_by_deletion(dom, t) {
                revive.push((t, W::name(live)));
            }
        }
    }
    for (node, name) in revive {
        dom.set_name(node, name);
    }
}

/// Every bookmark whose start or end sits in a single-value content control
/// is dropped whole (both marks, wherever the other one is).
fn drop_bookmarks_in_single_value_controls(dom: &mut Dom, root: NodeId) {
    let ids: HashSet<String> = bookmarks_in_single_value_controls(dom, root)
        .into_iter()
        .filter_map(|(m, _, _)| dom.attribute(m, &W::id()).map(str::to_string))
        .collect();
    let doomed: Vec<NodeId> = ["bookmarkStart", "bookmarkEnd"]
        .into_iter()
        .flat_map(|kind| dom.descendants(root, Some(&W::name(kind))))
        .filter(|&m| {
            dom.attribute(m, &W::id())
                .is_some_and(|id| ids.contains(id))
        })
        .collect();
    for m in doomed {
        dom.remove(m);
    }
}

/// Elements whose `r:id` the schema requires: without the relationship the
/// element itself has to go, not just the attribute.
const REQUIRES_RELATIONSHIP: [&str; 7] = [
    "headerReference",
    "footerReference",
    "altChunk",
    "attachedTemplate",
    "contentPart",
    "movie",
    "subDoc",
];

/// Relationship attributes (`r:id`, `r:embed`, `r:link`, …) naming no
/// relationship of the part are dropped, which is what stops Word's
/// "unreadable content" repair. An element the schema gives no meaning
/// without its relationship (a header reference, an altChunk, …) is removed
/// whole.
fn drop_dangling_relationship_attributes(dom: &mut Dom, root: NodeId, ids: &HashSet<String>) {
    let mut doomed: Vec<(NodeId, XName)> = Vec::new();
    let mut removed: Vec<NodeId> = Vec::new();
    for el in dom.descendants_and_self(root, None) {
        for (name, value) in dom.attributes(el) {
            if name.namespace_name() == R::URI && !value.is_empty() && !ids.contains(&value) {
                let required = name.local_name() == "id"
                    && dom
                        .name(el)
                        .is_some_and(|n| REQUIRES_RELATIONSHIP.contains(&n.local_name()));
                if required {
                    removed.push(el);
                } else {
                    doomed.push((el, name));
                }
            }
        }
    }
    for (el, name) in doomed {
        dom.set_attribute_value(el, &name, None);
    }
    for el in removed {
        dom.remove(el);
    }
}

/// A second use of a revision id is renumbered from the part's highest
/// `w:id` upwards; a move range start takes its end along.
fn renumber_duplicate_revision_ids(dom: &mut Dom, root: NodeId) {
    let id_name = W::name("id");
    let all: Vec<NodeId> = dom.descendants(root, None);
    let mut next: u64 = all
        .iter()
        .filter_map(|&n| dom.attribute(n, &id_name))
        .filter_map(|v| v.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let mut seen: HashSet<String> = HashSet::new();
    let mut renames: Vec<(NodeId, String)> = Vec::new();
    for (i, &n) in all.iter().enumerate() {
        let Some(name) = dom.name(n) else { continue };
        let local = name.local_name();
        if !UNIQUE_REVISION_IDS.contains(&local) {
            continue;
        }
        let Some(id) = dom.attribute(n, &id_name).map(str::to_string) else {
            continue;
        };
        if seen.insert(id.clone()) {
            continue;
        }
        let fresh = next.to_string();
        next += 1;
        renames.push((n, fresh.clone()));
        let end_local = match local {
            "moveFromRangeStart" => Some("moveFromRangeEnd"),
            "moveToRangeStart" => Some("moveToRangeEnd"),
            _ => None,
        };
        if let Some(end_local) = end_local
            && let Some(&end) = all[i + 1..].iter().find(|&&e| {
                dom.name(e).is_some_and(|x| x.local_name() == end_local)
                    && dom.attribute(e, &id_name) == Some(id.as_str())
            })
        {
            renames.push((end, fresh));
        }
    }
    for (n, id) in renames {
        dom.set_attribute_value(n, &id_name, Some(&id));
    }
}

/// Every `w14:paraId` / `w14:textId` at or above `0x80000000` is masked
/// into Word's range, and every reference to it (`paraIdParent`, the
/// comment parts) follows.
fn renumber_out_of_range_para_ids(pkg: &mut PartFs) {
    let parts = xml_parts(pkg);
    let mut used: HashSet<u32> = HashSet::new();
    for name in &parts {
        let Some(xml) = pkg.part_string(name) else {
            continue;
        };
        used.extend(para_id_values(&xml));
    }
    let mut map: HashMap<String, String> = HashMap::new();
    for name in &parts {
        let Some(xml) = pkg.part_string(name) else {
            continue;
        };
        for (_, old) in out_of_range_para_ids(&xml) {
            if map.contains_key(&old) {
                continue;
            }
            // The masked value first; if that is zero or taken, the next free
            // id upwards, wrapping inside Word's range.
            let mut n = u32::from_str_radix(&old, 16).unwrap_or(0) & 0x7FFF_FFFF;
            while n == 0 || used.contains(&n) {
                n = (n + 1) & 0x7FFF_FFFF;
            }
            used.insert(n);
            map.insert(old, format!("{n:08X}"));
        }
    }
    if map.is_empty() {
        return;
    }
    for name in &parts {
        let Some(mut xml) = pkg.part_string(name) else {
            continue;
        };
        let before = xml.clone();
        for (old, new) in &map {
            for attr in ["paraId", "textId", "paraIdParent"] {
                xml = xml.replace(&format!("{attr}=\"{old}\""), &format!("{attr}=\"{new}\""));
            }
        }
        if xml != before {
            pkg.set_part(name, xml.into_bytes());
        }
    }
}

/// A table cell whose last child (its properties aside) is not a paragraph
/// gets an empty one.
fn end_cells_with_a_paragraph(dom: &mut Dom, root: NodeId) {
    let cells: Vec<NodeId> = dom
        .descendants(root, Some(&W::name("tc")))
        .into_iter()
        .filter(|&tc| {
            let last = dom
                .elements(tc, None)
                .into_iter()
                .filter_map(|c| dom.name(c).map(|n| n.local_name().to_string()))
                .rfind(|c| c != "tcPr");
            last.as_deref() != Some("p")
        })
        .collect();
    for tc in cells {
        let p = dom.new_element(W::p());
        dom.add(tc, p);
    }
}

/// Comment range marks and references naming no comment definition are
/// dropped; a run left holding only its properties goes with its reference.
fn drop_orphan_comment_anchors(dom: &mut Dom, root: NodeId, defined: &HashSet<String>) {
    let orphan = |dom: &Dom, n: NodeId| {
        dom.attribute(n, &W::id())
            .is_some_and(|id| !defined.contains(id))
    };
    let marks: Vec<NodeId> = ["commentRangeStart", "commentRangeEnd"]
        .into_iter()
        .flat_map(|kind| dom.descendants(root, Some(&W::name(kind))))
        .filter(|&m| orphan(dom, m))
        .collect();
    for m in marks {
        dom.remove(m);
    }
    let references: Vec<NodeId> = dom
        .descendants(root, Some(&W::name("commentReference")))
        .into_iter()
        .filter(|&r| orphan(dom, r))
        .collect();
    for reference in references {
        let run = dom
            .parent(reference)
            .filter(|&p| dom.name_is(p, &W::name("r")));
        dom.remove(reference);
        if let Some(run) = run
            && dom
                .elements(run, None)
                .iter()
                .all(|&c| dom.name_is(c, &W::name("rPr")))
        {
            dom.remove(run);
        }
    }
}

// ── Tracked-edit audit ────────────────────────────────────────────────────

/// The body's paragraphs followed by every story's (`header1:p:0`, ...).
fn all_paragraphs(docx: &[u8]) -> Result<Vec<crate::inspect::Paragraph>, ValidateError> {
    let mut all = crate::inspect::paragraphs(docx)?;
    for story in crate::inspect::stories(docx)? {
        all.extend(story.paragraphs);
    }
    Ok(all)
}

/// The part a `{story}:...` id lives in: `body` is the main document, any
/// other story is `word/{story}.xml` (`header1`, `footnotes`, ...).
fn story_part(id: &str) -> String {
    match id.split(':').next().unwrap_or("body") {
        "body" | "" => "word/document.xml".to_string(),
        story => format!("word/{story}.xml"),
    }
}

/// `validate.py --original --author` without an XSD: every visible-text
/// difference between `original` and `edited` must sit in a revision by
/// `author`. Rejecting that author's changes must give back `original`'s
/// text, body and stories alike; any residue is an `UNTRACKED_EDIT` finding
/// at its paragraph id, and a change by anyone else is a `FOREIGN_AUTHOR`
/// finding.
///
/// # Errors
///
/// [`ValidateError`] when either document cannot be read.
pub fn audit_tracked(
    original: &[u8],
    edited: &[u8],
    author: &str,
) -> Result<Vec<Finding>, ValidateError> {
    let mut out = Vec::new();
    let listed = list_changes(edited)?;
    for c in listed
        .iter()
        .filter(|c| c.author.as_deref() != Some(author))
    {
        out.push(Finding::new(
            "FOREIGN_AUTHOR",
            &story_part(&c.id),
            "",
            format!(
                "{} by {}",
                c.id,
                c.author
                    .clone()
                    .unwrap_or_else(|| "<no author>".to_string())
            ),
        ));
    }
    let filter = ChangeFilter {
        authors: Some(vec![author.to_string()]),
        ..ChangeFilter::default()
    };
    let reverted = reject_changes(edited, &filter)?;
    // Paragraph-by-paragraph text: the same projection `jubarte text` prints,
    // body first, then every header, footer and notes story.
    let before = all_paragraphs(original)?;
    let after = all_paragraphs(&reverted)?;
    if before.len() != after.len() {
        out.push(Finding::new(
            "UNTRACKED_EDIT",
            "word/document.xml",
            "",
            format!(
                "{} paragraphs before, {} after rejecting {author}'s changes",
                before.len(),
                after.len()
            ),
        ));
        return Ok(out);
    }
    for (b, a) in before.iter().zip(&after) {
        if b.text != a.text {
            out.push(Finding::new(
                "UNTRACKED_EDIT",
                &story_part(&a.id),
                a.id.clone(),
                format!(
                    "{} differs after rejecting {author}'s changes: {:?} vs {:?}",
                    a.id, b.text, a.text
                ),
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod validation_boundary_tests {
    use super::*;

    fn pkg() -> PartFs {
        PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap()
    }

    fn xml(dom: &mut Dom, body: &str) -> NodeId {
        let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{}" xmlns:mc="{}"><w:body>{body}</w:body></w:document>"#,
            W::URI,
            MC::URI
        ));
        dom.root(doc).unwrap()
    }

    #[test]
    fn metadata_ids_check_width_hex_digits_and_the_word_boundary_independently() {
        for (value, bound, expected) in [
            ("12345678", true, None),
            ("abcdef01", false, None),
            ("7FFFFFFF", true, None),
            ("80000000", true, Some("outside Word")),
            ("80000000", false, None),
            ("1234567", true, Some("not an 8-digit")),
            ("123456789", false, Some("not an 8-digit")),
            ("1234567z", true, Some("not an 8-digit")),
            ("é234567", false, Some("not an 8-digit")),
        ] {
            let mut findings = Vec::new();
            check_hex_id("test", value, bound, "part", &mut findings);
            assert_eq!(findings.len(), usize::from(expected.is_some()), "{value}");
            if let Some(message) = expected {
                assert_eq!(findings[0].code, "COMMENT_PARTS_INCONSISTENT");
                assert!(findings[0].message.contains(message));
                assert!(findings[0].word_fatal);
            }
        }
    }

    #[test]
    fn paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes() {
        let source = r#"w14:paraId="7fffffff" w14:textId="80000000" w14:paraId="ZZZZZZZZ" w14:textId="00000000" w14:paraId="FFFFFFFF" w14:textId="unfinished"#;
        assert_eq!(
            para_id_values(source),
            vec![0x7fffffff, u32::MAX, 0x80000000, 0]
        );
        assert_eq!(
            out_of_range_para_ids(source),
            vec![
                ("w14:paraId=\"", "FFFFFFFF".into()),
                ("w14:textId=\"", "80000000".into()),
                ("w14:textId=\"", "00000000".into())
            ]
        );
    }

    #[test]
    fn deletion_scope_stops_at_textboxes_and_nested_insertions_stay_live() {
        let mut dom = Dom::new();
        let root = xml(
            &mut dom,
            "<w:p><w:del><w:r><w:t>deleted</w:t></w:r><w:ins><w:r><w:t>inserted</w:t></w:r></w:ins><w:r><w:txbxContent><w:p><w:r><w:delText>uncovered</w:delText></w:r></w:p></w:txbxContent></w:r></w:del></w:p><w:p><w:moveFrom><w:r><w:delText>moved</w:delText></w:r></w:moveFrom><w:r><w:delText>stranded</w:delText></w:r></w:p>",
        );
        let texts = dom.descendants(root, Some(&W::name("delText")));
        assert!(!covered_by_deletion(&dom, texts[0]));
        assert!(covered_by_deletion(&dom, texts[1]));
        assert!(!covered_by_deletion(&dom, texts[2]));
        fix_deleted_text(&mut dom, root);
        let live = dom
            .descendants(root, Some(&W::t()))
            .into_iter()
            .map(|n| dom.value(n))
            .collect::<Vec<_>>();
        assert_eq!(live, ["inserted", "uncovered", "moved", "stranded"]);
        assert_eq!(
            dom.value(dom.descendants(root, Some(&W::name("delText")))[0]),
            "deleted"
        );
    }

    #[test]
    fn bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text() {
        for control in SINGLE_VALUE_CONTROLS.into_iter().chain(["richText"]) {
            let mut dom = Dom::new();
            let root = xml(
                &mut dom,
                &format!(
                    "<w:sdt><w:sdtPr><w:{control}/></w:sdtPr><w:sdtContent><w:p><w:bookmarkStart w:id=\"1\"/><w:r><w:t>value</w:t></w:r><w:bookmarkEnd w:id=\"1\"/></w:p></w:sdtContent></w:sdt><w:p><w:bookmarkStart w:id=\"2\"/></w:p>"
                ),
            );
            let forbidden = bookmarks_in_single_value_controls(&dom, root);
            assert_eq!(forbidden.len(), if control == "richText" { 0 } else { 2 });
            drop_bookmarks_in_single_value_controls(&mut dom, root);
            assert_eq!(
                dom.descendants(root, Some(&W::name("bookmarkStart"))).len(),
                if control == "richText" { 2 } else { 1 }
            );
            assert_eq!(dom.value(root), "value");
        }
    }

    #[test]
    fn comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids() {
        let mut pkg = pkg();
        let expected = HashSet::from(["11111111".to_string()]);
        pkg.set_part("word/commentsExtended.xml", br#"<root><row/><row paraId="11111111" paraIdParent="11111111"/><row paraId="11111111"/><row paraId="22222222"/></root>"#.to_vec());
        let mut out = Vec::new();
        let graph = check_comments_extended(&pkg, &expected, &mut out).unwrap();
        assert_eq!(
            graph.keys,
            HashSet::from(["11111111".into(), "22222222".into()])
        );
        assert_eq!(out.len(), 4);
        assert!(out.iter().any(|f| f.code == "COMMENT_PARENT_CYCLE"));
        assert!(out.iter().any(|f| f.message.contains("has no paraId")));
        assert!(out.iter().any(|f| f.message.contains("duplicate")));
        assert!(out.iter().any(|f| f.message.contains("no matching")));
        pkg.set_part("word/commentsIds.xml", br#"<root><row/><row paraId="11111111"/><row paraId="11111111" durableId="22222222"/><row paraId="33333333" durableId="22222222"/></root>"#.to_vec());
        out.clear();
        assert_eq!(
            check_comments_ids(&pkg, &expected, &mut out),
            Some(HashSet::from(["22222222".into()]))
        );
        assert_eq!(out.len(), 5);
        pkg.set_part(
            "word/commentsExtensible.xml",
            br#"<root><row/><row durableId="22222222"/><row durableId="22222222"/></root>"#
                .to_vec(),
        );
        out.clear();
        check_comments_extensible(&pkg, Some(&HashSet::from(["22222222".into()])), &mut out);
        assert_eq!(out.len(), 2);
        out.clear();
        check_comments_extensible(&pkg, None, &mut out);
        assert_eq!(out.len(), 3);
        assert!(
            out.iter()
                .any(|f| f.message.contains("without commentsIds"))
        );
    }

    #[test]
    fn graph_cycles_are_reported_without_confusing_acyclic_chains() {
        let mut out = Vec::new();
        check_parent_cycles(
            &HashMap::from([("a".into(), "b".into()), ("b".into(), "c".into())]),
            &mut out,
        );
        assert!(out.is_empty());
        check_parent_cycles(
            &HashMap::from([
                ("a".into(), "b".into()),
                ("b".into(), "a".into()),
                ("tail".into(), "a".into()),
            ]),
            &mut out,
        );
        assert_eq!(out.len(), 3);
        assert!(out.iter().all(|f| f.code == "COMMENT_PARENT_CYCLE"));
    }

    #[test]
    fn namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes() {
        let mut pkg = pkg();
        pkg.set_part("custom.xml", format!(r#"<root xmlns:mc="{}" xmlns:known="urn:known" Requires="application" mc:Ignorable="known xml w14 unknown"><mc:Choice Requires="known"/><child mc:PreserveElements="known:thing unknown:thing"/></root>"#, MC::URI).into_bytes());
        let mut out = Vec::new();
        check_namespace_qname_context(&pkg, "custom.xml", &mut out);
        assert_eq!(out.len(), 3);
        assert_eq!(out.iter().filter(|f| f.repairable).count(), 1);
        assert!(out.iter().all(|f| f.code == "MC_UNBOUND_PREFIX"));
        for attribute in [
            "Ignorable",
            "PreserveAttributes",
            "PreserveElements",
            "ProcessContent",
            "MustUnderstand",
        ] {
            assert!(is_namespace_qname_list(&W::p(), &MC::name(attribute)));
        }
        assert!(!is_namespace_qname_list(&W::p(), &MC::name("other")));
        assert!(!is_namespace_qname_list(
            &W::p(),
            &XName::get("Requires", "")
        ));
        assert!(is_namespace_qname_list(
            &MC::name("Choice"),
            &XName::get("Requires", "")
        ));
    }

    #[test]
    fn absent_or_unreadable_parts_are_not_modified_by_edit_part() {
        let mut pkg = pkg();
        edit_part(&mut pkg, "absent.xml", |_, _| {
            panic!("missing part must not call editor")
        });
        assert!(pkg.part_bytes("absent.xml").is_none());
        pkg.set_part("empty.xml", Vec::new());
        edit_part(&mut pkg, "empty.xml", |_, _| {
            panic!("empty part must not call editor")
        });
        assert_eq!(pkg.part_bytes("empty.xml"), Some([].as_slice()));
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod validation_owner_boundary_tests {
    use super::*;
    fn package() -> PartFs {
        PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap()
    }
    #[test]
    fn relationship_target_diagnostics_distinguish_owned_payloads_from_unresolved_and_external_targets()
     {
        for (target, mode, present, missing) in [
            ("media/picture.bin", false, true, false),
            ("/word/media/picture.bin", false, true, false),
            ("word/media/picture.bin", false, true, false),
            ("media/missing.bin", false, false, true),
            ("https://example.invalid/picture.bin", false, false, false),
            ("http://example.invalid/picture.bin", false, false, false),
            ("mailto:owner@example.invalid", false, false, false),
            ("media/missing.bin", true, false, false),
        ] {
            let mut pkg = package();
            let id = if mode {
                pkg.add_document_relationship_external(
                    "word/document.xml",
                    "urn:test:payload",
                    target,
                )
            } else {
                pkg.add_document_relationship("word/document.xml", "urn:test:payload", target)
            };
            if present {
                pkg.set_part("word/media/picture.bin", vec![1, 2, 3, 4]);
            }
            pkg.set_part("word/document.xml", format!("<w:document xmlns:w='{}' xmlns:r='{}'><w:body><w:p><w:r><w:drawing r:embed=\"{id}\"/></w:r></w:p></w:body></w:document>", W::URI, R::URI).into_bytes());
            let before = pkg.to_zip().unwrap();
            let mut findings = Vec::new();
            check_relationship_integrity(&pkg, &mut findings);
            assert_eq!(
                findings
                    .iter()
                    .filter(|f| f.code == "MISSING_REL_TARGET")
                    .count(),
                usize::from(missing),
                "{target} external={mode}"
            );
            assert!(findings.iter().all(|f| f.code != "DANGLING_RELATIONSHIP"));
            if missing {
                assert!(
                    findings
                        .iter()
                        .any(|f| f.message.contains(&id) && f.message.contains(target))
                );
            }
            assert_eq!(pkg.to_zip().unwrap(), before);
        }
    }
    #[test]
    fn orphan_comment_cleanup_preserves_live_references_and_neighbor_run_properties() {
        for (content, remains) in [
            ("", false),
            ("<w:rPr><w:b/></w:rPr>", false),
            ("<w:rPr><w:i/></w:rPr><w:t>Keep</w:t>", true),
        ] {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body><w:p><w:commentRangeStart w:id='3'/><w:r>{content}<w:commentReference w:id='3'/></w:r><w:commentRangeEnd w:id='3'/><w:r><w:rPr><w:color w:val='246810'/></w:rPr><w:commentReference w:id='4'/><w:t>Live</w:t></w:r></w:p></w:body></w:document>", W::URI));
            let root = dom.root(doc).unwrap();
            drop_orphan_comment_anchors(&mut dom, root, &HashSet::from(["4".to_string()]));
            assert!(
                dom.descendants(root, Some(&W::name("commentRangeStart")))
                    .is_empty()
            );
            assert!(
                dom.descendants(root, Some(&W::name("commentRangeEnd")))
                    .is_empty()
            );
            let refs = dom.descendants(root, Some(&W::name("commentReference")));
            assert_eq!(refs.len(), 1);
            assert_eq!(dom.attribute(refs[0], &W::id()), Some("4"));
            assert_eq!(
                dom.descendants(root, Some(&W::r())).len(),
                if remains { 2 } else { 1 }
            );
            let texts: Vec<_> = dom
                .descendants(root, Some(&W::t()))
                .into_iter()
                .map(|n| dom.value(n))
                .collect();
            assert_eq!(
                texts,
                if remains {
                    vec!["Keep", "Live"]
                } else {
                    vec!["Live"]
                }
            );
            let live_pr = dom
                .element(dom.parent(refs[0]).unwrap(), &W::r_pr())
                .unwrap();
            assert_eq!(
                dom.attribute(dom.element(live_pr, &W::name("color")).unwrap(), &W::val()),
                Some("246810")
            );
            let once = dom.serialize_element(root);
            drop_orphan_comment_anchors(&mut dom, root, &HashSet::from(["4".to_string()]));
            assert_eq!(dom.serialize_element(root), once);
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod public_memory_validator_owner_boundary_tests {
    use super::*;
    fn package(body: &str) -> PartFs {
        let mut pkg =
            PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
        pkg.set_part("word/document.xml",format!("<w:document xmlns:w=\"{}\" xmlns:w14=\"{}\" xmlns:mc=\"{}\" mc:Ignorable=\"w14\"><w:body>{body}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>",W::URI,W14::URI,MC::URI).into_bytes());
        pkg
    }
    fn snapshot(pkg: &PartFs) -> Vec<(String, Vec<u8>)> {
        let mut parts = pkg.parts();
        parts.sort();
        parts
            .into_iter()
            .map(|name| {
                let bytes = pkg.part_bytes(&name).unwrap().to_vec();
                (name, bytes)
            })
            .collect()
    }

    #[test]
    fn missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair() {
        let mut pkg = package("<w:p><w:r><w:t>Owned source payload</w:t></w:r></w:p>");
        assert!(
            ring1(&pkg).is_empty(),
            "the source fixture has no unrelated Ring1 defects"
        );
        pkg.set_part(
            "customXml/source-owned.opaque",
            b"Independent opaque payload".to_vec(),
        );
        let source = snapshot(&pkg);
        let bytes = pkg.to_zip().unwrap();
        let expected = Finding {
            code: "MISSING_CONTENT_TYPE".into(),
            part: "customXml/source-owned.opaque".into(),
            path: String::new(),
            message: "part 'customXml/source-owned.opaque' has no content type".into(),
            word_fatal: true,
            repairable: false,
        };
        assert_eq!(ring1(&pkg), vec![expected.clone()]);
        let report = validate(&bytes).unwrap();
        assert_eq!(report, vec![expected.clone()]);
        let result = repair(&bytes).unwrap();
        assert_eq!(
            result.docx, bytes,
            "an unknown opaque part requires a maintainer-selected type, not guessed rewriting"
        );
        assert!(result.repaired.is_empty());
        assert_eq!(result.remaining, vec![expected]);
        assert_eq!(snapshot(&PartFs::open(&result.docx).unwrap()), source);
        let again = repair(&result.docx).unwrap();
        assert_eq!(again, result);
    }

    #[test]
    fn repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships() {
        // Ring1 deliberately diagnoses a serialized Word range violation.
        // The actual source package is otherwise complete: both comment
        // definitions have balanced body anchors and a real acyclic reply
        // graph. Identical paraId/textId aliases refer to the same source
        // owner; masked collisions must choose a free ID exactly once.
        for old in ["80000001", "00000000", "FFFFFFFF"] {
            let body = "<w:p w14:paraId=\"00000003\"><w:pPr><w:spacing w:after=\"80\"/></w:pPr><w:commentRangeStart w:id=\"1\"/><w:r><w:rPr><w:b/></w:rPr><w:t>Root source ação</w:t></w:r><w:commentRangeEnd w:id=\"1\"/><w:r><w:commentReference w:id=\"1\"/></w:r></w:p><w:p w14:paraId=\"00000004\" w14:textId=\"7FFFFFFF\"><w:commentRangeStart w:id=\"2\"/><w:r><w:rPr><w:i/></w:rPr><w:t>Reply source café</w:t></w:r><w:commentRangeEnd w:id=\"2\"/><w:r><w:commentReference w:id=\"2\"/></w:r></w:p>";
            let mut pkg = package(body);
            let comments = format!(
                "<w:comments xmlns:w=\"{}\" xmlns:w14=\"{}\" xmlns:mc=\"{}\" mc:Ignorable=\"w14\"><w:comment w:id=\"1\" w:author=\"Original reviewer\" w:initials=\"OR\" w:date=\"2025-02-03T04:05:06Z\"><w:p w14:paraId=\"00000001\"><w:pPr><w:spacing w:after=\"120\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Root comment ação</w:t></w:r></w:p></w:comment><w:comment w:id=\"2\" w:author=\"Reply reviewer\" w:initials=\"RR\" w:date=\"2025-03-04T05:06:07Z\"><w:p w14:paraId=\"{old}\" w14:textId=\"{old}\"><w:pPr><w:spacing w:after=\"240\"/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Reply comment café</w:t></w:r></w:p></w:comment></w:comments>",
                W::URI,
                W14::URI,
                MC::URI
            );
            let extended = format!(
                "<w15:commentsEx xmlns:w15=\"http://schemas.microsoft.com/office/word/2012/wordml\"><w15:commentEx w15:paraId=\"00000001\" w15:done=\"0\"/><w15:commentEx w15:paraId=\"{old}\" w15:paraIdParent=\"00000001\" w15:done=\"1\"/></w15:commentsEx>"
            );
            pkg.set_part("word/comments.xml", comments.clone().into_bytes());
            pkg.set_part("word/commentsExtended.xml", extended.clone().into_bytes());
            for (part, content_type, relationship) in COMMENT_FAMILY.into_iter().take(2) {
                pkg.add_content_type_override(&format!("/{part}"), content_type);
                pkg.add_document_relationship(
                    "word/document.xml",
                    relationship,
                    part.strip_prefix("word/").unwrap(),
                );
            }
            let source = snapshot(&pkg);
            let bytes = pkg.to_zip().unwrap();
            let original_records = crate::comments::list_comments(&bytes).unwrap();
            let before = validate(&bytes).unwrap();
            let range = before
                .iter()
                .filter(|finding| finding.code == "PARA_ID_OUT_OF_RANGE")
                .cloned()
                .collect::<Vec<_>>();
            let expected_range=["w14:paraId=\"","w14:textId=\""].map(|attribute|Finding {code:"PARA_ID_OUT_OF_RANGE".into(),part:"word/comments.xml".into(),path:String::new(),message:format!("{attribute} value '{old}' outside Word's range 1..0x7FFFFFFF (>= 0x80000000 or zero) in 'word/comments.xml' (id-paraid-overflow)"),word_fatal:true,repairable:true});
            assert_eq!(range, expected_range);
            let result = repair(&bytes).unwrap();
            assert!(
                result.remaining.is_empty(),
                "{old}: every reported range or graph inconsistency must be resolved, got {:?}",
                result.remaining
            );
            assert_eq!(result.repaired, expected_range);
            let output = PartFs::open(&result.docx).unwrap();
            let expected_comments = comments
                .replace(&format!("paraId=\"{old}\""), "paraId=\"00000002\"")
                .replace(&format!("textId=\"{old}\""), "textId=\"00000002\"");
            let expected_extended =
                extended.replace(&format!("paraId=\"{old}\""), "paraId=\"00000002\"");
            let expected = source
                .iter()
                .map(|(name, contents)| {
                    (
                        name.clone(),
                        match name.as_str() {
                            "word/comments.xml" => expected_comments.as_bytes().to_vec(),
                            "word/commentsExtended.xml" => expected_extended.as_bytes().to_vec(),
                            _ => contents.clone(),
                        },
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                snapshot(&output),
                expected,
                "{old}: repair must change only the owned identifier aliases, not source formats/text/history/relationships"
            );
            assert_eq!(
                crate::comments::list_comments(&result.docx).unwrap(),
                original_records,
                "{old}: complete comment records and reply topology survive"
            );
            assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), source);
            let again = repair(&result.docx).unwrap();
            assert_eq!(again.docx, result.docx);
            assert!(again.repaired.is_empty());
            assert!(again.remaining.is_empty());
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod literal_revision_repair_contract_tests {
    use super::*;

    fn tree(source: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{}" xmlns:r="{}"><w:body>{source}</w:body></w:document>"#,
            W::URI,
            R::URI
        ));
        (dom, doc)
    }

    #[test]
    fn duplicate_move_ids_keep_exact_range_pairings_and_every_unrelated_source_node() {
        let source = r#"<w:p><w:bookmarkStart w:id="40" w:name="owned"/><w:moveFromRangeStart w:id="7"/><w:r><w:t>first</w:t></w:r><w:moveFromRangeEnd w:id="7"/><w:moveFromRangeStart w:id="7"/><w:r><w:t>second</w:t></w:r><w:moveFromRangeEnd w:id="7"/><w:moveToRangeStart w:id="8"/><w:r><w:t>third</w:t></w:r><w:moveToRangeEnd w:id="8"/><w:moveToRangeStart w:id="8"/><w:r><w:t>fourth</w:t></w:r><w:moveToRangeEnd w:id="8"/><w:ins w:id="9" w:author="A" w:date="2026-10-09T00:00:00Z"><w:r><w:t>fifth</w:t></w:r></w:ins><w:ins w:id="9" w:author="B" w:date="2026-10-09T00:00:00Z"><w:r><w:t>sixth</w:t></w:r></w:ins><w:bookmarkEnd w:id="40"/></w:p>"#;
        let (mut dom, doc) = tree(source);
        let root = dom.root(doc).unwrap();
        let expected=source.replacen(r#"<w:moveFromRangeStart w:id="7"/><w:r><w:t>second</w:t></w:r><w:moveFromRangeEnd w:id="7"/>"#,r#"<w:moveFromRangeStart w:id="41"/><w:r><w:t>second</w:t></w:r><w:moveFromRangeEnd w:id="41"/>"#,1)
          .replacen(r#"<w:moveToRangeStart w:id="8"/><w:r><w:t>fourth</w:t></w:r><w:moveToRangeEnd w:id="8"/>"#,r#"<w:moveToRangeStart w:id="42"/><w:r><w:t>fourth</w:t></w:r><w:moveToRangeEnd w:id="42"/>"#,1)
          .replacen(r#"<w:ins w:id="9" w:author="B""#,r#"<w:ins w:id="43" w:author="B""#,1);
        let (expected_dom, expected_doc) = tree(&expected);
        renumber_duplicate_revision_ids(&mut dom, root);
        assert_eq!(
            dom.serialize_document(doc),
            expected_dom.serialize_document(expected_doc)
        );
        renumber_duplicate_revision_ids(&mut dom, root);
        assert_eq!(
            dom.serialize_document(doc),
            expected_dom.serialize_document(expected_doc)
        );
    }

    #[test]
    fn repair_declines_missing_ids_and_unmatched_ranges_without_inventing_content() {
        let source = r#"<w:p><w:ins><w:r><w:t>missing id</w:t></w:r></w:ins><w:moveFromRangeStart w:id="3"/><w:moveFromRangeStart w:id="3"/><w:moveToRangeEnd w:id="3"/><w:r><w:t>unmatched range retained</w:t></w:r></w:p>"#;
        let expected = source.replacen(
            r#"<w:moveFromRangeStart w:id="3"/><w:moveToRangeEnd"#,
            r#"<w:moveFromRangeStart w:id="4"/><w:moveToRangeEnd"#,
            1,
        );
        let (mut dom, doc) = tree(source);
        let root = dom.root(doc).unwrap();
        let (expected_dom, expected_doc) = tree(&expected);
        renumber_duplicate_revision_ids(&mut dom, root);
        assert_eq!(
            dom.serialize_document(doc),
            expected_dom.serialize_document(expected_doc)
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod paragraph_id_collision_owner_tests {
    use super::*;

    fn snapshot(pkg: &PartFs) -> Vec<(String, Vec<u8>)> {
        let mut names = pkg.parts();
        names.sort();
        names
            .into_iter()
            .map(|name| {
                let bytes = pkg.part_bytes(&name).unwrap().to_vec();
                (name, bytes)
            })
            .collect()
    }

    #[test]
    fn paragraph_id_collision_repair_preserves_all_owners_and_shared_references() {
        for old_id in ["00000000", "80000000", "80000001", "FFFFFFFF"] {
            let mut pkg =
                PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
            let document = format!(
                r#"<w:document xmlns:w="{}" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:body><w:p w14:paraId="00000001" w14:textId="00000002"><w:pPr><w:spacing w:after="80"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>existing ids</w:t></w:r></w:p><w:p w14:paraId="7FFFFFFF"><w:r><w:t>upper existing id</w:t></w:r></w:p><w:p w14:paraId="{old_id}" w14:textId="{old_id}"><w:pPr><w:keepNext/><w:pPrChange w:id="51" w:author="Source" w:date="2026-10-09T00:00:00Z"><w:pPr><w:spacing w:before="120"/></w:pPr></w:pPrChange></w:pPr><w:r><w:rPr><w:i/><w:color w:val="123456"/></w:rPr><w:t>ação τέλος</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#,
                W::URI
            );
            pkg.set_part("word/document.xml", document.as_bytes().to_vec());
            let refs = format!(
                r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="{old_id}" w15:paraIdParent="{old_id}" w15:done="0"/></w15:commentsEx>"#
            );
            pkg.set_part("word/commentsExtended.xml", refs.as_bytes().to_vec());
            pkg.add_content_type_override(
                "/word/commentsExtended.xml",
                "application/vnd.ms-word.commentsExtended+xml",
            );
            pkg.add_document_relationship(
                "word/document.xml",
                "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
                "commentsExtended.xml",
            );
            let before = snapshot(&pkg);
            let mut expected = before.clone();
            for (name, bytes) in &mut expected {
                if name == "word/document.xml" || name == "word/commentsExtended.xml" {
                    *bytes = String::from_utf8(bytes.clone())
                        .unwrap()
                        .replace(&format!("=\"{old_id}\""), "=\"00000003\"")
                        .into_bytes();
                }
            }
            renumber_out_of_range_para_ids(&mut pkg);
            assert_eq!(
                snapshot(&pkg),
                expected,
                "{old_id}: only the out-of-range shared id changes"
            );
            renumber_out_of_range_para_ids(&mut pkg);
            assert_eq!(
                snapshot(&pkg),
                expected,
                "{old_id}: a repeat preserves every part byte"
            );
            assert_ne!(before, expected);
        }
    }

    #[test]
    fn dangling_relationship_repair_keeps_empty_known_and_nonrelationship_owners() {
        let source = format!(
            r#"<w:document xmlns:w="{}" xmlns:r="{}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><w:body><w:p><w:pPr><w:spacing w:after="80"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:drawing><a:blip r:embed="missing" r:link="known"/></w:drawing><w:t>owned</w:t></w:r><w:hyperlink r:id="known"><w:r><w:t>keep</w:t></w:r></w:hyperlink><w:hyperlink r:id=""><w:r><w:t>empty reference</w:t></w:r></w:hyperlink><w:hyperlink r:id="missing"><w:r><w:t>attribute removed, text kept</w:t></w:r></w:hyperlink><w:ins w:id="52" w:author="Source" w:date="2026-10-09T00:00:00Z"><w:r><w:t>tracked source</w:t></w:r></w:ins></w:p><w:altChunk r:id="missing"/><w:sectPr><w:headerReference w:type="default" r:id="missing"/><w:footerReference w:type="default" r:id="known"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#,
            W::URI,
            R::URI
        );
        let expected = source
            .replace(r#" r:embed="missing""#, "")
            .replace(r#"<w:hyperlink r:id="missing">"#, "<w:hyperlink>")
            .replace(r#"<w:altChunk r:id="missing"/>"#, "")
            .replace(
                r#"<w:headerReference w:type="default" r:id="missing"/>"#,
                "",
            );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&source);
        let root = dom.root(doc).unwrap();
        let mut expected_dom = Dom::new();
        let expected_doc = expected_dom.parse_xdocument(&expected);
        let ids = HashSet::from(["known".to_string()]);
        drop_dangling_relationship_attributes(&mut dom, root, &ids);
        assert_eq!(
            dom.serialize_document(doc),
            expected_dom.serialize_document(expected_doc)
        );
        drop_dangling_relationship_attributes(&mut dom, root, &ids);
        assert_eq!(
            dom.serialize_document(doc),
            expected_dom.serialize_document(expected_doc)
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod authored_comment_family_validation_contract_tests {
    use super::*;

    fn package() -> PartFs {
        let mut pkg =
            PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
        pkg.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body><w:p><w:r><w:t>Owned body</w:t></w:r></w:p></w:body></w:document>",W::URI).into_bytes());
        pkg
    }
    fn frozen(pkg: &PartFs) -> Vec<(String, Vec<u8>)> {
        let mut parts = pkg.parts();
        parts.sort();
        parts
            .into_iter()
            .map(|part| {
                let bytes = pkg.part_bytes(&part).unwrap().to_vec();
                (part, bytes)
            })
            .collect()
    }

    #[test]
    fn auxiliary_comment_parts_without_their_source_definition_are_reported_individually() {
        for (part, content_type, rel_type) in &COMMENT_FAMILY[1..] {
            let mut pkg = package();
            pkg.set_part(part, b"<root/>".to_vec());
            pkg.add_content_type_override(part, content_type);
            pkg.add_document_relationship("word/document.xml", rel_type, &format!("/{part}"));
            let before = frozen(&pkg);
            let mut findings = Vec::new();
            check_comment_graph(&pkg, &mut findings);
            assert_eq!(
                findings,
                vec![Finding::new(
                    "COMMENT_PARTS_INCONSISTENT",
                    part,
                    "",
                    format!("'{part}' exists without word/comments.xml")
                )]
            );
            assert!(ring1(&pkg).contains(&findings[0]));
            assert_eq!(frozen(&pkg), before);
        }
    }

    #[test]
    fn each_comment_family_relationship_requires_its_exact_owned_part_and_content_type() {
        for (part, content_type, rel_type) in COMMENT_FAMILY {
            for case in 0..6 {
                let mut pkg = package();
                if case != 5 {
                    pkg.set_part(part, b"<root/>".to_vec());
                }
                pkg.add_content_type_override(
                    part,
                    if case == 0 {
                        "application/wrong"
                    } else {
                        content_type
                    },
                );
                let mut rid = None;
                let target = if case == 4 {
                    "wrong.xml".to_string()
                } else {
                    format!("/{part}")
                };
                if case != 1 {
                    rid = Some(if case == 3 {
                        pkg.add_document_relationship_external(
                            "word/document.xml",
                            rel_type,
                            &format!("/{part}"),
                        )
                    } else {
                        pkg.add_document_relationship("word/document.xml", rel_type, &target)
                    });
                }
                if case == 2 {
                    pkg.add_document_relationship(
                        "word/document.xml",
                        rel_type,
                        &format!("/{part}"),
                    );
                }
                let before = frozen(&pkg);
                let mut findings = Vec::new();
                check_comment_family_packaging(&pkg, "word/document.xml", &mut findings);
                let expected = match case {
                    0 => Finding::new(
                        "COMMENT_PARTS_INCONSISTENT",
                        part,
                        "",
                        format!("'{part}' has the wrong content type (expected '{content_type}')"),
                    ),
                    1 | 2 => Finding::new(
                        "COMMENT_PARTS_INCONSISTENT",
                        "word/document.xml",
                        "",
                        format!(
                            "'word/document.xml' needs exactly one relationship to '{part}', found {}",
                            if case == 1 { 0 } else { 2 }
                        ),
                    ),
                    3 | 4 => Finding::new(
                        "COMMENT_PARTS_INCONSISTENT",
                        "word/document.xml",
                        "",
                        format!(
                            "comment relationship '{}' on 'word/document.xml' resolves to '{}' instead of '{part}'",
                            rid.unwrap(),
                            if case == 3 {
                                format!("/{part}")
                            } else {
                                "wrong.xml".into()
                            }
                        ),
                    ),
                    _ => Finding::new(
                        "COMMENT_PARTS_INCONSISTENT",
                        "word/document.xml",
                        "",
                        format!(
                            "'word/document.xml' has a relationship for missing comment part '{part}'"
                        ),
                    ),
                };
                assert_eq!(findings, vec![expected], "{part}/case{case}");
                assert_eq!(frozen(&pkg), before);
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod retained_namespace_finding_source_contract_tests {
    use super::*;

    #[test]
    fn repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content() {
        for known in [false, true] {
            let ignorable = if known {
                "w14 UnknownFoo UnknownBar"
            } else {
                "UnknownFoo UnknownBar"
            };
            let mut pkg =
                PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
            let body = format!(
                "<w:document xmlns:w='{}' xmlns:mc='{}' mc:Ignorable='{ignorable}'><w:body><w:p><w:pPr><w:spacing w:after='120'/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Owned body</w:t></w:r></w:p></w:body></w:document>",
                W::URI,
                MC::URI
            );
            let header = format!(
                "<w:hdr xmlns:w='{}' xmlns:mc='{}' mc:Ignorable='UnknownFoo'><w:p><w:r><w:rPr><w:i/></w:rPr><w:t>Owned header</w:t></w:r></w:p></w:hdr>",
                W::URI,
                MC::URI
            );
            pkg.set_part("word/document.xml", body.into_bytes());
            pkg.set_part("word/header1.xml", header.into_bytes());
            pkg.add_content_type_override(
                "word/header1.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            );
            pkg.add_document_relationship(
                "word/document.xml",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                "header1.xml",
            );
            let source = pkg.to_zip().unwrap();
            let before = validate(&source).unwrap();
            assert_eq!(before.len(), 3 + usize::from(known), "{before:?}");
            assert!(before.iter().all(|f| f.code == "MC_UNBOUND_PREFIX"));
            assert_eq!(
                before
                    .iter()
                    .filter(|f| f.part == "word/document.xml")
                    .count(),
                2 + usize::from(known)
            );
            assert_eq!(
                before
                    .iter()
                    .filter(|f| f.part == "word/header1.xml")
                    .count(),
                1
            );
            let out = repair(&source).unwrap();
            assert_eq!(
                out.repaired,
                before
                    .iter()
                    .filter(|finding| finding.repairable)
                    .cloned()
                    .collect::<Vec<_>>()
            );
            assert_eq!(out.repaired.len(), usize::from(known));
            assert_eq!(
                out.remaining,
                before
                    .iter()
                    .filter(|finding| !finding.repairable)
                    .cloned()
                    .collect::<Vec<_>>()
            );
            let after = PartFs::open(&out.docx).unwrap();
            for part in pkg.parts() {
                let old = pkg.part_bytes(&part).unwrap();
                let new = after.part_bytes(&part).unwrap();
                if matches!(part.as_str(), "word/document.xml" | "word/header1.xml") {
                    let mut dom = Dom::new();
                    let old_document = dom.parse_xdocument(std::str::from_utf8(old).unwrap());
                    let old_root = dom.root(old_document).unwrap();
                    if known && part == "word/document.xml" {
                        dom.set_attribute_value(
                            old_root,
                            &crate::xmllinq::XNamespace::xmlns().name("w14"),
                            Some(crate::namespaces::W14::URI),
                        );
                    }
                    let expected = dom.serialize_element(old_root);
                    let new_document = dom.parse_xdocument(std::str::from_utf8(new).unwrap());
                    let new_root = dom.root(new_document).unwrap();
                    assert_eq!(dom.serialize_element(new_root), expected, "{part}");
                } else {
                    assert_eq!(new, old, "{part}");
                }
            }
            assert_eq!(pkg.to_zip().unwrap(), source);
            let again = repair(&out.docx).unwrap();
            assert_eq!(again.remaining, out.remaining);
            assert!(again.repaired.is_empty());
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod duplicate_drawing_owned_repair_contract_tests {
    use super::*;

    #[test]
    fn duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner() {
        let mut pkg =
            PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
        let drawing = |id, name, color| {
            format!(
                "<w:r><w:rPr><w:b/></w:rPr><w:drawing><wp:inline><wp:extent cx='914400' cy='457200'/><wp:docPr id='{id}' name='{name}' descr='Owned shape'/><a:graphic><a:graphicData uri='http://schemas.microsoft.com/office/word/2010/wordprocessingShape'><wps:wsp><wps:cNvSpPr/><wps:spPr><a:xfrm><a:off x='0' y='0'/><a:ext cx='914400' cy='457200'/></a:xfrm><a:prstGeom prst='rect'><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val='{color}'/></a:solidFill></wps:spPr><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
            )
        };
        let xml = |first, second| {
            format!(
                "<w:document xmlns:w='{}' xmlns:wp='http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing' xmlns:a='http://schemas.openxmlformats.org/drawingml/2006/main' xmlns:wps='http://schemas.microsoft.com/office/word/2010/wordprocessingShape'><w:body><w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Owned prefix</w:t></w:r>{}{}<w:r><w:t>Owned suffix</w:t></w:r></w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",
                W::URI,
                drawing(first, "First", "123456"),
                drawing(second, "Second", "987654")
            )
        };
        pkg.set_part("word/document.xml", xml(7, 7).into_bytes());
        let source = pkg.to_zip().unwrap();
        let findings = validate(&source).unwrap();
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, "DUPLICATE_DOCPR_ID");
        assert!(findings[0].repairable);
        assert!(!findings[0].word_fatal);
        let repaired = repair(&source).unwrap();
        assert_eq!(repaired.repaired, findings);
        assert!(repaired.remaining.is_empty(), "{:?}", repaired.remaining);
        let output = PartFs::open(&repaired.docx).unwrap();
        let mut dom = Dom::new();
        let expected = dom.parse_xdocument(&xml(1, 2));
        let expected_root = dom.root(expected).unwrap();
        let expected = dom.serialize_element(expected_root);
        let actual = dom.parse_xdocument(&output.part_string("word/document.xml").unwrap());
        let actual_root = dom.root(actual).unwrap();
        assert_eq!(dom.serialize_element(actual_root), expected);
        for part in pkg
            .parts()
            .into_iter()
            .filter(|part| part != "word/document.xml")
        {
            assert_eq!(output.part_bytes(&part), pkg.part_bytes(&part), "{part}");
        }
        assert_eq!(pkg.to_zip().unwrap(), source);
        let again = repair(&repaired.docx).unwrap();
        assert!(again.repaired.is_empty());
        assert!(again.remaining.is_empty());
        assert_eq!(again.docx, repaired.docx);
    }
}
