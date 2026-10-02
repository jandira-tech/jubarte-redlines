// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Document settings an edit plan may set: `w:trackRevisions`,
//! `w:updateFields` and `w:documentProtection`, each written at its place in
//! the `CT_Settings` sequence (Word refuses a settings part out of order).
//! The settings part is created when the document has none.

use serde::{Deserialize, Serialize};

use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const SETTINGS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";
const SETTINGS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml";

/// `CT_Settings` children in schema order, as `(prefix, local name)`.
/// `tests/schema_consistency.rs` holds it equal to `wml_main_schema.json`.
pub const SETTINGS_ORDER: &[(&str, &str)] = &[
    ("w", "writeProtection"),
    ("w", "view"),
    ("w", "zoom"),
    ("w", "removePersonalInformation"),
    ("w", "removeDateAndTime"),
    ("w", "doNotDisplayPageBoundaries"),
    ("w", "displayBackgroundShape"),
    ("w", "printPostScriptOverText"),
    ("w", "printFractionalCharacterWidth"),
    ("w", "printFormsData"),
    ("w", "embedTrueTypeFonts"),
    ("w", "embedSystemFonts"),
    ("w", "saveSubsetFonts"),
    ("w", "saveFormsData"),
    ("w", "mirrorMargins"),
    ("w", "alignBordersAndEdges"),
    ("w", "bordersDoNotSurroundHeader"),
    ("w", "bordersDoNotSurroundFooter"),
    ("w", "gutterAtTop"),
    ("w", "hideSpellingErrors"),
    ("w", "hideGrammaticalErrors"),
    ("w", "activeWritingStyle"),
    ("w", "proofState"),
    ("w", "formsDesign"),
    ("w", "attachedTemplate"),
    ("w", "linkStyles"),
    ("w", "stylePaneFormatFilter"),
    ("w", "stylePaneSortMethod"),
    ("w", "documentType"),
    ("w", "mailMerge"),
    ("w", "revisionView"),
    ("w", "trackRevisions"),
    ("w", "doNotTrackMoves"),
    ("w", "doNotTrackFormatting"),
    ("w", "documentProtection"),
    ("w", "autoFormatOverride"),
    ("w", "styleLockTheme"),
    ("w", "styleLockQFSet"),
    ("w", "defaultTabStop"),
    ("w", "autoHyphenation"),
    ("w", "consecutiveHyphenLimit"),
    ("w", "hyphenationZone"),
    ("w", "doNotHyphenateCaps"),
    ("w", "showEnvelope"),
    ("w", "summaryLength"),
    ("w", "clickAndTypeStyle"),
    ("w", "defaultTableStyle"),
    ("w", "evenAndOddHeaders"),
    ("w", "bookFoldRevPrinting"),
    ("w", "bookFoldPrinting"),
    ("w", "bookFoldPrintingSheets"),
    ("w", "drawingGridHorizontalSpacing"),
    ("w", "drawingGridVerticalSpacing"),
    ("w", "displayHorizontalDrawingGridEvery"),
    ("w", "displayVerticalDrawingGridEvery"),
    ("w", "doNotUseMarginsForDrawingGridOrigin"),
    ("w", "drawingGridHorizontalOrigin"),
    ("w", "drawingGridVerticalOrigin"),
    ("w", "doNotShadeFormData"),
    ("w", "noPunctuationKerning"),
    ("w", "characterSpacingControl"),
    ("w", "printTwoOnOne"),
    ("w", "strictFirstAndLastChars"),
    ("w", "noLineBreaksAfter"),
    ("w", "noLineBreaksBefore"),
    ("w", "savePreviewPicture"),
    ("w", "doNotValidateAgainstSchema"),
    ("w", "saveInvalidXml"),
    ("w", "ignoreMixedContent"),
    ("w", "alwaysShowPlaceholderText"),
    ("w", "doNotDemarcateInvalidXml"),
    ("w", "saveXmlDataOnly"),
    ("w", "useXSLTWhenSaving"),
    ("w", "saveThroughXslt"),
    ("w", "showXMLTags"),
    ("w", "alwaysMergeEmptyNamespace"),
    ("w", "updateFields"),
    ("w", "hdrShapeDefaults"),
    ("w", "footnotePr"),
    ("w", "endnotePr"),
    ("w", "compat"),
    ("w", "docVars"),
    ("w", "rsids"),
    ("m", "mathPr"),
    ("w", "uiCompat97To2003"),
    ("w", "attachedSchema"),
    ("w", "themeFontLang"),
    ("w", "clrSchemeMapping"),
    ("w", "doNotIncludeSubdocsInStats"),
    ("w", "doNotAutoCompressPictures"),
    ("w", "forceUpgrade"),
    ("w", "captions"),
    ("w", "readModeInkLockDown"),
    ("sl", "schemaLibrary"),
    ("w", "shapeDefaults"),
    ("w", "decimalSymbol"),
    ("w", "listSeparator"),
    ("w14", "docId"),
    ("w14", "discardImageEditingData"),
    ("w14", "defaultImageDpi"),
    ("w14", "conflictMode"),
    ("w15", "chartTrackingRefBased"),
    ("w15", "docId"),
];

fn namespace(prefix: &str) -> &'static str {
    match prefix {
        "m" => "http://schemas.openxmlformats.org/officeDocument/2006/math",
        "sl" => "http://schemas.openxmlformats.org/schemaLibrary/2006/main",
        "w14" => "http://schemas.microsoft.com/office/word/2010/wordml",
        "w15" => "http://schemas.microsoft.com/office/word/2012/wordml",
        _ => W,
    }
}

/// `ST_DocProtect`: what an editor may still change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtectionEdit {
    /// No protection: the element is removed.
    None,
    /// Nothing may change.
    ReadOnly,
    /// Only comments may be added.
    Comments,
    /// Every change is tracked.
    TrackedChanges,
    /// Only form fields may be filled.
    Forms,
}

impl ProtectionEdit {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ReadOnly => "readOnly",
            Self::Comments => "comments",
            Self::TrackedChanges => "trackedChanges",
            Self::Forms => "forms",
        }
    }
}

/// `w:documentProtection`. Without a password this is Word's protection
/// enforced without a password, which any user can turn off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protection {
    /// What may still change.
    pub edit: ProtectionEdit,
    /// Enforce it (default) or only record it.
    #[serde(default = "enforced")]
    pub enforcement: bool,
    /// Refused (`UNSUPPORTED`): Word's password hash is not written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

fn enforced() -> bool {
    true
}

/// The settings one `settings` operation sets; `None` leaves one alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsRequest {
    /// `w:trackRevisions`: present when true, removed when false.
    pub track_revisions: Option<bool>,
    /// `w:updateFields`: present when true, removed when false.
    pub update_fields: Option<bool>,
    /// `w:documentProtection`; `edit: none` removes it.
    pub protection: Option<Protection>,
}

/// Write `request` into the settings part of the package whose main part
/// is `main`, creating the part (with its relationship and content type)
/// when there is none.
pub fn apply_settings(pkg: &mut PartFs, main: &str, request: &SettingsRequest) {
    let part = settings_part(pkg, main);
    let xml = pkg
        .part_string(&part)
        .unwrap_or_else(|| format!(r#"<w:settings xmlns:w="{W}"/>"#));
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    if let Some(on) = request.track_revisions {
        toggle(&mut dom, root, "trackRevisions", on);
    }
    if let Some(on) = request.update_fields {
        toggle(&mut dom, root, "updateFields", on);
    }
    if let Some(p) = &request.protection {
        if p.edit == ProtectionEdit::None {
            remove_child(&mut dom, root, "documentProtection");
        } else {
            let e = set_child(&mut dom, root, "documentProtection");
            let enforcement = if p.enforcement { "1" } else { "0" };
            dom.set_attribute_value(e, &XName::get("edit", W), Some(p.edit.as_str()));
            dom.set_attribute_value(e, &XName::get("enforcement", W), Some(enforcement));
        }
    }
    pkg.set_part(&part, dom.serialize_document(doc).into_bytes());
}

/// The settings part's name, after adding the part's relationship and
/// content type when the package has none.
fn settings_part(pkg: &mut PartFs, main: &str) -> String {
    let target = pkg.read_rels_for(main).and_then(|rels| {
        rels.items
            .iter()
            .find(|r| r.rel_type == SETTINGS_REL)
            .map(|r| r.target.clone())
    });
    if let Some(target) = target {
        return pkg.resolve_rel_target(main, &target);
    }
    let dir = main.rsplit_once('/').map_or("", |(d, _)| d);
    let part = if dir.is_empty() {
        "settings.xml".to_string()
    } else {
        format!("{dir}/settings.xml")
    };
    pkg.add_content_type_override(&format!("/{part}"), SETTINGS_CT);
    pkg.add_document_relationship(main, SETTINGS_REL, "settings.xml");
    part
}

fn toggle(dom: &mut Dom, root: NodeId, local: &str, on: bool) {
    if on {
        let e = set_child(dom, root, local);
        // An empty element is on; drop an explicit `w:val` so it cannot
        // say otherwise.
        dom.set_attribute_value(e, &XName::get("val", W), None);
    } else {
        remove_child(dom, root, local);
    }
}

fn rank(dom: &Dom, node: NodeId) -> Option<usize> {
    let name = dom.name(node)?;
    SETTINGS_ORDER.iter().position(|(prefix, local)| {
        name.local_name() == *local && name.namespace_name() == namespace(prefix)
    })
}

/// The `w:` child `local` of the settings root: the existing one, or a new
/// empty one inserted before the first child that ranks after it.
pub fn set_child(dom: &mut Dom, root: NodeId, local: &str) -> NodeId {
    let name = XName::get(local, W);
    if let Some(e) = dom.element(root, &name) {
        return e;
    }
    let mine = SETTINGS_ORDER
        .iter()
        .position(|(p, l)| *p == "w" && *l == local)
        .unwrap_or(usize::MAX);
    let next = dom
        .elements(root, None)
        .into_iter()
        .find(|&c| rank(dom, c).is_some_and(|r| r > mine));
    let e = dom.new_element(name);
    match next {
        Some(next) => dom.add_before_self(next, e),
        None => dom.add(root, e),
    }
    e
}

/// Remove every `w:` child `local` of the settings root.
pub fn remove_child(dom: &mut Dom, root: NodeId, local: &str) {
    for e in dom.elements(root, Some(&XName::get(local, W))) {
        dom.remove(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_of(xml: &str) -> Vec<String> {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).unwrap();
        set_child(&mut dom, root, "trackRevisions");
        dom.elements(root, None)
            .into_iter()
            .map(|e| dom.name(e).unwrap().local_name().to_string())
            .collect()
    }

    #[test]
    fn a_new_child_lands_before_the_first_that_ranks_after_it_and_unknowns_are_skipped() {
        let xml = format!(
            r#"<w:settings xmlns:w="{W}" xmlns:v="urn:x"><w:zoom/><v:vendor/><w:compat/></w:settings>"#
        );
        assert_eq!(
            order_of(&xml),
            ["zoom", "vendor", "trackRevisions", "compat"]
        );
        let xml = format!(r#"<w:settings xmlns:w="{W}"><w:zoom/></w:settings>"#);
        assert_eq!(order_of(&xml), ["zoom", "trackRevisions"]);
    }

    #[test]
    fn every_edit_value_has_its_schema_spelling() {
        let all = [
            ProtectionEdit::None,
            ProtectionEdit::ReadOnly,
            ProtectionEdit::Comments,
            ProtectionEdit::TrackedChanges,
            ProtectionEdit::Forms,
        ];
        for e in all {
            let json = serde_json::to_string(&e).unwrap();
            assert_eq!(json, format!("\"{}\"", e.as_str()));
        }
        assert_eq!(namespace("w"), W);
    }
}
