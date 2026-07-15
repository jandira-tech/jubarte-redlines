//! M4.A.1 — the static element-name tables that drive atomization.
//! Faithful to WmlComparer.ts: WordBreakElements (:8469), AllowableRunChildren
//! (:8998), ElementsToThrowAway (:9023), ElementsToHaveSha1Hash (:9045),
//! InvalidElements (:9055), ComparisonGroupingElements (:9071), and
//! RecursionElements (:9074).

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::namespaces::{A, C, DGM, M, O, R, VML, W, W10, WNE, WP14};
use crate::xmllinq::XName;

fn set(names: &[XName]) -> HashSet<XName> {
    names.iter().cloned().collect()
}

/// `WordBreakElements` (:8469) — a non-`w:t` content atom whose name is here
/// forces a word boundary.
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

/// `AllowableRunChildren` (:8998) — run-level leaves emitted as single verbatim
/// atoms. NOTE: `w:object` is NOT here (handled by an explicit dispatch arm).
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
        W::name("fldChar"),
        W::name("instrText"),
    ])
});

/// `ElementsToThrowAway` (:9023) — produce no atoms.
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
        W::name("moveFromRangeEnd"),
        W::move_to_range_start(),
        W::name("moveToRangeEnd"),
    ])
});

/// `ElementsToHaveSha1Hash` (:9045) — get `pt:SHA1Hash` stamped.
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

/// `InvalidElements` (:9055) — cause `VerifyNoInvalidContent` to throw. NOTE:
/// `w:moveFrom`/`w:moveTo` are explicitly NOT invalid.
pub static INVALID_ELEMENTS: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::name("altChunk"),
        W::name("customXml"),
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

/// `ComparisonGroupingElements` (:9071) — ancestors used for hierarchical keys.
pub static COMPARISON_GROUPING_ELEMENTS: LazyLock<HashSet<XName>> = LazyLock::new(|| {
    set(&[
        W::p(),
        W::tbl(),
        W::tr(),
        W::tc(),
        W::txbx_content(),
    ])
});

/// One `RecursionElements` (:9074) entry: an element that recurses into children
/// while skipping the named property children (rebuilt structurally in Coalesce).
pub struct RecursionInfo {
    pub element_name: XName,
    pub child_property_names: Option<Vec<XName>>,
}

/// `RecursionElements` (:9074), verbatim order.
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
            Some(vec![
                W::tbl_pr(),
                W::name("tblGrid"),
                W::name("tblPrEx"),
            ]),
        ),
        mk(
            W::tr(),
            Some(vec![W::tr_pr(), W::name("tblPrEx")]),
        ),
        mk(
            W::tc(),
            Some(vec![W::tc_pr(), W::name("tblPrEx")]),
        ),
        mk(W::pict(), Some(vec![VML::name("shapetype")])),
        mk(VML::name("group"), None),
        mk(VML::name("shape"), None),
        mk(VML::name("rect"), None),
        mk(VML::name("textbox"), None),
        mk(O::name("lock"), None),
        mk(W::txbx_content(), None),
        mk(W10::name("wrap"), None),
        mk(
            W::name("sdt"),
            Some(vec![W::name("sdtPr"), W::name("sdtEndPr")]),
        ),
        mk(W::name("sdtContent"), None),
        mk(W::name("hyperlink"), None),
        mk(W::name("fldSimple"), None),
        mk(VML::name("shapetype"), None),
        mk(W::name("smartTag"), Some(vec![W::name("smartTagPr")])),
        mk(W::name("ruby"), Some(vec![W::name("rubyPr")])),
    ]
});

/// Look up a `RecursionInfo` by element name.
pub fn recursion_info(name: &XName) -> Option<&'static RecursionInfo> {
    RECURSION_ELEMENTS
        .iter()
        .find(|ri| &ri.element_name == name)
}

/// `AttributesToTrimWhenCloning` (:5133) — attributes dropped by the default and
/// rel-id clone branches.
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

/// `s_RelationshipAttributeNames` (:9218) — attributes that carry an rId.
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

/// `s_ElementsWithRelationshipIds` (:9182) — elements whose rId attributes are
/// hashed (replaced by the referenced part's content hash) during diffing.
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
        W::name("hyperlink"),
        W::name("printerSettings"),
        W::name("recipientData"),
        W::name("saveThroughXslt"),
        W::name("sourceFileName"),
        W::name("src"),
        W::name("subDoc"),
        WNE::name("toolbarData"),
    ])
});
