// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::pdf_page_count;

#[test]
fn page_count_ignores_pages_dictionary() {
    let pdf = b"%PDF-1.4\n/Type /Pages\n/Type /Page\n/Type /Page\n";
    assert_eq!(pdf_page_count(pdf), 2);
}
