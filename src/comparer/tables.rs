// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Element-name tables that drive atomization, hashing, and cloning.
//!
//! Faithful to `WmlComparer.ts`: `WordBreakElements` (:8469),
//! `AllowableRunChildren` (:8998), `ElementsToThrowAway` (:9023),
//! `ElementsToHaveSha1Hash` (:9045), `InvalidElements` (:9055),
//! `ComparisonGroupingElements` (:9071), `RecursionElements` (:9074),
//! `AttributesToTrimWhenCloning` (:5133), `s_RelationshipAttributeNames`
//! (:9218), `s_ElementsWithRelationshipIds` (:9182). The compare pipeline
//! in [`super::atomize`], [`super::units`], and [`super::preprocess`]
//! reads these tables at every call; the redline writer at
//! [`super::parts`] and [`super::fixups`] reads the relationship-id sets.
//!
//! Every entry mirrors the C# TS port. Add a name here when a Word feature
//! shows up in tests as "stuck on the atom itself" / "thrown away when it
//! should be hashed" / "tagged by its rel id" — the schema-consistency
//! oracle in `tests/schema_consistency.rs` only checks the rank tables in
//! [`super::order_tables`].

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::namespaces::{A, C, DGM, M, O, R, VML, W, W10, WNE, WP14};
use crate::xmllinq::XName;

fn set(names: &[XName]) -> HashSet<XName> {
    names.iter().cloned().collect()
}

/// `WordBreakElements` (WmlComparer.ts :8469) — a non-`w:t` content atom whose
/// name is in this set forces a word boundary during atomization. The
/// comparer uses this set inside `super::atomize::atom_hash`
/// (via `super::units::is_word_break_element`) so a tab, break, drawing,
/// or footnote/endnote reference starts a new word atom even mid-run.
pub static WORD_BREAK_ELEMENTS: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::p_pr(),
        W::name("tab"),
        W::name("br"),
        W::name("continuationSeparator"),
        W::name("cr"),
        W::name("dayLong"),
        W::name("dayShort"),
        W::drawing(),
        W::pict(),
        W::name("endnoteRef"),
        W::name("footnoteRef"),
        W::name("monthLong"),
        W::name("monthShort"),
        W::name("noBreakHyphen"),
        W::object(),
        W::name("ptab"),
        W::name("separator"),
        W::name("sym"),
        W::name("yearLong"),
        W::name("yearShort"),
        M::name("oMathPara"),
        M::name("oMath"),
        W::name("footnoteReference"),
        W::name("endnoteReference"),
    ])
});

/// `AllowableRunChildren` (WmlComparer.ts :8998) — run-level leaves emitted
/// as a single verbatim atom by `super::atomize::recurse`. `w:object`
/// is intentionally absent; the atomizer has an explicit dispatch arm for
/// it.
pub static ALLOWABLE_RUN_CHILDREN: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::name("br"),
        W::drawing(),
        W::name("cr"),
        W::name("dayLong"),
        W::name("dayShort"),
        W::name("footnoteReference"),
        W::name("endnoteReference"),
        W::name("monthLong"),
        W::name("monthShort"),
        W::name("noBreakHyphen"),
        W::name("pgNum"),
        W::name("ptab"),
        W::name("softHyphen"),
        W::name("sym"),
        W::name("tab"),
        W::name("yearLong"),
        W::name("yearShort"),
        M::name("oMathPara"),
        M::name("oMath"),
        W::fld_char(),
        W::instr_text(),
    ])
});

/// `ElementsToThrowAway` (WmlComparer.ts :9023) — the atomizer descends
/// into these elements (or skips over them) but never emits an atom for
/// them: bookmarks, proofErr, table/section properties, move-range markers,
/// and the legacy separator/continuationSeparator inside a notes part.
pub static ELEMENTS_TO_THROW_AWAY: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::bookmark_start(),
        W::bookmark_end(),
        W::name("commentRangeStart"),
        W::name("commentRangeEnd"),
        W::name("lastRenderedPageBreak"),
        W::name("proofErr"),
        W::tbl_pr(),
        W::sect_pr(),
        W::name("permEnd"),
        W::name("permStart"),
        W::name("footnoteRef"),
        W::name("endnoteRef"),
        W::name("separator"),
        W::name("continuationSeparator"),
        W::move_from_range_start(),
        W::move_from_range_end(),
        W::move_to_range_start(),
        W::move_to_range_end(),
    ])
});

/// `ElementsToHaveSha1Hash` (WmlComparer.ts :9045) — block-level elements
/// that [`super::preprocess::add_sha1_hash_to_block_level_content`]
/// stamps `pt:SHA1Hash` (and `pt:StructureSHA1Hash` for tables/rows) onto.
pub static ELEMENTS_TO_HAVE_SHA1: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::p(),
        W::tbl(),
        W::tr(),
        W::tc(),
        W::drawing(),
        W::pict(),
        W::txbx_content(),
    ])
});

/// `InvalidElements` (WmlComparer.ts :9055) — names that
/// [`super::atomize::verify_no_invalid_content`] rejects before atomizing.
/// `w:moveFrom` / `w:moveTo` are deliberately NOT invalid: the redline
/// keeps moves as their own output shape.
pub static INVALID_ELEMENTS: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::name("altChunk"),
        W::custom_xml(),
        W::name("customXmlDelRangeStart"),
        W::name("customXmlDelRangeEnd"),
        W::name("customXmlInsRangeStart"),
        W::name("customXmlInsRangeEnd"),
        W::name("customXmlMoveFromRangeStart"),
        W::name("customXmlMoveFromRangeEnd"),
        W::name("customXmlMoveToRangeStart"),
        W::name("customXmlMoveToRangeEnd"),
        W::name("subDoc"),
    ])
});

/// `ComparisonGroupingElements` (WmlComparer.ts :9071) — the ancestors
/// [`super::units::get_comparison_unit_list`] walks to build hierarchical
/// keys for paragraph / table / row / cell / text-box groups.
pub static COMPARISON_GROUPING_ELEMENTS: LazyLock<HashSet<XName>> =
    LazyLock::new(|| set(&[W::p(), W::tbl(), W::tr(), W::tc(), W::txbx_content()]));

/// One `RecursionElements` (WmlComparer.ts :9074) entry: an element the
/// atomizer recurses into, plus the property children that
/// `super::atomize::coalesce_recurse` re-attaches structurally after
/// atomization.
pub struct RecursionInfo {
    /// The element the atomizer recurses into.
    pub element_name: XName,
    /// Property children to skip during recursion (rebuilt by Coalesce).
    /// `None` means no skip; a table's `w:tblPr` / `w:tblGrid` / `w:tblPrEx`,
    /// for example, are skipped and reconstructed from the cloned attributes.
    pub child_property_names: Option<Vec<XName>>,
}

/// `RecursionElements` (WmlComparer.ts :9074), in the verbatim C# order the
/// atomizer walks. Look up an element with [`recursion_info`].
pub static RECURSION_ELEMENTS: LazyLock<Vec<RecursionInfo>> = LazyLock::new(|| {
    let mk = |n: XName, props: Option<Vec<XName>>| RecursionInfo {
        element_name: n,
        child_property_names: props,
    };
    vec![
        mk(W::del(), None),
        mk(W::ins(), None),
        mk(W::move_from(), None),
        mk(W::move_to(), None),
        mk(
            W::tbl(),
            Some(vec![W::tbl_pr(), W::name("tblGrid"), W::name("tblPrEx")]),
        ),
        mk(W::tr(), Some(vec![W::tr_pr(), W::name("tblPrEx")])),
        mk(W::tc(), Some(vec![W::tc_pr(), W::name("tblPrEx")])),
        mk(W::pict(), Some(vec![VML::name("shapetype")])),
        mk(VML::name("group"), None),
        mk(VML::name("shape"), None),
        mk(VML::name("rect"), None),
        mk(VML::name("textbox"), None),
        mk(O::name("lock"), None),
        mk(W::txbx_content(), None),
        mk(W10::name("wrap"), None),
        mk(W::sdt(), Some(vec![W::name("sdtPr"), W::name("sdtEndPr")])),
        mk(W::sdt_content(), None),
        mk(W::hyperlink(), None),
        mk(W::name("fldSimple"), None),
        mk(VML::name("shapetype"), None),
        mk(W::smart_tag(), Some(vec![W::name("smartTagPr")])),
        mk(W::name("ruby"), Some(vec![W::name("rubyPr")])),
    ]
});

/// Find the [`RecursionInfo`] entry for an atomizer element name, if any.
pub fn recursion_info(name: &XName) -> Option<&'static RecursionInfo> {
    RECURSION_ELEMENTS
        .iter()
        .find(|ri| &ri.element_name == name)
}

/// `AttributesToTrimWhenCloning` (WmlComparer.ts :5133) — attributes the
/// hashing clone drops by default and in the rel-id branch. Word-instance
/// ids and shape ids: they survive one compare, but two compares see two
/// `wp:editId` strings and break content correlation.
pub static ATTRIBUTES_TO_TRIM_WHEN_CLONING: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        WP14::name("anchorId"),
        WP14::name("editId"),
        XName::get("ObjectID", ""),
        XName::get("ShapeID", ""),
        XName::get("id", ""),
        XName::get("type", ""),
    ])
});

/// `s_RelationshipAttributeNames` (WmlComparer.ts :9218) — every attribute
/// that carries a relationship id. Read by [`super::parts`] when it
/// reconciles dangling rIds and by the rel-id clone branch in
/// [`super::preprocess`].
pub static S_RELATIONSHIP_ATTRIBUTE_NAMES: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        R::name("embed"),
        R::name("link"),
        R::name("id"),
        R::name("cs"),
        R::name("dm"),
        R::name("lo"),
        R::name("qs"),
        R::name("href"),
        R::name("pict"),
    ])
});

/// `s_ElementsWithRelationshipIds` (WmlComparer.ts :9182) — elements
/// whose rId attributes the hashing clone replaces with the referenced
/// part's content hash (so identical media and embedded charts correlate
/// equal across the two documents).
pub static S_ELEMENTS_WITH_RELATIONSHIP_IDS: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        A::name("blip"),
        A::name("hlinkClick"),
        A::name("relIds"),
        C::name("chart"),
        C::name("externalData"),
        C::name("userShapes"),
        DGM::name("relIds"),
        O::name("OLEObject"),
        VML::name("fill"),
        VML::name("imagedata"),
        VML::name("stroke"),
        W::name("altChunk"),
        W::name("attachedTemplate"),
        W::name("control"),
        W::name("dataSource"),
        W::name("embedBold"),
        W::name("embedBoldItalic"),
        W::name("embedItalic"),
        W::name("embedRegular"),
        W::name("footerReference"),
        W::name("headerReference"),
        W::name("headerSource"),
        W::hyperlink(),
        W::name("printerSettings"),
        W::name("recipientData"),
        W::name("saveThroughXslt"),
        W::name("sourceFileName"),
        W::name("src"),
        W::name("subDoc"),
        WNE::name("toolbarData"),
    ])
});
