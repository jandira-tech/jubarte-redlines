# Threat Model

Jubarte is a local document-processing engine exposed through Rust APIs, a native CLI, Python bindings/CLI, WebAssembly packages and a Python stdio MCP server. It consumes untrusted document/archive/XML/media bytes and edit plans, returns transformed documents, views and renderings, and optionally publishes local files. Native CLI I/O has ambient filesystem authority; MCP document I/O uses a held directory capability. Native rendering also accesses ambient font resources and a persistent font index. Explicit self-update downloads and installs GitHub release binaries. Maintainer release workflows publish distributable artifacts. Evidence: src/lib.rs:86; Cargo.toml:83; jubarte-python/Cargo.toml:37; jubarte-wasm/cli/package.json:35; jubarte-python/python/jubarte_redlines/mcp_server.py:72; src/publication.rs:155; src/publication.rs:211; src/convert/font.rs:3100; src/update.rs:198; .github/workflows/release.yml:364.

## Assets

- Confidential document text, comments, revisions, embedded media and resulting DOCX/PDF/PNG/report artifacts. MCP edit output includes clean.docx, redline.docx, patch.diff and report.json beneath the selected out_dir: jubarte-python/python/jubarte_redlines/mcp_server.py:185; jubarte-python/python/jubarte_redlines/mcp_server.py:195.
- Host filesystem contents and existing outputs. MCP grants a directory capability once; CLI publication explicitly grants ambient local access: src/publication.rs:155; src/publication.rs:162; src/publication.rs:175; src/publication.rs:211.
- Host memory and CPU handling untrusted packages. Default admission caps compressed bytes at 64 MiB, entries at 10,000, individual inflated parts at 64 MiB, total inflation at 256 MiB and XML depth at 256. Comparison defaults are materially larger: 512 MiB compressed/per-part and 2 GiB total: src/admission.rs:63; src/admission.rs:87.
- Installed executable integrity, GitHub release artifacts, release signing authority and package publication authority: src/update.rs:198; src/update.rs:245; .github/workflows/release.yml:263; .github/workflows/release.yml:358; scripts/release.sh:919; scripts/release.sh:935; scripts/release.sh:952; scripts/release.sh:1069.
- Native renderer font files and persistent index. Font search uses configured/platform directories; default index is beside the fonts directory, not inside it, and records font file paths consumed on later conversions: src/convert/font.rs:2637; src/convert/font.rs:3100; src/convert/font.rs:3165.

## Trust Boundaries

- Untrusted document suppliers cross into in-process ZIP/XML/media parsing. Admission applies configured byte/count/depth budgets and bounded inflated reads; shared archive opening applies zip_defects before use. Direct unchecked archive opening is retained for diagnostic inspection: src/admission.rs:439; src/admission.rs:531; src/opc/archive.rs:20; src/opc/archive.rs:42.
- Public checked XML parsing validates before DOM construction; permissive parse_xdocument remains a different API whose caller must establish appropriate trust/admission. Tag defaults include 1,024 attributes, 1,024-byte qualified attribute names, 64 MiB attribute values and 128 MiB start tags: src/xmllinq/parse.rs:49; src/xmllinq/parse.rs:119; src/xmllinq/parse.rs:134.
- MCP caller arguments cross into host document reads and publication. Python resolves paths for early diagnostics, while native _OutputRoot performs capability-relative read/publish using the directory held by publication::OutputRoot. Tool annotations are hints to the host, not an approval or per-tool authorization mechanism: jubarte-python/python/jubarte_redlines/mcp_server.py:72; jubarte-python/python/jubarte_redlines/mcp_server.py:75; jubarte-python/python/jubarte_redlines/mcp_server.py:84; jubarte-python/python/jubarte_redlines/mcp_server.py:101; jubarte-python/src/lib.rs:1254; jubarte-python/src/lib.rs:1260; jubarte-python/src/lib.rs:1265; docs/adoption/mcp.md:39.
- Native CLI and Python CLI callers already possess ambient local path authority. CLI publication uses publish_paths; Python CLI reads caller-specified paths. Their authority must not be conflated with the MCP root capability: src/publication.rs:211; jubarte-python/python/jubarte_redlines/__main__.py:61; jubarte-python/python/jubarte_redlines/__main__.py:73.
- Rendering crosses from document-requested font families into ambient platform/configured font resources and cache writes. JUBARTE_FONT_DIR controls the font folder; JUBARTE_FONT_INDEX independently overrides or disables the index; absent override, the index is parent(font-folder)/font-index.tsv. These operations use std::fs rather than MCP OutputRoot: src/convert/font.rs:2637; src/convert/font.rs:3100; src/convert/font.rs:3165; src/convert/font.rs:3213.
- Explicit updater crosses from GitHub release authority into installed executable replacement. It selects a platform-specific archive/member, requests SHA256SUMS.txt verification and checks the extracted binary is a regular file. Signature verification is conditional, and this source has RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None: src/update.rs:25; src/update.rs:44; src/update.rs:48; src/update.rs:91; src/update.rs:104; src/update.rs:198; src/update.rs:245; src/update.rs:308.
- Tag-triggered CI builds cross into a privileged release job with contents:write. Only the signing step receives ZIPSIGN_PRIVATE_KEY, which is decoded into ephemeral runner files; the publication step receives GITHUB_TOKEN and uploads dist/* to the current repository release: .github/workflows/release.yml:15; .github/workflows/release.yml:20; .github/workflows/release.yml:230; .github/workflows/release.yml:234; .github/workflows/release.yml:263; .github/workflows/release.yml:278; .github/workflows/release.yml:285; .github/workflows/release.yml:358; .github/workflows/release.yml:364.

## Attacker Capabilities

- A document supplier can control input package structures, XML, media, embedded fonts and document content without already controlling the host filesystem, operator environment or release infrastructure: src/admission.rs:439; src/opc/archive.rs:42; src/xmllinq/parse.rs:134.
- An MCP client can supply tool arguments, document paths, edit plans, output directories and explicit overwrite flags within the granted root; it does not thereby receive ambient CLI authority. The server is started on stdio, without a network listener: jubarte-python/python/jubarte_redlines/mcp_server.py:75; jubarte-python/python/jubarte_redlines/mcp_server.py:152; jubarte-python/python/jubarte_redlines/mcp_server.py:373; jubarte-python/python/jubarte_redlines/mcp_server.py:381.
- An actor able to replace release archives and their adjacent checksum file has meaningful influence over checksum-only updater installations. Such release-write authority is a separate prerequisite, not an ordinary document supplier capability: src/update.rs:44; src/update.rs:48; src/update.rs:245; docs/SELF_UPDATE.md:59.

## Security Objectives

- Reject malformed or over-budget untrusted packages before expensive document processing; retain host-enforced process limits because admission budgets do not establish a universal CPU/memory bound: src/admission.rs:63; src/admission.rs:87; src/admission.rs:439; src/admission.rs:531.
- Confine MCP document reads/publication to the held root, preserve existing outputs unless replacement is explicitly authorized, and do not treat Python prechecks as the actual I/O authorization: src/publication.rs:155; src/publication.rs:162; src/publication.rs:175; jubarte-python/python/jubarte_redlines/mcp_server.py:75; jubarte-python/python/jubarte_redlines/mcp_server.py:101.
- Stage all output bytes before publishing and expose accurate partial-commit state if later commits fail. Publication is atomic per file, not across an output set: src/publication.rs:83; src/publication.rs:123.
- Keep update execution explicit and preserve executable integrity through release selection, checksums, configured signature policy and extracted-file type verification: src/update.rs:186; src/update.rs:198; src/update.rs:245; src/update.rs:255; src/update.rs:308.
- Keep release signing material confined to the signing step and distinguish artifact-build read permissions from release-publication write permissions: .github/workflows/release.yml:20; .github/workflows/release.yml:234; .github/workflows/release.yml:263; .github/workflows/release.yml:278.

## Assumptions

- No user deployment context or external authoritative threat model was supplied. This is a source-backed architecture map, not completed security-audit coverage or proof of deployment isolation.
- MCP documentation says outputs are under root, but native rendering also writes an ambient font index outside root under default platform configurations. The document-output capability is established; a claim that the whole renderer process touches only root is not: docs/adoption/mcp.md:61; docs/adoption/mcp.md:69; src/convert/font.rs:3100; src/convert/font.rs:3213.
- Signature support is implemented but not enabled by this source's empty key list and unset first-signed version. Documentation explicitly describes this staged checksum-only state; do not infer mandatory signatures from feature support: src/update.rs:44; src/update.rs:48; docs/SELF_UPDATE.md:69.
- Native rendering configuration and environment are operator-owned unless a particular embedding exposes them. Documents do not automatically gain control of JUBARTE_FONT_DIR or JUBARTE_FONT_INDEX: src/convert/font.rs:2637; src/convert/font.rs:3100.
- Browser/Node WebAssembly and native bindings must retain separate runtime authority assumptions. The npm CLI is a Node entrypoint backed by jubarte-wasm, while Python uses native Rust with the shared CLI feature and disables engine default features: jubarte-wasm/cli/package.json:35; jubarte-wasm/cli/package.json:46; jubarte-python/Cargo.toml:37.
- Registry credentials, GitHub repository permissions, host filesystem ACLs and actual signing-secret availability are deployment prerequisites not verified by offline source inspection: .github/workflows/release.yml:263; .github/workflows/release.yml:358; scripts/release.sh:919; scripts/release.sh:935; scripts/release.sh:1069.

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
