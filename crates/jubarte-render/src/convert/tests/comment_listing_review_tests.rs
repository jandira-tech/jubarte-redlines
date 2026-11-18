// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    // Use the dependency's embedded catalogue to bypass every system
    // font override: unit tests must not read installed font files.
    for face in FaceId::all() {
        let key = face.key();
        fonts.insert_embedded(&key.family, key.bold, key.italic, face.bytes());
    }
    fonts
}

fn sheet() -> StyleSheet {
    StyleSheet {
        defaults: Defaults::word(),
        by_id: HashMap::new(),
        tables: HashMap::new(),
        theme: ThemeFonts::default(),
        latent: true,
        default_table: true,
    }
}

fn note(author: &str, contents: &str) -> PdfComment {
    PdfComment {
        id: "1".into(),
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
        top: 0.0,
        bottom: 0.0,
        contents: contents.into(),
        author: author.into(),
        initials: "JR".into(),
        label: "1".into(),
        color: [0.8, 0.0, 0.0],
        resolved: false,
    }
}

#[test]
fn synthetic_comment_markers_do_not_enter_field_sources() {
    let mut host = TextRun::new("Hello", default_run_style());
    host.comments.push(CommentNote {
        id: "1".into(),
        author: "Jane".into(),
        initials: "JR".into(),
        label: "1".into(),
        color: [0.8, 0.0, 0.0],
        after: true,
        resolved: false,
        text: "Note".into(),
    });
    let mut runs = vec![host];
    mark_comment_anchors(&mut runs);
    assert_eq!(run_word_count(&runs), 1);
    let block = Block::Paragraph {
        runs: runs.clone(),
        style: Defaults::word().para,
        list: false,
        images: Vec::new(),
        boxes: Vec::new(),
        bookmarks: vec!["Here".into()],
    };
    assert_eq!(document_bookmark_texts(&[block])["Here"], "Hello");
    let fonts = fonts();
    let mut layout = Layout::new(&fonts, Defaults::word().page, HfChrome::default(), 15);
    layout.last_style_id = "Heading1".into();
    for run in &runs {
        layout.note_style_hit(run);
    }
    assert_eq!(layout.style_hits[0][0].text, "Hello");
    layout.style_hits = Default::default();
    layout.paint_stretched(&runs, 72.0, 700.0, 2.0);
    assert_eq!(layout.style_hits[0][0].text, "Hello");
    assert!(runs.iter().any(|run| run.text == "[JR1]"));
}

#[test]
fn wrapping_keeps_synthetic_marker_classification_separate() {
    let mut style = default_run_style();
    style.vert = VertAlign::Super;
    let host = TextRun::new("Hello ", style.clone());
    let mut marker = TextRun::new("[JR1]", style);
    marker.comment_marker = true;
    let lines = wrap_runs(&fonts(), &[host, marker], 300.0, 300.0, false);
    assert_eq!(
        lines[0].len(),
        2,
        "display-only text must not merge with field sources"
    );
    assert!(!lines[0][0].comment_marker);
    assert!(lines[0][1].comment_marker);
    assert_eq!(run_word_count(&lines[0]), 1);
}

#[test]
fn a_script_listing_phrase_keeps_spaces_and_word_order_when_shaping() {
    let mut fonts = fonts();
    // A private embedded family bypasses catalogue system overrides.
    fonts.insert_embedded("Review Latin", false, false, FaceId::CarlitoRegular.bytes());
    let mut style = default_run_style();
    style.family = "Review Latin".into();
    style.family_cs = None;
    style.size = 10.0;
    let phrase = "שלום עולם";
    let fallback = ink_face(&fonts, &style, phrase);
    assert_ne!(fallback, fonts.resolve(&style.family, false, false));
    let pieces = listing_faces(&fonts, &style, phrase);
    assert_eq!(
        pieces,
        vec![(fallback, phrase)],
        "neutral spaces stay with the script face"
    );
    let lines = wrap_listing_runs(&fonts, &[TextRun::new(phrase, style)], 400.0);
    assert_eq!(lines[0].len(), 1, "coalesce words for shaping and kerning");
    let mut page = Page::new(612.0, 792.0);
    paint_listing_line(&fonts, &mut page, &lines[0], 72.0, 700.0);
    let shaped: Vec<u16> = fonts
        .get(fallback)
        .shape_kern(phrase, 10.0, true)
        .iter()
        .map(|(glyph, _)| *glyph)
        .collect();
    assert!(
        matches!(&page.ops[0], Op::Text { text, glyphs, .. } if text == phrase && glyphs == &shaped)
    );
    assert_eq!(page.ops.len(), 1);
}

#[test]
fn comment_listing_uses_loaded_script_fallback_for_paint_and_width() {
    // Inject a deterministic document-local face in the dependency's
    // own font catalogue; no installed font or filesystem lookup.
    let mut fonts = fonts();
    fonts.insert_embedded(
        font::CJK_FALLBACK,
        false,
        false,
        FaceId::SansRegular.bytes(),
    );
    let fallback = fonts
        .cjk_glyph_fallback(false, None)
        .expect("loaded fallback");
    let mut pages = vec![Page::new(240.0, 300.0)];
    pages[0]
        .comments
        .push(note("Jane", "中文中文中文中文中文中文中文中文中文中文"));
    list_comments_at_end(&fonts, &sheet(), &mut pages, 0);
    let mut painted = 0;
    for page in &pages[1..] {
        for op in &page.ops {
            if let Op::Text {
                face,
                text,
                size,
                x,
                ..
            } = op
                && text.chars().any(is_cjk)
            {
                assert_eq!(*face, fallback);
                assert!(*x + advance(&fonts, *face, text, *size) <= page.width - 72.0 + 0.01);
                painted += text.chars().count();
            }
        }
    }
    assert_eq!(painted, 20);
}

#[test]
fn long_comment_attributions_wrap_and_preserve_printable_page_bounds() {
    let fonts = fonts();
    let mut pages = vec![Page::new(240.0, 230.0)];
    pages[0].number = 1;
    let author = "An exceptionally long author attribution that needs several lines";
    pages[0].comments.push(note(author, "First content line"));
    list_comments_at_end(&fonts, &sheet(), &mut pages, 0);
    let mut all = String::new();
    for page in &pages[1..] {
        for op in &page.ops {
            if let Op::Text {
                face,
                text,
                size,
                x,
                y,
                ..
            } = op
            {
                assert!(
                    *x + advance(&fonts, *face, text, *size) <= page.width - 72.0 + 0.01,
                    "{text}"
                );
                assert!(*y >= 72.0, "{text}");
                all.push_str(text);
                all.push(' ');
            }
        }
    }
    assert!(
        pages.len() > 2,
        "heading and contents must spill onto a fresh listing page"
    );
    let all = all.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(all.contains("First content"));
    assert!(all.contains(author));
    assert!(all.contains("page 1"));
}

#[test]
fn a_comment_attribution_never_ends_a_page_without_its_first_line() {
    let fonts = fonts();
    // Every height puts some comment's end at a different place on the
    // page, so one of them leaves room for the attribution alone.
    for height in 200..300 {
        let mut pages = vec![Page::new(400.0, height as f32)];
        pages[0].number = 1;
        for i in 0..6 {
            pages[0]
                .comments
                .push(note(&format!("A{i}"), &format!("Body{i}")));
        }
        list_comments_at_end(&fonts, &sheet(), &mut pages, 0);
        let page_of = |needle: &str| {
            pages.iter().position(|page| {
                page.ops
                    .iter()
                    .any(|op| matches!(op, Op::Text { text, .. } if text.contains(needle)))
            })
        };
        for i in 0..6 {
            let by = page_of(&format!("A{i},")).expect("attribution painted");
            let body = page_of(&format!("Body{i}")).expect("contents painted");
            assert_eq!(
                by, body,
                "height {height}: comment {i} split from its first line"
            );
        }
    }
}

#[test]
fn a_listing_after_vertical_pages_takes_their_paper_size() {
    let fonts = fonts();
    // A4 laid out turned a quarter: the page stores its height as width.
    let mut page = Page::new(841.9, 595.3);
    page.vertical = true;
    page.number = 1;
    page.comments.push(note("Jane", "縦書き"));
    let mut pages = vec![page];
    list_comments_at_end(&fonts, &sheet(), &mut pages, 0);
    let listing = &pages[1];
    assert_eq!((listing.width, listing.height), (595.3, 841.9));
    assert!(!listing.vertical);
}
