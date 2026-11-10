# Findings — compare (redline) Word worker

Scope: `/tmp/welogs/logs/compare-worker.log` (19,673 lines, 2026-10-04 -> 2026-11-05 06:58:54) and `compare-stderr.log` (empty, 0 bytes). Server/convert logs not read, per task boundary.

Population: 3,307 unique job ids; 3,280 `kind=compare` + 30 `kind=compare_pdf` completions with `word_ms`; 212 jobs rejected (`word_failed` x211, `compare_failed` x1) -> **overall rejection rate 6.4%**. Duration: p50 4.46s, p95 11.3s, p99 18.7s, max 120.2s, mean 5.3s.

## SUMMARY — top 5

1. **The `lock_fields` hardening step rejects 6.3% of all jobs (211/3,307), every single day.** `worker.py:507` sets `fields.Locked = True` on the uploaded original right after `open_original`; on documents with protection/locked areas Word refuses (COM error 37373) and the worker rejects the whole job as `word_failed` even though the document opened fine. This is 99.5% of all rejections (211 of 212) and the single largest correctness defect in this endpoint.
2. **The failure->recovery path leaves a stale COM proxy and a busy Word: after any COM failure the worker cannot quit (`quit failed` AttributeError x23), and the first job on the fresh Word instance fails with "Call was rejected by callee" (x7).** Worst burst: 2026-10-31 18:21:42-18:25:04 — one upload (job `33436d2a...`) failed twice (word_failed then compare_failed), job `6ff24e6d...` failed twice in a row, 3 Word restarts in 4 minutes, 4 "could not close a leftover document".
3. **The 120s kill threshold is enforced (no job exceeded 120.2s) but is never logged by the worker — kills are only inferable from `word_ms=120,190/120,200` plus an RPC-failed COM error.** Two jobs hit it: 2026-10-14 17:55:28 (`open_revised` hung 118,454ms) and 2026-10-15 06:16:08 (`compare` phase 116,496ms then `-2147023170 'The remote procedure call failed.'`). 5 jobs total ran >60s.
4. **38 document-open refusals, all clustered in the old worker window (10-08->10-12): 32 x "The file appears to be corrupted" (hresult 25272) + 6 x "Word experienced an error trying to open the file" (24601)** — 4 on `revised.docx`, 2 on `original.docx` (dialog text names the file). In that old worker these produced `ERROR Word COM failed` with **no `worker rejected` line**, unlike the new worker — client-visible classification of the same condition changed across the upgrade (exact HTTP outcome not verifiable from this log).
5. **Word lifecycle churn and blind spots: 38 Word (re)starts, of which 8 occurred in one 3-hour window (10-13 01:04-04:00) with no error before them (idle recycles); a 10-day traffic gap (10-19->10-27); `compare-stderr.log` is 0 bytes; and no log line exists for template detach, external-relationship handling, or template-stamp removal in this worker.** After the claimed 2026-11-05 ~06:00 reboot there is *no* Word-start marker (quiet 05:57:32->06:11:10), jobs resumed normally (2.8-8.5s), and the only defects after 06:00 were the two 06:16/06:18 `field lock failed` rejections — item 1 persists right through the last log line.

## EVIDENCE (verbatim, timestamps as logged)

### E1 — lock_fields rejects protected documents (top defect)
```
2026-11-05 06:16:16,898 ERROR field lock failed
  File "C:\word-endpoint\word_endpoint\worker.py", line 486, in lock_fields
    _lock_fields(document)
  File "C:\word-endpoint\word_endpoint\worker.py", line 507, in _lock_fields
    fields.Locked = True
pywintypes.com_error: (-2147352567, 'Exception occurred.', (0, 'Microsoft Word', 'This method or property is not available because the object refers to a protected area of the document.', 'wdmain11.chm', 37373, -2146823683), None)
2026-11-05 06:16:17,281 INFO worker rejected the job: word_failed
```
- Counts: `field lock failed` x211; `worker rejected the job: word_failed` x211 (1:1); plus 1 `compare_failed`. Message split of the 211: "protected area of the document" x94, "current selection is locked for format changes" x77, "the document is locked" x38, 'Call was rejected by callee' x1, RPC-failed x1.
- Rejections occur every active day (10-13: 18 ... 11-05: 4); e.g. 2026-11-04 23:48:31,792 and 23:49:45,792 — consecutive uploads, both lost.
- The rejection fires *after* a successful open (`phase=open_original ms=1061` precedes it at 06:16:16,273), i.e. the file itself is fine.

### E2 — broken recovery: stale proxy, callee rejections, leftover documents
```
2026-10-31 18:21:42,083 INFO worker rejected the job: word_failed
2026-10-31 18:21:42,196 ERROR quit failed
    word.Quit(SaveChanges=_DO_NOT_SAVE)
AttributeError: Word.Application.Quit
2026-10-31 18:21:44,710 INFO Word option CheckSpellingAsYouType disabled
2026-10-31 18:21:45,436 INFO job id=33436d2a... phase=open_original ms=536
2026-10-31 18:21:46,685 INFO could not mark the document unchanged
2026-10-31 18:21:47,847 INFO worker rejected the job: compare_failed
...
2026-10-31 18:24:57,586 INFO job id=6ff24e6d... phase=open_original ms=833
2026-10-31 18:24:57,587 ERROR Word COM failed
    full_name = str(document.FullName)
pywintypes.com_error: (-2147418111, 'Call was rejected by callee.', None, None)
```
- `quit failed` x23 (always `AttributeError: Word.Application.Quit` — proxy points at a dead Word); `could not silence alerts during quit` x9; `could not close a leftover document` x8.
- 'Call was rejected by callee' x7 total, all at `open_document`/`lock_fields` **immediately after a fresh Word start** (18:24:57, 18:25:00) — the same job id retried twice failed twice, recovery only at 18:27:22.

### E3 — 120s threshold: enforced, invisible, preceded by hangs
```
2026-10-15 06:14:11,944 INFO job id=ab6b176b... phase=open_revised ms=2311
2026-10-15 06:16:08,443 INFO job id=ab6b176b... phase=compare ms=116496
2026-10-15 06:16:08,446 ERROR Word COM failed
pywintypes.com_error: (-2147023170, 'The remote procedure call failed.', None, None)
2026-10-15 06:16:08,515 INFO job id=ab6b176b... kind=compare word_ms=120190
```
```
2026-10-14 17:53:30,031 INFO job id=c9af75c7... phase=open_original ms=1716
2026-10-14 17:55:28,497 INFO job id=c9af75c7... phase=open_revised ms=118454
2026-10-14 17:55:28,498 INFO worker rejected the job: word_failed
2026-10-14 17:55:28,506 INFO job id=c9af75c7... kind=compare word_ms=120200
```
- Exactly 2 jobs >=120s (both clipped at 120,190/120,200ms — the watchdog kill), 5 jobs >60s (60.8s, 62.7s, 86.8s succeeded normally). No job exceeded the threshold.
- The 10-14 case was preceded *in the same job* by a `field lock failed` (17:55:28,404, 'callee' variant) — E1 and E3 chained.
- No explicit kill/restart log line exists anywhere: `kill` appears 0 times in this log; restarts must be inferred from the paired `Word option ... disabled` lines.

### E4 — open refusals (old worker only)
```
2026-10-08 22:02:53,409 ERROR Word COM failed
  File "C:\word-endpoint\word_endpoint\worker.py", line 380, in open_document
  File "<COMObject <unknown>>", line 5, in Open
pywintypes.com_error: (... 'The file appears to be corrupted.', 'wdmain11.chm', 25272, -2146822496), None)
```
```
pywintypes.com_error: (... 'Word experienced an error trying to open the file.\n... (C:\word-endpoint\work\...\revised.docx', 'wdmain11.chm', 24601, -2146823167), None)
```
- 25272 x32, 24601 x6 (4 `revised.docx`, 2 `original.docx`), all 10-08->10-12, none after the ~10-13 redeployment. These often arrive in immediate pairs (e.g. 22:02:53,409 + 22:02:55,765) — consistent with one bad upload failing, then its retry/pair failing too.
- Old-worker tracebacks run under `.tools\python\...`, new ones under `.tools\python315-bench\...` (229 vs 12 lines) -> worker + runtime redeployed ~2026-10-13 01:04, which also introduced `kind=compare_pdf` (first: 2026-10-13 01:06:32,120).

### E5 — lifecycle churn, gaps, observability holes
- 38 Word starts (`Word option CheckSpellingAsYouType disabled` x38, always paired with `CheckGrammarAsYouType`). 8 fall on 10-13 01:04:37->04:00:23 with **no** preceding error and ~10-60min idle gaps (idle recycles); each restart's next job pays a slow first open (e.g. 01:05:45 `open_original ms=4717` vs p50 ~1s).
- Zero traffic 2026-10-19 -> 2026-10-27 (10 days, log resumes 10-28 17:21). Cause not determinable from this log (rotation? outage? out of slice).
- `compare-stderr.log` = 0 bytes over the whole window while the worker raised 285 ERROR tracebacks -> stderr capture for this worker is broken or unused.
- Settings application is logged only as the two option-disable lines (x38 each). **No log line in this file matches template detach, external relationships, or template-stamp removal** (`rg -i 'template|stamp|detach|external|relationsh|sanitiz|scrub'` hits only traceback boilerplate `FileName=str(path.resolve()),`). Memory hints: none — "memory" occurs only inside Word's 24601 dialog text.
- Startup race in old worker: `ERROR could not determine the WINWORD pid` x2 (10-04 00:17:58, 00:33:37).
- 2026-11-05 ~06:00 reboot: last pre-gap job 05:57:32,387, next 06:11:10,888, **no Word-start marker in between** — for this worker the reboot (if it happened at 06:00) left no lifecycle trace; post-06:00 behavior is normal (word_ms 2.8-8.5s) except the E1 rejections at 06:16:17 and 06:18:43. One further Word start at 06:56:42 with no error before it (idle recycle, ~13min after the 06:43 job).

## ROOT-CAUSE HYPOTHESES

- **H1 (E1):** `lock_fields` assumes every document's `Fields` collection is writable. Uploaded documents with document protection (`w:documentProtection`), content controls in protected sections, or signed/IRM-locked areas make `fields.Locked = True` throw; the worker treats a *hardening nicety* as fatal and rejects a document Word itself opened without complaint. Confidence: high — the failing call and its line number are in every traceback; three distinct Word messages all map to protection semantics.
- **H2 (E2):** After a COM failure the worker's `word` proxy is stale (the underlying WINWORD died or was killed), so `quit_word` raises `AttributeError` and leftover documents persist into the next instance; additionally the worker dispatches the next job the instant Word's options lines are written, while Word's COM server is still not accepting calls -> 'Call was rejected by callee'. No warm-up or callee-retry exists. Confidence: high for the sequence (timestamps within 2-4s), medium for the "not ready yet" mechanism (inferred).
- **H3 (E3):** An external watchdog kills WINWORD at 120s; the worker's `_timed_call` then surfaces RPC-failed. The worker itself has no phase-level deadline below 120s and logs nothing at kill time — the phase timer (116s, 118s) shows Word genuinely hung on that input (pathological comparison), not a slow leak. Confidence: high on enforcement (exact 120,19x/120,20x clipping), medium on watchdog ownership (this log never names the killer; supervisor log out of scope).
- **H4 (E4):** Old worker (pre-10-13) opened uploads without the new pre-open acceptance checks, so genuinely corrupt/protected uploads reached Word and produced 25272/24601; after redeployment these vanished — either the new worker's rejection (possibly the missing template/external-relationship handling, E5) now filters them server-side before Word, or the traffic changed. Confidence: low-medium; the sanitize/acceptance steps are not logged here at all, so their existence can only be inferred from the error disappearing.
- **H5 (E5):** Idle Word recycling is by design (keep-alive), costing one slow open per recycle; the missing stderr and missing kill/sanitization logging are logging-coverage gaps, not absence of the behavior.

## CONCRETE FIXES

1. **Make `lock_fields` best-effort.** Change: catch COM 37373 in `_lock_fields` (worker.py:507) and continue the job with fields unlocked (log `field lock skipped`). Effect: ~211 of 212 rejections across the window disappear (~6.3% of all jobs); worst-day impact 25 rejected jobs. Verification: replay a protected-docx fixture; assert 200 + redline output and a `field lock skipped` INFO line; rejection count for `word_failed` drops to ~0 in a canary day.
2. **Retry 'Call was rejected by callee' with backoff + warm-up probe.** Change: in `open_document`/`run_job`, retry COM -2147418111 3x with 2s/5s/10s sleep, and after every Word start do a no-op COM round-trip (e.g. read `word.Version`) before accepting the first job. Effect: the 18:24/18:25-style double failures and the 7 callee errors stop amplifying single incidents into multi-restart bursts. Verification: fault-inject a kill before a job; the next job must succeed without a `quit failed`/restart pair; count `quit failed` -> ~0.
3. **Recreate the COM proxy (or the worker) after any COM failure, and log the kill.** Change: on any `com_error`, release the `word` object and rebuild via `Dispatch` instead of calling `Quit` on a dead proxy; have the worker log an explicit `winword killed after 120s (job ...)` line when `_timed_call` sees RPC-failed at the deadline. Effect: no `AttributeError: Word.Application.Quit` noise (x23), no leftover documents (x8), kills become first-class log events. Verification: rerun the 10-14/10-15 scenarios; expect one clean `killed` line + one restart marker, zero `quit failed`.
4. **Surface open refusals with a stable cause code and file role.** Change: when `open_document` catches 25272/24601, log `open_refused file=original|revised code=25272 job=...` and map it to the same client answer the new worker already gives (the 24601 dialog already names the file; keep that). Effect: the classification drift between the old and new worker (E4) can never recur silently; uploads that Word refuses become countable per-upload. Verification: feed a corrupt docx; exactly one `open_refused code=25272` line; client error mentions which upload.
5. **Close the observability gaps before the next incident review.** Change: (a) wire the worker's stderr into `compare-stderr.log` (today 0 bytes while 285 ERROR tracebacks exist); (b) add one INFO line each for template detach / external-relationship handling / template-stamp removal wherever they actually run (if they run in the server process, mirror a marker into this log); (c) log idle Word recycles as `word recycled (idle Xmin)`. Effect: items the task asks about (sanitization, memory hints — of which this log currently has zero evidence) become auditable; recycle cost becomes measurable (8 recycles in 3h on 10-13, each followed by a 1.5-5s first open). Verification: after deploy, a day of traffic shows non-empty stderr file and >=1 of each marker; re-run this analysis and no category returns "no evidence".

## Not found / out of scope
- Template detach, external-relationship handling, template-stamp removal: zero matching lines in this worker's log (E5); may live in `serve.py`/server log — out of this slice.
- The 10-19->10-27 gap and the exact 11-05 06:00 reboot effect need `supervise.log`/server log cross-check — out of this slice.
- Client-visible HTTP status for each failure class is not determinable from this log (word_failed vs COM-fail paths log differently).
