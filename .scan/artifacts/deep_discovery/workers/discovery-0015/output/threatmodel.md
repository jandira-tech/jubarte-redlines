# Threat Model

Jubarte is an in-process Word document engine exposed as a Rust library/native CLI, PyO3 Python library/CLI and stdio MCP server, and wasm-bindgen Node/browser APIs with a Node CLI. It accepts DOCX, supported legacy DOC and Markdown, then compares, inspects, edits, resolves revisions, renders and exports byte documents (README.md:29-42; src/lib.rs:15-30; jubarte-python/pyproject.toml:43-49; jubarte-wasm/src/lib.rs:7-28). Native hosts additionally provide filesystem authority, font discovery/cache persistence and publication. Self-update downloads fixed GitHub release assets and replaces the executable (src/update.rs:186-258); tag CI and operator release scripts publish distributions using separate credentials (.github/workflows/release.yml:15-21,230-235,356-369; scripts/release.sh:907-959). No deployment context or supplied authoritative model exists.

## Assets

- Document contents, comments, authorship, revision history, media and exported Markdown/JSON/PDF/PNG, including confidential views/reports (README.md:32-42; jubarte-python/python/jubarte_redlines/mcp_server.py:131-150,185-218).
- Source-bound edit integrity: legacy plans optionally guard source SHA-256; V2 requires matching source.sha256 before resolution (src/edit.rs:2081-2089; src/edit/v2_apply.rs:84-111).
- Native filesystem and output integrity: MCP explicit document/output I/O uses held OutputRoot; native CLI publication grants ambient selected local paths (jubarte-python/src/lib.rs:1244-1278; src/publication.rs:146-218).
- Embedding application/browser/CLI CPU and memory: default admission 64 MiB compressed/per-part, 256 MiB inflated, 10,000 entries and depth 256; comparison 512 MiB compressed/per-part, 2 GiB inflated with same entry/depth caps (src/admission.rs:63-99).
- Native font files and persistent font-index paths/stamps, whose host-environment-derived location and temporary sibling are independent of document output roots (src/convert/font.rs:2637-2652,3100-3106,3154-3168,3183-3229).
- Installed executable, release assets, crate/npm/PyPI distributions and conditional signing-key authority (src/update.rs:25-48,198-258; .github/workflows/release.yml:261-310,356-369; scripts/release.sh:907-959,1042-1067).

## Trust Boundaries

- Untrusted document producer -> in-process parser/comparer/renderer. Admission checks package budgets; shared production archive open checks structural defects. Debug deliberately uses unchecked archive open for triage. Engine controls differ from host CPU/memory obligations (src/admission.rs:13-28,63-99; src/opc/archive.rs:15-51; src/convert/mod.rs:769-778).
- Edit-plan producer -> mutation: V2 validates plan, required hash, scope/context and maps operations; legacy source binding is conditional on source_sha256 being present (src/edit/v2_apply.rs:84-111; src/edit.rs:2073-2094).
- MCP client -> operator-granted root. --root or cwd resolves and opens native OutputRoot once. Inputs/outputs pass relative names through held cap_std handles; Python containment is diagnostic. ToolAnnotations are hints, not separate caller authorization/approval (jubarte-python/python/jubarte_redlines/mcp_server.py:54-108,152-159,373-381; jubarte-python/src/lib.rs:1244-1278; src/publication.rs:146-199).
- MCP/native rendering -> ambient fonts/cache: docx_render and optional edit rendering load font resources outside explicit document/output capability. Index path is JUBARTE_FONT_INDEX (empty/off disables), else parent(user_font_dir)/font-index.tsv; user_font_dir is JUBARTE_FONT_DIR when nonempty, else macOS $HOME/Library/Application Support/jubarte/fonts, Windows $APPDATA/jubarte/fonts, Linux $XDG_DATA_HOME/jubarte/fonts or $HOME/.local/share/jubarte/fonts. Temporary sibling replaces extension with tsv.<pid>.tmp. Readers are later native processes/local filesystem users; control is host permissions and index version/stamps, not MCP OutputRoot (jubarte-python/python/jubarte_redlines/mcp_server.py:185-191,234-255; src/convert/font.rs:2637-2652,3100-3112,3154-3168,3183-3229).
- Native renderer -> system/Word font discovery: macOS /System/Library/Fonts[/Supplemental], /Library/Fonts[/Microsoft], /Applications/Microsoft Word.app/Contents/Resources/DFonts; Windows WINDIR/SystemRoot/default C:\Windows plus Fonts and LOCALAPPDATA/Microsoft/Windows/Fonts; macOS Word cloud $HOME/Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/<family> with one-normal-component family constraint. Host environment/permissions own these resources; font bytes reach parser/shaper/outputs (src/convert/font.rs:2228-2235,2281-2315,2855-2862).
- Local CLI operator -> ambient files: Rust opens selected parent once, stages .jubarte-<UUID>.partial with Unix mode 0700, verifies all bytes and commits per file. Python/Node wrappers directly read/write selected paths under OS authority; no MCP confinement is implied (src/bin/jubarte.rs:261-273; src/publication.rs:100-130,203-218,440-495; jubarte-python/python/jubarte_redlines/__main__.py:61-89,101-133; jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-79,106-111).
- MCP exports -> root filesystem readers: edit writes fixed clean.docx/redline.docx/patch.diff/report.json plus optional redline.pdf/redline-page-NN.png below out_dir; render writes page-NN.png and optional render.pdf. overwrite defaults false, native OutputRoot validates names/stages all bytes, but each file commits independently. Responses expose paths/report/text/font information, not document bytes as export payloads (jubarte-python/python/jubarte_redlines/mcp_server.py:152-255; src/publication.rs:100-130,175-199).
- Markdown image references -> native local reads. --resource-path wins, else Markdown parent or '.', canonicalized and held as directory capability; percent-decoding precedes URL rejection, containment and capability-relative reads. Image bytes enter generated DOCX/output. Python/Node reject unsupported resource-path rather than inheriting native resource authority (src/bin/jubarte.rs:2835-2876,2936-2943; jubarte-python/python/jubarte_redlines/__main__.py:1058-1062; jubarte-wasm/cli/bin/jubarte-redlines.mjs:724-726).
- Browser/Node caller -> WASM byte API: host supplies/receives bytes or JSON, native publication is excluded on wasm32; Node CLI separately adds filesystem authority (jubarte-wasm/src/lib.rs:7-28; src/lib.rs:163-165; jubarte-wasm/cli/bin/jubarte-redlines.mjs:16-21,52-79,106-111).
- GitHub release authority -> installed executable. Explicit self-update selects fixed jandira-tech/jubarte-redlines and platform, latest/requested version, binary member jubarte-<version>-<platform>/jubarte[.exe], SHA256SUMS.txt, confirmation/--yes and regular-file verifier. Current RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None mean checksum-only; signatures are conditional support (src/update.rs:25-48,83-108,186-258,299-310; docs/SELF_UPDATE.md:59-80).
- Maintainer/tag authority -> publication. v* CI checks version/tag match, read-only build jobs, contents:write release job. Step-scoped ZIPSIGN_PRIVATE_KEY is decoded to $RUNNER_TEMP/zipsign.<random>/keyN, cleanup trap, 64-byte length/committed-public-key checks, signatures before checksums. Missing secret fails only if public keys exist. GH_TOKEN references GITHUB_TOKEN for publishing dist/* to github.repository release ( .github/workflows/release.yml:15-21,40-49,230-321,356-369).
- Operator registry authority -> crates.io jubarte-redlines, npm jubarte-wasm then jubarte-redlines, PyPI dist/pypi wheel/sdist sets. Local registry credentials, optional npm OTP, packaged-summary/complete-wheel-set checks apply; --no-wait explicitly consents to local partial platform set (scripts/release.sh:907-959,1042-1067).
- Redaction producer -> distributed output recipient: untouched package parts retain original bytes, while the intended publication guard scans both clean/redline for redacted text. Admission supports UTF-16LE/BE XML but the leak scanner's XML branch uses lossy UTF-8, creating a different representation at the confidentiality boundary (src/edit.rs:4727-4746,5381-5386; src/edit/prepared.rs:798-808,873; src/admission.rs:777-781; src/scrub.rs:467-475).

## Attacker Capabilities

- Document suppliers control malformed DOCX/XML/media/fonts/Markdown and image references; host environment, root grants, release credentials and installation remain outside this capability (src/admission.rs:13-28; src/bin/jubarte.rs:2851-2876).
- MCP clients invoke exposed read/edit/render/compare/revision tools and may request overwrite=true under granted root; no independent caller role or human-approval gate is established (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,152-159,258-327).
- Library callers may override InputLimits, including raising them; this is already their embedding configuration authority (src/admission.rs:103-165).
- CLI/harness callers choose paths under existing OS authority; no new isolation is inferred (src/publication.rs:203-218; jubarte-rust-inproc/src/main.rs:91-99).
- Release asset control affects updater inputs; checksum-only verification depends on release hosting trust, distinct from document submission or compiled signing keys (docs/SELF_UPDATE.md:59-80; src/update.rs:25-48,245-257).

## Security Objectives

- Reject unsupported, inconsistent and over-budget packages before unrestricted downstream inflation/parsing, while retaining the diagnostic debug exception (src/admission.rs:13-28,63-99; src/opc/archive.rs:15-51).
- Confine explicit MCP reads/exports to startup directory capability, portable relative output names and overwrite policy (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108; src/publication.rs:146-199,440-463).
- Enforce promised source hash/scope/context binding and refuse invalid plans before document publication (src/edit/v2_apply.rs:84-111; src/edit.rs:2081-2089; jubarte-python/python/jubarte_redlines/mcp_server.py:170-217).
- Stage/verify all native outputs before first commit and report partial commits/durability failures honestly, without promising global atomicity (src/publication.rs:100-130,168-174).
- Keep Markdown images within native resource capability and prohibit URL fetches (src/bin/jubarte.rs:2835-2876,2936-2943).
- Reserve executable replacement for explicit update with fixed asset/checksum/configured-signature controls; scope signing/release secrets to privileged consumers and protect confidential reports (src/update.rs:25-48,186-258; .github/workflows/release.yml:261-310; SECURITY.md:12-35).

## Assumptions

- Exact userContext=null; no supplied model, source inventory or KB. Remote uploads, multi-tenant service and hostile MCP clients are deployment conditions, not established HTTP/authentication surfaces.
- Root SECURITY.md is the sole inherited security policy, covers engine/CLI/bindings, requires caller CPU/memory limits and distinguishes supported signatures from actual checksum-only releases (SECURITY.md:7-9,65-80).
- Documentation discrepancy: src/lib.rs:7-11 says byte APIs use no temporary files, but native font index persists/stages a sibling file; MCP build_server says tools only touch root yet transitive font operations use ambient paths (jubarte-python/python/jubarte_redlines/mcp_server.py:67-68; src/convert/font.rs:2637-2652,3100-3106,3183-3229). No attacker-controlled index path or exploitable cache disclosure is established.
- Current empty compiled keys/threshold agree with staged-signing docs; dependency/CI signature capability does not mean enforced signing (src/update.rs:44-48,245-249; docs/SELF_UPDATE.md:69-80).
- docs/SELF_UPDATE.md:22-24 advertises GH_TOKEN/GITHUB_TOKEN support but src/update.rs:198-210 does not explicitly configure a token; dependency credential lookup/recipient behavior remains unverified.
- Publication is per-file, not an atomic set; Windows directory sync is deliberately a no-op, without an established authorization failure (src/publication.rs:100-130,168-174,387-405).
- The benchmark worker is stdin/stdout developer tooling with ambient paths, not evidence of a network service (jubarte-rust-inproc/src/main.rs:5-20,91-99).
- Actual host/root/env permissions, deployed client exposure and registry/release approval settings are unknown. Review is offline static; no application execution, crafted-input execution, network or source changes.
- Source validation found suffix-selected auxiliary XML admission and unchecked post-HTML-expansion parsing, so package byte/depth admission must not be interpreted as universal downstream semantic-XML validation (src/admission.rs:534-569; src/document_comparer.rs:4724-4750; src/convert/mod.rs:789-798). Findings/coverage record each control outcome.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Model scope description: Repository-wide source-backed architecture mapping; architecture inspection does not establish completed audit coverage.
- Scan scope: .

This is a provisional model saved before successful completion.
