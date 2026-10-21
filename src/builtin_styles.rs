// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word's built-in style names: the 376 `w:lsdException` names Word 16 writes
//! into every stylesheet it saves (395 of its redlines in the bench corpus
//! carry the same set).

/// Word's built-in style names, lowercased and sorted.
const BUILT_IN: [&str; 376] = [
    "annotation reference",
    "annotation subject",
    "annotation text",
    "balloon text",
    "bibliography",
    "block text",
    "body text",
    "body text 2",
    "body text 3",
    "body text first indent",
    "body text first indent 2",
    "body text indent",
    "body text indent 2",
    "body text indent 3",
    "book title",
    "caption",
    "closing",
    "colorful grid",
    "colorful grid accent 1",
    "colorful grid accent 2",
    "colorful grid accent 3",
    "colorful grid accent 4",
    "colorful grid accent 5",
    "colorful grid accent 6",
    "colorful list",
    "colorful list accent 1",
    "colorful list accent 2",
    "colorful list accent 3",
    "colorful list accent 4",
    "colorful list accent 5",
    "colorful list accent 6",
    "colorful shading",
    "colorful shading accent 1",
    "colorful shading accent 2",
    "colorful shading accent 3",
    "colorful shading accent 4",
    "colorful shading accent 5",
    "colorful shading accent 6",
    "dark list",
    "dark list accent 1",
    "dark list accent 2",
    "dark list accent 3",
    "dark list accent 4",
    "dark list accent 5",
    "dark list accent 6",
    "date",
    "default paragraph font",
    "document map",
    "e-mail signature",
    "emphasis",
    "endnote reference",
    "endnote text",
    "envelope address",
    "envelope return",
    "followedhyperlink",
    "footer",
    "footnote reference",
    "footnote text",
    "grid table 1 light",
    "grid table 1 light accent 1",
    "grid table 1 light accent 2",
    "grid table 1 light accent 3",
    "grid table 1 light accent 4",
    "grid table 1 light accent 5",
    "grid table 1 light accent 6",
    "grid table 2",
    "grid table 2 accent 1",
    "grid table 2 accent 2",
    "grid table 2 accent 3",
    "grid table 2 accent 4",
    "grid table 2 accent 5",
    "grid table 2 accent 6",
    "grid table 3",
    "grid table 3 accent 1",
    "grid table 3 accent 2",
    "grid table 3 accent 3",
    "grid table 3 accent 4",
    "grid table 3 accent 5",
    "grid table 3 accent 6",
    "grid table 4",
    "grid table 4 accent 1",
    "grid table 4 accent 2",
    "grid table 4 accent 3",
    "grid table 4 accent 4",
    "grid table 4 accent 5",
    "grid table 4 accent 6",
    "grid table 5 dark",
    "grid table 5 dark accent 1",
    "grid table 5 dark accent 2",
    "grid table 5 dark accent 3",
    "grid table 5 dark accent 4",
    "grid table 5 dark accent 5",
    "grid table 5 dark accent 6",
    "grid table 6 colorful",
    "grid table 6 colorful accent 1",
    "grid table 6 colorful accent 2",
    "grid table 6 colorful accent 3",
    "grid table 6 colorful accent 4",
    "grid table 6 colorful accent 5",
    "grid table 6 colorful accent 6",
    "grid table 7 colorful",
    "grid table 7 colorful accent 1",
    "grid table 7 colorful accent 2",
    "grid table 7 colorful accent 3",
    "grid table 7 colorful accent 4",
    "grid table 7 colorful accent 5",
    "grid table 7 colorful accent 6",
    "grid table light",
    "hashtag",
    "header",
    "heading 1",
    "heading 2",
    "heading 3",
    "heading 4",
    "heading 5",
    "heading 6",
    "heading 7",
    "heading 8",
    "heading 9",
    "html acronym",
    "html address",
    "html bottom of form",
    "html cite",
    "html code",
    "html definition",
    "html keyboard",
    "html preformatted",
    "html sample",
    "html top of form",
    "html typewriter",
    "html variable",
    "hyperlink",
    "index 1",
    "index 2",
    "index 3",
    "index 4",
    "index 5",
    "index 6",
    "index 7",
    "index 8",
    "index 9",
    "index heading",
    "intense emphasis",
    "intense quote",
    "intense reference",
    "light grid",
    "light grid accent 1",
    "light grid accent 2",
    "light grid accent 3",
    "light grid accent 4",
    "light grid accent 5",
    "light grid accent 6",
    "light list",
    "light list accent 1",
    "light list accent 2",
    "light list accent 3",
    "light list accent 4",
    "light list accent 5",
    "light list accent 6",
    "light shading",
    "light shading accent 1",
    "light shading accent 2",
    "light shading accent 3",
    "light shading accent 4",
    "light shading accent 5",
    "light shading accent 6",
    "line number",
    "list",
    "list 2",
    "list 3",
    "list 4",
    "list 5",
    "list bullet",
    "list bullet 2",
    "list bullet 3",
    "list bullet 4",
    "list bullet 5",
    "list continue",
    "list continue 2",
    "list continue 3",
    "list continue 4",
    "list continue 5",
    "list number",
    "list number 2",
    "list number 3",
    "list number 4",
    "list number 5",
    "list paragraph",
    "list table 1 light",
    "list table 1 light accent 1",
    "list table 1 light accent 2",
    "list table 1 light accent 3",
    "list table 1 light accent 4",
    "list table 1 light accent 5",
    "list table 1 light accent 6",
    "list table 2",
    "list table 2 accent 1",
    "list table 2 accent 2",
    "list table 2 accent 3",
    "list table 2 accent 4",
    "list table 2 accent 5",
    "list table 2 accent 6",
    "list table 3",
    "list table 3 accent 1",
    "list table 3 accent 2",
    "list table 3 accent 3",
    "list table 3 accent 4",
    "list table 3 accent 5",
    "list table 3 accent 6",
    "list table 4",
    "list table 4 accent 1",
    "list table 4 accent 2",
    "list table 4 accent 3",
    "list table 4 accent 4",
    "list table 4 accent 5",
    "list table 4 accent 6",
    "list table 5 dark",
    "list table 5 dark accent 1",
    "list table 5 dark accent 2",
    "list table 5 dark accent 3",
    "list table 5 dark accent 4",
    "list table 5 dark accent 5",
    "list table 5 dark accent 6",
    "list table 6 colorful",
    "list table 6 colorful accent 1",
    "list table 6 colorful accent 2",
    "list table 6 colorful accent 3",
    "list table 6 colorful accent 4",
    "list table 6 colorful accent 5",
    "list table 6 colorful accent 6",
    "list table 7 colorful",
    "list table 7 colorful accent 1",
    "list table 7 colorful accent 2",
    "list table 7 colorful accent 3",
    "list table 7 colorful accent 4",
    "list table 7 colorful accent 5",
    "list table 7 colorful accent 6",
    "macro",
    "medium grid 1",
    "medium grid 1 accent 1",
    "medium grid 1 accent 2",
    "medium grid 1 accent 3",
    "medium grid 1 accent 4",
    "medium grid 1 accent 5",
    "medium grid 1 accent 6",
    "medium grid 2",
    "medium grid 2 accent 1",
    "medium grid 2 accent 2",
    "medium grid 2 accent 3",
    "medium grid 2 accent 4",
    "medium grid 2 accent 5",
    "medium grid 2 accent 6",
    "medium grid 3",
    "medium grid 3 accent 1",
    "medium grid 3 accent 2",
    "medium grid 3 accent 3",
    "medium grid 3 accent 4",
    "medium grid 3 accent 5",
    "medium grid 3 accent 6",
    "medium list 1",
    "medium list 1 accent 1",
    "medium list 1 accent 2",
    "medium list 1 accent 3",
    "medium list 1 accent 4",
    "medium list 1 accent 5",
    "medium list 1 accent 6",
    "medium list 2",
    "medium list 2 accent 1",
    "medium list 2 accent 2",
    "medium list 2 accent 3",
    "medium list 2 accent 4",
    "medium list 2 accent 5",
    "medium list 2 accent 6",
    "medium shading 1",
    "medium shading 1 accent 1",
    "medium shading 1 accent 2",
    "medium shading 1 accent 3",
    "medium shading 1 accent 4",
    "medium shading 1 accent 5",
    "medium shading 1 accent 6",
    "medium shading 2",
    "medium shading 2 accent 1",
    "medium shading 2 accent 2",
    "medium shading 2 accent 3",
    "medium shading 2 accent 4",
    "medium shading 2 accent 5",
    "medium shading 2 accent 6",
    "mention",
    "message header",
    "no list",
    "no spacing",
    "normal",
    "normal (web)",
    "normal indent",
    "normal table",
    "note heading",
    "outline list 1",
    "outline list 2",
    "outline list 3",
    "page number",
    "placeholder text",
    "plain table 1",
    "plain table 2",
    "plain table 3",
    "plain table 4",
    "plain table 5",
    "plain text",
    "quote",
    "revision",
    "salutation",
    "signature",
    "smart hyperlink",
    "smart link",
    "strong",
    "subtitle",
    "subtle emphasis",
    "subtle reference",
    "table 3d effects 1",
    "table 3d effects 2",
    "table 3d effects 3",
    "table classic 1",
    "table classic 2",
    "table classic 3",
    "table classic 4",
    "table colorful 1",
    "table colorful 2",
    "table colorful 3",
    "table columns 1",
    "table columns 2",
    "table columns 3",
    "table columns 4",
    "table columns 5",
    "table contemporary",
    "table elegant",
    "table grid",
    "table grid 1",
    "table grid 2",
    "table grid 3",
    "table grid 4",
    "table grid 5",
    "table grid 6",
    "table grid 7",
    "table grid 8",
    "table list 1",
    "table list 2",
    "table list 3",
    "table list 4",
    "table list 5",
    "table list 6",
    "table list 7",
    "table list 8",
    "table of authorities",
    "table of figures",
    "table professional",
    "table simple 1",
    "table simple 2",
    "table simple 3",
    "table subtle 1",
    "table subtle 2",
    "table theme",
    "table web 1",
    "table web 2",
    "table web 3",
    "title",
    "toa heading",
    "toc 1",
    "toc 2",
    "toc 3",
    "toc 4",
    "toc 5",
    "toc 6",
    "toc 7",
    "toc 8",
    "toc 9",
    "toc heading",
    "unresolved mention",
];

/// Whether `name` names one of Word's built-in styles, in any case. Word pairs
/// built-in styles across documents whatever the case (`normal` with
/// `Normal`, `Footnote Reference` with `footnote reference`: 54 of 54 pairs
/// in its redlines) and custom styles only by their exact name (`Subsection`
/// beside `subsection`, `Definition` beside `definition`: 6 of 6 kept apart).
pub(crate) fn is_built_in(name: &str) -> bool {
    BUILT_IN
        .binary_search_by(|probe| {
            probe
                .bytes()
                .cmp(name.bytes().map(|b| b.to_ascii_lowercase()))
        })
        .is_ok()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn built_in_names_match_in_any_case() {
        assert!(BUILT_IN.array_windows().all(|[a, b]| a < b));
        for name in [
            "Normal",
            "normal",
            "heading 1",
            "Footnote Reference",
            "Table Grid",
        ] {
            assert!(is_built_in(name), "{name}");
        }
        for name in ["Subsection", "Definition", "TableText", "Clause Char"] {
            assert!(!is_built_in(name), "{name}");
        }
    }

    /// The allocation-free lookup answers exactly as the lowercase-then-search
    /// definition it replaced did.
    fn is_built_in_by_lowercasing(name: &str) -> bool {
        BUILT_IN
            .binary_search(&name.to_ascii_lowercase().as_str())
            .is_ok()
    }

    fn title_case(name: &str) -> String {
        let mut start = true;
        name.chars()
            .map(|c| {
                let out = if start {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                };
                start = c == ' ';
                out
            })
            .collect()
    }

    fn alternating_case(name: &str) -> String {
        name.chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect()
    }

    #[test]
    fn is_built_in_matches_the_lowercasing_definition() {
        for entry in BUILT_IN {
            for name in [
                entry.to_string(),
                entry.to_ascii_uppercase(),
                title_case(entry),
                alternating_case(entry),
            ] {
                assert!(is_built_in(&name), "{name}");
                assert_eq!(is_built_in(&name), is_built_in_by_lowercasing(&name));
            }
            let prefix = &entry[..entry.len() - 1];
            let longer = format!("{entry}x");
            let spaced = format!("{entry} ");
            let non_ascii = format!("{entry}\u{e9}");
            let folded = entry.replacen(|c: char| c.is_ascii_lowercase(), "\u{212a}", 1);
            for name in [prefix, &longer, &spaced, &non_ascii, &folded] {
                assert_eq!(
                    is_built_in(name),
                    is_built_in_by_lowercasing(name),
                    "{name}"
                );
            }
        }
        for name in ["", " ", "\u{e9}", "\u{212a}eep", "NORMAL\u{0}"] {
            assert!(!is_built_in(name), "{name}");
            assert_eq!(is_built_in(name), is_built_in_by_lowercasing(name));
        }
    }
}
