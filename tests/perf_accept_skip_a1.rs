// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ACCEPT-SKIP-A1 — field-code fixup transfers root when no fldChar/instrText.

use jubarte::namespaces::W;
use jubarte::revision_processor::fix_up_deleted_or_inserted_field_codes_transform;
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

#[test]
fn skip_a1_no_fields_preserves_root() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "hi");
    d.add(r, t);
    d.add(p, r);
    d.add(body, p);
    let id = body;
    assert_eq!(
        fix_up_deleted_or_inserted_field_codes_transform(&mut d, body),
        id
    );
}
