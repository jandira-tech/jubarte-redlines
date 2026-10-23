# Threat Model

Generated source-backed model of current /Users/arthrod/temp/T/jubarte-redlines, scope ., observed live HEAD 4e10eca69c42211651ca907f38b74b6d19e6928c. Jubarte is a local Rust DOCX compare/edit/validate/inspect/scrub/render engine with legacy DOC and Markdown conversion; bytes flow through Rust APIs, PyO3, wasm-bindgen, native/Python/Node CLIs and optional stdio MCP (src/lib.rs:7-36; jubarte-python/src/lib.rs:5-15; jubarte-wasm/src/lib.rs:7-28; jubarte-python/python/jubarte_redlines/mcp_server.py:373-381). Conditional privileged workflows are explicit native self-update and maintainer release publication.

## Assets

- Document bytes/text/revisions/comments/authors/metadata, PDF/PNG output, edit reports; MCP read tools return content while edit/render publish derived files (jubarte-python/python/jubarte_redlines/mcp_server.py:131-150,185-218,229-255).
- Worker CPU/memory during ZIP/XML/comparison/image/font processing. InputLimits defaults 64 MiB compressed/per part, 256 MiB inflated, 10000 entries and depth 256; comparison uses 512 MiB compressed/per part and 2 GiB inflated (src/admission.rs:63-99).
- Document/export confinement under the operator-selected MCP root; native CLI has ambient local authority. Publication stages and verifies all files, then commits atomically per file, not across the set (src/publication.rs:146-225).
- Ambient font files, persistent font-index paths and sibling temporary cache files outside document roots (src/convert/font.rs:2258-2275,2637-2652,3100-3107,3183-3188,3213-3229).
- Installed executable integrity and release/package identities. Signing keys are secret references decoded into temporary runner files; publishing token is separate (.github/workflows/release.yml:261-309,356-369; src/update.rs:198-210).

## Trust Boundaries

- Document supplier -> in-process package admission -> XML/DOM/media consumers. Admission checks declared/streamed budgets, duplicate names and ZIP defects; comparison admits both active-setting inputs (src/admission.rs:439-470; src/document_comparer.rs:6530-6567). Ordinary budgets and comparison budgets differ (src/admission.rs:63-99).
- Raw XML caller -> checked try_parse_xdocument_with_limits versus intentionally permissive parse_xdocument. Checked parsing validates before construction; permissive parsing requires caller gating (src/xmllinq/parse.rs:84-135).
- MCP --root defaults cwd -> expanduser/resolve once -> native _OutputRoot -> retained cap-std directory. Consumer document read effective path <root>/<client-relative input>; output edit effective paths <root>/<out_dir>/{clean.docx,redline.docx,patch.diff,report.json,optional redline.pdf/redline-page-NN.png}; render outputs <root>/<out_dir>/{page-NN.png,optional render.pdf}. Python containment is diagnostic; native capability-relative reads/publish enforce actual I/O, overwrite is explicit (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,170-255,373-381; src/publication.rs:146-199).
- All connected stdio MCP callers share exposed tool authority. Read/write annotations are descriptive, not independent authentication or tool-level authorization (jubarte-python/python/jubarte_redlines/mcp_server.py:54-55,126-159,373-381).
- Native CLI explicit destinations -> publish_paths(root=None) -> ambient OS filesystem permissions, staged verified replacement. Node CLI explicit paths -> fs.readFileSync/writeFileSync with existsSync precheck; it does not inherit native publication guarantees (src/bin/jubarte.rs:261-273; src/publication.rs:203-225; jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-79,106-111,137-145).
- Native document font requests -> ambient lookup: nonempty JUBARTE_FONT_DIR overrides user directory; defaults macOS $HOME/Library/Application Support/jubarte/fonts, Linux ${XDG_DATA_HOME:-$HOME/.local/share}/jubarte/fonts, Windows $APPDATA/jubarte/fonts; installed/Word/cloud directories also searched. Recipients are native font parsers and renderer; OS permissions apply independently of MCP root (src/convert/font.rs:2258-2275,2281-2315,2637-2652).
- Native persistent index consumer: JUBARTE_FONT_INDEX explicit path wins; empty/off disables; absent -> parent(user_font_dir)/font-index.tsv. Effective defaults macOS $HOME/Library/Application Support/jubarte/font-index.tsv, Linux ${XDG_DATA_HOME:-$HOME/.local/share}/jubarte/font-index.tsv, Windows $APPDATA/jubarte/font-index.tsv; JUBARTE_FONT_DIR changes default to its parent's index. Cache is consumed by subsequent renderer processes under OS permissions; save writes sibling path.with_extension(tsv.<pid>.tmp), then renames (src/convert/font.rs:2637-2652,3100-3112,3154-3188,3192-3229).
- Explicit updater -> fixed jandira-tech/jubarte-redlines release and supported platform -> selected tag -> SHA256SUMS.txt -> archive child jubarte-<version>-<platform>-<arch>/jubarte[.exe] -> delegated executable install. Terminal/--yes preflight, check-only mode, checksum and regular-file hook enforce separate controls. RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None mean signatures are not currently required (src/update.rs:25-48,87-108,120-125,186-257,299-310).
- Tag-driven maintainer release -> five native targets -> dist/jubarte-*.tar.gz or .zip. secrets.ZIPSIGN_PRIVATE_KEY -> signing-step environment -> base64 decode -> $RUNNER_TEMP/zipsign.XXXXXX/keyN -> zipsign; 64-byte key check, cleanup trap and committed-key verification; optional secret only when no public keys committed. SHA256SUMS.txt generated after signing. secrets.GITHUB_TOKEN -> GH_TOKEN -> gh release create github.repository/GITHUB_REF_NAME with notes.md and dist/* (.github/workflows/release.yml:15-21,58-144,261-322,356-369).
- scripts/release.sh registry publication uses maintainer cargo/npm/uv credentials independently of document processing (scripts/release.sh:370-380,878-919,924-959,1069-1070).

## Attacker Capabilities

- Document adversary supplies ZIP entries, XML, relationship targets, images/fonts and content chosen for processing by a victim. No assumption of control over host environment, MCP root, outside filesystem, release accounts or signing keys.
- Connected MCP client invokes all granted tools and selects root-relative documents/outputs, including overwrite=true. Root selection is operator authority; no remotely authenticated or multi-tenant deployment is established (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,152-159,373-381).
- Document-only attacker must not gain arbitrary host file access, outbound fetching, executable installation or writes outside authorized destinations. Representative relationship consumers distinguish External targets from package bytes instead of fetching them; this is not exhaustive no-network proof (src/document_comparer.rs:100-108; src/opc/mod.rs:766-783).
- Release-asset adversary has a separate capability from a document supplier. Current checksum-only update depends on release-hosting and TLS integrity; signature separation from release-asset control is inactive (src/update.rs:44-48,245-257; docs/SELF_UPDATE.md:69-87).

## Security Objectives

- Enforce active archive and XML limits before expensive processing and require checked raw XML or prior admission (src/admission.rs:63-99,439-470; src/xmllinq/parse.rs:84-135).
- Keep MCP document reads and exports under retained directory capability, reject unsafe names/conflicts and accurately report partial commit/durability outcomes; retain distinction from native ambient CLI and Node direct writes (src/publication.rs:146-225; jubarte-wasm/cli/bin/jubarte-redlines.mjs:76-79,106-111).
- Require explicit updater/install authority, bind platform/version/member, checksum and extracted file type; communicate current absence of signature enforcement (src/update.rs:87-108,120-125,198-257,299-310).
- Constrain signing credentials to intended release step and avoid disclosure in artifacts/reports; actual hosted secret values and runner permissions are external facts (.github/workflows/release.yml:261-309,356-369).
- Treat admission budgets as controls rather than complete process isolation. Embedders choose tighter CPU/memory budgets; WASM documentation recommends lowering the 2 GiB comparison budget for browser hosts (jubarte-wasm/src/lib.rs:77-85; src/admission.rs:80-99).

## Assumptions

- Model reflects explicitly supplied live target, not revision in scan directory name. No deployment, tenant model, listener, container mapping or actual environment supplied; paths are symbolic code-derived values.
- Documentation disagreement retained: MCP says every path/output/tool operation is under --root (jubarte-python/python/jubarte_redlines/mcp_server.py:5-9,67-82), whereas native rendering reads ambient fonts and writes the independently configured index outside it (src/convert/font.rs:2258-2275,2637-2652,3100-3107,3183-3188,3213-3229). This qualifies global claim without itself proving arbitrary document-directed file access.
- Documentation disagreement retained: src/lib.rs:7-11 says no temp files; enabled native font-index persistence writes sibling tsv.<pid>.tmp then renames (src/convert/font.rs:3213-3229).
- docs/SELF_UPDATE.md:69-87 accurately documents current conditional-signing/checksum-only bootstrap. Feature support alone does not establish enforcement (src/update.rs:44-48,245-257).
- Documentation discrepancy resolved by focused offline source: docs/SELF_UPDATE.md:22-24 says GH_TOKEN/GITHUB_TOKEN lift rate limits, but src/update.rs:198-211 calls neither auth_token nor auth_token_from_env. Locked local self_update1.3.1 macros.rs:542-580 and backends/common.rs:555-575 require opt-in and default auth_token=None. Inspected updater remains unauthenticated; documentation mismatch, no independent security finding.
- Host OS permissions, cap-std platform internals, decoder internals, updater dependency transport/extraction, release job exact token scope and actual hosted secret state require separate verification where material. No runtime application execution, network requests or vulnerability-triggering inputs.
- Root SECURITY.md is analysis context, including external-relationship no-fetch claim, raw parser caller obligation and process memory/CPU recommendation. Architecture mapping alone is not completed security audit coverage.
- Validated scenarios are retained in findings; quadratic table-row matrix reachability remains a candidate in coverage.deferred. Whole-scope source review remains partial, with exact unreviewed paths retained. All three nested agents have returned.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: not recorded
- Model scope: not recorded
- Scan scope: .

This is a provisional model saved before successful completion.
