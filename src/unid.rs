//! Port of `UnidHelper` (`UnidHelper.ts`) — M1.7.
//!
//! The `PtOpenXml.Unid` is a 32-hex stable id. `WmlComparer` uses the random-Guid
//! path (`AssignToAllElements`), whose only requirement (per the TS docs) is that
//! ids be **unique and content-independent within each version**. We satisfy that
//! with a process-wide monotonic counter formatted as 32 hex chars instead of a
//! random GUID: same algorithmic properties, but deterministic (so the whole
//! pipeline is reproducible and testable). The counter is content-independent and
//! unique, exactly what the matching heuristics assume.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::namespaces::{PT, W};
use crate::xmllinq::{Dom, NodeId};

static UNID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// `UnidHelper.GenerateUnid()` — a unique 32-char hex id.
pub fn generate_unid() -> String {
    let n = UNID_COUNTER.fetch_add(1, Ordering::Relaxed);
    // 32 hex chars: high 16 zero-padded + the counter in the low 16.
    format!("{n:032x}")
}

/// `UnidHelper.AssignToAllElements(contentParent)` — stamp a `PtOpenXml.Unid` on
/// the root (if it is a footnote/endnote) and on every descendant lacking one.
pub fn assign_to_all_elements(dom: &mut Dom, content_parent: NodeId) {
    let unid = PT::unid();
    if let Some(name) = dom.name(content_parent)
        && (name == W::footnote() || name == W::endnote())
        && dom.attribute(content_parent, &unid).is_none()
    {
        let v = generate_unid();
        dom.set_attribute_value(content_parent, &unid, Some(&v));
    }
    for d in dom.descendants(content_parent, None) {
        if dom.attribute(d, &unid).is_none() {
            let v = generate_unid();
            dom.set_attribute_value(d, &unid, Some(&v));
        }
    }
}

/// `UnidHelper.AssignToSelfAndDescendants(root)` — like the above but always
/// stamps the root regardless of its name.
pub fn assign_to_self_and_descendants(dom: &mut Dom, root: NodeId) {
    let unid = PT::unid();
    if dom.attribute(root, &unid).is_none() {
        let v = generate_unid();
        dom.set_attribute_value(root, &unid, Some(&v));
    }
    for d in dom.descendants(root, None) {
        if dom.attribute(d, &unid).is_none() {
            let v = generate_unid();
            dom.set_attribute_value(d, &unid, Some(&v));
        }
    }
}
