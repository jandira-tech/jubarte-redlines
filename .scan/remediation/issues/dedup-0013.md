@coderabbitai plan
Please enrich, fix and assess the plan below.

# Findings being tackled (dedup-0013)

This round owns the four findings below (the only ones whose provenance is
solely this round's worker, `6d215851-0bd3-4a82-8506-b63fc4c12def`; the other
findings mentioning `dedup-0013` in their semantic-reduction notes are
composites absorbed from earlier rounds and are owned by their own folders).

## csf_38d5438aa76b1bf2cdbb12d0

**Title:** Revision settlement performs quadratic paragraph correspondence work

**Severity:** medium

**Summary:** A compact DOCX with many paragraphs and one tracked insertion can force a Cartesian scan of original and settled paragraphs when an ordinary edit/read/template request accepts or rejects revisions. The scan runs even when no paragraph merged, allowing disproportionate CPU use.

**Remediation:** Build merged-member counts and singleton source indexes in one pass over fates, keyed by settled paragraph, then inspect only singleton merged groups. Review neighboring repeated scans and enforce an explicit correspondence-work budget.

## csf_5ca5660e582c260e59b28647

**Title:** Nested Markdown header or footer fields can overflow the conversion stack

**Severity:** medium

**Summary:** An attacker-controlled header/footer scalar can contain thousands of nested short field results. The Markdown story writer recursively materializes each result without a nesting limit, causing stack exhaustion or substantial repeated scanning before DOCX output validation.

**Remediation:** Enforce a field-result nesting and generated-XML budget before materialization, returning a typed conversion refusal. Prefer an explicit stack and avoid rescanning nested suffixes at every level.

## csf_6971fbd04aa6a684463b89f5

**Title:** UTF-16 XML can retain confidential text after successful scrub or redaction

**Severity:** medium

**Summary:** Jubarte accepts UTF-16 XML but scrub and leak detection read it as lossy UTF-8. Metadata transforms silently skip it, and redaction postchecks can miss a remaining confidential copy in UTF-16 custom XML while returning successful output.

**Remediation:** Use one encoding-aware XML decoder for admission, privacy transformations and leakage checks. Fail closed when an accepted XML part cannot be decoded or parsed for a privacy operation; scan all supported encodings consistently before certifying redaction.

## csf_9885a1f1ab291001eef81daa

**Title:** Privacy scrubbing can return identifying data in valid relocated DOCX parts

**Severity:** low

**Summary:** A document returned successfully by scrub can retain creator/editor identities, company/custom properties, revision authors, session IDs and people information when valid OPC relationships locate those parts outside conventional filenames or word/. Sharing the supposedly scrubbed output discloses the retained data. Supported relocated stories include content/main.xml and word/main.dat; a conventional prefix or .xml suffix is not semantic part identity.

**Remediation:** Discover all applicable metadata, people, comments, settings and story parts through relationships and content types. Apply the requested transforms to every discovered part and assert privacy-specific postconditions before returning success.

# TDD plan

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

# Patch

```diff
diff --git a/src/edit/origin.rs b/src/edit/origin.rs
index 54a91a95..b3b104d7 100644
--- a/src/edit/origin.rs
+++ b/src/edit/origin.rs
@@ -642,14 +642,23 @@ fn fates(
             cursor = Some((from, post, end));
         }
     }
+    // One pass gathers each settled paragraph's merged members; a group of
+    // exactly one whose text did not change is that paragraph, kept.
+    // Scanning every fate for every settled paragraph made a settlement over
+    // N paragraphs cost N² comparisons even when nothing merged.
+    let mut member_count = vec![0usize; tags.len()];
+    let mut only_member = vec![0usize; tags.len()];
+    for (g, fate) in fates.iter().enumerate() {
+        if let Fate::Merged(post) = *fate {
+            if member_count[post] == 0 {
+                only_member[post] = g;
+            }
+            member_count[post] += 1;
+        }
+    }
     for post in 0..tags.len() {
-        let members: Vec<usize> = (0..count)
-            .filter(|&g| fates[g] == Fate::Merged(post))
-            .collect();
-        if let [g] = members.as_slice()
-            && source_text(*g) == settled_text(post)
-        {
-            fates[*g] = Fate::Kept(post);
+        if member_count[post] == 1 && source_text(only_member[post]) == settled_text(post) {
+            fates[only_member[post]] = Fate::Kept(post);
         }
     }
     fates
@@ -1019,4 +1028,59 @@ mod tests {
             [Fate::Kept(0), Fate::Gone]
         );
     }
+
+    #[test]
+    fn fates_promotes_an_unchanged_singleton_merge_to_kept() {
+        let story = |_: usize| "body".to_string();
+        fn texts(all: &'static [&'static str]) -> impl Fn(usize) -> String {
+            move |i| all[i].to_string()
+        }
+        // Settled 1 carries no tag, but holds exactly source 1's text: the
+        // singleton merge is that paragraph, kept.
+        let source: &[&str] = &["A", "X"];
+        let settled: &[&str] = &["A", "X"];
+        assert_eq!(
+            fates(
+                &[Some(0), None],
+                story,
+                story,
+                texts(source),
+                texts(settled),
+                2
+            ),
+            [Fate::Kept(0), Fate::Kept(1)]
+        );
+        // A singleton whose text changed on the way stays a merge.
+        let settled: &[&str] = &["A", "X!"];
+        assert_eq!(
+            fates(
+                &[Some(0), None],
+                story,
+                story,
+                texts(source),
+                texts(settled),
+                2
+            ),
+            [Fate::Kept(0), Fate::Merged(1)]
+        );
+    }
+
+    #[test]
+    fn fates_costs_one_pass_over_the_fates_when_nothing_merged() {
+        // Settling a document whose paragraphs all kept their tags must not
+        // rescan every source fate for every settled paragraph: a long
+        // document read for edit paid N² comparisons here.
+        let count = 120_000;
+        let tags: Vec<Option<usize>> = (0..count).map(Some).collect();
+        let story = |_: usize| "body".to_string();
+        let text = |_: usize| "x".to_string();
+        let start = std::time::Instant::now();
+        let fates = fates(&tags, story, story, text, text, count);
+        assert!(fates.iter().all(|f| matches!(f, Fate::Kept(_))));
+        assert!(
+            start.elapsed().as_secs() < 5,
+            "merged-singleton reconciliation must stay linear: {:?}",
+            start.elapsed()
+        );
+    }
 }
diff --git a/src/markdown/view_stories.rs b/src/markdown/view_stories.rs
index edfec5b6..87e036e7 100644
--- a/src/markdown/view_stories.rs
+++ b/src/markdown/view_stories.rs
@@ -691,7 +691,28 @@ pub(super) fn section(entry: &str) -> Stories {
 /// header or footer. A `![…][dN]` or `![…][imageK]` token (or an older
 /// view's `![…](drawing:…)`) splices its materialized drawing from
 /// `drawings`.
+///
+/// A field result holds fields of its own, and each nesting level costs one
+/// `runs` frame: a story that nests deeper than [`MOST_FIELD_DEPTH`] is
+/// refused through `marks.error` instead of being materialized frame by
+/// frame.
 fn runs(text: &str, marks: &mut Marks<'_>, drawings: &HashMap<String, String>) -> String {
+    runs_at_depth(text, marks, drawings, 0)
+}
+
+/// The most field results one inside another a story's text may hold.
+/// Word's own fields nest a handful deep; a scalar nesting thousands is a
+/// stack bomb, not a document, and would overflow the conversion stack
+/// before any DOCX validation runs.
+const MOST_FIELD_DEPTH: usize = 20;
+
+/// [`runs`] already `depth` field results deep.
+fn runs_at_depth(
+    text: &str,
+    marks: &mut Marks<'_>,
+    drawings: &HashMap<String, String>,
+    depth: usize,
+) -> String {
     let mut out = String::new();
     let mut plain = String::new();
     let mut last_revision = Vec::new();
@@ -837,11 +858,28 @@ fn runs(text: &str, marks: &mut Marks<'_>, drawings: &HashMap<String, String>) -
             noid_parent.clear();
             flush(&mut plain, &mut out, false);
             match field {
-                Field::Coded(code, result) => out.push_str(&format!(
-                    "<w:fldSimple w:instr=\" {} \">{}</w:fldSimple>",
-                    super::xml::escape(code.trim()),
-                    runs(result, marks, drawings)
-                )),
+                Field::Coded(code, result) => {
+                    if depth >= MOST_FIELD_DEPTH {
+                        // The story is refused (the caller reads
+                        // `marks.error`); the field is still written as the
+                        // empty simple field Word accepts.
+                        marks.error.get_or_insert_with(|| {
+                            format!(
+                                "a header or footer nests field results more than {MOST_FIELD_DEPTH} deep"
+                            )
+                        });
+                        out.push_str(&format!(
+                            "<w:fldSimple w:instr=\" {} \"/>",
+                            super::xml::escape(code.trim())
+                        ));
+                    } else {
+                        out.push_str(&format!(
+                            "<w:fldSimple w:instr=\" {} \">{}</w:fldSimple>",
+                            super::xml::escape(code.trim()),
+                            runs_at_depth(result, marks, drawings, depth + 1)
+                        ));
+                    }
+                }
                 // No result: Word computes it (`PAGE` on every page).
                 Field::Bare(name) => {
                     out.push_str(&format!("<w:fldSimple w:instr=\" {name} \"/>"));
@@ -1936,4 +1974,68 @@ mod tests {
             "<w:r><w:t xml:space=\"preserve\">{Client Name}</w:t></w:r>"
         );
     }
+
+    /// The people every [`written_with_marks`] run writes as.
+    fn written_people() -> People {
+        People {
+            by_handle: HashMap::from([(
+                "AC".to_string(),
+                (
+                    "Ann Counsel".to_string(),
+                    "2026-10-01T09:00:00Z".to_string(),
+                ),
+            )]),
+            author: "Redline".into(),
+            date: "2026-01-01T00:00:00Z".into(),
+            ..People::default()
+        }
+    }
+
+    /// `text` as runs, with the marks the run wrote, so a refusal is
+    /// observable.
+    fn written_with_marks<'p>(text: &str, people: &'p People) -> (String, Marks<'p>) {
+        let mut marks = Marks {
+            people,
+            next_id: 100,
+            opens: HashMap::new(),
+            error: None,
+        };
+        let out = runs(text, &mut marks, &HashMap::new());
+        (out, marks)
+    }
+
+    /// `{PAGE}` repeated `depth` times is one field result inside another,
+    /// that deep.
+    fn nested_fields(depth: usize) -> String {
+        format!("{}x{}", "{PAGE}".repeat(depth), "{/}".repeat(depth))
+    }
+
+    #[test]
+    fn field_results_nested_past_the_budget_refuse_the_story() {
+        let people = written_people();
+        // Twenty levels is a document: every level materializes.
+        let (out, marks) = written_with_marks(&nested_fields(MOST_FIELD_DEPTH), &people);
+        assert_eq!(marks.error, None);
+        assert_eq!(out.matches("w:fldSimple").count(), MOST_FIELD_DEPTH * 2);
+        // One past the budget refuses the story instead of nesting further.
+        let (out, marks) = written_with_marks(&nested_fields(MOST_FIELD_DEPTH + 1), &people);
+        assert_eq!(
+            marks.error,
+            Some(format!(
+                "a header or footer nests field results more than {MOST_FIELD_DEPTH} deep"
+            ))
+        );
+        // What was written before the refusal is still closed markup.
+        assert_eq!(out.matches("w:fldSimple").count(), MOST_FIELD_DEPTH * 2 + 1);
+    }
+
+    #[test]
+    fn a_scalar_of_thousands_of_nested_fields_refuses_rather_than_recursing() {
+        // The red line for this test was a stack overflow: each nesting
+        // level was one more `runs` frame, and no limit stood before it.
+        let people = written_people();
+        let (out, marks) = written_with_marks(&nested_fields(50_000), &people);
+        assert!(marks.error.is_some());
+        assert_eq!(out.matches("w:fldSimple").count(), MOST_FIELD_DEPTH * 2 + 1);
+    }
 }
diff --git a/src/admission.rs b/src/admission.rs
index d5566100..cf492c2d 100644
--- a/src/admission.rs
+++ b/src/admission.rs
@@ -764,8 +764,10 @@ fn check_part_name(name: &str) -> Result<(), AdmissionError> {
 /// A part's text for scanning. Word writes UTF-8, but a part may be UTF-16
 /// with a byte-order mark (SharePoint's `customXml` items often are) and
 /// Word opens it, so it is transcoded; other bytes decode lossily, as the
-/// rest of the engine reads them.
-fn part_text(xml: &[u8]) -> Cow<'_, str> {
+/// rest of the engine reads them. Every consumer that reads a part the
+/// same way — admission, the privacy transformations, the leak check —
+/// decodes through here, so one encoding cannot slip past the others.
+pub(crate) fn part_text(xml: &[u8]) -> Cow<'_, str> {
     let utf16 = |rest: &[u8], unit: fn([u8; 2]) -> u16| {
         let units = rest.as_chunks::<2>().0.iter().map(|&pair| unit(pair));
         Cow::Owned(
diff --git a/src/scrub.rs b/src/scrub.rs
index 641d4272..b311971c 100644
--- a/src/scrub.rs
+++ b/src/scrub.rs
@@ -39,8 +39,13 @@ use crate::xmllinq::{Dom, NodeId, XName};
 const CORE: &str = "docProps/core.xml";
 const APP: &str = "docProps/app.xml";
 const CUSTOM: &str = "docProps/custom.xml";
+const CORE_REL: &str =
+    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
+const APP_REL: &str =
+    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
 const CUSTOM_REL: &str =
     "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
+const PEOPLE_REL: &str = "http://schemas.microsoft.com/office/2011/relationships/people";
 const CP: &str = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";
 const DC: &str = "http://purl.org/dc/elements/1.1/";
 const DCTERMS: &str = "http://purl.org/dc/terms/";
@@ -177,9 +182,47 @@ fn finding_counts(
     counts
 }
 
-/// One parsed XML part, written back only when changed.
+/// How a part's bytes spell their XML: plain (UTF-8, or anything else the
+/// engine reads lossily), or UTF-16 with a byte-order mark, which Word
+/// accepts (SharePoint writes `customXml` items that way).
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum Encoding {
+    Plain,
+    Utf16Le,
+    Utf16Be,
+}
+
+impl Encoding {
+    fn of(bytes: &[u8]) -> Self {
+        match bytes {
+            [0xFF, 0xFE, ..] => Self::Utf16Le,
+            [0xFE, 0xFF, ..] => Self::Utf16Be,
+            _ => Self::Plain,
+        }
+    }
+
+    /// `xml` written back the way the part spelled it, mark included.
+    fn encode(self, xml: &str) -> Vec<u8> {
+        match self {
+            Self::Plain => xml.as_bytes().to_vec(),
+            Self::Utf16Le => [0xFF, 0xFE]
+                .into_iter()
+                .chain(xml.encode_utf16().flat_map(u16::to_le_bytes))
+                .collect(),
+            Self::Utf16Be => [0xFE, 0xFF]
+                .into_iter()
+                .chain(xml.encode_utf16().flat_map(u16::to_be_bytes))
+                .collect(),
+        }
+    }
+}
+
+/// One parsed XML part, written back only when changed. A UTF-16 part is
+/// decoded for the transformation and written back as UTF-16, so a scrub
+/// never silently re-encodes a part Word reads.
 struct Part {
     name: String,
+    encoding: Encoding,
     dom: Dom,
     doc: NodeId,
     root: NodeId,
@@ -187,12 +230,15 @@ struct Part {
 
 impl Part {
     fn load(pkg: &PartFs, name: &str) -> Option<Self> {
-        let xml = pkg.part_string(name)?;
+        let bytes = pkg.part_bytes(name)?;
+        let encoding = Encoding::of(bytes);
+        let xml = crate::admission::part_text(bytes);
         let mut dom = Dom::new();
         let doc = dom.parse_xdocument(&xml);
         let root = dom.root(doc)?;
         Some(Self {
             name: name.to_string(),
+            encoding,
             dom,
             doc,
             root,
@@ -202,7 +248,8 @@ impl Part {
     fn store(self, pkg: &mut PartFs) {
         pkg.set_part(
             &self.name,
-            self.dom.serialize_document(self.doc).into_bytes(),
+            self.encoding
+                .encode(&self.dom.serialize_document(self.doc)),
         );
     }
 }
@@ -215,6 +262,48 @@ fn word_parts(pkg: &PartFs) -> Vec<String> {
         .collect()
 }
 
+/// The parts `rel_type` reaches from the package's own relationships,
+/// resolved to part names; an external target names no part. Word reads
+/// the property parts through these relationships, wherever the package
+/// put them, so a package that relocates `docProps/` is still scrubbed.
+fn package_related_parts(pkg: &PartFs, rel_type: &str) -> Vec<String> {
+    pkg.package_relationships()
+        .get_all_by_type(rel_type)
+        .into_iter()
+        .filter(|rel| rel.target_mode.as_deref() != Some("External"))
+        .map(|rel| pkg.resolve_rel_target("", &rel.target))
+        .collect()
+}
+
+/// `rel_type`'s targets plus the conventional `name`, when the package
+/// holds it, without duplicates: the relationships are authoritative, the
+/// conventional name covers a package that names none (and an orphan at
+/// the conventional name keeps none of its data either).
+fn part_candidates(pkg: &PartFs, rel_type: &str, name: &str) -> Vec<String> {
+    let mut names = package_related_parts(pkg, rel_type);
+    if pkg.part_bytes(name).is_some() {
+        names.push(name.to_string());
+    }
+    names.sort();
+    names.dedup();
+    names
+}
+
+/// The people part, wherever its relationship from the main document puts
+/// it; the conventional name when the document names none.
+fn people_part(pkg: &PartFs) -> String {
+    if let Some(main) = pkg.main_document_part()
+        && let Some(rels) = pkg.read_rels_for(&main)
+        && let Some(rel) = rels
+            .items
+            .iter()
+            .find(|r| r.rel_type == PEOPLE_REL && r.target_mode.as_deref() != Some("External"))
+    {
+        return pkg.resolve_rel_target(&main, &rel.target);
+    }
+    "word/people.xml".to_string()
+}
+
 /// Drop every comment reference and range, then the comment parts and
 /// `people.xml` that no comment needs any more.
 fn remove_comments(pkg: &mut PartFs) {
@@ -264,14 +353,16 @@ fn initials(alias: &str) -> String {
         .collect()
 }
 
-/// Every author in the `word/` parts becomes `alias`; `people.xml` keeps
-/// one person, the alias, without presence information.
+/// Every author in the `word/` parts becomes `alias`; the people part,
+/// wherever its relationship put it, keeps one person, the alias, without
+/// presence information.
 fn alias_authors(pkg: &mut PartFs, alias: &str) {
     let author = W::author();
     let initials_name = W::name("initials");
     let short = initials(alias);
+    let people_name = people_part(pkg);
     for name in word_parts(pkg) {
-        if name == "word/people.xml" {
+        if name == people_name {
             continue;
         }
         let Some(mut part) = Part::load(pkg, &name) else {
@@ -301,7 +392,7 @@ fn alias_authors(pkg: &mut PartFs, alias: &str) {
             part.store(pkg);
         }
     }
-    let Some(mut people) = Part::load(pkg, "word/people.xml") else {
+    let Some(mut people) = Part::load(pkg, &people_name) else {
         return;
     };
     let person = W15::name("person");
@@ -353,41 +444,50 @@ fn remove_children(part: &mut Part, names: &[XName]) -> bool {
 }
 
 /// Drop the people and dates of the core and extended properties, and the
-/// custom properties part.
+/// custom properties part. Word finds these parts through the package's
+/// relationships, wherever the package put them: a valid package that
+/// relocates `docProps/` is scrubbed at its actual part names, and the
+/// conventional names still cover a package that names no relationship.
 fn scrub_properties(pkg: &mut PartFs) {
-    if let Some(mut core) = Part::load(pkg, CORE) {
-        let names = [
-            XName::get("creator", DC),
-            XName::get("lastModifiedBy", CP),
-            XName::get("revision", CP),
-            XName::get("lastPrinted", CP),
-            XName::get("created", DCTERMS),
-            XName::get("modified", DCTERMS),
-        ];
-        if remove_children(&mut core, &names) {
-            core.store(pkg);
+    for name in part_candidates(pkg, CORE_REL, CORE) {
+        if let Some(mut core) = Part::load(pkg, &name) {
+            let names = [
+                XName::get("creator", DC),
+                XName::get("lastModifiedBy", CP),
+                XName::get("revision", CP),
+                XName::get("lastPrinted", CP),
+                XName::get("created", DCTERMS),
+                XName::get("modified", DCTERMS),
+            ];
+            if remove_children(&mut core, &names) {
+                core.store(pkg);
+            }
         }
     }
-    if let Some(mut app) = Part::load(pkg, APP) {
-        let names = [
-            XName::get("Manager", EXTENDED),
-            XName::get("Company", EXTENDED),
-        ];
-        if remove_children(&mut app, &names) {
-            app.store(pkg);
+    for name in part_candidates(pkg, APP_REL, APP) {
+        if let Some(mut app) = Part::load(pkg, &name) {
+            let names = [
+                XName::get("Manager", EXTENDED),
+                XName::get("Company", EXTENDED),
+            ];
+            if remove_children(&mut app, &names) {
+                app.store(pkg);
+            }
         }
     }
-    if pkg.part_bytes(CUSTOM).is_some() {
-        pkg.remove_part(CUSTOM);
-        pkg.remove_package_relationships_by_type(CUSTOM_REL);
-        pkg.remove_content_type_override(&format!("/{CUSTOM}"));
+    for name in part_candidates(pkg, CUSTOM_REL, CUSTOM) {
+        pkg.remove_part(&name);
+        pkg.remove_content_type_override(&format!("/{name}"));
     }
+    pkg.remove_package_relationships_by_type(CUSTOM_REL);
 }
 
 /// The parts of `docx` where `needle` still occurs, sorted: in a text node,
 /// an attribute value that can hold text, an XML comment, the joined run
 /// text of a paragraph (a text split across runs), or the raw bytes of a
-/// part that is not XML (UTF-8 or UTF-16LE). An empty needle occurs nowhere.
+/// part that is not XML (UTF-8 or UTF-16LE). An XML part is decoded the
+/// way admission decodes it, BOM-marked UTF-16 included. An empty needle
+/// occurs nowhere.
 /// An unreadable archive or entry is a finding: an incomplete scan cannot
 /// certify a redaction.
 ///
@@ -466,7 +566,10 @@ pub(crate) fn leaks_within(
         }
         let xml = name.ends_with(".xml") || name.ends_with(".rels");
         let hit = if xml {
-            xml_holds(&String::from_utf8_lossy(&bytes), needle)
+            // The same BOM-aware decode admission uses: a UTF-16 XML part
+            // holds real text, not NUL-interleaved bytes, and a redaction
+            // of that text must find its copy here.
+            xml_holds(&crate::admission::part_text(&bytes), needle)
         } else {
             contains(&bytes, needle.as_bytes()) || contains(&bytes, &utf16)
         };
@@ -715,6 +818,39 @@ mod public_memory_leak_boundary_tests {
             assert_eq!(snapshot(&PartFs::open(&bytes).unwrap()), original);
         }
     }
+    #[test]
+    fn a_needle_in_utf16_xml_is_a_leak() {
+        // A UTF-16 XML part holds real text: reading it as lossy UTF-8
+        // hid it behind NULs, and a redaction certified a document whose
+        // secret still sat in `customXml`.
+        for (name, mark, unit) in [
+            (
+                "customXml/utf16le.xml",
+                [0xFF_u8, 0xFE],
+                u16::to_le_bytes as fn(u16) -> [u8; 2],
+            ),
+            (
+                "customXml/utf16be.xml",
+                [0xFE_u8, 0xFF],
+                u16::to_be_bytes as fn(u16) -> [u8; 2],
+            ),
+        ] {
+            let mut pkg = package("<w:p><w:r><w:t>Independent body payload</w:t></w:r></w:p>");
+            let xml = "<metadata><note>PrivateReviewer</note></metadata>";
+            let bytes: Vec<u8> = mark
+                .into_iter()
+                .chain(xml.encode_utf16().flat_map(unit))
+                .collect();
+            pkg.set_part(name, bytes);
+            let bytes = pkg.to_zip().unwrap();
+            assert_eq!(
+                leaks(&bytes, "PrivateReviewer"),
+                [name],
+                "{name} must be read as the UTF-16 it declares"
+            );
+        }
+    }
+
     #[test]
     fn decoded_entities_and_split_word_runs_are_secrets_but_numeric_ids_are_not() {
         let body = "<w:p><w:pPr><w:spacing w:before='1234' w:after='-1234'/></w:pPr><w:r><w:t>Jane </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>Doe &amp; </w:t></w:r><w:r><w:t>Associates</w:t></w:r></w:p>";
diff --git a/tests/scrub_and_redact.rs b/tests/scrub_and_redact.rs
index 2a16d5d8..bea0a603 100644
--- a/tests/scrub_and_redact.rs
+++ b/tests/scrub_and_redact.rs
@@ -793,3 +793,120 @@ fn redact_revision_refusal_never_reports_the_document_copy() {
         Some("UNSUPPORTED_STRUCTURE"),
     );
 }
+
+const CORE_REL: &str =
+    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";
+const APP_REL: &str =
+    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties";
+const CUSTOM_PROPS_REL: &str =
+    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
+const CORE_CT: &str = "application/vnd.openxmlformats-package.core-properties+xml";
+
+/// [`identifying_document`] with its property and people parts moved to
+/// names no conventional path reaches, located only by the relationships
+/// a valid OPC package carries.
+fn relocated_identifying_document() -> Vec<u8> {
+    let mut pkg = PartFs::open(&identifying_document()).unwrap();
+    for (from, to, rel) in [
+        ("docProps/core.xml", "metadata/core.xml", CORE_REL),
+        ("docProps/app.xml", "metadata/app.xml", APP_REL),
+        ("docProps/custom.xml", "metadata/custom.xml", CUSTOM_PROPS_REL),
+    ] {
+        let bytes = pkg.part_bytes(from).unwrap().to_vec();
+        let ct = pkg.content_type_for(from).unwrap();
+        pkg.remove_part(from);
+        pkg.remove_content_type_override(&format!("/{from}"));
+        pkg.remove_package_relationships_by_type(rel);
+        pkg.set_part(to, bytes);
+        pkg.add_content_type_override(&format!("/{to}"), &ct);
+        pkg.add_package_relationship(rel, &format!("/{to}"));
+    }
+    // The people part hangs off the main document's relationship.
+    let main = pkg.main_document_part().unwrap();
+    let bytes = pkg.part_bytes("word/people.xml").unwrap().to_vec();
+    pkg.remove_part("word/people.xml");
+    pkg.remove_content_type_override("/word/people.xml");
+    pkg.remove_relationships_by_type(&main, PEOPLE_REL);
+    pkg.set_part("metadata/people.xml", bytes);
+    pkg.add_content_type_override("/metadata/people.xml", PEOPLE_CT);
+    pkg.add_document_relationship(&main, PEOPLE_REL, "/metadata/people.xml");
+    pkg.to_zip().unwrap()
+}
+
+#[test]
+fn scrub_finds_property_and_people_parts_through_their_relationships() {
+    let source = relocated_identifying_document();
+    assert_word_valid_package(&source);
+    let out = scrub(&source, &ScrubOptions::default()).unwrap();
+    assert_word_valid_package(&out);
+    let bytes = all_bytes(&out);
+    for gone in [
+        "Jane Secret",
+        "jane@secret.example",
+        "Secret Corp",
+        "<cp:revision>",
+        "<dcterms:created",
+    ] {
+        assert!(
+            !contains(&bytes, gone),
+            "{gone:?} survives relocation: run `unzip -p` on the output"
+        );
+    }
+    // The custom properties part is gone with its relationship, wherever
+    // the package put it.
+    let names = part_names(&out);
+    assert!(
+        !names.iter().any(|n| n == "metadata/custom.xml"),
+        "custom survives"
+    );
+    let rels = part_string(&out, "_rels/.rels").unwrap();
+    assert!(!rels.contains("custom-properties"), "{rels}");
+    // The relocated people part keeps one person, the alias, without
+    // presence information.
+    let people = part_string(&out, "metadata/people.xml").unwrap();
+    assert!(
+        people.contains("w15:author=\"Author\"") && !people.contains("presenceInfo"),
+        "{people}"
+    );
+    // The relocated core keeps what says nothing about a person.
+    let core = part_string(&out, "metadata/core.xml").unwrap();
+    assert!(!core.contains("<dc:creator>") && core.contains("Deal memo"), "{core}");
+}
+
+#[test]
+fn utf16_property_parts_are_scrubbed_and_stay_utf16() {
+    // Word opens UTF-16 parts (SharePoint writes customXml items that
+    // way); reading them as lossy UTF-8 left their identities in place.
+    for (mark, unit) in [
+        ([0xFF_u8, 0xFE], u16::to_le_bytes as fn(u16) -> [u8; 2]),
+        ([0xFE_u8, 0xFF], u16::to_be_bytes as fn(u16) -> [u8; 2]),
+    ] {
+        let core = r#"<?xml version="1.0" encoding="UTF-16" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Deal memo</dc:title><dc:creator>Jane Secret</dc:creator><cp:lastModifiedBy>Jane Secret</cp:lastModifiedBy></cp:coreProperties>"#;
+        let mut pkg = PartFs::open(&docx(&para("Kept text."))).unwrap();
+        let bytes: Vec<u8> = mark
+            .into_iter()
+            .chain(core.encode_utf16().flat_map(unit))
+            .collect();
+        pkg.set_part("docProps/core.xml", bytes);
+        pkg.add_content_type_override("/docProps/core.xml", CORE_CT);
+        pkg.add_package_relationship(CORE_REL, "docProps/core.xml");
+        let source = pkg.to_zip().unwrap();
+        let out = scrub(&source, &ScrubOptions::default()).unwrap();
+        assert_word_valid_package(&out);
+        let raw = PartFs::open(&out).unwrap();
+        let raw = raw.part_bytes("docProps/core.xml").unwrap();
+        assert_eq!(&raw[..2], &mark, "the part keeps its byte-order mark");
+        let text: String = char::decode_utf16(raw[2..].chunks_exact(2).map(|pair| {
+            let pair: [u8; 2] = pair.try_into().unwrap();
+            if mark == [0xFF, 0xFE] {
+                u16::from_le_bytes(pair)
+            } else {
+                u16::from_be_bytes(pair)
+            }
+        }))
+        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
+        .collect();
+        assert!(!text.contains("Jane Secret"), "{text}");
+        assert!(text.contains("Deal memo"), "{text}");
+    }
+}
```
