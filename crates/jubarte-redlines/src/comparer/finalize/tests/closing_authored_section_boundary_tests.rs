// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn parse(dom: &mut Dom, xml: &str) -> NodeId {
    let document = dom.parse_xdocument(xml);
    dom.root(document).unwrap()
}

fn section(width: u32) -> String {
    format!("<w:sectPr><w:pgSz w:w='{width}' w:h='15840'/></w:sectPr>")
}

fn props(section: Option<u32>) -> String {
    format!(
        "<w:pPr><w:spacing w:before='120' w:after='80'/>{}</w:pPr>",
        section.map(self::section).unwrap_or_default()
    )
}

fn canonical(dom: &Dom, node: NodeId) -> String {
    let mut attributes = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .collect::<Vec<_>>();
    attributes.sort_by(|a, b| {
        (a.0.namespace_name(), a.0.local_name()).cmp(&(b.0.namespace_name(), b.0.local_name()))
    });
    let header = format!(
        "{:?}:{attributes:?}:{:?}",
        dom.name(node),
        dom.text_value(node)
    );
    let children = dom
        .nodes(node)
        .into_iter()
        .map(|child| {
            let child = canonical(dom, child);
            format!("{}:{child}", child.len())
        })
        .collect::<String>();
    format!("{}:{header}{}:{children}", header.len(), children.len())
}

fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
    let copy = dom.clone_subtree(root);
    let root = if accept {
        crate::revision_processor::accept_revisions_document(dom, copy)
    } else {
        crate::revision_processor::reject_revisions_document(dom, copy)
    };
    remove_powertools_scratch_markup(dom, root);
    for property in dom.descendants(root, None).into_iter().rev() {
        if [W::p_pr(), W::r_pr()]
            .iter()
            .any(|name| dom.name_is(property, name))
            && dom.nodes(property).is_empty()
            && dom
                .attributes(property)
                .iter()
                .all(|(name, _)| dom.is_namespace_declaration(name))
        {
            dom.remove(property);
        }
    }
    canonical(dom, dom.element(root, &W::body()).unwrap())
}

#[test]
fn section_addition_guard_respects_tracking_source_presence_and_exact_live_identity() {
    for (old_section, new_section, live_section, tracking, needs_history) in [
        (None, Some(12240), Some(12240), true, true),
        (None, Some(12240), Some(12240), false, false),
        (None, Some(12240), Some(10000), true, false),
        (None, Some(12240), None, true, false),
        (None, None, Some(12240), true, false),
        (Some(12240), Some(12240), Some(12240), true, false),
        (Some(12240), None, None, true, false),
    ] {
        let mut dom = Dom::new();
        let old = parse(
            &mut dom,
            &format!(
                "<w:root xmlns:w='{}'>{}</w:root>",
                W::URI,
                props(old_section)
            ),
        );
        let original = dom.element(old, &W::p_pr()).unwrap();
        let new = parse(
            &mut dom,
            &format!(
                "<w:root xmlns:w='{}'>{}</w:root>",
                W::URI,
                props(new_section)
            ),
        );
        let revised = dom.element(new, &W::p_pr()).unwrap();
        let root = parse(
            &mut dom,
            &format!(
                "<w:document xmlns:w='{}'><w:body><w:p>{}<w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>ONE</w:t></w:r></w:p>{}</w:body></w:document>",
                W::URI,
                props(live_section),
                section(12240)
            ),
        );
        let before = canonical(&dom, root);
        let original_snapshot = canonical(&dom, original);
        let revised_snapshot = canonical(&dom, revised);
        let settings = WmlComparerSettings {
            detect_format_changes: tracking,
            author_for_revisions: "Closing editor".into(),
            date_time_for_revisions: "2026-01-01T00:00:00Z".into(),
            ..WmlComparerSettings::default()
        };
        let mut next_id = 70;
        closing_mark_takes_revised_properties(
            &mut dom,
            root,
            Some(original),
            Some(revised),
            (None, None),
            &settings,
            &mut next_id,
        );
        let histories = dom.descendants(root, Some(&W::p_pr_change()));
        assert_eq!(
            histories.len(),
            usize::from(needs_history),
            "old={old_section:?},new={new_section:?},live={live_section:?},tracking={tracking}"
        );
        assert_eq!(next_id, 70 + u32::from(needs_history));
        assert_eq!(canonical(&dom, original), original_snapshot);
        assert_eq!(canonical(&dom, revised), revised_snapshot);
        if needs_history {
            let history = histories[0];
            assert_eq!(dom.attribute(history, &W::id()), Some("70"));
            assert_eq!(dom.attribute(history, &W::author()), Some("Closing editor"));
            assert_eq!(
                dom.attribute(history, &W::date()),
                Some("2026-01-01T00:00:00Z")
            );
            let saved = dom.element(history, &W::p_pr()).unwrap();
            assert_eq!(canonical(&dom, saved), original_snapshot);
            assert!(dom.descendants(saved, Some(&W::sect_pr())).is_empty());
            let accepted = dom.clone_subtree(root);
            let accepted = crate::revision_processor::accept_revisions_document(&mut dom, accepted);
            let rejected = dom.clone_subtree(root);
            let rejected = crate::revision_processor::reject_revisions_document(&mut dom, rejected);
            let accepted_paragraph = dom.descendants(accepted, Some(&W::p()))[0];
            let rejected_paragraph = dom.descendants(rejected, Some(&W::p()))[0];
            assert_eq!(
                dom.descendants(accepted_paragraph, Some(&W::sect_pr()))
                    .len(),
                1
            );
            // The saved pPr cannot hold a section break, so Reject keeps
            // the live one, as Word's Reject All does (#1708). An added
            // break should be an inserted paragraph mark instead (#1718).
            assert_eq!(
                dom.descendants(rejected_paragraph, Some(&W::sect_pr()))
                    .len(),
                1
            );
            assert_eq!(dom.value(accepted_paragraph), "ONE");
            assert_eq!(dom.value(rejected_paragraph), "ONE");
        } else {
            assert_eq!(canonical(&dom, root), before);
        }
        let once = canonical(&dom, root);
        closing_mark_takes_revised_properties(
            &mut dom,
            root,
            Some(original),
            Some(revised),
            (None, None),
            &settings,
            &mut next_id,
        );
        assert_eq!(canonical(&dom, root), once);
        assert_eq!(next_id, 70 + u32::from(needs_history));
    }
}

#[test]
fn matching_closing_history_keeps_its_existing_revision_owner_and_id() {
    let mut dom = Dom::new();
    let old = parse(
        &mut dom,
        &format!("<w:root xmlns:w='{}'>{}</w:root>", W::URI, props(None)),
    );
    let original = dom.element(old, &W::p_pr()).unwrap();
    let new = parse(
        &mut dom,
        &format!(
            "<w:root xmlns:w='{}'>{}</w:root>",
            W::URI,
            props(Some(12240))
        ),
    );
    let revised = dom.element(new, &W::p_pr()).unwrap();
    let live=props(Some(12240)).replace("</w:pPr>",&format!("<w:pPrChange w:id='55' w:author='Prior property editor' w:date='2025-11-12T00:00:00Z'>{}</w:pPrChange></w:pPr>",props(None)));
    let root = parse(
        &mut dom,
        &format!(
            "<w:document xmlns:w='{}'><w:body><w:p>{live}<w:r><w:t>ONE</w:t></w:r></w:p>{}</w:body></w:document>",
            W::URI,
            section(12240)
        ),
    );
    let before = canonical(&dom, root);
    let mut next_id = 70;
    for _ in 0..2 {
        closing_mark_takes_revised_properties(
            &mut dom,
            root,
            Some(original),
            Some(revised),
            (None, None),
            &WmlComparerSettings::default(),
            &mut next_id,
        );
        assert_eq!(canonical(&dom, root), before);
        assert_eq!(next_id, 70);
    }
    let history = dom.descendants(root, Some(&W::p_pr_change()));
    assert_eq!(history.len(), 1);
    assert_eq!(dom.attribute(history[0], &W::id()), Some("55"));
    assert_eq!(
        dom.attribute(history[0], &W::author()),
        Some("Prior property editor")
    );
    assert_eq!(
        dom.attribute(history[0], &W::date()),
        Some("2025-11-12T00:00:00Z")
    );
    let accepted = dom.clone_subtree(root);
    let accepted = crate::revision_processor::accept_revisions_document(&mut dom, accepted);
    let rejected = dom.clone_subtree(root);
    let rejected = crate::revision_processor::reject_revisions_document(&mut dom, rejected);
    let ap = dom.descendants(accepted, Some(&W::p()))[0];
    let rp = dom.descendants(rejected, Some(&W::p()))[0];
    assert_eq!(
        canonical(&dom, dom.element(ap, &W::p_pr()).unwrap()),
        canonical(&dom, revised)
    );
    // Reject keeps the live section break, as Word's Reject All does
    // (#1708, #1718). The saved and revised spacing are the same here,
    // so the reverted pPr is the revised one.
    assert_eq!(
        canonical(&dom, dom.element(rp, &W::p_pr()).unwrap()),
        canonical(&dom, revised)
    );
}

#[test]
fn public_added_and_unchanged_authored_sections_keep_complete_source_bodies() {
    for (original_section, revised_section, word) in [
        (false, true, true),
        (true, true, true),
        (true, true, false),
        (false, false, true),
        (false, false, false),
    ] {
        let mut dom = Dom::new();
        let roots=[original_section,revised_section].map(|has_section|parse(&mut dom,&format!("<w:document xmlns:w='{}'><w:body><w:p>{}<w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>ONE</w:t></w:r></w:p>{}</w:body></w:document>",W::URI,props(has_section.then_some(12240)),section(12240))));
        // An added break is tracked as a pPr change, whose saved pPr
        // cannot hold it, so Reject keeps it, as Word's Reject All does:
        // the revised body with the original spacing, which here is the
        // same (#1708). Word's Compare inserts the paragraph mark (#1718).
        let added = !original_section && revised_section;
        let expected_original = projection(&mut dom, roots[usize::from(added)], false);
        let expected_revised = projection(&mut dom, roots[1], true);
        let bodies = roots.map(|root| dom.element(root, &W::body()).unwrap());
        let settings = if word {
            WmlComparerSettings::default()
        } else {
            WmlComparerSettings::powertools_faithful()
        };
        let output = crate::comparer::compare_bodies_faithful(
            &mut dom, roots[0], roots[1], bodies[0], bodies[1], &settings,
        );
        assert_eq!(
            projection(&mut dom, output, false),
            expected_original,
            "reject originalsection={original_section},revisedsection={revised_section},word={word}"
        );
        assert_eq!(
            projection(&mut dom, output, true),
            expected_revised,
            "accept originalsection={original_section},revisedsection={revised_section},word={word}"
        );
        assert_eq!(
            dom.descendants(output, Some(&W::p_pr_change())).len(),
            usize::from(!original_section && revised_section)
        );
        assert!(
            dom.descendants(output, Some(&W::name("sectPrChange")))
                .is_empty(),
            "unchanged final body geometry owns no new section history"
        );
    }
}
