// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `format_run`: change the run formatting of one occurrence of existing
//! text. The copy splits the runs at the range's edges and sets the
//! requested properties on the runs inside; the comparer then records the
//! old properties as `w:rPrChange`.

use crate::inspect::Projection;

use super::{EditOutcome, RunFormat, Transaction, check_format};

impl Transaction<'_> {
    /// Resolve a `format_run`: the range to format, or the error code and
    /// message.
    pub(super) fn resolve_format_run(
        &self,
        projection: &Projection,
        find: &str,
        occurrence: Option<usize>,
        format: &RunFormat,
        outcome: &mut EditOutcome,
    ) -> Result<(usize, usize), (String, String)> {
        if *format == RunFormat::default() {
            return Err((
                "INVALID_EDIT".into(),
                "format_run needs at least one format field".into(),
            ));
        }
        check_format(format, find).map_err(|m| ("INVALID_EDIT".to_string(), m))?;
        self.find_range(projection, find, occurrence, outcome)
    }
}

/// A `format_run` range and a text edit's range in one paragraph conflict
/// when the edit changes text inside the formatted range: a replacement or
/// deletion overlapping it, or an insertion strictly inside it.
pub(super) fn format_overlaps_edit(
    (format_start, format_end): (usize, usize),
    (start, end): (usize, usize),
) -> bool {
    if start == end {
        format_start < start && start < format_end
    } else {
        start < format_end && end > format_start
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::format_overlaps_edit;

    #[test]
    fn insertions_at_the_edges_and_edits_beside_the_range_do_not_overlap() {
        assert!(!format_overlaps_edit((4, 8), (4, 4)));
        assert!(!format_overlaps_edit((4, 8), (8, 8)));
        assert!(format_overlaps_edit((4, 8), (5, 5)));
        assert!(!format_overlaps_edit((4, 8), (0, 4)));
        assert!(!format_overlaps_edit((4, 8), (8, 10)));
        assert!(format_overlaps_edit((4, 8), (7, 10)));
        assert!(format_overlaps_edit((4, 8), (0, 5)));
        assert!(format_overlaps_edit((4, 8), (2, 12)));
    }
}
