<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Handoff: Plan 1, Task 3 (S2, comment threads), branch `adopt/s2-comments`

Written 2026-10-02 at the end of a session that was stopped before any
implementation. Everything below was read from the tree at `main`
`b420d64` (the branch has no commits of its own besides this file).
No subagents were used; there is no agent output to fold in.

## State of the branch

- Branch: `adopt/s2-comments`, created from `main` `b420d64`.
- Source changes: none. Tests: none. Nothing compiled, nothing run.
- Toolchain verified present: cargo 1.97.0, rustc 1.97, uv 0.8.17.
- The plan file was fetched to `/tmp/plan.md` with
  `git show origin/ccr-a92b0695-1adjlu:docs/superpowers/plans/2026-10-02-provider-adoption-1-s01-s10.md`.
  Task 3 is lines 725 to 968 of that file. Re-fetch it; `/tmp` does not
  survive the container.

## The task (from the plan, verbatim scope)

S2, TODO.md section 6: `src/comments.rs` with `list_comments` and the
part-family writers; edit operations `reply_comment`, `resolve_comment`,
`edit_comment`, `delete_comment`, and `through` on `comment`;
`write_comments_part` writing `w14:paraId` and the three extended parts;
capabilities; CLI `jubarte comments`; Python `Document.comments()` and
`EditPlan` builders; WASM `listComments`; skill, TODO.md, CHANGELOG.

Gates before each commit: `cargo fmt --check`; `cargo clippy --all-targets
--all-features -- -D warnings`; the task's tests plus `tests/edit_plan.rs`,
`tests/m_validity_ring1.rs`, `tests/agent_contracts.rs`,
`tests/m147_comments_union_carryover.rs`; `cargo run --bin jubarte --
--help`; `uv tool run --from 'reuse[charset-normalizer]' reuse lint`; for
Python, `cd jubarte-python && uv run --with maturin maturin develop
--release && uv run --with pytest --with pytest-cov pytest -q
--cov=jubarte_redlines --cov-branch --cov-report=term-missing`. Full
`cargo test --all-features` once before the final push. Finish: rebase on
`origin/main`, rerun gates, push, open a DRAFT PR against `main`.

Conflict discipline: append new `OperationKind` variants at the END of the
enum, new rows at the END of the `match` in `check_operation_keys`, new
names at the END of `capabilities.edit_operations` and of the ordered list
in `tests/agent_contracts.rs`; new code in new files; CHANGELOG bullets
under `## [Unreleased]` only.

## Verified facts about the current code (file:line are on `b420d64`)

### `src/edit.rs` (3527 lines)

- `OperationKind` enum: `src/edit.rs:129-270`, serde `tag = "kind"`,
  `rename_all = "snake_case"`. Last variant today is `MergeParagraphs`.
  `Comment { paragraph: Selector, find: Option<String>, text: String }`
  at about line 196.
- `check_operation_keys` at `src/edit.rs:783-830`. The row for `comment`
  is `"comment" => &["find", "text"]`; `COMMON` is `["id","kind","paragraph"]`.
  The `other =>` arm refuses unknown kinds with `INVALID_PLAN`. New kinds
  that have no `paragraph` still pass because `paragraph` is only allowed,
  not required, by this check.
- `kind_name` at `src/edit.rs:2716-2727` maps every variant to its wire
  name; it must gain the four new names (it is an exhaustive match).
- `Transaction` struct at `src/edit.rs:1034-1065`: holds `base: Vec<u8>`,
  `opened: Opened`, `stories: Vec<StoryPart>` (body first), `paragraph_nodes`,
  `paragraph_story: Vec<(story, local index)>`, `projections`, `outcomes`,
  `resolved: Vec<(op index, Resolved)>`, `comments: Vec<(u32, String)>`
  (new comments pending write), `deletion_comments`, `preset_comment_ids`,
  `next_comment_id: u64`, `comments_added`, `whole_marks`.
- `Resolved` enum at `src/edit.rs:1003-1030`: `Text`, `CommentRange`,
  `DeleteParagraph`, `FormatParagraph`, `MergeParagraphs`, `InsertParagraph`.
  `touched_stories` (line ~1895) and `check_conflicts` (line ~2008) and
  `check_comment_ids_fit` (line ~2196) match on it exhaustively, so every
  new variant must be added to those matches.
- `resolve` at `src/edit.rs:1221-1273`; `resolve_one` at `1275-1640`
  starts by extracting the `paragraph` selector from every variant with an
  exhaustive `match` (line ~1296). The new kinds have no paragraph, so
  `resolve_one` must branch before that match (or `resolve` must route
  them, as it routes `Rewrite` to `resolve_rewrite`).
- The comment-in-body guard is at `src/edit.rs:1313-1323`: it refuses with
  `UNSUPPORTED_STRUCTURE` and message "comments are supported in the body
  only". The plan wants the code `COMMENT_NOT_IN_BODY` for `comment` on a
  header/footer story. Changing the code of the existing path changes
  behaviour tests may assert; grep `tests/` for "supported in the body"
  before deciding. Safer: use `COMMENT_NOT_IN_BODY` only for the new
  `through` path and for `comment`, and leave `replace`/`insert` with
  `comment` fields on the old code, or change all and update tests.
- `apply` at `src/edit.rs:2231-2460`: step 0 assigns comment ids in plan
  order (`new_comment`), step 1 applies text edits and calls
  `anchor_comment(dom, paragraph, start, end, id)` (line 2907), steps 2 to
  5 insert paragraphs, format, merge, delete.
- `anchor_comment` wraps one paragraph range with `commentRangeStart`,
  `commentRangeEnd`, and a `w:r/w:commentReference` run after the end
  marker (`src/edit.rs:2907-2924`, uses `wrap_range` and `split_run_at`).
  A multi-paragraph range (`through`) needs a new helper: start marker
  placed by `wrap_range` logic in the first paragraph, end marker and
  reference run in the last paragraph. `wrap_range` returns false when the
  range holds no run; then markers are appended to the paragraph.
- `finish` at `src/edit.rs:2518-2548` calls `write_comments_part` only
  when `self.comments` is nonempty, then writes the body and touched
  stories and zips.
- `write_comments_part` at `src/edit.rs:2584-2655`: loads or creates
  `word/comments.xml` (root `w:comments` with `xmlns:w` and `xmlns:r`),
  appends one `w:comment` per `(id, text)` with `w:id`, `w:author`,
  `w:date`, `w:initials`, a first run holding `w:annotationRef`, then one
  run per `\n`-separated line (`w:br` before lines after the first) in ONE
  `w:p`. It writes no `w14:paraId` and none of the extended parts. When
  the part is new it adds the relationship (`COMMENTS_REL`, target
  `comments.xml`) and the content type override (`COMMENTS_CT`).
- `existing_comment_ids` at `src/edit.rs:2691` returns the max `w:id`.
- `commented_base` (line ~2470) builds a second Transaction on the base
  with `Comment` operations for commented paragraph deletions; it
  constructs `OperationKind::Comment { paragraph, find: None, text }`
  literally, so adding a `through` field to `Comment` requires adding
  `through: None` there.

### `src/inspect.rs`

- `Opened` is `pub(crate)` (`src/inspect.rs:280-294`): `pkg: PartFs`,
  `main`, `dom`, `document`, `body`. `Opened::open` runs admission, strict
  translation, parses the main part.
- `Opened::related(kind) -> BTreeSet<String>` (line 436) resolves the main
  part's relationships whose type ends in `/kind` ("comments", "header",
  "styles"...).
- `Opened::story_parts()` (line 406): `(id, kind, part)` for headers,
  footers, footnotes, endnotes.
- `parse_part(pkg, name) -> (Dom, document, root)` (line 460).
- `body_paragraph_nodes` (523), `story_paragraph_nodes` (531),
  `project_paragraph(dom, p) -> Projection` (618). `Projection` is
  `pub(crate)`: `text`, `segments: Vec<Segment{start,end,piece,direct}>`,
  `spans`, `field_marks`. Comment range markers are skipped by the walker
  (`walk_container`, line ~660), so projection offsets ignore them.
- `InspectError` variants: `Package`, `MissingDocument`, `Invalid`,
  `Admission(AdmissionError)`; `edit::open_error` maps `Admission` to its
  own code and the rest to `INVALID_DOCUMENT`.

### Ring 1 validity (`tests/common/validity.rs`, the oracle)

- `check_comment_graph` (495-618): every `commentRangeStart`,
  `commentRangeEnd`, `commentReference` id must be defined once in
  `word/comments.xml` and each definition must have all three markers.
  Note it reads markers from the MAIN part only.
- If any of `commentsExtended.xml`, `commentsIds.xml`,
  `commentsExtensible.xml` is present, every `w:comment` needs at least one
  `w:p` carrying `w14:paraId` (8 hex digits, value below `0x8000_0000`,
  unique within `comments.xml`). The key of a comment is the paraId of its
  LAST paragraph.
- `check_comments_extended` (705): rows `w15:commentEx` keyed by
  `w15:paraId`; the key set must EQUAL the set of last paraIds; a
  `w15:paraIdParent` must resolve to a key and not cycle.
- `check_comments_ids` (739): rows keyed by `paraId` with a `durableId`
  (8 hex digits, no range bound); key set must EQUAL the last paraIds;
  durableIds unique.
- `check_comments_extensible` (774): rows keyed by `durableId`; the set
  must EQUAL the commentsIds durableIds; cannot exist without commentsIds.
- `check_comment_family_packaging` (642): for each of the four parts that
  exists, exactly ONE relationship from the main part with the exact
  relationship type and the exact content type override. The constants are
  `COMMENT_FAMILY` in validity.rs and, identically, `FAMILY` in
  `src/comparer/comments.rs:35-56` (`pub(crate)`): part name, content type,
  relationship type for comments, commentsExtended, commentsIds,
  commentsExtensible.
- `check_namespace_qname_contexts` requires every prefix listed in an
  `mc:Ignorable` to be declared in scope; declare what you list.
- Helpers to reuse from tests: `assert_word_valid_package(&bytes)`,
  `check_word_valid_package(&bytes) -> ValidityReport`.

### Serializer and namespaces

- `src/xmllinq/serialize.rs:22-60` `WELL_KNOWN_PREFIXES` includes `w`, `r`,
  `mc`, `w14`, `w15`, `w16cex`, `w16cid`, `w16se`, so elements and
  attributes in those namespaces serialize with Word's prefixes. Still
  verify at test time that an attribute in the W14 namespace on a `w:p`
  whose root lacks `xmlns:w14` is serialized with a declaration; if not,
  declare `xmlns:w14` on the `w:comments` root with
  `dom.set_attribute_value(root, &XNamespace::xmlns().name("w14"), Some(W14::URI))`.
- `src/namespaces.rs` has `W`, `R`, `MC`, `W14`, `W15` via `ns_struct!`.
  No `W16CID`/`W16CEX` structs: `src/revision_processor/comments.rs:21-22`
  defines `CID = "http://schemas.microsoft.com/office/word/2016/wordml/cid"`
  and `CEX = "http://schemas.microsoft.com/office/word/2018/wordml/cex"` as
  private consts and builds names with `XName::get(local, URI)`. Do the
  same in `src/comments.rs` (or add `ns_struct!` entries; adding to
  namespaces.rs is a small shared-file edit, acceptable if appended).
- Dom API (`src/xmllinq/mod.rs`): `new_element(XName)`, `add`, `add_text`,
  `add_before_self`, `add_after_self`, `remove`, `attribute(id, &XName)`,
  `set_attribute_value(id, &XName, Option<&str>)`, `attr_count`/`attr_at`,
  `elements(id, Option<&XName>)`, `element`, `descendants`, `parent`,
  `name`, `name_is`, `value(id) -> String` (text content),
  `parse_xdocument(&str) -> doc`, `root(doc)`, `serialize_document(doc)`.
- `PartFs` (`src/opc/mod.rs`): `part_string`, `part_bytes`, `set_part`,
  `remove_part`, `read_rels_for(main) -> Option<&Relationships>` (items
  have `id`, `rel_type`, `target`, `target_mode`), `add_document_relationship(main, rel_type, target) -> id`,
  `remove_relationships_by_type(main, rel_type)`,
  `add_content_type_override("/word/x.xml", ct)`,
  `remove_content_type_override`, `resolve_rel_target`,
  `main_document_part()`, `to_zip()`. `crate::opc::relative_rel_target(main, part)`
  builds the rel target string.

### Existing reusable logic for delete and pruning

`src/revision_processor/comments.rs` (`pub(super)` only):
- `prune_orphan_comments(pkg, story_parts: &[String])` drops every
  `w:comment` with no `commentReference` in the stories, removes stray
  markers, prunes the three extended parts by paraId/durableId
  (`prune_by_para`), removes the whole family when no comment is left
  (`remove_part`), and prunes `word/people.xml` (`prune_people`, part
  `word/people.xml`, rel type
  `http://schemas.microsoft.com/office/2011/relationships/people`,
  rows `w15:person w15:author="..."`).
- Options: (a) make `prune_orphan_comments` `pub(crate)` and call it from
  `edit.rs` after removing the deleted comment's markers from the story
  DOMs (one-line visibility change in a file other sessions should not be
  touching), or (b) reimplement the small pruning in `src/comments.rs`
  (keeps the diff in the new file, which the conflict discipline prefers).
  Recommendation: (b), since `comments.rs` must own the rebuild of the
  extended parts anyway.

### The comparer's carry rule (decides what the redline shows)

`src/comparer/comments.rs` module doc and `b_carries_same_comments_as_a`
(line 111): B's four parts are emitted byte-identical only when B holds
every one of A's comments with the SAME id and the SAME fingerprint
(normalized text, author, date, initials). Otherwise the parts are UNIONED
(`union_comments_xml`, line 1180) and ids renumbered on collision.
Consequences for this task:
- `reply_comment`, `resolve_comment`: B is a superset with unchanged
  fingerprints of A's comments, so B's parts ride the redline unchanged.
  Wait: `resolve_comment` changes only `commentsExtended`, not the
  fingerprint, so B's parts are carried. Verify with the test
  `assert_eq!(list_comments(&second.redline)...len(), 2)` and a `done`
  check on the redline.
- `edit_comment` changes the text, so the fingerprint differs: the union
  path runs and the redline may carry BOTH the old and new definition, or
  A's. This is NOT settled; the plan's test only checks `edited.clean`.
  Decide and record: either accept the redline showing the union (and say
  so in the PR), or make `edit_comment` and `delete_comment` write their
  comment part changes into the base the comparer reads as well
  (`commented_base` already produces a modified base for deletion
  comments; the same hook can carry the comment-part edits so A and B
  agree and B's parts are carried byte-identical). The second approach is
  what makes "the thread rides the redline" true for edits and deletes.
- `delete_comment`: with the union rule, the deleted comment comes back in
  the redline from A unless the base is also edited. Same remedy.

### Capabilities, contracts, CLI, Python, WASM

- `src/capabilities.rs:140-151` `edit_operations` list ends with
  `"rewrite"`; append `"reply_comment"`, `"resolve_comment"`,
  `"edit_comment"`, `"delete_comment"`. Add `comment_threads: bool` to
  `Operations` with `#[serde(default)]` (append at the end of the struct).
  The unit test in that file asserts `edit_operations.len() == 9`; update
  to 13.
- `tests/agent_contracts.rs:13-24` asserts `manifest.edit_operations ==
  kinds` in order; append four JSON operations at the END, e.g.
  `{"kind":"reply_comment","comment_id":1,"text":"x"}`,
  `{"kind":"resolve_comment","comment_id":1}`,
  `{"kind":"edit_comment","comment_id":1,"text":"x"}`,
  `{"kind":"delete_comment","comment_id":1}`. That test also injects a
  misspelled key `replacment` into each operation and expects
  `INVALID_PLAN`, which `check_operation_keys` gives for free.
- `src/bin/jubarte.rs`: `enum Command` at line 136; last variant is
  `Debug` (line 420). Pattern to copy: `Changes { file, json }` (148-155)
  dispatched at line 1901 to `run_changes(&file, json)` (845-872), using
  `read_document(path)` (1451) and `exit_code` (1405). Add
  `Comments { file, json, author: Option<String>, latest: bool }` at the
  END of the enum and a `run_comments` next to `run_changes`.
- `src/lib.rs`: add `pub mod comments;` with a doc comment, alphabetically
  after `changes` (the module list is alphabetical; inserting in the
  middle is an additive one-line change).
- Python: `jubarte-python/src/lib.rs` pattern `list_changes_json`
  (`py.detach`, `serde_json::to_string`); register in `_native` at the
  end of the `add_function` list. `python/jubarte_redlines/_native.pyi`:
  add `def list_comments_json(docx: bytes) -> str: ...`.
  `models.py`: add a frozen `Comment` dataclass (fields as
  `CommentRecord`) and `_decode_comments(payload)`; add builders on
  `EditPlan` after `rewrite`: `reply_comment(comment_id, *, text, id=None)`,
  `resolve_comment(comment_id, *, done=True, id=None)`,
  `edit_comment(comment_id, *, text, id=None)`,
  `delete_comment(comment_id, *, id=None)`, and `through: Selector | None = None`
  on `comment(...)`. `document.py`: `Document.comments(self, *, author=None,
  latest=False) -> tuple[Comment, ...]` next to `changes()`; export
  `Comment` from `__init__.py` (check how `Change` is exported there).
  `__main__.py`: `cmd_changes` (249-261) and its parser (356-359) are the
  pattern for a `comments` subcommand with `--json`, `--author`, `--latest`.
  Python tests: `jubarte-python/tests/docx_fixture.py` has `docx(body_xml,
  header=None)`, `para(text)`; the plan's `test_comments.py` imports
  `from docx_fixture import docx, para`.
- WASM: `jubarte-wasm/src/lib.rs:92-96` `listChanges` returns a JSON ARRAY
  string. The plan says `listComments(docx) -> string` as JSON lines; the
  sibling API returns an array, so returning an array is the consistent
  choice. Record whichever you pick as a plan deviation in the PR.
- Skill: `skills/jubarte-documents/SKILL.md` section 1 (line 29) gets the
  `jubarte comments FILE --json` sentence; section 2's operation list
  (lines 112-125) gets the four kinds and `through`.
- `TODO.md` section 6 (lines 204-227): tick Add (range), Delete, Modify,
  Reply, List, See surroundings, Resolve; leave "Remember which party
  cares" open.
- `CHANGELOG.md`: `## [Unreleased]` / `### Added` starts at line 17.

## Design decided during the session (not yet written)

Wire types in `src/comments.rs` exactly as the plan's `CommentRecord`
(`id`, `author`, `initials?`, `date?`, `text`, `parent?`, `done`,
`paragraph?`, `anchor_text`, `before`, `after`) plus:

- `pub enum CommentError { Admission(AdmissionError), Package(String) }`
  with `Display` and `Error`, mirroring `edit::open_error`.
- `pub fn list_comments(docx) -> Result<Vec<CommentRecord>, CommentError>`:
  open through `Opened::open`, find the comments part via
  `related("comments")`, parse comments.xml, build `last paraId -> comment
  id` map, read `commentsExtended` for `parent` (paraIdParent -> comment id)
  and `done`, then walk EVERY story (body first, then `story_parts()`),
  locate `commentRangeStart`/`End` by id, and for a range inside one
  paragraph use `project_paragraph` to produce `anchor_text` (text between
  the markers, found by projecting the paragraph before and after the
  marker positions), `before`/`after` capped at 80 chars; for a range that
  spans paragraphs, `anchor_text` is the joined paragraph texts with `\n`
  and `paragraph` is the start paragraph's id (`body:p:N`, matching the
  index in `body_paragraph_nodes`). A comment with a reference but no
  range (dead comment) gets `anchor_text` empty, `paragraph` of the
  reference's paragraph.
- `pub fn select_comments(records, author: Option<&str>, latest: bool)`:
  author filter is exact match; `latest` keeps one record per thread
  (thread root = walk `parent` to the root), choosing the member with the
  greatest `date` string, ties broken by later position. Rust-side so CLI,
  Python and WASM agree.
- A `pub(crate) struct CommentFamily` loaded from `PartFs` + main part
  name: holds the comments.xml DOM and maps read from the extended parts
  (`paraId -> (parent paraId, done)`, `paraId -> durableId`,
  `durableId -> dateUtc`). Methods: `add(id, author, date, initials, text,
  parent: Option<u32>)`, `set_text(id, text)` (keeps the LAST paragraph's
  paraId stable so replies keyed on it still resolve; new paragraphs get
  fresh ids; first paragraph keeps the `annotationRef` run),
  `set_done(id, done)` on the comment and its replies, `remove(id)`
  removing the comment and its replies and returning the removed ids,
  `thread_root(id)`, and `store(pkg, main)`: ensure every comment paragraph
  has a `w14:paraId`, REBUILD the three extended parts from the comments
  part preserving known parent/done/durableId/dateUtc values, write all
  four parts with relationships and content types from `FAMILY`, or remove
  all four when no comment is left; prune `people.xml` rows to the
  remaining authors (drop the part when empty) and, when the part exists,
  add a `w15:person` for a new author (`ensure_person`).
- paraId generation: deterministic from `(comment id, paragraph index)`:
  start at `(0x1000_0000 + id * 0x11 + index * 0x100) & 0x7FFF_FFFF`,
  increment until unused within comments.xml (Ring 1 only checks
  uniqueness there). durableId: derived from the paraId with a fixed xor
  mixing constant, incremented until unique among durableIds. `dateUtc`
  = the comment's `w:date` when present.
- Root elements to write (declare every prefix listed in `mc:Ignorable`):
  `<w15:commentsEx xmlns:w15=... xmlns:w14=... xmlns:mc=... mc:Ignorable="w14 w15">`,
  `<w16cid:commentsIds xmlns:w16cid=... xmlns:mc=... mc:Ignorable="w16cid">`,
  `<w16cex:commentsExtensible xmlns:w16cex=... xmlns:mc=... mc:Ignorable="w16cex">`.
  Rows: `<w15:commentEx w15:paraId="..." [w15:paraIdParent="..."] w15:done="0|1"/>`,
  `<w16cid:commentId w16cid:paraId="..." w16cid:durableId="..."/>`,
  `<w16cex:commentExtensible w16cex:durableId="..." [w16cex:dateUtc="..."]/>`.
  `src/convert/mod.rs:6790-6816` reads `w15:commentEx` by `w15:paraId` and
  `w15:paraIdParent`; keep that shape so rendering agrees.

Edit transaction changes (`src/edit.rs`):

- Enum additions at the END: `ReplyComment { comment_id: u32, text: String }`,
  `ResolveComment { comment_id: u32, #[serde(default = "true")] done: bool }`
  (serde needs a fn returning true), `EditComment { comment_id, text }`,
  `DeleteComment { comment_id }`; `Comment` gains
  `#[serde(default, skip_serializing_if = "Option::is_none")] through: Option<Selector>`.
- `check_operation_keys` rows at the END: `"reply_comment" =>
  &["comment_id","text"]`, `"resolve_comment" => &["comment_id","done"]`,
  `"edit_comment" => &["comment_id","text"]`, `"delete_comment" =>
  &["comment_id"]`; `"comment" => &["find","text","through"]`.
- New `Resolved` variants: `CommentThread { op kind, comment_id, text,
  done }` or one per kind; `CommentSpan { first para, start, last para,
  end, text }` for `through`.
- `resolve`: route the four thread kinds to a new `resolve_thread` that
  validates `comment_id` against `list_comments(&self.base)` (refuse
  `UNKNOWN_COMMENT`), checks text with `check_comment`, and for a reply
  requires the parent to have a reference in some story
  (`UNSUPPORTED_STRUCTURE` "comment N has no anchor" otherwise). Set
  `outcome.matches = 1`, `outcome.context` to a short description.
- `through`: both selectors resolved; `through` must be in the same story
  and at or after `paragraph` (else `INVALID_EDIT`); must be body
  (`COMMENT_NOT_IN_BODY`). Range: start of first paragraph to end of last.
  `check_conflicts`: a span counts as a comment on every paragraph it
  covers for the deleted-paragraph check.
- `apply`: replies get ids from `new_comment`-like reservation but the
  text goes to `CommentFamily::add` with `parent`; the reply's markers are
  placed next to the parent's (start after the parent's start, end after
  the parent's end, reference run after the parent's reference run) in
  whichever story DOM holds them (mark that story touched). Resolve: no
  DOM change. Edit: no DOM change. Delete: remove the markers and
  reference runs of the comment and its replies from every story DOM
  (remove the `w:r` that holds only the `commentReference`), mark those
  stories touched.
- `finish`/`write_comments_part`: replace the body of
  `write_comments_part` with `CommentFamily` usage so the family is whole
  whenever jubarte touches it, and call it whenever `self.comments` is
  nonempty OR any thread operation resolved. A plain `comment` on a
  document without comments still produces all four parts (Ring 1's
  `check_comment_family_packaging` is the oracle; the plan says so).
- `commented_base`: extend so that edit/delete/resolve/reply changes to
  the comment parts are ALSO applied to the base the comparer reads (see
  the carry rule above), otherwise the redline carries stale or deleted
  comments. This is the one design point the plan does not spell out;
  decide it first and write a test on `second.redline` /
  `deleted.redline` that pins the chosen behaviour.
- `check_comment_ids_fit`: count replies as new comments.
- `EditReport.comments_added`: count replies.

## Tests to write first (red), then make green

1. `tests/edit_comment_threads.rs`: the plan's three tests
   (`reply_and_resolve_round_trip`, `edit_and_delete_leave_a_valid_package`,
   `unknown_ids_and_header_anchors_are_refused`) verbatim as the starting
   point; they use `common::docx::{docx, para, part_string}`,
   `common::validity::assert_word_valid_package`,
   `jubarte::comments::list_comments`,
   `jubarte::document_comparer::accept_revisions`,
   `jubarte::edit::{EditPlan, apply_plan}`. Expected first failure:
   "could not find `comments` in `jubarte`", then `INVALID_PLAN: unknown
   kind "reply_comment"`. Add: a redline-side assertion for edit/delete
   once the carry decision is made; a `through` test that the start marker
   is in paragraph 0 and the end marker and reference in paragraph 1; a
   `select_comments` unit test for `--author` and `--latest`; a test that
   a plain `comment` on a comment-less document now yields all four parts
   and passes Ring 1; a test that a document whose existing comments.xml
   already has paraIds and extended rows keeps them after a reply.
2. `tests/agent_contracts.rs`: append the four operations (order matters).
3. `src/capabilities.rs` unit test: length 13 and `comment_threads`.
4. CLI: a test in `tests/m_cli_agent.rs` style (`run(&[...])` helper at
   lines 28-45, `BIN` const) for `jubarte comments FILE --json`.
5. `jubarte-python/tests/test_comments.py` from the plan (step 5) plus a
   CLI test for `python -m jubarte_redlines comments`.
6. Watch `tests/edit_plan.rs:454-470` and `650-670`: they check
   `comments.xml` contents by substring and `summary().comments` counts,
   not exact XML, so paraIds should not break them; the redline comment
   count at line 457 (`summary(&result.redline).comments == 2`) will
   exercise the carry rule.
7. `tests/m147_comments_union_carryover.rs` must not change.

## Open questions to settle before coding

1. Redline semantics for `edit_comment` and `delete_comment` (carry rule
   above). Recommendation: apply the comment-part changes to the comparer's
   base too, so A and B agree and B's parts are carried byte-identical.
2. Error code for `comment`/`through` on a header story:
   `COMMENT_NOT_IN_BODY` (plan) versus the existing `UNSUPPORTED_STRUCTURE`
   at `src/edit.rs:1313`. Check `grep -rn "supported in the body" tests/`
   before changing the existing code.
3. WASM `listComments` return shape: JSON array (consistent with
   `listChanges`) versus JSON lines (plan text).
4. `people.xml`: only prune/extend when the part exists; never create it.
5. Replying to a reply: Word threads are one level deep; set
   `paraIdParent` to the thread root's paraId and record that in the docs.

## How to resume

```bash
cd /home/user/jubarte-redlines
git fetch origin main adopt/s2-comments ccr-a92b0695-1adjlu
git checkout adopt/s2-comments
git show origin/ccr-a92b0695-1adjlu:docs/superpowers/plans/2026-10-02-provider-adoption-1-s01-s10.md > /tmp/plan.md
sed -n 725,968p /tmp/plan.md
```

Then write `tests/edit_comment_threads.rs`, run
`cargo test --all-features --test edit_comment_threads` to see it fail,
create `src/comments.rs` and `pub mod comments;` in `src/lib.rs`, and
proceed through the design above, committing after each green gate run.
Remove this handoff file (or move it under the PR description) before the
final push if the maintainers do not want handoffs in `docs/`.
