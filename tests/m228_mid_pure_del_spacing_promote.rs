//! M228 — mid pure-deleted paragraphs must promote spacing-only `pPrChange`
//! onto live `w:spacing` (and drop the change mark).
//!
//! Hidden gem (CR #5): `cleanup_spacing_and_default_jc` ran M226's
//! `if !para_is_mixed_revision { continue }` before M228, so pure-deleted
//! paragraphs never reached the promote path. This fixture is pure-del mid
//! body with spacing parked only in pPrChange.

use jubarte::comparer::finalize::cleanup_spacing_and_default_jc;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const WNS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn mid_pure_del_with_pprchange_spacing() -> String {
    format!(
        r#"<w:document xmlns:w="{WNS}"><w:body>
  <w:p>
    <w:pPr>
      <w:pPrChange w:id="1" w:author="A" w:date="2020-01-01T00:00:00Z">
        <w:pPr>
          <w:spacing w:before="200" w:after="100" w:line="240"/>
        </w:pPr>
      </w:pPrChange>
    </w:pPr>
    <w:del w:id="2" w:author="A" w:date="2020-01-01T00:00:00Z">
      <w:r><w:delText>deleted middle</w:delText></w:r>
    </w:del>
  </w:p>
  <w:p><w:r><w:t>trailing kept</w:t></w:r></w:p>
</w:body></w:document>"#
    )
}

#[test]
fn m228_promotes_mid_pure_del_spacing_from_pprchange_to_live() {
    let xml = mid_pure_del_with_pprchange_spacing();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    cleanup_spacing_and_default_jc(&mut dom, root);

    let body = dom.element(root, &W::body()).expect("body");
    let kids = dom.elements(body, None);
    let mid = kids[0];
    let ppr = dom.element(mid, &W::p_pr()).expect("pPr");
    let live_sp = dom
        .element(ppr, &W::name("spacing"))
        .expect("M228 must promote spacing onto live pPr");
    assert_eq!(
        dom.attribute(live_sp, &W::name("before")),
        Some("200"),
        "live before=200 after promote"
    );
    assert_eq!(
        dom.attribute(live_sp, &W::name("after")),
        Some("100"),
        "live after=100 after promote"
    );
    assert_eq!(
        dom.attribute(live_sp, &W::name("line")),
        Some("240"),
        "live line=240 after promote"
    );
    assert!(
        dom.element(ppr, &W::name("pPrChange")).is_none(),
        "pPrChange must be removed after promote; ser={}",
        dom.serialize_element(mid)
    );
}

#[test]
fn m228_line276_noise_on_pure_del_strips_pprchange() {
    let xml = format!(
        r#"<w:document xmlns:w="{WNS}"><w:body>
  <w:p>
    <w:pPr>
      <w:pPrChange w:id="1" w:author="A" w:date="2020-01-01T00:00:00Z">
        <w:pPr><w:spacing w:line="276"/></w:pPr>
      </w:pPrChange>
    </w:pPr>
    <w:del w:id="2" w:author="A" w:date="2020-01-01T00:00:00Z">
      <w:r><w:delText>noise</w:delText></w:r>
    </w:del>
  </w:p>
  <w:p><w:r><w:t>tail</w:t></w:r></w:p>
</w:body></w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    cleanup_spacing_and_default_jc(&mut dom, root);
    let body = dom.element(root, &W::body()).expect("body");
    let mid = dom.elements(body, None)[0];
    let ppr = dom.element(mid, &W::p_pr()).expect("pPr");
    assert!(
        dom.element(ppr, &W::name("pPrChange")).is_none(),
        "line=276-only noise must drop pPrChange"
    );
    assert!(
        dom.element(ppr, &W::name("spacing")).is_none(),
        "line=276 noise must not promote to live"
    );
}
