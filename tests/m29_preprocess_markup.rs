//! M-C — package-level PreProcessMarkup (`WmlComparer.PreProcessMarkup` :434)
//! wired into the compare path (`CompareInternal` :152–:155).
//!
//! C.1: ChangeFootnoteEndnoteReferencesToUniqueRange (:1627) — doc A is
//! renumbered from `starting_id + 1000`, doc B from `starting_id + 2000`, so
//! the two inputs' footnote/endnote id spaces are disjoint entering compare.

use jubarte::document_comparer::{
    compare_documents_internal, compare_documents_with_options, pre_process_markup,
};
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;

const DATE: &str = "2020-01-01T00:00:00Z";

/// Build a .docx (on the f4 fixture package base) whose body is `paras`
/// (text, optional footnoteReference id) and whose footnotes part holds the
/// two separator notes plus `notes` (id, text) definitions, wired via rels +
/// content types.
fn note_doc(paras: &[(&str, Option<&str>)], notes: &[(&str, &str)]) -> Vec<u8> {
    let original = std::fs::read("tests/fixtures/f4/original.docx").unwrap();
    let mut pkg = PartFs::open(&original).unwrap();

    let mut body = String::new();
    for (text, rid) in paras {
        body.push_str(&format!("<w:p><w:r><w:t>{text}</w:t></w:r>"));
        if let Some(rid) = rid {
            body.push_str(&format!("<w:r><w:footnoteReference w:id=\"{rid}\"/></w:r>"));
        }
        body.push_str("</w:p>");
    }
    let doc = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{body}</w:body></w:document>",
        w = W::URI
    );
    pkg.set_part("word/document.xml", doc.into_bytes());

    let mut fx = format!(
        "<w:footnotes xmlns:w=\"{w}\">\
         <w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>\
         <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>",
        w = W::URI
    );
    for (id, text) in notes {
        fx.push_str(&format!(
            "<w:footnote w:id=\"{id}\"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:footnote>"
        ));
    }
    fx.push_str("</w:footnotes>");
    pkg.set_part("word/footnotes.xml", fx.into_bytes());
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        "footnotes.xml",
    );
    pkg.add_content_type_override(
        "/word/footnotes.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
    );
    pkg.to_zip().unwrap()
}

/// All `w:footnoteReference` ids in a part.
fn ref_ids(xml: &str) -> Vec<i64> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::name("footnoteReference")))
        .into_iter()
        .filter_map(|r| dom.attribute(r, &W::id()).and_then(|v| v.parse().ok()))
        .collect()
}

/// All `w:footnote` definition ids in a notes part.
fn def_ids(xml: &str) -> Vec<i64> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    dom.elements(root, Some(&W::footnote()))
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id()).and_then(|v| v.parse().ok()))
        .collect()
}

/// C.1 unit — `pre_process_markup` renumbers every reference (document order)
/// and its definition from the starting id; separators are untouched; the
/// rewritten parts are reported.
#[test]
fn c1_pre_process_markup_renumbers_to_unique_range() {
    let doc = note_doc(
        &[("hello", Some("2")), ("world", Some("5"))],
        &[("2", "alpha"), ("5", "beta")],
    );
    let mut pkg = PartFs::open(&doc).unwrap();
    let changed = pre_process_markup(&mut pkg, 1001);
    assert!(
        changed.iter().any(|p| p == "word/document.xml")
            && changed.iter().any(|p| p == "word/footnotes.xml"),
        "both parts rewritten, got {changed:?}"
    );

    let dx = pkg.part_string("word/document.xml").unwrap();
    assert_eq!(ref_ids(&dx), vec![1001, 1002]);

    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let defs = def_ids(&fx);
    assert!(defs.contains(&-1) && defs.contains(&0), "separators kept");
    assert!(
        defs.contains(&1001) && defs.contains(&1002),
        "defs renumbered"
    );
    assert!(!defs.contains(&2) && !defs.contains(&5), "old ids gone");
    // note content still with its (renumbered) definition
    assert!(fx.contains("alpha") && fx.contains("beta"));
}

/// C.2 — `AddFootnotesEndnotesParts` (:1604) is UNCONDITIONAL: every
/// preprocessed document lacking a footnotes/endnotes part gains an EMPTY
/// namespace-decorated one, wired via rels + content types. (No separator
/// notes — C# adds those only when Rectify rebuilds output parts.) Verified
/// against the C# golden: the note-free `redline` pair's output carries
/// exactly these empty parts.
#[test]
fn c2_missing_notes_parts_created_empty() {
    let original = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let mut pkg = PartFs::open(&original).unwrap();
    let changed = pre_process_markup(&mut pkg, 1001);
    assert!(
        changed.iter().any(|p| p == "word/footnotes.xml")
            && changed.iter().any(|p| p == "word/endnotes.xml"),
        "created parts reported, got {changed:?}"
    );
    for (part, root_name) in [
        ("word/footnotes.xml", W::name("footnotes")),
        ("word/endnotes.xml", W::name("endnotes")),
    ] {
        let xml = pkg
            .part_string(part)
            .unwrap_or_else(|| panic!("{part} missing"));
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        assert_eq!(dom.name(root), Some(root_name), "{part} root");
        assert!(
            dom.elements(root, None).is_empty(),
            "{part} must be empty (no separators pre-Rectify)"
        );
        assert!(
            xml.contains("mc:Ignorable=\"w14 wp14\""),
            "{part} namespace-decorated like C# FreshNamespaceAttributes"
        );
        // content type registered
        let ct = pkg.content_type_for(part).unwrap_or_default();
        assert!(ct.contains("wordprocessingml"), "{part} content type: {ct}");
    }
    // rels wired off the main document part
    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    for suffix in ["/footnotes", "/endnotes"] {
        assert!(
            rels.items.iter().any(|r| r.rel_type.ends_with(suffix)),
            "main rels gained {suffix}"
        );
    }
}

/// C.2 — documents that already have both notes parts get NO new parts and
/// keep their existing content (f4 carries separator-only parts).
#[test]
fn c2_existing_notes_parts_untouched() {
    let original = std::fs::read("tests/fixtures/f4/original.docx").unwrap();
    let mut pkg = PartFs::open(&original).unwrap();
    let parts_before = pkg.parts().len();
    pre_process_markup(&mut pkg, 1001);
    assert_eq!(pkg.parts().len(), parts_before, "no parts added");
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let defs = def_ids(&fx);
    assert!(
        defs.contains(&-1) && defs.contains(&0),
        "existing separators kept, got {defs:?}"
    );
}

/// C.2 e2e (the plan assert, adapted to the faithful fn) — A without a
/// footnotes part vs B with a real footnote: the output package has a
/// footnotes part, and B's inserted reference resolves to its (renumbered)
/// definition in it — no dangling refs.
#[test]
fn c2_inserted_note_resolves_when_a_had_no_part() {
    // A: note-free package with a plain body.
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let mut pa = PartFs::open(&base).unwrap();
    let doc_a = format!(
        "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>hello</w:t></w:r></w:p></w:body></w:document>",
        w = W::URI
    );
    pa.set_part("word/document.xml", doc_a.into_bytes());
    let a = pa.to_zip().unwrap();

    // B: same body plus an inserted paragraph carrying footnote id=2.
    let b = note_doc(
        &[("hello", None), ("brand new", Some("2"))],
        &[("2", "beta")],
    );

    let out = compare_documents_with_options(&a, &b, "Test Author", DATE).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    let dx = pkg.part_string("word/document.xml").unwrap();
    let refs = ref_ids(&dx);
    // post-B.4: Rectify renumbers output references 1..n in document order
    assert_eq!(refs, vec![1], "one inserted reference renumbered to 1");
    let fx = pkg
        .part_string("word/footnotes.xml")
        .expect("output package has a footnotes part");
    let defs = def_ids(&fx);
    assert!(
        defs.contains(&refs[0]),
        "inserted ref {} resolves (defs {defs:?})",
        refs[0]
    );
    assert!(fx.contains("beta"), "inserted note content carried");
}

/// C.1 — an orphaned reference (no matching definition) panics: C# throws
/// DocxodusException when no ComparisonLog is wired (:1676).
#[test]
fn c1_orphan_reference_without_log_panics() {
    let doc = note_doc(&[("hello", Some("77"))], &[("2", "alpha")]);
    let mut pkg = PartFs::open(&doc).unwrap();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pre_process_markup(&mut pkg, 1001)
    }));
    assert!(r.is_err(), "orphan reference must panic");
}

/// C.1 e2e — both inputs use footnote id=2. The disjoint pre-compare ranges
/// (A→1001+, B→2001+; unit-tested above) are what make the pairing sound;
/// the OUTPUT contract (post-B.4) is: references renumbered 1..n, the shared
/// note untouched, the inserted note's def present, nothing dangling. The
/// colliding raw ids must never leak through as a bogus pair.
#[test]
fn c1_compare_inputs_get_disjoint_id_ranges() {
    let a = note_doc(&[("hello", Some("2"))], &[("2", "alpha")]);
    let b = note_doc(
        &[("hello", Some("2")), ("brand new", Some("3"))],
        &[("2", "alpha"), ("3", "beta")],
    );
    let out = compare_documents_with_options(&a, &b, "Test Author", DATE).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    let dx = pkg.part_string("word/document.xml").unwrap();
    let refs = ref_ids(&dx);
    assert_eq!(refs, vec![1, 2], "refs renumbered 1..n, got {refs:?}");

    // consistency: every referenced id has a definition in the output part
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let defs = def_ids(&fx);
    for id in &refs {
        assert!(defs.contains(id), "ref {id} dangling (defs {defs:?})");
    }
    // the shared note (alpha) is unmarked; the inserted note (beta) is w:ins
    assert!(fx.contains("alpha") && fx.contains("beta"));
}

/// C.3 — `FillInEmptyFootnotesEndnotes` (:513) wired into PreProcessMarkup:
/// a childless `w:footnote` gains the stock FootnoteText/footnoteRef
/// paragraph before diffing (and is still renumbered by C.1).
#[test]
fn c3_empty_footnote_filled_pre_compare() {
    // note_doc gives the def a paragraph; strip it down to a childless def.
    let doc = note_doc(&[("hello", Some("2"))], &[]);
    let mut pkg = PartFs::open(&doc).unwrap();
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let fx = fx.replace(
        "</w:footnotes>",
        "<w:footnote w:id=\"2\"></w:footnote></w:footnotes>",
    );
    pkg.set_part("word/footnotes.xml", fx.into_bytes());
    let doc = pkg.to_zip().unwrap();

    let mut pkg = PartFs::open(&doc).unwrap();
    pre_process_markup(&mut pkg, 1001);

    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&fx);
    let root = dom.root(d).unwrap();
    let def = dom
        .elements(root, Some(&W::footnote()))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some("1001"))
        .expect("renumbered def present");
    let p = dom.element(def, &W::p()).expect("stock paragraph added");
    let px = dom.serialize_element(p);
    assert!(px.contains("FootnoteText"), "pStyle FootnoteText: {px}");
    assert!(px.contains("FootnoteReference"), "rStyle: {px}");
    assert!(px.contains("footnoteRef"), "footnoteRef run: {px}");
}

/// C.4 — `DetachExternalData` (:497): `c:externalData` is stripped from every
/// chart part related to the main document; the chart's rels are untouched.
#[test]
fn c4_chart_external_data_detached() {
    use jubarte::namespaces::C;

    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let mut pkg = PartFs::open(&base).unwrap();
    let chart_xml = format!(
        "<c:chartSpace xmlns:c=\"{c}\" xmlns:r=\"{r}\">\
         <c:chart><c:plotArea/></c:chart>\
         <c:externalData r:id=\"rId1\"><c:autoUpdate val=\"0\"/></c:externalData>\
         </c:chartSpace>",
        c = C::URI,
        r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    );
    pkg.set_part("word/charts/chart1.xml", chart_xml.into_bytes());
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart",
        "charts/chart1.xml",
    );
    pkg.add_content_type_override(
        "/word/charts/chart1.xml",
        "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
    );
    // the chart's own rels (external workbook link) must survive
    let chart_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/oleObject\" Target=\"embeddings/book1.xlsx\" TargetMode=\"External\"/>\
        </Relationships>";
    pkg.set_part(
        "word/charts/_rels/chart1.xml.rels",
        chart_rels.as_bytes().to_vec(),
    );
    let doc = pkg.to_zip().unwrap();

    let mut pkg = PartFs::open(&doc).unwrap();
    let changed = pre_process_markup(&mut pkg, 1001);
    assert!(
        changed.iter().any(|p| p == "word/charts/chart1.xml"),
        "chart part rewritten, got {changed:?}"
    );

    let cx = pkg.part_string("word/charts/chart1.xml").unwrap();
    assert!(!cx.contains("externalData"), "externalData stripped: {cx}");
    assert!(cx.contains("plotArea"), "chart content kept: {cx}");
    let rels = pkg
        .read_rels_for("word/charts/chart1.xml")
        .expect("chart rels part survives");
    assert!(
        rels.items.iter().any(|r| r.target.contains("book1.xlsx")),
        "chart rels untouched"
    );
}

/// C.5 — `AddUnidsToMarkupInContentParts` (:600): after preprocessing, main
/// AND notes-part elements carry `pt:Unid` (the plan assert: note definitions
/// carry pt:Unid before diffing), and each part root declares pt14 as
/// mc:Ignorable.
#[test]
fn c5_unids_assigned_across_content_parts() {
    use jubarte::namespaces::{MC, PT};

    let doc = note_doc(&[("hello", Some("2"))], &[("2", "alpha")]);
    let mut pkg = PartFs::open(&doc).unwrap();
    pre_process_markup(&mut pkg, 1001);

    // main part: block elements carry pt:Unid; root declares pt14 ignorable
    let dx = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&dx);
    let root = dom.root(d).unwrap();
    assert!(
        dom.attribute(root, &MC::name("Ignorable"))
            .unwrap_or("")
            .contains("pt14"),
        "main root mc:Ignorable declares pt14"
    );
    let body = dom.element(root, &W::body()).unwrap();
    let p = dom.element(body, &W::p()).unwrap();
    assert!(
        dom.attribute(p, &PT::unid()).is_some(),
        "main paragraph carries pt:Unid"
    );

    // notes part: the definition carries pt:Unid too
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let fd = dom.parse_xdocument(&fx);
    let froot = dom.root(fd).unwrap();
    assert!(
        dom.attribute(froot, &MC::name("Ignorable"))
            .unwrap_or("")
            .contains("pt14"),
        "notes root mc:Ignorable declares pt14"
    );
    let def = dom
        .elements(froot, Some(&W::footnote()))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some("1001"))
        .expect("renumbered def");
    assert!(
        dom.attribute(def, &PT::unid()).is_some(),
        "note definition carries pt:Unid before diffing"
    );
}

/// B.4 — reference-driven notes diffing survives Word's note renumbering.
/// A has notes 1(alpha), 2(beta ORIGINAL); B inserts gamma BEFORE them so
/// Word renumbers: 1(gamma NEW), 2(alpha), 3(beta EDITED). A by-id model
/// diffs 1↔1 (alpha vs gamma — bogus) and 2↔2 (beta vs alpha — bogus).
/// Reference-driven pairing must instead: leave alpha untouched, diff beta's
/// definition (del ORIGINAL / ins EDITED), emit gamma all-inserted, and
/// renumber output refs+defs 1..n in document order.
#[test]
fn b4_reference_driven_pairing_survives_renumbering() {
    let a = note_doc(
        &[("first para", Some("1")), ("second para", Some("2"))],
        &[("1", "alpha note"), ("2", "beta ORIGINAL")],
    );
    let b = note_doc(
        &[
            ("inserted para", Some("1")),
            ("first para", Some("2")),
            ("second para", Some("3")),
        ],
        &[
            ("1", "gamma note"),
            ("2", "alpha note"),
            ("3", "beta EDITED"),
        ],
    );
    let out = compare_documents_with_options(&a, &b, "Test Author", DATE).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    // output body refs renumbered 1..n in doc order (gamma, alpha, beta)
    let dx = pkg.part_string("word/document.xml").unwrap();
    assert_eq!(ref_ids(&dx), vec![1, 2, 3], "refs renumbered 1..n");

    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&fx);
    let root = dom.root(d).unwrap();
    let def = |dom: &Dom, id: &str| {
        dom.elements(root, Some(&W::footnote()))
            .into_iter()
            .find(|&n| dom.attribute(n, &W::id()) == Some(id))
            .unwrap_or_else(|| panic!("def {id} missing in {fx}"))
    };

    // def 1 = gamma (inserted note): all-inserted content
    let d1 = def(&dom, "1");
    let d1x = dom.serialize_element(d1);
    assert!(d1x.contains("gamma"), "def 1 is gamma: {d1x}");
    assert!(
        !dom.descendants(d1, Some(&W::ins())).is_empty(),
        "inserted note marked w:ins: {d1x}"
    );

    // def 2 = alpha (equal): NO revision markup
    let d2 = def(&dom, "2");
    let d2x = dom.serialize_element(d2);
    assert!(d2x.contains("alpha"), "def 2 is alpha: {d2x}");
    assert!(
        dom.descendants(d2, Some(&W::ins())).is_empty()
            && dom.descendants(d2, Some(&W::del())).is_empty(),
        "equal note untouched: {d2x}"
    );

    // def 3 = beta (edited): the RIGHT pair was diffed — del ORIGINAL / ins EDITED
    let d3 = def(&dom, "3");
    let d3x = dom.serialize_element(d3);
    assert!(
        dom.descendants(d3, Some(&W::del()))
            .iter()
            .any(|&e| dom.value(e).contains("ORIGINAL")),
        "old beta text deleted: {d3x}"
    );
    assert!(
        dom.descendants(d3, Some(&W::ins()))
            .iter()
            .any(|&e| dom.value(e).contains("EDITED")),
        "new beta text inserted: {d3x}"
    );
}

/// C.1 — the `pre_process_original` knob (C# `preProcessMarkupInOriginal`,
/// false only for Consolidate, whose original is already preprocessed).
/// Post-B.4 the knob's effect is invisible in the final package by design —
/// Rectify renumbers everything 1..n either way — so the contract asserted
/// here is: the knob-off path still yields a fully consistent output (no
/// dangling refs, notes diffed by reference).
#[test]
fn c1_pre_process_original_knob_skips_doc_a() {
    let a = note_doc(&[("hello", Some("2"))], &[("2", "alpha")]);
    let b = note_doc(
        &[("hello", Some("2")), ("brand new", Some("3"))],
        &[("2", "alpha"), ("3", "beta")],
    );
    let out = compare_documents_internal(&a, &b, "Test Author", DATE, false).unwrap();
    let pkg = PartFs::open(&out).unwrap();

    let dx = pkg.part_string("word/document.xml").unwrap();
    let refs = ref_ids(&dx);
    assert_eq!(refs, vec![1, 2], "refs renumbered 1..n, got {refs:?}");
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let defs = def_ids(&fx);
    for id in &refs {
        assert!(defs.contains(id), "ref {id} dangling (defs {defs:?})");
    }
    assert!(fx.contains("beta"), "inserted note carried");
}

/// B.4 gate — WC034/WC035/WC036 footnote+endnote fixtures (read in place from
/// the C# test corpus; skipped when Docxodus/ is absent). Text-level asserts
/// now; exact revision COUNTS arrive with GetRevisions (D.6).
#[test]
fn b4_wc_footnote_fixture_sweep() {
    let wc = std::path::Path::new("tests/corpus/Docxodus/TestFiles/WC");
    if !wc.is_dir() {
        eprintln!("skipping: Docxodus WC corpus not present");
        return;
    }
    let read = |n: &str| std::fs::read(wc.join(n)).unwrap();

    // consistency helper: every body note ref resolves in its notes part
    let assert_consistent = |out: &[u8], label: &str| {
        let pkg = PartFs::open(out).unwrap();
        let dx = pkg.part_string("word/document.xml").unwrap();
        for (ref_name, part, def_name) in [
            ("footnoteReference", "word/footnotes.xml", W::footnote()),
            ("endnoteReference", "word/endnotes.xml", W::endnote()),
        ] {
            let mut dom = Dom::new();
            let d = dom.parse_xdocument(&dx);
            let root = dom.root(d).unwrap();
            let refs: Vec<String> = dom
                .descendants(root, Some(&W::name(ref_name)))
                .into_iter()
                .filter_map(|r| dom.attribute(r, &W::id()).map(str::to_string))
                .collect();
            if refs.is_empty() {
                continue;
            }
            let px = pkg
                .part_string(part)
                .unwrap_or_else(|| panic!("{label}: {part} missing with live refs"));
            let pd = dom.parse_xdocument(&px);
            let proot = dom.root(pd).unwrap();
            let defs: Vec<String> = dom
                .elements(proot, Some(&def_name))
                .into_iter()
                .filter_map(|n| dom.attribute(n, &W::id()).map(str::to_string))
                .collect();
            for r in &refs {
                assert!(
                    defs.contains(r),
                    "{label}: ref {r} dangling (defs {defs:?})"
                );
            }
        }
        pkg
    };

    // WC034-Footnotes Before→After1: "new " inserted inside the footnote
    let out = compare_documents_with_options(
        &read("WC034-Footnotes-Before.docx"),
        &read("WC034-Footnotes-After1.docx"),
        "Test Author",
        DATE,
    )
    .unwrap();
    let pkg = assert_consistent(&out, "WC034-fn-after1");
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&fx);
    let root = dom.root(d).unwrap();
    assert!(
        dom.descendants(root, Some(&W::ins()))
            .iter()
            .any(|&i| dom.value(i).contains("new")),
        "WC034-fn-after1: inserted word marked in footnote: {fx}"
    );
    assert!(fx.contains("footnote."), "original note text retained");

    // WC035-Footnote Before→After: a footnote is ADDED — its def is inserted
    let out = compare_documents_with_options(
        &read("WC035-Footnote-Before.docx"),
        &read("WC035-Footnote-After.docx"),
        "Test Author",
        DATE,
    )
    .unwrap();
    let pkg = assert_consistent(&out, "WC035-fn");
    let fx = pkg.part_string("word/footnotes.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&fx);
    let root = dom.root(d).unwrap();
    assert!(
        dom.descendants(root, Some(&W::ins()))
            .iter()
            .any(|&i| dom.value(i).contains("This is a test")),
        "WC035: added note content marked inserted: {fx}"
    );

    // WC034-Endnotes Before→After1: same shape through the ENDNOTE path
    let out = compare_documents_with_options(
        &read("WC034-Endnotes-Before.docx"),
        &read("WC034-Endnotes-After1.docx"),
        "Test Author",
        DATE,
    )
    .unwrap();
    let pkg = assert_consistent(&out, "WC034-en-after1");
    let ex = pkg.part_string("word/endnotes.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&ex);
    let root = dom.root(d).unwrap();
    assert!(
        dom.descendants(root, Some(&W::ins()))
            .iter()
            .any(|&i| dom.value(i).contains("interesting")),
        "WC034-en-after1: inserted word marked in endnote: {ex}"
    );

    // WC036 tables + remaining WC034/35 projections: must run panic-free and
    // stay internally consistent.
    for (a, b) in [
        ("WC034-Footnotes-Before.docx", "WC034-Footnotes-After2.docx"),
        ("WC034-Footnotes-Before.docx", "WC034-Footnotes-After3.docx"),
        ("WC034-Footnotes-After3.docx", "WC034-Footnotes-Before.docx"),
        ("WC034-Endnotes-Before.docx", "WC034-Endnotes-After2.docx"),
        ("WC034-Endnotes-Before.docx", "WC034-Endnotes-After3.docx"),
        ("WC034-Endnotes-After3.docx", "WC034-Endnotes-Before.docx"),
        ("WC035-Endnote-Before.docx", "WC035-Endnote-After.docx"),
        (
            "WC036-Footnote-With-Table-Before.docx",
            "WC036-Footnote-With-Table-After.docx",
        ),
        (
            "WC036-Endnote-With-Table-Before.docx",
            "WC036-Endnote-With-Table-After.docx",
        ),
    ] {
        let out = compare_documents_with_options(&read(a), &read(b), "Test Author", DATE)
            .unwrap_or_else(|e| panic!("{a} vs {b}: {e}"));
        assert_consistent(&out, &format!("{a}->{b}"));
    }
}

/// Regression (found via Jubarte's identical "class A" bug): an INSERTED
/// external hyperlink's relationship carried into the output by
/// reconcile_dangling_relationships must keep TargetMode="External" — the
/// default mode is Internal, and an absolute http(s) target is ILLEGAL for
/// Internal, so strict packaging layers (System.IO.Packaging, Word itself)
/// reject the package. LibreOffice is lenient, which masks it in PDF runs.
#[test]
fn external_hyperlink_rel_keeps_target_mode() {
    let base = std::fs::read("tests/fixtures/redline/original.docx").unwrap();

    let mut pa = PartFs::open(&base).unwrap();
    pa.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p><w:r><w:t>plain text</w:t></w:r></w:p></w:body></w:document>",
            w = W::URI
        )
        .into_bytes(),
    );
    let a = pa.to_zip().unwrap();

    // doc B inserts a paragraph with an EXTERNAL hyperlink
    let mut pb = PartFs::open(&base).unwrap();
    let rid = pb.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
        "https://www.example.com/page",
    );
    // mark it external in the SOURCE package (as real docs have it)
    pb.set_rel_target_mode_external("word/document.xml", &rid);
    pb.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>\
             <w:p><w:r><w:t>plain text</w:t></w:r></w:p>\
             <w:p><w:hyperlink r:id=\"{rid}\"><w:r><w:t>click here now</w:t></w:r></w:hyperlink></w:p>\
             </w:body></w:document>",
            w = W::URI,
            r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        )
        .into_bytes(),
    );
    let b = pb.to_zip().unwrap();

    let out = compare_documents_with_options(&a, &b, "Test Author", DATE).unwrap();
    let pkg = PartFs::open(&out).unwrap();
    let rels = pkg.read_rels_for("word/document.xml").unwrap();
    let hyper: Vec<_> = rels
        .items
        .iter()
        .filter(|r| r.rel_type.ends_with("/hyperlink") && r.target.contains("example.com"))
        .collect();
    assert!(!hyper.is_empty(), "hyperlink rel reconciled into output");
    for h in hyper {
        assert_eq!(
            h.target_mode.as_deref(),
            Some("External"),
            "TargetMode=External preserved on {h:?}"
        );
    }
}
