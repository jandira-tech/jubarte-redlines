@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_3201e6caaef21850d61fa69c | WMF record-boundary failures amplify polygon work and retained text | medium
csf_eaffa9e4af9c9c07d800ef1b | ZIP footer and count interpretation mismatch bypasses entry-count admission | medium
csf_d22452ac5ec901578d340334 | Opaque DTD and top-level CDATA bypass package XML depth checks | medium
csf_6a40bc41bdf34a737ceac40d | Entity decoding repeatedly scans the input suffix | medium
csf_631ab5c4311b813e6ceb189f | Pull-request code runs on shared self-hosted infrastructure | medium
csf_27ca2e0d965d8b1341a32909 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_3ec09f0cecec11243e8a8f5e | Entity decoding repeatedly scans the input suffix | medium
csf_7084a83c24deb556942e2d43 | WMF record-boundary failures amplify polygon work and retained text | medium
csf_b0aa536248aea0f44dd30d47 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_f756bbe5509c42bdcaa64921 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_88d031991ca3e20e4aae2f24 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_80edacafa6b67d4864116a38 | WMF record-boundary failures amplify polygon work and retained text | medium
csf_eb94605aebd8f08907c46e8c | Entity decoding repeatedly scans the input suffix | medium
csf_4b734e13a64fed0f0a4aaf25 | Pull-request code runs on shared self-hosted infrastructure | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -63,6 +63,14 @@
 pub(crate) const MAX_LIVE_NS_BINDINGS: usize = 256;
 /// Total prefix+URI bytes declared by xmlns attributes in one document.
 pub(crate) const MAX_NS_DECLARED_BYTES: usize = 1024 * 1024;
+/// Hard ceiling on element nesting during DOM construction. Package
+/// admission bounds depth at 256 by default (`InputLimits::max_xml_depth`),
+/// but parts reach the permissive parser under other names (a renamed
+/// auxiliary part behind a relationship) or through limits a caller raised;
+/// recursion past this ceiling is consumed without building nodes, so no
+/// input can exhaust the stack through DOM depth. Real parts nest far
+/// shallower than this even at the admission boundary.
+pub(crate) const MAX_DOM_DEPTH: usize = 1024;
 
 // N02 mechanism counter: bytes of (prefix, uri) pairs the parser's
 // namespace scope has recorded. The scoped-overlay scope only ever records
@@ -72,6 +80,15 @@
 #[cfg(test)]
 thread_local! {
     pub(crate) static NS_SCOPE_DECLARED_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
+}
+
+// dedup-0008 mechanism counter: characters the entity decoder scanned
+// looking for ';'. The TS port rescanned the whole remaining suffix for
+// every '&', which made a run of bare ampersands quadratic; the monotonic
+// cursor scans each character once. Per thread, as above.
+#[cfg(test)]
+thread_local! {
+    pub(crate) static ENTITY_SUFFIX_SCAN_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
 }
 
 /// Add to this thread's [`NS_SCOPE_DECLARED_BYTES`].
@@ -81,6 +98,13 @@
     NS_SCOPE_DECLARED_BYTES.with(|c| c.set(c.get() + by));
 }
 
+/// Add to this thread's [`ENTITY_SUFFIX_SCAN_BYTES`].
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+fn bump_entity_scan_bytes(by: usize) {
+    ENTITY_SUFFIX_SCAN_BYTES.with(|c| c.set(c.get() + by));
+}
+
 /// Parse an XML string into a Document node allocated into `dom`. The
 /// parser is permissive on real-world OOXML inputs (truncated closing
 /// tags, missing `?>` terminators, mixed-quote attribute values) — see
@@ -104,6 +128,7 @@
         dom,
         c: chars,
         pos: 0,
+        depth: 0,
     };
     p.parse_document()
 }
@@ -139,6 +164,8 @@
     dom: &'a mut Dom,
     c: Vec<char>,
     pos: usize,
+    /// Nesting depth of the element being parsed (`MAX_DOM_DEPTH` caps it).
+    depth: usize,
 }
 
 /// In-scope namespace bindings as an overlay stack (N02): one `HashMap`
@@ -543,7 +570,16 @@
                 continue;
             }
             if self.cur() == '<' {
+                if self.depth >= MAX_DOM_DEPTH {
+                    // Top-level elements sit at depth 1; a fragment with
+                    // unbounded sibling roots never recurses, so this can
+                    // only fire for markup nested past the cap.
+                    self.skip_element_subtree();
+                    continue;
+                }
+                self.depth += 1;
                 let el = self.parse_element(&mut scope);
+                self.depth -= 1;
                 self.dom.add(doc, el);
                 continue;
             }
@@ -658,7 +694,18 @@
                 continue;
             }
             if self.cur() == '<' {
+                if self.depth >= MAX_DOM_DEPTH {
+                    // The hard structural cap: whatever admitted these
+                    // bytes, DOM construction never recurses past this
+                    // depth. The over-depth subtree is consumed without
+                    // building nodes; siblings at this level keep their
+                    // places.
+                    self.skip_element_subtree();
+                    continue;
+                }
+                self.depth += 1;
                 let child = self.parse_element(scope);
+                self.depth -= 1;
                 self.dom.add(el, child);
                 continue;
             }
@@ -676,8 +723,68 @@
         }
         scope.pop(mark);
         el
+    }
+
+    /// Consume one element subtree the DOM-depth cap refused, without
+    /// building nodes: its start tag (a quoted `>` inside an attribute
+    /// value respected), then balanced scanning until the matching close
+    /// tag or the end of input. Bounded by the input length, like every
+    /// other scan in this parser, and tolerant of missing close tags the
+    /// same way the element loop is.
+    fn skip_element_subtree(&mut self) {
+        if !self.skip_start_tag() {
+            let mut nest = 1usize;
+            while nest > 0 && self.pos < self.len() {
+                if self.starts_with("</") {
+                    nest -= 1;
+                    let end = self.index_of(">", self.pos);
+                    self.pos = end.map_or(self.len(), |e| e + 1);
+                } else if self.starts_with("<!--") {
+                    let end = self.index_of("-->", self.pos);
+                    self.pos = end.map_or(self.len(), |e| e + 3);
+                } else if self.starts_with("<![CDATA[") {
+                    let end = self.index_of("]]>", self.pos);
+                    self.pos = end.map_or(self.len(), |e| e + 3);
+                } else if self.starts_with("<?") {
+                    let end = self.index_of("?>", self.pos);
+                    self.pos = end.map_or(self.len(), |e| e + 2);
+                } else if self.cur() == '<' {
+                    if !self.skip_start_tag() {
+                        nest += 1;
+                    }
+                } else {
+                    self.pos += 1;
+                }
+            }
+        }
     }
 
+    /// Consume a start or empty tag from `self.pos` (on `<`) up to and past
+    /// its `>`, respecting quotes, and report whether the tag self-closes
+    /// (`/>`): the character before an unquoted `>` is `/` exactly when the
+    /// tag carries no separate end tag.
+    fn skip_start_tag(&mut self) -> bool {
+        self.pos += 1; // '<'
+        let mut quote = '\0';
+        let mut last = '\0';
+        while self.pos < self.len() {
+            let ch = self.cur();
+            if quote == '\0' {
+                if ch == '"' || ch == '\'' {
+                    quote = ch;
+                } else if ch == '>' {
+                    self.pos += 1;
+                    return last == '/';
+                }
+            } else if ch == quote {
+                quote = '\0';
+            }
+            last = ch;
+            self.pos += 1;
+        }
+        false
+    }
+
     /// Parse a processing instruction (`<?target data?>`) into a `Pi` node.
     /// `self.pos` is on `<?`.
     fn parse_pi_node(&mut self) -> NodeId {
@@ -761,6 +868,11 @@
 /// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
 /// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
 /// literal. Input with no `&` short-circuits to a clone of `s`.
+///
+/// The next `;` is found through a cursor that only moves forward: the
+/// TS port rescanned the whole remaining suffix for every `&`, which made
+/// a run of bare ampersands (HTML altChunk text, `.rels` targets) cost
+/// quadratic time. Every character is examined once across the whole call.
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
@@ -768,23 +880,28 @@
     let chars: Vec<char> = s.chars().collect();
     let mut out = String::with_capacity(s.len());
     let mut i = 0;
+    #[cfg(test)]
+    bump_entity_scan_bytes(chars.len());
+    // The first ';' at or after the cursor, exactly the one the suffix
+    // scan below `i` would have found.
+    let mut next_semi = chars.iter().position(|&c| c == ';');
     while i < chars.len() {
-        if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
-                let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
-                if let Some(decoded) = decode_entity(&body) {
-                    out.push_str(&decoded);
-                    i = i + 1 + semi_rel + 1;
-                    continue;
-                }
-            }
-            out.push('&');
-            i += 1;
+        if chars[i] == '&'
+            && let Some(semi) = next_semi.filter(|&semi| semi > i)
+            && let Some(decoded) = decode_entity(&chars[i + 1..semi].iter().collect::<String>())
+        {
+            out.push_str(&decoded);
+            i = semi + 1;
         } else {
             out.push(chars[i]);
             i += 1;
         }
+        if next_semi.is_some_and(|semi| i > semi) {
+            let scan = chars[i..].iter().position(|&c| c == ';');
+            #[cfg(test)]
+            bump_entity_scan_bytes(scan.map_or(chars.len().saturating_sub(i), |p| p + 1));
+            next_semi = scan.map(|p| p + i);
+        }
     }
     out
 }
@@ -1178,11 +1295,143 @@
         let root = dom.root(document).unwrap();
         assert_eq!(dom.value(root), "owned");
         assert!(validate_xml("<?xml-stylesheet type='text/css'?>\n<p>owned</p><?tail?>").is_ok());
+    }
+}
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod dom_depth_cap_tests {
+    //! dedup-0008: the permissive parser recursed once per nesting level
+    //! with no bound of its own, so markup that reached it without an XML
+    //! admission check (a renamed auxiliary part, a raised limit, markup
+    //! hidden from the old scan inside an opaque DTD body) could exhaust
+    //! the stack during DOM construction. The hard cap keeps recursion
+    //! bounded whatever admitted the bytes; documents within the admission
+    //! depth (256 by default) parse exactly as before.
+
+    use super::*;
+
+    fn nesting_of(xml: &str) -> usize {
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, xml);
+        let root = dom.root(document).expect("root");
+        let mut node = root;
+        let mut depth = 1usize;
+        while let Some(child) = dom.elements(node, None).first().copied() {
+            node = child;
+            depth += 1;
+        }
+        depth
+    }
+
+    #[test]
+    fn dom_construction_nests_to_the_cap_and_no_deeper() {
+        // At the cap the full chain is kept; one level past it the extra
+        // element's subtree (the <b/> and <c/> inside it) is consumed
+        // without nodes and the document still parses to its root.
+        let at_cap = format!(
+            "{}{}",
+            "<a>".repeat(MAX_DOM_DEPTH),
+            "</a>".repeat(MAX_DOM_DEPTH)
+        );
+        assert_eq!(nesting_of(&at_cap), MAX_DOM_DEPTH);
+        let over = format!(
+            "{}<b/><c/>{}",
+            "<a>".repeat(MAX_DOM_DEPTH + 1),
+            "</a>".repeat(MAX_DOM_DEPTH + 1)
+        );
+        assert_eq!(nesting_of(&over), MAX_DOM_DEPTH);
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &over);
+        let root = dom.root(document).expect("root");
+        assert_eq!(dom.name(root).unwrap().local_name(), "a");
+    }
+
+    #[test]
+    fn fragments_with_many_roots_never_recurse() {
+        // A fragment of many sibling roots parses without a single nested
+        // call: the cap guards nesting, not sibling count.
+        let xml = format!("{}<b>x</b>", "<a/>".repeat(200_000));
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let root = dom
+            .root(document)
+            .expect("the first root is the document's root");
+        assert_eq!(dom.name(root).unwrap().local_name(), "a");
+        // Every sibling parsed: nothing was dropped or nested.
+        assert_eq!(dom.elements(document, None).len(), 200_001);
+        let last = dom.elements(document, None).pop().expect("last root");
+        assert_eq!(dom.name(last).unwrap().local_name(), "b");
+        assert_eq!(dom.value(last), "x");
     }
+
+    #[test]
+    fn quoted_right_angle_in_an_over_depth_tag_stays_in_the_tag() {
+        // The subtree skipper must consume a start tag's quoted '>' before
+        // balancing closes, or a quote-protected angle would re-open as
+        // markup and the skip would end early.
+        let xml = format!(
+            "<r>{}<q a=\"{}>\"></q></r>",
+            "<d>".repeat(MAX_DOM_DEPTH - 1),
+            "x"
+        );
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let root = dom.root(document).expect("root");
+        assert_eq!(dom.name(root).unwrap().local_name(), "r");
+        // The capped subtree ran to EOF without a panic; the document keeps
+        // its root either way.
+    }
 }
 
 #[cfg(test)]
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod entity_scan_budget_tests {
+    //! dedup-0008: `unescape_xml_text` scanned the whole remaining suffix
+    //! for `;` at every `&`, so bare-ampersand runs cost quadratic time in
+    //! every attribute and text body. The monotonic cursor scans each
+    //! character once; the counter pins that mechanism, and the exact-value
+    //! tests below keep the port's decoding contract.
+
+    use super::*;
+
+    #[test]
+    fn entity_delimiter_scans_stay_linear_on_ampersand_runs() {
+        for n in [4_usize * 1024, 16 * 1024, 64 * 1024] {
+            let input = "&".repeat(n);
+            ENTITY_SUFFIX_SCAN_BYTES.with(|c| c.set(0));
+            assert_eq!(unescape_xml_text(&input), input);
+            let scanned = ENTITY_SUFFIX_SCAN_BYTES.with(|c| c.get());
+            assert!(
+                scanned <= 4 * n,
+                "decoding {n} bare ampersands scanned {scanned} characters"
+            );
+        }
+    }
+
+    #[test]
+    fn known_entities_after_ampersand_runs_still_decode() {
+        // A decodable reference after a long bare run must decode exactly
+        // as the suffix scan decoded it, and the run itself stays literal.
+        assert_eq!(
+            unescape_xml_text(&format!("{}&amp;", "&".repeat(1000))),
+            "&".repeat(1001)
+        );
+        assert_eq!(
+            unescape_xml_text(&format!("{}&lt;x&gt;", "&".repeat(1000))),
+            format!("{}<x>", "&".repeat(1000))
+        );
+        // Semicolons behind the cursor are never re-examined: every
+        // reference in a dense field decodes once, in order.
+        assert_eq!(
+            unescape_xml_text("&amp;&amp;&amp;&lt;&#65;&#x42;"),
+            "&&&<AB"
+        );
+    }
+}
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod tag_policy_tests {
     //! N01/N02: the checked gate (`validate_xml`, `try_parse_xdocument`, and
     //! `admission::check_xml_depth`) bounds tag width and namespace budgets
diff --git a/src/admission.rs b/src/admission.rs
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -447,6 +447,15 @@
     // its typed kind.
     let mut archive = crate::opc::open_archive_unchecked(Cursor::new(bytes))
         .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("not a ZIP file: {e}")))?;
+    // dedup-0008: the declared count above came from this module's own
+    // footer scan, which can settle on a different end-of-central-directory
+    // record than the delegated reader (a false small-count EOCD planted in
+    // or after a real comment is skipped by zip's validation but found by
+    // the backward scan here). The archive's own count is what is actually
+    // indexed and read, so the entry budget must hold on it too.
+    if archive.len() > limits.max_entries {
+        return Err(input_over_entries(archive.len() as u64, &limits));
+    }
     // The reader keys entries by name, so exact duplicates collapse into one.
     if (archive.len() as u64) < declared {
         return Err(AdmissionError::new(
@@ -568,11 +577,115 @@
             .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("{main_part}: {e}")))?;
         check_xml_depth(&main_part, &buf, limits)?;
     }
+    // dedup-0008: the extension scan checks parts named `*.xml` / `*.rels`;
+    // rendering, revision processing and comparison follow relationships
+    // and parse the parts those name as XML whatever they are called, so a
+    // renamed auxiliary part carries the same XML budget or refuses with
+    // the same typed INVALID_XML.
+    check_relationship_selected_xml(&archive, &main_part, limits)?;
     Ok(AdmittedPackage {
         entries: archive.len(),
         inflated_bytes: total,
         main_part,
     })
+}
+
+/// The relationship kinds whose targets this engine parses as XML through
+/// the main document part's relationships, whatever the part is named:
+/// rendering loads theme, styles, numbering, settings, comments and the
+/// header/footer/notes stories; the comparer and revision processing add
+/// the extended comments parts, people, the glossary and web settings.
+/// Binary kinds (media, fonts, embedded objects) are deliberately absent.
+const XML_RELATIONSHIP_SUFFIXES: &[&str] = &[
+    "comments",
+    "commentsExtended",
+    "commentsIds",
+    "endnotes",
+    "fontTable",
+    "footer",
+    "glossaryDocument",
+    "header",
+    "numbering",
+    "people",
+    "settings",
+    "styles",
+    "theme",
+    "webSettings",
+];
+
+/// Depth-check the main part's relationship-selected XML parts that the
+/// extension scan skipped: `word/styles.bin` reached through a styles
+/// relationship parses exactly like `word/styles.xml`, so a deeply nested
+/// one refuses with the same typed `INVALID_XML` whatever its name
+/// (dedup-0008). Parts already scanned by their `.xml`/`.rels` name are
+/// not read again; targets the archive does not hold are left to the
+/// reader that treats a missing part as empty.
+fn check_relationship_selected_xml<R: Read + std::io::Seek>(
+    archive: &ZipArchive<R>,
+    main: &str,
+    limits: InputLimits,
+) -> Result<(), AdmissionError> {
+    let stripped = main.strip_prefix('/').unwrap_or(main);
+    let (dir, file) = stripped.rsplit_once('/').unwrap_or(("", stripped));
+    let rels_name = if dir.is_empty() {
+        format!("_rels/{file}.rels")
+    } else {
+        format!("{dir}/_rels/{file}.rels")
+    };
+    let rels = match archive.by_name(&rels_name) {
+        Ok(mut entry) => {
+            let mut buf = Vec::new();
+            (&mut entry)
+                .take(limits.max_part_bytes.saturating_add(1))
+                .read_to_end(&mut buf)
+                .map_err(|e| {
+                    AdmissionError::new(
+                        AdmissionErrorKind::InvalidPackage,
+                        format!("{rels_name}: {e}"),
+                    )
+                })?;
+            buf
+        }
+        Err(_) => return Ok(()),
+    };
+    let mut checked = HashSet::new();
+    for attrs in elements(&rels, "Relationship") {
+        if attrs
+            .get("TargetMode")
+            .is_some_and(|mode| mode == "External")
+        {
+            continue;
+        }
+        let Some(rel_type) = attrs.get("Type") else {
+            continue;
+        };
+        let suffix = rel_type.rsplit('/').next().unwrap_or("");
+        if !XML_RELATIONSHIP_SUFFIXES.contains(&suffix) {
+            continue;
+        }
+        let Some(target) = attrs.get("Target") else {
+            continue;
+        };
+        let part = crate::opc::resolve_rel_target(main, target);
+        let lower = part.to_ascii_lowercase();
+        // `.xml`/`.rels` names were depth-checked in the entry loop; each
+        // renamed target is read once however many relationships name it.
+        if lower.ends_with(".xml") || lower.ends_with(".rels") || !checked.insert(lower) {
+            continue;
+        }
+        let Ok(mut entry) = archive.by_name(&part) else {
+            continue;
+        };
+        let mut buf = Vec::new();
+        (&mut entry)
+            .take(limits.max_part_bytes.saturating_add(1))
+            .read_to_end(&mut buf)
+            .map_err(|e| {
+                AdmissionError::new(AdmissionErrorKind::InvalidPackage, format!("{part}: {e}"))
+            })?;
+        check_xml_depth(&part, &buf, limits)?;
+    }
+    Ok(())
 }
 
 /// The entry count the end-of-central-directory record declares (ZIP64
@@ -812,6 +925,18 @@
                     .ok_or_else(|| invalid("end tag with no open element".to_string()))?;
                 policy.close();
             }
+            // dedup-0008: the two opaque constructs `validate_xml` already
+            // refuses used to pass here untouched, while the permissive DOM
+            // treats a top-level `<!` as ending at the first `>` — deeply
+            // nested markup hidden inside a quoted DTD body or a top-level
+            // CDATA section reached DOM construction unmeasured. The two
+            // gates now refuse the same constructs, with the same words.
+            Ok(Event::DocType(_)) => {
+                return Err(invalid("DTD declarations are unsupported".to_string()));
+            }
+            Ok(Event::CData(_)) if depth == 0 => {
+                return Err(invalid("CDATA outside XML root".to_string()));
+            }
             Ok(Event::Eof) if depth > 0 => {
                 return Err(invalid(format!("{depth} element(s) left unclosed")));
             }
@@ -2442,8 +2567,120 @@
             set_central_attributes(&mut bytes, name, version_made_by, external);
             admit(&bytes, InputLimits::default())
                 .unwrap_or_else(|err| panic!("{version_made_by:#06x}/{external:o} {name}: {err}"));
+        }
+    }
+
+    // ── dedup-0008 ──────────────────────────────────────────────────────────
+
+    /// A minimal package whose main document part is `body` instead of the
+    /// stock one (`docx_with` appends, which would duplicate the name).
+    fn docx_main(body: &str) -> Vec<u8> {
+        let entries: Vec<(&str, &[u8])> = vec![
+            ("[Content_Types].xml", TYPES.as_bytes()),
+            ("_rels/.rels", RELS.as_bytes()),
+            ("word/document.xml", body.as_bytes()),
+        ];
+        zip_of(&entries, CompressionMethod::Deflated)
+    }
+
+    /// dedup-0008 renamed auxiliary XML: the extension scan checks parts
+    /// named `*.xml`/`*.rels`, but the engine follows relationships and
+    /// parses what they name whatever it is called. A styles relationship
+    /// to a deeply nested `styles.bin` must refuse with the typed
+    /// `INVALID_XML`, and the same relationship to a shallow renamed part
+    /// must still admit.
+    #[test]
+    fn relationship_selected_xml_parts_carry_the_xml_budget_whatever_their_name() {
+        let deep: Vec<u8> = format!("{}{}", "<a>".repeat(257), "</a>".repeat(257)).into_bytes();
+        let styles_rel = r#"<Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.bin"/>"#;
+        let refused = zip_names::docx_package(
+            styles_rel,
+            &[zip_names::RawEntry::raw(b"word/styles.bin", &deep)],
+        );
+        assert_eq!(kind(&refused, InputLimits::default()).code(), "INVALID_XML");
+
+        let shallow: &[u8] =
+            b"<s:styles xmlns:s=\"urn:styles\"><s:style s:styleId=\"a\"/></s:styles>";
+        let admitted = zip_names::docx_package(
+            styles_rel,
+            &[zip_names::RawEntry::raw(b"word/styles.bin", shallow)],
+        );
+        assert!(admit(&admitted, InputLimits::default()).is_ok());
+    }
+
+    /// The same typed refusal for the notes and chrome stories get_revisions,
+    /// comparison and revision acceptance/rejection parse under non-XML
+    /// part names.
+    #[test]
+    fn renamed_notes_and_chrome_refuse_when_too_deep() {
+        let deep: Vec<u8> = format!("{}{}", "<a>".repeat(257), "</a>".repeat(257)).into_bytes();
+        for rel in ["footnotes", "endnotes", "header", "footer"] {
+            let rels = format!(
+                r#"<Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/{rel}" Target="{rel}.bin"/>"#
+            );
+            let part = format!("word/{rel}.bin");
+            let bytes =
+                zip_names::docx_package(&rels, &[zip_names::RawEntry::raw(part.as_bytes(), &deep)]);
+            assert_eq!(
+                kind(&bytes, InputLimits::default()).code(),
+                "INVALID_XML",
+                "{rel}"
+            );
         }
     }
+
+    /// dedup-0008 DTD/CDATA differential: admission used to ignore opaque
+    /// DTD and top-level CDATA events, while the permissive DOM ends a
+    /// top-level `<!` construct at the first `>`, so nested markup hidden
+    /// inside a quoted DTD body or a top-level CDATA section reached DOM
+    /// construction unmeasured. Admission now refuses both exactly like
+    /// `validate_xml`, and in-root CDATA stays admitted.
+    #[test]
+    fn admission_refuses_dtd_and_top_level_cdata_like_the_checked_gate() {
+        let hidden_depth = r#"<!DOCTYPE a [ <!ENTITY e "<a><a><a>x</a></a></a>"> ]><w:document/>"#;
+        let bytes = docx_main(hidden_depth);
+        assert_eq!(kind(&bytes, InputLimits::default()).code(), "INVALID_XML");
+
+        let top_cdata = "<![CDATA[ <a/> ]]><w:document/>";
+        let bytes = docx_main(top_cdata);
+        assert_eq!(kind(&bytes, InputLimits::default()).code(), "INVALID_XML");
+
+        let in_root = "<w:document><![CDATA[raw <>]]></w:document>";
+        let bytes = docx_main(in_root);
+        assert!(admit(&bytes, InputLimits::default()).is_ok());
+    }
+
+    /// dedup-0008 EOCD selection differential: this module's footer scan
+    /// takes the last EOCD-looking signature, while the delegated reader
+    /// validates each candidate (a comment length that overruns the file
+    /// makes it skip a planted record). A false small-count footer must not
+    /// bypass the entry budget: the archive the reader actually opened is
+    /// capped too.
+    #[test]
+    fn a_false_small_count_footer_cannot_bypass_the_entry_budget() {
+        let false_footer = |total: u16| {
+            let mut eocd = [0u8; 22];
+            eocd[0..4].copy_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
+            eocd[10..12].copy_from_slice(&total.to_le_bytes());
+            // The overrunning comment length is what makes zip reject the
+            // planted record; the scan here never read it.
+            eocd[20..22].copy_from_slice(&u16::MAX.to_le_bytes());
+            eocd
+        };
+        let four_max = InputLimits {
+            max_entries: 4,
+            ..InputLimits::default()
+        };
+        let four = docx_with(&[("word/extra.xml", b"<e/>")]);
+        let mut within = four;
+        within.extend_from_slice(&false_footer(1));
+        assert!(admit(&within, four_max).is_ok());
+
+        let five = docx_with(&[("word/extra.xml", b"<e/>"), ("word/extra2.xml", b"<e/>")]);
+        let mut over = five;
+        over.extend_from_slice(&false_footer(1));
+        assert_eq!(kind(&over, four_max).code(), "INPUT_LIMIT");
+    }
 }
 
 #[cfg(test)]
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1582,6 +1582,22 @@
     /// CPs of the section marks (`\x0C` ending a section, which ends its
     /// paragraph too); any other `\x0C` is a page break inside a paragraph.
     section_marks: Vec<u32>,
+}
+
+/// Deepest field nesting `Story::blocks` tracks (dedup-0008). Word nests
+/// fields at most a few levels deep (it refuses past 20), so a bound far
+/// above that only bites crafted input: begin markers past the bound are
+/// dropped, and by then the text sits inside some open code phase anyway.
+const MAX_FIELD_NESTING: usize = 1024;
+
+// dedup-0008 mechanism counter: field-stack entries the `in_code`
+// predicate examined. The original scan looked at every stack entry for
+// every character, making a story that opens many fields quadratic; the
+// counter looks at one. Per thread, so neighbouring tests do not count
+// each other's work.
+#[cfg(test)]
+thread_local! {
+    pub(crate) static FIELD_CODE_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
 }
 
 impl Story<'_> {
@@ -1591,9 +1607,15 @@
     fn blocks(&self, chars: &[StoryChar]) -> Vec<Block> {
         let mut out = Vec::new();
         let mut spans: Vec<Span> = Vec::new();
-        // Field nesting: true while inside a field's code (before its
-        // separator).
+        // Field nesting: one frame per begin marker (`\u{13}`), flipped to
+        // its result phase by its separator (`\u{14}`) and popped by its end
+        // (`\u{15}`). `code_fields` counts the frames still in their code
+        // phase, so "any code ancestor suppresses" is one comparison per
+        // character instead of a scan of the whole stack (dedup-0008: a
+        // crafted .doc that opens thousands of fields used to pay quadratic
+        // work).
         let mut fields: Vec<bool> = Vec::new();
+        let mut code_fields = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1623,30 @@
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            #[cfg(test)]
+            FIELD_CODE_PROBES.with(|c| c.set(c.get() + 1));
+            let in_code = code_fields > 0;
             let text = match story.ch {
                 '\u{13}' => {
-                    fields.push(true);
+                    if fields.len() < MAX_FIELD_NESTING {
+                        fields.push(true);
+                        code_fields = code_fields.saturating_add(1);
+                    }
                     continue;
                 }
                 '\u{14}' => {
-                    if let Some(top) = fields.last_mut() {
+                    if let Some(top) = fields.last_mut()
+                        && *top
+                    {
                         *top = false;
+                        code_fields = code_fields.saturating_sub(1);
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        code_fields = code_fields.saturating_sub(1);
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +1999,109 @@
         assert_eq!(texts, ["aoneb"]);
     }
 
+    /// dedup-0008: `in_code` used to scan the whole field stack before
+    /// every character, so a .doc that opens thousands of fields — each
+    /// begin leaving another frame behind — cost quadratic work to read or
+    /// convert, even after every field had entered its result phase (the
+    /// all-false stack scanned to its end every time). The counter makes it
+    /// one probe per character.
+    #[test]
+    fn field_code_suppression_costs_constant_work_per_character() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let mut text = String::new();
+        for _ in 0..600 {
+            // Begin a field and enter its result phase without ending it:
+            // the stack grows, every frame false.
+            text.push('\u{13}');
+            text.push('\u{14}');
+        }
+        text.push_str("shown\r");
+        let chars = story_chars(&text);
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        FIELD_CODE_PROBES.with(|c| c.set(0));
+        let blocks = story.blocks(&chars);
+        assert_eq!(
+            FIELD_CODE_PROBES.with(|c| c.get()),
+            chars.len(),
+            "one in_code probe per character"
+        );
+        let texts: Vec<String> = blocks
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        // Every field sits in its result phase, so the text is shown — the
+        // suppression rule only bites while a code ancestor is open.
+        assert_eq!(texts, ["shown"]);
+    }
+
+    /// Past the nesting budget the stack stops growing: further begin
+    /// markers are dropped, and the story stays suppressed inside the
+    /// frames already open.
+    #[test]
+    fn field_nesting_past_the_budget_drops_further_begins() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let mut text = String::new();
+        for _ in 0..(MAX_FIELD_NESTING + 5) {
+            text.push('\u{13}');
+        }
+        text.push_str("hidden\r");
+        let chars = story_chars(&text);
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        FIELD_CODE_PROBES.with(|c| c.set(0));
+        let blocks = story.blocks(&chars);
+        assert_eq!(
+            FIELD_CODE_PROBES.with(|c| c.get()),
+            chars.len(),
+            "the cap changes the cost of nothing else"
+        );
+        // Everything after the first begin sits in code and is suppressed,
+        // paragraph mark included.
+        assert!(blocks.is_empty());
+    }
+
+    /// The counter must follow the stack exactly: repeated separators leave
+    /// result phase once, an end pops the frame it closes, and unmatched
+    /// markers on an empty stack change nothing.
+    #[test]
+    fn separator_and_end_markers_keep_their_meaning_on_any_stack() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let chars = story_chars("\u{14}a\u{15}\u{15}\u{13}code\u{14}\u{14}one\u{15}b\r");
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let texts: Vec<String> = story
+            .blocks(&chars)
+            .iter()
+            .map(|b| match b {
+                Block::Paragraph(p) => p.text(),
+                Block::Table(_) => "table".into(),
+            })
+            .collect();
+        assert_eq!(texts, ["aoneb"]);
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -487,6 +487,13 @@
             break;
         }
         let payload = off + 6;
+        // dedup-0008: every count- or string-taking handler reads inside
+        // this record's declared extent only. A short record that declares
+        // counts its own bytes cannot cover used to consume payload from
+        // the records after it — which the outer loop then revisited, so a
+        // few hundred crafted bytes could amplify into billions of decoded
+        // coordinates or gigabytes of overlapping retained text.
+        let rec_end = off + size2;
         match func {
             0x0000 => break,
             0x020B => {
@@ -533,7 +540,7 @@
             }
             0x02FB => {
                 if let Some(i) = objects.iter().position(|o| matches!(o, GdiObj::Empty)) {
-                    objects[i] = WmfFont::parse(data, payload)
+                    objects[i] = WmfFont::parse(&data[..rec_end], payload)
                         .map_or(GdiObj::Other, |f| GdiObj::Font(std::rc::Rc::new(f)));
                 }
             }
@@ -606,16 +613,22 @@
                 let n = usize::from(read_u16(data, payload + 4)?);
                 let opts = read_u16(data, payload + 6)?;
                 let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
-                if let Some(bytes) = data.get(at..at + n) {
+                // The string belongs to this record: a count reaching into
+                // the following records is refused, not decoded.
+                if at.checked_add(n).is_some_and(|end| end <= rec_end)
+                    && let Some(bytes) = data.get(at..at + n)
+                {
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
             // TEXTOUT: count, string (word-padded), y, x.
             0x0521 => {
                 let n = usize::from(read_u16(data, payload)?);
-                let at = payload + 2 + n.next_multiple_of(2);
-                let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
-                if let Some(bytes) = data.get(payload + 2..payload + 2 + n) {
+                let padded = payload + 2 + n.next_multiple_of(2);
+                if padded.checked_add(4).is_some_and(|end| end <= rec_end)
+                    && let Some(bytes) = data.get(payload + 2..payload + 2 + n)
+                {
+                    let (y, x) = (read_i16(data, padded)?, read_i16(data, padded + 2)?);
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
@@ -627,37 +640,54 @@
             }
             0x0324 => {
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut pts = Vec::with_capacity(n);
-                let mut p = payload + 2;
-                for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
-                    pts.push(map.map(x, y));
-                    p += 4;
-                }
-                if let Some(fill) = dc.brush {
-                    canvas.fill_polygon(&pts, fill);
+                // The declared point count must fit the record's own bytes;
+                // coordinates reaching into the following records are not
+                // decoded (the outer loop reads those bytes once already).
+                if (payload + 2).saturating_add(n.saturating_mul(4)) <= rec_end {
+                    let mut pts = Vec::with_capacity(n);
+                    let mut p = payload + 2;
+                    for _ in 0..n {
+                        let x = read_i16(data, p)? as i32;
+                        let y = read_i16(data, p + 2)? as i32;
+                        pts.push(map.map(x, y));
+                        p += 4;
+                    }
+                    if let Some(fill) = dc.brush {
+                        canvas.fill_polygon(&pts, fill);
+                    }
                 }
             }
             // META_POLYPOLYGON: ring count, each ring's point count, then
             // every ring's points, filled as one shape (b88ac900's logo).
             0x0538 => {
                 let rings = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut p = payload + 2 + 2 * rings;
-                let mut subpaths = Vec::with_capacity(rings);
-                for r in 0..rings {
-                    let n = read_u16(data, payload + 2 + 2 * r)? as usize;
-                    let mut ring = Vec::with_capacity(n);
-                    for _ in 0..n {
-                        let x = read_i16(data, p)? as i32;
-                        let y = read_i16(data, p + 2)? as i32;
-                        ring.push(map.map(x, y));
-                        p += 4;
+                let counts_at = payload + 2;
+                let counts_end = counts_at.saturating_add(rings.saturating_mul(2));
+                let fits = counts_end <= rec_end && {
+                    let mut end = counts_end;
+                    for r in 0..rings {
+                        let n = usize::from(read_u16(data, counts_at + 2 * r).unwrap_or(0));
+                        end = end.saturating_add(n.saturating_mul(4));
                     }
-                    subpaths.push(ring);
-                }
-                if let Some(fill) = dc.brush {
-                    canvas.fill_path(&subpaths, fill, winding);
+                    end <= rec_end
+                };
+                if fits {
+                    let mut p = counts_end;
+                    let mut subpaths = Vec::with_capacity(rings);
+                    for r in 0..rings {
+                        let n = read_u16(data, counts_at + 2 * r).unwrap_or(0) as usize;
+                        let mut ring = Vec::with_capacity(n);
+                        for _ in 0..n {
+                            let x = read_i16(data, p)? as i32;
+                            let y = read_i16(data, p + 2)? as i32;
+                            ring.push(map.map(x, y));
+                            p += 4;
+                        }
+                        subpaths.push(ring);
+                    }
+                    if let Some(fill) = dc.brush {
+                        canvas.fill_path(&subpaths, fill, winding);
+                    }
                 }
             }
             0x0325 => {
@@ -666,17 +696,19 @@
                     continue;
                 };
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut prev: Option<(i32, i32)> = None;
-                let mut p = payload + 2;
-                for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
-                    let cur = map.map(x, y);
-                    if let Some(pr) = prev {
-                        canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, map.pen_px(pen_w));
+                if (payload + 2).saturating_add(n.saturating_mul(4)) <= rec_end {
+                    let mut prev: Option<(i32, i32)> = None;
+                    let mut p = payload + 2;
+                    for _ in 0..n {
+                        let x = read_i16(data, p)? as i32;
+                        let y = read_i16(data, p + 2)? as i32;
+                        let cur = map.map(x, y);
+                        if let Some(pr) = prev {
+                            canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, map.pen_px(pen_w));
+                        }
+                        prev = Some(cur);
+                        p += 4;
                     }
-                    prev = Some(cur);
-                    p += 4;
                 }
             }
             _ => {}
@@ -1899,7 +1931,197 @@
             assert_eq!(render(&bad).unwrap(), expected);
         }
         assert_eq!(render(&bytes[..bytes.len() - 6]).unwrap(), expected);
+    }
+}
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod wmf_record_bounds_tests {
+    //! dedup-0008: a WMF handler used to read counts and strings from the
+    //! whole buffer, so a short record could claim payload belonging to the
+    //! records after it — which the outer loop then revisited, turning a
+    //! few hundred crafted bytes into thousands of repeated claims. Every
+    //! read is now confined to the record's own declared extent, and a
+    //! claim that does not fit is refused before any decoding.
+    use super::*;
+
+    fn placeable() -> Vec<u8> {
+        let mut d = vec![0u8; 22];
+        d[0..4].copy_from_slice(&PLACEABLE_KEY);
+        for (i, v) in [0_i16, 0, 100, 100].iter().enumerate() {
+            d[6 + 2 * i..8 + 2 * i].copy_from_slice(&v.to_le_bytes());
+        }
+        d[14..16].copy_from_slice(&1440u16.to_le_bytes());
+        let mut header = vec![0u8; 18];
+        header[0..2].copy_from_slice(&1u16.to_le_bytes());
+        header[2..4].copy_from_slice(&9u16.to_le_bytes());
+        d.extend_from_slice(&header);
+        d
+    }
+
+    /// One record whose declared size (words) covers exactly `payload`.
+    fn record(d: &mut Vec<u8>, func: u16, payload: &[u8]) {
+        d.extend_from_slice(&(3 + payload.len() as u32 / 2).to_le_bytes());
+        d.extend_from_slice(&func.to_le_bytes());
+        d.extend_from_slice(payload);
+    }
+
+    #[test]
+    fn a_short_exttextout_cannot_claim_the_following_bytes_as_text() {
+        let mut d = placeable();
+        let mut payload = Vec::new();
+        payload.extend_from_slice(&20i16.to_le_bytes()); // y
+        payload.extend_from_slice(&40i16.to_le_bytes()); // x
+        payload.extend_from_slice(&60000u16.to_le_bytes()); // string length
+        payload.extend_from_slice(&0u16.to_le_bytes()); // options
+        record(&mut d, 0x0A32, &payload);
+        // Bytes the old reader retained as 60000 characters of text.
+        d.extend_from_slice(&[b'x'; 60_000]);
+        record(&mut d, 0x0000, &[]); // EOF
+        let (_, _, _, texts) = render(&d).expect("wmf");
+        assert!(
+            texts.is_empty(),
+            "a string reaching past its own record must be refused: {texts:?}"
+        );
     }
+
+    #[test]
+    fn repeated_short_claims_retain_nothing() {
+        // The amplification was per record: many short claims over one
+        // shared tail. Each claim is now refused before decoding.
+        let mut d = placeable();
+        for _ in 0..50 {
+            let mut payload = Vec::new();
+            payload.extend_from_slice(&20i16.to_le_bytes());
+            payload.extend_from_slice(&40i16.to_le_bytes());
+            payload.extend_from_slice(&60000u16.to_le_bytes());
+            payload.extend_from_slice(&0u16.to_le_bytes());
+            record(&mut d, 0x0A32, &payload);
+        }
+        d.extend_from_slice(&[b'x'; 60_000]);
+        record(&mut d, 0x0000, &[]);
+        let (_, _, _, texts) = render(&d).expect("wmf");
+        assert!(texts.is_empty(), "{texts:?}");
+    }
+
+    #[test]
+    fn a_short_polygon_cannot_decode_the_following_bytes_as_points() {
+        let mut d = placeable();
+        // Black brush into slot 0, selected.
+        let mut brush_payload = Vec::new();
+        brush_payload.extend_from_slice(&0u16.to_le_bytes()); // style
+        brush_payload.extend_from_slice(&0u32.to_le_bytes()); // color: black
+        brush_payload.extend_from_slice(&0u16.to_le_bytes()); // hatch
+        record(&mut d, 0x02FC, &brush_payload);
+        record(&mut d, 0x012D, &0u16.to_le_bytes());
+        // POLYGON declaring 20000 points in a record holding only the count.
+        record(&mut d, 0x0324, &20000u16.to_le_bytes());
+        // The triangle those points describe — pixels the old reader would
+        // have painted from bytes that belong to no record of their own.
+        let mut filler = Vec::new();
+        for _ in 0..20_000 {
+            for (x, y) in [(0_i16, 0_i16), (99, 99), (0, 99)] {
+                filler.extend_from_slice(&x.to_le_bytes());
+                filler.extend_from_slice(&y.to_le_bytes());
+            }
+        }
+        d.extend_from_slice(&filler);
+        record(&mut d, 0x0000, &[]);
+        let (w, h, rgb) = rasterize(&d).expect("wmf");
+        let inside = ((60 * h as usize / 100) * w as usize + 10 * w as usize / 100) * 3;
+        assert_eq!(
+            [rgb[inside], rgb[inside + 1], rgb[inside + 2]],
+            [255, 255, 255],
+            "points reaching past the record must not be painted"
+        );
+        assert!(rgb.iter().all(|&channel| channel == 255));
+    }
+
+    #[test]
+    fn a_short_polypolygon_ring_table_is_refused_whole() {
+        let mut d = placeable();
+        let mut brush_payload = Vec::new();
+        brush_payload.extend_from_slice(&0u16.to_le_bytes());
+        brush_payload.extend_from_slice(&0u32.to_le_bytes());
+        brush_payload.extend_from_slice(&0u16.to_le_bytes());
+        record(&mut d, 0x02FC, &brush_payload);
+        record(&mut d, 0x012D, &0u16.to_le_bytes());
+        // 2 rings whose point counts the record does not carry.
+        record(&mut d, 0x0538, &2u16.to_le_bytes());
+        let mut filler = Vec::new();
+        for _ in 0..2000 {
+            for (x, y) in [(0_i16, 0_i16), (99, 0), (99, 99)] {
+                filler.extend_from_slice(&x.to_le_bytes());
+                filler.extend_from_slice(&y.to_le_bytes());
+            }
+        }
+        d.extend_from_slice(&filler);
+        record(&mut d, 0x0000, &[]);
+        let (_, _, rgb) = rasterize(&d).expect("wmf");
+        assert!(rgb.iter().all(|&channel| channel == 255));
+    }
+
+    #[test]
+    fn a_short_polyline_strokes_nothing() {
+        let mut d = placeable();
+        // Black pen into slot 0, selected.
+        let mut pen_payload = Vec::new();
+        pen_payload.extend_from_slice(&0u16.to_le_bytes()); // style
+        pen_payload.extend_from_slice(&1i16.to_le_bytes()); // width
+        pen_payload.extend_from_slice(&0u32.to_le_bytes()); // color
+        pen_payload.extend_from_slice(&0u16.to_le_bytes()); // ignored word
+        record(&mut d, 0x02FA, &pen_payload);
+        record(&mut d, 0x012D, &0u16.to_le_bytes());
+        // POLYLINE declaring 10000 points in a record holding only the count.
+        record(&mut d, 0x0325, &10000u16.to_le_bytes());
+        let mut filler = Vec::new();
+        for _ in 0..10_000 {
+            filler.extend_from_slice(&10i16.to_le_bytes());
+            filler.extend_from_slice(&50i16.to_le_bytes());
+            filler.extend_from_slice(&90i16.to_le_bytes());
+            filler.extend_from_slice(&50i16.to_le_bytes());
+        }
+        d.extend_from_slice(&filler);
+        record(&mut d, 0x0000, &[]);
+        let (_, _, rgb) = rasterize(&d).expect("wmf");
+        assert!(
+            rgb.iter().all(|&channel| channel == 255),
+            "points reaching past the record must not be stroked"
+        );
+    }
+
+    #[test]
+    fn well_sized_records_still_paint_and_write_text() {
+        // The confinement must not refuse honest records: a POLYGON that
+        // carries its points and an EXTTEXTOUT that carries its string.
+        let mut d = placeable();
+        let mut brush_payload = Vec::new();
+        brush_payload.extend_from_slice(&0u16.to_le_bytes());
+        brush_payload.extend_from_slice(&0u32.to_le_bytes());
+        brush_payload.extend_from_slice(&0u16.to_le_bytes());
+        record(&mut d, 0x02FC, &brush_payload);
+        record(&mut d, 0x012D, &0u16.to_le_bytes());
+        let mut polygon = Vec::new();
+        polygon.extend_from_slice(&3u16.to_le_bytes());
+        for (x, y) in [(10_i16, 10), (90, 10), (50, 90)] {
+            polygon.extend_from_slice(&x.to_le_bytes());
+            polygon.extend_from_slice(&y.to_le_bytes());
+        }
+        record(&mut d, 0x0324, &polygon);
+        let mut text = Vec::new();
+        text.extend_from_slice(&20i16.to_le_bytes());
+        text.extend_from_slice(&40i16.to_le_bytes());
+        text.extend_from_slice(&2u16.to_le_bytes());
+        text.extend_from_slice(&0u16.to_le_bytes());
+        text.extend_from_slice(b"hi");
+        record(&mut d, 0x0A32, &text);
+        record(&mut d, 0x0000, &[]);
+        let (w, h, rgb, texts) = render(&d).expect("wmf");
+        assert_eq!(texts.len(), 1);
+        assert_eq!(texts[0].text, "hi");
+        let inside = ((50 * h as usize / 100) * w as usize + 50 * w as usize / 100) * 3;
+        assert_eq!([rgb[inside], rgb[inside + 1], rgb[inside + 2]], [0, 0, 0]);
+    }
 }
 
 #[cfg(test)]
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -720,6 +720,12 @@
     }
     out
 }
+
+/// A supported entity reference is at most 10 bytes from its `&` to its
+/// `;` (`&#1114111;` is the longest that decodes), so the delimiter search
+/// never needs to look further: an HTML text run of bare ampersands used
+/// to scan the whole remaining suffix per `&` (dedup-0008).
+const MAX_ENTITY_BYTES: usize = 10;
 
 pub(crate) fn decode_entities(s: &str) -> String {
     if !s.contains('&') {
@@ -730,7 +736,18 @@
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // Search only the window a supported reference can occupy; a ';'
+        // past it can never terminate one (the accepted range was already
+        // `e <= 10`, but the search itself used to scan the whole suffix).
+        // Byte search: ';' is ASCII, so no multi-byte character can match,
+        // and the window may end mid-character where a str slice could not.
+        let window = tail
+            .as_bytes()
+            .get(..MAX_ENTITY_BYTES + 1)
+            .unwrap_or(tail.as_bytes());
+        #[cfg(test)]
+        bump_semi_search_bytes(window.len());
+        let semi = window.iter().position(|&b| b == b';');
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -762,6 +779,20 @@
     out
 }
 
+// dedup-0008 mechanism counter: bytes the HTML entity decoder scanned
+// looking for ';'. Per thread, so neighbouring tests do not count each
+// other's work.
+#[cfg(test)]
+thread_local! {
+    static SEMI_SEARCH_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
+}
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+fn bump_semi_search_bytes(by: usize) {
+    SEMI_SEARCH_BYTES.with(|c| c.set(c.get() + by));
+}
+
 fn xml_escape(s: &str) -> String {
     s.replace('&', "&amp;")
         .replace('<', "&lt;")
@@ -930,7 +961,29 @@
             );
         }
 
+        /// dedup-0008: the `;` search used to scan the whole remaining
+        /// suffix for every `&`, so a bare-ampersand run in an HTML altChunk
+        /// cost quadratic time. The search now covers only the window a
+        /// supported reference can occupy.
         #[test]
+        fn entity_delimiter_searches_stay_bounded_on_ampersand_runs() {
+            for n in [4_usize * 1024, 16 * 1024, 64 * 1024] {
+                let input = "&".repeat(n);
+                SEMI_SEARCH_BYTES.with(|c| c.set(0));
+                assert_eq!(decode_entities(&input), input);
+                let scanned = SEMI_SEARCH_BYTES.with(|c| c.get());
+                assert!(
+                    scanned <= 12 * n,
+                    "decoding {n} bare ampersands scanned {scanned} bytes"
+                );
+            }
+            // A non-ASCII text run between ampersands keeps the window
+            // byte-sliced without ever splitting a character boundary.
+            let mixed = "&ééé&amp;ééé&";
+            assert_eq!(decode_entities(mixed), "&ééé&ééé&");
+        }
+
+        #[test]
         fn css_lengths_convert_physical_units_and_decimal_commas() {
             for (length, points) in [
                 ("12pt", 12.0),
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -282,6 +282,18 @@
 fn read_package(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<PartFs, OpcError> {
     crate::admission::preindex_budget(bytes, limits).map_err(refused)?;
     let mut archive = open_archive(Cursor::new(bytes))?;
+    // dedup-0008: the preindex count comes from jubarte's own footer scan,
+    // which can settle on a different end-of-central-directory record than
+    // the delegated reader (a false small-count EOCD planted in or after a
+    // real comment is skipped by zip's validation but found by the scan).
+    // The archive's own count is what this loop reads, so the budget holds
+    // on it as well.
+    if archive.len() > limits.max_entries {
+        return Err(refused(crate::admission::input_over_entries(
+            archive.len() as u64,
+            &limits,
+        )));
+    }
     let mut parts = HashMap::new();
     let mut source_order = HashMap::new();
     let mut identities = HashSet::new();
@@ -674,11 +686,15 @@
     /// (the form `parts()` returns) and `.`/`..` segments resolved. An echoed
     /// absolute target ("/word/footer1.xml") was later re-prefixed into a
     /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
+    ///
+    /// The target is taken verbatim: relationship attributes are entity-
+    /// decoded once where they are parsed (`oxml_opc::Relationships::from_xml`
+    /// over the raw part), so decoding again here would double-decode a
+    /// target that legitimately carries a literal `&…;` sequence and made
+    /// repeated `&amp;` in one `.rels` attribute cost quadratic work
+    /// (dedup-0008).
     pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
-        // PartFs receives raw relationship attributes; DOM consumers already
-        // decoded XML entities and call the pure resolver directly.
-        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
-        resolve_rel_target(source_part, &target)
+        resolve_rel_target(source_part, rel_target)
     }
 
     /// `contentTypeFor(part)`.
@@ -1251,7 +1267,33 @@
         assert_eq!(
             fs.add_document_relationship("word/document.xml", "image", "next.png"),
             "rId8"
+        );
+    }
+
+    /// dedup-0008: relationship targets are entity-decoded once, where the
+    /// `.rels` part is parsed — resolving must not decode them again, or a
+    /// part whose name legitimately carries `&amp;` (the single decode of
+    /// `&amp;amp;`) stops resolving, and one target full of `&amp;` costs a
+    /// full suffix scan per resolution.
+    #[test]
+    fn relationship_targets_decode_once_at_parse_not_again_at_resolution() {
+        let bytes = package_with_document_rels(
+            r#"<Relationship Id="rId7" Type="image" Target="media/a&amp;amp;b.png"/>"#,
+        );
+        let fs = PartFs::open(&bytes).unwrap();
+        let rel = &fs.read_rels_for("word/document.xml").unwrap().items[0];
+        assert_eq!(rel.target, "media/a&amp;b.png");
+        assert_eq!(
+            fs.resolve_rel_target("word/document.xml", &rel.target),
+            "word/media/a&amp;b.png"
         );
+        // A long literal ampersand run resolves verbatim, however many
+        // times a crafted .rels attribute repeats the sequence.
+        let heavy = format!("media/{}", "&amp;".repeat(20_000));
+        assert_eq!(
+            fs.resolve_rel_target("word/document.xml", &heavy),
+            format!("word/{}", &heavy)
+        );
     }
 
     #[test]
@@ -1667,6 +1709,34 @@
     #[test]
     fn exact_budget_control() {
         let bytes = fixture::package();
+        assert!(PartFs::open_within(&bytes, fixture::limits()).is_ok());
+    }
+
+    /// dedup-0008: a false small-count EOCD planted after the real one (its
+    /// overrunning comment length makes the delegated reader skip it) must
+    /// not bypass the entry budget — the archive the reader actually opened
+    /// is capped too.
+    #[test]
+    fn a_false_footer_cannot_bypass_the_opened_entry_budget() {
+        let mut bytes = fixture::package(); // 3 entries
+        let mut eocd = [0u8; 22];
+        eocd[0..4].copy_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
+        eocd[10..12].copy_from_slice(&1u16.to_le_bytes());
+        eocd[20..22].copy_from_slice(&u16::MAX.to_le_bytes());
+        bytes.extend_from_slice(&eocd);
+
+        let two = InputLimits {
+            max_entries: 2,
+            ..fixture::limits()
+        };
+        let text = PartFs::open_within(&bytes, two)
+            .err()
+            .expect("over-budget package must be refused")
+            .to_string();
+        assert!(text.contains("INPUT_LIMIT"), "{text}");
+        assert!(text.contains("3 ZIP entries"), "{text}");
+        // At the real count the package still opens: the planted footer
+        // changes nothing for an honest budget.
         assert!(PartFs::open_within(&bytes, fixture::limits()).is_ok());
     }
 }

```
