# dedup-0007 — TDD remediation plan

Folder: `.scan/artifacts/deep_discovery/dedup/dedup-0007` (reducer round
`dedup-0007`, claimed worker `fe0f541f-91fe-400a-8489-6886ba837b3f`).
This round's own contributions are the three findings whose evidence refs
cite `dedup0007-*` and the claimed worker; all are CWE-674 uncontrolled
recursion, severity medium, confidence high:

1. `uncontrolled-recursion.unchecked-auxiliary-xml` — "Processing renamed
   auxiliary XML bypasses XML depth limits".
2. `xml-depth.dtd-parser-differential` — "Opaque DTD and top-level CDATA
   bypass package XML depth checks".
3. `uncontrolled-recursion.markdown-drawing-dependencies` — "Cyclic
   Markdown drawing dependencies terminate conversion through unbounded
   recursion".

Grounding (read this session): `src/admission.rs:531-576` (extension gate,
renamed-main re-check), `src/admission.rs:788-823`
(`check_xml_depth`), `src/xmllinq/parse.rs:500-553` (`parse_document`
`<!` skip-to-first-`>`), `src/xmllinq/parse.rs:555-679`
(`parse_element` recursion, no depth bound), `src/xmllinq/parse.rs:919-975`
(`validate_xml_with_limits` already refusing DTD / top-level CDATA /
top-level text), `src/wml_document.rs:85-96` (lazy unchecked main parse),
`src/convert/mod.rs:3480,6558,7675,7786,8527,8700` and
`src/document_comparer.rs:4744-4751,6308,6324,6676-6689` (consumer-side
unchecked `parse_xdocument`), `src/markdown/from_docx/drawing.rs:1397-1409`
(`decode` without import budgets), `:1881-1905` (`materialize`),
`:1956-2008` (`dep_part` recursion without cycle guard), and quick-xml
0.42.0 `src/parser/dtd.rs` (quote-aware internal-subset skip — the lexical
boundary the permissive parser lacked).

Hard constraint honored: no cargo/mbx/rustc was run (concurrent Cargo in
this checkout is forbidden for this task); the patch was authored by
reading code. Two non-build checks were run: `git apply --check` against
the work tree (no repository file modified), and the `rustfmt` binary
(edition 2024, `skip_children`) over the six patched files' scratch
copies — this validated that every new block parses and is
rustfmt-clean (it caught and fixed an unclosed delimiter and a
reformatted arm before the patch was finalized). All cargo checks below
are listed for the applying engineer and are **not run** here.

---

## Vulnerability 1 — `xml-depth.dtd-parser-differential`

Opaque DTD and top-level CDATA bypass the package XML depth checks.

**Root cause.** `check_xml_depth` (`src/admission.rs:788-823`) counts
`Start` events and ignores `DocType`, `CData` and `Text` (`Ok(_) => {}`,
line 819). quick-xml consumes a complete DTD — quotes and internal subset
included — as one ignored event, so hidden nesting never counts. The
permissive `parse_document` (`src/xmllinq/parse.rs:539-544`) instead skips
a `<!` construct only to the **first** `>`, so a quoted `>` inside an
entity value (or an internal-subset comment, or a top-level CDATA body)
exposes the remaining bytes as real markup, and `parse_element`
(`src/xmllinq/parse.rs:660-663`) recurses on it with no depth bound.
`validate_xml_with_limits` (`src/xmllinq/parse.rs:954,961-964`) already
refuses exactly these constructs; only the package gate and the permissive
parser disagree.

**Failing tests (red).**

- `src/admission.rs` (module `tests`)
  `opaque_dtd_cdata_and_top_level_text_are_refused_like_checked_xml` —
  builds `word/dtd.xml` = `<!DOCTYPE d [<!ENTITY e "x><a><a/></a>">]>`,
  `word/dtdcomment.xml` = the internal-subset-comment framing,
  `word/cdata.xml` = `<![CDATA[ > <a/> ]]><w:document/>`, and
  `word/text.xml` = `text<w:document/>`; asserts each package is refused
  with `AdmissionErrorKind::InvalidXml` and that `validate_xml` refuses
  the same strings; asserts in-root CDATA (`word/fine.xml` =
  `<a><![CDATA[raw > <b/>]]></a>`) still admits.
  Red today: all four packages admit (`kind(...)` panics with
  `expect_err`).
- `src/xmllinq/parse.rs` (module `xml_lexical_boundary_contract_tests`)
  `top_level_declarations_hide_their_markup_completely` — parses
  `<!DOCTYPE d [<!ENTITY e "x><a><b/></a>">]><r/>` (and the comment
  framing); asserts the document level holds exactly one element, the
  real root `r`, with no children, and `validate_xml` refuses the input.
  Red today: the hidden `<a>` parses as a real first element, so
  `dom.nodes(document).len()` is 2 and the root is `a`.
- `src/xmllinq/parse.rs`
  `top_level_cdata_is_skipped_whole_and_in_root_cdata_stays_text` —
  parses `<![CDATA[ > <a/> ]]><r><![CDATA[x]]></r>`; asserts the root is
  `r` with no element children and in-root CDATA value `"x"` preserved.
  Red today: the CDATA body's remainder parses as elements.
- `src/xmllinq/parse.rs`
  `unadmitted_runaway_nesting_truncates_instead_of_exhausting_the_stack` —
  `parse_xdocument` on 100 000 nested `<b>` returns a root instead of
  blowing the stack (the pre-fix process aborts — nextest isolates the
  crash to this test).
- `tests/input_admission.rs`
  `get_revisions_refuses_dtd_hidden_nesting` — a `word/document.xml`
  whose DTD internal subset hides 100 000 `<a>` behind a quoted `>`
  reaches `get_revisions`; asserts a typed `Err` containing
  `INVALID_XML`. Red today: stack overflow in the permissive parse
  (`src/document_comparer.rs:6308`).
- `tests/input_admission.rs`
  `a_renamed_shallow_notes_part_and_in_root_cdata_keep_admitting` — the
  green companion: `WmlDocument::from_bytes` accepts a shallow,
  well-formed renamed notes part and a main part with in-root CDATA.

**Minimal source change.**

- `src/admission.rs` `check_xml_depth`: add three arms matching
  `validate_xml_with_limits` verbatim — `Event::DocType` →
  `"DTD declarations are unsupported"`, `Event::CData` at depth 0 →
  `"CDATA outside XML root"`, non-whitespace `Event::Text` at depth 0 →
  `"text outside XML root"`, all typed `InvalidXml`.
- `src/xmllinq/parse.rs` `parse_document`: skip a top-level
  `<![CDATA[` section whole (to `]]>`); skip a `<!` declaration whole via
  a new quote/bracket-aware `skip_declaration` (quote bodies and the
  `[...]` internal subset are opaque; the declaration ends at the `>`
  that closes it), mirroring quick-xml's `DocType` boundary.
- `src/xmllinq/parse.rs` `parse_element`: a DOM-construction backstop
  `MAX_DOM_NESTING = 4096` — past it the parser stops descending and
  drops the remainder. It is stack safety, not policy: every shipped
  budget (`max_xml_depth` 256, default and compare) is far below it, so
  gated input never reaches it; only a caller that bypassed every gate
  sees truncation where it previously saw a crash. Stack headroom: the
  backstop test holds 4096 recursive `parse_element` frames in a debug
  test binary, which is why the repo standard (and CI) runs unit tests
  under `cargo nextest` — one process per test on the main thread's
  8 MiB stack (`.github/workflows/ci.yml:99`; CI's only `cargo test
  --lib` is the separate jubarte-wasm workspace at `ci.yml:112`). If the
  suite is ever run through libtest's spawned 2 MiB test threads, lower
  the constant (1024 keeps 4× the strictest budget) rather than
  re-widening the exposure.

**Existing behaviour that must keep passing.**

- Ordinary OOXML parts (no top-level `<!` constructs, in-root CDATA only)
  parse byte-identically; the M1.8 round-trip gate is untouched (a
  complete-skip changes only how far a top-level declaration is consumed,
  and OOXML parts have none).
- `validate_xml`/`try_parse_xdocument` semantics unchanged (the new
  admission arms copy their strings).
- `checked_xml_rejects_invalid_or_unbounded_structure` and
  `checked_xml_accepts_normal_documents_and_entities`
  (`src/xmllinq/parse.rs`) keep passing, including the `257`/`256` depth
  boundary (far under the 4096 backstop).
- `src/admission.rs` `a_plain_docx_is_admitted`, `deep_or_malformed_xml_is_refused`,
  `mismatched_unclosed_and_stray_end_tags_are_refused`,
  `utf16_and_non_utf8_parts_are_scanned_not_refused` (whitespace-only
  leading text and UTF-16 transcoding still admit; the 300-deep UTF-16
  part still refuses).

## Vulnerability 2 — `uncontrolled-recursion.unchecked-auxiliary-xml`

Relationship-selected auxiliary XML under non-`.xml` names bypasses the
XML depth limits.

**Root cause.** The admission scan (`src/admission.rs:533-547`) applies
`check_xml_depth` by filename suffix (`.xml`/`.rels`) and re-checks only
the renamed main part (`:556-570`). Consumers select parts by
relationship: `main_rel_part` (theme/styles/numbering/settings/fontTable —
`src/convert/mod.rs:3307,3480,6558,21310-21320`,
`src/convert/font_table.rs:67`), `part_xml_by_rel_kind`
(comments/commentsExtended/footnotes/endnotes/diagramDrawing —
`src/convert/mod.rs:8473-8489,7675,7786,8527,8700`),
`story_parts_xml` (headers/footers — `src/convert/mod.rs:21285-21299`),
and the comparer's `notes_part_names`/`pre_process_markup`
(`src/document_comparer.rs:6461-6480,4724-4751`, used by `get_revisions`
`:6318-6328` and `compare` `:6676-6689`). None requires an XML suffix, so
a deeply nested `word/styles.bin` (or renamed footnotes part) is admitted
unread and then recursed on by `parse_xdocument` with no depth bound.

**Failing tests (red).**

- `src/admission.rs` `related_parts_are_depth_checked_whatever_their_extension`
  — a `word/_rels/document.xml.rels` styles relationship to
  `word/styles.bin` holding 300-deep XML is refused `InvalidXml`; the
  same package with a shallow `styles.bin` still admits (arbitrary part
  names remain supported).
- `src/admission.rs` `an_xml_relationship_kind_checks_even_a_binary_typed_target`
  — the same refusal when `[Content_Types].xml` declares
  `bin → application/octet-stream` (the relationship kind governs: the
  engine parses what it selects).
- `src/admission.rs` `an_untyped_related_target_that_looks_like_xml_is_checked`
  — a thumbnail-kind relationship to an untyped `word/thumb.dat` holding
  deep XML is refused; a shallow one admits (the BOM-aware sniff:
  `<` in UTF-8 or either UTF-16 endianness).
- `src/admission.rs` `media_and_altchunk_relationship_targets_are_not_xml_checked`
  — the green companions: an image relationship to PNG bytes and an
  `aFChunk` relationship to non-well-formed HTML still admit (an
  `altChunk` is read by the HTML parser, not this DOM; media is never
  XML).
- `tests/input_admission.rs` `get_revisions_refuses_deep_xml_under_a_renamed_notes_part`,
  `compare_refuses_deep_xml_under_a_renamed_notes_part`,
  `wml_document_refuses_deep_xml_under_a_renamed_notes_part` — a
  footnotes relationship to `word/notes.bin` holding 100 000-deep XML is
  a typed `INVALID_XML` refusal from each public entry point (admission
  runs first in all three: `src/document_comparer.rs:6299`, `:6607-6608`,
  `src/wml_document.rs:51`). Red today: stack overflow at
  `src/document_comparer.rs:6324` / `6678` / `4744-4751` and the render
  loaders.
- `tests/input_admission.rs`
  `a_renamed_shallow_notes_part_keeps_working_end_to_end` — green: the
  same package with a shallow, well-formed renamed notes part opens
  (`WmlDocument::from_bytes` ok).

**Minimal source change.** One central fix in `src/admission.rs::admit`,
the boundary every entry point already crosses — the remediation's
"resolve/classify and admit XML through actual content-type/relationship
use":

- During the entry scan, every `.rels` part's internal `Relationship`
  targets are collected (owner derived by reusing
  `crate::opc::rels_path_to_part_name`, now `pub(crate)`; targets
  resolved by the existing `crate::opc::resolve_rel_target`).
- After the scan (and after the renamed-main re-check), each related
  target not already covered (`.xml`/`.rels` suffix, or the main part) is
  depth-checked when any of: an XML relationship kind names it
  (`XML_REL_KINDS` — exactly the kinds the engine's loaders select:
  styles, settings, theme, numbering, fontTable, comments,
  commentsExtended, commentsIds, commentsExtensible, footnotes,
  endnotes, header, footer, chart (the comparer's chart preprocessing,
  `src/document_comparer.rs:4861-4878`, selects by `/chart` and parses
  ungated), diagramData, diagramDrawing); its declared
  content type is an XML media type (`+xml` suffix, `application/xml`,
  `text/xml`, `…/xml` — **not** a bare `contains("xml")`, which would
  swallow `application/vnd.openxmlformats-…sheet` ZIP embeddings such as
  `tests/fixtures/strict/Strict01.docx`'s xlsx); or it carries no content
  type and its first bytes look like XML (BOM-aware `<` sniff; media and
  embedded fonts never start with one). A declared non-XML content type
  settles the part as non-XML with no sniff; `aFChunk` relationship kinds
  are skipped outright — an altChunk is HTML the engine reads with an
  HTML parser, not this DOM — so sloppy HTML chunks stay open exactly as
  today.

**Existing behaviour that must keep passing.**

- The whole fixture corpus was swept for internal non-XML relationship
  targets (command in the session log): only `docProps/thumbnail.jpeg`,
  `media/*.png|jpeg|emf|bin` and `embeddings/*.xlsx` — all skipped by the
  classification (image/thumbnail/package kinds, non-XML content types,
  non-`<` bytes). No fixture changes behaviour.
- `tests/m_part_rels_dangling.rs` (dangling targets): `by_name` misses →
  skipped, exactly as today.
- `a_main_part_without_an_xml_extension_is_still_depth_checked` and
  `the_main_part_follows_the_relationship` (`src/admission.rs`) — the
  renamed-main pass is untouched and runs first.
- External and URL-scheme targets never resolve to an entry → skipped.

**Stated residual (not fixed here).** A related target that no XML
relationship kind names, whose content type declares non-XML media, and
whose bytes still look like XML is skipped without a sniff (e.g. an
image-kind relationship to `<a>…` bytes declared `image/png`). This is
not consumer-free: `linked_txbx_content` (`src/convert/mod.rs:17417-17428`)
and `diagram_label_runs` (`src/convert/mod.rs:17453-17464`) resolve parts
by relationship **id** — any kind, via `rel_target_path`/`resolve_media`
— and parse the bytes ungated at `:17426`/`:17462`, so a declared-non-XML
part they select still reaches the recursive DOM (the corpus sweep found
no such part: every non-`.xml` related target is media/font/package
bytes, and the one `txbx`-kind rel targets `txbx1.xml`). Closing it needs
the finding's other option — checked parses at the consumer
(`try_parse_xdocument_with_limits`) or a per-relationship consumer map;
recorded as follow-up.

## Vulnerability 3 — `uncontrolled-recursion.markdown-drawing-dependencies`

Cyclic Markdown drawing dependencies terminate conversion through
unbounded recursion.

**Root cause.** `![label](drawing:PAYLOAD)` is base64-decoded and
serde-deserialized into a `Drawing` with no import validation
(`src/markdown/from_docx/drawing.rs:1397-1409`): the capture-side budgets
(`MAX_DEPS`/`MAX_DEP_RELS`, `:61-63`, enforced only at capture `:596-617`)
do not apply. `materialize` → `dep_part` (`:1956-2008`) recursively
follows `rel.part` indices with no active-node set, so a one-node
self-reference (or a two-node cycle) recurses until the stack ends; a
long acyclic chain does the same past 100 k deps. `name_for`'s 998-name
cap (`:2012-2028`) does not stop a self-edge, and the byte-equality reuse
(`:1998-2004`) happens only after the recursion.

**Failing tests (red).**

- `src/markdown/from_docx/drawing.rs` (module `tests`)
  `cyclic_dependency_payloads_refuse_instead_of_recursing` — a
  one-node self-cycle and a two-node cycle through a second dep both
  make `materialize` return `None` (the existing refusal channel: the
  payload is skipped, exactly like a payload whose relationships name no
  part). Red today: stack overflow.
- `src/markdown/from_docx/drawing.rs`
  `acyclic_chains_and_shared_dependencies_still_materialize` — green
  companion: an 8-part linear chain materializes, every part written and
  reachable (unchanged behaviour for every valid payload, whose bytes
  round-trip untouched).
- `src/markdown/from_docx/drawing.rs`
  `decode_refuses_payloads_past_the_dependency_budgets` — a payload with
  `MAX_DEPS + 1` deps, one with a dep holding `MAX_DEP_RELS + 1`
  relationships, and one with `MAX_DEP_RELS + 1` top-level relationships
  are refused at `decode` (typed `Err(String)` → alt-text fallback at
  `src/markdown/write.rs:4193,4202`); exactly-at-budget still decodes.
  With deps bounded at 64, `dep_part`'s active-path bound also bounds
  recursion depth for acyclic chains.
- `tests/markdown_to_docx.rs` `a_cyclic_drawing_payload_converts_without_recursing`
  — public-API red: hand-crafted payload JSON (the serde field names)
  base64-encoded into `![box](drawing:…)`, converted with `write()`
  (which also asserts the OOXML SDK reads the result); red today: stack
  overflow during `package::assemble`.

**Minimal source change.**

- `decode` re-imposes the capture budgets at import via
  `check_import_budgets` (deps ≤ `MAX_DEPS`; top-level and per-dep
  relationships ≤ `MAX_DEP_RELS`), the decode-side
  dependency/edge budget the finding's tests ask for.
- `dep_part` gains an `active: &mut Vec<usize>` path (push on entry, pop
  on exit); a revisit of an active index is a cycle → `None`, and with
  deps ≤ 64 the recursion depth is bounded by the budget, closing the
  long-chain case too. No memoization is added: two rels sharing a dep
  keep writing it under distinct names today, and changing that would
  alter output bytes of valid documents.

**Existing behaviour that must keep passing.**

- Every valid `drawing:` payload decodes and materializes exactly as
  before (no budget it observes changed: capture already capped at these
  values); `mermaid_unchanged`/`rebuild_mermaid` byte-for-byte round trips
  are untouched.
- `tests/markdown_to_docx.rs` and `tests/docx_markdown_round_trip.rs`
  drawing/round-trip tests (e.g. `a_kept_part_whose_w_prefix_*`):
  unchanged — the guard only fires on graphs capture can never produce.

## Patch

`../patches/dedup-0007.patch` — one unified diff over
`src/admission.rs`, `src/opc/mod.rs`, `src/xmllinq/parse.rs`,
`src/markdown/from_docx/drawing.rs`, `tests/input_admission.rs`,
`tests/markdown_to_docx.rs`. It applies with `git apply` from the
repository root (verified with `git apply --check` this session; no
repository file was modified).

## Verification (for the applying engineer — NOT RUN here; cargo is
forbidden in this checkout for this task)

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run -E 'test(opaque_dtd) or test(top_level_) or test(unadmitted_runaway)
  or test(related_parts_) or test(an_xml_relationship) or test(an_untyped_related)
  or test(media_and_altchunk) or test(get_revisions_refuses) or test(compare_refuses_deep)
  or test(wml_document_refuses) or test(a_renamed_shallow)
  or test(cyclic_dependency) or test(acyclic_chains) or test(decode_refuses)
  or test(a_cyclic_drawing)'
cargo nextest run -E 'test(admission) or test(input_admission) or test(markdown_to_docx)'
jubarte --help   # CLI smoke test after install/PATH, per AGENTS.md
```

Then the pre-existing suites that guard the touched behaviour:
`cargo nextest run --test input_admission`, `--test m_cli_no_panic`,
`--test m_part_rels_dangling`, `--test markdown_to_docx`,
`--test docx_markdown_round_trip`, and the unit suites in
`src/admission.rs`, `src/xmllinq/parse.rs`,
`src/markdown/from_docx/drawing.rs`.

## Second-pass review record (dedup-0007 issue opener)

Re-verified against the repository without cargo: `git apply --check`
passes from the repo root and every hunk header's counts match its body.
Every helper the new code calls exists and is in scope (`elements`
`src/admission.rs:893`, `content_type_of` `:873`, `resolve_rel_target`
already `pub(crate)` `src/opc/mod.rs:473`, `rels_path_to_part_name`
`:400`, test helpers `docx_with`/`kind`/`zip_of`/`TYPES`/`RELS`/`DOC`
`:1345-1371`; `PartFs::parts`/`part_bytes`/`set_part`/
`add_content_type_override`; `base64_encode` `drawing.rs:1417`; the
serde field names of `PayRel`/`Dep`/`Drawing` match the hand-written
JSON in `tests/markdown_to_docx.rs`; `write` `tests/markdown_to_docx.rs:24`
asserts word-validity and OOXML-SDK readability; admission runs first in
all three public entry points — `get_revisions`
`src/document_comparer.rs:6299`, compare `:6530`/`:6559`,
`WmlDocument::from_bytes` `src/wml_document.rs:51`). A UTF-8 BOM cannot
trip the new depth-0 text arm: quick-xml 0.42 strips one leading BOM
during `ParseState::Init` (`bom_from_str` in its own reader tests), the
same reader `validate_xml_with_limits` uses. Fixture sweep (934 zip
fixtures, unzip + first-bytes): no `.xml`/`.rels` part carries a DTD or
top-level text, and no related non-`.xml` target looks like XML — the
refusals and the classification change no fixture's behaviour.

Three corrections made in this pass:

1. Unit test `an_xml_relationship_kind_checks_even_a_binary_typed_target`
   used `Target="word/notes.bin"` from owner `word/document.xml`;
   `resolve_rel_target` joins the owner's directory
   (`oxml-opc-0.14.2/src/package.rs:451`), so it resolved to
   `word/word/notes.bin`, missed the zip entry, and the expected refusal
   never happened — the test failed even with the fix applied. Now
   `Target="notes.bin"`, as the integration helper already spells it.
2. `chart` added to `XML_REL_KINDS` (15 → 16): the comparer's chart
   preprocessing selects parts by `rel_type.ends_with("/chart")`
   (`src/document_comparer.rs:4867`) and parses them ungated (`:4878`),
   a consumer the finding explicitly names; the corpus carries 8 chart
   relationships (all `.xml` targets, so no fixture changes behaviour).
3. The residual paragraph claimed "no confirmed engine consumer parses
   such a part through the recursive DOM" — false:
   `linked_txbx_content` and `diagram_label_runs`
   (`src/convert/mod.rs:17417-17464`) resolve by relationship id (any
   kind) and parse ungated; the residual now names them and their
   follow-up (consumer-side checked parses).

Hunk headers updated for the two added lines
(`@@ -816,10 +852,162 @@`, `@@ -1870,6 +2058,135 @@`);
`git apply --check` re-run clean after the edits.

Fourth correction (issue opener, this round): `looks_like_xml`'s
`match head { … => true, _ => false }` shape fires clippy's
`match_like_matches_macro` (style group; CI's `cargo clippy --all-targets
-- -D warnings` makes it a build failure) — verified with the
stable toolchain's `clippy-driver` on the exact shape, standalone. It is
now `matches!(head, [b'<', ..] | [0x00, b'<', ..] | [0xFF, 0xFE, b'<',
0x00, ..] | [0xFE, 0xFF, 0x00, b'<', ..])` (behaviour identical), the two
affected hunk headers are now `@@ -816,10 +852,161 @@` and
`@@ -1870,6 +2057,135 @@`, and `git apply --check` plus a
rustfmt `--check` over fresh scratch copies of all six patched files
(edition 2024, with the sibling modules copied in so rustfmt resolves
them) both pass. Also re-probed every other new free-standing shape
(`rel_kind_is_xml`, `content_type_is_xml`, `part_head`, `admit_related_xml`
skeleton, `skip_declaration`, `check_import_budgets`) under the workspace
lint set from `Cargo.toml [lints.clippy]` (correctness denied;
suspicious/perf plus `semicolon_if_nothing_returned`,
`uninlined_format_args`, `manual_let_else`, `needless_pass_by_value`):
clean. The fixture sweep was independently repeated this round (478
`.rels` parts across every `.docx`/`.dotx`/`.docm` under `tests/fixtures`,
Python zipfile): the only non-`.xml`/`.rels` internal targets are
`image`/`thumbnail`/`package`-kinded media, thumbnails and embedded
`.xlsx` — same population the first pass found; the two `.bin` media
targets and both `.xlsx` embeddings start `0xd7…`/`0x89PNG…`/`PK…`, never
`<`.
