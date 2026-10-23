@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_9e40f23e742aa43493a0fb92 | Rendering a small DOCX table can exhaust host memory | medium
csf_8dcaf50784fd95a7f0c2ec28 | Processing renamed auxiliary XML bypasses XML depth limits | medium
csf_d46d1a3ef7bb52454cbd3c68 | Large Chicago numbering values can exhaust memory while reading or rendering a DOCX | medium
csf_dc0d34a6ea9a964733efe547 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_68acfaf376a60cf8c3d8e2d4 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_cac980ab6041d4ec91b39fd1 | Repeated relationship IDs trigger quadratic repair work | medium
csf_c2ab52b1c8614c7fd5356cf8 | Python and Node plan recording can overwrite unrelated files through staging symlinks | low
csf_c1fd47c87c992981879d9be6 | Tolerated XML syntax can hide recursive elements from admission | medium
csf_dbf35c828f9082997b9ef8fc | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_33c5fede5e569e4c1531e38a | Rendering a small DOCX table can exhaust host memory | medium
csf_bff2c0151aeb6ffaa6f96d4a | Repeated relationship IDs trigger quadratic repair work | medium
csf_85faece8ffd45c8ee175c288 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_5a86b3f5aa767e4e64bd9e71 | Rendering a small DOCX table can exhaust host memory | medium
csf_91956fe2d99a3a2871c06916 | Aliased legacy DOC pieces expand small physical text into enormous logical stories | medium
csf_f1c7e6e8877805d7436a18fa | Python and Node plan recording can overwrite unrelated files through staging symlinks | low
csf_f756bbe5509c42bdcaa64921 | Processing renamed auxiliary XML bypasses XML depth limits | medium

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
--- a/src/opc/mod.rs	2026-10-21 14:44:05
+++ b/src/opc/mod.rs	2026-10-21 15:09:03
@@ -99,6 +99,10 @@
 fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>, OpcError> {
     let mut seen: HashMap<&str, &Relationship> = HashMap::new();
     let mut repaired: Vec<String> = Vec::new();
+    // A Vec used as the membership set made a package whose every Id
+    // repeats identically quadratic in distinct Ids; the set answers
+    // membership, the Vec only keeps the first-duplicate order (dedup-0012).
+    let mut reported: HashSet<&str> = HashSet::new();
     let mut keep = Vec::with_capacity(rels.items.len());
     for item in &rels.items {
         match seen.get(item.id.as_str()) {
@@ -113,7 +117,7 @@
                 {
                     return Err(OpcError::InvalidRelationship);
                 }
-                if !repaired.contains(&item.id) {
+                if reported.insert(item.id.as_str()) {
                     repaired.push(item.id.clone());
                 }
                 keep.push(false);
@@ -966,6 +970,70 @@
     const RELS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
     const HYPERLINK: &str =
         "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
+
+    #[test]
+    fn identical_duplicate_repair_reports_each_id_once_in_order() {
+        // Every Id repeated identically: the first record is kept, each Id
+        // is reported once, and the report follows first-duplicate order at
+        // both the package and the part level (dedup-0012).
+        let rels = "<Relationship Id=\"rId9\" Type=\"{HYPERLINK}\" Target=\"https://a/\"/>\
+<Relationship Id=\"rId9\" Type=\"{HYPERLINK}\" Target=\"https://a/\"/>\
+<Relationship Id=\"rId2\" Type=\"{HYPERLINK}\" Target=\"https://b/\"/>\
+<Relationship Id=\"rId2\" Type=\"{HYPERLINK}\" Target=\"https://b/\"/>\
+<Relationship Id=\"rId9\" Type=\"{HYPERLINK}\" Target=\"https://a/\"/>";
+        let fs = PartFs::open(&package_with_document_rels(rels)).unwrap();
+        let kept = &fs.part_rels["/word/document.xml"].items;
+        assert_eq!(kept.len(), 2, "{kept:?}");
+        assert_eq!(
+            kept.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
+            ["rId9", "rId2"]
+        );
+        assert_eq!(
+            fs.repaired_duplicate_ids
+                .iter()
+                .map(|(part, id)| (part.as_str(), id.as_str()))
+                .collect::<Vec<_>>(),
+            [("word/document.xml", "rId9"), ("word/document.xml", "rId2")]
+        );
+    }
+
+    #[test]
+    fn conflicting_duplicate_relationship_ids_are_refused() {
+        let rels = "<Relationship Id=\"rId9\" Type=\"{HYPERLINK}\" Target=\"https://a/\"/>\
+<Relationship Id=\"rId9\" Type=\"{HYPERLINK}\" Target=\"https://other/\"/>";
+        assert!(PartFs::open(&package_with_document_rels(rels)).is_err());
+    }
+
+    #[test]
+    fn identical_duplicate_repair_stays_linear_in_distinct_ids() {
+        // The pre-fix repair scanned every already-reported Id per duplicate,
+        // so many distinct Ids each repeated twice cost quadratic string
+        // comparisons during an ordinary open (dedup-0012). The time bound
+        // is the demonstration: the fixed repair opens this package in well
+        // under a second where the scanning one needs tens of seconds.
+        let distinct = 30_000;
+        let rels = (0..distinct)
+            .map(|i| {
+                let one = format!(
+                    "<Relationship Id=\"rId{i}\" Type=\"{HYPERLINK}\" Target=\"https://e.com/{i}\"/>"
+                );
+                format!("{one}{one}")
+            })
+            .collect::<String>();
+        let bytes = package_with_document_rels(&rels);
+        let start = std::time::Instant::now();
+        let fs = PartFs::open(&bytes).expect("identical duplicates are repaired, not refused");
+        let elapsed = start.elapsed();
+        assert_eq!(
+            fs.repaired_duplicate_ids.len(),
+            distinct,
+            "each Id reported once"
+        );
+        assert!(
+            elapsed < std::time::Duration::from_secs(10),
+            "repair of {distinct} duplicated Ids took {elapsed:?}"
+        );
+    }
 
     #[test]
     fn public_and_package_relationship_readers_agree() {
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
--- a/src/convert/mod.rs	2026-10-21 14:44:05
+++ b/src/convert/mod.rs	2026-10-21 16:40:00
@@ -6011,11 +6011,18 @@
         other => format!("{other}th"),
     }
 }
+
+/// The most times a Chicago label repeats its mark. Word's own Chicago
+/// lists never come near it; a `w:start`, a `startOverride` or a section
+/// `pgNumType/@w:start` near `u32::MAX` must not allocate a label of
+/// `n / 4` marks, so values past the cap keep the cap's mark run
+/// (dedup-0012).
+const CHICAGO_MAX_REPEATS: usize = 1024;
 
 fn chicago_label(n: u32) -> String {
     const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
     let i = n.saturating_sub(1) as usize;
-    MARKS[i % 4].repeat(i / 4 + 1)
+    MARKS[i % 4].repeat((i / 4 + 1).min(CHICAGO_MAX_REPEATS))
 }
 
 fn ideograph_digital_label(n: u32) -> String {
@@ -13208,6 +13215,9 @@
             PrefWidth::Dxa(cols.iter().skip(at).take(span).sum())
         };
         if let Some((span, pref)) = grid_before {
+            // The row's cumulative grid claim is clamped to the column
+            // budget; a skip past it covers one column (dedup-0012).
+            let span = span.min(MAX_TABLE_COLUMNS.saturating_sub(grid_at).max(1));
             let pref = match pref {
                 PrefWidth::Auto => grid_width(grid_at, span),
                 p => p,
@@ -13464,6 +13474,9 @@
                 &nested_at,
             );
             let (colspan, vmerge) = cell_span(dom, cell);
+            // Same cumulative clamp for cells: no row's RawCell spans can
+            // size a derived vector past the budget (dedup-0012).
+            let colspan = colspan.min(MAX_TABLE_COLUMNS.saturating_sub(grid_at).max(1));
             let last_col = grid_at + colspan.max(1) >= cols.len();
             let at = |b: TblBorders| {
                 row_exception_cell_borders(b, ri == 0, ri + 1 == row_count, grid_at == 0, last_col)
@@ -13553,6 +13566,7 @@
             });
         }
         if let Some((span, pref)) = grid_after {
+            let span = span.min(MAX_TABLE_COLUMNS.saturating_sub(grid_at).max(1));
             let pref = match pref {
                 PrefWidth::Auto => grid_width(grid_at, span),
                 p => p,
@@ -13601,7 +13615,15 @@
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
-        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
+        // Checked and budgeted: the fold cannot overflow (32-bit WASM
+        // included) and no row claims more columns than the budget
+        // (dedup-0012).
+        let claimed = row
+            .iter()
+            .map(|c| c.colspan)
+            .fold(0usize, usize::saturating_add)
+            .min(MAX_TABLE_COLUMNS);
+        occupancy = occupancy.max(claimed);
     }
     if cols.len() < occupancy {
         if cols.len() == 1 && occupancy > 1 {
@@ -14185,22 +14207,30 @@
                 let mut extra = 0usize;
                 while extra < spare
                     && want - covered > STALE_SPAN_SLACK_PT
-                    && let Some(&next) = cols.get(col + span + extra)
+                    && let Some(&next) = cols.get(col.saturating_add(span).saturating_add(extra))
                     && (covered + next - want).abs() < (want - covered).abs()
                 {
                     covered += next;
                     extra += 1;
                 }
                 if extra > 0 {
-                    cell.colspan = span + extra;
+                    cell.colspan = span.saturating_add(extra);
                     spare -= extra;
                 }
             }
-            col += cell.colspan.max(1);
+            col = col.saturating_add(cell.colspan.max(1));
         }
     }
 }
 
+/// The most grid columns a table may claim before the renderer clamps it.
+/// Word's own tables hold at most 63 columns; 4096 is a memory floor far
+/// above any real document, so nothing Word writes changes, while a tiny
+/// document with `w:gridSpan`/`w:gridBefore`/`w:gridAfter` in the billions
+/// can no longer size column, preference or merge vectors from one number
+/// (dedup-0012).
+const MAX_TABLE_COLUMNS: usize = 4096;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14212,7 +14242,8 @@
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
         .unwrap_or(1)
-        .max(1);
+        .max(1)
+        .min(MAX_TABLE_COLUMNS);
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14342,7 +14373,8 @@
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
-        .filter(|n| *n > 0)?;
+        .filter(|n| *n > 0)?
+        .min(MAX_TABLE_COLUMNS);
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
         .and_then(|n| attr_any(dom, n, "w"))
@@ -14634,14 +14666,15 @@
                 borders: raw.borders,
                 style_fill: false,
             });
-            if open.len() < col + span {
-                open.resize(col + span, None);
+            let end = col.saturating_add(span).min(MAX_TABLE_COLUMNS);
+            if open.len() < end {
+                open.resize(end, None);
             }
             let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
             for slot in open.iter_mut().skip(col).take(span) {
                 *slot = mark;
             }
-            col += span;
+            col = col.saturating_add(span);
         }
     }
     origins
@@ -36377,9 +36410,10 @@
 #[cfg_attr(all(coverage_nightly, test), coverage(off))]
 mod page_num_fmt_labels {
     use super::{
-        NumFmt, chicago_label, decimal_fullwidth_label, format_num, ideograph_digital_label,
-        ideograph_enclosed_circle_label, ideograph_legal_traditional_label,
-        ideograph_zodiac_traditional_label, japanese_counting_label, parse_num_fmt,
+        CHICAGO_MAX_REPEATS, NumFmt, chicago_label, decimal_fullwidth_label, format_num,
+        ideograph_digital_label, ideograph_enclosed_circle_label,
+        ideograph_legal_traditional_label, ideograph_zodiac_traditional_label,
+        japanese_counting_label, parse_num_fmt,
     };
 
     #[test]
@@ -36393,6 +36427,27 @@
         assert_eq!(chicago_label(1), "*");
         assert_eq!(chicago_label(2), "†");
         assert_eq!(chicago_label(5), "**");
+    }
+
+    #[test]
+    fn chicago_labels_stop_growing_at_the_repeat_cap() {
+        // A list start or startOverride near u32::MAX once allocated a
+        // gigabyte mark run through list, page-label and Markdown consumers
+        // alike (dedup-0012). At and below the cap the cycle is unchanged;
+        // past it the run holds at the cap instead of growing with n.
+        assert_eq!(chicago_label(4).chars().count(), 1);
+        assert_eq!(chicago_label(5).chars().count(), 2);
+        let at_cap = 4 * CHICAGO_MAX_REPEATS; // the first n whose run is the cap
+        assert_eq!(
+            chicago_label(at_cap as u32).chars().count(),
+            CHICAGO_MAX_REPEATS
+        );
+        assert_eq!(chicago_label(u32::MAX).chars().count(), CHICAGO_MAX_REPEATS);
+        // Four marks later the label is the same capped run of the same mark.
+        assert_eq!(
+            chicago_label((at_cap + 4) as u32),
+            chicago_label(at_cap as u32)
+        );
     }
 
     #[test]
@@ -48599,6 +48654,49 @@
     }
 
     #[test]
+    fn huge_spans_are_clamped_to_the_column_budget() {
+        // A tiny admitted table once sized its grid vector from one
+        // authored number: gridSpan 4294967295 asked for a 16 GiB f32
+        // vector before layout (dedup-0012). The budget clamps the claim
+        // while ordinary spans (Word's tables hold at most 63 columns)
+        // keep their exact grid.
+        let block = table(
+            "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>\
+<w:tc><w:tcPr><w:gridSpan w:val=\"4294967295\"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>",
+        );
+        let Block::Table { cols, rows, .. } = block else {
+            panic!("table fixture")
+        };
+        assert!(cols.len() <= MAX_TABLE_COLUMNS, "{} columns", cols.len());
+        assert!(rows[0][0].colspan <= MAX_TABLE_COLUMNS);
+
+        let block = table(
+            "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:trPr>\
+<w:gridBefore w:val=\"2000000000\"/></w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl>",
+        );
+        let Block::Table { cols, .. } = block else {
+            panic!("table fixture")
+        };
+        assert!(cols.len() <= MAX_TABLE_COLUMNS, "{} columns", cols.len());
+    }
+
+    #[test]
+    fn ordinary_spanned_tables_keep_their_exact_grid() {
+        let block = table(
+            "<w:tbl><w:tblGrid><w:gridCol w:w=\"1000\"/><w:gridCol w:w=\"1000\"/>\
+<w:gridCol w:w=\"1000\"/></w:tblGrid><w:tr>\
+<w:tc><w:tcPr><w:gridSpan w:val=\"2\"/></w:tcPr><w:p/></w:tc>\
+<w:tc><w:p/></w:tc></w:tr></w:tbl>",
+        );
+        let Block::Table { cols, rows, .. } = block else {
+            panic!("table fixture")
+        };
+        assert_eq!(cols.len(), 3);
+        assert_eq!((rows[0][0].col, rows[0][0].colspan), (0, 2));
+        assert_eq!(rows[0][1].col, 2);
+    }
+
+    #[test]
     fn stale_spans_expand_only_when_an_extra_column_is_closer() {
         let mut rows = vec![vec![raw("wide", 100.0, 1), raw("last", 50.0, 1)]];
         stretch_stale_spans(&mut rows, &[50.0, 50.0, 50.0]);
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
--- a/src/legacy_doc.rs	2026-10-21 14:44:05
+++ b/src/legacy_doc.rs	2026-10-21 15:35:22
@@ -1139,14 +1139,45 @@
     modifier: CharModifier,
 }
 
-/// The main story's characters, CP 0 to `ccp_text`.
+/// The most main-story characters a legacy document may claim before the
+/// reader refuses it. The piece table tiles the story exactly, so this
+/// bounds both the decoded `StoryChar` vector and the decode loop; Word's
+/// own stories are orders of magnitude shorter (dedup-0012).
+const MAX_MAIN_STORY_CHARS: usize = 16 * 1024 * 1024;
+
+/// The main story's characters, CP 0 to `ccp_text`, within the reader's
+/// default story budget.
 ///
 /// # Errors
 ///
-/// A piece that points past the `WordDocument` stream: a truncated or
-/// corrupt file, refused rather than converted short.
+/// As [`main_text_within`].
 fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {
+    main_text_within(word, pieces, ccp_text, MAX_MAIN_STORY_CHARS)
+}
+
+/// The main story's characters, CP 0 to `ccp_text`, decoding at most
+/// `budget` characters in all. Aliased pieces — many logical ranges over
+/// one physical range, a fast-save shape — decode each logical character,
+/// so the budget is charged per decoded character, not per story claim.
+///
+/// # Errors
+///
+/// A piece that points past the `WordDocument` stream: a truncated or
+/// corrupt file, refused rather than converted short. A story or piece
+/// table whose decoded characters exceed `budget`, refused before any
+/// `StoryChar` is allocated.
+fn main_text_within(
+    word: &[u8],
+    pieces: &[Piece],
+    ccp_text: u32,
+    budget: usize,
+) -> Result<Vec<StoryChar>> {
     let past = || LegacyDocError::new("a text piece points past the WordDocument stream");
+    if ccp_text as usize > budget {
+        return Err(LegacyDocError::new(&format!(
+            "the main story claims {ccp_text} characters, over the reader's {budget}-character budget"
+        )));
+    }
     // The pieces run from CP 0 through the main story without a gap, or
     // the document would come out quietly shorter than it is.
     // The CPs only ever increase ([MS-DOC] 2.8.35): a piece that runs
@@ -1173,6 +1204,18 @@
             "the piece table does not cover the main story",
         ));
     }
+    // Overlapping pieces decode a CP more than once, so charge every
+    // character the decode loop will emit before allocating for any.
+    let planned: u64 = pieces
+        .iter()
+        .take_while(|piece| piece.cp_start < ccp_text)
+        .map(|piece| u64::from(piece.cp_end.min(ccp_text) - piece.cp_start))
+        .sum();
+    if planned > budget as u64 {
+        return Err(LegacyDocError::new(&format!(
+            "the piece table decodes {planned} characters, over the reader's {budget}-character budget"
+        )));
+    }
     let mut out = Vec::new();
     // A high surrogate waiting for its low half, which may open the next
     // Unicode piece (a supplementary character split between two pieces).
@@ -1843,6 +1886,60 @@
     }
 
     #[test]
+    fn a_story_larger_than_the_reader_budget_is_refused_before_expansion() {
+        // 17 Mi of logical characters tiled over one physical kilobyte:
+        // aliased pieces whose metadata costs ~260 KiB. The pre-budget
+        // reader materialized a StoryChar for every logical CP before any
+        // later table could refuse it (dedup-0012).
+        let word = vec![b'A'; 1024];
+        let piece = |cp_start: u32| Piece {
+            cp_start,
+            cp_end: cp_start + 1024,
+            fc: 0,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let aliased: Vec<Piece> = (0..17 * 1024).map(|i| piece(i * 1024)).collect();
+        let ccp = 17 * 1024 * 1024;
+        let error = main_text(&word, &aliased, ccp).unwrap_err().to_string();
+        assert!(error.contains("budget"), "{error}");
+    }
+
+    #[test]
+    fn aliased_pieces_within_the_budget_still_decode() {
+        // The same fast-save aliasing, inside the budget: every logical
+        // character decodes exactly as it always did (dedup-0012).
+        let word = b"Hello".to_vec();
+        let piece = |cp_start: u32| Piece {
+            cp_start,
+            cp_end: cp_start + 5,
+            fc: 0,
+            compressed: true,
+            modifier: CharModifier::default(),
+        };
+        let aliased: Vec<Piece> = (0..10).map(|i| piece(i * 5)).collect();
+        let chars = main_text_within(&word, &aliased, 50, 50).unwrap();
+        assert_eq!(chars.len(), 50);
+        assert!(chars
+            .iter()
+            .all(|c| matches!(c.ch, 'H' | 'e' | 'l' | 'o')));
+        // One character over the budget: refused before allocating.
+        let error = main_text_within(&word, &aliased, 50, 49).unwrap_err().to_string();
+        assert!(error.contains("budget"), "{error}");
+        // Overlapping pieces charge every decoded character, not the story
+        // claim: three copies of one range are fifteen, not five.
+        let overlapping = vec![piece(0), piece(0), piece(0)];
+        assert_eq!(
+            main_text_within(&word, &overlapping, 5, 15).unwrap().len(),
+            15
+        );
+        let error = main_text_within(&word, &overlapping, 5, 14)
+            .unwrap_err()
+            .to_string();
+        assert!(error.contains("budget"), "{error}");
+    }
+
+    #[test]
     fn a_piece_past_the_stream_is_refused_not_truncated() {
         let word = b"Hello".to_vec();
         let piece = Piece {
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
--- a/src/markdown/from_docx/drawing.rs	2026-10-21 14:44:05
+++ b/src/markdown/from_docx/drawing.rs	2026-10-21 16:47:26
@@ -1393,7 +1393,11 @@
 }
 
 /// Decode the payload of `![label](drawing:PAYLOAD)`; `dest` is what
-/// [`at`] returns (`drawing:…`).
+/// [`at`] returns (`drawing:…`). The decoded envelope is attacker-authored
+/// Markdown, not this engine's output, so the same dependency, edge and
+/// byte budgets the capture side enforces apply here too, and a dependency
+/// graph that depends on itself is refused rather than materialized
+/// recursively (dedup-0012).
 pub(crate) fn decode(dest: &str) -> Result<Drawing, String> {
     let encoded = dest
         .strip_prefix("drawing:")
@@ -1405,9 +1409,101 @@
             base64::engine::general_purpose::URL_SAFE.decode(encoded.trim_end_matches('='))
         })
         .map_err(|e| format!("drawing payload is not base64: {e}"))?;
-    serde_json::from_slice(&bytes).map_err(|e| format!("drawing payload is not its JSON: {e}"))
+    let drawing: Drawing = serde_json::from_slice(&bytes)
+        .map_err(|e| format!("drawing payload is not its JSON: {e}"))?;
+    validate(&drawing)?;
+    Ok(drawing)
 }
 
+/// An imported payload must obey the capture-side budgets ([`MAX_DEPS`],
+/// [`MAX_DEP_RELS`], [`MAX_DEP_BYTES`]), name only parts it holds, and
+/// carry no dependency cycle.
+fn validate(drawing: &Drawing) -> Result<(), String> {
+    if drawing.deps.len() > MAX_DEPS {
+        return Err(format!(
+            "drawing payload holds more than {MAX_DEPS} dependent parts"
+        ));
+    }
+    let in_range = |rel: &PayRel| {
+        rel.part.is_none_or(|index| index < drawing.deps.len())
+    };
+    for rel in &drawing.rels {
+        if !in_range(rel) {
+            return Err(format!(
+                "drawing payload relationship \"{}\" names a part it does not hold",
+                rel.sym
+            ));
+        }
+    }
+    for dep in &drawing.deps {
+        if dep.rels.len() > MAX_DEP_RELS {
+            return Err(format!(
+                "drawing payload part {} holds more than {MAX_DEP_RELS} relationships",
+                dep.name
+            ));
+        }
+        // The base64 text implies its decoded bytes: three bytes per four
+        // characters, so the length alone bounds what fill can allocate.
+        if dep.b64.len() as u64 * 3 / 4 > MAX_DEP_BYTES {
+            return Err(format!(
+                "drawing payload part {} exceeds {MAX_DEP_BYTES} bytes",
+                dep.name
+            ));
+        }
+        // Capture serializes `xml` from a part it already bounded at
+        // MAX_DEP_BYTES raw bytes; symbolized ids and injected namespace
+        // declarations rename rather than grow it, and twice the budget is
+        // ample headroom for that while still bounding the envelope.
+        if dep.xml.len() as u64 > 2 * MAX_DEP_BYTES {
+            return Err(format!(
+                "drawing payload part {} exceeds {MAX_DEP_BYTES} bytes",
+                dep.name
+            ));
+        }
+        for rel in &dep.rels {
+            if !in_range(rel) {
+                return Err(format!(
+                    "drawing payload part {} names a part it does not hold",
+                    dep.name
+                ));
+            }
+        }
+    }
+    // A cycle in the part graph recurses forever in materialize; refuse it
+    // here, where the refusal is the payload's own error.
+    let mut color = vec![0u8; drawing.deps.len()]; // 0 new, 1 visiting, 2 done
+    for start in 0..drawing.deps.len() {
+        if color[start] == 0 {
+            visit(drawing, start, &mut color)?;
+        }
+    }
+    Ok(())
+}
+
+/// Depth-first walk of one part's dependency edges; `color` marks the
+/// active path so an edge back into it is a cycle. Depth is bounded by
+/// `MAX_DEPS`, the validated part count.
+fn visit(drawing: &Drawing, node: usize, color: &mut [u8]) -> Result<(), String> {
+    color[node] = 1;
+    for rel in &drawing.deps[node].rels {
+        if let Some(inner) = rel.part
+            && inner < drawing.deps.len()
+        {
+            if color[inner] == 1 {
+                return Err(format!(
+                    "drawing payload part {} depends on itself",
+                    drawing.deps[node].name
+                ));
+            }
+            if color[inner] == 0 {
+                visit(drawing, inner, color)?;
+            }
+        }
+    }
+    color[node] = 2;
+    Ok(())
+}
+
 pub(crate) fn encode(drawing: &Drawing) -> String {
     use base64::Engine;
     let json = serde_json::to_vec(drawing).unwrap_or_default();
@@ -1876,6 +1972,16 @@
         self.mhash = Some(mermaid_hash(text));
         true
     }
+}
+
+/// Traversal state for materializing one payload's dependency graph: the
+/// parts whose expansion is active (an edge back into that set is a cycle
+/// the payload must not carry) and the output name each finished part took
+/// (a shared dependency writes once, not once per reference) (dedup-0012).
+#[derive(Default)]
+struct DepsVisit {
+    visiting: HashSet<usize>,
+    done: HashMap<usize, String>,
 }
 
 pub(crate) fn materialize(
@@ -1886,12 +1992,14 @@
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
 ) -> Option<String> {
+    let mut visit = DepsVisit::default();
     let mut xml = drawing.xml.clone();
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+                let name =
+                    dep_part(drawing, index, package, existing, used, next_doc_pr, &mut visit)?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1952,7 +2060,9 @@
 }
 
 /// Write dependent part `index` and its own dependency closure; its new
-/// part name.
+/// part name. A part already materialized by this payload returns its
+/// recorded name; a part whose expansion is still active is a cycle, and
+/// the payload is refused instead of recursed into.
 fn dep_part(
     drawing: &Drawing,
     index: usize,
@@ -1960,7 +2070,31 @@
     existing: &HashSet<String>,
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
+    visit: &mut DepsVisit,
 ) -> Option<String> {
+    if let Some(name) = visit.done.get(&index) {
+        return Some(name.clone());
+    }
+    if !visit.visiting.insert(index) {
+        return None;
+    }
+    let written = dep_part_inner(drawing, index, package, existing, used, next_doc_pr, visit);
+    visit.visiting.remove(&index);
+    if let Some(name) = &written {
+        visit.done.insert(index, name.clone());
+    }
+    written
+}
+
+fn dep_part_inner(
+    drawing: &Drawing,
+    index: usize,
+    package: &mut PartFs,
+    existing: &HashSet<String>,
+    used: &mut HashSet<String>,
+    next_doc_pr: &mut u32,
+    visit: &mut DepsVisit,
+) -> Option<String> {
     let dep = drawing.deps.get(index)?;
     let name = name_for(&dep.name, existing, used);
     used.insert(name.clone());
@@ -1971,7 +2105,8 @@
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name =
+                    dep_part(drawing, inner, package, existing, used, next_doc_pr, visit)?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -2036,7 +2171,169 @@
         parse_xml(xml.as_bytes()).unwrap()
     }
 
+    fn package() -> PartFs {
+        PartFs::open(&crate::admission::zip_names::docx_package("", &[])).unwrap()
+    }
+
+    fn cycle_drawing() -> Drawing {
+        // One part whose own relationship points back at it: the payload
+        // shape that recursed forever (dedup-0012).
+        Drawing {
+            xml: "<w:drawing/>".into(),
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: "image".into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![Dep {
+                name: "word/media/cycle.png".into(),
+                kind: "image/png".into(),
+                b64: "AAAA".into(),
+                rels: vec![PayRel {
+                    sym: "d0.1".into(),
+                    kind: "image".into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                }],
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        }
+    }
+
     #[test]
+    fn a_self_dependent_part_is_refused_instead_of_recursing() {
+        let mut pkg = package();
+        assert!(materialize(
+            &cycle_drawing(),
+            &mut pkg,
+            "word/document.xml",
+            &HashSet::new(),
+            &mut HashSet::new(),
+            &mut 1
+        )
+        .is_none());
+    }
+
+    #[test]
+    fn a_two_part_cycle_is_refused_instead_of_recursing() {
+        let dep = |name: &str, part: usize| Dep {
+            name: name.into(),
+            kind: "image/png".into(),
+            b64: "AAAA".into(),
+            rels: vec![PayRel {
+                sym: format!("d{part}.1"),
+                kind: "image".into(),
+                part: Some(part),
+                ..PayRel::default()
+            }],
+            ..Dep::default()
+        };
+        let drawing = Drawing {
+            xml: "<w:drawing/>".into(),
+            rels: vec![PayRel {
+                sym: "r1".into(),
+                kind: "image".into(),
+                part: Some(0),
+                ..PayRel::default()
+            }],
+            deps: vec![dep("word/media/a.png", 1), dep("word/media/b.png", 0)],
+            ..Drawing::default()
+        };
+        let mut pkg = package();
+        assert!(materialize(
+            &drawing,
+            &mut pkg,
+            "word/document.xml",
+            &HashSet::new(),
+            &mut HashSet::new(),
+            &mut 1
+        )
+        .is_none());
+    }
+
+    #[test]
+    fn a_shared_dependency_writes_once_under_two_references() {
+        // Two relationships to one part: the part is materialized once and
+        // both relationships name it, instead of expanding the closure per
+        // reference (dedup-0012).
+        let drawing = Drawing {
+            xml: "<w:drawing/>".into(),
+            rels: vec![
+                PayRel {
+                    sym: "r1".into(),
+                    kind: "image".into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                },
+                PayRel {
+                    sym: "r2".into(),
+                    kind: "image".into(),
+                    part: Some(0),
+                    ..PayRel::default()
+                },
+            ],
+            deps: vec![Dep {
+                name: "word/media/shared.png".into(),
+                kind: "image/png".into(),
+                b64: "AAAA".into(),
+                ..Dep::default()
+            }],
+            ..Drawing::default()
+        };
+        let mut pkg = package();
+        assert!(materialize(
+            &drawing,
+            &mut pkg,
+            "word/document.xml",
+            &HashSet::new(),
+            &mut HashSet::new(),
+            &mut 1
+        )
+        .is_some());
+        let media: Vec<String> = pkg
+            .parts()
+            .into_iter()
+            .filter(|part| part.ends_with(".png"))
+            .collect();
+        assert_eq!(media, ["word/media/shared.png".to_string()], "{media:?}");
+    }
+
+    #[test]
+    fn decode_refuses_cycles_and_over_budget_payloads() {
+        use base64::Engine;
+        let payload = |json: &str| {
+            format!(
+                "drawing:{}",
+                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes())
+            )
+        };
+        let ok = payload(
+            r#"{"xml":"","rels":[],"deps":[{"name":"a","t":"image/png","b64":"AAAA","rels":[]}]}"#,
+        );
+        assert!(decode(&ok).is_ok(), "{ok}");
+        // decode is the import boundary of `![label](drawing:...)`: the
+        // same graph the materializer refuses is refused here (dedup-0012).
+        assert!(decode(&encode(&cycle_drawing())).is_err());
+        let cyclic = payload(
+            r#"{"xml":"","rels":[],"deps":[{"name":"a","t":"image/png","b64":"AAAA","rels":[{"sym":"d0.1","t":"image","part":0}]}]}"#,
+        );
+        assert!(decode(&cyclic).is_err());
+        let dangling = payload(
+            r#"{"xml":"","rels":[{"sym":"r1","t":"image","part":7}],"deps":[{"name":"a","t":"image/png","b64":"AAAA","rels":[]}]}"#,
+        );
+        assert!(decode(&dangling).is_err());
+        let mut deps = String::new();
+        for i in 0..=MAX_DEPS {
+            deps.push_str(&format!(r#"{{"name":"d{i}","t":"image/png","b64":"AAAA","rels":[]}},"#));
+        }
+        let deps = deps.trim_end_matches(',');
+        let many = format!(r#"{{"xml":"","rels":[],"deps":[{deps}]}}"#);
+        assert!(decode(&payload(&many)).is_err());
+    }
+
+    #[test]
     fn xml_round_trips_verbatim() {
         let xml = r#"<wp:anchor xmlns:wp="wp" wp14:anchorId="AB"><wp:docPr id="2" name="P"/><x:y a="1&quot;2"><t>x &amp; y</t></x:y></wp:anchor>"#;
         let e = element(xml);
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
--- a/src/xmllinq/parse.rs	2026-10-21 14:44:05
+++ b/src/xmllinq/parse.rs	2026-10-21 15:50:11
@@ -57,6 +57,14 @@
 pub(crate) const MAX_TAG_ATTRIBUTE_BYTES: usize = 128 * 1024 * 1024;
 /// Bytes in one namespace URI.
 pub(crate) const MAX_NAMESPACE_URI_BYTES: usize = 8192;
+/// Nesting levels the permissive DOM builder constructs before it stops
+/// descending and treats deeper markup as inert text. A floor against
+/// stack exhaustion for parts no admission gate depth-checked — a
+/// relationship-selected part under a non-XML name, whose bytes reach
+/// `parse_xdocument` whatever the file is called — far above the 256-level
+/// `InputLimits::max_xml_depth` default and any document Word writes
+/// (dedup-0012).
+pub(crate) const MAX_ELEMENT_DEPTH: usize = 4096;
 /// Bytes in one namespace prefix.
 pub(crate) const MAX_NAMESPACE_PREFIX_BYTES: usize = 256;
 /// Bindings live in one lexical scope (all levels, shadowed included).
@@ -104,6 +112,7 @@
         dom,
         c: chars,
         pos: 0,
+        depth: 0,
     };
     p.parse_document()
 }
@@ -137,6 +146,8 @@
 
 struct Parser<'a> {
     dom: &'a mut Dom,
+    /// Elements open on the current path; see [`MAX_ELEMENT_DEPTH`].
+    depth: usize,
     c: Vec<char>,
     pos: usize,
 }
@@ -552,7 +563,16 @@
         doc
     }
 
+    /// The element at `self.pos`, which is on `<`, behind the depth
+    /// accounting: the builder never recurses past [`MAX_ELEMENT_DEPTH`].
     fn parse_element(&mut self, scope: &mut NsScope) -> NodeId {
+        self.depth += 1;
+        let el = self.parse_element_at(scope);
+        self.depth -= 1;
+        el
+    }
+
+    fn parse_element_at(&mut self, scope: &mut NsScope) -> NodeId {
         // at '<'
         self.pos += 1; // consume '<'
         let raw_name = self.read_name();
@@ -658,6 +678,17 @@
                 continue;
             }
             if self.cur() == '<' {
+                if self.depth >= MAX_ELEMENT_DEPTH {
+                    // Beyond the construction floor: consume the tag without
+                    // building it or recursing. Markup this deep reached the
+                    // builder through a part no gate depth-checked; it stays
+                    // inert text instead of exhausting the stack (dedup-0012).
+                    self.pos += 1;
+                    self.read_name();
+                    let end = self.index_of(">", self.pos);
+                    self.pos = end.map_or(self.len(), |e| e + 1);
+                    continue;
+                }
                 let child = self.parse_element(scope);
                 self.dom.add(el, child);
                 continue;
@@ -1031,6 +1062,32 @@
         assert!(validate_xml(&deep).is_err());
         let ok_deep = format!("{}{}", "<a>".repeat(256), "</a>".repeat(256));
         assert_eq!(validate_xml(&ok_deep), Ok(()));
+    }
+
+    #[test]
+    fn permissive_parser_stops_descending_at_the_construction_floor() {
+        // A relationship-selected part under a non-.xml name crosses no
+        // admission XML gate, so the builder itself is the last bound: a
+        // 60000-deep payload once exhausted the stack (dedup-0012). Count
+        // the elements down the first-child chain, root included.
+        let first_child_chain = |dom: &Dom, document| {
+            let mut depth = 1usize;
+            let mut node = dom.root(document).expect("a root");
+            while let Some(child) = dom.nodes(node).into_iter().find(|&n| dom.is_element(n)) {
+                depth += 1;
+                node = child;
+            }
+            depth
+        };
+        let deep = format!("{}{}", "<a>".repeat(60_000), "</a>".repeat(60_000));
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &deep);
+        assert_eq!(first_child_chain(&dom, document), super::MAX_ELEMENT_DEPTH);
+        // A document within the floor keeps every level.
+        let within = format!("{}{}", "<a>".repeat(100), "</a>".repeat(100));
+        let mut dom = Dom::new();
+        let document = parse_xdocument(&mut dom, &within);
+        assert_eq!(first_child_chain(&dom, document), 100);
     }
 
     #[test]
diff --git a/jubarte-python/python/jubarte_redlines/__main__.py b/jubarte-python/python/jubarte_redlines/__main__.py
--- a/jubarte-python/python/jubarte_redlines/__main__.py	2026-10-21 14:44:05
+++ b/jubarte-python/python/jubarte_redlines/__main__.py	2026-10-21 15:54:29
@@ -33,6 +33,7 @@
 import json
 import os
 import sys
+import tempfile
 from dataclasses import asdict
 from collections.abc import Sequence
 from pathlib import Path
@@ -402,14 +403,24 @@
             print(f"note: {note}")
     if not dry_run:
         # Through a symbolic link to the file it names, written yet or not;
-        # via a hidden partial file named for this process, removed if the
+        # via an exclusive, unpredictable partial file created beside it —
+        # the native recorder's staging design, never a predictable
+        # PID-named partial a planted symlink can sit on — removed if the
         # write fails; bytes as they are, so no platform newline translation.
         target = Path(os.path.realpath(plan))
-        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
-        try:
-            partial.write_bytes(recorded["json"].encode("utf-8"))
-            os.replace(partial, target)
+        fd, staged = tempfile.mkstemp(
+            dir=str(target.parent), prefix=f".{target.name}.", suffix=".partial"
+        )
+        partial = Path(staged)
+        try:
+            with os.fdopen(fd, "wb") as staged_file:
+                staged_file.write(recorded["json"].encode("utf-8"))
+            os.replace(staged, target)
         except OSError as exc:
+            try:
+                os.close(fd)
+            except OSError:
+                pass
             partial.unlink(missing_ok=True)
             raise CliError(f"writing {plan}: {exc}") from exc
     if not args.quiet:
diff --git a/jubarte-wasm/cli/bin/jubarte-redlines.mjs b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
--- a/jubarte-wasm/cli/bin/jubarte-redlines.mjs	2026-10-21 14:44:05
+++ b/jubarte-wasm/cli/bin/jubarte-redlines.mjs	2026-10-21 16:07:08
@@ -13,6 +13,7 @@
 // 3 edit plan refused (its per-operation report is on stdout; nothing was
 // written).
 
+import crypto from "node:crypto";
 import fs from "node:fs";
 import { createRequire } from "node:module";
 import path from "node:path";
@@ -406,8 +407,10 @@
   if (!preview.ok) return printRefusal(preview);
   if (!o.quiet) for (const note of recorded.notes) console.log(`note: ${note}`);
   if (!o.dry_run) {
-    // Through a symbolic link to the file it names, written yet or not; via a hidden partial
-    // file named for this process, removed if the write fails.
+    // Through a symbolic link to the file it names, written yet or not; via an
+    // exclusive, unpredictable partial file created beside it — the native
+    // recorder's staging design, never a predictable PID-named partial a
+    // planted symlink can sit on — removed if the write fails.
     let target = o.plan;
     try {
       target = fs.realpathSync(o.plan);
@@ -417,12 +420,24 @@
         target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
       } catch {}
     }
-    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
+    const dir = path.dirname(target);
+    let partial;
     try {
-      fs.writeFileSync(partial, recorded.json);
+      for (let attempt = 0; ; attempt++) {
+        partial = path.join(dir, `.${path.basename(target)}.${process.pid}.${crypto.randomUUID()}.partial`);
+        try {
+          const fd = fs.openSync(partial, "wx"); // exclusive: never an existing name, never a link
+          fs.writeFileSync(fd, recorded.json);
+          fs.closeSync(fd);
+          break;
+        } catch (e) {
+          if (e.code === "EEXIST" && attempt < 4) continue;
+          throw e;
+        }
+      }
       fs.renameSync(partial, target);
     } catch (e) {
-      fs.rmSync(partial, { force: true });
+      if (partial !== undefined) fs.rmSync(partial, { force: true });
       throw new CliError(`writing ${o.plan}: ${osError(e)}`);
     }
   }
diff --git a/jubarte-python/tests/test_cli.py b/jubarte-python/tests/test_cli.py
--- a/jubarte-python/tests/test_cli.py	2026-10-21 14:44:05
+++ b/jubarte-python/tests/test_cli.py	2026-10-21 15:54:29
@@ -620,6 +620,26 @@
     assert not plan.exists(), "--consume deletes the applied plan"
     out = capsys.readouterr().out
     assert "{~~his or her~>their~~}" in out and "Why 3?" in out, out
+
+
+def test_plan_recording_never_writes_through_a_planted_staging_symlink(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    """A staging name another process can pre-plant is never used: the
+    recorder stages in an exclusive, unpredictable partial file (the native
+    recorder's design), so a symlink at the old PID-named partial is left
+    alone and the file it names is never written through."""
+    plan = tmp_path / "plan.json"
+    victim = tmp_path / "victim.txt"
+    victim.write_text("do not touch")
+    planted = tmp_path / f".plan.json.{os.getpid()}.partial"
+    planted.symlink_to(victim)
+    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--anchor", "his or her", "--content", "their"]) == 0
+    out = capsys.readouterr().out
+    assert plan.is_file() and "operations" in plan.read_text(), out
+    # The planted link is untouched and its referent keeps its bytes; the
+    # only partial-shaped entry beside the plan is the planted link itself.
+    assert victim.read_text() == "do not touch"
+    assert planted.is_symlink()
+    assert [p.name for p in tmp_path.glob(".*.partial")] == [planted.name]
 
 
 def test_plan_paths_as_typed_and_strict_utf8_as_the_binary(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch) -> None:
diff --git a/jubarte-wasm/cli/test/cli.test.mjs b/jubarte-wasm/cli/test/cli.test.mjs
--- a/jubarte-wasm/cli/test/cli.test.mjs	2026-10-21 14:44:05
+++ b/jubarte-wasm/cli/test/cli.test.mjs	2026-10-21 16:08:48
@@ -788,6 +788,33 @@
   });
 }
 
+test("plan recording never writes through a planted staging symlink (integration)", () => {
+  const plan = path.join(tmp, "staged-plan.json");
+  assert.equal(run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.").code, 0);
+  const held = fs.readFileSync(plan, "utf8");
+  // The pre-hook body runs inside the recording process: process.pid is the
+  // PID the old recorder's predictable partial name used, so a symlink
+  // planted at exactly that name is the attack. The verify closure runs
+  // after the CLI finished.
+  const result = runWithPlanHook(plan, (fs, mock, assert, plan) => {
+    const victim = plan.replace(/staged-plan\.json$/, "staged-victim.txt");
+    const planted = plan.replace(/([^/]+)$/, ".$1." + process.pid + ".partial");
+    fs.writeFileSync(victim, "do not touch");
+    fs.symlinkSync(victim, planted);
+    return () => {
+      assert.equal(process.exitCode, 0, "the recording itself succeeds");
+      assert.equal(fs.readFileSync(victim, "utf8"), "do not touch", "a planted staging symlink is never written through");
+      assert.ok(fs.lstatSync(planted).isSymbolicLink(), "the planted link itself is left alone");
+      const plan_now = fs.readFileSync(plan, "utf8");
+      assert.notEqual(plan_now, held, "the new recording was published");
+      const dir = plan.slice(0, plan.lastIndexOf("/"));
+      const partials = fs.readdirSync(dir).filter((n) => n.startsWith(".staged-plan.json.") && n.endsWith(".partial"));
+      assert.deepEqual(partials, [planted.split("/").pop()], "the recorder leaves no staging file behind");
+    };
+  }, ["add", untracked, "--plan", plan, "-p", "p2", "--comment", "--content", "Why?"]);
+  assert.equal(result.code, 0, result.err);
+});
+
 test("--consume holds its descriptor through unlink and rereads from offset zero (integration)", () => {
   const plan = path.join(tmp, "consume-unlink.json");
   const out = path.join(tmp, "consume-unlink.docx");
diff --git a/tests/markdown_to_docx.rs b/tests/markdown_to_docx.rs
--- a/tests/markdown_to_docx.rs	2026-10-21 14:44:05
+++ b/tests/markdown_to_docx.rs	2026-10-21 16:08:48
@@ -64,7 +64,38 @@
     part_string(docx, "word/document.xml").unwrap()
 }
 
+/// `![label](drawing:PAYLOAD)` with URL-safe base64, unpadded, as `at`
+/// accepts: the payload's own alphabet.
+fn drawing_payload(json: &str) -> String {
+    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
+    let mut out = String::new();
+    for chunk in json.as_bytes().chunks(3) {
+        let bytes = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
+        let n = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
+        out.push(TABLE[(n >> 18) as usize & 63] as char);
+        out.push(TABLE[(n >> 12) as usize & 63] as char);
+        if chunk.len() > 1 {
+            out.push(TABLE[(n >> 6) as usize & 63] as char);
+        }
+        if chunk.len() > 2 {
+            out.push(TABLE[n as usize & 63] as char);
+        }
+    }
+    out
+}
+
 #[test]
+fn a_cyclic_drawing_payload_falls_back_to_its_alt_text() {
+    // A `drawing:` payload whose part depends on itself once recursed
+    // until the stack died during Markdown conversion; decode refuses the
+    // cycle and the writer keeps the label's alt text (dedup-0012).
+    let json = r#"{"xml":"<w:drawing/>","rels":[{"sym":"r1","t":"image","part":0}],"deps":[{"name":"word/media/x.png","t":"image/png","b64":"AAAA","rels":[{"sym":"d0.1","t":"image","part":0}]}]}"#;
+    let payload = drawing_payload(json);
+    let docx = write(&format!("![fallback alt](drawing:{payload})"));
+    assert_eq!(texts(&docx), ["fallback alt"]);
+}
+
+#[test]
 fn every_block_kind_is_written() {
     let docx = write(
         "---\ntitle: The Terms\nauthor: Jane Roe\n---\n\n\

```
