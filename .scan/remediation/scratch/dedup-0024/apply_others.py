import pathlib, sys

def sub(text, old, new, label):
    n = text.count(old)
    if n != 1:
        print(f"MISMATCH {label} ({n})", file=sys.stderr)
        sys.exit(1)
    return text.replace(old, new)

# --- altchunk.rs: bounded entity window + test ---
p = pathlib.Path("new/src/convert/altchunk.rs")
text = p.read_text()
text = sub(text, """        let tail = &rest[a..];
        let semi = tail.find(';').filter(|&e| e <= 10);""",
"""        let tail = &rest[a..];
        // Only a `;` within the accepted entity window (the filter below)
        // can close a reference; searching the whole suffix made a run of
        // bare `&` quadratic. The cut is extended to a UTF-8 boundary so
        // a multibyte character beside the window never splits.
        let mut window = tail.len().min(11);
        while window < tail.len() && !tail.is_char_boundary(window) {
            window += 1;
        }
        let semi = tail[..window].find(';').filter(|&e| e <= 10);""", "altchunk window")
anchor = """            let invalid = "&unknown; &#xD800; &#1114112; &#xZZ; &unfinished";
            assert_eq!(decode_entities(invalid), invalid);
            let wml = html_to_wml("<p>&lt;tag&gt; &amp; &unknown;</p>");
            assert!(
                wml.contains(">&lt;tag&gt; &amp; &amp;unknown;</w:t>"),
                "{wml}"
            );
        }
"""
addition = anchor + """
        #[test]
        fn bare_ampersand_runs_decode_in_one_pass() {
            // The `;` that can close an entity is only sought within the
            // ten-byte window that can hold one; a run of bare `&` (or a
            // `;` past a non-entity character) never restarts the search
            // over the whole suffix.
            let bare = "&".repeat(100_000);
            assert_eq!(decode_entities(&bare), bare);
            let far = format!("&{};", "x".repeat(40));
            assert_eq!(decode_entities(&far), far);
            // A multibyte character right after `&` is not split by the
            // window, and the entity after it still decodes.
            assert_eq!(decode_entities("&\u00e9&nbsp;"), "&\u00e9\\u{a0}");
        }
"""
text = sub(text, anchor, addition, "altchunk test anchor")
p.write_text(text)
print("altchunk.rs applied")

# --- opc/mod.rs: single decode + test ---
p = pathlib.Path("new/src/opc/mod.rs")
text = p.read_text()
text = sub(text, """    pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
        // PartFs receives raw relationship attributes; DOM consumers already
        // decoded XML entities and call the pure resolver directly.
        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
        resolve_rel_target(source_part, &target)
    }""",
"""    pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
        // Relationship targets are stored decoded exactly once (the OPC
        // parser normalizes attribute values); resolving used to decode
        // them again, so a part literally named `a&amp;b.png` resolved to
        // `a&b.png` — a part that does not exist — and a Target full of
        // `&amp;` reached the entity decoder as a run of bare `&`.
        resolve_rel_target(source_part, rel_target)
    }""", "opc resolve")
anchor = """        assert_eq!(
            fs.resolve_rel_target("/word/document.xml", "styles.xml"),
            "word/styles.xml"
        );
    }
"""
addition = anchor + """
    #[test]
    fn a_rel_target_is_decoded_once_a_literal_ampersand_part_name_resolves() {
        // oxml-opc stores Target already entity-decoded; resolving used to
        // decode it a second time, so the only correct spelling of a part
        // named `a&amp;b.png` (Target="media/a&amp;amp;b.png") resolved to
        // `a&b.png`, a part that does not exist.
        let bytes = package_with_document_rels(
            r#"<Relationship Id="rId7" Type="image" Target="media/a&amp;amp;b.png"/>"#,
        );
        let fs = PartFs::open(&bytes).unwrap();
        assert_eq!(
            fs.read_rels_for("word/document.xml").unwrap().items[0].target,
            "media/a&amp;b.png"
        );
        assert_eq!(
            fs.resolve_rel_target("word/document.xml", "media/a&amp;b.png"),
            "word/media/a&amp;b.png"
        );
    }
"""
text = sub(text, anchor, addition, "opc test anchor")
p.write_text(text)
print("opc/mod.rs applied")
