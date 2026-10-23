# Threat Model

Jubarte is a Rust Word-document toolkit exposed as native CLI, Rust byte APIs, Python/PyO3 facade and stdio MCP server, and browser/Node WASM APIs plus npm CLI. Documents and edit plans pass through admission/XML/package processing to DOCX, Markdown, PDF/PNG and reports. Normal processing is local; explicit native self-update downloads and installs a GitHub release (src/lib.rs:7-35,96-165; jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,373-381; src/update.rs:186-258).

## Assets

- Confidential document bytes, revisions/comments/authors and derived renderings/reports; integrity of selected input/output files and admitted package structures (src/lib.rs:17-30; src/publication.rs:146-225).
- MCP root filesystem capability; native executable integrity on self-update; release publishing/signing authority (.github/workflows/release.yml:230-275,356-369; src/update.rs:25-48,198-258).
- Native renderers read installed/user/Word cloud-cache fonts and persist a per-user font index, distinct from document output-root authority (src/convert/font.rs:2228-2236,2633-2652,3090-3107,3213-3227).
- MCP edit outputs are root/out_dir/{clean.docx,redline.docx,patch.diff,report.json}, optional redline.pdf and redline-page-NN.png, available to authorized root filesystem readers and reported to attached client; native OutputRoot publishes with default no overwrite and per-file atomicity (jubarte-python/python/jubarte_redlines/mcp_server.py:153-217; src/publication.rs:168-199).

## Trust Boundaries

- Untrusted DOCX/ZIP/XML, legacy DOC, embedded images/fonts and edit plans enter the engine. Default admission bounds archive/part bytes 64 MiB, 10000 entries, inflation 256 MiB, XML depth 256; comparison allows 512 MiB archive/part and 2 GiB inflation. Permissive parse_xdocument requires a prior gate (src/admission.rs:15-28,63-99; src/xmllinq/parse.rs:21-36).
- MCP stdio client invokes document tools under operator --root (default cwd). Python resolves paths for diagnostics; native _OutputRoot owns confined reads/publication through a held capability. Annotations are hints, not caller authorization (jubarte-python/python/jubarte_redlines/mcp_server.py:55,67-108,373-381; jubarte-python/src/lib.rs:1244-1277; src/publication.rs:146-199).
- Native CLI grants ambient paths with staged per-file publication. Python/Node CLI ambient reads/writes are separate from MCP confinement (src/bin/jubarte.rs:261-273; src/publication.rs:203-225; jubarte-python/python/jubarte_redlines/document.py:142-145; jubarte-python/python/jubarte_redlines/__main__.py:126-132; jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-56,106-110).
- Native Markdown images use --resource-path or Markdown parent directory; URL schemes/canonical escape targets are refused, followed by capability-relative read (src/bin/jubarte.rs:2835-2876,2936-2943).
- Self-update enters executable replacement only on explicit command and confirmation/--yes. RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None mean checksum-only today; optional Ed25519 support is inactive (src/update.rs:25-48,120-125,198-258,274-285; docs/SELF_UPDATE.md:59-82).
- Tag-triggered release builds are read-only; release job has contents:write, signing step receives signing secret and publishes dist/* with workflow token. These are CI/maintainer authorities (.github/workflows/release.yml:15-21,51-55,230-275,356-369).
- Native rendering, including MCP rendering, has separate ambient font discovery/cache I/O: JUBARTE_FONT_DIR overrides user font roots; JUBARTE_FONT_INDEX controls index persistence independently of document-root grants (src/convert/font.rs:2633-2652,3098-3107,3192-3229).
- PR workflows run checked-out code on self-hosted runners sharing the private mbx cache http://mbx-cache:7543, namespace jandira-tech/jubarte-redlines. Comments promise protected-push publishing and PR reads, but server writer authorization and object validation are absent from this repository (.github/workflows/ci.yml:13-17,34-48,72-101; .github/actions/mbx/action.yml:8-22).

## Attacker Capabilities

- Document suppliers control package/XML/relationship/font/image/Markdown bytes, not operator environment, MCP root grants or release infrastructure (src/admission.rs:15-28; src/bin/jubarte.rs:2851-2876).
- Authorized MCP clients read/publish inside granted root including explicit overwrite; tools do not grant updater or ambient path authority (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,153-217,373-381).
- Control of release assets permits changing archive and adjacent checksum; this is a separate supply-chain starting capability not authenticated by checksum-only policy (docs/SELF_UPDATE.md:59-77; src/update.rs:44-48,245-248).

## Security Objectives

- Do not fetch external document relationships or Markdown URL resources; confine MCP and resource reads to granted directory capabilities (src/admission.rs:15-22; src/bin/jubarte.rs:2835-2876; src/publication.rs:146-199).
- Refuse malformed/over-budget packages/XML before DOM processing; host memory/CPU process bounds remain necessary (src/admission.rs:39-99; src/xmllinq/parse.rs:21-36; SECURITY.md:70-85).
- Bind source_sha256 to document bytes when supplied and reject stale plans; staged publication is atomic per file only (src/edit.rs:2081-2089; src/publication.rs:83-131,168-174).
- Protect updater integrity with HTTPS/checksums, configured signatures and regular-file check; keep document processing distinct from release publication (src/update.rs:198-258,299-307; .github/workflows/release.yml:230-275).

## Assumptions

- No deployment, tenants or HTTP listener supplied; MCP uses stdio and operator controls client attachment/root (jubarte-python/python/jubarte_redlines/mcp_server.py:373-381).
- Limits are configurable; hosts should not silently raise them for hostile inputs. Low-level permissive XML APIs require prior checking (src/admission.rs:103-164; src/xmllinq/parse.rs:21-30; SECURITY.md:70-85).
- Updater docs disclose checksum-only operation until a key is committed; signature support is not active authentication (docs/SELF_UPDATE.md:59-82; src/update.rs:44-48).
- Overview says no temporary files, but native font lookup persists an index and PID temporary sibling: a documentation qualification, not demonstrated vulnerability (src/lib.rs:7-11; src/convert/font.rs:3098-3107,3213-3227).
- Current source audit only, no Git history or runtime execution; architecture mapping does not establish full audit coverage.
- Effective font resources: JUBARTE_FONT_DIR nonempty is used verbatim; otherwise macOS HOME/Library/Application Support/jubarte/fonts, Windows APPDATA/jubarte/fonts, Linux XDG_DATA_HOME/jubarte/fonts or HOME/.local/share/jubarte/fonts. The renderer reads these under host permissions (src/convert/font.rs:2633-2652).
- Effective font-index state: explicit JUBARTE_FONT_INDEX is used except empty/off disables; absent derives parent(user_font_dir)/font-index.tsv. Default macOS HOME/Library/Application Support/jubarte/font-index.tsv, Windows APPDATA/jubarte/font-index.tsv, Linux XDG_DATA_HOME/jubarte/font-index.tsv or HOME/.local/share/jubarte/font-index.tsv. Temporary sibling uses path.with_extension(tsv.<pid>.tmp); current/subsequent renderer processes consume index records under host permissions (src/convert/font.rs:3090-3107,3155-3170,3192-3229).
- Installed native font resources include macOS Word DFonts, /Library/Fonts/Microsoft, /System/Library/Fonts/Supplemental and /Library/Fonts; Office cloud family lookup uses HOME/Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/<family> and requires one normal component. Windows uses WINDIR/SystemRoot/C:\Windows Fonts plus optional LOCALAPPDATA/Microsoft/Windows/Fonts. Recipients are font parsing/shaping and generated renderings; trusted environment, host permissions and family validation apply (src/convert/font.rs:2181-2236,2855-2862).
- Updater effective resource: fixed GitHub repo jandira-tech/jubarte-redlines, selected v<version> and platform archive, binary jubarte-<version>-<target>/jubarte[.exe], SHA256SUMS.txt and compiled key configuration. self_update owns download/extraction/replacement; local verify_binary rejects nonregular files (src/update.rs:25-48,91-108,198-258,299-321).
- Release secret references: secrets.ZIPSIGN_PRIVATE_KEY reaches only signing-step env and RUNNER_TEMP/zipsign.XXXXXX/keyN files with EXIT cleanup; secrets.GITHUB_TOKEN reaches publication env for github.repository/dist/*. No literal secret was reproduced. Maintainer/tag/CI compromise is a separate prerequisite (.github/workflows/release.yml:230-309,317-322,356-369).
- Supported fallback scripts/release.sh release_from_artifacts creates checksums and publishes downloaded artifacts without normal signing step. Current updater is checksum-only, so no active signature bypass was validated; recheck this workflow when signing is activated (scripts/release.sh:976-1012; .github/workflows/release.yml:252-309).
- Private-cache investigation remains unresolved: source establishes self-hosted PR code and documented unauthenticated cache reachability, but server write/delete routes, object integrity checks, namespace isolation, trusted-consumer behavior and runner/fork approval settings are not supplied (.github/actions/mbx/action.yml:8-22; .github/workflows/ci.yml:34-101).
- The supplied configuration targets the live directory, not a requested immutable commit. The initial HEAD observation was 6a34e1c498bcf403ed98a443ac11a9fced70892f; artifact-directory SHA was not used as source authority. Source inventory changed during review, so whole-tree coverage cannot be claimed immutable.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Model scope description: Architecture of current source in supplied repository; no immutable revision was requested.
- Scan scope: .

This is a provisional model saved before successful completion.
