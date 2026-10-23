# dedup-0013 remediation plan

TDD red/green plan and patch for the findings this dedup round owns.
Patch: `.scan/remediation/patches/dedup-0013.patch` (one unified diff, applies
with `git apply` from the repository root — verified with
`git apply --check --verbose .scan/remediation/patches/dedup-0013.patch`,
which printed `Checking patch …` for all five files and exited 0).

## Scope

`prompt.md` names one claimed worker for this round:
`6d215851-0bd3-4a82-8506-b63fc4c12def`. Of the 46 findings in
`output/result.json`, four carry **only** that worker's provenance (this
round's own contributions, per the serial-dedup rule in the ask):

1. "Revision settlement performs quadratic paragraph correspondence work"
   (`…6d215851:13`)
2. "Nested Markdown header or footer fields can overflow the conversion
   stack" (`…6d215851:15`)
3. "UTF-16 XML can retain confidential text after successful scrub or
   redaction" (`…6d215851:12`)
4. "Privacy scrubbing can return identifying data in valid relocated DOCX
   parts" (`…6d215851:11`)

The other twelve findings whose `extensions.semanticReductionNotes` mention
`dedup-0013` are composites this round only *absorbed a source into*; their
identities and bulk evidence come from earlier rounds
(`dedup-0002`…`dedup-0012` workers, visible in each finding's
`provenance.sourceFindingIds`). Per the ask's preference for this round's own
contributions over inherited material, they are **not** patched here; the
earlier-round folders own them. Within the four patched findings, two
sub-aspects stayed out of this patch and are listed under "Known remaining
gaps" below.

Threat-model context read: `output/threatmodel.md` lines 129, 161, 355, 419
(each names the concrete `src/` files this plan patches).

Nothing in this patch touches rendering, layout, or DOCX writing shapes, so
no Word-parity surface changes. Two fixes change privacy-relevant behavior
only by catching more (fail-closed), never by letting more through.

**Not run:** cargo/mbx/rustc and every test command — forbidden for this
task (concurrent Cargo in this checkout). All red/green claims below are
code-reading traces against the current sources, cited line by line; each
test is written to be the executable proof when the patch is applied and
`cargo nextest run` is next available.

---

## 1. Revision settlement performs quadratic paragraph correspondence work

**Finding:** `…6d215851:13` (medium, availability). "A compact DOCX with many
paragraphs and one tracked insertion can force a Cartesian scan of original
and settled paragraphs… The scan runs even when no paragraph merged."

**Vulnerable code (read):** `src/edit/origin.rs:645-654` — the final
merged-singleton reconciliation of `fates()`:

```rust
for post in 0..tags.len() {
    let members: Vec<usize> = (0..count)
        .filter(|&g| fates[g] == Fate::Merged(post))
        .collect();
    …
}
```

`tags.len()` settled paragraphs × `count` source fates = O(N²) comparisons,
unconditionally, even when no paragraph merged. Reached from every revision
settlement: `src/edit.rs:2128-2136` builds `Origin` whenever `changed`
(`src/edit.rs:1378-1414` settles revisions on read/edit/template/v2 paths —
`src/edit/v2_apply.rs:186-196`, `src/edit/for_edit.rs:1622-1633`,
`src/edit/template.rs:92-98`).

**Failing tests (in the patch):**

- `src/edit/origin.rs`, test `fates_costs_one_pass_over_the_fates_when_nothing_merged`
  — builds 120,000 tagged paragraphs (the no-merge settlement shape:
  every settled paragraph carries its source's tag), calls the private
  `fates()` directly, asserts every fate is `Kept` **and** that the call
  finishes in under 5 seconds. Red today: 1.44×10¹⁰ comparisons (minutes in
  a debug build, ≥7 s even in release). Wall-clock bound tests are house
  style (`tests/perf_identical_input01.rs:32-39` uses the same shape).
- `src/edit/origin.rs`, test `fates_promotes_an_unchanged_singleton_merge_to_kept`
  — the semantic the loop implements, previously untested: an untagged
  settled paragraph holding exactly one source's unchanged text is `Kept`,
  not `Merged`; a singleton whose text changed stays `Merged`. This is the
  correctness pin for the rewrite.

**Minimal source change:** replace the per-post rescans with one pass that
groups merged members by settled paragraph (`member_count` / `only_member`
arrays), then inspect only singleton groups. The rewrite is
observationally equivalent: each source paragraph is `Merged` under exactly
one settled index, so per-post member sets are disjoint and promotions at
earlier posts cannot change later posts' sets (this is also why the old
in-place mutation while looping over `post` was safe).

**Must keep passing:** the existing `fates` tests
(`untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them`,
`src/edit/origin.rs:944-1021`) and every `Origin` consumer test
(`split_correspondence_is_refused_by_origin` and the exact-correspondence
tests, `src/edit/origin.rs:684-818`).

## 2. Nested Markdown header or footer fields can overflow the conversion stack

**Finding:** `…6d215851:15` (medium, availability). "An attacker-controlled
header/footer scalar can contain thousands of nested short field results.
The Markdown story writer recursively materializes each result without a
nesting limit."

**Vulnerable code (read):** `src/markdown/view_stories.rs:835-852` (old
numbering) — `Field::Coded` recursed via `runs(result, marks, drawings)` with
no depth bound; one `runs` frame per nesting level. `result_end`
(`src/markdown/view_stories.rs:1235-1260` old numbering; the finding's
1173-1197 range is `tracked_piece`, which writes fields inside marks without
recursing) already counts
nesting when finding the matching `{/}`, and `critic::field_code_close`
bounds a single code (`LONGEST_FIELD_CODE`, `src/markdown/critic.rs:1607`),
but nothing bounded the recursion. `Marks.error`
(`src/markdown/view_stories.rs:206-212`) is an existing, never-yet-set error
channel that the story loop already fails the conversion with
(`src/markdown/view_stories.rs:1413-1415` → `MarkdownError::Package`).

**Failing tests (in the patch):**

- `src/markdown/view_stories.rs`, test
  `a_scalar_of_thousands_of_nested_fields_refuses_rather_than_recursing` —
  50,000 nested `{PAGE}` results; asserts `marks.error` is set and exactly
  `MOST_FIELD_DEPTH * 2 + 1` `w:fldSimple` occurrences were written. Red
  today: the process dies with a stack overflow (each `runs` frame is
  several hundred bytes; ~2 MB test-thread stacks overflow well before
  50,000 frames), so the test binary aborts.
- `src/markdown/view_stories.rs`, test
  `field_results_nested_past_the_budget_refuse_the_story` — the boundary:
  20 levels materialize completely with no error; 21 levels set
  `marks.error` to the typed refusal message and still write closed markup
  (the refusing level emits the empty `<w:fldSimple w:instr=" … "/>` Word
  accepts). Red today: 21 levels set no error.

**Minimal source change:** split `runs` into a depth-0 wrapper plus
`runs_at_depth(text, marks, drawings, depth)`; at `depth >=
MOST_FIELD_DEPTH` (20 — far past any nesting Word writes, far below stack
exhaustion) set `marks.error` once (`get_or_insert_with`) and emit the empty
simple field instead of recursing. The conversion then refuses through the
existing `MarkdownError::Package` path — typed refusal, no panic, no
partially-open markup.

**Must keep passing:** `fields_and_changes_become_runs_and_marks` and
`a_field_with_its_result_keeps_its_code_and_result`
(`src/markdown/view_stories.rs:1888-1938` old numbering) — single fields,
fields inside tracked marks, escaped braces and bare fields are unchanged
(the recursion is the only touched branch); `MOST_DEPTH` map-nesting and
`MOST_ID` behavior untouched.

## 3. UTF-16 XML can retain confidential text after successful scrub or redaction

**Finding:** `…6d215851:12` (medium, confidentiality). "Jubarte accepts
UTF-16 XML but scrub and leak detection read it as lossy UTF-8… redaction
postchecks can miss a remaining confidential copy in UTF-16 custom XML while
returning successful output."

**Vulnerable code (read):**

- `src/scrub.rs:467-475` (old numbering) — `leaks_within`'s XML branch read
  every `.xml`/`.rels` part with `String::from_utf8_lossy`, so a BOM-marked
  UTF-16 part became NUL-interleaved text: `xml_holds` never matched, and
  the raw-UTF-16 fallback ran only for non-XML names.
- `src/opc/mod.rs:561-564` — `part_string` is lossy for everyone.
- `src/scrub.rs:189-208` (old numbering) — `Part::load/store` used it, so
  `scrub`'s transforms silently skipped UTF-16 parts (parse of NUL-named
  elements finds no `dc:creator` to remove) and `store` could only write
  UTF-8.
- `src/edit.rs:4727-4749` — `check_redactions` calls `crate::scrub::leaks`
  on both outputs (`src/edit/prepared.rs:804`, `:873`), so the gap certified
  redactions.
- `src/admission.rs:764-782` — `part_text` already decodes BOM-marked
  UTF-16 exactly as Word accepts (SharePoint `customXml` note in its doc
  comment), but only admission used it.

**Failing tests (in the patch):**

- `src/scrub.rs` (`mod public_memory_leak_boundary_tests`), test
  `a_needle_in_utf16_xml_is_a_leak` — a `customXml/utf16le.xml` and a
  `customXml/utf16be.xml` part, each a BOM-marked UTF-16 encoding of
  `<metadata><note>PrivateReviewer</note></metadata>`; asserts
  `leaks(&bytes, "PrivateReviewer") == [name]`. Red today: the lossy read
  hides the needle behind NULs, `leaks` returns `[]` and a redaction
  certifies.
- `tests/scrub_and_redact.rs`, test `utf16_property_parts_are_scrubbed_and_stay_utf16`
  — a UTF-16LE and a UTF-16BE `docProps/core.xml` carrying
  `<dc:creator>Jane Secret</dc:creator>`; after `scrub` with default
  options the output part must still start with the same BOM (UTF-16 stays
  UTF-16), contain no `Jane Secret`, and keep `Deal memo` (the title
  survives). Red today: the part is skipped wholesale, the creator survives
  the scrub.

**Minimal source change:**

- `src/admission.rs`: `part_text` becomes `pub(crate)` (doc comment now
  states every same-reading consumer decodes through it). No behavior
  change in admission.
- `src/scrub.rs` `leaks_within`: the XML branch decodes with
  `crate::admission::part_text(&bytes)` before `xml_holds` — one decoder for
  admission and the leak check, BOM-marked UTF-16 included.
- `src/scrub.rs` `Part`: a small `Encoding` enum (`Plain`/`Utf16Le`/`Utf16Be`)
  detected from the part's BOM; `load` decodes through
  `crate::admission::part_text`, `store` re-encodes through
  `Encoding::encode`, so a scrubbed UTF-16 part is written back as UTF-16
  with its mark — never silently re-encoded.

**Must keep passing:** every existing leak test
(`public_leaks_reports_exact_xml_and_binary_part_owners_without_mutating_sources`,
`decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not`,
`src/scrub.rs:666-736` old numbering; `leaks_names_each_part_that_holds_the_text`
and `leaks_finds_escaped_text_split_across_runs`,
`tests/scrub_and_redact.rs:456-473`, `:538-544`): UTF-8 XML, entities,
split runs, numeric-attribute exemptions and binary parts behave exactly as
before (`part_text` is byte-identical to the lossy read for BOM-less
input). Scrub tests `default_scrub_removes_authors_rsids_properties_and_comments`
and `an_alias_alone_renames_comment_authors_and_people_and_keeps_the_rest`
(`tests/scrub_and_redact.rs:147-225`) pin the UTF-8 transforms.

## 4. Privacy scrubbing can return identifying data in valid relocated DOCX parts

**Finding:** `…6d215851:11` (low, confidentiality — `severity.level: "low"`
in `findings.json`). "A document returned
successfully by scrub can retain creator/editor identities, company/custom
properties, revision authors, session IDs and people information when valid
OPC relationships locate those parts outside conventional filenames or
`word/`."

**Vulnerable code (read):** `src/scrub.rs:357-384` (old numbering) —
`scrub_properties` hardcoded `docProps/core.xml`, `app.xml`, `custom.xml`;
`src/scrub.rs:304-324` — the people cleanup loaded only `word/people.xml`;
OPC part identity is relationship/content-type based
(`src/revision_processor.rs:3689-3715` already discovers headers, footers,
comments and styles through the main document's relationships — the
convention `scrub` ignored for its own parts). `scrub`'s final gate
(`introduced`, `src/scrub.rs:142-150` old numbering) only rejects *new*
validation findings, so a skipped part means "success".

**Failing test (in the patch):**

- `tests/scrub_and_redact.rs`, test
  `scrub_finds_property_and_people_parts_through_their_relationships` —
  helper `relocated_identifying_document()` rebuilds
  `identifying_document()` with `docProps/{core,app,custom}.xml` moved to
  `metadata/{core,app,custom}.xml` (package relationships retargeted,
  content-type overrides moved) and `word/people.xml` moved to
  `metadata/people.xml` (people relationship retargeted from the main
  document) — a valid OPC shape Word reads through its relationships. The
  test asserts the scrubbed output carries none of `Jane Secret`,
  `jane@secret.example`, `Secret Corp`, `<cp:revision>`, `<dcterms:created`;
  that `metadata/custom.xml` is gone with its relationship; that
  `metadata/people.xml` keeps one aliased person without `presenceInfo`;
  and that `metadata/core.xml` keeps the harmless title `Deal memo`. Red
  today: `scrub_properties` finds no `docProps/*` parts, the people cleanup
  finds no `word/people.xml`, and every one of those strings survives a
  scrub that returns `Ok`.

**Minimal source change (`src/scrub.rs`):**

- `package_related_parts(pkg, rel_type)` — a part-name list for a package
  relationship type (`get_all_by_type`, external targets excluded,
  targets resolved through `PartFs::resolve_rel_target("", …)`).
- `part_candidates(pkg, rel_type, conventional)` — relationship targets
  plus the conventional name when the package holds it, deduped: the
  relationships are authoritative; the conventional name keeps today's
  behavior for packages that name none, and an orphan at the conventional
  name is scrubbed too.
- `scrub_properties` iterates `part_candidates(…, CORE_REL, CORE)` and
  `part_candidates(…, APP_REL, APP)` for the element removals, removes
  every `part_candidates(…, CUSTOM_REL, CUSTOM)` part with its content-type
  override, then drops the custom-properties package relationships.
- `people_part(pkg)` — the people part through the main document's
  `people` relationship (`http://schemas.microsoft.com/office/2011/relationships/people`,
  the type `src/document_comparer.rs:3371-3376` already uses), conventional
  name as fallback; `alias_authors` uses it both for its skip and for the
  `Part::load`.

New constants `CORE_REL`, `APP_REL`, `PEOPLE_REL` sit beside the existing
`CUSTOM_REL` (`src/scrub.rs:39-47` old numbering).

**Must keep passing:** `default_scrub_removes_authors_rsids_properties_and_comments`,
`an_alias_alone_renames_comment_authors_and_people_and_keeps_the_rest`,
`people_with_several_authors_become_one_alias`, `scrub_keeps_a_finding_the_source_already_had`
(`tests/scrub_and_redact.rs:147-260`, `:439-453`) — conventional packages
take the same paths as before (conventional names are always candidates),
so their scrubbed output is byte-for-byte what it was.

## Known remaining gaps (deliberately out of this patch)

- **Relocated *story* parts for `alias_authors`/`remove_rsids`** (evidence
  `scrub-word-filter`, `src/scrub.rs:210-216` old numbering): authors/rsids
  inside parts under non-`word/` names. The whole engine's story consumers
  key on `word/` and the relationship set `revision_bearing_parts` builds;
  widening `word_parts` alone would be inconsistent with
  comments/settings handling and needs its own round with
  `revision_bearing_parts`-aligned discovery. This sub-aspect is shared
  with inherited finding `335ff7a0…:5`, which earlier rounds own.
- **UTF-16 core/app under *relocated* names combined with `remove_rsids`'s
  lossy `part_string` pre-filter** (`src/scrub.rs:329` old numbering): a
  UTF-16 `word/settings.xml`-style part holding rsids is skipped by the
  `contains("rsid")` gate. Word writes these parts as UTF-8; the BOM-aware
  `Part::load/store` from fix 3 already covers every part the transforms
  do reach, at conventional or relocated names.
- **The comments pass's people removal keeps the conventional name**
  (`prune_people` loads `word/people.xml` through the `PEOPLE` constant,
  `src/revision_processor/comments.rs:27-28, 466-493`): with
  `comments: true` and no comment left, a conventional package drops its
  people part, while a *relocated* one survives the comments pass and is
  then aliased by the patched `alias_authors` (one `Author` person, no
  `presenceInfo` — nothing identifying survives, but the output shape
  differs: the part is kept, not dropped). The relocated test asserts that
  actual behavior; widening `prune_people` belongs with the
  `revision_bearing_parts`-aligned discovery round above.
- The twelve composite findings inherited from earlier dedup rounds (table
  spans, HTML colspan, legacy DOC fields/pieces, Chicago numbering, ZIP
  footer mismatch, DTD/CDATA, three symlink findings, cyclic drawings, list
  levels, entity decoding) — owned by their own folders per the ask's
  preference rule.

## How to run the tests once Cargo is available

Per AGENTS.md, sequentially in this checkout:

```
cargo nextest run -E 'test(fates_) or test(field_results_nested) or test(a_scalar_of_thousands) or test(a_needle_in_utf16_xml) or test(utf16_property_parts) or test(scrub_finds_property_and_people_parts)'
```

plus the neighboring suites the plan says must keep passing:

```
cargo nextest run -E 'test(leaks_) or test(scrub_) or test(exact_correspondence) or test(written) or test(fields_and_changes)'
cargo fmt --check && cargo clippy -- -D warnings
```

The patch adds no `#[allow]`; every new warning would be a clippy failure
by policy.
