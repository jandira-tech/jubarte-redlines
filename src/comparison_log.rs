// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `ComparisonLog.ts` — diagnostic log emitted during comparison.
//!
//! A `ComparisonLog` accumulates entries at three severity levels; the
//! module itself just stores them in insertion order, and consumers
//! (the host, the `error_count` helper) interpret the severities.
//! Severity is therefore purely a tag the caller chooses when it appends.

/// Severity / category tag of a [`ComparisonLogEntry`]. The variant has
/// no behaviour of its own; the meaning (`Info` / `Warning` / `Error`)
/// is whatever the host or downstream tool assigns to it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComparisonLogCode {
    /// An entry logged via [`ComparisonLog::info`].
    Info,
    /// An entry logged via [`ComparisonLog::warning`].
    Warning,
    /// An entry logged via [`ComparisonLog::error`].
    Error,
}

/// One entry in a [`ComparisonLog`]: a severity tag and the message.
#[derive(Clone, Debug)]
pub struct ComparisonLogEntry {
    /// Severity / category the entry was logged at.
    pub code: ComparisonLogCode,
    /// Human-readable message, English, one line.
    pub message: String,
}

/// Accumulates log entries during a comparison run, in insertion order.
#[derive(Debug, Default)]
pub struct ComparisonLog {
    /// Entries in the order they were logged.
    pub entries: Vec<ComparisonLogEntry>,
}

impl ComparisonLog {
    /// Returns an empty log; equivalent to [`Default::default`].
    pub fn new() -> Self {
        ComparisonLog::default()
    }

    /// Appends a [`ComparisonLogCode::Info`] entry with `message`.
    pub fn info(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Info,
            message: message.into(),
        });
    }

    /// Appends a [`ComparisonLogCode::Warning`] entry with `message`.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Warning,
            message: message.into(),
        });
    }

    /// Appends a [`ComparisonLogCode::Error`] entry with `message`.
    pub fn error(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Error,
            message: message.into(),
        });
    }

    /// Number of [`ComparisonLogCode::Error`] entries currently in the log.
    pub fn error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.code == ComparisonLogCode::Error)
            .count()
    }
}
