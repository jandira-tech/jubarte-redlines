# Threat Model

Jubarte is an in-process Word document engine exposed through a Rust library and native CLI, PyO3 Python bindings and Python CLI, wasm-bindgen browser/Node bindings and a Node CLI, and an optional Python stdio MCP server. Untrusted DOCX packages pass through ZIP/package admission, XML parsing and document transformations before returning document bytes, extracted text, reports, PDF or PNG. Legacy DOC conversion follows a separate compound-file parser. Native publication, MCP filesystem confinement, ambient CLI writes, self-update, and privileged release/CI automation are distinct authority surfaces. The library forbids unsafe Rust, but process isolation and CPU/memory enforcement remain caller responsibilities. Evidence: src/lib.rs:5-30,85-86,96-165; jubarte-python/src/lib.rs:5-15; jubarte-wasm/src/lib.rs:7-28; jubarte-python/python/jubarte_redlines/mcp_server.py:373-381; src/legacy_doc.rs:170-215.

## Assets

- Confidential source documents and derived text, comments, authors, revisions, Markdown, redlines, PDF/PNG and reports. MCP returns extracted text/structured data over its client stream and publishes files under its granted root: jubarte-python/python/jubarte_redlines/mcp_server.py:126-150,185-205.
- Document integrity and correspondence between inspected snapshot, edit anchors, clean output and redline. Optional source_sha256 guards are not authentication: src/edit.rs:2081-2089. Schema/operation checks are distinct: src/edit.rs:1744-1761,2073-2100.
- Caller-owned files and output directories. Native CLI grants ambient parent authority; MCP grants persistent directory capability with separate read/publish: src/publication.rs:146-199,203-210; src/bin/jubarte.rs:261-273.
- Installed native executable and release integrity. Updater fixed repository jandira-tech/jubarte-redlines and SHA256SUMS.txt; RELEASE_KEYS empty and FIRST_SIGNED_RELEASE None: src/update.rs:25-48,198-210,245-257.
- Release authority: GITHUB_TOKEN, optional ZIPSIGN_PRIVATE_KEY, Cargo credential store/CARGO_REGISTRY_TOKEN, npm auth/NPM_OTP, UV_PUBLISH_TOKEN. Distinct publisher recipients: .github/workflows/release.yml:230-263,356-369; scripts/release.sh:374-380,904-939,1063-1069.
- Shared runner resources, build state and cache integrity. http://mbx-cache:7543 namespace jandira-tech/jubarte-redlines, no configured token: .github/actions/mbx/action.yml:8-22. Isolation external.
- Review credentials, comment/review authority and Jules spending. Distinct pinned actions with JULES_API_KEY: .github/workflows/jules-review.yml:19-50; .github/workflows/jules-el-conejo-review.yml:31-54,262-324.

## Trust Boundaries

- Document supplier -> package/parser process: ZIP metadata, entries, XML, relationships, embedded media. Admission checks preindex size/entries, inflation and zip defects. Default 64 MiB compressed/per part, 256 MiB total,10000 entries,depth256; compare/render/WmlDocument 512 MiB compressed/per part,2 GiB total,same depth/entries. src/admission.rs:63-100,397-425,439-470; src/opc/archive.rs:34-51; src/inspect.rs:605-615; src/document_comparer.rs:4926-4927; src/convert/mod.rs:773.
- Raw XML -> DOM: checked APIs enforce InputLimits/tag policy; infallible parse_xdocument is permissive. Defaults 1024 attrs,1024-byte names,64 MiB values,128 MiB start tags. src/xmllinq/parse.rs:19-36,48-57,98-129,910-919; SECURITY.md:43-60.
- Legacy DOC -> compound-file parser: streams/FIB-selected tables -> text/style/list; distinct from DOCX ZIP admission and needs separate resource assessment: src/legacy_doc.rs:170-215,225-236.
- Agent/edit plan -> mutation: schema,optional source hash,author,anchors checked before outputs. Snapshot guard conditional. Plan refusal distinct from publication atomicity: src/edit.rs:1740-1761,2073-2100; src/publication.rs:83-131.
- MCP client -> host files: --root defaults cwd, expanded/resolved once, must directory. Path diagnostics plus actual native _OutputRoot backed cap_std directory. Stdio tools confer root-scoped authority; annotations not per-user authorization. jubarte-python/python/jubarte_redlines/mcp_server.py:54-55,67-108,373-381; jubarte-python/src/lib.rs:1244-1277; src/publication.rs:146-199.
- MCP artifacts: <root>/<out_dir>/{clean.docx,redline.docx,patch.diff,report.json}, optional redline.pdf,redline-page-NN.png. Capability-relative parents,portable names,output symlink refusal; staging <destination parent>/.jubarte-<UUID>.partial/bytes; byte readback; overwrite=false atomic hard link,overwrite=true rename; per-file partial commits reported. jubarte-python/python/jubarte_redlines/mcp_server.py:185-205; src/publication.rs:83-131,168-199,419-505,535-570.
- Native CLI -> ambient selected parents via publish_paths; differs from MCP. Ordinary Python/Node CLI helpers directly Path.write_bytes/fs.writeFileSync after prechecks; common grammar does not prove shared write controls. src/bin/jubarte.rs:261-273; src/publication.rs:203-210; jubarte-python/python/jubarte_redlines/__main__.py:101-133; jubarte-wasm/cli/bin/jubarte-redlines.mjs:76-79,106-110.
- Markdown -> native image loader: --resource-path else Markdown parent else '.'; percent decode,scheme refusal,canonical containment and actual capability-relative read. Explicit image embedding authority,not external relationship fetching: src/bin/jubarte.rs:2811-2870,2936-2943.
- Document font request -> renderer-owned local font access: JUBARTE_FONT_DIR overrides; macOS HOME/Library/Application Support/jubarte/fonts; Linux XDG_DATA_HOME/jubarte/fonts or HOME/.local/share/jubarte/fonts, requires HOME before XDG lookup; Windows APPDATA/jubarte/fonts. Installed Windows fonts WINDIR else SystemRoot else C:\Windows/Fonts plus LOCALAPPDATA/Microsoft/Windows/Fonts. Native catalogue includes /System/Library/Fonts/Supplemental,/Library/Fonts,/Applications/Microsoft Word.app/Contents/Resources/DFonts,/Library/Fonts/Microsoft,/System/Library/Fonts and HOME/Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts. Readers renderer/font parser and generated PDF/subsets; controls operator env/host permissions/family matching, not MCP root. src/convert/font.rs:2228-2232,2258-2315,2427-2466,2633-2652,2831-2862.
- WASM host -> byte engine: Uint8Array/JSON same core; native publication excluded wasm32. Browser hosts advised lower compare defaults since 2GiB inflated exceeds practical heap. Node file authority JS host not WASM API: src/lib.rs:163-165; jubarte-wasm/src/lib.rs:7-28,63-95; jubarte-wasm/cli/bin/jubarte-redlines.mjs:16-21,52-55.
- Self-update -> fixed GitHub release -> jubarte-<version>-<target>/jubarte[.exe] -> installed executable. macos/linux aarch64/x86_64 tar.gz; windows-x86_64 ZIP. Noninteractive requires --yes or --check; --check no replacement; explicit --version allows downgrade. SHA256SUMS.txt and regular-file verify hook; signing currently inactive. Bindings disable updater default features. src/update.rs:25-48,91-108,120-125,144-150,198-257,299-315; jubarte-python/Cargo.toml:35-39; jubarte-wasm/Cargo.toml:18-20.
- Release tag/operator -> public supply chain: v* -> hosted builds -> contents:write release job; step-scoped ZIPSIGN_PRIVATE_KEY base64 -> RUNNER_TEMP/zipsign.<random>/keyN -> zipsign -> trap cleanup,64-byte key check,committed public-key verification; checksums then gh release create GH_TOKEN=GITHUB_TOKEN. Local script separately Cargo auth -> crates.io jubarte-redlines@$VER; npm auth/NPM_OTP -> jubarte-wasm/jubarte-redlines; UV_PUBLISH_TOKEN -> dist/pypi/* to PyPI with wheel inventory or explicit --no-wait partial fallback. Local artifact recovery downloads completed workflow artifacts to dist/release,creates sdist/checksums and gh release create $TAG without signing step. .github/workflows/release.yml:15-21,40-49,51-89,230-308,317-321,356-369; scripts/release.sh:374-380,876-939,976-1012,1028-1032,1039-1069.
- PR -> internal runner/cache: PR CI/adoption/docs/roundtrip use self-hosted; contents:read and no persisted checkout credentials. Composite mbx remote http://mbx-cache:7543 namespace jandira-tech/jubarte-redlines,cache-links=false,MBX_GC_MAX_TOTAL_SIZE=20GiB. Comments claim private Docker network and protected-push-only publication,not enforced by repository server configuration. Network ACLs,upstream action enforcement and runner lifecycle external. .github/workflows/ci.yml:13-48; .github/workflows/adoption.yml:16-42,48-59; .github/workflows/docs.yml:76-99; .github/actions/mbx/action.yml:8-38.
- PR/dispatch -> review services: Jules Review same-repo non-Dependabot guard,runner var else ubuntu-latest,pinned action receives JULES_API_KEY/GITHUB_TOKEN,contents:read,pull-requests:write,checks:write,enable_approve=false. Conejo resolves/rejects forks,checks exact SHA,no persisted credentials,contents:read,pull-requests:write,issues:write; trusted user arthrod gates session; JULES_API_KEY reaches separate pinned action. External service/install authority absent. .github/workflows/jules-review.yml:19-50; .github/workflows/jules-el-conejo-review.yml:31-54,65-102,262-324.

## Attacker Capabilities

- Document supplier controls ZIP names,relationships,XML,embedded font/image,legacy DOC and extracted text,not trusted env/roots/release configuration or CI credentials.
- MCP stdio client can request documented root reads/mutations and explicit overwrite; no remote HTTP or multi-tenant deployment established. Outside-root authority would be new; authorized root access expected.
- Markdown supplier controls resource names relative to independently selected root,not --resource-path. Scheme/confinement controls prevent beyond-root fetch/read.
- PR author supplies build/test/docs code run by applicable jobs subject to actual GitHub approvals. Approval bypass or actual run permission is not assumed.
- Release-asset-only attacker can replace archive and adjacent checksum in current checksum-only model; no operator/signing-key compromise assumed,explicit updater invocation still required.
- Developer benchmark stdin/stdout client supplies local compare paths to existing process authority; no authenticated production server: jubarte-rust-inproc/src/main.rs:5-20,91-98.

## Security Objectives

- Bound indexing,inflation and XML construction under entrypoint-selected budgets; ZIP header/extent/type checks; compare defaults differ from agent defaults: src/admission.rs:63-100,397-470; src/opc/archive.rs:34-51.
- Keep checked/permissive XML APIs distinct; validate untrusted XML; namespace/resource checks not full Word schema: SECURITY.md:43-65; src/xmllinq/parse.rs:98-129,910-919.
- No document-driven external relationship fetch; native Markdown images confined to explicit capability,not macro execution or Word launch: src/admission.rs:20-22; src/bin/jubarte.rs:2811-2870.
- MCP actual I/O stays in root; overwrite opt-in,output symlinks refused,partial per-file commits accurately reported: src/publication.rs:83-131,146-199,419-505,535-570.
- Optional source_sha256 binds exact edit snapshot; invalid/unresolved plans refused before outputs,not authentication/approval: src/edit.rs:1740-1761,2073-2100.
- Native install only explicit updater,confirmation/checksum/regular-file/conditional signatures. Current releases checksum-only until keys and first signed version: src/update.rs:44-48,120-125,198-257; docs/SELF_UPDATE.md:59-80.
- Scope independent publisher credentials to consumers and preserve account/package/tag/artifact binding; architecture mapping does not prove release execution/registry account setup.
- Protect shared runners/cache and Jules spend with effective external authorization/isolation; token permissions/action inputs alone do not prove external controls.
- Scrub and redact exports must remove requested identifying/confidential data from returned document bytes, not merely visible body text: src/scrub.rs:5-27; jubarte-python/python/jubarte_redlines/models.py:826-833.

## Assumptions

- Current checkout HEAD verified 4e10eca69c42211651ca907f38b74b6d19e6928c; historical artifact label not inspected. Static/read-only/offline; no application/test/exploit/network execution.
- Architecture mapping is not fully audited coverage. Large parser/render/transformation files sampled only for boundaries unless separately audited.
- Root SECURITY.md inherited; no nested policy returned. Process CPU/memory limits caller-owned,not engine-enforced: SECURITY.md:65-68.
- No supplied hosting,tenants,remote MCP,auth,container mounts or ACLs; callers choose identity/root.
- MCP invalid-plan promise and publisher per-file atomicity are distinct; I/O failure may leave reported earlier commits: mcp_server.py:161-168 (full path jubarte-python/python/jubarte_redlines/mcp_server.py); src/publication.rs:5-7,168-174.
- Native staged publication differs from ordinary Python/Node direct helpers despite CLI parity: src/bin/jubarte.rs:261-273; jubarte-python/python/jubarte_redlines/__main__.py:126-133; jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-110.
- docs/SELF_UPDATE.md:22-24 claims GH_TOKEN/GITHUB_TOKEN raises API budget; src/update.rs:198-210 never explicitly configures token/env auth. Actual dependency default unresolved within authorized source; no credential recipient inferred from docs.
- Signing inactive agrees with staged documentation,configuration limitation not independently reported: src/update.rs:29-48; docs/SELF_UPDATE.md:59-80.
- Future signing-path discrepancy: normal release workflow signs and fails missing secret once keys exist; local artifact recovery checksums/publishes without signing. Current keys empty; already-signed artifact handling and future impact unresolved: .github/workflows/release.yml:252-308; scripts/release.sh:976-1012.
- MCP root only path arguments; renderer additionally reads installed/configured fonts. Not process-wide sandbox: jubarte-python/python/jubarte_redlines/mcp_server.py:67-108; src/convert/font.rs:2258-2315,2633-2652.
- Cache comments claim runner-only private network/protected-push publication; source proves endpoint/namespace/no token config,not ACLs/upstream implementation: .github/actions/mbx/action.yml:8-22.
- External actions,Jules service,GitHub approvals,registry account policies,cache storage/authorization and runner reuse absent from repository. Remain conditional.
- No credential store contents or secret environment values read. All credential material described by references only.
- userContext was exactly null. No user-supplied threat model/knowledge base; fresh independent architecture review obtained and material resource consumers verified by parent source excerpts. Findings will reconcile model scenarios through coverage.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Model scope description: Current working-tree architecture of /Users/arthrod/temp/T/jubarte-redlines, inspected read-only. HEAD resolved to 4e10eca69c42211651ca907f38b74b6d19e6928c; independent reviewer reported clean status. Historical artifact directory label is not the authorized revision.
- Scan scope: .

This is a provisional model saved before successful completion.
