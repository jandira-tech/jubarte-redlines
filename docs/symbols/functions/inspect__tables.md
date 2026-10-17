# Functions and methods in `src/inspect/tables.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `nearest`  (L43-45)
`fn nearest(dom: &Dom, node: NodeId, name: &XName) -> Option<NodeId>`

- **Does:** The nearest ancestor named `name`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L44
- **Calls (std/external/unresolved):** `.copied`, `.first`, `Some`
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:95) (?); `body_tables` (inspect/tables.rs:48); `operations` (markdown/patch.rs:496) (?)

#### `body_tables`  (L48-122, pub(super))
`pub(super) fn body_tables(dom: &Dom, body: NodeId) -> Vec<Table>`

- **Does:** Every table under `body`, in document order.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:747) L54; `nearest` (inspect/tables.rs:43) L55,66,96; `Dom::descendants` (xmllinq/mod.rs:668) L59,63,94; `Dom::element` (xmllinq/mod.rs:655) L71,72,78; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L74; `Dom::elements` (xmllinq/mod.rs:640) L81; `Dom::attribute` (xmllinq/mod.rs:874) L84; `paragraph_id` (ids.rs:410) L102; `project_paragraph` (inspect.rs:1024) L106
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.count`, `.entry`, `.enumerate`, `.filter`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.join`, `.map`, `.ok`, `.or_default`, `.parse`, `.push`, `.take_while`, `.unwrap_or_default`, `.unwrap_or`, `HashMap::new`, `Some`, `W::name`, `W::tbl`, `W::tc`, `W::tr_pr`, `W::val`
- **Macros:** `matches!`
- **Called by:** `tables` (inspect.rs:348); `snapshot` (inspect.rs:362)
