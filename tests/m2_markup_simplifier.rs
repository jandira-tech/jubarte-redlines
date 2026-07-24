// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::markup_simplifier::{
    remove_rsid_transform, transform_element_to_single_character_runs,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, XName, XNamespace};

fn w(local: &str) -> XName {
    XName::get(local, W::URI)
}

/// `<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>abc</w:t></w:r></w:p>` →
/// three `<w:r>`, each with the shared rPr and a one-char `<w:t>`.
#[test]
fn single_char_runs_split_text_and_keep_rpr() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let rpr = d.new_element(w("rPr"));
    let b = d.new_element(w("b"));
    d.add(rpr, b);
    let t = d.new_element(w("t"));
    d.add_text(t, "abc");
    d.add(r, rpr);
    d.add(r, t);
    d.add(p, r);

    let out = transform_element_to_single_character_runs(&mut d, p);

    let runs = d.elements(out, Some(&w("r")));
    assert_eq!(runs.len(), 3, "one run per character");
    for (i, &run) in runs.iter().enumerate() {
        // each run has an rPr containing <w:b/>
        let rprs = d.elements(run, Some(&w("rPr")));
        assert_eq!(rprs.len(), 1, "run {i} keeps shared rPr");
        assert_eq!(d.elements(rprs[0], Some(&w("b"))).len(), 1);
        // each run has a single-char <w:t>
        let ts = d.elements(run, Some(&w("t")));
        assert_eq!(ts.len(), 1);
        let expected = ["a", "b", "c"][i];
        assert_eq!(d.value(ts[0]), expected);
    }
}

/// A space character gets `xml:space="preserve"`.
#[test]
fn single_char_runs_preserve_space() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "a b");
    d.add(r, t);
    d.add(p, r);

    let out = transform_element_to_single_character_runs(&mut d, p);
    let runs = d.elements(out, Some(&w("r")));
    assert_eq!(runs.len(), 3);
    // middle run is the space → has xml:space="preserve"
    let space_run_t = d.elements(runs[1], Some(&w("t")))[0];
    let xml_space = XNamespace::xml().name("space");
    assert_eq!(d.value(space_run_t), " ");
    assert_eq!(d.attribute(space_run_t, &xml_space), Some("preserve"));
}

/// Non-text run children (e.g. `<w:tab/>`, `<w:br/>`) become their own runs.
#[test]
fn single_char_runs_split_non_text_children() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t1 = d.new_element(w("t"));
    d.add_text(t1, "x");
    let tab = d.new_element(w("tab"));
    let t2 = d.new_element(w("t"));
    d.add_text(t2, "y");
    d.add(r, t1);
    d.add(r, tab);
    d.add(r, t2);
    d.add(p, r);

    let out = transform_element_to_single_character_runs(&mut d, p);
    let runs = d.elements(out, Some(&w("r")));
    // x | tab | y → three runs
    assert_eq!(runs.len(), 3);
    assert_eq!(d.elements(runs[1], Some(&w("tab"))).len(), 1);
}

/// Idempotence: applying the transform twice yields the same run count.
#[test]
fn single_char_runs_idempotent() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "hello");
    d.add(r, t);
    d.add(p, r);

    let once = transform_element_to_single_character_runs(&mut d, p);
    let n1 = d.elements(once, Some(&w("r"))).len();
    let twice = transform_element_to_single_character_runs(&mut d, once);
    let n2 = d.elements(twice, Some(&w("r"))).len();
    assert_eq!(n1, 5);
    assert_eq!(n1, n2);
}

/// RemoveRsid drops `<w:rsid>` elements and `w:rsid*` attributes everywhere.
#[test]
fn remove_rsid_strips_elements_and_attributes() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    d.set_attribute_value(p, &w("rsidR"), Some("00AA11"));
    d.set_attribute_value(p, &w("rsidRDefault"), Some("00BB22"));
    let ppr = d.new_element(w("pPr"));
    let rsid = d.new_element(w("rsid")); // a <w:rsid> element to be dropped
    d.add(ppr, rsid);
    let r = d.new_element(w("r"));
    d.set_attribute_value(r, &w("rsidRPr"), Some("00CC33"));
    let t = d.new_element(w("t"));
    d.add_text(t, "keep");
    d.add(r, t);
    d.add(p, ppr);
    d.add(p, r);

    let out = remove_rsid_transform(&mut d, p).unwrap();

    // no rsid attributes survive anywhere
    for el in d.descendants_and_self(out, None) {
        for (name, _) in d.attributes(el) {
            assert!(
                !(name.namespace_name() == W::URI && name.local_name().starts_with("rsid")),
                "rsid attr leaked: {}",
                name.clark()
            );
        }
    }
    // the <w:rsid> element is gone
    assert_eq!(d.descendants(out, Some(&w("rsid"))).len(), 0);
    // content preserved
    assert_eq!(d.value(out), "keep");
}
