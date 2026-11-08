# Word endpoint — supervision/infrastructure findings (2026-11-05)

Scope: `logs/supervise.log` (2,420 lines, 2026-10-04 01:20 -> 2026-11-05 06:56), `logs/tunnel.log` (0 bytes), `logs/tunnel.err.log` (5.8 KB, single session 2026-10-03), `logs/public_url.txt`, `logs/server.pid`, `logs/supervise.pid`, `eventlogs.txt` (30-day grouped digest + shutdown events). Server/worker logs deliberately not read. Confidence: high for supervise.log facts (primary source, verbatim quotes); medium for Windows-event interpretation (grouped digest lacks instance details).

## SUMMARY — top 5

1. **15.6-day supervision blackout (2026-10-13 04:00 -> 2026-10-28 17:21).** No heartbeats, no keep-alive runs; the Oct 18 18:11 power-off came and went with no supervisor activity. The scheduled task does not reliably trigger at boot/logon, and nothing alerts on heartbeat absence. (Certain — timestamps in-log.)
2. **Oct 12-13 flapping: 7 server restarts + 6 keep-alive restarts in ~4 h, ending only at the Oct 13 03:58 planned maintenance reboot.** Restart threshold is `misses=1` (hair-trigger) with no backoff; two `health failed after restart` in a row; the supervisor itself restarted 6x in 3 h (02:02:11 -> 02:03:13 is 62 s apart). (Certain for counts; cause inferred.)
3. **PID bookkeeping is broken.** `started serve.py X` repeatedly disagrees with `health ok pid=Y` (Oct 28: three distinct pids 7272/7348/5736 in play; Nov 5: started 6964, health ok 7020, `server.pid`=7020), and five restart lines log an **empty** pid (`pid= misses=1`). Old servers are never stopped on keep-alive restarts (no `stopped serve.py` lines there) -> orphan/port-collision risk; plausible cause of the `health failed after restart` loop. (Facts certain; mechanism is hypothesis.)
4. **The tunnel is unsupervised and its public URL is unstable or misrecorded.** `tunnel.err.log` shows a single 2026-10-03 session of an account-less **quick Tunnel** (`trycloudflare.com`, "no uptime guarantee", `ha-connections:1`), yet `public_url.txt` says `https://docx.rodrigues.ai` — a custom domain a quick tunnel cannot serve. No tunnel logs exist across >=3 reboots and supervise.log never mentions cloudflared. Either the tunnel is dead after every reboot or the state file is wrong; both are defects. (Contradiction certain; cause unresolved.)
5. **Nov 5 maintenance reboot: server came back, but recovery is unverified and the timeline does not reconcile.** Keep-alive returned 06:56:25, health ok 06:56:32 — only ~72 s after TrustedInstaller initiated the 06:55:13 "Upgrade (Planned)" restart — and 0 heartbeats were logged before the export. Tunnel restart after reboot: no evidence at all. Also, keep-alive restarts killed 15 WINWORD workers over the month (in-flight job loss each time).

Secondary: Schannel 36874 x701/30 d (~23/day, TLS cipher-mismatch probes — noisy, not crash-inducing); SCM 7034 x4 + 7031 x1 + 7023 x1 (which service crashed is NOT determinable from the grouped digest); DCOM 10000 x3; Kernel-Boot 124 x4; VSS 13 x3 + 8193; no 6008/41 unexpected-shutdown events in the digest (all four captured reboots are 1074 planned/administrative).

## EVIDENCE (verbatim, timestamps as logged)

### Supervisor restarts of server/workers

Server starts: 19 total; 12 by keep-alive session restarts, 7 by health-failure restarts — all 7 inside one window (2026-10-12 23:32 -> 2026-10-13 01:04):

    2026-10-12 23:32:14 restarting healthy=False pid= misses=1
    2026-10-12 23:33:47 restarting healthy=False pid=8636 misses=3
    2026-10-12 23:35:05 restarting healthy=False pid=7200 misses=4
    2026-10-12 23:48:23 restarting healthy=False pid= misses=1
    2026-10-13 00:16:03 restarting healthy=False pid= misses=1
    2026-10-13 00:23:59 restarting healthy=False pid= misses=1
    2026-10-13 01:04:33 restarting healthy=False pid= misses=1

Health check contradicting itself in the same second:

    2026-10-12 23:35:11 health ok pid=8992
    2026-10-12 23:35:11 heartbeat pid=8992 healthy=False
    2026-10-12 23:45:18 heartbeat pid=8992 healthy=True   <- same pid, healthy 10 min later, no restart in between

-> the post-restart health probe and the heartbeat probe disagree on the same live process.

Two consecutive failed restarts during the flap:

    2026-10-12 23:33:17 health failed after restart
    2026-10-12 23:34:50 health failed after restart

Keep-alive (supervisor) itself restarted 12x in 32 days, 6x on Oct 13 alone; 02:02:11 -> 02:03:13 = 62 s apart, and each restart kills both Word workers:

    2026-10-13 01:18:40 keep-alive pid 9976 session 2 parent svchost.exe 2196
    2026-10-13 02:02:11 keep-alive pid 2308 session 2 ... / 02:02:11 stopped WINWORD 916 / stopped WINWORD 5444
    2026-10-13 02:03:13 keep-alive pid 8956 session 2 ... / 02:03:13 stopped WINWORD 9344 / stopped WINWORD 4064
    2026-10-13 02:31:28 keep-alive pid 156 session 2 ...
    2026-10-13 02:50:16 keep-alive pid 9880 session 2 ...
    2026-10-13 03:14:30 keep-alive pid 4748 session 2 ...

The supervisor never logs why it exits or why health failed — no error capture for its own crashes.
Old serve.py is never stopped on keep-alive restarts (lines above stop WINWORD only); `serve.py 7928` (started 2026-10-04 05:05:59) has **no** `stopped serve.py 7928` line anywhere before the Oct 12 23:32 restart-with-empty-pid.
WINWORD kills: 15 total — 2 on Oct 4, 1 on Oct 12, 10 on Oct 13, 2 on Oct 28.

### PID mismatches (started != health-ok)

    2026-10-13 02:50:17 started serve.py 6868 python=C:\word-endpoint\.tools\python315-bench\Scripts\python.exe
    2026-10-13 02:50:20 health ok pid=8628
    2026-10-13 03:14:32 started serve.py 9460 python=...
    2026-10-13 03:14:33 health ok pid=4648
    2026-10-13 04:00:10 started serve.py 7536 python=...
    2026-10-13 04:00:15 health ok pid=7592
    2026-10-28 17:21:20 started serve.py 7272 python=...
    2026-10-28 17:21:23 health ok pid=7348
    2026-10-28 19:10:48 started serve.py 7096 python=...
    2026-10-28 19:10:50 health ok pid=5736      <- neither 7272, 7348, nor 7096
    2026-11-05 06:56:27 started serve.py 6964 python=...
    2026-11-05 06:56:32 health ok pid=7020

All mismatches begin 2026-10-13 02:50, exactly when the `python315-bench` wrapper path first appears. Before that date started-pid == health-pid (`started serve.py 7928` / `health ok pid=7928`, 2026-10-04 05:05). Heartbeats then track the health pid (`heartbeat pid=5736 healthy=True` for 8 days, Oct 28 -> Nov 5). State files now: `server.pid` = 7020, `supervise.pid` = 4308 (= the 06:56 keep-alive pid — consistent).

### Heartbeat gaps (service unmonitored / absent)

    GAP  9,288 s: 2026-10-04 02:41:19 -> 2026-10-04 05:16:07   (~2.6 h; keep-alive back 05:05:58 as "session 2")
    GAP  1,349 s: 2026-10-13 01:06:21 -> 2026-10-13 01:28:49   (during flap)
    GAP  2,549 s: 2026-10-13 01:59:09 -> 2026-10-13 02:41:38   (during flap)
    GAP 1,349,392 s: 2026-10-13 02:41:38 -> 2026-10-28 17:31:30  (15.6 DAYS)
    GAP  1,108 s: 2026-10-28 19:02:29 -> 2026-10-28 19:20:57   (keep-alive restarted 19:10:47)
    GAP  4,205 s: 2026-11-01 01:31:36 -> 2026-11-01 01:41:41   (70 min, NO restart lines — supervisor silently absent)

The 15.6-day gap begins despite a clean post-reboot start: `2026-10-13 04:00:15 health ok pid=7592` is followed by **zero** heartbeats until Oct 28 (first post-gap heartbeat is pid=7348, the Oct 28 17:21 start). The Oct 13 04:00 server was left running with no monitoring from Oct 13 04:00 onward.

### Tunnel

- `tunnel.log` is 0 bytes; `tunnel.err.log` contains exactly one session, started 2026-10-03T20:53:53Z, nothing since — across the Oct 13, Oct 18 and Nov 5 reboots, no new tunnel session was ever logged.
- Quick tunnel, not a named tunnel:

      2026-10-03T20:53:53Z INF Thank you for trying Cloudflare Tunnel. Doing so, without a Cloudflare account, is a quick way to experiment ... these account-less Tunnels have no uptime guarantee ...
      2026-10-03T20:53:53Z INF Requesting new quick Tunnel on trycloudflare.com...
      2026-10-03T20:53:58Z INF |  https://formula-electric-checkout-some.trycloudflare.com  |
      2026-10-03T20:53:58Z INF Settings: map[ha-connections:1 no-autoupdate:true protocol:quic url:http://127.0.0.1:8787]

- `public_url.txt` (whole file): `https://docx.rodrigues.ai` — a custom domain, which a quick tunnel cannot publish. The contradiction stands in the exported files.
- A quick tunnel URL is generated per-run: every tunnel restart changes it, and nothing in these logs rewrites `public_url.txt`.
- `ha-connections:1` — a single edge connection; one lost QUIC connection drops public requests until reconnect.
- supervise.log contains zero tunnel/cloudflared lines: the supervisor neither starts nor health-checks the tunnel.

### Windows event digest (eventlogs.txt, grouped 30 d)

    701 Schannel, 36874        305 TPM-WMI, 1796        4 Kernel-Boot, 124      4 SCM, 7034
      3 DCOM, 10000            4 Hyper-V-Hypervisor, 42  1 SCM, 7031            1 SCM, 7023
      3 VSS, 13                1 VSS, 8193              3 ESENT, 533/508       235 Security-SPP, 8233

- The "APPLICATION: WINWORD/python crash details" section is **empty** — no WER/crash events for WINWORD or python, despite 15 supervisor-killed Word processes (task-kill != crash, but the Oct 12 health failures also left no OS-level crash trace).
- Shutdown events, all 1074 (planned/administrative; **no** 6008/41 unexpected-shutdown events in the digest):

      10/13/2026 3:58:39 AM  1074 SystemSettingsAdminFlows.exe, Administrator, "Application: Maintenance (Planned)", 0x84040001, restart
      10/18/2026 6:11:20 PM  1074 winlogon.exe, SYSTEM, power off, 0x500ff          <- "No title"; 0x500ff is the code typically logged for power-button/hard power-off
      11/5/2026 6:45:18 AM   1074 shutdown.exe, Administrator, restart, 0x800000ff  <- "No title for this reason could be found"
      11/5/2026 6:55:13 AM   1074 TrustedInstaller.exe, SYSTEM, "Operating System: Upgrade (Planned)", 0x80020003, restart

### Nov 5 maintenance reboot correlation (supervise.log x event log)

    2026-11-05 06:27:40 heartbeat pid=5736 healthy=True     <- last pre-reboot heartbeat
    [event] 11/5/2026 6:45:18 AM shutdown.exe restart (Administrator)
    [event] 11/5/2026 6:55:13 AM TrustedInstaller "Upgrade (Planned)" restart
    2026-11-05 06:56:25 keep-alive pid 4308 session 1 parent svchost.exe 2140
    2026-11-05 06:56:27 started serve.py 6964 python=C:\word-endpoint\.tools\python315-bench\Scripts\python.exe
    2026-11-05 06:56:32 health ok pid=7020                    <- last line of the log; 0 heartbeats since

- Health was continuously `healthy=True` through 06:27:40; the service did not fail before the reboot — the outage window is purely the reboot.
- Keep-alive returning 06:56:25 in `session 1` is consistent with an interactive logon having happened (auto-logon armed and working), but it is only 72 s after the 06:55:13 restart initiation — either the VM rebooted in ~1 minute, or this start belongs to the dying boot and the machine went down again right after 06:56:32. Not resolvable from this slice.
- Between 06:27:40 and 06:56:25 there is no supervisor output at all (~29 min dark, expected during reboot).

## ROOT-CAUSE HYPOTHESES

1. **H1 — scheduled-task trigger gap (blackout).** The keep-alive runs as a scheduled task (parent always svchost.exe) whose trigger appears periodic-while-running rather than at-boot/at-logon; when the VM was off/stopped (Oct 13 04:00 -> Oct 28, power-off Oct 18 18:11) nothing re-armed it, and no external alert exists on heartbeat absence. Explains the 15.6-day, Oct 4 and Nov 1 gaps. Confidence: medium-high (trigger config not in this slice).
2. **H2 — restart threshold 1 + no backoff amplified a transient health miss into the Oct 12-13 restart storm.** `misses=1` restarts the whole server (and later both Word workers) on one failed probe; during long compare jobs (contract allows 120 s jobs / 180 s queue waits) a busy loop can miss a probe. Each restart killed in-flight work -> client retries -> more load -> more misses; the storm ended only at the maintenance reboot. Confidence: medium (worker logs out of scope).
3. **H3 — double-spawn via the python315-bench wrapper breaks pid identity.** Since 2026-10-13 02:50, `Scripts\python.exe` (a launcher) is used; the supervisor records the launcher pid while /health reports the real interpreter pid (6868->8628, 6964->7020). Then `stopped serve.py <pid>` can kill the launcher and orphan the real listener on :8787; `server.pid` goes stale/empty (5 x `pid= misses=1`); a new server failing to bind (port held by an orphan) yields `health failed after restart`. Competing hypothesis: the health probe reached an older orphaned server (Oct 28: three pids 7272/7348/5736). Both fit the quotes; distinguishing needs `netstat -ano` / `tasklist` on the host. Confidence: mechanism medium, facts high.
4. **H4 — the quick tunnel is a leftover experiment or dead weight; the production path is elsewhere or broken.** The only tunnel evidence is an account-less quick tunnel from Oct 3 whose URL (formula-electric-checkout-some.trycloudflare.com) matches neither public_url.txt nor a stable endpoint. If docx.rodrigues.ai is a *named* tunnel, it runs outside this supervision stack with no captured logs/health; if not, the public URL rotates (or dies) on every reboot with no re-publish of public_url.txt. Confidence: contradiction certain; which branch is true unknown.
5. **H5 — the supervisor has no self-observability.** No exit reasons, no stderr capture, no tunnel checks, no alert on heartbeat gap; every "why" above is inference because the layer logs only happy-path transitions. Confidence: high (absence is verifiable).

## CONCRETE FIXES

1. **Fix the task trigger + add dead-man alerting.**
   - Change: give the keep-alive task *At startup / At logon* triggers + Task Scheduler restart-on-failure; add an off-host alert when no heartbeat appears for 15 min (heartbeat also pings a healthchecks.io-style dead-man switch).
   - Effect: no repeat of the 15.6-day silent blackout; reboot recovery guaranteed rather than assumed.
   - Verification: reboot the VM outside maintenance hours; assert a keep-alive line within 5 min of logon; stop the task deliberately and confirm the external alert fires.
2. **Raise restart threshold, add backoff, unify health probes.**
   - Change: require `misses>=3` consecutive failures before restarting; exponential backoff (30 s -> 2 min -> 10 min) between restarts; make the heartbeat use the same probe as the post-restart health check (the 23:35:11 `health ok` + `healthy=False` same-second contradiction shows two different probes); do not kill WINWORD workers during a restart the server itself can recycle.
   - Effect: transient misses during 120 s jobs no longer destroy in-flight work; restart storms like Oct 12-13 become structurally impossible.
   - Verification: unit-test the supervisor loop with an injected flaky /health; assert no restart under 2 misses and backoff spacing; replay the Oct 12 timeline in simulation.
3. **Make pid identity single and authoritative.**
   - Change: launch the real python.exe directly (bypass the `Scripts\` launcher shim) or resolve and record the listening pid (from /health immediately, written to `server.pid`); on stop, kill the process tree (`taskkill /T`), not the launcher pid; log `stopped serve.py <pid>` for every stop including keep-alive restarts.
   - Effect: kills reach the real server; orphans on :8787 stop accumulating; `pid= misses=1` lines disappear.
   - Verification: after the change every `started serve.py X` is followed by `health ok pid=X`; `netstat -ano | findstr 8787` shows exactly one LISTEN pid equal to `server.pid`; run 10 keep-alive restarts and count surviving pythons.
4. **Put the tunnel under supervision with a stable identity.**
   - Change: replace the quick tunnel with a pre-created **named tunnel** (`cloudflared tunnel create`) bound to docx.rodrigues.ai via DNS CNAME, run as a Windows service or supervised by the keep-alive, log captured, plus an HTTP GET through the public URL added to the heartbeat; automate `public_url.txt` to reflect reality.
   - Effect: URL never changes across reboots; tunnel restarts are logged and detected; raise `ha-connections` to 4 for redundancy.
   - Verification: `nslookup docx.rodrigues.ai` shows the `<tunnel-id>.cfargotunnel.com` CNAME; reboot the VM and confirm the public URL serves 200 within 5 min with zero URL change; grep supervise log for the new tunnel check lines.
5. **Close the Windows-event blind spots.**
   - Change: re-export with instance-level details (digest is grouped only): which service threw the 4x 7034 / 1x 7031 / 7023, what the 3x DCOM 10000 and 4x Kernel-Boot 124 events name, sample Schannel 36874 source IPs (likely internet scanning; silence by hardening TLS on the exposed listener, e.g., RDP/NLA). Add WER LocalDumps for WINWORD/python so the empty "WINWORD/python crash details" section can populate.
   - Effect: separates infrastructure crashes (SCM) from scanning noise (Schannel) and yields crash evidence for the next health-failure investigation.
   - Verification: rerun the export per event Id; confirm each 7034 names a service and none are cloudflared/word-endpoint components.

## Open items (not resolvable from this slice)

- Whether a *named* tunnel exists serving docx.rodrigues.ai (needs `cloudflared tunnel list` / DNS check on the host).
- Which services produced the 7034/7031/7023/10000 events (needs instance-level event export).
- Whether the 06:56:25 keep-alive run was pre- or post-Upgrade-reboot (needs Kernel-General event 12 boot time or the next heartbeat after export).
- Whether the Oct 18 18:11 0x500ff power-off was a hard loss (would normally produce a 41/6008; their absence is worth re-checking with an ungrouped query).
- Job-level cause of the first Oct 12 23:32 health failure (needs worker logs — out of scope here).
