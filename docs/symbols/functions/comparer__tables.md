# Functions and methods in `src/comparer/tables.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `set`  (L29-31)
`fn set(names: &[XName]) -> HashSet<XName>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.iter`
- **Called by:** `windows_font_dirs_from` (convert/font.rs:2855) (?); `merge_normal_style_spacing` (document_comparer.rs:777) (?); `tests::with` (edit/flags.rs:1496) (?); `Geometry::changed` (edit/sections.rs:183) (?)

#### `recursion_info`  (L215-219, pub)
`pub fn recursion_info(name: &XName) -> Option<&'static RecursionInfo>`

- **Does:** Find the [`RecursionInfo`] entry for an atomizer element name, if any.
- **Calls (std/external/unresolved):** `.find`, `.iter`
- **Called by:** `recurse` (comparer/atomize.rs:410)
