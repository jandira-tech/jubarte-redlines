//! Targeted unit tests for under-covered modules: comparison_log, unid,
//! formatchg helpers, the revision-processor reject/move transforms, and opc.

use jubarte::namespaces::{PT, W};
use jubarte::xmllinq::{Dom, NodeId};

fn body_from(dom: &mut Dom, inner: &str) -> NodeId {
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body>{}</w:body></w:document>",
        W::URI,
        inner
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.element(root, &W::body()).unwrap()
}

fn all_text(dom: &Dom, root: NodeId) -> String {
    let mut s = String::new();
    for d in dom.descendants(root, None) {
        match dom.name(d) {
            Some(n) if n == W::t() || n == W::name("delText") => s.push_str(&dom.value(d)),
            _ => {}
        }
    }
    s
}

// ── comparison_log ──────────────────────────────────────────────────────────
mod comparison_log {
    use jubarte::comparison_log::{ComparisonLog, ComparisonLogCode};

    #[test]
    fn records_entries_by_severity_and_counts_errors() {
        let mut log = ComparisonLog::new();
        log.info("starting");
        log.warning("odd content");
        log.error("bad 1");
        log.error("bad 2");
        assert_eq!(log.entries.len(), 4);
        assert_eq!(log.error_count(), 2);
        assert_eq!(log.entries[0].code, ComparisonLogCode::Info);
        assert_eq!(log.entries[1].code, ComparisonLogCode::Warning);
        assert_eq!(log.entries[2].code, ComparisonLogCode::Error);
        assert_eq!(log.entries[2].message, "bad 1");
    }

    #[test]
    fn default_is_empty() {
        let log = ComparisonLog::default();
        assert!(log.entries.is_empty());
        assert_eq!(log.error_count(), 0);
    }
}

// ── unid ────────────────────────────────────────────────────────────────────
mod unid {
    use super::*;
    use jubarte::unid::{assign_to_all_elements, assign_to_self_and_descendants, generate_unid};

    #[test]
    fn generate_unid_is_32_hex_unique_and_monotonic() {
        let a = generate_unid();
        let b = generate_unid();
        assert_eq!(a.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b, "ids are unique");
        // monotonic counter → b parses greater than a
        assert!(u128::from_str_radix(&b, 16).unwrap() > u128::from_str_radix(&a, 16).unwrap());
    }

    #[test]
    fn assign_to_self_and_descendants_stamps_root_and_children_without_overwrite() {
        let mut d = Dom::new();
        let body = body_from(&mut d, "<w:p><w:r><w:t>hi</w:t></w:r></w:p>");
        let p = d.element(body, &W::p()).unwrap();
        // pre-stamp the paragraph; it must be preserved
        d.set_attribute_value(p, &PT::unid(), Some("KEEP"));

        assign_to_self_and_descendants(&mut d, body);

        assert!(d.attribute(body, &PT::unid()).is_some(), "root stamped");
        assert_eq!(
            d.attribute(p, &PT::unid()),
            Some("KEEP"),
            "existing unid preserved"
        );
        for desc in d.descendants(body, None) {
            assert!(
                d.attribute(desc, &PT::unid()).is_some(),
                "every descendant stamped"
            );
        }
    }

    #[test]
    fn assign_to_all_elements_stamps_footnote_root() {
        let mut d = Dom::new();
        let fnx = d.new_element(W::footnote());
        let p = d.new_element(W::p());
        d.add(fnx, p);
        assign_to_all_elements(&mut d, fnx);
        assert!(
            d.attribute(fnx, &PT::unid()).is_some(),
            "footnote root gets a unid"
        );
        assert!(d.attribute(p, &PT::unid()).is_some());
    }

    #[test]
    fn assign_to_all_elements_does_not_stamp_non_footnote_root() {
        let mut d = Dom::new();
        let body = body_from(&mut d, "<w:p/>");
        assign_to_all_elements(&mut d, body);
        // body root is not a footnote/endnote → only descendants are stamped
        assert!(
            d.attribute(body, &PT::unid()).is_none(),
            "plain root not stamped by assign_to_all_elements"
        );
        let p = d.element(body, &W::p()).unwrap();
        assert!(d.attribute(p, &PT::unid()).is_some());
    }
}

// ── formatchg helpers ───────────────────────────────────────────────────────
mod formatchg {
    use super::*;
    use jubarte::comparer::atoms::ComparisonUnitAtom;
    use jubarte::comparer::formatchg::{
        friendly_property_name, get_changed_property_names, get_run_properties_from_atom,
        normalize_run_properties,
    };

    #[test]
    fn friendly_property_name_covers_all_known_mappings() {
        let cases = [
            ("b", "bold"),
            ("bCs", "boldComplex"),
            ("i", "italic"),
            ("iCs", "italicComplex"),
            ("u", "underline"),
            ("strike", "strikethrough"),
            ("dstrike", "doubleStrikethrough"),
            ("sz", "fontSize"),
            ("szCs", "fontSizeComplex"),
            ("rFonts", "font"),
            ("color", "color"),
            ("highlight", "highlight"),
            ("shd", "shading"),
            ("vertAlign", "verticalAlign"),
            ("caps", "allCaps"),
            ("smallCaps", "smallCaps"),
            ("outline", "outline"),
            ("shadow", "shadow"),
            ("emboss", "emboss"),
            ("imprint", "imprint"),
            ("vanish", "hidden"),
            ("spacing", "characterSpacing"),
            ("w", "characterWidth"),
            ("kern", "kerning"),
            ("position", "position"),
        ];
        for (local, expected) in cases {
            assert_eq!(
                friendly_property_name(local),
                expected,
                "mapping for {local}"
            );
        }
        // unknown local passes through unchanged
        assert_eq!(friendly_property_name("xyzzy"), "xyzzy");
    }

    #[test]
    fn get_changed_property_names_detects_value_change() {
        let mut d = Dom::new();
        let mk_color = |d: &mut Dom, val: &str| {
            let rpr = d.new_element(W::r_pr());
            let color = d.new_element(W::name("color"));
            d.set_attribute_value(color, &W::val(), Some(val));
            d.add(rpr, color);
            rpr
        };
        let old = mk_color(&mut d, "FF0000");
        let new = mk_color(&mut d, "00FF00");
        let changed = get_changed_property_names(&mut d, Some(old), Some(new));
        assert_eq!(
            changed,
            vec!["color"],
            "same property, different value → changed"
        );
        // identical value → no change
        let a = mk_color(&mut d, "123456");
        let b = mk_color(&mut d, "123456");
        assert!(get_changed_property_names(&mut d, Some(a), Some(b)).is_empty());
    }

    #[test]
    fn get_run_properties_from_atom_finds_rpr_via_run_ancestor() {
        let mut d = Dom::new();
        let p = d.new_element(W::p());
        let r = d.new_element(W::r());
        let rpr = d.new_element(W::r_pr());
        d.add(r, rpr);
        let t = d.new_element(W::t());
        d.add(r, t);
        let atom = ComparisonUnitAtom::new(t, vec![p, r, t], "h".into());
        assert_eq!(get_run_properties_from_atom(&d, &atom), Some(rpr));
    }

    #[test]
    fn get_run_properties_from_atom_none_without_run_ancestor() {
        let mut d = Dom::new();
        let t = d.new_element(W::t());
        let atom = ComparisonUnitAtom::new(t, vec![], "h".into());
        assert_eq!(get_run_properties_from_atom(&d, &atom), None);
        // run ancestor but no rPr child → None
        let r = d.new_element(W::r());
        let atom2 = ComparisonUnitAtom::new(t, vec![r, t], "h".into());
        assert_eq!(get_run_properties_from_atom(&d, &atom2), None);
    }

    #[test]
    fn normalize_drops_rprchange_and_sorts_children() {
        let mut d = Dom::new();
        let rpr = d.new_element(W::r_pr());
        // insertion order i, b, rPrChange; rsid attr on i should be stripped
        let i = d.new_element(W::name("i"));
        d.set_attribute_value(i, &W::name("rsidR"), Some("X"));
        let b = d.new_element(W::name("b"));
        let chg = d.new_element(W::name("rPrChange"));
        d.add(rpr, i);
        d.add(rpr, b);
        d.add(rpr, chg);

        let norm = normalize_run_properties(&mut d, Some(rpr));
        let kids = d.elements(norm, None);
        assert_eq!(kids.len(), 2, "rPrChange dropped");
        assert_eq!(d.name(kids[0]).unwrap(), W::name("b"), "sorted: b before i");
        assert_eq!(d.name(kids[1]).unwrap(), W::name("i"));
        // rsid attribute filtered out of the normalized child
        assert!(
            d.attribute(kids[1], &W::name("rsidR")).is_none(),
            "rsid stripped"
        );
    }

    #[test]
    fn normalize_of_none_is_empty_rpr() {
        let mut d = Dom::new();
        let norm = normalize_run_properties(&mut d, None);
        assert_eq!(d.name(norm).unwrap(), W::r_pr());
        assert!(d.elements(norm, None).is_empty());
    }
}

// ── revision_processor: reject + move transforms ────────────────────────────
mod revision_processor {
    use super::*;
    use jubarte::revision_processor::{
        accept_move_from_move_to_transform, reject_revisions_document,
    };

    #[test]
    fn reject_yields_original_projection() {
        // ins NEW + del OLD → original (pre-revision) text is "OLD" (insert undone,
        // deletion undone → the deleted text is restored, the inserted text removed).
        let mut d = Dom::new();
        let body = body_from(
            &mut d,
            "<w:p>\
               <w:ins w:id=\"1\"><w:r><w:t>NEW</w:t></w:r></w:ins>\
               <w:del w:id=\"2\"><w:r><w:delText>OLD</w:delText></w:r></w:del>\
             </w:p>",
        );
        let res = reject_revisions_document(&mut d, body);
        let text = all_text(&d, res);
        assert!(text.contains("OLD"), "deleted text restored, got {text:?}");
        assert!(!text.contains("NEW"), "inserted text removed, got {text:?}");
    }

    #[test]
    fn accept_move_unwraps_moveto_and_drops_movefrom() {
        let mut d = Dom::new();
        let p = d.new_element(W::p());
        // moveTo wrapping run "X"
        let mt = d.new_element(W::name("moveTo"));
        let r1 = d.new_element(W::r());
        let t1 = d.new_element(W::t());
        d.add_text(t1, "X");
        d.add(r1, t1);
        d.add(mt, r1);
        d.add(p, mt);
        // moveFrom wrapping run "Y"
        let mf = d.new_element(W::name("moveFrom"));
        let r2 = d.new_element(W::r());
        let t2 = d.new_element(W::t());
        d.add_text(t2, "Y");
        d.add(r2, t2);
        d.add(mf, r2);
        d.add(p, mf);

        let out = accept_move_from_move_to_transform(&mut d, p);
        assert_eq!(out.len(), 1);
        let new_p = out[0];
        let text = all_text(&d, new_p);
        assert_eq!(text, "X", "moveTo unwrapped, moveFrom dropped");
        // moveTo/moveFrom wrappers gone
        assert!(d.descendants(new_p, Some(&W::name("moveTo"))).is_empty());
        assert!(d.descendants(new_p, Some(&W::name("moveFrom"))).is_empty());
    }
}

// ── opc::PartFs ─────────────────────────────────────────────────────────────
mod opc {
    use jubarte::opc::PartFs;

    fn fixture() -> Vec<u8> {
        std::fs::read("tests/fixtures/redline/original.docx").unwrap()
    }

    #[test]
    fn open_lists_parts_and_finds_main_document() {
        let pkg = PartFs::open(&fixture()).unwrap();
        let parts = pkg.parts();
        assert!(
            parts.iter().any(|p| p.ends_with("document.xml")),
            "has a document part"
        );
        let main = pkg.main_document_part().expect("main document part");
        assert!(pkg.part_bytes(&main).is_some());
        assert!(pkg.part_string(&main).unwrap().contains("<w:document"));
    }

    #[test]
    fn content_type_and_set_part_roundtrip() {
        let mut pkg = PartFs::open(&fixture()).unwrap();
        let main = pkg.main_document_part().unwrap();
        assert!(
            pkg.content_type_for(&main)
                .unwrap()
                .contains("wordprocessingml")
        );
        // overwrite a part and read it back, then re-zip
        pkg.set_part("word/custom.xml", b"<x/>".to_vec());
        assert_eq!(pkg.part_bytes("word/custom.xml").unwrap(), b"<x/>");
        let zipped = pkg.to_zip().unwrap();
        let reopened = PartFs::open(&zipped).unwrap();
        assert_eq!(reopened.part_bytes("word/custom.xml").unwrap(), b"<x/>");
    }

    #[test]
    fn resolve_rel_target_normalizes_relative_paths() {
        let pkg = PartFs::open(&fixture()).unwrap();
        let resolved = pkg.resolve_rel_target("word/document.xml", "media/image1.png");
        assert_eq!(resolved, "word/media/image1.png");
    }

    #[test]
    fn add_document_relationship_returns_id() {
        let mut pkg = PartFs::open(&fixture()).unwrap();
        let main = pkg.main_document_part().unwrap();
        let rid = pkg.add_document_relationship(&main, "http://example/rel/custom", "custom.xml");
        assert!(rid.starts_with("rId"), "minted a rId, got {rid}");
    }

    /// Removing a relationship type from a part that has no `.rels` must be a
    /// pure no-op — `get_or_create_part_rels` would invent an empty rels part
    /// and leave it in the package (PR #81 review).
    #[test]
    fn remove_relationships_by_type_is_noop_when_part_has_no_rels() {
        let mut pkg = PartFs::open(&fixture()).unwrap();
        // A part that exists but has no relationships of its own.
        pkg.set_part("word/orphan_no_rels.xml", b"<x/>".to_vec());
        assert!(
            pkg.read_rels_for("word/orphan_no_rels.xml").is_none(),
            "fixture has no rels for the orphan part"
        );
        pkg.remove_relationships_by_type(
            "word/orphan_no_rels.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        );
        assert!(
            pkg.read_rels_for("word/orphan_no_rels.xml").is_none(),
            "remove must not create an empty .rels part"
        );
    }
}
