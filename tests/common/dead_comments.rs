// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Synthetic Word-shaped probes for issue #1141.

use crate::common::docx::{Part, W_NS, docx_with};

pub const FAMILY: [&str; 4] = [
    "word/comments.xml",
    "word/commentsExtended.xml",
    "word/commentsIds.xml",
    "word/commentsExtensible.xml",
];
pub const REV: &str = r#"w:author="Carol Diaz" w:date="2026-10-18T00:00:00Z""#;

pub fn start(id: u32) -> String {
    format!(r#"<w:commentRangeStart w:id="{id}"/>"#)
}
pub fn end(id: u32) -> String {
    format!(r#"<w:commentRangeEnd w:id="{id}"/>"#)
}
pub fn reference(id: u32) -> String {
    format!(r#"<w:r><w:commentReference w:id="{id}"/></w:r>"#)
}
pub fn deleted(text: &str, id: u32) -> String {
    format!(r#"<w:del w:id="{id}" {REV}><w:r><w:delText>{text}</w:delText></w:r></w:del>"#)
}

/// Probe e has reply markers beside the parent's, as Word writes them,
/// and an independent surviving thread;
/// this exercises pruning individual metadata entries, not only whole parts.
pub fn probe(case: char) -> Vec<u8> {
    let s = start(0);
    let e = end(0);
    let r = reference(0);
    let d = deleted("beta", 7);
    let body = match case {
        'a' => format!(
            r#"<w:p><w:ins w:id="6" w:author="Alice"><w:r><w:t xml:space="preserve">alpha </w:t></w:r>{s}{d}{e}{r}</w:ins><w:ins w:id="8" {REV}><w:r><w:t>NEW</w:t></w:r></w:ins><w:ins w:id="9" w:author="Alice"><w:r><w:t xml:space="preserve"> gamma</w:t></w:r></w:ins></w:p>"#
        ),
        'b' => format!(
            r#"<w:p>{s}<w:del w:id="7" {REV}><w:r><w:delText>beta</w:delText></w:r>{r}</w:del>{e}</w:p>"#
        ),
        'c' => format!(r#"<w:p>{s}{d}<w:r><w:t>x</w:t></w:r>{e}{r}</w:p>"#),
        'd' => format!(
            r#"<w:p>{s}<w:ins w:id="7" {REV}><w:r><w:t>beta</w:t></w:r></w:ins>{e}{r}</w:p>"#
        ),
        'e' => format!(
            r#"<w:p>{s}{}{d}{e}{}{r}{}</w:p><w:p>{}<w:r><w:t>kept</w:t></w:r>{}{}</w:p>"#,
            start(1),
            end(1),
            reference(1),
            start(2),
            end(2),
            reference(2)
        ),
        'f' => format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="10" {REV}/></w:rPr></w:pPr>{s}{d}</w:p><w:p>{}{e}{r}</w:p>"#,
            deleted("gamma", 11)
        ),
        _ => panic!("unknown probe {case}"),
    };
    package(&body, case == 'e')
}

pub fn package(body: &str, threaded: bool) -> Vec<u8> {
    let w14 = "http://schemas.microsoft.com/office/word/2010/wordml";
    let w15 = "http://schemas.microsoft.com/office/word/2012/wordml";
    let cid = "http://schemas.microsoft.com/office/word/2016/wordml/cid";
    let cex = "http://schemas.microsoft.com/office/word/2018/wordml/cex";
    let rows: Vec<_> = if threaded {
        vec![
            (0, "Alice", "Existing range"),
            (1, "Bob", "Reply to range"),
            (2, "Carol Diaz", "Kept range"),
        ]
    } else {
        vec![(0, "Alice", "Existing range")]
    };
    let comments: String = rows.iter().map(|(id, author, text)| format!(r#"<w:comment w:id="{id}" w:author="{author}"><w:p w14:paraId="{:08X}"><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#, id + 1)).collect();
    let comments =
        format!(r#"<w:comments xmlns:w="{W_NS}" xmlns:w14="{w14}">{comments}</w:comments>"#);
    let mut parts = vec![Part {
        name: FAMILY[0],
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        xml: &comments,
    }];
    let ext = format!(
        r#"<w15:commentsEx xmlns:w15="{w15}">{}</w15:commentsEx>"#,
        rows.iter()
            .map(|(id, _, _)| format!(
                r#"<w15:commentEx w15:paraId="{:08X}" w15:done="0"{}/>"#,
                id + 1,
                if *id == 1 {
                    r#" w15:paraIdParent="00000001""#
                } else {
                    ""
                }
            ))
            .collect::<String>()
    );
    let ids = format!(
        r#"<w16cid:commentsIds xmlns:w16cid="{cid}">{}</w16cid:commentsIds>"#,
        rows.iter()
            .map(|(id, _, _)| format!(
                r#"<w16cid:commentId w16cid:paraId="{:08X}" w16cid:durableId="{:08X}"/>"#,
                id + 1,
                id + 0x1000_0001
            ))
            .collect::<String>()
    );
    let cex = format!(r#"<w16cex:commentsExtensible xmlns:w16cex="{cex}">{}</w16cex:commentsExtensible>"#, rows.iter().map(|(id, _, _)| format!(r#"<w16cex:commentExtensible w16cex:durableId="{:08X}" w16cex:dateUtc="2026-10-18T00:00:00Z"/>"#, id + 0x1000_0001)).collect::<String>());
    if threaded {
        parts.extend([
            Part { name: FAMILY[1], content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml", rel_type: "http://schemas.microsoft.com/office/2011/relationships/commentsExtended", xml: &ext },
            Part { name: FAMILY[2], content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml", rel_type: "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds", xml: &ids },
            Part { name: FAMILY[3], content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml", rel_type: "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible", xml: &cex },
        ]);
    }
    docx_with(body, &parts)
}

/// Exact comment identities, anchors and metadata keys after resolution.
pub fn assert_state(bytes: &[u8], modern: bool, expected: &[(u32, &str, Option<u32>)]) {
    use crate::common::docx::part_string;
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;
    crate::common::validity::assert_word_valid_package(bytes);
    let comments = jubarte::comments::list_comments(bytes).unwrap();
    let actual: Vec<_> = comments
        .iter()
        .map(|c| (c.id, c.text.as_str(), c.parent))
        .collect();
    assert_eq!(actual, expected);
    let doc = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&doc);
    let root = dom.root(d).unwrap();
    let mut anchored_ids: Vec<_> = expected.iter().map(|(id, _, _)| id.to_string()).collect();
    anchored_ids.sort();
    for tag in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
        let mut ids: Vec<_> = dom
            .descendants(root, Some(&W::name(tag)))
            .iter()
            .map(|&n| dom.attribute(n, &W::id()).unwrap().to_string())
            .collect();
        ids.sort();
        assert_eq!(ids, anchored_ids, "{tag}: {doc}");
    }
    if expected.is_empty() {
        for part in FAMILY {
            assert!(part_string(bytes, part).is_none(), "{part} remains");
        }
        for part in ["word/_rels/document.xml.rels", "[Content_Types].xml"] {
            assert!(
                !part_string(bytes, part).unwrap().contains("comments"),
                "{part}"
            );
        }
        return;
    }
    let xml = part_string(bytes, FAMILY[0]).unwrap();
    assert_eq!(xml.matches("<w:comment ").count(), expected.len(), "{xml}");
    for part in &FAMILY[1..] {
        let xml = part_string(bytes, part);
        if !modern {
            assert!(xml.is_none(), "unexpected {part}");
            continue;
        }
        let xml = xml.unwrap();
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let rows = dom.elements(root, None);
        assert_eq!(rows.len(), expected.len(), "{part}: {xml}");
        let mut keys: Vec<String> = rows
            .iter()
            .map(|&row| {
                dom.attributes(row)
                    .into_iter()
                    .find(|(name, _)| {
                        name.local_name()
                            == if part.ends_with("Extensible.xml") {
                                "durableId"
                            } else {
                                "paraId"
                            }
                    })
                    .unwrap()
                    .1
            })
            .collect();
        keys.sort();
        let mut wanted: Vec<String> = expected
            .iter()
            .map(|(_, text, _)| {
                let original: u32 = match *text {
                    "Existing range" => 1,
                    "Reply to range" => 2,
                    "Kept range" => 3,
                    _ => panic!("unknown comment"),
                };
                if part.ends_with("Extensible.xml") {
                    format!("{:08X}", 0x1000_0000 + original)
                } else {
                    format!("{original:08X}")
                }
            })
            .collect();
        wanted.sort();
        assert_eq!(keys, wanted, "{part}: {xml}");
    }
}
