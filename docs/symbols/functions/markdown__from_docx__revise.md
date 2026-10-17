# Functions and methods in `src/markdown/from_docx/revise.rs`

10 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `resolve`  (L20-24, pub(crate))
`pub(crate) fn resolve(element: &Element, accept: bool) -> Element`

- **Does:** The part with every tracked change accepted (`accept`) or rejected.
- **Calls (crate):** `resolve_in` (markdown/from_docx/revise.rs:59) L22
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `removed`  (L27-33)
`fn removed(local: &str, accept: bool) -> bool`

- **Does:** Content that goes away: deletions on accept, insertions on reject.
- **Macros:** `matches!`
- **Called by:** `marked_removed` (markdown/from_docx/revise.rs:55); `resolve_in` (markdown/from_docx/revise.rs:59)

#### `kept`  (L36-42)
`fn kept(local: &str, accept: bool) -> bool`

- **Does:** Content that stays, without its wrapper.
- **Macros:** `matches!`
- **Called by:** `run_flags` (bin/jubarte.rs:1804) (?); `render_best` (comparer/cross_para.rs:689) (?); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47636) (?); `resolve_in` (markdown/from_docx/revise.rs:59)

#### `is_marker`  (L46-51)
`fn is_marker(local: &str) -> bool`

- **Does:** A revision marker inside a properties element (`trPr`, `tcPr`, a paragraph mark's `rPr`), or a record of a formatting change.
- **Calls (std/external/unresolved):** `.ends_with`
- **Macros:** `matches!`
- **Called by:** `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1484) (?); `Layout::line_face_metrics` (convert/mod.rs:23832) (?); `resolve_in` (markdown/from_docx/revise.rs:59)

#### `marked_removed`  (L55-57)
`fn marked_removed(properties: Option<&Element>, accept: bool) -> bool`

- **Does:** Whether a properties element marks its row, cell or paragraph break as going away.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L56; `removed` (markdown/from_docx/revise.rs:27) L56; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L56
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:59)

#### `resolve_in`  (L59-98)
`fn resolve_in(element: &mut Element, accept: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:277) L60; `restore_earlier` (markdown/from_docx/revise.rs:114) L62; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L70; `is_marker` (markdown/from_docx/revise.rs:46) L71; `marked_removed` (markdown/from_docx/revise.rs:55) L75,76,87; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L75,76; `removed` (markdown/from_docx/revise.rs:27) L77; `range_ends` (markdown/from_docx/revise.rs:101) L83; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L87; `resolve_in` (markdown/from_docx/revise.rs:59) L90; `kept` (markdown/from_docx/revise.rs:36) L91; `join_paragraphs` (markdown/from_docx/revise.rs:136) L97
- **Calls (std/external/unresolved):** `.as_str`, `.ends_with`, `.extend`, `.len`, `.push`, `.to_string`, `Node::Element`, `String::new`, `Vec::with_capacity`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `resolve` (markdown/from_docx/revise.rs:20); `resolve_in` (markdown/from_docx/revise.rs:59)

#### `range_ends`  (L101-108)
`fn range_ends(element: &Element, out: &mut Vec<Node>)`

- **Does:** The comment range starts and ends inside `element`, in order.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L102; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L103; `range_ends` (markdown/from_docx/revise.rs:101) L105
- **Calls (std/external/unresolved):** `.clone`, `.push`, `Node::Element`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:59); `range_ends` (markdown/from_docx/revise.rs:101)

#### `restore_earlier`  (L114-131)
`fn restore_earlier(properties: &mut Element)`

- **Does:** On reject, puts back the properties a formatting change replaced.
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L115,121,128; `Element::child` (markdown/from_docx/ooxml.rs:315) L116; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L119; `Element::local` (markdown/from_docx/ooxml.rs:277) L121
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.filter`, `.find`, `.into_iter`, `.map`, `.unwrap_or_default`, `Node::Element`, `std::mem::take`
- **Macros:** `format!`, `matches!`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:59)

#### `join_paragraphs`  (L136-193)
`fn join_paragraphs(children: Vec<Node>) -> Vec<Node>`

- **Does:** Joins each paragraph whose break goes away with the paragraph after it.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L142,178,181; `is_empty_marker` (markdown/from_docx/revise.rs:197) L146; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L146; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L165
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.collect`, `.filter`, `.find`, `.into_iter`, `.iter_mut`, `.iter`, `.len`, `.map`, `.partition`, `.push_str`, `.push`, `.retain`, `.then_some`, `.to_string`, `Node::Element`, `Some`, `Vec::with_capacity`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:59)

#### `is_empty_marker`  (L197-204)
`fn is_empty_marker(local: &str) -> bool`

- **Does:** An empty marker Word writes between paragraphs: a bookmark, a comment or move range, a permission range, or a proofing mark.
- **Calls (std/external/unresolved):** `.ends_with`
- **Macros:** `matches!`
- **Called by:** `join_paragraphs` (markdown/from_docx/revise.rs:136)
