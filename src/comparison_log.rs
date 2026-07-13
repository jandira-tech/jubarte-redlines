//! Port of `ComparisonLog.ts` — diagnostic log emitted during comparison.

/// Severity / category codes for comparison log entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComparisonLogCode {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct ComparisonLogEntry {
    pub code: ComparisonLogCode,
    pub message: String,
}

/// Accumulates log entries during a comparison run.
#[derive(Default)]
pub struct ComparisonLog {
    pub entries: Vec<ComparisonLogEntry>,
}

impl ComparisonLog {
    pub fn new() -> Self {
        ComparisonLog::default()
    }
    pub fn info(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Info,
            message: message.into(),
        });
    }
    pub fn warning(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Warning,
            message: message.into(),
        });
    }
    pub fn error(&mut self, message: impl Into<String>) {
        self.entries.push(ComparisonLogEntry {
            code: ComparisonLogCode::Error,
            message: message.into(),
        });
    }
    pub fn error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.code == ComparisonLogCode::Error)
            .count()
    }
}
