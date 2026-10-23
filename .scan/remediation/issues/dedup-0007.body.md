@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_d22452ac5ec901578d340334 | Opaque DTD and top-level CDATA bypass package XML depth checks | medium
csf_68acfaf376a60cf8c3d8e2d4 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_85faece8ffd45c8ee175c288 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_f756bbe5509c42bdcaa64921 | Processing renamed auxiliary XML bypasses XML depth limits | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/src/admission.rs b/src/admission.rs
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -515,6 +515,10 @@
 
     let mut total = 0u64;
     let mut kept: BTreeMap<String, Vec<u8>> = BTreeMap::new();
+    // Internal relationship targets (part name → the relationship types
+    // that name it), gathered from every `.rels` part during the scan for
+    // the XML admission pass below the loop.
+    let mut related: BTreeMap<String, Vec<String>> = BTreeMap::new();
     for i in 0..archive.len() {
         let mut entry = archive
             .by_index(i)
@@ -541,6 +545,27 @@
             let n = buf.len() as u64;
             if n <= cap {
                 check_xml_depth(&name, &buf, limits)?;
+                if lower.ends_with(".rels") {
+                    // Consumers select parts through relationships whatever
+                    // the parts are called; remember every internal target
+                    // so the pass below the loop can admit its XML too.
+                    let owner = crate::opc::rels_path_to_part_name(&name)
+                        .unwrap_or_else(|| name.trim_end_matches(".rels").to_string());
+                    for attrs in elements(&buf, "Relationship") {
+                        if attrs
+                            .get("TargetMode")
+                            .is_some_and(|mode| mode == "External")
+                        {
+                            continue;
+                        }
+                        let Some(target) = attrs.get("Target").filter(|t| !t.is_empty()) else {
+                            continue;
+                        };
+                        let kind = attrs.get("Type").cloned().unwrap_or_default();
+                        let resolved = crate::opc::resolve_rel_target(&owner, target);
+                        related.entry(resolved).or_default().push(kind);
+                    }
+                }
                 if lower == "[content_types].xml" || lower == "_rels/.rels" {
                     kept.insert(lower, buf);
                 }
@@ -568,6 +593,17 @@
             .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("{main_part}: {e}")))?;
         check_xml_depth(&main_part, &buf, limits)?;
     }
+    // Parts a relationship reaches under a non-XML name never met the XML
+    // budgets above; admit the XML among them before any consumer parses
+    // it (render, revision listing and comparison all select parts by
+    // relationship, extension notwithstanding).
+    admit_related_xml(
+        &mut archive,
+        &related,
+        kept.get("[content_types].xml"),
+        &main_part,
+        limits,
+    )?;
     Ok(AdmittedPackage {
         entries: archive.len(),
         inflated_bytes: total,
@@ -816,10 +852,161 @@
                 return Err(invalid(format!("{depth} element(s) left unclosed")));
             }
             Ok(Event::Eof) => return Ok(()),
+            // The same refusals `validate_xml_with_limits` gives: for the
+            // streaming reader an opaque construct is one lexical unit
+            // (quick-xml consumes the complete DTD, quotes included), so
+            // ignoring it let markup hidden behind a quoted '>' reach the
+            // permissive DOM parser unaudited while the DOM parser read it
+            // as real, deeply nested elements.
+            Ok(Event::DocType(_)) => {
+                return Err(invalid("DTD declarations are unsupported".to_string()));
+            }
+            Ok(Event::CData(_)) if depth == 0 => {
+                return Err(invalid("CDATA outside XML root".to_string()));
+            }
+            Ok(Event::Text(text))
+                if depth == 0 && text.bytes().any(|b| !b.is_ascii_whitespace()) =>
+            {
+                return Err(invalid("text outside XML root".to_string()));
+            }
             Ok(_) => {}
             Err(e) => return Err(invalid(e.to_string())),
         }
     }
+}
+
+/// Relationship kinds whose targets this engine reads as XML parts
+/// whatever their file extension: the render, revision and comparison
+/// loaders select theme, styles, numbering, settings, fonts, comments,
+/// notes and chart parts by relationship type (`convert::main_rel_part`,
+/// `convert::part_xml_by_rel_kind`, `convert::story_parts_xml`,
+/// `document_comparer::notes_part_names`, and the comparer's chart
+/// preprocessing at `document_comparer.rs:4861-4878`), so the XML budgets
+/// must cover their targets even under non-`.xml` names.
+const XML_REL_KINDS: [&str; 16] = [
+    "styles",
+    "settings",
+    "theme",
+    "numbering",
+    "fontTable",
+    "comments",
+    "commentsExtended",
+    "commentsIds",
+    "commentsExtensible",
+    "footnotes",
+    "endnotes",
+    "header",
+    "footer",
+    "chart",
+    "diagramData",
+    "diagramDrawing",
+];
+
+/// Whether a relationship `Type` names an XML part kind ([`XML_REL_KINDS`]).
+fn rel_kind_is_xml(rel_type: &str) -> bool {
+    rel_type
+        .rsplit('/')
+        .next()
+        .is_some_and(|last| XML_REL_KINDS.contains(&last))
+}
+
+/// Whether a content type names an XML media type: the `+xml` family
+/// (`…wordprocessingml.styles+xml`), `application/xml`, `text/xml` and
+/// the `…/xml` forms. A bare substring test does not do —
+/// `application/vnd.openxmlformats-…sheet` (an embedded XLSX, a ZIP)
+/// contains "xml" without being XML.
+fn content_type_is_xml(content_type: &str) -> bool {
+    let ct = content_type.to_ascii_lowercase();
+    ct.ends_with("+xml") || ct == "application/xml" || ct == "text/xml" || ct.ends_with("/xml")
+}
+
+/// Whether a part's first bytes could begin an XML document: `<` in
+/// UTF-8, or in either UTF-16 endianness, BOM included. Binary parts —
+/// media, embedded fonts, OLE objects — never begin with one.
+fn looks_like_xml(head: &[u8]) -> bool {
+    let head = match head {
+        // A UTF-8 BOM precedes the markup itself.
+        [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
+        _ => head,
+    };
+    matches!(
+        head,
+        // '<' as UTF-8 (or an unmarked UTF-16LE), or as UTF-16BE; a
+        // UTF-16 BOM sits ahead of the same bytes.
+        [b'<', ..] | [0x00, b'<', ..] | [0xFF, 0xFE, b'<', 0x00, ..] | [0xFE, 0xFF, 0x00, b'<', ..]
+    )
+}
+
+/// The first bytes of a related target, for the [`looks_like_xml`] sniff.
+fn part_head<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>, target: &str) -> Vec<u8> {
+    let mut head = Vec::new();
+    if let Ok(mut entry) = archive.by_name(target) {
+        let _ = (&mut entry).take(64).read_to_end(&mut head);
+    }
+    head
+}
+
+/// XML admission for parts the extension scan never saw. Relationships
+/// select parts for XML consumers whatever the parts are called, so a
+/// related target is depth-checked when an XML relationship kind names it
+/// or its content type is XML — and, when it carries no content type at
+/// all, when its bytes look like XML. `altChunk` targets are HTML the
+/// engine reads with an HTML parser, not this DOM, and stay on their own
+/// path; media and other binary parts never look like XML.
+fn admit_related_xml<R: Read + std::io::Seek>(
+    archive: &mut ZipArchive<R>,
+    related: &BTreeMap<String, Vec<String>>,
+    types: Option<&Vec<u8>>,
+    main_part: &str,
+    limits: InputLimits,
+) -> Result<(), AdmissionError> {
+    let types: &[u8] = types.map_or(&[], Vec::as_slice);
+    for (target, kinds) in related {
+        if target == main_part {
+            continue; // the renamed-main pass above checks it whatever its name
+        }
+        let lower = target.to_ascii_lowercase();
+        if lower.ends_with(".xml") || lower.ends_with(".rels") {
+            continue; // the extension scan already checked it
+        }
+        // An altChunk target is HTML the engine reads with an HTML
+        // parser, not this DOM; it keeps its own path, sloppy or not.
+        if kinds
+            .iter()
+            .any(|kind| kind.rsplit('/').next() == Some("aFChunk"))
+        {
+            continue;
+        }
+        let named_by_xml_kind = kinds.iter().any(|kind| rel_kind_is_xml(kind));
+        let check = named_by_xml_kind
+            || match content_type_of(types, target) {
+                // A declared content type settles it: XML types are
+                // checked, everything else (media, HTML, embedded
+                // packages) is not.
+                Some(ct) => content_type_is_xml(&ct),
+                // Untyped: admit it as XML only when its bytes look like
+                // XML.
+                None => looks_like_xml(&part_head(archive, target)),
+            };
+        if !check {
+            continue;
+        }
+        let Ok(mut entry) = archive.by_name(target) else {
+            continue; // a dangling target is the consumer's absent part
+        };
+        let mut buf = Vec::new();
+        let cap = limits.max_part_bytes;
+        let read = (&mut entry)
+            .take(cap.saturating_add(1))
+            .read_to_end(&mut buf)
+            .map_err(|e| {
+                AdmissionError::new(AdmissionErrorKind::InvalidPackage, format!("{target}: {e}"))
+            })?;
+        if u64::try_from(read).unwrap_or(u64::MAX) <= cap {
+            check_xml_depth(target, &buf, limits)?;
+        }
+    }
+    Ok(())
 }
 
 /// The main document part from the package relationships (falling back to
@@ -1870,6 +2057,135 @@
         );
     }
 
+    /// The XML budgets follow relationships, not file extensions: a part an
+    /// XML relationship kind names is depth-checked whatever it is called,
+    /// and a shallow renamed part still admits (other producers may name
+    /// these parts freely).
+    #[test]
+    fn related_parts_are_depth_checked_whatever_their_extension() {
+        let doc_rels = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.bin"/></Relationships>"#;
+        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
+        let bytes = docx_with(&[
+            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+            ("word/styles.bin", deep.as_bytes()),
+        ]);
+        assert_eq!(
+            kind(&bytes, InputLimits::default()),
+            AdmissionErrorKind::InvalidXml
+        );
+        let shallow = docx_with(&[
+            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+            ("word/styles.bin", DOC.as_bytes()),
+        ]);
+        assert!(admit(&shallow, InputLimits::default()).is_ok());
+    }
+
+    /// A relationship kind that names an XML part governs over a content
+    /// type that claims binary: the engine's consumers parse what the
+    /// relationship selects.
+    #[test]
+    fn an_xml_relationship_kind_checks_even_a_binary_typed_target() {
+        let types = TYPES.replace(
+            "</Types>",
+            "<Default Extension=\"bin\" ContentType=\"application/octet-stream\"/></Types>",
+        );
+        let doc_rels = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="notes.bin"/></Relationships>"#;
+        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
+        let entries: Vec<(&str, &[u8])> = vec![
+            ("[Content_Types].xml", types.as_bytes()),
+            ("_rels/.rels", RELS.as_bytes()),
+            ("word/document.xml", DOC.as_bytes()),
+            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+            ("word/notes.bin", deep.as_bytes()),
+        ];
+        assert_eq!(
+            kind(
+                &zip_of(&entries, CompressionMethod::Deflated),
+                InputLimits::default()
+            ),
+            AdmissionErrorKind::InvalidXml
+        );
+    }
+
+    /// A related target no XML kind names and no content type types is
+    /// admitted as XML when its bytes look like XML — the untyped case a
+    /// producer that skips `[Content_Types].xml` coverage leaves.
+    #[test]
+    fn an_untyped_related_target_that_looks_like_xml_is_checked() {
+        let doc_rels = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail" Target="thumb.dat"/></Relationships>"#;
+        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
+        let deep_pkg = docx_with(&[
+            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+            ("word/thumb.dat", deep.as_bytes()),
+        ]);
+        assert_eq!(
+            kind(&deep_pkg, InputLimits::default()),
+            AdmissionErrorKind::InvalidXml
+        );
+        let shallow = docx_with(&[
+            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+            ("word/thumb.dat", b"<t/>"),
+        ]);
+        assert!(admit(&shallow, InputLimits::default()).is_ok());
+    }
+
+    /// Media a relationship reaches is not XML and is not refused; an
+    /// `altChunk` target is HTML for the HTML parser, not this DOM, and
+    /// keeps its own (unopened) path however sloppy its markup — declared
+    /// (`text/html`) or not.
+    #[test]
+    fn media_and_altchunk_relationship_targets_are_not_xml_checked() {
+        let doc_rels = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/pic.png"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/aFChunk" Target="frag.htm"/></Relationships>"#;
+        let types = TYPES.replace(
+            "</Types>",
+            "<Default Extension=\"png\" ContentType=\"image/png\"/></Types>",
+        );
+        let entries: Vec<(&str, &[u8])> = vec![
+            ("[Content_Types].xml", types.as_bytes()),
+            ("_rels/.rels", RELS.as_bytes()),
+            ("word/document.xml", DOC.as_bytes()),
+            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+            ("word/media/pic.png", b"\x89PNG not really"),
+            // Untyped on purpose: only the aFChunk kind keeps it open.
+            ("word/frag.htm", b"<html><body><p>x<br>y</body>"),
+        ];
+        assert!(
+            admit(
+                &zip_of(&entries, CompressionMethod::Deflated),
+                InputLimits::default()
+            )
+            .is_ok()
+        );
+    }
+
+    /// Opaque constructs are one lexical unit for admission too: a DTD
+    /// (with quoted markup or an internal-subset comment hiding nesting),
+    /// a top-level CDATA section and top-level text are refused exactly as
+    /// the checked XML gate refuses them, instead of streaming past
+    /// unaudited while the permissive DOM parser finds the hidden markup.
+    #[test]
+    fn opaque_dtd_cdata_and_top_level_text_are_refused_like_checked_xml() {
+        let dtd_quoted = concat!("<!DOCTYPE d [<!ENTITY e \"x>", "<a><a/></a>", "\">]>");
+        let dtd_comment = concat!("<!DOCTYPE d [<!-- c>", "<a><a/></a>", " -->]>");
+        for (name, xml) in [
+            ("word/dtd.xml", dtd_quoted),
+            ("word/dtdcomment.xml", dtd_comment),
+            ("word/cdata.xml", "<![CDATA[ > <a/> ]]><w:document/>"),
+            ("word/text.xml", "text<w:document/>"),
+        ] {
+            let bytes = docx_with(&[(name, xml.as_bytes())]);
+            assert_eq!(
+                kind(&bytes, InputLimits::default()),
+                AdmissionErrorKind::InvalidXml,
+                "{xml}"
+            );
+            assert!(crate::xmllinq::parse::validate_xml(xml).is_err(), "{xml}");
+        }
+        // In-root CDATA and ordinary documents keep admitting.
+        let fine = docx_with(&[("word/fine.xml", b"<a><![CDATA[raw > <b/>]]></a>")]);
+        assert!(admit(&fine, InputLimits::default()).is_ok());
+    }
+
     /// Replace the trailing end-of-central-directory record of `zip` by a
     /// ZIP64 locator pointing at `record_offset`, an EOCD declaring 0xFFFF
     /// entries, and `comment` as the archive comment.
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -397,7 +397,7 @@
 /// The part a `.rels` path belongs to (`word/_rels/header1.xml.rels` →
 /// `word/header1.xml`; `_rels/.rels` → the package, as `""`), or None when
 /// `name` is not a relationships path. Inverse of `part_name_to_rels_path`.
-fn rels_path_to_part_name(name: &str) -> Option<String> {
+pub(crate) fn rels_path_to_part_name(name: &str) -> Option<String> {
     let name = name.strip_prefix('/').unwrap_or(name);
     let file = name.strip_suffix(".rels")?;
     let (dir, base) = file.rsplit_once('/')?;
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -27,7 +27,14 @@
 //!   gate first: [`validate_xml_with_limits`], or package admission with
 //!   the caller's active limits. A caller holding unchecked XML uses
 //!   [`try_parse_xdocument`], which runs [`validate_xml`] before it
-//!   builds any node.
+//!   builds any node. Two structural rules it does keep on its own: a
+//!   top-level `<!` construct is skipped whole (quotes and internal
+//!   subset included — the boundary quick-xml's `DocType` event uses), so
+//!   markup hidden behind a quoted `>` cannot parse as elements, and DOM
+//!   construction stops at [`MAX_DOM_NESTING`] open elements — a
+//!   stack-safety backstop far above every shipped depth budget, so a
+//!   gated input never reaches it and a caller that skipped every gate
+//!   gets a truncated tree, not a blown stack.
 //! - [`unescape_xml_text`] — decode XML entities / character references
 //!   in attribute / text bodies.
 //! - [`validate_xml`] — structural admission for a part *before* the
@@ -63,6 +70,12 @@
 pub(crate) const MAX_LIVE_NS_BINDINGS: usize = 256;
 /// Total prefix+URI bytes declared by xmlns attributes in one document.
 pub(crate) const MAX_NS_DECLARED_BYTES: usize = 1024 * 1024;
+/// DOM-construction nesting backstop: parsing stops descending past this
+/// many open elements. Not a policy — the checked gates refuse far
+/// shallower input (admission's default `max_xml_depth` is 256) — but a
+/// caller feeding raw bytes straight to [`parse_xdocument`] gets a
+/// truncated tree instead of a blown stack.
+pub(crate) const MAX_DOM_NESTING: usize = 4096;
 
 // N02 mechanism counter: bytes of (prefix, uri) pairs the parser's
 // namespace scope has recorded. The scoped-overlay scope only ever records
@@ -104,6 +117,7 @@
         dom,
         c: chars,
         pos: 0,
+        depth: 0,
     };
     p.parse_document()
 }
@@ -139,6 +153,9 @@
     dom: &'a mut Dom,
     c: Vec<char>,
     pos: usize,
+    /// Open-element nesting while `parse_element` recurses; see
+    /// [`MAX_DOM_NESTING`].
+    depth: usize,
 }
 
 /// In-scope namespace bindings as an overlay stack (N02): one `HashMap`
@@ -536,10 +553,22 @@
                 self.pos = end.map(|e| e + 3).unwrap_or(len);
                 continue;
             }
+            if self.starts_with("<![CDATA[") {
+                // Top-level CDATA is opaque at the document level (the
+                // checked gate refuses it); skip the complete section so a
+                // `>` inside its body cannot expose the rest as markup.
+                // In-element CDATA is text (parse_element).
+                let end = self.index_of("]]>", self.pos);
+                self.pos = end.map_or(len, |e| e + 3);
+                continue;
+            }
             if self.starts_with("<!") {
-                // DOCTYPE — skip
-                let end = self.index_of(">", self.pos);
-                self.pos = end.map(|e| e + 1).unwrap_or(len);
+                // DOCTYPE — skip the complete declaration, quotes and
+                // bracketed internal subset included: the same lexical
+                // boundary quick-xml's DocType event uses. Stopping at the
+                // first '>' instead let markup hidden behind a quoted one
+                // parse as real elements.
+                self.skip_declaration();
                 continue;
             }
             if self.cur() == '<' {
@@ -658,7 +687,16 @@
                 continue;
             }
             if self.cur() == '<' {
+                if self.depth >= MAX_DOM_NESTING {
+                    // Stack-safety backstop, not a policy: inputs this deep
+                    // were refused by a gate long ago; a caller that skipped
+                    // every gate gets the rest of the document dropped.
+                    self.pos = self.len();
+                    break;
+                }
+                self.depth += 1;
                 let child = self.parse_element(scope);
+                self.depth -= 1;
                 self.dom.add(el, child);
                 continue;
             }
@@ -696,6 +734,40 @@
         let data = self.slice(self.pos, end.unwrap_or(len));
         self.pos = end.map(|e| e + 2).unwrap_or(len);
         self.dom.new_pi(&target, &data)
+    }
+
+    /// Skip one top-level `<!` construct (a DOCTYPE declaration)
+    /// completely: quoted strings and the bracketed internal subset are
+    /// opaque, and the declaration ends at the `>` that closes it — never
+    /// at a `>` a quote or the subset wraps. An unterminated declaration
+    /// skips to the end of the input.
+    fn skip_declaration(&mut self) {
+        let len = self.len();
+        let mut quote = '\0';
+        let mut subset_open = false;
+        let mut subset_done = false;
+        let mut i = self.pos + 2; // past "<!"
+        while i < len {
+            let ch = self.c[i];
+            if quote != '\0' {
+                if ch == quote {
+                    quote = '\0';
+                }
+            } else {
+                match ch {
+                    '\'' | '"' => quote = ch,
+                    '[' => subset_open = true,
+                    ']' => subset_done = true,
+                    '>' if !subset_open || subset_done => {
+                        self.pos = i + 1;
+                        return;
+                    }
+                    _ => {}
+                }
+            }
+            i += 1;
+        }
+        self.pos = len;
     }
 }
 
@@ -1179,6 +1251,58 @@
         assert_eq!(dom.value(root), "owned");
         assert!(validate_xml("<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>").is_ok());
     }
+
+    /// A top-level `<!` construct is one lexical unit: the permissive
+    /// parser skips the complete declaration (quotes and internal subset
+    /// included), so markup hidden behind a quoted `>` — or behind a `>`
+    /// inside an internal-subset comment — cannot re-enter the parse as
+    /// real elements. quick-xml's `DocType` event draws the same boundary,
+    /// which is what makes admission and this parser agree.
+    #[test]
+    fn top_level_declarations_hide_their_markup_completely() {
+        let dtd_quoted = concat!("<!DOCTYPE d [<!ENTITY e \"x>", "<a><b/></a>", "\">]><r/>");
+        let dtd_comment = concat!("<!DOCTYPE d [<!-- c>", "<a><b/></a>", " -->]><r/>");
+        for xml in [dtd_quoted, dtd_comment] {
+            let mut dom = Dom::new();
+            let document = parse_xdocument(&mut dom, xml);
+            // The document level holds exactly one element, the real root;
+            // the hidden `<a>` never became a sibling.
+            assert_eq!(dom.nodes(document).len(), 1, "{xml}");
+            let root = dom.root(document).expect("the real root survives");
+            assert_eq!(dom.name(root).unwrap().local_name(), "r", "{xml}");
+            assert!(dom.elements(root, None).is_empty(), "{xml}");
+            assert!(validate_xml(xml).is_err(), "{xml}");
+        }
+    }
+
+    /// Top-level CDATA is skipped whole (`]]>` included), so a `>` inside
+    /// its body cannot expose the rest of the body as markup. In-root
+    /// CDATA stays text, as before.
+    #[test]
+    fn top_level_cdata_is_skipped_whole_and_in_root_cdata_stays_text() {
+        let xml = "<![CDATA[ > <a/> ]]><r><![CDATA[x]]></r>";
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, xml);
+        let root = dom.root(document).expect("the real root survives");
+        assert_eq!(dom.name(root).unwrap().local_name(), "r");
+        assert!(dom.elements(root, None).is_empty());
+        assert_eq!(dom.value(root), "x");
+        assert!(validate_xml(xml).is_err());
+    }
+
+    /// The DOM-construction backstop: nesting past [`MAX_DOM_NESTING`]
+    /// truncates the tree instead of recursing to stack exhaustion. Every
+    /// gate refuses far shallower input; this is the last line for callers
+    /// that skipped them all.
+    #[test]
+    fn unadmitted_runaway_nesting_truncates_instead_of_exhausting_the_stack() {
+        let xml = format!("<a>{}{}", "<b>".repeat(100_000), "</b>".repeat(100_000));
+        assert!(validate_xml(&xml).is_err());
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let root = dom.root(document).expect("truncated, not crashed");
+        assert_eq!(dom.name(root).unwrap().local_name(), "a");
+    }
 }
 
 #[cfg(test)]
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
--- a/src/markdown/from_docx/drawing.rs
+++ b/src/markdown/from_docx/drawing.rs
@@ -1405,7 +1405,36 @@
             base64::engine::general_purpose::URL_SAFE.decode(encoded.trim_end_matches('='))
         })
         .map_err(|e| format!("drawing payload is not base64: {e}"))?;
-    serde_json::from_slice(&bytes).map_err(|e| format!("drawing payload is not its JSON: {e}"))
+    let drawing: Drawing = serde_json::from_slice(&bytes)
+        .map_err(|e| format!("drawing payload is not its JSON: {e}"))?;
+    check_import_budgets(&drawing)?;
+    Ok(drawing)
+}
+
+/// The budgets a captured payload already obeys ([`MAX_DEPS`],
+/// [`MAX_DEP_RELS`]), imposed again at import: a `drawing:` destination in
+/// untrusted Markdown cannot grow a dependency graph beyond what capture
+/// ever writes, which is also what bounds `materialize`'s recursion depth.
+fn check_import_budgets(drawing: &Drawing) -> Result<(), String> {
+    if drawing.deps.len() > MAX_DEPS {
+        return Err(format!(
+            "drawing payload holds more than {MAX_DEPS} dependent parts"
+        ));
+    }
+    if drawing.rels.len() > MAX_DEP_RELS {
+        return Err(format!(
+            "drawing payload holds more than {MAX_DEP_RELS} relationships"
+        ));
+    }
+    for dep in &drawing.deps {
+        if dep.rels.len() > MAX_DEP_RELS {
+            return Err(format!(
+                "{} holds more than {MAX_DEP_RELS} relationships",
+                dep.name
+            ));
+        }
+    }
+    Ok(())
 }
 
 pub(crate) fn encode(drawing: &Drawing) -> String {
@@ -1887,11 +1916,24 @@
     next_doc_pr: &mut u32,
 ) -> Option<String> {
     let mut xml = drawing.xml.clone();
+    // The dependency graph an imported payload carries is attacker data:
+    // materialize walks it with the active path on the stack, so a cycle
+    // (a part depending on itself, directly or through another) refuses
+    // the payload instead of recursing forever.
+    let mut active: Vec<usize> = Vec::new();
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+                let name = dep_part(
+                    drawing,
+                    index,
+                    package,
+                    existing,
+                    used,
+                    next_doc_pr,
+                    &mut active,
+                )?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1952,7 +1994,9 @@
 }
 
 /// Write dependent part `index` and its own dependency closure; its new
-/// part name.
+/// part name. `active` holds the `deps` indices on the current recursion
+/// path: a revisit is a dependency cycle and refuses the payload (`None`,
+/// the same refusal a relationship that names no part gets).
 fn dep_part(
     drawing: &Drawing,
     index: usize,
@@ -1960,6 +2004,27 @@
     existing: &HashSet<String>,
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
+    active: &mut Vec<usize>,
+) -> Option<String> {
+    if active.contains(&index) {
+        return None; // a dependency cycle
+    }
+    active.push(index);
+    let written = dep_part_closure(drawing, index, package, existing, used, next_doc_pr, active);
+    active.pop();
+    written
+}
+
+/// `dep_part`'s body below the cycle guard: write `index`'s part and every
+/// part its relationships reach.
+fn dep_part_closure(
+    drawing: &Drawing,
+    index: usize,
+    package: &mut PartFs,
+    existing: &HashSet<String>,
+    used: &mut HashSet<String>,
+    next_doc_pr: &mut u32,
+    active: &mut Vec<usize>,
 ) -> Option<String> {
     let dep = drawing.deps.get(index)?;
     let name = name_for(&dep.name, existing, used);
@@ -1971,7 +2036,8 @@
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name =
+                    dep_part(drawing, inner, package, existing, used, next_doc_pr, active)?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -2148,4 +2214,187 @@
         );
         assert_eq!(next, 42);
     }
-}
+
+    // ── imported dependency graphs are attacker data ─────────────────────
+    // `drawing:` payloads decode from untrusted Markdown: their graphs get
+    // the capture-side budgets re-imposed at import, and a cycle refuses
+    // the payload instead of recursing until the stack ends.
+
+    const REL_IMAGE: &str =
+        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
+
+    fn fixture_package() -> PartFs {
+        PartFs::open(include_bytes!(
+            "../../../tests/fixtures/redline/original.docx"
+        ))
+        .unwrap()
+    }
+
+    fn one_png_named(name: &str) -> Dep {
+        Dep {
+            name: name.to_string(),
+            kind: "image/png".to_string(),
+            b64: base64_encode(b"PNGDATA"),
+            ..Dep::default()
+        }
+    }
+
+    fn rel_to(part: Option<usize>) -> PayRel {
+        PayRel {
+            sym: "d0.1".to_string(),
+            kind: REL_IMAGE.to_string(),
+            part,
+            ..PayRel::default()
+        }
+    }
+
+    #[test]
+    fn cyclic_dependency_payloads_refuse_instead_of_recursing() {
+        // A one-node cycle: the part's own relationship depends on itself.
+        let mut self_cycle = one_png_named("media/a.png");
+        self_cycle.rels = vec![rel_to(Some(0))];
+        let one_node = Drawing {
+            xml: "<w:drawing/>".to_string(),
+            rels: vec![PayRel {
+                sym: "r1".to_string(),
+                kind: REL_IMAGE.to_string(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![self_cycle],
+            ..Drawing::default()
+        };
+        // A two-node cycle through a second part.
+        let mut a = one_png_named("media/a.png");
+        a.rels = vec![rel_to(Some(1))];
+        let mut b = one_png_named("media/b.png");
+        b.rels = vec![rel_to(Some(0))];
+        let two_node = Drawing {
+            xml: "<w:drawing/>".to_string(),
+            rels: vec![PayRel {
+                sym: "r1".to_string(),
+                kind: REL_IMAGE.to_string(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![a, b],
+            ..Drawing::default()
+        };
+        for drawing in [&one_node, &two_node] {
+            let mut package = fixture_package();
+            let existing: HashSet<String> = package.parts().into_iter().collect();
+            let mut used: HashSet<String> = HashSet::new();
+            let mut next = 1;
+            assert!(
+                materialize(
+                    drawing,
+                    &mut package,
+                    "word/document.xml",
+                    &existing,
+                    &mut used,
+                    &mut next
+                )
+                .is_none(),
+                "a cyclic dependency graph refuses the payload"
+            );
+        }
+    }
+
+    /// The refusal is the graph's, not the shape's: an acyclic chain still
+    /// materializes and every part is written.
+    #[test]
+    fn acyclic_chains_and_shared_dependencies_still_materialize() {
+        let mut chain: Vec<Dep> = Vec::new();
+        for i in 0..8usize {
+            chain.push(Dep {
+                name: format!("media/part{i}.xml"),
+                kind: "application/xml".to_string(),
+                xml: format!("<p>{i}</p>"),
+                rels: (i + 1 < 8)
+                    .then(|| vec![rel_to(Some(i + 1))])
+                    .unwrap_or_default(),
+                ..Dep::default()
+            });
+        }
+        let drawing = Drawing {
+            xml: "<w:drawing/>".to_string(),
+            rels: vec![PayRel {
+                sym: "r1".to_string(),
+                kind: REL_IMAGE.to_string(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: chain,
+            ..Drawing::default()
+        };
+        let mut package = fixture_package();
+        let existing: HashSet<String> = package.parts().into_iter().collect();
+        let mut used: HashSet<String> = HashSet::new();
+        let mut next = 1;
+        let xml = materialize(
+            &drawing,
+            &mut package,
+            "word/document.xml",
+            &existing,
+            &mut used,
+            &mut next,
+        )
+        .expect("an acyclic chain materializes");
+        assert_eq!(xml, "<w:drawing/>");
+        for i in 0..8 {
+            assert!(
+                package.part_bytes(&format!("media/part{i}.xml")).is_some(),
+                "part {i} written"
+            );
+        }
+    }
+
+    /// Import re-imposes the capture-side budgets: a payload with more
+    /// dependencies or relationships than capture ever writes is refused
+    /// at decode, before any materialization runs.
+    #[test]
+    fn decode_refuses_payloads_past_the_dependency_budgets() {
+        let over_deps = Drawing {
+            deps: (0..=MAX_DEPS)
+                .map(|_| one_png_named("media/a.png"))
+                .collect(),
+            ..Drawing::default()
+        };
+        let mut over_dep_rels = one_png_named("media/a.png");
+        over_dep_rels.rels = (0..=MAX_DEP_RELS)
+            .map(|i| PayRel {
+                sym: format!("d0.{i}"),
+                kind: REL_IMAGE.to_string(),
+                raw: Some("gone.png".to_string()),
+                ..PayRel::default()
+            })
+            .collect();
+        let over_dep_rels_drawing = Drawing {
+            deps: vec![over_dep_rels],
+            ..Drawing::default()
+        };
+        let over_top_rels = Drawing {
+            rels: (0..=MAX_DEP_RELS)
+                .map(|i| PayRel {
+                    sym: format!("r{i}"),
+                    kind: REL_IMAGE.to_string(),
+                    raw: Some("gone.png".to_string()),
+                    ..PayRel::default()
+                })
+                .collect(),
+            ..Drawing::default()
+        };
+        for drawing in [over_deps, over_dep_rels_drawing, over_top_rels] {
+            let dest = format!("drawing:{}", encode(&drawing));
+            assert!(decode(&dest).is_err(), "a budgeted payload is refused");
+        }
+        let within = Drawing {
+            deps: (0..MAX_DEPS)
+                .map(|_| one_png_named("media/a.png"))
+                .collect(),
+            ..Drawing::default()
+        };
+        let dest = format!("drawing:{}", encode(&within));
+        assert!(decode(&dest).is_ok(), "exactly at the budget decodes");
+    }
+}
diff --git a/tests/input_admission.rs b/tests/input_admission.rs
--- a/tests/input_admission.rs
+++ b/tests/input_admission.rs
@@ -1390,3 +1390,94 @@
         assert!(result.unwrap_err().to_string().contains("INVALID_XML"));
     }
 }
+
+// ── relationship-selected XML parts and opaque XML constructs ────────────
+// A package whose auxiliary XML hides behind a non-XML part name (or
+// behind a DTD the extension scan streamed past) must come back as a typed
+// INVALID_XML refusal from every public entry point, never as a parser
+// stack overflow the Python wheel and WASM build cannot catch.
+
+const OFFICE_REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
+
+/// The base package plus a document-level footnotes relationship to
+/// `word/notes.bin` holding `notes_xml`.
+fn package_with_renamed_notes(notes_xml: &str) -> Vec<u8> {
+    let doc_rels = format!(
+        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
+<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
+<Relationship Id=\"rId1\" Type=\"{OFFICE_REL_NS}/footnotes\" Target=\"notes.bin\"/>\
+</Relationships>"
+    );
+    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
+    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
+    for (name, body) in [
+        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
+        ("_rels/.rels", PACKAGE_RELS.as_bytes()),
+        ("word/document.xml", DOCUMENT.as_bytes()),
+        ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
+        ("word/notes.bin", notes_xml.as_bytes()),
+    ] {
+        writer.start_file(name, options).unwrap();
+        writer.write_all(body).unwrap();
+    }
+    writer.finish().unwrap().into_inner()
+}
+
+#[test]
+fn get_revisions_refuses_deep_xml_under_a_renamed_notes_part() {
+    let deep = format!("{}{}", "<a>".repeat(100_000), "</a>".repeat(100_000));
+    let bytes = package_with_renamed_notes(&deep);
+    let err = jubarte::document_comparer::get_revisions(&bytes, &WmlComparerSettings::default())
+        .expect_err("typed refusal, not a stack overflow");
+    let text = format!("{err}");
+    assert!(text.contains("INVALID_XML"), "{text}");
+    assert!(text.contains("notes.bin"), "{text}");
+}
+
+#[test]
+fn compare_refuses_deep_xml_under_a_renamed_notes_part() {
+    let deep = format!("{}{}", "<a>".repeat(100_000), "</a>".repeat(100_000));
+    let bytes = package_with_renamed_notes(&deep);
+    let err = compare_documents(&bytes, ORIGINAL, "Author")
+        .expect_err("typed refusal, not a stack overflow");
+    assert!(format!("{err}").contains("INVALID_XML"), "{err}");
+}
+
+#[test]
+fn wml_document_refuses_deep_xml_under_a_renamed_notes_part() {
+    let deep = format!("{}{}", "<a>".repeat(100_000), "</a>".repeat(100_000));
+    let bytes = package_with_renamed_notes(&deep);
+    let err = WmlDocument::from_bytes(&bytes).expect_err("typed refusal");
+    assert!(format!("{err}").contains("INVALID_XML"), "{err}");
+}
+
+/// A quoted `>` inside a DTD internal subset hides deeply nested markup
+/// from the streaming depth scan while the permissive DOM parser reads it
+/// as real elements: admission must refuse the construct, exactly as the
+/// checked XML gate already does.
+#[test]
+fn get_revisions_refuses_dtd_hidden_nesting() {
+    let document = format!(
+        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE d [<!ENTITY e \"x>{}\">]>\
+<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
+<w:body><w:p/></w:body></w:document>",
+        "<a>".repeat(100_000)
+    );
+    let bytes = package_with_document(&document);
+    let err = jubarte::document_comparer::get_revisions(&bytes, &WmlComparerSettings::default())
+        .expect_err("typed refusal, not a stack overflow");
+    assert!(format!("{err}").contains("INVALID_XML"), "{err}");
+}
+
+/// The green companions: a shallow renamed notes part (other producers
+/// name parts freely) and in-root CDATA keep working end to end.
+#[test]
+fn a_renamed_shallow_notes_part_and_in_root_cdata_keep_admitting() {
+    let notes = "<w:footnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
+<w:footnote w:id=\"1\"><w:p/></w:footnote></w:footnotes>";
+    let bytes = package_with_renamed_notes(notes);
+    assert!(WmlDocument::from_bytes(&bytes).is_ok());
+    let cdata = "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
+<w:body><w:p><w:r><w:t><![CDATA[x]]></w:t></w:r></w:p></w:body></w:document>";
+    assert!(WmlDocument::from_bytes(&package_with_document(cdata)).is_ok());
+}
diff --git a/tests/markdown_to_docx.rs b/tests/markdown_to_docx.rs
--- a/tests/markdown_to_docx.rs
+++ b/tests/markdown_to_docx.rs
@@ -2347,3 +2347,16 @@
         Some(item)
     );
 }
+
+/// A `drawing:` payload whose dependency graph cycles is refused at
+/// materialization, not recursed into stack exhaustion: the payload is
+/// skipped and the document still converts (and stays a package the OOXML
+/// SDK reads).
+#[test]
+fn a_cyclic_drawing_payload_converts_without_recursing() {
+    use base64::Engine;
+    let json = r#"{"xml":"<w:drawing/>","rels":[{"sym":"r1","t":"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image","part":0}],"deps":[{"name":"media/a.png","t":"image/png","b64":"iVBORw0K","rels":[{"sym":"d0.1","t":"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image","part":0}]}]}"#;
+    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json);
+    let docx = write(&format!("![box](drawing:{payload})"));
+    assert!(docx.starts_with(b"PK"));
+}

```
