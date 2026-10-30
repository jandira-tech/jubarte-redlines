// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A paragraph's inserted words move past the deleted tables after it
//! (#1851). Word's own Compare leaves them in a paragraph whose mark is
//! deleted, right before a table every row of which is deleted: Accept All
//! joins them to the paragraph after the table. Word 16.115 crashed
//! opening 6 of the 16 such release-sample redlines (0.14.0), its table
//! cache corrupt at the deleted row's end, and Word's own redlines of those
//! pairs crash too. Once the words start the paragraph they join, Word
//! opened them, and its Accept All and Reject All gave back the revision
//! and the original (fix23 probe33, rt30). The pass settles the finished
//! body by the validator's own rule
//! ([`crate::validate::inserted_text_before_deleted_tables`]), so the two
//! cannot disagree.

use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

/// Move every inserted word Accept All joins across a deleted table to the
/// start of the paragraph it joins.
pub fn move_inserted_text_past_deleted_tables(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    // Each step empties one paragraph of its inserted words, the last
    // first: an earlier one's chain may pass it once emptied, and its
    // words then go ahead of the later one's.
    for _ in 0..dom.elements(body, Some(&W::p())).len() {
        let Some(&(paragraph, joined)) =
            crate::validate::inserted_text_before_deleted_tables(dom, body).last()
        else {
            break;
        };
        let words = dom.elements(paragraph, Some(&W::ins()));
        let anchor = dom.element(joined, &W::p_pr());
        for word in words.into_iter().rev() {
            dom.remove(word);
            match anchor {
                Some(ppr) => dom.add_after_self(ppr, word),
                None => dom.add_first(joined, word),
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    const BY: &str = r#"w:author="Ana" w:date="2026-10-25T00:00:00Z""#;

    /// A deleted mark.
    fn mark() -> String {
        format!(r#"<w:pPr><w:rPr><w:del w:id="1" {BY}/></w:rPr></w:pPr>"#)
    }

    fn inserted(text: &str) -> String {
        format!(r#"<w:ins w:id="3" {BY}><w:r><w:t>{text}</w:t></w:r></w:ins>"#)
    }

    fn deleted(text: &str) -> String {
        format!(r#"<w:del w:id="2" {BY}><w:r><w:delText>{text}</w:delText></w:r></w:del>"#)
    }

    /// A table whose rows are deleted (`true`) or live.
    fn table(rows: &[bool]) -> String {
        let rows: String = rows
            .iter()
            .map(|&gone| {
                let (row, mark, text) = if gone {
                    (
                        format!(r#"<w:trPr><w:del w:id="4" {BY}/></w:trPr>"#),
                        mark(),
                        deleted("Fee"),
                    )
                } else {
                    (
                        String::new(),
                        String::new(),
                        "<w:r><w:t>Fee</w:t></w:r>".into(),
                    )
                };
                format!("<w:tr>{row}<w:tc><w:p>{mark}{text}</w:p></w:tc></w:tr>")
            })
            .collect();
        format!("<w:tbl><w:tblPr/><w:tblGrid><w:gridCol/></w:tblGrid>{rows}</w:tbl>")
    }

    /// `body` serialized, after the pass when `pass`.
    fn serialized(body: &str, pass: bool) -> String {
        let xml = format!(
            r#"<w:document xmlns:w="{}"><w:body>{body}<w:sectPr/></w:body></w:document>"#,
            W::URI
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        if pass {
            move_inserted_text_past_deleted_tables(&mut dom, root);
        }
        dom.serialize_element(root)
    }

    /// The words leave their paragraph, which keeps its deleted text and
    /// mark, and start the paragraph after the deleted tables and deleted
    /// paragraphs, after its properties.
    #[test]
    fn inserted_words_start_the_paragraph_they_join() {
        let gone = format!("<w:p>{}{}</w:p>", mark(), deleted("Gone."));
        let tables = table(&[true, true]);
        let joined = |words: &str| {
            format!(
                r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr>{words}<w:r><w:t>Kept.</w:t></w:r></w:p>"#
            )
        };
        let (old, new) = (deleted("Old."), inserted("New."));
        let body = format!(
            "<w:p>{}{old}{new}</w:p>{tables}{gone}{tables}{gone}{}",
            mark(),
            joined("")
        );
        let expected = format!(
            "<w:p>{}{old}</w:p>{tables}{gone}{tables}{gone}{}",
            mark(),
            joined(&new)
        );
        assert_eq!(serialized(&body, true), serialized(&expected, false));
    }

    /// Two paragraphs' words, each before deleted tables, start the last
    /// paragraph in document order; an empty paragraph takes words too.
    #[test]
    fn the_words_of_two_paragraphs_keep_their_order() {
        let (one, two, old, gone) = (
            inserted("One."),
            inserted("Two."),
            deleted("Old."),
            table(&[true]),
        );
        let body = format!(
            "<w:p>{0}{one}</w:p>{gone}<w:p>{0}{old}{two}</w:p>{gone}<w:p/>",
            mark()
        );
        let expected = format!(
            "<w:p>{0}</w:p>{gone}<w:p>{0}{old}</w:p>{gone}<w:p>{one}{two}</w:p>",
            mark()
        );
        assert_eq!(serialized(&body, true), serialized(&expected, false));
    }

    /// A bookmark or a comment range between the blocks takes no part:
    /// the words still move, and the marker stays where it was.
    #[test]
    fn markers_between_the_blocks_do_not_count() {
        let (new, gone) = (inserted("New."), table(&[true]));
        let markers = r#"<w:bookmarkStart w:id="9" w:name="Fees"/><w:commentRangeStart w:id="0"/>"#;
        let body = format!(
            "<w:p>{}{new}</w:p>{markers}{gone}<w:bookmarkEnd w:id=\"9\"/><w:p/>",
            mark()
        );
        let expected = format!(
            "<w:p>{}</w:p>{markers}{gone}<w:bookmarkEnd w:id=\"9\"/><w:p>{new}</w:p>",
            mark()
        );
        assert_eq!(serialized(&body, true), serialized(&expected, false));
    }

    /// Nothing moves where no deleted table separates the words from the
    /// paragraph they join, or where the paragraph keeps other words: a
    /// live row, a live paragraph, a live mark, a live word, a deleted
    /// section break, no table, no paragraph after the table, a deleted
    /// paragraph keeping words after it. Nor where a deleted paragraph
    /// sits between the words and the table: Word placement kept, the
    /// shape crashed Word no more often than any redline (#1851).
    #[test]
    fn words_no_deleted_table_separates_stay() {
        let p = format!(
            "<w:p>{}{}{}</w:p>",
            mark(),
            deleted("Old."),
            inserted("New.")
        );
        let kept = "<w:p><w:r><w:t>Kept.</w:t></w:r></w:p>".to_string();
        let live_word = format!(
            "<w:p>{}<w:r><w:t>Stays.</w:t></w:r>{}</w:p>",
            mark(),
            inserted("New.")
        );
        let section = format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {BY}/></w:rPr><w:sectPr/></w:pPr>{}</w:p>"#,
            inserted("New.")
        );
        for (case, body) in [
            ("a live row", format!("{p}{}{kept}", table(&[true, false]))),
            (
                "a live paragraph",
                format!("{p}{kept}{}{kept}", table(&[true])),
            ),
            (
                "a live mark",
                format!("<w:p>{}</w:p>{}{kept}", inserted("New."), table(&[true])),
            ),
            (
                "a live word",
                format!("{live_word}{}{kept}", table(&[true])),
            ),
            (
                "a section break",
                format!("{section}{}{kept}", table(&[true])),
            ),
            (
                "no table",
                format!("{p}<w:p>{}{}</w:p>{kept}", mark(), deleted("Gone.")),
            ),
            ("no paragraph after", format!("{p}{}", table(&[true]))),
            (
                "a deleted paragraph before the table",
                format!(
                    "{p}<w:p>{}{}</w:p>{}{kept}",
                    mark(),
                    deleted("Gone."),
                    table(&[true])
                ),
            ),
            (
                "a deleted paragraph keeping words",
                format!("{p}{}{live_word}{kept}", table(&[true])),
            ),
        ] {
            assert_eq!(serialized(&body, true), serialized(&body, false), "{case}");
        }
    }

    /// A document with no body is left alone.
    #[test]
    fn no_body_no_move() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:hdr xmlns:w="{}"/>"#, W::URI));
        let root = dom.root(doc).unwrap();
        move_inserted_text_past_deleted_tables(&mut dom, root);
        assert_eq!(dom.elements(root, None), []);
    }
}
