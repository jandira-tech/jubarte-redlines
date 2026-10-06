<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# MCP server: Claude Code, Codex and Gemini CLI

`jubarte-mcp` serves the Python `Document` API as Model Context Protocol tools
over stdio. It ships in the `jubarte-redlines` wheel behind the `mcp` extra,
so any host that can start a command can use it:

```sh
uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .
# or
pip install 'jubarte-redlines[mcp]' && jubarte-mcp --root .
```

## Tools

| Tool | Arguments | Returns |
| --- | --- | --- |
| `docx_capabilities` | | the engine's capability manifest |
| `docx_text` | `path` | Markdown with a `[body:p:N]` id before each paragraph |
| `docx_inspect` | `path` | the engine snapshot (`summary`, `paragraphs`, `stories`) |
| `docx_edit` | `path`, `plan`, `out_dir`, `pdf=false`, `png_dpi=null`, `overwrite=false` | paths of `clean.docx`, `redline.docx`, `patch.diff`, `report.json` (and `redline.pdf`, `redline-page-NN.png`), plus the report (the CLI writes the same report as `report.jsonl`) |
| `docx_render` | `path`, `out_dir`, `dpi=96`, `pages=null`, `pdf=false`, `overwrite=false` | paths of `page-NN.png` (and `render.pdf`), page count, page text, fonts |
| `docx_compare` | `original`, `modified`, `out`, `author`, `date=null`, `overwrite=false` | `out` and the change list |
| `docx_changes` | `path` | every tracked change, with the id accept and reject select by |
| `docx_accept`, `docx_reject` | `path`, `out`, `ids=null`, `authors=null`, `kinds=null`, `overwrite=false` | `out` and the number of changes left |
| `docx_validate` | `path`, `original=null`, `author=null` | findings |
| `docx_comments` | `path` | comment records |
| `docx_audit` | `path`, `rules=null` (rule sets `a11y`, `style`, `structure` or codes) | accessibility, style and structure findings, each with its paragraph id |

`docx_validate`, `docx_comments` and `docx_audit` are part of the tool
contract. An engine build without the matching `Document` method returns an
error saying the build lacks the feature.

Read tools carry the MCP `readOnlyHint`, so a host that asks before writes
(Codex's `default_tools_approval_mode = "writes"`) only asks for the tools
that write files.

A refused edit plan returns an error whose text holds the engine's JSON:
`code` (`ANCHOR_NOT_FOUND`, `STALE_SOURCE`, ...), `operation`, `message` and
every operation's `outcomes`. Nothing is written in that case.

## Security model

The server is a file-system tool that reads untrusted documents and untrusted
arguments from a model.

- Every path argument is resolved against `--root` (default: the current
  directory) after following symlinks. A path outside it, including a symlink
  that points outside it, is refused with "outside root".
- Every output file is resolved the same way before it is written, so an
  output name that is a symlink out of the root, dangling or not, is refused.
- No tool replaces an existing file unless `overwrite` is true. `docx_edit`
  and `docx_render` check every output before writing any.
- DPI is bounded by the engine; an out-of-range value is an error.
- Outputs (documents, PDFs, PNG pages) are written as files under the root
  and their paths returned, never their bytes.
- Document text is returned only by the tools whose job is to return it
  (`docx_text`, `docx_inspect`, `docx_changes`, `docx_comments`,
  `docx_render`'s page text).
- Every engine call goes through `Document`, which checks the package before
  parsing it.
- The server speaks stdio only. It opens no network port.

## Claude Code

Project scope, checked into the repository as `.mcp.json`:

```json
{
  "mcpServers": {
    "jubarte": {
      "type": "stdio",
      "command": "uvx",
      "args": ["--from", "jubarte-redlines[mcp]", "jubarte-mcp", "--root", "."]
    }
  }
}
```

Or from the command line:

```sh
claude mcp add --transport stdio jubarte --scope project -- \
  uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .
```

Shape checked against code.claude.com/docs/en/mcp on 2026-10-02.

## Codex

`~/.codex/config.toml`, or `.codex/config.toml` in a trusted project:

```toml
[mcp_servers.jubarte]
command = "uvx"
args = ["--from", "jubarte-redlines[mcp]", "jubarte-mcp", "--root", "."]
# The first run downloads the wheel; give it longer than the 10 s default.
startup_timeout_sec = 60
# Rendering a long document can pass the 60 s default.
tool_timeout_sec = 180
```

Or from the command line:

```sh
codex mcp add jubarte -- uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .
```

Shape checked against the Codex MCP documentation on 2026-10-02.

## Gemini CLI

The repository is a Gemini CLI extension: `gemini-extension.json` at its root
starts `jubarte-mcp` with `--root ${workspacePath}`, `GEMINI.md` is the
context file, and `skills/jubarte-documents/SKILL.md` is loaded as the
`jubarte-documents` skill.

```sh
gemini extensions install https://github.com/jandira-tech/jubarte-redlines
# for a local checkout
gemini extensions link .
```

Inside the CLI, `/extensions list` shows it. The manifest shape follows the
Gemini CLI extension reference (page dated 2026-05-14). The same page says
unpaid and Google One accounts moved to Antigravity CLI on 2026-06-18;
enterprise and API-key users stay on Gemini CLI.

Gemini CLI issue
[#20298](https://github.com/google-gemini/gemini-cli/issues/20298) (opened
2026-02-25) reported that Gemini CLI treats `.docx` as binary and proposed an
extension for reading and editing it; it was closed as not planned on
2026-05-08.

## Other docx MCP servers

[safe-docx](https://github.com/UseJunior/safe-docx), as its repository page
read on 2026-10-02 describes it:

| | jubarte-mcp | safe-docx |
| --- | --- | --- |
| License | AGPL-3.0-only | Apache-2.0 |
| Tracked-changes output | yes | yes ("clean or tracked-changes output") |
| Rendering to PDF or PNG | yes, its own layout engine | no ("not a visual editor or layout engine") |
| Gemini CLI extension | yes | yes (`gemini-extension.json` in the repository) |

## Not verified here

- A transcript of Gemini CLI installing the extension and reading a `.docx`
  through it. No Gemini CLI was available where this was written.
- The listing process for the gallery on geminicli.com/extensions.
- safe-docx's tool list, tracked-change fidelity and Word validity. Only its
  repository page was read.
- MCP `roots/list`. The server does not ask the host for roots; `--root` is
  the only boundary.
