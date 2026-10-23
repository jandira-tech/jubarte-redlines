# Threat Model

Jubarte is a local Word-document processing engine providing Rust byte APIs, a native CLI, PyO3/Python interfaces, WASM interfaces, an npm CLI, and a stdio MCP server. Document packages, XML, legacy DOC data, embedded media, Markdown and edit plans cross into parsers and transformations; results return as bytes or are published to host files. The MCP adapter grants a persistent directory capability for explicit document reads and output publication. Native rendering additionally consumes ambient font resources and maintains a font index. Optional self-update and maintainer publication workflows introduce separate network, executable replacement and registry authority. Sources: src/lib.rs:7-35, src/lib.rs:163-165, jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,373-381, jubarte-wasm/cli/bin/jubarte-redlines.mjs:16-21, src/update.rs:186-210. Architecture reflects inspected HEAD 4e10eca69c42211651ca907f38b74b6d19e6928c; architecture mapping is not completed audit coverage.

## Assets

- Confidential document text, comments, revisions, metadata, embedded media and derived reports/rendered pages. src/lib.rs:17-29; jubarte-python/python/jubarte_redlines/mcp_server.py:131-150,161-218,229-250.
- Original-document, source-snapshot and output integrity; optional source SHA-256 guard, staged byte verification and atomic per-file commits. src/edit.rs:97-106,2081-2088; src/publication.rs:168-174,535-569.
- Host filesystem authority: ambient CLI paths; native cap-std root capability for MCP explicit document I/O; native font resources/index outside that capability. src/publication.rs:153-165,203-205; mcp_server.py:72-108 under jubarte-python/python/jubarte_redlines; src/convert/font.rs:2633-2652,3095-3106,3213-3229.
- Host CPU/memory availability and browser WASM heaps; general and comparison admission budgets differ. src/admission.rs:63-99; jubarte-wasm/src/lib.rs:80-96; SECURITY.md:71-74.
- Executable and publication authority: current updater checksum-only; separate GitHub/crates.io/npm/PyPI credentials. src/update.rs:25-48,198-210,245-257; .github/workflows/release.yml:230-235,356-369; scripts/release.sh:919-920,934-955,1063-1069.

## Trust Boundaries

- Untrusted bytes enter public APIs and adapters. Admission controls defaults64MiB compressed,10000 entries,64MiB part,256MiB aggregate,depth256; compare permits512MiB compressed/part and2GiB aggregate. Input controls do not isolate the process. src/admission.rs:13-28,63-99.
- ZIP metadata passes regular open_archive/zip_defects refusal; unchecked opener is distinct and used for typed admission checks and maintainer debugging. src/opc/archive.rs:15-51.
- MCP operator sets --root (default cwd), expanded/resolved once; Python lexical-path diagnostics precede native persistent _OutputRoot reads/publication. Read/write tools share stdio without distinct caller roles; annotations are hints. jubarte-python/python/jubarte_redlines/mcp_server.py:54-55,67-108,126-159,373-381; jubarte-python/src/lib.rs:1244-1278; src/publication.rs:153-199.
- MCP root-relative writes reject nonportable names/output symlinks, stage every file, require overwrite=true for existing outputs, then atomically commit per file; later failure can leave prior commits. src/publication.rs:168-199,431-463,558-569.
- Native and npm CLIs intentionally use ambient host authority; npm node:fs reads/writes caller paths and checks existing outputs. src/publication.rs:203-205; src/bin/jubarte.rs:261-273; jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-79,106-110.
- Native rendering consumes ambient installed font files and independently derived font index via std::fs, outside explicit MCP root I/O. Browser WASM uses byte rendering; native system/cloud font overrides are no-ops. src/convert/font.rs:2633-2652,2855-2876,3095-3106,3213-3229; jubarte-wasm/src/lib.rs:231-235,262-273.
- Explicit self-update selects fixed GitHub owner/repo and platform release, delegates checksum/download/extraction/swap to self_update; RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None disable signature requirement. src/update.rs:25-48,120-125,198-221,234-257.
- Privileged release tag workflow builds with contents:read, publishes with contents:write; signing secret only enters its step, decodes under RUNNER_TEMP and EXIT trap deletes keys. Local release script separately publishes registries. .github/workflows/release.yml:15-21,51-55,230-235,261-309,356-369; scripts/release.sh:919-955,1063-1069.

## Attacker Capabilities

- Input suppliers control DOCX/ZIP/XML, embedded fonts/images, legacy DOC, Markdown or edit-plan data supplied to corresponding APIs; no operator environment, filesystem ownership or registry credentials assumed. src/lib.rs:17-30; SECURITY.md:36-40.
- An authorized MCP client chooses registered read/write operations, root-relative paths and overwrite flags, but native directory capability independently limits explicit path authority. jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,152-159; src/publication.rs:153-199.
- Concurrent filesystem writers matter only when deployment grants access to affected directories; publication anticipates replacement races using held handles and no-clobber hard links. src/publication.rs:146-165,558-569.
- Release-asset control can affect explicit checksum-only updates; requires publication compromise or defeat of download trust chain, not document input. src/update.rs:44-48,245-257; docs/SELF_UPDATE.md:59-80.
- Embedding hosts can choose exposed InputLimitOverrides; caller policy authority itself is not a bypass. src/admission.rs:103-108,137-163; jubarte-wasm/src/lib.rs:80-96.

## Security Objectives

- Reject unsafe package structures and bound ZIP/XML expansion with typed refusals before expensive work. src/admission.rs:13-28,63-99; src/opc/archive.rs:34-51.
- Keep explicit MCP document reads/publication within startup directory capability under path replacement; require overwrite for existing outputs. jubarte-python/python/jubarte_redlines/mcp_server.py:67-108; src/publication.rs:146-199,448-463,563-569.
- Preserve old outputs on staging failure, verify staged bytes, accurately report partial commits. src/publication.rs:168-174,535-569.
- Bind guarded edit plans to exact source SHA-256 when supplied; stale-source check is optional caller policy. src/edit.rs:97-106,2081-2088.
- Do not fetch external relationships or execute macros; host owns CPU/memory isolation and downstream-consumer safety. SECURITY.md:71-74; src/admission.rs:13-22; src/document_comparer.rs:100-108.
- Explicit updater invocation/confirmation/version decision precedes executable replacement; checksum configured, signatures required only when keys/threshold activate policy. src/update.rs:120-125,198-210,234-257,274-285.

## Assumptions

- No userContext, supplied threat model or KB. No remote HTTP exposure, multi-tenancy, or authentication assumed. MCP stdio: jubarte-python/python/jubarte_redlines/mcp_server.py:373-381; SECURITY.md:84-85.
- Actual target HEAD observed4e10eca69c42211651ca907f38b74b6d19e6928c differs from enclosing scan-directory hash; supplied current target is authoritative.
- MCP build_server says tools only touch files under root, but native font/index consumers have ambient access. Explicit I/O confinement is established; whole-process confinement is not. No arbitrary file capability inferred without additional attacker control. jubarte-python/python/jubarte_redlines/mcp_server.py:67-87,229-236; src/convert/font.rs:2633-2652,3095-3106,3213-3229.
- Current checksum-only updater agrees with staged signing documentation; zipsign support is not active signature enforcement. SECURITY.md:79-83; docs/SELF_UPDATE.md:59-80; src/update.rs:44-48,245-257.
- Library overview describes byte-in/out and no temporary files, but ambient font indexing uses a temporary save path. src/lib.rs:7-11; src/convert/font.rs:3095-3106,3213-3229.
- Admission is not absolute DoS protection; comparison2GiB inflation exceeds practical browser heap, wrapper advises lower limits. src/admission.rs:39-40,81-99; jubarte-wasm/src/lib.rs:80-85; SECURITY.md:71-74.
- Delegated self_update extraction, every generated bundle, every renderer/parser branch, and all privileged automation were not fully audited; unresolved coverage remains explicit.
- Verified native font-resource chain: JUBARTE_FONT_DIR if nonempty, else macOS HOME/Library/Application Support/jubarte/fonts; Linux XDG_DATA_HOME/jubarte/fonts or HOME/.local/share/jubarte/fonts; Windows APPDATA/jubarte/fonts plus WINDIR or SystemRoot or C:\Windows/Fonts and LOCALAPPDATA/Microsoft/Windows/Fonts. src/convert/font.rs:2633-2652,2855-2876.
- Verified font-index consumer: JUBARTE_FONT_INDEX if set; empty/off disables; otherwise parent(user_font_dir)/font-index.tsv. Temporary save uses with_extension(tsv.<PID>.tmp), fs::write and rename. Records family paths/metadata; ambient permissions enforce access. src/convert/font.rs:3095-3112,3195-3229,3264-3301.
- Verified MCP output consumer: root/out_dir/{clean.docx,redline.docx,patch.diff,report.json}, optional redline.pdf and redline-page-NN.png; native publisher stages private parent/.jubarte-UUID.partial/bytes, verifies bytes and commits atomically per file. jubarte-python/python/jubarte_redlines/mcp_server.py:152-218; src/publication.rs:431-569.
- Verified updater resource: jandira-tech/jubarte-redlines release, member jubarte-<version>-<target>/jubarte[.exe], targets macos/linux aarch64/x86_64 and windows-x86_64, SHA256SUMS.txt; dependency selects installed executable location, extracted_binary_is_regular_file hook guards entry type. src/update.rs:25-48,91-108,198-210,234-257,310-329.
- Verified signer resource: secret reference ZIPSIGN_PRIVATE_KEY -> RUNNER_TEMP/zipsign.<random>/keyN, 64-byte validation, optional keys/*.pub verification, EXIT cleanup; release publisher GH_TOKEN references secrets.GITHUB_TOKEN, contents:write publishes dist/* to GITHUB_REF_NAME in github.repository. .github/workflows/release.yml:261-309,317-322,356-369.
- Verified registry consumers: cargo publish --locked, npm publish from jubarte-wasm/npm then jubarte-wasm/cli with optional NPM_OTP, uv publish using UV_PUBLISH_TOKEN and dist/pypi/*; provider account authorization required. scripts/release.sh:913-920,928-958,1051-1069.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Scan scope: .

This is a provisional model saved before successful completion.
