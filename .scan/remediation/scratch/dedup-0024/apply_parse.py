import pathlib, sys

path = pathlib.Path("new/src/xmllinq/parse.rs")
text = path.read_text()

def sub(old, new, count=1):
    global text
    n = text.count(old)
    if n != count:
        print(f"MISMATCH ({n} != {count}): {old[:80]!r}", file=sys.stderr)
        sys.exit(1)
    text = text.replace(old, new)

# --- bang_construct_end helper after index_of ---
sub("""        None
    }

    fn slice(&self, start: usize, end: usize) -> String {""",
    """        None
    }

    /// End offset of the top-level `<!` construct that starts at `at`: a
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
            }
            i += 1;
        }
        len
    }

    fn slice(&self, start: usize, end: usize) -> String {""")

# --- parse_document: complete-construct skip ---
sub("""            if self.starts_with("<!") {
                // DOCTYPE — skip
                let end = self.index_of(">", self.pos);
                self.pos = end.map(|e| e + 1).unwrap_or(len);
                continue;
            }""",
    """            if self.starts_with("<![CDATA[") {
                // A CDATA section outside the root is one opaque token, as
                // the element-level branch below reads it inside a root:
                // its body can hold any character, `>` included, and is
                // never markup.
                let end = self.index_of("]]>", self.pos);
                self.pos = end.map(|e| e + 3).unwrap_or(len);
                continue;
            }
            if self.starts_with("<!") {
                // A declaration (a DOCTYPE with its whole internal subset)
                // is one opaque token too: a quoted `>` inside an entity
                // value, or one inside an internal-subset comment, is not
                // the construct's end. Skipping to the first `>` used to
                // reparse the rest of the subset as elements, nesting past
                // the depth the admission scan had cleared.
                self.pos = self.bang_construct_end(self.pos);
                continue;
            }""")

# --- unescape_xml_text linear rewrite ---
sub("""/// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
/// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
/// literal. Input with no `&` short-circuits to a clone of `s`.
pub fn unescape_xml_text(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            // find ';'
            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
                let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                if let Some(decoded) = decode_entity(&body) {
                    out.push_str(&decoded);
                    i = i + 1 + semi_rel + 1;
                    continue;
                }
            }
            out.push('&');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}""",
    """/// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
/// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
/// literal. Input with no `&` short-circuits to a clone of `s`.
///
/// One pass: the `;` that can close a reference is only ever sought at
/// the end of the short run that could form a body, so a suffix of bare
/// `&` is never rescanned (the whole-suffix search made such runs
/// quadratic).
pub fn unescape_xml_text(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            // The body of a reference that can decode is ASCII
            // alphanumeric, with `#` first (`+` too: the number parser
            // accepts a leading sign). A `;` past any other character can
            // no longer close a reference this `&` opens.
            let mut j = i + 1;
            if chars.get(j) == Some(&'#') {
                j += 1;
            }
            while chars
                .get(j)
                .is_some_and(|&c| c.is_ascii_alphanumeric() || c == '+')
            {
                j += 1;
            }
            if chars.get(j) == Some(&';') {
                let body: String = chars[i + 1..j].iter().collect();
                if let Some(decoded) = decode_entity(&body) {
                    out.push_str(&decoded);
                    i = j + 1;
                    continue;
                }
            }
            out.push('&');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}""")

# --- boundary-contract test ---
sub("""        let root = dom.root(document).unwrap();
        assert_eq!(dom.value(root), "owned");
        assert!(validate_xml("<?xml-stylesheet type='text/css'?>\\n<p>owned</p><?tail?>").is_ok());
    }
}""",
    """        let root = dom.root(document).unwrap();
        assert_eq!(dom.value(root), "owned");
        assert!(validate_xml("<?xml-stylesheet type='text/css'?>\\n<p>owned</p><?tail?>").is_ok());
    }

    #[test]
    fn a_dtds_internal_subset_is_one_opaque_token_whatever_it_quotes() {
        // A quoted `>` inside an entity value, a comment holding one and a
        // top-level CDATA body are not markup: the permissive DOM consumes
        // the whole construct, as the streaming reader and the admission
        // scan do, and the elements they hide never enter the document.
        // Skipping to the first `>` used to parse that hidden literal
        // markup as real elements, nesting past the depth the admission
        // scan had cleared.
        for source in [
            "<?xml version=\\"1.0\\"?><!DOCTYPE d [<!ENTITY e \\"x><a><a><a>\\">]><r>t</r>",
            "<!DOCTYPE d [<!-- c > d -->]><r/>",
            "<![CDATA[a > b]]><r/>",
        ] {
            let mut dom = Dom::new();
            let document = parse_xdocument(&mut dom, source);
            let root = dom
                .root(document)
                .unwrap_or_else(|| panic!("the root after the opaque construct: {source}"));
            assert_eq!(dom.name(root).unwrap().local_name(), "r", "{source}");
            assert!(validate_xml(source).is_err(), "{source}");
        }
    }
}""")

path.write_text(text)
print("parse.rs edits applied")
