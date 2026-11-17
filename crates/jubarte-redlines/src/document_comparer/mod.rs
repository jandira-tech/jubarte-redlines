// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! DocumentComparer façade (M5). Port of `DocumentComparer.ts` (compare path).
//!
//! `compare_documents(original, modified, author) -> Vec<u8>` opens both
//! packages, diffs their main-document bodies, and writes a redline `.docx` (the
//! original package with `word/document.xml` replaced by the tracked-revision
//! result).
//!
//! NOTE: this is the text-level compare path. Full DocumentComparer additionally
//! runs PreProcessMarkup/accept/hash and relocates footnotes/comments/related
//! parts (the M4.5–M4.6 refinements) for exact golden parity on complex docs.

use crate::comparer::{WmlComparerSettings, compare_bodies_faithful};
use crate::namespaces::{R, W, W14};
use crate::opc::{OpcError, PartFs};
use crate::xmllinq::{Dom, NodeId, XName};

mod pre_process;
mod stories;
mod styles;
use pre_process::*;
use stories::*;
use styles::*;

/// A header/footer comparison that cannot produce a faithful redline.
/// Carried inside [`OpcError::Io`]; callers can downcast its `get_ref()`.
#[derive(Debug)]
pub struct StoryDiffError {
    /// Package part whose comparison failed.
    pub part: String,
    /// The failed operation or output invariant.
    pub reason: &'static str,
}

impl std::fmt::Display for StoryDiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STORY_DIFF_FAILED: {}: {}", self.part, self.reason)
    }
}

impl std::error::Error for StoryDiffError {}

pub use crate::limits::UnsupportedContentError;

/// Each section of the redline `root` in order, as (its break is deleted,
/// the header/footer parts it references). A paragraph's `w:sectPr` whose
/// mark is deleted is a deleted break; any other paragraph's, and the
/// body's own, is a live section.
#[doc(hidden)]
pub fn section_story_parts(
    dom: &Dom,
    root: NodeId,
    pkg: &PartFs,
    main: &str,
) -> Vec<(bool, Vec<String>)> {
    let Some(rels) = pkg.read_rels_for(main) else {
        return Vec::new();
    };
    let targets: std::collections::HashMap<&str, &str> = rels
        .items
        .iter()
        .map(|r| (r.id.as_str(), r.target.as_str()))
        .collect();
    section_props(dom, root)
        .into_iter()
        .map(|sect| {
            let deleted = dom
                .parent(sect)
                .filter(|&p| dom.name_is(p, &W::p_pr()))
                .and_then(|p| dom.element(p, &W::r_pr()))
                .is_some_and(|r| dom.element(r, &W::del()).is_some());
            let parts = dom
                .elements(sect, None)
                .into_iter()
                .filter(|&r| {
                    dom.name_is(r, &W::header_reference()) || dom.name_is(r, &W::footer_reference())
                })
                .filter_map(|r| targets.get(dom.attribute(r, &R::id())?))
                .map(|target| referenced_part(pkg, main, target))
                .collect();
            (deleted, parts)
        })
        .collect()
}

/// Default pinned revision date when the caller doesn't specify one.
pub const DEFAULT_DATE: &str = "1970-01-01T00:00:00Z";

/// C.1/C.2 — `WmlComparer.PreProcessMarkup` (:434) at package level:
/// `ChangeFootnoteEndnoteReferencesToUniqueRange` (:1627) then
/// `AddFootnotesEndnotesParts` (:1604). C.3–C.5 extend it with
/// FillInEmptyFootnotesEndnotes, DetachExternalData and
/// AddUnidsToMarkupInContentParts in the C# order. Returns the names of the
/// parts it rewrote or created (empty = pure no-op, bytes untouched). An
/// orphaned footnote/endnote reference is an `invalid_content` error — C#
/// throws DocxodusException when no ComparisonLog is wired (:1676), and the
/// compare path wires none.
///
/// # Errors
///
/// [`OpcError`] when an orphan footnote/endnote reference has no matching
/// definition, or the OPC layer cannot parse / serialise a referenced part.
pub fn pre_process_markup(
    pkg: &mut PartFs,
    starting_id_for_footnotes_endnotes: i32,
) -> Result<Vec<String>, OpcError> {
    pre_process(pkg, starting_id_for_footnotes_endnotes, false)
}

/// A.11 — `RevisionProcessor.AcceptRevisions` byte facade: accept every
/// tracked revision across main + headers/footers + notes + comments +
/// styles parts and return the result as a `.docx` byte vector. The input
/// is admitted under [`crate::admission::InputLimits::compare`] first.
///
/// # Errors
///
/// [`OpcError`] when admission refuses the input or the OPC layer cannot
/// parse / serialise the package.
pub fn accept_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError> {
    crate::admission::admit_package(docx)?;
    let mut pkg = PartFs::open(docx)?;
    crate::revision_processor::accept_revisions_package(&mut pkg);
    repair_resolved_package_validity(&mut pkg);
    pkg.to_zip()
}

/// A.11 — `RevisionProcessor.RejectRevisions` byte facade: reject every
/// tracked revision across main + headers/footers + notes + comments +
/// styles parts.
pub fn reject_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError> {
    crate::admission::admit_package(docx)?;
    let mut pkg = PartFs::open(docx)?;
    crate::revision_processor::reject_revisions_package(&mut pkg);
    repair_resolved_package_validity(&mut pkg);
    pkg.to_zip()
}

/// M481 — Word-repair: wire core parts the package carries but the main
/// document never references. superdoc_hyperlink_cases ships styles.xml
/// WITHOUT a styles relationship in document.xml.rels; a spec-following
/// consumer (LO included) then never loads the stylesheet and the whole
/// document renders in fallback fonts. Word repairs the relationship on
/// open (oracle rels: rId1 -> styles.xml), so match it at compare time —
/// same family as the dangling-numbering repair.
const CUSTOM_PROPS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";

/// The part the package's custom-properties relationship reaches.
fn custom_props_part(pkg: &PartFs) -> Option<String> {
    pkg.package_relationships()
        .items
        .iter()
        .find(|r| r.rel_type == CUSTOM_PROPS_REL)
        .map(|r| r.target.trim_start_matches('/').to_string())
        .filter(|p| pkg.part_bytes(p).is_some())
}

/// Word's redline keeps the custom document properties of both sides: the
/// revision's alone when the original has none (83 of the 96 Word redlines
/// where only the revision has them), else the union, the original's value
/// winning where both name a property (every conflict among the 47 with
/// both). Without them a `DOCPROPERTY` field of the accepted text reads
/// "Error! Unknown document property name." (f8c1ce3e92).
fn merge_custom_properties(out: &mut PartFs, pkg2: &PartFs) {
    let Some(b_part) = custom_props_part(pkg2) else {
        return;
    };
    let Some(b_xml) = pkg2.part_string(&b_part) else {
        return;
    };
    let Some(a_part) = custom_props_part(out) else {
        if out.part_bytes(&b_part).is_some() {
            return;
        }
        out.set_part(&b_part, b_xml.into_bytes());
        out.add_content_type_override(
            &format!("/{b_part}"),
            "application/vnd.openxmlformats-officedocument.custom-properties+xml",
        );
        out.add_package_relationship(CUSTOM_PROPS_REL, &b_part);
        return;
    };
    let Some(a_xml) = out.part_string(&a_part) else {
        return;
    };
    let mut dom = Dom::new();
    let a_doc = dom.parse_xdocument(&a_xml);
    let b_doc = dom.parse_xdocument(&b_xml);
    let (Some(a_root), Some(b_root)) = (dom.root(a_doc), dom.root(b_doc)) else {
        return;
    };
    let name = XName::get("name", "");
    let pid = XName::get("pid", "");
    let a_props = dom.elements(a_root, None);
    let mut next_pid = a_props
        .iter()
        .filter_map(|&p| dom.attribute(p, &pid)?.parse::<u32>().ok())
        .max()
        .unwrap_or(1);
    // Property names match without regard to case: the original's
    // "Actno" and the revision's "ActNo" are one property, and two of them
    // make Word refuse the package (19eb128df9 vs 9666fa13fb).
    let mut a_names: std::collections::HashSet<String> = a_props
        .iter()
        .filter_map(|&p| dom.attribute(p, &name).map(str::to_lowercase))
        .collect();
    let mut added = false;
    for bp in dom.elements(b_root, None) {
        let Some(n) = dom.attribute(bp, &name) else {
            continue;
        };
        if !a_names.insert(n.to_lowercase()) {
            continue;
        }
        let clone = dom.clone_subtree(bp);
        next_pid += 1;
        dom.set_attribute_value(clone, &pid, Some(&next_pid.to_string()));
        dom.add(a_root, clone);
        added = true;
    }
    if added {
        out.set_part(&a_part, dom.serialize_document(a_doc).into_bytes());
    }
}

fn repair_missing_core_relationships(out: &mut PartFs, out_main: &str) {
    const CORE: [(&str, &str); 5] = [
        ("word/styles.xml", "styles"),
        ("word/settings.xml", "settings"),
        ("word/webSettings.xml", "webSettings"),
        ("word/fontTable.xml", "fontTable"),
        ("word/theme/theme1.xml", "theme"),
    ];
    for (part, rel_suffix) in CORE {
        if out.part_bytes(part).is_none() {
            continue;
        }
        let has_rel = out.read_rels_for(out_main).is_some_and(|r| {
            r.items
                .iter()
                .any(|i| i.rel_type.ends_with(&format!("/{rel_suffix}")))
        });
        if !has_rel {
            let target = crate::opc::relative_rel_target(out_main, part);
            out.add_document_relationship(
                out_main,
                &format!(
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{rel_suffix}"
                ),
                &target,
            );
        }
    }
}

/// D.6 — `WmlComparer.GetRevisions` (:3940) byte facade: list every tracked
/// revision in a redline `.docx` — main-part groups, footnote/endnote
/// definition groups, `w:rPrChange` format changes, then (settings-gated)
/// move detection. `TestForInvalidContent` failures are an `invalid_content`
/// error where C# throws.
pub fn get_revisions(
    docx: &[u8],
    settings: &crate::comparer::WmlComparerSettings,
) -> Result<Vec<crate::comparer::WmlComparerRevision>, OpcError> {
    use crate::comparer::{preprocess, revisions};

    crate::admission::admit_input("input", docx, settings.input_limits)?;
    let pkg = PartFs::open_within(docx, settings.input_limits)?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg
        .part_string(&main)
        .ok_or_else(|| OpcError::PartNotFound(main.clone()))?;
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom
        .root(d)
        .ok_or_else(|| OpcError::PartNotFound(format!("{main}: no root element")))?;

    // C# :3948–:3949 — TestForInvalidContent (throws) +
    // RemoveExistingPowerToolsMarkup on main and both notes parts.
    refuse_unatomizable(&dom, root, "input document")?;
    preprocess::remove_existing_powertools_markup(&mut dom, root);
    let mut note_roots: Vec<(NodeId, &str, crate::xmllinq::XName)> = Vec::new();
    let (fn_part, en_part) = notes_part_names(&pkg);
    for (part, def) in [
        (fn_part.as_str(), W::footnote()),
        (en_part.as_str(), W::endnote()),
    ] {
        if let Some(x) = pkg.part_string(part) {
            let nd = dom.parse_xdocument(&x);
            if let Some(r) = dom.root(nd) {
                refuse_unatomizable(&dom, r, part)?;
                preprocess::remove_existing_powertools_markup(&mut dom, r);
                note_roots.push((r, part, def));
            }
        }
    }
    let units = note_roots
        .iter()
        .fold(story_units(&dom, root), |sum, (r, _, _)| {
            sum.saturating_add(story_units(&dom, *r))
        });
    refuse_past_story_budget(units, "input document", settings.input_limits)?;

    let body = dom
        .element(root, &W::body())
        .ok_or_else(|| OpcError::PartNotFound(format!("{main}: no w:body")))?;
    let mut revs = revisions::get_revisions_from_body(&mut dom, body, &main, settings);
    for (r, part, def) in &note_roots {
        revs.extend(revisions::get_revisions_from_note_definitions(
            &mut dom, *r, def, part, settings,
        ));
    }
    let mut fc_parts: Vec<(NodeId, &str)> = vec![(root, main.as_str())];
    for (r, part, _) in &note_roots {
        fc_parts.push((*r, part));
    }
    revs.extend(revisions::get_format_change_revisions(&mut dom, &fc_parts));
    revisions::detect_moves(&mut revs, settings);
    Ok(revs)
}

/// Serialize one revision to the stable JSON object shape of the wasm
/// `getRevisions` and Python `get_revisions` bindings. Strings escape
/// backslash, quote and every control character below U+0020, so document
/// text can carry `\t`, `\r`, vertical tabs, etc.; U+007F passes as is.
pub fn revision_to_json(r: &crate::comparer::WmlComparerRevision) -> String {
    fn esc(s: &str) -> String {
        let mut o = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\\' => o.push_str("\\\\"),
                '"' => o.push_str("\\\""),
                '\n' => o.push_str("\\n"),
                '\r' => o.push_str("\\r"),
                '\t' => o.push_str("\\t"),
                c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                c => o.push(c),
            }
        }
        o
    }
    let format_change = r.format_change.as_ref().map_or("null".to_string(), |fc| {
        let props: Vec<String> = fc
            .changed_properties
            .iter()
            .map(|p| format!("\"{}\"", esc(p)))
            .collect();
        format!("{{\"changedProperties\":[{}]}}", props.join(","))
    });
    format!(
        "{{\"type\":\"{:?}\",\"author\":\"{}\",\"date\":\"{}\",\"part\":\"{}\",\"moveGroupId\":{},\"isMoveSource\":{},\"formatChange\":{},\"text\":\"{}\"}}",
        r.revision_type,
        esc(r.author.as_deref().unwrap_or("")),
        esc(r.date.as_deref().unwrap_or("")),
        esc(&r.part_name),
        r.move_group_id
            .map_or("null".to_string(), |v| v.to_string()),
        r.is_move_source
            .map_or("null".to_string(), |v| v.to_string()),
        format_change,
        esc(r.text.as_deref().unwrap_or("")),
    )
}

/// Serialize a revision list to a single JSON array string — the wasm
/// `getRevisions` binding shape. The CLI prints one object per line
/// instead.
pub fn revisions_to_json(revs: &[crate::comparer::WmlComparerRevision]) -> String {
    let items: Vec<String> = revs.iter().map(revision_to_json).collect();
    format!("[{}]", items.join(","))
}

/// `DocumentComparer.CompareDocuments(original, modified, author)`.
pub fn compare_documents(
    original: &[u8],
    modified: &[u8],
    author: &str,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_with_options(original, modified, author, DEFAULT_DATE)
}

/// Compare with an explicit revision timestamp (for reproducible output).
pub fn compare_documents_with_options(
    original: &[u8],
    modified: &[u8],
    author: &str,
    date: &str,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_internal(original, modified, author, date, true)
}

/// Compare with caller-supplied [`WmlComparerSettings`] (author/date/detail
/// threshold/…). The other entry points delegate here with defaults.
pub fn compare_documents_with_settings(
    original: &[u8],
    modified: &[u8],
    settings: &WmlComparerSettings,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_impl(original, modified, settings, true, true)
}

/// Edit restores permissions under its own revised-by-id policy (#1176).
#[doc(hidden)]
pub fn compare_documents_for_edit(
    original: &[u8],
    modified: &[u8],
    settings: &WmlComparerSettings,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_impl(original, modified, settings, true, false)
}

/// `WmlComparer.CompareInternal` (:152). `pre_process_original` mirrors C#'s
/// `preProcessMarkupInOriginal` — true for Compare; Consolidate passes false
/// because its original is already preprocessed (`CompareInternal(..., false)`).
pub fn compare_documents_internal(
    original: &[u8],
    modified: &[u8],
    author: &str,
    date: &str,
    pre_process_original: bool,
) -> Result<Vec<u8>, OpcError> {
    let settings = WmlComparerSettings {
        author_for_revisions: author.to_string(),
        date_time_for_revisions: date.to_string(),
        ..WmlComparerSettings::default()
    };
    compare_documents_impl(original, modified, &settings, pre_process_original, true)
}

/// Resolve the footnotes/endnotes part names via the main-document rels,
/// falling back to the standard names. OPC makes the rels authoritative —
/// producers legally use nonstandard part names, and hardcoding
/// `word/footnotes.xml` silently skipped their notes (PR #51 review).
fn notes_part_names(pkg: &PartFs) -> (String, String) {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let mut fn_p = "word/footnotes.xml".to_string();
    let mut en_p = "word/endnotes.xml".to_string();
    if let Some(rels) = pkg.read_rels_for(&main) {
        for r in &rels.items {
            if r.target_mode.as_deref() == Some("External") {
                continue;
            }
            match r.rel_type.rsplit('/').next().unwrap_or("") {
                "footnotes" => fn_p = pkg.resolve_rel_target(&main, &r.target),
                "endnotes" => en_p = pkg.resolve_rel_target(&main, &r.target),
                _ => {}
            }
        }
    }
    (fn_p, en_p)
}

/// True when any package part's XML carries tracked-change markup that Word
/// Compare would fold into the final view before diffing. Opened under the
/// same budget the caller admitted the input with, so a settings budget
/// looser than the constructor's still governs (#645).
fn main_part_xml(docx: &[u8], limits: crate::admission::InputLimits) -> Option<String> {
    let pkg = PartFs::open_within(docx, limits).ok()?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg.part_string(&main)?;
    Some(normalize_strict_namespaces(&xml).into_owned())
}

fn docx_has_tracked_changes(docx: &[u8], limits: crate::admission::InputLimits) -> bool {
    let Ok(pkg) = PartFs::open_within(docx, limits) else {
        return false;
    };
    for name in pkg.parts() {
        if !name.ends_with(".xml") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        // Whole element names: `<w:instrText>` and `<w:insideH>` are not
        // revisions, and tidying a document with none rewrites it (#1870).
        if crate::revision_processor::xml_has_tracked_revisions(&xml) {
            return true;
        }
    }
    false
}

fn compare_documents_impl(
    original: &[u8],
    modified: &[u8],
    settings: &WmlComparerSettings,
    pre_process_original: bool,
    union_permissions: bool,
) -> Result<Vec<u8>, OpcError> {
    // Admit both packages before anything inflates them: the central
    // directory's sizes are untrusted, and an unbounded inflate aborts the
    // Python and WASM hosts rather than failing the call.
    crate::admission::admit_input("original", original, settings.input_limits)?;
    // IDENTICAL-INPUT-01: same input bytes → empty redline is the (accepted)
    // original package. Avoids dual package prep, Dom parse, LCS, and produce.
    // Critical for self-compare fixtures (e.g. redline × self).
    if original == modified {
        let mut owned = crate::strict_translation::strict_to_transitional_docx_within(
            original,
            settings.input_limits,
        )
        .map_err(crate::opc::refused)?;
        if settings.merge_replaced_paragraphs
            && !settings.preserve_untouched
            && docx_has_tracked_changes(&owned, settings.input_limits)
        {
            owned = accept_source_revisions(&owned, settings.input_limits)?;
        }
        return finalize_identical_compare(&owned, settings);
    }

    // M8: normalize ISO/IEC 29500 "Strict" inputs to "Transitional" before any
    // PartFs::open sees them (mirrors the OpenXML SDK's pre-compare step).
    // Transitional packages round-trip byte-identical (zero-churn), so the
    // golden/parity paths are unaffected; only Strict inputs are rewritten.
    crate::admission::admit_input("modified", modified, settings.input_limits)?;
    let mut original_owned = crate::strict_translation::strict_to_transitional_docx_within(
        original,
        settings.input_limits,
    )
    .map_err(crate::opc::refused)?;
    let mut modified_owned = crate::strict_translation::strict_to_transitional_docx_within(
        modified,
        settings.input_limits,
    )
    .map_err(crate::opc::refused)?;

    // Accept-before-diff flattens a revised insertion into live text, so a late
    // pass can no longer see where that insertion ended. Keep both main parts
    // from this moment. Faithful mode never reads them, nor does an edit:
    // the ladder matches Word Compare's shapes, not the plan's.
    let ladder_sources = if settings.merge_replaced_paragraphs && !settings.preserve_untouched {
        match (
            main_part_xml(&original_owned, settings.input_limits),
            main_part_xml(&modified_owned, settings.input_limits),
        ) {
            (Some(original_xml), Some(revised_xml)) => Some((original_xml, revised_xml)),
            _ => None,
        }
    } else {
        None
    };

    // Word-visual mode + either side already carries track changes: accept both
    // packages first (Word Compare of *finals*). Without this, stamp/re-emit of
    // pre-existing TC drowns real moves and inflates ins/del (broken_ones_two
    // file_8_file_9 37→56, file_27_file_28 38→61 with accept-then). PowerTools
    // faithful leaves inputs as-is. So does an edit (`preserve_untouched`):
    // its plan settled the revisions it resolves, and those left (a style's
    // record) are the source's to keep, untidied (#1885).
    if settings.merge_replaced_paragraphs
        && !settings.preserve_untouched
        && (docx_has_tracked_changes(&original_owned, settings.input_limits)
            || docx_has_tracked_changes(&modified_owned, settings.input_limits))
    {
        original_owned = accept_source_revisions(&original_owned, settings.input_limits)?;
        modified_owned = accept_source_revisions(&modified_owned, settings.input_limits)?;
    }

    // After prep, packages may still be byte-identical (rare non-self paths).
    if original_owned == modified_owned {
        return finalize_identical_compare(&original_owned, settings);
    }

    let original: &[u8] = &original_owned;
    let modified: &[u8] = &modified_owned;

    let mut pkg1 = PartFs::open_within(original, settings.input_limits)?;
    let mut pkg2 = PartFs::open_within(modified, settings.input_limits)?;

    // The notes parts the original has before preprocessing adds empty ones.
    let original_notes = {
        let (footnotes, endnotes) = notes_part_names(&pkg1);
        (
            pkg1.part_bytes(&footnotes).is_some(),
            pkg1.part_bytes(&endnotes).is_some(),
        )
    };

    // CompareInternal :154–:155 — disjoint footnote/endnote id spaces: doc A
    // gets starting_id+1000, doc B +2000 (block hashing ignores ref ids, so
    // correlation is unaffected; the disjoint spaces make reference-driven
    // note pairing sound).
    let changed1 = if pre_process_original {
        pre_process(
            &mut pkg1,
            settings.starting_id_for_footnotes_endnotes + 1000,
            settings.preserve_untouched,
        )?
    } else {
        Vec::new()
    };
    pre_process(
        &mut pkg2,
        settings.starting_id_for_footnotes_endnotes + 2000,
        settings.preserve_untouched,
    )?;

    let main1 = pkg1
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let main2 = pkg2
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());

    let xml1 = pkg1.part_string(&main1).ok_or_else(|| {
        OpcError::PartNotFound(format!("original main document missing: {main1}"))
    })?;
    let xml2 = pkg2.part_string(&main2).ok_or_else(|| {
        OpcError::PartNotFound(format!("modified main document missing: {main2}"))
    })?;
    // Strict/ISO OOXML uses purl.oclc.org namespace URIs; normalize to Transitional
    // (the only variant our XName tables model) so the body/markup is recognized.
    let xml1 = normalize_strict_namespaces(&xml1);
    let xml2 = normalize_strict_namespaces(&xml2);

    // Parse both into one arena so the comparer can work across them.
    let mut dom = Dom::new();
    let d1 = dom.parse_xdocument(&xml1);
    let d2 = dom.parse_xdocument(&xml2);
    let root1 = dom
        .root(d1)
        .ok_or_else(|| OpcError::PartNotFound("original has no root element".into()))?;
    let root2 = dom
        .root(d2)
        .ok_or_else(|| OpcError::PartNotFound("modified has no root element".into()))?;
    let body1 = merged_body(&mut dom, root1)
        .ok_or_else(|| OpcError::PartNotFound("original has no w:body".into()))?;
    let body2 = merged_body(&mut dom, root2)
        .ok_or_else(|| OpcError::PartNotFound("modified has no w:body".into()))?;
    refuse_unatomizable(&dom, root1, "original document")?;
    refuse_unatomizable(&dom, root2, "modified document")?;

    // #531: canonicalize measure spellings on BOTH inputs before the diff —
    // here, where Strict is translated. The diff's property signatures
    // compare raw attribute strings, so "2in" vs "2880" recorded a
    // formatting revision whose old and new sides serialized identically
    // (the output sweep alone normalized only what the record showed).
    // Plain integers — every Transitional spelling — are untouched.
    for root in [root1, root2] {
        crate::comparer::finalize::normalize_universal_measures(&mut dom, root);
    }

    // B.4 — reference-driven notes processing: parse both documents' notes
    // parts AND an independent copy of the original's parts (the withRevisions
    // parts — C# gets them from the wmlResult clone of preprocessed source1)
    // into the same Dom as the bodies. The pipeline (B.2/B.3) diffs each
    // definition by its reference's correlation status and rebuilds the
    // withRevisions parts renumbered 1..n.
    fn parse_part_root(dom: &mut Dom, pkg: &PartFs, name: &str) -> Option<NodeId> {
        let xml = pkg.part_string(name)?;
        let d = dom.parse_xdocument(&xml);
        dom.root(d)
    }
    let (fn1, en1) = notes_part_names(&pkg1);
    let (fn2, en2) = notes_part_names(&pkg2);
    let mut notes_ctx = crate::comparer::NotesContext {
        fn_before: parse_part_root(&mut dom, &pkg1, &fn1),
        fn_after: parse_part_root(&mut dom, &pkg2, &fn2),
        en_before: parse_part_root(&mut dom, &pkg1, &en1),
        en_after: parse_part_root(&mut dom, &pkg2, &en2),
        fn_with_revisions: parse_part_root(&mut dom, &pkg1, &fn1),
        en_with_revisions: parse_part_root(&mut dom, &pkg1, &en1),
    };
    // Same canonicalization for the notes parts the pipeline diffs: a
    // spelling-only footnote geometry pair is not a formatting change.
    for root in [
        notes_ctx.fn_before,
        notes_ctx.fn_after,
        notes_ctx.en_before,
        notes_ctx.en_after,
        notes_ctx.fn_with_revisions,
        notes_ctx.en_with_revisions,
    ]
    .into_iter()
    .flatten()
    {
        crate::comparer::finalize::normalize_universal_measures(&mut dom, root);
    }
    for (root, part) in [
        (notes_ctx.fn_before, &fn1),
        (notes_ctx.fn_after, &fn2),
        (notes_ctx.en_before, &en1),
        (notes_ctx.en_after, &en2),
    ] {
        if let Some(root) = root {
            refuse_unatomizable(&dom, root, part)?;
        }
    }
    for (side, main, notes) in [
        (
            "original document",
            root1,
            [notes_ctx.fn_before, notes_ctx.en_before],
        ),
        (
            "modified document",
            root2,
            [notes_ctx.fn_after, notes_ctx.en_after],
        ),
    ] {
        let units = notes
            .into_iter()
            .flatten()
            .fold(story_units(&dom, main), |sum, root| {
                sum.saturating_add(story_units(&dom, root))
            });
        refuse_past_story_budget(units, side, settings.input_limits)?;
    }

    // Word mode: each side's unstyled-paragraph line pitch, for the
    // demo-default spacing strip.
    if settings.merge_replaced_paragraphs {
        for (pkg, root) in [(&pkg1, root1), (&pkg2, root2)] {
            let line = pkg
                .part_string("word/styles.xml")
                .map_or_else(|| "240".to_string(), |x| default_paragraph_line(&x));
            dom.set_attribute_value(root, &crate::namespaces::PT::default_line(), Some(&line));
            if pkg
                .part_string("word/styles.xml")
                .is_some_and(|x| has_default_table_style(&x))
            {
                dom.set_attribute_value(
                    root,
                    &crate::namespaces::PT::has_default_table_style(),
                    Some("1"),
                );
            }
        }
    }
    let result_root = crate::comparer::try_compare_bodies_faithful_with_notes(
        &mut dom,
        root1,
        root2,
        body1,
        body2,
        settings,
        Some(&mut notes_ctx),
    )
    .map_err(crate::opc::refused)?;
    // A break the revision adds is an inserted paragraph mark. The body
    // comparison still leaves that break live, so Reject All would keep it.
    track_added_section_breaks(&mut dom, result_root, body1, body2, settings);
    for root in [root1, root2] {
        dom.set_attribute_value(root, &crate::namespaces::PT::default_line(), None);
        dom.set_attribute_value(
            root,
            &crate::namespaces::PT::has_default_table_style(),
            None,
        );
    }

    // Base the output on the original package, replacing the main document
    // part. When PreProcessMarkup rewrote parts (notes renumbering), base it
    // on the PREPROCESSED original instead — C# builds wmlResult from the
    // preprocessed source1 (:197 comment: the renumbered/unid'd markup must be
    // the one appearing in the result). Note-free inputs keep the raw bytes.
    let mut out = if changed1.is_empty() {
        PartFs::open_within(original, settings.input_limits)?
    } else {
        PartFs::open_within(&pkg1.to_zip()?, settings.input_limits)?
    };
    // M4.H.3: carry over / drop dangling relationship references from inserted
    // content so the output has no dangling rId (Word-repair preventer).
    crate::comparer::parts::reconcile_dangling_relationships(
        &mut dom,
        result_root,
        &mut out,
        &[&pkg1, &pkg2],
    );
    // Word-alignment mode: adopt the revised document's headers/footers when
    // the original supplied none (must run BEFORE the result serialization —
    // it adds references to the final sectPr).
    if settings.merge_replaced_paragraphs {
        adopt_revised_header_footer(&mut dom, result_root, &pkg2, &mut out, &main1, settings);
        // Word inheritance: drop body-final HF slots already set on an earlier
        // mid-section break (dual chrome otherwise). Mid multi-section copies stay.
        strip_final_sectpr_inherited_header_footer(
            &mut dom,
            result_root,
            &final_header_footer_slots(&pkg2),
        );
    }
    if needs_unmerged_section_story_cleanup(settings) {
        strip_final_a_only_deleted_break_stories(&mut dom, result_root, &out, &pkg2, &main1);
    }
    // A-only stories on deleted breaks or solely on the final section must
    // disappear on Accept All, including when paragraph merging is disabled.
    let mut dead_only = parts_only_deleted_breaks_reference(&dom, result_root, &out, &main1);
    if needs_unmerged_section_story_cleanup(settings) {
        let a_only = a_only_header_footer_parts(&pkg1, &pkg2);
        dead_only.retain(|part| a_only.contains(part));
    }
    mark_a_only_hf_content_as_deleted(&mut out, &pkg1, &pkg2, &dead_only, settings);
    move_page_story_to_final_section(&mut dom, result_root, &out, &main1, &dead_only);

    // M35: comments carryover is a package-validity invariant, not a
    // Word-visual formatting pass. Both supported comparer presets must union
    // comment parts and re-inject their anchors; otherwise PowerTools-faithful
    // output can retain an original comment definition after its source
    // paragraph becomes a deletion while silently losing the anchor triplet.
    let has_comments = pkg1.part_string("word/comments.xml").is_some()
        || pkg2.part_string("word/comments.xml").is_some();
    crate::comparer::comments::carry_comments(
        &mut dom,
        result_root,
        (&pkg1, &main1),
        (&pkg2, &main2),
        (&mut out, &main1),
        &settings.author_for_revisions,
    );
    if has_comments {
        // Comment anchors keep source ids (aligned with comments.xml). Re-run
        // revision renumber with those ids reserved so move/tblPrChange never
        // share an id with commentRange* (Word "unreadable content").
        crate::comparer::finalize::fix_up_revision_ids(&mut dom, &[result_root]);
    }

    // M472 — re-assert Word's ins-before-del replacement order after the
    // comment-carry rebuilds run blocks (diff_before10 × diff_before11: the
    // early reorder's work was undone by comment anchoring, leaving
    // [ins][del][ins] where Word emits ins…ins del). Idempotent and
    // text-preserving.
    crate::comparer::finalize::reorder_replacements_ins_before_del(&mut dom, result_root);

    // Word mode: the deleted rows of a paired table whose style or layout
    // changed become tables of their own, as in Word's redlines (#1817).
    // Before the pass below, which reads the deleted tables.
    if settings.merge_replaced_paragraphs {
        use crate::comparer::table_split::{TableStyleNames, split_rows_by_table_identity};
        split_rows_by_table_identity(
            &mut dom,
            result_root,
            &TableStyleNames::original(pkg1.part_string("word/styles.xml").as_deref()),
            &TableStyleNames::revision(pkg2.part_string("word/styles.xml").as_deref()),
        );
    }

    // Inserted words a deleted paragraph mark joins across a deleted table
    // start the paragraph they join (#1851): Word 16.115 crashes opening
    // some such redlines. After the comment carry and M472, which rebuild
    // runs; the bookmarks below are carried around the moved runs.
    crate::comparer::deleted_tables::move_inserted_text_past_deleted_tables(&mut dom, result_root);

    // M463 — rewrite outer-wrapped math revisions into Word's internal form.
    // Must run after every mesh/finalize pass (they reason about the outer
    // wrap) and before serialize.
    crate::comparer::produce::convert_outer_math_wraps_to_internal(&mut dom, result_root, settings);

    // Word keeps both documents' body bookmarks (union by name); without
    // them every updated TOC/REF field prints "Error! Bookmark not defined."
    // Runs on the finished body so no later pass moves the markers.
    crate::comparer::bookmarks::carry_bookmarks(
        &mut dom,
        result_root,
        (&pkg1, &main1),
        (&pkg2, &main2),
        &settings.author_for_revisions,
    );
    if union_permissions {
        crate::comparer::bookmarks::carry_permission_union(
            &mut dom,
            result_root,
            (&pkg1, &main1),
            (&pkg2, &main2),
            &settings.author_for_revisions,
        );
    }

    // Final drawing/shape id renumber immediately before serialize — package
    // post-steps (reconcile, header/footer adopt, comments) can clone/graft
    // drawings after the mid-produce FixUpDocPrIds pass (S-dup-docpr-id).
    crate::comparer::fixups::fix_up_drawing_ids(&mut dom, result_root, settings.preserve_untouched);

    let result_xml = dom.serialize_element(result_root);
    out.set_part(&main1, result_xml.into_bytes());

    // M4.H.8/H.9: copy styles/numbering referenced by inserted (modified) content
    // into the output so it stays Word-valid.
    // Word-mode also: adopt missing docDefaults/latentStyles from B, then
    // canonicalize styleIds (numeric/`styleN` → Heading1/…) and remap refs.
    let mut style_renames: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    // POSTSTEP-STYLES-CACHE-01: the style-validity passes each re-parse the
    // (often large) output stylesheet — 3-4× per compare dominates medium-doc
    // poststeps. Capture the defined style-id set here, after styles-copy has
    // finished mutating `word/styles.xml` (copy-missing + canonicalize), so the
    // pStyle/rStyle strip below reuses it instead of re-parsing. Nothing between
    // here and that strip changes the style-id SET (the main-doc / spacing
    // passes touch the body, not styles.xml), so the cached set is exact.
    let mut cached_defined_styles: Option<std::collections::HashSet<String>> = None;
    // POSTSTEP-STYLES-CACHE-02: keep the parsed styles arena (out stylesheet
    // `tr`, revised stylesheet `fr`) alive so the M-PAG Normal-merge below can
    // reuse it instead of re-parsing both stylesheets. styles-copy still
    // serializes the stylesheet (so a compare that skips M-PAG is unaffected);
    // M-PAG then mutates the same in-memory `tr` and re-serializes. serialize→
    // parse is a lossless structural round-trip, so `tr` is identical to what
    // M-PAG would have re-parsed.
    let mut styles_arena: Option<(Dom, NodeId, NodeId)> = None;
    for (part, is_styles) in [("word/styles.xml", true), ("word/numbering.xml", false)] {
        match (out.part_string(part), pkg2.part_string(part)) {
            (Some(to_xml), Some(from_xml)) => {
                let mut sd = Dom::new();
                let td = sd.parse_xdocument(&to_xml);
                let fd = sd.parse_xdocument(&from_xml);
                if let (Some(tr), Some(fr)) = (sd.root(td), sd.root(fd)) {
                    if is_styles {
                        // An edit's revised stylesheet is the source's plus
                        // what the plan added, under the same ids: copy the
                        // additions, rename nothing.
                        if settings.merge_replaced_paragraphs && !settings.preserve_untouched {
                            let a_ids = crate::comparer::footnotes::defined_style_ids(&sd, tr);
                            materialize_builtin_paragraph_defaults(&mut sd, tr);
                            materialize_builtin_paragraph_defaults(&mut sd, fr);
                            let b_to_out = copy_missing_styles_by_name(&mut sd, tr, fr);
                            let _ = adopt_missing_styles_structure(&mut sd, tr, fr);
                            style_renames = canonicalize_style_ids(&mut sd, tr);
                            // B's content names paired styles by B's ids; send
                            // them to the output style's final id. An id A
                            // also declares names A's own style: leave it. The
                            // copied B styles link to them by B's ids too.
                            let mut b_renames = std::collections::HashMap::new();
                            for (b_id, out_id) in b_to_out {
                                let target = style_renames.get(&out_id).cloned().unwrap_or(out_id);
                                if b_id != target && !a_ids.contains(&b_id) {
                                    b_renames.insert(b_id, target);
                                }
                            }
                            for local in ["basedOn", "next", "link"] {
                                for e in sd.descendants(tr, Some(&W::name(local))) {
                                    if let Some(to) =
                                        sd.attribute(e, &W::val()).and_then(|v| b_renames.get(v))
                                    {
                                        let to = to.clone();
                                        sd.set_attribute_value(e, &W::val(), Some(&to));
                                    }
                                }
                            }
                            style_renames.extend(b_renames);
                        } else {
                            crate::comparer::footnotes::copy_missing_styles(&mut sd, tr, fr);
                        }
                    } else {
                        let (num_remap, copied_bullets) =
                            crate::comparer::footnotes::copy_missing_numbering(&mut sd, tr, fr);
                        // B's picture bullets draw through B's numbering rels;
                        // give each its own relationships, since the same ids
                        // may already name A's images here.
                        for bullet in copied_bullets {
                            for el in sd.descendants_and_self(bullet, None) {
                                for (an, rid) in sd.attributes(el) {
                                    if an.namespace_name() != crate::namespaces::R::URI {
                                        continue;
                                    }
                                    let carried = crate::comparer::parts::carry_relationship(
                                        &mut out,
                                        part,
                                        &pkg2,
                                        part,
                                        &rid,
                                        |_| true,
                                    );
                                    sd.set_attribute_value(el, &an, carried.as_deref());
                                }
                            }
                        }
                        // M482: colliding B numIds were renumbered in the
                        // merged numbering part — rewrite the refs inside
                        // B-INSERTED paragraphs (mark rPr carries w:ins) or
                        // they resolve against A's same-id definitions
                        // (bullets where B's decimals should render).
                        // Word mode also moves unchanged paragraphs to B's
                        // list and records A's numId in a pPrChange: the
                        // circle bullet struck, the disc inserted (native
                        // bullet circle × disc).
                        if !num_remap.is_empty()
                            && let Some(doc_xml) = out.part_string(&main1)
                        {
                            let mut pd = Dom::new();
                            let dd = pd.parse_xdocument(&doc_xml);
                            if let Some(droot) = pd.root(dd) {
                                let mut changed = false;
                                // Comment anchors and carried bookmarks keep their
                                // ids; stay clear of both (Word never shares one).
                                let mut next_id = [
                                    "commentRangeStart",
                                    "commentRangeEnd",
                                    "commentReference",
                                    "bookmarkStart",
                                    "bookmarkEnd",
                                ]
                                .into_iter()
                                .flat_map(|n| pd.descendants(droot, Some(&W::name(n))))
                                .filter_map(|c| pd.attribute(c, &W::id())?.parse::<u32>().ok())
                                .map(|n| n + 1)
                                .fold(next_free_revision_id(&pd, droot), u32::max);
                                for p in pd.descendants(droot, Some(&W::p())) {
                                    let Some(ppr) = pd.element(p, &W::p_pr()) else {
                                        continue;
                                    };
                                    let mark = pd.element(ppr, &W::r_pr());
                                    let mark_has = |n: &str| {
                                        mark.is_some_and(|r| pd.element(r, &W::name(n)).is_some())
                                    };
                                    let mark_inserted = mark_has("ins");
                                    let unchanged = settings.merge_replaced_paragraphs
                                        && !mark_inserted
                                        && !mark_has("del");
                                    if !mark_inserted && !unchanged {
                                        continue;
                                    }
                                    let Some(nid) = pd
                                        .element(ppr, &W::num_pr())
                                        .and_then(|np| pd.element(np, &W::name("numId")))
                                    else {
                                        continue;
                                    };
                                    let cur = pd.attribute(nid, &W::val()).map(str::to_string);
                                    let Some(new_id) =
                                        cur.as_deref().and_then(|c| num_remap.get(c)).cloned()
                                    else {
                                        continue;
                                    };
                                    if unchanged && pd.element(ppr, &W::p_pr_change()).is_none() {
                                        let old_ppr = pd.new_element(W::p_pr());
                                        for c in pd.elements(ppr, None) {
                                            if pd.name(c).is_some_and(|n| {
                                                n == W::r_pr() || n == W::sect_pr()
                                            }) {
                                                continue;
                                            }
                                            let clone = pd.clone_subtree(c);
                                            pd.add(old_ppr, clone);
                                        }
                                        let chg = pd.new_element(W::p_pr_change());
                                        pd.set_attribute_value(
                                            chg,
                                            &W::id(),
                                            Some(&next_id.to_string()),
                                        );
                                        next_id += 1;
                                        pd.set_attribute_value(
                                            chg,
                                            &W::author(),
                                            Some(&settings.author_for_revisions),
                                        );
                                        pd.set_attribute_value(
                                            chg,
                                            &W::date(),
                                            Some(&settings.date_time_for_revisions),
                                        );
                                        pd.add(chg, old_ppr);
                                        pd.add(ppr, chg);
                                    }
                                    pd.set_attribute_value(nid, &W::val(), Some(&new_id));
                                    changed = true;
                                }
                                if changed {
                                    out.set_part(&main1, pd.serialize_element(droot).into_bytes());
                                }
                            }
                        }
                    }
                    if is_styles {
                        cached_defined_styles =
                            Some(crate::comparer::footnotes::defined_style_ids(&sd, tr));
                    }
                    out.set_part(part, sd.serialize_element(tr).into_bytes());
                    if is_styles {
                        // Hand the parsed arena (out `tr`, revised `fr`) to M-PAG.
                        styles_arena = Some((sd, tr, fr));
                    }
                }
            }
            // A has no numbering part, B does (file_21×file_22): copy B's
            // numbering wholesale + document rel. Prior match required both
            // sides present → insert-heavy next with lists lost numbering.
            // An edit takes the stylesheet its plan created the same way.
            (None, Some(from_xml)) if !is_styles || settings.preserve_untouched => {
                let (ct_type, rel_type, rel_suffix, target) = if is_styles {
                    (
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                        "/styles",
                        "styles.xml",
                    )
                } else {
                    (
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
                        "/numbering",
                        "numbering.xml",
                    )
                };
                out.set_part(part, from_xml.into_bytes());
                out.add_content_type_override(&format!("/{part}"), ct_type);
                let has_rel = out
                    .read_rels_for(&main1)
                    .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with(rel_suffix)));
                if !has_rel {
                    out.add_document_relationship(&main1, rel_type, target);
                }
            }
            // A has no styles part, B does: copy B, canonicalize, then M462 —
            // swap in Word's FACTORY docDefaults and bake B-effective metrics
            // into each style (Word scaffolds from its blank document, not
            // from B: tiff_image × two_column oracle). A's referenced theme
            // stays; only a missing one becomes Word's default, in
            // ensure_factory_package_chrome.
            (None, Some(from_xml)) if is_styles && settings.merge_replaced_paragraphs => {
                let mut sd = Dom::new();
                let fd = sd.parse_xdocument(&from_xml);
                if let Some(fr) = sd.root(fd) {
                    style_renames = canonicalize_style_ids(&mut sd, fr);
                    factory_scaffold_bake_b_styles(&mut sd, fr, settings);
                    out.set_part(part, sd.serialize_element(fr).into_bytes());
                    out.add_content_type_override(
                        "/word/styles.xml",
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                    );
                    let has_rel = out
                        .read_rels_for(&main1)
                        .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/styles")));
                    if !has_rel {
                        out.add_document_relationship(
                            &main1,
                            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                            "styles.xml",
                        );
                    }
                }
            }
            // A has styles, B has none: still canonicalize A's ids (Word does).
            (Some(to_xml), None)
                if is_styles
                    && settings.merge_replaced_paragraphs
                    && !settings.preserve_untouched =>
            {
                let mut sd = Dom::new();
                let td = sd.parse_xdocument(&to_xml);
                if let Some(tr) = sd.root(td) {
                    style_renames = canonicalize_style_ids(&mut sd, tr);
                    out.set_part(part, sd.serialize_element(tr).into_bytes());
                }
            }
            _ => {}
        }
    }
    // Remap pStyle/rStyle/tblStyle in every XML part that can carry them.
    if settings.merge_replaced_paragraphs && !style_renames.is_empty() {
        let part_names: Vec<String> = out
            .parts()
            .into_iter()
            .filter(|p| {
                p.ends_with(".xml")
                    && (p.starts_with("word/document")
                        || p.starts_with("word/header")
                        || p.starts_with("word/footer")
                        || p.starts_with("word/footnotes")
                        || p.starts_with("word/endnotes")
                        || p.starts_with("word/comments")
                        || p == "word/styles.xml")
            })
            .collect();
        for part in part_names {
            let Some(xml) = out.part_string(&part) else {
                continue;
            };
            let mut pd = Dom::new();
            let doc = pd.parse_xdocument(&xml);
            let Some(root) = pd.root(doc) else {
                continue;
            };
            // styles.xml basedOn/next/link already remapped inside canonicalize.
            if part == "word/styles.xml" {
                continue;
            }
            if remap_style_refs(&mut pd, root, &style_renames) > 0 {
                out.set_part(&part, pd.serialize_element(root).into_bytes());
            }
        }
    }

    // The revised text brings its fonts: their table entries come too.
    if let (Some(out_fonts), Some(b_fonts)) = (
        out.part_string("word/fontTable.xml"),
        pkg2.part_string("word/fontTable.xml"),
    ) && let Some(merged) = merge_revised_font_table(&out_fonts, &b_fonts)
    {
        out.set_part("word/fontTable.xml", merged.into_bytes());
    }
    // Word-mode: adopt B's package chrome (settings/fontTable/theme) when A is
    // thin. When BOTH sides are bare demos (C5 formatting one-pagers), Word
    // still saves factory settings/theme/fontTable — inject if still missing.
    if settings.merge_replaced_paragraphs {
        // An edit's source keeps the parts it has: no chrome comes from
        // Word's blank document.
        if !settings.preserve_untouched {
            adopt_revised_styles_chrome(&mut out, &pkg2, &main1);
            ensure_factory_package_chrome(&mut out, &main1);
        }
        repair_missing_core_relationships(&mut out, &main1);
        merge_custom_properties(&mut out, &pkg2);
        // M492: deleted paragraphs keep their A-ORIGINAL direct spacing.
        if let (Some(a_xml), Some(out_xml)) = (
            pkg1.part_string("word/document.xml"),
            out.part_string(&main1),
        ) && let Some(restored) = restore_deleted_paragraph_spacing(&a_xml, &out_xml)
        {
            out.set_part(&main1, restored.into_bytes());
        }
        // M491: B's DOCUMENT-FINAL paragraph mark never materializes as a
        // mid-document insertion — Word EQ-pairs the two documents' final
        // marks. Our windowed diff can splice B's terminal empty ¶ into the
        // body as an ins-para; the extra rendered line cascades every page
        // break below it (h_f_normal × sd_1495: paraId 5352EB15, 46.66 with
        // its content-ceiling at 97.13; 16-pair census). Remove the
        // ins-marked EMPTY paragraph carrying B's final paraId unless it is
        // itself the output's final paragraph.
        if let (Some(b_xml), Some(out_xml)) = (
            pkg2.part_string("word/document.xml"),
            out.part_string(&main1),
        ) {
            let b_final_pid = {
                let mut bd = Dom::new();
                let d = bd.parse_xdocument(&b_xml);
                bd.root(d)
                    .and_then(|r| bd.element(r, &W::body()))
                    .and_then(|body| {
                        let bdom = bd;
                        let paras: Vec<NodeId> = bdom
                            .elements(body, None)
                            .into_iter()
                            .filter(|&k| bdom.name_is(k, &W::p()))
                            .collect();
                        paras.last().and_then(|&p| {
                            bdom.attribute(p, &crate::namespaces::W14::name("paraId"))
                                .map(str::to_string)
                        })
                    })
            };
            if let Some(bf) = b_final_pid {
                let mut pd = Dom::new();
                let d = pd.parse_xdocument(&out_xml);
                if let Some(root) = pd.root(d)
                    && let Some(body) = pd.element(root, &W::body())
                {
                    let paras: Vec<NodeId> = pd
                        .elements(body, None)
                        .into_iter()
                        .filter(|&k| pd.name_is(k, &W::p()))
                        .collect();
                    let w14_pid = crate::namespaces::W14::name("paraId");
                    let target = paras.iter().enumerate().find(|&(i, &p)| {
                        i + 1 != paras.len() && pd.attribute(p, &w14_pid) == Some(bf.as_str())
                    });
                    if let Some((idx, &p)) = target {
                        let is_ins_empty = |pd: &Dom, q: NodeId| {
                            let mark_ins = pd
                                .element(q, &W::p_pr())
                                .and_then(|pr| pd.element(pr, &W::r_pr()))
                                .is_some_and(|r| pd.element(r, &W::ins()).is_some());
                            // A paragraph whose only content is a real embedded
                            // object (DrawingML image/shape, OLE object, a VML
                            // picture, a text box, or an AlternateContent shape)
                            // carries no w:t, but it is NOT empty — removing it
                            // drops the graphic (vrect_node × wmf_emf: B's final ¶
                            // is a full-page WMF, misjudged empty → whole page lost,
                            // 18.21). A BARE w:pict horizontal rule (o:hr `v:rect`,
                            // no imagedata/textbox) is decorative, not content, and
                            // must stay droppable — checking for the real content
                            // markers (not the w:pict wrapper) avoids regressing the
                            // trailing-hr-spacer pairs (sd_2517 hr rules).
                            let has_embedded = [
                                W::drawing(),
                                W::object(),
                                crate::namespaces::MC::name("AlternateContent"),
                                crate::namespaces::VML::name("imagedata"),
                                W::txbx_content(),
                                crate::namespaces::WNE::name("txbxContent"),
                            ]
                            .iter()
                            .any(|nm| !pd.descendants(q, Some(nm)).is_empty());
                            mark_ins
                                && !has_embedded
                                && !pd
                                    .descendants(q, Some(&W::t()))
                                    .iter()
                                    .any(|&t| !pd.value_str(t).trim().is_empty())
                                && pd.descendants(q, Some(&W::del_text())).is_empty()
                        };
                        // Run gate: Word trims B's final ¶ out of a RUN of
                        // inserted empties (h_f: three in a row, two kept);
                        // a LONE inserted empty spacer stays — Word keeps it
                        // under another identity (file_82 × file_83, M389:
                        // stripping it cost −18.9).
                        let prev_also_empty = idx > 0 && is_ins_empty(&pd, paras[idx - 1]);
                        if is_ins_empty(&pd, p) && prev_also_empty {
                            pd.remove(p);
                            out.set_part(&main1, pd.serialize_element(root).into_bytes());
                        }
                    }
                }
            }
        }
        // #1678: a table that replaced the body's last paragraph keeps the
        // revised document's empty close. After M491, which only removes a
        // misplaced copy of that paragraph.
        if let (Some(revised_xml), Some(produced_xml)) =
            (pkg2.part_string(&main2), out.part_string(&main1))
            && let Some(kept) =
                crate::comparer::finalize::keep_revised_empty_close(&produced_xml, &revised_xml)
        {
            out.set_part(&main1, kept.into_bytes());
        }
    }

    // Word-parity: strip pStyle/rStyle that styles.xml does not define. LO maps
    // built-in names (Heading1, Title, …) even when the style entry is absent;
    // Word's redline omits the attribute entirely (heading_1_bold×heading_1_style).
    // An edit keeps the source's references as they are.
    if settings.merge_replaced_paragraphs && !settings.preserve_untouched {
        // POSTSTEP-STYLES-CACHE-01: reuse the defined-style-id set captured after
        // styles-copy instead of re-parsing word/styles.xml. Fall back to a fresh
        // parse only when styles-copy did not run (no output stylesheet).
        let defined = cached_defined_styles.clone().or_else(|| {
            out.part_string("word/styles.xml").map(|sx| {
                let mut sd = Dom::new();
                let doc = sd.parse_xdocument(&sx);
                sd.root(doc)
                    .map(|r| crate::comparer::footnotes::defined_style_ids(&sd, r))
                    .unwrap_or_default()
            })
        });
        if let Some(defined) = defined {
            // Main document + common related parts that can carry pStyle/rStyle.
            let mut part_names: Vec<String> = vec![main1.clone()];
            for p in out.parts() {
                let is_style_carrier = p.starts_with("word/header")
                    || p.starts_with("word/footer")
                    || p == "word/footnotes.xml"
                    || p == "word/endnotes.xml"
                    || p == "word/comments.xml"
                    || p.starts_with("word/comments");
                if is_style_carrier {
                    part_names.push(p);
                }
            }
            part_names.sort();
            part_names.dedup();
            for part in part_names {
                let Some(xml) = out.part_string(&part) else {
                    continue;
                };
                let mut pd = Dom::new();
                let doc = pd.parse_xdocument(&xml);
                let Some(root) = pd.root(doc) else {
                    continue;
                };
                let n = crate::comparer::footnotes::strip_unresolved_style_refs(
                    &mut pd, root, &defined,
                );
                if n > 0 {
                    out.set_part(&part, pd.serialize_element(root).into_bytes());
                }
            }
        }
    }

    // M-PAG mechanism 2 (word mode): merged Normal style. Word's redline
    // stylesheet carries the REVISED document's effective Normal spacing with
    // a w:pPrChange recording the old value; ours (A-based) kept A's Normal
    // verbatim, diffusing ~1 page of drift per ~20 (sd-2517_sectpr-headerref:
    // 111 → 117 pages with this patch, GT 116). Provenance: when B's Normal
    // pPr is empty/absent, Word resolves it to FACTORY defaults
    // (after=160 line=278 lineRule=auto — matches neither side's docDefaults).
    // An edit's Normal is the source's.
    if settings.merge_replaced_paragraphs && !settings.preserve_untouched {
        // POSTSTEP-STYLES-CACHE-02: reuse the arena styles-copy already parsed;
        // fall back to a fresh parse of both stylesheets only when it did not run.
        let arena = styles_arena.take().or_else(|| {
            match (
                out.part_string("word/styles.xml"),
                pkg2.part_string("word/styles.xml"),
            ) {
                (Some(out_xml), Some(b_xml)) => {
                    let mut sd = Dom::new();
                    let od = sd.parse_xdocument(&out_xml);
                    let bd = sd.parse_xdocument(&b_xml);
                    match (sd.root(od), sd.root(bd)) {
                        (Some(or), Some(br)) => Some((sd, or, br)),
                        _ => None,
                    }
                }
                _ => None,
            }
        });
        if let Some((mut sd, or, br)) = arena {
            // Workstream S runs FIRST, while the output stylesheet still holds
            // A's definitions: `merge_normal_style_*` below rewrites Normal to
            // B's values, and every style based on Normal would then resolve its
            // "original" chain against the already-revised Normal.
            // S2: the styles A actually declared, by (type, name) — styles in
            // `or` beyond this set were copied from B and take the phase-2
            // docDefaults bake.
            let a_declared_keys: std::collections::HashSet<(String, String)> = pkg1
                .part_string("word/styles.xml")
                .map(|xml| {
                    let mut ad = Dom::new();
                    let doc = ad.parse_xdocument(&xml);
                    ad.root(doc)
                        .map(|r| {
                            ad.elements(r, Some(&W::name("style")))
                                .into_iter()
                                .filter_map(|s| style_match_key(&ad, s))
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            // Word leaves a style the revision's stylesheet lacks as the
            // original defines it, without a change record (51 of 51 accept
            // pairs), whatever Normal and the docDefaults become under it:
            // the passes below only see it to be put back.
            let a_styles = pkg1.part_string("word/styles.xml").and_then(|xml| {
                let doc = sd.parse_xdocument(&xml);
                sd.root(doc)
            });
            let inherited_only = a_styles.map_or_else(std::collections::HashSet::new, |a_root| {
                inherited_only_style_changes(&sd, or, a_root, br)
            });
            let only_a = styles_the_revision_lacks(&mut sd, or, br);
            let mut changed = merge_revised_style_definitions(
                &mut sd,
                or,
                br,
                settings,
                &a_declared_keys,
                &inherited_only,
            );
            changed |= merge_normal_style_spacing(&mut sd, or, br, settings);
            // M-PAG mechanism 2b / M71: rewrite Normal rPr to B's effective
            // metrics when they differ. Formerly gated on header/footer→Normal
            // (footer knife-edge). That skipped file_197 (no HF): Word writes
            // B's Calibri dd onto Normal + rPrChange(A Ubuntu); we kept A.
            // M65 still skips both-bare (file_170). HF-linked cases unchanged.
            changed |= merge_normal_style_rpr(&mut sd, or, br, settings);
            // M467: Word keeps only per-attr DELTAS vs the output docDefaults
            // on the merged Normal (tab_test × table_autofit: kern/szCs/
            // eastAsiaTheme/ligatures dropped as redundant, Arial+sz=20 kept).
            changed |= prune_normal_rpr_context_equal_attrs(&mut sd, or);
            // M480b: dd-delta disabling neutralizers on both-sides merged
            // styles — after the Normal merges (so a stamped Normal reads as
            // a provider), before M111 adds cascade records.
            changed |= bake_bothsides_dd_disabling_neutralizers(&mut sd, or, br);
            // M111: cascade Normal pPrChange/rPrChange onto basedOn=Normal styles
            // (ListParagraph/BodyText/Header/… — file_130 Word has ~30).
            changed |= cascade_normal_change_to_based_styles(&mut sd, or, settings);
            // M79: Word single-line on Heading/Title/ListParagraph (file_33 3→2pp).
            changed |= normalize_word_paragraph_style_line(&mut sd, or);
            // M80: Title/ListParagraph/Highlighted Arial + Heading Latin inherit.
            changed |= align_paragraph_style_fonts_with_normal(&mut sd, or);
            // Redefined styles take B's effective metrics as a delta against
            // the output context (Word's rule, mined over 4,924 styles). Runs
            // last so it settles what the heuristic passes above wrote.
            changed |= resolve_redefined_style_metrics(&mut sd, or, br);
            // Root styles record their old properties in full against the
            // docDefaults, the way Word's Reject All needs them.
            changed |= complete_root_style_change_records(&mut sd, or, settings);
            // Below the roots, the record carries what the original's chain
            // gave the style, which Reject All would otherwise read as built-in.
            if let Some(a_root) = a_styles {
                changed |= complete_based_style_change_records(&mut sd, or, a_root);
            }
            changed |= remove_inherited_only_style_records(&mut sd, &inherited_only);
            changed |= restore_styles(&mut sd, or, only_a);
            // M483: re-cache themed color hexes against the shipped theme —
            // must run AFTER the merge writes B's blocks (their w:val hexes
            // were cached under B's theme). The shipped theme is the one the
            // main part references, wherever it lives.
            if let Some(theme_xml) =
                referenced_theme_part(&out, &main1).and_then(|p| out.part_string(&p))
            {
                changed |= reresolve_theme_color_hexes(&mut sd, or, &theme_xml);
            }
            if changed {
                out.set_part("word/styles.xml", sd.serialize_element(or).into_bytes());
            }
            // M487 — bake B-effective paragraph spacing onto B-INSERTED
            // paragraphs. The output ships A's docDefaults, so an inserted
            // paragraph styled by B renders with the wrong spacing unless its
            // pPr carries B's effective values directly. Word synthesizes
            // exactly this (rstyle_combos × pre_separated_list oracle: every
            // inserted ListParagraph gains `w:spacing w:after="0" w:line=
            // "240" w:lineRule="auto"` that B's source never declared;
            // content-ceiling for the pair 44.83 → 100.00). The M480
            // neutralizer principle at the document.xml level.
            {
                let index = |dom: &Dom,
                             root: NodeId|
                 -> std::collections::HashMap<String, NodeId> {
                    dom.elements(root, Some(&W::name("style")))
                        .into_iter()
                        .filter_map(|s| Some((dom.attribute(s, &W::style_id())?.to_string(), s)))
                        .collect()
                };
                let out_idx = index(&sd, or);
                let b_idx = index(&sd, br);
                // B's counterpart of an output style id: the style of the same
                // (type, name), as Word pairs them — ids name nothing across
                // documents (b4cd671041's revision calls List Paragraph
                // `Listaszerbekezds` and Table Grid `Rcsostblzat`).
                let mut b_by_key: std::collections::HashMap<(String, String), String> =
                    std::collections::HashMap::new();
                for s in sd.elements(br, Some(&W::name("style"))) {
                    if let (Some(k), Some(id)) =
                        (style_match_key(&sd, s), sd.attribute(s, &W::style_id()))
                    {
                        b_by_key.entry(k).or_insert_with(|| id.to_string());
                    }
                }
                let b_id_for = |out_id: &str| -> Option<String> {
                    out_idx
                        .get(out_id)
                        .and_then(|&s| style_match_key(&sd, s))
                        .and_then(|k| b_by_key.get(&k).cloned())
                        .or_else(|| b_idx.contains_key(out_id).then(|| out_id.to_string()))
                };
                let (b_default, out_default) = {
                    let dp = |dom: &Dom, root: NodeId| -> Option<String> {
                        dom.elements(root, Some(&W::name("style")))
                            .into_iter()
                            .find(|&s| {
                                dom.attribute(s, &W::type_()) == Some("paragraph")
                                    && matches!(
                                        dom.attribute(s, &W::name("default")),
                                        Some("1") | Some("true")
                                    )
                            })
                            .and_then(|s| dom.attribute(s, &W::style_id()).map(str::to_string))
                    };
                    (dp(&sd, br), dp(&sd, or))
                };
                if let Some(doc_xml) = out.part_string(&main1) {
                    let mut pd = Dom::new();
                    let dd = pd.parse_xdocument(&doc_xml);
                    if let Some(droot) = pd.root(dd) {
                        let mut doc_changed = false;
                        for p in pd.descendants(droot, Some(&W::p())) {
                            let Some(ppr) = pd.element(p, &W::p_pr()) else {
                                continue;
                            };
                            let mark_inserted = pd
                                .element(ppr, &W::r_pr())
                                .is_some_and(|r| pd.element(r, &W::ins()).is_some());
                            // A container's last paragraph keeps its mark when
                            // a paragraph is added after it, so the new words
                            // end on the old mark, under a record of its old
                            // properties (#1669): every word inserted, that
                            // paragraph is B's as much as an inserted one, and
                            // Reject All restores the record whatever is baked.
                            let words_inserted = || {
                                let words = pd.descendants(p, Some(&W::t()));
                                !words.is_empty()
                                    && pd.element(ppr, &W::p_pr_change()).is_some()
                                    && words.iter().all(|&t| {
                                        pd.ancestors(t, Some(&W::ins())).first().is_some_and(
                                            |&ins| {
                                                pd.ancestors(ins, Some(&W::p())).first() == Some(&p)
                                            },
                                        )
                                    })
                            };
                            if !mark_inserted && !words_inserted() {
                                continue;
                            }
                            // EMPTY inserted paragraphs keep their bare pPr —
                            // the oracle never bakes spacing onto them, and the
                            // empty line's default height decides the page
                            // break (rstyle_combos: baking onto the one empty
                            // separator para cost the whole second page,
                            // 98.18 → 44.70).
                            let has_text = pd
                                .descendants(p, Some(&W::t()))
                                .iter()
                                .any(|&t| !pd.value_str(t).trim().is_empty());
                            if !has_text {
                                continue;
                            }
                            // STYLED OR NUMBERED paragraphs only: the oracle
                            // bakes pStyle'd paras and pStyle-less LIST paras
                            // (numPr present — rstyle "List item 2"/"Back"),
                            // never truly-bare ones (m370 title;
                            // tiff×h_f_normal regressed −56 when its 52 bare
                            // inserted paras took the bake).
                            let pstyle = pd
                                .element(ppr, &W::p_style())
                                .and_then(|ps| pd.attribute(ps, &W::val()))
                                .map(str::to_string);
                            let has_numpr = pd.element(ppr, &W::num_pr()).is_some();
                            if pstyle.is_none() && !has_numpr {
                                continue;
                            }
                            let (Some(out_style), Some(b_style)) = (match pstyle {
                                Some(id) => (Some(id.clone()), b_id_for(&id)),
                                None => (out_default.clone(), b_default.clone()),
                            }) else {
                                continue; // style not from B — no B-effective target
                            };
                            // The innermost table's style, on each side.
                            let out_table = pd
                                .ancestors(p, Some(&W::tbl()))
                                .first()
                                .and_then(|&t| pd.element(t, &W::tbl_pr()))
                                .and_then(|t| pd.element(t, &W::name("tblStyle")))
                                .and_then(|t| pd.attribute(t, &W::val()))
                                .map(str::to_string);
                            let b_table = out_table.as_deref().and_then(b_id_for);
                            let (b_eff, b_declared, _) = effective_para_spacing(
                                &sd,
                                br,
                                &b_idx,
                                &b_style,
                                b_table.as_deref(),
                            );
                            let (o_eff, _, _) = effective_para_spacing(
                                &sd,
                                or,
                                &out_idx,
                                &out_style,
                                out_table.as_deref(),
                            );
                            if b_eff == o_eff {
                                continue;
                            }
                            let sp = pd.element(ppr, &W::spacing_el());
                            let attr_names = ["after", "before", "line", "lineRule"];
                            let mut to_write: Vec<(usize, String)> = Vec::new();
                            for i in 0..4 {
                                if b_eff[i] == o_eff[i] || b_declared[i] {
                                    continue;
                                }
                                let declared = sp.is_some_and(|s| {
                                    pd.attribute(s, &W::name(attr_names[i])).is_some()
                                });
                                if !declared {
                                    to_write.push((i, b_eff[i].clone()));
                                }
                            }
                            // line without lineRule renders as exact twips —
                            // carry the rule whenever line is written.
                            if to_write.iter().any(|(i, _)| *i == 2)
                                && !to_write.iter().any(|(i, _)| *i == 3)
                                && !sp.is_some_and(|s| {
                                    pd.attribute(s, &W::name("lineRule")).is_some()
                                })
                            {
                                to_write.push((3, b_eff[3].clone()));
                            }
                            if to_write.is_empty() {
                                continue;
                            }
                            let sp = match sp {
                                Some(s) => s,
                                None => {
                                    let s = pd.new_element(W::spacing_el());
                                    insert_child_by_rank(
                                        &mut pd,
                                        ppr,
                                        s,
                                        "spacing",
                                        &ppr_child_rank,
                                    );
                                    s
                                }
                            };
                            for (i, v) in to_write {
                                pd.set_attribute_value(sp, &W::name(attr_names[i]), Some(&v));
                                doc_changed = true;
                            }
                        }
                        if doc_changed {
                            out.set_part(&main1, pd.serialize_element(droot).into_bytes());
                        }
                    }
                }
            }
        }
    }

    // Word-mode repair: Word synthesizes a default numbering definition when
    // the document references a numId no w:num defines — dangling numbering
    // silently renders lists as plain paragraphs; Word repairs it on open, so
    // match it at compare time (evidence: nested-table-rowspan_numbered-list).
    // An edit leaves the source's numbering as Word opens it.
    if settings.merge_replaced_paragraphs && !settings.preserve_untouched {
        let num_id = W::name("numId");
        let mut referenced: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for np in dom.descendants(result_root, Some(&W::num_pr())) {
            // numeric guard: ST_DecimalNumber ids only — also keeps the
            // synthesized w:num XML injection-proof (ids are interpolated
            // into a template downstream)
            if let Some(nid) = dom.element(np, &num_id)
                && let Some(v) = dom.attribute(nid, &W::val())
                && v != "0"
                && v.parse::<u32>().is_ok()
                && seen.insert(v.to_string())
            {
                referenced.push(v.to_string());
            }
        }
        if !referenced.is_empty() {
            let existing = out.part_string("word/numbering.xml");
            let mut nd = Dom::new();
            let (nroot, part_was_missing) = match &existing {
                Some(xml) => {
                    let d = nd.parse_xdocument(xml);
                    (nd.root(d), false)
                }
                None => {
                    let d = nd.parse_xdocument(&format!(
                        "<w:numbering xmlns:w=\"{}\"></w:numbering>",
                        W::URI
                    ));
                    (nd.root(d), true)
                }
            };
            if let Some(nroot) = nroot {
                let defined: std::collections::HashSet<String> = nd
                    .elements(nroot, Some(&W::name("num")))
                    .into_iter()
                    .filter_map(|e| nd.attribute(e, &num_id).map(|s| s.to_string()))
                    .collect();
                let dangling: Vec<String> = referenced
                    .into_iter()
                    .filter(|r| !defined.contains(r))
                    .collect();
                if !dangling.is_empty() {
                    crate::comparer::footnotes::synthesize_dangling_numbering(
                        &mut nd, nroot, &dangling,
                    );
                    out.set_part(
                        "word/numbering.xml",
                        nd.serialize_element(nroot).into_bytes(),
                    );
                    if part_was_missing {
                        out.add_content_type_override(
                            "/word/numbering.xml",
                            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                        );
                        let has_rel = out.read_rels_for(&main1).is_some_and(|r| {
                            r.items.iter().any(|i| i.rel_type.ends_with("/numbering"))
                        });
                        if !has_rel {
                            out.add_document_relationship(
                                &main1,
                                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
                                "numbering.xml",
                            );
                        }
                    }
                }
            }
        }
    }

    // B.4 — write the rectified withRevisions notes parts (separators +
    // referenced definitions renumbered 1..n with real revision markup) into
    // the output package. Replaces the old by-id `compare_note_parts` model,
    // whose pairing broke whenever Word renumbered notes.
    //
    // Remap style refs on notes_ctx *before* serialize/writeback so B.4 does
    // not clobber an earlier styles pass with un-remapped note content.
    if settings.merge_replaced_paragraphs && !style_renames.is_empty() {
        for note_root in [notes_ctx.fn_with_revisions, notes_ctx.en_with_revisions]
            .into_iter()
            .flatten()
        {
            remap_style_refs(&mut dom, note_root, &style_renames);
        }
    }
    let mut footnote_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut endnote_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (part, root, is_fn) in [
        (fn1.as_str(), notes_ctx.fn_with_revisions, true),
        (en1.as_str(), notes_ctx.en_with_revisions, false),
    ] {
        if let Some(r) = root {
            if union_permissions {
                crate::comparer::bookmarks::carry_permission_union(
                    &mut dom,
                    r,
                    (&pkg1, part),
                    (&pkg2, if is_fn { &fn2 } else { &en2 }),
                    &settings.author_for_revisions,
                );
            }
            let def = if is_fn { W::footnote() } else { W::endnote() };
            let ids: std::collections::HashSet<String> = dom
                .elements(r, Some(&def))
                .into_iter()
                .filter_map(|n| dom.attribute(n, &W::id()).map(str::to_string))
                .collect();
            if is_fn {
                footnote_ids = ids;
            } else {
                endnote_ids = ids;
            }
            out.set_part(part, dom.serialize_element(r).into_bytes());
        }
    }
    // M379: after B.4 notes writeback — if A had no separators, B.4 left empty
    // shells or skipped; copy B's separator/continuationSeparator notes.
    if settings.merge_replaced_paragraphs {
        adopt_b_notes_when_a_lacks_separators(&mut out, &pkg1, &pkg2, &main1);
    }
    // An edit leaves no empty notes part the source did not have: the one
    // preprocessing added held nothing the plan wrote.
    if settings.preserve_untouched {
        for (had, ids, kind) in [
            (original_notes.0, &footnote_ids, "footnotes"),
            (original_notes.1, &endnote_ids, "endnotes"),
        ] {
            if had || !ids.is_empty() {
                continue;
            }
            let rel_type = format!(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}"
            );
            let rel_ids: Vec<String> = out
                .read_rels_for(&main1)
                .map(|rels| {
                    rels.items
                        .iter()
                        .filter(|r| r.rel_type == rel_type)
                        .map(|r| r.id.clone())
                        .collect()
                })
                .unwrap_or_default();
            for id in rel_ids {
                out.remove_related_part(&main1, &id);
            }
        }
    }
    // settings.xml may still list special footnote/endnote ids (e.g. id=1
    // continuationNotice) that rectify dropped. Dangling settings refs make
    // Word show "unreadable content" (OpenXmlValidator Semantic).
    if let Some(sx) = out.part_string("word/settings.xml") {
        let mut sd = Dom::new();
        let sdoc = sd.parse_xdocument(&sx);
        if let Some(sroot) = sd.root(sdoc) {
            crate::comparer::footnotes::sync_settings_special_note_ids(
                &mut sd,
                sroot,
                &footnote_ids,
                &endnote_ids,
            );
            out.set_part(
                "word/settings.xml",
                sd.serialize_element(sroot).into_bytes(),
            );
        }
    }
    // M4.H.x: header/footer CONTENT diff (Word redlines header/footer changes; we
    // previously only copied the original's). Match A's parts to B's by reference
    // (kind,type), diff the content and write the redline into the output's
    // (original's) part. Neutralize dangling references on both sides and
    // carry B's valid references into A's rels before diffing. A failed story
    // comparison must refuse the package, never silently retain A (#1709).
    {
        let (sections_a, refs_a) = header_footer_refs_by_section(&pkg1);
        let (sections_b, refs_b) = header_footer_refs_by_section(&pkg2);
        let pairing = out
            .part_string(&main1)
            .and_then(|x| section_pairing(&x, sections_a, sections_b));
        let own_slots: std::collections::HashSet<(usize, String, String)> = refs_a
            .iter()
            .map(|(s, k, t, _)| (*s, k.clone(), t.clone()))
            .collect();
        let mut ordinals: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        let mut diffed = std::collections::HashSet::new();
        for (section_a, kind, ty, part_a) in refs_a {
            let ordinal = ordinals.entry((kind.clone(), ty.clone())).or_default();
            let at = *ordinal;
            *ordinal += 1;
            if !diffed.insert(part_a.clone()) {
                continue;
            }
            // The part also shows in the later sections that inherit it:
            // the first whose break survives says what it became (Word
            // probe hs5 diffs a deleted section's header that the last
            // section inherits).
            let section = pairing.as_ref().map(|p| {
                (section_a..p.len())
                    .take_while(|&s| {
                        s == section_a || !own_slots.contains(&(s, kind.clone(), ty.clone()))
                    })
                    .find_map(|s| p[s])
            });
            let Some(part_b) =
                pair_header_footer(&pkg1, &pkg2, &refs_b, (&kind, &ty, &part_a), at, section)
            else {
                continue;
            };
            let part_b = &part_b;
            {
                let xa = pkg1.part_string(&part_a).ok_or_else(|| {
                    story_diff_error(&part_a, "original story XML is unavailable")
                })?;
                let xb = pkg2
                    .part_string(part_b)
                    .ok_or_else(|| story_diff_error(part_b, "revised story XML is unavailable"))?;
                let xa = neutralize_story_relationships(&pkg1, &part_a, &xa)?;
                let xb = neutralize_story_relationships(&pkg2, part_b, &xb)?;
                // B's references that mean something else in A's part (both
                // headers picture their own logo as rId1) move to fresh ids.
                let xb = if part_rels_agree((&pkg1, &part_a), (&pkg2, part_b), &xb) {
                    xb
                } else {
                    carry_revised_part_relationships(&mut out, &part_a, (&pkg2, part_b), &xb)
                        .ok_or_else(|| {
                            story_diff_error(part_b, "cannot carry revised story relationships")
                        })?
                };
                let mut hd = Dom::new();
                let da = hd.parse_xdocument(&xa);
                let db = hd.parse_xdocument(&xb);
                let ra = hd.root(da).ok_or_else(|| {
                    story_diff_error(&part_a, "original story has no root element")
                })?;
                let rb = hd
                    .root(db)
                    .ok_or_else(|| story_diff_error(part_b, "revised story has no root element"))?;
                let container_name = if kind == "header" { W::hdr() } else { W::ftr() };
                for (root, part) in [(ra, part_a.as_str()), (rb, part_b.as_str())] {
                    if !hd.name_is(root, &container_name) {
                        return Err(story_diff_error(part, "unexpected story root element"));
                    }
                    refuse_unatomizable(&hd, root, part)?;
                    refuse_past_story_budget(story_units(&hd, root), part, settings.input_limits)?;
                }
                {
                    let text_of = |d: &Dom, n: crate::xmllinq::NodeId| -> String {
                        d.descendants(n, Some(&W::t()))
                            .into_iter()
                            .map(|t| d.value(t))
                            .collect()
                    };
                    // capture BEFORE the compare mutates the arena
                    let a_text = text_of(&hd, ra);
                    let b_text = text_of(&hd, rb);
                    let b_ends_with_table = hd
                        .elements(rb, None)
                        .last()
                        .is_some_and(|&e| hd.name_is(e, &W::tbl()));
                    let res = compare_bodies_faithful(&mut hd, ra, rb, ra, rb, settings);
                    // compare_bodies_faithful always rebuilds into
                    // <w:document><w:body>…</w:body></w:document>, even when the
                    // source roots are w:hdr / w:ftr. Looking for a nested
                    // hdr/ftr under that wrapper is dead (PR #81 / kilo): re-wrap
                    // the body children as the original container type so the
                    // redlined part stays a valid header/footer part.
                    let out_body = hd.element(res, &W::body()).ok_or_else(|| {
                        story_diff_error(&part_a, "story comparison produced no body")
                    })?;
                    let container = hd.new_element(container_name);
                    // Preserve source-root namespace decls (w already on body
                    // children; copy any extras from A's original root).
                    let xmlns_ns = crate::xmllinq::XNamespace::xmlns();
                    for (an, av) in hd.attributes(ra) {
                        if an.namespace_name() == xmlns_ns.namespace_name()
                            || an.local_name() == "Ignorable"
                        {
                            hd.set_attribute_value(container, &an, Some(&av));
                        }
                    }
                    // Body-level sectPr is document geometry from the
                    // compare_bodies_faithful wrap — not valid inside hdr/ftr.
                    for c in hd.elements(out_body, None) {
                        if hd.name(c) == Some(W::sect_pr()) {
                            continue;
                        }
                        hd.remove(c);
                        hd.add(container, c);
                    }
                    if b_ends_with_table {
                        delete_story_closing_mark(&mut hd, container, settings);
                    }
                    if union_permissions {
                        crate::comparer::bookmarks::carry_permission_union(
                            &mut hd,
                            container,
                            (&pkg1, &part_a),
                            (&pkg2, part_b),
                            &settings.author_for_revisions,
                        );
                    }
                    // A changed story with no revision marks is not a redline.
                    // The old M-PAG guard silently kept A when this failed;
                    // refuse instead, including when the diff lost B's text.
                    let redlined = hd.serialize_element(container);
                    let has_revisions =
                        crate::revision_processor::xml_has_tracked_revisions(&redlined);
                    ensure_story_text_tracked(
                        &part_a,
                        &a_text,
                        &b_text,
                        &text_of(&hd, container),
                        has_revisions,
                    )?;
                    out.set_part(&part_a, redlined.into_bytes());
                }
            }
        }
    }

    // Every other part resolves its own r:* references too (B's comments, notes,
    // headers/footers and picture bullets arrive without B's per-part rels).
    crate::comparer::parts::reconcile_part_relationships(&mut out, &main1, &pkg1, &pkg2);
    // …and every copied part keeps the content type its source declared (a B
    // header's `.wdp` HD Photo arrived without B's `wdp` Default).
    out.adopt_missing_content_types(&[&pkg2, &pkg1]);

    // Strict/ISO OOXML: when the original is Strict, the output package would mix
    // a Transitional comparison-result document.xml with Strict styles/numbering/
    // rels (and copied-in Transitional styles) — an invalid mixed package. Make
    // the whole package consistently Transitional. No-op for Transitional packages.
    for part in out.parts() {
        if !(part.ends_with(".xml") || part.ends_with(".rels")) {
            continue;
        }
        if let Some(s) = out.part_string(&part)
            && s.contains("purl.oclc.org/ooxml/")
        {
            let n = normalize_strict_namespaces(&s).into_owned();
            out.set_part(&part, n.into_bytes());
        }
    }
    sweep_part_validity(
        &mut out,
        &main1,
        Some(&ComparisonSweep {
            original: &pkg1,
            settings,
            ladder_sources: ladder_sources.as_ref(),
        }),
    );
    // Final package-level notes↔settings coherence (after the validity sweep
    // re-serialized those parts). Dangling special-note ids in settings are a
    // package bug, not a document.xml bug.
    {
        let collect_ids = |part: &str, local: &str| -> std::collections::HashSet<String> {
            let mut set = std::collections::HashSet::new();
            let Some(x) = out.part_string(part) else {
                return set;
            };
            let mut d = Dom::new();
            let doc = d.parse_xdocument(&x);
            let Some(root) = d.root(doc) else {
                return set;
            };
            let name = W::name(local);
            for n in d.elements(root, Some(&name)) {
                if let Some(id) = d.attribute(n, &W::id()) {
                    set.insert(id.to_string());
                }
            }
            set
        };
        let fn_ids = collect_ids("word/footnotes.xml", "footnote");
        let en_ids = collect_ids("word/endnotes.xml", "endnote");
        if let Some(sx) = out.part_string("word/settings.xml") {
            let mut sd = Dom::new();
            let sdoc = sd.parse_xdocument(&sx);
            if let Some(sroot) = sd.root(sdoc) {
                crate::comparer::footnotes::sync_settings_special_note_ids(
                    &mut sd, sroot, &fn_ids, &en_ids,
                );
                out.set_part(
                    "word/settings.xml",
                    sd.serialize_element(sroot).into_bytes(),
                );
            }
        }
    }
    // A header or footer of the original whose slots the revised section
    // breaks took over (redlineB_* parts) is left with no reference: Word's
    // own redlines carry none, and its half-matched fields read as broken.
    crate::revision_processor::drop_unreferenced_headers_footers(&mut out);
    out.to_zip()
}

/// The inherited-shape repairs and schema-order pass every package we build
/// gets (#1712, #1796): an identical compare's output and, since #1717, a
/// standalone accept's or reject's.
fn repair_resolved_package_validity(pkg: &mut PartFs) {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    sweep_part_validity(pkg, &main, None);
}

/// Identical inputs need repairs, not the chrome of a newly produced redline.
/// Work on the admitted package directly: serializing drawing repairs and then
/// reopening them would incorrectly apply input limits to our expanded output.
fn finalize_identical_compare(
    bytes: &[u8],
    settings: &WmlComparerSettings,
) -> Result<Vec<u8>, OpcError> {
    let mut out = PartFs::open_within(bytes, settings.input_limits)?;
    let main = out
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = out
        .part_string(&main)
        .ok_or_else(|| OpcError::PartNotFound(format!("main document missing: {main}")))?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    if dom.root(doc).is_none() {
        return Err(OpcError::PartNotFound(format!(
            "main document has no root element: {main}"
        )));
    }
    repair_resolved_package_validity(&mut out);
    out.to_zip()
}

/// Context available only when comparison has actually produced a redline.
struct ComparisonSweep<'a> {
    original: &'a PartFs,
    settings: &'a WmlComparerSettings,
    ladder_sources: Option<&'a (String, String)>,
}

/// Repairs run at every successful exit. Comparison cleanup is interleaved
/// only on the full path, in exactly its original order (#1796).
fn sweep_part_validity(out: &mut PartFs, main: &str, comparison: Option<&ComparisonSweep<'_>>) {
    // Word-validity normalization on every validity-swept content part — NOT
    // document.xml alone. Word opens the package (headers/footers/notes/
    // settings/styles/rels/content-types); a clean body with a corrupt notes
    // or settings part still raises "unreadable content".
    //
    // (validator sweep: 146/166 outputs carried schema errors Word's own
    // redlines don't): canonicalize universal measures / fractional ints,
    // fix Strict artifacts (cnfStyle bitmask, wp14 percents, out-of-range
    // paraIds), and strip pt:* scratch so headers/notes don't ship Unids.
    // Scope notes: `word/charts/` (DrawingML) and `word/theme/` are included
    // ON PURPOSE — the Strict percent→per-thousand rewrite covers drawingml
    // namespaces; `word/media/*.xml` is vacuous for binary payloads.
    // Custom XML data holds the application's own XML whatever its name
    // (`word/headerData.xml`), so no sweep below touches it (codex on #1680).
    let custom_data = {
        let parts = out.parts();
        crate::opc::custom_xml_data(std::iter::once(("", out.package_relationships())).chain(
            parts.iter().filter_map(|source| {
                out.read_rels_for(source)
                    .map(|rels| (source.as_str(), rels))
            }),
        ))
    };
    // Comment bodies are a story the comparison writes, and can hold a table
    // (#1712). The glossary is no story and stays as it came (#1680).
    for part in out.parts() {
        if custom_data
            .iter()
            .any(|data| data.eq_ignore_ascii_case(&part))
        {
            continue;
        }
        let is_swept = part == main
            || part == "word/styles.xml"
            || part == "word/numbering.xml"
            || part == "word/footnotes.xml"
            || part == "word/endnotes.xml"
            || part == "word/comments.xml"
            || part == "word/settings.xml"
            || (part.starts_with("word/header") && part.ends_with(".xml"))
            || (part.starts_with("word/footer") && part.ends_with(".xml"))
            || (part.starts_with("word/diagrams/") && part.ends_with(".xml"))
            || (part.starts_with("word/charts/") && part.ends_with(".xml"))
            || (part.starts_with("word/theme/") && part.ends_with(".xml"))
            || (part.starts_with("word/media/") && part.ends_with(".xml"));
        if !is_swept {
            continue;
        }
        if let Some(x) = out.part_string(&part) {
            let mut vd = Dom::new();
            let doc = vd.parse_xdocument(&x);
            if let Some(vr) = vd.root(doc) {
                // Preserve every untouched part byte-for-byte on an identical
                // exit; the full path historically does this for comments only.
                let carried = (comparison.is_none() || part == "word/comments.xml")
                    .then(|| vd.serialize_element(vr));
                if comparison.is_none() && part == main {
                    // Unique DrawingML ids are valid; VML duplicates are too.
                    // Only missing/invalid or duplicate docPr ids need repair.
                    crate::comparer::fixups::rename_repeated_doc_pr_ids(&mut vd, vr);
                }
                repair_part_validity(&mut vd, vr, comparison.is_some());
                if let Some(context) = comparison {
                    sweep_comparison_chrome(out, &part, main, &mut vd, vr, context);
                }
                // Last: every pass above may append properties out of order.
                if comparison.is_some() {
                    crate::comparer::finalize::enforce_part_schema_order(&mut vd, vr);
                    crate::comparer::finalize::declare_extension_namespaces_ignorable(&mut vd, vr);
                } else {
                    crate::comparer::finalize::repair_part_schema_order(&mut vd, vr);
                }
                crate::comparer::finalize::bind_compatibility_prefixes(&mut vd, vr);
                let swept = vd.serialize_element(vr);
                if carried.as_deref() == Some(swept.as_str()) {
                    continue;
                }
                out.set_part(&part, swept.into_bytes());
            }
        }
    }
    // The glossary is not swept above: it is not a story. Repair the three
    // inherited shapes in it anyway, and leave the part untouched when that
    // repair changes nothing.
    repair_glossary_shapes(out);
    // Settling final marks is comparison chrome, including renamed stories.
    if comparison.is_none() {
        return;
    }
    // A story part under a name of its own (a header at `word/chrome.xml`)
    // is not swept above, but its cells' final marks settle too (codex on
    // #1680, round 10). Stories are known by the main document's
    // relationships to them: a custom XML data item or the glossary holds
    // cells too, but the comparison wrote neither and they stay as they
    // were (minimax on #1680, round 11).
    let stories: Vec<String> = out
        .read_rels_for(main)
        .map(|rels| {
            rels.items
                .iter()
                .filter(|r| {
                    r.target_mode.as_deref() != Some("External")
                        && ["/header", "/footer", "/footnotes", "/endnotes", "/comments"]
                            .iter()
                            .any(|kind| r.rel_type.ends_with(kind))
                })
                .map(|r| out.resolve_rel_target(main, &r.target))
                .collect()
        })
        .unwrap_or_default();
    for part in stories {
        // Settled above already. One predicate for both sweeps, so a story
        // part cannot fall between them (`word/headerA.dat`, CodeRabbit on
        // #1680, round 12).
        if sweeps_final_marks(&part, main) {
            continue;
        }
        let Some(x) = out.part_string(&part).filter(|x| x.contains("tc")) else {
            continue;
        };
        let mut vd = Dom::new();
        let doc = vd.parse_xdocument(&x);
        let Some(vr) = vd.root(doc) else {
            continue;
        };
        let unsettled = vd.name(vr).is_some_and(|n| n.namespace_name() == W::URI)
            && vd
                .descendants(vr, Some(&W::tc()))
                .into_iter()
                .any(|cell| !crate::validate_dom::uncovered_final_marks(&vd, cell).is_empty());
        if unsettled {
            crate::comparer::final_marks::settle_cell_final_marks(&mut vd, vr);
            // The finalizers the sweep above runs, so a renamed story keeps
            // its extension namespaces declared and ignorable (CodeRabbit on
            // #1680).
            crate::comparer::finalize::enforce_part_schema_order(&mut vd, vr);
            crate::comparer::finalize::declare_extension_namespaces_ignorable(&mut vd, vr);
            crate::comparer::finalize::bind_compatibility_prefixes(&mut vd, vr);
            out.set_part(&part, vd.serialize_element(vr).into_bytes());
        }
    }
}

/// Schema/structural validity repairs shared by the two sweep modes.
fn repair_part_validity(dom: &mut Dom, root: NodeId, comparison: bool) {
    if comparison {
        crate::comparer::finalize::normalize_universal_measures(dom, root);
        crate::comparer::finalize::fix_strict_validity_artifacts(dom, root);
    } else {
        crate::comparer::finalize::repair_invalid_measures(dom, root);
        crate::comparer::finalize::repair_strict_validity_artifacts(dom, root);
    }
    crate::comparer::finalize::repair_inherited_invalidity(dom, root);
    // CT_RunTrackChange excludes hyperlink and fldSimple wrappers. Hoist
    // only those illegal immediate children, preserving revision ownership.
    crate::comparer::finalize::hoist_hyperlinks_out_of_revisions(dom, root);
    if comparison {
        crate::comparer::finalize::enforce_deleted_text_kinds(dom, root);
    }
}

/// Formatting/default removal belongs to a comparison, never an identical exit.
fn sweep_comparison_chrome(
    out: &PartFs,
    part: &str,
    main: &str,
    vd: &mut Dom,
    root: NodeId,
    context: &ComparisonSweep<'_>,
) {
    let ComparisonSweep {
        original,
        settings,
        ladder_sources,
    } = context;
    crate::comparer::finalize::remove_powertools_scratch_markup(vd, root);
    // Produce strips an empty pPr shell, then later package steps
    // (style cascade, spacing restore) can take the live children
    // back off and leave the shell. Word's redline of the short
    // title mixes has neither the shell nor the children.
    if part == main {
        crate::comparer::finalize::strip_propertyless_ppr(vd, root);
        // Word-visual chrome. Faithful mode runs none of these;
        // the body path already gates them the same way, and an
        // edit keeps the source's markup and the plan's links.
        if settings.merge_replaced_paragraphs && !settings.preserve_untouched {
            // Package steps can put a default nextPage back into the
            // recorded sectPr, or leave line=276 on a deleted mark.
            let original_line = original
                .part_string("word/styles.xml")
                .map(|x| default_paragraph_line(&x));
            crate::comparer::finalize::strip_unrecorded_word_defaults(
                vd,
                root,
                original_line.as_deref(),
            );
            let id_to_target: std::collections::HashMap<String, String> = out
                .read_rels_for(main)
                .map(|rels| {
                    rels.items
                        .iter()
                        .filter(|r| r.rel_type.ends_with("/hyperlink"))
                        .map(|r| (r.id.clone(), r.target.clone()))
                        .collect()
                })
                .unwrap_or_default();
            let base_targets: std::collections::HashSet<String> = original
                .read_rels_for(main)
                .map(|rels| {
                    rels.items
                        .iter()
                        .filter(|r| r.rel_type.ends_with("/hyperlink"))
                        .map(|r| r.target.clone())
                        .collect()
                })
                .unwrap_or_default();
            crate::comparer::finalize::rewrite_inserted_external_hyperlinks(
                vd,
                root,
                &id_to_target,
                &base_targets,
            );
            crate::comparer::finalize::align_word_table_and_comment_chrome(vd, root);
        }
        if let Some((original_xml, revised_xml)) = ladder_sources {
            crate::comparer::finalize::align_remaining_ladder_rungs(
                vd,
                root,
                &crate::comparer::finalize::LadderSources {
                    original_xml,
                    revised_xml,
                    settings,
                },
            );
            // After every mark-writing pass and before the cells settle:
            // the inserted paragraph and the deleted table are both
            // finished here (#1683, case 2).
            if let Some(body) = vd.element(root, &W::body()) {
                crate::comparer::finalize::pair_closing_insert_across_deleted_table(
                    vd,
                    body,
                    revised_xml,
                );
            }
        }
    }
    // A cell's final mark carries no revision its row does not
    // (#1669, #1684), in every story: the body (its text boxes
    // too), the notes, the headers and footers. After every pass
    // that writes marks.
    if sweeps_final_marks(part, main) {
        crate::comparer::final_marks::settle_cell_final_marks(vd, root);
    }
}

/// The parts the validity sweep settles cells' final marks in, by name: the
/// main part, the notes, and `word/header*.xml` / `word/footer*.xml`. The
/// story sweep after it settles every other story part.
fn sweeps_final_marks(part: &str, main: &str) -> bool {
    part == main
        || part == "word/footnotes.xml"
        || part == "word/endnotes.xml"
        || ((part.starts_with("word/header") || part.starts_with("word/footer"))
            && part.ends_with(".xml"))
}

/// Glossary parts: the canonical building-block part, and any part a
/// `glossaryDocument` relationship reaches. Not a story, so cell marks are
/// left alone; only the inherited schema shapes are repaired.
fn glossary_parts(out: &PartFs) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut push = |name: String| {
        if name.is_empty()
            || found
                .iter()
                .any(|have: &String| have.eq_ignore_ascii_case(&name))
        {
            return;
        }
        found.push(name);
    };
    for part in out.parts() {
        if part.eq_ignore_ascii_case("word/glossary/document.xml") {
            push(part);
        }
    }
    for (source, rels) in out.relationship_parts() {
        for rel in &rels.items {
            if rel.target_mode.as_deref() == Some("External")
                || !rel.rel_type.ends_with("/glossaryDocument")
            {
                continue;
            }
            push(out.resolve_rel_target(source, &rel.target));
        }
    }
    found
}

/// Drop an empty shading fill, a grid-column type, and out-of-order cell
/// margins in a glossary. Write the part back only when that changes it.
/// The glossary is carried, not compared, so compare repairs it as accept
/// and reject do: the comparison's canonicalizer would drop comments and
/// whitespace in a valid part's properties and rewrite it (#2117 review).
fn repair_glossary_shapes(out: &mut PartFs) {
    for part in glossary_parts(out) {
        let Some(xml) = out.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        let before = dom.serialize_element(root);
        crate::comparer::finalize::repair_inherited_invalidity(&mut dom, root);
        crate::comparer::finalize::repair_part_schema_order(&mut dom, root);
        let after = dom.serialize_element(root);
        if before != after {
            out.set_part(&part, after.into_bytes());
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/based_style_record_tests.rs"]
mod based_style_record_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/tests.rs"]
mod tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_boundary_tests.rs"]
mod coverage_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/comparer_relationship_contract_tests.rs"]
mod comparer_relationship_contract_tests;
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_package_source_matrix_tests.rs"]
mod coverage_package_source_matrix_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/authored_style_chain_boundary_tests.rs"]
mod authored_style_chain_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/isolated_header_container_revision_boundary_tests.rs"]
mod isolated_header_container_revision_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/redefined_style_chain_source_owner_tests.rs"]
mod redefined_style_chain_source_owner_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/namespace_declared_default_property_owner_tests.rs"]
mod namespace_declared_default_property_owner_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/adopted_story_relationship_owner_tests.rs"]
mod adopted_story_relationship_owner_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/word_style_line_owned_boundary_tests.rs"]
mod word_style_line_owned_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/public_numbering_definition_source_owner_tests.rs"]
mod public_numbering_definition_source_owner_tests;

#[cfg(test)]
#[path = "tests/story_diff_failure_tests.rs"]
mod story_diff_failure_tests;

/// A section break the revised paragraph carries and the original paragraph
/// does not. The final body section, hoisted into the last paragraph before
/// the diff, is not one: it wears the hoist stamp.
fn authored_section_break(dom: &Dom, paragraph: NodeId) -> bool {
    let hoist = crate::namespaces::PT::name("BodySectPrHoist");
    dom.element(paragraph, &W::p_pr()).is_some_and(|ppr| {
        dom.elements(ppr, Some(&W::sect_pr()))
            .into_iter()
            .any(|section| dom.attribute(section, &hoist) != Some("1"))
    })
}

fn paragraph_plain_text(dom: &Dom, paragraph: NodeId) -> String {
    dom.descendants(paragraph, Some(&W::t()))
        .into_iter()
        .map(|text| dom.value(text))
        .collect()
}

struct SectionRow {
    text: String,
    authored: bool,
    /// A paragraph, table or block wrapper follows it in its body. A
    /// break added there takes Word's middle model, even when what follows
    /// is a blank paragraph or a table (#2126 review).
    followed: bool,
}

fn followed_by_block(dom: &Dom, paragraph: NodeId) -> bool {
    let mut next = dom.next_element(paragraph);
    while let Some(node) = next {
        if dom
            .name(node)
            .is_some_and(|name| matches!(name.local_name(), "p" | "tbl" | "sdt" | "customXml"))
        {
            return true;
        }
        next = dom.next_element(node);
    }
    false
}

fn section_rows(dom: &Dom, body: NodeId) -> Vec<SectionRow> {
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .filter_map(|paragraph| {
            let text = paragraph_plain_text(dom, paragraph);
            if text.is_empty() {
                None
            } else {
                Some(SectionRow {
                    authored: authored_section_break(dom, paragraph),
                    followed: followed_by_block(dom, paragraph),
                    text,
                })
            }
        })
        .collect()
}

fn has_paragraph_mark(dom: &Dom, paragraph: NodeId) -> bool {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.elements(rpr, None).into_iter().any(|child| {
                dom.name(child).is_some_and(|name| {
                    matches!(name.local_name(), "ins" | "del" | "moveFrom" | "moveTo")
                })
            })
        })
}

fn next_revision_id(dom: &Dom, root: NodeId) -> u32 {
    let mut next = 0u32;
    for node in dom.descendants_and_self(root, None) {
        let Some(raw) = dom.attribute(node, &W::id()) else {
            continue;
        };
        let Ok(id) = raw.parse::<u32>() else {
            continue;
        };
        next = next.max(id.saturating_add(1));
    }
    next
}

fn paragraph_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    if let Some(ppr) = dom.element(paragraph, &W::p_pr()) {
        return ppr;
    }
    let ppr = dom.new_element(W::p_pr());
    dom.add_first(paragraph, ppr);
    ppr
}

fn run_properties_before_section(dom: &mut Dom, ppr: NodeId) -> NodeId {
    if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
        return rpr;
    }
    let rpr = dom.new_element(W::r_pr());
    let anchor = dom
        .elements(ppr, None)
        .into_iter()
        .find(|&child| dom.name_is(child, &W::sect_pr()) || dom.name_is(child, &W::p_pr_change()));
    if let Some(anchor) = anchor {
        dom.add_before_self(anchor, rpr);
    } else {
        dom.add(ppr, rpr);
    }
    rpr
}

fn drop_empty_paragraph_change(dom: &mut Dom, ppr: NodeId) {
    let Some(change) = dom.element(ppr, &W::p_pr_change()) else {
        return;
    };
    let records_layout = dom.element(change, &W::p_pr()).is_some_and(|inner| {
        dom.elements(inner, None)
            .into_iter()
            .any(|child| !dom.name_is(child, &W::r_pr()))
    });
    if !records_layout {
        dom.remove(change);
    }
}

fn push_paragraph_mark(
    dom: &mut Dom,
    rpr: NodeId,
    inserted: bool,
    id: u32,
    author: &str,
    date: &str,
) {
    let marked = dom.elements(rpr, None).into_iter().any(|child| {
        dom.name(child)
            .is_some_and(|name| matches!(name.local_name(), "ins" | "del" | "moveFrom" | "moveTo"))
    });
    if marked {
        return;
    }
    let mark = dom.new_element(if inserted { W::ins() } else { W::del() });
    let id_text = id.to_string();
    dom.set_attribute_value(mark, &W::id(), Some(&id_text));
    dom.set_attribute_value(mark, &W::author(), Some(author));
    dom.set_attribute_value(mark, &W::date(), Some(date));
    dom.add_first(rpr, mark);
}

fn following_paragraph(dom: &Dom, paragraph: NodeId) -> Option<NodeId> {
    let parent = dom.parent(paragraph)?;
    let siblings = dom.elements(parent, Some(&W::p()));
    let index = siblings.iter().position(|&sibling| sibling == paragraph)?;
    siblings.get(index + 1).copied()
}

fn paragraph_is_blank(dom: &Dom, paragraph: NodeId) -> bool {
    paragraph_plain_text(dom, paragraph).is_empty()
        && dom
            .element(paragraph, &W::p_pr())
            .is_none_or(|ppr| dom.element(ppr, &W::sect_pr()).is_none())
}

fn track_closing_added_break(dom: &mut Dom, paragraph: NodeId, id: u32, author: &str, date: &str) {
    let ppr = paragraph_properties(dom, paragraph);
    if dom.element(ppr, &W::sect_pr()).is_none() {
        return;
    }
    drop_empty_paragraph_change(dom, ppr);
    let rpr = run_properties_before_section(dom, ppr);
    push_paragraph_mark(dom, rpr, true, id, author, date);
    let followed_by_blank =
        following_paragraph(dom, paragraph).is_some_and(|next| paragraph_is_blank(dom, next));
    if !followed_by_blank {
        let blank = dom.new_element(W::p());
        dom.add_after_self(paragraph, blank);
    }
}

fn track_middle_added_break(
    dom: &mut Dom,
    paragraph: NodeId,
    deleted_id: u32,
    inserted_id: u32,
    author: &str,
    date: &str,
) {
    let ppr = paragraph_properties(dom, paragraph);
    let Some(section) = dom.element(ppr, &W::sect_pr()) else {
        return;
    };
    drop_empty_paragraph_change(dom, ppr);
    let rpr = run_properties_before_section(dom, ppr);
    push_paragraph_mark(dom, rpr, false, deleted_id, author, date);
    let blank = dom.new_element(W::p());
    let blank_ppr = dom.new_element(W::p_pr());
    dom.add(blank, blank_ppr);
    // Accept All merges this paragraph into the carrier and keeps the
    // carrier's properties, so it carries the paragraph's own (#2126
    // review). Its mark, break and recorded change stay apart.
    for child in dom.elements(ppr, None) {
        if dom.name_is(child, &W::r_pr())
            || dom.name_is(child, &W::sect_pr())
            || dom.name_is(child, &W::p_pr_change())
        {
            continue;
        }
        let copy = dom.clone_subtree(child);
        dom.add(blank_ppr, copy);
    }
    let blank_rpr = dom.new_element(W::r_pr());
    dom.add(blank_ppr, blank_rpr);
    push_paragraph_mark(dom, blank_rpr, true, inserted_id, author, date);
    dom.remove(section);
    dom.add(blank_ppr, section);
    dom.add_after_self(paragraph, blank);
}

/// Track a section break the revision adds, when the same paragraphs are
/// still in order on both sides. The closing paragraph keeps the break
/// under an inserted mark and an empty paragraph follows it. A break
/// before a later paragraph deletes this mark and the break moves onto
/// that new empty paragraph.
fn track_added_section_breaks(
    dom: &mut Dom,
    result_root: NodeId,
    original_body: NodeId,
    revised_body: NodeId,
    settings: &WmlComparerSettings,
) {
    if !settings.merge_replaced_paragraphs {
        return;
    }
    let Some(result_body) = dom.element(result_root, &W::body()) else {
        return;
    };
    let original = section_rows(dom, original_body);
    let revised = section_rows(dom, revised_body);
    let result: Vec<NodeId> = dom
        .elements(result_body, Some(&W::p()))
        .into_iter()
        .filter(|&paragraph| !paragraph_plain_text(dom, paragraph).is_empty())
        .collect();
    if original.len() != revised.len() || original.len() != result.len() {
        return;
    }
    let mut next_id = next_revision_id(dom, result_root);
    for index in (0..result.len()).rev() {
        if original[index].text != revised[index].text
            || original[index].text != paragraph_plain_text(dom, result[index])
        {
            continue;
        }
        if !revised[index].authored || original[index].authored {
            continue;
        }
        if has_paragraph_mark(dom, result[index]) {
            continue;
        }
        let author = settings.author_for_revisions.as_str();
        let date = settings.date_time_for_revisions.as_str();
        if !original[index].followed {
            track_closing_added_break(dom, result[index], next_id, author, date);
            next_id = next_id.saturating_add(1);
        } else {
            let inserted_id = next_id.saturating_add(1);
            track_middle_added_break(dom, result[index], next_id, inserted_id, author, date);
            next_id = inserted_id.saturating_add(1);
        }
    }
}
