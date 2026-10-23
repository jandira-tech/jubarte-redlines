@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_3201e6caaef21850d61fa69c | WMF record-boundary failures amplify polygon work and retained text | medium
csf_dc0d34a6ea9a964733efe547 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_eaffa9e4af9c9c07d800ef1b | ZIP footer and count interpretation mismatch bypasses entry-count admission | medium
csf_68acfaf376a60cf8c3d8e2d4 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_b7534c87815cfbca97ca8543 | Ragged legacy DOC tables expand into an unbounded padded rectangle | medium
csf_eb4256ff45478e77c3906750 | Scanning a document for retained text can overflow the stack | medium
csf_93c93094934792b4e7e4a94c | HTML selector groups multiply CSS values into excessive memory | medium
csf_85faece8ffd45c8ee175c288 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_7084a83c24deb556942e2d43 | WMF record-boundary failures amplify polygon work and retained text | medium
csf_b0aa536248aea0f44dd30d47 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_91956fe2d99a3a2871c06916 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_f756bbe5509c42bdcaa64921 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_88d031991ca3e20e4aae2f24 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_32625277093e7f286a267e2b | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_80edacafa6b67d4864116a38 | WMF record-boundary failures amplify polygon work and retained text | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/src/admission.rs b/src/admission.rs
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -592,11 +592,18 @@
         .rev()
         .find(|&i| bytes.get(i..).is_some_and(|tail| tail.starts_with(&EOCD)))
         .ok_or_else(|| invalid("no end-of-central-directory record"))?;
-    let total = bytes
-        .get(at.saturating_add(10)..at.saturating_add(12))
-        .and_then(|field| <[u8; 2]>::try_from(field).ok())
-        .map(u16::from_le_bytes)
-        .ok_or_else(|| invalid("truncated end-of-central-directory record"))?;
+    // The reader indexes entries by the "on this disk" count (+8) while the
+    // total lives at +10: a footer declaring one entry in total and 65535 on
+    // this disk indexes 65535 entries under the entry budget. The declared
+    // count must cover both fields, whichever is larger.
+    let count_at = |from: usize, to: usize| {
+        bytes
+            .get(at.saturating_add(from)..at.saturating_add(to))
+            .and_then(|field| <[u8; 2]>::try_from(field).ok())
+            .map(u16::from_le_bytes)
+            .ok_or_else(|| invalid("truncated end-of-central-directory record"))
+    };
+    let total = count_at(10, 12)?.max(count_at(8, 10)?);
     if total != u16::MAX {
         return Ok(u64::from(total));
     }
@@ -785,7 +792,11 @@
 /// inside the tag/namespace policy (`xmllinq::parse::TagPolicy`) so no part
 /// reaching the permissive DOM parser can carry pathological attribute or
 /// namespace-declaration widths.
-fn check_xml_depth(name: &str, xml: &[u8], limits: InputLimits) -> Result<(), AdmissionError> {
+pub(crate) fn check_xml_depth(
+    name: &str,
+    xml: &[u8],
+    limits: InputLimits,
+) -> Result<(), AdmissionError> {
     let invalid =
         |why: String| AdmissionError::new(AdmissionErrorKind::InvalidXml, format!("{name}: {why}"));
     let text = part_text(xml);
@@ -1916,5 +1927,24 @@
         assert_eq!(declared_entry_count(&bytes).unwrap(), 7);
     }

+    /// The EOCD carries two counts: entries on this disk (+8), which the
+    /// reader indexes by, and the total (+10), which admission alone read. A
+    /// footer declaring 60000 on this disk and 3 in total indexed 60000
+    /// entries under the entry budget. The declared count must cover both.
+    #[test]
+    fn an_eocd_disk_count_above_the_total_is_the_declared_count() {
+        let mut bytes = docx_with(&[]);
+        let eocd_at = bytes.len() - 22;
+        assert_eq!(&bytes[eocd_at..eocd_at + 4], &[0x50, 0x4b, 0x05, 0x06]);
+        bytes[eocd_at + 8..eocd_at + 10].copy_from_slice(&60_000u16.to_le_bytes());
+        // The total stays 3, the count admission used to read alone.
+        assert_eq!(&bytes[eocd_at + 10..eocd_at + 12], &3u16.to_le_bytes());
+        assert_eq!(declared_entry_count(&bytes).unwrap(), 60_000);
+        assert_eq!(
+            kind(&bytes, InputLimits::default()),
+            AdmissionErrorKind::InputLimit
+        );
+    }
+
     #[test]
     fn a_zip64_locator_offset_past_the_end_is_refused_not_indexed() {
diff --git a/src/scrub.rs b/src/scrub.rs
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -464,9 +464,18 @@
             found.insert(format!("{name} ({error})"));
             break;
         }
         let xml = name.ends_with(".xml") || name.ends_with(".rels");
-        let hit = if xml {
-            xml_holds(&String::from_utf8_lossy(&bytes), needle)
+        let hit = if xml {
+            match xml_holds(&String::from_utf8_lossy(&bytes), needle, limits) {
+                Ok(hit) => hit,
+                // A part the checked reader cannot validate cannot be
+                // certified clean either: report it as a finding, as an
+                // unreadable entry is, so `edit`'s redaction check refuses.
+                Err(error) => {
+                    found.insert(format!("{name} (cannot scan XML: {error})"));
+                    break;
+                }
+            }
         } else {
             contains(&bytes, needle.as_bytes()) || contains(&bytes, &utf16)
         };
@@ -524,51 +533,60 @@
     !digits.is_empty() && digits.chars().all(|c| c.is_ascii_hexdigit())
 }

-fn xml_holds(xml: &str, needle: &str) -> bool {
-    let mut dom = Dom::new();
-    let doc = dom.parse_xdocument(xml);
-    let Some(root) = dom.root(doc) else {
-        // Not XML after all: read it as bytes.
-        return xml.contains(needle);
-    };
-    for node in dom.descendant_nodes(doc) {
-        if dom.is_text(node) || dom.is_comment(node) {
-            if dom.text_value(node).is_some_and(|v| v.contains(needle)) {
-                return true;
-            }
-        } else if dom.is_pi(node) {
-            if dom.pi_data(node).is_some_and(|v| v.contains(needle)) {
-                return true;
-            }
-        } else if dom.is_element(node) {
-            for i in 0..dom.attr_count(node) {
-                let (name, value) = dom.attr_at(node, i);
-                if !value.contains(needle) || dom.is_namespace_declaration(name) {
-                    continue;
-                }
-                if NUMERIC_ATTRIBUTES.contains(&name.local_name()) && is_number(value) {
-                    continue;
-                }
-                return true;
-            }
-        }
-    }
-    // A text split across runs shows only in the paragraph's joined text.
-    let texts = [W::t(), W::del_text(), W::name("instrText")];
-    for p in dom.descendants_and_self(root, Some(&W::p())) {
-        let joined: String = dom
-            .descendants(p, None)
-            .into_iter()
-            .filter(|&n| texts.iter().any(|t| dom.name_is(n, t)))
-            .map(|n| dom.value(n))
-            .collect();
-        if joined.contains(needle) {
-            return true;
-        }
-    }
-    false
-}
+/// Whether `xml` holds `needle`, validated under `limits` before any DOM is
+/// built: the permissive parser's recursion is bounded only by the stack, so
+/// a part past the depth budget comes back as an error the caller reports as
+/// a finding instead of a scan that cannot finish.
+fn xml_holds(
+    xml: &str,
+    needle: &str,
+    limits: crate::admission::InputLimits,
+) -> Result<bool, String> {
+    crate::xmllinq::parse::validate_xml_with_limits(xml, limits)?;
+    let mut dom = Dom::new();
+    let doc = dom.parse_xdocument(xml);
+    let Some(root) = dom.root(doc) else {
+        // Not XML after all: read it as bytes.
+        return Ok(xml.contains(needle));
+    };
+    for node in dom.descendant_nodes(doc) {
+        if dom.is_text(node) || dom.is_comment(node) {
+            if dom.text_value(node).is_some_and(|v| v.contains(needle)) {
+                return Ok(true);
+            }
+        } else if dom.is_pi(node) {
+            if dom.pi_data(node).is_some_and(|v| v.contains(needle)) {
+                return Ok(true);
+            }
+        } else if dom.is_element(node) {
+            for i in 0..dom.attr_count(node) {
+                let (name, value) = dom.attr_at(node, i);
+                if !value.contains(needle) || dom.is_namespace_declaration(name) {
+                    continue;
+                }
+                if NUMERIC_ATTRIBUTES.contains(&name.local_name()) && is_number(value) {
+                    continue;
+                }
+                return Ok(true);
+            }
+        }
+    }
+    // A text split across runs shows only in the paragraph's joined text.
+    let texts = [W::t(), W::del_text(), W::name("instrText")];
+    for p in dom.descendants_and_self(root, Some(&W::p())) {
+        let joined: String = dom
+            .descendants(p, None)
+            .into_iter()
+            .filter(|&n| texts.iter().any(|t| dom.name_is(n, t)))
+            .map(|n| dom.value(n))
+            .collect();
+        if joined.contains(needle) {
+            return Ok(true);
+        }
+    }
+    Ok(false)
+}

 #[cfg(test)]
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod tests {
@@ -627,6 +645,7 @@
         let w = W::URI;
         let size = format!(r#"<w:document xmlns:w="{w}"><w:pgSz w:w="12240"/></w:document>"#);
-        assert!(!xml_holds(&size, "1224"));
+        let limits = crate::admission::InputLimits::compare();
+        assert!(!xml_holds(&size, "1224", limits).unwrap());
         let alt = format!(r#"<w:document xmlns:w="{w}"><w:x w:val="12240"/></w:document>"#);
-        assert!(xml_holds(&alt, "1224"));
+        assert!(xml_holds(&alt, "1224", limits).unwrap());
     }
@@ -634,8 +653,9 @@
     #[test]
     fn element_names_are_no_leak() {
         let w = W::URI;
         let xml = format!(r#"<w:document xmlns:w="{w}"><w:body/></w:document>"#);
-        assert!(!xml_holds(&xml, "body"));
-        assert!(xml_holds(&format!("{xml}<!-- body -->"), "body"));
+        let limits = crate::admission::InputLimits::compare();
+        assert!(!xml_holds(&xml, "body", limits).unwrap());
+        assert!(xml_holds(&format!("{xml}<!-- body -->"), "body", limits).unwrap());
     }
 }
@@ -734,5 +754,26 @@
         assert!(leaks(&bytes, "spacing").is_empty());
         assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
     }
+    #[test]
+    fn an_xml_part_past_the_depth_budget_is_reported_not_dom_parsed() {
+        // 300 nested elements pass the compare budget's 256-element depth
+        // cap to the error path; the permissive parser would build them all
+        // before any needle check, and a deeper part runs the stack out.
+        let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
+        pkg.set_part("customXml/deep.xml", deep.into_bytes());
+        let bytes = pkg.to_zip().unwrap();
+        let found = leaks(&bytes, "absent needle");
+        assert!(
+            found
+                .iter()
+                .any(|f| f.contains("customXml/deep.xml") && f.contains("cannot scan XML")),
+            "{found:?}"
+        );
+        // A shallow part under a custom name still scans clean.
+        let mut shallow = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        shallow.set_part("customXml/shallow.xml", b"<metadata/>".to_vec());
+        assert!(leaks(&shallow.to_zip().unwrap(), "absent needle").is_empty());
+    }
     #[test]
     fn invalid_aliases_are_rejected_before_any_source_property_is_rewritten() {
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -66,5 +66,10 @@
 /// No sector, or no directory entry.
 const NO_STREAM: u32 = 0xFFFF_FFFF;

+/// The most columns a Word table holds: a wider row is not a shape Word can
+/// open, and padding every row of a ragged table to such a width would
+/// multiply the table into a rectangle the file never had.
+const MAX_TABLE_COLUMNS: usize = 63;
+
 /// Why a `.doc` could not be read, formatted as `LEGACY_DOC: …` on the wire.
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -510,6 +515,11 @@
             },
             Block::Table(Table { rows, .. }) => {
-                let width = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
+                let width = rows
+                    .iter()
+                    .map(Vec::len)
+                    .max()
+                    .unwrap_or(0)
+                    .clamp(1, MAX_TABLE_COLUMNS);
                 for (index, row) in rows.iter().enumerate() {
                     out.push('|');
                     for column in 0..width {
@@ -1147,6 +1157,14 @@
 /// corrupt file, refused rather than converted short.
 fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {
     let past = || LegacyDocError::new("a text piece points past the WordDocument stream");
+    // The story lives in the WordDocument stream, one or two bytes per
+    // character: a ccpText longer than the stream is a corrupt count that
+    // aliased pieces would expand into billions of story characters.
+    if u64::from(ccp_text) > word.len() as u64 {
+        return Err(LegacyDocError::new(
+            "the main story is longer than the WordDocument stream",
+        ));
+    }
     // The pieces run from CP 0 through the main story without a gap, or
     // the document would come out quietly shorter than it is.
     // The CPs only ever increase ([MS-DOC] 2.8.35): a piece that runs
@@ -1592,31 +1610,39 @@
         let mut out = Vec::new();
         let mut spans: Vec<Span> = Vec::new();
-        // Field nesting: true while inside a field's code (before its
-        // separator).
-        let mut fields: Vec<bool> = Vec::new();
+        // Field nesting: true while inside a field's code (before its
+        // separator). `open_fields` counts the code frames so the
+        // per-character test stays O(1) however deep the nesting grows.
+        let mut fields: Vec<bool> = Vec::new();
+        let mut open_fields = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
         let mut cell: Vec<String> = Vec::new();

-        for story in chars {
-            // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
-            let text = match story.ch {
-                '\u{13}' => {
-                    fields.push(true);
-                    continue;
-                }
-                '\u{14}' => {
-                    if let Some(top) = fields.last_mut() {
-                        *top = false;
-                    }
-                    continue;
-                }
-                '\u{15}' => {
-                    fields.pop();
-                    continue;
-                }
+        for story in chars {
+            // Inside any field's code, even a nested field's result is code.
+            let in_code = open_fields > 0;
+            let text = match story.ch {
+                '\u{13}' => {
+                    fields.push(true);
+                    open_fields += 1;
+                    continue;
+                }
+                '\u{14}' => {
+                    if let Some(top) = fields.last_mut() {
+                        if *top {
+                            open_fields = open_fields.saturating_sub(1);
+                        }
+                        *top = false;
+                    }
+                    continue;
+                }
+                '\u{15}' => {
+                    if fields.pop() == Some(true) {
+                        open_fields = open_fields.saturating_sub(1);
+                    }
+                    continue;
+                }
                 _ if in_code => continue,
                 '\u{0C}' if self.section_marks.binary_search(&story.cp).is_err() => {
                     // A page break: Markdown has no pages, so a line break.
@@ -1786,4 +1812,21 @@
             "# Fees\n\n| Item | Price |\n|---|---|\n| Setup \\| once |  |\n\n"
         );
     }
+
+    #[test]
+    fn a_table_wider_than_words_column_ceiling_is_trimmed_to_it() {
+        let wide: Vec<String> = (0..100).map(|i| format!("c{i}")).collect();
+        let document = LegacyDocument {
+            blocks: vec![Block::Table(Table {
+                rows: vec![wide],
+                header_rows: 0,
+            })],
+        };
+        let markdown = to_markdown(&document);
+        // Word tables hold at most 63 columns: the separator carries one
+        // `---|` a column, not one a declared cell.
+        assert_eq!(markdown.matches("---|").count(), 63, "{markdown}");
+        assert!(markdown.contains("c62 |"), "{markdown}");
+        assert!(!markdown.contains("c63 |"), "{markdown}");
+    }

@@ -1966,5 +2009,33 @@
             .collect();
         assert_eq!(texts, ["aoneb"]);
     }
+
+    #[test]
+    fn deeply_nested_field_codes_suppress_all_their_content() {
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        // Four thousand fields opened and never closed: everything after
+        // the first begin is field code, as the shallow case reads it.
+        let mut text = String::from("a\u{13}");
+        text.push_str(&"\u{13}".repeat(4_000));
+        text.push_str("hidden\r");
+        let chars = story_chars(&text);
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
+        assert_eq!(texts, ["a"]);
+    }

     /// A supplementary character split across two Unicode pieces is one
@@ -2047,6 +2118,32 @@
         ];
         assert_eq!(main_text(&word, &empty, 10).unwrap().len(), 10);
     }
+
+    #[test]
+    fn a_story_longer_than_the_word_stream_is_refused() {
+        let word = b"HelloWorld".to_vec();
+        let piece = |cp_start, cp_end, fc| Piece {
+            cp_start,
+            cp_end,
+            fc,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        // Five thousand pieces each re-pointing at the same ten bytes: a
+        // fifty-thousand-character logical story from a ten-byte stream.
+        let aliased: Vec<Piece> = (0..5_000u32)
+            .map(|i| piece(i * 10, i * 10 + 10, 0))
+            .collect();
+        let error = main_text(&word, &aliased, 50_000).map(|chars| chars.len());
+        assert!(
+            error
+                .as_ref()
+                .is_err_and(|e| e.to_string().contains("longer than the WordDocument stream")),
+            "{error:?}"
+        );
+        // A story the stream holds still reads.
+        assert_eq!(main_text(&word, &[piece(0, 10, 0)], 10).unwrap().len(), 10);
+    }

     #[test]
     fn a_property_table_outside_its_stream_is_refused_not_dropped() {
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -8,7 +8,14 @@
 //! the chunk lays out like any other content.

 use std::collections::HashMap;
 use std::fmt::Write as _;
+use std::rc::Rc;

+/// The most columns a Word table holds ([MS-DOC] limits a row to 63 cells).
+/// An HTML span past it is not a table Word can import, so the span — and
+/// the grid it would build — is capped at the ceiling instead of allocating
+/// a column per declared span unit.
+const MAX_TABLE_COLUMNS: usize = 63;
+
 /// `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
 /// `load` returns a chunk's bytes by relationship id.
@@ -149,23 +156,36 @@
 type Props = HashMap<String, String>;

-/// `selector { prop: value; … }` rules; a selector list shares its block.
-fn parse_css(css: &str) -> HashMap<String, Props> {
-    let mut rules: HashMap<String, Props> = HashMap::new();
+/// `selector { prop: value; … }` rules; a selector list shares its block —
+/// and, through [`Rc`], one declaration map however long the comma list is:
+/// a group of N selectors costs one map, not N copies of it.
+fn parse_css(css: &str) -> HashMap<String, Rc<Props>> {
+    let mut rules: HashMap<String, Rc<Props>> = HashMap::new();
     for block in css.split('}') {
         let Some((sel, body)) = block.split_once('{') else {
             continue;
         };
         let props: Props = body
             .split(';')
             .filter_map(|d| d.split_once(':'))
             .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
             .collect();
-        for s in sel.split(',') {
-            let s = s.trim().to_ascii_lowercase();
-            if !s.is_empty() {
-                rules.entry(s).or_default().extend(props.clone());
-            }
-        }
+        let mut shared: Option<Rc<Props>> = None;
+        for s in sel.split(',') {
+            let s = s.trim().to_ascii_lowercase();
+            if s.is_empty() {
+                continue;
+            }
+            match rules.get_mut(&s) {
+                // A selector an earlier block already holds merges that
+                // block's declarations over these; `make_mut` copies only
+                // when another selector still shares the earlier map.
+                Some(merged) => Rc::make_mut(merged).extend(props.iter().cloned()),
+                None => {
+                    let map = shared.get_or_insert_with(|| Rc::new(props.clone()));
+                    rules.insert(s, Rc::clone(map));
+                }
+            }
+        }
     }
     rules
 }
@@ -269,7 +289,7 @@
 }

 struct Builder {
-    css: HashMap<String, Props>,
+    css: HashMap<String, Rc<Props>>,
     /// Output targets: the body, then each open table cell.
     sinks: Vec<String>,
     para: Option<Para>,
@@ -298,9 +318,9 @@

     fn props(&self, tag: &str, class: Option<&str>, style: Option<&str>) -> Props {
         let mut p = Props::new();
         let mut take = |key: String| {
             if let Some(r) = self.css.get(&key) {
-                p.extend(r.clone());
+                p.extend(r.iter().cloned());
             }
         };
         take(tag.to_string());
@@ -466,13 +486,17 @@
         let Some(table) = self.tables.pop() else {
             return;
         };
-        let cols = table
-            .rows
-            .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
-            .max()
-            .unwrap_or(1)
-            .max(1);
+        let cols = table
+            .rows
+            .iter()
+            .map(|r| {
+                // Saturating: each span is capped, but a row of many cells
+                // must not overflow the column count either.
+                r.iter().fold(0usize, |sum, c| sum.saturating_add(c.span))
+            })
+            .max()
+            .unwrap_or(1)
+            .clamp(1, MAX_TABLE_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -559,12 +583,12 @@
                 self.close_open_cell();
                 self.flush();
                 self.sinks.push(String::new());
                 self.cell_spans.push(
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .clamp(1, MAX_TABLE_COLUMNS),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
                     fmt.bold = true;
@@ -788,6 +812,42 @@
         );
         assert!(wml.contains(">Title &amp; more</w:t>"), "{wml}");
     }
+
+    #[test]
+    fn a_colspan_past_words_column_ceiling_is_capped_not_allocated() {
+        let wml = html_to_wml("<table><tr><td colspan=\"10000\">x</td></tr></table>");
+        // One grid column a column of the ceiling, not one a declared span
+        // unit: a single attribute value must not size the generated table.
+        assert_eq!(wml.matches("<w:gridCol").count(), 63, "{wml}");
+        assert!(wml.contains("<w:gridSpan w:val=\"63\"/>"), "{wml}");
+        // A span inside the ceiling is kept exactly.
+        let sane = html_to_wml("<table><tr><td colspan=\"3\">x</td></tr></table>");
+        assert_eq!(sane.matches("<w:gridCol").count(), 3, "{sane}");
+        assert!(sane.contains("<w:gridSpan w:val=\"3\"/>"), "{sane}");
+    }
+
+    #[test]
+    fn a_selector_group_shares_one_declaration_map() {
+        let rules = parse_css("p, div, span { color: red }");
+        for selector in ["p", "div", "span"] {
+            assert_eq!(
+                rules.get(selector).map(|props| props.get("color")),
+                Some(&Some("red".to_string())),
+                "{selector}"
+            );
+        }
+        // The sharing the fix exists for: one map per group, not a deep
+        // clone a selector.
+        assert!(Rc::ptr_eq(&rules["p"], &rules["div"]), "one map per group");
+        // A selector an earlier block holds merges the later block over it.
+        let merged = parse_css("p { color: red } p { font-weight: bold }");
+        assert_eq!(
+            merged
+                .get("p")
+                .map(|props| (props.get("color"), props.get("font-weight"))),
+            Some((&Some("red".to_string()), &Some("bold".to_string())))
+        );
+    }

     #[test]
     fn an_alt_chunk_is_replaced_by_its_paragraphs() {
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -600,28 +600,37 @@
                 }
             }
-            // EXTTEXTOUT: y, x, count, options, [rect], string.
-            0x0A32 => {
-                let y = read_i16(data, payload)?;
-                let x = read_i16(data, payload + 2)?;
-                let n = usize::from(read_u16(data, payload + 4)?);
-                let opts = read_u16(data, payload + 6)?;
-                let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
-                if let Some(bytes) = data.get(at..at + n) {
-                    texts.extend(wmf_text(&dc, &map, x, y, bytes));
-                }
-            }
-            // TEXTOUT: count, string (word-padded), y, x.
-            0x0521 => {
-                let n = usize::from(read_u16(data, payload)?);
-                let at = payload + 2 + n.next_multiple_of(2);
-                let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
-                if let Some(bytes) = data.get(payload + 2..payload + 2 + n) {
-                    texts.extend(wmf_text(&dc, &map, x, y, bytes));
-                }
-            }
+            // EXTTEXTOUT: y, x, count, options, [rect], string. The string
+            // ends at this record's declared extent: a count that runs past
+            // it would read and retain the bytes of the records that follow.
+            0x0A32 => {
+                let y = read_i16(data, payload)?;
+                let x = read_i16(data, payload + 2)?;
+                let n = usize::from(read_u16(data, payload + 4)?);
+                let opts = read_u16(data, payload + 6)?;
+                let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
+                let end = (at + n).min(off + size2);
+                if let Some(bytes) = data.get(at..end) {
+                    texts.extend(wmf_text(&dc, &map, x, y, bytes));
+                }
+            }
+            // TEXTOUT: count, string (word-padded), y, x — every field
+            // inside the record's declared extent, or the record is
+            // malformed and writes no text.
+            0x0521 => {
+                let n = usize::from(read_u16(data, payload)?);
+                let at = payload + 2 + n.next_multiple_of(2);
+                if at + 4 <= off + size2 {
+                    let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
+                    let from = payload + 2;
+                    let to = (from + n).min(off + size2);
+                    if let Some(bytes) = data.get(from..to) {
+                        texts.extend(wmf_text(&dc, &map, x, y, bytes));
+                    }
+                }
+            }
             0x01F0 => {
                 let idx = read_u16(data, payload).unwrap_or(0) as usize;
                 if let Some(slot) = objects.get_mut(idx) {
                     *slot = GdiObj::Empty;
                 }
             }
@@ -1167,5 +1176,64 @@
         d.extend_from_slice(&0u16.to_le_bytes()); // func
         d.extend_from_slice(&[0u8; 32]);
         assert!(rasterize(&d).is_some());
     }
+
+    /// A text record whose declared string count runs past its record's
+    /// extent: pre-fix, the text kept the bytes that follow the record; the
+    /// string must be confined to what the record declares.
+    #[test]
+    fn wmf_text_records_stop_at_their_declared_extent() {
+        let mut header = vec![0u8; 40];
+        header[0..4].copy_from_slice(&[0xD7, 0xCD, 0xC6, 0x9A]);
+        header[6..8].copy_from_slice(&0i16.to_le_bytes()); // left
+        header[8..10].copy_from_slice(&0i16.to_le_bytes()); // top
+        header[10..12].copy_from_slice(&64i16.to_le_bytes()); // right
+        header[12..14].copy_from_slice(&64i16.to_le_bytes()); // bottom
+        let record = |func: u16, payload: &[u8]| -> Vec<u8> {
+            let mut d = Vec::new();
+            let words = (6 + payload.len()).div_ceil(2);
+            d.extend_from_slice(&(words as u32).to_le_bytes());
+            d.extend_from_slice(&func.to_le_bytes());
+            d.extend_from_slice(payload);
+            d
+        };
+        // TEXTOUT: count 8, but the record holds only "AB", y and x — the
+        // coordinates the count would place lie outside the record, so the
+        // malformed record writes no text at all.
+        let mut textout = header.clone();
+        let mut payload = Vec::new();
+        payload.extend_from_slice(&8u16.to_le_bytes());
+        payload.extend_from_slice(b"AB");
+        payload.extend_from_slice(&1i16.to_le_bytes());
+        payload.extend_from_slice(&1i16.to_le_bytes());
+        textout.extend(record(0x0521, &payload));
+        textout.extend(b"following records' bytes");
+        let (_, _, _, texts) = render(&textout).expect("raster");
+        assert!(texts.is_empty(), "{texts:?}");
+        // EXTTEXTOUT: y, x, count 8, options 0, then only "AB" — the string
+        // is truncated at the record end, not read out of what follows.
+        let mut exttextout = header.clone();
+        let mut payload = Vec::new();
+        payload.extend_from_slice(&1i16.to_le_bytes()); // y
+        payload.extend_from_slice(&1i16.to_le_bytes()); // x
+        payload.extend_from_slice(&8u16.to_le_bytes()); // count
+        payload.extend_from_slice(&0u16.to_le_bytes()); // options
+        payload.extend_from_slice(b"AB");
+        exttextout.extend(record(0x0A32, &payload));
+        exttextout.extend(b"following records' bytes");
+        let (_, _, _, texts) = render(&exttextout).expect("raster");
+        assert_eq!(texts.len(), 1, "{texts:?}");
+        assert_eq!(texts[0].text, "AB", "{texts:?}");
+        // The well-formed TEXTOUT twin reads "AB" with its coordinates.
+        let mut wellformed = header.clone();
+        let mut payload = Vec::new();
+        payload.extend_from_slice(&2u16.to_le_bytes());
+        payload.extend_from_slice(b"AB");
+        payload.extend_from_slice(&1i16.to_le_bytes());
+        payload.extend_from_slice(&1i16.to_le_bytes());
+        wellformed.extend(record(0x0521, &payload));
+        let (_, _, _, texts) = render(&wellformed).expect("raster");
+        assert_eq!(texts.len(), 1, "{texts:?}");
+        assert_eq!(texts[0].text, "AB", "{texts:?}");
+    }
 }
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
--- a/src/markdown/from_docx/drawing.rs
+++ b/src/markdown/from_docx/drawing.rs
@@ -1404,8 +1404,23 @@
         .or_else(|_| {
             base64::engine::general_purpose::URL_SAFE.decode(encoded.trim_end_matches('='))
         })
-        .map_err(|e| format!("drawing payload is not base64: {e}"))?;
-    serde_json::from_slice(&bytes).map_err(|e| format!("drawing payload is not its JSON: {e}"))
+        .map_err(|e| format!("drawing payload is not base64: {e}"))?;
+    let drawing: Drawing = serde_json::from_slice(&bytes)
+        .map_err(|e| format!("drawing payload is not its JSON: {e}"))?;
+    // The capture side refuses payloads past these caps, so one that exceeds
+    // them was not written by this engine: refuse it at the door instead of
+    // materializing a dependency graph of unbounded depth.
+    if drawing.deps.len() > MAX_DEPS {
+        return Err(format!(
+            "drawing payload holds more than {MAX_DEPS} dependent parts"
+        ));
+    }
+    if drawing.deps.iter().any(|dep| dep.rels.len() > MAX_DEP_RELS) {
+        return Err(format!(
+            "drawing payload holds a part with more than {MAX_DEP_RELS} relationships"
+        ));
+    }
+    Ok(drawing)
 }

 pub(crate) fn encode(drawing: &Drawing) -> String {
@@ -1888,10 +1903,19 @@
 ) -> Option<String> {
     let mut xml = drawing.xml.clone();
+    let mut visiting = vec![false; drawing.deps.len()];
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
-            (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+            (_, Some(index), _) if index < drawing.deps.len() => {
+                let name = dep_part(
+                    drawing,
+                    index,
+                    package,
+                    existing,
+                    used,
+                    next_doc_pr,
+                    &mut visiting,
+                )?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1953,17 +1977,27 @@

 /// Write dependent part `index` and its own dependency closure; its new
 /// part name.
-fn dep_part(
-    drawing: &Drawing,
-    index: usize,
-    package: &mut PartFs,
-    existing: &HashSet<String>,
-    used: &mut HashSet<String>,
-    next_doc_pr: &mut u32,
-) -> Option<String> {
-    let dep = drawing.deps.get(index)?;
+fn dep_part(
+    drawing: &Drawing,
+    index: usize,
+    package: &mut PartFs,
+    existing: &HashSet<String>,
+    used: &mut HashSet<String>,
+    next_doc_pr: &mut u32,
+    visiting: &mut Vec<bool>,
+) -> Option<String> {
+    let dep = drawing.deps.get(index)?;
+    // A dependency cycle (a part whose closure names itself, directly or
+    // through another) has no order to write parts in: refuse the payload
+    // like any other that cannot name its targets, rather than recursing
+    // until the stack ends. `unwrap_or(true)` treats an impossible
+    // out-of-range index as on-path, so the failure stays a refusal.
+    if visiting.get(index).copied().unwrap_or(true) {
+        return None;
+    }
+    visiting[index] = true;
     let name = name_for(&dep.name, existing, used);
     used.insert(name.clone());
     // The part's own relationships name the parts the recursion is writing;
     // their ids land in a rewrite of its `d{index}.{n}` symbols.
     let mut xml = dep.xml.clone();
@@ -1971,7 +2005,8 @@
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name =
+                    dep_part(drawing, inner, package, existing, used, next_doc_pr, visiting)?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -1995,16 +2030,18 @@
     // Two payloads may depend on the one part (two drawings share an
     // image): a part already written byte for byte is the same one, not
     // a collision to rename around.
-    if package
-        .part_bytes(&dep.name)
-        .is_some_and(|written| written == bytes.as_slice())
-    {
-        used.insert(dep.name.clone());
-        return Some(dep.name.clone());
-    }
-    package.set_part(&name, bytes);
-    package.add_content_type_override(&format!("/{name}"), &dep.kind);
-    Some(name)
+    if package
+        .part_bytes(&dep.name)
+        .is_some_and(|written| written == bytes.as_slice())
+    {
+        used.insert(dep.name.clone());
+        visiting[index] = false;
+        return Some(dep.name.clone());
+    }
+    package.set_part(&name, bytes);
+    package.add_content_type_override(&format!("/{name}"), &dep.kind);
+    visiting[index] = false;
+    Some(name)
 }

 /// `want` when nothing holds it, else `want` with `-j2`, `-j3`, … before
@@ -2096,6 +2133,74 @@
         assert!(at("![a][other]").is_none());
         assert!(at("![a][d1").is_none());
     }
+
+    #[test]
+    fn a_cyclic_dependency_closure_is_refused_not_recursed() {
+        // Dep 0's relationship targets dep 0: materializing the closure
+        // would walk it forever.
+        // The entry relationship reaches dep 0, whose own relationship
+        // targets dep 0 again: materializing the closure would walk it
+        // forever.
+        let drawing = Drawing {
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: "image".into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![Dep {
+                name: "word/media/cycle.xml".into(),
+                kind: "application/xml".into(),
+                xml: "<cycle/>".into(),
+                rels: vec![PayRel {
+                    kind: "image".into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                }],
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        let mut package = PartFs::open(include_bytes!(
+            "../../../tests/fixtures/word_probes/tokens/cell_a.docx"
+        ))
+        .unwrap();
+        let existing = HashSet::new();
+        let mut used = HashSet::new();
+        let mut next_doc_pr = 0;
+        assert!(materialize(
+            &drawing,
+            &mut package,
+            "word/document.xml",
+            &existing,
+            &mut used,
+            &mut next_doc_pr
+        )
+        .is_none());
+    }
+
+    #[test]
+    fn a_payload_past_the_dependency_caps_is_refused_at_decode() {
+        let big = Drawing {
+            deps: (0..=MAX_DEPS)
+                .map(|i| Dep {
+                    name: format!("word/media/p{i}.bin"),
+                    ..Dep::default()
+                })
+                .collect(),
+            ..Drawing::default()
+        };
+        assert!(decode(&format!("drawing:{}", encode(&big))).is_err());
+        // A payload within the caps still decodes.
+        let small = Drawing {
+            deps: vec![Dep {
+                name: "word/media/p.bin".into(),
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        assert!(decode(&format!("drawing:{}", encode(&small))).is_ok());
+    }

     #[test]
     fn unchanged_blank_box_keeps_controls_and_remains_editable() {
diff --git a/src/revision_processor.rs b/src/revision_processor.rs
--- a/src/revision_processor.rs
+++ b/src/revision_processor.rs
@@ -3728,6 +3728,29 @@
     }
     out
 }
+
+/// The relationship-selected revision bearers and the numbering part,
+/// checked under the XML policy admission gives `.xml`-named parts: the
+/// processor picks headers, footers, notes, comments, styles and numbering
+/// by relationship type, so a renamed `notes.bin` reaches the recursive DOM
+/// parser without the suffix-based depth check. Refused here exactly as its
+/// `.xml`-named twin is refused at admission.
+pub(crate) fn check_revision_parts(
+    pkg: &crate::opc::PartFs,
+    limits: crate::admission::InputLimits,
+) -> Result<(), crate::opc::OpcError> {
+    for part in revision_bearing_parts(pkg)
+        .into_iter()
+        .map(|(part, _)| part)
+        .chain(numbering_part(pkg))
+    {
+        if let Some(bytes) = pkg.part_bytes(&part) {
+            crate::admission::check_xml_depth(&part, bytes, limits)
+                .map_err(crate::opc::refused)?;
+        }
+    }
+    Ok(())
+}

 fn process_part<F>(pkg: &mut crate::opc::PartFs, part: &str, f: F)
 where
@@ -4816,7 +4839,61 @@
         );
         pkg.set_part(MAIN,format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{content}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI,M::URI).into_bytes());
         pkg.to_zip().unwrap()
     }
+
+    /// A footnotes relationship may target a part with any name; the XML
+    /// policy that admits `notes.xml` must refuse a renamed twin whose
+    /// nesting passes the depth budget, or revision processing parses it
+    /// with a recursion only the stack bounds.
+    #[test]
+    fn a_renamed_deep_notes_part_is_refused_like_notes_xml() {
+        let deep = format!(
+            "{}{}",
+            "<w:footnote>".repeat(300),
+            "</w:footnote>".repeat(300)
+        );
+        let footnotes =
+            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes";
+        // The `.xml`-named twin is refused by admission already.
+        let bytes = package("<w:p/>");
+        let mut pkg = PartFs::open(&bytes).unwrap();
+        pkg.set_part(
+            "word/notes.xml",
+            format!("<w:footnotes xmlns:w='{}'>{deep}</w:footnotes>", W::URI).into_bytes(),
+        );
+        pkg.add_document_relationship(MAIN, footnotes, "notes.xml");
+        let err = crate::document_comparer::accept_revisions(&pkg.to_zip().unwrap())
+            .expect_err("admission refuses the .xml twin");
+        assert!(err.to_string().contains("nests deeper"), "{err}");
+        // The renamed twin gets the same refusal at the processor's door.
+        let bytes = package("<w:p/>");
+        let mut pkg = PartFs::open(&bytes).unwrap();
+        pkg.set_part(
+            "word/notes.bin",
+            format!("<w:footnotes xmlns:w='{}'>{deep}</w:footnotes>", W::URI).into_bytes(),
+        );
+        pkg.add_document_relationship(MAIN, footnotes, "notes.bin");
+        let err = crate::document_comparer::accept_revisions(&pkg.to_zip().unwrap())
+            .expect_err("the renamed twin gets the same refusal");
+        let text = err.to_string();
+        assert!(
+            text.contains("INVALID_XML")
+                && text.contains("notes.bin")
+                && text.contains("nests deeper"),
+            "{text}"
+        );
+        // A shallow renamed notes part still processes: valid arbitrary
+        // part names remain supported.
+        let bytes = package("<w:p/>");
+        let mut pkg = PartFs::open(&bytes).unwrap();
+        pkg.set_part(
+            "word/notes.bin",
+            format!("<w:footnotes xmlns:w='{}'><w:footnote/></w:footnotes>", W::URI).into_bytes(),
+        );
+        pkg.add_document_relationship(MAIN, footnotes, "notes.bin");
+        let accepted = crate::document_comparer::accept_revisions(&pkg.to_zip().unwrap());
+        assert!(accepted.is_ok(), "{accepted:?}");
+    }
     fn resolved(bytes: &[u8], reject: bool) -> (Dom, NodeId) {
         let out = if reject {
             crate::document_comparer::reject_revisions(bytes).unwrap()
diff --git a/src/document_comparer.rs b/src/document_comparer.rs
--- a/src/document_comparer.rs
+++ b/src/document_comparer.rs
@@ -4961,6 +4961,10 @@
 pub fn accept_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError> {
     admit_package(docx)?;
     let mut pkg = PartFs::open(docx)?;
+    crate::revision_processor::check_revision_parts(
+        &pkg,
+        crate::admission::InputLimits::compare(),
+    )?;
     crate::revision_processor::accept_revisions_package(&mut pkg);
     pkg.to_zip()
 }
@@ -4974,6 +4978,7 @@
     limits: crate::admission::InputLimits,
 ) -> Result<Vec<u8>, OpcError> {
     let mut pkg = PartFs::open_within(docx, limits)?;
+    crate::revision_processor::check_revision_parts(&pkg, limits)?;
     crate::revision_processor::accept_revisions_package_keeping_ids(&mut pkg);
     pkg.to_zip()
 }
@@ -4984,6 +4989,10 @@
 pub fn reject_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError> {
     admit_package(docx)?;
     let mut pkg = PartFs::open(docx)?;
+    crate::revision_processor::check_revision_parts(
+        &pkg,
+        crate::admission::InputLimits::compare(),
+    )?;
     crate::revision_processor::reject_revisions_package(&mut pkg);
     pkg.to_zip()
 }

```
