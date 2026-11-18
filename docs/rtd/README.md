<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# ReadTheDocs books

One ReadTheDocs project per workspace crate, all built from this
repository with mdBook:

| Project (slug) | Book | Build configuration file | Served at |
|---|---|---|---|
| `jubarte` (main) | `docs/rtd/jubarte/` | `docs/rtd/jubarte/.readthedocs.yaml` | `https://jubarte.readthedocs.io/en/latest/` |
| `jubarte-dom` | `docs/rtd/jubarte-dom/` | `docs/rtd/jubarte-dom/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-dom/en/latest/` |
| `jubarte-opc` | `docs/rtd/jubarte-opc/` | `docs/rtd/jubarte-opc/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-opc/en/latest/` |
| `jubarte-redlines` | `docs/rtd/jubarte-redlines/` | `docs/rtd/jubarte-redlines/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-redlines/en/latest/` |
| `jubarte-render` | `docs/rtd/jubarte-render/` | `docs/rtd/jubarte-render/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-render/en/latest/` |
| `jubarte-doc` | `docs/rtd/jubarte-doc/` | `docs/rtd/jubarte-doc/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-doc/en/latest/` |
| `jubarte-markdown` | `docs/rtd/jubarte-markdown/` | `docs/rtd/jubarte-markdown/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-markdown/en/latest/` |
| `jubarte-edit` | `docs/rtd/jubarte-edit/` | `docs/rtd/jubarte-edit/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-edit/en/latest/` |
| `jubarte-cli` | `docs/rtd/jubarte-cli/` | `docs/rtd/jubarte-cli/.readthedocs.yaml` | `https://jubarte.readthedocs.io/projects/jubarte-cli/en/latest/` |

`shared/` holds pages every book includes (the layer table). It is not a
project.

## How a book builds

Each `.readthedocs.yaml` replaces ReadTheDocs' build with one command,
`bash docs/rtd/build.sh PROJECT "$READTHEDOCS_OUTPUT/html"`. `build.sh`:

1. downloads the prebuilt mdBook release pinned in it (`MDBOOK_VERSION`,
   0.5.4) for the build machine, into `.scratch/rtd/mdbook-VERSION-TARGET/`,
   and checks the archive's sha256 against the value pinned beside it
   before unpacking it, every run;
2. reads the repository URL from `[workspace.package] repository` in the
   root `Cargo.toml` and passes it to mdBook as the book's GitHub link and
   "edit this page" link (`MDBOOK_OUTPUT__HTML__GIT_REPOSITORY_URL`,
   `MDBOOK_OUTPUT__HTML__EDIT_URL_TEMPLATE`). That line in `Cargo.toml` is
   the only copy: renaming the repository means changing it there, and
   nothing here;
3. builds the book and fails on any mdBook warning (mdBook reports a
   missing `{{#include}}` as a warning only).

Build a book locally with `docs/rtd/build.sh jubarte-render`; the HTML
lands in `.scratch/rtd/books/jubarte-render/`.

To move to another mdBook release, change `MDBOOK_VERSION` and the four
sha256 values in `build.sh` (Linux x86_64 and aarch64, macOS arm64 and
x86_64), each computed from the release's own archive.

## What the pages are made of

- Code samples are crate examples (`crates/<crate>/examples/*.rs`, and
  `examples/quickstart.rs` for the facade), pulled in with
  `{{#include FILE:example}}`. `cargo clippy --all-targets` compiles them,
  and each runs with `cargo run -p <crate> --example <name>`. The crates'
  `include` lists leave `examples/` out of the published packages.
- The CLI, Python and JavaScript references are the generated blocks of
  `docs/rust.md`, `docs/python.md` and `docs/javascript.md`, included by
  the `ANCHOR` comments around their `gen:` markers, so they stay as current
  as `scripts/gen_docs.sh --check` keeps them.
- The API reference stays on docs.rs; the books link to it.

`scripts/check_rtd.py` (with PyYAML) checks that every workspace member has a
project, every `.readthedocs.yaml` is valid and builds its own book, every
`SUMMARY.md` link resolves and every page is linked, every include and anchor
exists, and no file here names the repository's GitHub owner. The `rtd` job
of `.github/workflows/docs.yml` runs it and builds every book.

## Import checklist (readthedocs.org)

Do this once, signed in to <https://app.readthedocs.org> with the GitHub
account that administers the repository. Set up `jubarte` first and get one
green build before adding the rest.

### 1. Connect GitHub

1. Sign in with GitHub, and install the **Read the Docs** GitHub App on the
   organisation when asked, granting it the repository named by
   `[workspace.package] repository` in `Cargo.toml`.
2. With the GitHub App, ReadTheDocs subscribes to pushes and pull requests
   itself: no webhook is needed (see step 5 for the fallback).

### 2. Create the main project, `jubarte`

1. **Add project**, pick the repository, and name the project exactly
   `jubarte`. Check that the slug it shows is `jubarte`; if that slug is
   taken, stop: every crate README links to `jubarte.readthedocs.io`.
2. Default branch: `main`.
3. Open the project's **Settings** and set **Build configuration file** to
   `docs/rtd/jubarte/.readthedocs.yaml`. Save.
4. Build `latest` and open `https://jubarte.readthedocs.io/en/latest/`.

### 3. Create the eight crate projects

For each crate, in this order: `jubarte-dom`, `jubarte-opc`,
`jubarte-redlines`, `jubarte-render`, `jubarte-doc`, `jubarte-markdown`,
`jubarte-edit`, `jubarte-cli`:

1. **Add project** again, the same repository (if the list hides a
   repository that is already imported, use **Configure manually** with the
   repository's clone URL from `Cargo.toml`).
2. Name the project exactly the crate's name, for example `jubarte-dom`,
   and check the slug matches it.
3. **Settings** → **Build configuration file**:
   `docs/rtd/<crate>/.readthedocs.yaml`, for example
   `docs/rtd/jubarte-dom/.readthedocs.yaml`. Save.
4. Build `latest` once and confirm it passes.

### 4. Make them subprojects of `jubarte`

In the `jubarte` project: **Settings** → **Subprojects** → **Add
subproject**, once per crate:

- **Subproject**: the crate's project (`jubarte-dom`, ...);
- **Alias**: the crate's name, exactly (`jubarte-dom`, ...), so the URL is
  `https://jubarte.readthedocs.io/projects/jubarte-dom/en/latest/`, the one
  the READMEs link to.

You must be a maintainer of both projects. After this, the crate projects'
own `*.readthedocs.io` domains redirect to the `/projects/` URLs, and a
search on `jubarte` includes the crates.

### 5. Webhook (only without the GitHub App)

If a project was imported without the GitHub App, each of the nine projects
needs its own webhook:

1. In the project: **Settings** → **Integrations** → **Add integration** →
   **GitHub incoming webhook**. Copy its URL and secret.
2. On GitHub: repository **Settings** → **Webhooks** → **Add webhook**.
   Payload URL: the integration's URL; Content type: `application/json`;
   Secret: the integration's secret; events: **Let me select individual
   events** → Branch or tag creation, Branch or tag deletion, Pull requests,
   Pushes.
3. Push a commit and check the project's **Builds** page shows a triggered
   build.

### 6. After the repository rename

When the repository moves from its current name to `jandira-tech/jubarte`
(change `[workspace.package] repository` in `Cargo.toml` in the same PR),
the GitHub App follows the rename. In each of the nine projects, update the
repository URL under **Settings** if it still shows the old one. The books
pick the new URL up from `Cargo.toml` on their next build.
