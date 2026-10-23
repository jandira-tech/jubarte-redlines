@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_49256aa9e91ee0159c917b13 | Rendering an admitted DOCX can retain gigabytes of duplicate embedded fonts | medium
csf_cac980ab6041d4ec91b39fd1 | Repeated relationship IDs trigger quadratic repair work | medium
csf_6a40bc41bdf34a737ceac40d | Entity decoding repeatedly scans the input suffix | medium
csf_d1ff97a577444c45ea8ba899 | Legacy DOC placeholder selection repeatedly rescans predictable document text | medium
csf_918c9b892d0d08ee23454ad8 | Report text extraction quadratically groups baselines on tall pages | medium
csf_88d031991ca3e20e4aae2f24 | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_32625277093e7f286a267e2b | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_eb94605aebd8f08907c46e8c | Entity decoding repeatedly scans the input suffix | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda2..a7da279b 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -90,6 +90,23 @@ pub(crate) use archive::{
 pub use zip32::Zip32Limit;
 pub(crate) use zip32::zip32_plan;
 
+// Duplicate-repair mechanism counter (tests only): one per duplicate-Id
+// membership probe. Per thread, so tests running side by side in one
+// `cargo test` process do not count each other's work. The reported list
+// must stay near-linear in the number of duplicates: probing a growing
+// Vec instead of this set is the quadratic this counter watches for.
+#[cfg(test)]
+thread_local! {
+    static DUP_MEMBERSHIP_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
+}
+
+/// Add one to this thread's [`DUP_MEMBERSHIP_PROBES`].
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+fn bump_dup_membership_probes() {
+    DUP_MEMBERSHIP_PROBES.with(|c| c.set(c.get() + 1));
+}
+
 /// Exactly one leading slash: callers build `/{part}` from names that may
 /// already be absolute, and a `//word/…` part name is invalid OPC.
 /// Keeps one of each relationship `rels` repeats identically (same Id,
@@ -98,6 +115,10 @@ pub(crate) use zip32::zip32_plan;
 /// only the first is a repair that cannot guess wrong.
 fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>, OpcError> {
     let mut seen: HashMap<&str, &Relationship> = HashMap::new();
+    // Reported duplicates keep first-seen order (the Vec) while membership
+    // asks the set: `repaired.contains` made n distinct Ids each repeated
+    // cost n²/2 string comparisons on metadata that fits every budget.
+    let mut reported: HashSet<&str> = HashSet::new();
     let mut repaired: Vec<String> = Vec::new();
     let mut keep = Vec::with_capacity(rels.items.len());
     for item in &rels.items {
@@ -113,7 +134,9 @@ fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>,
                 {
                     return Err(OpcError::InvalidRelationship);
                 }
-                if !repaired.contains(&item.id) {
+                #[cfg(test)]
+                bump_dup_membership_probes();
+                if reported.insert(item.id.as_str()) {
                     repaired.push(item.id.clone());
                 }
                 keep.push(false);
@@ -675,10 +698,14 @@ impl PartFs {
     /// absolute target ("/word/footer1.xml") was later re-prefixed into a
     /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
     pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
-        // PartFs receives raw relationship attributes; DOM consumers already
-        // decoded XML entities and call the pure resolver directly.
-        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
-        resolve_rel_target(source_part, &target)
+        // Relationship targets arrive already entity-decoded: oxml-opc 0.14.2
+        // reads each attribute through quick-xml 0.41's
+        // `decoded_and_normalized_value`, which resolves the predefined
+        // entities. A second unescape here corrupted a literal `&amp;` in a
+        // part name (`Target="a&amp;amp;b.xml"` resolved to `a&b.xml`, a part
+        // that does not exist) and rescanned ampersand-heavy targets. Holders
+        // of raw XML attribute text decode it once themselves before calling.
+        resolve_rel_target(source_part, rel_target)
     }
 
     /// `contentTypeFor(part)`.
@@ -1100,6 +1127,81 @@ mod tests {
         );
     }
 
+    #[test]
+    fn many_distinct_duplicate_pairs_repair_once_each_with_linear_membership_work() {
+        // n distinct Ids each repeated identically used to cost n²/2
+        // membership comparisons (`repaired.contains` over a growing Vec);
+        // the probe counter watches the set-probed repair stays linear.
+        let n = 4096;
+        let mut rels = String::new();
+        for i in 0..n {
+            rels.push_str(&format!(
+                r#"<Relationship Id="rId{i}" Type="{HYPERLINK}" Target="https://a.example/{i}"/>"#
+            ));
+        }
+        for i in 0..n {
+            rels.push_str(&format!(
+                r#"<Relationship Id="rId{i}" Type="{HYPERLINK}" Target="https://a.example/{i}"/>"#
+            ));
+        }
+        let bytes = package_with_document_rels(&rels);
+        let before = DUP_MEMBERSHIP_PROBES.with(|c| c.get());
+        let fs = PartFs::open(&bytes).expect("duplicates repair");
+        let probes = DUP_MEMBERSHIP_PROBES.with(|c| c.get()) - before;
+        assert_eq!(fs.repaired_duplicate_ids().len(), n);
+        assert!(
+            probes <= n,
+            "one membership probe a duplicate: {probes} probes for {n} duplicates"
+        );
+        // First-seen reporting order is deterministic: rId0, rId1, …
+        let reported: Vec<&str> = fs
+            .repaired_duplicate_ids()
+            .iter()
+            .map(|(_, id)| id.as_str())
+            .collect();
+        assert_eq!(reported.first(), Some(&"rId0"));
+        let last = format!("rId{}", n - 1);
+        assert_eq!(reported.last(), Some(&last.as_str()));
+    }
+
+    #[test]
+    fn an_escaped_ampersand_target_decodes_exactly_once() {
+        // oxml-opc hands over decoded attribute values, so a part literally
+        // named `a&amp;b.xml` is written `Target="a&amp;amp;b.xml"`. The old
+        // second unescape resolved it to `a&b.xml`, a part that does not
+        // exist; resolution must decode exactly once and find the part.
+        let mut buf = Vec::new();
+        {
+            let mut z = ZipWriter::new(Cursor::new(&mut buf));
+            let opt = SimpleFileOptions::default();
+            let parts: [(&str, String); 4] = [
+                ("[Content_Types].xml", r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/></Types>"#.to_string()),
+                ("_rels/.rels".to_string(), format!(
+                    r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#
+                )),
+                ("word/document.xml".to_string(), "<w:document/>".to_string()),
+                ("word/_rels/document.xml.rels".to_string(), format!(
+                    r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rIdA" Type="{HYPERLINK}" Target="a&amp;amp;b.xml"/></Relationships>"#
+                )),
+            ];
+            for (name, xml) in parts {
+                z.start_file(name, opt).unwrap();
+                z.write_all(xml.as_bytes()).unwrap();
+            }
+            // The part name itself carries the literal five characters `&amp;`.
+            z.start_file("word/a&amp;b.xml", opt).unwrap();
+            z.write_all(b"<w:p/>").unwrap();
+            z.finish().unwrap();
+        }
+        let fs = PartFs::open(&buf).expect("package with a literal &amp; part");
+        let resolved = fs.resolve_rel_target("word/document.xml", "a&amp;b.xml");
+        assert_eq!(resolved, "word/a&amp;b.xml");
+        assert!(
+            fs.part_bytes(&resolved).is_some(),
+            "the once-decoded target names a real part"
+        );
+    }
+
     #[test]
     fn the_package_is_written_in_the_source_order_then_new_parts_by_name() {
         // Parts and relationships live in hash maps, so the zip entries came
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053a..b0ea0e77 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -49,8 +49,26 @@
 //! the structured [`LegacyDocument`]; [`doc_to_markdown`] and
 //! [`doc_to_docx`] for the two written outputs.
 
+use std::collections::HashSet;
 use std::fmt;
 
+// Placeholder-selection mechanism counter (tests only): one per text block
+// the candidate scan walks. Per thread, so tests running side by side in
+// one `cargo test` process do not count each other's work. Selection must
+// stay one pass over the text: a candidate-by-candidate `contains` loop is
+// the (candidates × text) rescan this counter watches for.
+#[cfg(test)]
+thread_local! {
+    static PLACEHOLDER_SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
+}
+
+/// Add one to this thread's [`PLACEHOLDER_SCANS`].
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+fn bump_placeholder_scans() {
+    PLACEHOLDER_SCANS.with(|c| c.set(c.get() + 1));
+}
+
 /// The first bytes of an OLE compound file.
 const OLE_MAGIC: &[u8] = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1";
 
@@ -240,7 +258,22 @@ pub fn doc_to_docx(bytes: &[u8]) -> Result<Vec<u8>> {
 /// Markdown cannot say put back: Title and Heading 7-9, empty paragraphs,
 /// and which table rows repeat as headers.
 fn document_to_docx(document: &LegacyDocument) -> Result<Vec<u8>> {
-    let placeholder = empty_placeholder(document);
+    let empties: usize = document
+        .blocks
+        .iter()
+        .filter_map(|block| match block {
+            Block::Paragraph(p) if p.text().trim().is_empty() => Some(placeholder_lines(p)),
+            _ => None,
+        })
+        .sum();
+    // No empty paragraph, no placeholder to look for: a document without
+    // one skips the text scan entirely (`blocks_to_markdown` only uses the
+    // placeholder for empty paragraphs).
+    let placeholder = if empties == 0 {
+        String::new()
+    } else {
+        empty_placeholder(document)
+    };
     let markdown = blocks_to_markdown(document, Some(&placeholder));
     let options = crate::markdown::DocxOptions {
         critic: false,
@@ -264,14 +297,6 @@ fn document_to_docx(document: &LegacyDocument) -> Result<Vec<u8>> {
     let restyle = styles
         .iter()
         .any(|id| matches!(id.as_str(), "Title" | "Heading7" | "Heading8" | "Heading9"));
-    let empties: usize = document
-        .blocks
-        .iter()
-        .filter_map(|block| match block {
-            Block::Paragraph(p) if p.text().trim().is_empty() => Some(placeholder_lines(p)),
-            _ => None,
-        })
-        .sum();
     let header_rows: Vec<usize> = document
         .blocks
         .iter()
@@ -309,7 +334,17 @@ fn docx_error(what: &str) -> LegacyDocError {
 /// A word that stands in for an empty paragraph's text on its way through
 /// the Markdown writer: letters and digits only (nothing to escape, and the
 /// writer drops private-use characters), and in none of the document's text.
+///
+/// One pass collects every digit run that follows the candidate prefix; a
+/// number is occupied exactly when its digits begin one of those runs, which
+/// is the same absence the old candidate-by-candidate `contains` established
+/// without rescanning the whole document per candidate (a document holding
+/// `jubarteemptyparagraph0…99999` forced a hundred thousand full scans).
 fn empty_placeholder(document: &LegacyDocument) -> String {
+    const PREFIX: &str = "jubarteemptyparagraph";
+    // A candidate's number is at most `u32::MAX`'s ten digits, so no digit
+    // past the first ten after the prefix can occupy one.
+    const DIGITS: usize = 10;
     let texts: Vec<String> = document
         .blocks
         .iter()
@@ -318,9 +353,35 @@ fn empty_placeholder(document: &LegacyDocument) -> String {
             Block::Table(table) => table.rows.iter().flatten().cloned().collect(),
         })
         .collect();
+    let mut occupied: HashSet<&str> = HashSet::new();
+    for text in &texts {
+        #[cfg(test)]
+        bump_placeholder_scans();
+        let mut rest = text.as_str();
+        while let Some(at) = rest.find(PREFIX) {
+            // `PREFIX` has no border (first 'j', last 'h'), so no second
+            // occurrence can start inside this one: continuing after it
+            // misses nothing.
+            let after = rest
+                .get(at.saturating_add(PREFIX.len())..)
+                .unwrap_or_default();
+            for (index, c) in after.char_indices().take(DIGITS) {
+                if !c.is_ascii_digit() {
+                    break;
+                }
+                if let Some(number) = after.get(..=index) {
+                    occupied.insert(number);
+                }
+            }
+            rest = after;
+        }
+    }
     (0u32..)
-        .map(|n| format!("jubarteemptyparagraph{n}"))
-        .find(|candidate| texts.iter().all(|text| !text.contains(candidate.as_str())))
+        .map(|n| format!("{PREFIX}{n}"))
+        .find(|candidate| {
+            let number = candidate.get(PREFIX.len()..).unwrap_or_default();
+            !occupied.contains(number)
+        })
         .unwrap_or_default()
 }
 
@@ -1591,9 +1652,15 @@ impl Story<'_> {
     fn blocks(&self, chars: &[StoryChar]) -> Vec<Block> {
         let mut out = Vec::new();
         let mut spans: Vec<Span> = Vec::new();
-        // Field nesting: true while inside a field's code (before its
-        // separator).
+        // Field nesting: `fields` keeps one bool a nesting level (true while
+        // inside a field's code, before its separator) so a separator or end
+        // knows its own frame's state, and `code_frames` counts the true
+        // ones: the per-character "am I inside any code?" reads the count
+        // instead of rescanning the whole stack (a story with a hundred
+        // thousand nested field results used to pay that stack once per
+        // character).
         let mut fields: Vec<bool> = Vec::new();
+        let mut code_frames: usize = 0;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1668,26 @@ impl Story<'_> {
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = code_frames > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    code_frames = code_frames.saturating_add(1);
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
+                        if *top {
+                            code_frames = code_frames.saturating_sub(1);
+                        }
                         *top = false;
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        code_frames = code_frames.saturating_sub(1);
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +2040,161 @@ mod tests {
         assert_eq!(texts, ["aoneb"]);
     }
 
+    #[test]
+    fn deeply_nested_field_results_read_without_rescanning_the_field_stack() {
+        // Ten thousand begin/separator pairs leave an all-result stack; the
+        // visible text after them is read with a code-frame count, not one
+        // stack scan a character (which made this input pay hundreds of
+        // millions of boolean inspections).
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let mut text = String::from("a");
+        for _ in 0..10_000 {
+            text.push('\u{13}');
+            text.push('\u{14}');
+        }
+        text.push('R');
+        for _ in 0..10_000 {
+            text.push('\u{15}');
+        }
+        text.push_str("b\r");
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
+        assert_eq!(texts, ["aRb"]);
+    }
+
+    #[test]
+    fn field_marker_edge_shapes_keep_their_visible_text() {
+        // A repeated separator, an unmatched end, a separator with no
+        // begin, and a code ancestor hiding a nested result: the frame
+        // count must answer every one exactly as the stack scan did.
+        for (text, expected) in [
+            ("a\u{13}C\u{14}R\u{14}R2\u{15}b\r", "aRR2b"),
+            ("a\u{15}b\r", "ab"),
+            ("a\u{14}b\r", "ab"),
+            (
+                "a\u{13}code\u{13}\u{14}hidden\u{15}\u{14}shown\u{15}b\r",
+                "ashownb",
+            ),
+        ] {
+            let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+            let lists = Lists::default();
+            let story = Story {
+                papx: &papx,
+                chpx: &chpx,
+                styles: &[],
+                lists: &lists,
+                section_marks: Vec::new(),
+            };
+            let texts: Vec<String> = story
+                .blocks(&story_chars(text))
+                .iter()
+                .map(|b| match b {
+                    Block::Paragraph(p) => p.text(),
+                    Block::Table(_) => "table".into(),
+                })
+                .collect();
+            assert_eq!(texts, [expected], "{text:?}");
+        }
+    }
+
+    #[test]
+    fn empty_placeholder_skips_exactly_the_occupied_numbers() {
+        let doc = |text: &str| LegacyDocument {
+            blocks: vec![Block::Paragraph(plain(text))],
+        };
+        assert_eq!(
+            empty_placeholder(&doc("plain text")),
+            "jubarteemptyparagraph0"
+        );
+        // 0 is occupied; 1 and 12 begin the digit run of "12xyz"; 2 is the
+        // first free number — the same candidate the whole-text `contains`
+        // loop chose.
+        assert_eq!(
+            empty_placeholder(&doc(
+                "x jubarteemptyparagraph0 y jubarteemptyparagraph12xyz"
+            )),
+            "jubarteemptyparagraph2"
+        );
+        // A leading zero occupies only 0: "1" is not "01".
+        assert_eq!(
+            empty_placeholder(&doc("x jubarteemptyparagraph012 y")),
+            "jubarteemptyparagraph1"
+        );
+        // The prefix alone, or followed by no digit, occupies nothing.
+        assert_eq!(
+            empty_placeholder(&doc("jubarteemptyparagraph and jubarteemptyparagraphx")),
+            "jubarteemptyparagraph0"
+        );
+    }
+
+    #[test]
+    fn sequential_occupied_candidates_select_with_one_scan_a_text_block() {
+        // A document holding the sequential candidates 0..2000 selects the
+        // next free number in one pass over its text blocks, not one full
+        // scan a rejected candidate (the old loop paid candidates × text).
+        let mut text = String::new();
+        for n in 0..2_000 {
+            text.push_str(&format!("jubarteemptyparagraph{n} "));
+        }
+        let document = LegacyDocument {
+            blocks: vec![Block::Paragraph(plain(&text)), Block::Paragraph(plain(""))],
+        };
+        let before = PLACEHOLDER_SCANS.with(|c| c.get());
+        let placeholder = empty_placeholder(&document);
+        let scans = PLACEHOLDER_SCANS.with(|c| c.get()) - before;
+        assert_eq!(placeholder, "jubarteemptyparagraph2000");
+        assert!(
+            scans <= document.blocks.len(),
+            "one pass a text block: {scans} scans for {} blocks",
+            document.blocks.len()
+        );
+    }
+
+    #[test]
+    fn a_document_without_empty_paragraphs_skips_placeholder_selection() {
+        let document = LegacyDocument {
+            blocks: vec![Block::Paragraph(plain("text only"))],
+        };
+        let before = PLACEHOLDER_SCANS.with(|c| c.get());
+        let docx = document_to_docx(&document).expect("docx");
+        let scans = PLACEHOLDER_SCANS.with(|c| c.get()) - before;
+        assert_eq!(scans, 0, "no empty paragraph, no candidate scan");
+        assert!(!docx.is_empty());
+    }
+
+    #[test]
+    fn a_document_with_empty_paragraphs_still_converts_and_cleans_placeholders() {
+        let document = LegacyDocument {
+            blocks: vec![
+                Block::Paragraph(plain("around")),
+                Block::Paragraph(plain("")),
+            ],
+        };
+        let docx = document_to_docx(&document).expect("docx");
+        let pkg = crate::opc::PartFs::open(&docx).expect("reopen the written package");
+        let body = pkg.part_string("word/document.xml").expect("document body");
+        assert!(
+            !body.contains("jubarteemptyparagraph"),
+            "the placeholder never survives into the package: {body}"
+        );
+        assert!(body.contains("around"), "{body}");
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f98..19209060 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -291,6 +291,14 @@ const BLOCKS: [&str; 10] = [
     "blockquote",
 ];
 
+/// Word's table column limit: a row holds at most 63 cells, so a wider
+/// HTML table (one huge `colspan`, or cell spans summing past 63) is not a
+/// table Word can represent at all. The per-cell span and the generated
+/// `gridCol` count are capped here, before any markup is built: a
+/// `colspan="4294967295"` in a few hundred bytes used to demand a
+/// 4-billion-iteration expansion before any parser or budget saw it.
+const MAX_TABLE_COLUMNS: usize = 63;
+
 impl Builder {
     fn fmt(&self) -> Fmt {
         self.fmts.last().map(|(_, f)| f.clone()).unwrap_or_default()
@@ -472,7 +480,8 @@ impl Builder {
             .map(|r| r.iter().map(|c| c.span).sum::<usize>())
             .max()
             .unwrap_or(1)
-            .max(1);
+            .max(1)
+            .min(MAX_TABLE_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -563,7 +572,8 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .max(1)
+                        .min(MAX_TABLE_COLUMNS),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -730,7 +740,18 @@ pub(crate) fn decode_entities(s: &str) -> String {
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // The longest supported reference body is ten characters
+        // (`#x` + eight hex digits covers `&#x1F433;`), so the `;` is
+        // searched for inside that window only: `find` used to drag the
+        // whole remaining suffix into every unrecognized `&`'s scan
+        // before the length filter rejected it. `char_indices` walks
+        // boundaries, so a multibyte character inside the window cannot
+        // split the search.
+        let semi = tail
+            .char_indices()
+            .take_while(|(i, _)| *i <= 10)
+            .find(|&(_, c)| c == ';')
+            .map(|(i, _)| i);
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -1054,6 +1075,53 @@ mod tests {
                 assert!(!wml.contains("<w:tblBorders>"));
             }
         }
+
+        #[test]
+        fn a_colspan_past_words_table_limit_caps_at_63_columns() {
+            // Word holds at most 63 columns in a table row; one huge or
+            // summed-past-63 span is not a table Word can represent. The
+            // generated grid is capped before any markup is built, instead
+            // of expanding one gridCol per declared column (a
+            // `colspan="4294967295"` used to demand 4 billion appends).
+            for span in ["64", "1000", "1000000000", "4294967295"] {
+                let wml = html_to_wml(&format!(
+                    "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
+                ));
+                assert_eq!(wml.matches("<w:gridCol ").count(), 63, "{span}: {wml}");
+                assert!(wml.len() < 10_000, "{span}: {wml.len()} bytes");
+            }
+            // Cell spans that are individually valid but sum past the limit
+            // generate a bounded grid the same way.
+            let wml = html_to_wml(
+                "<table border='0'><tr>\
+                 <td colspan='30'>A</td><td colspan='30'>B</td><td colspan='30'>C</td>\
+                 </tr></table>",
+            );
+            assert_eq!(wml.matches("<w:gridCol ").count(), 63, "{wml}");
+            // A span at the inclusive limit keeps today's merged-cell shape.
+            let wml = html_to_wml("<table border='0'><tr><td colspan='63'>A</td></tr></table>");
+            assert_eq!(wml.matches("<w:gridCol ").count(), 63, "{wml}");
+            assert!(wml.contains("<w:gridSpan w:val=\"63\"/>"), "{wml}");
+        }
+
+        #[test]
+        fn a_multibyte_character_never_splits_the_entity_window() {
+            // The `;` search is bounded to the ten-byte supported-entity
+            // window, but a `;` inside it is still found when a multibyte
+            // character straddles the window's edge, and a bare `&` with no
+            // `;` in the window stays literal.
+            assert_eq!(decode_entities("&amp;\u{1F600}"), "&\u{1F600}");
+            assert_eq!(
+                decode_entities("&nbsp;aaaa\u{1F600}"),
+                "\u{a0}aaaa\u{1F600}",
+                "the emoji starts at byte 10, inside the edge the window walks"
+            );
+            assert_eq!(
+                decode_entities("a&ampb&ampc&amp&amp"),
+                "a&ampb&ampc&amp&amp"
+            );
+            assert_eq!(decode_entities("&#x1F433;"), "\u{1F433}");
+        }
     }
 }
 
diff --git a/src/convert/raster.rs b/src/convert/raster.rs
index 8286bb30..bc6d64cc 100644
--- a/src/convert/raster.rs
+++ b/src/convert/raster.rs
@@ -21,6 +21,23 @@ use super::pdf::{Op, Page, markup_chrome};
 /// Points per inch.
 const PT_PER_INCH: f32 = 72.0;
 
+// Baseline-grouping mechanism counter (tests only): one per text op's
+// baseline lookup. Per thread, so tests running side by side in one
+// `cargo test` process do not count each other's work. The indexed lookup
+// must stay near-linear in the op count: a scan of the existing baselines
+// per op is the (ops × baselines) blow-up this counter watches for.
+#[cfg(test)]
+thread_local! {
+    static BASELINE_LOOKUPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
+}
+
+/// Add one to this thread's [`BASELINE_LOOKUPS`].
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+fn bump_baseline_lookups() {
+    BASELINE_LOOKUPS.with(|c| c.set(c.get() + 1));
+}
+
 /// Largest page raster, in pixels (1 GiB of RGBA): Letter and A4 fit at
 /// 1200 dpi. A failed allocation would abort the process, Python host included.
 const MAX_PAGE_PIXELS: u64 = 1 << 28;
@@ -78,8 +95,12 @@ pub(crate) fn encode_png(pixmap: &Pixmap) -> Vec<u8> {
 /// x. The layout paints one glyph per op, so a space is written only where
 /// the pen jumps by more than a fifth of the font size.
 pub(crate) fn page_text(fonts: &Fonts<'_>, page: &Page) -> String {
-    // (baseline key, pieces)
+    // (baseline key, pieces) in first-seen order, plus the key→line index
+    // that finds one: a linear `Vec` scan per text op made a tall page with
+    // many distinct baselines cost (ops × baselines) key comparisons before
+    // any page-size budget applies — report extraction renders no raster.
     let mut lines: Vec<(i64, Vec<Piece>)> = Vec::new();
+    let mut at: HashMap<i64, usize> = HashMap::new();
     for op in &page.ops {
         let (face, size, x, y, glyphs, hscale, text) = match op {
             Op::Text {
@@ -121,9 +142,19 @@ pub(crate) fn page_text(fonts: &Fonts<'_>, page: &Page) -> String {
             size,
             text: text.clone(),
         };
-        match lines.iter_mut().find(|(k, _)| *k == key) {
-            Some((_, pieces)) => pieces.push(piece),
-            None => lines.push((key, vec![piece])),
+        #[cfg(test)]
+        bump_baseline_lookups();
+        let line = match at.get(&key) {
+            Some(&index) => index,
+            None => {
+                let index = lines.len();
+                at.insert(key, index);
+                lines.push((key, Vec::new()));
+                index
+            }
+        };
+        if let Some((_, pieces)) = lines.get_mut(line) {
+            pieces.push(piece);
         }
     }
     let mut out = String::new();
@@ -812,6 +843,46 @@ mod tests {
         assert_eq!(page_text(&fonts, &page), "Hello world\nSecond\n");
     }
 
+    #[test]
+    fn page_text_groups_many_distinct_baselines_with_linear_lookups() {
+        // A tall page can carry as many distinct baselines as the document
+        // asks for, and report extraction walks them without any raster
+        // page-size budget: each text op's baseline lookup must stay an
+        // indexed probe, not a scan of every baseline seen so far.
+        let fonts = Fonts::new();
+        let face = FaceRef::Catalogue(FaceId::CarlitoRegular);
+        let f = fonts.get(face);
+        let size = 10.0;
+        let text = |x: f32, y: f32, s: &str| Op::text(face, size, x, y, f.glyphs(s), [0.0; 3], s);
+        let mut counts = Vec::new();
+        for n in [256usize, 1024, 4096] {
+            let ops: Vec<Op> = (0..n)
+                .map(|i| {
+                    // Distinct baselines (a whole point apart, so the
+                    // quarter-point keys cannot collide), painted highest
+                    // first: first-seen order and numeric order agree.
+                    let y = 800.0 - f32::from(i as i32);
+                    let digit = char::from(b'0' + (i % 10) as u8);
+                    text(72.0, y, &digit.to_string())
+                })
+                .collect();
+            let page = page_with(ops);
+            let before = BASELINE_LOOKUPS.with(|c| c.get());
+            let out = page_text(&fonts, &page);
+            counts.push(BASELINE_LOOKUPS.with(|c| c.get()) - before);
+            assert_eq!(out.lines().count(), n, "one line a baseline");
+            assert!(out.starts_with('0'), "first-seen baseline first: {out}");
+        }
+        assert!(
+            counts.iter().all(|&c| c > 0),
+            "the lookup counter must measure grouping: {counts:?}"
+        );
+        assert!(
+            counts[2] <= 16 * counts[0],
+            "lookups must grow with the op count, not baselines × ops: {counts:?}"
+        );
+    }
+
     #[test]
     fn images_and_shapes_paint_without_panicking() {
         let fonts = Fonts::new();
diff --git a/src/convert/font_table.rs b/src/convert/font_table.rs
index c1063f85..01d7d375 100644
--- a/src/convert/font_table.rs
+++ b/src/convert/font_table.rs
@@ -202,8 +202,14 @@ pub(crate) fn deobfuscate_odttf(bytes: &[u8], font_key: &str) -> Vec<u8> {
 /// variants a–f, 2026-10-03: the pitch survives every other change). A
 /// machine without the family paints the embedded copy: the catalogue's
 /// slot is then a metric twin under its own name, not the family.
+///
+/// The decoded payload is decoded once per `(part, fontKey)` and shared:
+/// font-table aliases that name one embedded part keep one `Arc` between
+/// them, so a package with thousands of aliases of one font holds one
+/// copy of its bytes instead of one private copy each.
 pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::font::EmbeddedFonts {
     let mut out = HashMap::new();
+    let mut decoded: HashMap<(String, String), Option<std::sync::Arc<[u8]>>> = HashMap::new();
     let part = font_table_part(pkg);
     let rels = pkg.read_rels_for(&part);
     for entry in table.iter() {
@@ -218,19 +224,147 @@ pub(crate) fn load_embedded_fonts(pkg: &PartFs, table: &FontTable) -> super::fon
                 continue;
             };
             let path = pkg.resolve_rel_target(&part, &rel.target);
-            let Some(raw) = pkg.part_bytes(&path) else {
-                continue;
+            let data = match decoded.get(&(path.clone(), font_key.clone())) {
+                Some(Some(shared)) => shared.clone(),
+                Some(None) => continue,
+                None => {
+                    let loaded = pkg.part_bytes(&path).and_then(|raw| {
+                        let data = deobfuscate_odttf(raw, font_key);
+                        if ttf_parser::Face::parse(&data, 0).is_ok() {
+                            Some(data.into())
+                        } else {
+                            None
+                        }
+                    });
+                    decoded.insert((path, font_key.clone()), loaded.clone());
+                    match loaded {
+                        Some(shared) => shared,
+                        None => continue,
+                    }
+                }
             };
-            let data = deobfuscate_odttf(raw, font_key);
-            if ttf_parser::Face::parse(&data, 0).is_err() {
-                continue;
-            }
-            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data.into());
+            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data);
         }
     }
     out
 }
 
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod embedded_font_alias_tests {
+    use super::*;
+    use std::io::{Cursor, Write};
+
+    const LIBERATION_MONO: &[u8] = include_bytes!("../../assets/fonts/LiberationMono-Regular.ttf");
+
+    /// A package whose `word/fontTable.xml` holds `table` and whose
+    /// `word/_rels/fontTable.xml.rels` points `rId1` at a binary part.
+    fn package_with_font_part(table: &str, font_part: &str, font: &[u8]) -> PartFs {
+        let mut buf = Vec::new();
+        {
+            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
+            let opt = zip::write::SimpleFileOptions::default();
+            let parts: [(&str, String); 4] = [
+                ("[Content_Types].xml", r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="odttf" ContentType="application/vnd.openxmlformats-officedocument.obfuscatedFont"/></Types>"#.to_string()),
+                ("_rels/.rels".to_string(), r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.to_string()),
+                ("word/document.xml".to_string(), r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body/></w:document>"#.to_string()),
+                ("word/_rels/fontTable.xml.rels".to_string(), format!(
+                    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/font" Target="{font_part}"/></Relationships>"#
+                )),
+            ];
+            for (name, xml) in parts {
+                z.start_file(name, opt).unwrap();
+                z.write_all(xml.as_bytes()).unwrap();
+            }
+            z.start_file("word/fontTable.xml", opt).unwrap();
+            z.write_all(table.as_bytes()).unwrap();
+            z.start_file(format!("word/{font_part}"), opt).unwrap();
+            z.write_all(font).unwrap();
+            z.finish().unwrap();
+        }
+        PartFs::open(&buf).expect("package")
+    }
+
+    /// One `w:font` row embedding `rid` under `key`.
+    fn alias(name: &str, rid: &str, key: &str) -> String {
+        format!(
+            r#"<w:font w:name="{name}"><w:embedRegular r:id="{rid}" w:fontKey="{key}"/></w:font>"#
+        )
+    }
+
+    fn fonts_xml(fonts: String) -> String {
+        format!(
+            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">{fonts}</w:fonts>"#
+        )
+    }
+
+    #[test]
+    fn many_aliases_of_one_embedded_font_share_one_decoded_payload() {
+        let names = ["RareOne", "RareTwo", "RareThree", "RareFour"];
+        let pkg = package_with_font_part(
+            &fonts_xml(
+                names
+                    .iter()
+                    .map(|name| alias(name, "rId1", "{00000000-0000-0000-0000-000000000001}"))
+                    .collect(),
+            ),
+            "fonts/rare.odttf",
+            LIBERATION_MONO,
+        );
+        let table = load_font_table(&pkg);
+        let embeds = load_embedded_fonts(&pkg, &table);
+        assert_eq!(embeds.len(), names.len(), "an entry an alias");
+        let first = embeds
+            .get(&("rareone".into(), false, false))
+            .expect("first alias embedded");
+        for name in names {
+            let data = embeds
+                .get(&(name.to_ascii_lowercase(), false, false))
+                .expect("alias embedded");
+            assert!(
+                std::sync::Arc::ptr_eq(data, first),
+                "{name} must share the one decoded copy"
+            );
+            assert_eq!(data.as_ref(), LIBERATION_MONO);
+        }
+    }
+
+    #[test]
+    fn distinct_keys_and_malformed_fonts_stay_distinct_and_refused() {
+        let key_a = "{00000000-0000-0000-0000-000000000001}";
+        let key_b = "{00000000-0000-0000-0000-000000000002}";
+        // Two aliases of one part under two keys: the plain TTF header
+        // means neither key deobfuscates, but the payloads must not be
+        // shared — a buffer decoded under one key is never handed to a
+        // slot that names another.
+        let pkg = package_with_font_part(
+            &fonts_xml(alias("KeyA", "rId1", key_a) + &alias("KeyB", "rId1", key_b)),
+            "fonts/keys.odttf",
+            LIBERATION_MONO,
+        );
+        let table = load_font_table(&pkg);
+        let embeds = load_embedded_fonts(&pkg, &table);
+        let a = embeds.get(&("keya".into(), false, false)).expect("KeyA");
+        let b = embeds.get(&("keyb".into(), false, false)).expect("KeyB");
+        assert!(
+            !std::sync::Arc::ptr_eq(a, b),
+            "distinct keys, distinct data"
+        );
+        assert_eq!(a.as_ref(), LIBERATION_MONO);
+        assert_eq!(b.as_ref(), LIBERATION_MONO);
+        // A malformed font is refused for every alias.
+        let pkg = package_with_font_part(
+            &fonts_xml(alias("Broken", "rId1", key_a) + &alias("BrokenToo", "rId1", key_a)),
+            "fonts/broken.odttf",
+            b"not a font at all",
+        );
+        let table = load_font_table(&pkg);
+        let embeds = load_embedded_fonts(&pkg, &table);
+        assert!(embeds.get(&("broken".into(), false, false)).is_none());
+        assert!(embeds.get(&("brokentoo".into(), false, false)).is_none());
+    }
+}
+
 #[cfg(test)]
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod tests {
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..9584b8c 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -39,6 +39,8 @@ use std::collections::HashMap;
 
 use crate::admission::InputLimits;
 
+#[cfg(test)]
+use super::bump_entity_scan_steps;
 use super::{Dom, NodeId, XDeclaration, XName, XNamespace};
 
 // ── tag policy budgets (N01/N02) ────────────────────────────────────────────
@@ -761,6 +763,12 @@ fn extract_pseudo_attr(s: &str, key: &str) -> Option<String> {
 /// through. ASCII-only names (matching the TS regex `[a-zA-Z]+`) are
 /// recognized; names with non-ASCII letters (e.g. `&eacute;`) are left
 /// literal. Input with no `&` short-circuits to a clone of `s`.
+///
+/// The reference body is walked with the entity grammar (a `#`-prefixed
+/// digit run, a `#x`-prefixed hex run, or at most the four letters of the
+/// longest predefined name), so one unrecognized `&` costs work
+/// proportional to the reference's own length, never a rescan of the
+/// whole remaining suffix.
 pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
@@ -770,23 +778,73 @@ pub fn unescape_xml_text(s: &str) -> String {
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
-            // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
-                let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
-                if let Some(decoded) = decode_entity(&body) {
-                    out.push_str(&decoded);
-                    i = i + 1 + semi_rel + 1;
-                    continue;
-                }
+            if let Some((decoded, next)) = entity_at(&chars, i) {
+                out.push_str(&decoded);
+                i = next;
+                continue;
             }
-            out.push('&');
+        }
+        out.push(chars[i]);
+        i += 1;
+    }
+    out
+}
+
+/// The reference starting at `chars[at] == '&'`: its decoded text and the
+/// index just past the closing `;`, or `None` when what follows cannot be a
+/// reference this decoder recognizes. The walk stops at the first character
+/// outside the body grammar, so an unterminated reference never drags the
+/// rest of the input into the search. `at + 1` cannot overflow: the caller
+/// holds `at < chars.len()`.
+fn entity_at(chars: &[char], at: usize) -> Option<(String, usize)> {
+    #[cfg(test)]
+    bump_entity_scan_steps();
+    let mut i = at + 1;
+    match chars.get(i) {
+        Some('#') => {
             i += 1;
-        } else {
-            out.push(chars[i]);
+            if matches!(chars.get(i), Some('x') | Some('X')) {
+                i += 1;
+                while let Some(c) = chars.get(i) {
+                    #[cfg(test)]
+                    bump_entity_scan_steps();
+                    if !c.is_ascii_hexdigit() {
+                        break;
+                    }
+                    i += 1;
+                }
+            } else {
+                while let Some(c) = chars.get(i) {
+                    #[cfg(test)]
+                    bump_entity_scan_steps();
+                    if !c.is_ascii_digit() {
+                        break;
+                    }
+                    i += 1;
+                }
+            }
+        }
+        // The predefined names are at most four letters (`quot`, `apos`);
+        // a longer alphabetic run decodes as nothing.
+        Some(c) if c.is_ascii_alphabetic() => {
+            #[cfg(test)]
+            bump_entity_scan_steps();
             i += 1;
+            while i < at + 5 && matches!(chars.get(i), Some(c) if c.is_ascii_alphabetic()) {
+                #[cfg(test)]
+                bump_entity_scan_steps();
+                i += 1;
+            }
         }
+        _ => return None,
     }
-    out
+    if chars.get(i) != Some(&';') {
+        return None;
+    }
+    #[cfg(test)]
+    bump_entity_scan_steps();
+    let body: String = chars[at + 1..i].iter().collect();
+    decode_entity(&body).map(|decoded| (decoded, i + 1))
 }
 
 fn decode_entity(body: &str) -> Option<String> {
@@ -1069,6 +1127,69 @@ mod xml_lexical_boundary_contract_tests {
         }
     }
 
+    #[test]
+    fn grammar_bounded_reference_walk_keeps_the_literal_fallback_cases() {
+        // Each shape the walker meets at a `&`: the outcome must stay what
+        // the old whole-suffix `;` search produced, only the scan is bounded.
+        for (input, expected) in [
+            // A bare `&` before anything but `#` or a letter stays literal.
+            ("&&&&", "&&&&"),
+            ("& 65;", "& 65;"),
+            // Numeric bodies end at the first non-digit, `;` or not.
+            ("&#65", "&#65"),
+            ("&#6x;", "&#6x;"),
+            ("&#65x;", "&#65x;"),
+            ("&#x4a;", "J"),
+            ("&#x4G;", "&#x4G;"),
+            ("&#xzz;", "&#xzz;"),
+            ("&#;", "&#;"),
+            ("&#x;", "&#x;"),
+            // Long zero-padded numeric references still decode.
+            ("&#00000000000000000065;", "A"),
+            // A name longer than `quot`/`apos` decodes as nothing, and one
+            // reference's unrecognized tail does not swallow later text.
+            ("&abcde;", "&abcde;"),
+            ("&ab;", "&ab;"),
+            ("&bad; more &amp; text", "&bad; more & text"),
+            // A multibyte character right after a short reference's `;`
+            // never splits the walk.
+            ("&amp;\u{1F600}", "&\u{1F600}"),
+            ("&amp;&am\u{e9};", "&&am\u{e9};"),
+        ] {
+            assert_eq!(unescape_xml_text(input), expected, "{input}");
+        }
+    }
+
+    #[test]
+    fn long_bare_ampersand_runs_decode_with_linear_scan_work() {
+        // Each unrecognized `&` used to rescan the whole remaining suffix
+        // for `;`; the walker now examines a bounded body per `&`, so the
+        // examined-character count stays proportional to the input.
+        let mut counts = Vec::new();
+        for n in [4_096usize, 16_384, 65_536] {
+            let input = "&".repeat(n);
+            let before = crate::xmllinq::ENTITY_SCAN_STEPS.with(|c| c.get());
+            assert_eq!(unescape_xml_text(&input), input);
+            counts.push(crate::xmllinq::ENTITY_SCAN_STEPS.with(|c| c.get()) - before);
+        }
+        assert!(
+            counts.iter().all(|&c| c > 0),
+            "the step counter must measure the reference walk: {counts:?}"
+        );
+        assert!(
+            counts[2] <= 4 * counts[1],
+            "scan steps must grow with the input, not its square: {counts:?}"
+        );
+        // Unterminated named references are bounded the same way: the body
+        // walk stops after the four letters of the longest predefined name.
+        let input = "&quuu".repeat(8_192);
+        let before = crate::xmllinq::ENTITY_SCAN_STEPS.with(|c| c.get());
+        let decoded = unescape_xml_text(&input);
+        assert_eq!(decoded, input);
+        let steps = crate::xmllinq::ENTITY_SCAN_STEPS.with(|c| c.get()) - before;
+        assert!(steps <= 10 * 8_192, "bounded name walk: {steps} steps");
+    }
+
     #[test]
     fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
         // XML 1.0 §2.11 + §3.3.3, the value a conforming reader (Word) sees:
diff --git a/src/xmllinq/mod.rs b/src/xmllinq/mod.rs
index 3e606ae6..f8bab114 100644
--- a/src/xmllinq/mod.rs
+++ b/src/xmllinq/mod.rs
@@ -284,6 +284,23 @@ fn bump_attr_name_probes() {
     ATTR_NAME_PROBES.with(|c| c.set(c.get() + 1));
 }
 
+// Entity-decode mechanism counter (tests only): characters the reference
+// walker of `unescape_xml_text` examines, one `&` at a time. The walk is
+// bounded by each reference's own grammar, never the remaining suffix: a
+// long run of bare `&` costs one step each instead of rescanning the
+// whole suffix per `&`. Per thread, like `ATTR_NAME_PROBES`.
+#[cfg(test)]
+thread_local! {
+    pub(crate) static ENTITY_SCAN_STEPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
+}
+
+/// Add one to this thread's [`ENTITY_SCAN_STEPS`].
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+pub(crate) fn bump_entity_scan_steps() {
+    ENTITY_SCAN_STEPS.with(|c| c.set(c.get() + 1));
+}
+
 /// Handle into a [`Dom`] arena: the slot index of a node, plus a `Debug` /
 /// `Copy` so it can travel cheaply through hot loops (compare, atomize).
 #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]

```
