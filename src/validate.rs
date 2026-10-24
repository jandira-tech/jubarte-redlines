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
//! | `DUPLICATE_REVISION_ID` | no: Word opens same-kind duplicates (102 probes, 2026-10-18, #1137); the OOXML validator refuses them (`Sem_UniqueAttributeValue`); error diagnostic | yes: renumbered |
//! | `DUPLICATE_DOCPR_ID` | no (lead) | yes: renumbered |
//! | `PARA_ID_OUT_OF_RANGE` | no: Word opens it; the schema forbids it | yes: renumbered |
//! | `TEXT_INSIDE_DELETION`, `DELTEXT_OUTSIDE_DELETION`, `MOVEFROM_WITH_DELTEXT` | yes | yes: `w:t` and `w:delText` swapped |
//! | `INSTR_TEXT_INSIDE_DELETION` | no (not probed in Word) | yes |
//! | `BOOKMARK_IN_SINGLE_VALUE_CONTROL` | yes | yes: the bookmark is dropped |
//! | `COMMENT_ANCHOR_ORPHAN` | yes | yes: the anchor is dropped |
//! | `COMMENT_RANGE_ORPHAN` | no: Word opens it | yes: the marker is dropped |
//! | `COMMENT_PARTS_INCONSISTENT` | yes | no |
//! | `COMMENT_UNANCHORED`, `COMMENT_ID_REPEATED`, `COMMENT_METADATA_MISMATCH`, `COMMENT_PARENT_CYCLE` | no: Word opens it | no |
//! | `RUN_OUTSIDE_PARAGRAPH` | yes | no |
//! | `FIELD_UNBALANCED`, `FIELD_SPLIT_DELETION` | yes | no |
//! | `FIELD_CODE_STATE` | no | no |
//! | `EMPTY_FIELD_CODE`, `ROW_WITHOUT_CELL`, `SECTPR_NOT_LAST` | yes | no |
//! | `NESTED_SAME_REVISION` | no: Word opens direct same-kind nests faithfully (info); alternating nests misread reject-all (warning); probe 2026-10-17, #1134 | no |
//! | `CELL_WITHOUT_PARAGRAPH` | yes | yes: an empty paragraph is appended |
//! | `NOTE_REFERENCE_DANGLING` | yes | no |
//! | `DUPLICATE_EXPANDED_ATTRIBUTE` | yes: aliased duplicate refused by Word (#650) | no |
//! | `NESTED_TEXT_BOX` | yes: A/C/E crash; F/H refuse to load; B/D/G open (Word probes 2026-10-18, #1132) | no |
//! | `COMMENT_RANGE_UNPAIRED`, `BARE_RUN_IN_DELETED_TEXTBOX`, `BOOKMARK_*` | no | no |
//! | `MOVE_RANGE_COUNT_MISMATCH` | no: Word opens it but Reject All can duplicate moved text (#1214); warning | no |
//! | `DATA_BINDING_WITHOUT_STORE` | no: Word opens it and shows the bound control empty (#1427); warning | no |
//! | `UNTRACKED_EDIT`, `FOREIGN_AUTHOR` | n/a | no |
//!
//! Revision ids are unique per OOXML record type within a part, as
//! OpenXmlValidator checks them (`Sem_UniqueAttributeValue`): two `w:ins`
//! runs sharing an id are a finding, a `w:ins` and a `w:del` sharing one are
//! not, and neither is the `pPrChange`/`rPrChange` pair Word writes in style
//! histories. Word opens every duplicate-id probe (102 files, 2026-10-18,
//! #1137), so this is an error diagnostic, not Word-fatal.
//!
//! The comment codes follow a 98-file Word probe (2026-10-10). Word opens
//! a comment whatever its range markers: missing, reversed, repeated, at
//! block level, across paragraphs or cells. It shows the comment only from
//! a `w:commentReference` in a paragraph of the main story. It refuses a
//! reference whose id has no definition, two references for one id, any
//! anchor when there is no comments part, and a run outside a paragraph.
//! Of the parts beside `word/comments.xml` it refuses an id that is not
//! hexadecimal, an entry without its paraId, and a part not related exactly
//! once, to itself, with its content type; any other disagreement between
//! them (missing, extra or repeated entries, out-of-range ids, dangling or
//! cyclic parents) opens and shows the comment.

use std::collections::{HashMap, HashSet};
use std::fmt;

use quick_xml::events::Event;
use serde::{Deserialize, Serialize};

use crate::admission::{AdmissionError, InputLimits};
use crate::changes::{Change, ChangeError, ChangeFilter, list_changes, reject_changes};
use crate::inspect::InspectError;
use crate::namespaces::{MC, R, VML, W, W14, WP};
use crate::opc::PartFs;
use crate::xmllinq::serialize::{well_known_namespace, well_known_prefix};
use crate::xmllinq::{Dom, NodeId, XName};

/// One thing wrong with a package, from any rule set: Word validity
/// (`word`), tracked-edit discipline (`edits`), or the [`crate::audit`] sets
/// (`a11y`, `style`, `structure`). For `word` and `edits`, `location` is the
/// element chain inside `part` (`w:body[0]/w:p[3]/w:r[2]`, each name indexed
/// among its same-named siblings, root excluded), stable across runs, or a
/// paragraph id (`p3`); empty for a package-level finding. For the audit
/// sets it is the paragraph id (`p5`, `header1`, `header1.p1`:
/// [`crate::ids::paragraph_id`]) or, when no paragraph applies, the part
/// name. `ref` is the same text as `location`. Messages name paragraphs by
/// the same ids.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable code (`TEXT_INSIDE_DELETION`, `MC_UNBOUND_PREFIX`, ...).
    pub code: String,
    /// The rule set it belongs to: one of `RULE_SETS`.
    pub set: String,
    /// `error`, `warning` or `info`. A `word` finding is an `error` when
    /// Word-fatal or OOXML-invalid (`DUPLICATE_REVISION_ID`); otherwise a
    /// `warning` (a lead), or `info` for direct
    /// same-kind revision nesting that Word reads faithfully.
    pub severity: String,
    /// Package part the finding sits in.
    pub part: String,
    /// Where in the part (see the type's doc), or empty.
    pub location: String,
    /// The short id the text listing prints for [`Finding::location`].
    #[serde(default, rename = "ref")]
    pub r#ref: String,
    /// Human-readable detail.
    pub message: String,
    /// True when Word refuses or repairs the file for this (Ring-1 and
    /// Ring-3 evidence); false for leads (`DUPLICATE_DOCPR_ID`) and for
    /// every finding outside the `word` set.
    pub word_fatal: bool,
    /// True when [`repair`] fixes it deterministically.
    pub repairable: bool,
}

/// Every rule set `validate` runs, `word` (the default) first.
pub(crate) const RULE_SETS: &[&str] = &["word", "edits", "a11y", "style", "structure"];

impl Finding {
    /// A `word` finding with `word_fatal` and `repairable` taken from the
    /// code's table ([`traits`]). OOXML-invalid duplicate revision ids
    /// are errors, though Word opens them.
    pub fn new(
        code: &str,
        part: &str,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let (word_fatal, repairable) = traits(code);
        let location = location.into();
        let r#ref = location.clone();
        Finding {
            code: code.to_string(),
            set: "word".to_string(),
            severity: if word_fatal || code == "DUPLICATE_REVISION_ID" {
                "error"
            } else {
                "warning"
            }
            .to_string(),
            part: part.to_string(),
            location,
            r#ref,
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
        | "DUPLICATE_EXPANDED_ATTRIBUTE"
        | "MISSING_REL_TARGET"
        | "PICTURE_BULLET_UNDEFINED"
        | "COMMENT_PARTS_INCONSISTENT"
        | "FIELD_UNBALANCED"
        | "RUN_OUTSIDE_PARAGRAPH"
        | "FIELD_SPLIT_DELETION"
        | "EMPTY_FIELD_CODE"
        | "ROW_WITHOUT_CELL"
        | "SECTPR_NOT_LAST"
        | "NESTED_TEXT_BOX"
        | "NOTE_REFERENCE_DANGLING" => (true, false),
        "DUPLICATE_REVISION_ID"
        | "DUPLICATE_DOCPR_ID"
        | "INSTR_TEXT_INSIDE_DELETION"
        | "PARA_ID_OUT_OF_RANGE"
        | "COMMENT_RANGE_ORPHAN" => (false, true),
        _ => (false, false),
    }
}

/// Codes the Ring-1 checks own; the triage checks' findings under these
/// codes are the same problems seen twice and are left out of [`validate`].
const RING1_CODES: [&str; 29] = [
    "DATA_BINDING_WITHOUT_STORE",
    "MOVE_RANGE_COUNT_MISMATCH",
    "NESTED_TEXT_BOX",
    "NESTED_PPR",
    "MC_UNBOUND_PREFIX",
    "MISSING_CONTENT_TYPE",
    "MALFORMED_XML",
    "DUPLICATE_EXPANDED_ATTRIBUTE",
    "DANGLING_RELATIONSHIP",
    "DUPLICATE_RID",
    "MISSING_REL_TARGET",
    "PICTURE_BULLET_UNDEFINED",
    "DUPLICATE_REVISION_ID",
    "DUPLICATE_DOCPR_ID",
    "PARA_ID_OUT_OF_RANGE",
    "TEXT_INSIDE_DELETION",
    "INSTR_TEXT_INSIDE_DELETION",
    "DELTEXT_OUTSIDE_DELETION",
    "MOVEFROM_WITH_DELTEXT",
    "BOOKMARK_IN_SINGLE_VALUE_CONTROL",
    "COMMENT_ANCHOR_ORPHAN",
    "COMMENT_RANGE_ORPHAN",
    "COMMENT_PARTS_INCONSISTENT",
    "COMMENT_PARENT_CYCLE",
    "RUN_OUTSIDE_PARAGRAPH",
    "COMMENT_UNANCHORED",
    "COMMENT_ID_REPEATED",
    "COMMENT_METADATA_MISMATCH",
    "STYLE_NAME_COMMA",
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
    /// The rules asked for cannot run: an unknown rule, or `edits` without
    /// the original document and author.
    Usage(String),
    /// The layout pass an audit rule reads failed.
    Layout(String),
}

impl fmt::Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(e) => write!(f, "DOCX refused: {e}"),
            Self::Package(m) => write!(f, "invalid package: {m}"),
            Self::Changes(e) => write!(f, "{e}"),
            Self::Inspect(e) => write!(f, "{e}"),
            Self::Usage(m) => write!(f, "{m}"),
            Self::Layout(m) => write!(f, "layout pass failed: {m}"),
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

/// [`validate`] with rule sets: `word` (the default), `edits`, and the
/// [`crate::audit`] sets or codes (`a11y`, `style`, `structure`,
/// `LITERAL_BULLET`, ...). `original` (the document before the edits and the
/// author every change must carry) adds `edits`. `layout` is the pass the
/// audit's `FONT_SUBSTITUTED` and `STALE_FIELD_CACHE` read
/// ([`crate::audit::word_layout`] natively; `None` in a build without the
/// renderer). Findings come word first (by part, location, code and
/// message), then edits, then the audit's in its rule order.
///
/// # Errors
///
/// [`ValidateError::Usage`] for an unknown rule, or `edits` without
/// `original`; otherwise as [`validate`] and [`audit_tracked`].
pub fn validate_with(
    docx: &[u8],
    rules: &[&str],
    original: Option<(&[u8], &str)>,
    layout: Option<crate::audit::LayoutPass<'_>>,
) -> Result<Vec<Finding>, ValidateError> {
    let mut word = rules.is_empty();
    let mut edits = original.is_some();
    let mut audit = Vec::new();
    for &rule in rules {
        match rule {
            "word" => word = true,
            "edits" => edits = true,
            _ if crate::audit::is_rule(rule) => audit.push(rule),
            _ => return Err(ValidateError::Usage(unknown_rule(rule))),
        }
    }
    let mut out = Vec::new();
    if word {
        out.extend(validate(docx)?);
    }
    if edits {
        let Some((before, author)) = original else {
            return Err(ValidateError::Usage(
                "rules edits needs --author, and --original or --baseline".to_string(),
            ));
        };
        out.extend(audit_tracked(before, docx, author)?);
    }
    if !audit.is_empty() {
        let report =
            crate::audit::audit_report_with(docx, &audit, layout).map_err(|e| match e {
                crate::audit::AuditError::Inspect(e) => ValidateError::Inspect(e),
                crate::audit::AuditError::Layout(m) => ValidateError::Layout(m),
                crate::audit::AuditError::UnknownRule(rule) => {
                    ValidateError::Usage(unknown_rule(&rule))
                }
            })?;
        out.extend(report.findings.into_iter().map(Finding::from));
    }
    Ok(out)
}

/// [`validate_with`] on `docx`, split by the findings of `base`, the
/// document the edits started from, under the same rules and against the
/// same original ([`baseline_keyed`], each side's keys read in its own
/// document by [`identities`]). That original is `base` unless `original`
/// names another; `author` turns the `edits` rule on as `validate_with`'s
/// `original` does, so the other party's changes `base` already had are
/// the input's, not the edit's.
///
/// # Errors
///
/// As [`validate_with`], for either document.
pub fn validate_baselined(
    docx: &[u8],
    base: &[u8],
    rules: &[&str],
    original: Option<&[u8]>,
    author: Option<&str>,
    mut layout: Option<crate::audit::LayoutPass<'_>>,
) -> Result<Baselined, ValidateError> {
    let against = original.or(Some(base)).zip(author);
    let findings = validate_with(
        docx,
        rules,
        against,
        layout
            .as_mut()
            .map(|pass| &mut **pass as crate::audit::LayoutPass<'_>),
    )?;
    let inherited = validate_with(base, rules, against, layout)?;
    let keys = identities(docx, &findings);
    let inherited_keys = identities(base, &inherited);
    Ok(baseline_keyed(findings, &keys, inherited, &inherited_keys))
}

/// The usage message for a rule that is neither a set nor an audit code.
fn unknown_rule(rule: &str) -> String {
    let codes: Vec<&str> = crate::audit::RULES.iter().map(|rule| rule.0).collect();
    format!(
        "unknown rule {rule}; expected one of {} or {}",
        RULE_SETS.join(", "),
        codes.join(", ")
    )
}

/// True when `findings` fail a run: any `word` or `edits` finding, any
/// `error`, or (`strict`) any `warning`.
pub fn fails(findings: &[Finding], strict: bool) -> bool {
    findings.iter().any(|f| {
        matches!(f.set.as_str(), "word" | "edits")
            || f.severity == "error"
            || (strict && f.severity == "warning")
    })
}

/// One line per finding, then a summary. A line is `* CODE\tSEVERITY\t
/// PART#REF\tMESSAGE` for `word` and `edits` (`*` marks Word-fatal, a
/// space otherwise) and `  CODE\tSEVERITY\tREF\tMESSAGE` for the audit
/// sets, where `REF` is [`Finding::location`] (a paragraph id such as
/// `p5`, an element chain or a part name). The summary is `no findings` or
/// `N finding(s): E error, W warning, I info`.
pub fn findings_text(findings: &[Finding]) -> String {
    let mut out = String::new();
    for f in findings {
        out.push_str(&finding_line(f));
    }
    if findings.is_empty() {
        out.push_str("no findings\n");
    } else {
        let count = |severity: &str| findings.iter().filter(|f| f.severity == severity).count();
        out.push_str(&format!(
            "{} finding(s): {} error, {} warning, {} info\n",
            findings.len(),
            count("error"),
            count("warning"),
            count("info")
        ));
    }
    out
}

/// One [`findings_text`] line, newline included.
fn finding_line(f: &Finding) -> String {
    let star = if f.word_fatal { '*' } else { ' ' };
    let place = if matches!(f.set.as_str(), "word" | "edits") {
        format!("{}#{}", f.part, f.location)
    } else {
        f.location.clone()
    };
    format!(
        "{star} {}\t{}\t{place}\t{}\n",
        f.code, f.severity, f.message
    )
}

/// What makes a finding in one version of a document the same finding in
/// another ([`Finding::identity`]): its code, set and part, where it sits
/// (its paragraph id or element chain, empty for a part-level finding), its
/// message with element indices dropped (`w:p[3]` reads `w:p`) and
/// paragraph ids spelled `p#`, and what surrounds it there ([`Place`]). A
/// finding named by an entity, a durableId or a bookmark, keeps that
/// entity in its message.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FindingKey {
    /// [`Finding::code`].
    pub code: String,
    /// [`Finding::set`].
    pub set: String,
    /// [`Finding::part`].
    pub part: String,
    /// [`Finding::location`]; from [`identities`], an element chain with
    /// its indices dropped, or `p#` for a paragraph id.
    pub location: String,
    /// [`Finding::message`], normalised.
    pub message: String,
    /// What surrounds the finding in its document, read by [`identities`];
    /// empty from [`Finding::identity`].
    pub place: Place,
}

/// What surrounds a finding in its document ([`identities`]), each part
/// `None` where there is none: what [`baseline_keyed`] tells two findings
/// of one kind apart by.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Place {
    /// The text of the paragraph the finding sits in; outside one, of its
    /// table, or else of its element with the attributes of that element
    /// and of everything holding it (a style's `w:styleId`), leaving out
    /// the ones an edit renumbers.
    pub text: Option<String>,
    /// That paragraph's `w14:paraId`.
    pub paragraph_id: Option<String>,
    /// The tracked change, or comment, the finding sits in: its element
    /// and `w:id` (`del 3`); for `FOREIGN_AUTHOR`, the change's id (`r3`).
    /// `jubarte edit` keeps the input's `w:id` values.
    pub revision: Option<String>,
    /// The text of the element before the one [`Self::text`] reads, of its
    /// name and in the same parent.
    pub previous: Option<String>,
    /// The text of the element after it, of its name and in the same
    /// parent.
    pub next: Option<String>,
}

impl Finding {
    /// This finding's [`FindingKey`]: [`baseline`] matches findings by it.
    pub fn identity(&self) -> FindingKey {
        FindingKey {
            code: self.code.clone(),
            set: self.set.clone(),
            part: self.part.clone(),
            location: self.location.clone(),
            message: normalised(&self.message),
            place: Place::default(),
        }
    }
}

/// Each of `findings`' [`FindingKey`], read in `docx`, the document they
/// were found in. A finding at an element chain or a paragraph id, in any
/// story, keys by the chain with its indices dropped (`p#` for the id) and
/// by its [`Place`], so it keeps its identity when an edit moves or
/// renumbers it. A `FOREIGN_AUTHOR` change keys by its kind, author and
/// text, and by its id. Anything else keys as [`Finding::identity`] does.
#[must_use]
pub fn identities(docx: &[u8], findings: &[Finding]) -> Vec<FindingKey> {
    let mut anchors = Anchors::new(docx);
    findings
        .iter()
        .map(|f| {
            let mut key = f.identity();
            if f.code == "FOREIGN_AUTHOR" {
                if let Some((id, rest)) = f.message.split_once(' ')
                    && let Some(change) = anchors.change(id)
                {
                    key.message = normalised(rest);
                    key.place = Place {
                        text: Some(change),
                        revision: Some(id.to_string()),
                        ..Place::default()
                    };
                }
            } else if let Some((location, place)) = anchors.place(&f.part, &f.location) {
                key.location = location;
                key.place = place;
            }
            key
        })
        .collect()
}

/// What [`identities`] reads a document's places by, each part parsed once.
struct Anchors<'a> {
    docx: &'a [u8],
    pkg: Option<PartFs>,
    parts: HashMap<String, Option<Part>>,
    changes: Option<HashMap<String, String>>,
}

impl<'a> Anchors<'a> {
    fn new(docx: &'a [u8]) -> Self {
        Anchors {
            docx,
            pkg: PartFs::open(docx).ok(),
            parts: HashMap::new(),
            changes: None,
        }
    }

    /// `kind, author, text` of change `id` (`r3`).
    fn change(&mut self, id: &str) -> Option<String> {
        let docx = self.docx;
        self.changes
            .get_or_insert_with(|| {
                list_changes(docx)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| {
                        let anchor = format!(
                            "{:?}\u{1f}{}\u{1f}{}",
                            c.kind,
                            c.author.unwrap_or_default(),
                            c.text
                        );
                        (c.id, anchor)
                    })
                    .collect()
            })
            .get(id)
            .cloned()
    }

    /// The location `location` in `part` keys by, and its [`Place`];
    /// `None` when it is neither an element chain of the part nor one of
    /// the part's paragraph ids.
    fn place(&mut self, part: &str, location: &str) -> Option<(String, Place)> {
        if location.is_empty() {
            return None;
        }
        let pkg = self.pkg.as_ref()?;
        let part = self
            .parts
            .entry(part.to_string())
            .or_insert_with(|| parse_part(pkg, part).map(|(dom, root)| Part::new(dom, root)))
            .as_mut()?;
        if location.contains('[') {
            let node = part.resolve(location)?;
            let chain: Vec<&str> = location
                .split('/')
                .map(|step| step.split_once('[').map_or(step, |(name, _)| name))
                .collect();
            Some((chain.join("/"), part.around(node)))
        } else {
            let (story, index) = crate::ids::paragraph_place(location)?;
            let node = *part.paragraphs(&story).get(index)?;
            Some(("p#".to_string(), part.around(node)))
        }
    }
}

/// A parsed part and the lookups [`Anchors`] made in it, so reading every
/// finding's place costs one walk of the part.
struct Part {
    dom: Dom,
    root: NodeId,
    /// Each parent's child elements of one name, keyed by the parent, the
    /// namespace and the local name.
    children: HashMap<(NodeId, String, String), Vec<NodeId>>,
    /// Where each element listed in `children` stands in its list.
    rank: HashMap<NodeId, usize>,
    /// The part's paragraphs in the order its paragraph ids count them.
    paragraphs: Option<Vec<NodeId>>,
}

impl Part {
    fn new(dom: Dom, root: NodeId) -> Self {
        Part {
            dom,
            root,
            children: HashMap::new(),
            rank: HashMap::new(),
            paragraphs: None,
        }
    }

    /// The element `chain` ([`element_path`]) names under the root.
    fn resolve(&mut self, chain: &str) -> Option<NodeId> {
        let Part {
            dom,
            root,
            children,
            rank,
            ..
        } = self;
        let mut node = *root;
        for step in chain.split('/') {
            let (name, index) = step.strip_suffix(']')?.split_once('[')?;
            let index: usize = index.parse().ok()?;
            let (namespace, local) = match name.split_once(':') {
                Some((prefix, local)) => (well_known_namespace(prefix)?, local),
                None => ("", name),
            };
            node = *named(dom, children, rank, node, namespace, local).get(index)?;
        }
        Some(node)
    }

    /// The paragraphs of `story`, this part's, in the order its paragraph
    /// ids (`p3`, `header1.p2`) count them: the order `inspect` and the
    /// audit number them in.
    fn paragraphs(&mut self, story: &str) -> &[NodeId] {
        let Part {
            dom,
            root,
            paragraphs,
            ..
        } = self;
        paragraphs.get_or_insert_with(|| {
            if story == "body" {
                dom.element(*root, &W::body())
                    .map(|body| crate::inspect::body_paragraph_nodes(dom, body))
                    .unwrap_or_default()
            } else {
                crate::inspect::story_paragraph_nodes(dom, *root)
            }
        })
    }

    /// The [`Place`] of `node`: what it sits in, read from its paragraph
    /// (else its table, else itself), and the change around it.
    fn around(&mut self, node: NodeId) -> Place {
        let Part {
            dom,
            children,
            rank,
            ..
        } = self;
        let held: Vec<NodeId> = std::iter::once(node)
            .chain(dom.ancestors(node, None))
            .collect();
        let anchor = [W::p(), W::tbl()]
            .iter()
            .find_map(|name| held.iter().copied().find(|&n| dom.name_is(n, name)));
        let revision = held.iter().find_map(|&n| {
            let id = dom.attribute(n, &W::id())?;
            dom.attribute(n, &W::author())?;
            Some(format!("{} {id}", dom.name(n)?.local_name()))
        });
        let (anchor, text) = match anchor {
            Some(anchor) => (anchor, dom.value(anchor)),
            None => (node, signature(dom, &held)),
        };
        let paragraph_id = if dom.name_is(anchor, &W::p()) {
            dom.attribute(anchor, &W14::name("paraId"))
                .map(str::to_string)
        } else {
            None
        };
        let (previous, next) = match (dom.parent(anchor), dom.name(anchor)) {
            (Some(parent), Some(name)) => {
                let siblings = named(
                    dom,
                    children,
                    rank,
                    parent,
                    name.namespace_name(),
                    name.local_name(),
                );
                let at = rank.get(&anchor).copied();
                let text =
                    |i: Option<usize>| i.and_then(|i| siblings.get(i)).map(|&n| dom.value(n));
                (
                    text(at.and_then(|at| at.checked_sub(1))),
                    text(at.map(|at| at + 1)),
                )
            }
            _ => (None, None),
        };
        Place {
            text: Some(text),
            paragraph_id,
            revision,
            previous,
            next,
        }
    }
}

/// `parent`'s child elements named `local` in `namespace`, listed once
/// into `children`, each one's place in the list into `rank`.
fn named<'c>(
    dom: &Dom,
    children: &'c mut HashMap<(NodeId, String, String), Vec<NodeId>>,
    rank: &mut HashMap<NodeId, usize>,
    parent: NodeId,
    namespace: &str,
    local: &str,
) -> &'c [NodeId] {
    children
        .entry((parent, namespace.to_string(), local.to_string()))
        .or_insert_with(|| {
            let list: Vec<NodeId> = dom
                .elements(parent, None)
                .into_iter()
                .filter(|&e| {
                    dom.name(e)
                        .is_some_and(|n| n.local_name() == local && n.namespace_name() == namespace)
                })
                .collect();
            for (i, &e) in list.iter().enumerate() {
                rank.insert(e, i);
            }
            list
        })
}

/// The text of `held[0]`, then the attributes of it and of each element
/// holding it (`held[1..]`), but namespace declarations and the ones an
/// edit renumbers (`rsid…`, `paraId`, `textId`, `id`).
fn signature(dom: &Dom, held: &[NodeId]) -> String {
    let mut out = held.first().map(|&n| dom.value(n)).unwrap_or_default();
    for &n in held {
        for (name, value) in dom.attributes(n) {
            let local = name.local_name();
            if dom.is_namespace_declaration(&name)
                || local.starts_with("rsid")
                || matches!(local, "paraId" | "textId" | "id")
            {
                continue;
            }
            out.push_str(&format!("\u{1f}{local}={value}"));
        }
    }
    out
}

/// `message` with each `[N]` element index dropped and each paragraph or
/// cell id ([`is_paragraph_id`]) spelled `p#`, so an inherited finding keeps
/// its identity when an edit renumbers what it names in passing.
fn normalised(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(start) = rest.find(|c: char| c.is_ascii_alphanumeric() || c == '[') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        if let Some(tail) = rest.strip_prefix('[') {
            let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 0 && tail[digits..].starts_with(']') {
                rest = &tail[digits + 1..];
            } else {
                out.push('[');
                rest = tail;
            }
            continue;
        }
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.'))
            .unwrap_or(rest.len());
        // A sentence's full stop is not part of the id before it.
        let token = rest[..end].trim_end_matches('.');
        if is_paragraph_id(token) {
            out.push_str("p#");
        } else {
            out.push_str(token);
        }
        rest = &rest[token.len()..];
    }
    out.push_str(rest);
    out
}

/// True for a paragraph or cell id `-p` takes (`p12`, `header1.p2`,
/// `t0.row1.cell2`, `t0.row1.cell2.p1`): a position, which an edit
/// renumbers. A story's bare name (`header1`) is no position, and a word
/// that parses as one is not taken for an id.
fn is_paragraph_id(token: &str) -> bool {
    let last = token.rsplit('.').next().unwrap_or(token);
    let numbered = |prefix: &str| {
        last.strip_prefix(prefix)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    };
    (numbered("p") || numbered("cell"))
        && matches!(
            crate::ids::parse_location(token),
            Ok(crate::ids::Location::Paragraph(_))
        )
}

/// A document's findings against those of the document it was made from
/// ([`baseline`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Baselined {
    /// The document's findings the baseline does not have: the edit's.
    pub new: Vec<Finding>,
    /// The document's findings the baseline already had.
    pub baseline: Vec<Finding>,
    /// The baseline's findings the document no longer has.
    pub resolved: Vec<Finding>,
}

/// Splits `new` (the findings of an edited document) by those of `base`
/// (the document before the edits), matched by [`Finding::identity`]: each
/// baseline finding vouches for one finding of `new` with its identity, so
/// a second finding with the same identity is new, and a finding that
/// replaced another is new while the one it replaced is resolved, whatever
/// the counts. Each list keeps its input's order.
pub fn baseline(new: Vec<Finding>, base: Vec<Finding>) -> Baselined {
    let new_keys: Vec<FindingKey> = new.iter().map(Finding::identity).collect();
    let base_keys: Vec<FindingKey> = base.iter().map(Finding::identity).collect();
    baseline_keyed(new, &new_keys, base, &base_keys)
}

/// [`baseline`] with each finding's key given, in step with its list:
/// [`identities`] reads them in the documents themselves. Two findings
/// match when their keys agree but for [`Place`] and they share a place,
/// tried strongest first: all of it, then the change they sit in, their
/// paragraph's id, their text with its neighbours, their text alone, and
/// last their neighbours (their paragraph rewritten). Each pass pairs the
/// findings left, in order, each with the earliest the baseline has left.
#[must_use]
pub fn baseline_keyed(
    new: Vec<Finding>,
    new_keys: &[FindingKey],
    base: Vec<Finding>,
    base_keys: &[FindingKey],
) -> Baselined {
    let mut paired = vec![false; new.len()];
    let mut matched = vec![false; base.len()];
    for pass in 0..PASSES {
        let mut open: HashMap<_, Vec<usize>> = HashMap::new();
        for (i, key) in base_keys.iter().enumerate().rev() {
            if !matched[i]
                && let Some(tier) = tier(pass, key)
            {
                open.entry(tier).or_default().push(i);
            }
        }
        for (j, key) in new_keys.iter().enumerate() {
            if !paired[j]
                && let Some(tier) = tier(pass, key)
                && let Some(i) = open.get_mut(&tier).and_then(Vec::pop)
            {
                paired[j] = true;
                matched[i] = true;
            }
        }
    }
    let mut out = Baselined::default();
    for (f, paired) in new.into_iter().zip(paired) {
        if paired {
            out.baseline.push(f);
        } else {
            out.new.push(f);
        }
    }
    out.resolved = base
        .into_iter()
        .zip(matched)
        .filter_map(|(f, matched)| (!matched).then_some(f))
        .collect();
    out
}

/// How many passes [`baseline_keyed`] makes ([`tier`]).
const PASSES: usize = 6;

/// What pass `pass` of [`baseline_keyed`] matches `key` by: the key but its
/// [`Place`], and the parts of the place the pass reads; `None` when `key`
/// lacks what the pass needs.
fn tier(pass: usize, key: &FindingKey) -> Option<([&str; 5], [Option<&str>; 5])> {
    let place = &key.place;
    let text = place.text.as_deref();
    let id = place.paragraph_id.as_deref();
    let change = place.revision.as_deref();
    let (previous, next) = (place.previous.as_deref(), place.next.as_deref());
    let read = match pass {
        // Nothing around it changed.
        0 => [text, id, change, previous, next],
        // The change it sits in.
        1 if change.is_some() => [None, None, change, None, None],
        // Its paragraph, by id.
        2 if id.is_some() => [None, id, None, None, None],
        // Its text, between the same neighbours.
        3 if text.is_some() => [text, None, None, previous, next],
        // Its text.
        4 if text.is_some() => [text, None, None, None, None],
        // Between the same neighbours, not both empty: its paragraph
        // rewritten.
        5 if previous.is_some() && next.is_some() && (previous != Some("") || next != Some("")) => {
            [None, None, None, previous, next]
        }
        _ => return None,
    };
    let kind = [
        key.code.as_str(),
        key.set.as_str(),
        key.part.as_str(),
        key.location.as_str(),
        key.message.as_str(),
    ];
    Some((kind, read))
}

/// [`baseline`]'s split as `validate --baseline` prints it: `new: none` or
/// `new: N` and the new findings ([`findings_text`]'s lines), then
/// `already in the input: N (not yours)` (or `none`), then, when the edits
/// fixed some, `resolved: N (in the input, not in this file)` and those.
pub fn baselined_text(split: &Baselined) -> String {
    let count = |n: usize| {
        if n == 0 {
            "none".to_string()
        } else {
            n.to_string()
        }
    };
    let mut out = format!("new: {}\n", count(split.new.len()));
    for f in &split.new {
        out.push_str(&finding_line(f));
    }
    out.push_str(&format!(
        "already in the input: {}{}\n",
        count(split.baseline.len()),
        if split.baseline.is_empty() {
            ""
        } else {
            " (not yours)"
        }
    ));
    if !split.resolved.is_empty() {
        out.push_str(&format!(
            "resolved: {} (in the input, not in this file)\n",
            split.resolved.len()
        ));
        for f in &split.resolved {
            out.push_str(&finding_line(f));
        }
    }
    out
}

/// [`baseline`]'s split as JSON Lines: each finding of the document, new
/// ones first, as its JSON object with `"status": "new"` or `"baseline"`,
/// then one `{"resolved": [...]}` line with the baseline's findings the
/// document no longer has (an empty array when there are none).
///
/// # Errors
///
/// A finding that does not serialise.
pub fn baselined_json(split: &Baselined) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct Marked<'a> {
        #[serde(flatten)]
        finding: &'a Finding,
        status: &'static str,
    }
    let mut out = String::new();
    for (findings, status) in [(&split.new, "new"), (&split.baseline, "baseline")] {
        for finding in findings {
            out.push_str(&serde_json::to_string(&Marked { finding, status })?);
            out.push('\n');
        }
    }
    out.push_str(&serde_json::to_string(
        &serde_json::json!({ "resolved": split.resolved }),
    )?);
    out.push('\n');
    Ok(out)
}

/// The one stderr line `read`, `changes` and `comments` give an input with
/// validity findings: `input has 2 validity warnings; jubarte validate FILE
/// lists them` (an `error` counts as an error, `info` not at all), or
/// `None` when there is nothing to say. `file` is the path as given, quoted
/// for a shell when it needs to be ([`crate::util::shell_word`]).
pub fn input_notice(findings: &[Finding], file: &str) -> Option<String> {
    input_notice_of(&input_validity(findings, file))
}

/// [`input_notice`] from counts already taken, for a read that also
/// reports them in its JSON (`read -o FILE.json`).
pub fn input_notice_of(validity: &InputValidity) -> Option<String> {
    let InputValidity {
        errors,
        warnings,
        next,
    } = validity;
    let (errors, warnings) = (*errors, *warnings);
    let next = next.as_deref()?;
    let spell = |n: usize, noun: &str| format!("{n} {noun}{}", if n == 1 { "" } else { "s" });
    let what = match (errors, warnings) {
        (0, 0) => return None,
        (0, w) => spell(w, "validity warning"),
        (e, 0) => spell(e, "validity error"),
        (e, w) => format!("{} and {}", spell(e, "validity error"), spell(w, "warning")),
    };
    let them = if errors + warnings == 1 { "it" } else { "them" };
    Some(format!("input has {what}; {next} lists {them}"))
}

/// The input's own validity findings as the JSON reads report them
/// (#1309): `read --json` in its snapshot's `summary`, `changes --json` and
/// `comments --json` on their last line ([`input_validity_line`]), where
/// [`input_notice`]'s stderr line would break `--json 2>&1 | jq`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputValidity {
    /// Findings of severity `error`.
    pub errors: usize,
    /// Findings of severity `warning`; `info` is not counted.
    pub warnings: usize,
    /// The command that lists them (`jubarte validate FILE`), when there
    /// are any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// What [`InputValidity`] says of `findings`; `file` is the path as given,
/// quoted for a shell when it needs to be ([`crate::util::shell_word`]).
#[must_use]
pub fn input_validity(findings: &[Finding], file: &str) -> InputValidity {
    let count = |severity: &str| findings.iter().filter(|f| f.severity == severity).count();
    let (errors, warnings) = (count("error"), count("warning"));
    InputValidity {
        errors,
        warnings,
        next: (errors + warnings > 0)
            .then(|| format!("jubarte validate {}", crate::util::shell_word(file))),
    }
}

/// [`input_validity`] of the package `docx` read from `file`, as the CLI
/// and the bindings report it: a package that cannot be validated has no
/// findings to count.
#[must_use]
pub fn input_validity_of(docx: &[u8], file: &str) -> InputValidity {
    input_validity(&validate(docx).unwrap_or_default(), file)
}

/// The last line `changes --json` and `comments --json` print:
/// `{"input_validity":{"errors":0,"warnings":2,"next":"jubarte validate FILE"}}`,
/// its keys in that order.
///
/// # Errors
/// When the validity does not serialize.
pub fn input_validity_line(validity: &InputValidity) -> Result<String, serde_json::Error> {
    Ok(format!(
        r#"{{"input_validity":{}}}"#,
        serde_json::to_string(validity)?
    ))
}

impl From<crate::audit::AuditFinding> for Finding {
    /// The audit's finding in the shared shape: its part is the story's
    /// part for a paragraph id (`p3` → `word/document.xml`), else the
    /// location itself (a part name); it is never Word-fatal or repairable.
    fn from(f: crate::audit::AuditFinding) -> Self {
        let part = if crate::ids::paragraph_place(&f.location).is_some() {
            story_part(&f.location)
        } else {
            f.location.clone()
        };
        let r#ref = f.location.clone();
        Finding {
            code: f.code,
            set: f.rule_set,
            severity: f.severity,
            part,
            location: f.location,
            r#ref,
            message: f.message,
            word_fatal: false,
            repairable: false,
        }
    }
}

/// Findings sorted by part, location, code and message (the same order every
/// run); an empty vector is a pass.
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
    // A total order: findings at one location (comment metadata gathered
    // from hash sets) would otherwise print in a different order each run.
    out.sort_by(|a, b| {
        (&a.part, &a.location, &a.code, &a.message).cmp(&(
            &b.part,
            &b.location,
            &b.code,
            &b.message,
        ))
    });
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
    check_runs_inside_paragraphs(pkg, &mut out);
    check_style_names(pkg, &mut out);
    check_nested_text_boxes(pkg, &mut out);
    check_nested_ppr(pkg, &mut out);
    check_move_range_counts(pkg, &mut out);
    check_data_bindings_have_stores(pkg, &mut out);
    out
}

/// Word fills a bound content control from the custom XML store its
/// `w:storeItemID` names, and shows the control empty when the main part
/// reaches no such store (#1427). Word opens the file: a lead. Word's own
/// stores (the document properties) live in `docProps`.
fn check_data_bindings_have_stores(pkg: &PartFs, out: &mut Vec<Finding>) {
    use crate::markdown::data_stores::{built_in, stores};
    let Some(main) = main_part(pkg) else {
        return;
    };
    let mut held: Option<Vec<String>> = None;
    for part in xml_parts(pkg) {
        if part.starts_with("customXml/")
            || !pkg
                .part_bytes(&part)
                .is_some_and(|bytes| bytes.windows(11).any(|w| w == b"dataBinding"))
        {
            continue;
        }
        let Some((dom, root)) = parse_part(pkg, &part) else {
            continue;
        };
        for binding in dom.descendants(root, Some(&W::name("dataBinding"))) {
            let Some(store) = dom.attribute(binding, &W::name("storeItemID")) else {
                continue;
            };
            if built_in(store)
                || held
                    .get_or_insert_with(|| stores(pkg, &main))
                    .contains(&store.to_ascii_uppercase())
            {
                continue;
            }
            out.push(Finding::new(
                "DATA_BINDING_WITHOUT_STORE",
                &part,
                element_path(&dom, binding),
                format!("{store}: no custom XML store the main part reaches holds it; Word shows the bound control empty (#1427)"),
            ));
        }
    }
}

/// Word opens unequal per-name move ranges cleanly, but Reject All can
/// restore the destination over only one source range and duplicate text.
fn check_move_range_counts(pkg: &PartFs, out: &mut Vec<Finding>) {
    for part in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &part) else {
            continue;
        };
        let mut moves = std::collections::BTreeMap::new();
        for (side, source) in [("moveFromRangeStart", true), ("moveToRangeStart", false)] {
            for start in dom.descendants(root, Some(&W::name(side))) {
                let Some(name) = dom
                    .attribute(start, &W::name("name"))
                    .filter(|n| !n.is_empty())
                else {
                    continue;
                };
                let counts = moves
                    .entry(name.to_string())
                    .or_insert((0_usize, 0_usize, start));
                if source {
                    counts.0 += 1;
                } else {
                    counts.1 += 1;
                }
            }
        }
        for (name, (source, destination, start)) in moves {
            if source != destination {
                out.push(Finding::new(
                    "MOVE_RANGE_COUNT_MISMATCH",
                    &part,
                    element_path(&dom, start),
                    format!("{name}: {source} source ranges, {destination} destination ranges; Word opens it but Reject All can duplicate moved text (#1214)"),
                ));
            }
        }
    }
}

/// A `w:pPr` inside a `w:pPr`: `CT_PPr`'s children are `CT_PPrBase`'s, then
/// `w:rPr`, `w:sectPr` and `w:pPrChange`, so it takes none of their own (the
/// previous properties of a `w:pPrChange` sit one level down, under the
/// change, and are Word's own shape). `tools/validate-docx` refuses the part
/// with `Sch_InvalidElementContentExpectingComplex`, so it is OOXML-invalid
/// whatever Word does with it; no Word probe is recorded, so it is a lead
/// and not Word-fatal (#1595 review).
fn check_nested_ppr(pkg: &PartFs, out: &mut Vec<Finding>) {
    for part in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &part) else {
            continue;
        };
        for ppr in dom.descendants(root, Some(&W::name("pPr"))) {
            for inner in dom.elements(ppr, Some(&W::name("pPr"))) {
                out.push(Finding::new(
                    "NESTED_PPR",
                    &part,
                    element_path(&dom, inner),
                    format!("a w:pPr sits inside another ({})", element_path(&dom, ppr)),
                ));
            }
        }
    }
}

/// Word crashes or refuses a box in another box's readable content, even
/// though OpenXmlValidator passes it. The 2026-10-18 A–H probes (#1132)
/// establish that a nest only in an ignored MC Fallback is harmless.
fn check_nested_text_boxes(pkg: &PartFs, out: &mut Vec<Finding>) {
    let dml_box = XName::get(
        "txbx",
        "http://schemas.microsoft.com/office/word/2010/wordprocessingShape",
    );
    let vml_box = VML::name("textbox");
    for part in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &part) else {
            continue;
        };
        for node in dom.descendants(root, None) {
            if !dom
                .name(node)
                .is_some_and(|name| name == dml_box || name == vml_box)
            {
                continue;
            }
            let ancestors = dom.ancestors(node, None);
            if ancestors
                .iter()
                .any(|&a| dom.name(a).is_some_and(|n| n == MC::name("Fallback")))
            {
                continue;
            }
            if let Some(&outer) = ancestors
                .iter()
                .find(|&&a| dom.name(a).is_some_and(|n| n == W::name("txbxContent")))
            {
                out.push(Finding::new(
                    "NESTED_TEXT_BOX",
                    &part,
                    element_path(&dom, node),
                    format!(
                        "text box {} holds another text box ({}); Word cannot open this document",
                        text_box_name(&dom, outer),
                        text_box_name(&dom, node)
                    ),
                ));
            }
        }
    }
}

/// Prefer the shape's own name, with its stable element path as a fallback.
fn text_box_name(dom: &Dom, node: NodeId) -> String {
    for ancestor in dom.ancestors(node, None) {
        if dom.name(ancestor).is_some_and(|n| n == W::name("drawing")) {
            if let Some(name) = dom
                .descendants(ancestor, Some(&WP::name("docPr")))
                .into_iter()
                .find_map(|id| dom.attribute(id, &XName::get("name", "")))
                .filter(|name| !name.is_empty())
            {
                return name.to_string();
            }
        } else if dom.name(ancestor).is_some_and(|n| n == VML::name("shape"))
            && let Some(name) = dom.attribute(ancestor, &XName::get("id", ""))
            && !name.is_empty()
        {
            return name.to_string();
        }
    }
    element_path(dom, node)
}

/// A style's `w:name` holds no comma: the schema's pattern is `[^,]*`
/// (Word reads a comma there as the start of an alias list), while its
/// `w:styleId` may hold one. The OOXML validator refuses the part; Word
/// opens it (`List,Para`, probed 2026-10-15), so it is no Word-fatal.
fn check_style_names(pkg: &PartFs, out: &mut Vec<Finding>) {
    for part in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &part) else {
            continue;
        };
        if dom.name(root).is_none_or(|n| n.local_name() != "styles") {
            continue;
        }
        for name in dom.descendants(root, Some(&W::name("name"))) {
            let in_style = dom
                .parent(name)
                .and_then(|p| dom.name(p))
                .is_some_and(|p| p.local_name() == "style");
            if let Some(value) = dom.attribute(name, &W::val())
                && in_style
                && value.contains(',')
            {
                out.push(Finding::new(
                    "STYLE_NAME_COMMA",
                    &part,
                    element_path(&dom, name),
                    format!("style name '{value}' holds a comma, which w:name forbids"),
                ));
            }
        }
    }
}

/// A run outside any paragraph: Word loads the document empty. Probed
/// 2026-10-10 with a comment's reference run straight in `w:body`, in a
/// `w:tc`, and in a body-level `w:ins`, `w:del`, `w:moveTo`, block
/// `w:sdtContent` or block `w:customXml`; and with a run straight under a
/// part's root (`w:document`, `w:comments`, `w:footnotes`) or a footnote.
fn check_runs_inside_paragraphs(pkg: &PartFs, out: &mut Vec<Finding>) {
    // Story and cell containers: a run reaching one before any paragraph is
    // outside a paragraph, whatever wraps it on the way.
    const BLOCKS: [&str; 9] = [
        "body",
        "tc",
        "hdr",
        "ftr",
        "footnote",
        "endnote",
        "comment",
        "txbxContent",
        "docPartBody",
    ];
    for name in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &name) else {
            continue;
        };
        for run in dom.descendants(root, Some(&W::r())) {
            let mut up = dom.parent(run);
            let outside = loop {
                let Some(node) = up else { break true };
                if dom.name_is(node, &W::p()) {
                    break false;
                }
                if BLOCKS
                    .iter()
                    .any(|block| dom.name_is(node, &W::name(block)))
                {
                    break true;
                }
                up = dom.parent(node);
            };
            if outside {
                out.push(Finding::new(
                    "RUN_OUTSIDE_PARAGRAPH",
                    &name,
                    element_path(&dom, run),
                    format!("a run outside any paragraph in '{name}'"),
                ));
            }
        }
    }
}

/// `w:body[0]/w:p[3]/w:r[2]`: conventional prefix and local name with the
/// index among same-named siblings, root excluded.
pub(crate) fn element_path(dom: &Dom, node: NodeId) -> String {
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

/// The package's XML parts: by name, or by content type (a main document
/// part may be `word/main.dat`).
fn xml_parts(pkg: &PartFs) -> Vec<String> {
    pkg.parts()
        .into_iter()
        .filter(|name| {
            crate::opc::ends_with_ignore_case(name, ".xml")
                || pkg
                    .content_type_for(name)
                    .is_some_and(|ct| crate::opc::is_xml_content_type(&ct))
        })
        .collect()
}

/// The XML parts that hold WordprocessingML revisions: every XML part but
/// custom XML data, which holds the application's own XML whatever its name
/// ([`crate::opc::custom_xml_data`]: a part only `customXml` relationships
/// reach, the package's own included). OpenXmlValidator checks no revision
/// id there.
fn revision_parts(pkg: &PartFs) -> Vec<String> {
    let parts = pkg.parts();
    let data =
        crate::opc::custom_xml_data(std::iter::once(("", pkg.package_relationships())).chain(
            parts.iter().filter_map(|source| {
                pkg.read_rels_for(source)
                    .map(|rels| (source.as_str(), rels))
            }),
        ));
    xml_parts(pkg)
        .into_iter()
        .filter(|name| !data.iter().any(|item| item.eq_ignore_ascii_case(name)))
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
        if pkg.content_type_for(&name).is_none()
            && name != "[Content_Types].xml"
            && !name.starts_with(crate::debug::OFFICE_TRASH)
        {
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
            let mut reader = quick_xml::NsReader::from_str(&xml);
            reader.config_mut().trim_text(false);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(Event::Start(tag) | Event::Empty(tag)) => {
                        let mut expanded = HashSet::new();
                        for attr in tag.attributes().with_checks(false).flatten() {
                            if attr.key.as_ref() == "xmlns"
                                || attr.key.as_ref().starts_with("xmlns:")
                            {
                                continue;
                            }
                            let (namespace, local) = reader.resolver().resolve_attribute(attr.key);
                            let namespace = match namespace {
                                quick_xml::name::ResolveResult::Bound(ns) => {
                                    quick_xml::escape::unescape(ns.as_ref()).map_or_else(
                                        |_| ns.as_ref().to_string(),
                                        |uri| uri.into_owned(),
                                    )
                                }
                                quick_xml::name::ResolveResult::Unbound => String::new(),
                                quick_xml::name::ResolveResult::Unknown(_) => continue,
                            };
                            if !expanded.insert((namespace, local.as_ref().to_string())) {
                                let element = tag.name().as_ref().to_string();
                                out.push(Finding::new(
                                    "DUPLICATE_EXPANDED_ATTRIBUTE",
                                    &name,
                                    &element,
                                    format!("part '{name}', element <{element}>: attribute {} repeats an expanded name", attr.key.as_ref()),
                                ));
                            }
                        }
                    }
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
    for name in revision_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &name) else {
            continue;
        };
        let mut rev_ids: HashSet<String> = HashSet::new();
        let mut docpr_ids: HashSet<String> = HashSet::new();
        collect_ids(&dom, root, &mut rev_ids, &mut docpr_ids, out, &name);
    }
}

/// Revision elements whose `w:id` must be unique among elements of the
/// same record type within a part (the SDK's `Sem_UniqueAttributeValue`).
/// Different kinds can share an id: Word pairs a style's `pPrChange` and `rPrChange`.
/// A range end shares its start's id, but two ends of one kind may not
/// share one; OpenXmlValidator lets two `customXmlMoveToRangeEnd` do so.
/// Comment and bookmark ids have other scopes.
pub(crate) fn has_unique_revision_id(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && (name.local_name().ends_with("PrChange")
            || matches!(
                name.local_name(),
                "ins"
                    | "del"
                    | "moveFrom"
                    | "moveTo"
                    | "moveFromRangeStart"
                    | "moveFromRangeEnd"
                    | "moveToRangeStart"
                    | "moveToRangeEnd"
                    | "customXmlInsRangeStart"
                    | "customXmlInsRangeEnd"
                    | "customXmlDelRangeStart"
                    | "customXmlDelRangeEnd"
                    | "customXmlMoveFromRangeStart"
                    | "customXmlMoveFromRangeEnd"
                    | "customXmlMoveToRangeStart"
                    | "tblGridChange"
                    | "tblPrExChange"
                    | "numberingChange"
                    | "cellIns"
                    | "cellDel"
                    | "cellMerge"
            ))
}

/// The SDK distinguishes run revisions from row/paragraph-mark revisions,
/// even though both use `ins`, `del`, `moveFrom` or `moveTo`. Row and
/// paragraph marks share their own scope; the immediate parent alone is
/// therefore not the scope. Keep detection and repair on the same key.
/// The SDK compares some kinds' ids case-blind (`ABC` repeats `abc`).
fn revision_id_key(dom: &Dom, node: NodeId, name: &XName, id: &str) -> String {
    let local = name.local_name();
    let property_mark = matches!(local, "ins" | "del" | "moveFrom" | "moveTo")
        && dom.parent(node).is_some_and(|parent| {
            dom.name(parent).is_some_and(|parent| {
                parent.namespace_name() == W::URI && matches!(parent.local_name(), "rPr" | "trPr")
            })
        });
    if id_case_blind(local) {
        format!("{local}:{property_mark}:{}", id.to_lowercase())
    } else {
        format!("{local}:{property_mark}:{id}")
    }
}

/// Whether the SDK compares a kind's ids case-blind (`ABC` repeats `abc`).
fn id_case_blind(local: &str) -> bool {
    matches!(
        local,
        "customXmlDelRangeStart" | "customXmlDelRangeEnd" | "moveToRangeStart"
    )
}

/// Two ids compared as [`revision_id_key`] compares them, without building
/// either key.
fn same_revision_id(case_blind: bool, a: &str, b: &str) -> bool {
    if !case_blind {
        a == b
    } else if a.is_ascii() && b.is_ascii() {
        a.eq_ignore_ascii_case(b)
    } else {
        a.to_lowercase() == b.to_lowercase()
    }
}

/// The end that closes a range start of this kind.
fn range_end_of(start: &str) -> Option<&'static str> {
    Some(match start {
        "moveFromRangeStart" => "moveFromRangeEnd",
        "moveToRangeStart" => "moveToRangeEnd",
        "customXmlInsRangeStart" => "customXmlInsRangeEnd",
        "customXmlDelRangeStart" => "customXmlDelRangeEnd",
        "customXmlMoveFromRangeStart" => "customXmlMoveFromRangeEnd",
        "customXmlMoveToRangeStart" => "customXmlMoveToRangeEnd",
        _ => return None,
    })
}

/// The range start a range end of this kind closes.
fn range_start_of(end: &str) -> Option<&'static str> {
    Some(match end {
        "moveFromRangeEnd" => "moveFromRangeStart",
        "moveToRangeEnd" => "moveToRangeStart",
        "customXmlInsRangeEnd" => "customXmlInsRangeStart",
        "customXmlDelRangeEnd" => "customXmlDelRangeStart",
        "customXmlMoveFromRangeEnd" => "customXmlMoveFromRangeStart",
        "customXmlMoveToRangeEnd" => "customXmlMoveToRangeStart",
        _ => return None,
    })
}

/// How often each revision id occurs, by the scope `DUPLICATE_REVISION_ID`
/// checks: part, then record type and id ([`revision_id_key`]). A Strict
/// package is read as the Transitional one an edit writes.
pub(crate) fn revision_id_occurrences(
    docx: &[u8],
) -> Result<HashMap<(String, String), usize>, ValidateError> {
    let transitional = crate::strict_translation::strict_to_transitional_docx(docx)
        .map_err(|e| ValidateError::Package(e.to_string()))?;
    let pkg = PartFs::open(&transitional)?;
    let mut out = HashMap::new();
    for part in revision_parts(&pkg) {
        let Some((dom, root)) = parse_part(&pkg, &part) else {
            continue;
        };
        let mut counts: HashMap<String, usize> = HashMap::new();
        let id_name = W::id();
        for e in dom.descendants(root, None) {
            if let Some(name) = dom.name(e)
                && has_unique_revision_id(&name)
                && let Some(id) = dom.attribute(e, &id_name)
            {
                *counts
                    .entry(revision_id_key(&dom, e, &name, id))
                    .or_default() += 1;
            }
        }
        out.extend(
            counts
                .into_iter()
                .map(|(key, count)| ((part.clone(), key), count)),
        );
    }
    Ok(out)
}

fn collect_ids(
    dom: &Dom,
    root: NodeId,
    rev_ids: &mut HashSet<String>,
    docpr_ids: &mut HashSet<String>,
    out: &mut Vec<Finding>,
    part: &str,
) {
    // Repeated range starts not yet paired with a repeated end, by key.
    let mut repeated_starts: HashMap<String, usize> = HashMap::new();
    let id_name = W::id();
    for e in dom.descendants(root, None) {
        let Some(name) = dom.name(e) else {
            continue;
        };
        let local = name.local_name();
        // Revision ids are scoped by record type, not by the union of
        // ins/del/move/property records. Comment ids have a separate scope.
        if has_unique_revision_id(&name)
            && let Some(id) = dom.attribute(e, &id_name)
        {
            let key = revision_id_key(dom, e, &name, id);
            if rev_ids.insert(key.clone()) {
                continue;
            }
            let mut finding = Finding::new(
                "DUPLICATE_REVISION_ID",
                part,
                element_path(dom, e),
                format!("duplicate w:id '{id}' on revision markup in '{part}' ({local}:{id})"),
            );
            // Repair renames an end only with a repeated start before it
            // ([`renumber_duplicate_revision_ids`]); an end with none stays
            // as it was (CodeRabbit on #1666).
            if local.ends_with("RangeStart") {
                *repeated_starts.entry(key).or_default() += 1;
            } else if let Some(start) = range_start_of(local) {
                let start = W::name(start);
                match repeated_starts.get_mut(&revision_id_key(dom, e, &start, id)) {
                    Some(n) if *n > 0 => *n -= 1,
                    _ => finding.repairable = false,
                }
            }
            out.push(finding);
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
/// The nearest deletion owns the text: an insertion inside the move source
/// does not shelter `w:delText` (Word refuses it), a `w:del` inside it does
/// (Word opens it; probes M1/M5, 2026-10-18).
fn check_del_text_under_del(pkg: &PartFs, out: &mut Vec<Finding>) {
    for name in xml_parts(pkg) {
        let Some((dom, root)) = parse_part(pkg, &name) else {
            continue;
        };
        for (live, deleted, code) in [
            ("t", "delText", "TEXT_INSIDE_DELETION"),
            ("instrText", "delInstrText", "INSTR_TEXT_INSIDE_DELETION"),
        ] {
            for t in dom.descendants(root, Some(&W::name(live))) {
                if under_deletion(&dom, t) {
                    out.push(Finding::new(
                        code,
                        &name,
                        element_path(&dom, t),
                        format!("w:{live} under w:del in '{name}' (must be w:{deleted})"),
                    ));
                }
            }
        }
        let move_from = W::name("moveFrom");
        for mf in dom.descendants(root, Some(&move_from)) {
            for dt in dom.descendants(mf, Some(&W::name("delText"))) {
                if ancestor_has(&dom, dt, mf, "del") {
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

/// Any enclosing deletion counts, even through an insertion, but only
/// within the run's own story (#1134, Word probe 2026-10-17).
fn under_deletion(dom: &Dom, node: NodeId) -> bool {
    dom.ancestors(node, None)
        .into_iter()
        .take_while(|&a| dom.name(a).is_none_or(|n| n.local_name() != "txbxContent"))
        .any(|a| dom.name(a).is_some_and(|n| n == W::del()))
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

/// The comments part as Word finds it: the first relationship of the
/// comments type whose target part exists (`word/comments.xml` is only the
/// conventional name — Word follows the relationship, so a differently
/// named part is the comments part), else the conventional name when it
/// exists unlinked. A relationship whose target is missing does not stop
/// the walk: a later live relationship still names the part.
fn comments_part(pkg: &PartFs, main: &str) -> Option<String> {
    if let Some(name) = pkg
        .read_rels_for(main)
        .into_iter()
        .flat_map(|relationships| &relationships.items)
        .filter(|r| {
            r.rel_type == COMMENT_FAMILY[0].2 && r.target_mode.as_deref() != Some("External")
        })
        .map(|r| {
            pkg.resolve_rel_target(main, &r.target)
                .trim_start_matches('/')
                .to_string()
        })
        .find(|name| pkg.part_bytes(name).is_some())
    {
        return Some(name);
    }
    pkg.part_bytes(COMMENTS_PART)
        .map(|_| COMMENTS_PART.to_string())
}

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

    let Some(comments_name) = comments_part(pkg, &main) else {
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
                    "COMMENT_METADATA_MISMATCH",
                    part,
                    format!("'{part}' exists without word/comments.xml"),
                );
            }
        }
        return;
    };
    let Some((comments_dom, comments_root)) = parse_part(pkg, &comments_name) else {
        return;
    };

    let definitions = comment_id_counts(&comments_dom, comments_root, "comment");
    for (kind, counts, part) in [
        ("comment definition", &definitions, comments_name.as_str()),
        ("commentRangeStart", &starts, main.as_str()),
        ("commentRangeEnd", &ends, main.as_str()),
        ("commentReference", &references, main.as_str()),
    ] {
        // Word hangs on two references for one comment; it opens repeated
        // range markers or definitions, showing the first definition.
        let code = if kind == "commentReference" {
            "COMMENT_PARTS_INCONSISTENT"
        } else {
            "COMMENT_ID_REPEATED"
        };
        for (id, count) in counts {
            if *count != 1 {
                comment_finding(
                    out,
                    code,
                    part,
                    format!("{kind} id '{id}' occurs {count} times"),
                );
            }
        }
    }
    // Word shows a comment from its reference in a paragraph and from
    // nothing else; without one the file opens and the comment is not shown
    // (2026-10-10), so it is a lead. Missing range markers are no finding:
    // the reference alone shows a point comment.
    for id in definitions
        .keys()
        .filter(|id| !references.contains_key(*id))
    {
        comment_finding(
            out,
            "COMMENT_UNANCHORED",
            &comments_name,
            format!(
                "comment definition id '{id}' has no commentReference in the document: Word opens it and shows no comment"
            ),
        );
    }
    // A reference to no definition is refused; a range marker to none is
    // opened as it is.
    for (kind, counts, code) in [
        ("commentRangeStart", &starts, "COMMENT_RANGE_ORPHAN"),
        ("commentRangeEnd", &ends, "COMMENT_RANGE_ORPHAN"),
        ("commentReference", &references, "COMMENT_ANCHOR_ORPHAN"),
    ] {
        for id in counts.keys().filter(|id| !definitions.contains_key(*id)) {
            comment_finding(
                out,
                code,
                &main,
                format!("{kind} id '{id}' has no entry in {comments_name}"),
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
            check_hex_id("paraId", para_id, true, &comments_name, out);
            if !all_para_ids.insert(key.clone()) {
                comment_finding(
                    out,
                    "COMMENT_METADATA_MISMATCH",
                    &comments_name,
                    format!("duplicate comment paraId '{para_id}' in {comments_name}"),
                );
            }
            comment_para_ids.push(key);
        }
        if aux_present && comment_para_ids.is_empty() {
            comment_finding(
                out,
                "COMMENT_METADATA_MISMATCH",
                &comments_name,
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
                    "COMMENT_METADATA_MISMATCH",
                    "word/commentsExtended.xml",
                    format!("commentsExtended paraIdParent '{parent}' does not resolve"),
                );
            }
        }
        check_parent_cycles(&extended.parents, out);
    }
}

/// How often each id occurs on `local` elements. An `mc:Fallback` copy is
/// not counted: Word reads one branch of a dual-saved shape, so markers
/// repeated in its fallback are no second anchor (probe k05, 2026-10-10).
fn comment_id_counts(dom: &Dom, root: NodeId, local: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for element in dom.descendants(root, Some(&W::name(local))) {
        if !dom
            .ancestors(element, Some(&MC::name("Fallback")))
            .is_empty()
        {
            continue;
        }
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
                "COMMENT_METADATA_MISMATCH",
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
    // Word reads up to eight hex digits: seven open (k06), nine are refused
    // (k07, 2026-10-10).
    let parsed =
        if (1..=8).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            u32::from_str_radix(value, 16).ok()
        } else {
            None
        };
    let Some(parsed) = parsed else {
        comment_finding(
            out,
            "COMMENT_PARTS_INCONSISTENT",
            part,
            format!("{label} '{value}' is not a hexadecimal id of up to 8 digits"),
        );
        return;
    };
    if word_para_bound && parsed >= 0x8000_0000 {
        comment_finding(
            out,
            "COMMENT_METADATA_MISMATCH",
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
                "COMMENT_METADATA_MISMATCH",
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
                "COMMENT_METADATA_MISMATCH",
                PART,
                format!("duplicate commentsIds paraId '{para_id}'"),
            );
        }
        let Some(durable_id) = attribute_by_local(&dom, entry, "durableId") else {
            comment_finding(
                out,
                "COMMENT_METADATA_MISMATCH",
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
                "COMMENT_METADATA_MISMATCH",
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
                "COMMENT_METADATA_MISMATCH",
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
                "COMMENT_METADATA_MISMATCH",
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
            "COMMENT_METADATA_MISMATCH",
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
            "COMMENT_METADATA_MISMATCH",
            part,
            format!("{label} is missing '{key}'"),
        );
    }
    for key in actual.difference(expected) {
        comment_finding(
            out,
            "COMMENT_METADATA_MISMATCH",
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
        for (name, value) in dom.attributes_iter(element) {
            if !is_namespace_qname_list(&element_name, name) {
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

/// The namespace `prefix` names at `element`: the nearest `xmlns:prefix`
/// declaration on it or an ancestor.
pub(crate) fn namespace_in_scope<'a>(
    dom: &'a Dom,
    element: NodeId,
    prefix: &str,
) -> Option<&'a str> {
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
        edit_part(&mut pkg, &part, |dom, root| {
            renumber_duplicate_revision_ids(dom, root);
        });
    }
    if has("PARA_ID_OUT_OF_RANGE") {
        renumber_out_of_range_para_ids(&mut pkg);
    }
    for part in parts_with(&["CELL_WITHOUT_PARAGRAPH"]) {
        edit_part(&mut pkg, &part, end_cells_with_a_paragraph);
    }
    if (has("COMMENT_ANCHOR_ORPHAN") || has("COMMENT_RANGE_ORPHAN"))
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
    for (live, deleted) in [("t", "delText"), ("instrText", "delInstrText")] {
        for t in dom.descendants(root, Some(&W::name(live))) {
            if under_deletion(dom, t) {
                renames.push((t, W::name(deleted)));
            }
        }
    }
    for mf in dom.descendants(root, Some(&W::name("moveFrom"))) {
        for dt in dom.descendants(mf, Some(&W::name("delText"))) {
            if !ancestor_has(dom, dt, mf, "del") {
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

/// A second use of a revision id on the same record type is renumbered
/// from the part's highest `w:id` upwards; a range start takes its end
/// along. An end is never renumbered on its own: which start it closes is
/// not known. Accepted cross-kind pairs retain their ids.
pub(crate) fn renumber_duplicate_revision_ids(
    dom: &mut Dom,
    root: NodeId,
) -> Vec<(String, String)> {
    let id_name = W::id();
    let all: Vec<NodeId> = dom.descendants(root, None);
    let mut next: u64 = all
        .iter()
        .filter_map(|&n| dom.attribute(n, &id_name))
        .filter_map(|v| v.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    // The revision records with an id (range ends aside), and how often
    // each key occurs.
    let mut starts = Vec::new();
    let mut count: HashMap<String, usize> = HashMap::new();
    for (i, &n) in all.iter().enumerate() {
        let Some(name) = dom.name(n) else { continue };
        if !has_unique_revision_id(&name) || name.local_name().ends_with("RangeEnd") {
            continue;
        }
        let Some(id) = dom.attribute(n, &id_name).map(str::to_string) else {
            continue;
        };
        let key = revision_id_key(dom, n, &name, &id);
        *count.entry(key.clone()).or_default() += 1;
        starts.push((i, n, name, id, key));
    }
    let mut seen: HashSet<String> = HashSet::new();
    // Range ends a start before has taken.
    let mut taken: HashSet<NodeId> = HashSet::new();
    let mut renames: Vec<(NodeId, String)> = Vec::new();
    let mut revisions = Vec::new();
    for (i, n, name, id, key) in starts {
        if count[&key] < 2 {
            continue;
        }
        // A range start's end shares its id: the first that follows with
        // it, by the rule the duplicate check reads (`ABC` closes `abc`
        // where ids compare case-blind, MiniMax on #1666), and that no
        // start before has taken, so three starts of one id before their
        // three ends pair one to one (codex on #1666).
        let end = range_end_of(name.local_name()).and_then(|end_local| {
            let blind = id_case_blind(end_local);
            all[i + 1..].iter().copied().find(|&e| {
                !taken.contains(&e)
                    && dom.name(e).is_some_and(|x| {
                        x.namespace_name() == W::URI && x.local_name() == end_local
                    })
                    && dom
                        .attribute(e, &id_name)
                        .is_some_and(|own| same_revision_id(blind, own, &id))
            })
        });
        if let Some(end) = end {
            taken.insert(end);
        }
        if seen.insert(key) {
            continue;
        }
        let fresh = next.to_string();
        next += 1;
        revisions.push((id, fresh.clone()));
        renames.push((n, fresh.clone()));
        if let Some(end) = end {
            renames.push((end, fresh));
        }
    }
    for (n, id) in renames {
        dom.set_attribute_value(n, &id_name, Some(&id));
    }
    revisions
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
        .filter(|&tc| !crate::debug::cell_ends_in_paragraph(dom, tc))
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

/// The body's paragraphs followed by every story's (`header1`, ...).
fn all_paragraphs(docx: &[u8]) -> Result<Vec<crate::inspect::Paragraph>, ValidateError> {
    let mut all = crate::inspect::paragraphs(docx)?;
    for story in crate::inspect::stories(docx)? {
        all.extend(story.paragraphs);
    }
    Ok(all)
}

/// The part a paragraph or change id lives in: the body's (`p3`, `r12`)
/// is the main document, any other story's (`header1.p1`, `footnotes.r2`)
/// `word/{story}.xml`.
fn story_part(id: &str) -> String {
    let story = crate::ids::paragraph_place(id)
        .map(|(story, _)| story)
        .or_else(|| crate::ids::change_id(id).map(|c| c.story));
    match story.as_deref().unwrap_or("body") {
        "body" | "" => "word/document.xml".to_string(),
        story => format!("word/{story}.xml"),
    }
}

/// `validate.py --original --author` without an XSD: every visible-text
/// difference between `original` and `edited` must sit in a revision by
/// `author`. Rejecting that author's changes must give back `original`'s
/// text, body and stories alike; any residue is an `UNTRACKED_EDIT` finding
/// at its paragraph id. Foreign revisions already in `original` may remain:
/// only added, altered or removed foreign history is a `FOREIGN_AUTHOR`
/// finding. Paragraphs are aligned by full text (including deleted text), story
/// by story. An untracked paragraph merge or split around another author's
/// revisions reports those revisions as new and gone alongside `UNTRACKED_EDIT`.
/// Within each paragraph or non-paragraph owner, matching author,
/// kind, target, date and move-side groups are compared in document order,
/// ignoring revision ids. Split carriers combine only when they touch with
/// no visible content or still-present revision between them; a boundary
/// explained solely by a missing separator revision is not another change.
/// Missing original permission endpoints are reported as
/// `PERMISSION_RANGE_DROPPED`, including in headers, footers and notes.
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
    check_dropped_permissions(&PartFs::open(original)?, &PartFs::open(edited)?, &mut out);
    let filter = ChangeFilter {
        authors: Some(vec![author.to_string()]),
        ..ChangeFilter::default()
    };
    let reverted = reject_changes(edited, &filter)?;
    check_foreign_revisions(original, &reverted, author, &mut out)?;
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
        return Ok(as_edits(out));
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
    Ok(as_edits(out))
}

/// Order-stable alignment. Sparse matches use Hunt--Szymanski with a linear
/// link budget. Dense matches use Myers with at most 64 edits, then a
/// deterministic forward-only fallback. Memory is O(N + 64²), even for
/// repeated text; unlike a multiset no path can match both sides of a swap.
fn ordered_pairs<T: Eq + std::hash::Hash>(a: &[T], b: &[T]) -> Vec<(usize, usize)> {
    let prefix = a.iter().zip(b).take_while(|(a, b)| a == b).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let mut positions: HashMap<&T, Vec<usize>> = HashMap::new();
    for (j, value) in b.iter().enumerate().take(b.len() - suffix).skip(prefix) {
        positions.entry(value).or_default().push(j);
    }
    let budget = a.len().saturating_add(b.len()).saturating_mul(8);
    let matches = a[prefix..a.len() - suffix]
        .iter()
        .fold(0usize, |count, value| {
            count.saturating_add(positions.get(value).map_or(0, Vec::len))
        });
    if matches > budget {
        let middle_a = &a[prefix..a.len() - suffix];
        let middle_b = &b[prefix..b.len() - suffix];
        let middle = bounded_myers_pairs(middle_a, middle_b).map_or_else(
            || {
                // Earliest available counterpart, with no backtracking. A
                // bounded approximation can add findings, never reorder a
                // match or invent an equality that hides altered text.
                let mut next = prefix;
                let mut pairs = Vec::new();
                for (i, value) in a.iter().enumerate().take(a.len() - suffix).skip(prefix) {
                    if let Some(indices) = positions.get(value) {
                        let at = indices.partition_point(|&j| j < next);
                        if let Some(&j) = indices.get(at) {
                            pairs.push((i, j));
                            next = j + 1;
                        }
                    }
                }
                pairs
            },
            |pairs| {
                pairs
                    .into_iter()
                    .map(|(i, j)| (i + prefix, j + prefix))
                    .collect()
            },
        );
        let mut pairs: Vec<_> = (0..prefix).map(|i| (i, i)).collect();
        pairs.extend(middle);
        pairs.extend((0..suffix).map(|i| (a.len() - suffix + i, b.len() - suffix + i)));
        return pairs;
    }
    let mut links: Vec<(usize, usize, Option<usize>)> = Vec::new();
    let mut tails: Vec<usize> = Vec::new();
    for (i, value) in a.iter().enumerate().take(a.len() - suffix).skip(prefix) {
        for &j in positions.get(value).into_iter().flatten().rev() {
            let at = tails.partition_point(|&link| links[link].1 < j);
            let previous = at.checked_sub(1).map(|at| tails[at]);
            let link = links.len();
            links.push((i, j, previous));
            if at == tails.len() {
                tails.push(link);
            } else {
                tails[at] = link;
            }
        }
    }
    let mut middle = Vec::new();
    let mut link = tails.last().copied();
    while let Some(at) = link {
        let (i, j, previous) = links[at];
        middle.push((i, j));
        link = previous;
    }
    let mut pairs: Vec<_> = (0..prefix).map(|i| (i, i)).collect();
    pairs.extend(middle.into_iter().rev());
    pairs.extend((0..suffix).map(|i| (a.len() - suffix + i, b.len() - suffix + i)));
    pairs
}

/// Myers' shortest edit path, bounded independently of input length. Keeping
/// only 65 narrow frontiers bounds both the search and its reconstruction.
fn bounded_myers_pairs<T: Eq>(a: &[T], b: &[T]) -> Option<Vec<(usize, usize)>> {
    const LIMIT: usize = 64;
    const OFFSET: usize = LIMIT + 1;
    let mut frontier = vec![0; 2 * LIMIT + 3];
    let mut trace = Vec::new();
    for depth in 0..=LIMIT {
        trace.push(frontier.clone());
        for diagonal in (OFFSET - depth..=OFFSET + depth).step_by(2) {
            let down = diagonal == OFFSET - depth
                || (diagonal != OFFSET + depth && frontier[diagonal - 1] < frontier[diagonal + 1]);
            let mut x = if down {
                frontier[diagonal + 1]
            } else {
                frontier[diagonal - 1] + 1
            };
            let mut y = x + OFFSET - diagonal;
            while x < a.len() && y < b.len() && a[x] == b[y] {
                x += 1;
                y += 1;
            }
            frontier[diagonal] = x;
            if x == a.len() && y == b.len() {
                let mut pairs = Vec::new();
                for d in (1..=depth).rev() {
                    let diagonal = x + OFFSET - y;
                    let previous = &trace[d];
                    let down = diagonal == OFFSET - d
                        || (diagonal != OFFSET + d
                            && previous[diagonal - 1] < previous[diagonal + 1]);
                    let prior_diagonal = if down { diagonal + 1 } else { diagonal - 1 };
                    let prior_x = previous[prior_diagonal];
                    let prior_y = prior_x + OFFSET - prior_diagonal;
                    while x > prior_x && y > prior_y {
                        x -= 1;
                        y -= 1;
                        pairs.push((x, y));
                    }
                    x = prior_x;
                    y = prior_y;
                }
                while x > 0 && y > 0 {
                    x -= 1;
                    y -= 1;
                    pairs.push((x, y));
                }
                pairs.reverse();
                return Some(pairs);
            }
        }
    }
    None
}

/// Pair replacement gaps between the LCS anchors. Inserted/deleted tails
/// have no counterpart; a changed paragraph between anchors still has one.
fn aligned_pairs<T: Eq + std::hash::Hash>(a: &[T], b: &[T]) -> Vec<(usize, usize)> {
    let exact = ordered_pairs(a, b);
    let mut pairs = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (x, y) in exact.into_iter().chain(std::iter::once((a.len(), b.len()))) {
        pairs.extend((i..x).zip(j..y));
        if x < a.len() {
            pairs.push((x, y));
        }
        i = x + 1;
        j = y + 1;
    }
    pairs
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ForeignKey {
    part: String,
    paragraph: Option<(bool, usize)>,
    owner: String,
    author: Option<String>,
    kind: crate::changes::ChangeKind,
    target: &'static str,
    date: Option<String>,
    move_side: Option<crate::changes::MoveSide>,
    /// A run formatting change's recorded old properties: fragments of one
    /// record compare equal only when they recorded the same formatting.
    payload: Option<String>,
}

struct ForeignGroup<'a> {
    key: ForeignKey,
    change: &'a Change,
    text: String,
    start: usize,
    end: usize,
    relaxed_start: usize,
    relaxed_end: usize,
    /// The characters a run formatting change covers in its paragraph's
    /// text. Rejecting the author's changes restores the paragraph's text,
    /// so these compare across the two documents (#1656).
    characters: Option<(usize, usize)>,
}

impl ForeignGroup<'_> {
    /// What the comparison matches: the text, and for a run formatting
    /// change also where in its paragraph the text is.
    fn label(&self) -> String {
        match self.characters {
            Some((start, end)) => format!("{start}-{end}:{}", self.text),
            None => self.text.clone(),
        }
    }
}

/// Story-local paragraph identities. Full text includes deleted text, so a
/// deletion-only paragraph participates in the alignment too.
fn foreign_paragraphs(
    before: &[crate::changes::AuditStory],
    after: &[crate::changes::AuditStory],
) -> HashMap<(String, String), usize> {
    let mut identities = HashMap::new();
    for original in before {
        let Some(edited) = after.iter().find(|s| s.part == original.part) else {
            continue;
        };
        let a: Vec<_> = original.paragraphs.iter().map(|(_, text)| text).collect();
        let b: Vec<_> = edited.paragraphs.iter().map(|(_, text)| text).collect();
        // An LCS alone can choose the moved paragraph as an anchor in a
        // swap. Crossing unique text counterparts establish relocation;
        // inserting a new paragraph above history creates no such crossing.
        let mut old_positions: HashMap<&String, Vec<usize>> = HashMap::new();
        let mut new_positions: HashMap<&String, Vec<usize>> = HashMap::new();
        for (i, text) in a.iter().enumerate() {
            old_positions.entry(text).or_default().push(i);
        }
        for (j, text) in b.iter().enumerate() {
            new_positions.entry(text).or_default().push(j);
        }
        let unique: Vec<_> = a
            .iter()
            .map(|text| {
                let old = old_positions.get(text)?;
                let new = new_positions.get(text)?;
                (old.len() == 1 && new.len() == 1).then_some(new[0])
            })
            .collect();
        let mut crossed = vec![false; a.len()];
        let mut maximum = None;
        for (i, position) in unique.iter().enumerate() {
            if let Some(j) = position {
                crossed[i] |= maximum.is_some_and(|max| max > *j);
                maximum = Some(maximum.map_or(*j, |max: usize| max.max(*j)));
            }
        }
        let mut minimum = None;
        for (i, position) in unique.iter().enumerate().rev() {
            if let Some(j) = position {
                crossed[i] |= minimum.is_some_and(|min| min < *j);
                minimum = Some(minimum.map_or(*j, |min: usize| min.min(*j)));
            }
        }
        for (i, j) in aligned_pairs(&a, &b) {
            if !crossed[i] {
                identities.insert((edited.part.clone(), edited.paragraphs[j].0.clone()), i);
            }
        }
    }
    identities
}

/// Where each counted character of an edited paragraph sits in the
/// original paragraph aligned with it, for the aligned paragraphs whose
/// content differs: (original, edited) index pairs, both increasing.
type CharacterPairs = HashMap<(String, String), (Vec<(usize, usize)>, usize)>;

fn foreign_characters(
    before: &[crate::changes::AuditStory],
    after: &[crate::changes::AuditStory],
    paragraphs: &HashMap<(String, String), usize>,
) -> CharacterPairs {
    let mut out = HashMap::new();
    for edited in after {
        let Some(original) = before.iter().find(|s| s.part == edited.part) else {
            continue;
        };
        for ((id, _), content) in edited.paragraphs.iter().zip(&edited.contents) {
            let key = (edited.part.clone(), id.clone());
            let Some(old) = paragraphs.get(&key).and_then(|&i| original.contents.get(i)) else {
                continue;
            };
            if old != content {
                let a: Vec<char> = old.chars().collect();
                let b: Vec<char> = content.chars().collect();
                out.insert(key, (ordered_pairs(&a, &b), a.len()));
            }
        }
    }
    out
}

/// An edited paragraph's characters `start..end` in the original's
/// positions ([`foreign_characters`]): from the first kept character at or
/// after `start` to the last kept one before `end`.
fn original_characters(
    (pairs, length): &(Vec<(usize, usize)>, usize),
    (start, end): (usize, usize),
) -> (usize, usize) {
    let first = pairs.partition_point(|&(_, j)| j < start);
    let from = pairs.get(first).map_or(*length, |&(i, _)| i);
    let last = pairs.partition_point(|&(_, j)| j < end);
    let to = last.checked_sub(1).map_or(0, |k| pairs[k].0 + 1);
    (from, to.max(from))
}

/// Keep each carrier for attribution. Windows may combine touching text,
/// but an altered window must still name the carrier that actually changed.
fn foreign_revisions<'a>(
    stories: &'a [crate::changes::AuditStory],
    paragraphs: &HashMap<(String, String), usize>,
    characters: &CharacterPairs,
    edited: bool,
) -> Vec<ForeignGroup<'a>> {
    let mut out: Vec<ForeignGroup<'a>> = Vec::new();
    for story in stories {
        let paragraph_indices: HashMap<_, _> = story
            .paragraphs
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id.as_str(), i))
            .collect();
        for located in &story.changes {
            let c = &located.change;
            let paragraph = c.paragraph.as_ref().map(|id| {
                if edited && let Some(&i) = paragraphs.get(&(story.part.clone(), id.clone())) {
                    (false, i)
                } else {
                    (
                        edited,
                        paragraph_indices
                            .get(id.as_str())
                            .copied()
                            .unwrap_or(usize::MAX),
                    )
                }
            });
            let key = ForeignKey {
                part: story.part.clone(),
                paragraph,
                owner: located.owner.clone(),
                author: c.author.clone(),
                kind: c.kind,
                target: c.target,
                date: c.date.clone(),
                move_side: c.move_side,
                payload: located.payload.clone(),
            };
            out.push(ForeignGroup {
                key,
                change: c,
                text: c.text.clone(),
                start: located.start,
                end: located.end,
                relaxed_start: located.start,
                relaxed_end: located.end,
                characters: located.characters.map(|span| {
                    match c
                        .paragraph
                        .as_ref()
                        .filter(|_| edited)
                        .and_then(|id| characters.get(&(story.part.clone(), id.clone())))
                    {
                        Some(pairs) => original_characters(pairs, span),
                        None => span,
                    }
                }),
            });
        }
    }
    out
}

/// Remove only the document spans of revisions with no sequence counterpart.
/// A still-present separator, even with changed text, remains a boundary.
fn relax_foreign_boundaries(
    groups: &mut [ForeignGroup<'_>],
    present: &[bool],
    mut missing: HashMap<String, Vec<(usize, usize)>>,
) {
    for (group, &present) in groups.iter().zip(present) {
        if !present {
            missing
                .entry(group.key.part.clone())
                .or_default()
                .push((group.start, group.end));
        }
    }
    let mut ranges: HashMap<String, Vec<(usize, usize, usize)>> = HashMap::new();
    for (part, mut spans) in missing {
        spans.sort_unstable();
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (start, end) in spans {
            if let Some(last) = merged.last_mut()
                && start <= last.1
            {
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }
        let mut removed = 0;
        ranges.insert(
            part.to_string(),
            merged
                .into_iter()
                .map(|(start, end)| {
                    let prior = removed;
                    removed += end - start;
                    (start, end, prior)
                })
                .collect(),
        );
    }
    for group in groups {
        let Some(spans) = ranges.get(&group.key.part) else {
            continue;
        };
        let position = |pos: usize| {
            let index = spans.partition_point(|&(start, _, _)| start < pos);
            if index == 0 {
                return pos;
            }
            let (start, end, prior) = spans[index - 1];
            pos - prior - (pos.min(end) - start)
        };
        group.relaxed_start = position(group.start);
        group.relaxed_end = position(group.end);
    }
}

/// A comparison window may lose boundaries only where a missing separator
/// explains them. Retain every carrier for attribution if the text differs.
/// The fragments of a run formatting change join only where their
/// characters touch, and the window keeps where its characters are.
fn foreign_window(groups: &[ForeignGroup<'_>], indices: &[usize]) -> Vec<(Vec<usize>, String)> {
    // Carriers, joined text and, for run formatting, joined characters.
    type Window = (Vec<usize>, String, Option<(usize, usize)>);
    let mut windows: Vec<Window> = Vec::new();
    let mut previous = None;
    for &index in indices {
        let group = &groups[index];
        let touches = |p: usize| match (groups[p].characters, group.characters) {
            (Some((_, end)), Some((start, _))) => end == start,
            (None, None) => groups[p].relaxed_end == group.relaxed_start,
            _ => false,
        };
        if previous.is_some_and(touches)
            && let Some((carriers, text, characters)) = windows.last_mut()
        {
            carriers.push(index);
            text.push_str(&group.text);
            if let (Some(window), Some((_, end))) = (characters, group.characters) {
                window.1 = end;
            }
        } else {
            windows.push((vec![index], group.text.clone(), group.characters));
        }
        previous = Some(index);
    }
    windows
        .into_iter()
        .map(|(carriers, text, characters)| match characters {
            Some((start, end)) => (carriers, format!("{start}-{end}:{text}")),
            None => (carriers, text),
        })
        .collect()
}

fn foreign_finding(group: &ForeignGroup<'_>, state: &str) -> Finding {
    let c = group.change;
    let author = c.author.as_deref().unwrap_or("<no author>");
    let message = if state == "gone" {
        format!("{author}'s {} in the original is gone", c.id)
    } else {
        format!("{} by {author} {state}", c.id)
    };
    Finding::new("FOREIGN_AUTHOR", &group.key.part, "", message)
}

type ForeignSpans = HashMap<String, Vec<(usize, usize)>>;
type ForeignFindings = Vec<(bool, usize, Finding)>;

/// Match omitted markers within aligned paragraphs, ignoring revision ids.
/// Retained slots remain boundaries. Only absent slots enter the span union.
fn missing_foreign_markers(
    before: &[crate::changes::AuditStory],
    after: &[crate::changes::AuditStory],
    paragraphs: &HashMap<(String, String), usize>,
) -> (ForeignSpans, ForeignSpans) {
    let keys = |stories: &[crate::changes::AuditStory], edited: bool| {
        stories
            .iter()
            .flat_map(|story| {
                story.markers.iter().map(move |marker| {
                    let paragraph = marker.paragraph.map(|i| {
                        if edited
                            && let Some(&index) =
                                paragraphs.get(&(story.part.clone(), story.paragraphs[i].0.clone()))
                        {
                            (false, index)
                        } else {
                            (edited, i)
                        }
                    });
                    (
                        story.part.clone(),
                        paragraph,
                        marker.owner.clone(),
                        marker.kind.clone(),
                    )
                })
            })
            .collect::<Vec<_>>()
    };
    let a = keys(before, false);
    let b = keys(after, true);
    let pairs = ordered_pairs(&a, &b);
    let present_a: HashSet<_> = pairs.iter().map(|&(i, _)| i).collect();
    let present_b: HashSet<_> = pairs.iter().map(|&(_, j)| j).collect();
    let missing = |stories: &[crate::changes::AuditStory], present: &HashSet<usize>| {
        let mut spans: ForeignSpans = HashMap::new();
        for (i, (story, marker)) in stories
            .iter()
            .flat_map(|story| story.markers.iter().map(move |marker| (story, marker)))
            .enumerate()
        {
            if !present.contains(&i) {
                spans
                    .entry(story.part.clone())
                    .or_default()
                    .push((marker.start, marker.start + 1));
            }
        }
        spans
    };
    (missing(before, &present_a), missing(after, &present_b))
}

/// Exact carriers within an unequal combined window precede replacements.
/// No finding is allowed to borrow an unchanged neighbor's id.
fn report_foreign_carriers(
    before: &[ForeignGroup<'_>],
    after: &[ForeignGroup<'_>],
    old: &[usize],
    new: &[usize],
    findings: &mut ForeignFindings,
) {
    let a: Vec<_> = old.iter().map(|&i| before[i].label()).collect();
    let b: Vec<_> = new.iter().map(|&i| after[i].label()).collect();
    let (mut i, mut j) = (0, 0);
    for (x, y) in ordered_pairs(&a, &b)
        .into_iter()
        .chain(std::iter::once((a.len(), b.len())))
    {
        let paired = (x - i).min(y - j);
        for (offset, &index) in new[j..y].iter().enumerate() {
            let state = if offset < paired {
                "differs from the original"
            } else {
                "is new"
            };
            findings.push((false, index, foreign_finding(&after[index], state)));
        }
        for &index in &old[i + paired..x] {
            findings.push((true, index, foreign_finding(&before[index], "gone")));
        }
        i = x + 1;
        j = y + 1;
    }
}

fn check_foreign_revisions(
    original: &[u8],
    reverted: &[u8],
    author: &str,
    out: &mut Vec<Finding>,
) -> Result<(), ValidateError> {
    let original = crate::changes::audit_stories(original)?;
    let reverted = crate::changes::audit_stories(reverted)?;
    let paragraphs = foreign_paragraphs(&original, &reverted);
    let characters = foreign_characters(&original, &reverted, &paragraphs);
    let mut before = foreign_revisions(&original, &paragraphs, &characters, false);
    let mut after = foreign_revisions(&reverted, &paragraphs, &characters, true);
    let mut by_key: HashMap<ForeignKey, (Vec<usize>, Vec<usize>)> = HashMap::new();
    for (i, group) in before.iter().enumerate() {
        by_key.entry(group.key.clone()).or_default().0.push(i);
    }
    for (i, group) in after.iter().enumerate() {
        by_key.entry(group.key.clone()).or_default().1.push(i);
    }
    let mut before_present = vec![false; before.len()];
    let mut after_present = vec![false; after.len()];
    for (old, new) in by_key.values() {
        let a: Vec<_> = old.iter().map(|&i| before[i].label()).collect();
        let b: Vec<_> = new.iter().map(|&i| after[i].label()).collect();
        for (i, j) in aligned_pairs(&a, &b) {
            before_present[old[i]] = true;
            after_present[new[j]] = true;
        }
    }
    let (old_markers, new_markers) = missing_foreign_markers(&original, &reverted, &paragraphs);
    relax_foreign_boundaries(&mut before, &before_present, old_markers);
    relax_foreign_boundaries(&mut after, &after_present, new_markers);
    let mut findings = Vec::new();
    for (key, (old, new)) in &by_key {
        if key.author.as_deref() == Some(author) {
            continue;
        }
        let a: Vec<_> = old.iter().map(|&i| before[i].label()).collect();
        let b: Vec<_> = new.iter().map(|&i| after[i].label()).collect();
        let (mut i, mut j) = (0, 0);
        for (x, y) in ordered_pairs(&a, &b)
            .into_iter()
            .chain(std::iter::once((a.len(), b.len())))
        {
            let left = foreign_window(&before, &old[i..x]);
            let right = foreign_window(&after, &new[j..y]);
            let left_text: Vec<_> = left.iter().map(|(_, text)| text).collect();
            let right_text: Vec<_> = right.iter().map(|(_, text)| text).collect();
            let (mut p, mut q) = (0, 0);
            for (u, v) in ordered_pairs(&left_text, &right_text)
                .into_iter()
                .chain(std::iter::once((left.len(), right.len())))
            {
                let old_carriers: Vec<_> = left[p..u]
                    .iter()
                    .flat_map(|(indices, _)| indices.iter().copied())
                    .collect();
                let new_carriers: Vec<_> = right[q..v]
                    .iter()
                    .flat_map(|(indices, _)| indices.iter().copied())
                    .collect();
                report_foreign_carriers(
                    &before,
                    &after,
                    &old_carriers,
                    &new_carriers,
                    &mut findings,
                );
                p = u + 1;
                q = v + 1;
            }
            i = x + 1;
            j = y + 1;
        }
    }
    findings.sort_by_key(|(original, index, _)| (*original, *index));
    out.extend(findings.into_iter().map(|(_, _, finding)| finding));
    Ok(())
}

/// Permission exceptions are edit fidelity, not Word validity. Check both
/// endpoints by id in every XML part, so dropping only one is visible too.
fn check_dropped_permissions(original: &PartFs, edited: &PartFs, out: &mut Vec<Finding>) {
    for part in xml_parts(original) {
        if original
            .part_string(&part)
            .is_none_or(|xml| !xml.contains("permStart") && !xml.contains("permEnd"))
        {
            continue;
        }
        let Some((before, root)) = parse_part(original, &part) else {
            continue;
        };
        let after = parse_part(edited, &part);
        for kind in ["permStart", "permEnd"] {
            let name = W::name(kind);
            let mut remaining: HashMap<String, usize> = HashMap::new();
            if let Some((dom, root)) = &after {
                for n in dom.descendants(*root, Some(&name)) {
                    if let Some(id) = dom.attribute(n, &W::id()) {
                        *remaining.entry(id.to_string()).or_default() += 1;
                    }
                }
            }
            for n in before.descendants(root, Some(&name)) {
                let Some(id) = before.attribute(n, &W::id()) else {
                    continue;
                };
                if let Some(count) = remaining.get_mut(id)
                    && *count > 0
                {
                    *count -= 1;
                    continue;
                }
                out.push(Finding::new(
                    "PERMISSION_RANGE_DROPPED",
                    &part,
                    element_path(&before, n),
                    format!("original w:{kind} with w:id={id} is missing from the edited file"),
                ));
            }
        }
    }
}

/// [`audit_tracked`]'s findings belong to the `edits` set and fail a run.
fn as_edits(mut findings: Vec<Finding>) -> Vec<Finding> {
    for f in &mut findings {
        f.set = "edits".to_string();
        f.severity = "error".to_string();
    }
    findings
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
            // Word opens a comment paraId at 0x80000000 (probe h07) and
            // hangs on one that is not hexadecimal (h08, 2026-10-10).
            ("80000000", true, Some("outside Word")),
            ("80000000", false, None),
            // Seven digits open (k06); nine are refused (k07, 2026-10-10).
            ("1234567", true, None),
            ("1", false, None),
            ("", true, Some("not a hexadecimal id")),
            ("123456789", false, Some("not a hexadecimal id")),
            ("1234567z", true, Some("not a hexadecimal id")),
            ("é234567", false, Some("not a hexadecimal id")),
        ] {
            let mut findings = Vec::new();
            check_hex_id("test", value, bound, "part", &mut findings);
            assert_eq!(findings.len(), usize::from(expected.is_some()), "{value}");
            if let Some(message) = expected {
                let refused = message == "not a hexadecimal id";
                assert_eq!(
                    findings[0].code,
                    if refused {
                        "COMMENT_PARTS_INCONSISTENT"
                    } else {
                        "COMMENT_METADATA_MISMATCH"
                    }
                );
                assert!(findings[0].message.contains(message));
                assert_eq!(findings[0].word_fatal, refused, "{value}");
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
    fn deletion_scope_stops_at_textboxes_and_covers_nested_insertions() {
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
        assert_eq!(live, ["uncovered", "moved", "stranded"]);
        let deleted = dom
            .descendants(root, Some(&W::name("delText")))
            .into_iter()
            .map(|n| dom.value(n))
            .collect::<Vec<_>>();
        assert_eq!(deleted, ["deleted", "inserted"]);
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
            set: "word".into(),
            severity: "error".into(),
            part: "customXml/source-owned.opaque".into(),
            location: String::new(),
            r#ref: String::new(),
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
            let expected_range=["w14:paraId=\"","w14:textId=\""].map(|attribute|Finding {code:"PARA_ID_OUT_OF_RANGE".into(),set:"word".into(),severity:"warning".into(),part:"word/comments.xml".into(),location:String::new(),r#ref:String::new(),message:format!("{attribute} value '{old}' outside Word's range 1..0x7FFFFFFF (>= 0x80000000 or zero) in 'word/comments.xml' (id-paraid-overflow)"),word_fatal:false,repairable:true});
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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

    #[test]
    fn three_starts_of_one_id_before_their_ends_pair_one_to_one() {
        // Each start takes the first end of its id that no earlier start
        // has taken; the first start keeps its id (codex on #1666).
        let range = |tag: &str, id: u32| format!(r#"<w:customXmlIns{tag} w:id="{id}"/>"#);
        let body = |ids: [u32; 3]| {
            let starts: String = ids.iter().map(|&id| range("RangeStart", id)).collect();
            let ends: String = ids.iter().map(|&id| range("RangeEnd", id)).collect();
            format!("<w:p>{starts}<w:r><w:t>held</w:t></w:r>{ends}</w:p>")
        };
        let (mut dom, doc) = tree(&body([9, 9, 9]));
        let root = dom.root(doc).unwrap();
        let (expected_dom, expected_doc) = tree(&body([9, 10, 11]));
        renumber_duplicate_revision_ids(&mut dom, root);
        assert_eq!(
            dom.serialize_document(doc),
            expected_dom.serialize_document(expected_doc)
        );
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
                    *bytes = std::str::from_utf8(bytes)
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
                    "COMMENT_METADATA_MISMATCH",
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
                        "COMMENT_METADATA_MISMATCH",
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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

#[cfg(test)]
mod foreign_alignment_budget_tests {
    use super::ordered_pairs;
    use std::{
        cell::Cell,
        hash::{Hash, Hasher},
        rc::Rc,
    };

    struct Counted<'a> {
        text: &'a str,
        comparisons: Rc<Cell<usize>>,
    }
    impl PartialEq for Counted<'_> {
        fn eq(&self, other: &Self) -> bool {
            let count = self.comparisons.get() + 1;
            self.comparisons.set(count);
            assert!(
                count <= 1_000_000,
                "alignment exceeded 1,000,000 equality comparisons"
            );
            self.text == other.text
        }
    }
    impl Eq for Counted<'_> {}
    impl Hash for Counted<'_> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.text.hash(state);
        }
    }

    fn bounded_alignment(a: &[&'static str], b: &[&'static str], expected: usize) {
        let comparisons = Rc::new(Cell::new(0));
        let wrap = |values: &[&'static str]| {
            values
                .iter()
                .map(|&text| Counted {
                    text,
                    comparisons: comparisons.clone(),
                })
                .collect::<Vec<_>>()
        };
        // The test values are static; keep the counting type independent of
        // production code. The public audit tests separately bound allocation size.
        let left = wrap(a);
        let right = wrap(b);
        let pairs = ordered_pairs(&left, &right);
        assert_eq!(pairs.len(), expected);
        assert!(pairs.windows(2).all(|w| w[0].0 < w[1].0 && w[0].1 < w[1].1));
        assert!(pairs.iter().all(|&(i, j)| a[i] == b[j]));
    }

    #[test]
    fn foreign_alignment_ten_thousand_empty_paragraphs_is_bounded() {
        let mut a = vec![""; 10_002];
        let mut b = a.clone();
        a[0] = "old first";
        a[10_001] = "old last";
        b[0] = "new first";
        b[10_001] = "new last";
        bounded_alignment(&a, &b, 10_000);
    }

    #[test]
    fn foreign_alignment_ten_thousand_same_key_deletions_is_bounded() {
        let a = vec!["deleted"; 10_000];
        let mut b = a.clone();
        b[0] = "changed first";
        b[9_999] = "changed last";
        bounded_alignment(&a, &b, 9_998);
    }

    #[test]
    fn foreign_alignment_ten_thousand_alternating_paragraphs_is_bounded() {
        let a: Vec<_> = (0..10_000)
            .map(|i| if i % 2 == 0 { "a" } else { "b" })
            .collect();
        let mut b = a.clone();
        b[5_000] = "changed";
        bounded_alignment(&a, &b, 9_999);
    }
    #[test]
    fn foreign_alignment_dense_fallback_is_order_stable() {
        let a = vec!["a"; 200];
        let mut b = vec!["b"; 200];
        b[32..168].fill("a");
        bounded_alignment(&a, &b, 136);
        assert_eq!(ordered_pairs(&a, &b), ordered_pairs(&a, &b));
        // 66 edits exceed the Myers budget. The deterministic fallback can
        // select fewer anchors, but must preserve order and exact equality.
        let alternating: Vec<_> = (0..200)
            .map(|i| if i % 2 == 0 { "a" } else { "b" })
            .collect();
        let mut changed = alternating.clone();
        for i in 0..32 {
            changed[i * 5] = "changed";
        }
        changed[199] = "changed";
        bounded_alignment(&alternating, &changed, 135);
    }

    #[test]
    fn foreign_alignment_dense_myers_matches_short_edit_lcs() {
        for changed in [1, 8, 16, 31] {
            let a: Vec<_> = (0..200)
                .map(|i| if i % 2 == 0 { "a" } else { "b" })
                .collect();
            let mut b = a.clone();
            for i in 0..changed {
                b[i * 5] = "changed";
            }
            b[199] = "changed";
            bounded_alignment(&a, &b, 199 - changed);
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod baseline_tests {
    use super::*;

    fn finding(code: &str, location: &str, message: &str) -> Finding {
        Finding::new(code, "word/document.xml", location, message)
    }

    #[test]
    fn a_chain_resolves_to_the_element_it_names() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{W_NS}"><w:body><w:p/><w:p><w:r/><w:r><w:t>x</w:t></w:r></w:p></w:body></w:document>"#,
            W_NS = W::URI
        ));
        let root = dom.root(doc).unwrap();
        let t = dom.descendants(root, Some(&W::t()))[0];
        let chain = element_path(&dom, t);
        assert_eq!(chain, "w:body[0]/w:p[1]/w:r[1]/w:t[0]");
        let mut part = Part::new(dom, root);
        assert_eq!(part.resolve(&chain), Some(t));
        assert_eq!(part.resolve("w:body[0]/w:p[2]"), None);
        assert_eq!(part.resolve("w:body[0]/w:p"), None);
    }

    #[test]
    fn identity_drops_element_indices_and_paragraph_ids_from_the_message() {
        let a = finding(
            "BOOKMARK_CROSSES_REVISION",
            "",
            "_Hlk1: start w:body[0]/w:p[3]/w:bookmarkStart[0] and end p3, p4-p5.",
        );
        let b = finding(
            "BOOKMARK_CROSSES_REVISION",
            "",
            "_Hlk1: start w:body[0]/w:p[9]/w:bookmarkStart[0] and end p9, p10-p11.",
        );
        assert_eq!(a.identity(), b.identity());
        assert_eq!(
            a.identity().message,
            "_Hlk1: start w:body/w:p/w:bookmarkStart and end p#, p#-p#."
        );
        // The entity a message names stays: another bookmark is another
        // finding, and so is a bracket that is no index.
        let other = finding(
            "BOOKMARK_CROSSES_REVISION",
            "",
            "_Hlk2: start w:body[0]/w:p[3]/w:bookmarkStart[0] and end p3, p4-p5.",
        );
        assert_ne!(a.identity(), other.identity());
        assert_eq!(normalised("id [a] and [12x]"), "id [a] and [12x]");
        assert_eq!(
            normalised("header1.p2, t0.row1.cell2 and t0.row1.cell2.p1"),
            "p#, p# and p#"
        );
        // Stories, parts and words are kept.
        assert_eq!(
            normalised("header1 footnotes commentsExtensible word/document.xml"),
            "header1 footnotes commentsExtensible word/document.xml"
        );
        // Comments and revisions are entities, not positions.
        assert_eq!(normalised("c5 on r12 and c5.text"), "c5 on r12 and c5.text");
        assert_eq!(normalised("durableId '00251B63'"), "durableId '00251B63'");
    }

    #[test]
    fn identity_keeps_where_the_finding_sits() {
        let here = finding("TEXT_INSIDE_DELETION", "w:body[0]/w:p[3]", "w:t in w:del");
        let there = finding("TEXT_INSIDE_DELETION", "w:body[0]/w:p[9]", "w:t in w:del");
        assert_ne!(here.identity(), there.identity());
        let mut audit = here.clone();
        audit.set = "a11y".to_string();
        assert_ne!(here.identity(), audit.identity());
    }

    #[test]
    fn baseline_matches_one_for_one_by_identity() {
        let x = finding("DUPLICATE_DOCPR_ID", "", "docPr id 1 appears 2 times");
        let y = finding("DUPLICATE_DOCPR_ID", "", "docPr id 7 appears 2 times");
        // The same identity twice in the output, once in the input: one of
        // them is new.
        let split = baseline(vec![x.clone(), x.clone()], vec![x.clone()]);
        assert_eq!(split.baseline, vec![x.clone()]);
        assert_eq!(split.new, vec![x.clone()]);
        assert!(split.resolved.is_empty());
        // One replaced by another at the same count.
        let split = baseline(vec![y.clone()], vec![x.clone()]);
        assert_eq!(split.new, vec![y.clone()]);
        assert!(split.baseline.is_empty());
        assert_eq!(split.resolved, vec![x.clone()]);
        // Order is each input's own.
        let split = baseline(
            vec![y.clone(), x.clone()],
            vec![x.clone(), y.clone(), x.clone()],
        );
        assert_eq!(split.baseline, vec![y.clone(), x.clone()]);
        assert_eq!(split.resolved, vec![x]);
    }

    #[test]
    fn baselined_text_and_json_name_each_list() {
        let x = finding("DUPLICATE_DOCPR_ID", "", "docPr id 1 appears 2 times");
        let empty = Baselined::default();
        assert_eq!(
            baselined_text(&empty),
            "new: none\nalready in the input: none\n"
        );
        assert_eq!(baselined_json(&empty).unwrap(), "{\"resolved\":[]}\n");
        let split = Baselined {
            new: vec![x.clone()],
            baseline: vec![x.clone()],
            resolved: vec![x.clone()],
        };
        let line = finding_line(&x);
        assert_eq!(
            baselined_text(&split),
            format!(
                "new: 1\n{line}already in the input: 1 (not yours)\n\
                 resolved: 1 (in the input, not in this file)\n{line}"
            )
        );
        let json = baselined_json(&split).unwrap();
        let rows: Vec<serde_json::Value> = json
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(rows[0]["status"], "new");
        assert_eq!(rows[1]["status"], "baseline");
        assert_eq!(rows[0]["code"], "DUPLICATE_DOCPR_ID");
        assert_eq!(rows[2]["resolved"][0]["code"], "DUPLICATE_DOCPR_ID");
        assert!(rows[2]["resolved"][0].get("status").is_none());
    }

    #[test]
    fn input_notice_counts_errors_and_warnings_but_not_info() {
        let warning = finding("DUPLICATE_DOCPR_ID", "", "w");
        let error = finding("TEXT_INSIDE_DELETION", "", "e");
        let mut info = warning.clone();
        info.severity = "info".to_string();
        assert_eq!(warning.severity, "warning");
        assert_eq!(error.severity, "error");
        assert_eq!(input_notice(&[], "a.docx"), None);
        assert_eq!(input_notice(std::slice::from_ref(&info), "a.docx"), None);
        assert_eq!(
            input_notice(std::slice::from_ref(&warning), "a.docx").unwrap(),
            "input has 1 validity warning; jubarte validate a.docx lists it"
        );
        assert_eq!(
            input_notice(&[warning.clone(), warning.clone(), info], "a.docx").unwrap(),
            "input has 2 validity warnings; jubarte validate a.docx lists them"
        );
        assert_eq!(
            input_notice(&[error.clone(), error.clone()], "a.docx").unwrap(),
            "input has 2 validity errors; jubarte validate a.docx lists them"
        );
        assert_eq!(
            input_notice(&[error, warning.clone(), warning.clone()], "my file.docx").unwrap(),
            "input has 1 validity error and 2 warnings; \
             jubarte validate 'my file.docx' lists them"
        );
    }
}
