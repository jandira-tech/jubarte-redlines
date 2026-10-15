// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Document settings an edit plan writes into `word/settings.xml`: Track
//! Changes on or off, `updateFields` on open, and editing restrictions.
//!
//! Every element lands in `CT_Settings` sequence ([`SETTINGS_ORDER`], the
//! schema order held by `tests/schema_consistency.rs` against
//! `tests/data/wml_main_schema.json`), so Word reads the part back. A
//! package without a settings part gets one, with its relationship and
//! content type, the first time a request is applied.
//!
//! Entry points: [`apply_settings`] to write into an open [`PartFs`] given
//! the main part's name, and [`apply_settings_to_docx`] for the byte-for-byte
//! form an edit plan returns. [`SettingsRequest`] is what an operation
//! builds; [`Protection`] and [`ProtectionEdit`] cover `documentProtection`.

use serde::{Deserialize, Serialize};

use crate::namespaces::{M, W, W14, W15};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

const SL: &str = "http://schemas.openxmlformats.org/schemaLibrary/2006/main";
const SETTINGS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";
const SETTINGS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml";

/// The children of `w:settings` in `CT_Settings` sequence, as
/// `(namespace URI, local name)`. New settings go in the position the schema
/// expects; [`set_child`] refuses to leave an unknown name in the middle.
pub const SETTINGS_ORDER: &[(&str, &str)] = &[
    (W::URI, "writeProtection"),
    (W::URI, "view"),
    (W::URI, "zoom"),
    (W::URI, "removePersonalInformation"),
    (W::URI, "removeDateAndTime"),
    (W::URI, "doNotDisplayPageBoundaries"),
    (W::URI, "displayBackgroundShape"),
    (W::URI, "printPostScriptOverText"),
    (W::URI, "printFractionalCharacterWidth"),
    (W::URI, "printFormsData"),
    (W::URI, "embedTrueTypeFonts"),
    (W::URI, "embedSystemFonts"),
    (W::URI, "saveSubsetFonts"),
    (W::URI, "saveFormsData"),
    (W::URI, "mirrorMargins"),
    (W::URI, "alignBordersAndEdges"),
    (W::URI, "bordersDoNotSurroundHeader"),
    (W::URI, "bordersDoNotSurroundFooter"),
    (W::URI, "gutterAtTop"),
    (W::URI, "hideSpellingErrors"),
    (W::URI, "hideGrammaticalErrors"),
    (W::URI, "activeWritingStyle"),
    (W::URI, "proofState"),
    (W::URI, "formsDesign"),
    (W::URI, "attachedTemplate"),
    (W::URI, "linkStyles"),
    (W::URI, "stylePaneFormatFilter"),
    (W::URI, "stylePaneSortMethod"),
    (W::URI, "documentType"),
    (W::URI, "mailMerge"),
    (W::URI, "revisionView"),
    (W::URI, "trackRevisions"),
    (W::URI, "doNotTrackMoves"),
    (W::URI, "doNotTrackFormatting"),
    (W::URI, "documentProtection"),
    (W::URI, "autoFormatOverride"),
    (W::URI, "styleLockTheme"),
    (W::URI, "styleLockQFSet"),
    (W::URI, "defaultTabStop"),
    (W::URI, "autoHyphenation"),
    (W::URI, "consecutiveHyphenLimit"),
    (W::URI, "hyphenationZone"),
    (W::URI, "doNotHyphenateCaps"),
    (W::URI, "showEnvelope"),
    (W::URI, "summaryLength"),
    (W::URI, "clickAndTypeStyle"),
    (W::URI, "defaultTableStyle"),
    (W::URI, "evenAndOddHeaders"),
    (W::URI, "bookFoldRevPrinting"),
    (W::URI, "bookFoldPrinting"),
    (W::URI, "bookFoldPrintingSheets"),
    (W::URI, "drawingGridHorizontalSpacing"),
    (W::URI, "drawingGridVerticalSpacing"),
    (W::URI, "displayHorizontalDrawingGridEvery"),
    (W::URI, "displayVerticalDrawingGridEvery"),
    (W::URI, "doNotUseMarginsForDrawingGridOrigin"),
    (W::URI, "drawingGridHorizontalOrigin"),
    (W::URI, "drawingGridVerticalOrigin"),
    (W::URI, "doNotShadeFormData"),
    (W::URI, "noPunctuationKerning"),
    (W::URI, "characterSpacingControl"),
    (W::URI, "printTwoOnOne"),
    (W::URI, "strictFirstAndLastChars"),
    (W::URI, "noLineBreaksAfter"),
    (W::URI, "noLineBreaksBefore"),
    (W::URI, "savePreviewPicture"),
    (W::URI, "doNotValidateAgainstSchema"),
    (W::URI, "saveInvalidXml"),
    (W::URI, "ignoreMixedContent"),
    (W::URI, "alwaysShowPlaceholderText"),
    (W::URI, "doNotDemarcateInvalidXml"),
    (W::URI, "saveXmlDataOnly"),
    (W::URI, "useXSLTWhenSaving"),
    (W::URI, "saveThroughXslt"),
    (W::URI, "showXMLTags"),
    (W::URI, "alwaysMergeEmptyNamespace"),
    (W::URI, "updateFields"),
    (W::URI, "hdrShapeDefaults"),
    (W::URI, "footnotePr"),
    (W::URI, "endnotePr"),
    (W::URI, "compat"),
    (W::URI, "docVars"),
    (W::URI, "rsids"),
    (M::URI, "mathPr"),
    (W::URI, "uiCompat97To2003"),
    (W::URI, "attachedSchema"),
    (W::URI, "themeFontLang"),
    (W::URI, "clrSchemeMapping"),
    (W::URI, "doNotIncludeSubdocsInStats"),
    (W::URI, "doNotAutoCompressPictures"),
    (W::URI, "forceUpgrade"),
    (W::URI, "captions"),
    (W::URI, "readModeInkLockDown"),
    (SL, "schemaLibrary"),
    (W::URI, "shapeDefaults"),
    (W::URI, "decimalSymbol"),
    (W::URI, "listSeparator"),
    (W14::URI, "docId"),
    (W14::URI, "discardImageEditingData"),
    (W14::URI, "defaultImageDpi"),
    (W14::URI, "conflictMode"),
    (W15::URI, "chartTrackingRefBased"),
    (W15::URI, "docId"),
];

/// What `w:documentProtection` allows (`ST_DocProtect`).
///
/// [`Self::None`] removes the element entirely: the document is unrestricted.
/// The other variants are the four values Word's Restrict Editing pane lets
/// you pick (`readOnly`, `comments`, `trackedChanges`, `forms`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtectionEdit {
    /// No restriction: the element is removed.
    None,
    /// No changes.
    ReadOnly,
    /// Comments only.
    Comments,
    /// Every change is tracked.
    TrackedChanges,
    /// Filling in forms only.
    Forms,
}

impl ProtectionEdit {
    /// The exact `w:edit` value Word writes on the wire
    /// (`"none"`, `"readOnly"`, `"comments"`, `"trackedChanges"`,
    /// `"forms"`).
    fn value(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ReadOnly => "readOnly",
            Self::Comments => "comments",
            Self::TrackedChanges => "trackedChanges",
            Self::Forms => "forms",
        }
    }
}

fn default_true() -> bool {
    true
}

/// An editing restriction. Without a password this is what Word's
/// Restrict Editing pane calls "No password (just this single user can
/// remove the restriction)": the restriction is checked but any user can
/// turn it off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protection {
    /// What the restriction allows (`ST_DocProtect`).
    pub edit: ProtectionEdit,
    #[serde(default = "default_true")]
    /// `w:enforcement="1"` when true. False records the element
    /// unenforced; Word shows it greyed out in the Restrict Editing pane.
    pub enforcement: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Word's legacy password hash. Always refused: a hashed password is
    /// never written, because the matcher has nothing to compare against.
    pub password: Option<String>,
}

/// The settings one plan operation writes. `None` on a field leaves the
/// existing value on the wire; `Some(true)` adds the element,
/// `Some(false)` removes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsRequest {
    /// `w:trackRevisions`: present when true, removed when false.
    pub track_revisions: Option<bool>,
    /// `w:updateFields`: present when true, removed when false.
    pub update_fields: Option<bool>,
    /// `w:documentProtection`, replaced whole; `edit: none` removes it.
    pub protection: Option<Protection>,
}

impl SettingsRequest {
    /// A short human-readable description of what this request writes,
    /// for a report line: never the password; protection prints its edit
    /// value and whether it is enforced.
    #[must_use]
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(on) = self.track_revisions {
            parts.push(format!("track_revisions={on}"));
        }
        if let Some(on) = self.update_fields {
            parts.push(format!("update_fields={on}"));
        }
        if let Some(p) = &self.protection {
            let enforced = if p.enforcement {
                "enforced"
            } else {
                "not enforced"
            };
            parts.push(format!("protection={} {enforced}", p.edit.value()));
        }
        format!("{{settings {}}}", parts.join(" "))
    }
}

/// The rank of `node` in [`SETTINGS_ORDER`], if it is a known child.
fn rank(dom: &Dom, node: NodeId) -> Option<usize> {
    let name = dom.name(node)?;
    SETTINGS_ORDER
        .iter()
        .position(|&(ns, local)| name.namespace_name() == ns && name.local_name() == local)
}

/// Insert or replace `<w:local attrs/>` under `root`, ordered against the
/// children already there by the `CT_Settings` sequence
/// ([`SETTINGS_ORDER`]). Elements that are schema-incorrect (an unknown
/// local, the wrong namespace) stay where their position lets them.
pub fn set_child(dom: &mut Dom, root: NodeId, local: &str, attrs: &[(&str, &str)]) -> NodeId {
    remove_child(dom, root, local);
    let ours = SETTINGS_ORDER
        .iter()
        .position(|&(ns, l)| ns == W::URI && l == local)
        .expect("a CT_Settings child");
    let node = dom.new_element(W::name(local));
    for (name, value) in attrs {
        dom.set_attribute_value(node, &W::name(name), Some(value));
    }
    let next = dom
        .elements(root, None)
        .into_iter()
        .find(|&child| rank(dom, child).is_some_and(|r| r > ours));
    match next {
        Some(next) => dom.add_before_self(next, node),
        None => dom.add(root, node),
    }
    node
}

/// Remove every `<w:local>` child of `root` (none or many). No-op when the
/// element is absent.
pub fn remove_child(dom: &mut Dom, root: NodeId, local: &str) {
    let name: XName = W::name(local);
    for child in dom.elements(root, Some(&name)) {
        dom.remove(child);
    }
}

/// Apply `request` to the settings root.
fn apply_to_root(dom: &mut Dom, root: NodeId, request: &SettingsRequest) {
    for (local, on) in [
        ("trackRevisions", request.track_revisions),
        ("updateFields", request.update_fields),
    ] {
        match on {
            Some(true) => {
                set_child(dom, root, local, &[]);
            }
            Some(false) => remove_child(dom, root, local),
            None => {}
        }
    }
    if let Some(protection) = &request.protection {
        if protection.edit == ProtectionEdit::None {
            remove_child(dom, root, "documentProtection");
        } else {
            let enforcement = if protection.enforcement { "1" } else { "0" };
            set_child(
                dom,
                root,
                "documentProtection",
                &[
                    ("edit", protection.edit.value()),
                    ("enforcement", enforcement),
                ],
            );
        }
    }
}

/// The main part's settings part, created with its relationship and
/// content type when the package has none.
fn settings_part(pkg: &mut PartFs, main: &str) -> String {
    let existing = pkg.read_rels_for(main).and_then(|rels| {
        rels.items
            .iter()
            .find(|r| r.rel_type == SETTINGS_REL && r.target_mode.as_deref() != Some("External"))
            .map(|r| r.target.clone())
    });
    if let Some(target) = existing {
        return pkg.resolve_rel_target(main, &target);
    }
    let part = match main.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/settings.xml"),
        None => "settings.xml".to_string(),
    };
    if pkg.part_bytes(&part).is_none() {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="{}"/>"#,
            W::URI
        );
        pkg.set_part(&part, xml.into_bytes());
    }
    pkg.add_content_type_override(&format!("/{part}"), SETTINGS_CT);
    pkg.add_document_relationship(main, SETTINGS_REL, "settings.xml");
    part
}

/// Apply `request` into the package's settings part, creating it with its
/// relationship and content type when the package has none.
///
/// # Errors
///
/// A message when the part the main document rels name is not a
/// `w:settings` part (the package is corrupt or the part has been edited
/// out of shape).
pub fn apply_settings(
    pkg: &mut PartFs,
    main: &str,
    request: &SettingsRequest,
) -> Result<(), String> {
    let part = settings_part(pkg, main);
    let xml = pkg.part_string(&part).unwrap_or_default();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom
        .root(doc)
        .filter(|&r| dom.name_is(r, &W::name("settings")))
        .ok_or_else(|| format!("{part} is not a w:settings part"))?;
    apply_to_root(&mut dom, root, request);
    pkg.set_part(&part, dom.serialize_document(doc).into_bytes());
    Ok(())
}

/// The children of `w:compat` that come before `w:doNotExpandShiftReturn`
/// in `CT_Compat`.
const BEFORE_SHIFT_RETURN: [&str; 10] = [
    "useSingleBorderforContiguousCells",
    "wpJustification",
    "noTabHangInd",
    "noLeading",
    "spaceForUL",
    "noColumnBalance",
    "balanceSingleByteDoubleByteWidth",
    "noExtraLineSpacing",
    "doNotLeaveBackslashAlone",
    "ulTrailSpace",
];

/// Word's "Don't justify lines ending in soft line break"
/// (`w:compat/w:doNotExpandShiftReturn`, ECMA-376 17.15.3.10) turned on
/// under the settings `root`, in schema order.
fn do_not_expand_shift_return(dom: &mut Dom, root: NodeId) {
    let compat = match dom.elements(root, Some(&W::name("compat"))).first() {
        Some(&compat) => compat,
        None => set_child(dom, root, "compat", &[]),
    };
    for old in dom.elements(compat, Some(&W::name("doNotExpandShiftReturn"))) {
        dom.remove(old);
    }
    let node = dom.new_element(W::name("doNotExpandShiftReturn"));
    let next = dom.elements(compat, None).into_iter().find(|&child| {
        !dom.name(child).is_some_and(|name| {
            name.namespace_name() == W::URI && BEFORE_SHIFT_RETURN.contains(&name.local_name())
        })
    });
    match next {
        Some(next) => dom.add_before_self(next, node),
        None => dom.add(compat, node),
    }
}

/// [`do_not_expand_shift_return`] on a whole package: a justified line
/// that ends in a line break is not stretched to the margin.
pub(crate) fn no_stretched_line_breaks(docx: &[u8]) -> Result<Vec<u8>, String> {
    let mut pkg = PartFs::open(docx).map_err(|e| e.to_string())?;
    let main = pkg
        .main_document_part()
        .ok_or("the package has no main document part")?;
    let part = settings_part(&mut pkg, &main);
    let xml = pkg.part_string(&part).unwrap_or_default();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom
        .root(doc)
        .filter(|&r| dom.name_is(r, &W::name("settings")))
        .ok_or_else(|| format!("{part} is not a w:settings part"))?;
    do_not_expand_shift_return(&mut dom, root);
    pkg.set_part(&part, dom.serialize_document(doc).into_bytes());
    pkg.to_zip().map_err(|e| e.to_string())
}

/// [`apply_settings`] on a whole package.
pub fn apply_settings_to_docx(docx: &[u8], request: &SettingsRequest) -> Result<Vec<u8>, String> {
    let mut pkg = PartFs::open(docx).map_err(|e| e.to_string())?;
    let main = pkg
        .main_document_part()
        .ok_or("the package has no main document part")?;
    apply_settings(&mut pkg, &main, request)?;
    pkg.to_zip().map_err(|e| e.to_string())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn settings(children: &str) -> (Dom, NodeId, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:settings xmlns:w="{}">{children}</w:settings>"#,
            W::URI
        ));
        let root = dom.root(doc).unwrap();
        (dom, doc, root)
    }

    #[test]
    fn each_protection_is_spelled_on_the_wire_as_in_the_xml() {
        for edit in [
            ProtectionEdit::None,
            ProtectionEdit::ReadOnly,
            ProtectionEdit::Comments,
            ProtectionEdit::TrackedChanges,
            ProtectionEdit::Forms,
        ] {
            let wire = serde_json::to_string(&edit).unwrap();
            assert_eq!(wire, format!("\"{}\"", edit.value()), "{edit:?}");
            assert_eq!(serde_json::from_str::<ProtectionEdit>(&wire).unwrap(), edit);
        }
    }

    #[test]
    fn a_shift_return_flag_lands_in_compat_in_schema_order() {
        for (compat, expected) in [
            (
                r#"<w:compat><w:ulTrailSpace/><w:spacingInWholePoints/><w:compatSetting w:name="x" w:uri="u" w:val="1"/></w:compat>"#,
                r#"<w:compat><w:ulTrailSpace /><w:doNotExpandShiftReturn /><w:spacingInWholePoints /><w:compatSetting"#,
            ),
            (
                r#"<w:compat><w:doNotExpandShiftReturn w:val="0"/></w:compat>"#,
                "<w:compat><w:doNotExpandShiftReturn /></w:compat>",
            ),
            (
                r#"<w:zoom w:percent="90"/><w:rsids/>"#,
                r#"<w:zoom w:percent="90" /><w:compat><w:doNotExpandShiftReturn /></w:compat><w:rsids />"#,
            ),
        ] {
            let (mut dom, doc, root) = settings(compat);
            do_not_expand_shift_return(&mut dom, root);
            let xml = dom.serialize_document(doc);
            assert!(xml.contains(expected), "{xml}");
        }
    }

    #[test]
    fn a_child_lands_before_the_first_later_one_and_after_unknown_ones() {
        let (mut dom, doc, root) =
            settings(r#"<w:zoom w:percent="90"/><x:vendor xmlns:x="urn:x"/><w:compat/>"#);
        set_child(&mut dom, root, "updateFields", &[]);
        set_child(&mut dom, root, "trackRevisions", &[]);
        let xml = dom.serialize_document(doc);
        let at = |s: &str| xml.find(s).unwrap();
        assert!(at("zoom") < at("trackRevisions"), "{xml}");
        assert!(at("vendor") < at("trackRevisions"), "{xml}");
        assert!(at("trackRevisions") < at("updateFields"), "{xml}");
        assert!(at("updateFields") < at("compat"), "{xml}");
    }

    #[test]
    fn a_child_with_no_later_one_is_appended() {
        let (mut dom, doc, root) = settings(r#"<w:zoom w:percent="90"/>"#);
        set_child(&mut dom, root, "updateFields", &[]);
        let xml = dom.serialize_document(doc);
        assert!(xml.ends_with("<w:updateFields /></w:settings>"), "{xml}");
    }

    #[test]
    fn the_description_names_each_setting_given() {
        let request = SettingsRequest {
            track_revisions: Some(true),
            update_fields: None,
            protection: Some(Protection {
                edit: ProtectionEdit::Forms,
                enforcement: false,
                password: None,
            }),
        };
        assert_eq!(
            request.describe(),
            "{settings track_revisions=true protection=forms not enforced}"
        );
    }

    #[test]
    fn a_part_that_is_not_settings_is_refused() {
        let mut pkg = PartFs::open(
            &crate::markdown::markdown_to_docx("x", &Default::default())
                .unwrap()
                .docx,
        )
        .unwrap();
        let main = pkg.main_document_part().unwrap();
        let part = settings_part(&mut pkg, &main);
        pkg.set_part(&part, b"<nope/>".to_vec());
        let e = apply_settings(&mut pkg, &main, &SettingsRequest::default()).unwrap_err();
        assert!(e.contains("not a w:settings part"), "{e}");
    }
}
