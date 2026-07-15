//! NAME-01 — hottest W/PT XNames are cached and stay equal to uncached get.

use jubarte::namespaces::{PT, W};
use jubarte::xmllinq::XName;

#[test]
fn name01_w_t_equals_xname_get() {
    let a = W::t();
    let b = XName::get("t", W::URI);
    assert_eq!(a, b);
    assert_eq!(a.local_name(), "t");
    assert_eq!(a.namespace_name(), W::URI);
}

#[test]
fn name01_cached_stable_across_calls() {
    let a = W::p();
    let b = W::p();
    assert_eq!(a, b);
    // Same Arc local after OnceLock warm-up (clone of cached XName).
    assert_eq!(a.local_name(), b.local_name());
    assert_eq!(W::r_pr(), XName::get("rPr", W::URI));
    assert_eq!(W::del(), XName::get("del", W::URI));
    assert_eq!(W::ins(), XName::get("ins", W::URI));
}

#[test]
fn name01_pt_unid_equals_get() {
    assert_eq!(PT::unid(), XName::get("Unid", PT::URI));
    assert_eq!(PT::sha1_hash(), XName::get("SHA1Hash", PT::URI));
}

#[test]
fn name01_distinct_names_still_unequal() {
    assert_ne!(W::t(), W::r());
    assert_ne!(W::p(), W::body());
}
