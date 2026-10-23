import pathlib, sys

p = pathlib.Path("new/src/xmllinq/parse.rs")
text = p.read_text()
old = """    /// End offset of the top-level `<!` construct that starts at `at`: a
    /// declaration (a DOCTYPE with its whole internal subset) closes at
    /// the first `>` outside a quoted literal, an internal-subset comment
    /// or a conditional section — none of which a bare first-`>` skip
    /// respects. The input's length when the construct never closes.
    fn bang_construct_end(&self, at: usize) -> usize {
        let len = self.len();
        let mut quote = '\\0';
        let mut i = at + 2; // past "<!"
        while i < len {
            let ch = self.c[i];
            if quote != '\\0' {
                if ch == quote {
                    quote = '\\0';
                }
            } else if ch == '"' || ch == '\\'' {
                quote = ch;
            } else if ch == '>' {
                return i + 1;
            } else if ch == '<' && self.c.get(i + 1) == Some(&'!') {
                // An internal-subset comment (`<!-- … -->`) or conditional
                // section (`<![ … ]]>`) is opaque inside the declaration:
                // skip it whole; its own `>` is not this one. Any other
                // nested `<!` (an `<!ENTITY>`) is scanned like the rest —
                // its quotes and its `>` are handled above.
                let close = match self.c.get(i + 2) {
                    Some('-') if self.c.get(i + 3) == Some(&'-') => Some("-->"),
                    Some('[') => Some("]]>"),
                    _ => None,
                };
                if let Some(close) = close {
                    let Some(end) = self.index_of(close, i) else {
                        return len;
                    };
                    i = end + close.chars().count();
                    continue;
                }
            }
            i += 1;
        }
        len
    }"""
new = """    /// End offset of the top-level `<!` construct that starts at `at`: a
    /// declaration closes at the first `>` outside a quoted literal, an
    /// internal-subset comment or conditional section, and outside the
    /// `[`-`[` bracket of a DOCTYPE's internal subset — none of which a
    /// bare first-`>` skip respects. The input's length when the
    /// construct never closes.
    fn bang_construct_end(&self, at: usize) -> usize {
        let len = self.len();
        let mut quote = '\\0';
        let mut brackets = 0usize;
        let mut i = at + 2; // past "<!"
        while i < len {
            let ch = self.c[i];
            if quote != '\\0' {
                if ch == quote {
                    quote = '\\0';
                }
            } else if ch == '"' || ch == '\\'' {
                quote = ch;
            } else if ch == '>' && brackets == 0 {
                return i + 1;
            } else if ch == '[' {
                brackets += 1;
            } else if ch == ']' {
                brackets = brackets.saturating_sub(1);
            } else if ch == '<' && self.c.get(i + 1) == Some(&'!') {
                // An internal-subset comment (`<!-- … -->`) or conditional
                // section (`<![ … ]]>`) is opaque inside the declaration:
                // skip it whole; its own `>` is not this one. Any other
                // nested `<!` (an `<!ENTITY>`) is scanned like the rest —
                // its quotes and its `>` are handled above.
                let close = match self.c.get(i + 2) {
                    Some('-') if self.c.get(i + 3) == Some(&'-') => Some("-->"),
                    Some('[') => Some("]]>"),
                    _ => None,
                };
                if let Some(close) = close {
                    let Some(end) = self.index_of(close, i) else {
                        return len;
                    };
                    i = end + close.chars().count();
                    continue;
                }
            }
            i += 1;
        }
        len
    }"""
n = text.count(old)
if n != 1:
    print(f"MISMATCH ({n})", file=sys.stderr)
    sys.exit(1)
p.write_text(text.replace(old, new))
print("bang_construct_end bracket depth added")
