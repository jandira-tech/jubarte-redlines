// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-validity regressions for synthesized revision records. Word treats
//! a colliding `w:id` on two `w:*Change` records as the same revision and
//! drops the later one, and repairs an out-of-order `CT_RPr`/`CT_TblPrBase`
//! child sequence — so the synthesized records must use a free id and land
//! in their schema slot.
use super::*;

/// One unit per element and per character of `w:t`, `w:delText`,
/// `w:instrText` and `w:delInstrText`; other text costs nothing.
#[test]
fn story_units_count_elements_and_the_characters_of_text() {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
            "<w:body xmlns:w=\"{}\"><w:p><w:r><w:t>ab\u{e9}</w:t><w:delText>cd</w:delText><w:instrText> P </w:instrText><w:delInstrText>e</w:delInstrText><w:tab/></w:r></w:p><w:p/></w:body>",
            W::URI
        ));
    let root = dom.root(doc).unwrap();
    // 9 elements (body, 2 p, r, 4 text, tab) + 3 + 2 + 3 + 1 characters.
    assert_eq!(story_units(&dom, root), 9 + 9);
    let limits = crate::admission::InputLimits {
        max_story_units: 18,
        ..crate::admission::InputLimits::compare()
    };
    assert!(refuse_past_story_budget(18, "s", limits).is_ok());
    let err = refuse_past_story_budget(19, "s", limits)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("s holds 19 story units") && err.contains("limit is 18"),
        "{err}"
    );
}

fn parse(dom: &mut Dom, xml: &str) -> (NodeId, NodeId) {
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let styles = dom
        .element(root, &W::name("styles"))
        .or_else(|| dom.descendants(root, None).first().copied())
        .expect("styles root");
    (root, styles)
}

/// A's working copy carries `pt14:Unid` on its spacing; the restored
/// spacing on the deleted paragraph must hold only the `w:` attributes.
#[test]
fn restored_deleted_spacing_leaves_scratch_unids_behind() {
    let ns = format!(
        "xmlns:w=\"{}\" xmlns:w14=\"{}\" xmlns:pt14=\"{}\"",
        W::URI,
        crate::namespaces::W14::URI,
        crate::namespaces::PT::URI
    );
    let a = format!(
        "<w:document {ns}><w:body><w:p w14:paraId=\"4B0CC135\"><w:pPr>\
             <w:spacing w:line=\"276\" w:lineRule=\"auto\" pt14:Unid=\"76\"/></w:pPr>\
             <w:r><w:t>x</w:t></w:r></w:p></w:body></w:document>"
    );
    let out = format!(
        "<w:document {ns}><w:body><w:p w14:paraId=\"4B0CC135\"><w:pPr>\
             <w:jc w:val=\"right\"/><w:rPr><w:del w:id=\"1\" w:author=\"a\"/></w:rPr></w:pPr>\
             </w:p></w:body></w:document>"
    );
    let restored = restore_deleted_paragraph_spacing(&a, &out).expect("spacing restored");
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&restored);
    let root = dom.root(d).unwrap();
    let sp = dom.descendants(root, Some(&W::spacing_el()))[0];
    let mut names: Vec<String> = dom.attributes_iter(sp).map(|(n, _)| n.clark()).collect();
    names.sort();
    let w = |l: &str| W::name(l).clark();
    assert_eq!(names, vec![w("line"), w("lineRule")], "{restored}");
}

/// Fonts only the revised document declares join the output's font table
/// (file_46 × file_47: without Liberation Serif's entry Word fell back to
/// Times New Roman where its own redline used Hiragino Mincho). Fonts the
/// output already lists stay as they are, and embedded-font children are
/// left behind: their relationships and obfuscation keys belong to B.
#[test]
fn revised_fonts_join_the_font_table() {
    let ns = format!("xmlns:w=\"{}\" xmlns:r=\"{}\"", W::URI, R::URI);
    let out = format!(
        "<w:fonts {ns}><w:font w:name=\"Calibri\"><w:charset w:val=\"00\"/></w:font></w:fonts>"
    );
    let b = format!(
        "<w:fonts {ns}><w:font w:name=\"Calibri\"><w:charset w:val=\"86\"/></w:font>\
             <w:font w:name=\"Droid Sans Fallback\"><w:charset w:val=\"86\"/>\
             <w:family w:val=\"auto\"/><w:embedRegular r:id=\"rId1\" w:fontKey=\"{{0}}\"/></w:font></w:fonts>"
    );
    let merged = merge_revised_font_table(&out, &b).expect("B's font added");
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&merged);
    let root = dom.root(d).unwrap();
    let fonts: Vec<(String, String)> = dom
        .elements(root, Some(&W::name("font")))
        .into_iter()
        .map(|f| {
            let cs = dom
                .element(f, &W::name("charset"))
                .and_then(|c| dom.attribute(c, &W::val()))
                .unwrap_or("")
                .to_string();
            (dom.attribute(f, &W::name("name")).unwrap().to_string(), cs)
        })
        .collect();
    assert_eq!(
        fonts,
        vec![
            ("Calibri".to_string(), "00".to_string()),
            ("Droid Sans Fallback".to_string(), "86".to_string())
        ],
        "{merged}"
    );
    assert!(!merged.contains("embedRegular"), "{merged}");
    assert_eq!(merge_revised_font_table(&merged, &b), None, "nothing new");
}

/// `next_free_revision_id` must be one greater than the max numeric id on
/// ANY `w:*Change` revision element in the stylesheet, never a hardcoded 1.
#[test]
fn next_free_revision_id_is_max_plus_one_across_change_families() {
    let mut dom = Dom::new();
    // A stylesheet whose Normal carries a high-id rPrChange and another
    // style carries a pPrChange — both must raise the floor.
    let xml = concat!(
        "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
        "<w:style w:type=\"paragraph\" w:styleId=\"Normal\">",
        "<w:rPr><w:rFonts w:ascii=\"Times\"/><w:rPrChange w:id=\"147\" w:author=\"x\" w:date=\"d\">",
        "<w:rPr><w:sz w:val=\"22\"/></w:rPr></w:rPrChange></w:rPr>",
        "</w:style>",
        "<w:style w:type=\"paragraph\" w:styleId=\"Heading1\">",
        "<w:pPr><w:pPrChange w:id=\"93\" w:author=\"x\" w:date=\"d\">",
        "<w:pPr/></w:pPrChange></w:pPr>",
        "</w:style>",
        "</w:styles>"
    );
    let (_root, styles) = parse(&mut dom, xml);
    assert_eq!(
        next_free_revision_id(&dom, styles),
        148,
        "next free id must exceed the highest existing *Change id (147), not be 1"
    );
}

/// Every `w:id` revision carrier raises the floor, not only the `*Change`
/// records: a list, move-range or table-cell revision holding the next id
/// would collide with a synthesized `w:pPrChange`.
#[test]
fn next_free_revision_id_counts_list_move_range_and_cell_carriers() {
    for (carrier, id) in [
        ("numberingChange", 40),
        ("moveFromRangeStart", 41),
        ("moveToRangeEnd", 42),
        ("cellIns", 43),
        ("cellDel", 44),
        ("cellMerge", 45),
    ] {
        let mut dom = Dom::new();
        let xml = format!(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>\
                 <w:ins w:id=\"7\" w:author=\"x\" w:date=\"d\"/><w:{carrier} w:id=\"{id}\" w:author=\"x\" w:date=\"d\"/>\
                 </w:body></w:document>"
        );
        let (_root, doc) = parse(&mut dom, &xml);
        assert_eq!(next_free_revision_id(&dom, doc), id + 1, "{carrier}");
    }
}

/// sz must be inserted after position/kern, NOT immediately after rFonts —
/// rFonts < color < spacing < w < kern < position < sz in EG_RPrBase.
#[test]
fn add_rpr_child_keeps_sz_in_schema_order_after_position() {
    let mut dom = Dom::new();
    let xml = concat!(
        "<w:rPr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
        "<w:rFonts w:ascii=\"Times\"/>",
        "<w:color w:val=\"auto\"/>",
        "<w:spacing w:val=\"0\"/>",
        "<w:kern w:val=\"0\"/>",
        "<w:position w:val=\"0\"/>",
        "</w:rPr>"
    );
    let doc = dom.parse_xdocument(xml);
    let rpr = dom.root(doc).expect("root");
    let sz = dom.new_element(W::name("sz"));
    add_rpr_child_in_order(&mut dom, rpr, sz, "sz");
    let order: Vec<String> = dom
        .elements(rpr, None)
        .into_iter()
        .map(|e| dom.name(e).unwrap().local_name().to_string())
        .collect();
    let sz_pos = order.iter().position(|n| n == "sz").unwrap();
    let pos_pos = order.iter().position(|n| n == "position").unwrap();
    assert!(
        sz_pos > pos_pos,
        "sz must follow position (EG_RPrBase order), got order: {order:?}"
    );
}

/// `revision_to_json` / `revisions_to_json` are the single serialization
/// of the wasm `getRevisions` and Python `get_revisions` bindings: one
/// object shape, full string escaping —
/// backslash, quote, and EVERY control char < 0x20 (document text can
/// carry tabs, CRs, vertical tabs).
#[test]
fn revision_json_matches_cli_shape_and_escapes_control_chars() {
    use crate::comparer::atoms::FormatChangeInfo;
    use crate::comparer::{WmlComparerRevision, WmlComparerRevisionType};

    let inserted = WmlComparerRevision {
        revision_type: WmlComparerRevisionType::Inserted,
        text: Some("a\"b\\c\nd\te\u{000B}f".to_string()),
        author: Some("Reviewer \"X\"".to_string()),
        date: Some("2026-07-17T00:00:00Z".to_string()),
        content_element: None,
        revision_element: None,
        part_name: "word/document.xml".to_string(),
        move_group_id: Some(3),
        is_move_source: Some(true),
        format_change: None,
    };
    assert_eq!(
        revision_to_json(&inserted),
        concat!(
            "{\"type\":\"Inserted\",\"author\":\"Reviewer \\\"X\\\"\",",
            "\"date\":\"2026-07-17T00:00:00Z\",\"part\":\"word/document.xml\",",
            "\"moveGroupId\":3,\"isMoveSource\":true,\"formatChange\":null,",
            "\"text\":\"a\\\"b\\\\c\\nd\\te\\u000bf\"}"
        )
    );

    let format_changed = WmlComparerRevision {
        revision_type: WmlComparerRevisionType::FormatChanged,
        text: None,
        author: None,
        date: None,
        content_element: None,
        revision_element: None,
        part_name: "word/document.xml".to_string(),
        move_group_id: None,
        is_move_source: None,
        format_change: Some(FormatChangeInfo {
            changed_properties: vec!["bold".to_string(), "sz".to_string()],
            ..FormatChangeInfo::default()
        }),
    };
    assert_eq!(
        revision_to_json(&format_changed),
        concat!(
            "{\"type\":\"FormatChanged\",\"author\":\"\",\"date\":\"\",",
            "\"part\":\"word/document.xml\",\"moveGroupId\":null,",
            "\"isMoveSource\":null,",
            "\"formatChange\":{\"changedProperties\":[\"bold\",\"sz\"]},",
            "\"text\":\"\"}"
        )
    );

    // The wasm array shape is exactly the objects joined inside [].
    let expected_array = format!(
        "[{},{}]",
        revision_to_json(&inserted),
        revision_to_json(&format_changed)
    );
    assert_eq!(
        revisions_to_json(&[inserted, format_changed]),
        expected_array
    );
    assert_eq!(revisions_to_json(&[]), "[]");
}

#[test]
fn word_canonical_style_id_preserves_toc_builtins() {
    // Regression: TOC 1..9 are ALL-CAPS built-in styleIds (TOC1..TOC9,
    // name "toc N"). The generic PascalCase fallback would mangle them to
    // Toc1.., renaming a live built-in to a custom id — LibreOffice/Word
    // then drop the built-in TOC indents + dot-leader tabs and the table of
    // contents reflows, collapsing the visual redline score.
    assert_eq!(word_canonical_style_id("toc 1"), "TOC1");
    assert_eq!(word_canonical_style_id("toc 9"), "TOC9");
    assert_eq!(word_canonical_style_id("TOC 2"), "TOC2"); // name matched case-insensitively
    assert_eq!(word_canonical_style_id("toc heading"), "TOCHeading");
    // Sibling built-ins and the generic PascalCase path are unaffected.
    assert_eq!(word_canonical_style_id("heading 1"), "Heading1");
    assert_eq!(word_canonical_style_id("document title"), "DocumentTitle");
    assert_eq!(word_canonical_style_id("my custom style"), "MyCustomStyle");
}

#[test]
fn word_canonical_style_id_keeps_builtin_ids_that_differ_from_their_names() {
    // Word's own ids; the PascalCased name would rename a live built-in.
    assert_eq!(
        word_canonical_style_id("annotation reference"),
        "CommentReference"
    );
    assert_eq!(word_canonical_style_id("annotation text"), "CommentText");
    assert_eq!(
        word_canonical_style_id("annotation subject"),
        "CommentSubject"
    );
    assert_eq!(word_canonical_style_id("macro"), "MacroText");
    assert_eq!(word_canonical_style_id("toa heading"), "TOAHeading");
    assert_eq!(
        word_canonical_style_id("table of figures"),
        "TableofFigures"
    );
    assert_eq!(
        word_canonical_style_id("table of authorities"),
        "TableofAuthorities"
    );
}

/// Word's style ids hold letters and digits only: "Normal (Web)" is
/// `NormalWeb` (evals comments_doc × document). Keeping the parentheses
/// renamed B's live `NormalWeb` to a custom `Normal(Web)`.
#[test]
fn word_canonical_style_id_drops_punctuation() {
    assert_eq!(word_canonical_style_id("Normal (Web)"), "NormalWeb");
    assert_eq!(word_canonical_style_id("Body Text 2"), "BodyText2");
}

/// B's docDefaults name their fonts by theme (`w:asciiTheme="minorHAnsi"`,
/// the form Word itself writes) and A's name Times New Roman outright.
/// Word's live Normal carries B's theme fonts, so the output renders in
/// Calibri as B does (instrtext_angled_brackets_bug × table_merged_cells:
/// the theme attributes were not read, Normal kept A's Times New Roman and
/// the pair scored 0.13 Jaccard against Word, docxodus 0.82).
#[test]
fn normal_takes_b_theme_fonts_from_its_doc_defaults() {
    let ns = format!("xmlns:w=\"{}\"", W::URI);
    let a = format!(
        "<w:styles {ns}><w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:ascii=\"Times New Roman\" w:eastAsia=\"Times New Roman\" \
             w:hAnsi=\"Times New Roman\" w:cs=\"Times New Roman\"/></w:rPr></w:rPrDefault>\
             </w:docDefaults><w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/><w:rPr><w:sz w:val=\"24\"/></w:rPr></w:style></w:styles>"
    );
    let b = format!(
        "<w:styles {ns}><w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:asciiTheme=\"minorHAnsi\" w:eastAsiaTheme=\"minorHAnsi\" \
             w:hAnsiTheme=\"minorHAnsi\" w:cstheme=\"minorBidi\"/><w:sz w:val=\"24\"/>\
             </w:rPr></w:rPrDefault></w:docDefaults>\
             <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/></w:style></w:styles>"
    );
    let mut dom = Dom::new();
    let (out_root, _) = parse(&mut dom, &a);
    let (b_root, _) = parse(&mut dom, &b);
    let settings = WmlComparerSettings::default();
    assert!(merge_normal_style_rpr(
        &mut dom, out_root, b_root, &settings
    ));
    let normal = find_normal_style(&dom, out_root).expect("Normal");
    let rpr = dom.element(normal, &W::r_pr()).expect("live rPr");
    let fonts = dom.element(rpr, &W::name("rFonts")).expect("live rFonts");
    let attr = |n: &str| dom.attribute(fonts, &W::name(n)).map(str::to_string);
    assert_eq!(attr("asciiTheme").as_deref(), Some("minorHAnsi"));
    assert_eq!(attr("hAnsiTheme").as_deref(), Some("minorHAnsi"));
    assert_eq!(attr("eastAsiaTheme").as_deref(), Some("minorHAnsi"));
    assert_eq!(attr("cstheme").as_deref(), Some("minorBidi"));
    // No explicit face left behind to contradict the theme on the page.
    assert_eq!(attr("ascii"), None);
    assert_eq!(attr("hAnsi"), None);
}

/// LibreOffice names its Normal `style0` and marks no paragraph style as
/// the default; Word still finds it by its name (multi_section_nested_
/// table_rowspan: the Word redline carries B's whole Normal, ours left it
/// bare because B "had no Normal").
#[test]
fn normal_is_found_by_name_when_no_style_is_the_default() {
    let ns = format!("xmlns:w=\"{}\"", W::URI);
    let s = format!(
        "<w:styles {ns}><w:style w:styleId=\"style15\" w:type=\"paragraph\">\
             <w:name w:val=\"Heading\"/></w:style>\
             <w:style w:styleId=\"style0\" w:type=\"paragraph\"><w:name w:val=\"Normal\"/>\
             </w:style></w:styles>"
    );
    let mut dom = Dom::new();
    let (root, _) = parse(&mut dom, &s);
    let normal = find_normal_style(&dom, root).expect("Normal found by name");
    assert_eq!(dom.attribute(normal, &W::style_id()), Some("style0"));
}

/// A stylesheet with a Normal; `dd` = (rPrDefault, pPrDefault) contents,
/// `None` for no docDefaults at all.
fn stylesheet(dd: Option<(&str, &str)>, normal: &str) -> String {
    let ns = format!("xmlns:w=\"{}\"", W::URI);
    let dd = dd
        .map(|(r, p)| {
            format!(
                "<w:docDefaults><w:rPrDefault><w:rPr>{r}</w:rPr></w:rPrDefault>\
                     <w:pPrDefault><w:pPr>{p}</w:pPr></w:pPrDefault></w:docDefaults>"
            )
        })
        .unwrap_or_default();
    format!(
        "<w:styles {ns}>{dd}<w:style w:type=\"paragraph\" w:default=\"1\" \
             w:styleId=\"Normal\"><w:name w:val=\"Normal\"/>{normal}</w:style></w:styles>"
    )
}

fn normal_pair(a_dd_ppr: &str, a_normal: &str, b_dd_ppr: &str, b_normal: &str) -> (String, String) {
    (
        stylesheet(Some(("<w:sz w:val=\"22\"/>", a_dd_ppr)), a_normal),
        stylesheet(
            Some(("<w:kern w:val=\"2\"/><w:sz w:val=\"21\"/>", b_dd_ppr)),
            b_normal,
        ),
    )
}

/// Run `merge_normal_style_spacing` then `merge_normal_style_rpr` over
/// A's and B's stylesheets; returns the dom and the output root.
fn merge_normals(a: &str, b: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let (out_root, _) = parse(&mut dom, a);
    let (b_root, _) = parse(&mut dom, b);
    let settings = WmlComparerSettings::default();
    merge_normal_style_spacing(&mut dom, out_root, b_root, &settings);
    merge_normal_style_rpr(&mut dom, out_root, b_root, &settings);
    (dom, out_root)
}

/// `(attribute, value)` pairs of the live Normal's `container/local`.
fn live_normal_attrs(
    dom: &Dom,
    root: NodeId,
    container: &str,
    local: &str,
) -> Option<Vec<(String, String)>> {
    let normal = find_normal_style(dom, root)?;
    let c = dom.element(normal, &W::name(container))?;
    let e = dom.element(c, &W::name(local))?;
    let mut v: Vec<(String, String)> = dom
        .attributes(e)
        .into_iter()
        .map(|(n, v)| (n.local_name().to_string(), v))
        .collect();
    v.sort();
    Some(v)
}

fn attrs(pairs: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
    let mut v: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    v.sort();
    Some(v)
}

/// A LibreOffice B with no docDefaults at all: Word reads it with its own
/// factory defaults (after 160, line 278, kern 2, standard contextual
/// ligatures) and writes them into the live Normal
/// (multi_section_nested_table_rowspan, table_bookmark_end ×
/// table_vmerge_colspan).
#[test]
fn normal_uses_word_factory_defaults_when_b_has_no_doc_defaults() {
    let a = stylesheet(
        Some((
            "<w:sz w:val=\"22\"/>",
            "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
        )),
        "",
    );
    let b = stylesheet(
        None,
        "<w:pPr><w:widowControl w:val=\"false\"/></w:pPr><w:rPr><w:sz w:val=\"24\"/></w:rPr>",
    );
    let (dom, root) = merge_normals(&a, &b);
    assert_eq!(
        live_normal_attrs(&dom, root, "pPr", "spacing"),
        attrs(&[("after", "160"), ("line", "278"), ("lineRule", "auto")])
    );
    assert_eq!(
        live_normal_attrs(&dom, root, "rPr", "kern"),
        attrs(&[("val", "2")])
    );
    let normal = find_normal_style(&dom, root).unwrap();
    let rpr = dom.element(normal, &W::r_pr()).unwrap();
    let lig = dom
        .element(rpr, &W14::name("ligatures"))
        .expect("ligatures");
    assert_eq!(
        dom.attribute(lig, &W14::name("val")),
        Some("standardContextual")
    );
}

/// B declares no complex-script size while A's docDefaults do: Word
/// writes the implicit 20 half-points (17 of 17 corpus Word redlines).
#[test]
fn normal_writes_the_implicit_complex_script_size() {
    let a = stylesheet(
        Some(("<w:sz w:val=\"21\"/><w:szCs w:val=\"22\"/>", "")),
        "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
    );
    let b = stylesheet(Some(("<w:sz w:val=\"22\"/>", "")), "");
    let (dom, root) = merge_normals(&a, &b);
    assert_eq!(
        live_normal_attrs(&dom, root, "rPr", "sz"),
        attrs(&[("val", "22")])
    );
    assert_eq!(
        live_normal_attrs(&dom, root, "rPr", "szCs"),
        attrs(&[("val", "20")])
    );
}

/// Word writes only the language attributes B changes against A's
/// docDefaults, reading a missing attribute as en-US / en-US / ar-SA
/// (file_103 × file_104: eastAsia zh-CN alone; mcdoc_meeting_agenda:
/// B silent, A zh-CN → eastAsia en-US).
#[test]
fn normal_writes_only_the_language_attributes_that_change() {
    let lang = |v: &str, ea: &str, bidi: &str| {
        format!("<w:lang w:val=\"{v}\" w:eastAsia=\"{ea}\" w:bidi=\"{bidi}\"/>")
    };
    let a = stylesheet(
        Some((
            &format!("<w:sz w:val=\"22\"/>{}", lang("en-US", "en-US", "ar-SA")),
            "",
        )),
        "",
    );
    let b = stylesheet(
        Some((
            &format!("<w:sz w:val=\"21\"/>{}", lang("en-US", "zh-CN", "ar-SA")),
            "",
        )),
        &format!("<w:rPr>{}</w:rPr>", lang("en-US", "zh-CN", "ar-SA")),
    );
    let (dom, root) = merge_normals(&a, &b);
    assert_eq!(
        live_normal_attrs(&dom, root, "rPr", "lang"),
        attrs(&[("eastAsia", "zh-CN")])
    );

    let a = stylesheet(
        Some((
            &format!("<w:sz w:val=\"22\"/>{}", lang("en-US", "zh-CN", "ar-SA")),
            "",
        )),
        "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
    );
    let b = stylesheet(Some(("<w:sz w:val=\"21\"/>", "")), "");
    let (dom, root) = merge_normals(&a, &b);
    assert_eq!(
        live_normal_attrs(&dom, root, "rPr", "lang"),
        attrs(&[("eastAsia", "en-US")])
    );
}

/// file_103 × file_104: A's Normal is bare, B's carries only a pPr. Word
/// still writes B's run defaults into the live Normal (sz 21, kern 2);
/// the structure gate is either side's pPr or rPr, not rPr alone.
#[test]
fn normal_takes_b_run_defaults_when_only_b_paragraph_props_are_stored() {
    let (a, b) = normal_pair(
        "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
        "",
        "",
        "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
    );
    let mut dom = Dom::new();
    let (out_root, _) = parse(&mut dom, &a);
    let (b_root, _) = parse(&mut dom, &b);
    let settings = WmlComparerSettings::default();
    assert!(merge_normal_style_rpr(
        &mut dom, out_root, b_root, &settings
    ));
    let normal = find_normal_style(&dom, out_root).expect("Normal");
    let rpr = dom.element(normal, &W::r_pr()).expect("live rPr");
    let val = |n: &str| {
        dom.element(rpr, &W::name(n))
            .and_then(|e| dom.attribute(e, &W::val()))
            .map(str::to_string)
    };
    assert_eq!(val("sz").as_deref(), Some("21"));
    assert_eq!(val("kern").as_deref(), Some("2"));
}

/// Live Normal pPr, children in order, pPrChange left out.
fn live_normal_ppr(dom: &Dom, root: NodeId) -> Vec<(String, Vec<(String, String)>)> {
    let normal = find_normal_style(dom, root).expect("Normal");
    let Some(ppr) = dom.element(normal, &W::p_pr()) else {
        return Vec::new();
    };
    dom.elements(ppr, None)
        .into_iter()
        .filter(|&c| dom.name(c).is_some_and(|n| n != W::p_pr_change()))
        .map(|c| {
            let name = dom.name(c).unwrap().local_name().to_string();
            let attrs = dom
                .attributes(c)
                .into_iter()
                .map(|(n, v)| (n.local_name().to_string(), v))
                .collect();
            (name, attrs)
        })
        .collect()
}

/// file_103 × file_104: B keeps indents, justification and line-unit
/// spacing in its docDefaults, which the redline (A's docDefaults) loses.
/// Word writes them into the live Normal.
#[test]
fn normal_takes_b_doc_default_indent_and_justification() {
    let (a, b) = normal_pair(
        "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
        "",
        "<w:spacing w:beforeLines=\"50\" w:afterLines=\"50\"/>\
             <w:ind w:leftChars=\"50\" w:left=\"50\" w:firstLine=\"200\"/><w:jc w:val=\"both\"/>",
        "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
    );
    let mut dom = Dom::new();
    let (out_root, _) = parse(&mut dom, &a);
    let (b_root, _) = parse(&mut dom, &b);
    let settings = WmlComparerSettings::default();
    assert!(merge_normal_style_spacing(
        &mut dom, out_root, b_root, &settings
    ));
    let live = live_normal_ppr(&dom, out_root);
    let get = |n: &str| live.iter().find(|(k, _)| k == n).map(|(_, a)| a.clone());
    let attr =
        |n: &str, a: &str| get(n).and_then(|v| v.into_iter().find(|(k, _)| k == a).map(|(_, v)| v));
    assert_eq!(
        attr("spacing", "beforeLines").as_deref(),
        Some("50"),
        "{live:?}"
    );
    assert_eq!(
        attr("spacing", "afterLines").as_deref(),
        Some("50"),
        "{live:?}"
    );
    assert_eq!(attr("spacing", "after").as_deref(), Some("0"), "{live:?}");
    assert_eq!(attr("ind", "firstLine").as_deref(), Some("200"), "{live:?}");
    assert_eq!(attr("jc", "val").as_deref(), Some("both"), "{live:?}");
    assert!(get("widowControl").is_some(), "{live:?}");
    let order: Vec<&str> = live.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(order, ["widowControl", "spacing", "ind", "jc"]);
}

/// file_104 × file_105, the reverse: A's docDefaults indent and justify,
/// B's do not. Word neutralizes each A value in the live Normal (every
/// indent attribute 0, jc left, line-unit spacing 0).
#[test]
fn normal_neutralizes_a_doc_default_indent_and_justification() {
    let (a, b) = normal_pair(
        "<w:spacing w:beforeLines=\"50\" w:afterLines=\"50\"/>\
             <w:ind w:leftChars=\"50\" w:left=\"50\" w:firstLine=\"200\"/><w:jc w:val=\"both\"/>",
        "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
        "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
        "",
    );
    let mut dom = Dom::new();
    let (out_root, _) = parse(&mut dom, &a);
    let (b_root, _) = parse(&mut dom, &b);
    let settings = WmlComparerSettings::default();
    assert!(merge_normal_style_spacing(
        &mut dom, out_root, b_root, &settings
    ));
    let mut live = live_normal_ppr(&dom, out_root);
    for (_, attrs) in &mut live {
        attrs.sort();
    }
    let s = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        v.sort();
        v
    };
    assert_eq!(
        live,
        vec![
            (
                "spacing".to_string(),
                s(&[
                    ("beforeLines", "0"),
                    ("afterLines", "0"),
                    ("after", "200"),
                    ("line", "276"),
                    ("lineRule", "auto")
                ])
            ),
            (
                "ind".to_string(),
                s(&[("leftChars", "0"), ("left", "0"), ("firstLine", "0")])
            ),
            ("jc".to_string(), s(&[("val", "left")])),
        ]
    );
}

/// A's docDefaults draw paragraph borders, B's do not: Word writes a
/// `nil` border on every edge A draws, even when no spacing changes
/// (sd_1494_table_left_indent × sdpr_titleonly).
#[test]
fn normal_clears_a_doc_default_borders() {
    let (a, b) = normal_pair(
        "<w:pBdr><w:top w:val=\"single\" w:sz=\"4\" w:space=\"1\" w:color=\"auto\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"1\" w:color=\"auto\"/></w:pBdr>",
        "<w:rPr><w:b/></w:rPr>",
        "",
        "",
    );
    let mut dom = Dom::new();
    let (out_root, _) = parse(&mut dom, &a);
    let (b_root, _) = parse(&mut dom, &b);
    let settings = WmlComparerSettings::default();
    assert!(merge_normal_style_spacing(
        &mut dom, out_root, b_root, &settings
    ));
    let normal = find_normal_style(&dom, out_root).expect("Normal");
    let bdr = dom
        .element(normal, &W::p_pr())
        .and_then(|p| dom.element(p, &W::name("pBdr")))
        .expect("live pBdr");
    let edges: Vec<(String, Option<String>, usize)> = dom
        .elements(bdr, None)
        .into_iter()
        .map(|e| {
            (
                dom.name(e).unwrap().local_name().to_string(),
                dom.attribute(e, &W::val()).map(str::to_string),
                dom.attributes_iter(e).len(),
            )
        })
        .collect();
    assert_eq!(
        edges,
        vec![
            ("top".to_string(), Some("nil".to_string()), 1),
            ("bottom".to_string(), Some("nil".to_string()), 1)
        ]
    );
}
