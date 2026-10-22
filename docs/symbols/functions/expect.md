# Functions and methods in `src/expect.rs`

24 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `is_false`  (L192-194)
`fn is_false(value: &bool) -> bool`

- **Does:** Marker fields the receipt never prints: a separately supplied file is not an independence claim, and no deterministic profile completes the caller's task.
- **Calls:** none
- **Called by:** none found in src

#### `<RefusalReason as std::fmt::Display>::fmt`  (L198-210, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.write_str`
- **Called by:** none found in src

#### `ExpectationReport::blocking_failure`  (L277-281, pub)
`pub fn blocking_failure(&self) -> Option<&CheckOutcome>`

- **Does:** The first check that blocks publication: a `Failed` predicate, or any `Refused` check (missing, ambiguous, stale, unsupported or unopenable target).
- **Calls (std/external/unresolved):** `.find`, `.iter`
- **Macros:** `matches!`
- **Called by:** `evaluate_expectations` (bin/jubarte/edit_typed.rs:200) (?); `ExpectationReport::has_blocking_failure` (expect.rs:285)

#### `ExpectationReport::has_blocking_failure`  (L285-287, pub)
`pub fn has_blocking_failure(&self) -> bool`

- **Does:** Whether any typed check blocks publication (see [`Self::blocking_failure`]).
- **Calls (crate):** `ExpectationReport::blocking_failure` (expect.rs:277) L286
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** none found in src

#### `CorrespondenceEntry::one_to_one`  (L292-298, pub)
`pub fn one_to_one(short_id: impl Into<String>, output: usize) -> Self`

- **Does:** A 1:1 mapping of one source paragraph to one output paragraph.
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `vec!`
- **Called by:** `expectation_correspondence` (edit/v2_apply.rs:371); `identity_correspondence` (expect.rs:536)

#### `CorrespondenceEntry::merged`  (L301-307, pub)
`pub fn merged(short_id: impl Into<String>, outputs: &[usize]) -> Self`

- **Does:** A merged mapping (one source paragraph → many output paragraphs).
- **Calls (std/external/unresolved):** `.into`, `.to_vec`
- **Called by:** `expectation_correspondence` (edit/v2_apply.rs:371)

#### `CorrespondenceEntry::split`  (L311-317, pub)
`pub fn split(short_id: impl Into<String>) -> Self`

- **Does:** A split/vanished mapping (one source paragraph → no output paragraph).
- **Calls (std/external/unresolved):** `.into`, `Vec::new`
- **Called by:** `expectation_correspondence` (edit/v2_apply.rs:371)

#### `SourceToOutputCorrespondence::from_entries`  (L343-354, pub)
`pub fn from_entries(entries: Vec<CorrespondenceEntry>) -> Result<Self, String>`

- **Does:** Build a correspondence from explicit entries.
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.insert`, `.is_some`, `.iter`, `.len`, `Err`, `Ok`, `std::collections::HashMap::with_capacity`
- **Macros:** `format!`
- **Called by:** `expectation_correspondence` (edit/v2_apply.rs:371); `identity_correspondence` (expect.rs:536)

#### `SourceToOutputCorrespondence::entries`  (L357-359, pub)
`pub fn entries(&self) -> &[CorrespondenceEntry]`

- **Does:** The correspondence entries.
- **Calls:** none
- **Called by:** `rebind_to_output` (expect.rs:1008) (?)

#### `SourceToOutputCorrespondence::lookup`  (L362-366, pub)
`pub fn lookup(&self, short_id: &str) -> Option<&CorrespondenceEntry>`

- **Does:** Look up an entry by source short id.
- **Calls (std/external/unresolved):** `.get`, `.map`
- **Called by:** `evaluate_with_receipt` (expect.rs:590); `rebind_to_output` (expect.rs:1008)

#### `parse_expectations`  (L410-492, pub)
`pub fn parse_expectations(json: &str) -> Result<ExpectationsV1, String>`

- **Does:** Parse the closed `schema_version: 1` expectations document.
- **Calls (crate):** `is_lowercase_hex_64` (expect.rs:496) L425; `is_valid_key` (expect.rs:505) L446
- **Calls (std/external/unresolved):** `.as_str`, `.insert`, `.is_empty`, `.map_err`, `.to_string`, `Err`, `Ok`, `serde_json::from_str`, `std::collections::BTreeSet::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `run` (bin/jubarte/check.rs:125) (?); `evaluate_expectations` (bin/jubarte/edit_typed.rs:200) (?)

#### `is_lowercase_hex_64`  (L496-502)
`fn is_lowercase_hex_64(s: &str) -> bool`

- **Does:** True iff `s` is exactly 64 bytes of `[0-9a-f]` (no trailing newline, no uppercase).
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.contains`, `.is_ascii_digit`, `.iter`, `.len`
- **Called by:** `parse_expectations` (expect.rs:410)

#### `is_valid_key`  (L505-517)
`fn is_valid_key(s: &str) -> bool`

- **Does:** True iff `s` matches the schema's key regex `^[A-Za-z][A-Za-z0-9_.-]*$`.
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.is_ascii_alphabetic`, `.is_ascii_alphanumeric`, `.is_empty`, `.iter`
- **Called by:** `parse_expectations` (expect.rs:410)

#### `content_sha256`  (L520-524, pub)
`pub fn content_sha256(bytes: &[u8]) -> String`

- **Does:** SHA-256 of arbitrary content, lowercase hex.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `Sha256::digest`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte/check.rs:125); `evaluate_expectations` (bin/jubarte/edit_typed.rs:200); `evaluate_with_receipt` (expect.rs:590); `rebind_to_output` (expect.rs:1008)

#### `identity_correspondence`  (L536-548, pub)
`pub fn identity_correspondence( source_bytes: &[u8], ) -> Result<SourceToOutputCorrespondence, String>`

- **Does:** Build the identity correspondence: the inspected file IS the reopened output.
- **Calls (crate):** `projected_paragraphs` (expect.rs:554) L540; `CorrespondenceEntry::one_to_one` (expect.rs:292) L544; `SourceToOutputCorrespondence::from_entries` (expect.rs:343) L546
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.expect`, `.into_iter`, `.map_err`, `.map`, `Ok`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte/check.rs:125) (?); `apply_bound` (edit/v2_apply.rs:121); `rebind_to_output` (expect.rs:1008)

#### `projected_paragraphs`  (L554-562)
`fn projected_paragraphs( bytes: &[u8], ) -> Result<Vec<crate::inspect::Paragraph>, crate::inspect::InspectError>`

- **Does:** The paragraphs a expectations run judges, in the fixed projection order: the body's paragraphs in body order, then the story paragraphs (`header1`, `header1.p1`, ...) story by story — the same order `inspect::markdown` prints.
- **Calls (crate):** `paragraphs` (inspect.rs:312) L557; `stories` (inspect.rs:353) L558
- **Calls (std/external/unresolved):** `.extend`, `Ok`
- **Called by:** `identity_correspondence` (expect.rs:536); `evaluate_with_receipt` (expect.rs:590)

#### `evaluate`  (L567-574, pub)
`pub fn evaluate( spec: &ExpectationsV1, source: &[u8], output: &[u8], correspondence: &SourceToOutputCorrespondence, ) -> ExpectationReport`

- **Does:** Evaluate typed checks on the reopened output, given the bytes the correspondence was built from, the output bytes, and the correspondence.
- **Calls (crate):** `evaluate_with_receipt` (expect.rs:590) L573
- **Calls (std/external/unresolved):** `ReceiptMeta::default`
- **Called by:** none found in src

#### `evaluate_with_receipt`  (L590-803, pub)
`pub fn evaluate_with_receipt( spec: &ExpectationsV1, source: &[u8], output: &[u8], correspondence: &SourceToOutputCorrespondence, meta: ReceiptMeta, ) -> ExpectationReport`

- **Does:** As [`evaluate`], but also records receipt metadata: the file path, the file digest, and any caller-declared provenance.
- **Calls (crate):** `projected_paragraphs` (expect.rs:554) L607; `assemble_report` (expect.rs:809) L626,802; `content_sha256` (expect.rs:520) L629; `native_key_inventory` (expect.rs:919) L634; `is_paragraph_target` (expect.rs:971) L668; `ids_name_one_entity` (expect.rs:875) L685; `SourceToOutputCorrespondence::lookup` (expect.rs:362) L699
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.get`, `.insert`, `.is_empty`, `.len`, `.push`, `.to_string`, `Some`, `Vec::new`, `std::collections::BTreeSet::new`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte/check.rs:125) (?); `evaluate_expectations` (bin/jubarte/edit_typed.rs:200) (?); `evaluate` (expect.rs:567)

#### `assemble_report`  (L809-857)
`fn assemble_report( spec: &ExpectationsV1, meta: ReceiptMeta, checks: Vec<CheckOutcome>, unassessed_requirements: Vec<String>, targets: Vec<String>, ) -> ExpectationReport`

- **Does:** Assemble the report and its receipt.
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.count`, `.filter`, `.iter`, `.len`, `Some`
- **Macros:** `matches!`
- **Called by:** `evaluate_with_receipt` (expect.rs:590)

#### `ids_name_one_entity`  (L875-908)
`fn ids_name_one_entity( short_id: &str, long_id: &str, native_keys: &std::collections::HashMap<String, String>, ) -> bool`

- **Does:** Whether a target's two ids name the same entity, judged through the one ids codec (`crate::ids`); there is no second parser.
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.collect`, `.get`, `.is_ok_and`, `.is_some_and`, `.parse`, `.splitn`
- **Called by:** `evaluate_with_receipt` (expect.rs:590)

#### `native_key_inventory`  (L919-941)
`fn native_key_inventory(source: &[u8]) -> std::collections::HashMap<String, String>`

- **Does:** The `w14:paraId` native key each source paragraph carries, keyed by the printed short id the correspondence carries (`p0`, `header1`, `header1.p2`).
- **Calls (crate):** `Opened::open` (inspect.rs:735) L921; `story_keys` (expect.rs:945) L924,932; `body_paragraph_nodes` (inspect.rs:988) L926; `Opened::story_parts` (inspect.rs:861) (by name) L930; `parse_part` (inspect.rs:926) L931; `story_paragraph_nodes` (inspect.rs:1178) L934
- **Calls (std/external/unresolved):** `std::collections::HashMap::new`
- **Called by:** `evaluate_with_receipt` (expect.rs:590)

#### `story_keys`  (L945-969)
`fn story_keys( dom: &crate::xmllinq::Dom, nodes: &[crate::xmllinq::NodeId], story: &str, out: &mut std::collections::HashMap<String, String>, )`

- **Does:** [`native_key_inventory`] for one story's paragraphs, in the same walk order [`crate::ids::paragraph_id`] numbers them in.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L954,962; `para_id` (edit/native_keys.rs:18) L965; `paragraph_id` (ids.rs:410) L966
- **Calls (std/external/unresolved):** `.and_then`, `.entry`, `.enumerate`, `.insert`, `.is_some_and`, `.iter`, `.or_default`, `.to_string`, `crate::namespaces::W14::name`, `std::collections::HashMap::new`
- **Called by:** `native_key_inventory` (expect.rs:919)

#### `is_paragraph_target`  (L971-989)
`fn is_paragraph_target(short_id: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_slice`, `.parse`, `Step::Paragraph`, `Story::Part`
- **Macros:** `matches!`
- **Called by:** `evaluate_with_receipt` (expect.rs:590)

#### `rebind_to_output`  (L1008-1066, pub)
`pub fn rebind_to_output( spec: &ExpectationsV1, correspondence: &SourceToOutputCorrespondence, output: &[u8], ) -> Option<ExpectationsV1>`

- **Does:** Build the output-bound copy of evaluated expectations (proposal §10: "The apply receipt may expose an explicitly output-bound copy of evaluated text expectations, with new references/hash, for later rechecking").
- **Calls (crate):** `identity_correspondence` (expect.rs:536) L1013; `.entries` (ambiguous: `Operations::entries` (capabilities.rs:210); `SourceToOutputCorrespondence::entries` (expect.rs:357)) L1016; `SourceToOutputCorrespondence::lookup` (expect.rs:362) L1037; `content_sha256` (expect.rs:520) L1061
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.clone`, `.get`, `.insert`, `.len`, `.ok`, `.parse`, `.push`, `Some`, `Vec::with_capacity`, `long_id_of`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `evaluate_expectations` (bin/jubarte/edit_typed.rs:200)
