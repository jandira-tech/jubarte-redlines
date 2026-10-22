# Functions and methods in `src/edit/refusal.rs`

5 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `RevisionRefusal::new`  (L51-126, pub(super))
`pub(super) fn new(tx: &Transaction<'_>, reason: &str, nodes: &[NodeId]) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L55; `revision_pair` (edit/for_edit.rs:904) L61; `XName::local_name` (xmllinq/mod.rs:208) (by name) L69; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L74,76; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L75,78; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L115; `PlanSyntax::revision` (edit/hint.rs:33) L124
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.expect`, `.find`, `.into`, `.is_some_and`, `.iter`, `.name`, `.push`, `.to_string`, `.unwrap_or_default`, `Vec::new`, `W::author`, `W::p_pr`, `W::r_pr`
- **Called by:** `Transaction::revision_hint` (edit.rs:5259)

#### `RevisionRefusal::consequence`  (L128-141)
`fn consequence(&self, accept: bool) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.join`, `.len`, `.map`
- **Macros:** `format!`
- **Called by:** `EditError::revision_next_actions` (edit/refusal.rs:159) (?)

#### `EditError::with_revision_syntax`  (L147-154, pub)
`pub fn with_revision_syntax(mut self, syntax: RevisionSyntax) -> Self`

- **Does:** Render all revision refusals, including operation outcomes, for a caller.
- **Calls (std/external/unresolved):** `.iter_mut`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `run_edit` (bin/jubarte.rs:1341) (?); `record_plan` (bin/jubarte.rs:2625) (?)

#### `EditError::revision_next_actions`  (L159-191, pub)
`pub fn revision_next_actions(&self, source_path: &str, source_digest: &str) -> Vec<NextAction>`

- **Does:** Two explicit decisions as executable template-generation argument arrays.
- **Calls (crate):** `NextAction::with_purpose` (result.rs:288) (by name) L186; `NextAction::new` (result.rs:278) L186; `RevisionRefusal::consequence` (edit/refusal.rs:128) (by name) L188
- **Calls (std/external/unresolved):** `.all`, `.clone`, `.collect`, `.contains`, `.extend`, `.filter`, `.first`, `.into_iter`, `.into`, `.is_some_and`, `.iter`, `.lines`, `.map`, `.next`, `.starts_with`, `.strip_prefix`, `.unwrap_or_default`, `.unwrap_or`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `refused` (bin/jubarte.rs:2697)

#### `Transaction::attach_revision_refusal`  (L195-202, pub(super))
`pub(super) fn attach_revision_refusal(&self, mut error: EditError) -> EditError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into_boxed_slice`, `.into_vec`, `.push`, `.take`
- **Called by:** `check` (edit/tracked.rs:107); `resolve` (edit/v2_apply.rs:1656); `Transaction::resolve` (edit.rs:3267)
