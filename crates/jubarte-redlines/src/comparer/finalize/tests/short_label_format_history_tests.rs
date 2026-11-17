// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

const DATE: &str = "2001-02-03T04:05:06Z";
const LAYOUT: &str = "<w:pPr><w:spacing w:before=\"120\" w:after=\"80\"/><w:ind w:left=\"720\" w:hanging=\"360\"/><w:jc w:val=\"right\"/></w:pPr>";
const CALIBRI: &str = "<w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\" w:eastAsia=\"Calibri\" w:cs=\"Calibri\"/><w:sz w:val=\"20\"/><w:szCs w:val=\"20\"/></w:rPr>";

fn document(deleted: &str, inserted: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\"><w:body><w:p>{LAYOUT}<w:ins w:id=\"23\" w:author=\"Revised editor\" w:date=\"{DATE}\">{inserted}</w:ins><w:del w:id=\"43\" w:author=\"Original editor\" w:date=\"{DATE}\">{deleted}</w:del></w:p></w:body></w:document>", W::URI
        ));
    let root = dom.root(doc).unwrap();
    let p = dom.descendants(root, Some(&W::p()))[0];
    (dom, root, p)
}

fn properties(dom: &Dom, run: NodeId) -> String {
    dom.element(run, &W::r_pr())
        .map_or_else(String::new, |rpr| {
            dom.elements(rpr, None)
                .into_iter()
                .map(|c| dom.serialize_element(c))
                .collect()
        })
}

fn resolved(dom: &mut Dom, root: NodeId, accept: bool) -> (String, Vec<(char, String)>, String) {
    let copy = dom.clone_subtree(root);
    let copy = if accept {
        crate::revision_processor::accept_revisions_document(dom, copy)
    } else {
        crate::revision_processor::reject_revisions_document(dom, copy)
    };
    let p = dom.descendants(copy, Some(&W::p()))[0];
    let chars: Vec<_> = dom
        .descendants(p, Some(&W::t()))
        .into_iter()
        .flat_map(|t| {
            let rpr = properties(dom, dom.parent(t).unwrap());
            dom.value(t)
                .chars()
                .map(|ch| (ch, rpr.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let text = chars.iter().map(|&(ch, _)| ch).collect();
    let layout = dom.serialize_element(dom.element(p, &W::p_pr()).unwrap());
    (text, chars, layout)
}

#[test]
fn shared_label_tracks_complete_old_and_revised_run_formats_and_layout() {
    // The bold/italic case was formerly a no-op test. Tracking the change
    // now preserves that case's exact two source formats while matching
    // the actual M375 Word EQ shape (Calibri live, empty old properties).
    for (old, new, label) in [
        ("", CALIBRI, "a"),
        ("<w:rPr><w:b/></w:rPr>", "<w:rPr><w:i/></w:rPr>", "AB"),
        (CALIBRI, "", "b"),
        (CALIBRI, CALIBRI, "12"),
        ("", "", "a"),
    ] {
        let (mut dom, root, p) = document(
            &format!(
                "<w:r>{old}<w:delText xml:space=\"preserve\">Lvl 1 – {label}</w:delText></w:r>"
            ),
            &format!("<w:r>{new}<w:t xml:space=\"preserve\">{label} </w:t></w:r>"),
        );
        let accepted = resolved(&mut dom, root, true);
        let rejected = resolved(&mut dom, root, false);
        assert_eq!(accepted.0, format!("{label} "));
        assert_eq!(rejected.0, format!("Lvl 1 – {label}"));
        assert!(
            accepted
                .1
                .iter()
                .all(|(_, props)| props == &accepted.1[0].1)
        );
        assert!(
            rejected
                .1
                .iter()
                .all(|(_, props)| props == &rejected.1[0].1)
        );
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(resolved(&mut dom, root, true), accepted);
        assert_eq!(resolved(&mut dom, root, false), rejected);
        let body = dom.elements(p, None);
        assert_eq!(
            body.iter()
                .filter_map(|&n| dom.name(n))
                .map(|n| n.local_name().to_string())
                .collect::<Vec<_>>(),
            ["pPr", "del", "r", "ins"]
        );
        let eq = dom.elements(p, Some(&W::r()))[0];
        assert_eq!(dom.value(dom.element(eq, &W::t()).unwrap()), label);
        assert_eq!(
            markup_text(&dom, dom.elements(p, Some(&W::del()))[0]),
            "Lvl 1 – "
        );
        assert_eq!(markup_text(&dom, dom.elements(p, Some(&W::ins()))[0]), " ");
        let changes = dom.descendants(eq, Some(&W::r_pr_change()));
        if old == new {
            assert!(changes.is_empty());
        } else {
            assert_eq!(changes.len(), 1);
            let change = changes[0];
            assert_eq!(dom.attribute(change, &W::id()), Some("44"));
            assert_eq!(dom.attribute(change, &W::author()), Some("Revised editor"));
            assert_eq!(dom.attribute(change, &W::date()), Some(DATE));
            assert_eq!(dom.elements(change, Some(&W::r_pr())).len(), 1);
            // Empty A properties remain an explicit old snapshot, as in Word.
            if old.is_empty() {
                assert!(
                    dom.elements(dom.element(change, &W::r_pr()).unwrap(), None)
                        .is_empty()
                );
            }
        }
    }
}

#[test]
fn formerly_unsupported_bold_to_italic_label_preserves_the_exact_fixture() {
    let (mut dom, root, p) = document(
        "<w:r><w:rPr><w:b/></w:rPr><w:delText>Lvl 1 AB</w:delText></w:r>",
        "<w:r><w:rPr><w:i/></w:rPr><w:t>AB</w:t></w:r>",
    );
    let accepted = resolved(&mut dom, root, true);
    let rejected = resolved(&mut dom, root, false);
    assert_eq!(accepted.0, "AB");
    assert_eq!(rejected.0, "Lvl 1 AB");
    mesh_short_label_shared_eq(&mut dom, p);
    assert_eq!(resolved(&mut dom, root, true), accepted);
    assert_eq!(resolved(&mut dom, root, false), rejected);
    assert!(dom.elements(p, Some(&W::ins())).is_empty());
    let eq = dom.elements(p, Some(&W::r()))[0];
    assert_eq!(markup_text(&dom, eq), "AB");
    let live = dom.element(eq, &W::r_pr()).unwrap();
    assert!(dom.element(live, &W::name("i")).is_some());
    assert!(dom.element(live, &W::name("b")).is_none());
    let change = dom.element(live, &W::r_pr_change()).unwrap();
    let old = dom.element(change, &W::r_pr()).unwrap();
    assert!(dom.element(old, &W::name("b")).is_some());
    assert!(dom.element(old, &W::name("i")).is_none());
}

#[test]
fn fragmented_label_may_share_only_with_consistent_properties_on_each_side() {
    let deleted = format!(
        "<w:r>{CALIBRI}<w:delText>Lvl 1 A</w:delText></w:r><w:r>{CALIBRI}<w:delText>B</w:delText></w:r>"
    );
    let inserted =
        "<w:r><w:rPr><w:b/></w:rPr><w:t>A</w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>B</w:t></w:r>";
    let (mut dom, root, p) = document(&deleted, inserted);
    let accepted = resolved(&mut dom, root, true);
    let rejected = resolved(&mut dom, root, false);
    assert_eq!(accepted.0, "AB");
    assert_eq!(rejected.0, "Lvl 1 AB");
    mesh_short_label_shared_eq(&mut dom, p);
    assert_eq!(resolved(&mut dom, root, true), accepted);
    assert_eq!(resolved(&mut dom, root, false), rejected);
    assert!(dom.elements(p, Some(&W::ins())).is_empty());
    assert_eq!(dom.elements(p, Some(&W::r())).len(), 1);
    assert_eq!(dom.descendants(p, Some(&W::r_pr_change())).len(), 1);
}

#[test]
fn label_property_boundaries_history_and_opaque_properties_stay_unchanged() {
    for (deleted, inserted) in [
        (
            "<w:r><w:rPr><w:b/></w:rPr><w:delText>Lvl 1 A</w:delText></w:r><w:r><w:rPr><w:i/></w:rPr><w:delText>B</w:delText></w:r>",
            "<w:r><w:t>AB</w:t></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:rPr><w:b/></w:rPr><w:t>A</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>B</w:t></w:r>",
        ),
        (
            "<w:r><w:rPr><w:b/><w:rPrChange w:id=\"7\"><w:rPr/></w:rPrChange></w:rPr><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:t>AB</w:t></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:rPr><w:ins w:id=\"7\"/></w:rPr><w:t>AB</w:t></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:rPr><x:payload xmlns:x=\"urn:opaque\" x:owner=\"retained\"/></w:rPr><w:t>AB</w:t></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:rPr><w:rFonts><w:opaque/></w:rFonts></w:rPr><w:t>AB</w:t></w:r>",
        ),
    ] {
        let (mut dom, root, p) = document(deleted, inserted);
        let before = dom.serialize_element(p);
        let accepted = resolved(&mut dom, root, true);
        let rejected = resolved(&mut dom, root, false);
        assert_eq!(accepted.0, "AB");
        assert_eq!(rejected.0, "Lvl 1 AB");
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(dom.serialize_element(p), before);
        assert_eq!(resolved(&mut dom, root, true), accepted);
        assert_eq!(resolved(&mut dom, root, false), rejected);
    }
}
#[test]
fn configured_label_tracking_preserves_sources_and_respects_disabled_changes() {
    for tracking in [false, true] {
        for separate in [false, true] {
            let (mut dom, root, p) = document(
                "<w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:delText>Lvl 1 AB</w:delText></w:r>",
                "<w:r><w:rPr><w:i/><w:sz w:val='28'/></w:rPr><w:t>AB</w:t></w:r>",
            );
            if separate {
                let inserted = dom.new_element(W::p());
                let props = dom.clone_subtree(dom.element(p, &W::p_pr()).unwrap());
                let mark = dom.new_element(W::r_pr());
                let ins = dom.new_element(W::ins());
                dom.set_attribute_value(ins, &W::id(), Some("47"));
                dom.set_attribute_value(ins, &W::author(), Some("Prior editor"));
                dom.add(mark, ins);
                dom.add(props, mark);
                dom.add(inserted, props);
                let content = dom.element(p, &W::ins()).unwrap();
                dom.remove(content);
                dom.add(inserted, content);
                dom.add_before_self(p, inserted);
                let mark = dom.new_element(W::r_pr());
                let del = dom.new_element(W::del());
                dom.set_attribute_value(del, &W::id(), Some("48"));
                dom.set_attribute_value(del, &W::author(), Some("Prior editor"));
                dom.add(mark, del);
                dom.add(dom.element(p, &W::p_pr()).unwrap(), mark);
            }
            let accepted = resolved(&mut dom, root, true);
            let rejected = resolved(&mut dom, root, false);
            let settings = WmlComparerSettings {
                detect_format_changes: tracking,
                author_for_revisions: "Configured comparison editor".into(),
                date_time_for_revisions: "2002-03-04T05:06:07Z".into(),
                ..WmlComparerSettings::default()
            };
            let old_max = max_numeric_id(&dom, root);
            if separate {
                residual_short_label_zip_with_settings(&mut dom, root, &settings);
            } else {
                mesh_short_label_shared_eq_with_settings(&mut dom, p, &settings);
            }
            assert_eq!(resolved(&mut dom, root, true), accepted);
            assert_eq!(resolved(&mut dom, root, false), rejected);
            let changes = dom.descendants(root, Some(&W::r_pr_change()));
            assert_eq!(changes.len(), usize::from(tracking));
            if tracking {
                let change = changes[0];
                assert_eq!(
                    dom.attribute(change, &W::id()),
                    Some((old_max + 1).to_string().as_str())
                );
                assert_eq!(
                    dom.attribute(change, &W::author()),
                    Some("Configured comparison editor")
                );
                assert_eq!(
                    dom.attribute(change, &W::date()),
                    Some("2002-03-04T05:06:07Z")
                );
            } else {
                assert!(
                    dom.descendants(root, Some(&W::ins()))
                        .iter()
                        .any(|&n| !dom.descendants(n, Some(&W::t())).is_empty())
                );
                assert!(
                    dom.descendants(root, Some(&W::del()))
                        .iter()
                        .any(|&n| !dom.descendants(n, Some(&W::del_text())).is_empty())
                );
                assert!(
                    dom.elements(p, Some(&W::r())).is_empty(),
                    "no live label may choose only one source format"
                );
            }
        }
    }
}
