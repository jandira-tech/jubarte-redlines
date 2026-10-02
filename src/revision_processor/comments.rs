// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comments after Accept All / Reject All, as Word saves them.
//!
//! A comment whose `w:commentReference` went with the accepted deletion (or
//! the rejected insertion) is gone: its range markers, its `comments.xml`
//! entry and its `commentsExtended` / `commentsIds` / `commentsExtensible`
//! entries; with none left, the comment parts are not written. The comments
//! left are renumbered with the bookmarks ([`super::annotation_ids`]). `people.xml` keeps the
//! authors of the comments left (no revision survives Accept or Reject All),
//! and goes when it keeps nobody.

use std::collections::HashSet;

use crate::comparer::comments::FAMILY;
use crate::namespaces::{W, W14, W15};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

const CID: &str = "http://schemas.microsoft.com/office/word/2016/wordml/cid";
const CEX: &str = "http://schemas.microsoft.com/office/word/2018/wordml/cex";
const PEOPLE: (&str, &str) = (
    "word/people.xml",
    "http://schemas.microsoft.com/office/2011/relationships/people",
);

/// One parsed part, written back by [`Parsed::store`].
pub(super) struct Parsed {
    name: String,
    pub(super) dom: Dom,
    doc: NodeId,
    pub(super) root: NodeId,
}

impl Parsed {
    pub(super) fn load(pkg: &PartFs, name: &str) -> Option<Self> {
        let xml = pkg.part_string(name)?;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc)?;
        Some(Self {
            name: name.to_string(),
            dom,
            doc,
            root,
        })
    }

    pub(super) fn store(self, pkg: &mut PartFs) {
        pkg.set_part(
            &self.name,
            self.dom.serialize_document(self.doc).into_bytes(),
        );
    }

    /// Descendants named `local` in the W namespace.
    pub(super) fn w(&self, local: &str) -> Vec<NodeId> {
        self.dom.descendants(self.root, Some(&W::name(local)))
    }
}

/// Drop the comments Accept / Reject All left without a reference, prune the
/// comment family and `people.xml` (module docs).
pub(super) fn prune_orphan_comments(pkg: &mut PartFs, story_parts: &[String]) {
    let Some(mut comments) = Parsed::load(pkg, FAMILY[0].0) else {
        return;
    };
    let stories: Vec<Parsed> = story_parts
        .iter()
        .filter_map(|p| Parsed::load(pkg, p))
        .collect();
    let id = W::id();
    let referenced: HashSet<String> = stories
        .iter()
        .flat_map(|s| {
            s.w("commentReference")
                .into_iter()
                .filter_map(|r| s.dom.attribute(r, &id).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect();

    let mut kept: HashSet<String> = HashSet::new();
    let mut dropped = false;
    let mut dropped_paras: HashSet<String> = HashSet::new();
    let mut authors: HashSet<String> = HashSet::new();
    let para_id = W14::name("paraId");
    for c in comments.w("comment") {
        let cid = comments.dom.attribute(c, &id).unwrap_or("").to_string();
        if referenced.contains(&cid) {
            kept.insert(cid);
            if let Some(a) = comments.dom.attribute(c, &W::name("author")) {
                authors.insert(a.to_string());
            }
        } else {
            for p in comments.dom.descendants(c, Some(&W::p())) {
                if let Some(pid) = comments.dom.attribute(p, &para_id) {
                    dropped_paras.insert(pid.to_string());
                }
            }
            comments.dom.remove(c);
            dropped = true;
        }
    }
    // Anchors of the dropped comments go (their commentReference went with
    // the deleted text by construction).
    for mut s in stories {
        let mut changed = false;
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for m in s.w(local) {
                if !s.dom.attribute(m, &id).is_some_and(|v| kept.contains(v)) {
                    s.dom.remove(m);
                    changed = true;
                }
            }
        }
        changed |= collapse_cut_ranges(&mut s);
        if changed {
            s.store(pkg);
        }
    }

    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    if kept.is_empty() {
        for (part, _, rel_type) in FAMILY {
            remove_part(pkg, &main, part, rel_type);
        }
    } else if dropped {
        // An untouched part keeps its bytes.
        comments.store(pkg);
        prune_by_para(pkg, &dropped_paras);
    }
    prune_people(pkg, &main, &authors);
}

/// A comment whose reference stayed while resolved content took one end of
/// its range (resolving some changes and keeping others can cut a range
/// that way) gets the missing marker back where the range was cut short:
/// the start just before the surviving end, the end just before the
/// reference's run. True when a marker was added.
fn collapse_cut_ranges(s: &mut Parsed) -> bool {
    let id = W::id();
    let ids_of = |s: &Parsed, local: &str| -> HashSet<String> {
        s.w(local)
            .into_iter()
            .filter_map(|m| s.dom.attribute(m, &id).map(str::to_string))
            .collect()
    };
    let (starts, ends) = (ids_of(s, "commentRangeStart"), ids_of(s, "commentRangeEnd"));
    let mut changed = false;
    for reference in s.w("commentReference") {
        let Some(cid) = s.dom.attribute(reference, &id).map(str::to_string) else {
            continue;
        };
        let run = s
            .dom
            .parent(reference)
            .filter(|&r| s.dom.name(r) == Some(W::r()))
            .unwrap_or(reference);
        let end = if ends.contains(&cid) {
            s.w("commentRangeEnd")
                .into_iter()
                .find(|&e| s.dom.attribute(e, &id) == Some(cid.as_str()))
        } else if starts.contains(&cid) {
            let e = marker(&mut s.dom, "commentRangeEnd", &cid);
            s.dom.add_before_self(run, e);
            changed = true;
            Some(e)
        } else {
            None
        };
        if !starts.contains(&cid)
            && let Some(end) = end
        {
            let start = marker(&mut s.dom, "commentRangeStart", &cid);
            s.dom.add_before_self(end, start);
            changed = true;
        }
    }
    changed
}

fn marker(dom: &mut Dom, local: &str, cid: &str) -> NodeId {
    let m = dom.new_element(W::name(local));
    dom.set_attribute_value(m, &W::id(), Some(cid));
    m
}

/// Drop the commentsExtended / commentsIds entries of the dropped comments'
/// paragraphs and the commentsExtensible entries of their durable ids.
fn prune_by_para(pkg: &mut PartFs, dropped_paras: &HashSet<String>) {
    if dropped_paras.is_empty() {
        return;
    }
    if let Some(mut ext) = Parsed::load(pkg, FAMILY[1].0) {
        let key = W15::name("paraId");
        for e in ext.dom.descendants(ext.root, Some(&W15::name("commentEx"))) {
            if ext
                .dom
                .attribute(e, &key)
                .is_some_and(|p| dropped_paras.contains(p))
            {
                ext.dom.remove(e);
            }
        }
        ext.store(pkg);
    }
    let mut durable: HashSet<String> = HashSet::new();
    if let Some(mut ids) = Parsed::load(pkg, FAMILY[2].0) {
        let (key, dur) = (XName::get("paraId", CID), XName::get("durableId", CID));
        for e in ids
            .dom
            .descendants(ids.root, Some(&XName::get("commentId", CID)))
        {
            if ids
                .dom
                .attribute(e, &key)
                .is_some_and(|p| dropped_paras.contains(p))
            {
                if let Some(d) = ids.dom.attribute(e, &dur) {
                    durable.insert(d.to_string());
                }
                ids.dom.remove(e);
            }
        }
        ids.store(pkg);
    }
    if let Some(mut cex) = Parsed::load(pkg, FAMILY[3].0) {
        let key = XName::get("durableId", CEX);
        for e in cex
            .dom
            .descendants(cex.root, Some(&XName::get("commentExtensible", CEX)))
        {
            if cex
                .dom
                .attribute(e, &key)
                .is_some_and(|d| durable.contains(d))
            {
                cex.dom.remove(e);
            }
        }
        cex.store(pkg);
    }
}

/// Keep the people who authored a comment left; drop the part when none is.
fn prune_people(pkg: &mut PartFs, main: &str, authors: &HashSet<String>) {
    let Some(mut people) = Parsed::load(pkg, PEOPLE.0) else {
        return;
    };
    let author = W15::name("author");
    let mut left = 0;
    for p in people
        .dom
        .descendants(people.root, Some(&W15::name("person")))
    {
        if people
            .dom
            .attribute(p, &author)
            .is_some_and(|a| authors.contains(a))
        {
            left += 1;
        } else {
            people.dom.remove(p);
        }
    }
    if left == 0 {
        remove_part(pkg, main, PEOPLE.0, PEOPLE.1);
    } else {
        people.store(pkg);
    }
}

/// Remove `part`, the main part's relationship to it and its content type.
fn remove_part(pkg: &mut PartFs, main: &str, part: &str, rel_type: &str) {
    if pkg.part_bytes(part).is_none() {
        return;
    }
    pkg.remove_part(part);
    pkg.remove_relationships_by_type(main, rel_type);
    pkg.remove_content_type_override(&format!("/{part}"));
}
