<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# Keeping the repository tidy

This page is for maintainers. It says what the machine enforces, what the
rules are where it cannot, how often each chore happens, and what is still
open. Review it at every minor release.

## What the machine enforces

A rule that depends on memory is forgotten. These run in CI:

| Check | Refuses |
|---|---|
| `tests/repo_hygiene.rs` | A new top-level file or folder that is not on its list; a tracked file over 1 MB outside the listed folders; tracked files that outgrow the size budget of the whole tree (72 MiB) or of their top-level folder; a tracked executable, WebAssembly module, log or empty file; a home-directory path or the retired project name in a tracked file; a `LICENSE` copy that differs from the root one; a file outside the listed few that still names the former licence |
| CI's `wasm` job | An npm package that does not build, fails its smoke test or its CLI tests, names a builder's home directory, or would be packed without a target, `LICENSE` or `NOTICE` |
| `tests/corpus_present.rs` | Running the tests without the corpus submodule or without the downloaded test documents, with the command that fetches each |
| `scripts/fetch_fixtures.py` | A downloaded test document whose SHA-256 is not the one `tests/corpus/hf_fixtures.sha256` records |
| `dco.yml` (`scripts/check_dco.py`) | A commit in a pull request to `main` that its author did not sign off |
| `scripts/check_links.py` | A relative link in tracked Markdown that names no tracked file |
| `scripts/gen_docs.sh --check` | A generated guide or README edited by hand, or left stale |
| `scripts/sync_license_files.py --check` | A package folder whose `LICENSE` or `NOTICE` is missing or differs from the root file; an upstream notice missing from `NOTICE` |
| `reuse lint` | A file with no licence and copyright record |
| `cargo deny check` | A dependency with an advisory, or a licence that is not on the list |
| gitleaks, CodeQL | A committed secret; a known vulnerable pattern |
| OpenSSF Scorecard (`scorecard.yml`, weekly) | Nothing: it publishes the score the README badge shows and keeps the findings as a run artifact |
| `cargo semver-checks` (the `semver` job) | Nothing until 1.0: it reports a public API change against the last release |
| `tests/suite_membership.rs` | A test file that belongs to no suite |

To add a top-level entry or a large file, change the list in
`tests/repo_hygiene.rs` in the same pull request, and say why in its
description.

## Rules

- **Scratch stays in `.scratch/`.** Probes, logs, sweep outputs and
  temporary clones never enter a commit.
- **Plans and review notes stay out.** A plan, an agent's notes, or a review
  transcript belongs in the pull request, an issue, or a private notes
  folder (`docs/archive/` is ignored by git for this). The repository holds
  what a user or a maintainer reads to use or change the code.
- **Every document has a reader.** A page under `docs/` is for users or for
  maintainers, and [docs/README.md](../README.md) lists it. A page nobody
  lists is deleted.
- **Every fixture has a source.** A new test document is recorded in
  `REUSE.toml` with where it came from and its licence. A third-party
  document, or one whose rights are unclear, goes to the
  `jubarte-test-corpus` repository (the submodule `tests/corpus/bench`),
  never into this one. Documents whose origin is not confirmed are in a
  public Hugging Face dataset instead; `tests/corpus/hf_fixtures.sha256`
  pins its revision and every file's SHA-256.
- **No built artifact is tracked.** The npm packages are built in CI and
  at the release tag; nothing compiled is committed or edited by hand.
- **A script earns its place.** A script that no workflow, no test, no other
  script and no document uses is deleted.
- **Contributions are signed off** (`git commit -s`) and reviewed by a code
  owner. `main` takes no direct push and no force push.

## Cadence

**Every week**

- Merge or close the Dependabot pull requests. Minor and patch updates
  arrive grouped; a major update is reviewed alone.
- Triage new issues: label, reproduce, and answer within three working
  days. Close an issue that has waited 60 days for its reporter.
- Look at the CI runs that failed and passed on retry. A flaky test is a
  bug: open an issue for it.

**Every release**

- Run `scripts/release.sh`; it is the checklist. It refuses a release whose
  changelog, benchmark evidence, generated documents or licence records are
  not in order.
- Record the public API listing in `docs/api/`.
- Open one built archive, one wheel and one npm tarball and confirm each
  holds `LICENSE`, `NOTICE` and the font licences.
- Announce it: the GitHub release, and a Discussions post.

**Every quarter**

- Review the minimum supported Rust version.
- Prune merged and stale branches and local worktrees.
- Delete scripts and documents that lost their last user.
- Re-run the OpenSSF best-practice self-assessment
  ([OPENSSF.md](OPENSSF.md)).
- Read this page and correct it.

## Backlog

The first three are meant for 0.20.0.

1. **Retire `oxml-opc` and `quick-xml` 0.41.** `jubarte-opc` still wraps
   `oxml_opc` for four small types, which keeps a second `quick-xml` in
   every build.
2. **Unify the Cargo workspaces.** `jubarte-python`, `jubarte-wasm` and
   `jubarte-rust-inproc` are separate workspaces with their own lockfiles.
   Make them members of the root workspace with one lockfile.
3. **Triage the open issues.** Close duplicates and fixed ones, give the
   rest a milestone, and keep about ten labelled `good first issue`.
4. **Sign release archives.** Add the zipsign public key under `keys/` and
   the private key as a release secret, so `jubarte self-update` verifies
   what it downloads.
5. **Split `tests/convert_docx_to_pdf.rs`** (2.4 MB in one file) and rename
   the corpus folders whose names describe work in progress
   (`batch_to_fix`, `broken_ones_two`).
6. **Decide where `jubarte-rust-inproc` lives.** It serves only the
   benchmark; the benchmark's repository may be its home.
7. **Make the semver check blocking** at 1.0. Until then the `semver`
   job only reports.

## Growing adoption

- **A five-minute start for each ecosystem.** The README shows one install
  line and one working command for the CLI, Rust, Python and Node before
  anything else. The documentation site (one ReadTheDocs project per crate)
  carries the rest.
- **Evidence over claims.** Publish the Word-parity benchmark numbers with
  every release, and keep the comparison pages in `docs/adoption/` current.
- **An approachable tracker.** Keep `good first issue` and `help wanted`
  stocked, answer questions in Discussions, and thank first-time
  contributors by name in the release notes.
- **Be where users look.** List jubarte in awesome-rust, the MCP server
  directories and the agent-skill registries. Announce a release that
  changes what users can do (a licence change, a new binding) on r/rust,
  This Week in Rust and Hacker News.
- **Stay easy to depend on.** Semantic versions, a changelog a user can
  read, no long-lived publish tokens, signed and attested releases.
