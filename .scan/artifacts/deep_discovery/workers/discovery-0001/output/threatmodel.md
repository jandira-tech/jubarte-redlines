# Threat Model

Jubarte is a local Rust Word-document toolkit exposed through Rust byte APIs, PyO3 Python, WASM/Node, native and language CLIs, and an optional Python MCP server over stdio. Untrusted DOCX/DOC/Markdown and JSON edit plans feed package admission, XML/legacy parsers, comparisons, edits and PDF/PNG rendering. Native publication and explicit self-update are separate host-authority boundaries (src/lib.rs:7-30,106-174; src/convert/mod.rs:769-798; jubarte-python/python/jubarte_redlines/mcp_server.py:67-112,373-381). An independent architecture reviewer identified the resource rows below before its execution was interrupted; parent verified their actual consumers offline.

## Assets

- Host process availability and bounded memory/CPU while parsing attacker-authored documents (src/admission.rs:42-105; src/legacy_doc.rs:179-216).
- Integrity/confidentiality of source documents and generated output; edit snapshot SHA-256 and output alias guards (src/bin/jubarte.rs:1610-1667,1768-1826).
- Files outside MCP's held root capability and the operator's installed executable (src/publication.rs:146-200; src/update.rs:198-257,299-321).
- Release/registry authority and signing credentials supplied by the operator or CI secret store, never document contents (.github/workflows/release.yml:261-310; scripts/release.sh:976-1012).

## Trust Boundaries

- Document author -> Rust parsing/rendering -> caller process. admission bounds compressed bytes, entry counts, per-part/total inflation and selected XML depth/tag policy; downstream XML, numeric layout, images/fonts and legacy fields require their own effective controls (src/admission.rs:442-580,788-827; src/convert/mod.rs:769-798).
- Python/Node/native CLI callers choose ambient local input/output paths; these callers already possess operator file authority. MCP clients instead provide relative document/output names beneath --root, default current working directory; build_server resolves that root once and _OutputRoot retains native cap_std::Dir authority for actual I/O (jubarte-python/python/jubarte_redlines/mcp_server.py:67-112,373-381; jubarte-python/src/lib.rs:1244-1279; src/publication.rs:146-200).
- MCP/native rendering -> ambient font resources: JUBARTE_FONT_DIR overrides the platform user font directory; macOS ~/Library/Application Support/jubarte/fonts, Unix $XDG_DATA_HOME/jubarte/fonts or ~/.local/share/jubarte/fonts, Windows %APPDATA%/jubarte/fonts. Fonts are read by conversion outside the document root (src/convert/font.rs:2637-2652).
- Rendering -> persistent font index: JUBARTE_FONT_INDEX selects a path; empty/off disables it; default parent(user_font_dir)/font-index.tsv. FONT_INDEX loads that path and save merges and writes path.with_extension(tsv.<pid>.tmp), then renames it to the index (src/convert/font.rs:3100-3107,3213-3229). This is component ambient authority, distinct from tool document/output confinement.
- Explicit self-update -> fixed jandira-tech/jubarte-redlines GitHub release and SHA256SUMS.txt -> extracted platform executable -> installed binary. TLS/checksum verification, optional compiled version-gated signing policy, and lstat regular-file hook enforce distinct steps (src/update.rs:25-48,198-257,274-285,299-321; Cargo.toml:137-148).
- Privileged release workflows: release.yml optionally signs with secret-store ZIPSIGN_PRIVATE_KEY, computes SHA256SUMS.txt and publishes GitHub assets. Operator scripts/release.sh:976-1012 can recover workflow artifacts into dist/release, compute checksums and create a release without that signing step.

## Attacker Capabilities

- An attacker can author DOCX ZIP entries, relationship targets/types, XML structures and numeric attributes, embedded media/font bytes, legacy DOC character streams, Markdown and edit-plan values that a caller elects to process.
- An MCP client can invoke registered tools over its existing stdio connection and request reads/writes within the granted root; overwrite=true explicitly permits replacement there. No remote listener or tenant model is established by the source (jubarte-python/python/jubarte_redlines/mcp_server.py:93-112,373-381).
- Document control does not imply control of host environment variables, font directories/index, release assets, signing credentials, operator checkout or trusted MCP root. No new authority is inferred from already possessing those privileges.

## Security Objectives

- Reject malformed or excessive document input before unbounded recursion, scalar-driven allocation or algorithmic amplification reaches the host process.
- Keep document relationship resolution within in-memory package data and do not fetch external references (src/convert/mod.rs:21310-21319; src/admission.rs:14-23).
- Enforce held-root filesystem confinement and no-clobber publication; retain source identity for guarded native edits (src/publication.rs:419-574; src/bin/jubarte.rs:1610-1667,1768-1826).
- Install only checksum-verified release binaries and enforce configured signing keys/version rules and regular-file validation; never expose CI secret material in generated analysis.

## Assumptions

- Exact userContext is null; no supplied threat model or CODEX_SECURITY_KNOWLEDGE_BASE was present. Only current on-disk source was reviewed; no Git history, network, application execution or credential validation was used.
- Rust library, Python and WASM hosts may pass adversarial documents; no specific internet-facing production deployment is assumed. SECURITY.md recommends process CPU/memory limits; caller-enforced isolation constrains blast radius but does not establish parser safety.
- MCP docstring claims tools only touch files under root (jubarte-python/python/jubarte_redlines/mcp_server.py:67-68), while native font resolution/index uses ambient directories (src/convert/font.rs:2637-2652,3100-3107,3213-3229). Retain that guarantee mismatch; no attacker-directed file disclosure or arbitrary overwrite beyond trusted font configuration was established.
- Compiled RELEASE_KEYS is empty and FIRST_SIGNED_RELEASE is None (src/update.rs:44-48), so current verification is checksum-only. docs/SELF_UPDATE.md:59-87 acknowledges this. The operator fallback release path lacks signing, a future signing-era compatibility concern rather than an established present substitution vulnerability.
- Runtime and deployment-specific exploit reliability remain untested. The interrupted architecture/baseline workers did not return final fully-reviewed inventories; partial review and remaining paths must remain explicit.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: not recorded
- Scan scope: .

This is a provisional model saved before successful completion.
