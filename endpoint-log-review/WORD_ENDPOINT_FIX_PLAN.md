# Word endpoint — log review and fix plan

Date: 2026-11-05 · Method: four isolated reviewers (HTTP server / convert worker / compare worker / supervision+infra), each restricted to its own log slice to avoid cross-contamination, plus an owner-side cross-correlation. Raw findings: `findings-http.md`, `findings-convert.md`, `findings-compare.md`, `findings-infra.md`, `eventlogs.txt` (this folder).

Evidence window: 2026-10-04 → 2026-11-05. Volume: ~12,000 HTTP requests (server.log starts 10-17), 15,415 convert jobs, 3,307 compare jobs.

## Cross-correlations (what the slices reveal together)

1. **The 247 HTTP 500s are one bug, not a mystery.** The HTTP reviewer saw `500 word_failed` with no cause; the worker reviewers independently found the cause: `worker.py:507 _lock_fields` sets `fields.Locked = True` on the whole `Fields` collection, and on documents with protected areas Word throws COM 37373 → the worker rejects an otherwise-fine job. It accounts for **211 of 212 compare rejections (99.5%)** and **all 120 convert `word_failed`**, and it fired as recently as 06:16 and 06:18 today. It also explains compare failing at 6.4% vs convert 1.1%: redline documents carry document protection far more often.
2. **Word's median lifetime is 7 jobs / 12 minutes** (946 restarts in the window; 408 restarts within 5 min of the previous; worst burst 9 starts/10 min on 11-04). Driver: corrupt uploads that pass the HTTP `400 corrupt` guard still reach `Documents.Open` (1,046 "file appears to be corrupted" + 88 open errors in the convert worker), each poisoning the shared Word process. Each poison POST costs ~17 s of convert capacity plus a +33% cold-start latency penalty on the next jobs — a cheap DoS amplifier behind the 300/min rate limit.
3. **Failure recovery is broken**: after any COM failure the worker calls `Quit` on a stale proxy (`AttributeError: Word.Application.Quit` ×869 convert, ×23 compare), leaves leftover documents (×8), and the first job on the fresh Word dies with "Call was rejected by callee" (×7) — single incidents amplify into multi-restart bursts (e.g. 10-31 18:21–18:27: two jobs failed twice, 3 Word restarts in 4 min).
4. **The 10-day outage (10-18 → 10-28) was real downtime, not lost logs**: the Windows event log shows a power-off (0x500ff) at 10-18 18:11 — exactly when all logs go silent — and nothing until 10-28 17:21. Nothing alerted. (Today's auto-logon + keep-alive closes the boot part of this hole; the alerting part remains open.)
5. **The "maintenance timeline mismatch" is benign**: the Nov 5 window had two reboots (planned 06:45, then a TrustedInstaller "Upgrade (Planned)" at 06:55). Logs are server-local time. The 06:56:41 "unexplained" Word recycle is simply the post-reboot start.
6. **The tunnel is healthy** (correcting an infra-slice hypothesis): `tunnel.err.log`'s quick-tunnel session from 10-03 is stale history; the named tunnel runs as the `Cloudflared agent` Windows service (survives reboots, verified live today), and `public_url.txt` = `https://docx.rodrigues.ai` is consistent with it.

## Fix list, ranked

### P0 — correctness (users lose valid work today)

1. **Make `lock_fields` fail open.** Catch COM 37373 in `_lock_fields` (worker.py:507), log `field lock skipped`, continue the job. Field locking is an anti-tamper nicety; it must never reject a document Word opened cleanly.
   Effect: ~all of the 247 500s/month disappear; compare 5xx rate 6.4% → ~0.1%.
   Verify: replay a form-protected docx → 200 + output + `field lock skipped` line; a canary day shows `word_failed` ≈ 0.
2. **Stop the silent markup fallback.** `PDF export with markup failed` (×7) currently falls back to a 57 ms markup-less export returned as success — invisible fidelity loss on /v1/convert for documents with tracked changes. Either reject with a distinct reason or return a visible flag (response field) that markup was dropped.
   Verify: replay job `2b2b801b…` (11-03 08:01:43); the client must be able to tell.
3. **Add a slow lane for heavy documents.** The flat 120 s budget turned the only heavy convert of the month (12 s open + 102 s export, real work, no hang) into a guaranteed 504 + Word recycle. Size-scaled budget or one retry at a higher budget.
   Verify: replay that document; it must deliver a PDF.

### P1 — availability and robustness

4. **Pre-validate uploads in Python before the shared Word touches them** (zip structure, `[Content_Types].xml`, `word/document.xml` parse — the existing guard already rejects 560/month as `400 corrupt`; extend it to catch the shapes that still reach Word). Goal: zero corrupt-open COM errors in the shared processes; Word restarts drop from ~946/month to the idle baseline.
   Verify: replay the poison uploads; zero `ERROR Word COM failed`; daily Word starts ≈ idle rate.
5. **Rebuild the COM proxy after failures + warm-up probe + callee retry.** On any `com_error`: release the dead proxy and re-Dispatch instead of calling `Quit` on a corpse; after each Word start, do a no-op COM round-trip before accepting jobs; retry `-2147418111 'Call was rejected by callee'` 3× with backoff.
   Verify: `quit failed` → 0; no double-failure bursts; fault-inject a kill mid-queue and the next job succeeds.
6. **Fix the supervisor.** (a) pid bookkeeping broken since the `python315-bench` wrapper landed (wrapper pid ≠ listening child pid; `started 6964` vs `health ok 7020`; five restarts logged an empty pid) — track the process tree, not the wrapper; (b) stop the old serve.py before starting a new one (no `stopped serve.py` lines exist — orphan/port-collision risk); (c) replace the `misses=1` hair-trigger with a threshold + exponential backoff (the 10-12/13 flap: 7 restarts in ~90 min, supervisor itself restarted 6× in 3 h, 10 WINWORD kills); (d) reconcile the health probe and heartbeat that declared the same pid healthy and unhealthy in the same second.
7. **Dead-man alerting.** The 10-day outage and a 19.5 h hole were invisible. Heartbeat file checked by an external uptime monitor on `https://docx.rodrigues.ai` (any HTTP answer ≠ 502 = alive); alert on N minutes of silence.
   Verify: stop the service in a test window; alert fires within the threshold.

### P2 — observability (makes the next review cheap)

8. **Log the cause on every 500** (exception class, message, phase, Word pid) — the HTTP slice could not attribute a single 500.
9. **Log kill and lifecycle events as first-class lines**: `winword killed job=… pid=… after=…ms` at the 120 s deadline; `word start pid=… cause=error|keepalive|reboot` (restarts are today only inferable from settings lines); recycle cause (`idle Xmin`).
10. **Log peer identity**: client address + `Cf-Connecting-Ip`/`X-Forwarded-For` on the request line. Today the two 401s and five HEAD probes (both 401s land seconds after a restart — an unauthenticated monitor probing startup) are unattributable, and per-IP rate limiting is impossible.
11. **Startup auth window**: health checks must use `GET /ready` (already exists, correctly 503s during warm-up), never unauthenticated POSTs; stamp `listening` lines with UTC + local offset.
12. **Wire the dead sinks**: both `*-stderr.log` files are 0 bytes despite 285+ ERROR tracebacks; add marker lines for template detach / external-relationship handling / template-stamp removal (currently zero evidence they run — they may live in the server process; mirror markers into the worker logs).

## Confirmed healthy (no action)

- 120 s deadline is honored (all five >120 s jobs terminated at 120.3–123.2 s; queue never exceeded 6.2 s of the 180 s contract; rate limit never fired; no upload approached 20 MB).
- No attack patterns: guard rejections (298) are spread out, max 3/min; no bursts, no repeated probes, no 5xx storms (worst minute: 3).
- No OS-level Word/python crashes in 30 days of event logs (the 15 supervisor kills are task-kills, not crashes); no unexpected shutdowns (all four reboots planned/administrative); no memory leak (late-session jobs are slightly faster; only 2 transient font/memory errors in a month).
- Output sanity: output_bytes 6.9 KB – 5 MB over 10,904 outputs, no anomalies.
- Tunnel: named Cloudflare tunnel as a Windows service, custom domain, survives reboots.
- Reboot recovery (verified end-to-end today): auto-logon → keep-alive → serve.py → both Word workers → 22/22 live checks.

## Suggested order

Ship P0-1 (one except clause, biggest user-visible win) immediately; then P1-4+P1-5 together (they close the DoS amplifier and the failure amplification); P1-6/7 next; P2 items batched into the same PRs they describe (each is a few log lines).


## Deployment status — 2026-11-05 (later the same day)

Deployed to C:\word-endpoint and verified live (unit 46/46 at review time, live_check 22/22, public URL 200):

- **P0-1** lock_fields fail-open — LIVE (see regression test). Protected-doc probes: convert 200, compare 200.
- **P0-2** markup fallback — already fixed in code before this review (export_pdf raises; guarded by tests). Closed without change.
- **P0-3** slow lane — LIVE: job_timeout_for() = 120 s + 20 s/MB above 2 MB, capped at 300 s. README updated.
- **P1-5** worker recovery — LIVE: COM-busy retry (hresult −2147418111/−2147417846, 3 retries, 2/4/6 s), warm-up probe after Word start, dead-proxy quit logged as INFO (no traceback), kill events logged as first-class WARNING lines.
- **P1-6** supervisor — LIVE: Get-ServerPid resolves the real port listener (started pid now matches health pid in supervise.log), restart backoff 30·2^n capped 120 s with health-recheck after the sleep, heartbeat carries utc=.
- **P1-7** dead-man watchdog — LIVE: scripts\public-watchdog.ps1 as the WordEndpointWatchdog SYSTEM task (every 5 min): 2 consecutive public failures → restart cloudflared service (tunnel-only outage) or stop+start the WordEndpoint task (local down). State in logs\public-watchdog.state, log in logs\public-watchdog.log. External alerting (SMS/email) still needs an account — local self-healing only.
- **P2** logging — LIVE: peer= (Cf-Connecting-Ip → XFF → socket) on every request line, WARNING with code on every 5xx, kill lines, worker stderr already wired to *-stderr.log (found already correct in _spawn_impl).

Independent review (read-only, second agent): verdict GO, 0 must-fix; its three worth-fixing items (backoff recheck, 120 s cap + stop/start remedy, -CheckOnly state purity) were fixed and redeployed same day. Known accepted nits: field-lock skip message logs the raw hresult (not always protection); Get-ServerPid assumes python.exe; XFF fallback is spoofable but log-only; runner.py/README line endings flipped LF.

**Not done (deliberate):** P1-4 corrupt-upload pre-validation — rejected per owner directive ("problematic docx are normal, we must try to open them; bare minimal"): the shared-Word poisoning cost is now bounded by the recovery fixes above instead of by rejecting uploads.
