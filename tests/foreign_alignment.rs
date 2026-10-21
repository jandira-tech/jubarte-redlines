// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Deterministic allocation budgets for dense foreign-history alignments.
#![expect(
    unsafe_code,
    reason = "counting allocator follows the existing perf test harness; production code stays safe"
)]

mod common;

use common::docx::{docx, para};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;
static LARGEST: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LARGEST.fetch_max(new_size, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;

fn bounded_audit(before: &str, after: &str, foreign: usize, untracked: usize) {
    let original = docx(before);
    let edited = docx(after);
    LARGEST.store(0, Ordering::Relaxed);
    let findings = jubarte::validate::audit_tracked(&original, &edited, "Bob").unwrap();
    let largest = LARGEST.load(Ordering::Relaxed);
    eprintln!("largest audit allocation: {largest} bytes");
    assert!(
        largest <= 128 * 1024 * 1024,
        "largest audit allocation {largest} exceeds 128 MiB"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.code == "FOREIGN_AUTHOR")
            .count(),
        foreign
    );
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.code == "UNTRACKED_EDIT")
            .count(),
        untracked
    );
    assert_eq!(findings.len(), foreign + untracked);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "10,000-paragraph allocation stress probe uses the native allocator"
)]
fn foreign_ten_thousand_empty_paragraphs_have_bounded_allocations() {
    let middle = "<w:p/>".repeat(10_000);
    bounded_audit(
        &(para("old first") + &middle + &para("old last")),
        &(para("new first") + &middle + &para("new last")),
        0,
        2,
    );
}

#[test]
#[cfg_attr(
    miri,
    ignore = "10,000-revision allocation stress probe uses the native allocator"
)]
fn foreign_ten_thousand_repeated_deletions_have_bounded_allocations() {
    let make = |changed: bool| {
        let mut body = String::from("<w:p>");
        for i in 0..10_000 {
            let text = if changed && (i == 0 || i == 9_999) {
                "changed"
            } else {
                "deleted"
            };
            body.push_str(&format!(r#"<w:del w:id="{}" w:author="Alice"><w:r><w:delText>{text}</w:delText></w:r></w:del><w:r><w:t> </w:t></w:r>"#, i + 1));
        }
        body.push_str("</w:p>");
        body
    };
    // Identical texts have no unique carrier identity: the ordered alignment
    // reports two edited carriers and one unmatched original carrier.
    bounded_audit(&make(false), &make(true), 3, 0);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "10,000-paragraph allocation stress probe uses the native allocator"
)]
fn foreign_ten_thousand_alternating_paragraphs_have_bounded_allocations() {
    let make = |changed: bool| {
        (0..10_000)
            .map(|i| {
                para(if changed && i == 5_000 {
                    "changed"
                } else if i % 2 == 0 {
                    "a"
                } else {
                    "b"
                })
            })
            .collect::<String>()
    };
    bounded_audit(&make(false), &make(true), 0, 1);
}
