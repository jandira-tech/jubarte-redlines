import pathlib, sys

def sub(text, old, new, label):
    n = text.count(old)
    if n != 1:
        print(f"MISMATCH {label} ({n})", file=sys.stderr)
        sys.exit(1)
    return text.replace(old, new)

# --- convert/mod.rs ---
p = pathlib.Path("new/src/convert/mod.rs")
text = p.read_text()
# grid_skip branch of column_prefs: saturate like every other column walk
text = sub(text, """        if cell.grid_skip {
            col += span;
            continue;
        }""",
"""        if cell.grid_skip {
            col = col.saturating_add(span);
            continue;
        }""", "column_prefs grid_skip col")
# share closure: one line (under rustfmt's width)
text = sub(text, """                grid.get(col.saturating_add(i))
                    .copied()
                    .unwrap_or(0.0)
                    / covered""",
"""                grid.get(col.saturating_add(i)).copied().unwrap_or(0.0) / covered""", "share closure")
p.write_text(text)
print("convert fixups applied")

# --- xmllinq/parse.rs ---
p = pathlib.Path("new/src/xmllinq/parse.rs")
text = p.read_text()
text = sub(text, """            while chars
                .get(j)
                .is_some_and(|&c| c.is_ascii_alphanumeric() || c == '+')
            {
                j += 1;
            }""",
"""            while chars.get(j).is_some_and(|&c| c.is_ascii_alphanumeric() || c == '+') {
                j += 1;
            }""", "unescape while")
p.write_text(text)
print("parse fixups applied")

# --- legacy_doc.rs ---
p = pathlib.Path("new/src/legacy_doc.rs")
text = p.read_text()
text = sub(text, """        logical = logical
            .checked_add(u64::from(end - piece.cp_start))
            .ok_or_else(huge_story)?;""",
"""        logical = logical
            .checked_add(u64::from(end - piece.cp_start))
            .ok_or_else(huge_story)?;""", "noop check")
# one line fits in 100: join
text = sub(text, """        logical = logical
            .checked_add(u64::from(end - piece.cp_start))
            .ok_or_else(huge_story)?;""",
"""        logical = logical.checked_add(u64::from(end - piece.cp_start)).ok_or_else(huge_story)?;""", "logical one line")
p.write_text(text)
print("legacy fixups applied")
