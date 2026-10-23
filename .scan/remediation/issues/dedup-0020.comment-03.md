## Patch (continued) (1/1)

+pub fn main_part_is_strict_within(bytes: &[u8], max_head: usize) -> bool {
     let Ok(mut archive) = crate::opc::open_archive(Cursor::new(bytes)) else {
         return false;
     };
@@ -689,6 +705,9 @@ pub fn main_part_is_strict(bytes: &[u8]) -> bool {
             break;
         }
         head.extend(chunk.iter().copied().take(read));
+        if head.len() > max_head {
+            return false;
+        }
         let window = String::from_utf8_lossy(&head);
         if let Some((start, _)) = root_open(&window)
             && window.split_at(start).1.contains('>')
@@ -1150,6 +1169,25 @@ mod tests {
         )));
     }
 
+    /// A root tag that never closes keeps the last `<` at offset zero, so
+    /// nothing is ever drained and the probe used to retain the whole part
+    /// while rescanning it per 16 KiB chunk. Under an explicit budget the
+    /// probe stops and answers no; a legal preamble (the 100 KiB comment
+    /// above) still classifies under the default.
+    #[test]
+    fn an_oversized_opening_tag_stops_the_probe_within_its_head_budget() {
+        let doc = format!("<w:document {}", " ".repeat(256 * 1024));
+        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
+        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
+        writer.start_file("word/document.xml", options).unwrap();
+        writer.write_all(doc.as_bytes()).unwrap();
+        let pkg = writer.finish().unwrap().into_inner();
+        assert!(!main_part_is_strict_within(&pkg, 64 * 1024));
+        // Within the default budget the same bytes are read to their end
+        // and decide as they always did: no `>`, so no Strict namespace.
+        assert!(!main_part_is_strict(&pkg));
+    }
+
     /// A Transitional package is returned byte-for-byte unchanged (zero-churn).
     #[test]
     fn transitional_package_returned_byte_identical() {
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..69d4732 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -104,6 +104,7 @@ pub fn parse_xdocument(dom: &mut Dom, xml: &str) -> NodeId {
         dom,
         c: chars,
         pos: 0,
+        depth: 0,
     };
     p.parse_document()
 }
@@ -139,8 +140,20 @@ struct Parser<'a> {
     dom: &'a mut Dom,
     c: Vec<char>,
     pos: usize,
+    /// Construction depth of the element being parsed. Admission counts
+    /// depth with a tolerant scanner that can disagree with this parser
+    /// (`<p a=//>` reads self-closed there, unclosed here, and unclosed
+    /// start tags nest), so the parser bounds its own recursion instead of
+    /// trusting the gate.
+    depth: usize,
 }
 
+/// The deepest element the permissive parser constructs. Real Word parts
+/// nest a few dozen deep; admission's default XML budget is 256. Past this
+/// the rest of an element's content is consumed without nodes rather than
+/// recursing once per remaining tag.
+const MAX_DOM_DEPTH: usize = 1024;
+
 /// In-scope namespace bindings as an overlay stack (N02): one `HashMap`
 /// holds the live prefix → URI bindings, and each element's declarations
 /// push overrides that `pop` restores — where the old port cloned the
@@ -658,7 +671,16 @@ impl<'a> Parser<'a> {
                 continue;
             }
             if self.cur() == '<' {
+                if self.depth >= MAX_DOM_DEPTH {
+                    // Over cap: consume this element's remaining content
+                    // without building nodes. The permissive parser never
+                    // refuses; it just stops growing with the document.
+                    self.skip_deep_content();
+                    break;
+                }
+                self.depth += 1;
                 let child = self.parse_element(scope);
+                self.depth -= 1;
                 self.dom.add(el, child);
                 continue;
             }
@@ -678,6 +700,35 @@ impl<'a> Parser<'a> {
         el
     }
 
+    /// Consume the remainder of an over-depth element's content — nested
+    /// start tags, comments, CDATA, PIs and text — without building nodes,
+    /// stopping at this element's own end tag. Every branch advances, so
+    /// the skip itself cannot loop.
+    fn skip_deep_content(&mut self) {
+        let len = self.len();
+        while self.pos < len {
+            if self.starts_with("</") {
+                let end = self.index_of(">", self.pos).unwrap_or(len);
+                self.pos = (end + 1).min(len);
+                return;
+            }
+            if self.starts_with("<!--") {
+                self.pos = self.index_of("-->", self.pos).map_or(len, |e| e + 3);
+            } else if self.starts_with("<![CDATA[") {
+                self.pos = self.index_of("]]>", self.pos).map_or(len, |e| e + 3);
+            } else if self.starts_with("<?") {
+                self.pos = self.index_of("?>", self.pos).map_or(len, |e| e + 2);
+            } else if self.cur() == '<' {
+                // A start tag: past its `>`. A quoted `>` inside an
+                // attribute value ends the skip early, which only leaves
+                // text unread — never a loop.
+                self.pos = self.index_of(">", self.pos).map_or(len, |e| e + 1);
+            } else {
+                self.pos += 1;
+            }
+        }
+    }
+
     /// Parse a processing instruction (`<?target data?>`) into a `Pi` node.
     /// `self.pos` is on `<?`.
     fn parse_pi_node(&mut self) -> NodeId {
@@ -770,8 +821,18 @@ pub fn unescape_xml_text(s: &str) -> String {
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
+            // A reference body fits a handful of chars (the longest
+            // canonical one is `&#4294967295;`, 12 chars); a `;` past the
+            // window cannot close a canonical reference, and scanning the
+            // whole remaining suffix per `&` made bare ampersand runs
+            // quadratic. A padded numeric body longer than the window
+            // (e.g. `&#0000000000000065;`) now stays literal.
+            const MAX_ENTITY_WINDOW: usize = 16;
+            let window_end = (i + 1 + MAX_ENTITY_WINDOW).min(chars.len());
+            if let Some(semi_rel) = chars[i + 1..window_end]
+                .iter()
+                .position(|&c| c == ';')
+            {
                 let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
@@ -1199,6 +1260,50 @@ mod tag_policy_tests {
         format!("<a{attrs}/>")
     }
 
+    /// A run of bare ampersands: each `&` used to search the whole
+    /// remaining suffix for `;` before deciding there was no entity there,
+    /// quadratic work on plain text. The search now stops past the longest
+    /// reference this decoder accepts, and every decodable reference — and
+    /// every preserved one — comes out exactly as before.
+    #[test]
+    fn bare_ampersand_runs_decode_without_rescanning_the_suffix() {
+        let ampersands = "&".repeat(100_000);
+        assert_eq!(unescape_xml_text(&ampersands), ampersands);
+        let mixed = "a&".repeat(50_000);
+        assert_eq!(unescape_xml_text(&mixed), mixed);
+        assert_eq!(unescape_xml_text("x&amp;y"), "x&y");
+        assert_eq!(unescape_xml_text("&#65;&#x42;"), "AB");
+        assert_eq!(unescape_xml_text("&#x10FFFF;"), "\u{10FFFF}");
+        // Unknown and truncated references stay literal, as they always did.
+        assert_eq!(unescape_xml_text("&nbsp;"), "&nbsp;");
+        assert_eq!(unescape_xml_text("a&b"), "a&b");
+        assert_eq!(unescape_xml_text("a&"), "a&");
+    }
+
+    /// `<p a=//>` reads as a self-closed tag to the tolerant admission
+    /// scanner (unquoted `/` value, `/` before `>`), but as a start tag to
+    /// this parser — and unclosed start tags nest, so a package admission
+    /// saw as shallow built a DOM as deep as its byte count, one stack
+    /// frame per tag. Construction depth is capped whatever the gate saw.
+    #[test]
+    fn a_differential_empty_tag_shape_builds_a_bounded_dom() {
+        let xml = "<p a=//>".repeat(50_000);
+        let mut dom = Dom::new();
+        let doc = parse_xdocument(&mut dom, &xml);
+        let root = dom
+            .nodes(doc)
+            .into_iter()
+            .find(|n| dom.is_element(*n))
+            .expect("root");
+        let mut depth = 0usize;
+        let mut node = root;
+        while let Some(child) = dom.nodes(node).into_iter().find(|n| dom.is_element(*n)) {
+            depth += 1;
+            node = child;
+        }
+        assert!(depth <= MAX_DOM_DEPTH, "nested chain of {depth}");
+    }
+
     #[test]
     fn element_attribute_count_is_capped_at_the_boundary() {
         assert_eq!(
diff --git a/tests/input_admission.rs b/tests/input_admission.rs
index 2f9769e..f1252a5 100644
--- a/tests/input_admission.rs
+++ b/tests/input_admission.rs
@@ -434,6 +434,45 @@ fn document_open_bounds_opc_metadata_attributes() {
     }
 }
 
+/// The pre-index reads the last EOCD-looking signature in the tail; the
+/// zip reader validates candidates and falls back to an earlier footer. A
+/// false EOCD appended past the real one (count 1, central-directory
+/// offset past itself so the reader rejects it) declares 1 entry while the
+/// reader indexes 11 — and the entry budget held only the declared count.
+/// The archive as actually opened stays under the budget.
+#[test]
+fn a_false_eocd_in_the_comment_cannot_hide_the_real_entry_count() {
+    use jubarte::admission::admit;
+
+    let mut bytes = package(|writer, options| {
+        for i in 0..8 {
+            writer.start_file(format!("part{i}.bin"), options).unwrap();
+            writer.write_all(b"x").unwrap();
+        }
+    });
+    let mut fake = Vec::new();
+    fake.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]); // EOCD signature
+    fake.extend_from_slice(&0u16.to_le_bytes()); // disk number
+    fake.extend_from_slice(&0u16.to_le_bytes()); // central-directory disk
+    fake.extend_from_slice(&1u16.to_le_bytes()); // entries on this disk
+    fake.extend_from_slice(&1u16.to_le_bytes()); // total entries: the lie
+    fake.extend_from_slice(&0u32.to_le_bytes()); // central-directory size
+    fake.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // cd offset: past the footer itself
+    fake.extend_from_slice(&0u16.to_le_bytes()); // comment length
+    bytes.extend_from_slice(&fake);
+
+    let limits = InputLimits {
+        max_entries: 10,
+        ..InputLimits::default()
+    };
+    let error = admit(&bytes, limits)
+        .err()
+        .expect("the archive as opened carries 11 entries; the budget is 10");
+    let message = error.to_string();
+    assert!(message.contains("11 ZIP entries"), "{message}");
+    assert!(message.contains("the limit is 10"), "{message}");
+}
+
 /// Package admission admitted unquoted and valueless attributes before it
 /// checked attribute budgets (#1055), and the permissive parser reads them:
 /// Word never writes them, but other writers' files carry them. They are
