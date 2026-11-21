// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn xml(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{}" xmlns:pt="{}" xmlns:r="{}" xmlns:mc="{}" xmlns:w14="{}"><w:body>{body}</w:body></w:document>"#,
            W::URI, PT::URI, R::URI, MC::URI, W14::URI,
        ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

fn visible(dom: &Dom, node: NodeId) -> String {
    dom.descendants(node, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
        .map(|n| dom.value(n))
        .collect()
}

#[test]
fn carried_revision_matrix_preserves_order_formatting_and_original_attribution() {
    for insertion in [false, true] {
        let marker = if insertion { "PreIns" } else { "PreDelete" };
        let author_marker = if insertion {
            "PreInsAuthor"
        } else {
            "PreDelAuthor"
        };
        let date_marker = if insertion {
            "PreInsDate"
        } else {
            "PreDelDate"
        };
        let wanted = if insertion { "ins" } else { "del" };
        for parent in ["p", "ins", "del", "hyperlink"] {
            for count in [1, 2, 3, 5] {
                for index in 0..count {
                    for metadata in [0, 1, 2, 3] {
                        let mut runs = String::new();
                        for i in 0..count {
                            let stamp = if i == index {
                                format!(
                                    "pt:{marker}=\"1\"{}{}",
                                    if metadata & 1 != 0 {
                                        format!(" pt:{author_marker}=\"Source Author\"")
                                    } else {
                                        String::new()
                                    },
                                    if metadata & 2 != 0 {
                                        format!(" pt:{date_marker}=\"2026-01-01T00:00:00Z\"")
                                    } else {
                                        String::new()
                                    }
                                )
                            } else {
                                String::new()
                            };
                            // The text kind agrees with the enclosing source revision.
                            let leaf = if parent == "del" { "delText" } else { "t" };
                            runs.push_str(&format!("<w:r {stamp}><w:rPr><w:b/><w:color w:val=\"112233\"/></w:rPr><w:{leaf}>segment{i};</w:{leaf}></w:r>"));
                        }
                        let fragment = if parent == "p" {
                            format!("<w:p>{runs}</w:p>")
                        } else if parent == "hyperlink" {
                            format!("<w:p><w:hyperlink r:id=\"rId1\">{runs}</w:hyperlink></w:p>")
                        } else {
                            format!(
                                "<w:p><w:{parent} w:id=\"7\" w:author=\"Comparer\" w:date=\"2025-01-01T00:00:00Z\">{runs}</w:{parent}></w:p>"
                            )
                        };
                        let (mut dom, root) = xml(&fragment);
                        let expected = visible(&dom, root);
                        let mut next_id = 100;
                        if insertion {
                            convert_stamped_preins(
                                &mut dom,
                                root,
                                &WmlComparerSettings::default(),
                                &mut next_id,
                            );
                        } else {
                            convert_stamped_predeletes(
                                &mut dom,
                                root,
                                &WmlComparerSettings::default(),
                                &mut next_id,
                            );
                        }
                        assert_eq!(visible(&dom, root), expected, "{fragment}");
                        let after_runs = dom.descendants(root, Some(&W::r()));
                        assert_eq!(after_runs.len(), count);
                        let stamped = after_runs[index];
                        for name in [marker, author_marker, date_marker] {
                            assert!(dom.attribute(stamped, &PT::name(name)).is_none());
                        }
                        let wrapper = dom.parent(stamped).unwrap();
                        assert!(dom.name_is(wrapper, &W::name(wanted)), "{fragment}");
                        assert!(
                            dom.element(dom.element(stamped, &W::r_pr()).unwrap(), &W::name("b"))
                                .is_some()
                        );
                        // An existing multi-run deletion retains shared attribution rather than
                        // relabeling neighboring deletions as the stamped source author's edits.
                        let can_restamp = insertion || parent != "del" || count == 1;
                        if can_restamp && metadata & 1 != 0 {
                            assert_eq!(dom.attribute(wrapper, &W::author()), Some("Source Author"));
                        }
                        if can_restamp && metadata & 2 != 0 {
                            assert_eq!(
                                dom.attribute(wrapper, &W::date()),
                                Some("2026-01-01T00:00:00Z")
                            );
                        }
                        let leaf = if insertion { W::t() } else { W::del_text() };
                        assert!(dom.element(stamped, &leaf).is_some());
                        let once = dom.serialize_element(root);
                        if insertion {
                            convert_stamped_preins(
                                &mut dom,
                                root,
                                &WmlComparerSettings::default(),
                                &mut next_id,
                            );
                        } else {
                            convert_stamped_predeletes(
                                &mut dom,
                                root,
                                &WmlComparerSettings::default(),
                                &mut next_id,
                            );
                        }
                        assert_eq!(dom.serialize_element(root), once);
                    }
                }
            }
        }
    }
}

#[test]
fn external_link_rewrite_matrix_requires_new_target_and_wholly_inserted_content() {
    for outer_ins in [false, true] {
        for content in ["inserted", "live", "mixed", "deleted", "empty"] {
            for mapping in ["new", "existing", "missing", "quoted"] {
                for anchor in [false, true] {
                    let kids = match content {
                        "inserted" => "<w:ins w:id=\"9\"><w:r><w:t>caption</w:t></w:r></w:ins>",
                        "live" => "<w:r><w:t>caption</w:t></w:r>",
                        "mixed" => {
                            "<w:ins><w:r><w:t>caption</w:t></w:r></w:ins><w:r><w:t>live</w:t></w:r>"
                        }
                        "deleted" => "<w:del><w:r><w:delText>caption</w:delText></w:r></w:del>",
                        _ => "",
                    };
                    let link = format!(
                        "<w:hyperlink r:id=\"rId9\"{}>{kids}</w:hyperlink>",
                        if anchor {
                            " w:anchor=\"LocalBookmark\""
                        } else {
                            ""
                        }
                    );
                    let body = if outer_ins {
                        format!("<w:p><w:ins w:id=\"8\">{link}</w:ins></w:p>")
                    } else {
                        format!("<w:p>{link}</w:p>")
                    };
                    let (mut dom, root) = xml(&body);
                    let expected = visible(&dom, root);
                    let before = dom.serialize_element(root);
                    let url = if mapping == "quoted" {
                        "https://example.test/\"quoted"
                    } else {
                        "https://example.test/report"
                    };
                    let targets = if mapping == "missing" {
                        HashMap::new()
                    } else {
                        HashMap::from([("rId9".to_string(), url.to_string())])
                    };
                    let base = if mapping == "existing" {
                        HashSet::from([url.to_string()])
                    } else {
                        HashSet::new()
                    };
                    rewrite_inserted_external_hyperlinks(&mut dom, root, &targets, &base);
                    let rewritten =
                        !anchor && mapping == "new" && (outer_ins || content == "inserted");
                    assert_eq!(
                        dom.descendants(root, Some(&W::hyperlink())).is_empty(),
                        rewritten,
                        "{body} {mapping}"
                    );
                    assert_eq!(visible(&dom, root), expected);
                    if rewritten {
                        assert_eq!(dom.descendants(root, Some(&W::instr_text())).len(), 1);
                        assert_eq!(
                            dom.value(dom.descendants(root, Some(&W::instr_text()))[0]),
                            format!("HYPERLINK \"{url}\" \\h")
                        );
                        let kinds = dom
                            .descendants(root, Some(&W::fld_char()))
                            .into_iter()
                            .map(|n| {
                                dom.attribute(n, &W::name("fldCharType"))
                                    .unwrap()
                                    .to_string()
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(kinds, ["begin", "separate", "end"]);
                    } else {
                        assert_eq!(dom.serialize_element(root), before);
                    }
                }
            }
        }
    }
}

#[test]
fn hyperlink_hoisting_preserves_empty_wrappers_and_run_order_across_revisions() {
    for revision in ["ins", "del"] {
        for wrapper in ["hyperlink", "fldSimple"] {
            for before in [false, true] {
                for after in [false, true] {
                    for empty in [false, true] {
                        let leaf = if revision == "del" { "delText" } else { "t" };
                        let run = |value| format!("<w:r><w:{leaf}>{value}</w:{leaf}></w:r>");
                        let wrapper_attributes = if wrapper == "hyperlink" {
                            "r:id=\"rId1\""
                        } else {
                            "w:instr=\"DATE\""
                        };
                        let fragment = format!(
                            "<w:p><w:{revision} w:id=\"15\" w:author=\"Author\" w:date=\"2026-01-01T00:00:00Z\">{}<w:{wrapper} {wrapper_attributes}>{}</w:{wrapper}>{}</w:{revision}></w:p>",
                            if before { run("before") } else { String::new() },
                            if empty { String::new() } else { run("middle") },
                            if after { run("after") } else { String::new() }
                        );
                        let (mut dom, root) = xml(&fragment);
                        let expected = visible(&dom, root);
                        hoist_hyperlinks_out_of_revisions(&mut dom, root);
                        assert_eq!(visible(&dom, root), expected);
                        if wrapper == "fldSimple" && empty {
                            // No result to carry the revision: the field
                            // becomes Word's complex form inside it (b15).
                            assert!(dom.descendants(root, Some(&W::name(wrapper))).is_empty());
                            let instr = if revision == "del" {
                                "delInstrText"
                            } else {
                                "instrText"
                            };
                            let code = dom.descendants(root, Some(&W::name(instr)));
                            assert_eq!(code.len(), 1);
                            assert_eq!(dom.value(code[0]), "DATE");
                            let kinds: Vec<_> = dom
                                .descendants(root, Some(&W::fld_char()))
                                .into_iter()
                                .map(|c| {
                                    dom.attribute(c, &W::name("fldCharType"))
                                        .unwrap()
                                        .to_string()
                                })
                                .collect();
                            assert_eq!(kinds, ["begin", "separate", "end"]);
                            let revisions = dom.descendants(root, Some(&W::name(revision)));
                            assert_eq!(revisions.len(), 1, "one revision holds the field");
                            for r in dom.descendants(root, Some(&W::r())) {
                                assert_eq!(dom.parent(r), Some(revisions[0]));
                            }
                            let once = dom.serialize_element(root);
                            hoist_hyperlinks_out_of_revisions(&mut dom, root);
                            assert_eq!(dom.serialize_element(root), once);
                            continue;
                        }
                        let wrapper_node = dom.descendants(root, Some(&W::name(wrapper)))[0];
                        assert!(dom.name_is(dom.parent(wrapper_node).unwrap(), &W::p()));
                        assert_eq!(
                            dom.elements(wrapper_node, Some(&W::name(revision))).len(),
                            usize::from(!empty)
                        );
                        let revisions = dom.descendants(root, Some(&W::name(revision)));
                        let mut ids = HashSet::new();
                        for rev in revisions {
                            assert!(ids.insert(dom.attribute(rev, &W::id()).unwrap().to_string()));
                            assert_eq!(dom.attribute(rev, &W::author()), Some("Author"));
                            assert_eq!(
                                dom.attribute(rev, &W::date()),
                                Some("2026-01-01T00:00:00Z")
                            );
                        }
                        let once = dom.serialize_element(root);
                        hoist_hyperlinks_out_of_revisions(&mut dom, root);
                        assert_eq!(dom.serialize_element(root), once);
                    }
                }
            }
        }
    }
}

#[test]
fn empty_field_in_a_revision_keeps_its_flags_and_padded_code() {
    let (mut dom, root) = xml(
        "<w:p><w:del w:id=\"4\" w:author=\"A\" w:date=\"D\"><w:fldSimple w:instr=\" PAGE \" w:dirty=\"true\" w:fldLock=\"1\"/></w:del></w:p>",
    );
    hoist_hyperlinks_out_of_revisions(&mut dom, root);
    let begin = dom.descendants(root, Some(&W::fld_char()))[0];
    assert_eq!(dom.attribute(begin, &W::name("dirty")), Some("true"));
    assert_eq!(dom.attribute(begin, &W::name("fldLock")), Some("1"));
    let code = dom.descendants(root, Some(&W::del_instr_text()))[0];
    assert_eq!(dom.value(code), " PAGE ");
    assert_eq!(
        dom.attribute(code, &XNamespace::xml().name("space")),
        Some("preserve")
    );
}

#[test]
fn inherited_invalidity_repairs_only_wml_schema_defects() {
    for declared in [false, true] {
        for outside_numbering in [false, true] {
            for foreign in [false, true] {
                let mut dom = Dom::new();
                let val = if declared { " w:val=\"solid\"" } else { "" };
                let highlights = if outside_numbering {
                    "<w:rPr><w:highlight w:val=\"yellow\"/></w:rPr>"
                } else {
                    "<w:lvl w:ilvl=\"0\"><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr></w:lvl>"
                };
                let foreign_node = if foreign {
                    "<ext:metadata unqualified=\"keep\"/>"
                } else {
                    ""
                };
                let doc = dom.parse_xdocument(&format!(
                        "<w:abstractNum xmlns:w=\"{}\" xmlns:ext=\"urn:source-metadata\" w:abstractNumId=\"1\" accidental=\"[object Object]\"><w:rPr><w:shd{val} w:fill=\"ABCDEF\"/><w:color w:themeColor=\"accent1\"/></w:rPr>{highlights}{foreign_node}</w:abstractNum>", W::URI));
                let root = dom.root(doc).unwrap();
                repair_inherited_invalidity(&mut dom, root);
                assert!(
                    dom.attribute(root, &XNamespace::none().name("accidental"))
                        .is_none()
                );
                assert_eq!(dom.attribute(root, &W::abstract_num_id()), Some("1"));
                let shd = dom.descendants(root, Some(&W::shd()))[0];
                assert_eq!(
                    dom.attribute(shd, &W::val()),
                    Some(if declared { "solid" } else { "clear" })
                );
                assert_eq!(dom.attribute(shd, &W::fill()), Some("ABCDEF"));
                let color = dom.descendants(root, Some(&W::name("color")))[0];
                assert_eq!(dom.attribute(color, &W::val()), Some("auto"));
                assert_eq!(
                    dom.attribute(color, &W::name("themeColor")),
                    Some("accent1")
                );
                assert_eq!(
                    dom.descendants(root, Some(&W::name("highlight"))).len(),
                    usize::from(outside_numbering)
                );
                if foreign {
                    let ext = dom.descendants(
                        root,
                        Some(&XNamespace::get("urn:source-metadata").name("metadata")),
                    )[0];
                    assert_eq!(
                        dom.attribute(ext, &XNamespace::none().name("unqualified")),
                        Some("keep")
                    );
                }
                let once = dom.serialize_element(root);
                repair_inherited_invalidity(&mut dom, root);
                assert_eq!(dom.serialize_element(root), once);
            }
        }
    }
}

#[test]
fn compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions() {
    for existing in ["absent", "correct", "conflict"] {
        for placement in ["element", "attribute"] {
            for already_ignorable in [false, true] {
                let mut dom = Dom::new();
                let root = dom.new_element(W::document());
                if existing != "absent" {
                    dom.set_attribute_value(
                        root,
                        &XNamespace::xmlns().name("w14"),
                        Some(if existing == "correct" {
                            W14::URI
                        } else {
                            "urn:foreign"
                        }),
                    );
                }
                if already_ignorable {
                    dom.set_attribute_value(root, &MC::ignorable(), Some("legacy w14"));
                }
                let p = dom.new_element(if placement == "element" {
                    W14::name("paraId")
                } else {
                    W::p()
                });
                dom.add(root, p);
                if placement == "attribute" {
                    dom.set_attribute_value(p, &W14::name("paraId"), Some("00000001"));
                }
                declare_extension_namespaces_ignorable(&mut dom, root);
                assert_eq!(
                    dom.attribute(root, &XNamespace::xmlns().name("w14")),
                    Some(if existing == "conflict" {
                        "urn:foreign"
                    } else {
                        W14::URI
                    })
                );
                let tokens = dom
                    .attribute(root, &MC::ignorable())
                    .unwrap_or("")
                    .split_whitespace()
                    .collect::<Vec<_>>();
                assert_eq!(
                    tokens.contains(&"w14"),
                    existing != "conflict" || already_ignorable
                );
                assert_eq!(
                    tokens.iter().filter(|&&p| p == "w14").count(),
                    usize::from(existing != "conflict" || already_ignorable)
                );
                let once = dom.serialize_element(root);
                declare_extension_namespaces_ignorable(&mut dom, root);
                assert_eq!(dom.serialize_element(root), once);
            }
        }
    }
}
