// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Ring 1 — Word-validity package invariants (plan D1), as the library's
//! `jubarte::validate` reports them.
//!
//! `assert_word_valid_package` fails a test when a produced package would make
//! Word offer repair (dangling rels, duplicate revision ids, orphan comment
//! anchors, …). Intentional broken probes live in `tests/m_validity_ring1.rs`.

use jubarte::validate::{Finding, validate};

/// Failures collected by the Ring-1 checks: one message per finding.
#[derive(Debug, Default)]
pub struct ValidityReport {
    pub errors: Vec<String>,
    pub findings: Vec<Finding>,
}

impl ValidityReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Assert every Ring-1 invariant; panics with the full error list on failure.
pub fn assert_word_valid_package(bytes: &[u8]) {
    let report = check_word_valid_package(bytes);
    assert!(
        report.ok(),
        "Ring-1 Word-validity failed:\n  - {}",
        report.errors.join("\n  - ")
    );
}

/// Run all Ring-1 checks without panicking (for probe tests). A package
/// `validate` cannot read at all is one error naming why.
pub fn check_word_valid_package(bytes: &[u8]) -> ValidityReport {
    match validate(bytes) {
        Ok(findings) => ValidityReport {
            errors: findings
                .iter()
                .map(|f| format!("{} [{}#{}]", f.message, f.part, f.path))
                .collect(),
            findings,
        },
        Err(e) => ValidityReport {
            errors: vec![format!("not a readable package: {e}")],
            findings: Vec::new(),
        },
    }
}
