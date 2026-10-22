# Functions and methods in `src/changes/logical.rs`

17 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `Phase::of`  (L64-70)
`fn of(kind: ChangeKind) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `groups` (changes/logical.rs:163)

#### `Phase::then`  (L75-85)
`fn then(self, kind: ChangeKind) -> Option<Self>`

- **Does:** The phase after a mark of `kind`, or `None` when the mark starts a new change: a replacement holds one run of each kind, in either order.
- **Calls (std/external/unresolved):** `.then_some`, `Some`
- **Called by:** none found in src

#### `logical`  (L90-152, pub(super))
`pub(super) fn logical(dom: &Dom, root: NodeId, carriers: &[Carrier]) -> Vec<Change>`

- **Does:** The logical changes of one story, in document order: each group of `carriers` folded into one change, every other listed carrier as it is.
- **Calls (crate):** `groups` (changes/logical.rs:163) L91; `composite` (changes/logical.rs:295) L100; `listed` (changes/logical.rs:154) L100,109,133; `folded` (changes/logical.rs:316) L134
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.as_str`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.filter`, `.flat_map`, `.flatten`, `.get`, `.insert`, `.iter`, `.map`, `.push`, `.then`, `.to_string`, `.unwrap_or`, `.zip`, `HashMap::new`, `Vec::new`, `rename`
- **Called by:** `logical_changes_in` (changes.rs:584)

#### `listed`  (L154-159)
`fn listed(carriers: &[Carrier], i: usize) -> &Change`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `.expect`
- **Called by:** `logical` (changes/logical.rs:90); `groups` (changes/logical.rs:163); `apply_ppr` (convert/mod.rs:4774) (?)

#### `groups`  (L163-266)
`fn groups(dom: &Dom, root: NodeId, carriers: &[Carrier]) -> Vec<Vec<usize>>`

- **Does:** The groups of two or more carriers, each in reading order (a paragraph's mark after its text).
- **Calls (crate):** `walk` (changes/logical.rs:399) L180; `listed` (changes/logical.rs:154) L204,217,226; `groupable` (changes/logical.rs:280) L227; `Phase::of` (changes/logical.rs:64) L251
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.clear`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter_map`, `.filter`, `.get`, `.insert`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push`, `.remove`, `.retain`, `.take_while`, `.then_some`, `.then`, `HashMap::new`, `HashSet::new`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `logical` (changes/logical.rs:90); `AnchorChanges::records` (edit/comment_settlement.rs:78) (?)

#### `wraps`  (L270-275)
`fn wraps(change: &Change) -> bool`

- **Does:** The listed carriers whose content the walk descends: run-level insertions, deletions and moves, and paragraph marks.
- **Macros:** `matches!`
- **Called by:** none found in src

#### `groupable`  (L280-284)
`fn groupable(change: &Change) -> bool`

- **Does:** Whether a carrier can join others: an insertion or deletion of text or of a paragraph mark whose id has a short spelling (a composite holds short ids only).
- **Calls (crate):** `single` (changes/logical.rs:287) L283
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `matches!`
- **Called by:** `groups` (changes/logical.rs:163)

#### `single`  (L287-292)
`fn single(id: &str) -> Option<(Story, u32)>`

- **Does:** The story and `w:id` of a one-change short id.
- **Calls (std/external/unresolved):** `.len`, `.parse`, `Some`
- **Called by:** `groupable` (changes/logical.rs:280); `outside_anchor` (markdown/xml.rs:2331) (?)

#### `composite`  (L295-297)
`fn composite<'a>(members: impl IntoIterator<Item = &'a Change>) -> String`

- **Does:** The id of changes `members` together: `r7+8+9+10`, `header1.r1+2`.
- **Calls (crate):** `joined` (changes/logical.rs:301) L296
- **Calls (std/external/unresolved):** `.as_str`, `.into_iter`, `.map`
- **Called by:** `logical` (changes/logical.rs:90); `tests::a_composite_keeps_its_story_and_reading_order` (changes/logical.rs:612)

#### `joined`  (L301-313)
`fn joined<'a>(ids: impl IntoIterator<Item = &'a str>) -> String`

- **Does:** The composite id of the marks `ids` names, in their order: every mark of a change is in one story, the first's.
- **Calls (std/external/unresolved):** `.filter_map`, `.get_or_insert`, `.into_iter`, `.push`, `.to_string`, `.unwrap_or`, `Vec::new`
- **Called by:** `composite` (changes/logical.rs:295); `composite_of` (changes/logical.rs:572); `field_tests::omml_d_draws_its_delimiters` (convert/mod.rs:38043) (?); `tests::a_space_before_a_change_that_ends_the_paragraph_goes_inside_it` (markdown/from_docx/critic.rs:2111) (?); `tests::a_separator_next_to_a_wholly_changed_part_joins_its_change` (markdown/from_docx/critic.rs:2415) (?)

#### `folded`  (L316-380)
`fn folded(id: &str, members: &[&Change]) -> Change`

- **Does:** One change standing for `members`, in reading order.
- **Calls (crate):** `text_of` (changes.rs:1518) (by name) L344,345,348; `earliest` (changes/logical.rs:383) L362
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.clone`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.iter`, `.map`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `logical` (changes/logical.rs:90); `Writer::end_paragraph` (markdown/write.rs:3102) (?)

#### `earliest`  (L383-395)
`fn earliest(members: &[&Change]) -> Option<String>`

- **Does:** The earliest member date that reads as an instant; else the first one.
- **Calls (crate):** `timestamp_order_key` (convert/mod.rs:12427) L385; `key` (ambiguous: `same_path::{fn key}` (bin/jubarte.rs:201); `key` (edit/ranges.rs:25); `key` (markdown/patch.rs:316)) L391
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.cmp`, `.filter_map`, `.find_map`, `.iter`, `.map`, `.min_by`, `.or_else`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `folded` (changes/logical.rs:316)

#### `walk`  (L399-484)
`fn walk( dom: &Dom, node: NodeId, wrappers: &HashMap<NodeId, usize>, stack: &mut Vec<usize>, out: &mut Vec<Event>, )`

- **Does:** Walk `node`'s children in document order, with `stack` the listed carriers around them.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L406; `Dom::name` (xmllinq/mod.rs:526) L407; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L410; `walk` (changes/logical.rs:399) L417,423,435,440; `XName::local_name` (xmllinq/mod.rs:208) (by name) L421; `mark_carriers` (changes/logical.rs:494) L427; `empty_text` (changes/logical.rs:487) L479
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.get`, `.len`, `.pop`, `.push`, `.truncate`, `Event::Atom`
- **Called by:** `groups` (changes/logical.rs:163); `walk` (changes/logical.rs:399)

#### `empty_text`  (L487-491)
`fn empty_text(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether text element `node` holds no character.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L488; `Dom::text_value` (xmllinq/mod.rs:547) L490
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_none_or`
- **Called by:** `walk` (changes/logical.rs:399)

#### `mark_carriers`  (L494-504)
`fn mark_carriers(dom: &Dom, p: NodeId, wrappers: &HashMap<NodeId, usize>) -> Vec<usize>`

- **Does:** The listed carriers on paragraph `p`'s mark, in order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L495,496; `Dom::elements` (xmllinq/mod.rs:640) L498
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.map`, `.unwrap_or_default`, `W::p_pr`, `W::r_pr`
- **Called by:** `walk` (changes/logical.rs:399)

#### `check_whole`  (L517-570, pub(super))
`pub(super) fn check_whole(filter: &ChangeFilter, logical: &[Change]) -> Result<(), ChangeError>`

- **Does:** Refuse a selection that names only some of a logical change's marks: the marks `filter` selects (its ids, [`ChangeFilter::normalized`], with its authors and kinds), taken together, must be every mark of each change they touch, or one whole side of a replacement (rejecting only its deletion keeps both texts, as Word al...
- **Calls (crate):** `side` (markdown/borders.rs:189) (by name) L547; `composite_of` (changes/logical.rs:572) L552,564
- **Calls (std/external/unresolved):** `.all`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.iter`, `.len`, `Err`, `Ok`, `Some`
- **Called by:** `check_whole_changes` (changes.rs:596); `resolve_for_edit_or_cli` (changes.rs:922)

#### `composite_of`  (L572-574)
`fn composite_of(members: &[&ChangeMember]) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `joined` (changes/logical.rs:301) L573
- **Calls (std/external/unresolved):** `.as_str`, `.iter`, `.map`
- **Called by:** `check_whole` (changes/logical.rs:517)

## Test functions (3)

- `tests::a_replacement_holds_one_run_of_each_kind` (L581): _no doc comment_
- `tests::change` (L593): _no doc comment_
- `tests::a_composite_keeps_its_story_and_reading_order` (L612): _no doc comment_
