# Functions and methods in `src/markdown/from_docx/revise.rs`

11 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `resolve`  (L20-37, pub(crate))
`pub(crate) fn resolve(element: &Element, accept: bool) -> Element`

- **Does:** The part with every tracked change accepted (`accept`) or rejected.
- **Calls (crate):** `resolve_in` (markdown/from_docx/revise.rs:72) L22; `resolve::{fn box_text}` (markdown/from_docx/revise.rs:24) L35
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `resolve::{fn box_text}`  (L24-34)
`fn box_text(element: &mut Element, in_box: bool)`

- **Does:** Captured settled box templates must use ordinary text slots too. _(from a code comment)_
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L25,26; `resolve::{fn box_text}` (markdown/from_docx/revise.rs:24) L31
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `resolve` (markdown/from_docx/revise.rs:20); `resolve::{fn box_text}` (markdown/from_docx/revise.rs:24)

#### `removed`  (L40-46)
`fn removed(local: &str, accept: bool) -> bool`

- **Does:** Content that goes away: deletions on accept, insertions on reject.
- **Macros:** `matches!`
- **Called by:** `marked_removed` (markdown/from_docx/revise.rs:68); `resolve_in` (markdown/from_docx/revise.rs:72)

#### `kept`  (L49-55)
`fn kept(local: &str, accept: bool) -> bool`

- **Does:** Content that stays, without its wrapper.
- **Macros:** `matches!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511) (?); `render_best` (comparer/cross_para.rs:689) (?); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47672) (?); `resolve_in` (markdown/from_docx/revise.rs:72)

#### `is_marker`  (L59-64)
`fn is_marker(local: &str) -> bool`

- **Does:** A revision marker inside a properties element (`trPr`, `tcPr`, a paragraph mark's `rPr`), or a record of a formatting change.
- **Calls (std/external/unresolved):** `.ends_with`
- **Macros:** `matches!`
- **Called by:** `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1526) (?); `Layout::line_face_metrics` (convert/mod.rs:23868) (?); `resolve_in` (markdown/from_docx/revise.rs:72)

#### `marked_removed`  (L68-70)
`fn marked_removed(properties: Option<&Element>, accept: bool) -> bool`

- **Does:** Whether a properties element marks its row, cell or paragraph break as going away.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L69; `removed` (markdown/from_docx/revise.rs:40) L69; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L69
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:72)

#### `resolve_in`  (L72-111)
`fn resolve_in(element: &mut Element, accept: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:318) L73; `restore_earlier` (markdown/from_docx/revise.rs:127) L75; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L83; `is_marker` (markdown/from_docx/revise.rs:59) L84; `marked_removed` (markdown/from_docx/revise.rs:68) L88,89,100; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L88,89; `removed` (markdown/from_docx/revise.rs:40) L90; `range_ends` (markdown/from_docx/revise.rs:114) L96; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L100; `resolve_in` (markdown/from_docx/revise.rs:72) L103; `kept` (markdown/from_docx/revise.rs:49) L104; `join_paragraphs` (markdown/from_docx/revise.rs:149) L110
- **Calls (std/external/unresolved):** `.as_str`, `.ends_with`, `.extend`, `.len`, `.push`, `.to_string`, `Node::Element`, `String::new`, `Vec::with_capacity`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `resolve` (markdown/from_docx/revise.rs:20); `resolve_in` (markdown/from_docx/revise.rs:72)

#### `range_ends`  (L114-121)
`fn range_ends(element: &Element, out: &mut Vec<Node>)`

- **Does:** The comment range starts and ends inside `element`, in order.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L115; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L116; `range_ends` (markdown/from_docx/revise.rs:114) L118
- **Calls (std/external/unresolved):** `.clone`, `.push`, `Node::Element`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:72); `range_ends` (markdown/from_docx/revise.rs:114)

#### `restore_earlier`  (L127-144)
`fn restore_earlier(properties: &mut Element)`

- **Does:** On reject, puts back the properties a formatting change replaced.
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L128,134,141; `Element::child` (markdown/from_docx/ooxml.rs:386) L129; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L132; `Element::local` (markdown/from_docx/ooxml.rs:318) L134
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.filter`, `.find`, `.into_iter`, `.map`, `.unwrap_or_default`, `Node::Element`, `std::mem::take`
- **Macros:** `format!`, `matches!`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:72)

#### `join_paragraphs`  (L149-206)
`fn join_paragraphs(children: Vec<Node>) -> Vec<Node>`

- **Does:** Joins each paragraph whose break goes away with the paragraph after it.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L155,191,194; `is_empty_marker` (markdown/from_docx/revise.rs:210) L159; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L159; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L178
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.collect`, `.filter`, `.find`, `.into_iter`, `.iter_mut`, `.iter`, `.len`, `.map`, `.partition`, `.push_str`, `.push`, `.retain`, `.then_some`, `.to_string`, `Node::Element`, `Some`, `Vec::with_capacity`, `std::mem::take`
- **Macros:** `matches!`
- **Called by:** `resolve_in` (markdown/from_docx/revise.rs:72)

#### `is_empty_marker`  (L210-217)
`fn is_empty_marker(local: &str) -> bool`

- **Does:** An empty marker Word writes between paragraphs: a bookmark, a comment or move range, a permission range, or a proofing mark.
- **Calls (std/external/unresolved):** `.ends_with`
- **Macros:** `matches!`
- **Called by:** `join_paragraphs` (markdown/from_docx/revise.rs:149)
