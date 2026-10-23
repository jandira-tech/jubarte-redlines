import pathlib, sys

path = pathlib.Path("new/src/legacy_doc.rs")
text = path.read_text()

anchor = """        let whole = main_text(&word, &[piece(0, 5, 0), piece(5, 10, 5)], 10).unwrap();
        assert_eq!(whole.len(), 10);
        // An empty piece (fast-saved files carry them) is no gap.
        let empty = [
            piece(0, 0, 0),
            piece(0, 5, 0),
            piece(5, 5, 9),
            piece(5, 10, 5),
        ];
        assert_eq!(main_text(&word, &empty, 10).unwrap().len(), 10);
    }
"""
addition = anchor + """
    #[test]
    fn aliased_pieces_claiming_a_huge_story_are_refused_before_expansion() {
        // 4,096 pieces that all reuse one ten-byte physical range would
        // materialize millions of StoryChar records from a ~49 KiB piece
        // table. The story a document may claim is bounded by its stream.
        let word = b"HelloWorld".to_vec();
        let piece = |cp_start, cp_end| Piece {
            cp_start,
            cp_end,
            fc: 0,
            compressed: true,
            modifier: CharModifier::default(),
        };
        let mut aliased = Vec::new();
        let mut at = 0u32;
        for _ in 0..4096 {
            aliased.push(piece(at, at + 1000));
            at += 1000;
        }
        let error = main_text(&word, &aliased, at).unwrap_err().to_string();
        assert!(error.contains("claims more characters"), "{error}");
    }

    #[test]
    fn a_fast_saved_piece_reusing_a_range_still_reads() {
        // Two pieces may share one physical range (a fast save): within
        // the budget the story reads the same bytes twice, as Word does.
        let word = b"AbAb".to_vec();
        let piece = |cp_start, cp_end, fc| Piece {
            cp_start,
            cp_end,
            fc,
            compressed: true,
            modifier: CharModifier::default(),
        };
        let chars: String = main_text(&word, &[piece(0, 2, 0), piece(2, 4, 0)], 4)
            .unwrap()
            .iter()
            .map(|c| c.ch)
            .collect();
        assert_eq!(chars, "AbAb");
    }

    #[test]
    fn field_markers_keep_their_meaning_at_the_edges() {
        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
        let lists = Lists::default();
        let texts = |text: &str| {
            let story = Story {
                papx: &papx,
                chpx: &chpx,
                styles: &[],
                lists: &lists,
                section_marks: Vec::new(),
            };
            story
                .blocks(&story_chars(text))
                .into_iter()
                .map(|b| match b {
                    Block::Paragraph(p) => p.text(),
                    Block::Table(_) => "table".into(),
                })
                .collect::<Vec<_>>()
        };
        // A separator with no begin, a second separator and an unmatched
        // end change nothing: only an open code frame hides text.
        assert_eq!(texts("\\u{14}x\\r"), ["x"]);
        assert_eq!(texts("\\u{13}a\\u{14}b\\u{14}c\\u{15}d\\r"), ["bcd"]);
        assert_eq!(texts("\\u{15}e\\r"), ["e"]);
    }

    #[test]
    fn deeply_nested_field_results_read_without_rescanning_the_stack() {
        // 2^19 begin/separator pairs leave every frame in its result
        // phase, so the payload stays visible. Deciding that is one count
        // comparison per character; rescanning every enclosing frame (the
        // old rule) made this input quadratic, and a small crafted story
        // monopolized the reader.
        let n = 1 << 19;
        let mut text = String::with_capacity(3 * n + 8);
        for _ in 0..n {
            text.push('\\u{13}');
            text.push('\\u{14}');
        }
        let payload = "payload ".repeat(n / 8);
        text.push_str(&payload);
        for _ in 0..n {
            text.push('\\u{15}');
        }
        text.push_str("done\\r");
        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
        let lists = Lists::default();
        let story = Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks: Vec::new(),
        };
        let texts: Vec<String> = story
            .blocks(&story_chars(&text))
            .into_iter()
            .map(|b| match b {
                Block::Paragraph(p) => p.text(),
                Block::Table(_) => "table".into(),
            })
            .collect();
        assert_eq!(texts, [format!("{payload}done")]);
    }
"""
count = text.count(anchor)
if count != 1:
    print(f"anchor count {count}", file=sys.stderr)
    sys.exit(1)
text = text.replace(anchor, addition)
path.write_text(text)
print("legacy tests applied")
