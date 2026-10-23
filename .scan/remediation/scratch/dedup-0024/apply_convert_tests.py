import pathlib, sys

path = pathlib.Path("new/src/convert/mod.rs")
text = path.read_text()

anchor = """            | Block::SectionCols { .. } => {
                panic!("expected table")
            }
        }
    }

    #[test]
    fn a_shading_pattern_paints_its_colour_over_the_fill() {"""
new_tests = """            | Block::SectionCols { .. } => {
                panic!("expected table")
            }
        }
    }

    /// A `table_block` run over a whole document (the module's fixture
    /// pattern), for the grid-budget tests below.
    fn block_of(body: &str) -> Block {
        let (dom, tbl) = first_tbl(body);
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
            latent: true,
            default_table: true,
        };
        let mut numbering = Numbering::default();
        table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
            None,
        )
    }

    #[test]
    fn a_huge_grid_span_cannot_size_the_grid() {
        // One cell claiming a billion columns used to size the column
        // vector from the claim (about 4 GiB of f32) before any layout
        // happened. The grid a document can ask for is bounded.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>
<w:tr><w:tc><w:tcPr><w:gridSpan w:val="1000000000"/></w:tcPr>
  <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl></w:body></w:document>"#;
        match block_of(xml) {
            Block::Table { cols, .. } => {
                assert!(cols.len() <= MAX_TABLE_COLUMNS, "{}", cols.len());
            }
            Block::Paragraph { .. }
            | Block::PageBreak { .. }
            | Block::ColumnBreak
            | Block::SectionCols { .. } => {
                panic!("expected table")
            }
        }
    }

    #[test]
    fn a_huge_grid_skip_cannot_size_the_grid() {
        // `w:gridBefore`/`w:gridAfter` are placeholder spans the same
        // budget holds: a row skipping four billion columns cannot size
        // the grid either.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>
<w:tr><w:trPr><w:gridBefore w:val="4294967295"/></w:trPr><w:tc>
  <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl></w:body></w:document>"#;
        match block_of(xml) {
            Block::Table { cols, .. } => {
                assert!(cols.len() <= MAX_TABLE_COLUMNS, "{}", cols.len());
            }
            Block::Paragraph { .. }
            | Block::PageBreak { .. }
            | Block::ColumnBreak
            | Block::SectionCols { .. } => {
                panic!("expected table")
            }
        }
    }

    #[test]
    fn a_row_of_spans_beyond_the_budget_keeps_the_grid_bounded() {
        // Each span is under the ceiling; their sum (8000) is not, and the
        // grid they size is capped at the same budget.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>
<w:tr>
  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc>
  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc>
  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc>
  <w:tc><w:tcPr><w:gridSpan w:val="2000"/></w:tcPr><w:p><w:r><w:t>d</w:t></w:r></w:p></w:tc>
</w:tr>
</w:tbl></w:body></w:document>"#;
        match block_of(xml) {
            Block::Table { cols, .. } => {
                assert!(cols.len() <= MAX_TABLE_COLUMNS, "{}", cols.len());
            }
            Block::Paragraph { .. }
            | Block::PageBreak { .. }
            | Block::ColumnBreak
            | Block::SectionCols { .. } => {
                panic!("expected table")
            }
        }
    }

    #[test]
    fn a_wide_word_table_still_lays_out_its_columns() {
        // Word's own tables never pass 63 columns: a 63-span cell still
        // splits the one grid column into 63 (the degenerate-grid rule).
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblGrid><w:gridCol w:w="6300"/></w:tblGrid>
<w:tr><w:tc><w:tcPr><w:gridSpan w:val="63"/></w:tcPr>
  <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl></w:body></w:document>"#;
        match block_of(xml) {
            Block::Table { cols, .. } => assert_eq!(cols.len(), 63),
            Block::Paragraph { .. }
            | Block::PageBreak { .. }
            | Block::ColumnBreak
            | Block::SectionCols { .. } => {
                panic!("expected table")
            }
        }
    }

    #[test]
    fn a_shading_pattern_paints_its_colour_over_the_fill() {"""

count = text.count(anchor)
if count != 1:
    print(f"anchor count {count}", file=sys.stderr)
    sys.exit(1)
text = text.replace(anchor, new_tests)
path.write_text(text)
print("table_tests additions applied")
