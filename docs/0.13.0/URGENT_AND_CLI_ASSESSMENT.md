# URGENT audit rows and cli-titled issues: assessment

Date: 2026-10-19. Checkout `59d8d96c`; `origin/main` at `b86fe5ac`; stack
top `fix/win-wrote-path-1244` (#1248) at `9dac2916`. Sources:
`/tmp/urgent.txt` (47 bodies), issue #1216 (per-row triage of
2026-10-18), `/tmp/cli_issues.tsv` (198 cli-titled issues),
`/tmp/cli_verdicts.tsv` (the 83 recorded design-review verdicts),
`/tmp/cli_noverdict.txt` (the 115 without one).

Companion: `VERIFY_AND_CLOSE.md` turns every row below into a
double-check procedure for an agent.

## Headline

46 URGENT rows plus the tracker. Four are done, nine are fixed on the
open PR stack, two are truly open, seventeen are partly done, thirteen
are umbrellas with no gate of their own. The nine "fixed" rows live on a
stack of 93 chained PRs whose base had not merged, so none of them
protected a binary anyone ran. The urgency was in merging the ZIP batch
(#1233 through #1248), not in more fixes.

## URGENT audit rows

| Issue | Where it stands (2026-10-19, before the merge) | Opinion |
|---|---|---|
| #612 D13 atomic publication | done on PR #1122 | Close when #1122 reaches main. |
| #666 D35 export confinement | done on PR #1122 | Close when #1122 reaches main. |
| #655 N03 docx2typed patcher | done, nothing imports it | Close now. |
| #671 ZIP-03 cargo-deny exceptions | done | Close now. |
| #645 ZIP-14 decompression limits | PR #1233 | Close when merged. |
| #649 ZIP-13 counts before indexing | PR #1233 | Close when merged. |
| #670 ZIP-32 raw preflight | PR #1233 | Close when merged. |
| #644 ZIP-17 structural consistency | PR #1241 | Close when merged. |
| #639 ZIP-16 special entries | PR #1242 | Close when merged. |
| #594 ZIP-01 advisory remediation | PR #1237 | Close with #605 when merged. |
| #617 J05 Markdown asset loading | PR #1238 | Close when merged. |
| #650 N01 quadratic attributes | PR #1246 | Close when merged. |
| #604 ZIP-10 ZIP32 boundaries | PR #1247 | Close when merged, after the API fix (option C: enforce `max_entries` in `strict_to_transitional_docx_within`; the plain function keeps returning bytes). |
| #592 ZIP-27 update authenticity | open, code ready, key never rolled out | The one row only Arthur can close. Until the key ships, self-update is unauthenticated: the most serious open item. |
| #640 ZIP-12 raw filename decoding | open, size M | Keep urgent. The one open ZIP row with a real spoofing path (the Unicode Path extra field overriding the raw name). Do it after #600. |
| #600 ZIP-15 duplicate identities | partly | Keep. #1237 fixed the two-header mismatch; duplicate central names and case-folded OPC part collisions remain. Depends on #646. |
| #646 ZIP-11 OPC part-name policy | partly | Keep. Precondition for #600. |
| #643 ZIP-47 strict XML before metadata | partly, M/L | Keep but narrow to the content-types and rels readers. The main DOM is already strict. |
| #647 ZIP-43 hostile corpus | partly, L | Downgrade from urgent to "gate for the ZIP batch". A corpus is evidence, not a fix; every PR in the batch ships its own probes. Collect them into one directory and one walking test. |
| #657 ZIP-18 compression and encryption | partly, S | Finish. Refusing encrypted entries and unsupported methods with a named code is ring-1-sized. |
| #668 ZIP-19 ratio heuristics | partly, M | Keep the Markdown-route aggregate cap here. Move the repeated-work budget to #651 so two rows do not own it. |
| #651 D17 bounded XML and semantic work | partly, L | Keep urgent. The real remaining bomb row; #722 and #668 both point at it. |
| #658 N02 namespace map cloning | partly, M | Downgrade. Performance, not safety, unless a measurement shows retained memory above budget. |
| #656 ZIP-28 updater extraction | partly | Finish the download and inflation caps and the duplicate check here. Signatures belong to #592. |
| #662 DX02 native candidate verification | partly | The missing piece is the per-part changed/unchanged receipt: Plan 3 of the 0.13 plan (#1253). Fold it there. |
| #605 ZIP-02 advisory mapping | partly, S | Update the three stale inventory places, record the new advisory. Close together with #594. |
| #654 ZIP-44 dependency gates | partly, S | Make the ZIP-drift check fail CI or delete it. A warn-only gate is noise. |
| #660 J01 strict Markdown edit completion | partly | Keep urgent: the "round trip silently drops an edit" class. The test pinning the anti-gate must be flipped, not deleted. |
| #661 D16 completeness reporting | partly, L | Keep as the engine owner; cli #954 is the CLI-surface owner. |
| #667 D18 text-output limits | partly, M | Keep. Copy the for-edit route's shape to read, snapshot, changes and markdown. Fold cli #989 into it. |
| #672 D20 asset generation bounds | partly, M | Keep; urgent only for the WASM and MCP exposure. |
| #616 N13 redaction leak scan | partly | Decide once: either the scan is complete and typed, or the docs say scrub is not redaction (cli #977). Pick the second and downgrade. |
| #669 N12 legacy DOC reader | partly, M | Keep. Size caps and FIB sanity checks are cheap. |
| #703 R01, #711 R03, #722 R05, #726 R40, #730 R07, #737 R09, #738 R42, #739 R10, #675 R32 | umbrellas | Convert each to a checklist of its owner rows; close when the owners close. |
| #727 R06, #682 R34, #693 R36 | umbrellas with an unowned gap | File the gap (zip-slip residue, HTML/LaTeX residue, N14/N17) as its own row, then close the umbrella. |
| #733 R41 panic-free robustness | umbrella | Keep: a fuzz or Miri run that fails on panic is a real standing gate. |
| #1216 tracker | open | Keep. Add a "closes when" column; prune rows as PRs merge. |

## cli bundle: the 83 issues with a recorded verdict

Act on the verdicts as recorded.

- Close as commentary (21): #872, #873, #874, #875, #876, #879, #882,
  #885, #894, #900, #911, #929, #940, #944, #953, #974, #978, #982,
  #986, #1035, #1040.
- Close as duplicates after copying any testable clause into the owner
  (45). Into #816: #877, #881, #884, #893, #903, #912, #914. Into #821:
  #987, #990, #1003, #1010, #1012. Into #823: #965. Into #825: #1005.
  Into #827: #1014. Into #829: #996. Into #831: #973, #995, #997, #999,
  #1000, #1002, #1004, #1008, #1011. Into #833: #971, #975. Into #835:
  #976, #981, #984, #988, #993, #998. Into #837: #967. Into #843: #886,
  #983, #985, #994. Into #847: #908. Into #849: #1018, #1033. Into
  #873: #1021. Into #912: #962. Into #973: #1039. Into #999: #1022.
- Keep only the residue (16 PARTIAL): #936 (restore `--expect-sha256`,
  `--report`, `--force`, `-o` to the template option surface), #1041
  (NO_COLOR, TERM=dumb, broken pipe), #1049 (binary stdout needs an
  explicit `-o -`), #1007 (v1 recorded digests stay valid), #1009
  (merged-paragraph rewrite is forbidden), #1013 (keep the released
  refusal boundaries), #1017 (receipt contract without harness conduct),
  #1052 (report which requirements had no check), #1056 (one compiled
  model drives clap, schema and help; Plan 22's basis). #915, #968,
  #970, #1024, #1027, #1032, #1045 are test-matrix rows whose clauses
  all have owners: fold into #849 as a test list and close.
- Wrong (1): #1006 says exactly eight top-level fields; it is eight
  required plus two optional. Correct the text and fold into #821.

## cli bundle: the 115 without a verdict

Key: Close, Fold into N, Keep, Defer (needs a command that does not
exist), Done (true on main or on the stack).

| Issue | Opinion |
|---|---|
| #870 | Close. A summary of four decisions owned by #835, #831, #957 and #897. |
| #871 | Fold into #829. The publication gate; the explicit allowance flag is the test. |
| #878 | Defer with #909. The operations exist as plan ops; the `review` namespace does not. |
| #880, #883, #895 | Defer. `create` is gated new engineering by the proposal's own words. Close #895; it only says "do not build yet". |
| #887 | Fold into #1028 (retryable and commit-state fields). |
| #888 | Park under #907. The twelve-domain tree is a rename of the surface; not piecemeal, not before the 1.0 boundary. |
| #889 | Keep as the one "root options" issue; merge #1046, #1050, #1051 into it. Help clauses go to #1273. |
| #890, #891 | Fold into #1273 (Plan 22). |
| #892 | Fold into #839. The `--pages` clause needs its own decision: it contradicts "no layout pass". |
| #896 | Fold into #835. |
| #897, #898, #924, #926 | Fold into #837 (typed actions, PR #1078). Done on the stack except `--out-dir` semantics (#1001). |
| #899 | Keep, small. UTF-8 exactness and "plain text is never Markdown" are cheap tests. The image half closes with #617. |
| #901, #934, #941, #948, #955, #960 | Fold into #843 (edit template, PR #1081). |
| #902 | Done on the stack: `edit template` is the command. Close with #843. |
| #904, #913 | Keep as one convert contract issue. Inference refusal is testable now; the preserved/approximated/dropped report is the new work. |
| #905 | Fold into #821 (pinned v1 adapter). |
| #906 | Done. v1 rejects unknown fields. One test, close. |
| #907 | Keep. The 1.0 breaking-boundary issue. #888, #942, #956, #961, #972 park here. |
| #909 | Defer under #907. |
| #910 | Keep. Each clause is testable against today's accept and reject; `--allow-empty` is the only new flag. |
| #916 | Done. The default author is Jubarte. One test, close. |
| #918 | Close. No ambient configuration surface exists, so the rule is vacuous. Re-file with the surface. |
| #919, #933 | Done on the stack (immutable plan apply, PR #1078). Close. |
| #920 | Keep, flagged. The proposal says `--existing-revisions` defaults to keep; v1 defaults to refuse and #906 forbids reinterpreting v1. So v2 keeps, v1 refuses; the help must say so. |
| #921, #930, #939, #945, #946, #951, #1057 | Fold into #839 (read `--for-edit`, PR #1075). Mostly done on the stack. |
| #922 | Fold into #1001. |
| #923, #928, #938 | Fold into #835 (PreparedEdit, PR #1076). Done on the stack; #938 adds the bookmark fixture from #827. |
| #925, #935 | Defer as one `render` issue. PDF and PNG exist under convert; the rest is a rename plus options. |
| #927 | Defer. `format` is not in the eleven-kind profile; v1 typed ops cover it. |
| #931 | Fold into #905. Small refusal tests on the v1 control ops. |
| #932 | Done. Fields never fetch; one test. |
| #937, #943, #950, #959, #964, #1047, #1048, #1053 | Fold into #845 (`--expect`, PRs #1080 and #1084). Done on the stack; #950 and #959 are doc sentences. |
| #942 | Park under #907. Do not change compare's default: the redline DOCX is the product people use. |
| #947, #1036, #1038, #1044 | Close. "Do not add" lists, scope statements and a tautology. |
| #949 | Keep as a decision issue. The comparer accepts revisions in both inputs silently, as PowerTools does. A warning plus a flag, not a refusal, or the bench breaks. |
| #952, #1023 | Fold into #849 as the fixture matrix. |
| #954 | Keep. The CLI owner of completeness; #661 is the engine owner. |
| #956 | Park under #907. `append` is `assemble`. |
| #957 | Done. Ambiguous and overlapping anchors refuse today. One regression test, close. |
| #958 | Keep, medium. The baseline versus introduced split for validate. Fold #966 into it. |
| #961 | Defer under #907. `check` is `validate`; profiles are new. |
| #963 | Close. Already policy in AGENTS.md. |
| #969 | Keep, small. Repair recheck after serialization and separate output are testable now. |
| #972, #977 | Keep as one small `scrub` issue: additive selection, and no removed value echoed in a receipt. |
| #979 | Fold into #833 (PR #1079). Done on the stack. |
| #980 | Done apart from signatures (#592). One test, close. |
| #989 | Fold into #667. |
| #992 | Close. True today; not testable beyond a doc sentence. |
| #1001 | Keep as the owner of the output surface; fold #922 and the `--out-dir` half of #924 into it. `-o FILE` stays the default; `--out-dir` is opt-in. |
| #1015, #1019, #1025, #1029, #1037, #1042, #1043 | Fold into #821 or #831. The PlanV2 model already holds them. |
| #1016 | Keep, small. A concrete fixture ticket for #849. |
| #1020, #1026, #1034 | Fold into #825 (the envelope). |
| #1028 | Fold into #841 (PR #1074), keeping the retryable and commit-state clause. |
| #1031 | Keep and verify. PR #1120 prints a "focused regeneration command"; in JSON mode it must be an argv array. |
| #1058 | Keep as the owner of the exit-code table. #1151 (PR #1165) is its first fix. |
| #1061 | Close until someone asks for git integration. |

Standalone cli issues:

- #816 tracking: keep.
- #837, #1082, #1093, #1096, #1140, #1151, #1222, #1224, #1244: fixed
  on the stack (PRs #1078, #1085, #1103, #1112, #1144, #1165, #1226,
  #1236, #1248). Close each once its commit is on main.
- #863 ZIP-45 feature isolation: keep, before 1.0.
- #1059 CodeQL alerts: do it now; CI hygiene, small.
- #1273 Plan 22 help: keep; ours.
