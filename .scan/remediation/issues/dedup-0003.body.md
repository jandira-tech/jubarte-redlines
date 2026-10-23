@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_64b247931cd1f053217fde7a | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_3201e6caaef21850d61fa69c | WMF record-boundary failures amplify polygon work and retained text | medium
csf_de54caf2d47d7e03bce741e0 | Metadata scrub leaves property parts under alternate names | medium
csf_d22452ac5ec901578d340334 | Opaque DTD and top-level CDATA bypass package XML depth checks | medium
csf_4de0d1e5f809171f3b8323d9 | Repeated HTML chunk references bypass conversion memory budgets | medium
csf_550f8cf900e415a773aa959d | Repeated HTML chunk references bypass conversion memory budgets | medium
csf_8510b5bc0ddac96422dfcdc6 | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_4b64ae60e06cc87420036fe5 | Metadata scrub leaves property parts under alternate names | medium
csf_7084a83c24deb556942e2d43 | WMF record-boundary failures amplify polygon work and retained text | medium
csf_f756bbe5509c42bdcaa64921 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_32625277093e7f286a267e2b | Rendering an HTML chunk can allocate gigabytes from a single colspan value | medium
csf_80edacafa6b67d4864116a38 | WMF record-boundary failures amplify polygon work and retained text | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/src/admission.rs b/src/admission.rs
index d556610..4accb49 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -785,7 +785,20 @@ fn part_text(xml: &[u8]) -> Cow<'_, str> {
 /// inside the tag/namespace policy (`xmllinq::parse::TagPolicy`) so no part
 /// reaching the permissive DOM parser can carry pathological attribute or
 /// namespace-declaration widths.
-fn check_xml_depth(name: &str, xml: &[u8], limits: InputLimits) -> Result<(), AdmissionError> {
+///
+/// Package admission applies this to every `.xml`/`.rels` part and to the
+/// main part whatever its name. The consumers that load a part as XML by
+/// relationship (a renamed theme, notes or comments part) call it directly,
+/// so a part cannot skip the package's XML policy by its file name alone.
+/// A DOCTYPE is refused here exactly as the checked gate
+/// (`validate_xml_with_limits`) refuses it, and so is a CDATA section
+/// outside the root: both carry markup-shaped bytes the permissive parser
+/// must never turn into elements.
+pub(crate) fn check_xml_depth(
+    name: &str,
+    xml: &[u8],
+    limits: InputLimits,
+) -> Result<(), AdmissionError> {
     let invalid =
         |why: String| AdmissionError::new(AdmissionErrorKind::InvalidXml, format!("{name}: {why}"));
     let text = part_text(xml);
@@ -812,6 +825,12 @@ fn check_xml_depth(name: &str, xml: &[u8], limits: InputLimits) -> Result<(), Ad
                     .ok_or_else(|| invalid("end tag with no open element".to_string()))?;
                 policy.close();
             }
+            Ok(Event::DocType(_)) => {
+                return Err(invalid("DTD declarations are unsupported".to_string()));
+            }
+            Ok(Event::CData(_)) if depth == 0 => {
+                return Err(invalid("CDATA outside XML root".to_string()));
+            }
             Ok(Event::Eof) if depth > 0 => {
                 return Err(invalid(format!("{depth} element(s) left unclosed")));
             }
@@ -1784,6 +1803,83 @@ mod tests {
         );
     }
 
+    /// A DOCTYPE was one ignored event for the admission scan while the
+    /// checked gate (`validate_xml`) refused it. A quoted internal-subset
+    /// literal can carry arbitrary element-like markup, so admission must
+    /// refuse a DTD too, whatever part holds it.
+    #[test]
+    fn a_dtd_in_the_main_part_is_refused_like_the_checked_gate() {
+        let dtd = concat!(
+            r#"<!DOCTYPE w:document [ <!ENTITY deep "<a><b><c>x</c></b></a>"> ]>"#,
+            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#
+        );
+        let entries: Vec<(&str, &[u8])> = vec![
+            ("[Content_Types].xml", TYPES.as_bytes()),
+            ("_rels/.rels", RELS.as_bytes()),
+            ("word/document.xml", dtd.as_bytes()),
+        ];
+        let bytes = zip_of(&entries, CompressionMethod::Deflated);
+        assert_eq!(
+            kind(&bytes, InputLimits::default()),
+            AdmissionErrorKind::InvalidXml
+        );
+        let err =
+            check_xml_depth("word/document.xml", dtd.as_bytes(), InputLimits::default())
+                .expect_err("the scan itself refuses the DTD");
+        assert!(
+            err.message.contains("DTD declarations are unsupported"),
+            "{}",
+            err.message
+        );
+    }
+
+    /// A CDATA section before the root is the checked gate's other opaque
+    /// refusal, and its body can carry `]` before a `>` followed by
+    /// element-like markup; the permissive parser must never see it as
+    /// elements, so admission refuses it like the gate. In-root CDATA stays
+    /// admitted.
+    #[test]
+    fn a_top_level_cdata_in_the_main_part_is_refused_like_the_checked_gate() {
+        let cdata = concat!(
+            "<![CDATA[a]><d><e>deep markup</e></d>]]>",
+            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#
+        );
+        let entries: Vec<(&str, &[u8])> = vec![
+            ("[Content_Types].xml", TYPES.as_bytes()),
+            ("_rels/.rels", RELS.as_bytes()),
+            ("word/document.xml", cdata.as_bytes()),
+        ];
+        let bytes = zip_of(&entries, CompressionMethod::Deflated);
+        assert_eq!(
+            kind(&bytes, InputLimits::default()),
+            AdmissionErrorKind::InvalidXml
+        );
+        let err =
+            check_xml_depth("word/document.xml", cdata.as_bytes(), InputLimits::default())
+                .expect_err("the scan itself refuses the top-level CDATA");
+        assert!(
+            err.message.contains("CDATA outside XML root"),
+            "{}",
+            err.message
+        );
+        let rooted = concat!(
+            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
+            "<![CDATA[kept raw]]></w:document>"
+        );
+        let rooted_bytes = zip_of(
+            &[
+                ("[Content_Types].xml", TYPES.as_bytes()),
+                ("_rels/.rels", RELS.as_bytes()),
+                ("word/document.xml", rooted.as_bytes()),
+            ],
+            CompressionMethod::Deflated,
+        );
+        assert!(
+            admit(&rooted_bytes, InputLimits::default()).is_ok(),
+            "in-root CDATA is Word-valid and stays admitted"
+        );
+    }
+
     #[test]
     fn mismatched_unclosed_and_stray_end_tags_are_refused() {
         for (name, xml) in [
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..19cde01 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -10,10 +10,39 @@
 use std::collections::HashMap;
 use std::fmt::Write as _;
 
+/// Word's own table limit: a Word table has at most 63 columns. A cell
+/// span (or a row's summed spans) beyond it is clamped to the grid maximum
+/// instead of emitting one `w:gridCol` per declared column — a single
+/// authored colspan used to demand billions of columns of markup.
+const MAX_GRID_COLUMNS: usize = 63;
+
 /// `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
 /// `load` returns a chunk's bytes by relationship id.
 pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> String {
+    expand_within(xml, load, default_expansion_budget())
+}
+
+/// The cumulative generated-WML budget of [`expand`]: the default package
+/// admission aggregate. One chunk may be referenced many times, so the
+/// expanded story is bounded by this budget rather than by the original
+/// package's inflated size.
+fn default_expansion_budget() -> usize {
+    usize::try_from(crate::admission::InputLimits::default().max_uncompressed_bytes)
+        .unwrap_or(usize::MAX)
+}
+
+/// [`expand`] under an explicit cumulative budget for the generated WML:
+/// once the converted chunks have spent it, later `w:altChunk` references
+/// contribute nothing (their tags are still consumed and the rest of the
+/// document is kept), so repeated references to one modest part cannot
+/// expand a small package past every admission budget.
+pub(crate) fn expand_within(
+    xml: &str,
+    load: impl Fn(&str) -> Option<Vec<u8>>,
+    budget: usize,
+) -> String {
     let mut out = String::with_capacity(xml.len());
+    let mut spent = 0usize;
     let mut rest = xml;
     while let Some(at) = rest.find("<w:altChunk") {
         out.push_str(&rest[..at]);
@@ -31,12 +60,15 @@ pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> Strin
             tail.find("</w:altChunk>")
                 .map_or(open_end + 1, |i| i + "</w:altChunk>".len())
         };
-        let wml = attr(open, "id")
-            .and_then(|rid| load(&rid))
-            .and_then(|bytes| chunk_html(&bytes))
-            .map(|html| html_to_wml(&html))
-            .unwrap_or_default();
-        out.push_str(&wml);
+        if spent <= budget {
+            let wml = attr(open, "id")
+                .and_then(|rid| load(&rid))
+                .and_then(|bytes| chunk_html(&bytes))
+                .map(|html| html_to_wml(&html))
+                .unwrap_or_default();
+            spent += wml.len();
+            out.push_str(&wml);
+        }
         rest = &tail[end..];
     }
     out.push_str(rest);
@@ -469,10 +501,10 @@ impl Builder {
         let cols = table
             .rows
             .iter()
-            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
+            .map(|r| r.iter().fold(0usize, |sum, c| sum.saturating_add(c.span)))
             .max()
             .unwrap_or(1)
-            .max(1);
+            .clamp(1, MAX_GRID_COLUMNS);
         let col_w = 9360 / cols;
         let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
         if table.border {
@@ -563,7 +595,7 @@ impl Builder {
                     attr(open, "colspan")
                         .and_then(|s| s.trim().parse::<usize>().ok())
                         .unwrap_or(1)
-                        .max(1),
+                        .clamp(1, MAX_GRID_COLUMNS),
                 );
                 let mut fmt = self.fmt();
                 if tag == "th" {
@@ -1050,10 +1082,67 @@ mod tests {
                     "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
                 ));
                 assert_eq!(wml.matches("<w:gridCol ").count(), 1, "{span}: {wml}");
-                assert!(!wml.contains("<w:gridSpan"), "{wml}");
+                assert!(!wml.contains("<w:gridSpan"), "{span}");
                 assert!(!wml.contains("<w:tblBorders>"));
             }
         }
+
+        /// A colspan (or a row's summed spans) beyond Word's own 63-column
+        /// table limit is clamped to the grid maximum instead of emitting
+        /// one `w:gridCol` per declared column: a single authored value
+        /// used to demand billions of columns of markup before any parser
+        /// or output budget could act.
+        #[test]
+        fn a_colspan_beyond_words_column_limit_clamps_the_grid() {
+            for span in ["64", "1000", "4294967295"] {
+                let wml = html_to_wml(&format!(
+                    "<table><tr><td colspan='{span}'>A</td></tr></table>"
+                ));
+                assert_eq!(
+                    wml.matches("<w:gridCol ").count(),
+                    MAX_GRID_COLUMNS,
+                    "{span}: {wml}"
+                );
+                assert!(
+                    wml.contains("<w:gridSpan w:val=\"63\"/>"),
+                    "{span}: the cell keeps Word's widest legal span: {wml}"
+                );
+            }
+            // A row of many authored cells saturates the span sum; the grid
+            // stays within Word's column limit while every cell survives.
+            let html = format!("<table><tr>{}</tr></table>", "<td>A</td>".repeat(200));
+            let wml = html_to_wml(&html);
+            assert_eq!(wml.matches("<w:gridCol ").count(), MAX_GRID_COLUMNS);
+            assert_eq!(wml.matches("<w:tc>").count(), 200, "{wml}");
+        }
+
+        /// One modest chunk referenced repeatedly is converted once per
+        /// reference and appended to one story; the cumulative generated
+        /// WML is bounded by the expansion budget, not by the package's
+        /// original size.
+        #[test]
+        fn repeated_chunk_references_stop_at_the_expansion_budget() {
+            let xml = "<w:body><w:altChunk r:id='c'/><w:altChunk r:id='c'/><w:sectPr/></w:body>";
+            let out = expand_within(xml, |rid| (rid == "c").then(|| b"<p>Hello</p>".to_vec()), 20);
+            assert!(
+                out.starts_with("<w:body>") && out.ends_with("<w:sectPr/></w:body>"),
+                "the document's own markup is kept: {out}"
+            );
+            assert_eq!(
+                out.matches("Hello").count(),
+                1,
+                "the first conversion already spends a 20-byte budget: {out}"
+            );
+        }
+
+        /// The cap is a budget, not a cache: repeated references well inside
+        /// it all expand.
+        #[test]
+        fn repeated_references_under_the_default_budget_all_expand() {
+            let xml = "<w:body><w:altChunk r:id='c'/><w:altChunk r:id='c'/></w:body>";
+            let out = expand(xml, |rid| (rid == "c").then(|| b"<p>Hi</p>".to_vec()));
+            assert_eq!(out.matches("Hi").count(), 2, "{out}");
+        }
     }
 }
 
diff --git a/src/convert/metafile.rs b/src/convert/metafile.rs
index c1c75da..3a78f6f 100644
--- a/src/convert/metafile.rs
+++ b/src/convert/metafile.rs
@@ -487,6 +487,12 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
             break;
         }
         let payload = off + 6;
+        // The record's own bytes end here: the handlers below read counts
+        // and arrays only inside this extent, so a short record cannot
+        // consume — and force the outer loop to revisit — the bytes of the
+        // records after it (the declared size, not the file, bounds every
+        // count).
+        let rec_end = off + size2;
         match func {
             0x0000 => break,
             0x020B => {
@@ -606,17 +612,31 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 let n = usize::from(read_u16(data, payload + 4)?);
                 let opts = read_u16(data, payload + 6)?;
                 let at = payload + 8 + if opts & 0x0006 != 0 { 8 } else { 0 };
-                if let Some(bytes) = data.get(at..at + n) {
+                // The string belongs to this record: one whose declared
+                // extent cannot hold it is refused rather than read across
+                // the next records' bytes (and the outer loop then revisits
+                // them).
+                if let Some(end) = at.checked_add(n)
+                    && end <= rec_end
+                    && let Some(bytes) = data.get(at..end)
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
-                    texts.extend(wmf_text(&dc, &map, x, y, bytes));
+                let padded = payload + 2 + n.next_multiple_of(2);
+                // The padded string plus the trailing y and x must fit the
+                // record's declared extent before anything is retained.
+                if let Some(end) = padded.checked_add(4)
+                    && end <= rec_end
+                    && let Some(bytes) = data.get(payload + 2..payload + 2 + n)
+                {
+                    if let (Some(y), Some(x)) = (read_i16(data, padded), read_i16(data, padded + 2))
+                    {
+                        texts.extend(wmf_text(&dc, &map, x, y, bytes));
+                    }
                 }
             }
             0x01F0 => {
@@ -627,37 +647,69 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
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
+                // The point array must fit the record's declared extent
+                // before any of it is decoded or allocated.
+                if let Some(end) =
+                    (payload + 2).checked_add(n.checked_mul(4).unwrap_or(usize::MAX))
+                    && end <= rec_end
+                {
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
+                // The ring counts, and every ring's points, must fit the
+                // record's declared extent before any ring is allocated.
+                let mut needed = payload + 2 + 2 * rings;
+                let mut fits = needed <= rec_end;
+                if fits {
+                    for r in 0..rings {
+                        let Some(count) = read_u16(data, payload + 2 + 2 * r) else {
+                            fits = false;
+                            break;
+                        };
+                        needed = match needed.checked_add(usize::from(count) * 4) {
+                            Some(total) => total,
+                            None => {
+                                fits = false;
+                                break;
+                            }
+                        };
+                        if needed > rec_end {
+                            fits = false;
+                            break;
+                        }
                     }
-                    subpaths.push(ring);
                 }
-                if let Some(fill) = dc.brush {
-                    canvas.fill_path(&subpaths, fill, winding);
+                if fits {
+                    let mut p = payload + 2 + 2 * rings;
+                    let mut subpaths = Vec::with_capacity(rings);
+                    for r in 0..rings {
+                        let n = read_u16(data, payload + 2 + 2 * r)? as usize;
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
@@ -668,15 +720,22 @@ fn raster_wmf(data: &[u8]) -> Option<(u32, u32, Vec<u8>, Vec<MetaText>)> {
                 let n = read_u16(data, payload).unwrap_or(0) as usize;
                 let mut prev: Option<(i32, i32)> = None;
                 let mut p = payload + 2;
-                for _ in 0..n {
-                    let x = read_i16(data, p)? as i32;
-                    let y = read_i16(data, p + 2)? as i32;
-                    let cur = map.map(x, y);
-                    if let Some(pr) = prev {
-                        canvas.stroke_line(pr.0, pr.1, cur.0, cur.1, pen, map.pen_px(pen_w));
+                // The point array must fit the record's declared extent
+                // before any segment is stroked.
+                if let Some(end) =
+                    (payload + 2).checked_add(n.checked_mul(4).unwrap_or(usize::MAX))
+                    && end <= rec_end
+                {
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
@@ -1572,6 +1631,80 @@ mod wmf_object_tests {
         (0x02FC, vec![0, u16::from_le_bytes([r, g]), u16::from(b), 0])
     }
 
+    /// `records` with record `lie`'s declared size shrunk to the minimum
+    /// three words: the payload bytes stay in place, so a handler that
+    /// trusts its counts instead of the declared extent reads across the
+    /// records that follow — and the outer loop revisits them.
+    fn wmf_with_lying_size(records: &[(u16, Vec<u16>)], nobj: u16, lie: usize) -> Vec<u8> {
+        let mut d = wmf(records, nobj);
+        let mut off = 40;
+        for (i, (_, params)) in records.iter().enumerate() {
+            if i == lie {
+                d[off..off + 4].copy_from_slice(&3u32.to_le_bytes());
+                return d;
+            }
+            off += (3 + params.len()) * 2;
+        }
+        panic!("record {lie} not found among {} records", records.len());
+    }
+
+    #[test]
+    fn a_polygon_record_lying_about_its_size_does_not_paint_across_records() {
+        let records = [
+            brush(0, 0, 255),
+            (0x012D, vec![0]),
+            // A triangle whose points live in this record's own bytes.
+            (0x0324, vec![3, 10, 10, 90, 10, 90, 90]),
+        ];
+        let honest = wmf(&records, 2);
+        let (w, h, px) = rasterize(&honest).expect("honest wmf");
+        let at = |x: u32, y: u32, w: u32, h: u32, px: &[u8]| {
+            let i = ((y * h / 100) * w + x * w / 100) as usize * 3;
+            [px[i], px[i + 1], px[i + 2]]
+        };
+        assert_eq!(
+            at(80, 20, w, h, &px),
+            [0, 0, 255],
+            "an honest record fills the triangle"
+        );
+        assert_eq!(at(5, 5, w, h, &px), [255, 255, 255], "outside stays white");
+
+        // The same bytes with the polygon's declared size shrunk to the
+        // minimum: the points no longer fit the record's extent, so the
+        // polygon is refused — not painted out of the next records' bytes.
+        let lying = wmf_with_lying_size(&records, 2, 2);
+        let (w, h, px) = rasterize(&lying).expect("the lying wmf still rasterizes");
+        assert_eq!(
+            at(80, 20, w, h, &px),
+            [255, 255, 255],
+            "the refused polygon paints nothing"
+        );
+    }
+
+    #[test]
+    fn an_exttextout_record_lying_about_its_size_retains_no_text() {
+        let records = [
+            (0x020B, vec![0, 0]),
+            (0x020C, vec![200, 400]),
+            font(-25, 400, false, "Times New Roman"),
+            (0x012D, vec![0]),
+            ext_text(40, 20, "Retained"),
+        ];
+        let honest = wmf(&records, 1);
+        let (_, _, _, texts) = render(&honest).expect("honest wmf");
+        assert_eq!(
+            texts.len(),
+            1,
+            "an honest record keeps its text: {texts:?}"
+        );
+        let lying = wmf_with_lying_size(&records, 1, 4);
+        let (_, _, _, texts) = render(&lying).expect("the lying wmf still rasterizes");
+        assert!(
+            texts.is_empty(),
+            "text beyond a record's declared extent must not be retained: {texts:?}"
+        );
+    }
+
     #[test]
     fn a_polypolygon_fills_with_the_brush_selected_by_zero_based_index() {
         let square = vec![1, 4, 10, 10, 90, 10, 90, 90, 10, 90];
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..7d07644 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -3306,6 +3306,7 @@ fn load_theme(pkg: &PartFs) -> ThemeFonts {
     let Some(xml) = pkg
         .part_string(&main_rel_part(pkg, "theme", "word/theme/theme1.xml"))
         .or_else(|| pkg.part_string("word/theme/theme2.xml"))
+        .filter(|xml| part_xml_admissible("theme part", xml))
     else {
         return ThemeFonts::default();
     };
@@ -3477,7 +3478,10 @@ fn load_stylesheet(pkg: &PartFs) -> StyleSheet {
     defaults.wp_justify = settings_flag(pkg, "wpJustification");
     defaults.para.sum_spacing = settings_flag(pkg, "doNotUseHTMLParagraphAutoSpacing");
     let mut raw: std::collections::HashMap<String, RawStyle> = std::collections::HashMap::new();
-    let Some(xml) = pkg.part_string(&main_rel_part(pkg, "styles", "word/styles.xml")) else {
+    let Some(xml) = pkg
+        .part_string(&main_rel_part(pkg, "styles", "word/styles.xml"))
+        .filter(|xml| part_xml_admissible("styles part", xml))
+    else {
         return StyleSheet {
             defaults,
             by_id: HashMap::new(),
@@ -6556,7 +6560,10 @@ fn bullet_glyph(raw: &str) -> String {
 
 fn load_numbering(pkg: &PartFs) -> Numbering {
     let numbering_part = main_rel_part(pkg, "numbering", "word/numbering.xml");
-    let Some(xml) = pkg.part_string(&numbering_part) else {
+    let Some(xml) = pkg
+        .part_string(&numbering_part)
+        .filter(|xml| part_xml_admissible("numbering part", xml))
+    else {
         return Numbering::default();
     };
     parse_numbering_xml(&xml, |rid| resolve_media(pkg, &numbering_part, rid))
@@ -7666,8 +7673,10 @@ fn comments_resolved(pkg: &PartFs, main: &str) -> HashSet<String> {
 /// parent (by comment id), and the resolved comments.
 fn comments_extended(pkg: &PartFs, main: &str) -> (HashMap<String, String>, HashSet<String>) {
     let (Some(comments), Some(extended)) = (
-        part_xml_by_rel_kind(pkg, main, "comments"),
-        part_xml_by_rel_kind(pkg, main, "commentsExtended"),
+        part_xml_by_rel_kind(pkg, main, "comments")
+            .filter(|xml| part_xml_admissible("comments part", xml)),
+        part_xml_by_rel_kind(pkg, main, "commentsExtended")
+            .filter(|xml| part_xml_admissible("commentsExtended part", xml)),
     ) else {
         return (HashMap::new(), HashSet::new());
     };
@@ -7779,7 +7788,9 @@ struct CommentRec {
 }
 
 fn load_comments(pkg: &PartFs, main: &str) -> HashMap<String, CommentRec> {
-    let Some(xml) = part_xml_by_rel_kind(pkg, main, "comments") else {
+    let Some(xml) = part_xml_by_rel_kind(pkg, main, "comments")
+        .filter(|xml| part_xml_admissible("comments part", xml))
+    else {
         return HashMap::new();
     };
     let mut dom = Dom::new();
@@ -8488,6 +8499,22 @@ fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String>
     pkg.part_string(&format!("word/{kind}.xml"))
 }
 
+/// A relationship-selected part's XML passes the same structural policy
+/// package admission applies to `.xml`/`.rels` parts — nesting depth, tag
+/// budgets and end-name balance, with the tolerant attribute rules —
+/// whatever the part is named. A renamed theme, notes or comments part
+/// must not reach the permissive DOM parser just because its file name
+/// skipped the extension gate; a part that refuses renders as absent (the
+/// same default as a missing part).
+fn part_xml_admissible(name: &str, xml: &str) -> bool {
+    crate::admission::check_xml_depth(
+        name,
+        xml.as_bytes(),
+        crate::admission::InputLimits::compare(),
+    )
+    .is_ok()
+}
+
 /// Separator / continuation notes are chrome, not note text. Since plan
 /// Step 7, `paint_page_footnotes` draws the separator itself: a
 /// `FOOTNOTE_SEP_PT` rule from `margin_l` to `margin_l + FOOTNOTE_SEP_W`
@@ -8521,7 +8548,9 @@ impl EndnoteBag {
             pending: Vec::new(),
             seen: HashSet::new(),
         };
-        let Some(xml) = part_xml_by_rel_kind(pkg, main, "endnotes") else {
+        let Some(xml) = part_xml_by_rel_kind(pkg, main, "endnotes")
+            .filter(|xml| part_xml_admissible("endnotes part", xml))
+        else {
             return bag;
         };
         let doc = bag.ndom.parse_xdocument(&xml);
@@ -8693,7 +8722,9 @@ fn load_footnotes(
     main: &str,
     sheet: &StyleSheet,
 ) -> (HashMap<String, Vec<FootnotePara>>, Vec<FootnotePara>) {
-    let Some(xml) = part_xml_by_rel_kind(pkg, main, "footnotes") else {
+    let Some(xml) = part_xml_by_rel_kind(pkg, main, "footnotes")
+        .filter(|xml| part_xml_admissible("footnotes part", xml))
+    else {
         return (HashMap::new(), Vec::new());
     };
     let mut ndom = Dom::new();
@@ -20791,6 +20822,9 @@ fn chrome_part_xml(
     sheet: &StyleSheet,
     text_w: f32,
 ) -> ChromePart {
+    if !part_xml_admissible(path, xml) {
+        return empty_chrome();
+    }
     let mut part_dom = Dom::new();
     let doc = part_dom.parse_xdocument(xml);
     let Some(root) = part_dom.root(doc) else {
@@ -37080,6 +37114,59 @@ mod theme_slot_tests {
         );
     }
 
+    /// A theme part under a non-`.xml` name (the relationship still names
+    /// it) skips admission's extension gate; the layout must hold it to
+    /// the same XML policy as a conventional theme instead of building a
+    /// DOM from arbitrarily deep markup. A part past the depth budget
+    /// keeps the default theme fonts, exactly like a missing part.
+    #[test]
+    fn a_renamed_theme_part_past_the_depth_budget_keeps_default_fonts() {
+        use std::io::Write as _;
+
+        let theme = |wrap: usize| {
+            format!(
+                r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">{}<a:themeElements><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri"/></a:majorFont></a:fontScheme></a:themeElements>{}</a:theme>"#,
+                "<a:d>".repeat(wrap),
+                "</a:d>".repeat(wrap)
+            )
+        };
+        let package = |theme_xml: &str| {
+            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
+            let options = zip::write::SimpleFileOptions::default();
+            for (name, body) in [
+                ("[Content_Types].xml", "<Types/>".to_string()),
+                (
+                    "_rels/.rels",
+                    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#
+                        .to_string(),
+                ),
+                ("word/document.xml", "<w:document/>".to_string()),
+                (
+                    "word/_rels/document.xml.rels",
+                    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme.bin"/></Relationships>"#
+                        .to_string(),
+                ),
+                ("word/theme.bin", theme_xml.to_string()),
+            ] {
+                writer.start_file(name, options).unwrap();
+                writer.write_all(body.as_bytes()).unwrap();
+            }
+            PartFs::open(&writer.finish().unwrap().into_inner()).unwrap()
+        };
+        let shallow = package(&theme(0));
+        assert_eq!(
+            load_theme(&shallow).major.as_deref(),
+            Some("Calibri"),
+            "a renamed but well-formed theme still loads"
+        );
+        let deep = package(&theme(300));
+        assert_eq!(
+            load_theme(&deep).major,
+            None,
+            "a renamed theme past the depth budget is refused, not recursively parsed"
+        );
+    }
+
     #[test]
     fn load_theme_reads_minor_east_asia_typeface() {
         let xml = r#"<?xml version="1.0"?>
diff --git a/src/document_comparer.rs b/src/document_comparer.rs
index febe703..66db1f2 100644
--- a/src/document_comparer.rs
+++ b/src/document_comparer.rs
@@ -6320,7 +6320,13 @@ pub fn get_revisions(
         (fn_part.as_str(), W::footnote()),
         (en_part.as_str(), W::endnote()),
     ] {
-        if let Some(x) = pkg.part_string(part) {
+        // A notes part is whatever its relationship names — the extension
+        // gate never checked it — so the package XML policy is applied here
+        // under the caller's active limits before the DOM is built.
+        if let Some(x) = pkg.part_string(part)
+            && crate::admission::check_xml_depth(part, x.as_bytes(), settings.input_limits)
+                .is_ok()
+        {
             let nd = dom.parse_xdocument(&x);
             if let Some(r) = dom.root(nd) {
                 preprocess::remove_existing_powertools_markup(&mut dom, r);
@@ -6673,20 +6679,29 @@ fn compare_documents_impl(
     // into the same Dom as the bodies. The pipeline (B.2/B.3) diffs each
     // definition by its reference's correlation status and rebuilds the
     // withRevisions parts renumbered 1..n.
-    fn parse_part_root(dom: &mut Dom, pkg: &PartFs, name: &str) -> Option<NodeId> {
+    fn parse_part_root(
+        dom: &mut Dom,
+        pkg: &PartFs,
+        name: &str,
+        limits: crate::admission::InputLimits,
+    ) -> Option<NodeId> {
         let xml = pkg.part_string(name)?;
+        // A notes part is whatever its relationship names — the extension
+        // gate never checked it — so the package XML policy is applied here
+        // under the comparer's active limits before the DOM is built.
+        crate::admission::check_xml_depth(name, xml.as_bytes(), limits).ok()?;
         let d = dom.parse_xdocument(&xml);
         dom.root(d)
     }
     let (fn1, en1) = notes_part_names(&pkg1);
     let (fn2, en2) = notes_part_names(&pkg2);
     let mut notes_ctx = crate::comparer::NotesContext {
-        fn_before: parse_part_root(&mut dom, &pkg1, &fn1),
-        fn_after: parse_part_root(&mut dom, &pkg2, &fn2),
-        en_before: parse_part_root(&mut dom, &pkg1, &en1),
-        en_after: parse_part_root(&mut dom, &pkg2, &en2),
-        fn_with_revisions: parse_part_root(&mut dom, &pkg1, &fn1),
-        en_with_revisions: parse_part_root(&mut dom, &pkg1, &en1),
+        fn_before: parse_part_root(&mut dom, &pkg1, &fn1, settings.input_limits),
+        fn_after: parse_part_root(&mut dom, &pkg2, &fn2, settings.input_limits),
+        en_before: parse_part_root(&mut dom, &pkg1, &en1, settings.input_limits),
+        en_after: parse_part_root(&mut dom, &pkg2, &en2, settings.input_limits),
+        fn_with_revisions: parse_part_root(&mut dom, &pkg1, &fn1, settings.input_limits),
+        en_with_revisions: parse_part_root(&mut dom, &pkg1, &en1, settings.input_limits),
     };
     // Same canonicalization for the notes parts the pipeline diffs: a
     // spelling-only footnote geometry pair is not a formatting change.
diff --git a/src/scrub.rs b/src/scrub.rs
index 641d427..c310114 100644
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -41,6 +41,10 @@ const APP: &str = "docProps/app.xml";
 const CUSTOM: &str = "docProps/custom.xml";
 const CUSTOM_REL: &str =
     "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
+const CORE_REL: &str =
+    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
+const EXTENDED_REL: &str =
+    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
 const CP: &str = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
 const DC: &str = "http://purl.org/dc/elements/1.1/";
 const DCTERMS: &str = "http://purl.org/dc/terms/";
@@ -352,36 +356,65 @@ fn remove_children(part: &mut Part, names: &[XName]) -> bool {
     changed
 }
 
+/// The parts holding one kind of document property: every internal package
+/// relationship of `rel_type` — the authoritative targets, since a producer
+/// can keep its properties under any part name — plus the conventional
+/// `docProps` name for packages without the relationship. Deduplicated and
+/// sorted, so the scrub below is deterministic.
+fn property_parts(pkg: &PartFs, rel_type: &str, conventional: &str) -> Vec<String> {
+    let mut names: Vec<String> = pkg
+        .package_relationships()
+        .items
+        .iter()
+        .filter(|r| r.rel_type == rel_type && r.target_mode.as_deref() != Some("External"))
+        .map(|r| pkg.resolve_rel_target("/", &r.target))
+        .collect();
+    names.push(conventional.to_string());
+    names.sort();
+    names.dedup();
+    names
+}
+
 /// Drop the people and dates of the core and extended properties, and the
-/// custom properties part.
+/// custom properties part — under whatever part names the package's own
+/// relationships point at, not only the conventional `docProps` ones.
 fn scrub_properties(pkg: &mut PartFs) {
-    if let Some(mut core) = Part::load(pkg, CORE) {
-        let names = [
-            XName::get("creator", DC),
-            XName::get("lastModifiedBy", CP),
-            XName::get("revision", CP),
-            XName::get("lastPrinted", CP),
-            XName::get("created", DCTERMS),
-            XName::get("modified", DCTERMS),
-        ];
-        if remove_children(&mut core, &names) {
-            core.store(pkg);
+    for name in property_parts(pkg, CORE_REL, CORE) {
+        if let Some(mut core) = Part::load(pkg, &name) {
+            let names = [
+                XName::get("creator", DC),
+                XName::get("lastModifiedBy", CP),
+                XName::get("revision", CP),
+                XName::get("lastPrinted", CP),
+                XName::get("created", DCTERMS),
+                XName::get("modified", DCTERMS),
+            ];
+            if remove_children(&mut core, &names) {
+                core.store(pkg);
+            }
         }
     }
-    if let Some(mut app) = Part::load(pkg, APP) {
-        let names = [
-            XName::get("Manager", EXTENDED),
-            XName::get("Company", EXTENDED),
-        ];
-        if remove_children(&mut app, &names) {
-            app.store(pkg);
+    for name in property_parts(pkg, EXTENDED_REL, APP) {
+        if let Some(mut app) = Part::load(pkg, &name) {
+            let names = [
+                XName::get("Manager", EXTENDED),
+                XName::get("Company", EXTENDED),
+            ];
+            if remove_children(&mut app, &names) {
+                app.store(pkg);
+            }
         }
     }
-    if pkg.part_bytes(CUSTOM).is_some() {
-        pkg.remove_part(CUSTOM);
-        pkg.remove_package_relationships_by_type(CUSTOM_REL);
-        pkg.remove_content_type_override(&format!("/{CUSTOM}"));
+    for name in property_parts(pkg, CUSTOM_REL, CUSTOM) {
+        if pkg.part_bytes(&name).is_some() {
+            pkg.remove_part(&name);
+            pkg.remove_content_type_override(&format!("/{name}"));
+        }
     }
+    // The custom-property relationships go whatever the parts were called:
+    // a package keeping them under another name used to keep the
+    // relationship (and so a reader that follows it) too.
+    pkg.remove_package_relationships_by_type(CUSTOM_REL);
 }
 
 /// The parts of `docx` where `needle` still occurs, sorted: in a text node,
@@ -752,6 +785,74 @@ mod public_memory_leak_boundary_tests {
             assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
         }
     }
+
+    /// The property parts are what the package's own relationships name:
+    /// a document keeping them under non-`docProps` targets must lose its
+    /// creators, editors, company and custom properties too, or scrub
+    /// reports success while the metadata survives.
+    #[test]
+    fn docprops_scrub_reaches_property_parts_the_relationships_name() {
+        let alias_core = format!(
+            "<cp:coreProperties xmlns:cp=\"{CP}\" xmlns:dc=\"{DC}\" xmlns:dcterms=\"{DCTERMS}\">\
+             <dc:creator>Jane Doe</dc:creator>\
+             <cp:lastModifiedBy>Editor</cp:lastModifiedBy>\
+             <dcterms:created>2026-01-01T00:00:00Z</dcterms:created>\
+             </cp:coreProperties>"
+        );
+        let alias_app = format!(
+            "<Properties xmlns=\"{EXTENDED}\">\
+             <Manager>Bob</Manager><Company>Acme Ltd</Company>\
+             </Properties>"
+        );
+        let alias_custom = format!(
+            "<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/custom-properties\">\
+             <property name=\"Reviewer\" fmtid=\"{{D6D2761D-68F2-430E-8AD6-1C56C094485A}}\" pid=\"2\" type=\"lpwstr\">Jane Doe</property>\
+             </Properties>"
+        );
+        let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+        for name in [CORE, APP, CUSTOM] {
+            pkg.remove_part(name);
+        }
+        pkg.remove_package_relationships_by_type(CORE_REL);
+        pkg.remove_package_relationships_by_type(EXTENDED_REL);
+        pkg.remove_package_relationships_by_type(CUSTOM_REL);
+        pkg.set_part("metadata/zcore.xml", alias_core.into_bytes());
+        pkg.set_part("metadata/zapp.xml", alias_app.into_bytes());
+        pkg.set_part("metadata/zcustom.xml", alias_custom.into_bytes());
+        pkg.add_content_type_override(
+            "/metadata/zcore.xml",
+            "application/vnd.openxmlformats-package.core-properties+xml",
+        );
+        pkg.add_content_type_override(
+            "/metadata/zapp.xml",
+            "application/vnd.openxmlformats-officedocument.extended-properties+xml",
+        );
+        pkg.add_content_type_override(
+            "/metadata/zcustom.xml",
+            "application/vnd.openxmlformats-officedocument.custom-properties+xml",
+        );
+        pkg.add_package_relationship(CORE_REL, "metadata/zcore.xml");
+        pkg.add_package_relationship(EXTENDED_REL, "metadata/zapp.xml");
+        pkg.add_package_relationship(CUSTOM_REL, "metadata/zcustom.xml");
+        let bytes = pkg.to_zip().unwrap();
+
+        let out = scrub(&bytes, &ScrubOptions::default()).expect("scrub");
+        assert_eq!(
+            leaks(&out, "Jane Doe"),
+            Vec::<String>::new(),
+            "the renamed property parts must not keep the names"
+        );
+        assert_eq!(leaks(&out, "Bob"), Vec::<String>::new());
+        let done = PartFs::open(&out).unwrap();
+        let core = done.part_string("metadata/zcore.xml").unwrap_or_default();
+        assert!(!core.contains("creator"), "{core}");
+        assert!(done.part_bytes("metadata/zcustom.xml").is_none());
+        assert!(done
+            .package_relationships()
+            .items
+            .iter()
+            .all(|r| r.rel_type != CUSTOM_REL));
+    }
 }
 
 #[cfg(test)]
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..d42461a 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -536,10 +536,49 @@ impl<'a> Parser<'a> {
                 self.pos = end.map(|e| e + 3).unwrap_or(len);
                 continue;
             }
+            if self.starts_with("<![CDATA[") {
+                // A top-level CDATA section is one construct, not markup:
+                // consume its complete `…]]>` grammar. The declaration skip
+                // below would end a body holding `]` before a `>` at that
+                // `>`, and the rest of the section would parse as elements.
+                // (In-root CDATA, which Word does write, is kept by
+                // `parse_element`.)
+                let end = self.index_of("]]>", self.pos);
+                self.pos = end.map(|e| e + 3).unwrap_or(len);
+                continue;
+            }
             if self.starts_with("<!") {
-                // DOCTYPE — skip
-                let end = self.index_of(">", self.pos);
-                self.pos = end.map(|e| e + 1).unwrap_or(len);
+                // A declaration (a DOCTYPE, possibly with an internal
+                // subset `[…]` whose quoted entity values may hold
+                // element-like text and `>` of their own): skip all of it.
+                // Stopping at the first `>` can stop inside a quoted
+                // literal, and the rest of the literal then parses as real
+                // elements — a deeply nested one reaches the recursive
+                // element parser with no depth budget. Quotes and the
+                // subset brackets are honoured, so the declaration is
+                // consumed whole.
+                let mut i = self.pos + 2;
+                let mut quote = '\0';
+                let mut subset = false;
+                while i < len {
+                    let ch = self.c[i];
+                    if quote != '\0' {
+                        if ch == quote {
+                            quote = '\0';
+                        }
+                    } else if ch == '"' || ch == '\'' {
+                        quote = ch;
+                    } else if ch == '[' {
+                        subset = true;
+                    } else if ch == ']' {
+                        subset = false;
+                    } else if ch == '>' && !subset {
+                        i += 1;
+                        break;
+                    }
+                    i += 1;
+                }
+                self.pos = i;
                 continue;
             }
             if self.cur() == '<' {
@@ -1044,6 +1083,72 @@ mod checked_xml_tests {
             assert_eq!(validate_xml(xml), Ok(()), "{xml}");
         }
     }
+
+    /// A DOCTYPE's quoted internal-subset literal holds element-like markup
+    /// with `>` characters of its own: the permissive parser must skip the
+    /// whole declaration (quotes and `[…]` honoured), not stop at the first
+    /// `>`, or the rest of the literal parses as real elements — a deeply
+    /// nested literal reaches the recursive element parser with no depth
+    /// budget. The checked gate refuses a DTD outright.
+    #[test]
+    fn a_dtd_internal_subset_is_skipped_whole_not_to_the_first_gt() {
+        let xml = concat!(
+            "<!DOCTYPE w:document [",
+            "<!ENTITY deep \"<a><b><c>payload</c></b></a>\">",
+            "]>",
+            "<w:document/>"
+        );
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, xml);
+        let root = dom
+            .root(document)
+            .expect("the document's real root survives the declaration");
+        assert_eq!(dom.name(root).unwrap().local_name(), "document");
+        let elements = dom
+            .nodes(document)
+            .iter()
+            .filter(|&&node| dom.is_element(node))
+            .count();
+        assert_eq!(
+            elements, 1,
+            "markup inside the DTD literal must not become elements"
+        );
+        assert!(validate_xml(xml).is_err(), "the checked gate refuses a DTD");
+    }
+
+    /// A top-level CDATA body is skipped whole, `]]>` and all: a body
+    /// holding `]` before a `>` makes the first `>` look like the end of
+    /// the construct, and the markup after it would parse as real elements
+    /// with no depth budget. The checked gate refuses CDATA outside the
+    /// root outright; in-root CDATA stays text (parse_element keeps it).
+    #[test]
+    fn a_top_level_cdata_is_skipped_whole_even_with_brackets_in_its_body() {
+        let xml = concat!(
+            "<![CDATA[a]>",
+            "<d>deep markup</d>",
+            "]]>",
+            "<w:document/>"
+        );
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, xml);
+        let root = dom
+            .root(document)
+            .expect("the document's real root survives the CDATA");
+        assert_eq!(dom.name(root).unwrap().local_name(), "document");
+        let elements = dom
+            .nodes(document)
+            .iter()
+            .filter(|&&node| dom.is_element(node))
+            .count();
+        assert_eq!(
+            elements, 1,
+            "markup inside a top-level CDATA must not become elements"
+        );
+        assert!(
+            validate_xml(xml).is_err(),
+            "the checked gate refuses CDATA outside the root"
+        );
+    }
 }
 #[cfg(test)]
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
diff --git a/tests/input_admission.rs b/tests/input_admission.rs
index 2f9769e..dc3bd69 100644
--- a/tests/input_admission.rs
+++ b/tests/input_admission.rs
@@ -374,6 +374,110 @@ fn get_revisions_admits_under_the_settings_budget() {
         .expect("the default budget admits the fixture");
 }
 
+/// A main part carrying a DOCTYPE used to pass admission as one ignored
+/// event while the permissive DOM parser skipped the declaration only to
+/// the first `>` — inside a quoted internal-subset literal — and built the
+/// literal's markup as elements. Admission must refuse the DTD with the
+/// same INVALID_XML the checked gate reports.
+#[test]
+fn wml_document_refuses_a_dtd_main_part() {
+    let dtd = concat!(
+        r#"<!DOCTYPE w:document [ <!ENTITY deep "<a><b><c>x</c></b></a>"> ]>"#,
+        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>hi</w:t></w:r></w:p></w:body></w:document>"#
+    );
+    let bytes = package_with_document(dtd);
+    let Err(err) = WmlDocument::from_bytes(&bytes) else {
+        panic!("a DTD main part must not be admitted");
+    };
+    assert!(err.to_string().contains("INVALID_XML"), "{err}");
+}
+
+/// A main part opening with a CDATA section whose body holds `]` before a
+/// `>` used to pass admission as one ignored event while the permissive
+/// parser ended the section at that `>` and built the body's markup as
+/// elements. Admission must refuse CDATA outside the root with the same
+/// INVALID_XML the checked gate reports.
+#[test]
+fn wml_document_refuses_a_top_level_cdata_main_part() {
+    let cdata = concat!(
+        "<![CDATA[a]><d><e>deep markup</e></d>]]>",
+        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>hi</w:t></w:r></w:p></w:body></w:document>"#
+    );
+    let bytes = package_with_document(cdata);
+    let Err(err) = WmlDocument::from_bytes(&bytes) else {
+        panic!("a top-level CDATA main part must not be admitted");
+    };
+    assert!(err.to_string().contains("INVALID_XML"), "{err}");
+}
+
+/// A deeply nested footnotes part under a non-`.xml` name skips
+/// admission's extension gate, but `get_revisions` follows the
+/// relationship and parses the part: the XML policy must be applied to the
+/// relationship-selected bytes before the DOM is built, and the notes part
+/// past the depth budget contributes no revisions instead of a
+/// recursively-parsed story.
+#[test]
+fn get_revisions_skips_a_renamed_notes_part_past_the_depth_budget() {
+    let mut notes = String::from(
+        r#"<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
+    );
+    notes.push_str(r#"<w:footnote w:id="1">"#);
+    notes.push_str(&"<w:deep>".repeat(300));
+    notes.push_str(
+        r#"<w:p><w:ins w:id="1" w:author="Deep" w:date="2026-01-01T00:00:00Z"><w:r><w:t>deep note</w:t></w:r></w:ins></w:p>"#,
+    );
+    notes.push_str(&"</w:deep>".repeat(300));
+    notes.push_str("</w:footnote></w:footnotes>");
+    let bytes = package(|writer, options| {
+        writer
+            .start_file("word/_rels/document.xml.rels", options)
+            .unwrap();
+        writer
+            .write_all(
+                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="notes.bin"/></Relationships>"#
+                    .as_bytes(),
+            )
+            .unwrap();
+        writer.start_file("word/notes.bin", options).unwrap();
+        writer.write_all(notes.as_bytes()).unwrap();
+    });
+    let revisions =
+        jubarte::document_comparer::get_revisions(&bytes, &WmlComparerSettings::default())
+            .expect("the package is valid; only the renamed part is skipped");
+    let listed = revisions
+        .iter()
+        .map(jubarte::document_comparer::revision_to_json)
+        .collect::<Vec<String>>();
+    assert!(
+        listed.iter().all(|json| !json.contains("Deep")),
+        "a notes part past the depth budget must not reach the DOM: {listed:?}"
+    );
+    // Control: the same nesting in a conventionally named part is caught
+    // by admission's extension gate instead.
+    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
+    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
+    for (name, body) in [
+        ("[Content_Types].xml", CONTENT_TYPES),
+        ("_rels/.rels", PACKAGE_RELS),
+        ("word/document.xml", DOCUMENT),
+        (
+            "word/_rels/document.xml.rels",
+            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="notes.xml"/></Relationships>"#,
+        ),
+        ("word/notes.xml", notes.as_str()),
+    ] {
+        writer.start_file(name, options).unwrap();
+        writer.write_all(body.as_bytes()).unwrap();
+    }
+    let conventional = writer.finish().unwrap().into_inner();
+    let err = jubarte::document_comparer::get_revisions(
+        &conventional,
+        &WmlComparerSettings::default(),
+    )
+    .expect_err("a .xml-named notes part past the depth budget is refused by admission");
+    assert!(err.to_string().contains("INVALID_XML"), "{err}");
+}
+
 /// The overrides the Python and WASM bindings accept: every key optional,
 /// the rest taken from the base budget, unknown keys refused.
 #[test]

```
