//! Word-validity normalization (OpenXmlValidator-driven, real-Word-arbitrated).
//!
//! The 166-pair sweep with DocumentFormat.OpenXml's validator showed 146/166
//! of our outputs carrying schema errors while Word's own redlines validate
//! clean (sans one math quirk) — and the strict01 family actually trips
//! Word's "unreadable content" repair dialog. Classes fixed here:
//!   1. pPr/rPr/tblPr/tcPr children out of schema order — C# runs
//!      `WmlOrderElementsPerStandard` in the produce path (WmlComparer.cs:1893);
//!      the port was MISSING it (faithfulness gap, ungated).
//!   2. Strict-conversion artifacts: cnfStyle without the required `val`
//!      bitmask, wp14 percentage element values ("20%" vs per-thousand ints).
//!   3. w14:paraId/textId values ≥ 0x80000000 (the `id-paraid-overflow`
//!      corpus passthrough) — Word regenerates its own; strip out-of-range.

use jubarte::comparer::finalize::{fix_strict_validity_artifacts, wml_order_elements_per_standard};
use jubarte::namespaces::{W, W14};
use jubarte::xmllinq::{Dom, NodeId};

const W14_URI: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
const WP14_URI: &str = "http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing";

fn parse(dom: &mut Dom, xml: &str) -> NodeId {
    let d = dom.parse_xdocument(xml);
    dom.root(d).unwrap()
}

fn child_locals(dom: &Dom, el: NodeId) -> Vec<String> {
    dom.elements(el, None)
        .into_iter()
        .filter_map(|c| dom.name(c).map(|n| n.local_name().to_string()))
        .collect()
}

#[test]
fn t1_ppr_children_ordered_per_standard() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<w:p xmlns:w=\"{w}\"><w:r><w:t>x</w:t></w:r><w:pPr>\
             <w:numPr><w:ilvl w:val=\"1\"/></w:numPr>\
             <w:jc w:val=\"center\"/>\
             <w:pStyle w:val=\"ListParagraph\"/>\
             <w:rPr><w:b/><w:ins w:id=\"1\"/></w:rPr>\
             <w:spacing w:after=\"0\"/>\
             </w:pPr></w:p>",
            w = W::URI
        ),
    );
    wml_order_elements_per_standard(&mut dom, root);
    // w:p puts pPr FIRST; pPr children per Order_pPr; rPr children per Order_rPr
    assert_eq!(child_locals(&dom, root), vec!["pPr", "r"]);
    let ppr = dom.element(root, &W::p_pr()).unwrap();
    assert_eq!(
        child_locals(&dom, ppr),
        vec!["pStyle", "numPr", "spacing", "jc", "rPr"],
        "{}",
        dom.serialize_element(root)
    );
    let rpr = dom.element(ppr, &W::r_pr()).unwrap();
    assert_eq!(child_locals(&dom, rpr), vec!["ins", "b"]);
}

#[test]
fn t2_tcpr_and_tblpr_ordered() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<w:tbl xmlns:w=\"{w}\"><w:tblPr>\
             <w:tblLook w:val=\"04A0\"/><w:tblW w:w=\"0\" w:type=\"auto\"/><w:tblStyle w:val=\"T\"/>\
             </w:tblPr><w:tr><w:tc><w:tcPr>\
             <w:vAlign w:val=\"center\"/><w:tcW w:w=\"100\" w:type=\"dxa\"/><w:gridSpan w:val=\"2\"/>\
             </w:tcPr><w:p/></w:tc></w:tr></w:tbl>",
            w = W::URI
        ),
    );
    wml_order_elements_per_standard(&mut dom, root);
    let tblpr = dom.element(root, &W::name("tblPr")).unwrap();
    assert_eq!(
        child_locals(&dom, tblpr),
        vec!["tblStyle", "tblW", "tblLook"]
    );
    let tcpr = dom
        .descendants(root, Some(&W::name("tcPr")))
        .first()
        .copied()
        .unwrap();
    assert_eq!(child_locals(&dom, tcpr), vec!["tcW", "gridSpan", "vAlign"]);
}

#[test]
fn t3_cnfstyle_gains_val_bitmask() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<w:tr xmlns:w=\"{w}\"><w:trPr>\
             <w:cnfStyle w:firstRow=\"1\" w:lastRow=\"0\" w:firstColumn=\"true\" w:oddHBand=\"1\"/>\
             </w:trPr></w:tr>",
            w = W::URI
        ),
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let cnf = dom
        .descendants(root, Some(&W::name("cnfStyle")))
        .first()
        .copied()
        .unwrap();
    // bit order: firstRow lastRow firstColumn lastColumn oddVBand evenVBand
    //            oddHBand evenHBand frFc frLc lrFc lrLc
    assert_eq!(
        dom.attribute(cnf, &W::val()),
        Some("101000100000"),
        "{}",
        dom.serialize_element(cnf)
    );
}

#[test]
fn t3b_cnfstyle_with_val_untouched() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<w:tr xmlns:w=\"{w}\"><w:trPr><w:cnfStyle w:val=\"100000000000\"/></w:trPr></w:tr>",
            w = W::URI
        ),
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let cnf = dom
        .descendants(root, Some(&W::name("cnfStyle")))
        .first()
        .copied()
        .unwrap();
    assert_eq!(dom.attribute(cnf, &W::val()), Some("100000000000"));
}

#[test]
fn t4_wp14_percentages_to_per_thousand() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<wp:anchor xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" xmlns:wp14=\"{wp14}\">\
             <wp14:sizeRelH relativeFrom=\"page\"><wp14:pctWidth>40%</wp14:pctWidth></wp14:sizeRelH>\
             <wp14:sizeRelV relativeFrom=\"page\"><wp14:pctHeight>20.5%</wp14:pctHeight></wp14:sizeRelV>\
             </wp:anchor>",
            wp14 = WP14_URI
        ),
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let x = dom.serialize_element(root);
    assert!(x.contains("<wp14:pctWidth>40000</wp14:pctWidth>"), "{x}");
    assert!(x.contains("<wp14:pctHeight>20500</wp14:pctHeight>"), "{x}");
}

#[test]
fn t5_out_of_range_paraid_stripped_in_range_kept() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<w:body xmlns:w=\"{w}\" xmlns:w14=\"{w14}\">\
             <w:p w14:paraId=\"FD679369\" w14:textId=\"FD679369\"><w:r><w:t>bad</w:t></w:r></w:p>\
             <w:p w14:paraId=\"1D679369\" w14:textId=\"77777777\"><w:r><w:t>good</w:t></w:r></w:p>\
             </w:body>",
            w = W::URI,
            w14 = W14_URI
        ),
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let ps = dom.descendants(root, Some(&W::p()));
    assert_eq!(dom.attribute(ps[0], &W14::name("paraId")), None);
    assert_eq!(dom.attribute(ps[0], &W14::name("textId")), None);
    assert_eq!(dom.attribute(ps[1], &W14::name("paraId")), Some("1D679369"));
    assert_eq!(dom.attribute(ps[1], &W14::name("textId")), Some("77777777"));
}

#[test]
fn t6_tab_pos_and_cell_margin_measures_normalized() {
    use jubarte::comparer::finalize::normalize_universal_measures;
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        &format!(
            "<w:body xmlns:w=\"{w}\">\
             <w:p><w:pPr><w:tabs><w:tab w:val=\"right\" w:pos=\"467.50pt\"/></w:tabs></w:pPr></w:p>\
             <w:tbl><w:tblPr><w:tblCellMar>\
             <w:top w:w=\"100.0\" w:type=\"dxa\"/><w:right w:w=\"7.09pt\" w:type=\"dxa\"/>\
             </w:tblCellMar></w:tblPr></w:tbl>\
             </w:body>",
            w = W::URI
        ),
    );
    normalize_universal_measures(&mut dom, root);
    let x = dom.serialize_element(root);
    assert!(x.contains("w:pos=\"9350\""), "467.50pt → 9350 twips: {x}");
    assert!(x.contains("w:w=\"100\""), "fractional dxa rounded: {x}");
    assert!(x.contains("w:w=\"142\""), "7.09pt → 142 twips: {x}");
}

#[test]
fn t7_drawingml_percent_attrs_to_per_thousand() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "<dsp:sp xmlns:dsp=\"http://schemas.microsoft.com/office/drawing/2008/diagram\" \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
         <dsp:style><a:fillRef idx=\"1\"><a:scrgbClr r=\"0%\" g=\"12.5%\" b=\"100%\"/></a:fillRef></dsp:style>\
         </dsp:sp>",
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let clr = dom
        .descendants(root, None)
        .into_iter()
        .find(|&e| dom.name(e).is_some_and(|n| n.local_name() == "scrgbClr"))
        .unwrap();
    let x = dom.serialize_element(clr);
    assert!(x.contains("r=\"0\""), "{x}");
    assert!(x.contains("g=\"12500\""), "{x}");
    assert!(x.contains("b=\"100000\""), "{x}");
}

/// The actual repair-dialog trigger (bisected in real Word): Strict inputs
/// bind wsp/spPr/txbx in the STRICT wordprocessingDrawing namespace inside
/// `a:graphicData uri=".../2010/wordprocessingShape"`; URI translation makes
/// them Transitional wp: — a namespace with no such elements. They must be
/// remapped into the uri's namespace (wps:), without touching the outer
/// anchor's legitimate wp: elements or nested w:drawing contexts.
#[test]
fn t8_wps_elements_remapped_from_wp() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "<w:drawing xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
         xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
         <wp:anchor simplePos=\"0\"><wp:extent cx=\"100\" cy=\"100\"/>\
         <a:graphic><a:graphicData uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
         <wp:wsp><wp:cNvSpPr txBox=\"1\"/><wp:spPr/><wp:txbx><w:txbxContent><w:p/></w:txbxContent></wp:txbx><wp:bodyPr/></wp:wsp>\
         </a:graphicData></a:graphic></wp:anchor></w:drawing>",
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let x = dom.serialize_element(root);
    assert!(!x.contains("<wp:wsp"), "wsp left in wp namespace: {x}");
    // the wsp subtree is in the wps namespace now
    let wsp = dom
        .descendants(root, None)
        .into_iter()
        .find(|&e| dom.name(e).is_some_and(|n| n.local_name() == "wsp"))
        .unwrap();
    assert_eq!(
        dom.name(wsp).unwrap().namespace_name(),
        "http://schemas.microsoft.com/office/word/2010/wordprocessingShape"
    );
    // outer anchor stays wp:
    let anchor = dom
        .descendants(root, None)
        .into_iter()
        .find(|&e| dom.name(e).is_some_and(|n| n.local_name() == "anchor"))
        .unwrap();
    assert_eq!(
        dom.name(anchor).unwrap().namespace_name(),
        "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
    );
    // w:txbxContent untouched
    assert!(x.contains("txbxContent"), "{x}");
}

/// Deleted field-instruction runs must carry `w:delInstrText` — Word rejects
/// `w:instrText` inside `w:del` (6x in the strict01 TOC; Word's own redline
/// of the pair emits 12x delInstrText and zero raw instrText).
#[test]
fn t9_deleted_instr_text_becomes_del_instr_text() {
    use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
    let mut dom = Dom::new();
    let parse_doc = |dom: &mut Dom, inner: &str| {
        let xml = format!(
            "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
            w = W::URI
        );
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        (root, body)
    };
    let (r1, b1) = parse_doc(
        &mut dom,
        "<w:p><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
         <w:r><w:instrText xml:space=\"preserve\"> PAGE </w:instrText></w:r>\
         <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>",
    );
    let (r2, b2) = parse_doc(
        &mut dom,
        "<w:p><w:r><w:t>replaced entirely</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        !x.contains("<w:instrText"),
        "raw instrText inside deletion: {x}"
    );
    assert!(x.contains("delInstrText"), "{x}");
}

/// Word-written Strict packages use wne:txbxContent; in Transitional, wne is
/// mc:Ignorable so the required wps:txbx child vanishes — repair dialog
/// (strict01 cover page; Word's own redline emits w:txbxContent).
#[test]
fn t10_wne_txbxcontent_renamed_to_w() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "<wps:txbx xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\" \
         xmlns:wne=\"http://schemas.microsoft.com/office/word/2006/wordml\" \
         xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <wne:txbxContent><w:p><w:r><w:t>boxed</w:t></w:r></w:p></wne:txbxContent></wps:txbx>",
    );
    fix_strict_validity_artifacts(&mut dom, root);
    let x = dom.serialize_element(root);
    assert!(!x.contains("wne:txbxContent"), "{x}");
    assert!(x.contains("w:txbxContent"), "{x}");
}

/// w:delText in a run NOT directly inside w:del trips Word's repair dialog —
/// THE residual strict01 trigger (bisected: wrapping the 5 bare-delText runs
/// inside the deleted cover-page text boxes made the file open). Word wraps
/// nested text-box runs in explicit w:del; schema-legal either way, so no
/// validator catches it.
#[test]
fn t11_deleted_textbox_runs_wrapped_in_del() {
    use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
    let mut dom = Dom::new();
    let parse_doc = |dom: &mut Dom, inner: &str| {
        let xml = format!(
            "<w:document xmlns:w=\"{w}\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
             xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
             xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\"><w:body>{inner}</w:body></w:document>",
            w = W::URI
        );
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        (root, body)
    };
    let (r1, b1) = parse_doc(
        &mut dom,
        "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\"><wp:extent cx=\"100\" cy=\"100\"/>\
         <a:graphic><a:graphicData uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
         <wps:wsp><wps:txbx><w:txbxContent><w:p><w:r><w:t>boxed text</w:t></w:r></w:p></w:txbxContent></wps:txbx>\
         </wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>\
         <w:p><w:r><w:t>shared tail</w:t></w:r></w:p>",
    );
    let (r2, b2) = parse_doc(&mut dom, "<w:p><w:r><w:t>shared tail</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let mut checked = 0;
    for dt in dom.descendants(out, Some(&W::name("delText"))) {
        // nearest ancestor run
        let run = dom
            .ancestors_and_self(dt, None)
            .into_iter()
            .find(|&a| dom.name(a) == Some(W::r()))
            .expect("delText inside a run");
        let parent = dom.parent(run).expect("run has parent");
        assert_eq!(
            dom.name(parent),
            Some(W::del()),
            "delText run must sit directly inside w:del: {}",
            dom.serialize_element(parent)
        );
        checked += 1;
    }
    assert!(checked >= 1, "deleted text-box content produced delText");
}
