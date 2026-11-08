# HTTP server log findings — word endpoint

Scope analyzed: `logs/server.log`, `server.log.1`, `server.log.2`, `server.log.3` (58,387 lines,
2026-10-17 12:22:42 → 2026-11-05 06:59:21), plus current-state check of `server.err.log` /
`server.out.log`. All other log files were out of scope and not read.

`server.err.log` and `server.out.log` are both **0 bytes right now**. No rotated `.1/.2/.3`
variants exist for them, so nothing can be said about their history — current state only.

## Baseline numbers (whole window)

12,027 requests with a final status line (12,016 POST + 6 `GET /ready` + 5 `HEAD`):

| status | count | share | breakdown |
|---|---|---|---|
| 200 | 10,907 | 90.7% | incl. 3 `GET /ready 200 ok` |
| 400 | 858 | 7.1% | corrupt 560, external_content 220, active_content 78 |
| 500 | 247 | 2.05% | word_failed 220, convert_failed 25, compare_failed 2 |
| 504 | 5 | 0.04% | timeout at 120,279–123,191 ms |
| 404 | 5 | — | HEAD probes |
| 503 | 3 | — | `GET /ready 503 degraded` during one startup |
| 401 | 2 | — | see ISSUE-2 |

- Endpoint split: `/v1/convert` 9,764 requests, 5xx rate **1.1%** (105×500 + 3×504);
  `/v1/compare` 2,248 requests, 5xx rate **6.4%** (142×500 + 2×504); `/v1/compare-pdf` 4 requests, all 200.
- Durations (12,016 POSTs): p50 = 2.9 s, p90 = 7.6 s, p99 = 19.3 s, max = 123.2 s. Only 14 >60 s; 5 >120 s (the five 504s).
- Queueing: `queue_ms=0` for 11,678 of 11,717 jobs; 39 jobs queued, max `queue_ms=6224`. Nowhere near the 180 s queue contract.
- Rate limit: **zero** 429s. Peak log volume 135 lines/min (≈30 req/min) on 2026-10-28 17:27 — far under 300 POSTs/min.
- Uploads: max `bytes=` seen is 12,512,213 (~12.5 MB); the 20 MB rule never fired in this window. No `too_large`-style rejections logged.
- Orphan check: all 11,717 `validation request=` ids have a matching final status line → no client disconnects/aborts are visible in these logs (also see ISSUE-5: none are logged, so "no aborts" = "no evidence", not proof).
- Guard rules that fired: `external_content` 218 warning lines (parts: `word/_rels/document.xml.rels` dominant, also `word/charts/_rels/chart1.xml.rels`, `word/_rels/settings.xml.rels`), `active_content` 78 (44 generic + 28 `forbidden_field_command` + 6 `active_word_element`, mostly `word/document.xml`).

## SUMMARY — top 5 issues ranked

1. **[S1] 247× HTTP 500 (`word_failed` 220, `convert_failed` 25) with no logged cause.** Failures are
   fast (0.5–2.8 s), which means Word refused/failed immediately rather than timing out, and the log
   carries zero diagnostic detail (no exception, no Word exit code, no kill event). Clustered:
   2026-11-04 23:47–23:49, 2026-11-05 03:56–04:01, and 06:15–06:18 (maintenance window). Compare
   endpoint fails 6× more often than convert (6.4% vs 1.1%).
2. **[S1] Unexplained server restart 2026-10-28 19:10:49.** Normal traffic until 19:08:24, 2 m 25 s
   of silence, then a fresh `listening` line — no error, crash, or shutdown line anywhere. Both 401s
   in the entire window occur within seconds of a restart (19:10:54 and 2026-11-05 06:57:04),
   suggesting an unauthenticated probe/monitor hits the endpoint while it is coming up.
3. **[S2] Maintenance-window timeline does not match the stated reboot.** The server was clearly up
   and serving 05:50–06:55 on 2026-11-05 (activity every few minutes through 06:43:43), with
   intermittent `word_failed` 500s at 06:15/06:16/06:18 and one 13.9 s success interleaved at
   06:17:45. The actual restart (`listening` 06:56:31) follows a 768 s silence. Either log
   timestamps are not server-local time, or the machine did not actually go down until ~06:55.
   Also: ~10 days of total darkness earlier (2026-10-18 18:10 → 2026-10-28 17:21, gap 861,024 s)
   plus ~19.5 h (2026-10-17 13:58 → 2026-10-18 09:30).
4. **[S2] No client/tunnel IP is ever logged.** The only IP in all four files is `127.0.0.1` (the
   listening line). The tunnel-origin-vs-localhost question the contract cares about is
   unanswerable from these logs, and the 401s/HEAD probes cannot be attributed.
5. **[S3] Contract/observability gaps around the 120 s kill.** The five >120 s jobs all terminated
   as 504 at 120,279–123,191 ms — the deadline is honored — but no kill/worker-restart event is
   written to `server.log`, so the "kill including its Word process" half of the contract cannot be
   verified here. Likewise client disconnects, rate-limit rejections, and size rejections produce no
   lines when they (never) fire.

## EVIDENCE (verbatim lines; file names as named in /tmp/welogs/logs/)

### E1 — 500 word_failed clusters, no cause logged (ISSUE-1)

server.log:
```
2026-11-04 23:47:59,651 INFO d1fad798c9144e9c8f5953d78b3dfa1c POST /v1/convert 500 word_failed 968ms
2026-11-04 23:48:32,031 INFO 029103e983424bf7864d2feff60ae425 POST /v1/compare 500 word_failed 1303ms
2026-11-04 23:49:45,901 INFO ba7348288dd443628867ae52e653c6b0 POST /v1/compare 500 word_failed 463ms
2026-11-05 03:56:43,879 INFO 987b12d3c0ff4ee6b0ab065ae5d5da02 POST /v1/convert 500 word_failed 919ms
2026-11-05 03:57:23,701 INFO 3d2be331012047a383e9d7d32a16bc96 POST /v1/compare 500 word_failed 2765ms
2026-11-05 04:01:31,575 INFO c99428ddda7c476ab87cb9d2e6060765 POST /v1/compare 500 word_failed 1044ms
```
Maintenance-window failures — note both worker kinds fail while one convert succeeds between them
(server.log):
```
2026-11-05 06:15:43,778 INFO 3296c80e72264e0c8f60f7005570b6f8 POST /v1/convert 500 word_failed 1995ms
2026-11-05 06:16:17,716 INFO b34147490bbf43b9b9b4caa0dc760639 POST /v1/compare 500 word_failed 2625ms
2026-11-05 06:17:45,874 INFO d7b0c0fefbe04f61a2b583ceb60c9673 POST /v1/convert 200 ok 13893ms
2026-11-05 06:18:43,412 INFO 8c67e2bf4b7b485685397e461299c1ef POST /v1/compare 500 word_failed 1094ms
```
And the surrounding job lines carry no error detail (server.log):
```
2026-11-05 06:15:41,861 INFO validation request=3296c80e72264e0c8f60f7005570b6f8 bytes=38426 ms=77
2026-11-05 06:15:41,861 INFO job request=3296c80e72264e0c8f60f7005570b6f8 kind=convert queue_ms=0
2026-11-05 06:15:41,866 INFO job request=3296c80e72264e0c8f60f7005570b6f8 id=329f2c60826347c4a90a08dc6a943bc3 kind=convert input_ms=1
2026-11-05 06:15:43,778 INFO 3296c80e72264e0c8f60f7005570b6f8 POST /v1/convert 500 word_failed 1995ms
```
Earliest-window examples (server.log.3):
```
2026-10-17 12:56:48,897 INFO 66bea13d197d4e3e9a72d1fa919df9d4 POST /v1/convert 500 word_failed 642ms
2026-10-17 12:57:03,450 INFO 0be6464600e940faa3403b473c50781 POST /v1/compare 500 word_failed 914ms
```
Isolated case spread across all four files, e.g. 500s appear on 2026-10-17, 10-28, 10-29, 10-30,
10-31, 11-01, 11-02, 11-03, 11-04, 11-05; worst hours 8 (2026-10-28 20h) and 7 (2026-11-01 18h/19h,
2026-10-30 02h).

### E2 — restarts, the unexplained 10-28 one, and both 401s next to restarts (ISSUE-2, ISSUE-3)

All `listening` lines (server.log, server.log.3):
```
2026-10-28 17:21:23,580 INFO listening on http://127.0.0.1:8787 warm-word=compare,convert rate=300/min
2026-10-28 19:10:49,586 INFO listening on http://127.0.0.1:8787 warm-word=compare,convert rate=300/min
2026-11-05 06:56:31,978 INFO listening on http://127.0.0.1:8787 warm-word=compare,convert rate=300/min
```
Last line before the 19:10 restart — normal traffic, then silence (server.log.3):
```
2026-10-28 19:08:24,182 INFO job id=051c59b35de243c193773569c27e2182 kind=convert output_ms=0 output_bytes=595397
2026-10-28 19:08:24,193 INFO f02a636a869840a6a68e763fe643f750 POST /v1/convert 200 ok 8403ms
```
Startup sequence of that restart, including readiness flaps and the 401 (server.log.3):
```
2026-10-28 19:10:50,498 INFO 62b7ab1a19ee44de9a529d40c6e91fc9 GET /ready 503 degraded 2ms
2026-10-28 19:10:51,588 INFO 9690e3c99a834e6a8238ef2b3ab5a7b2 GET /ready 503 degraded 0ms
2026-10-28 19:10:52,630 INFO 2ecfa435d88e42dfb3176a4fe7a0238d GET /ready 503 degraded 2ms
2026-10-28 19:10:52,276 INFO convert Word is ready
2026-10-28 19:10:53,204 INFO compare Word is ready
2026-10-28 19:10:53,646 INFO 6eb413ac961a4501a246912ea9764d3d GET /ready 200 ok 1ms
2026-10-28 19:10:54,466 INFO 3bf93ea4f85a42c6a5ceaa487480d848 POST /v1/convert 401 unauthorized 0ms
```
Second 401, again right after the post-maintenance restart (server.log):
```
2026-11-05 06:56:31,978 INFO listening on http://127.0.0.1:8787 warm-word=compare,convert rate=300/min
2026-11-05 06:56:42,116 INFO convert Word is ready
2026-11-05 06:56:42,552 INFO compare Word is ready
2026-11-05 06:56:48,380 INFO e4f2f76eca6c40b5a0a531a0339f1f58 HEAD /v1/convert 404 not_found 0ms
2026-11-05 06:57:04,028 INFO bcd3cc6694cb430aac6583288102625b POST /v1/convert 401 unauthorized 21ms
```

### E3 — maintenance-window timeline (ISSUE-3)

Traffic continues through the stated 05:50–06:55 downtime (server.log; per-minute counts):
05:00, 05:14, 05:16, 05:29–05:32, 05:34, 05:36–05:37, 05:41–05:43, 05:44, 05:47, 05:48,
05:50, 05:51, 05:52, 05:54, 05:57, 06:02, 06:03, 06:07, 06:08, 06:10, 06:11, 06:13, 06:14,
06:15–06:18, 06:20, 06:22, 06:23, 06:26, 06:32, 06:33, 06:42, 06:43 — then nothing until:
```
2026-11-05 06:43:43,394 INFO 0fea5f8e2d6645ae8fec49cf79ad15ba POST /v1/convert 200 ok 3580ms   (last line before gap)
2026-11-05 06:56:31,978 INFO listening on http://127.0.0.1:8787 warm-word=compare,convert rate=300/min
```
Long earlier outages (timestamp gaps >10 min; the full 108-gap list is dominated by quiet-night
gaps of 10–14 min, only these three are hours/days):
```
GAP 70334 s   2026-10-17 13:58:01 → 2026-10-18 09:30:15   (~19.5 h)
GAP 19032 s   2026-10-18 09:30:15 → 2026-10-18 14:47:27   (~5.3 h)
GAP 861024 s  2026-10-18 18:10:59 → 2026-10-28 17:21:23   (~9.97 days)
```
Boundary lines of the 10-day gap (server.log.3):
```
2026-10-18 18:10:59,409 INFO 05e4db1d5902471b9a4f441cebb33adb POST /v1/convert 200 ok 2516ms
2026-10-28 17:21:23,580 INFO listening on http://127.0.0.1:8787 warm-word=compare,convert rate=300/min
```

### E4 — 504 timeouts and queue behavior vs the 120 s / 180 s contract (ISSUE-5)

All five 504s; note the killed-job `output` line is absent and no kill event is logged (server.log.3,
server.log.2, server.log.1):
```
2026-10-30 01:17:10,119 INFO b7f33b1d6dcf48439a9ccf7c08381dff POST /v1/convert 504 timeout 121450ms
2026-10-30 01:31:15,858 INFO 60eacce8e18941e19c7438a565c531e9 POST /v1/convert 504 timeout 121279ms
2026-10-30 13:21:31,917 INFO f9523288808b45c8aa49b1775648d26 POST /v1/compare 504 timeout 121729ms
2026-11-01 07:06:23,438 INFO de3f330a71864671b0d548447f337d28 POST /v1/convert 504 timeout 120594ms
2026-11-03 17:32:19,583 INFO dfef0a9d7ce84dd995c71fcd1d237ce2 POST /v1/compare 504 timeout 123191ms
```
Recovery right after a 504 was clean (server.log.3):
```
2026-10-30 01:22:57,596 INFO 2dc0af088aa34129ba2064ba68cafe63 POST /v1/convert 200 ok 4081ms
2026-10-30 01:24:45,586 INFO 2160884f39494f1aba4949ab8bf6acf7 POST /v1/compare 200 ok 90690ms
```
Queueing never approached the 180 s contract: 39 jobs with `queue_ms>0`, max:
`queue_ms=6224` (values ≥2 s: 2019, 2044, 2231, 2439, 2744, 3191, 3343, 3583, 5793, 6224).

### E5 — rejected uploads and probes (ISSUE-4/6, all benign in shape)

Guard rejections total 298 (400) + 296 WARNING guard lines; max 2 guard hits in any single minute,
spread across all 18 days — no burst/scan pattern. Representative (all four files):
```
WARNING guard request=feaddf1f14b54b93812b87ad564f3fbd code=active_content reason=forbidden_field_command part='word/document.xml' field=original
WARNING guard request=ff8afd8cc86d47379ce53cd2686be042 code=external_content reason=external_content part='word/charts/_rels/chart1.xml.rels' field=original
WARNING guard request=ff836ff1dcb94a1ab49ff60a36e4e9c1 code=active_content reason=active_content part=None field=revised
```
`400 corrupt` totals 560 (548 convert, 12 compare), max 3 in one minute — steady background, not an attack.
Non-API requests, all five:
```
2026-10-18 09:30:15,471 INFO a98cb376524c45bb992f502190aa0c15 HEAD / 404 not_found 161ms
2026-10-18 09:30:15,477 INFO c4b23d3741114726840a4fb47f7c0d68 HEAD / 404 not_found 170ms
2026-11-01 08:33:21,597 INFO 5dd68a6117b043de97033b567494b75c HEAD / 404 not_found 0ms
2026-11-01 08:33:21,746 INFO 3a958f2db00842e9a446422549dd4cf7 HEAD / 404 not_found 0ms
2026-11-05 06:56:48,380 INFO e4f2f76eca6c40b5a0a531a0339f1f58 HEAD /v1/convert 404 not_found 0ms
```
`external_content` guard line with no duration field after the reason (trailing space, no `ms`)
— minor format inconsistency (server.log.2/server.log.3, several instances):
```
2026-10-29 22:38:51,859 INFO 55bbd7388ddc114e4485e5b2ab5bd4b99b POST /v1/compare 400 external_content
```

## ROOT-CAUSE HYPOTHESES

- **H1 (for ISSUE-1, high confidence about symptom, medium about cause):** Word processes get into
  a bad state (modal dialog, COM automation error, license/activation prompt, recovering document)
  and jobs then fail in 0.5–2.8 s. The interleaved 13.9 s success at 06:17:45 suggests intermittent
  rather than hard-down workers, or one worker healthy while the other is sick. The log omits the
  exception text, so the exact Word failure mode is unrecoverable from server.log alone (compare-
  worker/convert-worker logs are out of scope for this slice).
- **H2 (medium):** The 10-28 19:10 restart was a crash or supervisor kill: traffic stopped
  mid-normal-operation with no shutdown line, then `/ready 503 degraded` flaps during warm-up. If it
  were a planned restart there would usually be a stop line; there is none.
- **H3 (medium):** The two 401s and the 06:56:48 HEAD probe come from the supervisor/monitor or a
  health checker that does not send the token, firing only when the server has just come up (they
  never appear at any other time in 18 days).
- **H4 (medium):** The maintenance "reboot 05:50–06:55 server local" either happened in a different
  timezone than the log's timestamps, or actually took effect ~06:55 (the 06:43→06:56 silence +
  listening). The word_failed cluster at 06:15–06:18 fits a machine in a degraded/pre-reboot state
  (Word processes being stopped or patching in progress).
- **H5 (low):** Compare jobs fail 6.4% vs convert 1.1% — hypothesis: compare (redline) documents
  hit more complex Word paths (tracked changes, big files) or the compare worker is reused with
  accumulating state; needs worker logs to confirm.
- **H6 (low):** The 10-day hole (10-18→10-28) is service downtime, not lost logs: rotation kept
  server.log.3 beginning 10-17 12:22, so a hole this size with all four files intact means nothing
  was listening (or the tunnel/supervisor was down) for ~10 days.

## CONCRETE FIXES

1. **Log the failure cause on every 500.** Change the `word_failed`/`convert_failed` path in
   serve.py to log the exception class, message, Word process PID, and last COM operation at
   WARNING, before emitting the 500 line.
   Effect: ISSUE-1 becomes diagnosable; clusters can be tied to Word state.
   Verify: inject a failing document (or kill WINWORD between job lines) and check the log carries
   the cause; re-check that 500 counts do not change.
2. **Log kill and worker-restart events to server.log.** When the 120 s kill fires, write
   `kill job=… pid=… after=…ms`; same for worker respawn.
   Effect: the "killed including its Word process" half of the contract becomes verifiable from the
   server log; silent restarts like 10-28 19:10 become attributable.
   Verify: run a >120 s job (huge document), confirm one 504 line plus one kill line with matching
   job id and a non-zero PID.
3. **Log client identity.** Add the peer address and, when present, `Cf-Connecting-Ip` /
   `X-Forwarded-For` to the final status line (or a `peer=` field on the request-start line).
   Effect: tunnel-origin vs localhost traffic, the 401s, and the HEAD probes become attributable;
   rate limiting can be per-IP.
   Verify: curl from localhost and through https://docx.rodrigues.ai; both addresses must appear.
4. **Fix the readiness/auth window after restart.** Have the supervisor health check use a token or
   a dedicated unauthenticated `/ready` only (never POST /v1/* without auth), and return 503 (not
   401) for requests arriving before both workers are ready, so monitors retry instead of counting
   an auth failure.
   Effect: the restart-adjacent 401s disappear; startup metrics stop lying.
   Verify: restart the server with the monitor running; expect `GET /ready 503` then 200, and zero
   401s in the window.
5. **Reconcile the maintenance timeline.** Compare the Windows event log reboot time with the log's
   timezone; ensure serve.py logs UTC and stamps `listening` lines with the server's local timezone
   offset.
   Effect: makes "server was up during the stated downtime" (E3) interpretable.
   Verify: after the change, one restart line shows both clocks; cross-check against event log 6005/6006.
6. **Investigate the compare-endpoint failure rate (6.4% vs 1.1%).** With fix 1 in place, group
   500 causes by endpoint for a week; if compare failures share one exception, fix that path (e.g.
   document-open retry, worker recycle after N failures).
   Effect: removes the largest single bucket of user-visible 500s.
   Verify: compare 5xx rate trending toward the convert rate (~1%).
7. **Alert on silence.** The 10-day and 19.5 h holes were invisible to everyone; point the
   supervisor's keep-alive at a timestamped heartbeat file plus external uptime check on
   https://docx.rodrigues.ai.
   Effect: downtime measured in minutes, not days.
   Verify: stop the server in a test window; alert must fire within the agreed threshold.

## Notes / non-issues

- No 5xx storms from any single minute; worst single-minute 500 count is 3.
- No evidence of guard bypass attempts: no bursts, no repeated same-request guard hits, max upload
  12.5 MB < 20 MB, zero 429s.
- `compare-pdf` barely used (4 requests, all ok) — too little traffic to judge.
- Zero `output_bytes` anomalies: min 6,928 bytes, max 5.0 MB over 10,904 outputs.
- Slowest validation 7.6 s (a 1.1 MB upload); p99 well under 150 ms.
