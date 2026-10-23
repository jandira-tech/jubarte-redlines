## Patch (part 2 of 2: Rust engine files)

Continues the patch from the issue body (`src/convert/altchunk.rs`, `src/convert/metafile.rs`, `src/xmllinq/parse.rs`, in the patch's own file order). Concatenating the body's part 1 with this part 2 reproduces `.scan/remediation/patches/dedup-0026.patch` exactly.

```diff
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -34,7 +34,7 @@ pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> Strin
         let wml = attr(open, "id")
             .and_then(|rid| load(&rid))
             .and_then(|bytes| chunk_html(&bytes))
-            .map(|html| html_to_wml(&html))
+            .and_then(|html| html_to_wml(&html))
             .unwrap_or_default();
         out.push_str(&wml);
         rest = &tail[end..];
@@ -276,8 +276,20 @@ struct Builder {
     fmts: Vec<(String, Fmt)>,
     tables: Vec<Table>,
     cell_spans: Vec<usize>,
+    /// A cell span or a row's span sum exceeded its budget: the chunk is
+    /// refused instead of generating the columns it claims.
+    refused: bool,
 }
 
+/// The widest span one `<td>`/`<th>` may claim. HTML itself clamps
+/// `colspan` above 1000 (HTML5 §4.9.11), so no importer owes a grid column
+/// per claimed slot beyond it; a chunk that asks for more is refused.
+const MAX_CELL_SPAN: usize = 1000;
+/// Grid columns one imported HTML table may generate before the chunk is
+/// refused. Word's own table import tops out far below; generated
+/// `<w:gridCol>` markup is bounded by this budget.
+const MAX_TABLE_COLUMNS: usize = 4096;
+
 const BLOCKS: [&str; 10] = [
     "p",
     "h1",
@@ -469,10 +481,18 @@ impl Builder {
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| r.iter().fold(0usize, |acc, c| acc.saturating_add(c.span)))
             .max()
             .unwrap_or(1)
             .max(1);
+        if cols > MAX_TABLE_COLUMNS || self.refused {
+            // Refuse before generating: a table over the column budget — or
+            // any table after an earlier refusal — contributes no markup, so
+            // a lying row cannot amplify into `cols` gridCol appends that
+            // `html_to_wml` would only discard.
+            self.refused = true;
+            return;
+        }
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -559,12 +579,14 @@ impl Builder {
                 self.close_open_cell();
                 self.flush();
                 self.sinks.push(String::new());
-                self.cell_spans.push(
-                    attr(open, "colspan")
-                        .and_then(|s| s.trim().parse::<usize>().ok())
-                        .unwrap_or(1)
-                        .max(1),
-                );
+                let span = attr(open, "colspan")
+                    .and_then(|s| s.trim().parse::<usize>().ok())
+                    .unwrap_or(1)
+                    .max(1);
+                if span > MAX_CELL_SPAN {
+                    self.refused = true;
+                }
+                self.cell_spans.push(span.min(MAX_CELL_SPAN));
                 let mut fmt = self.fmt();
                 if tag == "th" {
                     fmt.bold = true;
@@ -600,8 +622,11 @@ impl Builder {
     }
 }
 
-/// The body WML of an HTML document.
-pub(crate) fn html_to_wml(html: &str) -> String {
+/// The body WML of an HTML document, or `None` when its tables exceed the
+/// cell-span or column budget ([`MAX_CELL_SPAN`], [`MAX_TABLE_COLUMNS`]):
+/// the caller refuses the chunk before any grid markup is generated,
+/// rather than allocating a column per claimed slot.
+pub(crate) fn html_to_wml(html: &str) -> Option<String> {
     let css = collect_between(html, "<style", "</style>")
         .map(|s| parse_css(&s))
         .unwrap_or_default();
@@ -620,6 +645,7 @@ pub(crate) fn html_to_wml(html: &str) -> String {
         )],
         tables: Vec::new(),
         cell_spans: Vec::new(),
+        refused: false,
     };
     let src = &html[body_at..];
     let mut i = 0;
@@ -684,7 +710,10 @@ pub(crate) fn html_to_wml(html: &str) -> String {
     while !b.tables.is_empty() {
         b.end_table();
     }
-    b.sinks.pop().unwrap_or_default()
+    if b.refused {
+        return None;
+    }
+    Some(b.sinks.pop().unwrap_or_default())
 }
 
 fn collect_between(s: &str, open: &str, close: &str) -> Option<String> {
@@ -779,7 +808,7 @@ mod tests {
         let html = "<html><head><style>p.t { text-align:center; margin-top:14,4px; }\
                     span.s { font-size:16pt; }</style></head><body>\
                     <p class=\"t\"><span class=\"s\"><b>Title &amp; more</b></span></p></body></html>";
-        let wml = html_to_wml(html);
+        let wml = html_to_wml(html).expect("html");
         assert!(wml.contains("<w:jc w:val=\"center\"/>"), "{wml}");
         assert!(wml.contains("w:before=\"216\""), "14.4px = 10.8pt; {wml}");
         assert!(
@@ -874,7 +903,9 @@ mod tests {
         #[test]
         fn omitted_cell_and_row_end_tags_keep_the_table() {
             // HTML lets </td>, </th> and </tr> go unwritten.
-            let wml = html_to_wml("<table><tr><th>H1<th>H2<tr><td>A<td>B</table><p>After</p>");
+            let wml =
+                html_to_wml("<table><tr><th>H1<th>H2<tr><td>A<td>B</table><p>After</p>")
+                    .expect("html");
             assert_eq!(wml.matches("<w:tbl>").count(), 1, "{wml}");
             assert_eq!(wml.matches("<w:tr>").count(), 2, "{wml}");
             assert_eq!(wml.matches("<w:tc>").count(), 4, "{wml}");
@@ -887,7 +918,8 @@ mod tests {
             // A nested table with implicit ends stays inside its outer cell.
             let wml = html_to_wml(
                 "<table><tr><td>Out<table><tr><td>In1<td>In2</table></td><td>Next</table>",
-            );
+            )
+            .expect("html");
             assert_eq!(wml.matches("<w:tbl>").count(), 2, "{wml}");
             for t in ["Out", "In1", "In2", "Next"] {
                 assert!(wml.contains(&format!(">{t}</w:t>")), "{t}: {wml}");
@@ -923,7 +955,7 @@ mod tests {
             );
             let invalid = "&unknown; &#xD800; &#1114112; &#xZZ; &unfinished";
             assert_eq!(decode_entities(invalid), invalid);
-            let wml = html_to_wml("<p>&lt;tag&gt; &amp; &unknown;</p>");
+            let wml = html_to_wml("<p>&lt;tag&gt; &amp; &unknown;</p>").expect("html");
             assert!(
                 wml.contains(">&lt;tag&gt; &amp; &amp;unknown;</w:t>"),
                 "{wml}"
@@ -949,7 +981,7 @@ mod tests {
 
         #[test]
         fn nested_inline_formatting_is_restored_after_closing_tags() {
-            let wml = html_to_wml("<p><b>bold<i>both</i>bold again</b>plain</p>");
+            let wml = html_to_wml("<p><b>bold<i>both</i>bold again</b>plain</p>").expect("html");
             let runs: Vec<&str> = wml.split("<w:r>").skip(1).collect();
             assert_eq!(runs.len(), 4, "{wml}");
             for (run, bold, italic) in [
@@ -968,7 +1000,8 @@ mod tests {
             let wml = html_to_wml(
                 "<html><head><style>p {font-size:10pt} .large {font-size:14pt; font-weight:bold}\
             p.large {font-size:18pt}</style></head><body><p class='large' style='font-size:20pt; font-weight:normal'>Text</p></body></html>",
-            );
+            )
+            .expect("html");
             assert!(wml.contains("<w:sz w:val=\"40\"/>"), "{wml}");
             assert!(!wml.contains("<w:b/>"), "{wml}");
             assert!(!wml.contains("<w:sz w:val=\"28\"/>"));
@@ -978,7 +1011,8 @@ mod tests {
         fn paragraph_css_controls_spacing_alignment_and_indent() {
             let wml = html_to_wml(
                 "<p style='margin-top:0; margin-bottom:6pt; margin-left:1in; line-height:150%; text-align:justify'>Text</p>",
-            );
+            )
+            .expect("html");
             assert!(
                 wml.contains(
                     "<w:spacing w:before=\"0\" w:after=\"120\" w:line=\"360\" w:lineRule=\"auto\"/>"
@@ -995,7 +1029,8 @@ mod tests {
             let wml = html_to_wml(
                 "<html><head><style>p {font-size:12pt}</style></head><body><!-- hidden -->\
             <p>Before<script>secret()</script><br/>After</p></body></html>",
-            );
+            )
+            .expect("html");
             assert!(
                 wml.contains(">Before</w:t>") && wml.contains(">After</w:t>"),
                 "text on both sides of the script must survive: {wml}"
@@ -1008,13 +1043,13 @@ mod tests {
 
         #[test]
         fn style_in_an_html_fragment_preserves_following_content() {
-            let wml = html_to_wml("<style>p {font-size:12pt}</style><p>After</p>");
+            let wml = html_to_wml("<style>p {font-size:12pt}</style><p>After</p>").expect("html");
             assert!(
                 wml.contains(">After</w:t>"),
                 "style content is skipped, but the following paragraph must survive: {wml}"
             );
             // A self-closed or stray closing tag has no body to swallow.
-            let wml = html_to_wml("<p>A<script/>B</script>C</p><p>D</p>");
+            let wml = html_to_wml("<p>A<script/>B</script>C</p><p>D</p>").expect("html");
             assert!(
                 wml.contains(">A</w:t>") && wml.contains(">D</w:t>"),
                 "{wml}"
@@ -1027,7 +1062,8 @@ mod tests {
             let wml = html_to_wml(
                 "<table border='1'><tr><th colspan='2'>Heading</th></tr>\
             <tr><td>A</td><td></td></tr></table><p>After</p>",
-            );
+            )
+            .expect("html");
             assert_eq!(wml.matches("<w:gridCol ").count(), 2, "{wml}");
             assert!(wml.contains("<w:tcW w:w=\"9360\" w:type=\"dxa\"/><w:gridSpan w:val=\"2\"/>"));
             assert!(wml.contains("<w:tcW w:w=\"4680\" w:type=\"dxa\"/></w:tcPr><w:p/></w:tc>"));
@@ -1048,12 +1084,69 @@ mod tests {
             for span in ["0", "invalid", "-1"] {
                 let wml = html_to_wml(&format!(
                     "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
-                ));
+                ))
+                .expect("html");
                 assert_eq!(wml.matches("<w:gridCol ").count(), 1, "{span}: {wml}");
                 assert!(!wml.contains("<w:gridSpan"), "{wml}");
                 assert!(!wml.contains("<w:tblBorders>"));
             }
         }
+
+        #[test]
+        fn a_cell_span_beyond_htmls_own_clamp_refuses_the_chunk() {
+            // HTML clamps colspan above 1000; an importer owes no grid
+            // column per claimed slot beyond that. The first value is one
+            // past the clamp so the refusal is observable without
+            // proportional work (the larger values exercise the same parse
+            // path; before this fix each demanded a `w:gridCol` per
+            // claimed column).
+            for span in [1001_usize, 5000, 1000000000, 4294967295] {
+                let html = format!("<table><tr><td colspan='{span}'>A</td></tr></table>");
+                assert!(html_to_wml(&html).is_none(), "span {span} must refuse");
+            }
+            // The clamp's inclusive boundary still renders, span intact.
+            let wml = html_to_wml(
+                "<table border='0'><tr><td colspan='1000'>A</td></tr></table>",
+            )
+            .expect("html");
+            assert_eq!(wml.matches("<w:gridCol ").count(), 1000, "{wml}");
+            assert!(wml.contains("<w:gridSpan w:val=\"1000\"/>"), "{wml}");
+        }
+
+        #[test]
+        fn row_span_sums_over_the_column_budget_refuse_the_chunk() {
+            // Five individually legal 1000-span cells sum past the
+            // 4096-column grid budget. (Runnable red: the pre-fix output is
+            // 5000 `w:gridCol` elements, fast to generate and easy to
+            // refuse.)
+            let cells = "<td colspan='1000'>x</td>".repeat(5);
+            let html = format!("<table><tr>{cells}</tr></table>");
+            assert!(
+                html_to_wml(&html).is_none(),
+                "5000 declared columns must refuse"
+            );
+            // Four such cells stay inside the budget and keep their grid.
+            let cells = "<td colspan='1000'>x</td>".repeat(4);
+            let wml = html_to_wml(&format!("<table><tr>{cells}</tr></table>")).expect("html");
+            assert_eq!(wml.matches("<w:gridCol ").count(), 4000, "{wml}");
+            // Many valid cells cannot overflow the aggregate count either.
+            let cells = "<td colspan='4096'>x</td>".repeat(2);
+            let html = format!("<table><tr>{cells}</tr></table>");
+            assert!(html_to_wml(&html).is_none(), "a saturated sum must refuse");
+        }
+
+        #[test]
+        fn a_refused_chunk_expands_to_nothing_like_a_missing_one() {
+            // The refusal rides the same path as a chunk that cannot be
+            // read: the altChunk is dropped and the body around it is kept.
+            let xml = "<w:body><w:altChunk r:id='bad'/><w:p>After</w:p></w:body>";
+            let out = expand(xml, |id| {
+                (id == "bad").then(|| {
+                    b"<table><tr><td colspan='5000'>A</td></tr></table>".to_vec()
+                })
+            });
+            assert_eq!(out, "<w:body><w:p>After</w:p></w:body>", "{out}");
+        }
     }
 }
 
@@ -1146,7 +1239,7 @@ mod imported_source_contract_tests {
                 "<style>.owned {{font-weight:600}} .wrong {{font-style:italic}}</style><p {class}>Owned</p>"
             );
             let expected = plain("Owned").replace("<w:sz ", "<w:b/><w:sz ");
-            assert_eq!(html_to_wml(&source), expected, "{class}");
+            assert_eq!(html_to_wml(&source).expect("html"), expected, "{class}");
         }
         // Explicit malformed-attribute diagnostics: no closing quote declines
         // the class, and an unquoted attribute stops at HTML's tag terminator.
@@ -1182,7 +1275,7 @@ mod imported_source_contract_tests {
                     }
                     rpr.push_str("<w:sz w:val=\"24\"/><w:szCs w:val=\"24\"/></w:rPr>");
                     assert_eq!(
-                        html_to_wml(&html),
+                        html_to_wml(&html).expect("html"),
                         format!(
                             "<w:p><w:pPr>{SPACING}</w:pPr><w:r>{rpr}<w:t xml:space=\"preserve\">Owned</w:t></w:r></w:p>"
                         )
@@ -1202,7 +1295,7 @@ mod imported_source_contract_tests {
         ] {
             let source = format!("<p style='font-family:{family};font-size:16px'>Owned</p>");
             assert_eq!(
-                html_to_wml(&source),
+                html_to_wml(&source).expect("html"),
                 plain("Owned").replace("Times New Roman", expected)
             );
         }
@@ -1215,7 +1308,7 @@ mod imported_source_contract_tests {
                 let source =
                     format!("<p style='margin-left:{margin};text-align:{align}'>Owned</p>");
                 assert_eq!(
-                    html_to_wml(&source),
+                    html_to_wml(&source).expect("html"),
                     plain("Owned").replace("</w:pPr>", &format!("{ind}{jc}</w:pPr>"))
                 );
             }
@@ -1226,36 +1319,36 @@ mod imported_source_contract_tests {
     fn paragraph_whitespace_empty_runs_and_leading_breaks_preserve_exact_imported_payloads() {
         let empty = format!("<w:p><w:pPr>{SPACING}</w:pPr></w:p>");
         assert_eq!(
-            html_to_wml("<body> \n\t<p></p><p> </p></body>"),
+            html_to_wml("<body> \n\t<p></p><p> </p></body>").expect("html"),
             empty.repeat(2)
         );
         assert_eq!(
-            html_to_wml("<p> A<b> </b> B </p>"),
+            html_to_wml("<p> A<b> </b> B </p>").expect("html"),
             format!(
                 "<w:p><w:pPr>{SPACING}</w:pPr><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">A</w:t></w:r><w:r>{}<w:t xml:space=\"preserve\"> </w:t></w:r><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">B</w:t></w:r></w:p>",
                 DEFAULT_RPR.replace("<w:sz ", "<w:b/><w:sz ")
             )
         );
         assert_eq!(
-            html_to_wml("<br/> A"),
+            html_to_wml("<br/> A").expect("html"),
             format!(
                 "<w:p><w:pPr>{SPACING}</w:pPr><w:r>{DEFAULT_RPR}<w:br/></w:r><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">A</w:t></w:r></w:p>"
             )
         );
         assert_eq!(
-            html_to_wml("<p>A<br/> B</p>"),
+            html_to_wml("<p>A<br/> B</p>").expect("html"),
             format!(
                 "<w:p><w:pPr>{SPACING}</w:pPr><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">A</w:t></w:r><w:r>{DEFAULT_RPR}<w:br/></w:r><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\"> B</w:t></w:r></w:p>"
             )
         );
         assert_eq!(
-            html_to_wml("<p>A<br/></p>"),
+            html_to_wml("<p>A<br/></p>").expect("html"),
             format!(
                 "<w:p><w:pPr>{SPACING}</w:pPr><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">A</w:t></w:r><w:r>{DEFAULT_RPR}<w:br/></w:r></w:p>"
             )
         );
         assert_eq!(
-            html_to_wml("<p><br/></p>"),
+            html_to_wml("<p><br/></p>").expect("html"),
             format!("<w:p><w:pPr>{SPACING}</w:pPr><w:r>{DEFAULT_RPR}<w:br/></w:r></w:p>")
         );
         assert_eq!(collapse_ws("A\u{a0}\tB \n"), "A\u{a0} B ");
@@ -1264,16 +1357,20 @@ mod imported_source_contract_tests {
     #[test]
     fn valid_html_declarations_headings_and_self_closed_spans_keep_exact_source_formats() {
         assert_eq!(
-            html_to_wml("<!DOCTYPE html><?xml version='1.0'?><h2>Heading</h2><p>A<span/>B</p>"),
+            html_to_wml("<!DOCTYPE html><?xml version='1.0'?><h2>Heading</h2><p>A<span/>B</p>")
+                .expect("html"),
             format!(
                 "{}<w:p><w:pPr>{SPACING}</w:pPr><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">A</w:t></w:r><w:r>{DEFAULT_RPR}<w:t xml:space=\"preserve\">B</w:t></w:r></w:p>",
                 plain("Heading").replace("<w:sz ", "<w:b/><w:sz ")
             )
         );
         let malformed = "<p>Kept</p><unfinished";
-        assert_eq!(html_to_wml(malformed), plain("Kept"));
-        assert_eq!(html_to_wml("<p>Kept</p><!-- unfinished"), plain("Kept"));
-        assert_eq!(html_to_wml("<p>Kept</p><>"), plain("Kept"));
+        assert_eq!(html_to_wml(malformed).expect("html"), plain("Kept"));
+        assert_eq!(
+            html_to_wml("<p>Kept</p><!-- unfinished").expect("html"),
+            plain("Kept")
+        );
+        assert_eq!(html_to_wml("<p>Kept</p><>").expect("html"), plain("Kept"));
         let unterminated = "<w:body><w:p/><w:altChunk r:id='x'";
         assert_eq!(
             expand(unterminated, |_| panic!("unterminated chunk is not loaded")),
@@ -1284,7 +1381,7 @@ mod imported_source_contract_tests {
     #[test]
     fn unclosed_html_table_cells_are_finalized_once_with_source_spans_and_empty_cells() {
         for source in ["<table><tr><td>A<td>B", "<table><tr><td>A</td><td>B</td>"] {
-            let actual = html_to_wml(source);
+            let actual = html_to_wml(source).expect("html");
             let header = "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/><w:tblLayout w:type=\"fixed\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"4680\"/><w:gridCol w:w=\"4680\"/></w:tblGrid><w:tr>";
             assert_eq!(
                 actual,
@@ -1297,7 +1394,10 @@ mod imported_source_contract_tests {
         }
         // A row outside a table is malformed HTML. Its text remains normal
         // body text; a stray table end must not create a phantom table.
-        assert_eq!(html_to_wml("<tr>Owned</tr></table>"), plain("Owned"));
+        assert_eq!(
+            html_to_wml("<tr>Owned</tr></table>").expect("html"),
+            plain("Owned")
+        );
     }
 
     #[test]
@@ -1305,7 +1405,7 @@ mod imported_source_contract_tests {
         let source =
             "<style>,p {font-weight:600} ; .unused {font-style:italic}</style><p>Owned</p>";
         assert_eq!(
-            html_to_wml(source),
+            html_to_wml(source).expect("html"),
             plain("Owned").replace("<w:sz ", "<w:b/><w:sz ")
         );
         assert_eq!(
@@ -1360,7 +1460,7 @@ mod residual_import_contract_tests {
             let expected = format!(
                 "<w:p><w:pPr>{spacing}</w:pPr><w:r>{RPR}<w:t xml:space=\"preserve\">Source</w:t></w:r></w:p>"
             );
-            assert_eq!(html_to_wml(&source), expected, "{height}");
+            assert_eq!(html_to_wml(&source).expect("html"), expected, "{height}");
             assert_eq!(source, before);
         }
     }
@@ -1376,7 +1476,7 @@ mod residual_import_contract_tests {
             "<w:p><w:pPr>{SPACING}</w:pPr><w:r>{rpr}<w:t xml:space=\"preserve\">A&amp;B &lt;C&gt; &quot;D&quot;</w:t></w:r></w:p>{}",
             plain("After")
         );
-        assert_eq!(html_to_wml(source), expected);
+        assert_eq!(html_to_wml(source).expect("html"), expected);
         assert_eq!(source, before);
     }
 
@@ -1397,7 +1497,7 @@ mod residual_import_contract_tests {
             cell("C"),
             plain("After")
         );
-        assert_eq!(html_to_wml(source), expected);
+        assert_eq!(html_to_wml(source).expect("html"), expected);
         assert_eq!(source, before);
     }
 }
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -400,8 +400,13 @@ struct WmfFont {
 }
 
 impl WmfFont {
-    fn parse(data: &[u8], at: usize) -> Option<Self> {
-        let face = data.get(at + 18..)?;
+    /// `end` is the extent of the record that carries the LogFont: every
+    /// field and face byte is read inside it, never past it.
+    fn parse(data: &[u8], at: usize, end: usize) -> Option<Self> {
+        if at > end || 18 > end - at {
+            return None;
+        }
+        let face = data.get(at + 18..end)?;
         let face = &face[..face
             .iter()
             .position(|&b| b == 0)
@@ -487,15 +492,28 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             break;
         }
         let payload = off + 6;
+        // The record's declared extent. Every field, count, string and
+        // point a handler reads stays inside it: a record cannot consume
+        // the next record's bytes (or loose trailing bytes) as its payload,
+        // so one small record's claimed work is bounded by its declared
+        // size, and repeated records cannot amplify overlapping suffixes.
+        let end = off + size2;
+        let fits = |at: usize, len: usize| at <= end && len <= end - at;
         match func {
             0x0000 => break,
             0x020B => {
+                if !fits(payload, 4) {
+                    break;
+                }
                 let y = read_i16(data, payload)? as i32;
                 let x = read_i16(data, payload + 2)? as i32;
                 map.org_x = x as f32;
                 map.org_y = y as f32;
             }
             0x020C => {
+                if !fits(payload, 4) {
+                    break;
+                }
                 let y = read_i16(data, payload)? as i32;
                 let x = read_i16(data, payload + 2)? as i32;
                 if x != 0 {
@@ -506,6 +524,9 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 }
             }
             0x02FC => {
+                if !fits(payload, 6) {
+                    break;
+                }
                 let style = read_u16(data, payload).unwrap_or(0);
                 let color = colorref(read_u32(data, payload + 2).unwrap_or(0));
                 let slot = objects.iter().position(|o| matches!(o, GdiObj::Empty));
@@ -520,6 +541,9 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 }
             }
             0x02FA => {
+                if !fits(payload, 8) {
+                    break;
+                }
                 let style = read_u16(data, payload).unwrap_or(0);
                 let color = colorref(read_u32(data, payload + 6).unwrap_or(0));
                 let width = read_i16(data, payload + 2).unwrap_or(1) as i32;
@@ -533,7 +557,7 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             }
             0x02FB => {
                 if let Some(i) = objects.iter().position(|o| matches!(o, GdiObj::Empty)) {
-                    objects[i] = WmfFont::parse(data, payload)
+                    objects[i] = WmfFont::parse(data, payload, end)
                         .map_or(GdiObj::Other, |f| GdiObj::Font(std::rc::Rc::new(f)));
                 }
             }
@@ -545,9 +569,17 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     objects[i] = GdiObj::Other;
                 }
             }
-            0x0106 => winding = read_u16(data, payload).unwrap_or(1) == 2,
+            0x0106 => {
+                if !fits(payload, 2) {
+                    break;
+                }
+                winding = read_u16(data, payload).unwrap_or(1) == 2;
+            }
             0x012D => {
                 // Object handles are 0-based table indices (b88ac900).
+                if !fits(payload, 2) {
+                    break;
+                }
                 let idx = read_u16(data, payload).unwrap_or(0) as usize;
                 if let Some(obj) = objects.get(idx) {
                     match obj {
@@ -564,6 +596,9 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             0x001E => saved.push(dc.clone()),
             // RESTOREDC -n pops n states (5a6c's diagram: -1 after each box).
             0x0127 => {
+                if !fits(payload, 2) {
+                    break;
+                }
                 let n = read_i16(data, payload).unwrap_or(-1);
                 let keep = if n < 0 {
                     saved.len().saturating_sub(usize::from(n.unsigned_abs()))
@@ -575,12 +610,25 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     saved.truncate(keep);
                 }
             }
-            0x0209 => dc.text_color = colorref(read_u32(data, payload).unwrap_or(0)),
-            0x012E => dc.text_align = read_u16(data, payload).unwrap_or(0),
+            0x0209 => {
+                if !fits(payload, 4) {
+                    break;
+                }
+                dc.text_color = colorref(read_u32(data, payload).unwrap_or(0));
+            }
+            0x012E => {
+                if !fits(payload, 2) {
+                    break;
+                }
+                dc.text_align = read_u16(data, payload).unwrap_or(0);
+            }
             // RECTANGLE / ROUNDRECT: bottom, right, top, left; filled with
             // the brush, outlined with the pen.
             0x041B | 0x061C => {
                 let at = if func == 0x061C { payload + 4 } else { payload };
+                if !fits(at, 8) {
+                    break;
+                }
                 let bottom = read_i16(data, at)? as i32;
                 let right = read_i16(data, at + 2)? as i32;
                 let top = read_i16(data, at + 4)? as i32;
@@ -601,32 +649,53 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             }
             // EXTTEXTOUT: y, x, count, options, [rect], string.
             0x0A32 => {
+                if !fits(payload, 8) {
+                    break;
+                }
                 let y = read_i16(data, payload)?;
                 let x = read_i16(data, payload + 2)?;
                 let n = usize::from(read_u16(data, payload + 4)?);
                 let opts = read_u16(data, payload + 6)?;
                 let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
+                if !fits(at, n) {
+                    break;
+                }
                 if let Some(bytes) = data.get(at..at + n) {
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
             // TEXTOUT: count, string (word-padded), y, x.
             0x0521 => {
+                if !fits(payload, 2) {
+                    break;
+                }
                 let n = usize::from(read_u16(data, payload)?);
                 let at = payload + 2 + n.next_multiple_of(2);
+                if !fits(payload + 2, n) || !fits(at, 4) {
+                    break;
+                }
                 let (y, x) = (read_i16(data, at)?, read_i16(data, at + 2)?);
                 if let Some(bytes) = data.get(payload + 2..payload + 2 + n) {
                     texts.extend(wmf_text(&dc, &map, x, y, bytes));
                 }
             }
             0x01F0 => {
+                if !fits(payload, 2) {
+                    break;
+                }
                 let idx = read_u16(data, payload).unwrap_or(0) as usize;
                 if let Some(slot) = objects.get_mut(idx) {
                     *slot = GdiObj::Empty;
                 }
             }
             0x0324 => {
-                let n = read_u16(data, payload).unwrap_or(0) as usize;
+                if !fits(payload, 2) {
+                    break;
+                }
+                let n = usize::from(read_u16(data, payload).unwrap_or(0));
+                if !fits(payload + 2, n * 4) {
+                    break;
+                }
                 let mut pts = Vec::with_capacity(n);
                 let mut p = payload + 2;
                 for _ in 0..n {
@@ -642,8 +711,21 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             // META_POLYPOLYGON: ring count, each ring's point count, then
             // every ring's points, filled as one shape (b88ac900's logo).
             0x0538 => {
-                let rings = read_u16(data, payload).unwrap_or(0) as usize;
-                let mut p = payload + 2 + 2 * rings;
+                if !fits(payload, 2) {
+                    break;
+                }
+                let rings = usize::from(read_u16(data, payload).unwrap_or(0));
+                if !fits(payload + 2, rings * 2) {
+                    break;
+                }
+                let points_at = payload + 2 + rings * 2;
+                let total: usize = (0..rings)
+                    .map(|r| usize::from(read_u16(data, payload + 2 + 2 * r).unwrap_or(0)))
+                    .fold(0, usize::saturating_add);
+                if !fits(points_at, total.saturating_mul(4)) {
+                    break;
+                }
+                let mut p = points_at;
                 let mut subpaths = Vec::with_capacity(rings);
                 for r in 0..rings {
                     let n = read_u16(data, payload + 2 + 2 * r)? as usize;
@@ -665,7 +747,13 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                     off += size2;
                     continue;
                 };
-                let n = read_u16(data, payload).unwrap_or(0) as usize;
+                if !fits(payload, 2) {
+                    break;
+                }
+                let n = usize::from(read_u16(data, payload).unwrap_or(0));
+                if !fits(payload + 2, n * 4) {
+                    break;
+                }
                 let mut prev: Option<(i32, i32)> = None;
                 let mut p = payload + 2;
                 for _ in 0..n {
@@ -1170,6 +1258,159 @@ mod hostile_input_tests {
     }
 }
 
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod wmf_record_boundary_tests {
+    //! A record's declared size bounds everything its handler reads: a
+    //! short record claiming the next record's bytes (or loose trailing
+    //! bytes) as its points or text ends the replay instead of amplifying
+    //! those bytes into raster work or retained strings.
+    use super::*;
+
+    /// A placeable WMF shell (bounds 0..100, so text is kept) over the
+    /// records a test appends.
+    fn shell() -> Vec<u8> {
+        let mut d = vec![0u8; 22];
+        d[..4].copy_from_slice(&PLACEABLE_KEY);
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
+    /// One record whose declared `size_words` the test chooses (so a test
+    /// may lie about it); `payload` is every byte after the six-byte header.
+    fn record(d: &mut Vec<u8>, size_words: u32, func: u16, payload: &[u8]) {
+        d.extend_from_slice(&size_words.to_le_bytes());
+        d.extend_from_slice(&func.to_le_bytes());
+        d.extend_from_slice(payload);
+    }
+
+    /// One record that declares exactly the payload it carries.
+    fn full_record(d: &mut Vec<u8>, func: u16, mut payload: Vec<u8>) {
+        if payload.len() % 2 == 1 {
+            payload.push(0);
+        }
+        record(d, (3 + payload.len() / 2) as u32, func, &payload);
+    }
+
+    fn words(values: &[u16]) -> Vec<u8> {
+        values.iter().flat_map(|v| v.to_le_bytes()).collect()
+    }
+
+    /// META_EXTTEXTOUT: y, x, count, options, string.
+    fn ext_text(y: i16, x: i16, text: &str) -> Vec<u8> {
+        let mut b = words(&[y as u16, x as u16, text.len() as u16, 0]);
+        b.extend_from_slice(text.as_bytes());
+        if b.len() % 2 == 1 {
+            b.push(0);
+        }
+        b
+    }
+
+    fn filler(d: &mut Vec<u8>, byte: u8, n: usize) {
+        d.extend(std::iter::repeat_n(byte, n));
+    }
+
+    #[test]
+    fn text_counts_may_not_read_past_their_own_record() {
+        // A header-only EXTTEXTOUT (declared size 3 words) whose count and
+        // string live in the loose bytes after it: before the record bound,
+        // six declared bytes retained a 0x4141-sized string of attacker
+        // bytes.
+        let mut d = shell();
+        full_record(&mut d, 0x0A32, ext_text(20, 40, "kept"));
+        record(&mut d, 3, 0x0A32, b"");
+        filler(&mut d, b'A', 40_000);
+        let (_, _, _, texts) = render(&d).expect("wmf");
+        assert_eq!(texts.len(), 1, "{texts:?}");
+        assert_eq!(texts[0].text, "kept");
+
+        // Four payload words declared, but options demand an optional
+        // rectangle and a string past the record's end.
+        let mut d = shell();
+        full_record(&mut d, 0x0A32, ext_text(20, 40, "kept"));
+        full_record(&mut d, 0x0A32, words(&[20, 40, 0x4141, 0x0006]));
+        filler(&mut d, b'A', 40_000);
+        let (_, _, _, texts) = render(&d).expect("wmf");
+        assert_eq!(texts.len(), 1, "{texts:?}");
+
+        // TEXTOUT: the count itself comes from the bytes after the record.
+        let mut d = shell();
+        let mut good = words(&[3]);
+        good.extend_from_slice(b"ABC0");
+        good.extend_from_slice(&words(&[20, 40]));
+        full_record(&mut d, 0x0521, good);
+        record(&mut d, 3, 0x0521, b"");
+        filler(&mut d, b'B', 40_000);
+        let (_, _, _, texts) = render(&d).expect("wmf");
+        assert_eq!(texts.len(), 1, "{texts:?}");
+        assert_eq!(texts[0].text, "ABC");
+    }
+
+    #[test]
+    fn geometry_counts_may_not_read_past_their_own_record() {
+        let sample = |rgb: &[u8], w: u32, h: u32, x: u32, y: u32| {
+            let i = ((y * h / 100) * w + x * w / 100) as usize * 3;
+            [rgb[i], rgb[i + 1], rgb[i + 2]]
+        };
+        // POLYGON: the count word is in the record, its 60000 points are
+        // not. Before the bound, the reads walked into trailing bytes and a
+        // truncated read failed the whole raster.
+        let mut d = shell();
+        full_record(&mut d, 0x02FC, words(&[0, u16::from_le_bytes([3, 4]), u16::from(5), 0]));
+        full_record(&mut d, 0x012D, words(&[0]));
+        full_record(&mut d, 0x041B, words(&[80, 80, 20, 20]));
+        record(&mut d, 4, 0x0324, &words(&[60000]));
+        filler(&mut d, 0, 700);
+        let (w, h, rgb, texts) = render(&d).expect("prior paint survives a lying polygon");
+        assert_eq!(sample(&rgb, w, h, 50, 50), [3, 4, 5]);
+        assert!(texts.is_empty());
+
+        // POLYPOLYGON: the ring table's first count lives past the record.
+        let mut d = shell();
+        full_record(&mut d, 0x02FC, words(&[0, u16::from_le_bytes([3, 4]), u16::from(5), 0]));
+        full_record(&mut d, 0x012D, words(&[0]));
+        full_record(&mut d, 0x041B, words(&[80, 80, 20, 20]));
+        record(&mut d, 4, 0x0538, &words(&[1]));
+        filler(&mut d, 0xFF, 2);
+        filler(&mut d, 0, 700);
+        let (w, h, rgb, _) = render(&d).expect("prior paint survives a lying polypolygon");
+        assert_eq!(sample(&rgb, w, h, 50, 50), [3, 4, 5]);
+
+        // POLYLINE: same shape (the initial DC carries a black pen).
+        let mut d = shell();
+        full_record(&mut d, 0x041B, words(&[80, 80, 20, 20]));
+        record(&mut d, 4, 0x0325, &words(&[60000]));
+        filler(&mut d, 0, 700);
+        let (w, h, rgb, _) = render(&d).expect("prior paint survives a lying polyline");
+        assert_eq!(sample(&rgb, w, h, 50, 50), [255, 255, 255]);
+    }
+
+    #[test]
+    fn repeated_short_records_cannot_amplify_trailing_bytes() {
+        // Alternating six-byte size-3 EXTTEXTOUT headers over loose 'A's:
+        // each record's options word is the next header's size (3), whose
+        // optional-rectangle branch put the string start past every record.
+        // Before the bound, 240 declared bytes retained 40 × 2610-char
+        // strings of the trailing filler.
+        let mut d = shell();
+        full_record(&mut d, 0x0A32, ext_text(20, 40, "kept"));
+        for _ in 0..40 {
+            record(&mut d, 3, 0x0A32, b"");
+        }
+        filler(&mut d, b'A', 40_000);
+        let (_, _, _, texts) = render(&d).expect("wmf");
+        assert_eq!(texts.len(), 1, "{texts:?}");
+        assert_eq!(texts[0].text, "kept");
+    }
+}
+
 #[cfg(test)]
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod emf_text_tests {
@@ -2024,20 +2265,25 @@ mod coverage_boundary_tests {
 
     #[test]
     fn font_records_preserve_cell_height_and_limit_unterminated_face_names() {
-        assert_eq!(WmfFont::parse(&[0; 17], 0), None);
+        assert_eq!(WmfFont::parse(&[0; 17], 0, 17), None);
         let mut d = vec![0; 18];
         d[..2].copy_from_slice(&(-12i16).to_le_bytes());
         d[8..10].copy_from_slice(&700i16.to_le_bytes());
         d[10] = 1;
         d.extend_from_slice(b"Arial\0ignored");
-        let font = WmfFont::parse(&d, 0).unwrap();
+        let font = WmfFont::parse(&d, 0, d.len()).unwrap();
         assert_eq!(
             (font.height, font.weight, font.italic, font.face.as_str()),
             (-12, 700, true, "Arial")
         );
         d.truncate(18);
         d.extend_from_slice(&[b'X'; 40]);
-        assert_eq!(WmfFont::parse(&d, 0).unwrap().face, "X".repeat(32));
+        assert_eq!(WmfFont::parse(&d, 0, d.len()).unwrap().face, "X".repeat(32));
+        // A record that declares fewer face bytes than the file holds reads
+        // only its own: the face stops at the record end.
+        let mut short = vec![0; 18];
+        short.extend_from_slice(b"Arial\0ignored");
+        assert_eq!(WmfFont::parse(&short, 0, 18 + 5).unwrap().face, "Arial");
         assert_eq!(ansi_text(&[b'A', 0x80, 0x91, 0x92, 0x97, 0xff]), "A€‘’—ÿ");
         assert_eq!(colorref(0xff123456), [0x56, 0x34, 0x12]);
     }
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -63,6 +63,17 @@ pub(crate) const MAX_NAMESPACE_PREFIX_BYTES: usize = 256;
 pub(crate) const MAX_LIVE_NS_BINDINGS: usize = 256;
 /// Total prefix+URI bytes declared by xmlns attributes in one document.
 pub(crate) const MAX_NS_DECLARED_BYTES: usize = 1024 * 1024;
+/// Open elements the permissive DOM parser builds before it stops
+/// descending. Admission and the strict gate refuse nesting beyond
+/// `InputLimits::max_xml_depth` (256 by default) before this parser runs;
+/// this cap is the parser's own backstop for the paths that reach it
+/// unadmitted — relationship-selected auxiliary parts under arbitrary
+/// names (theme, styles, numbering, settings, notes, linked text boxes,
+/// diagrams) — so DOM construction can never overflow the stack, on a
+/// native thread or a wasm32 browser stack, whatever the part is called.
+/// Nesting at or beyond the cap stops construction and leaves the rest of
+/// that part unparsed.
+pub(crate) const MAX_DOM_CONSTRUCTION_DEPTH: usize = 1024;
 
 // N02 mechanism counter: bytes of (prefix, uri) pairs the parser's
 // namespace scope has recorded. The scoped-overlay scope only ever records
@@ -90,6 +101,9 @@ fn bump_ns_scope_declared_bytes(by: usize) {
 /// This parser does not enforce input policy or panic on policy violations.
 /// Run [`validate_xml_with_limits`] (or package admission) with the active
 /// limits first, or call [`try_parse_xdocument_with_limits`] to receive errors.
+/// One bound it does enforce itself: construction stops descending at
+/// [`MAX_DOM_CONSTRUCTION_DEPTH`] open elements, so even a caller that
+/// skipped every gate cannot overflow the stack through this parser.
 /// Attributes retain their order; repeated expanded names keep the first
 /// position and the last value. Bulk attribute insertion is linear.
 ///
@@ -543,7 +557,7 @@ impl<'a> Parser<'a> {
                 continue;
             }
             if self.cur() == '<' {
-                let el = self.parse_element(&mut scope);
+                let el = self.parse_element(&mut scope, 1);
                 self.dom.add(doc, el);
                 continue;
             }
@@ -552,7 +566,8 @@ impl<'a> Parser<'a> {
         doc
     }
 
-    fn parse_element(&mut self, scope: &mut NsScope) -> NodeId {
+    /// `depth` is this element's own nesting level (a root is 1).
+    fn parse_element(&mut self, scope: &mut NsScope, depth: usize) -> NodeId {
         // at '<'
         self.pos += 1; // consume '<'
         let raw_name = self.read_name();
@@ -658,7 +673,15 @@ impl<'a> Parser<'a> {
                 continue;
             }
             if self.cur() == '<' {
-                let child = self.parse_element(scope);
+                if depth >= MAX_DOM_CONSTRUCTION_DEPTH {
+                    // Unadmitted nesting: stop building and leave the rest
+                    // of this part unparsed, rather than recurse without a
+                    // bound. Admitted documents never reach this — their
+                    // depth gate is tighter than this cap.
+                    self.pos = self.len();
+                    break;
+                }
+                let child = self.parse_element(scope, depth + 1);
                 self.dom.add(el, child);
                 continue;
             }
@@ -1678,3 +1701,70 @@ mod tag_policy_tests {
         );
     }
 }
+
+#[cfg(test)]
+#[cfg_attr(all(coverage_nightly, test), coverage(off))]
+mod dom_construction_depth_tests {
+    //! The permissive parser bounds its own recursion: a part that reaches
+    //! it without admission's depth gate (a renamed `styles.bin` selected
+    //! by relationship) stops descending at the construction cap instead of
+    //! recursing once per element.
+
+    use super::*;
+
+    fn nesting(dom: &Dom, document: NodeId) -> usize {
+        fn walk(dom: &Dom, id: NodeId, depth: usize) -> usize {
+            dom.elements(id, None)
+                .iter()
+                .map(|&child| walk(dom, child, depth + 1))
+                .max()
+                .unwrap_or(depth)
+        }
+        dom.root(document).map_or(0, |root| walk(dom, root, 1))
+    }
+
+    #[test]
+    fn unchecked_deep_input_stops_descending_at_the_construction_cap() {
+        let xml = "<a>".repeat(MAX_DOM_CONSTRUCTION_DEPTH + 500);
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        assert_eq!(
+            nesting(&dom, document),
+            MAX_DOM_CONSTRUCTION_DEPTH,
+            "construction must stop at the cap, not parse all {} levels",
+            MAX_DOM_CONSTRUCTION_DEPTH + 500
+        );
+    }
+
+    #[test]
+    fn ordinary_and_admitted_depths_parse_completely() {
+        // 256 levels is the depth admission still admits by default: the
+        // parser must build every one of them, and their text.
+        let deep = format!(
+            "<r>{}<leaf>owned</leaf></r>{}",
+            "<a>".repeat(255),
+            "</a>".repeat(255)
+        );
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &deep);
+        assert_eq!(nesting(&dom, document), 257);
+        let leaf = {
+            let mut cur = dom.root(document).unwrap();
+            for _ in 0..255 {
+                cur = dom.element(cur, &XName::get("a", "")).unwrap();
+            }
+            dom.element(cur, &XName::get("leaf", "")).unwrap()
+        };
+        assert_eq!(dom.value(leaf), "owned");
+        // Siblings after a deep-but-legal subtree still parse.
+        let xml = format!("<r>{}<after/></r>", "<a>".repeat(200));
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &xml);
+        let root = dom.root(document).unwrap();
+        assert!(dom.element(root, &XName::get("after", "")).is_some());
+        // The checked gate still refuses nesting above its own limit long
+        // before the parser's cap is reached.
+        let refused = format!("{}{}", "<a>".repeat(257), "</a>".repeat(257));
+        assert!(validate_xml(&refused).is_err());
+    }
+}
```
