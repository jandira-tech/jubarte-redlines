# Threat Model

Jubarte is a local Rust engine that compares, edits, inspects, repairs, scrubs, and renders Word documents. Native CLI, PyO3 Python APIs/CLI, WASM APIs/Node CLI and an optional Python stdio MCP server expose complete-buffer document processing (src/lib.rs:5-35; jubarte-python/pyproject.toml:42-49; jubarte-wasm/cli/package.json:33-46). Documents become in-memory OPC parts and XML trees. Native rendering separately reads fonts and persists an index. Explicit self-update, CI and release workflows have distinct executable, network and publication authority. This generated canonical model incorporates the independent architecture review.

## Assets

- Document contents, comments, revision authors, metadata, embedded media/fonts, generated DOCX/PDF/PNG, patches, snapshots and reports (src/lib.rs:15-35; jubarte-python/python/jubarte_redlines/mcp_server.py:185-217).
- Ambient host files accessible to local CLI processes; separately granted MCP root; staged outputs and destination integrity (src/publication.rs:146-225,421-569).
- Engine memory/CPU availability. Default admission allows64MiB compressed/per part,256MiB inflated,10000 entries, XML depth256; compare permits512MiB compressed/per part and2GiB inflated (src/admission.rs:63-100).
- Installed executable, release assets/checksums, optional signing keys and registry publishing credentials (src/update.rs:25-48,198-210; .github/workflows/release.yml:261-309,356-369; scripts/release.sh:374-380,919-959,1069).
- Native font files/index state: default index macOS $HOME/Library/Application Support/jubarte/font-index.tsv; Windows %APPDATA%/jubarte/font-index.tsv; Linux ${XDG_DATA_HOME:-$HOME/.local/share}/jubarte/font-index.tsv. JUBARTE_FONT_DIR changes default parent; JUBARTE_FONT_INDEX overrides or empty/off disables (src/convert/font.rs:2633-2652,3095-3107).
- Shared self-hosted CI runner files, reused checkout/disk state and private mbx-cache network ( .github/actions/mbx/action.yml:8-32; .github/workflows/ci.yml:13-48).

## Trust Boundaries

- Untrusted DOCX bytes cross admission/OPC parsing. Controls cover container budgets, duplicate identities, unsafe names, unsupported encryption/compression, header/extent defects, actual inflated streams and XML policy. Archive names are in-memory keys, not host extraction destinations (src/admission.rs:439-575; src/opc/mod.rs:282-379; src/opc/archive.rs:34-51). Validated entry-count and relationship-work findings qualify these controls.
- Admission differs by consumer: inspect/edit Opened uses default limits, comparison uses settings.input_limits, rendering calls compare-budget admit_package. Binding comparison overrides replace fields and reject unknown keys (src/inspect.rs:604-615; src/document_comparer.rs:4926-4927,6530-6567; src/convert/mod.rs:769-778; src/admission.rs:103-165; jubarte-python/src/lib.rs:65-79; jubarte-wasm/src/lib.rs:80-97).
- Checked XML validates before DOM creation: DTD/entity/structure and tag/namespace budgets. Public infallible parse_xdocument is explicitly unchecked. Defaults1024 attributes,1024-byte names,64MiB values,128MiB start tags; OPC metadata separately caps256 attributes (src/xmllinq/parse.rs:19-36,48-65,98-135,910-965; SECURITY.md:43-69).
- Legacy DOC is separate OLE parsing, not DOCX admission. It reads WordDocument and the FIB-selected table stream and bounds sector-chain traversal by file size (src/legacy_doc.rs:170-195,658-681,765-803).
- Edit instructions grant document mutation, not identity/authentication. Legacy source_sha256 is optional; PlanV2 hash match required; legacy editing refuses signed/macro-bearing packages. Native publication separately rechecks source/alias state (src/edit.rs:2072-2105; src/edit/v2_apply.rs:72-91; src/bin/jubarte.rs:1748-1824).
- Native CLI explicitly selected paths use ambient authority, held parents, .jubarte-<uuid>.partial/bytes staging, readback, rename overwrite or hard-link no-clobber. MCP OutputRoot uses held root authority, portable relative output names and symlink refusal. Sets commit per file; Windows directory sync is a no-op (src/publication.rs:146-225,228-273,390-411,421-569).
- Python/Node CLIs are separate writers: existence checks then Path.write_bytes/fs.writeFileSync; edit bundles directly write clean.docx,redline.docx,patch.diff, optional renders and report.jsonl in out_dir. They do not inherit native publication by sharing command grammar (jubarte-python/python/jubarte_redlines/__main__.py:101-133,478-517; jubarte-wasm/cli/bin/jubarte-redlines.mjs:76-109,517-552).
- MCP stdio clients control tool arguments; operator chooses --root or startup cwd, Gemini passes ${workspacePath}. build_server resolves once and opens _OutputRoot; actual document reads/publications are capability-relative, Python resolve is diagnostic. ToolAnnotations are hints, not caller auth; clients may request overwrite=true within root (jubarte-python/python/jubarte_redlines/mcp_server.py:54-108,152-160,258-277,373-381; jubarte-python/src/lib.rs:1244-1278; gemini-extension.json:5-9).
- Native Markdown image capability: --resource-path, otherwise input parent or '.', canonical root -> percent-decoded contained path -> OutputRoot read; schemes refused and unavailable images become alt text. Raw Rust defaults no loader; Python/WASM byte conversion supplies none (src/bin/jubarte.rs:2811-2878,2936-2943,3365-3375; src/markdown/mod.rs:159-170,196-198,216-229; jubarte-python/src/lib.rs:1200-1218; jubarte-wasm/src/lib.rs:1465-1488).
- Native rendering separately reads configured/per-user and fixed system/Word fonts. macOS cloud family must be one normal component; Windows uses WINDIR/SystemRoot/C:\Windows Fonts and LOCALAPPDATA Microsoft/Windows/Fonts. Index reads recorded face paths/stamps and writes a sibling tsv.<pid>.tmp then rename. WASM uses bundled/document fonts (src/convert/font.rs:2225-2236,2258-2315,2633-2652,2831-2862,3095-3107,3154-3189,3213-3229; README.md:901-920).
- Self-update is explicit native authority: fixed jandira-tech/jubarte-redlines release, platform archive jubarte-<version>-<target>/jubarte[.exe], checksum SHA256SUMS.txt, confirmation/--yes, --check lookup only. RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None mean checksum-only currently; verify_binary refuses extracted nonregular files (src/update.rs:25-48,91-108,120-125,186-259,274-285,299-321). Dependency default installation is current_exe; fonts are not installed (docs/SELF_UPDATE.md:36-45).
- Release CI is v* tag-triggered with read-only builds and contents:write final publication. ZIPSIGN_PRIVATE_KEY is a step-scoped reference decoded into $RUNNER_TEMP/zipsign.XXXXXX/keyN, length checked, archives signed/verified against committed public keys when present, and keys removed on exit. GH_TOKEN from secrets.GITHUB_TOKEN permits release create. Local release.sh separately consumes Cargo/npm/gh/UV_PUBLISH_TOKEN authority and confirms version/tag ( .github/workflows/release.yml:15-25,230-239,261-309,317-322,356-369; scripts/release.sh:360-389,870-901,919-959,1039-1069).
- Untrusted pull-request checkout code runs on self-hosted CI/Markdown-roundtrip/Adoption/Docs jobs. The local mbx action documents private tokenless http://mbx-cache:7543, namespace jandira-tech/jubarte-redlines, reused checkout/cache/disk state. Workflow token restrictions do not isolate OS/network capabilities ( .github/workflows/ci.yml:13-48,72-99,114-138,165-310; .github/workflows/md-roundtrip.yml:13-39; .github/workflows/adoption.yml:19-99; .github/workflows/docs.yml:42-99; .github/actions/mbx/action.yml:8-32).
- Unpublished jubarte-rust-inproc benchmark worker accepts COMPARE input/output paths over stdin and uses ambient fs reads/writes. It is a trusted harness tool, not MCP-root confined; untrusted protocol exposure is not established (jubarte-rust-inproc/Cargo.toml:5-18; jubarte-rust-inproc/src/main.rs:29-99).
- Rendering is a derived-representation boundary beyond archive admission: HTML/MHT expansion, table columns, numbering labels/levels, embedded-font ownership, WMF records and font/PDF/MATH parsing must independently bound work. Validated findings establish gaps; OS resource caps remain caller-owned (src/convert/mod.rs:769-808; src/convert/altchunk.rs:463-491; src/convert/pdf.rs:1434-1451,1531-1544).
- Scrub/redaction outputs cross a confidentiality boundary before sharing. Property scrub must remove requested metadata and residual-secret checks must interpret supported encodings before accepting output; alternate metadata names and UTF16 scanning gaps were validated (src/scrub.rs:357-384,463-475; src/edit.rs:4727-4746).

## Attacker Capabilities

- A document sender controls DOCX/DOC/XML/media/fonts, relationships, Markdown and edit-plan content supplied to a willing processor; this does not imply environment, CLI flags, host permissions, release assets or credential control.
- An MCP client can invoke registered tools, receive text/reports and request overwrite=true within the granted root; no separate tenant identity/read-only tier is established (jubarte-python/python/jubarte_redlines/mcp_server.py:54-108,126-160,258-277,373-381).
- A local CLI operator chooses ambient paths and resource roots. A different local writer of shared directory entries can redirect unsafe binding writes to files writable only by the operator; document-only control does not supply that local access (src/publication.rs:203-225; jubarte-python/python/jubarte_redlines/__main__.py:403-414).
- A PR contributor can alter proposed local action/build/test/scripts and gains runner OS/private-network authority if GitHub permits the self-hosted run. Approval and host isolation are deployment prerequisites, not proven absent.
- Release-channel control crosses an independent update-integrity boundary; checksums accompanying an archive are not independent publisher authentication. Host environment/index changes, tag-push authority, registry/signing secrets and external app checkout control require separate evidence (src/update.rs:44-48,245-257; docs/SELF_UPDATE.md:59-79).

## Security Objectives

- Enforce the same active archive interpretation/count and actual inflation limits before processing; bound derived parser representations and avoid archive extraction/macro execution (src/admission.rs:13-28,399-407,439-575; src/opc/archive.rs:34-51).
- Keep checked/unchecked XML contracts distinct and carry default/caller limits into every consuming representation (src/xmllinq/parse.rs:98-135; src/admission.rs:103-165).
- Confine MCP document I/O to held root capability during symlink/path changes; do not equate hints with authorization (src/publication.rs:146-199; jubarte-python/python/jubarte_redlines/mcp_server.py:54-108).
- Preserve destinations on staging failure, enforce atomic no-clobber/entry replacement and exact-byte staging; report partial commits/durability uncertainty. Apply equivalent controls to binding writers (src/publication.rs:29-46,168-174,421-569).
- Bind edits to intended snapshots where required, refuse unresolved edits and residual redaction content before publication, and keep in-memory validation separate from host commit checks (src/edit.rs:2081-2089,4727-4746; src/edit/v2_apply.rs:72-91; src/bin/jubarte.rs:1764-1824).
- Do not fetch external document relationships; native Markdown resource loading remains within explicitly selected authority, custom callbacks caller-owned (SECURITY.md:70-74; src/bin/jubarte.rs:2811-2878; src/markdown/mod.rs:159-170).
- Protect explicit executable updates with HTTPS, checksum/applicable signature policy and extracted-regular-file verification; protect release secrets and separate publication authority from untrusted PR/document authority (Cargo.toml:137-148; src/update.rs:120-125,198-210,245-257,299-321; .github/workflows/release.yml:261-309,356-369).
- Require caller CPU/memory containment; input and raster budgets are bounded controls, not comprehensive denial-of-service protection (SECURITY.md:70-74; src/convert/raster.rs:24-40).

## Assumptions

- Exact userContext is null, no supplied threat model or knowledge base; all reviewed source is current local state. Environment formulas are not observed live settings.
- MCP documentation says every path remains under --root (README.md:347-349; jubarte-python/python/jubarte_redlines/mcp_server.py:67-68), but font reads/index writes use separate native authority. Document confinement is not a process sandbox (src/convert/font.rs:2633-2652,3095-3107,3213-3229).
- Library no-temp-files description (src/lib.rs:8-11) needs qualification for enabled native font-index persistence (src/convert/font.rs:3213-3229).
- Filesystem publication docs:9-24 describe staged CLI writes/no direct fallback; native CLI matches, Python/Node helper writers do not (src/bin/jubarte.rs:261-273; jubarte-python/python/jubarte_redlines/__main__.py:101-133; jubarte-wasm/cli/bin/jubarte-redlines.mjs:76-109).
- Updater docs/SELF_UPDATE.md:22-24 advertise GH_TOKEN/GITHUB_TOKEN; repository builder sets neither token setter (src/update.rs:198-210). Independent architecture review inspected supporting self_update1.3.1 defaults as anonymous, but parent has not revalidated that dependency claim; retain as a documentation follow-up, not a vulnerability.
- SECURITY.md:89-95 names rdocx-opc0.2/quick-xml0.41; Cargo.toml:102-117 selects oxml-opc0.14.2 with crypto disabled, quick-xml0.42 and aliased0.41 for distinct consumers. Naming drift alone proves no advisory exposure.
- Empty signing keys/first-release flag deliberately match checksum-only rollout (src/update.rs:44-48,245-249; docs/SELF_UPDATE.md:67-79).
- Publication sets are per-file, not whole-set transactions; Windows parent sync is intentionally no-op. Native bundle publication has a distinct protocol (src/publication.rs:5-7,36-46,168-174,390-411; docs/FILESYSTEM_PUBLICATION.md:28-58).
- MCP SDK/host owns transport lifecycle/approval; only stdio is configured and no HTTP listener or host enforcement of readOnlyHint was established (jubarte-python/python/jubarte_redlines/mcp_server.py:26-28,54-55,373-381).
- Local release.sh delegates to ${JUBARTE_APP_DIR:-jubarte-app}/scripts/release-engine.sh after executable/preflight checks; downstream implementation, targets, accounts and approvals are outside selected source (scripts/release.sh:382-389).
- Conditional publisher resources: Cargo credential references at $HOME/.cargo/credentials.toml or legacy credentials/CARGO_REGISTRY_TOKEN -> configured registry; npm auth/optional NPM_OTP -> jubarte-wasm then jubarte-redlines; UV_PUBLISH_TOKEN -> dist/pypi/* -> configured index. Account/token/index settings not inspected (scripts/release.sh:360-380,919-959,1039-1069).
- PR approval settings and self-hosted lifecycle/extra sandboxing were not queried; no root/Docker-socket/release-key access or private-cache write protocol is established.
- Offline static review only. No application/tests, vulnerability inputs, credential use, external requests or Git history review. Generated WASM binaries, dependency internals beyond cited excerpts and the remaining file inventory are not exhaustively audited.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Model scope description: Current repository source; architecture mapping is broader than completed file-audit coverage.
- Scan scope: .

This is a provisional model saved before successful completion.
