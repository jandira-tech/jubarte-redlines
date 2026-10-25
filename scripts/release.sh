#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# One-stop release for the whole jubarte family:
#
#   scripts/release.sh 0.9.2 \
#     --changelog-summary "…"  required → `> **Summary.** …` under ## [0.9.2]
#     --crates-summary "…"     required → [package.metadata.release-notes]
#                              "0.9.2" = "…" in Cargo.toml (cargo preserves
#                              package.metadata in the published .crate)
#     --npm-summary "…"        required → releaseNotes."0.9.2" in
#                              jubarte-wasm/npm/package.json (npm publishes
#                              unknown fields into the packument)
#     --pypi-summary "…"       required → `# release-notes v0.9.2 — …` in
#                              pyproject.toml (ships verbatim in the sdist)
#                              + the metadata table in jubarte-python/Cargo.toml
#     --github-summary "…"     required → annotated-tag body; release.yml
#                              prepends it to the GitHub release notes
#     --how-readme-and-other-docs-were-updated "…"
#                              required → `> **Docs.** …` under the changelog
#                              summary + the release commit body; step 4
#                              lists the README/docs files changed since the
#                              previous tag beside it
#
#   A `--*-comments` alias exists for every `--*-summary` flag. All six are
#   mandatory — none can be skipped or overridden, and the verify phase greps
#   each shipped artifact/registry to prove the note actually landed.
#
#   other flags:  --dry-run     local only; writes the bump + summaries but
#                               commits/pushes/publishes nothing
#                 --yes         skip the type-the-version confirmation (CI)
#                 --skip-gates  reuse an earlier gate pass (retries)
#                 --local-publish
#                               steps 9-11 publish from this machine with the
#                               releaser's tokens (the old path) instead of
#                               waiting for release.yml's OIDC publish jobs —
#                               the fallback when a trusted publisher fails
#                 --no-wait     with --local-publish: PyPI gets sdist+local
#                               wheel instead of CI wheels
#                 --open-docs   step 6 also opens the rendered rustdoc in the
#                               browser (the API sign-off itself is automatic:
#                               cargo-semver-checks in step 0)
#                 --checklist   print the whole release checklist — before the
#                               script, each step, after the script — every
#                               line [auto] or [you] with its proving command;
#                               it covers the website's pages, the app and the
#                               App Store, both changelogs, release.yml, the
#                               registries and every post-release flow; exits
#                               0, touches nothing, works on a dirty tree and
#                               off main (takes an optional VERSION only to
#                               print it in the commands)
#
# What it does, in order:
#   0. preflight — tools, registry credentials, main branch, clean tree;
#      the bump kind docs/VERSIONING.md asks for (a patch has no ### Added
#      and passes cargo-semver-checks against the previous release); the
#      release.yml review (scripts/test_release_workflow.py) and the
#      GitHub settings it relies on (the release environment, the signing
#      secret)
#   1. version sync — Cargo.toml and the README's Socket badge version
#      (bump-version.mjs), jubarte-python/Cargo.toml,
#      jubarte-wasm/npm/package.json, jubarte-wasm/cli/package.json (the
#      `npx jubarte-redlines` CLI), and all four Cargo.lock files; then one
#      README per published library (crates.io, PyPI, both npm packages)
#      cut from README.md by scripts/library_readmes.py, links pinned to
#      the tag
#   2. changelog check — dated `## [x.y.z]` section + release-link footer;
#      every fix/feat/perf/security PR merged since the previous release is
#      cited in the section (its number or an issue it closes), and the
#      section holds no placeholder
#   3. release evidence — scripts/check_release_info.py proves the six
#      release_info/ files of $VER (the two 600-item sample CSVs with every
#      file's sha256 — hashed against the bench's own files when a bench
#      checkout is at hand (--bench-root), format-checked only otherwise
#      and then said so; the two results JSONs scored on exactly those
#      samples, bound to them by the CSV's real sha256 and to the scored
#      binary by its commit + sha256; and the website/app change lists);
#      the bench writes them before this script runs
#      (release_info/README.md), and the release commit carries them
#   4. summaries — the five summaries + the docs statement land in their
#      channels
#   5. gates — fmt, clippy -D warnings, test --all-features, convert-sweep
#      unit tests, REUSE lint (sequential cargo per AGENTS.md)
#   6. api docs drift — the private docs build with warnings denied (and
#      open in the browser with --open-docs); a machine-readable snapshot
#      lands in docs/api/ (rustdoc JSON + flat api.txt + the wasm .d.ts
#      files) and its drift from the previous release's copy is logged,
#      the public surface first; the bump it implies was proven in step 0
#   7. publish dry-runs — cargo publish --dry-run, npm --dry-run, maturin sdist
#      (with the pypi comment proven inside the sdist); the crate's
#      keywords and categories inside crates.io's limits
#   8. `chore(release): vX.Y.Z` commit, wasm npm rebuild (stamps the release
#      commit into ENGINE_COMMIT.txt), npm smoke test, artifacts commit,
#      annotated `vX.Y.Z` tag whose body is the github summary
#      point of no return — type `vX.Y.Z` to confirm, then push (where main
#      takes changes only through a pull request, scripts/push_main.sh opens
#      one for the release commits and merges it at once), cancel the CI
#      runs in flight so release.yml gets the runners, push the tag; release.yml
#      builds the five CLI binaries + seven PyPI wheels + sdist and creates
#      the `jubarte vX.Y.Z` GitHub release itself
#   9. crates.io — release.yml's `publish crates.io` job publishes through
#      OIDC trusted publishing (#1642), after proving the summary is inside
#      the .crate; this step waits for the registry to list it
#  10. npm — release.yml's `publish npm` job: jubarte-wasm, then the
#      jubarte-redlines CLI; this step waits for both
#  11. PyPI — release.yml's `publish PyPI` job uploads the CI wheels + the
#      sdist; this step waits for the GitHub release and for PyPI
#      With --local-publish, steps 9-11 publish from here instead:
#      `cargo publish`, `npm publish` ×2, `uv publish` of the CI wheels
#  12. verify — every registry answers with the new version AND its summary;
#      one archive's build provenance is verified (a warning when missing)
#  13. app — the app repository's scripts/release-engine.sh, from its
#      checkout (jubarte-app/, untracked here, or JUBARTE_APP_DIR; step 0 has
#      already run its --preflight): jubarte.pro moves to the release — the
#      download page, the demo engine and the benchmark figures of
#      release_info/website_data — is tested, committed to the app's main,
#      deployed, the live benchmark page is read against release_info/, and
#      the app's data/facts.jsonl must name the release and its every wheel;
#      the app's version files go on its release/vX.Y.Z branch with a pull
#      request; the Mac App Store and benchmark commands are printed
#
# Idempotent: each publish checks the registry first and skips a version
# that is already live, so a failed run can simply be re-run.
#
# Credentials (preflight checks each):
#   GitHub     `gh auth login`                    (drives the release + wheels)
#   The registries need none: release.yml publishes through OIDC trusted
#   publishing (workflow release.yml, environment release). Only
#   --local-publish asks for them:
#   crates.io  `cargo login`                      (~/.cargo/credentials.toml)
#   npm        `npm login`                        (npm whoami must answer)
#              NPM_OTP=<code>                     (two-factor accounts, non-TTY)
#   PyPI       UV_PUBLISH_TOKEN=pypi-…            (uv publish --token)
set -euo pipefail
# Under pipefail, `producer | grep -q` fails when grep exits on its first
# match and the producer dies of SIGPIPE, so a found line reads as missing
# (a resumed 0.9.3 run redid its release commit that way). Match with
# `grep … >/dev/null`, which reads the whole stream, never `grep -q`.
cd "$(dirname "$0")/.."

usage() { sed -n '/^# One-stop release/,/^set -euo pipefail$/p' "$0" | sed '$d' >&2; }

# --- the release checklist — ONE source of truth ------------------------------
# Every item a releaser must inspect, so none of it lives only in someone's
# head, a chat or a loop note.  Each line: phase|mark|item|proof.  phase is
# "before", a step number ("0".."13") or "after"; mark is "auto" (a
# release.sh line enforces it) or "you" (a human does it at that moment);
# proof is the command or file that proves it.  @V@ becomes $VER at print
# time.  --checklist prints all of it; check_now <step> prints one step's
# [you] lines inside a real run at the moment they matter; the closing
# summary prints every [you] line still owed after the script ends.
checklist_items() { cat <<'ITEMS'
0|auto|Security findings on GitHub, read with an admin's token (GitHub shows unpublished advisories to no other, #1761): no repository advisory in triage (a private report), no draft above low that this release does not patch (its patched versions name @V@), no open medium-or-higher Dependabot or code-scanning alert; low drafts and lower alerts are listed (#1750)|scripts/release.sh step 0 (security_gate: gh api repos/jandira-tech/jubarte-redlines/{security-advisories,dependabot/alerts,code-scanning/alerts})
before|you|Security review, the half a script cannot do: record known CVE/GHSA fixes in the changelog, and run all four sanitizer fuzz targets plus native allocator runs (step 0 reads the advisories and alerts itself); confirmed exploitable medium/higher findings block release|SECURITY.md; fuzz/README.md; cargo +nightly fuzz run <target> -- -max_total_time=60 (admit, strict_to_transitional, compare, relationships); bash scripts/fuzz-native.sh <target> -max_total_time=60 (compare, relationships)
before|you|Full suite once before the release: cargo miri nextest run over the whole crate — nightly with the miri component (.config/nextest.toml holds its default-miri profile); know its cost first, and read its CARGO_BIN_EXE and disk-fixture failures as Miri's isolation rule, not engine bugs; when the owner skips it (0.14.0: miri not installed), the release notes say the full suite did not run|cargo +nightly miri nextest run (AGENTS.md; Miri cannot spawn processes and refuses the host's file system under its default isolation)
before|you|The quality gate of docs/VERSIONING.md on the release binary: the parity ladder sweep shows no NEW fails, and the criterion bench holds its baseline — any case more than 5 percent worse blocks, a median generate-time regression past +10 percent triggers the speed review before the release|tools/parity_ladder.py sweep --bin target/release/jubarte; cargo bench --bench redline -- --baseline <baseline> (docs/SPEED_REVIEW.md)
before|you|Decide where the release runs: a clean worktree on main, release PR merged, main in sync with origin/main — the canonical checkout is the owner's and is never the release tree|git worktree add <dir> main; git -C <dir> status --porcelain (empty); git -C <dir> rev-parse HEAD == origin/main
before|you|release_info: the bench wrote the six evidence files for @V@ BEFORE this script runs (its step 3 refuses without them); with a bench checkout at hand the sha256 columns are verified against the real files. An interrupted flow resumes with --only and the stages still owed (install, sample, generate, export, measure, convert, score, write; the order never changes). LibreOffice renders the fixtures it lacks only as 26.8.0.3, the build its work folder names, and the flow refuses another — when the soffice on PATH is newer, install 26.8.0.3 beside it and export SOFFICE=<that LibreOffice.app>/Contents/MacOS/soffice|cd ../neurotic_docx_bench && uv run python -m neurotic_docx_bench.jubarte_release_info @V@ --engine-dir <engine> --binary <candidate>
before|you|identity: the candidate that produced the scored outputs is byte-identical in output to the release commit's binary on the release samples (the two-binary identity run, exit 0 and its identity.log kept), or the assumption is stated in the release notes when skipped|python3 scripts/identity.py <candidate> <release-commit binary> <out dir> release_info/sample_*_@V@_*.csv --root ../neurotic_docx_bench (outputs named by row index; keeps only differences)
before|you|parity: the CLI, PyO3 and WASM bindings agree on the samples, build identity proven (--expect-commit HEAD), stale artifacts refused|bash tools/parity/build.sh tools/parity/parity-work; python3 tools/parity/parity_check.py --cli target/release/jubarte --py-python jubarte-python/.venv/bin/python --wasm tools/parity/parity-work/wasm-pkg --expect-commit "$(git rev-parse HEAD)" --engine-dir . --pairs release_info/sample_redline_@V@_*.csv
0|auto|release.yml reviewed before the tag, by scripts/test_release_workflow.py on Python 3.12: the wheel matrix (7 jobs) equals the README's supported-environments row and scripts/check_release_artifacts.py REQUIRED_WHEEL_TAGS, the binary matrix (5 jobs) equals step 11's list, the three publish jobs (crates.io, PyPI, npm) run in the release environment the trusted publishers name (#1642) and alone with the GitHub-release attestation may mint an id-token, every action is SHA-pinned; GitHub holds the release environment and, whenever keys/ holds a public key, the ZIPSIGN_PRIVATE_KEY secret — the signed era refuses unsigned archives|uv run --no-project --python 3.12 --with pyyaml python scripts/test_release_workflow.py; gh api repos/jandira-tech/jubarte-redlines/environments/release (docs/SELF_UPDATE.md, Signed releases)
before|you|CI of the release PR: read each red check's log — fix only genuine test or coverage failures; billing, quota, paused-bot and runner-offline reds are reported and left; pushes that start hosted runners are batched into ONE|gh run view <run-id> --log-failed (one batched push per review round)
8|auto|main takes the release: scripts/push_main.sh pushes, or opens a pull request for the release commits and merges it at once; a refusal (a "Restrict updates" rule with an empty bypass list refuses even an admin merge, and only the owner changes rulesets) stops the run with nothing published|gh api repos/jandira-tech/jubarte-redlines/rulesets
before|you|a run in flight owns its worktree: the version bump stays uncommitted from step 1 to step 8 by design — nobody commits, pushes, edits or tidies there until the script ends (0.11.2: the bump was committed mid-run with a .gitignore line that made step 8 unable to stage the vendored app); and only release.sh pushes main and the tag — a hand-pushed tag starts release.yml while steps 9 to 13 never run here (0.13.0: step 13 never ran, so jubarte.pro and the app's facts stayed on 0.12.0); an interrupted run resumes by running release.sh again with the same summaries|git -C <release worktree> status --short (changes there during a run are the release, not dirt)
8|auto|Queued CI is cancelled right before the tag goes up, after the push to main started CI and Docs again, so release.yml gets the runners first; release.yml's own run is kept|scripts/release.sh (cancel_queued_ci, between push_main and the tag push)
before|you|stack: the release binary survives a 1 MiB main-thread stack (Windows' default) on the release samples — every job exits zero|python3 scripts/stack1m.py <release binary> ../neurotic_docx_bench <out dir> release_info/sample_*_@V@_*.csv
5|auto|Python 3.12: the script tests run on the scripts' floor, Python 3.12 (ruff.toml targets py312 and reports newer syntax), not on the machine's python3|uv run -q --no-project --python 3.12 python scripts/test_release_sh.py
before|you|unused: scan every manifest for dependencies nothing references — each workspace Cargo.toml, every package.json, pyproject (0.11.2 found js-sys in jubarte-wasm/Cargo.toml)|grep -rn <crate> src jubarte-wasm/src benches examples (empty output = removable)
before|you|API snapshot: docs/api holds the previous release's rustdoc snapshot, the baseline step 6 logs the drift against; a release that skipped step 6 left none (0.13.0), so backfill it from its tag before the run and commit it with the release pull request|git worktree add <dir> <previous tag>; (cd <dir> && python3 scripts/api_snapshot.py <previous version>); cp <dir>/docs/api/jubarte-<previous tag>.json.gz <dir>/docs/api/jubarte-<previous tag>.api.txt docs/api/
before|you|hand-run Python proofs pin the interpreter to --python 3.12: uv's default on the release machine may be a free-threaded build (0.14.0: 3.15t) that no abi3 wheel serves, and an unpinned pip proof then builds the sdist and passes on nothing PyPI serves (0.13.0's PyPI PASS)|uv python find; uv run -q --isolated --no-project --python 3.12 --no-build-package jubarte-redlines --with jubarte-redlines==@V@ python -c 'import jubarte_redlines as j; print(j.capabilities()["engine_version"])'
before|you|the app's scripts are bash (release-engine.sh, build-release-app.sh, notarize-direct.sh, publish-mac-app-store.sh): run each by its path or with bash, never sh or zsh — the release machine's shell is zsh, which does not word-split a variable and indexes arrays from 1|for f in release-engine.sh build-release-app.sh notarize-direct.sh publish-mac-app-store.sh; do bash -n jubarte-app/scripts/$f; done
7|auto|keywords: the crate's keywords and categories are inside crates.io's limits (at most 5 each, keywords ASCII, a letter first, at most 20 characters) — one more fails the publish after the point of no return|scripts/release.sh step 7 (crate_keywords_ok)
0|auto|The bump kind docs/VERSIONING.md asks for, proven before step 1 writes anything: pre-1.0 a patch carries no new feature (a ### Added heading) and no public API change cargo-semver-checks finds against the previous release's tag (0.10.1 shipped four breaking fields as a patch); a minor or major may|cargo semver-checks check-release -p jubarte-redlines --baseline-rev <previous tag> --release-type patch --default-features
2|auto|The engine CHANGELOG section is complete: release.yml cuts it verbatim into the GitHub release notes, so it cites every fix/feat/perf/security PR merged since the previous release (by its number, an issue it closes, or a <!-- changelog-skip: #N --> line) and holds no TODO, TBD, FIXME or XXX|scripts/release.sh step 2 (unlisted_prs, changelog_placeholders)
before|you|The six one-line notes are drafted before the run — the changelog, crates, npm, pypi and github summaries and the docs statement are mandatory flags; the github note leads the release notes from the tag body, the other four are grepped back out of their registries at verify|scripts/release.sh --help (the five --*-summary flags and --how-readme-and-other-docs-were-updated)
before|you|CHANGELOG: engine section dated with the ASCII heading, link footer present; the app repository's own CHANGELOG carries its ## [@V@] section naming jubarte-redlines @V@ on its main (step 0 checks it)|grep -n '^## \[@V@\]' CHANGELOG.md jubarte-app/CHANGELOG.md (both sections are written by hand; step 3 holds their measured sentence to the results)
before|you|NUMBERS: the measured sentence is copied from the evidence's own aggregates before the summaries and the app CHANGELOG are written — never guessed, never rounded by hand; step 3 refuses a sentence the results do not hold|python3 scripts/check_release_info.py @V@ --bench-root ../neurotic_docx_bench (prints n, failures, mean and median per tool)
before|you|app repository: its data/facts.jsonl names the new engine (step 13) and its release/v@V@ pull request (the version files, step 13) is merged BEFORE the app is built — the app compiles facts.jsonl in|gh pr view release/v@V@ -R arthrod/jubarte-app; grep engine.version jubarte-app/data/facts.jsonl
before|you|app lock: jubarte-app/src-tauri/Cargo.lock records jubarte-redlines @V@ before any app build|grep -A1 'name = "jubarte-redlines"' jubarte-app/src-tauri/Cargo.lock
before|you|the app builds against the engine at the tag (src-tauri takes it by path "../.."): build in a fresh engine-at-tag + app-at-main layout, never in a release worktree mid-run|jubarte-app/scripts/build-release-app.sh v@V@ <fresh dir> (clones the engine at the tag beside the app, discards the vendored copy)
before|you|the app's checkout is in place: a clone of arthrod/jubarte-app at jubarte-app/ (this repository does not track it) or named by JUBARTE_APP_DIR, on main, in sync with its origin, clean, able to build the site (node_modules, public/fixtures) — step 13 deploys jubarte.pro from it|jubarte-app/scripts/release-engine.sh @V@ --engine-dir "$PWD" --preflight
before|you|the App Store path is ready for the after-builds: the asc credentials exist (~/.appstoreconnect), the app's provisioning profiles are present and out of every commit (gitignored; if one ever reached a repository's history, accept it knowingly or rotate it), and App Store Connect holds no @V@ version record already used — Apple refuses a re-used number|ls jubarte-app/src-tauri/*.provisionprofile; (cd jubarte-app && ./scripts/asc-build-status.sh)
before|you|Trusted publishers (#1642): crates.io jubarte-redlines, PyPI jubarte-redlines, npm jubarte-wasm and npm jubarte-redlines each trust owner jandira-tech, repository jubarte-redlines, workflow release.yml, environment release — release.yml publishes with no token. npm: each configuration must allow npm publish (one made after 2026-09-03 allows only npm stage publish unless that box is ticked, and a plain publish then fails E404), and a new configuration expires unless its first publish lands within two days — 0.13.0 never reached npm for this|npm trust list jubarte-wasm; npm trust list jubarte-redlines (both after npm login); npm trust github <package> --file release.yml --repo jandira-tech/jubarte-redlines --env release --allow-publish -y
before|you|Only with --local-publish: UV_PUBLISH_TOKEN is exported before the run (name only, never print it), crates.io and npm are logged in, and the owner is at the keyboard for npm's browser approval (2FA) — step 0 checks them only then|set -a; source <owner .env>; set +a; test -n "$UV_PUBLISH_TOKEN"; npm whoami
before|you|Ring-2 validity ratchet: no NEW OpenXML validator keys vs tools/validity_baseline.tsv; an output the validator cannot even open needs a Word probe before it ships|scripts/redline-sweep.sh tests/corpus/neurotic_docx_bench/corpus/word_based/centralized_mapping.csv <its sources> <out> --validate (the 207-pair mapping the baseline is keyed on; then the --probe sweep for refusals)
before|you|Ring-3 Word open probe, the macOS release gate before any crates.io publish: every redline the release ships opens in real Word — probe_fail=0; paths never under /tmp, stage them under parity/_scratch|JUBARTE_BIN=<release binary> scripts/redline-sweep.sh release_info/sample_redline_@V@_*.csv ../neurotic_docx_bench parity/_scratch/sweep_<date> --probe (sweep.log names the binary; no pair generated is a failure)
before|you|lane validity: every validator error kind in the redlines the release ships already exists in one of its two sources; a kind of the redline's own is read, then fixed or shown to be Word's own writing|python3 scripts/lane_validity.py release_info/sample_redline_@V@_*.csv --root ../neurotic_docx_bench --redlines parity/_scratch/sweep_<date> (the Ring-3 sweep's folder)
before|you|bench hygiene while scoring: one scoring job at a time; Word only through the bench scripts (--timeout 300, WD_MAX=300); probe documents pass tools/validate-docx first; nothing in /tmp|pgrep -fl '[w]ord_pdf.py'; pgrep -fl '[w]ord_redline.py'; pgrep -fl '[j]ubarte_release_info' (each empty before a scoring job starts)
0|auto|Preflight: main branch, clean tree, tools, gh credentials; crates.io + npm credentials and UV_PUBLISH_TOKEN only with --local-publish|scripts/release.sh step 0 (its die lines)
0|auto|The release tree is never the canonical checkout (~/T/jubarte-redlines through any link, or JUBARTE_CANONICAL_CHECKOUT); HEAD's check runs that did not pass are printed as a CHECK NOW for the releaser to triage, and so is a HEAD with no check run or none that succeeded (#1763); the run goes on (#1751)|scripts/release.sh step 0 (canonical_checkout, head_ci_report)
0|auto|The app's step can run: its checkout (jubarte-app/ or JUBARTE_APP_DIR) carries scripts/release-engine.sh, a CHANGELOG section naming jubarte-redlines @V@, and is on main, in sync, clean, with the site's release script and its tools — proved before anything is published, not found out at step 13|jubarte-app/scripts/release-engine.sh @V@ --engine-dir "$PWD" --preflight
1|auto|Version sync: every manifest, the 4 tracked Cargo.locks (., jubarte-python, jubarte-wasm, jubarte-rust-inproc; fuzz/ tracks none), the 4 library READMEs; a half-bumped tree from an interrupted run dies naming the file|scripts/release.sh step 1 (the half_bumped loop)
2|auto|Engine CHANGELOG dated ## [@V@] section + link footer|grep -n "^## \[@V@\]" CHANGELOG.md
3|auto|The six release_info files verified: results bound to their sample CSV by sha256 and to the scored binary by commit + sha256, website and app data records sound|python3 scripts/check_release_info.py @V@ --bench-root ../neurotic_docx_bench
3|auto|The evidence was scored on this tree's CLI engine: the commit both results JSONs name differs from HEAD in no engine input (src/ outside its comments, a layout change still counting; assets, schemas, keys byte for byte; Cargo.toml and Cargo.lock beyond the version and crates.io listing) — 0.11.2's evidence missed two code changes|python3 scripts/check_evidence_engine.py @V@
3|auto|The measured NUMBERS sentence of --changelog-summary and of the app's CHANGELOG section states the evidence's own figures at two decimals — sample size, both means, the comparator, the failures and their noun, every "no failed" claim in the text; a median or interval it states is not checked (keep them out, or check them by hand); it reaches the GitHub notes, the app's About window and jubarte.pro, so a mismatch ships a wrong claim three times|python3 scripts/check_release_info.py @V@ --summary "<changelog summary>" --changelog jubarte-app/CHANGELOG.md
4|auto|The five summaries + the docs statement land in their channels; step 4 lists the docs files changed since the previous tag beside the statement|scripts/release.sh step 4 (it greps each landing)
4|auto|docs/adoption pages: no row this release ships is still labelled main — step 4 dies naming each, to be flipped to released (the README's labels name no moving version)|scripts/release.sh step 4 (adoption_main_rows)
5|auto|Gates: fmt, clippy -D warnings, tests, public docs -D warnings, sweep units, script tests, pytest, REUSE lint; and, even under --skip-gates, every tracked file that can carry a comment (outside wasm-bindgen's npm/ glue, its docs/api snapshots and tests/fixtures/) carries SPDX-License-Identifier in its first 4 KiB, which reuse lint cannot see under REUSE.toml's **|uv tool run --from reuse[charset-normalizer] reuse lint; scripts/release.sh step 5 (headerless_files)
5|auto|Security scan: cargo-deny advisories across the five workspace manifests and gitleaks over the whole reachable history plus every publishable file — it runs even under --skip-gates|bash scripts/security-check.sh (unconditionally, just above the gates block)
6|auto|API drift: the private rustdoc builds with warnings denied and its drift from the previous release's snapshot is logged; the sign-off is step 0's cargo-semver-checks gate (--open-docs also opens the rendered docs)|scripts/release.sh step 6; scripts/api_snapshot.py --drift <previous tag> v@V@
7|auto|Dry-runs: cargo, npm x2, maturin sdist (folder wiped first) carrying the pypi comment; every registry ships the README cut for v@V@|scripts/release.sh step 7
8|auto|Release commit, wasm artifacts rebuilt + smoke-tested with ENGINE_COMMIT.txt = the release commit, annotated tag with subject 'jubarte v@V@' and the github summary in its body (a local tag of another shape is re-created; a pushed one must be the local tag, and of another shape dies unless the GitHub notes carry the summary — 0.13.0's tag held it as its subject); a kept (resumed) build whose stamp does not vouch for the tagged engine dies|scripts/check_engine_commit.sh v@V@
8|you|The typed tag confirmation at the point of no return — after it main and the tag are pushed and release.yml publishes|type v@V@ at the prompt (--yes skips it in CI only)
9|auto|crates.io: the packaged manifest carries the crates summary before the publish|tar -xzOf target/package/jubarte-redlines-@V@.crate jubarte-redlines-@V@/Cargo.toml
10|auto|npm: jubarte-wasm first, then the jubarte-redlines CLI (it depends on jubarte-wasm ^@V@); a one-time password is passed when set|npm view jubarte-wasm@@V@ version
10|you|When a publish job of release.yml fails, read its log: a refused token is a trusted publisher that does not match; npm's E404 on PUT with no provenance line is a configuration that does not allow npm publish, or one that expired before its first publish; fix it and rerun the job, or publish from here with --local-publish (then each npm publish may need the owner's browser approval or a fresh one-time password — codes live about 30 s)|gh run view <run-id> --log-failed; gh run rerun <run-id> --failed; NPM_OTP=<code> scripts/release.sh @V@ <summaries> --skip-gates --local-publish (the resume skips what is live)
11|auto|release.yml attached the five CLI binaries and the seven wheels; a partial wheel set never reaches PyPI under a final version|python3 scripts/check_release_artifacts.py dist/pypi --version @V@
11|you|When a wheel job fails: fix the release jobs and rerun the workflow on the tag (a run that lost a wheel never becomes a public release), or consent explicitly with --local-publish --no-wait to the partial set|gh run rerun <run-id> --failed; scripts/release.sh @V@ <summaries> --local-publish --no-wait
12|auto|Verify: every registry answers with the new version AND its summary|scripts/release.sh step 12 checks (crates.io, npm x2, PyPI, GitHub)
12|auto|Each channel installs the way a user installs it: this machine's archive matches SHA256SUMS.txt and answers jubarte @V@; pip install jubarte-redlines==@V@ in a fresh uv environment reports engine @V@; npx -y jubarte-redlines@@V@ --version answers @V@; npm i jubarte-wasm@@V@ then require() loads — 0.13.0 never reached npm and no step said so; docs.rs only warns, since it lags|scripts/release.sh step 12 (archive_runs, pip_installs, npx_runs, npm_requires, docsrs_built)
12|you|Read the v@V@ GitHub release page once, human eyes: five CLI archives (each signed once keys/ holds a public key), seven wheels, the sdist and SHA256SUMS.txt attached; the notes lead with the github summary, carry the CHANGELOG section and the Full Changelog link — a binary that missed the workflow run cannot join this tag, the next patch ships it|gh release view v@V@ --web
13|auto|The app's part, in its checkout: the download page and demo engine move to @V@, the app's version files go on release/v@V@ with a pull request; the App Store upload and the benchmark flow are printed, never run|jubarte-app/scripts/release-engine.sh @V@ --engine-dir "$PWD"
13|auto|The site's own release ladder runs inside it: sync-release.ts moves the download page's Engine line, the archive and wheel tables and the release history off the GitHub release, the demo engine is repinned to jubarte-wasm@@V@, sync-bench.ts lands the sample figures proved against the results JSONs, and the built pages pass pnpm test, lint and typecheck before anything deploys|jubarte-app/jubarte-site/scripts/release.sh release @V@ --no-deploy (inside step 13's app script)
13|auto|jubarte.pro figures: the bench.* records of release_info/website_data go into the app's data/facts.jsonl once, are proved against release_info, tested, linted, typechecked, committed to the app's main (a push, else a pull request merged at once), deployed, and the live benchmark page is read against the results JSONs — a failed check stops before the deploy|python3 jubarte-app/scripts/check_site_live.py @V@
13|auto|The app checkout's data/facts.jsonl names @V@ and every required wheel, and is committed there — the site step's last check|python3 jubarte-app/scripts/check_release_facts.py @V@
13|you|Read every jubarte.pro page the deploy rebuilt: the home hero names @V@ with fresh bars, /download prints Engine jubarte-redlines @V@ with the archive and wheel tables and the history row, /benchmark carries the two sample tables, states and headlines, /demo actually converts a document in the browser on the new engine, /pro and /terms are unchanged unless the app release moved them — the script's own checks read only /download and /benchmark|the browser: jubarte.pro home, /download, /benchmark, /demo, /pro, /terms
after|you|Merge the app's release/v@V@ pull request once its checks pass, before any app build — the app compiles data/facts.jsonl and the engine in by path, so a build off an open branch is not the release|gh pr merge release/v@V@ -R arthrod/jubarte-app --merge
after|you|app build: build the app TWICE in the fresh engine-at-tag + app-at-branch layout — the Mac App Store build with --target aarch64-apple-darwin, the Developer ID build without --target (notarize-direct.sh reads that one)|jubarte-app/scripts/build-release-app.sh v@V@ <fresh dir> (the App Store build, then the Developer ID build)
after|you|App Store, all of MAC_APP_STORE_RELEASE.md: upload the pkg, wait for VALID, create and attach the version record AND the Jubarte Pro Yearly in-app purchase, set the en-US What's New text, verify the metadata (screenshots, Privacy Policy URL jubarte.pro/privacy, App Privacy), answer export compliance, then Submit for Review — a human's click; Apple rejects a re-used version number|(cd jubarte-app && ./scripts/asc-build-status.sh && uv run --with cryptography python3 scripts/asc-new-version.py @V@ --apply)
after|you|notarize: Developer ID sign inside-out (nested bundles deepest first, then bare dylibs, then every Mach-O), notarize, staple, validate each artifact — the DMG lands at src-tauri/target/release/bundle/dmg/Jubarte_@V@_aarch64.dmg|ENTITLEMENTS=<entitlements-direct.plist> jubarte-app/scripts/notarize-direct.sh @V@ <app dir> (keychain profile notarytool-cicero)
after|you|benchmark lane upload: after the full-corpus run, the site fixtures restage uploads to the Hugging Face dataset, pins fixtures.lock to the @V@ lane and deploys — the release's own figures are on jubarte.pro since step 13; the pages this second deploy rewrites are read again (the use-cases strips now come from the @V@ lane, the full-corpus tables moved)|(cd jubarte-app/jubarte-site && scripts/release.sh bench @V@ --redline-tool jubarte-@V@) — its fixtures stage uploads and pins
after|you|RESULTS.md: the full-corpus run that feeds the bench RESULTS.md still runs for @V@|(cd ../neurotic_docx_bench && uv run scripts/release_jubarte.py @V@)
after|you|bench rerun: the same release-info flow re-runs WITHOUT --binary, on the release's own binary, now that the GitHub release exists|(cd ../neurotic_docx_bench && uv run python -m neurotic_docx_bench.jubarte_release_info @V@ --engine-dir <engine>)
after|you|reproduces: the released binary (downloaded from the GitHub release) reproduces the candidate's outputs on the release samples|python3 scripts/identity.py <released binary> <candidate> <out dir> release_info/sample_*_@V@_*.csv --root ../neurotic_docx_bench
after|you|self-update serves the release: jubarte self-update --version @V@ installs the archive, verifies it against the release's SHA256SUMS.txt (and its signature once keys/ holds a public key), and the replaced binary answers @V@ — the path every prebuilt user takes|jubarte self-update --version @V@ --yes; jubarte --version
after|you|the public pages of @V@ are read once, human eyes: docs.rs built the release's docs (step 12 warns while it has not), and each registry page shows the README cut for v@V@ with its note where the channel accepts one|open https://docs.rs/jubarte-redlines/@V@ https://crates.io/crates/jubarte-redlines/@V@ https://pypi.org/project/jubarte-redlines/@V@/ https://www.npmjs.com/package/jubarte-wasm/v/@V@ https://www.npmjs.com/package/jubarte-redlines/v/@V@
after|you|handoff: the release's running log and handoff notes are written down — nothing a releaser must remember lives only in a head or a chat|the release work folder's STATE.md, kept current to the end
after|you|app_store facts: the Terms copy that says the store's current version "is a one-time purchase" flips when @V@ goes live — decide when to rewrite it (a facts-only app patch, or pre-emptively now)|grep app_store jubarte-app/data/facts.jsonl
ITEMS
}

# The phase headers of --checklist, in print order (the step names match the
# say lines of the run).
checklist_phase_title() {
  case "$1" in
    before) echo "BEFORE the script" ;;
    0) echo "step 0 · Preflight" ;;
    1) echo "step 1 · Version sync" ;;
    2) echo "step 2 · Changelog check" ;;
    3) echo "step 3 · release_info evidence" ;;
    4) echo "step 4 · Summaries" ;;
    5) echo "step 5 · Gates" ;;
    6) echo "step 6 · API docs drift" ;;
    7) echo "step 7 · Publish dry-runs" ;;
    8) echo "step 8 · Release commit and tag" ;;
    9) echo "step 9 · crates.io" ;;
    10) echo "step 10 · npm" ;;
    11) echo "step 11 · PyPI" ;;
    12) echo "step 12 · Verify" ;;
    13) echo "step 13 · Downstream" ;;
    after) echo "AFTER the script — still owed when it ends" ;;
  esac
}

print_checklist() {
  local v p
  v="${VER:-x.y.z}"
  printf 'jubarte %s release checklist — scripts/release.sh --checklist\n' "$v"
  for p in before 0 1 2 3 4 5 6 7 8 9 10 11 12 13 after; do
    printf '\n-- %s\n' "$(checklist_phase_title "$p")"
    checklist_items | sed "s/@V@/$v/g" | awk -F'|' -v ph="$p" '
      $1 == ph { printf "  [%s] %s\n", $2, $3
                  if ($4 != "") printf "        proof: %s\n", $4 }
    '
  done
  printf '\n[auto] a release.sh line enforces it   [you] a human does it at that moment\n'
}

# A real run prints a step's open [you] items the moment it reaches them
# (nothing at all when the step has none).
check_now() {
  checklist_items | sed "s/@V@/$VER/g" | awk -F'|' -v ph="$1" '
    $1 == ph && $2 == "you" { printf "  CHECK NOW  %s\n             %s\n", $3, $4 }
  '
}

VER=""
DRY_RUN=0; YES=0; SKIP_GATES=0; NO_WAIT=0; CHECKLIST=0; LOCAL_PUBLISH=0; OPEN_DOCS=0
CHANGELOG_SUMMARY=""; CRATES_SUMMARY=""; NPM_SUMMARY=""
PYPI_SUMMARY=""; GITHUB_SUMMARY=""; DOCS_UPDATED=""
need() { [ -n "${2:-}" ] || { echo "missing value for $1" >&2; exit 2; }; }
while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)    DRY_RUN=1 ;;
    --yes)        YES=1 ;;
    --skip-gates) SKIP_GATES=1 ;;
    --no-wait)    NO_WAIT=1 ;;
    --local-publish) LOCAL_PUBLISH=1 ;;
    --checklist)  CHECKLIST=1 ;;
    --open-docs)  OPEN_DOCS=1 ;;
    --changelog-summary|--changelog-comments) need "$@"; CHANGELOG_SUMMARY=$2; shift ;;
    --crates-summary|--crates-comments)       need "$@"; CRATES_SUMMARY=$2; shift ;;
    --npm-summary|--npm-comments)             need "$@"; NPM_SUMMARY=$2; shift ;;
    --pypi-summary|--pypi-comments)           need "$@"; PYPI_SUMMARY=$2; shift ;;
    --github-summary|--github-comments)       need "$@"; GITHUB_SUMMARY=$2; shift ;;
    --how-readme-and-other-docs-were-updated) need "$@"; DOCS_UPDATED=$2; shift ;;
    -h|--help)    usage; exit 0 ;;
    -*)           echo "unknown flag: $1" >&2; usage; exit 2 ;;
    *)            if [ -z "$VER" ]; then VER="$1"
                  else echo "unexpected arg: $1" >&2; exit 2; fi ;;
  esac
  shift
done

# --checklist prints the one source of truth and leaves — before any
# validation and before the preflight, so it needs no summaries, no main
# branch, no clean tree, and runs no command and no network.
if [ "$CHECKLIST" = 1 ]; then
  print_checklist
  exit 0
fi

# Fold accidental newlines — every destination takes a single line.
fold() { printf '%s' "$1" | tr '\n\t' '  ' | tr -s ' '; }
CHANGELOG_SUMMARY=$(fold "$CHANGELOG_SUMMARY")
CRATES_SUMMARY=$(fold "$CRATES_SUMMARY")
NPM_SUMMARY=$(fold "$NPM_SUMMARY")
PYPI_SUMMARY=$(fold "$PYPI_SUMMARY")
GITHUB_SUMMARY=$(fold "$GITHUB_SUMMARY")
DOCS_UPDATED=$(fold "$DOCS_UPDATED")

missing=""
for pair in \
  "--changelog-summary|$CHANGELOG_SUMMARY" \
  "--crates-summary|$CRATES_SUMMARY" \
  "--npm-summary|$NPM_SUMMARY" \
  "--pypi-summary|$PYPI_SUMMARY" \
  "--github-summary|$GITHUB_SUMMARY" \
  "--how-readme-and-other-docs-were-updated|$DOCS_UPDATED"; do
  # A value that folds to spaces only is as missing as an absent one.
  [ -n "$(printf '%s' "${pair#*|}" | tr -d ' ')" ] || missing="$missing ${pair%%|*}"
done
if [ -n "$missing" ] || [[ ! "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  [ -n "$missing" ] && printf 'missing required summaries:%s\n' "$missing" >&2
  usage
  exit 2
fi
# --no-wait changes what this machine uploads to PyPI; without
# --local-publish nothing is uploaded from here.
if [ "$NO_WAIT" = 1 ] && [ "$LOCAL_PUBLISH" = 0 ]; then
  echo "--no-wait needs --local-publish: release.yml's publish PyPI job uploads the CI wheels" >&2
  exit 2
fi
TAG="v$VER"

say()  { printf '\n\033[1m== %s\033[0m\n' "$*"; }
step() { printf '  - %s\n' "$*"; }
die()  { printf '\033[31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

# Run a cargo packaging command without its one warning per tests/*.rs and
# examples/*.rs the crate's `include` leaves out of the package (431 lines in
# the 0.11.2 dry run, all by design). Every other line of stderr, stdout and
# the command's exit status pass through.
packaged() {
  { "$@" 2>&1 1>&3 \
      | { grep --line-buffered -v 'is not included in the published package$' >&2 || true; }
  } 3>&1
}

crate_ver() { grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2; }

# --- registry liveness probes (idempotent resume) -----------------------------
# crates.io answers 403 to a request without a User-Agent.
crates_has()  { curl -sf -A "jubarte-release (github.com/jandira-tech/jubarte-redlines)" \
                  "https://crates.io/api/v1/crates/jubarte-redlines/$VER" >/dev/null; }
# The project listing, not /pypi/<name>/<version>/json: PyPI's CDN keeps a
# 404 for that URL once anything asked before the upload (the v0.11.0 verify
# failed on a live release), while the listing is purged on upload.
pypi_has()    { curl -sf "https://pypi.org/pypi/jubarte-redlines/json" | python3 -c 'import json, sys; sys.exit(sys.argv[1] not in json.load(sys.stdin)["releases"])' "$VER"; }
npm_has()     { [ "$(npm view "jubarte-wasm@$VER" version 2>/dev/null)" = "$VER" ]; }
npm_cli_has() { [ "$(npm view "jubarte-redlines@$VER" version 2>/dev/null)" = "$VER" ]; }
ghrel_has()   { gh release view "$TAG" -R jandira-tech/jubarte-redlines >/dev/null 2>&1; }
# The release carries every advertised wheel and the sdist. release.yml
# creates the release first and attaches its assets after (0.11.2: step 11
# found the release, then no wheel, and died minutes before they arrived).
ghrel_wheels() { gh release view "$TAG" --json assets -q '.assets[].name' -R jandira-tech/jubarte-redlines </dev/null 2>/dev/null \
                   | python3 scripts/check_release_artifacts.py - --version "$VER" --sdist >/dev/null 2>&1; }
# The CI wheels and sdist of $TAG into dist/pypi. gh gets no terminal: under
# one (0.11.3 ran inside `script` for its log) the download's progress
# display queried it (OSC 11, cursor position) and failed while the release
# held all seven wheels, and the discarded error read as "no wheels". gh's
# own words are kept, and a release that lists its whole set but would not
# hand it over stops the run: a one-platform local wheel never stands in.
download_ci_wheels() {
  local err=dist/pypi-download.err
  if gh release download "$TAG" --pattern 'jubarte_redlines-*' \
      --dir dist/pypi --clobber -R jandira-tech/jubarte-redlines </dev/null >/dev/null 2>"$err" \
     && ls dist/pypi/*.whl >/dev/null 2>&1; then
    rm -f "$err"
    return 0
  fi
  echo "  ! gh release download $TAG failed:" >&2
  sed 's/^/      /' "$err" >&2
  ghrel_wheels \
    && die "the $TAG release lists every wheel, but gh would not download them (above) — fix that (gh auth status, the network) and rerun; no local wheel stands in for them"
  return 1
}
# The release.yml run of $TAG: "<id> <status>", empty before it is listed.
release_run() {
  gh run list -w Release --branch "$TAG" -L 1 --json databaseId,status \
    -q '.[0] | "\(.databaseId) \(.status)"' -R jandira-tech/jubarte-redlines 2>/dev/null || true
}
# The conclusion of release.yml's job named $1 on $TAG: empty while it runs
# or before the run is listed.
ci_job_state() {
  local run_id
  read -r run_id _ <<<"$(release_run)"
  [ -n "${run_id:-}" ] || return 0
  gh run view "$run_id" --json jobs \
    -q ".jobs[] | select(.name == \"$1\") | .conclusion" -R jandira-tech/jubarte-redlines </dev/null 2>/dev/null || true
}
# Waits for release.yml's publish job $2 to put $VER on registry $1, which
# the probe "${@:3}" proves (#1642). A job that ended without the registry
# holding the version stops the run with the way out; an hour bounds it.
ci_published() {
  local what=$1 job=$2 state
  shift 2
  for _ in $(seq 1 120); do
    if "$@"; then
      step "$what $VER published by release.yml ($job)"
      return 0
    fi
    state=$(ci_job_state "$job")
    case "$state" in
      success)
        eventually "$@" && { step "$what $VER published by release.yml ($job)"; return 0; }
        die "release.yml's \"$job\" job succeeded but $what does not list $VER after two minutes — check the registry by hand, then rerun this command (every registry that holds $VER is skipped)" ;;
      failure|cancelled|skipped|timed_out|startup_failure)
        die "release.yml's \"$job\" job ended $state and $what does not hold $VER. Read it: gh run view --log-failed (run $(release_run)). A refused token means the registry's trusted publisher does not match: owner jandira-tech, repository jubarte-redlines, workflow release.yml, environment release. Fix it and rerun the job (gh run rerun <id> --failed), or rerun this command with --local-publish to publish from here" ;;
    esac
    sleep 30
  done
  die "$what still lacks $VER after an hour of release.yml — rerun this command later (every registry that holds $VER is skipped)"
}
# A registry answers a fresh upload late: PyPI's project listing (0.11.2:
# missing seconds after `uv publish`, listed a minute later), crates.io's
# index, npm's view. Asks up to 12 times, 10 s apart.
eventually() {
  local i
  for i in $(seq 1 12); do
    "$@" && return 0
    [ "$i" = 12 ] || sleep 10
  done
  return 1
}

# --- the bump kind (docs/VERSIONING.md) and the CHANGELOG section ---------------
# The newest release tag reachable from HEAD, other than $1: a plain vX.Y.Z,
# since a prerelease tag (v0.14.0-rc.1) is no release. Every step measures
# the bump, the CHANGELOG and the API drift from it.
last_release_tag() {
  git describe --tags --abbrev=0 --match 'v[0-9]*' --exclude 'v*-*' \
    --exclude "${1:-v*-*}" 2>/dev/null
}
# patch, minor or major from the previous release's version $1 to $2; fails
# when $2 is not above $1, or when either is no X.Y.Z.
bump_kind() {
  local a b v
  for v in "$1" "$2"; do
    [[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] \
      || { echo "  ! $v is no release version (X.Y.Z)" >&2; return 1; }
  done
  IFS=. read -r -a a <<<"$1"
  IFS=. read -r -a b <<<"$2"
  if [ "${b[0]}" -ne "${a[0]}" ]; then
    [ "${b[0]}" -gt "${a[0]}" ] && echo major
  elif [ "${b[1]}" -ne "${a[1]}" ]; then
    [ "${b[1]}" -gt "${a[1]}" ] && echo minor
  else
    [ "${b[2]}" -gt "${a[2]}" ] && echo patch
  fi
}
# The body of CHANGELOG.md's `## [$VER] - date` section: the lines
# release.yml cuts into the GitHub release notes.
changelog_section() {
  awk -v ver="$VER" '
    index($0, "## [" ver "] - ") == 1 { on = 1; next }
    on && /^## \[/ { exit }
    on { print }
  ' CHANGELOG.md
}
# cargo-semver-checks fails only on lints at deny, and ships some at allow or
# warn (#1744); Cargo.toml's lint table must raise every one of them.
semver_lints_ok() { python3 scripts/semver_lints.py; }
# Pre-1.0 (docs/VERSIONING.md) a patch carries no new feature and no public
# API change; a minor or a major may. Proven before step 1 writes anything
# (#1742): a patch whose section has `### Added` ships features, and
# cargo-semver-checks compares the public API with the previous release's
# tag. $1 is that tag.
bump_gate() {
  local prev=${1#v} kind minor
  kind=$(bump_kind "$prev" "$VER") || die "$VER is not above the previous release $prev"
  if [ "$kind" != patch ]; then
    step "$prev -> $VER is a $kind bump: new features and API changes may ship"
    return 0
  fi
  minor="${prev%%.*}.$(( $(echo "$prev" | cut -d. -f2) + 1 )).0"
  if changelog_section | grep -E '^### Added' >/dev/null; then
    die "$VER is a patch, but its CHANGELOG section has ### Added: new features take a minor bump (docs/VERSIONING.md) — release $minor instead"
  fi
  semver_lints_ok \
    || die "the cargo-semver-checks lint audit failed (above): a lint left below deny, or set below it by Cargo.toml's [package.metadata.cargo-semver-checks.lints], lets a patch pass it — deny each; no lint definitions found means the installed tool's sources are missing (scripts/semver_lints.py)"
  cargo semver-checks check-release -p jubarte-redlines --baseline-rev "v$prev" \
      --release-type patch --default-features \
    || die "cargo-semver-checks finds public API changes a patch cannot carry (above; docs/VERSIONING.md) — release $minor instead, or take the changes out"
  step "$prev -> $VER is a patch: no ### Added, and cargo-semver-checks finds no API change"
}
# The placeholder words left in the CHANGELOG section, with their line
# numbers (whole words: TODOS is no placeholder).
changelog_placeholders() { changelog_section | grep -nwE '(TODO|TBD|FIXME|XXX)'; }
# PRs merged since the release tag $1 that a user meets: a merge commit from
# a fix/, feat/, perf/ or security/ branch, or whose PR title (the merge
# commit's body) has one of those types, or a squashed commit whose subject
# says so. Release, chore, docs, test and CI PRs owe the notes nothing. A
# history git cannot read is a refusal, never an empty set.
user_facing_prs() {
  local log
  log=$(git log --format='%s%x1f%b%x1e' "$1..HEAD") \
    || { echo "  ! git cannot read the history since $1" >&2; return 1; }
  printf '%s' "$log" | awk -v RS='\036' -F'\037' '
    { sub(/^\n+/, "", $1); title = $2; sub(/^\n+/, "", title); sub(/\n.*/, "", title) }
    $1 ~ /^Merge pull request #[0-9]+ from / {
      match($1, /#[0-9]+/); n = substr($1, RSTART + 1, RLENGTH - 1)
      branch = $1; sub(/^Merge pull request #[0-9]+ from [^\/ ]+\//, "", branch)
      if (branch ~ /^(fix|feat|perf|security)[\/-]/ \
          || title ~ /^(fix|feat|perf|security)([(][^)]*[)])?!?: /) print n
      next
    }
    $1 ~ /^(fix|feat|perf|security)([(][^)]*[)])?!?: / && match($1, / [(]#[0-9]+[)]$/) {
      print substr($1, RSTART + 3, RLENGTH - 4)
    }
  ' | sort -un
}
# The user-facing PRs since $1 that the CHANGELOG section cites nowhere: by
# the PR's own number, an issue it closes, or an issue its title names. A
# `<!-- changelog-skip: #N ... -->` line in the section cites one on purpose
# (the release notes render it invisible). Step 2 calls it inside $(…),
# where bash clears errexit: every failure returns on its own.
unlisted_prs() {
  local cited n r hit refs prs why
  prs=$(user_facing_prs "$1") || return 1
  cited=" $(changelog_section | { grep -oE '#[0-9]+' || true; } | tr -d '#' | tr '\n' ' ') "
  for n in $prs; do
    case "$cited" in *" $n "*) continue ;; esac
    # A number that is no pull request (gh pr view fails) is the issue a
    # PR's own commit names ("fix(x): … (#1668)"), and it stays owed: a PR
    # merged inside another's atomic stack merge (#1670 in 0.14.0) has no
    # merge commit, so the issue its commits fix is all that shows it. Any
    # other failure is a refusal: it would hide a PR the notes owe.
    if ! refs=$(gh pr view "$n" -R jandira-tech/jubarte-redlines --json closingIssuesReferences,title \
                  -q '(.closingIssuesReferences[].number), .title' </dev/null 2>/dev/null); then
      why=$(gh pr view "$n" -R jandira-tech/jubarte-redlines --json number </dev/null 2>&1 >/dev/null || true)
      case "$why" in
        *"Could not resolve to a PullRequest"*) refs="" ;;
        *) echo "  ! gh cannot read #$n: $why" >&2; return 1 ;;
      esac
    fi
    hit=0
    for r in $(printf '%s\n' "$refs" | awk '
        /^[0-9]+$/ { print; next }
        { while (match($0, /#[0-9]+/)) { print substr($0, RSTART + 1, RLENGTH - 1); $0 = substr($0, RSTART + RLENGTH) } }'); do
      case "$cited" in *" $r "*) hit=1; break ;; esac
    done
    [ "$hit" = 1 ] || echo "$n"
  done
}
# GitHub's open security findings (#1750), one per line: "<kind> <id>
# <severity> <what>". Repository advisories in triage (a private report) or
# draft, and open Dependabot and code-scanning alerts. An API gh cannot read
# is a refusal, never an empty list.
security_findings() {
  local api=repos/jandira-tech/jubarte-redlines out admin ver_re
  # GitHub lists a repository's unpublished advisories (private reports,
  # drafts) only to its admins and security managers. Any other token gets
  # the published ones, successfully, and an empty list proves nothing
  # (#1761).
  admin=$(gh api "$api" -q .permissions.admin </dev/null) \
    || { echo "  ! gh cannot read the repository's permissions" >&2; return 1; }
  [ "$admin" = true ] \
    || { echo "  ! this gh token is no admin of $api, and GitHub shows it no unpublished advisory (a private report, a draft): an empty list would prove nothing — use an admin's token" >&2; return 1; }
  # A draft is patched when one of its patched versions names this release,
  # bare or after an operator (">= $VER"): the release that is the fix.
  ver_re="(^|[ =<>,~^])$(printf '%s' "$VER" | sed 's/\./\\\\./g')(\$|[ ,])"
  out=$(gh api "$api/security-advisories?per_page=100" --paginate \
          -q '.[] | select(.state == "triage" or .state == "draft") | "advisory-\(.state) \(.ghsa_id) \(.severity // "unrated") \(if any(.vulnerabilities[]?; (.patched_versions // "") | test("'"$ver_re"'")) then "patched" else "unpatched" end)"' </dev/null) \
    || { echo "  ! gh cannot read the repository security advisories" >&2; return 1; }
  [ -z "$out" ] || printf '%s\n' "$out"
  out=$(gh api "$api/dependabot/alerts?state=open&per_page=100" --paginate \
          -q '.[] | "dependabot #\(.number) \(.security_advisory.severity // "unrated") \(.security_advisory.ghsa_id)/\(.dependency.package.name)"' </dev/null) \
    || { echo "  ! gh cannot read the open Dependabot alerts" >&2; return 1; }
  [ -z "$out" ] || printf '%s\n' "$out"
  out=$(gh api "$api/code-scanning/alerts?state=open&per_page=100" --paginate \
          -q '.[] | "code-scanning #\(.number) \(.rule.security_severity_level // "unrated") \(.rule.id)@\(.most_recent_instance.location.path)"' </dev/null) \
    || { echo "  ! gh cannot read the open code-scanning alerts" >&2; return 1; }
  [ -z "$out" ] || printf '%s\n' "$out"
}
# The findings on stdin that block a release: SECURITY.md ships no confirmed
# medium-or-higher finding, and a report still in triage is unconfirmed
# either way until someone reads it. A draft is a confirmed report: it blocks
# unless it is low or this release patches it, so moving a report from
# triage to draft fixes nothing (#1761); an unrated draft is not yet judged.
security_blocking() {
  awk '$1 == "advisory-triage" || ($1 == "advisory-draft" && $3 != "low" && $4 == "unpatched") || ($1 !~ /^advisory-/ && $3 ~ /^(medium|high|critical)$/)'
}
security_gate() {
  local findings blocking
  findings=$(security_findings) || die "cannot read GitHub's security findings (above) — check gh auth status, then rerun"
  [ -z "$findings" ] || { step "open security findings:"; printf '%s\n' "$findings" | sed 's/^/        /'; }
  blocking=$(printf '%s\n' "$findings" | security_blocking)
  [ -z "$blocking" ] \
    || die "these security findings block the release (SECURITY.md): $(printf '%s\n' "$blocking" | paste -sd ';' - | sed 's/;/; /g') — fix them, or dismiss each with its reason on GitHub (a draft this release fixes names $VER as its patched version), then rerun"
  step "security: no advisory in triage, no unpatched draft above low, no open medium-or-higher Dependabot or code-scanning alert"
}
# The GitHub release notes carry the github summary (step 12; step 8 asks
# it of a tag already on origin).
gh_note() {
  gh release view "$TAG" --json body -q .body -R jandira-tech/jubarte-redlines 2>/dev/null \
    | grep -F -e "$GITHUB_SUMMARY" >/dev/null
}
# The annotated tag release.yml reads: subject `jubarte $TAG`, the github
# summary in its body, which release.yml prepends to the release notes.
# 0.13.0's tag held the summary as its subject and an empty body (#1755).
tag_shape_ok() {
  [ "$(git tag -l --format='%(objecttype)' "$TAG")" = tag ] || return 1
  [ "$(git tag -l --format='%(contents:subject)' "$TAG")" = "jubarte $TAG" ] || return 1
  git tag -l --format='%(contents:body)' "$TAG" | grep -F -e "$GITHUB_SUMMARY" >/dev/null
}
# docs/adoption labels each row released, main or pending; a release ships
# everything merged, so a row still labelled main is stale once it is out
# (#1754). Lists those rows; fails when the pages cannot be read.
adoption_main_rows() {
  local rc=0
  [ -d docs/adoption ] || return 1
  grep -rnE "^\|.*\|[[:space:]]*(\`main\`|\*\*main\*\*|main)[[:space:]]*\|[[:space:]]*\$" --include='*.md' docs/adoption || rc=$?
  [ "$rc" -le 1 ]
}
# Each tracked file that can carry a comment and lacks SPDX-License-Identifier
# in its first 4 KiB. `reuse lint` cannot see one, since REUSE.toml's
# `path = "**"` covers every file (the 0.13.0 audit found six by hand,
# #1754). wasm-bindgen writes npm/'s glue and the docs/api snapshots of it;
# REUSE.toml carries tests/fixtures/' provenance.
headerless_files() {
  local f top
  git ls-files -z -- '*.rs' '*.py' '*.sh' '*.ts' '*.tsx' '*.js' '*.mjs' '*.cjs' '*.zsh' '*.awk' \
      '*.toml' '*.yml' '*.yaml' '*.pyi' '*.ini' '*.cfg' \
      ':(exclude)jubarte-wasm/npm/*/jubarte_wasm*' ':(exclude)docs/api/*' ':(exclude)tests/fixtures/*' \
    | while IFS= read -r -d '' f; do
        top=$(head -c 4096 -- "$f" 2>/dev/null) || top=""
        case "$top" in *SPDX-License-Identifier*) ;; *) printf '%s\n' "$f" ;; esac
      done
}
# The canonical checkout (AGENTS.md) is the owner's working tree, never the
# release tree (#1751): true when this tree is it, through any link.
canonical_checkout() {
  local canon=${JUBARTE_CANONICAL_CHECKOUT:-$HOME/T/jubarte-redlines}
  [ -d "$canon" ] && [ "$(cd "$canon" && pwd -P)" = "$(pwd -P)" ]
}
# HEAD's check runs that did not pass, put in front of the releaser (#1751).
# A red run may be billing, quota or an offline runner, so the run goes on.
head_ci_report() {
  local sha runs reds total passed
  sha=$(git rev-parse HEAD)
  if ! runs=$(gh api "repos/jandira-tech/jubarte-redlines/commits/$sha/check-runs?per_page=100" --paginate \
                -q '.check_runs[] | if .conclusion == "success" then "ok" elif .conclusion == "skipped" or .conclusion == "neutral" then "skip" else "red \(.name) \(.conclusion // .status)" end' </dev/null); then
    echo "  CHECK NOW  gh cannot read HEAD's check runs — read them: gh run list -R jandira-tech/jubarte-redlines --commit $sha"
    return 0
  fi
  reds=$(printf '%s\n' "$runs" | sed -n 's/^red //p')
  total=$(printf '%s' "$runs" | grep -c '' || true)
  passed=$(printf '%s\n' "$runs" | grep -c '^ok$' || true)
  if [ -n "$reds" ]; then
    echo "  CHECK NOW  HEAD's check runs that did not pass — read each log; fix only genuine test or coverage failures:"
    printf '%s\n' "$reds" | sed 's/^/             /'
  elif [ "$total" = 0 ]; then
    # no run at all is no evidence, not green (#1763)
    echo "  CHECK NOW  HEAD has no check run: its workflows never ran — read gh run list -R jandira-tech/jubarte-redlines --commit $sha"
  elif [ "$passed" = 0 ]; then
    echo "  CHECK NOW  none of HEAD's $total check runs succeeded (each was skipped or neutral), so nothing passed — read gh run list -R jandira-tech/jubarte-redlines --commit $sha"
  else
    step "every check run of HEAD passed ($passed passed, $((total - passed)) skipped or neutral)"
  fi
}
# Queued CI waits behind nothing once release.yml needs the runners: every
# run still in flight but release.yml's own is cancelled right before the
# tag goes up (the push to main has just started CI and Docs again).
cancel_queued_ci() {
  local id name n=0
  while read -r id name; do
    [ -n "$id" ] && [ "$name" != Release ] || continue
    gh run cancel "$id" -R jandira-tech/jubarte-redlines >/dev/null 2>&1 && n=$((n + 1))
  done < <(gh run list -L 100 --json databaseId,name,status \
             -q '.[] | select(.status != "completed") | "\(.databaseId) \(.name)"' -R jandira-tech/jubarte-redlines </dev/null 2>/dev/null)
  step "cancelled $n CI run(s) in flight so release.yml gets the runners"
}
# crates.io takes at most five keywords (ASCII, a letter first, then letters,
# digits, _, - or +, at most 20 characters) and five categories; one more
# fails the publish itself, after the point of no return.
crate_keywords_ok() {
  python3 - <<'PY'
import re
import sys
import tomllib

with open("Cargo.toml", "rb") as f:
    package = tomllib.load(f)["package"]
keywords, categories = package.get("keywords", []), package.get("categories", [])
problems = [f"{len(keywords)} keywords (crates.io takes 5)"] if len(keywords) > 5 else []
if len(categories) > 5:
    problems.append(f"{len(categories)} categories (crates.io takes 5)")
problems += [f"keyword crates.io refuses: {k}" for k in keywords
             if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_+-]{0,19}", k)]
for problem in problems:
    print(f"  ! {problem}", file=sys.stderr)
sys.exit(1 if problems else 0)
PY
}

# =============================================================================
say "0. Preflight"
# =============================================================================

! canonical_checkout \
  || die "this is the canonical checkout ($(pwd -P)), the owner's working tree — release from a clean worktree of main: git worktree add <dir> main (AGENTS.md)"
[ "$(git branch --show-current)" = "main" ] \
  || die "release from main only (current: $(git branch --show-current))"

git fetch --quiet origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] \
  || die "main is not in sync with origin/main — push/pull first"

[ -z "$(git status --porcelain)" ] \
  || die "working tree is dirty — commit or stash first:
$(git status --porcelain | sed 's/^/       /')"
head_ci_report

for t in cargo cargo-deny cargo-semver-checks gitleaks bun npm gh wasm-pack node uv uvx; do
  command -v "$t" >/dev/null || die "missing tool: $t"
done

gh auth status >/dev/null 2>&1 || die "gh not authenticated — run \`gh auth login\`"
# release.yml publishes the registries through OIDC; tokens are asked for
# only when this machine publishes instead.
if [ "$LOCAL_PUBLISH" = 1 ]; then
  [ -s "$HOME/.cargo/credentials.toml" ] || [ -s "$HOME/.cargo/credentials" ] \
    || [ -n "${CARGO_REGISTRY_TOKEN:-}" ] \
    || die "no crates.io credentials — run \`cargo login\`"
  npm whoami >/dev/null 2>&1 || die "not logged in to npm — run \`npm login\`"
  [ -n "${UV_PUBLISH_TOKEN:-}" ] \
    || die "UV_PUBLISH_TOKEN not set — a pypi-… API token for \`uv publish\`"
fi
step "tools + credentials OK"
# Step 13 is the app's part, run from the app repository's checkout (this
# repository does not track it): prove it can run before anything is
# published, not after (0.11.2 found out at step 13).
APP_REPO=${JUBARTE_APP_DIR:-jubarte-app}
[ -x "$APP_REPO/scripts/release-engine.sh" ] \
  || die "no app checkout with scripts/release-engine.sh at $APP_REPO — git clone https://github.com/arthrod/jubarte-app jubarte-app (or export JUBARTE_APP_DIR)"
# The app's gh calls resolve the app repository from its own checkout; a
# GH_REPO in this shell would send them elsewhere (#1747).
app_release() { env -u GH_REPO "$APP_REPO/scripts/release-engine.sh" "$VER" --engine-dir "$PWD" "$@"; }
app_release --preflight

# The release.yml review — the wheel and binary matrices against the README
# and steps 11's lists, SHA-pinned actions, the publish jobs in the release
# environment with the only id-token grants — is scripts/test_release_workflow.py,
# on the scripts' floor. What it cannot read lives in GitHub's settings.
uv run -q --no-project --python 3.12 --with pyyaml python scripts/test_release_workflow.py >/dev/null 2>&1 \
  || die "release.yml fails its review — run: uv run --no-project --python 3.12 --with pyyaml python scripts/test_release_workflow.py"
gh api repos/jandira-tech/jubarte-redlines/environments/release >/dev/null 2>&1 \
  || die "no \`release\` environment on GitHub — the registries trust release.yml in that environment (#1642)"
if compgen -G 'keys/*.pub' >/dev/null; then
  gh secret list -R jandira-tech/jubarte-redlines --json name -q '.[].name' </dev/null 2>/dev/null \
    | grep -x ZIPSIGN_PRIVATE_KEY >/dev/null \
    || die "keys/ holds a release public key but the ZIPSIGN_PRIVATE_KEY secret is not set — signed-era binaries would refuse this release"
fi
step "release.yml reviewed; the release environment and the signing secret are in place"

security_gate

# The bump kind, before step 1 writes it. A resumed run (the tag exists)
# still gates against the release before this one.
PREV_RELEASE=$(last_release_tag "$TAG") \
  || die "no earlier release tag (vX.Y.Z) to gate the bump against — git fetch --tags"
bump_gate "$PREV_RELEASE"

# =============================================================================
say "1. Version sync → $VER"
# =============================================================================

CUR="$(crate_ver)"
if [ "$CUR" = "$VER" ]; then
  step "Cargo.toml already at $VER (resume)"
elif command -v bun >/dev/null; then
  JUBARTE_RELEASE_SH=1 bun scripts/bump-version.mjs "$VER"
else
  JUBARTE_RELEASE_SH=1 node scripts/bump-version.mjs "$VER"
fi

# bump-version.mjs also moved the README's Socket badge
# (badge.socket.dev/cargo/package/jubarte-redlines/<version>) to $VER.
# Manifests bump-version.mjs does not own.
sed -i.bak "s/^version = \"$CUR\"$/version = \"$VER\"/" jubarte-python/Cargo.toml \
  && rm jubarte-python/Cargo.toml.bak
# The two publish = false crates move with the engine too: wasm-pack writes
# the crate's own version into pkg/package.json, and a 0.10.0 stamped there
# already passed itself off as the engine's version in tooling. They may
# sit on an older version than $CUR (nothing ever bumped them), so the sed
# takes any x.y.z rather than $CUR; a resumed run rewrites the same $VER.
for f in jubarte-wasm/Cargo.toml jubarte-rust-inproc/Cargo.toml; do
  sed -i.bak -E "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"$/version = \"$VER\"/" "$f" \
    && rm "$f.bak"
done
step "jubarte-python/Cargo.toml, jubarte-wasm/Cargo.toml, jubarte-rust-inproc/Cargo.toml → $VER"
(cd jubarte-wasm/npm && npm pkg set "version=$VER" >/dev/null)
# `npx jubarte-redlines` runs on the jubarte-wasm of its own release.
(cd jubarte-wasm/cli && npm pkg set "version=$VER" "dependencies.jubarte-wasm=^$VER" >/dev/null)
step "jubarte-wasm/{npm,cli}/package.json → $VER"

# The desktop app is its own repository (arthrod/jubarte-app), checked out
# untracked at jubarte-app/ or named by JUBARTE_APP_DIR: its version files
# move at step 13, on its own release branch.

# A crash mid-step-1 leaves the manifests half-bumped, and a resumed run
# cannot repair it: CUR is then already $VER, every pattern above keys on a
# version the unbumped files no longer carry, and sed matches nothing while
# exiting 0 silently (wheels would ship as the old version). Prove every
# file step 1 touched really is on $VER, by name.
half_bumped() {
  die "$1 is not on $VER after the version sync — a half-bumped tree from an interrupted step 1; fix $1 by hand, commit or reset, and rerun"
}
for f in Cargo.toml jubarte-python/Cargo.toml jubarte-wasm/Cargo.toml \
         jubarte-rust-inproc/Cargo.toml; do
  grep -q "^version = \"$VER\"$" "$f" || half_bumped "$f"
done
for f in jubarte-wasm/npm/package.json jubarte-wasm/cli/package.json \
         gemini-extension.json; do
  grep -q "\"version\": \"$VER\"" "$f" || half_bumped "$f"
done
# The README pins a version only where it carries the Socket badge (the
# 0.11.0 rewrite dropped it): a pin that is there must be this release's,
# and a README without one has nothing to move.
stale_readme_pin() {
  grep -oE "badge\.socket\.dev/cargo/package/jubarte-redlines/[0-9]+\.[0-9]+\.[0-9]+" README.md \
    | grep -vxF "badge.socket.dev/cargo/package/jubarte-redlines/$VER" >/dev/null
}
stale_readme_pin && half_bumped "README.md (Socket badge)"
step "every file step 1 touched is on $VER"

# Lockfiles: re-resolve only jubarte-redlines (the root package, or the path
# dependency every other workspace pins), offline, so registry deps stay put.
# (A metadata-only pass resolves nothing: v0.10.1 was tagged with three locks
# still on 0.10.0.) Each `cargo update --offline -q -p jubarte-redlines`
# re-records, in that directory's own Cargo.lock, BOTH the new
# jubarte-redlines version AND the directory's own member version bumped
# above (cargo rewrites the whole lock once the manifests moved):
#   .                     → Cargo.lock
#   jubarte-python        → jubarte-python/Cargo.lock
#   jubarte-wasm          → jubarte-wasm/Cargo.lock
#   jubarte-rust-inproc   → jubarte-rust-inproc/Cargo.lock
# (The app's own lock moves in the app repository.)
for d in . jubarte-python jubarte-wasm jubarte-rust-inproc; do
  (cd "$d" && cargo update --offline -q -p jubarte-redlines)
done
step "Cargo.lock ×4 refreshed"

# One README per published library, cut from README.md and the library's
# README.fragment.md, links pinned to $TAG. It refuses when a public Python
# or WASM name is missing from its README: document it in the fragment.
python3 scripts/library_readmes.py --version "$VER" >/dev/null \
  || die "library READMEs: a public API name is undocumented (see above); add it to the fragment"
step "README.crates.md, jubarte-python/README.md, jubarte-wasm/{npm,cli}/README.md cut for v$VER"

# =============================================================================
say "2. Changelog check"
# =============================================================================

grep -q "^## \[$VER\] - [0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}" CHANGELOG.md \
  || die "CHANGELOG.md has no dated \`## [$VER] - YYYY-MM-DD\` section — write it first"
grep -q "^\[$VER\]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v$VER" CHANGELOG.md \
  || die "CHANGELOG.md is missing the \`[$VER]: …/tag/$TAG\` release-link footer"
step "$VER section + link footer present"
# release.yml cuts the section verbatim into the GitHub release notes.
if changelog_placeholders >&2; then
  die "CHANGELOG.md's $VER section holds placeholder text (above) — it ships as the release notes"
fi
missing_prs=$(unlisted_prs "$PREV_RELEASE") \
  || die "cannot tell which PRs merged since $PREV_RELEASE the notes owe (above)"
[ -z "$missing_prs" ] \
  || die "CHANGELOG.md's $VER section cites none of these user-facing PRs merged since $PREV_RELEASE, or issues their fix/feat commits name: $(printf '%s\n' "$missing_prs" | paste -sd ' ' -) — cite each by its number or an issue it closes, or skip it with a \`<!-- changelog-skip: #N -->\` line"
step "every user-facing PR merged since $PREV_RELEASE is cited"
check_now 2

# =============================================================================
say "3. release_info — the benchmark evidence of $VER"
# =============================================================================
# The six files release_info/README.md names must sit in release_info/ for
# this version before anything is written: the two 600-item sample CSVs
# (every file a row names carries its sha256 beside it), the two results
# JSONs scored on exactly those samples, and the website/app change lists.
# The bench writes them — neurotic_docx_bench's jubarte_release_info flow,
# against a release candidate built from this checkout (--binary), since
# these results are required before the release exists; the candidate
# names the commit that will become the release commit's parent
# (release_info/README.md, "Which binary the evidence names"). The checker
# prints the aggregates; the release commit (step 8) carries the folder.
#
# The sample paths name files of the BENCH repository. With a bench
# checkout at hand, --bench-root makes the checker hash every non-empty
# path cell and compare it with the CSV's sha256 — the real proof the
# evidence names real, unmodified files. Without one, the sha256 columns
# are format-checked only (present, paired, 64 lowercase hex), and that is
# said out loud below.
BENCH_ROOT=""
if [ -n "${NEUROTIC_DOCX_BENCH:-}" ] && [ -d "$NEUROTIC_DOCX_BENCH" ]; then
  BENCH_ROOT="$NEUROTIC_DOCX_BENCH"
elif [ -d ../neurotic_docx_bench ]; then
  BENCH_ROOT="$(cd ../neurotic_docx_bench && pwd)"
fi
if [ -n "$BENCH_ROOT" ]; then
  python3 scripts/check_release_info.py "$VER" --bench-root "$BENCH_ROOT" \
    || die "release_info/ does not carry $VER's six files, or they do not match the bench's files — run the bench flow that writes them (release_info/README.md), commit them, then rerun"
  step "six files verified: samples + results + website/app data (sha256 columns checked against $BENCH_ROOT)"
else
  echo "  ! no bench checkout found (NEUROTIC_DOCX_BENCH, ../neurotic_docx_bench) — the samples' sha256 columns are format-checked only, not verified against real files" >&2
  python3 scripts/check_release_info.py "$VER" \
    || die "release_info/ does not carry $VER's six files — run the bench flow that writes them (release_info/README.md), commit them, then rerun"
  step "six files verified: samples + results + website/app data (sha256 columns format-checked only)"
fi
# The candidate the bench scored was built from the commit both results
# name; its CLI engine inputs must be this tree's (#1752). HEAD is the
# committed tree: step 1's bump is not committed yet, and a resumed run's
# release commit changes only what the check sets aside.
python3 scripts/check_evidence_engine.py "$VER" \
  || die "the evidence was not scored on this tree's engine (above) — rebuild the candidate from HEAD and rerun the bench flow, or take the engine change out"
# The measured sentence ships in the CHANGELOG (from --changelog-summary, step
# 4), the GitHub notes, the app's changelog and About window and jubarte.pro:
# each figure must be the evidence's own, at two decimals (#1753).
python3 scripts/check_release_info.py "$VER" --summary "$CHANGELOG_SUMMARY" --changelog "$APP_REPO/CHANGELOG.md" >/dev/null \
  || die "the measured sentence of --changelog-summary or $APP_REPO/CHANGELOG.md does not state the evidence's aggregates (above)"
step "the measured sentence states the evidence's aggregates (the changelog summary and the app's changelog)"
check_now 3

# =============================================================================
say "4. Summaries → each registry's channel"
# =============================================================================

cat <<EOF
    changelog    > **Summary.** … under ## [$VER]
    crates.io    [package.metadata.release-notes] "$VER" in Cargo.toml
    npm          releaseNotes."$VER" in jubarte-wasm/npm/package.json
    PyPI         # release-notes comment in pyproject.toml + metadata table
    GitHub       tag annotation → release notes body
    docs         > **Docs.** … under the changelog summary + release commit body
EOF

# What the docs statement is about: README/docs files touched since the last
# release tag (the version sync above already moved the README badge).
PREV_TAG=$(last_release_tag || true)
if [ -n "$PREV_TAG" ]; then
  step "docs changed since $PREV_TAG:"
  { git diff --name-only "$PREV_TAG" -- '*.md' 'docs/' 'skills/'; \
    git ls-files --others --exclude-standard -- '*.md' 'docs/' 'skills/'; } \
    | sort -u | sed 's/^/        /'
fi
step "how they were updated: $DOCS_UPDATED"
main_rows=$(adoption_main_rows) || die "cannot read docs/adoption's status labels"
if [ -n "$main_rows" ]; then
  printf '%s\n' "$main_rows" | sed 's/^/        /' >&2
  die "docs/adoption still labels these rows main, and $VER ships them — flip each to released"
fi
step "docs/adoption: no row still labelled main"
check_now 4

# Escape a value for a TOML basic string.
toml_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

# Insert or replace `"$VER" = "…"` inside [package.metadata.release-notes],
# creating the table right after the [package] block when absent. cargo keeps
# package.metadata verbatim in the packaged manifest, so the note ships inside
# the .crate (crates.io) or vendored Cargo.toml (PyPI sdist).
toml_release_note() {
  local f=$1 esc
  esc=$(toml_escape "$2")
  if grep -qF '[package.metadata.release-notes]' "$f"; then
    VER_ESC="$VER" TXT_ESC="$esc" awk '
      BEGIN {
        key = "\"" ENVIRON["VER_ESC"] "\""
        line = key " = \"" ENVIRON["TXT_ESC"] "\""
      }
      $0 == "[package.metadata.release-notes]" { intable = 1; print; next }
      intable && index($0, "[") == 1 {
        if (!wrote) { print line; wrote = 1 }
        intable = 0; print; next
      }
      intable && index($0, key " =") == 1 {
        if (!wrote) { print line; wrote = 1 }
        next
      }
      { print }
      END { if (intable && !wrote) print line }
    ' "$f" > "$f.tmp"
  else
    VER_ESC="$VER" TXT_ESC="$esc" awk '
      BEGIN { line = "\"" ENVIRON["VER_ESC"] "\" = \"" ENVIRON["TXT_ESC"] "\"" }
      $0 == "[package]" { inpkg = 1; print; next }
      inpkg && index($0, "[") == 1 {
        print ""; print "[package.metadata.release-notes]"; print line; print ""
        inpkg = 0
      }
      { print }
    ' "$f" > "$f.tmp"
  fi
  mv "$f.tmp" "$f"
}

# changelog — `> **Summary.** …` then `> **Docs.** …` directly under the
# version heading; an earlier quote block for this run is replaced, all other
# content untouched.
VER="$VER" TXT="$CHANGELOG_SUMMARY" DOCS="$DOCS_UPDATED" awk '
  BEGIN { st = 0; s = ENVIRON["TXT"]; d = ENVIRON["DOCS"] }
  st == 0 && index($0, "## [" ENVIRON["VER"] "] - ") == 1 { print; st = 1; next }
  st == 1 || st == 2 {
    if ($0 ~ /^[[:space:]]*$/) next
    if (st == 1 && $0 ~ /^> \*\*Summary\.\*\*/) { st = 2; next }
    if (st == 2 && $0 ~ /^>/) next
    printf "\n> **Summary.** %s\n>\n> **Docs.** %s\n\n", s, d; print; st = 9; next
  }
  { print }
' CHANGELOG.md > .changelog.tmp && mv .changelog.tmp CHANGELOG.md
grep -qF "> **Summary.** $CHANGELOG_SUMMARY" CHANGELOG.md \
  || die "changelog summary failed to land"
grep -qF "> **Docs.** $DOCS_UPDATED" CHANGELOG.md \
  || die "docs statement failed to land in the changelog"

# crates.io — no per-release notes channel and cargo strips comments from the
# packaged manifest; the release-notes metadata table is what survives.
toml_release_note Cargo.toml "$CRATES_SUMMARY"

# npm — unknown package.json fields are published into the packument.
VER="$VER" TXT="$NPM_SUMMARY" node -e '
  const fs = require("fs");
  const f = "jubarte-wasm/npm/package.json";
  const j = JSON.parse(fs.readFileSync(f, "utf8"));
  (j.releaseNotes ||= {})[process.env.VER] = process.env.TXT;
  fs.writeFileSync(f, JSON.stringify(j, null, 2) + "\n");
'

# PyPI — no per-release channel; a comment in pyproject.toml ships verbatim in
# the sdist, and the Cargo.toml metadata table rides along in the vendored
# manifest.
if grep -q "^# release-notes v$VER" jubarte-python/pyproject.toml; then
  VER="$VER" TXT="$PYPI_SUMMARY" awk '
    index($0, "# release-notes v" ENVIRON["VER"]) == 1 {
      print "# release-notes v" ENVIRON["VER"] " — " ENVIRON["TXT"]; next
    }
    { print }
  ' jubarte-python/pyproject.toml > .rel.tmp \
    && mv .rel.tmp jubarte-python/pyproject.toml
else
  VER="$VER" TXT="$PYPI_SUMMARY" awk '
    !done && /^description = "/ {
      print
      print "# release-notes v" ENVIRON["VER"] " — " ENVIRON["TXT"]
      done = 1; next
    }
    { print }
  ' jubarte-python/pyproject.toml > .rel.tmp \
    && mv .rel.tmp jubarte-python/pyproject.toml
fi
toml_release_note jubarte-python/Cargo.toml "$PYPI_SUMMARY"

# GitHub — the summary rides in the annotated tag body (see step 8);
# release.yml prepends %(contents:body) to the release notes.
step "all five summaries + docs statement staged"

# =============================================================================
bash scripts/security-check.sh
headerless=$(headerless_files) || die "cannot list the tracked files (git ls-files)"
if [ -n "$headerless" ]; then
  printf '%s\n' "$headerless" | sed 's/^/        /' >&2
  die "these files carry no SPDX-License-Identifier in their first 4 KiB, which reuse lint cannot see (REUSE.toml's ** covers them) — add the header (AGENTS.md, Licensing and provenance)"
fi
if [ "$SKIP_GATES" = 0 ]; then
  say "5. Gates (sequential cargo, per AGENTS.md)"
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-features
  # The docs.rs build: a dead intra-doc link renders as bare brackets there.
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
  # The script tests run on the scripts' floor, Python 3.12 (ruff.toml
  # targets py312), not on whatever python3 this machine has.
  py312=(uv run -q --no-project --python 3.12 python)
  "${py312[@]}" scripts/test_convert_sweep.py
  "${py312[@]}" scripts/test_release_sh.py
  "${py312[@]}" scripts/test_semver_lints.py
  "${py312[@]}" scripts/test_check_engine_commit.py
  "${py312[@]}" scripts/test_check_release_info.py
  "${py312[@]}" scripts/test_check_evidence_engine.py
  "${py312[@]}" scripts/test_identity.py
  "${py312[@]}" scripts/test_lane_validity.py
  "${py312[@]}" scripts/test_redline_sweep.py
  # the parity tool (tools/parity) keeps its tests in pytest form
  uv run -q --no-project --python 3.12 --with pytest python -m pytest -q -p no:cacheprovider tools/parity >/dev/null
  "${py312[@]}" scripts/test_library_readmes.py
  "${py312[@]}" scripts/test_api_snapshot.py
  # Python bindings: build the extension from this checkout and run pytest.
  # jubarte-python/uv.lock is tracked; whatever uv rewrites in it ships in
  # the release commit (step 8), so the tree stays clean for a resumed run.
  (cd jubarte-python \
    && uv run --with maturin maturin develop --release >/dev/null \
    && uv run --with pytest pytest -q)
  uv tool run --from 'reuse[charset-normalizer]' reuse lint >/dev/null
  step "fmt / clippy / tests / docs / sweep-units / pytest / REUSE all green"
else
  say "5. Gates — SKIPPED (--skip-gates)"
fi
# =============================================================================

# =============================================================================
say "6. API docs — drift assessment"
# =============================================================================

# The API sign-off is automatic (#1742): step 0 proved the bump kind with
# cargo-semver-checks, and the drift below is logged. --open-docs opens the
# rendered docs for a releaser who wants to read them too.
if [ "$SKIP_GATES" = 0 ]; then
  # Warnings denied here too: step 5 builds the public docs only, and the
  # private items carried 12 dead links and stray HTML tags in 0.11.2.
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items
  if [ "$OPEN_DOCS" = 1 ]; then
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items --open
  fi
else
  step "doc review skipped (--skip-gates — assessed on the first run)"
fi

# Machine-friendly copy of the API (gzipped rustdoc JSON + a flat, sorted
# listing) kept under docs/api/ so the next release can diff the surface.
python3 scripts/api_snapshot.py "$VER"

# Refresh the generated CLI/API reference blocks in docs/rust.md,
# docs/python.md and docs/javascript.md so they quote this release's runners.
scripts/gen_docs.sh

# The drift is read off the two rustdoc JSON snapshots, not the two listings:
# both sides go through this checkout's filter, so a listing whose format
# changed between releases (v0.11.2's has three columns and every blanket
# impl) is never the drift.
PREV_API=""
if [ -n "$PREV_TAG" ] \
   && [ -f "docs/api/jubarte-$PREV_TAG.json.gz" ] \
   && [ "$PREV_TAG" != "$TAG" ]; then
  PREV_API="docs/api/jubarte-$PREV_TAG.json.gz"
fi
if [ "$SKIP_GATES" = 0 ]; then
  if [ -n "$PREV_API" ]; then
    step "API drift since $PREV_TAG — review before confirming the push:"
    python3 scripts/api_snapshot.py --drift "$PREV_TAG" "v$VER" | sed 's/^/        /'
  elif [ "$PREV_TAG" = "$TAG" ]; then
    step "snapshot already exists for $TAG (resumed run)"
  elif [ -n "$PREV_TAG" ]; then
    # a previous release that skipped step 6 (0.13.0) left no baseline
    step "no snapshot of $PREV_TAG in docs/api — no drift is logged; backfill it for the next release (git worktree add <dir> $PREV_TAG; (cd <dir> && python3 scripts/api_snapshot.py ${PREV_TAG#v}))"
  else
    step "no previous release — docs/api/jubarte-v$VER.api.txt is the baseline"
  fi
fi
check_now 6

# =============================================================================
say "7. Publish dry-runs"
crate_keywords_ok || die "Cargo.toml's keywords or categories break crates.io's limits (above)"
# The bump and summaries are staged but not committed until step 8, so the
# dry run packages the dirty tree; the real publish (step 9) stays clean.
# A resumed run skips the dry run of a registry that already holds $VER:
# npm refuses even a dry run over a published version.
if crates_has; then
  step "jubarte-redlines $VER already on crates.io — dry run skipped"
else
  packaged cargo publish --dry-run --locked --allow-dirty
fi
if npm_has; then
  step "jubarte-wasm $VER already on npm — dry run skipped"
else
  (cd jubarte-wasm/npm && npm publish --dry-run >/dev/null)
fi
if npm_cli_has; then
  step "jubarte-redlines $VER already on npm — dry run skipped"
else
  (cd jubarte-wasm/cli && npm publish --dry-run >/dev/null)
fi
# Wipe the folder first: a reused checkout kept the previous release's
# sdists and the `head -1` below picked the lexicographically oldest (a
# leftover 0.11.1 sorts before 0.11.2), killing the grep on a healthy tree.
rm -rf target/release-check
uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out target/release-check >/dev/null
# The pypi summary must survive into the sdist or we stop here.
sdist=$(find target/release-check -maxdepth 1 -name '*.tar.gz' 2>/dev/null | head -1)
[ -n "$sdist" ] || die "maturin produced no sdist"
member=$(tar -tzf "$sdist" | grep '/pyproject.toml$' | head -1)
[ -n "$member" ] || die "sdist has no pyproject.toml"
tar -xzOf "$sdist" "$member" | grep -F "# release-notes v$VER" >/dev/null \
  || die "pypi summary comment did not make it into the sdist"
step "cargo / npm / maturin dry-runs OK — sdist carries the pypi comment"

# Each registry ships the README cut for this release, gates skipped or not.
STAMP="for v$VER (README.md sha256"
python3 scripts/library_readmes.py --check --version "$VER" \
  || die "library READMEs are stale: run scripts/library_readmes.py --version $VER"
cargo package --list --allow-dirty 2>/dev/null | grep -qx README.crates.md \
  || die "the crate does not ship README.crates.md (Cargo.toml include)"
pkginfo=$(tar -tzf "$sdist" | grep '/PKG-INFO$' | head -1)
tar -xzOf "$sdist" "$pkginfo" | grep -qF "$STAMP" \
  || die "the sdist's PyPI description is not the v$VER README"
for d in jubarte-wasm/npm jubarte-wasm/cli; do
  grep -qF "$STAMP" "$d/README.md" || die "$d/README.md is not cut for v$VER"
done
step "crates.io, PyPI and npm READMEs are the ones cut for v$VER"

if [ "$DRY_RUN" = 1 ]; then
  say "DRY RUN complete"
  cat <<EOF
  The bump is applied but UNCOMMITTED (revert: git checkout -p).
  Nothing was committed, tagged, pushed, or published.
  Re-run without --dry-run for the real release.
EOF
  exit 0
fi

# =============================================================================
say "8. Release commit → wasm artifacts → annotated tag"
# =============================================================================

# A resumed run finds the release commit under the wasm-artifact commit. Once
# the tag exists everything below it was built, and npm may already ship that
# build: rebuilding would stamp a later ENGINE_COMMIT than the one published
# (the v0.10.1 resume did, on a tree the follow-up commits had moved on).
if git rev-parse -q --verify "refs/tags/$TAG^{commit}" >/dev/null; then
  step "tag $TAG exists — release commit $(git rev-parse --short "$TAG^{commit}") and its npm build are kept (resume)"
else
  if ! git log -3 --format=%s | grep -x "chore(release): v$VER" >/dev/null; then
    # docs/{rust,python,javascript}.md are regenerated by scripts/gen_docs.sh
    # in step 6; they ship in this commit beside docs/api, or the post-push
    # docs CI fails and the dirty tree blocks a resumed release.
    git add Cargo.toml Cargo.lock CHANGELOG.md README.md docs/VERSIONING.md \
      README.crates.md jubarte-wasm/npm/README.md jubarte-wasm/cli/README.md \
      jubarte-python/README.md \
      jubarte-python/Cargo.toml jubarte-python/Cargo.lock \
      jubarte-python/pyproject.toml jubarte-python/uv.lock \
      jubarte-wasm/Cargo.toml \
      jubarte-wasm/Cargo.lock jubarte-wasm/npm/package.json jubarte-wasm/cli/package.json \
      jubarte-rust-inproc/Cargo.toml jubarte-rust-inproc/Cargo.lock \
      gemini-extension.json \
      release_info \
      docs/api \
      docs/rust.md docs/python.md docs/javascript.md
    git commit -m "chore(release): v$VER" -m "Docs: $DOCS_UPDATED"
  fi
  step "release commit $(git rev-parse --short HEAD)"

  if git log -2 --format=%s | grep -x "build(wasm): regenerate npm artifacts for v$VER" >/dev/null; then
    step "npm artifacts for v$VER already committed (resume) — not rebuilt"
  else
    # Clean tree now, so build-npm.sh stamps ENGINE_COMMIT.txt with the release
    # commit — the commit the published artifacts can be rebuilt from.
    jubarte-wasm/build-npm.sh
    node jubarte-wasm/npm-smoke.mjs
    # Snapshot the shipped wasm typings beside the rust API dump, after the
    # rebuild so docs/api/ records what npm actually publishes.
    for t in node node-slim web web-slim; do
      cp "jubarte-wasm/npm/$t/jubarte_wasm.d.ts" "docs/api/jubarte-wasm-$t-v$VER.d.ts"
    done
    # docs/javascript.md quotes these typings; step 6 read the previous
    # release's (v0.11.0 shipped a reference missing 13 new functions).
    python3 scripts/gen_wasm_api.py \
      --dts jubarte-wasm/npm/node/jubarte_wasm.d.ts \
      --file docs/javascript.md --marker wasm-api
    # Its cli-npm block is the npm CLI's --help run on this build; step 6 ran
    # it on the committed artifacts of an older engine (the 0.12.0 dry run
    # rewrote the block back to help text the release had changed). The
    # cli's jubarte-wasm dependency is linked to npm/ as gen_docs.sh links it.
    mkdir -p jubarte-wasm/cli/node_modules
    rm -rf jubarte-wasm/cli/node_modules/jubarte-wasm
    ln -sfn ../../npm jubarte-wasm/cli/node_modules/jubarte-wasm
    python3 scripts/gen_cli_docs.py \
      --runner node "$PWD/jubarte-wasm/cli/bin/jubarte-redlines.mjs" \
      --display jubarte-redlines --file docs/javascript.md --marker cli-npm
    # The build re-resolves jubarte-wasm/Cargo.lock too; it ships in this commit.
    if [ -n "$(git status --porcelain -- jubarte-wasm/npm jubarte-wasm/Cargo.lock docs/api docs/javascript.md)" ]; then
      git add jubarte-wasm/npm jubarte-wasm/Cargo.lock docs/api docs/javascript.md
      git commit -m "build(wasm): regenerate npm artifacts for v$VER"
    fi
  fi
  step "npm artifacts rebuilt + smoke-tested (engine $(cut -c1-7 jubarte-wasm/npm/ENGINE_COMMIT.txt))"
fi

# The github summary is the tag annotation body; release.yml prepends it to
# the release notes (tag_shape_ok). A local tag of another shape is
# re-created. A tag on origin is the one release.yml read: it must be this
# one, and when its shape is wrong only notes put right by hand let the run
# go on (#1755).
remote_tag=$(git ls-remote --tags origin "refs/tags/$TAG" | cut -f1) \
  || die "cannot read origin's tags (git ls-remote)"
if [ -n "$remote_tag" ]; then
  [ "$remote_tag" = "$(git rev-parse -q --verify "refs/tags/$TAG" || true)" ] \
    || die "origin's $TAG ($remote_tag) is not the local tag — git fetch origin tag $TAG --force, then rerun"
  if tag_shape_ok; then
    step "tag $TAG on origin carries subject 'jubarte $TAG' and the github summary"
  else
    gh_note || {
      # notes can be edited only on a release that exists (#1766)
      gh release view "$TAG" -R jandira-tech/jubarte-redlines >/dev/null 2>&1 \
        || die "$TAG on origin is not subject 'jubarte $TAG' with the github summary in its body, and no GitHub release exists for it — read its release.yml run (gh run list -R jandira-tech/jubarte-redlines --workflow release.yml --branch $TAG): if it published nothing, replace the tag (git push origin :refs/tags/$TAG && git tag -d $TAG) and rerun, which writes it in shape; if it published, let it finish and put the summary into the notes it makes"
      die "$TAG on origin is not subject 'jubarte $TAG' with the github summary in its body, so release.yml's notes lack the summary — put it into the notes (gh release edit $TAG -R jandira-tech/jubarte-redlines --notes-file <file>), then rerun"
    }
    echo "  ! $TAG on origin lacks the summary in its body; its GitHub notes carry it" >&2
  fi
else
  if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null && ! tag_shape_ok; then
    git tag -d "$TAG" >/dev/null
    step "local tag $TAG was not subject 'jubarte $TAG' with the github summary in its body — re-creating it"
  fi
  git rev-parse -q --verify "refs/tags/$TAG" >/dev/null \
    || git tag -a "$TAG" -m "jubarte $TAG" -m "$GITHUB_SUMMARY"
  tag_shape_ok || die "tag $TAG was not written as release.yml reads it (subject 'jubarte $TAG', the github summary in its body)"
  step "tag $TAG: subject 'jubarte $TAG', the github summary in its body"
fi
# A resumed run keeps the npm build it finds, so the stamp is checked on the
# tag itself: npm ships only artifacts the tagged engine produces (#1088).
scripts/check_engine_commit.sh "$TAG^{commit}" || die "scripts/check_engine_commit.sh refused $TAG (its reason is above)"
check_now 8

# =============================================================================
say "POINT OF NO RETURN"
cat <<EOF
  About to push main + $TAG and publish:
    crates.io  jubarte-redlines $VER
    npm        jubarte-wasm $VER
    PyPI       jubarte-redlines $VER
    GitHub     release $TAG (release.yml builds binaries + wheels)
  Summaries ride along on every channel — verify greps them afterwards.
  The bump kind was proven in step 0 (cargo-semver-checks); the API drift is in the log above.
EOF
if [ "$YES" = 0 ]; then
  read -r -p "  type 'v$VER' to confirm: " a
  [ "$a" = "v$VER" ] || die "aborted — nothing pushed; local commits/tag remain"
fi

# shellcheck source=scripts/push_main.sh
. scripts/push_main.sh
push_main "release/$TAG" "chore(release): $TAG" \
  "The release commits of jubarte $TAG, from scripts/release.sh. Merged at once with a merge commit: the tag names the commit the gates ran on." \
  || die "main was not updated — nothing is published; the release commits and the tag are local, rerun once main takes them"
cancel_queued_ci
if git ls-remote --tags origin "$TAG" | grep . >/dev/null; then
  step "tag $TAG already on origin — push skipped"
else
  git push origin "$TAG"
fi
step "pushed — release workflow started"

# =============================================================================
say "9. crates.io"
# =============================================================================

if crates_has; then
  step "jubarte-redlines $VER already on crates.io — skipped"
elif [ "$LOCAL_PUBLISH" = 1 ]; then
  # Finder litter under an include glob is "uncommitted" to cargo and stops
  # the publish (gitignored, so the clean-tree preflight never sees it).
  find . -name .DS_Store -not -path './target/*' -not -path './.worktrees/*' -delete
  # Prove the summary survived cargo's manifest normalization before shipping.
  cargo package --locked --no-verify >/dev/null
  tar -xzOf "target/package/jubarte-redlines-$VER.crate" \
    "jubarte-redlines-$VER/Cargo.toml" \
    | grep -F "\"$VER\" = \"$CRATES_SUMMARY\"" >/dev/null \
    || die "crates summary missing from the packaged manifest — not publishing"
  cargo publish --locked
  step "cargo publish done (index lags ~1 min)"
else
  ci_published "crates.io" "publish crates.io" crates_has
fi

# =============================================================================
say "10. npm"
# =============================================================================
check_now 10

if [ "$LOCAL_PUBLISH" = 1 ]; then
  # An account with two-factor auth answers a non-interactive publish with
  # EOTP (v0.10.1 stopped here). NPM_OTP=<code> passes a one-time password;
  # an interactive terminal lets npm prompt for it.
  if npm_has; then
    step "jubarte-wasm $VER already on npm — skipped"
  else
    if [ -n "${NPM_OTP:-}" ]; then
      publish_ok() { (cd jubarte-wasm/npm && npm publish --otp "$NPM_OTP"); }
    else
      publish_ok() { (cd jubarte-wasm/npm && npm publish); }
    fi
    publish_ok || die "npm publish failed. If npm asked for a one-time password (EOTP), run
           (cd jubarte-wasm/npm && npm publish --otp <code>)
         then rerun this same command with --skip-gates: every registry that
         already holds $VER is skipped."
    step "npm publish done"
  fi
  # The CLI depends on jubarte-wasm@^$VER, so it goes second. A one-time
  # password lasts about 30 s; a stale NPM_OTP fails here alone and the rerun
  # skips jubarte-wasm.
  if npm_cli_has; then
    step "jubarte-redlines $VER already on npm — skipped"
  else
    if [ -n "${NPM_OTP:-}" ]; then
      publish_cli() { (cd jubarte-wasm/cli && npm publish --otp "$NPM_OTP"); }
    else
      publish_cli() { (cd jubarte-wasm/cli && npm publish); }
    fi
    publish_cli || die "npm publish of jubarte-redlines failed. With two-factor auth, run
           (cd jubarte-wasm/cli && npm publish --otp <code>)
         or rerun this same command with --skip-gates and a fresh NPM_OTP."
    step "npm publish (jubarte-redlines CLI) done"
  fi
else
  # One job publishes both, jubarte-wasm first: the CLI depends on it.
  ci_published "npm jubarte-wasm" "publish npm" npm_has
  ci_published "npm jubarte-redlines" "publish npm" npm_cli_has
fi

# =============================================================================
say "11. PyPI"
# =============================================================================
check_now 11

# release.yml attaches the wheels to the GitHub release it creates. When any
# binary fails its release job is skipped (v0.10.1: the Windows runner could
# not check out long fixture paths), and the wheels exist only as workflow
# artifacts: the release is then created here from that run's artifacts,
# with a note naming the binaries it lacks.
release_from_artifacts() {
  local run_id=$1 a missing=""
  rm -rf dist/release dist/release-art
  mkdir -p dist/release
  # The run's sdist artifact is the one publish-pypi uploads: the release
  # takes it as is, never a second maturin build with other bytes.
  gh run download "$run_id" -D dist/release-art -R jandira-tech/jubarte-redlines
  find dist/release-art -type f -exec mv {} dist/release/ \;
  rm -rf dist/release-art
  # A run that lost a wheel job (or the sdist) never becomes a public
  # release: step 11's own wheel check would fire only after the release is
  # out, and a rerun would then meet a release that already exists. A
  # missing binary is named in the notes below; a missing wheel stops here.
  python3 scripts/check_release_artifacts.py dist/release --version "$VER" --sdist \
    || die "release.yml run $run_id lacks wheels or the sdist — no GitHub release was created; fix the release jobs, rerun the workflow on $TAG, then rerun this script"
  (cd dist/release && shasum -a 256 -- * > SHA256SUMS.txt)
  for a in linux-x86_64 linux-aarch64 macos-x86_64 macos-aarch64 windows-x86_64; do
    ls dist/release/jubarte-"$VER"-"$a".* >/dev/null 2>&1 || missing="$missing $a"
  done
  {
    printf '%s\n\n' "$GITHUB_SUMMARY"
    if [ -n "$missing" ]; then
      printf '> Not attached:%s (release.yml run %s failed to build them).\n\n' "$missing" "$run_id"
    fi
    # Only release.yml's release job attests, and it may have attested these
    # bytes before it died, so the note says only what is known (#1768).
    printf '> These assets were not published by release.yml'"'"'s release job: they come from run %s'"'"'s artifacts. Whether a build-provenance attestation covers one, gh attestation verify FILE -R jandira-tech/jubarte-redlines says.\n\n' "$run_id"
    awk -v ver="$VER" '
      index($0, "## [" ver "] - ") == 1 { on=1; next }
      on && /^## \[/ { exit }
      on { print }
    ' CHANGELOG.md
    [ -n "$PREV_TAG" ] && [ "$PREV_TAG" != "$TAG" ] && printf \
      '\n**Full Changelog**: https://github.com/jandira-tech/jubarte-redlines/compare/%s...%s\n' \
      "$PREV_TAG" "$TAG"
  } > dist/release-notes.md
  gh release create "$TAG" --title "jubarte $TAG" --notes-file dist/release-notes.md -R jandira-tech/jubarte-redlines dist/release/*
  step "GitHub release $TAG created from run $run_id${missing:+ — missing:$missing}"
}

# Waits for the release WITH its wheels: a release whose run is still going
# is not done uploading. Once the run has completed, a release that still
# lacks a wheel is left to the download below, which names what is missing.
wait_for_release() {
  local RUN_ID RUN_STATE
  ghrel_wheels && return 0
  step "waiting for release.yml (up to ~45 min)…"
  for _ in $(seq 1 90); do
    ghrel_wheels && break
    read -r RUN_ID RUN_STATE <<<"$(release_run)"
    [ "${RUN_STATE:-}" = completed ] && break
    sleep 30
  done
  if ! ghrel_has; then
    read -r RUN_ID RUN_STATE <<<"$(release_run)"
    [ "${RUN_STATE:-}" = completed ] \
      || die "release.yml for $TAG has not finished — rerun this command later"
    release_from_artifacts "$RUN_ID"
  fi
}
if [ "$NO_WAIT" = 0 ]; then
  wait_for_release
fi

if pypi_has; then
  step "jubarte-redlines $VER already on PyPI — skipped"
elif [ "$LOCAL_PUBLISH" = 1 ]; then
  rm -rf dist/pypi; mkdir -p dist/pypi
  got_wheels=0
  if [ "$NO_WAIT" = 0 ]; then
    if download_ci_wheels; then
      got_wheels=1
    else
      echo "  ! CI wheels unavailable — falling back to a local wheel build" >&2
    fi
  fi
  if [ "$got_wheels" = 0 ]; then
    # No partial wheel set reaches PyPI under a final version by accident:
    # the local build below is one platform's wheel. --no-wait is the
    # explicit consent to exactly that (it is also what skipped the
    # download above); the app's facts check would otherwise die on the
    # missing wheels only after the release is out.
    [ "$NO_WAIT" = 1 ] \
      || die "the $TAG release carries no jubarte_redlines wheels — PyPI would get only the local wheel + sdist under a final version; fix the release jobs, or rerun with --local-publish --no-wait to consent to the partial set"
    # The wheel's panic locations would name this home and checkout (#1480);
    # jubarte-python has no .cargo/config.toml recipe for RUSTFLAGS to replace.
    # As build-npm.sh does: the checkout by both its paths, and no whitespace,
    # which cargo would split the list on.
    repo_logical=$(pwd -L)
    repo_physical=$(pwd -P)
    case "$HOME$repo_logical$repo_physical" in
      *[[:space:]]*) die "whitespace in \$HOME or the checkout path; cargo splits RUSTFLAGS on it" ;;
    esac
    RUSTFLAGS="--remap-path-prefix=$HOME=~ --remap-path-prefix=$repo_logical=jubarte-redlines --remap-path-prefix=$repo_physical=jubarte-redlines" \
      uvx maturin build --release --manifest-path jubarte-python/Cargo.toml --out dist/pypi
    uvx maturin sdist --manifest-path jubarte-python/Cargo.toml --out dist/pypi
    echo "  ! only the local-platform wheel + sdist will reach PyPI (--no-wait given)" >&2
  fi
  if [ "$got_wheels" = 1 ]; then
    # One wheel per README platform row (seven). A release.yml run that lost
    # a wheel job must not ship a partial set under a final version.
    python3 scripts/check_release_artifacts.py dist/pypi --version "$VER" \
      || die "wheel set incomplete; see scripts/check_release_artifacts.py"
  fi
  uv publish --token "$UV_PUBLISH_TOKEN" dist/pypi/*
  step "uv publish done"
else
  ci_published "PyPI" "publish PyPI" pypi_has
fi

# =============================================================================
say "12. Verify — versions AND summaries"
# =============================================================================

npm_note() {
  npm view "jubarte-wasm@$VER" releaseNotes --json 2>/dev/null | \
  VER="$VER" TXT="$NPM_SUMMARY" node -e '
    let s = ""; process.stdin.on("data", d => s += d).on("end", () => {
      try {
        const o = JSON.parse(s || "null");
        process.exit(o && o[process.env.VER] === process.env.TXT ? 0 : 1);
      } catch { process.exit(1) }
    })'
}
# release.yml's release job attests every asset SHA256SUMS.txt names; one
# archive, verified against the workflow that signed it on a GitHub-hosted
# runner, proves the attestation ran.
gh_attested() {
  local f="jubarte-$VER-linux-x86_64.tar.gz" rc=0 sha
  # bound to this tag and its commit: the repository, workflow and runner
  # alone pass an archive another release's run attested (#1749)
  sha=$(git rev-parse "$TAG^{commit}") || return 1
  rm -rf dist/attest
  gh release download "$TAG" -p "$f" -D dist/attest -R jandira-tech/jubarte-redlines >/dev/null 2>&1 \
    || { rm -rf dist/attest; return 1; }
  gh attestation verify "dist/attest/$f" -R jandira-tech/jubarte-redlines \
    --signer-workflow jandira-tech/jubarte-redlines/.github/workflows/release.yml \
    --source-ref "refs/tags/$TAG" --source-digest "$sha" \
    --deny-self-hosted-runners >/dev/null 2>&1 || rc=1
  rm -rf dist/attest
  return "$rc"
}

# Each channel installed the way a user installs it (#1756; checklist
# after#9-#11): 0.13.0 never reached npm, and no step said so. The proofs
# work under dist/install-proof and leave nothing behind.
host_archive() {
  local os arch
  case "$(uname -s)" in Darwin) os=macos ;; Linux) os=linux ;; *) return 1 ;; esac
  case "$(uname -m)" in arm64 | aarch64) arch=aarch64 ;; x86_64 | amd64) arch=x86_64 ;; *) return 1 ;; esac
  printf 'jubarte-%s-%s-%s.tar.gz\n' "$VER" "$os" "$arch"
}
# This machine's archive from the GitHub release: its line in SHA256SUMS.txt
# matches, and the binary inside answers `jubarte $VER`.
archive_runs() {
  local f dir=dist/install-proof/archive rc=0
  f=$(host_archive) || return 1
  rm -rf "$dir" && mkdir -p "$dir"
  gh release download "$TAG" -p "$f" -p SHA256SUMS.txt -D "$dir" -R jandira-tech/jubarte-redlines >/dev/null 2>&1 || rc=1
  [ "$rc" = 0 ] && (cd "$dir" \
    && awk -v f="$f" '$2 == f' SHA256SUMS.txt | grep . | shasum -a 256 -c - >/dev/null 2>&1 \
    && tar xzf "$f" \
    && [ "$("./${f%.tar.gz}/jubarte" --version 2>/dev/null)" = "jubarte $VER" ]) || rc=1
  rm -rf dist/install-proof
  return "$rc"
}
# A fresh environment installs the wheel PyPI serves this machine, and its
# engine is $VER.
pip_installs() {
  local got
  # --no-build-package: a wheel or nothing — this machine has Rust and
  # would build the sdist, hiding a wheel PyPI does not serve it (#1765).
  # --python 3.12, a standard CPython: uv's default here may be a
  # free-threaded build, which no abi3 wheel serves (0.13.0's PASS was an
  # sdist build on 3.15t).
  got=$(uv run -q --isolated --no-project --python 3.12 --refresh-package jubarte-redlines \
          --no-build-package jubarte-redlines --with "jubarte-redlines==$VER" \
    python -c 'import jubarte_redlines as j; print(j.capabilities()["engine_version"])' 2>/dev/null) || return 1
  [ "$got" = "$VER" ]
}
# The npm CLI answers in the engine's words, named by its argv[0]. It runs
# in an empty project of its own: npx answers from a node_modules it finds
# before it asks the registry (#1765).
npx_runs() {
  local dir=dist/install-proof/npx got="" rc=0
  rm -rf "$dir" && mkdir -p "$dir"
  got=$(cd "$dir" && npm init -y >/dev/null 2>&1 \
        && npx -y --prefer-online "jubarte-redlines@$VER" --version 2>/dev/null) || rc=1
  rm -rf dist/install-proof
  [ "$rc" = 0 ] || return 1
  case "$got" in "jubarte-redlines $VER" | "jubarte $VER") ;; *) return 1 ;; esac
}
# `npm i jubarte-wasm@$VER` into an empty project, then require() it.
npm_requires() {
  local dir=dist/install-proof/npm rc=0
  rm -rf "$dir" && mkdir -p "$dir"
  (cd "$dir" \
    && npm init -y >/dev/null 2>&1 \
    && npm i --prefer-online --no-audit --no-fund --silent "jubarte-wasm@$VER" >/dev/null 2>&1 \
    && VER="$VER" node -e '
      const w = require("jubarte-wasm");
      const v = JSON.parse(require("fs").readFileSync("node_modules/jubarte-wasm/package.json")).version;
      process.exit(v === process.env.VER && typeof w.compareDocuments === "function" ? 0 : 1)') || rc=1
  rm -rf dist/install-proof
  return "$rc"
}
# docs.rs builds after crates.io and lags, so step 12 only warns on it.
docsrs_built() {
  curl -fsS "https://docs.rs/crate/jubarte-redlines/$VER/status.json" 2>/dev/null \
    | grep '"doc_status":true' >/dev/null
}

ok=1
check() { if eval "$2"; then step "ok — $1"; else echo "  ✗ $1" >&2; ok=0; fi; }
check "crates.io  jubarte-redlines $VER" "eventually crates_has"
check "npm        jubarte-wasm $VER" "eventually npm_has"
check "npm        jubarte-redlines $VER" "eventually npm_cli_has"
check "npm        releaseNotes.$VER shipped" npm_note
check "PyPI       jubarte-redlines $VER" "eventually pypi_has"
check "GitHub     release $TAG" ghrel_has
check "GitHub     notes carry --github-summary" gh_note
check "GitHub     $(host_archive || echo "no archive for $(uname -sm)") matches SHA256SUMS.txt and answers jubarte $VER" archive_runs
check "PyPI       pip install jubarte-redlines==$VER in a fresh environment: engine $VER" "eventually pip_installs"
check "npm        npx jubarte-redlines@$VER --version answers $VER" "eventually npx_runs"
check "npm        npm i jubarte-wasm@$VER, then require()" "eventually npm_requires"
if docsrs_built; then
  step "ok — docs.rs   built jubarte-redlines $VER"
else
  echo "  ! docs.rs   has not built jubarte-redlines $VER yet (it lags crates.io): https://docs.rs/crate/jubarte-redlines/$VER" >&2
fi
# A warning, not a failure: a release made here from run artifacts (step
# 11, when release.yml's release job did not run) has no attestation.
if gh_attested; then
  step "ok — GitHub     assets carry build provenance (gh attestation verify)"
else
  echo "  ! GitHub     jubarte-$VER-linux-x86_64.tar.gz has no build provenance that verifies — release.yml's release job did not attest it" >&2
fi
[ "$ok" = 1 ] || die "verification failed — check the lines marked ✗"
check_now 12

# =============================================================================
say "13. App — jubarte.pro, the app's version, App Store, benchmark"
# =============================================================================
# After verify: the site reads the GitHub release's files and the npm package
# that step 12 just proved live, and release_info/website_data's figures. It
# runs in the app repository's checkout (proved in step 0), and ends on the
# check that its data/facts.jsonl — what jubarte.pro and the Mac app print
# the engine's version, date and files from — names $VER. A failure here
# leaves the release itself intact; rerun the app's script on its own.
app_release \
  || die "the app's step failed — the release is out; rerun $APP_REPO/scripts/release-engine.sh $VER --engine-dir $PWD"
check_now 13

say "Released jubarte $VER"
echo "  https://github.com/jandira-tech/jubarte-redlines/releases/tag/$TAG"
echo "  https://crates.io/crates/jubarte-redlines/$VER"
echo "  https://pypi.org/project/jubarte-redlines/$VER/"
echo "  https://www.npmjs.com/package/jubarte-wasm/v/$VER"

# =============================================================================
say "Checklist — [you] items still owed after the script"
# =============================================================================
# The script is done; the checklist is not. Every [you] item of the "after"
# phase is still owed (app build, App Store, notarization, the benchmark
# lane, the post-release reproduction, the handoff notes) — each
# printed with its command, from the same single source of truth, so none
# of it has to live in someone's head.
checklist_items | sed "s/@V@/$VER/g" | awk -F'|' '
  $1 == "after" && $2 == "you" { printf "  [you] %s\n             %s\n", $3, $4 }
'
