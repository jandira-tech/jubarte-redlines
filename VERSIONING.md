<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Versioning & release — jubarte family

All products under `ooxmlsdk/jubarte*` share **Semantic Versioning**
([semver.org](https://semver.org/)) and **Keep a Changelog**
([keepachangelog.com](https://keepachangelog.com/)). Pre-1.0 rule (this family):

| bump | when |
|---|---|
| **minor** (`0.x.0`) | new features, large perf wins that ship in production, public API surface growth |
| **patch** (`0.x.y`) | bugfixes, Q0 perf micro-wins, docs, package validity — no intentional Q1 semantic change |
| **major** (`1.0.0+`) | reserved for first stable API freeze |

## Repos in this folder

| repo | artifact | version files | bump tool |
|---|---|---|---|
| **jubarte-rs** | crates.io crate + CLI `jubarte` | `Cargo.toml` `[package].version`, `CHANGELOG.md` | `bun scripts/bump-version.mjs x.y.z` |
| **jubarte-app** | Mac App Store / Tauri shell | `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/index.html`, `CHANGELOG.md` | `bun run bump x.y.z` |
| **jubarte-site** | marketing/site (optional) | `package.json` | manual / site deploy only |

`jubarte-app` depends on the engine via:

```toml
jubarte = { path = "../../jubarte-rs", default-features = false }
```

So **always version and release jubarte-rs first**, then bump the app if the
shell needs a store build that embeds the new engine.

## Step-by-step: cut an engine release (jubarte-rs)

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
   - This branch stack (parity restore + package validity + accept skips + name cache)
     is a **minor** (`0.1.0` → `0.2.0`).

3. **Codemod the version**  
   ```bash
   bun scripts/bump-version.mjs 0.2.0
   ```

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

6. **Commit + tag**  
   ```bash
   git add Cargo.toml CHANGELOG.md VERSIONING.md scripts/bump-version.mjs
   git commit -m "chore(release): v0.2.0"
   git tag -a v0.2.0 -m "v0.2.0"
   # push when ready: git push && git push --tags
   ```

7. **Bench stamp** (full Word-visual ledger)  
   From `neurotic_docx_bench` (sibling of `ooxmlsdk` or `BENCH_DIR`):  
   ```bash
   uv run bench run --only jubarte-rust --rerun --accept-compare
   ```  
   That generates redlines, renders, scores **script_redlines**, and
   **accepted_changes** (accept-all on tool redlines vs Word accepted oracle).

8. **Publish** (when crates.io is intentional)  
   `cargo publish` from a clean tree matching the tag.

## Step-by-step: cut an app release (jubarte-app)

1. Point path dep at the engine commit/tag you intend to ship.  
2. `bun run bump 0.3.1` (syncs package / Cargo / tauri.conf / app-bar).  
3. CHANGELOG entry under `## [0.3.1]`.  
4. `bun run build` / MAS publish scripts per `MAC_APP_STORE_RELEASE.md`.

## What “version up” means for each surface

| surface | what bumps | who consumes |
|---|---|---|
| `Cargo.toml` version | crate semver | crates.io, docs.rs, dependants |
| git tag `vX.Y.Z` | immutable release id | humans, CI |
| binary content hash under `utils/jubarte/jubarte-rust/` | neurotic `tool_version` (`jubarte-rust@<sha12>`) | RESULTS.md ranking |
| app store build number | Tauri/MAS | App Store Connect |

The neurotic bench does **not** use Cargo semver for jubarte-rust; it hashes the
installed binary directory. **Refreshing the binary is the versioning step for
the visual ledger.** Cargo semver is for the library/CLI product line.

## Codemod contract

- One command bumps every hard-coded version field in that repo.  
- CHANGELOG is always human-written (never auto-generated prose).  
- Never bump version in the same commit as a multi-mechanism perf experiment;
  cut the release after the measured stack is closed.
