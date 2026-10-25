// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bookmarks through Accept All (and Reject All, which accepts the reversed
//! revisions), as Word keeps them.
//!
//! Word holds a bookmark as a range of characters, paragraph marks included.
//! Accepting a deletion removes characters, so a bookmark whose span held
//! characters that were all deleted is gone, both markers, while one that
//! keeps a character keeps both. An empty bookmark has no characters to lose:
//! it survives even inside deleted text, where the deletion was
//! (`_to_improve_accepted_changes`: ece42865e7, 221577c35b, 6fb9bbdb49).

use std::collections::{HashMap, HashSet};

use super::{TagType, descendant_and_self_tags, paragraph_mark_is_deleted_or_moved_from};
use crate::namespaces::{M, W};
use crate::xmllinq::{Dom, NodeId, XName};

/// Remove, before the accept transforms run, every bookmark whose span held
/// only deleted characters.
pub(super) fn drop_wholly_deleted_bookmarks(dom: &mut Dom, root: NodeId) {
    let (start, end) = (W::name("bookmarkStart"), W::name("bookmarkEnd"));
    if dom
        .find_descendant_element(root, Some(&start), |_| true)
        .is_none()
    {
        return;
    }
    let range_deleted = super::move_from_range_deleted_elements(dom, root);
    let removed_by_range = |node| {
        range_deleted.contains(&node)
            || dom
                .ancestors(node, None)
                .into_iter()
                .any(|a| range_deleted.contains(&a))
    };
    let id = W::id();
    // Open bookmark id → (characters, deleted characters).
    let mut open: HashMap<String, (usize, usize)> = HashMap::new();
    let mut dropped: HashSet<String> = HashSet::new();
    for tag in descendant_and_self_tags(dom, root) {
        let e = tag.element;
        let Some(name) = dom.name(e) else {
            continue;
        };
        let deleted = if tag.tag_type == TagType::EndElement {
            if name != W::p() {
                continue;
            }
            paragraph_mark_is_deleted_or_moved_from(dom, e)
                || in_deleted_row(dom, e)
                || removed_by_range(e)
        } else if name == start {
            if let Some(b) = dom.attribute(e, &id) {
                open.insert(b.to_string(), (0, 0));
            }
            continue;
        } else if name == end {
            if let Some(b) = dom.attribute(e, &id)
                && let Some((chars, gone)) = open.remove(b)
                && chars > 0
                && gone == chars
            {
                dropped.insert(b.to_string());
            }
            continue;
        } else if is_character(dom, e, &name) {
            in_deleted_content(dom, e) || removed_by_range(e)
        } else {
            continue;
        };
        for (chars, gone) in open.values_mut() {
            *chars += 1;
            *gone += usize::from(deleted);
        }
    }
    if dropped.is_empty() {
        return;
    }
    for marker in [start, end] {
        for b in dom.descendants(root, Some(&marker)) {
            if dom.attribute(b, &id).is_some_and(|v| dropped.contains(v)) {
                dom.remove(b);
            }
        }
    }
}

/// Does `e` stand for at least one character of the story?
fn is_character(dom: &Dom, e: NodeId, name: &XName) -> bool {
    if name.namespace_name() == M::URI {
        return name.local_name() == "t" && !dom.value_str(e).is_empty();
    }
    if name.namespace_name() != W::URI {
        return false;
    }
    match name.local_name() {
        "t" | "delText" | "instrText" | "delInstrText" => !dom.value_str(e).is_empty(),
        // w:tab is also a tab stop in pPr/tabs; only the run's is a character.
        "tab" => dom.parent(e).and_then(|p| dom.name(p)) == Some(W::r()),
        "br" | "cr" | "sym" | "drawing" | "pict" | "object" | "noBreakHyphen" | "softHyphen"
        | "footnoteReference" | "endnoteReference" | "fldChar" | "ptab" => true,
        _ => false,
    }
}

/// Is `e` inside deleted or moved-from content, or a deleted row?
fn in_deleted_content(dom: &Dom, e: NodeId) -> bool {
    let (del, move_from) = (W::del(), W::move_from());
    dom.ancestors(e, None).into_iter().any(|a| {
        dom.name(a)
            .is_some_and(|n| n == del || n == move_from || (n == W::tr() && row_is_deleted(dom, a)))
    })
}

fn in_deleted_row(dom: &Dom, p: NodeId) -> bool {
    dom.ancestors(p, Some(&W::tr()))
        .into_iter()
        .any(|tr| row_is_deleted(dom, tr))
}

fn row_is_deleted(dom: &Dom, tr: NodeId) -> bool {
    dom.element(tr, &W::tr_pr())
        .is_some_and(|pr| dom.element(pr, &W::del()).is_some())
}

/// Remove the bookmark markers left without their partner: Word's range model
/// has no half bookmark, and the accept transforms can drop one side with
/// the content around it (a deleted row, a moved-from run).
pub(super) fn drop_unpaired_bookmarks(dom: &mut Dom, root: NodeId) {
    let (start, end) = (W::name("bookmarkStart"), W::name("bookmarkEnd"));
    let id = W::id();
    let ids = |dom: &Dom, name: &XName| -> HashSet<String> {
        dom.descendants(root, Some(name))
            .into_iter()
            .filter_map(|b| dom.attribute(b, &id).map(str::to_string))
            .collect()
    };
    let (starts, ends) = (ids(dom, &start), ids(dom, &end));
    if starts == ends {
        return;
    }
    for (marker, partners) in [(start, &ends), (end, &starts)] {
        for b in dom.descendants(root, Some(&marker)) {
            if dom.attribute(b, &id).is_some_and(|v| !partners.contains(v)) {
                dom.remove(b);
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod move_range_bookmark_regressions {
    use crate::namespaces::W;
    use crate::revision_processor::accept_revisions_document;
    use crate::xmllinq::{Dom, NodeId};

    fn accept(body: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w='{}'><w:body>{body}<w:sectPr/></w:body></w:document>",
            W::URI
        ));
        let root = dom.root(doc).unwrap();
        let result = accept_revisions_document(&mut dom, root);
        (dom, result)
    }

    fn names(dom: &Dom, root: NodeId) -> Vec<String> {
        dom.descendants(root, Some(&W::bookmark_start()))
            .into_iter()
            .map(|node| dom.attribute(node, &W::name("name")).unwrap().to_string())
            .collect()
    }

    #[test]
    fn range_only_moved_nonempty_bookmark_disappears_but_original_empty_anchor_survives() {
        let (dom, accepted) = accept(
            "<w:moveFromRangeStart w:id='7'/><w:p><w:bookmarkStart w:id='1' w:name='Gone'/><w:r><w:t>moved away</w:t></w:r><w:bookmarkEnd w:id='1'/><w:bookmarkStart w:id='2' w:name='Empty'/><w:bookmarkEnd w:id='2'/></w:p><w:moveFromRangeEnd w:id='7'/><w:p><w:r><w:t>tail</w:t></w:r></w:p>",
        );
        assert_eq!(dom.value(accepted), "tail");
        assert_eq!(names(&dom, accepted), ["Empty"]);
        let ends = dom.descendants(accepted, Some(&W::bookmark_end()));
        assert_eq!(ends.len(), 1);
        assert_eq!(dom.attribute(ends[0], &W::id()), Some("2"));
        let start = dom.descendants(accepted, Some(&W::bookmark_start()))[0];
        assert_eq!(dom.next_element(start), Some(ends[0]));
    }

    #[test]
    fn partially_surviving_bookmark_keeps_both_anchors_after_range_only_move() {
        let (dom, accepted) = accept(
            "<w:moveFromRangeStart w:id='7'/><w:p><w:bookmarkStart w:id='3' w:name='Partial'/><w:r><w:t>gone</w:t></w:r></w:p><w:moveFromRangeEnd w:id='7'/><w:p><w:r><w:t>kept</w:t></w:r><w:bookmarkEnd w:id='3'/></w:p>",
        );
        assert_eq!(dom.value(accepted), "kept");
        assert_eq!(names(&dom, accepted), ["Partial"]);
        let start = dom.descendants(accepted, Some(&W::bookmark_start()))[0];
        let ends = dom.descendants(accepted, Some(&W::bookmark_end()));
        assert_eq!(ends.len(), 1);
        assert_eq!(
            dom.attribute(ends[0], &W::id()),
            dom.attribute(start, &W::id())
        );
        assert_eq!(dom.descendants(accepted, Some(&W::p())).len(), 1);
    }

    #[test]
    fn unmatched_move_range_keeps_nonempty_bookmark_and_its_source_content() {
        let (dom, accepted) = accept(
            "<w:moveFromRangeStart w:id='7'/><w:p><w:bookmarkStart w:id='4' w:name='Kept'/><w:r><w:t>unmatched</w:t></w:r><w:bookmarkEnd w:id='4'/></w:p><w:moveFromRangeEnd w:id='different'/>",
        );
        assert_eq!(dom.value(accepted), "unmatched");
        assert_eq!(names(&dom, accepted), ["Kept"]);
        assert_eq!(dom.descendants(accepted, Some(&W::bookmark_end())).len(), 1);
    }

    #[test]
    fn entering_only_paragraph_properties_does_not_delete_a_bookmarks_live_character() {
        let (dom, accepted) = accept(
            "<w:bookmarkStart w:id='5' w:name='Kept'/><w:moveFromRangeStart w:id='7'/><w:p><w:pPr><w:keepNext/></w:pPr><w:moveFromRangeEnd w:id='7'/><w:r><w:t>live</w:t></w:r><w:bookmarkEnd w:id='5'/></w:p>",
        );
        assert_eq!(dom.value(accepted), "live");
        assert_eq!(names(&dom, accepted), ["Kept"]);
        assert_eq!(
            dom.descendants(accepted, Some(&W::name("keepNext"))).len(),
            1
        );
        assert_eq!(dom.descendants(accepted, Some(&W::bookmark_end())).len(), 1);
    }
}
