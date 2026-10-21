# Functions and methods in `src/inspect/tables.rs`

5 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `nearest`  (L61-63)
`fn nearest(dom: &Dom, node: NodeId, name: &XName) -> Option<NodeId>`

- **Does:** The nearest ancestor named `name`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L62
- **Calls (std/external/unresolved):** `.copied`, `.first`, `Some`
- **Called by:** `opaque_table` (inspect/tables.rs:66); `top_level_table_nodes` (inspect/tables.rs:71); `body_tables` (inspect/tables.rs:79); `table_grid` (inspect/tables.rs:111)

#### `opaque_table`  (L66-68, pub(crate))
`pub(crate) fn opaque_table(dom: &Dom, table: NodeId) -> bool`

- **Does:** Whether a table is hidden from tN addressing by an opaque wrapper.
- **Calls (crate):** `in_text_box` (inspect.rs:962) L67; `nearest` (inspect/tables.rs:61) L67
- **Calls (std/external/unresolved):** `.is_some`, `MC::name`
- **Called by:** `top_level_table_nodes` (inspect/tables.rs:71); `table_grid` (inspect/tables.rs:111)

#### `top_level_table_nodes`  (L71-76, pub(crate))
`pub(crate) fn top_level_table_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The tables numbered by the view: outside cells, text boxes and AC.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L72; `nearest` (inspect/tables.rs:61) L74; `opaque_table` (inspect/tables.rs:66) L74
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_none`, `Some`, `W::tbl`, `W::tc`
- **Called by:** `body_tables` (inspect/tables.rs:79); `Opened::summary` (inspect.rs:765)

#### `body_tables`  (L79-102, pub(super))
`pub(super) fn body_tables(dom: &Dom, body: NodeId) -> Vec<Table>`

- **Does:** Top-level body tables, with their nested tables under their cells.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L83; `nearest` (inspect/tables.rs:61) L84; `top_level_table_nodes` (inspect/tables.rs:71) L88; `table_grid` (inspect/tables.rs:111) L92
- **Calls (std/external/unresolved):** `.collect`, `.entry`, `.enumerate`, `.into_iter`, `.map`, `.or_default`, `.push`, `HashMap::new`, `W::tc`
- **Macros:** `format!`
- **Called by:** `tables` (inspect.rs:389); `snapshot` (inspect.rs:403); `settled_snapshot` (inspect.rs:442); `view_tables` (inspect.rs:465)

#### `table_grid`  (L111-198)
`fn table_grid( dom: &Dom, table: NodeId, path: &str, by_cell: &HashMap<NodeId, Vec<(usize, NodeId)>>, ) -> Grid`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L120,153,159; `nearest` (inspect/tables.rs:61) L123,155,163; `Dom::element` (xmllinq/mod.rs:655) L128,129,135; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L131; `Dom::elements` (xmllinq/mod.rs:640) L138; `Dom::attribute` (xmllinq/mod.rs:874) L141; `opaque_table` (inspect/tables.rs:66) L163; `table_grid` (inspect/tables.rs:111) L167; `paragraph_id` (ids.rs:410) L180; `project_paragraph` (inspect.rs:1265) L184
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.count`, `.enumerate`, `.filter`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.join`, `.map`, `.ok`, `.parse`, `.push`, `.take_while`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::tbl`, `W::tc`, `W::tr_pr`, `W::val`
- **Macros:** `format!`, `matches!`
- **Called by:** `body_tables` (inspect/tables.rs:79); `table_grid` (inspect/tables.rs:111)
