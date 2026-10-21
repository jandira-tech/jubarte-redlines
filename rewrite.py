import re

with open("src/revision_processor.rs", "r") as f:
    content = f.read()

helpers = """
fn try_accept_wholly_revised_equation(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &M::name("oMath") || name == &M::name("oMathPara") {
        let owners: Vec<_> = dom
            .descendants(node, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &M::name("r")) || dom.name_is(n, &M::name("ctrlPr")))
            .collect();
        let has_authored_empty_equation = dom
            .descendants(node, Some(&M::name("oMath")))
            .into_iter()
            .any(|equation| {
                !dom.descendants(equation, None)
                    .into_iter()
                    .any(|n| dom.name_is(n, &M::name("r")) || dom.name_is(n, &M::name("ctrlPr")))
            });
        let has_unowned_content = dom.descendants(node, None).into_iter().any(|child| {
            let ancestors = dom.ancestors(child, None);
            if dom.name_is(child, &W::del())
                || ancestors
                    .iter()
                    .any(|&ancestor| dom.name_is(ancestor, &W::del()))
            {
                return false;
            }
            let Some(child_name) = dom.name(child) else {
                return !dom
                    .text_value(child)
                    .is_some_and(|text| text.trim().is_empty());
            };
            // oMath admits ordinary WML runs, anchors and transparent controls.
            // They are independent owners even when all m:r owners are deleted.
            if child_name.namespace_name() != M::URI {
                return true;
            }
            if child_name.local_name() == "r" || child_name.local_name() == "ctrlPr" {
                return false;
            }
            // Math property leaves describe the removed equation; an empty
            // expression/container outside properties is an authored sibling.
            let in_math_properties = child_name.local_name().ends_with("Pr")
                || ancestors.iter().any(|&ancestor| {
                    dom.name(ancestor).is_some_and(|name| {
                        name.namespace_name() == M::URI && name.local_name().ends_with("Pr")
                    })
                });
            !in_math_properties && dom.elements(child, None).is_empty()
        });
        if !has_authored_empty_equation
            && !has_unowned_content
            && !owners.is_empty()
            && owners.iter().all(|&owner| {
                let children = dom.elements(owner, None);
                !children.is_empty()
                    && children.iter().all(|&child| dom.name_is(child, &W::del()))
                    && dom.nodes(owner).into_iter().all(|child| {
                        dom.is_element(child)
                            || dom
                                .text_value(child)
                                .is_some_and(|text| text.trim().is_empty())
                    })
            })
        {
            return Some(hoist_range_markers_from(dom, node));
        }
    }
    None
}

fn try_accept_math_fraction(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &M::name("f") {
        let removed = dom
            .elements(node, Some(&M::name("fPr")))
            .into_iter()
            .any(|fpr| has_path(dom, fpr, &M::name("ctrlPr"), &W::del()));
        if removed {
            return Some(vec![]);
        }
    }
    None
}

fn try_accept_deleted_row(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::tr() && has_path(dom, node, &W::tr_pr(), &W::del()) {
        return Some(vec![]);
    }
    None
}

fn try_accept_deleted_table(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::tbl() && table_rows_all_deleted(dom, node) {
        return Some(vec![]);
    }
    None
}

fn try_accept_deleted_text(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::del() {
        return Some(hoist_range_markers_from(dom, node));
    }
    None
}

fn try_accept_cell_merge(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::cell_merge() {
        let parent_is_tcpr = dom
            .parent(node)
            .and_then(|p| dom.name(p))
            .is_some_and(|pn| pn == W::tc_pr());
        if parent_is_tcpr {
            let vmerge = dom.attribute(node, &W::v_merge()).map(|s| s.to_string());
            if vmerge.as_deref() == Some("rest") {
                let v = dom.new_element(W::v_merge());
                dom.set_attribute_value(v, &W::val(), Some("restart"));
                return Some(vec![v]);
            }
            if vmerge.as_deref() == Some("cont") {
                let v = dom.new_element(W::v_merge());
                dom.set_attribute_value(v, &W::val(), Some("continue"));
                return Some(vec![v]);
            }
        }
    }
    None
}

fn try_accept_hyperlink(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::hyperlink() {
        let ne = dom.new_element(name.clone());
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            for tn in accept_all_other_revisions_transform(dom, c) {
                dom.add(ne, tn);
            }
        }
        return Some(if has_run_content(dom, ne) {
            vec![ne]
        } else {
            hoist_range_markers_from(dom, ne)
        });
    }
    None
}

"""

start_idx = content.find("/// Port of `AcceptAllOtherRevisionsTransform`")
if start_idx == -1:
    print("Could not find start of accept_all_other_revisions_transform")
    exit(1)

func_decl_idx = content.find("pub fn accept_all_other_revisions_transform", start_idx)

end_idx = content.find("/// Port of `AcceptRevisionsForElement`", func_decl_idx)
if end_idx == -1:
    print("Could not find end of accept_all_other_revisions_transform")
    exit(1)

new_func = """pub fn accept_all_other_revisions_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    // Clean subtree (incl. plain text leaves): transfer, do not rebuild.
    if !element_has_tracked_revisions(dom, node) {
        return take_subtree(dom, node);
    }
    if !dom.is_element(node) {
        return take_subtree(dom, node);
    }
    let name = dom.name(node).unwrap();

    // Accept inserted content: collapse w:ins.
    // #1155: resolve even the final paragraph mark, unlike Word's Accept/Reject All.
    if name == W::ins() {
        let mut out = Vec::new();
        for c in dom.nodes(node) {
            out.extend(accept_all_other_revisions_transform(dom, c));
        }
        return out;
    }

    // Drop revision-range markers (handled by the range transforms).
    let drop_markers = [
        "customXmlDelRangeStart",
        "customXmlDelRangeEnd",
        "customXmlInsRangeStart",
        "customXmlInsRangeEnd",
        "customXmlMoveFromRangeStart",
        "customXmlMoveFromRangeEnd",
        "customXmlMoveToRangeStart",
        "customXmlMoveToRangeEnd",
        "moveFromRangeStart",
        "moveFromRangeEnd",
        "moveToRangeStart",
        "moveToRangeEnd",
    ];
    if name.namespace_name() == W::URI && drop_markers.contains(&name.local_name()) {
        return vec![];
    }

    // Accept formatting-change / deleted-content markers by removing them.
    let drop_format = [
        "pPrChange",
        "rPrChange",
        "tblPrChange",
        "tblGridChange",
        "tcPrChange",
        "trPrChange",
        "tblPrExChange",
        "sectPrChange",
        "numberingChange",
        "delInstrText",
        "delText",
        "cellIns",
    ];
    if name.namespace_name() == W::URI && drop_format.contains(&name.local_name()) {
        return vec![];
    }

    if let Some(res) = try_accept_wholly_revised_equation(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_math_fraction(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_deleted_row(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_deleted_table(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_deleted_text(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_cell_merge(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_hyperlink(dom, node, &name) {
        return res;
    }

    // Identity clone with transformed children.
    let ne = dom.new_element(name.clone());
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        for tn in accept_all_other_revisions_transform(dom, c) {
            dom.add(ne, tn);
        }
    }
    vec![ne]
}

"""

new_content = content[:start_idx] + helpers + content[start_idx:func_decl_idx] + new_func + content[end_idx:]

with open("src/revision_processor.rs", "w") as f:
    f.write(new_content)

print("Done")
