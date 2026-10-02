<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Versioning & release — jubarte family

All products under `jubarte*` share **Semantic Versioning**
([semver.org](https://semver.org/)) and **Keep a Changelog**
([keepachangelog.com](https://keepachangelog.com/)). Pre-1.0 rule (this family):

| bump | when |
|---|---|
| **minor** (`0.x.0`) | new features, large perf wins that ship in production, public API surface growth |
| **patch** (`0.x.y`) | bugfixes, Q0 perf micro-wins, docs, package validity — no intentional Q1 semantic change |
| **major** (`1.0.0+`) | reserved for first stable API freeze |

## Repos

| repo | artifact | version files | bump tool |
|---|---|---|---|
| **jubarte-redlines** (this repo) | crates.io crate + CLI `jubarte` | `Cargo.toml` `[package].version`, `CHANGELOG.md` | `scripts/release.sh x.y.z …` (calls `bump-version.mjs`) |
| **jubarte-redlines** npm CLI (`jubarte-wasm/cli/`) | npm package `jubarte-redlines` (the `npx jubarte-redlines` runner) | `jubarte-wasm/cli/package.json` | `scripts/release.sh` (publishes it with the engine version) |
| **jubarte-app** (`jubarte-app/` — a plain tracked directory in this repo that also carries its own nested `.git`; there is no `.gitmodules`) | Mac App Store / Tauri shell | `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/index.html`, `CHANGELOG.md` | `scripts/release.sh` step 1 (the engine's version); step 12 (`scripts/release_downstream.sh`) commits them on `release/vx.y.z` in the app's repository; `release_downstream.sh x.y.z --app` uploads the App Store build |
| **jubarte-site** (`jubarte-app/jubarte-site/`) | jubarte.pro (Cloudflare Worker) | `site/data/release.ts`, `package.json` (`jubarte-wasm`), `pnpm-workspace.yaml` | `scripts/release.sh` step 12: `jubarte-site/scripts/release.sh engine x.y.z` (download page, demo engine, deploy); benchmark figures follow with `release.sh bench x.y.z` after neurotic_docx_bench's `scripts/release_jubarte.py` |

`jubarte-app` depends on the engine via:

```toml
jubarte = { package = "jubarte-redlines", path = "../..", default-features = false }
```

So **always version and release jubarte-rs first**, then bump the app if the
shell needs a store build that embeds the new engine.

## Step-by-step: cut an engine release (jubarte-redlines)

> [!WARNING]
> **`scripts/release.sh` is the source of truth for releases.** It owns the
> version sync, the six required release notes, the gates, the commit, the
> tag and every publish. `scripts/bump-version.mjs` is only the Cargo.toml +
> README codemod `release.sh` calls; run on its own it leaves
> jubarte-python, jubarte-wasm/npm and the lockfiles behind. Steps 3 and 6
> below describe what `release.sh` does, not a manual alternative.

1. **Quality gate (do not skip)**  
   - `cargo test --all-features` (only known pre-existing failures allowed)
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `tools/parity_ladder.py sweep --bin target/release/jubarte` → 0 NEW  
   - Optional: permanent ABBA matrix if the change claimed a wall win  
   - **Ring 2 (OpenXmlValidator):** `scripts/redline-sweep.sh … --validate` → no NEW keys vs `tools/validity_baseline.tsv` (local; requires `dotnet`)
   - **Ring 3 (real Word open probe, macOS release gate):** before any crates.io publish or bench-pin promotion, run
     `scripts/redline-sweep.sh <both CSVs> <src> parity/_scratch/sweep_<date> --probe`
     Required: `probe_fail=0`. Never use `/tmp` for Word probe paths (sandbox); use `parity/_scratch`.
   - **Criterion local gate (perf-affecting PRs):** `cargo bench --bench redline -- --baseline m233_head` — a **>5%** regression on any case blocks merge.
   - **Speed vs quality (B-fixes):** median generate time > **+10%** vs the M233 baseline (see `docs/SPEED_REVIEW.md`) triggers a perf review before merge.

2. **Decide the bump**  
   - Perf banked-without-wall + package/notes validity → usually **patch** or **minor**  
     if the release aggregates many accepted ships.  
   - New public API or features (e.g. the `convert` revision-painting options)
     → **minor** is the safer call even when behavior is additive.

3. **Codemod the version** — done by `release.sh` step 1, which runs
   `bun scripts/bump-version.mjs x.y.z` and then syncs the manifests and
   lockfiles that script does not own. Do not run it by hand.

4. **Write CHANGELOG.md**  
   Add `## [0.2.0] - YYYY-MM-DD` with `### Added` / `### Changed` / `### Fixed` /
   `### Performance`. Link footer `[0.2.0]: …/tag/v0.2.0`.

5. **Build & refresh binaries**  
   ```bash
   cargo build --release --bin jubarte
   # neurotic_docx_bench probe (content-hashed as tool_version)
   cp -f target/release/jubarte \
     ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust/jubarte
   cp -f target/release/jubarte \
     ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust/redline
   # local CLI convenience
   cp -f target/release/jubarte "$HOME/.local/bin/jubarte"  # optional
   ```

6. **Commit + tag** — done by `release.sh`; shown for reference only.
   ```bash
   git add Cargo.toml CHANGELOG.md VERSIONING.md scripts/bump-version.mjs
   git commit -m "chore(release): v0.2.0"
   git tag -a v0.2.0 -m "v0.2.0"
   # push when ready: git push && git push --tags
   ```
   Pushing `v*` runs `.github/workflows/release.yml`: it refuses a tag that
   does not match `Cargo.toml`, builds `jubarte` for linux/macos/windows
   (x86_64 + aarch64), and creates the `jubarte vX.Y.Z` GitHub release with
   the CHANGELOG section as notes and the archives + `SHA256SUMS.txt`
   attached.

7. **Bench stamp** (full Word-visual ledger)  
   From `neurotic_docx_bench` (sibling of `ooxmlsdk` or `BENCH_DIR`):  
   ```bash
   uv run bench run --only jubarte-rust --rerun --accept-compare
   ```  
   That generates redlines, renders, scores **script_redlines**, and
   **accepted_changes** (accept-all on tool redlines vs Word accepted oracle).

8. **Publish**  
   `scripts/release.sh x.y.z` is the one-stop path: it re-checks the gates,
   syncs every manifest/lockfile (including the ones `bump-version.mjs` does
   not own), commits, tags, pushes, and publishes crates.io + npm + PyPI,
   while `.github/workflows/release.yml` builds the binaries/wheels and
   creates the GitHub release.

   Six per-release notes are **required flags** — each lands in the
   channel its registry accepts:

   | flag | lands in |
   |---|---|
   | `--changelog-summary` | `> **Summary.** …` under `## [x.y.z]` in CHANGELOG.md |
   | `--crates-summary` | `[package.metadata.release-notes]` in Cargo.toml (ships in the `.crate`) |
   | `--npm-summary` | `releaseNotes."x.y.z"` in the npm package.json (published packument) |
   | `--pypi-summary` | `# release-notes` comment in `jubarte-python/pyproject.toml` (ships in the sdist) |
   | `--github-summary` | annotated-tag body → top of the GitHub release notes |
   | `--how-readme-and-other-docs-were-updated` | `> **Docs.** …` under the summary in CHANGELOG.md + the release commit body |

   Before it writes the notes, `release.sh` lists the README, `docs/` and
   `skills/` files changed since the previous tag, so the docs statement is
   checked against what actually changed. A blank value counts as missing.

   `--*-comments` aliases work too. The verify step greps each registry/
   artifact to prove the note shipped. `scripts/release.sh x.y.z --dry-run`
   rehearses everything locally; every publish step skips a version that is
   already live, so a failed run can simply be re-run.

   Before the dry-runs, step 5 is a required API-docs drift assessment:
   `cargo doc --no-deps --document-private-items --open` opens the rendered
   docs for review, `scripts/api_snapshot.py` writes a machine-readable copy
   under `docs/api/` (`jubarte-vx.y.z.json.gz` rustdoc JSON +
   `jubarte-vx.y.z.api.txt` flat sorted listing + the four
   `jubarte-wasm-<target>-vx.y.z.d.ts` files), and the script diffs the new
   listing against the previous release's snapshot. The snapshot ships in the
   release commit, so `git diff` between release tags shows API drift.

   Read the drift as semver: a new field on a public struct that is not
   `#[non_exhaustive]`, or a changed public fn signature, breaks struct
   literals and callers, so it needs a **minor** bump before 1.0 (0.10.1
   shipped four such fields as a patch).

### Before you run it

- **Desktop app.** `release.sh` step 1 moves jubarte-app to the engine
  version (`package.json`, `src-tauri/tauri.conf.json`,
  `src-tauri/Cargo.toml`, `src/index.html`) and step 2 refuses to go on
  until `jubarte-app/CHANGELOG.md` has a `## [x.y.z]` section naming
  `jubarte-redlines x.y.z`. `tests/release_metadata.rs` fails main otherwise.
- **Lockfiles.** Step 1 runs `cargo update --offline -p jubarte-redlines` in
  the root, `jubarte-python`, `jubarte-wasm`, `jubarte-rust-inproc` and
  `jubarte-app/src-tauri`. A metadata-only pass does not refresh a path
  dependency's version.
- **npm two-factor auth.** A non-interactive `npm publish` answers `EOTP`.
  Run from a terminal, or pass a fresh code: `NPM_OTP=123456
  scripts/release.sh x.y.z …`. Codes live about 30 seconds, so export it just
  before step 9.
- **Credentials.** `cargo login`, `npm whoami`,
  `UV_PUBLISH_TOKEN`, `gh auth status`. Read tokens from their files at run
  time; never paste them into the command line history.
- **Windows paths.** Git for Windows stops at 260 characters. The release
  workflow sets `core.longpaths`, and `tests/repo_paths_fit_windows.rs` keeps
  every tracked path under 200 characters. Stage long fixture names under
  short ones instead of raising the limit.

### When a run stops

Re-run the same command with `--skip-gates` once the gates have passed on
the same commit. Every step checks what already exists:

| stopped at | what the re-run does |
|---|---|
| before the tag | redoes the version sync and the release commit |
| after the tag | keeps the release commit and the tag; never re-tags |
| after the wasm artifacts commit | keeps that commit (`build(wasm): regenerate npm artifacts for vX.Y.Z`) instead of rebuilding: a rebuild stamps a newer `ENGINE_COMMIT` than the package npm may already hold |
| any publish | skips every registry that already has the version |

Do not hand-edit a tracked file between the tag and the re-run: the clean
tree check stops the run, and a tag cannot be moved once pushed.

### When release.yml skips the GitHub release

The release job needs every binary. If one fails (v0.10.1: the Windows
checkout), the wheels and the other binaries exist only as workflow
artifacts. Step 10 notices a finished run with no release and then:

1. `gh run download <run>` fetches every artifact into `dist/release/`;
2. builds the sdist from `git archive vX.Y.Z` (the tagged source, not the
   working tree);
3. writes `SHA256SUMS.txt`, and creates the release with `gh release create`.
   The notes are the tag summary, a line naming the binaries that are
   missing, the CHANGELOG section and the compare link;
4. publishes the wheels plus the sdist to PyPI.

A missing binary cannot be added under the same tag once the fix lands on a
later commit. Ship it with the next patch release.

Archives are signed in release.yml's release job, so the archives this
path publishes are unsigned. Once release signing is on
(docs/SELF_UPDATE.md, "Signed releases"), signed-era binaries refuse them,
and `jubarte self-update` skips that release until a signed one follows.

### Verify every registry

`release.sh` step 11 does this. To check by hand:

```bash
V=x.y.z
curl -sfA jubarte-release https://crates.io/api/v1/crates/jubarte-redlines/$V | jq -r .version.num
npm view jubarte-wasm@$V version releaseNotes
curl -sf https://pypi.org/pypi/jubarte-redlines/$V/json | jq -r '.urls[].filename'
gh release view v$V --json assets -q '.assets[].name'
```

crates.io answers 403 to a request with no User-Agent, so always pass `-A`.

## Step-by-step: cut an app release (jubarte-app)

An engine release already moves the app's version (see above). For an
app-only release:

1. Point path dep at the engine commit/tag you intend to ship.  
2. `bun run bump 0.3.1` (syncs package / Cargo / tauri.conf / app-bar).  
3. CHANGELOG entry under `## [0.3.1]`.  
4. `bun run build` / MAS publish scripts per `MAC_APP_STORE_RELEASE.md`.

## What “version up” means for each surface

| surface | what bumps | who consumes |
|---|---|---|
| `Cargo.toml` version | crate semver | crates.io, docs.rs, dependants |
| git tag `vX.Y.Z` | immutable release id | humans, CI |
| binary content hash under `utils/jubarte/jubarte-rust/` | neurotic `tool_version` (`jubarte-rust@<sha12>`) | [bench RESULTS.md](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md) ranking |
| app store build number | Tauri/MAS | App Store Connect |

The neurotic bench does **not** use Cargo semver for jubarte-rust; it hashes the
installed binary directory. **Refreshing the binary is the versioning step for
the visual ledger.** Cargo semver is for the library/CLI product line.

## Codemod contract

- One command bumps every hard-coded version field in that repo.  
- CHANGELOG is always human-written (never auto-generated prose).  
- Never bump version in the same commit as a multi-mechanism perf experiment;
  cut the release after the measured stack is closed.
