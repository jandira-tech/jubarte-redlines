// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `ComparisonLog.ts` — diagnostic log emitted during comparison.

/// Severity / category codes for comparison log entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComparisonLogCode {
    /// Public API item.
    Info,
    /// Public API item.
    Warning,
    /// Public API item.
    Error,
}
/// `ComparisonLogEntry`.

#[derive(Clone, Debug)]
pub struct ComparisonLogEntry {
    /// `code`.
    pub code: ComparisonLogCode,
    /// `message`.
    pub message: String,
}

/// Accumulates log entries during a comparison run.
#[derive(Default)]
pub struct ComparisonLog {
    /// `entries`.
    pub entries: Vec<ComparisonLogEntry>,
}

impl ComparisonLog {
    /// `new`.
    pub fn new() -> Self {
        ComparisonLog::default()
    }
    /// `info`.
    pub fn info(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Info,
            message: message.into(),
        });
    }
    /// `warning`.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Warning,
            message: message.into(),
        });
    }
    /// `error`.
    pub fn error(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Error,
            message: message.into(),
        });
    }
    /// `error_count`.
    pub fn error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.code == ComparisonLogCode::Error)
            .count()
    }
}
