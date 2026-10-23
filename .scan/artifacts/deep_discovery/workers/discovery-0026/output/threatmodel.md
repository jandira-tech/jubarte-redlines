# Threat Model

Current authorized repository HEAD 4e10eca69c42211651ca907f38b74b6d19e6928c implements an in-process Rust DOCX comparison, edit, inspection and PDF/PNG rendering engine, exposed through native CLI, Python/PyO3, WASM Node/browser APIs and Node CLI. Python MCP serves document tools over stdio. Bindings disable engine defaults, enabling shared CLI grammar without native self-update. Native rendering additionally accesses host fonts/cache. Release and AI-review workflows exercise separate network/publication authority. No hosted document service or tenant deployment was established (src/lib.rs:7-30; jubarte-python/Cargo.toml:35-40; jubarte-wasm/Cargo.toml:18-32; jubarte-python/python/jubarte_redlines/mcp_server.py:373-381).

## Assets

- Confidential document bytes, revisions/deleted text, comments, metadata, images/fonts, projections and rendered outputs; caller memory, local files and MCP responses carry these assets (src/lib.rs:17-30; jubarte-python/python/jubarte_redlines/mcp_server.py:126-150,185-218).
- Source/output integrity and edit intent: optional legacy source_sha256 guard, prepared source/request/output digests, exact anchors and native prepublication source reread (src/edit.rs:2081-2088; src/edit/prepared.rs:196-235; src/bin/jubarte.rs:1642-1655).
- Process availability: default admission allows 64 MiB compressed/per-part, 256 MiB total inflation, 10000 entries and depth 256; comparison/WmlDocument allows 512 MiB compressed/per-part and 2 GiB total (src/admission.rs:63-99; src/wml_document.rs:50-55).
- Filesystem authority: MCP retained root and derived out_dir/{clean.docx,redline.docx,patch.diff,report.json}, optional redline.pdf/page PNGs; ambient CLI destinations; native Markdown resource-root image files; system/user/cloud fonts and persistent font index (jubarte-python/python/jubarte_redlines/mcp_server.py:185-217; src/publication.rs:146-225; src/bin/jubarte.rs:2858-2878; src/convert/font.rs:2281-2315,3098-3107).
- Native executable, release artifacts, optional signing key, GitHub publication tokens, cargo/npm/PyPI credentials and Jules service/token capabilities (.github/workflows/release.yml:230-369; scripts/release.sh:374-380,919,935-954,1069; .github/workflows/jules-review.yml:28-50).

## Trust Boundaries

- Untrusted package bytes enter ZIP/XML/DOM and image/font/legacy parsers. Admission checks budgets, header/extent consistency, safe canonical names, duplicate identities, encryption/compression and bounded inflation. Checked XML validates before DOM construction; permissive parse_xdocument deliberately does not enforce policy (src/admission.rs:439-575; src/xmllinq/parse.rs:90-135). Renderer first admits input, normalizes Strict packages, opens PartFs and expands package-contained altChunks (src/convert/mod.rs:769-808).
- JSON edit requests cross into mutation: preparation resolves/checks operations and candidate results before returning bytes; legacy SHA-256 guarding is optional, not authentication or approval. Signed/macro-bearing edit packages are refused (src/edit/prepared.rs:196-235; src/edit.rs:2081-2105).
- MCP clients share operator --root, default cwd. Startup normalizes root and opens _OutputRoot; actual reads/publication use retained cap-std authority, not merely Python resolve checks. Writes validate relative names, reject output symlinks and require overwrite for conflicts. No tenant separation or human approval is implemented (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,373-381; src/publication.rs:146-199,228-250,431-463).
- Native CLI publication grants ambient selected-path authority; Python/Node CLI handlers also own ambient host I/O and do not inherit MCP confinement. Native OutputRoot publication is atomic per file, not across the artifact set (src/publication.rs:168-225; jubarte-python/python/jubarte_redlines/__main__.py:126-133; jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-110).
- Native Markdown images use --resource-path or input parent (fallback .), decode escapes, reject URL schemes and require canonical containment plus capability-relative read. Python/Node CLIs reject resource_path (src/bin/jubarte.rs:2811-2878,2937-2943; jubarte-python/python/jubarte_redlines/__main__.py:1057-1064; jubarte-wasm/cli/bin/jubarte-redlines.mjs:724-727).
- Native rendering accesses ambient fonts independently of MCP root: macOS $HOME/Library/Application Support/jubarte/fonts; Linux ${XDG_DATA_HOME:-$HOME/.local/share}/jubarte/fonts; Windows %APPDATA%/jubarte/fonts. JUBARTE_FONT_DIR overrides these. Index defaults to parent(font-directory)/font-index.tsv; JUBARTE_FONT_INDEX overrides or empty/off disables it. Cached face paths are read after stamp checks; saves write a sibling filename ending .tsv.<pid>.tmp then rename (src/convert/font.rs:2637-2652,3098-3107,3154-3229).
- Explicit native self-update transfers release-publisher authority into executable replacement. Repository/platform/tag, SHA256SUMS.txt and regular-file extraction are configured; RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None leave signature enforcement disabled (src/update.rs:25-48,87-108,186-258,299-321).
- Tag-triggered GitHub release consumes build artifacts, optionally signs using step-scoped ZIPSIGN_PRIVATE_KEY staged under $RUNNER_TEMP/zipsign.<random>/keyN with exit cleanup, and publishes dist/* using contents:write. Local release additionally publishes cargo/npm/PyPI packages using operator credentials (.github/workflows/release.yml:15-21,230-321,356-369; scripts/release.sh:374-380,919,935-954,1069).
- Pinned external Jules review and companion-fix actions have distinct authority. Review gets PR/check writes with enable_approve=false. Companion invocation rejects fork heads, requires configured trusted author/dispatcher arthrod and qualifying events, then sets require_plan_approval=false and AUTO_CREATE_PR. External action internals are absent. Self-hosted CI/docs execute repository code under runner authority (.github/workflows/jules-review.yml:17-50; .github/workflows/jules-el-conejo-review.yml:69-92,259-324; .github/workflows/ci.yml:13-48; .github/workflows/docs.yml:76-99).
- Verified resource-row details retained from the independent architecture review: native cloud fonts derive from $HOME/Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/<single-normal-component-family>; renderer and rendered output receive font bytes under OS permissions (src/convert/font.rs:2225-2236). The persistent font index is read by the native renderer/current account and defaults to macOS ~/Library/Application Support/jubarte/font-index.tsv, Linux ${XDG_DATA_HOME:-~/.local/share}/jubarte/font-index.tsv, or Windows %APPDATA%/jubarte/font-index.tsv; JUBARTE_FONT_DIR changes the default parent and JUBARTE_FONT_INDEX separately overrides/disables it (src/convert/font.rs:2637-2652,3098-3107). Python CLI uses selected Path.write_bytes destinations, Node CLI selected fs.writeFileSync destinations; these recipients are local filesystem readers under operator/OS authority, distinct from MCP OutputRoot (jubarte-python/python/jubarte_redlines/__main__.py:126-133; jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-110). Updater's selected archive binary path is jubarte-<version>-<target>/jubarte[.exe], with SHA256SUMS.txt from the fixed repository (src/update.rs:25-48,104-108,198-207). Release private-key values are represented only by the ZIPSIGN_PRIVATE_KEY secret reference; the signing step and zipsign process receive temporary key files, then the GitHub publisher receives dist/* through GH_TOKEN using contents:write (.github/workflows/release.yml:261-309,356-369).

## Attacker Capabilities

- Document suppliers control package names/metadata, XML/relationships, embedded media/fonts, legacy structures and Markdown text; host environment, chosen trusted roots, release credentials and operator accounts are not implied capabilities.
- MCP clients can read/process under-root documents and invoke exposed writes with overwrite=true. Outside-root access would add authority; ordinary inside-root replacement is already granted (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,153-159).
- Local path-race or font-index adversaries require write access to the relevant directories/index. Document-only control does not establish that prerequisite (src/publication.rs:146-148; src/convert/font.rs:3154-3229).
- Library callers can select permissive parsing, budgets and callbacks; CLI operators already select ambient paths. These choices become boundaries only when a host exposes them to less-trusted callers.
- Release-asset publishers can replace archives and adjacent sums; current checksum-only updater provides no independent signature restriction. PR submitters control review context but lack the configured companion invocation authority unless its gates pass (src/update.rs:44-48,245-249; .github/workflows/jules-el-conejo-review.yml:259-324).

## Security Objectives

- Bound untrusted parsing and preserve safe package identities; callers must additionally enforce process CPU/memory limits (src/admission.rs:439-575; SECURITY.md:38-74).
- Avoid document-driven network fetching, macro execution and external office processes; confine native Markdown image reads to the caller-selected resource root (src/admission.rs:16-22; src/lib.rs:7-11; src/bin/jubarte.rs:2851-2878).
- Confine MCP document I/O, preserve source/intent guards, require explicit replacement and report partial publication accurately; application root checks are not a process sandbox.
- Protect executable/release integrity and secret recipients; keep review verdicts separate from human approval and maintain distinct external-action privileges.

## Assumptions

- No supplied user deployment context/model/knowledge base. Resource paths are configuration-derived expressions; actual runtime environment values, permissions, transport exposure and hosting limits were not supplied.
- MCP claims tools only touch root, but native rendering also reads fonts and persists the font index outside that root. Document I/O is confined; process-wide confinement is not established (jubarte-python/python/jubarte_redlines/mcp_server.py:5-9,67-108; src/convert/font.rs:2281-2315,3098-3107,3182-3229).
- Signing is explicitly staged in docs; current compiled updater uses checksums only. Workflow signing, if provisioned, does not imply these binaries verify signatures (docs/SELF_UPDATE.md:59-80; src/update.rs:44-48,245-249).
- SECURITY.md names rdocx-opc 0.2, while consumed manifest dependency is oxml-opc 0.14.2 with quick-xml 0.41/0.42. This is documentation drift, not an advisory finding (SECURITY.md:89-94; Cargo.toml:104-117).
- Engine edit refusal is distinct from filesystem publication failure; multi-file commit can be partial (src/publication.rs:168-174).
- Remote action internals, self_update dependency internals, actual credentials, registry/tag protections and self-hosted runner isolation remain unverified. Architecture mapping is not audit completion or a vulnerability finding.

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
