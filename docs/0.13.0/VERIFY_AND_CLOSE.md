# Verify and close: one procedure per URGENT row and per cli-titled issue

Written 2026-10-19 from `URGENT_AND_CLI_ASSESSMENT.md`. The stack merge is PR #1278 (head `9dac2916`, the former stack top `fix/win-wrote-path-1244`).

## Rules for the agent running this

- One issue at a time, in the order below. Never bulk-close, never bulk-comment, never merge a PR other than the one step 0 names.
- Every close carries evidence in its comment: the command, its output or exit code, the commit it ran on. A close without evidence is reverted.
- All commands run from the repository root on a fresh `git fetch origin main && git checkout -B verify origin/main`. Cargo runs through the mbx shim; one Cargo process per checkout; `cargo nextest run` for tests.
- A step that fails means: do not close; post what failed; move to the next issue. Do not fix anything while verifying; file or comment instead.
- `jubarte` means `target/debug/jubarte` built from that checkout (`cargo build`). A Word check means the neurotic_docx_bench scripts, never a hand-rolled converter.
- Issue comments and bodies never carry a model identifier.

## Step 0: the stack merge PR

### PR #1278 · merge: the open PR stack into main
**Action:** MERGE with a merge commit (not squash, not rebase), once CI is green  
**Double-check, all must hold:**
1. `gh pr view 1278 --json mergeable,headRefOid` says `MERGEABLE` and head `9dac2916…`; no new commits were pushed to `merge/stack-2026-10-19`.
2. `gh pr checks 1278`: every CI workflow job (fmt + clippy, the three `test` jobs, coverage, MSRV, fuzz smoke, publish dry run, cargo-deny + REUSE, python bindings, convert-sweep, docs drift, Markdown round trip, adoption) is `pass`. Bot reviews (CodeRabbit, Cursor, Sourcery, Socket) may be neutral or skipped. A failing job: read its log; if the failure also exists on `origin/main` or on #1248's own run for the same head, note it and ask Arthur before merging; otherwise do not merge.
3. `gh pr ready 1278` then `gh pr merge 1278 --merge --subject 'merge: the open PR stack (#1085 through #1248) into main (#1278)'`.
4. After the merge: `git fetch origin main && git merge-base --is-ancestor 9dac2916 origin/main` exits 0. Build `origin/main`, run `target/debug/jubarte --help` and `cargo nextest run --test input_admission --test cli_plan_record`.
5. Do not close the 93 stacked PRs here. Each closes through its issue's procedure below, or stays open as history until Arthur says otherwise.

## Part A: URGENT audit rows (47)

Order: done rows, stack-fixed rows, the two open rows, the partly-done rows, the umbrellas, the tracker.

### #655 · audit(N03): Do not import the handwritten docx2typed ZIP patcher [Urgent]
**Action:** CLOSE as already done  
**Double-check, all must hold:**
1. `grep -rn 'docx2typed' src/ Cargo.toml Cargo.lock` finds no dependency, module or import of the handwritten patcher.
2. `cargo tree` lists no `docx2typed` crate.
**Close only if** every step holds. **Comment to post:** `Nothing imports the docx2typed patcher (grep and cargo tree clean). Verified 2026-10-19 at <origin/main sha>.`
**Otherwise:** leave open and post what failed.

### #671 · audit(ZIP-03): Temporary cargo-deny exception policy [Urgent]
**Action:** CLOSE as already done  
**Double-check, all must hold:**
1. `deny.toml` (or `.cargo/deny.toml`): every advisory exception carries an expiry or a linked issue; none is older than its stated review date.
2. `cargo deny check advisories` passes on `origin/main`.
3. The policy text the issue asks for exists (grep `exception` in `deny.toml` comments or `docs/SECURITY.md`).
**Close only if** every step holds. **Comment to post:** `Exception policy present and cargo-deny clean. Verified 2026-10-19 at <origin/main sha>.`
**Otherwise:** leave open and post what failed.

### #612 · audit(D13): Atomic filesystem publication [Urgent]
**Action:** CLOSE as fixed by PR #1122 (`fix/audit-publish`, head `e55459dd96`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor e55459dd96 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_output --test cli_plan_record` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1122, on main via #1278 (merge-base check on e55459dd96). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #666 · audit(D35): Export path confinement [Urgent]
**Action:** CLOSE as fixed by PR #1122 (`fix/audit-publish`, head `e55459dd96`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor e55459dd96 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_output --test cli_plan_record` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1122, on main via #1278 (merge-base check on e55459dd96). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #645 · audit(ZIP-14): Actual decompression limits for every API [Urgent]
**Action:** CLOSE as fixed by PR #1233 (`fix/zip-bounded-645`, head `fd9aeefe9f`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor fd9aeefe9f origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1233, on main via #1278 (merge-base check on fd9aeefe9f). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #649 · audit(ZIP-13): Entry and metadata counts before indexing [Urgent]
**Action:** CLOSE as fixed by PR #1233 (`fix/zip-bounded-645`, head `fd9aeefe9f`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor fd9aeefe9f origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1233, on main via #1278 (merge-base check on fd9aeefe9f). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #670 · audit(ZIP-32): Low-allocation raw ZIP preflight [Urgent]
**Action:** CLOSE as fixed by PR #1233 (`fix/zip-bounded-645`, head `fd9aeefe9f`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor fd9aeefe9f origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1233, on main via #1278 (merge-base check on fd9aeefe9f). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #644 · audit(ZIP-17): Structural ZIP consistency, overlap and CRC [Urgent]
**Action:** CLOSE as fixed by PR #1241 (`fix/zip-struct-644`, head `5507bc6b68`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 5507bc6b68 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1241, on main via #1278 (merge-base check on 5507bc6b68). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #639 · audit(ZIP-16): Symlink and special file metadata [Urgent]
**Action:** CLOSE as fixed by PR #1242 (`fix/special-entries-639`, head `c30b7bc42b`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor c30b7bc42b origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1242, on main via #1278 (merge-base check on c30b7bc42b). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #617 · audit(J05): Capability-scoped Markdown asset loading [Urgent]
**Action:** CLOSE as fixed by PR #1238 (`fix/md-assets-617`, head `81239333a0`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 81239333a0 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test markdown_cli` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1238, on main via #1278 (merge-base check on 81239333a0). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #650 · audit(N01): Jubarte custom DOM has quadratic unique-attribute insertion [Urgent]
**Action:** CLOSE as fixed by PR #1246 (`fix/attr-caps-650`, head `b2b69e76fb`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor b2b69e76fb origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission --test m_validity_ring1` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1246, on main via #1278 (merge-base check on b2b69e76fb). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #594 · audit(ZIP-01): Advisory-driven ZIP dependency remediation [Urgent]
**Action:** CLOSE as fixed by PR #1237 (`fix/zip-cloak-594`, head `dde48e6034`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor dde48e6034 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1237, on main via #1278 (merge-base check on dde48e6034). Gates: <gate → file::test>. Verified 2026-10-19.` Then do the same for #605 (its advisory-inventory half rides on this PR).
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #604 · audit(ZIP-10): ZIP32/ZIP64 boundaries and integer safety [Urgent]
**Action:** CLOSE as fixed by PR #1247 (`fix/zip64-604`, head `888cfca454`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 888cfca454 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test input_admission --test m8_strict --test parity_surface_lossless_linq` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
4. `grep -n 'max_entries' src/strict_translation.rs`: the `_within` variant enforces the entry budget (option C of the API review). If it does not, file a follow-up issue naming `strict_to_transitional_docx_within` and still close #604 only if the ZIP32 planning gate (`zip32_plan` refuses ≥65,535 entries and ≥0xFFFFFFFF sizes/offsets, `tests/input_admission.rs`) holds.
**Close only if** every step holds. **Comment to post:** `Fixed by #1247, on main via #1278 (merge-base check on 888cfca454). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #640 · audit(ZIP-12): Raw filename decoding and Unicode extra fields [Urgent]
**Action:** VERIFY and keep open (open, size M)  
**Double-check:**
1. Build a probe zip whose central-directory name is `word/document.xml` but whose InfoZIP Unicode Path extra field (id 0x7075) names `word/evil.xml` (python `zipfile` plus a hand-written extra field).
2. `jubarte validate probe.docx` and `jubarte read probe.docx` must refuse with a named code or report the mismatch. Silent acceptance of either name is the defect.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; keep the [Urgent] tier; schedule after #600.

### #592 · audit(ZIP-27): Production update authenticity [Urgent]
**Action:** VERIFY and keep open (open; the code is ready, the signing key was never rolled out)  
**Double-check:**
1. `grep -n 'signature_required' src/update.rs` and its tests show the check exists.
2. `gh release view --json assets` on the latest release: no `.sig`/`.minisig` asset, so `jubarte self-update` still installs unsigned binaries.
3. This row is Arthur's: generate the key, embed the public key, sign the next release. Comment exactly that; do not attempt the rollout.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; ask Arthur in the comment.

### #668 · audit(ZIP-19): Bounded work and compression-ratio heuristics [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY — actual byte caps hold on admit routes; no aggregate cap on markdown route; no repeated-work budget anywhere; size M
**Opinion:** Keep the Markdown-route aggregate cap here; move the repeated-work budget to #651 so one row owns it.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #657 · audit(ZIP-18): Compression and encryption attack surface [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size S
**Opinion:** Finish it: refusing encrypted entries and unsupported compression methods with a named code is ring-1-sized.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #600 · audit(ZIP-15): Duplicate ZIP and OPC identities [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size M
**Opinion:** Keep. #1237 fixed the two-header mismatch; duplicate central-directory names and case-folded OPC part collisions remain. Depends on #646.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #646 · audit(ZIP-11): Strict OPC part-name policy [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size M
**Opinion:** Keep; precondition for #600.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #643 · audit(ZIP-47): Strict XML before package metadata interpretation [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size M/L
**Opinion:** Keep, narrowed to the `[Content_Types].xml` and `.rels` readers; the main DOM is already strict.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #647 · audit(ZIP-43): Hostile archive regression corpus [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size L
**Opinion:** Downgrade from [Urgent] to 'gate for the ZIP batch': a corpus is evidence, not a fix. Collect the probes the ZIP PRs already ship into one directory plus one walking test.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #651 · audit(D17): Bounded XML and semantic work [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size L
**Opinion:** Keep [Urgent]: the remaining bomb row (repeated work, live memory); #722 and #668 both point here.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #658 · audit(N02): Namespace map cloning multiplies retained memory and work [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY; size M
**Opinion:** Downgrade: performance, not safety, unless a measurement shows retained memory above budget. Ask for the measurement in the comment.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #656 · audit(ZIP-28): Updater ZIP extraction confinement and budgets [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (confinement ✓, download/inflation caps ✗, duplicates ✗, signatures ✗); size M–L
**Opinion:** Finish the download and inflation caps and the duplicate check here; signatures stay with #592.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #662 · audit(DX02): Independent native candidate verification before publication [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (gates 1 and intended-edits ✓; per-part changed/unchanged receipt ✗); size M
**Opinion:** The missing per-part changed/unchanged receipt is Plan 3 of the 0.13 plan (#1253): propose folding there.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #605 · audit(ZIP-02): Correct advisory mapping and reachability [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY — CI graph gates + reachability hold; advisory inventory stale in 3 places, new critical advisory unrecorded; size S. **the advisory inventory part rides on #594**
**Opinion:** Update the three stale inventory places and record the new advisory; close together with #594.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #654 · audit(ZIP-44): Clean dependency gates [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY — advisories gated on all 5 graphs; ZIP-drift gating is warn-only/orphaned; size S
**Opinion:** Make the ZIP-drift check fail CI or delete it; a warn-only gate is noise.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #660 · audit(J01): Strict Markdown edit completion [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (gate 4 holds; 1 partial; 2, 3 fail — a test pins the anti-gate); size M
**Opinion:** Keep [Urgent]: the 'round trip silently drops an edit' class. The test pinning the anti-gate is flipped, not deleted.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #661 · audit(D16): Truthful completeness and omission reporting [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (gate 2 + cursor half of 3 hold; 1, 3-first-half, 4 fail); size L
**Opinion:** Keep as the engine owner; cli #954 is the CLI-surface owner. Do not do it twice.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #667 · audit(D18): Configurable text-output limits [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (for-edit route + sot-sync obligation DONE; read/MCP/snapshot/changes/markdown unbounded; gate 4 partial); size M
**Opinion:** Keep. Copy the for-edit route's shape to read, snapshot, changes and markdown; cli #989 folds here.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #672 · audit(D20): Bound formula/image/PDF asset generation [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (gates 2, 4 fail; 1, 3 hold); size M
**Opinion:** Keep; urgent only for the WASM and MCP exposure.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #616 · audit(N13): Typed and bounded redaction leak-scan completeness [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (gates 1, 2, 4b fail; 3, 4a hold); size M
**Opinion:** Decide once: complete typed scan, or the docs say scrub is not redaction (cli #977). Propose the second and the downgrade.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #669 · audit(N12): Bound the existing legacy DOC reader and report conversion losses [Urgent]
**Action:** VERIFY the remaining gates, keep open, rescope  
**#1216 triage (2026-10-18):** triage: PARTLY (gates 2, 3 fail; 1 holds, 4 partly); size M
**Opinion:** Keep: size caps and FIB sanity checks on the legacy DOC reader are cheap.
**Double-check:**
1. For every gate the triage marks failing or partial, re-run its check on `origin/main` (the stack may have moved it): the audit row's `Gates` section names the command or the shape; build the probe file with `jubarte debug` or a small zip/XML, never a copied binary.
2. A gate that now holds: name the test that pins it (`grep -rn` in `tests/`); none pinned means it does not hold.
3. Edit the issue body: append `## Remaining after #1278 (2026-10-19)` with one checkbox per gate still failing, each with the command that shows it.
4. Where the opinion says downgrade, fold or narrow: post it as a comment with the reason; do not retitle, relabel or close without Arthur's reply on that comment.
**Close only if** step 1 finds no failing gate; then close with the per-gate commands as the comment and tick the row in #1216. **Otherwise** keep open.

### #703 · audit(R01): ZIP Container Reading [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA — gates in ZIP-11/12/13/14/17/32/N03; two gaps (local/central identity + overlap) carried by ZIP-11/17, not in this list
**Opinion:** Checklist of its owners; close when they close.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #711 · audit(R03): Deflate Stream Inflation [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA — gates in ZIP-14/17/18/19; ZIP-17/18 not in this list
**Opinion:** Same.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #722 · audit(R05): Decompression Bomb Prevention [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA — gates in ZIP-13/14/19/D17/N13; repeated-work/live-memory gap is D17, not in this list
**Opinion:** Same.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #726 · audit(R40): Resource Budgets and Timeouts [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA; size L
**Opinion:** Same.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #727 · audit(R06): Zip-Slip and Path Canonicalization [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA (1 unowned gap found)
**Opinion:** The triage found one unowned zip-slip gap: file it first.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #730 · audit(R07): XML Deserialization Architecture [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA (rows N01/N02 landed; ZIP-47 partly)
**Opinion:** Same as #703.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #733 · audit(R41): Panic-Free Robustness [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA; size L
**Opinion:** Keep as a standing gate even after the owners close: a fuzz or Miri run that fails on panic is a real check, not a pointer.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #737 · audit(R09): XML Bomb and Attribute Defense [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA; size L
**Opinion:** Same as #703.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #738 · audit(R42): Supply Chain and Build Purity [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA — gates live in ZIP-01/02/03/44 (audited) + 8 rows not in this list
**Opinion:** Same as #703.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #739 · audit(R10): Namespace Resolution and Bounds [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA; size L
**Opinion:** Same as #703.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #675 · audit(R32): Legacy Binary MS-DOC [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA → N12 (#669) + D16/D17/D25/D26/D28 (not in this list)
**Opinion:** Same as #703.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #682 · audit(R34): Markdown, HTML and LaTeX Conversion [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA (gates live in J01/D16/D18/D25/D26/DX01/DX04/N07/J04/N08/N19/D34; HTML/LaTeX residual carried by no row); size L (residual)
**Opinion:** The HTML/LaTeX residual is carried by no row: file it first.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #693 · audit(R36): RAG Chunking and Redaction [Urgent]
**Action:** VERIFY the owner rows; close when every owner is closed  
**#1216 triage:** triage: UMBRELLA → N13 (#616), D20 (#672) + 9 rows not in this list; N14/N17 gaps carried by no row here
**Opinion:** N14/N17 gaps are carried by no row: file them first.
**Double-check:**
1. List the owner rows the triage names; `gh issue view <n> --json state,title` for each.
2. If the triage names a gap carried by no row, file it first as its own `audit(...)` issue (body starts with `@coderabbitai plan`, cites the umbrella and the evidence line), and add it to the owner list.
3. Edit the umbrella body: append `## Owners (2026-10-19)` as a checklist, one line per owner with its state.
**Close only if** every owner is CLOSED and no unowned gap remains; comment `all owners closed: <list>`. **Otherwise** keep open; the checklist is the tracking.

### #1216 · tracking(audit-urgent): the 46 open [Urgent] audit rows, re-checked at the stack top and fixed PR over PR
**Action:** UPDATE the tracker; keep open  
**Double-check:**
1. After each row above is closed or commented, tick or annotate its line in #1216's table; add a `closes when` column value (`merged in #1278`, `owner rows`, `Arthur: key rollout`, …).
2. Every row whose status changed today has a comment on its own issue with the evidence; the tracker never carries evidence the row lacks.
**Never close** #1216 while any row is open.

## Part B: standalone cli issues (13)

### #837 · cli(v2): typed edit actions, edit apply --plan and --save-plan
**Action:** CLOSE as fixed by PR #1078 (`feat/v2-09`, head `45b060f2d2`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 45b060f2d2 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_parser --test cli_plan_record --test edit_single_output --test result_envelope` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1078, on main via #1278 (merge-base check on 45b060f2d2). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1082 · test(cli): the jubarte.exe help test hard-links across volumes on windows-latest
**Action:** CLOSE as fixed by PR #1085 (`fix/win-hardlink-1082`, head `9742f4c251`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 9742f4c251 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_plan_record` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1085, on main via #1278 (merge-base check on 9742f4c251). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1093 · capabilities(v2): the CLI reports PlanV2 as not dispatched although edit apply dispatches it
**Action:** CLOSE as fixed by PR #1103 (`fix/caps-1093`, head `2fd57c7b14`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 2fd57c7b14 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test capabilities_v2 --test m_cli_agent` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1103, on main via #1278 (merge-base check on 2fd57c7b14). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1096 · cli: --all-matches help says it replaces every nonoverlapping candidate; overlaps refuse
**Action:** CLOSE as fixed by PR #1112 (`fix/all-matches-help-1096`, head `25b853666a`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 25b853666a origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_parser` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1112, on main via #1278 (merge-base check on 25b853666a). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1140 · cli: print SUCCESS and a before/after diff of the agent view after every write (-q silences)
**Action:** CLOSE as fixed by PR #1144 (`feat/success-diff-1140`, head `3c219159d2`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 3c219159d2 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test agent_text_view --test cli_output --test cli_plan_record --test cli_success_diff --test edit_resolve_lines --test edit_single_output --test m7_cli --test m_cli_agent --test markdown_cli` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1144, on main via #1278 (merge-base check on 3c219159d2). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1151 · cli: an existing output without --force exits 2 from edit/add but 1 from accept, compare and the other tasks
**Action:** CLOSE as fixed by PR #1165 (`fix/force-exit-1151`, head `9a82aa9d44`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 9a82aa9d44 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_failure_paths --test cli_output --test convert_docx_to_pdf --test m7_cli --test m_cli_agent --test markdown_cli --test scrub_cli --test validate_cli` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1165, on main via #1278 (merge-base check on 9a82aa9d44). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1222 · cli: on Windows agent-facing paths print as //?/C:/… (slash_path of a verbatim canonical path) and two messages break the #568 '/' convention
**Action:** CLOSE as fixed by PR #1226 (`fix/win-paths-1222`, head `541b287563`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 541b287563 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_output --test cli_plan_record --test coverage_gates` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1226, on main via #1278 (merge-base check on 541b287563). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1224 · #1199 leftovers: A16 scope (story parts), tests beyond the body (moveTo, pPrChange, sectPr, only paragraph), a selective CLI case, Word check of inserted-content reject
**Action:** CLOSE as fixed by PR #1236 (`test/final-mark-1224`, head `22846a52fa`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 22846a52fa origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test m_accept_word_parity --test m_reject_word_parity` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1236, on main via #1278 (merge-base check on 22846a52fa). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #1244 · test(cli): review_10 expects canonicalize().display(); Windows prints a slash path with the 8.3 name
**Action:** CLOSE as fixed by PR #1248 (`fix/win-wrote-path-1244`, head `9dac2916d3`), on main through PR #1278  
**Double-check, all must hold:**
1. `git fetch origin main && git merge-base --is-ancestor 9dac2916d3 origin/main` exits 0. If it exits 1, PR #1278 has not merged or was squashed: stop, do not close.
2. On a checkout of `origin/main`: `cargo nextest run --test cli_expectations --test cli_plan_record` passes (mbx `cargo`, one Cargo process per checkout).
3. Read the issue's acceptance gates (the `Gates`/`Acceptance` section of the audit row). For each gate name the test that pins it as `file::test_name`, found with `grep -rn '<gate keyword>' tests/`. A gate with no test is a gap: do not close; comment the gap on the issue and on #1216.
**Close only if** every step holds. **Comment to post:** `Fixed by #1248, on main via #1278 (merge-base check on 9dac2916d3). Gates: <gate → file::test>. Verified 2026-10-19.`
**Otherwise:** leave open, post the failing step's command and output, and tick nothing in #1216.

### #816 · tracking(cli-v2): edit plan v2, typed actions, PreparedEdit, edit template, expectations
**Action:** UPDATE the tracking table; keep open  
**Double-check:** 1. For every V2 row whose PR is on main after #1278, set its status to merged with the merge commit. 2. Add a line for #1278. **Never close** while a V2 row is open.

### #863 · audit(ZIP-45): Core/CLI/oracle feature isolation [ASAP]
**Action:** VERIFY and keep open (ASAP; a Cargo feature split)  
**Double-check:**
1. `cargo tree -e features -i ooxmlsdk` (or the oracle crate's name in `Cargo.toml`): the oracle is behind a non-default feature.
2. `cargo build --no-default-features` builds the core without the CLI and oracle.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; keep; schedule before 1.0, after the ZIP batch.

### #1059 · security(codeql): fix incomplete escaping in bump-version.mjs and file-system races in the WASM CLI (#1, #2, #5, #6)
**Action:** VERIFY and keep open (CI hygiene, small)  
**Double-check:**
1. `gh api repos/jandira-tech/jubarte-redlines/code-scanning/alerts --jq '.[] | select(.state=="open") | .rule.id'` lists the four alerts.
2. Each is fixed in code on one PR (no dismissals); the escaping test in `jubarte-wasm` fails before and passes after.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; then close with the PR number.

### #1273 · cli: help that fits a clock (plan 22)
**Action:** KEEP (Plan 22 of the 0.13 plan; ours)  
**Double-check:** nothing to verify; it closes with its PR. Leave it.

## Part C: cli issues with a recorded design-review verdict (83)

### #872 · cli: the new CLI contracts, schemas and guarantees require implementation and conformance tests before any of this is behavior
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #873 · cli: evaluate operation fidelity, required content, preservation, delivery and claim accuracy separately, and build the executor guarantees before advertising them
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #874 · cli: treat the supplied reports as case-specific evidence only — the 114 mb corpus was not reprocessed, grades were never bound to artifacts, and the fourteen release probes are diagnostic tests, not an agent study
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #875 · cli: the delivered schemas and examples are design artifacts validated only on a four-paragraph in-memory text model — no CLI execution, DOCX mutation, revision settlement, comment construction, package preservation, publication fault handling, rendering or runtime conformance was tested — and the validation record is the authoritative inventory
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #876 · cli: report reported aggregates only with their coherent units and limits — score distribution on 56 judged runs with missing grades visible, gate failure as 28.57% of judged runs, 306/454 as pooled item failure, 64 events across 35 runs as distinct units, overflow and reread units kept apart, and trajectories as case studies not treatment arms
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #877 · cli: implement in the staged delivery order — freeze contracts, identities, shared vocabulary and regression fixtures; core source/refusal/receipt correctness with selected-review, body and comment preparation; working-text guards, exact composition, expectations and schemas; typed routes and bounded read --for-edit; edit template last on the shared extractor/compiler; controlled comparisons and the supported release profile; broader adapters after their preservation and usability gates — with the implementation plans' modules, fixtures and exit criteria taking precedence over estimates or one-large-rewrite sequencing, and executor capabilities built before the template generator over them
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #877: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #879 · cli: correct the eight-versus-fourteen never-editing contradiction — six listed names appear in the first-edit table and eight-plus-eight is a coherent reconstruction, not certified corpus truth
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #881 · cli: stage A — freeze the PlanV2 model and compatibility boundary on the existing edit.rs integrations, reusing and verifying the existing origin/settlement machinery (EditPlan/ResolveRevisions/resolve_selected/settle/Transaction/preview_plan/apply_plan in src/edit.rs plus cli/bin/markdown entry points) as an essential dependency rather than adding a parallel edit engine, with generated schema/examples/capabilities from the actual dispatch model and syntax-vs-resolution distinguished in result fields
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #881: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #882 · cli: correct the reported 89% first-edit median — the displayed table gives 64% (76.5%/76.4610% excluding the misclassified iNcXKY8 row), first edit is mixed with first dry run for 7R7A5GW, and the 78%-of-budget refusal median has no reconstructible table
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #884 · cli: stage A gate — all shipped examples validate through the exact production schema and enter the corresponding dispatcher; unknown fields, incompatible flags, unsupported versions and third ID representations fail predictably; legacy conformance fixtures still exercise their old contract
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #884: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #885 · cli: the two .319 means cannot both hold — the consistent pair is .319/.295075 or .344865/.319, and one must never be selected silently
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #886 · cli: the template generator reports row counts, conditional-support counts and unrepresentable content, and never claims to have identified what the user wants changed
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #886: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #893 · cli: stage B — establish the effective baseline and precise readiness from one shared extractor: resolve original pairs once verifying both IDs name one entity, freeze direct selection and original-match context on S before settlement with revision authorizations separate, validate accept/reject choices against the original inventory before modifying a transient working copy, retain original-to-working correspondence, compute exact logical text and before_sha256 from W, and add available/conditional/unavailable/unknown readiness records — shared across read --for-edit, template, inline actions and apply
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #893: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #894 · cli: the skill comparison has unresolved denominators and confounds — 'documentation placement does not help' exceeds the observational evidence and opening a document is not understanding it
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #900 · cli: the author gate is not a byte comparison — it evidences only its own predicate, not empty documents, unchanged bytes or absence of all work
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #903 · cli: stage B gate — a selected existing insertion is accepted and edited in one transaction; rejecting it changes the baseline so an old rewrite hash refuses before any output; keep leaves unselected revisions and attribution intact; refuse blocks a remaining revision outside the selected footprint; editable ranges outside a pending insertion stay available; merged/split targets stay read-only until the separate adapter passes its tests; readiness never predicts an unwritten rewrite
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #903: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #908 · cli: capabilities defaults to a compact task map and per-operation output carries actual input and result schemas, a minimal validated example, story and target support, tracked and direct modes, dependencies, constraints, side effects, error codes and limits — the file-specific variant describes known support and unknowns and never equates preservable with editable
**Action:** CLOSE as a duplicate, folded into #847  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #847's body (`gh issue view 847`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #847 as `From #908: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #847 or posted there. **Comment to post:** `Folded into #847: <clauses, one line each>. Verified 2026-10-19.`

### #911 · cli: keep report rate claims consistent — 2,531 calls are a corpus total not a per-run figure, judged-only exclusion conflicts with all-75 behavioral rates, and the 37-versus-two comparison cannot establish equivalence between validation-citing groups
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #912 · cli: stage C — compose mixed requests with counted anchors (aa in aaa has two candidates), interval composition over one baseline with one verified final paragraph, deterministic boundary order for same-edge insertions, attached_comment's exact nonempty anchors without another result-ID family, comment anchor preservation through defined range correspondence with accurate removal reporting, and enforced comment conflict rules; legacy matching stays versioned
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #912: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #914 · cli: stage C gate — two disjoint replacements in one paragraph succeed including repeated anchors made explicit; the first replacement cannot change which occurrence the second uses; true overlap or unsupported boundary ownership refuses; an unchanged scaffold plus an explicit replacement does not restore old text; attachments fire on unchanged text with tested nonempty anchors and empty ones refusing; a comment target removed by selected resolution reports that cause; conflicting review actions fail before publication; a changed full rewrite conflicts within its paragraph — not a blanket one-operation-per-paragraph rule
**Action:** CLOSE as a duplicate, folded into #816  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #816's body (`gh issue view 816`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #816 as `From #914: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #816 or posted there. **Comment to post:** `Folded into #816: <clauses, one line each>. Verified 2026-10-19.`

### #915 · cli: two investigated false-failure candidates do not justify keeping the error channel exactly as it is — missing target context and incomplete inline recovery options are observed gaps, and syntactic validity is not semantic support on documents with pending revisions
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #915: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #929 · cli: treat the report primer with caution — its highlighted criticmarkup example does not encode a tracked replacement, its in-place example conflicts with originals-never-modified, pending-insertion restrictions are not limited to another author's insertions, and synthetic examples establish no event frequency
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #936 · cli: jubarte edit template generates a source-bound plan with scope and working-basis options — --select/--story/--find/--regex/--ignore-case/--changed/--by/--context, repeatable --accept-change/--reject-change, --author/--datetime/--track/--existing-revisions/--dry-run/--json/--jsonl, each with its stated default and conflicts, no activation flag, empty-text and delete_paragraph semantics, and a receipt that is not another plan
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** restore `--expect-sha256`, `--report`, `--force` and `-o` to the `edit template` option surface
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #940 · cli: the 971-request telemetry cannot estimate a jubarte treatment effect — no reliable task/session join, a censored trajectory gets no failure outcome for not having edited, two cases make no scaling law, and confidence intervals do not repair task mismatch or item dependence
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #944 · cli: read the updated follow-up selectively — the pid proposal stays withdrawn and is not a feature to resurrect, its six-item historical list sets no today priorities, and global comment-detachment waivers, short digest prefixes and text-containment anchor-survival approximations need scrutiny rather than automatic implementation
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #953 · cli: transfer only what the literature supports — expose known mismatches and preserve meaning through adapters, but no blind fuzzy repair, no mandatory extra verification command, no autonomous completion classifier, and no forecast of reduced agent failures from any paper number
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #962 · cli: resolve all literal spans against one working snapshot — disjoint replacements in one paragraph compose into one desired result and one final postcondition and must survive the redesign; composition validates source, targets, working-basis hashes and selected scope before classifying rows; operations never observe earlier operations' inserted text; active whole-paragraph rewrites conflict with other active text edits in that paragraph; overlapping replacement/deletion ranges refuse; same-position insertions follow declared array order with other ambiguous boundary ownership refusing; insertion inside a replaced span or at an ambiguous boundary refuses
**Action:** CLOSE as a duplicate, folded into #912  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #912's body (`gh issue view 912`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #912 as `From #962: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #912 or posted there. **Comment to post:** `Folded into #912: <clauses, one line each>. Verified 2026-10-19.`

### #965 · cli: anchored operations expose 1-based --occurrence wherever the typed schema supports it — occurrence indexes every exact literal start position including overlaps (aa in aaa is two candidates), exactly one match is required without an occurrence, --all-matches demands --expect-count with tested count and non-overlap semantics, overlapping candidates are counted exhaustively never greedily, flags error when inapplicable, and legacy matching stays versioned
**Action:** CLOSE as a duplicate, folded into #823  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #823's body (`gh issue view 823`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #823 as `From #965: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #823 or posted there. **Comment to post:** `Folded into #823: <clauses, one line each>. Verified 2026-10-19.`

### #967 · cli: a missing anchor or find must never silently become whole-block or whole-paragraph deletion or rewrite — delete removes one exactly selected text range, and typed actions make edit scope explicit
**Action:** CLOSE as a duplicate, folded into #837  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #837's body (`gh issue view 837`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #837 as `From #967: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #837 or posted there. **Comment to post:** `Folded into #837: <clauses, one line each>. Verified 2026-10-19.`

### #968 · cli: close the demonstrated schema precision gaps — exact hash lengths with no trailing newline admitted in hashes or requirement keys, explicit character/zone constraints, a registered date-time check, and v2 uniqueness counting every possible literal start position (aa in aaa) rather than trusting a greedy nonoverlapping scan
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #968: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #970 · cli: keep archive/planning/template.edited.json valid as the canonical edit-plan v2 fixture — the schema and generator must keep every field and value there valid exactly as written
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #970: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #971 · cli: preserve review relationships through composed edits — comment ranges map through composition and are never silently detached when their anchor content vanishes (refuse instead); a reply to a settlement-removed comment refuses; deleting a comment conflicts with other mutations, replies or state changes on its thread; at most one edit_comment and one resolve_comment per comment/thread with the state applied after replies; replies preserve the original thread state absent a declared final state and sibling order is array order with no implicit chain; delete_comment root with replies requires include_replies; unknown or removed targets refuse; edit_comment preserves recorded attribution
**Action:** CLOSE as a duplicate, folded into #833  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #833's body (`gh issue view 833`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #833 as `From #971: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #833 or posted there. **Comment to post:** `Folded into #833: <clauses, one line each>. Verified 2026-10-19.`

### #973 · cli: selected revision decisions, body edits and review operations must work together in the first useful workflow — a text-only template that postpones revision handling repeats the documented obstacles, so no text-only template ships first
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #973: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #974 · cli: keep the four-layer responsibility split — CLI/engine for targeting, declared transformations, preservation boundaries, useful refusals, source/output identity, exact receipts and honest verification scope; agent for interpretation, decisions, edits and supported claims; harness for status/streams, budgets, artifact collection, reproducible identity and independent scoring; evaluation for outcomes, joins, adjudication and missing-data reporting — a receipt cannot repair a harness grading a different fixed path
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #975 · cli: attached_comment is supported on replace, rewrite, insert and insert_paragraph — plain text under the plan's new-author/date policy, anchored to the complete nonempty resulting span or paragraph from the compiler's verified span, refusing on empty resulting text with no invented neighbor, still firing on unchanged rewrites or identity replaces, needing no operation-result identifier and introducing no second public result-ID family
**Action:** CLOSE as a duplicate, folded into #833  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #833's body (`gh issue view 833`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #833 as `From #975: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #833 or posted there. **Comment to post:** `Folded into #833: <clauses, one line each>. Verified 2026-10-19.`

### #976 · cli: unchanged plans succeed explicitly — a no-effect apply returns ok with effect unchanged and, when output is requested, copies the source bytes exactly; --require-change is the opt-in for callers needing a nonempty primary effect
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #976: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #978 · cli: never collapse the released 0.12.0 asset, audited main, the trace build and the follow-up build into one 'current 0.12.0' — every claim names which identity it concerns, and the trace help build or a future-dated audit heading is not proof of later runtime behavior
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #981 · cli: composition returns committed output references with source-to-output correspondence verified after serialization — operation pointers like /operations/3 stay request correlation, never a document-ID family, and later review actions use verified output references or a new read
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #981: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #982 · cli: evaluate transaction size by workflow while retaining atomicity per batch — no blanket tiny-batch policy
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #983 · cli: source_context carries immutable original-source final-projection observations — never executable instructions or readiness authority; comment observations keep original bodies; supplied text is re-extracted and verified with changed text refused; removal requests no mutation; working text, hashes and readiness are recomputed by the engine from W
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #983: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #984 · cli: the actual primary bytes are selected before final checks so the validated package is the published one — an unchanged transaction publishes the exact original bytes, never a validated reserialization substituted at commit, with no invented revisions or metadata; text equality alone proves nothing (revisions, comments, formatting, media and empty-paragraph structure count as effects), and a requested change lost to normalization is postcondition_failed, not unchanged success
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #984: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #985 · cli: coverage records generated traversal and representation facts — never permission or task-completion evidence; actual constraints are recomputed, and no ready or conditional flag in a saved file grants permission or replaces preparation (editability, revision dependencies, scope and resource availability are recomputed)
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #985: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #986 · cli: no automatic LLM legal grader or opaque false-success classifier inside the CLI — harness grading binds the graded artifact to its actual path and hash and reports uncertainty
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #987 · cli: coverage is a closed counter object — traversal_complete, text_complete, eligible_paragraphs, context_paragraphs, opaque_regions and optional conditional_paragraphs as a subset of eligible
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #987: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #988 · cli: --require-change applies only to the primary transaction — NO_CHANGES_REQUESTED before publication when primary effect is zero; previews, reports, patches and clean derivatives never satisfy it, an unchanged tracked primary may still carry an explicitly requested clean derivative that accepts revisions, and it never waives failed postconditions or missing anchors
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #988: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #990 · cli: the versioned text codec is exact — w:t Unicode characters, U+0009 tabs, U+000A supported wrapping breaks, the paragraph mark outside text, opaque spans typed or read-only and never magic placeholders, no silent normalization, Markdown unescaping, fuzzy matching or first-match selection, legacy normalization versioned; the whitespace consequences (double-space preview reporting nothing changed, ordinary space matching nbsp) are fixed in the engine and adapters, not with help wording
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #990: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #993 · cli: every apply distinguishes immutable original source (S), working source after selected decisions (W) and the serialized candidate, and runs one ordered pipeline — admit inputs/resources, resolve decisions and scope, validate guards and ranges, compose and prepare effects, select the actual primary bytes (S verbatim when truly unchanged), reopen those bytes, verify exact postconditions and required preservation, evaluate supplied expectations, derive output references and digests, enforce derivative policies, then publish; the dry run follows the same path and stops before publication
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #993: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #994 · cli: remove run_operation — a desired-text change is a request and unsupported content stays identified context
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #994: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #995 · cli: selected decisions apply first, then the remaining-revision policy — keep preserves the remaining history, refuse is global (any remaining revision anywhere, including outside direct scope, blocks, covering unchanged and comment-only plans), explicit global accept/reject operates on the remainder even with empty operations and stays gated by correspondence/preservation tests, unknown revision forms cannot establish revision-free, and a resolution-only request publishes without a dummy replacement
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #995: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #996 · cli: all input identities — sources, plans, expectations and resources — are protected from output alias collisions (symlinks, hard links, normalized path aliases and identical paths detected); --force replaces only an explicitly named existing output and never overrides a stale hash, resolves a revision, bypasses a structural check or silently overwrites an input; fixed-delivery-path tasks preserve an explicit baseline source and publish to the required distinct destination
**Action:** CLOSE as a duplicate, folded into #829  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #829's body (`gh issue view 829`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #829 as `From #996: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #829 or posted there. **Comment to post:** `Folded into #829: <clauses, one line each>. Verified 2026-10-19.`

### #997 · cli: every revision resolves against the original source — accept/reject sets must be disjoint after aliases and dependencies are resolved with both move halves expanded (one logical change), dependents carried by a selected parent are reported separately, and a revision selected for both acceptance and rejection is a conflict
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #997: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #998 · cli: dry run follows the same required preparation but writes nothing — no document, plan, report, render, extracted asset, temporary filesystem file or persistent cache; bounded in-memory preparation with honest runtime limits (report the limitation rather than claim conformance); reserves no source or destination state and leaves path-specific checks unassessed without an output destination
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #998: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #999 · cli: acceptance is a substantive explicit review decision — never inferred from desired text, an agreed comment, a display projection or a refusal; the tool never chooses negotiating positions or accepts old revisions merely to reduce refusal rates; revision-only and comment-only transactions are valid work; selected decisions are an essential dependency, not a deferred convenience
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #999: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1000 · cli: direct content/comment scope stays separate from revision decisions — naming a revision authorizes only its whole logical decision and disclosed dependencies, never unrelated body edits; a through-range comment requires every affected paragraph in direct scope; selected accept/reject sets resolve on the original snapshot without overlap including logical move pairs; checks run after descendant expansion and canonicalization
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1000: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1002 · cli: scope descendant expansion uses the compiled source model, not guessed ID arithmetic
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1002: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1003 · cli: one versioned PlanV2 request model serves generated and hand-authored requests — jubarte edit template and jubarte edit apply --plan over one shared executor, with JSON Schema, examples, help and capabilities generated from the compiled model
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1003: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1004 · cli: new edits default to tracking on and remaining revisions kept — the track switch affects only new edits, never settlement; reading a final view accepts nothing; a faithfully unrepresentable tracked operation refuses; direct-only operations require deliberate --track off; direct mode with keep still preserves pre-existing revisions; unsupported track-off plus keep must not resolve to an all-accepted clean copy; keep preserving unselected revisions is a mandatory engine invariant, not an expectation predicate
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1004: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1005 · cli: machine output uses one consistent envelope — schema_version, command, engine version, runtime, status, source hash, phased data with counts and verification, artifact records with role, media type, absolute path, digest and committed state, warnings, error, effective_options and continuation, volatile timing kept separate, with bounded actual context, supported occurrence/source-hash flags and structured next-action arguments in failures
**Action:** CLOSE as a duplicate, folded into #825  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #825's body (`gh issue view 825`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #825 as `From #1005: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #825 or posted there. **Comment to post:** `Folded into #825: <clauses, one line each>. Verified 2026-10-19.`

### #1006 · cli: plan v2 is one closed model — required schema_version (constant 2), source, policy, resolve_revisions, author, date, scope and operations, plus optional source_context and coverage — no other properties; operation JSON pointers locate request entries in errors but are never a document-ID family
**Action:** CORRECT the text, then CLOSE folded into #821  
**Double-check:** 1. `schemas/edit-plan-v2.schema.json` (or the core schema in `archive/planning/`): count the top-level properties; eight are required and `source_context` and `coverage` are optional, so 'exactly eight permitted' is false. 2. Post the corrected sentence on #821 as `From #1006 (corrected): …`. **Close only if** the correction is posted. **Comment:** `Wrong as written (ten permitted, eight required); corrected clause posted on #821.`

### #1007 · cli: every plan binds to its original DOCX through the mandatory full 64-character lowercase-hex source.sha256 — a moved or copied path with identical bytes is valid
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** v1 recorded digests and `--expect-sha256` stay valid bindings; an explicit unbound mode exists
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1008 · cli: validate hashes even on apparently unchanged rows, then discard their verified-unchanged body-rewrite components before overlap checks and output postconditions — an attached comment remains an effect, and removing a row means no rewrite while settlement and other operations still act on the paragraph
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1008: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1009 · cli: whole-paragraph rewrites require exact one-to-one original-to-working paragraph correspondence — merges take one owner, splits hash nothing, removed targets refuse, merged/split paragraphs become read-only until a separately tested range-mapping adapter exists, and new pN addresses are never inferred
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** a merged-paragraph rewrite is forbidden (competing owners), not 'take one owner'
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1010 · cli: plan policy is a closed object — track on/off, existing_revisions keep/accept/reject/refuse, text_view locked to final
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1010: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1011 · cli: never advertise index mapping or text-based reconstruction as preservation-grade identity — origin/fate machinery and numeric IDs prove nothing about exact range provenance after merges or splits; the initial rewrite contract is one-to-one correspondence, with stronger mappings separately gated before any preservation claim
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1011: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1012 · cli: resolve_revisions is a closed object of unique accept and reject original-source target arrays; empty sets are valid
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1012: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1013 · cli: preserve the released probes' exact refusal boundaries — missing and repeated anchors refuse with stable codes and counts — while improving actual-context recovery
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** the released probes' missing-anchor and ambiguous-anchor refusals stay exactly as observed
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1014 · cli: add a serialization/range-preservation regression fixture for the bookmark-crosses-revision episode — 95 dry-run-passing operations whose scratch-docx application reveals the crossing, with the insertion moved from after p33 to before p34
**Action:** CLOSE as a duplicate, folded into #827  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #827's body (`gh issue view 827`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #827 as `From #1014: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #827 or posted there. **Comment to post:** `Folded into #827: <clauses, one line each>. Verified 2026-10-19.`

### #1017 · cli: receipts are compact and structured — actual absolute paths, digests, committed state, distinct operation/revision/comment counts, selected decisions, effects, preserved marks and the exact checks performed; the harness captures subprocess status directly and grades the artifact the receipt names, and the agent verifies and reports that artifact; a receipt never claims whole-task completion and cannot repair a harness grading a different fixed path
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** the receipt contract (absolute paths, digests, counts) without the harness-conduct clauses
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1018 · cli: stage-3 matched evaluation ablates compact discovery, readiness/selected settlement, desired-text templates and optional expectations separately where feasible across sparse edits, multi-change batches, large documents, revision-heavy cases, legitimate no-ops, unsupported requirements and duplicated clauses; primary outcomes are required changes completed, unintended edits, preservation, correct artifact delivery and truthful final claims, measuring false-positive refusals and false-negative acceptance, reporting legitimate refusals separately from nonexecution and never counting cosmetic edits as progress; secondary cost measures explain without substituting for completion; task/run-level uncertainty, paired/clustered accounting, ungraded/censored runs stay visible, with independent artifact-sample audits when LLM rubrics are used; human usability claims require human evaluation
**Action:** CLOSE as a duplicate, folded into #849  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1018: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1021 · cli: correct required edits, preservation and correct artifact delivery decide — success is never inferred from operation counts, edit count, passing an author gate or an arbitrary early edit (runs with 31, 35 and 95 applied operations still omitted required content); measure the first required committed change, eventual requirement coverage, unwanted changes and completion-claim accuracy, with a defined regression margin covering harmful or unintended edits before improvement is asserted
**Action:** CLOSE as a duplicate, folded into #873  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #873's body (`gh issue view 873`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #873 as `From #1021: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #873 or posted there. **Comment to post:** `Folded into #873: <clauses, one line each>. Verified 2026-10-19.`

### #1022 · cli: plan operations holds typed requests and may be empty — review decisions alone are valid work; the array is an ordered sequence
**Action:** CLOSE as a duplicate, folded into #999  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #999's body (`gh issue view 999`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #999 as `From #1022: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #999 or posted there. **Comment to post:** `Folded into #999: <clauses, one line each>. Verified 2026-10-19.`

### #1024 · cli: acceptance (stage A) — a usage/schema error with no invented conversion for bare psN or retired pid selectors; removed-row/empty-text/delete_paragraph row semantics (no rewrite, empty paragraph keeping the mark, structural deletion only where supported with affected anchors reported); exact handling of spaces/NBSP/tabs/soft breaks applied or refused with no normalization-based success; a usage error in the shared parser's recognized machine error mode for --json/--jsonl with -o -
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1024: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1027 · cli: a generated rewrite row carries kind, paired target, generated before_sha256 and one desired text value — changing the text requests the edit and no activation/run_operation boolean exists; empty text keeps the paragraph mark, removing the row requests no rewrite while decisions and other operations still apply, structural deletion is delete_paragraph, source IDs and hashes are generated preconditions rather than editable content, and examples copy real target pairs and hashes from the generator or reader
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1027: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1032 · cli: stage B acceptance — STALE_SOURCE on changed bytes; a moved path with identical bytes stays valid; before_sha256 mismatch on retargeted rewrites; mismatched short/long ID pairs refuse even when each names an entity; read-only rows for settlement-merged/split paragraphs; one-transaction accept-and-edit of a pending insertion; WORKING_BASE_CHANGED on rejection with old generated text/hash; unchanged hashes usable after resolving an unrelated revision; double selection conflicts before preparation; refuse policy blocks a remaining revision outside edit scope; revision effects disclosed, no unauthorized direct edits; a through-range skipping an intervening out-of-scope paragraph refuses; frozen original selection, labeled context; read-only --context; valid source_context under changed decisions with recomputed W; cursor reuse under changed decisions/tracking/policy refuses; replaces outside pending insertions succeed; a precise blocking revision/range, never automatic acceptance
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1032: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1033 · cli: stage 2 builds a one-row-per-run ledger linked to event and artifact records — task/input/prompt/build hashes, model/harness versions, documentation exposure, requested author/output, first valid request, first committed required effect, stop reason, actual output path/digest, gate predicate, grade and exclusion reason — distinguishing edit invocations from quoted commands, dry runs from commits, and repeated error events from affected runs, reconciling the disputed aggregates before using them as baselines
**Action:** CLOSE as a duplicate, folded into #849  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1033: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1035 · cli: release when the received-redline workflow works with verified effects and preservation and remaining uncertainty is acceptable for the advertised scope — and claim no predicted percentage reduction in agent errors; desired-text templates, read budgets and expectations-assisted completion stay empirical hypotheses with no estimated percentage improvement
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #1039 · cli: one apply transaction publishes a verified document from a source-bound edit request — obtain the request, change desired text in a few relevant paragraphs, make explicit decisions about blocking existing revisions, publish verified — and saving the request is distinct from changing the DOCX
**Action:** CLOSE as a duplicate, folded into #973  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #973's body (`gh issue view 973`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #973 as `From #1039: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #973 or posted there. **Comment to post:** `Folded into #973: <clauses, one line each>. Verified 2026-10-19.`

### #1040 · cli: separate verified arithmetic, reported episodes, source facts and proposed behavior, and adopt one contract across schemas, help, examples and implementation plans
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #1041 · cli: terminal detection changes presentation only — never data protocols or side effects; keep explicit JSON rather than switching protocols on TTY detection; broken pipes on read-only text output must not create files or produce long panic tracebacks; NO_COLOR, TERM=dumb, CI and a documented no-input signal are respected; keep force narrow, keep existing meaningful exit conditions (usage 2→64 needs versioned migration), and 'transient' never means a retry is safe after uncertain publication
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** terminal detection changes presentation only: NO_COLOR, TERM=dumb, CI and a broken pipe never change data protocols or exit codes
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1045 · cli: the delivered plan and expectations schemas are proposed design artifacts — the 2026-10-17 text-and-review profile validating its bundled examples only; versioned once released; the production schema is generated from the actual compiled dispatcher and the hand-written core file never replaces it; neither is a ratified public contract
**Action:** CLOSE as a duplicate, folded into #849  
**Note:** Test-matrix rows whose every clause has an owner; #849 (V2-15) is the harness that runs them. Post the rows there as a test list.
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1045: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1049 · cli: binary stdout requires explicit -o -, a declared target format and nonterminal stdout, conflicting with machine receipt output; a raw JSON plan on template -o - is the primary product conflicting with envelope modes, and JSONL is never an executable plan format
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** binary stdout requires an explicit `-o -`, a declared target format and a non-terminal
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1052 · cli: unchecked or unassessable requirements are reported unassessed — empty checks never means passed, and natural-language requirements stay explicitly unassessed
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** report which requirements had no executable check as unassessed, never as passed
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

### #1056 · cli: one compiled operation model drives clap grammar, JSON Schema, concise and complete help, examples and capabilities — parser validity is distinct from document support, machine-mode detection for usage errors respects option values and --, document text equal to --json never switches protocols, and bash/zsh/fish completions are generated follow-on work that never gates noninteractive calls
**Action:** REWRITE to its residue, then VERIFY and keep open  
**Residue to keep:** one compiled operation model drives clap grammar, JSON Schema, help and capabilities (Plan 22's basis)
**Double-check:**
1. Edit the body: keep the residue sentence above as the obligation; move every other clause into a `Folded elsewhere` list naming its owner (the recorded verdict names them).
2. Reproduce the residue on `origin/main` with one command (build it from the clause; e.g. for #1041 `NO_COLOR=1 jubarte read FILE --json | head -c 200` must be identical to the unset run).
**Close only if** the residue already holds and a test pins it (`grep -rn` in `tests/`); then close with the command as evidence. **Otherwise** keep open with the rewritten body; comment `rewritten to its residue 2026-10-19`.

## Part D: cli issues without a recorded verdict (102)

### #870 · cli: allow honest unchanged outcomes, require deliberate review decisions, preserve exact targets and support sparse inline or batch edits — no incentives toward cosmetic edits or blanket acceptance
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #871 · cli: known unsupported content loss blocks DOCX publication unless that exact diagnosed loss has an implemented explicit allowance — opaque preservation is not understanding or editing, and generic Markdown export is a lossy projection never used as the source for partial document replacement
**Action:** CLOSE as a duplicate, folded into #829  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #829's body (`gh issue view 829`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #829 as `From #871: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #829 or posted there. **Comment to post:** `Folded into #829: <clauses, one line each>. Verified 2026-10-19.`

### #878 · cli: review comment and reply text is separate from body text — existing-comment edits preserve attribution, already-resolved or reopened states are unchanged success, deleting a root with replies requires explicit inclusion, and mapped anchors, auxiliary parts and relationships stay consistent
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** the `review` namespace does not exist; the operations live in plan ops (#833) and accept/reject
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #878 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #880 · cli: create takes exactly one source mode — no content input makes a blank DOCX, --template with --data copies supported body and sections and fills typed controls with reference-doc as formatting only, unknown/duplicate/locked controls refuse, and template revisions default to refuse unless a policy is chosen
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** `create` is gated new engineering by the proposal's own words (#895)
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #880 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #883 · cli: create uses explicit units and section scope for page size, orientation and margins, fails on conflicting reference/template/spec definitions rather than silently overriding, requires explicit local resource roots, and gives CriticMarkup an explicit parse or literal setting
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** same as #880
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #883 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #887 · cli: retries require source and request identity with a known publication state — a transient I/O condition does not establish safe retry; a replay receipt identifies original source, normalized request, expectation, resource and option digests plus the actual output digest; reapplying to the output is a different request, not replay
**Action:** CLOSE as a duplicate, folded into #1028  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #1028's body (`gh issue view 1028`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #1028 as `From #887: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #1028 or posted there. **Comment to post:** `Folded into #1028: <clauses, one line each>. Verified 2026-10-19.`

### #888 · cli: implement the canonical twelve-domain-command tree — create, read, edit, review, compare, render, convert, assemble, check, sanitize, capabilities, self-update — with help as discovery infrastructure and one substantive interface for humans and agents
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** the twelve-domain tree is a rename of the surface; piecemeal renames break agents
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #888 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #889 · cli: root options — mutually exclusive --json/--jsonl, quiet that never suppresses requested data or errors, --color modes, --timeout with explicit publication state, --help-all, and version with build identity and runtime; document inputs remain unchanged and writes require an explicit destination
**Action:** VERIFY and keep open (root options)  
**Double-check:**
1. `jubarte read FILE --json --jsonl` refuses the pair as conflicting.
2. `jubarte read FILE --quiet --json | head -c 100` still prints the JSON; `--quiet` on a refusal still prints the error.
3. Fold #1046, #1050 and #1051 into this issue's body as clauses; send the help clauses to #1273.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; this issue becomes the one root-options owner.

### #890 · cli: ordinary help leads with examples and the side-effect contract and points to a schema instead of dumping all definitions
**Action:** CLOSE as a duplicate, folded into #1273  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #1273's body (`gh issue view 1273`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #1273 as `From #890: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #1273 or posted there. **Comment to post:** `Folded into #1273: <clauses, one line each>. Verified 2026-10-19.`

### #891 · cli: ordinary help shows implemented capabilities only
**Action:** CLOSE as a duplicate, folded into #1273  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #1273's body (`gh issue view 1273`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #1273 as `From #891: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #1273 or posted there. **Comment to post:** `Folded into #1273: <clauses, one line each>. Verified 2026-10-19.`

### #892 · cli: read returns bounded addressable text with revisions, comments, hash and completeness and no layout pass, with the documented selection, for-edit, view and presentation options including --pages requiring layout
**Action:** CLOSE as a duplicate, folded into #839  
**Note:** the `--pages` clause contradicts 'no layout pass'; post it on #839 as an open decision, not as a clause
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #892: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #895 · cli: the typed document-spec and strict template/data adapters stay gated new engineering behind the common render pipeline where validation does not imply visual review — and template creation in the implementation plan means edit-plan templates, not reusable Word forms
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #896 · cli: after mutation, address the output by receipt digest and output references or a fresh read of the output snapshot — never guess how old pN indices shifted
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #896: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #897 · cli: inline edits and coherent batches are both first-class under the same execution guarantees — immediate known changes, one-response investigation and batch authoring without a full-document planning pass
**Action:** CLOSE as a duplicate, folded into #837  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #837's body (`gh issue view 837`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #837 as `From #897: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #837 or posted there. **Comment to post:** `Folded into #837: <clauses, one line each>. Verified 2026-10-19.`

### #898 · cli: inline calls copy the complete returned hash; an unbound inline call validates the current anchor without proving continuity with a previous observation (SOURCE_SHA256 is a documentation placeholder)
**Action:** CLOSE as a duplicate, folded into #837  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #837's body (`gh issue view 837`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #837 as `From #898: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #837 or posted there. **Comment to post:** `Folded into #837: <clauses, one line each>. Verified 2026-10-19.`

### #899 · cli: text files are exact UTF-8 and plain text is never Markdown — a Markdown insertion subset needs an explicit advertised capability, and images come from admitted local file resources
**Action:** VERIFY and keep open (UTF-8 and plain text)  
**Double-check:**
1. `printf 'a\xffb' > bad.txt; jubarte edit FILE -p p1 --content-file bad.txt` refuses invalid UTF-8.
2. A `--content-file note.md` with `**bold**` inserts the literal asterisks (plain text is never Markdown).
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`.

### #901 · cli: generated plans default to the entire document across all stories; examples start from small selections
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #901: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #902 · cli: template creation is edit template next to edit apply; create --template keeps its reusable-docx meaning
**Action:** CLOSE as a duplicate, folded into #843  
**Note:** `edit template` exists on the stack (PR #1081); verify `jubarte edit template --help` on main before folding
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #902: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #904 · cli: convert infers formats only when unambiguous, reports preserved, approximated and dropped features, and DOCX to Markdown to DOCX is never lossless editing — Markdown tutorials use create, and layout output belongs to render
**Action:** VERIFY and keep open (convert contract (with #913))  
**Double-check:**
1. `jubarte convert in.docx -o out.xyz` refuses an unknown format; `-o out` with no extension refuses rather than guessing.
2. The output report names preserved, approximated and dropped features, or the issue stays open for that part.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; fold #913 into it.

### #905 · cli: typed v1 format, list, control, table, image, section, note, TOC and settings operations stay reachable through the exact pinned v1 compatibility adapter until their v2 counterparts pass their own conformance gates — never flattened to text, with capabilities naming each runtime's actual accepted plan versions and profiles
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #905: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #906 · cli: existing plan-v1 continues with its recorded attribution and historical defaults — new schema fields are not accepted secretly by v1, a missing existing-revision policy is not reinterpreted as keep, and the v2 core profile does not remove v1 operations
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. Write a v1 plan with an unknown operation key (`"bogus": 1`); `jubarte edit FILE --plan p.json --dry-run` refuses with `INVALID_PLAN` naming the key.
2. `grep -n 'deny_unknown_fields' src/edit.rs` shows the v1 model is closed.
3. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #907 · cli: changes to argument grammar, JSON/JSONL interpretation, ID serialization, output defaults, match normalization, revision defaults, usage exit status or quiet behavior require a declared breaking CLI boundary, preferably the 1.0 contract — until then only additive discovery and tested adapters land, legacy adapters preserve exact side effects or return a migration error, and aliases never silently promise compatibility
**Action:** VERIFY and keep open (1.0 boundary policy)  
**Double-check:**
1. `grep -rn 'breaking' docs/*.md README.md CHANGELOG.md` finds the declared boundary; if none, this issue is the place to write it.
2. Its checklist lists #888, #942, #956, #961, #972 as parked.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; never close before 1.0.

### #909 · cli: review exposes changes, comments, accept, reject, comment, reply, resolve, reopen, edit-comment and delete-comment — reads are bounded and complete-aware with ID, author, kind, state, story and anchor-intersection filters, and summaries distinguish threads from replies and logical moves from carrier elements
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** the `review` namespace is the twelve-domain regroup (#888)
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #909 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #910 · cli: review accept and reject require --all or a selection — unknown IDs refuse, an empty filtered selection refuses unless --allow-empty, explicit all on a revision-free input is unchanged success, moves resolve together with dependent effects reported, and accepting text never resolves comments
**Action:** VERIFY and keep open (accept/reject selection rules)  
**Double-check:**
1. `jubarte accept FILE -p r999 -o out.docx` refuses an unknown id.
2. `jubarte accept FILE --author Nobody -o out.docx` (empty selection) refuses unless `--allow-empty`, which may not exist yet: then that clause stays open.
3. `jubarte accept clean.docx -o out.docx` on a revision-free input succeeds and reports unchanged.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`.

### #913 · cli: convert's revision resolution is explicit — asset extraction names a new output directory and records mappings as a gated adapter where unsupported, with no implicit Office conversion and no external asset fetch
**Action:** CLOSE as a duplicate, folded into #904  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #904's body (`gh issue view 904`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #904 as `From #913: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #904 or posted there. **Comment to post:** `Folded into #904: <clauses, one line each>. Verified 2026-10-19.`

### #916 · cli: default the author to neutral Jubarte — never infer a named person from Git configuration or mandate an author flag; --author attributes new marks and --by filters existing ones; the --datetime CLI spelling stands with a stable JSON date field (no second spelling), a generated plan freezes attribution and time, and a fixed datetime alone cannot promise byte reproducibility across ZIP metadata, compression, fonts, resources or engines
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. `jubarte edit --help | grep -i -A1 author` shows the default `Jubarte`.
2. `grep -rn 'user.name\|gitconfig\|git2' src/` finds nothing: no author is read from Git.
3. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #918 · cli: flags override environment, project and user presentation defaults — ambient configuration cannot override a saved plan's policy, accept revisions, authorize loss, disable a hash guard or add publication products; source-specific decisions need explicit request data; resolved options and configuration provenance appear in machine output
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #919 · cli: apply cannot override plan-owned author, date, tracking, existing-revision policy or selected decisions — a redundant --expect-sha256 must agree, --expect never overrides, and --save-plan freezes targets, policy and time while excluding output, report, emit, expect and dry-run options
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. Apply a saved plan whose `author` is `X` with `--author Y`: the CLI refuses (or warns and keeps X); `unzip -p out.docx word/document.xml | grep -o 'w:author="[^"]*"' | sort -u` shows only X.
2. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #920 · cli: shared edit policy options — tracking default on, existing-revisions default keep, repeated decision IDs form a selection not a sequence, --author defaults to Jubarte, --datetime requires a timezone, and --allow-loss permits one specific loss
**Action:** VERIFY and keep open (v1/v2 default divergence)  
**Double-check:**
1. `jubarte edit --help | grep -B1 -A3 existing-revisions` states the v1 default (`refuse`) and the PlanV2 default (`keep`). If only one is stated, the help clause is open.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; flag the divergence to Arthur in the comment.

### #921 · cli: editing-read and template selection inspect original-source final text and original review marks and freeze original identities before settlement — working text displays separately labeled against the original match, --context adds read-only original-source context only (never rewrite rows or editing scope), source-context observations are checked on the same basis, only current readiness is recomputed, and read cursors bind source digest, frozen selection, selected decisions, tracking and remaining-revision policy
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #921: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #922 · cli: shared edit output options — --emit defaults to document,patch,report, a clean derivative requires --clean-revisions all, --report is a JSON receipt not JSON Lines, --preview defaults to changed and bounded, and --force replaces only a named output file
**Action:** CLOSE as a duplicate, folded into #1001  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #1001's body (`gh issue view 1001`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #1001 as `From #922: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #1001 or posted there. **Comment to post:** `Folded into #1001: <clauses, one line each>. Verified 2026-10-19.`

### #923 · cli: shared preparation options — --dry-run runs the same required candidate checks, --expect-sha256 pins the full source hash, --expect supplies an expected-result contract, and --require-change demands a primary document change that derivatives alone never satisfy
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #923: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #924 · cli: edit exposes nine actions — replace, rewrite, insert, delete, format, fill, fields, template and apply; writes require -o FILE or --out-dir NEW_DIR, dry runs write no files and need no destination, and inline actions share the saved-plan preparation engine
**Action:** CLOSE as a duplicate, folded into #837  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #837's body (`gh issue view 837`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #837 as `From #924: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #837 or posted there. **Comment to post:** `Folded into #837: <clauses, one line each>. Verified 2026-10-19.`

### #925 · cli: render takes DOCX inputs and shares one full-document layout for PDF and PNG — page selection never skips needed layout; defaults are markup view, conventional markup, comment balloons, 100 DPI and font-policy warn; DPI spans 1 to 1200; PDF compression is deflate or none
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** `render` is today's `convert --to pdf|png`; a rename plus options
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #925 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #926 · cli: edit rewrite sets exact desired paragraph text where empty text retains the paragraph, and the inline adapter computes its working-text precondition
**Action:** CLOSE as a duplicate, folded into #837  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #837's body (`gh issue view 837`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #837 as `From #926: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #837 or posted there. **Comment to post:** `Folded into #837: <clauses, one line each>. Verified 2026-10-19.`

### #927 · cli: edit format applies typed formatting to explicit units and scopes, where false differs from restoring inheritance — exactly one of --run, --style, --paragraph, --list or --section
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** `format` is not in the eleven-kind text-and-review profile; v1 typed ops cover it
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #927 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #928 · cli: stage D — one candidate-preparation pipeline that serializes a provisional candidate, classifies the primary transaction's effective changes, selects actual primary bytes (S verbatim when truly unchanged), reopens and checks them for structural/package validity, exact postconditions, tracking/attribution and preservation; blocking invariants defined explicitly with baseline/introduced/unresolved findings reported separately and quality warnings never promoted; initial-profile expectations evaluated on the actual bytes with missing targets refusing even for text_absent; derivatives validated under their policies before commit; --require-change enforced against the primary only; dry-run returns phase validated, effect not_applied and predicted_effect writing nothing; validated bytes staged on the destination filesystem and rechecked at commit — the validation preventing a known bad publication is never optimized away
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #928: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #930 · cli: readiness is operation- and range-specific and lives in the ordinary read — it identifies exact blocking ranges and authoritative revision references without choosing acceptance or rejection, and a paragraph holding a pending insertion is not automatically uneditable
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #930: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #931 · cli: edit fill refuses absent-key clearing and unknown, duplicate, locked or unsupported control targets
**Action:** CLOSE as a duplicate, folded into #905  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #905's body (`gh issue view 905`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #905 as `From #931: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #905 or posted there. **Comment to post:** `Folded into #905: <clauses, one line each>. Verified 2026-10-19.`

### #932 · cli: edit fields updates supported caches or the on-open setting only — no external fetch and no macro execution
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. A plan with `update_fields: true`: `unzip -p out.docx word/settings.xml | grep updateFields` shows the on-open flag; `grep -rn 'reqwest\|ureq\|TcpStream' src/` finds no network use in document commands.
2. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #933 · cli: edit apply consumes a plan only — it never records into or deletes the plan file
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. `sha256sum p.json` before and after `jubarte edit FILE --plan p.json -o out.docx` is identical and the file still exists.
2. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #934 · cli: plan output products stay distinct — -o FILE saves a complete raw PlanV2 file plus a compact receipt; -o - puts only the raw plan on stdout and conflicts with --json/--jsonl; a full template requires -o FILE or explicit -o -; neither result protocol is accepted as an edit plan
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #934: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #935 · cli: render exposes final/original/markup display, markup styles, comments mode, physical and changed pages, font directories and report — known missing content defaults to refusal, a font-policy error blocks the complete requested publication before artifacts appear, the manifest records provenance and physical indices versus printed labels, and same-engine rendering is never Word equivalence
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** same as #925
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #935 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #937 · cli: apply resolves expectation references against the original source, maps them through the prepared transaction, evaluates on reopened actual primary bytes, requires exact one-to-one correspondence for paragraph predicates, refuses unmapped or unsupported predicates as non-passing, and EXPECTATION_FAILED blocks publication
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #937: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #938 · cli: stage D gate — the bookmark-crossing scenario is caught by shared final-candidate checks before primary publication; a candidate that normalizes away a requested exact change fails POSTCONDITION_FAILED; missing expected text cannot become unchanged success; dry-run and actual apply agree on validity and predicted effects given identical source/options; unchanged-output checks/expectations/IDs/digests refer to S itself; the envelope status/phase/effect contract holds with independent artifact committed flags; a post-commit reporting failure identifies already-committed artifacts and their effects
**Action:** CLOSE as a duplicate, folded into #835  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #835's body (`gh issue view 835`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #835 as `From #938: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #835 or posted there. **Comment to post:** `Folded into #835: <clauses, one line each>. Verified 2026-10-19.`

### #939 · cli: readiness records carry source hash, paired target, working policy, operation family, exact available text, blocker ranges, revision/comment references and safe next actions — zero-based exclusive-end offsets, if emitted, are observations only and never another address family
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #939: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #941 · cli: edit template never modifies the DOCX — selected acceptance/rejection is recorded and the receipt distinguishes selected from committed decisions
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #941: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #942 · cli: compare defaults to a read-only content report where -o writes a report and never a DOCX by extension — --redline is an explicit single-file product conflicting with bundle mode, bundle roles are manifest-defined, inapplicable scope options are usage errors, and rendered comparison products use the explicit comparison redline or projection
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** compare's default output is the redline DOCX people use; changing it is a breaking boundary
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #942 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #943 · cli: standalone check --expect requires the expectation's hash and references to name the inspected file — reusing pre-edit IDs on the output is invalid, an output-bound copy may be generated explicitly for later rechecking, the caller's input expectation file is never rewritten, and a passing predicate does not certify the complete requirement
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #943: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #945 · cli: never turn scan uncertainty into a refusal to inspect, nor an unsupported paragraph into a promised editability — readiness reports present structure and support without predicting an unwritten rewrite, and apply recomputes all constraints
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #945: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #946 · cli: stage E — read --for-edit exposes exact source hash, target pairs, logical text hashes, readiness and focused operation contracts with --track/--existing-revisions and continuation cursors bound to all working-basis inputs; bounded read previews stay distinct from complete editable values; edit template is a call to the shared selector/extractor and PlanV2 serializer with canonicalized container descendants, deduplicated cell-paragraph IDs, immutable source_context plus conditional/read-only/opaque inventory with separately reported recomputed readiness, explicit file/stdout forms, collision checks, atomic plan publication, compact receipts, and practical help and examples generated from the same supported schema
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #946: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #947 · cli: no --runs, --ids-only, --scaffold delete, --scaffold format, executable comment scaffolds or JSONL plan dialect is added to edit template
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #948 · cli: a saved plan is never silently truncated — generation refuses before publishing a partial plan; EMPTY_SELECTION is a zero-match diagnostic distinct from an applied unchanged plan; a read-only-only scope yields an explicit zero-rewrite-row inventory with an explicit zero-editable-row receipt
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #948: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #949 · cli: comparison reports default to the markup input projection and a redline from revised inputs requires explicit final or original projection — no arbitrary review-history merging, no implicit acceptance from presentation or extension, more than two inputs only for documented package comparison, visual baselines need a compatible source/render/font/page manifest with no silent rescaling, persisted reports carry the complete selected scope, and --exit-code returns 5 across every comparison scope
**Action:** VERIFY and keep open (compare silently accepts input revisions)  
**Double-check:**
1. `jubarte compare tracked_a.docx b.docx -o out.docx 2>err.txt; cat err.txt`: a warning says the inputs' revisions were accepted before comparing, or a flag selects final/original. No warning and no flag: the defect stands.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; propose a warning plus a flag, not a refusal.

### #950 · cli: text absence in a final paragraph is not secure redaction and containment is not a legal-quality test — original or deleted text may remain in revision history, and a separate expectations file proves nothing about authorship, legal adequacy or full task completion
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #950: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #951 · cli: read --for-edit uses PlanV2's exact text codec on the final logical view of the explicit working source — an incompatible original/markup view is a usage error, review marks never become anchor syntax, it exposes --accept-change/--reject-change/--existing-revisions/--track, and its result cannot authorize broader changes than those inputs declare
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #951: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #952 · cli: stage E gate — the whole generate→edit→apply→inspect loop runs the production compiler; a full selected-scope plan is complete regardless of receipt size; selection is frozen on original S; --context adds no editable rows; any admission limit or incomplete traversal is explicit; no run_operation/--runs/delete-scaffold/plan-format-JSONL option is advertised; generator-only success reports phase generated with effect not_applied even when the plan file committed
**Action:** CLOSE as a duplicate, folded into #849  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #952: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #954 · cli: every returned text value declares completeness — a truncated preview is never copyable as a complete rewrite payload, and a policy that cannot form a working basis returns a diagnostic with blockers instead of a usable hash
**Action:** VERIFY and keep open (completeness on every text value)  
**Double-check:**
1. `jubarte read FILE --json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("complete", d.get("completeness")))'` prints a value, and a truncated preview carries `complete: false`.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; engine side is #661.

### #955 · cli: one rewrite row per canonically addressed representable paragraph leaf with exact one-to-one correspondence — tables expand to cell paragraphs without overlapping whole-table rewrites, aliases dedupe, merged/split/removed mappings stay inventoried with their limitation, the read-only inventory distinguishes represented text, unsupported constructs and opaque regions, opaque regions get a real containing reference with a reason and no invented public ID, and comments are context unless addressed by a typed review operation
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #955: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #956 · cli: assemble joins at least two inputs in order without reconciling competing versions — list-file paths are relative to the list file and stdin paths to the current directory; defaults preserve sections and supported comments, revisions, media and notes; styles rename on conflict; options explicitly choose section handling, break type, comment removal, revision resolution and style conflicts; IDs remap without accidental list continuation; unsupported preservation refuses rather than hiding behind the name
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** `append` is `assemble`; the list-file rule can be tested on append if wanted
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #956 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #957 · cli: never silently relocate an edit — a supplied --at is never retargeted, repeated clauses and an explicitly incorrect target still refuse, candidate matches may be suggested but never silently selected, recovery offers bounded actual target text and candidate matches elsewhere, and generated hashes do not resolve the identical-paragraph ambiguity; a desired rewrite or refusal does not authorize accepting an earlier revision
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. A `replace` whose `find` occurs twice in the paragraph refuses as ambiguous (needs `occurrence`); a `--at`/paragraph target that does not hold the anchor refuses rather than searching elsewhere (tests in `tests/edit_anchor_*.rs`).
2. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #958 · cli: integrity reports baseline, newly introduced and unresolved findings separately — required validity and preservation invariants block publication; bookmark/comment range ownership survives paragraph insertion; the known protected-range regression is detected before publication; quality warnings stay non-fatal and are never blanket-promoted to failure; blocking rules are defined explicitly; unsupported checks report unavailable never passed; intended range ownership is preserved rather than substituting the agent's workaround ('after p33' and 'before p34' need not be equivalent)
**Action:** VERIFY and keep open (baseline vs introduced findings)  
**Double-check:**
1. `jubarte validate out.docx --original in.docx --json | head`: each finding carries a baseline/new status (Plan 6, #1256). Until then the defect stands.
2. Fold #966 into this body.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`.

### #959 · cli: a separate expectations file is not automatically independent evidence — agent-copied checks repeat mistaken task interpretation, and the CLI cannot infer provenance from a filename or a caller's assertion
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #959: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #960 · cli: a representable paragraph with a pending revision gets a conditional-readiness row — the compiler checks actual changed spans, never rewrites unchanged runs, and paragraphs lacking faithful representation become read-only source context with reasons
**Action:** CLOSE as a duplicate, folded into #843  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #843's body (`gh issue view 843`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #843 as `From #960: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #843 or posted there. **Comment to post:** `Folded into #843: <clauses, one line each>. Verified 2026-10-19.`

### #961 · cli: check profiles integrity, quality, layout or all with default integrity doing no layout pass — results state checks run, coverage, skipped and unavailable checks, findings and thresholds; quality and layout rules stay separate; --part is an explicit partial scope; example limits affect presentation not actual checking
**Action:** VERIFY, then park under #907 (the 1.0 breaking boundary); keep open  
**Why deferred:** `check` is today's `validate`; profiles are new
**Double-check:**
1. `target/debug/jubarte --help` (built from `origin/main`): the command or namespace this issue specifies does not exist. If it does exist, reclassify as VERIFY and stop.
2. Comment on #907: `Parked: #961 (<one line>)`; comment on this issue: `Parked under #907 until the 1.0 boundary; not before.`
**Never close** this issue from here; it closes with #907's decision.

### #963 · cli: integration and release gates inspect actual packages and independently specified effects on real DOCX — matching schema examples is insufficient and a receipt merely naming an operation verifies nothing; the corrected design-validator bug that rejected two disjoint replacements its schema accepted is why gates test composition and actual DOCX behavior; the fixture-only text oracle may remain an explanatory test but never gates revision handling, structural preservation or multi-operation composition
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #964 · cli: expectations are optional and separately supplied — missing requirements are met with optional supplied expectations, precise receipts and unassessed natural-language requirements, never a source-only template; they must not become compulsory paperwork or block ordinary editing behind a large task-evaluation framework
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #964: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #966 · cli: baseline tracking audit and author requirements examine new supported text changes and distinguish preserved earlier revisions — never implying comprehensive formatting or media coverage from a text-only audit, with expectation rules following the expectations section
**Action:** CLOSE as a duplicate, folded into #958  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #958's body (`gh issue view 958`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #958 as `From #966: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #958 or posted there. **Comment to post:** `Folded into #958: <clauses, one line each>. Verified 2026-10-19.`

### #969 · cli: repairs are supported conservative operations, rechecked after serialization, and always write a separate output — remaining findings can produce exit 2 with a committed repair, explicitly reported
**Action:** VERIFY and keep open (repair writes a separate output)  
**Double-check:**
1. `jubarte validate broken.docx --repair -o broken.docx` refuses the alias; `-o fixed.docx` writes and re-validates `fixed.docx` (exit 2 if findings remain).
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`.

### #972 · cli: sanitize selections are explicit and additive over documented metadata categories — document properties, revision-session IDs and comments, with an author alias applying only to supported recorded identities, the share-metadata preset's exact actions documented, and dry runs stating coverage gaps
**Action:** VERIFY and keep open (scrub selections (with #977))  
**Double-check:**
1. `jubarte scrub FILE --json -o out.docx`: the receipt lists categories removed, never a removed value (grep the receipt for a removed author's name: none).
2. Two selection flags together remove both (additive), not the last.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; fold #977 into it.

### #977 · cli: sanitize is metadata preparation, not secure redaction of prose, images, embedded files or history — content redaction needs a separate direct operation with leakage checks, and removed secrets never leak into comments, patches, previews or reports
**Action:** VERIFY and keep open (scrub is not redaction)  
**Double-check:**
1. `jubarte scrub --help` says metadata only, not redaction of prose, images or history; the receipt copies no removed value.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; fold into #972.

### #979 · cli: --comment flags compile to the attached_comment field only on tested combinations — unsupported combinations refuse rather than drop a comment
**Action:** CLOSE as a duplicate, folded into #833  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #833's body (`gh issue view 833`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #833 as `From #979: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #833 or posted there. **Comment to post:** `Folded into #833: <clauses, one line each>. Verified 2026-10-19.`

### #980 · cli: self-update --check never installs and document operations never check for updates — managed packages report their package-manager route, the command never replaces an unrelated executable, explicit install verifies artifact integrity with separately reported signature assurance, and noninteractive installation requires the explicit install choice plus --yes
**Action:** VERIFY on main; CLOSE if a test pins it, else write the test first  
**Double-check:**
1. `sha256sum target/debug/jubarte; jubarte self-update --check; sha256sum target/debug/jubarte` unchanged.
2. `grep -rn 'self_update\|check_update' src/bin/jubarte.rs`: only the `self-update` command reaches it.
3. Find the test that pins the behaviour (`grep -rn '<keyword>' tests/`). If none: write it on a branch from `origin/main`, open a PR (PR over PR, draft), and close this issue only when that PR is on main.
**Close only if** the behaviour holds and a test pins it. **Comment to post:** `Already true on main: <command and output>; pinned by <file::test>. Verified 2026-10-19.`

### #989 · cli: ZIP expansion, entry count, XML depth and size, resource size, regex computation and rendering time and memory have documented limits — document prose, comment bodies, filenames and properties stay untrusted data never treated as instructions, reserved display delimiters are escaped, and machine provenance stays separate
**Action:** CLOSE as a duplicate, folded into #667  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #667's body (`gh issue view 667`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #667 as `From #989: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #667 or posted there. **Comment to post:** `Folded into #667: <clauses, one line each>. Verified 2026-10-19.`

### #992 · cli: no document command implicitly invokes Office, LibreOffice, pandoc, a package manager, a browser or the network — only explicit self-update uses the network, resources are admitted from declared local roots, and external relationships are never fetched or executed
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #1001 · cli: product roles and side effects get explicit destinations and receipts — edits default to bundle artifacts document.docx, changes.diff and report.json plus a manifest; optional clean, PDF and PNG roles have explicit filenames and revision state in the manifest; the primary document stays included under --emit; the clean derivative accepts all revisions only with --clean-revisions all and is never an implicit engine working copy
**Action:** VERIFY and keep open (output surface owner)  
**Double-check:**
1. `jubarte edit FILE --plan p.json --out-dir DIR`: `ls DIR` shows `document.docx`, `changes.diff`, `report.json` (and a manifest if specified); `-o FILE` still works as the default single output.
2. Fold #922 and the `--out-dir` half of #924 into this body.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; `-o FILE` stays the default; `--out-dir` opt-in.

### #1015 · cli: plan carries a required non-blank author and a timezone-bearing ISO date; production parsing must validate calendar/time/offset values itself rather than relying on a JSON Schema consumer enabling format assertions
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1015: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1016 · cli: the end-to-end scenario fixture supplies real source pairs — a paragraph whose clean wording is 'Payment is due within 30 days.' with a pending 30→45 revision, an unaffected 'Late interest is 2%.' sentence with an attached comment, and one unrelated preserved revision — and examples never fabricate long revision IDs
**Action:** VERIFY and keep open (end-to-end fixture)  
**Double-check:**
1. `grep -rn 'within 30 days' tests/ fixtures/` finds the scenario fixture with the pending 30→45 revision; none means the fixture is still to build (keep open under #849).
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`.

### #1019 · cli: plan scope is the direct operation scope — kind all (whole document) or kind selected with at least one unique paired target and declared descendant rules; revision decisions live on a separate explicit effect scope
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1019: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1020 · cli: counts distinguish operations, logical revisions, carrier elements, root threads, replies, matched records, contextual records and changed paragraphs — never all called changes
**Action:** CLOSE as a duplicate, folded into #825  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #825's body (`gh issue view 825`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #825 as `From #1020: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #825 or posted there. **Comment to post:** `Folded into #825: <clauses, one line each>. Verified 2026-10-19.`

### #1023 · cli: stage 1 conformance before agent trials — real DOCX fixtures for clean paragraphs, received redlines, duplicate anchors, cells, nonbody stories, bookmarks, comments and unsupported objects; assertions on selected decisions, unselected-revision preservation, unchanged source-byte identity, attached anchors, thread conflicts, one-to-one rewrite gates, same-paragraph composition and exact whitespace/Unicode; refusal boundaries for stale sources, mismatched pairs, out-of-scope targets, missing expectations, empty anchors, unsupported tracking and changed baselines; unnecessary-blocking and missed-defect probes; fault injection before and after commit (alias collisions, report failure, cancellation, bundle publication); findings compared by mapped entity/range; native/python/wasm common semantics verified with honest exclusions; external Word/render checks only for actual interoperability claims
**Action:** CLOSE as a duplicate, folded into #849  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #849's body (`gh issue view 849`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #849 as `From #1023: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #849 or posted there. **Comment to post:** `Folded into #849: <clauses, one line each>. Verified 2026-10-19.`

### #1025 · cli: the initial text-and-review profile defines eleven operation kinds — rewrite, replace, delete, insert, insert_paragraph, delete_paragraph, comment, reply_comment, resolve_comment, edit_comment, delete_comment — each entry matching exactly one operation definition, with explicit done/include_replies booleans and attribution preservation on edit_comment
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1025: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1026 · cli: results distinguish operation postconditions, integrity coverage, supplied expectation outcomes and unassessed task semantics — a schema-valid plan, successful dry run, many edits, named author or no integrity findings is never labeled task complete; check output's final line states the check's scope (for example integrity passed with user requirements not assessed) and machine results convey the same distinction
**Action:** CLOSE as a duplicate, folded into #825  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #825's body (`gh issue view 825`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #825 as `From #1026: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #825 or posted there. **Comment to post:** `Folded into #825: <clauses, one line each>. Verified 2026-10-19.`

### #1028 · cli: errors identify code, operation pointer, source hash, resolved target, match count where applicable, bounded actual context and safe next actions, keep existing per-operation refusal detail, add retryable and commit-state fields, and an unknown publication state never authorizes retry with force
**Action:** CLOSE as a duplicate, folded into #841  
**Note:** keep the `retryable` and commit-state fields clause: post it on #841 as the residue
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #841's body (`gh issue view 841`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #841 as `From #1028: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #841 or posted there. **Comment to post:** `Folded into #841: <clauses, one line each>. Verified 2026-10-19.`

### #1029 · cli: insert takes an explicit kind and typed payload — nonempty text anchored at exactly one working-text position via exactly one of before_text, after_text or edge; edge excludes occurrence; position never infers kind
**Action:** CLOSE as a duplicate, folded into #821  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #821's body (`gh issue view 821`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #821 as `From #1029: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #821 or posted there. **Comment to post:** `Folded into #821: <clauses, one line each>. Verified 2026-10-19.`

### #1031 · cli: next-call suggestions are argument arrays with prerequisites and purpose — never executable shell strings built from document text; a pending-revision refusal offers inspect and decision choices without accepting for the caller, and receipts use the actual output path
**Action:** VERIFY and keep open (next-call suggestions are argv arrays)  
**Double-check:**
1. Edit a saved plan after changing its source: `jubarte edit FILE --plan stale.json --json` → the suggestion/`next` value is a JSON array of argument strings, not one shell string (PR #1120's regeneration hint is the case to check).
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`.

### #1034 · cli: --json emits one object including empty results and recognized-mode usage failures; --jsonl emits a header, typed records and exactly one terminal result whose absence means an incomplete stream; neither mixes logs, ANSI, progress or prose into stdout; --report is JSON irrespective of filename suffix
**Action:** CLOSE as a duplicate, folded into #825  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #825's body (`gh issue view 825`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #825 as `From #1034: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #825 or posted there. **Comment to post:** `Folded into #825: <clauses, one line each>. Verified 2026-10-19.`

### #1036 · cli: the four-operation schema stays illustrative — no separate production dialect may exclude mixed redlining batches
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #1037 · cli: insert_paragraph places a paragraph before or after an original target in its supported container — same-side inserts retain array order, empty paragraphs are meaningful structural insertions, insertion beside a deleted or unmappable anchor refuses, and paragraph insertions stay in the anchor's original container
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1037: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1038 · cli: the edit-plan initiative excludes arbitrary OOXML editing, move-paragraph, a generalized result-reference language, DOCX creation and Markdown import — each having separate preservation and usability requirements
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #1042 · cli: delete_paragraph is an explicit structural deletion that reports affected ranges and dependencies and executes only for faithfully supported tracking/container combinations
**Action:** CLOSE as a duplicate, folded into #831  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #831's body (`gh issue view 831`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #831 as `From #1042: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #831 or posted there. **Comment to post:** `Folded into #831: <clauses, one line each>. Verified 2026-10-19.`

### #1043 · cli: comment targets a working paragraph, a literal range (find with optional occurrence) or a same-story paragraph range (through) — never both find and through, occurrence requires find, and every paragraph covered by through must lie within direct scope
**Action:** CLOSE as a duplicate, folded into #833  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #833's body (`gh issue view 833`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #833 as `From #1043: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #833 or posted there. **Comment to post:** `Folded into #833: <clauses, one line each>. Verified 2026-10-19.`

### #1044 · cli: the plan schema validates shape only — source resolution, before hashes, conflicts, preservation and effects require the compiler
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

### #1046 · cli: stdout carries requested data and stderr diagnostics — explicit machine modes put structured failures on stdout; quiet never suppresses requested JSON, data, comparison output or errors; a single - is stdin only where help declares it, only one argument consumes stdin, and DOCX buffers under package limits rather than streaming ZIP mutation
**Action:** CLOSE as a duplicate, folded into #889  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #889's body (`gh issue view 889`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #889 as `From #1046: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #889 or posted there. **Comment to post:** `Folded into #889: <clauses, one line each>. Verified 2026-10-19.`

### #1047 · cli: expectations documents are closed objects with exactly schema_version (constant 1), source (64-char lowercase-hex sha256 binding one source revision), text_view locked to final, and a non-empty requirements array of keyed, described, check-carrying closed objects; the apply receipt records the expectation file's path and digest
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #1047: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #1048 · cli: the initial expectations profile is exactly three final-paragraph text predicates — equality, containment and absence — bound to the original source and verified one-to-one against the actual primary bytes; unmapped, merged, split, ambiguous or unsupported targets refuse, a paragraph that vanished never passes text_absent, text_equals allows empty text and contains/absent require nonempty
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #1048: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #1050 · cli: every value-taking option requires a value — --plan, --save-plan and --out-dir have no optional-value or inferred recording mode; options are order-independent; repeats accumulate only where documented; a target flag never begins another implicit operation
**Action:** CLOSE as a duplicate, folded into #889  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #889's body (`gh issue view 889`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #889 as `From #1050: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #889 or posted there. **Comment to post:** `Folded into #889: <clauses, one line each>. Verified 2026-10-19.`

### #1051 · cli: support -- as end of options with one operand for editing, two ordered for comparison and many ordered for assembly — long and interspersed options are declared extensions, not full POSIX conformance
**Action:** CLOSE as a duplicate, folded into #889  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #889's body (`gh issue view 889`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #889 as `From #1051: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #889 or posted there. **Comment to post:** `Folded into #889: <clauses, one line each>. Verified 2026-10-19.`

### #1053 · cli: the compiler rejects duplicate requirement keys because the schema's key pattern alone cannot
**Action:** CLOSE as a duplicate, folded into #845  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #845's body (`gh issue view 845`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #845 as `From #1053: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #845 or posted there. **Comment to post:** `Folded into #845: <clauses, one line each>. Verified 2026-10-19.`

### #1057 · cli: filters are OR within repeated fields and AND across fields; matched records differ from contextual parents and dependencies; revision/comment selection includes intersecting ranges; cursors bind source digest, view, filters and working decisions; partial results state known omissions
**Action:** CLOSE as a duplicate, folded into #839  
**Double-check:**
1. Read this issue's obligation; list every testable clause (a behaviour, a refusal, a field, a default, a code).
2. For each clause: find it in #839's body (`gh issue view 839`), in its PR if one exists, or in a test on `origin/main` (`grep -rn '<keyword>' tests/ src/`).
3. A clause found in none of those: post it on #839 as `From #1057: <clause>` before closing. If more than half the clauses are missing, this is not a duplicate: reclassify as VERIFY and stop.
**Close only if** every clause is in #839 or posted there. **Comment to post:** `Folded into #839: <clauses, one line each>. Verified 2026-10-19.`

### #1058 · cli: the versioned CLI defines exit codes 0 through 5, 64 and 124 — unchanged success and comparison differences exit 0, check threshold findings exit 2, refusals and invariant failures exit 3, fidelity policy failure exits 4, requested comparison differences exit 5, usage errors move to 64 as a declared breaking boundary, deadline-before-commit exits 124, detailed codes kept in JSON; an explicit repair may commit and still return 2 and a requested comparison report may exist with exit 5, neither permitting ordinary edit publication after a required check fails, with receipts stating both condition and artifact state
**Action:** VERIFY and keep open (exit-code table)  
**Double-check:**
1. `jubarte --help` or `docs/` lists 0–5, 64, 124 with meanings; `jubarte edit FILE --plan p.json -o existing.docx; echo $?` is 2 (PR #1165). Each code has one command that produces it; a code with none is an open clause.
**Close only if** the defect no longer reproduces on `origin/main`: then close with the exact command and output as the comment. **Otherwise** comment `verified open 2026-10-19 at <sha>: <one line of evidence>`; #1151 was its first fix.

### #1061 · cli: a Git external-diff adapter maps difference 5 to Git's 1, keeps 0, and maps actual failures to a fatal status outside 0 and 1 — raw --exit-code is not advertised as a drop-in Git convention
**Action:** CLOSE as commentary (no repository change)  
**Double-check:**
1. Read the body. It must name no file, flag, error code, schema field or test to add or change. If it names one, it is not commentary: reclassify (FOLD or VERIFY) and stop.
2. Confirm the point is already recorded: grep a distinctive phrase of it in `archive/planning/jubarte-cli-evidence-review.md` and `docs/design/cli-v2/`. If it is recorded nowhere, add one sentence to the evidence review in a docs PR (PR over PR) before closing.
**Close only if** both hold. **Comment to post:** `Commentary on the supplied reports; recorded at <file:line>. No repository change. Verified 2026-10-19.`

## Coverage

URGENT rows: 47 (all above). cli-titled issues: 198 = 13 standalone + 83 with a verdict + 102 without. Every number in `/tmp/cli_issues.tsv` has a block; the generator asserts it.
