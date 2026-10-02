<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Handoff: Plan 3 Task 3 (S16, MCP server and Gemini CLI extension)

Branch: `adopt/s16-mcp` (created locally from `main` at `b420d64`; not yet on
origin before this handoff commit). Session ended before any implementation
was written. This file is the complete state; delete it before the PR merges.

No subagents were used. Everything below was done inline in one session.

## What was done

1. Plans extracted (not committed, they live on `origin/ccr-a92b0695-1adjlu`):
   `git show origin/ccr-a92b0695-1adjlu:docs/superpowers/plans/2026-10-02-provider-adoption-3-s14-s16.md`
   (Task 3 is lines 324-630) and the `-1-s01-s10.md` plan ("What was verified",
   "Gates for every task", lines 71-116).
2. The native extension was built once: `cd jubarte-python && uv run --with
   maturin maturin develop --release` exited 0 and left `jubarte-python/.venv`
   (untracked). Rebuild in a fresh container.
3. No source file was changed. No test was written. No commit of code exists.

## Facts verified live on 2026-10-02 (do not re-derive)

### MCP Python SDK 2.2.0 (`uv run --with 'mcp>=2.2,<3'`)

- `from mcp.server import MCPServer`; `MCPServer(name, instructions=...)`.
- `@mcp.tool(name=None, title=None, description=None, annotations=None, ...)`.
- `ToolError` at `mcp.server.mcpserver.exceptions.ToolError`.
- `from mcp import Client`; `async with Client(server) as c` connects
  in-process. `await c.list_tools()` returns `.tools`, each with
  `.name`, `.description`, `.input_schema` (snake_case; `inputSchema` raises
  AttributeError), `.output_schema`.
- `await c.call_tool(name, arguments)` returns `CallToolResult` with
  `.is_error`, `.content` (list, `.text`), `.structured_content`.
- `MCPServer.run(transport="stdio" | "sse" | "streamable-http")`.
- Return annotation matters: a bare `-> dict` yields NO `structured_content`
  (only JSON text in content). `-> dict[str, Any]` yields
  `structured_content` as the dict. `-> str` yields
  `{"result": "..."}`; `-> list[dict]` yields `{"result": [...]}`.
  The plan's test asserts `first.structured_content["clean"]`, so every
  dict-returning tool must be annotated `dict[str, Any]`.
- A `ToolError("msg")` arrives as `is_error=True` with content text
  `"Error executing tool NAME: msg"`. The plan's tests check substrings
  (`"outside root"`, `"overwrite"`), which pass.
- Any other exception is swallowed into `"Error executing tool NAME"` with the
  message hidden and a traceback logged. Therefore wrap `JubarteError`,
  `EditPlanError`, `OSError`, `ValueError` into `ToolError` inside each tool.
- `mcp.types.ToolAnnotations(readOnlyHint=..., destructiveHint=...,
  idempotentHint=..., openWorldHint=...)` (camelCase constructor aliases;
  fields are snake_case). Use `readOnlyHint=True` on read tools; Codex's
  `default_tools_approval_mode = "writes"` prompts for tools not marked
  read-only.
- Tools run in a worker thread (anyio `to_thread`), so sync functions are fine
  and the engine's GIL release holds.

### Gemini CLI extension reference (geminicli.com/docs/extensions/reference, "Last updated May 14, 2026")

- Manifest `gemini-extension.json` at repo root: `name`, `version`,
  `description`, `mcpServers` ({name: {command, args, cwd}}, every MCP
  settings option except `trust`), `contextFileName` (defaults to `GEMINI.md`
  if present), `excludeTools`, `settings` (envVar allowlist), `migratedTo`,
  `plan`, `themes`.
- `name` lowercase/digits/dashes, expected to match the extension directory
  name. Installing from GitHub copies the repo, so `jubarte-redlines` fits.
- Variables: `${extensionPath}`, `${workspacePath}`, `${/}`.
- `skills/<name>/SKILL.md` is loaded as a skill (the existing
  `skills/jubarte-documents/SKILL.md` therefore becomes the extension skill).
- Env vars are sanitized: only HOME/PATH/TMPDIR-type vars plus those declared
  in `settings[].envVar` reach the MCP server. jubarte needs none.
- Install: `gemini extensions install <github url> [--ref] [--consent]`;
  `gemini extensions link <path>` for local dev; `/extensions list` inside
  the CLI. Banner: unpaid tier and Google One users were moved to
  "Antigravity CLI" on 2026-06-18.

### Claude Code (code.claude.com/docs/en/mcp)

- Project scope: `.mcp.json` at project root,
  `{"mcpServers": {"jubarte": {"type": "stdio", "command": "uvx", "args":
  [...]}}}`. An entry without `type` is read as stdio; `type: "stdio"` is
  explicit and safe.
- CLI: `claude mcp add --transport stdio jubarte --scope project -- uvx
  --from 'jubarte-redlines[mcp]' jubarte-mcp --root .`
- Claude Code sets `CLAUDE_PROJECT_DIR` in the server's environment and
  answers `roots/list`; `${CLAUDE_PROJECT_DIR:-.}` needs the default in
  `.mcp.json`. Implementing `roots/list` is optional and outside the plan.
- Tool descriptions and server instructions are truncated at 2,048 chars.

### Codex (learn.chatgpt.com/docs/extend/mcp, reached from developers.openai.com/codex/mcp)

- `~/.codex/config.toml` or project `.codex/config.toml` (trusted projects):
  `[mcp_servers.jubarte]` with `command`, `args`, `env`, `env_vars`, `cwd`,
  `startup_timeout_sec` (default 10), `tool_timeout_sec` (default 60),
  `default_tools_approval_mode` (`auto|prompt|writes|approve`).
- CLI: `codex mcp add jubarte -- uvx --from 'jubarte-redlines[mcp]'
  jubarte-mcp --root .`
- Codex reads the server `instructions`; keep the first 512 characters
  self-contained.

### Repository facts

- `Cargo.toml` version is `0.10.1`; the manifest version must equal it (the
  plan's test asserts equality), not the plan's illustrative `0.11.0`.
- `jubarte-python/python/jubarte_redlines/document.py`: `Document` has
  `inspect()`, `markdown()`, `edit()`, `preview()`, `render(pdf=, png_dpi=,
  options=)`, `to_png()`, `to_pdf()`, `compare(modified, author=, options=
  CompareOptions(date=))`, `accept(ids=, authors=, kinds=)`, `reject(...)`,
  `changes()`, `revisions()`, `sha256()`. No `inspect_json`, `validate`,
  `comments`, or `audit` (those are on sibling branches).
- `_native.inspect_json(bytes) -> str` exists (`_native.pyi` line 43);
  `Document.inspect_json()` is a one-liner returning it.
- `models.py`: `EditResult(clean, redline, report, diff)`; `EditReport` is a
  frozen dataclass with `_json` field (asdict includes `_json`; drop it or
  use `json.loads(report._json)`); `Rendered(pdf, pngs, report)`;
  `Change` fields: id, kind, target, author, date, text, move_name,
  move_side, inside; `ChangeKind = Literal["insertion","deletion","move",
  "formatting"]`; `EditOutcome` dataclass for `EditPlanError.outcomes`.
- `EditPlanError(code, message, operation, outcomes)` is a `JubarteError`.
- Test fixtures: `tests/test_document.py::make_document(text)` (minimal docx,
  importable as `from test_document import make_document` as the plan does)
  and `tests/docx_fixture.py` (`docx(body)`, `para(text)`).
- `jubarte-python/pyproject.toml`: dynamic version from Cargo.toml,
  `[project.scripts] jubarte-redlines = "jubarte_redlines.__main__:main"`.
  No `[project.optional-dependencies]` yet. `uv.lock` is 7 lines (package
  only); CI does not use `--locked`.
- `.github/workflows/ci.yml`: `convert-sweep-unit` job (line 148) runs
  `python3 scripts/test_*.py` scripts and sets up bun before
  `test_bump_version.py`. The python job (line 142) runs `uv run --with
  maturin maturin develop --release && uv run --with pytest pytest -q`
  from `jubarte-python`; it does not install `mcp`, so
  `pytest.importorskip("mcp")` in the new test keeps it green, and the task
  may add `--with 'mcp>=2.2,<3' --with anyio` there.
- `scripts/bump-version.mjs` rewrites Cargo.toml `version` and README pins;
  `scripts/test_bump_version.py` copies the script into a temp dir with a
  fake Cargo.toml and README and runs `bun`. bun 1.3.14, node 22, uv 0.8.17,
  cargo 1.97 are installed in this container.
- `REUSE.toml` annotation `path = "**"` covers files that cannot carry a
  header (gemini-extension.json). Markdown files carry an HTML-comment SPDX
  header; Python and JS files carry `# SPDX-...` / `// SPDX-...` headers.
- README root headings: Install > Python (line 214) then Node and browser
  (254); CLI reference from 287. CHANGELOG `## [Unreleased]` > `### Added`.
- `skills/jubarte-documents/SKILL.md` has a `## Dependencies` section
  (line 239) to extend with the MCP line.
- `docs/adoption/` does not exist; create it. Doc style: see
  `docs/MARKDOWN.md` (SPDX HTML comment, H1, tables).

## Design decisions taken (apply them)

- Module `jubarte-python/python/jubarte_redlines/mcp_server.py` with
  `build_server(*, root: Path) -> MCPServer` and `main(argv) -> int`
  (`--root`, default cwd; stdio only). Import `mcp` inside the module with a
  clear ImportError message naming `pip install 'jubarte-redlines[mcp]'`;
  `jubarte_redlines/__init__.py` must not import it.
- `contained(path)`: `Path(path).expanduser().resolve()` then
  `is_relative_to(root)` else `ToolError("... is outside root ...")`.
  `resolve()` follows symlinks, so a symlink escaping root is refused.
- `fresh(path, overwrite)`: refuse an existing file unless `overwrite=True`
  (message contains "overwrite"); mkdir parents.
- Twelve tools, exact names: docx_capabilities, docx_text, docx_inspect,
  docx_edit, docx_render, docx_compare, docx_changes, docx_accept,
  docx_reject, docx_validate, docx_comments, docx_audit. Arguments as the
  plan's table; add `overwrite: bool = False` to every writing tool
  (render, compare, accept, reject) because the plan's security model says no
  tool overwrites without it (record as a deviation from the argument table).
- `docx_validate(path, original=None, author=None)`, `docx_comments(path)`,
  `docx_audit(path, rules=None)`: registered now; body checks
  `hasattr(Document, "validate"|"comments"|"audit")` and raises
  `ToolError("this engine build lacks <feature>")` when absent, so the tool
  set contract test passes and the bodies are one-line swaps later.
- `docx_render`: writes `page-NN.png` under `out_dir` at `dpi`, optional
  `render.pdf` when `pdf=True`; `pages: list[int] | None` selects which
  1-based pages are written (the engine renders all pages regardless).
- `docx_edit`: exactly the plan sketch, with `EditPlanError` mapped to a
  ToolError whose message is the JSON payload (`code`, `operation`,
  `message`, `outcomes`), and `report` serialised via `json.loads(result.
  report._json)` rather than `asdict` so `_json` does not leak.
- Server `instructions`: first 512 chars self-contained (Codex), under
  2,048 total (Claude Code).
- `gemini-extension.json`: as the plan, version `0.10.1`,
  `args: ["--from", "jubarte-redlines[mcp]", "jubarte-mcp", "--root",
  "${workspacePath}"]`, `contextFileName: "GEMINI.md"`.
- `scripts/bump-version.mjs`: after the README block, read
  `gemini-extension.json` if present and replace `"version": "x.y.z"`;
  `scripts/test_bump_version.py` gains a test that the manifest follows the
  bump and one that a missing manifest is tolerated.
- `scripts/test_gemini_extension.py` (unittest, no deps): manifest parses,
  name matches `^[a-z0-9-]+$`, version equals Cargo.toml's, every
  `mcpServers` entry has `command` and a list `args`, no `trust` key,
  `contextFileName` file exists, `skills/jubarte-documents/SKILL.md` exists.
  Wire into `ci.yml` `convert-sweep-unit` next to `test_bump_version.py`.
- `docs/adoption/mcp.md`: tool table, security model, Claude Code
  (`.mcp.json` + `claude mcp add`), Codex (`config.toml` + `codex mcp add`),
  Gemini install, and an explicit "not verified here" list: the Gemini
  install transcript (no Gemini CLI in the container), the geminicli.com
  gallery process, the safe-docx licence and feature comparison (not
  fetched), `roots/list` support (not implemented).
- README root: add `### MCP server` under Install after "Node and browser";
  one-line install per host. `jubarte-python/README.md`: a short pointer under
  "Also available as". CHANGELOG: bullets under Unreleased > Added. Skill
  Dependencies: one line for `jubarte-redlines[mcp]` / `jubarte-mcp`.

## Gates to run before each commit

```bash
cd jubarte-python && uv run --with maturin maturin develop --release \
  && uv run --with pytest --with pytest-cov --with 'mcp>=2.2,<3' --with anyio \
     pytest -q --cov=jubarte_redlines --cov-branch --cov-report=term-missing
python3 scripts/test_gemini_extension.py
python3 scripts/test_bump_version.py
echo '{}' | timeout 5 uv run --with 'mcp>=2.2,<3' jubarte-mcp --root /tmp; echo $?   # from jubarte-python, no traceback
uv tool run --from 'reuse[charset-normalizer]' reuse lint
```

Red/green order: write `tests/test_mcp_server.py` first (expect
`ModuleNotFoundError: jubarte_redlines.mcp_server`), then implement.

## Finish

`git fetch origin main && git rebase origin/main`, rerun gates, commit with a
conventional message, `git push -u origin adopt/s16-mcp`, open a DRAFT PR
against `main` with `mcp__github__create_pull_request` whose body states the
plan ask, what was done, exact gate output with coverage, every deviation
(the `overwrite` argument on writers, `inspect_json` added, feature-missing
tools returning ToolError, manifest version 0.10.1), and what is undone.
Remove this handoff file in that PR.
