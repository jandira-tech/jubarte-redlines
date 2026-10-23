import pathlib, sys

p = pathlib.Path("new/src/xmllinq/parse.rs")
text = p.read_text()
old = """            } else if ch == '<' && self.c.get(i + 1) == Some(&'!') {
                // An internal-subset comment (`<!-- … -->`) or conditional
                // section (`<![ … ]]>`) is opaque inside the declaration:
                // skip it whole; its own `>` is not this one.
                let close = match self.c.get(i + 2) {
                    Some('-') if self.c.get(i + 3) == Some(&'-') => Some("-->"),
                    Some('[') => Some("]]>"),
                    _ => None,
                };
                if let Some(close) = close
                    && let Some(end) = self.index_of(close, i)
                {
                    i = end + close.chars().count();
                    continue;
                }
                return len;
            }"""
new = """            } else if ch == '<' && self.c.get(i + 1) == Some(&'!') {
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
            }"""
n = text.count(old)
if n != 1:
    print(f"MISMATCH ({n})", file=sys.stderr)
    sys.exit(1)
p.write_text(text.replace(old, new))
print("bang_construct_end fixed")
