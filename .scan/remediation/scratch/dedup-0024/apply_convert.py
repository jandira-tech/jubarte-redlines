import pathlib, sys

path = pathlib.Path("new/src/convert/mod.rs")
text = path.read_text()
orig = text

def sub(old, new, count=1):
    global text
    n = text.count(old)
    if n != count:
        print(f"MISMATCH ({n} != {count}): {old[:90]!r}", file=sys.stderr)
        sys.exit(1)
    text = text.replace(old, new)

# --- V1a: stretch_stale_spans checked accumulation ---
sub("""        let used: usize = row.iter().map(|c| c.colspan.max(1)).sum();""",
    """        let used: usize = row
            .iter()
            .fold(0usize, |acc, c| acc.saturating_add(c.colspan.max(1)));""")
sub("""                    && let Some(&next) = cols.get(col + span + extra)""",
    """                    && let Some(&next) = cols.get(col.saturating_add(span).saturating_add(extra))""")
sub("""            col += cell.colspan.max(1);
        }
    }
}""",
    """            col = col.saturating_add(cell.colspan.max(1));
        }
    }
}""")

# --- V1b: budget constant + cell_span clamp ---
sub("""fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {""",
    """/// The widest table grid a document can ask the layout for: a cell's
/// `w:gridSpan`, a row's `w:gridBefore`/`w:gridAfter` and the columns a
/// row's cells add up to are held at or below this before any grid-sized
/// vector (columns, preferences, merge tracking) is allocated or summed.
/// Word's own tables never pass 63 columns, so every real document is
/// unchanged; a crafted ten-digit `w:gridSpan` sizes vectors in kilobytes,
/// not the ~4 GiB it used to request.
const MAX_TABLE_COLUMNS: usize = 4096;

fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {""")
sub("""        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);
    let vmerge = match direct_named(dom, pr, "vMerge") {""",
    """        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1)
        .min(MAX_TABLE_COLUMNS);
    let vmerge = match direct_named(dom, pr, "vMerge") {""")

# --- V1c: column_prefs saturating columns ---
sub("""        let covered: f32 = (col..col + span)""",
    """        let covered: f32 = (col..col.saturating_add(span))""")
sub("""                grid.get(col + i).copied().unwrap_or(0.0) / covered""",
    """                grid.get(col.saturating_add(i))
                    .copied()
                    .unwrap_or(0.0)
                    / covered""")
sub("""            let Some(slot) = pref.get_mut(col + i) else {""",
    """            let Some(slot) = pref.get_mut(col.saturating_add(i)) else {""")
sub("""                PrefWidth::Auto => {}
            }
        }
        col += span;
    }
    for row in raw_rows.iter().skip(1).filter(|_| fixed) {""",
    """                PrefWidth::Auto => {}
            }
        }
        col = col.saturating_add(span);
    }
    for row in raw_rows.iter().skip(1).filter(|_| fixed) {""")
sub("""            col += span;
        }
    }
    pref
}""",
    """            col = col.saturating_add(span);
        }
    }
    pref
}""")

# --- V1d: row_grid_skip clamp ---
sub("""        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)?;""",
    """        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)?
        .min(MAX_TABLE_COLUMNS);""")

# --- V1e: occupancy checked + capped ---
sub("""    for row in &raw_rows {
        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
    }""",
    """    for row in &raw_rows {
        // Checked accumulation, capped at MAX_TABLE_COLUMNS: however many
        // cells a row claims, the grid they size is bounded.
        occupancy = occupancy.max(
            row.iter()
                .fold(0usize, |acc, c| acc.saturating_add(c.colspan))
                .min(MAX_TABLE_COLUMNS),
        );
    }""")

# --- V1f: cell-loop grid_at ---
sub("""            let (colspan, vmerge) = cell_span(dom, cell);
            let last_col = grid_at + colspan.max(1) >= cols.len();""",
    """            let (colspan, vmerge) = cell_span(dom, cell);
            let last_col = grid_at.saturating_add(colspan.max(1)) >= cols.len();""")
sub("""            grid_at += colspan.max(1);""",
    """            grid_at = grid_at.saturating_add(colspan.max(1));""")

# --- V1g: mirror_table saturating inner add ---
sub("""            cell.col = n.saturating_sub(cell.col + cell.colspan);""",
    """            cell.col = n.saturating_sub(cell.col.saturating_add(cell.colspan));""")

# --- V1h: resolve_table_merges capped columns ---
sub("""                    origin.rowspan += 1;
                }
                col += span;
                continue;""",
    """                    origin.rowspan += 1;
                }
                col = col.saturating_add(span).min(MAX_TABLE_COLUMNS);
                continue;""")
sub("""            if open.len() < col + span {
                open.resize(col + span, None);
            }
            let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
            for slot in open.iter_mut().skip(col).take(span) {
                *slot = mark;
            }
            col += span;""",
    """            let width = col.saturating_add(span);
            if open.len() < width {
                open.resize(width, None);
            }
            let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
            for slot in open.iter_mut().skip(col).take(span) {
                *slot = mark;
            }
            col = col.saturating_add(span).min(MAX_TABLE_COLUMNS);""")

path.write_text(text)
print("convert/mod.rs: all", len(orig) != len(text) or "changed", "replacements applied")
