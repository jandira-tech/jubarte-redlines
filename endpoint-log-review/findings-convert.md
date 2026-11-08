# Convert Word worker — findings

Requested destination was `/tmp/welogs/findings-convert.md`; this sandbox only writes inside the workspace, so the report lives here. Copy to `/tmp/welogs/` on the host if needed.

Scope analyzed (read ONLY these, per assignment): `convert-worker.log`, `.1`, `.2`, `.3`, `convert-stderr.log`.
Coverage: 2026-10-04 00:17:59 -> 2026-11-05 06:59:21. `convert-stderr.log` is empty (0 bytes).
All counts below are over the concatenation of the four worker logs (chronological; `.3` oldest). Files use CRLF line endings.

Volume: 15,417 `open_document` phases, 14,362 `export_pdf` phases, 15,415 completed `kind=convert` job lines, 127 logged rejections (120 `word_failed` + 7 `convert_failed`), 946 Word process (re)creations, 1,138 `ERROR Word COM failed`.

Duration stats (n=15,415 `word_ms`): mean 3,452 ms, p50 2,529, p90 6,482, p99 18,078, p99.9 43,836, max 120,097 ms. Jobs >=30s: 40; >=60s: 10; >=100s: 1; >=120s: 1.

## SUMMARY — top 5

1. **Malformed/corrupt uploads kill the shared convert Word process ~1,100 times a month, and each kill costs a full Word recycle.** 1,046 `The file appears to be corrupted` + 88 `Word experienced an error trying to open the file` COM errors on `Documents.Open`; 1,138 `ERROR Word COM failed`; the Word process is dead by the time the worker tries to quit it (`AttributeError: Word.Application.Quit` x869, `could not silence alerts during quit` x136). Result: 946 Word (re)starts; median Word lifetime is **7 jobs / ~12 min**; 408 of 946 restarts occur within 5 min of the previous one (loops); worst burst **9 Word starts in 10 min** at 2026-11-04 10:47:56. One cheap POST with a poison file stalls every queued convert for the recycle (~17 s) plus cold-start tax — a trivial DoS amplifier behind the 300 POST/min rate limit.
2. **`lock_fields` rejects valid documents: 120 jobs failed `word_failed` solely because `fields.Locked = True` hit "protected area of the document" (wdmain11.chm 37373).** Documents with protected sections/forms never convert; clients get `word_failed` for a legitimate file (e.g. 2026-11-05 03:56:43 and 06:15:43).
3. **Restart churn measurably taxes latency.** Median `word_ms` of the first 5 jobs after a Word start is 3,248 ms vs 2,438 ms for the last 5 jobs before a restart (+33%); restart->first-job-completion median 6 s but **max 1,221 s**. There is **no leak-driven degradation** (jobs get slightly *faster* late in a session; restarts are event-driven), and memory pressure appears only twice (evidence par. 3).
4. **The 120 s kill threshold fired exactly once and worked, but it guarantees failure for legitimately heavy documents.** 2026-10-30 01:15-01:17: 12.0 s open + 102.4 s export -> `convert_failed` at 120,097 ms, Word killed (`AttributeError` on Quit), clean recovery: new Word 17 s later, next job 3.9 s. A document that needs >120 s total can never succeed — no retry, no slow lane.
5. **Two serious anomalies: a 10-day total silence (2026-10-18 18:10:59 -> 2026-10-28 17:21:31, 861,085 s, zero jobs and zero keep-alive Word starts — worker down or unreachable), and no fresh-start/reboot trace at 2026-11-05 ~06:00**: the Word instance created 05:36:54 served jobs 06:07-06:43 with no restart marker; the only lifecycle event in 06:00-07:00 is a clean, error-free Word recreation at 06:56:41. Also: `PDF export with markup failed` (7x) silently falls back and still returns a PDF (57 ms export) that likely lacks markup, with a leftover document Word cannot close.

## EVIDENCE (verbatim, file + timestamp)

### 1. Corrupt-upload -> dead-Word -> recycle loop
- Throughout all four files; 1,046 occurrences of:
  `pywintypes.com_error: (-2147352567, 'Exception occurred.', (0, 'Microsoft Word', 'The file appears to be corrupted.', 'wdmain11.chm', 25272, -2146822496), None)` — raised at `worker.py:425 open_document` / `worker.py:406 convert_document` (`word.Documents.Open(... OpenAndRepair=True ...)`, traceback shows `NoEncodingDialog=True, FileName=str(path.resolve())`).
- 88x: `pywintypes.com_error: ... 'Word experienced an error trying to open the file.\nTry these suggestions.\n* Check the file permissions ... (C:\\word-endpoint\\work\\...\\document.docx)', 'wdmain11.chm', 24601, -2146823167`
- Dead-Word signature (869x), e.g. `convert-worker.log.1:844-849`:
  `File "C:\word-endpoint\word_endpoint\worker.py", line 579, in quit_word` -> `word.Quit(SaveChanges=_DO_NOT_SAVE)` -> `File "...win32com\client\dynamic.py", line 631, in __getattr__` -> `AttributeError: Word.Application.Quit`
- Restart marker = options pair, e.g. `convert-worker.log.1:850-851`: `2026-11-01 07:25:52,152 INFO Word option CheckSpellingAsYouType disabled` / `2026-11-01 07:25:52,160 INFO Word option CheckGrammarAsYouType disabled` (946 pairs total).
- Rapid double failure: `convert-worker.log` 2026-11-05 04:08:55,256 `ERROR Word COM failed` -> 04:08:55,292 `ERROR quit failed` -> 04:08:57,689 options (new Word) -> 04:08:58,242 `ERROR Word COM failed` again -> 04:09:01,919 options again.
- Crash-loop scale: 946 starts; 408 within 5 min of the previous; max 9 starts/10 min at 2026-11-04 10:47:56; daily starts 52-110 on active days (peak 110 on 2026-11-03, which also has the peak 108 `ERROR Word COM failed`).

### 2. `lock_fields` rejects protected documents
- `convert-worker.log:183-193` (2026-11-05 03:56:43):
  `ERROR field lock failed` -> `File "...worker.py", line 486, in lock_fields` -> `line 507, in _lock_fields` -> `fields.Locked = True` ->
  `pywintypes.com_error: ... 'This method or property is not available because the object refers to a protected area of the document.', 'wdmain11.chm', 37373, -2146823683`
  then `2026-11-05 03:56:43,709 INFO worker rejected the job: word_failed`, `word_ms=798`.
- Same shape at `convert-worker.log:675` (2026-11-05 06:15:43, job id=329f2c60826347c4a90a08dc6a943bc3, open_document ms=1672, `word_ms=1879`) and `convert-worker.log.1:852` (2026-11-01 07:27:59). 120 total.

### 3. Restart churn taxes latency; no memory-growth signal
- Computed over 946 sessions (419 with >=10 jobs): median `word_ms` first-5-after-start 3,248 vs last-5-before-restart 2,438; p90 7,479 vs 6,951.
- Session stats: median 7 jobs / 703 s; p90 43 jobs / 3,759 s; max 172 jobs / 93,288 s. Restart->first-job-completion: median 6 s, p90 74 s, **max 1,221 s**.
- Late-session jobs are *faster*, i.e. restarts are not preceded by slowdown -> restarts are event-driven (bad uploads), not degradation-driven.
- Only memory-ish errors, 2x, both 2026-10-13, in `convert-worker.log.3`:
  `pywintypes.com_error: ... 'There is insufficient memory or disk space. Word cannot display the requested font.', 'wdmain11.chm', 24802, -2146822966` (at 10:28:51,893 also `ERROR close failed`; second at 10:35:58, `word_ms=296`). These are the only two font-related lines in 4.1 MB.

### 4. The single 120 s kill (threshold works; heavy docs guaranteed-fail)
- `convert-worker.log.2:13681-13705`, job id=314d6334a78f4a0fb2830230753b17e4:
  `2026-10-30 01:15:21,957 ... phase=open_document ms=12004`
  `2026-10-30 01:17:10,033 ... phase=export_pdf ms=102374`
  `2026-10-30 01:17:10,036 INFO worker rejected the job: convert_failed`
  `2026-10-30 01:17:10,044 ... kind=convert word_ms=120097`
  `2026-10-30 01:17:10,046 INFO could not silence alerts during quit` / `01:17:10,051 ERROR quit failed` (`AttributeError: Word.Application.Quit` — process killed with the job)
  `2026-10-30 01:17:27,126/134 INFO Word option ... disabled` (recycle in ~17 s); next job 01:22:56 completes normally (`word_ms=3953`). Recovery after kill: verified clean.
- Tail: >=60 s: 10 jobs; >=30 s: 40; p99 = 18.1 s — the threshold almost never fires; when it did, the job was doing real work (102 s export).

### 5. Anomalies: 10-day silence; no ~06:00 reboot marker; silent markup-export fallback
- Silence: last line before gap `convert-worker.log.2:5387` `2026-10-18 18:10:59,393 ... kind=convert word_ms=2486`; next lines `2026-10-28 17:21:31,657/668 INFO Word option ... disabled` then a job at 17:23:29. Gap = 861,085 s (~9 d 23 h). No jobs, no errors, no keep-alive Word starts in between. (Worker logs cannot distinguish outage from zero traffic, but zero *keep-alive restarts* suggests the worker was not running.)
- 2026-11-05 05:30-07:00: last pre-gap job `05:57:58,102 kind=convert word_ms=3094`; first post-gap job `06:07:25,169` — with **no** options pair between 05:36:54 and 06:56:41, i.e. the Word created 05:36:54 served the 06:07-06:43 jobs. A reboot at ~06:00 would have forced a fresh Word + options log; none exists. Only lifecycle event in the window: `convert-worker.log:735-736` `2026-11-05 06:56:41,656/783 INFO Word option ... disabled` — clean recycle, no preceding error (last job ended 06:43:43; consistent with an idle keep-alive/supervisor recycle, not a crash).
- Silent markup fallback, `convert-worker.log.1:12177-12213`, job id=2b2b801b4d7f4b46a67553535450dd29, 2026-11-03 08:01:43:
  `INFO ... phase=open_document ms=1538` / `ERROR PDF export with markup failed` / `INFO ... phase=export_pdf ms=57` / `INFO could not close a leftover document` / `kind=convert word_ms=1650` (job *succeeds*; client gets a 57 ms export). Also `INFO could not mark the document unchanged` at 08:01:43,436 and again on the client's 5 s-later retry (08:01:48, after another Word restart at 08:01:47): identical failure — this document shape reproducibly breaks markup export, "mark unchanged", and leftover-close, and costs two Word recycles.
- RPC: exactly 3x `pywintypes.com_error: (-2147023174, 'The RPC server is unavailable.', None, None)`, all on 2026-10-04 00:45 (also 2x `ERROR could not determine the WINWORD pid` that day, 1x 2026-10-13 02:50:45 followed by an immediate restart). No `0x800706BE` text appears anywhere.
- Minor: 7x `ERROR PDF export with markup failed`; 9x `could not close a leftover document`; 5x `could not mark the document unchanged`; 2x `could not disable style updates`; 1x `ERROR unexpected worker failure` (2026-10-12 15:21:28, worker.py:231 run_job); tracebacks from two different Python envs (`...site-packages\win32com...` under `python\` early Oct vs `python315-bench\` later) — the runtime was swapped mid-window.

## ROOT-CAUSE HYPOTHESES

1. **Corrupt-open failures are processed as "poison the process"**: the worker treats any `Documents.Open` COM exception as a poisoned Word and kills/recycles it (`worker.py:579 quit_word` fails with `AttributeError` because the process is already terminated). Whether Word actually crashes on these files or the worker kills it deliberately, the economics are the same: a nearly-free attacker POST costs ~17 s of convert capacity plus a +33% cold-start penalty on following jobs, ~50-100x/day (peak 108/day). The pre-parse validation (README claims DDE/macro/zip-bomb rejection) evidently does not catch the shapes that make Word's Open refuse with "file appears to be corrupted".
2. **`lock_fields` is not exception-proof**: it sets `fields.Locked = True` on the whole `Fields` collection; on documents with protected ranges (form protection) Word raises 37373 and the worker escalates a cosmetic anti-tamper step into a job rejection. Failing open (skip locking, log a warning) would convert these files fine.
3. **Kill threshold vs heavy documents**: the single 120 s kill was a legitimate 102 s PDF export, not a hang. There is no mechanism for slow-but-progressing jobs, so any document needing >120 s can never convert (and each attempt burns a Word recycle).
4. **10-day silence** is most consistent with the worker/service being down or unreachable 2026-10-18->28 (a running keep-alive supervisor should still have (re)created Word and logged options occasionally).
5. **Silent markup fallback** (`export with markup failed` -> 57 ms plain export still returned as success) risks shipping PDFs without tracked changes/comments with no client-visible signal; the same document also cannot be marked unchanged and leaves an unclosable leftover — likely a protected/read-only document shape.

## CONCRETE FIXES

1. **Pre-validate docx before giving it to the shared Word process** (zip structure + `[Content_Types].xml` + `word/document.xml` parse in Python, or a sacrificial throwaway Word instance for suspicious files). Effect: the 1,046+88 corrupt opens (~7% of jobs, up to 108/day) stop killing the shared process; expected Word restarts drop from ~946/month toward the keep-alive baseline; removes the DoS amplifier. Verify: replay the poison uploads against the new build and assert zero `ERROR Word COM failed` and zero `AttributeError: Word.Application.Quit`; daily restart count in the log drops to the idle keep-alive rate.
2. **Make `lock_fields` fail open**: catch com_error 37373 per-story/per-range (or skip locking on protected documents) and continue conversion. Effect: the 120 `word_failed` rejections become successful converts. Verify: regression-test with a form-protected docx; assert `phase=export_pdf` and no `ERROR field lock failed`; the `word_failed` counter for this cause goes to ~0.
3. **Add a slow lane or budget-aware export**: detect export progress or scale the per-job budget for huge documents (e.g. size-scaled timeout, or one retry at 300 s) instead of a flat 120 s that turned the only heavy document in a month into `convert_failed` + recycle. Effect: heavy-but-valid documents convert instead of guaranteed-failing. Verify: replay the 2026-10-30 document (12 s open / 102 s export) and assert a delivered PDF.
4. **Make the markup-export fallback explicit**: on `PDF export with markup failed`, either reject with a distinct reason or return a response flag noting markup was dropped, and log which job/document triggered it. Effect: no silent fidelity loss; the reproducible 2b2b801b-style shape becomes visible to clients and monitoring. Verify: unit-test the fallback path; grep logs for the new reason code instead of bare `ERROR PDF export with markup failed` followed by success.
5. **Close the two observability gaps**: (a) alert on worker silence — the 10-day hole (Oct 18->28) produced no signal here; a dead-man timer on `kind=convert`/options lines would catch it; (b) log Word process creation with pid and uptime plus recycle cause (`error|keepalive|kill-timeout`), so cases like the 06:00 window (05:36:54 Word serving 06:07 jobs; clean recycle 06:56:41) are facts, not inferences. Verify: simulate worker death and confirm the alert fires within one check interval; confirm each Word start logs `pid=` and cause.

## Not found (checked)
- No `0x800706BE` text anywhere; RPC errors only as `-2147023174 'The RPC server is unavailable.'` x3, all 2026-10-04 00:45.
- No font-loading or file-not-found warnings; the only "font" lines are the two `insufficient memory ... requested font` COM errors above.
- No screen-updating setting line exists in these logs (only spelling/grammar options are logged), so `ScreenUpdating` application cannot be verified from this slice.
- `convert-stderr.log`: empty (0 bytes).
