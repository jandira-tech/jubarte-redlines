@coderabbitai plan across the 23 plan issues this issue lists, against `src/edit.rs`, `src/edit/tracked.rs`, `src/changes.rs`, `src/validate.rs`, `src/cli.rs` and `src/markdown/from_docx/critic.rs` on `main` (59d8d96c); check that every plan's dependencies precede it in the stack; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different order or split if the code supports it.

# jubarte 0.13: contract-redlining plan (tracking)

What should change so jubarte is genuinely useful for redlining a counterparty's draft: edit inside their insertions as Word does, strike what we reject visibly, land work as it is made, read a 1,000-revision contract from two screens, and keep every output Word-valid with the right text under Accept All, Reject All and reject-ours. The evidence is 0.12.0's bench trials (23 observed problems); the designs are in the planning folder's `PROPOSALS_PLAN.md` (chosen alternative per change, scored) and the full implementation plan with conventions, red tests and Word experiments E1–E7 is `IMPLEMENTATION_PLAN.md` there.

## Rules

- One branch per plan, stacked PR over PR in the order below; nothing is merged until asked.
- Red test first, quoting the exact error text or XML shape; then `cargo fmt`, `cargo clippy --all-targets -- -D warnings` (no `#[allow]`), `cargo nextest run --test NAME`, `jubarte --help`.
- Word is the oracle: every new XML shape is opened in Word alone (normal / Accept All / Reject All) through the bench's `word_pdf.py`; a shape Word refuses becomes a Ring-1 check with a broken probe before the fix.
- CodeRabbit reviews one PR per hour: ask for a full review once a PR is complete.

## Stack

| step | plan issue | branch | stacks on |
|---|---|---|---|
| E1 | #1251 plan 1: edits inside another author's insertion (land #1136) | `fix/edit-inside-ins-1136` | origin/main |
| E1 | #1254 plan 4: refusal hints that name both decisions and the visible route (#1108) | `fix/edit-inside-ins-1136` | origin/main |
| E2 | #1252 plan 2: `strike` and `restore` (visible rejects) | `feat/edit-strike-restore` | E1 |
| E3 | #1253 plan 3: `--skip-failed`, the `next:` line and the output digest | `feat/edit-skip-failed` | E2 |
| E4 | #1255 plan 5: option conflicts, and JSON-shaped usage errors (#1158) | `fix/read-option-conflicts-1158` | E3 |
| E4 | #1256 plan 6: `validate --baseline`, and warnings printed once | `feat/validate-baseline` | E3 |
| E4 | #1257 plan 7: markers stay beside a new insertion (#1156) | `fix/markers-beside-insertion-1156` | E3 |
| E5 | #1258 plan 8: plan schema errors that teach, and three aliases | `feat/plan-errors-aliases` | E4 |
| E5 | #1269 plan 18: settled-id messages, and resolving through the mapping | `fix/settled-id-messages` | E4 |
| E6 | #1260 plan 10: report what a settle removes; replies in tracked-deleted text (#1159) | `fix/settle-reports-1159` | E5 |
| E6 | #1263 plan 13: insert into an empty paragraph | `fix/insert-empty-paragraph` | E5 |
| E6 | #1264 plan 14: stable comment ids; no editing another author's comment | `fix/stable-comment-ids` | E5 |
| E7 | #1261 plan 11: one notion of a change | `feat/one-change-notion` | E6 |
| E7 | #1268 plan 17: view attribution (#1111) | `fix/view-attribution-1111` | E6 |
| E8 | #1262 plan 12: `replace` keeps the formatting it keeps; lenient anchors keep the document's characters | `fix/replace-keeps-formatting` | E7 |
| E9 | #1266 plan 15: `insert_paragraph` with `style` copies nothing else; structural ops emitted directly | `fix/structural-direct-emission` | E8 |
| E9 | #1267 plan 16: numbering side effects | `fix/numbering-side-effects` | E8 |
| E9 | #1270 plan 19: a new paragraph beside a pending-deleted mark | `fix/paragraph-beside-deleted-mark` | E8 |
| E9 | #1271 plan 20: small fixes from the repro (tabs in `changes --json`; `validate --original` and inherited revisions) | `fix/changes-tabs-foreign-author-1192` | E8 |
| E10 | #1259 plan 9: summary-first reading | `feat/summary-reads` | E9 |
| E10 | #1273 plan 22: help that fits a clock | `feat/help-plan-errors` | E9 |
| E10 | #1274 plan 23: skill template (bench side) | `(bench side: jubarte-syntax / neurotic_docx_bench)` | E9 |
| E11 | #1272 plan 21: revision-aware compare (three-way) | `feat/compare-three-way` | E10 |

## Word experiments that gate shapes

E1 their inserted mark deleted by us (plans 1, 2) · E2 deletion inside a `moveTo` (1, 2) · E3 Enter beside a pending-deleted mark (19) · E4 retyped paragraph break (2, 19) · E5 editing another author's comment (14) · E6 duplicate `durableId` (16) · E7 Word's Compare of `Step_1` against `Step_2` (21). Run each through `word_pdf.py` normal, `--accept-all`, `--reject-all`; keep the files beside the `Step_2` fixtures.

## Decisions flagged for Arthur

- Plan 17 stops printing the dialect's two-author substitution (`{~~a~>b~~}{>>r1 @AC<<}{>>r2 @JD<<}` becomes read-only).
- Plan 15 widens direct emission to every `existing_revisions` mode and re-pins comparer outputs.
- Plan 14 stops renumbering comment ids on a bare accept/reject.

## Existing issues referenced, not duplicated

#1136, #1108, #1158, #1161, #1173, #1156, #827, #1159, #1192, #1150, #1134, #1111, #1135.
