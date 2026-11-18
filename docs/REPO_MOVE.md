<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Moving the repository to jandira-tech/jubarte

The engine's GitHub repository moves from `jandira-tech/jubarte-redlines`
to `jandira-tech/jubarte`. This is the runbook. Nothing in it has been done
yet.

The move is a GitHub **rename**, not a new repository plus a push. A rename
keeps the issues, pull requests, releases, tags, stars, secrets, variables,
environments, rulesets, webhooks, self-hosted runners and Actions history,
and GitHub redirects the old web, git and API URLs to the new name. Those
redirects last only while no repository takes the old name again, so never
create a new `jandira-tech/jubarte-redlines` afterwards.

Only the repository is renamed. The packages keep their names: crates.io
`jubarte-redlines` (the compare crate) and the other eight crates, PyPI
`jubarte-docx` and `jubarte-redlines`, npm `jubarte-wasm` and
`jubarte-redlines`. The local checkout folder `~/T/jubarte-redlines` keeps
its name as well, because the bench and the agents' instructions use that
path.

## 0. Blocker: the name is taken

`jandira-tech/jubarte` already exists. It is a public placeholder created on
2026-07-15, holding only a `.gitignore` and the Jules review workflow, with
two closed pull requests (#1 and #2). The engine's own `jules.yml` already
supersedes that workflow. GitHub will not rename onto a taken name. The
owner either:

- renames the placeholder (for example to `jubarte-placeholder`, then
  archives it), which keeps its two PRs, or
- deletes it.

Deleting cannot be undone. Renaming is the safer choice.

## 1. Order: move before publishing 0.19.0

Do the move before the 0.19.0 publish:

- Each registry stores the `repository` URL with each version, and a
  published version cannot be changed. 0.19.0 is the first publish of
  seven new crates and of PyPI `jubarte-docx`, so publishing first would
  record the old URL on all of them for good.
- Every trusted publisher (crates.io, PyPI, npm) names the repository.
  Configuring them before the move means configuring them twice.

## 2. The move itself (owner)

1. Free the name (section 0).
2. GitHub → `jandira-tech/jubarte-redlines` → Settings → General →
   Repository name: `jubarte` → Rename.
3. Merge the rewrite into `pre-release`, then into `main`. Run this on a
   branch:

   ```sh
   python3 scripts/repo_move.py           # dry run: lists every file
   python3 scripts/repo_move.py --apply   # rewrites 39 files
   python3 scripts/library_readmes.py     # regenerates the 4 generated READMEs, plus the crate READMEs once they are generated
   python3 scripts/repo_move.py --check   # must print nothing and exit 0
   ```

   The script rewrites `jandira-tech/jubarte-redlines` → `jandira-tech/jubarte`
   and two exact unqualified forms:
   - the self-updater's `REPO_NAME` in `crates/jubarte-cli/src/update.rs`;
   - release.sh's "repository jubarte-redlines," trusted-publisher lines.

   It leaves alone the historical records (plans, handoffs, reviews,
   evidence JSON, reports; see `FROZEN` in the script) and every bare
   package name. Before this commit merges, run:
   - `python3 scripts/test_release_sh.py`
   - `python3 scripts/library_readmes.py --check`
   - `cargo nextest run --test release_metadata --test readme_coverage`

   All three passed on the rewritten tree when this runbook was written.
4. Update every local clone: `git remote set-url origin
   https://github.com/jandira-tech/jubarte.git`. The old URL keeps working
   through the redirect, but `gh` calls without `-R` then resolve the new
   name anyway.

## 3. Registries and services (owner, after step 2)

The OIDC trusted publishers check the repository name in the token, and
after the rename the token says `jandira-tech/jubarte`.

| Service | What to change |
|---|---|
| crates.io | For each crate's Settings → Trusted Publishing, set owner `jandira-tech`, repository `jubarte`, workflow `release.yml`, environment `release`. Only `jubarte-redlines` exists today; the other eight get theirs after their first, token-based, publish (release.sh step "before" rows). |
| PyPI | Pending or existing publisher for `jubarte-docx` and `jubarte-redlines`: repository `jubarte` (not `jubarte-redlines`), workflow `release.yml`, environment `release`. If one was already made with the old name, delete it and add the new one. |
| npm | `npm trust github jubarte-wasm --file release.yml --repo jandira-tech/jubarte --env release --allow-publish -y`, and the same for `jubarte-redlines`. npm's provenance check also refuses a publish whose `package.json` `repository.url` names another repository. The rewrite fixes both package.json files, so publish only from a commit after step 2.3. A new npm configuration expires unless its first publish lands within two days (release.sh's "before" rows record the same rule), so run both `npm trust github` commands shortly before the 0.19.0 publish, and recreate any that expired. |
| ReadTheDocs | Import the projects after the rename, from `jandira-tech/jubarte` (docs/rtd/README.md). A project imported before the rename needs its repository URL changed in Admin → Settings, and its webhook re-created. |
| Codecov | Codecov follows the rename through the GitHub app. Check that `codecov.io/gh/jandira-tech/jubarte` shows the history and that the next CI upload lands there. Keep the `CODECOV_TOKEN` secret, which moves with the repository. |
| SonarCloud | The project stays bound by repository id. Its key may still read `jubarte-redlines`; that is cosmetic. Confirm the next PR analysis runs. |
| OpenSSF Best Practices | Edit the project's repository URL on bestpractices.dev. |
| Jules | The GitHub app is installed at org level; confirm the next PR gets its review. |

## 4. What keeps working through the redirect, and what to check

- **Installed binaries' `jubarte self-update`.** Releases up to 0.19.0 ask
  `api.github.com/repos/jandira-tech/jubarte-redlines/releases`. GitHub
  answers a renamed repository with a 301, and the updater's HTTP client
  (ureq, under `self_update`) follows redirects. Check it once after the
  rename by running `jubarte self-update --check` from an installed 0.18.0.
  If that fails, the old binaries cannot update themselves, and the release
  notes must tell users to reinstall with `scripts/install.sh`.
- **`git clone` and `cargo` git dependencies** on the old URL redirect.
- **Badges and links** on crates.io, PyPI and npm pages of already
  published versions redirect.

## 5. Other repositories (owner or a follow-up PR in each)

- `jandira-tech/jubarte-app`, `.github/workflows/ci.yml`: three `actions/checkout`
  steps use `repository: jandira-tech/jubarte-redlines`. They redirect, but
  rename them, and the comment about the fine-grained PAT; the PAT is bound
  to the repository id and keeps its access. README links: 2.
- `jandira-tech/neurotic_docx_bench`: README tables, `AGENTS.md` (the
  release-asset source) and `data/facts.jsonl` (`engine.repo`) name the old
  URL. These are data and links; update them in one bench commit.

## 6. After the move

- `python3 scripts/repo_move.py --check` stays in the tree, so a reference
  added later on the old name can be caught. It is not a CI gate before the
  move, because before the move the old name is still correct.
- Update the agent instructions that name the repository: this
  repository's `AGENTS.md` (rewritten by the script) and
  `~/T/reconciliation_plan/GET_JUBARTE_RUST.md`.
