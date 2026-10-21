# Functions and methods in `src/edit/hint.rs`

14 non-test, 13 test. Part of the [symbol manifest](../README.md).

#### `PlanSyntax::revision`  (L33-41, pub(super))
`pub(super) const fn revision(self) -> super::refusal::RevisionSyntax`

- **Does:** The same choice as a refusal's data records it.
- **Calls:** none
- **Called by:** `RevisionRefusal::new` (edit/refusal.rs:51)

#### `Held::of`  (L58-70, pub(super))
`pub(super) fn of(dom: &Dom, node: NodeId) -> Option<Self>`

- **Does:** The kind of `node`, a `w:ins`, `w:del`, `w:moveTo`, `w:moveFrom`, `w:pPrChange` or `w:rPrChange`.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L68
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.map`, `W::name`
- **Called by:** `removed_holder` (edit/hint.rs:144); `edited_revisions` (edit/v2_apply.rs:2344); `Transaction::refuse_opaque` (edit.rs:5196); `Transaction::revision_hint` (edit.rs:5259)

#### `Held::verb`  (L72-80)
`fn verb(self) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `named` (edit/hint.rs:259) (?); `check_keep` (edit/visible.rs:71) (?)

#### `Held::settles`  (L83-114)
`fn settles(self) -> [(&'static str, &'static str); 2]`

- **Does:** The accept and reject routes, each with what it costs.
- **Calls:** none
- **Called by:** `settle_hint` (edit/hint.rs:211)

#### `removed_holder`  (L144-181, pub(super))
`pub(super) fn removed_holder(dom: &Dom, paragraph: NodeId, find: &str) -> Option<NodeId>`

- **Does:** The `w:del` or `w:moveFrom` of `paragraph` whose text `find` reaches into, reading the paragraph's visible and removed text in document order, glyphs as the projection reads them (a tab as `\t`): an anchor copied from another author's deletion misses the visible text, and the hint should name the deletion, not the mis...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L150; `Dom::parent` (xmllinq/mod.rs:597) L152; `Dom::name_is` (xmllinq/mod.rs:535) L152; `Dom::ancestors` (xmllinq/mod.rs:770) L153,162; `run_text` (edit/hint.rs:185) L157; `Held::of` (edit/hint.rs:58) L165; `reading_hits` (edit/anchor.rs:79) L174
- **Calls (std/external/unresolved):** `.as_str`, `.find_map`, `.find`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `Some`, `String::new`, `Vec::new`, `W::p`, `W::r`, `crate::markdown::plain_anchor_with_removed`, `crate::markdown::plain_anchor`
- **Macros:** `matches!`
- **Called by:** `Transaction::removed_anchor_in` (edit.rs:5354)

#### `run_text`  (L185-204)
`fn run_text(dom: &Dom, node: NodeId) -> Option<String>`

- **Does:** What a run child reads as in the paragraph's text: its text, or the character the projection gives a glyph; `None` for the rest.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L186; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L186; `XName::local_name` (xmllinq/mod.rs:208) (by name) L187; `Dom::value` (xmllinq/mod.rs:1185) L188; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L192
- **Calls (std/external/unresolved):** `.filter`, `.to_string`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `removed_holder` (edit/hint.rs:144)

#### `settle_hint`  (L211-247, pub(super))
`pub(super) fn settle_hint(holder: &Holder<'_>, syntax: PlanSyntax) -> String`

- **Does:** The refusal for an edit on text `holder` holds: what holds it, then accept, reject and a comment, each a fragment the caller can paste.
- **Calls (crate):** `named` (edit/hint.rs:259) L212; `Held::settles` (edit/hint.rs:83) L213; `restore` (edit/hint.rs:285) L214; `settle` (edit/hint.rs:301) L224,225; `comment` (edit/hint.rs:330) L226
- **Calls (std/external/unresolved):** `.chain`, `.chars`, `.collect`, `.count`, `.filter`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.len`, `.map`, `.max`, `.push`, `.repeat`, `.unwrap_or`, `Some`
- **Macros:** `format!`, `vec!`
- **Called by:** `tests::a_deletion_offers_restore_first_then_both_settles_then_comment` (edit/hint.rs:379); `tests::each_syntax_quotes_its_own_fragments` (edit/hint.rs:394); `tests::every_kind_names_what_the_revision_did` (edit/hint.rs:442); `tests::a_deletion_restore_cannot_take_offers_no_visible_route` (edit/hint.rs:461); `tests::the_restore_route_parses_whatever_the_id_holds` (edit/hint.rs:474); `tests::a_mark_without_text_is_named_without_quotes` (edit/hint.rs:490); `tests::every_json_route_parses_whatever_the_ids_hold` (edit/hint.rs:527); `tests::v2_routes_parse_as_the_plan_v2_types` (edit/hint.rs:546); `tests::a_paragraph_the_hint_cannot_name_is_left_to_fill_in` (edit/hint.rs:560); `tests::a_long_revision_is_quoted_with_the_cut_said` (edit/hint.rs:580); `Transaction::revision_hint` (edit.rs:5259)

#### `restored_here_hint`  (L251-256, pub(super))
`pub(super) fn restored_here_hint(holder: &Holder<'_>) -> String`

- **Does:** The refusal of an edit inside a deletion the same command restores: the restore lands first, and the edit goes in a later command.
- **Calls (crate):** `named` (edit/hint.rs:259) L254
- **Macros:** `format!`
- **Called by:** `Transaction::revision_hint` (edit.rs:5259)

#### `named`  (L259-272)
`fn named(holder: &Holder<'_>) -> String`

- **Does:** The lead with the revision named: its id, text, what it did, by whom.
- **Calls (crate):** `quoted` (edit/hint.rs:353) L263; `.verb` (ambiguous: `Held::verb` (edit/hint.rs:72); `Visible::verb` (edit/visible.rs:33)) L269
- **Calls (std/external/unresolved):** `.is_empty`, `String::new`
- **Macros:** `format!`
- **Called by:** `settle_hint` (edit/hint.rs:211); `restored_here_hint` (edit/hint.rs:251)

#### `json`  (L279-281)
`fn json(text: &str) -> String`

- **Does:** `text` as a JSON string, quotes and backslashes escaped.
- **Calls (std/external/unresolved):** `.to_string`, `serde_json::Value::from`
- **Called by:** `restore` (edit/hint.rs:285); `settle` (edit/hint.rs:301); `comment` (edit/hint.rs:330)

#### `restore`  (L285-298)
`fn restore(syntax: PlanSyntax, holder: &Holder<'_>) -> Option<String>`

- **Does:** The restore of the revision, as `syntax` spells it: offered for a deletion with words restore takes, in a syntax that has it.
- **Calls (crate):** `json` (edit/hint.rs:279) L292
- **Calls (std/external/unresolved):** `.is_empty`, `Some`
- **Macros:** `format!`
- **Called by:** `publish_dir` (bin/jubarte.rs:2019) (?); `settle_hint` (edit/hint.rs:211)

#### `settle`  (L301-326)
`fn settle(syntax: PlanSyntax, verb: &str, holder: &Holder<'_>) -> String`

- **Does:** `verb` (`accept` or `reject`) of the revision, as `syntax` spells it.
- **Calls (crate):** `json` (edit/hint.rs:279) L306,315,316
- **Calls (std/external/unresolved):** `.as_str`
- **Macros:** `format!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581) (?); `settle_hint` (edit/hint.rs:211); `Select::parse` (markdown/mod.rs:461) (?)

#### `comment`  (L330-350)
`fn comment(syntax: PlanSyntax, paragraph: &str) -> String`

- **Does:** A comment on the paragraph, as `syntax` spells it; a paragraph the hint could not name is left to fill in.
- **Calls (crate):** `json` (edit/hint.rs:279) L339
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `format!`
- **Called by:** `settle_hint` (edit/hint.rs:211)

#### `quoted`  (L353-359)
`fn quoted(text: &str) -> String`

- **Does:** `text` in quotes, cut to [`QUOTED`] characters with the cut said.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.take`
- **Macros:** `format!`
- **Called by:** `named` (edit/hint.rs:259)

## Test functions (13)

- `tests::holder` (L365): _no doc comment_
- `tests::a_deletion_offers_restore_first_then_both_settles_then_comment` (L379): _no doc comment_
- `tests::each_syntax_quotes_its_own_fragments` (L394): _no doc comment_
- `tests::every_kind_names_what_the_revision_did` (L442): _no doc comment_
- `tests::a_deletion_restore_cannot_take_offers_no_visible_route` (L461): _no doc comment_
- `tests::the_restore_route_parses_whatever_the_id_holds` (L474): _no doc comment_
- `tests::a_mark_without_text_is_named_without_quotes` (L490): _no doc comment_
- `tests::fragment` (L506): A route line's fragment: what follows its label, without the cost.
- `tests::routes` (L513): The route lines of `hint` as JSON: accept, reject, comment (the last three, after any visible route).
- `tests::every_json_route_parses_whatever_the_ids_hold` (L527): _no doc comment_
- `tests::v2_routes_parse_as_the_plan_v2_types` (L546): _no doc comment_
- `tests::a_paragraph_the_hint_cannot_name_is_left_to_fill_in` (L560): _no doc comment_
- `tests::a_long_revision_is_quoted_with_the_cut_said` (L580): _no doc comment_
