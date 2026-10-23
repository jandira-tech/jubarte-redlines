# Threat Model

Jubarte processes untrusted DOCX/OOXML, Markdown, and legacy DOC into document views, edited packages, tracked-change redlines, PDF/PNG, and reports. The Rust engine is consumed by its native CLI, PyO3 Python APIs/CLI, and wasm-bindgen browser/Node APIs and Node CLI. Python additionally provides a stdio MCP server with explicit directory-capability document/artifact I/O. Native rendering also performs ambient font discovery and font-index persistence, so the engine is not universally filesystem-pure. The optional native self-update command downloads a GitHub release and replaces the executable; release automation separately holds publication/signing authority. Sources: src/lib.rs:7-35, src/lib.rs:163-165, Cargo.toml:82-99, jubarte-python/Cargo.toml:31-39, jubarte-wasm/Cargo.toml:14-32, jubarte-python/python/jubarte_redlines/mcp_server.py:67-108, jubarte-python/python/jubarte_redlines/mcp_server.py:373-381, src/convert/font.rs:3090-3107, src/convert/font.rs:3213-3229, src/update.rs:186-257, .github/workflows/release.yml:230-265.

## Assets

- Confidential document content, embedded media, revisions, comments, authors, metadata, rendered output, and edit reports entrusted to the processing host and intentionally returned or exported by its selected interface. These are not encrypted by the engine. Sources: src/lib.rs:17-30, jubarte-python/python/jubarte_redlines/mcp_server.py:185-217, SECURITY.md:79-85.
- Integrity of document edits and correspondence between a plan and its intended input snapshot. The legacy edit path verifies source_sha256 when provided; this guard is optional and recorded as guarded in its report. Sources: src/edit.rs:2081-2088, src/edit.rs:2480-2482.
- Filesystem authority granted to MCP: canonical root and paths beneath it, including out_dir/clean.docx, redline.docx, patch.diff, report.json, optional redline.pdf and redline-page-NN.png; render exports use out_dir/page-NN.png and render.pdf. Sources: jubarte-python/python/jubarte_redlines/mcp_server.py:69-108,185-217,234-255.
- Host filesystem integrity outside intentionally authorized CLI destinations and Markdown image resource roots. Native CLI publication uses ambient destination-parent authority; MCP uses a held root capability. Sources: src/publication.rs:146-165,175-225, src/bin/jubarte.rs:261-273,2858-2871.
- Native renderer fonts and persistent index. Defaults: macOS $HOME/Library/Application Support/jubarte/font-index.tsv; Linux $XDG_DATA_HOME/jubarte/font-index.tsv or $HOME/.local/share/jubarte/font-index.tsv; Windows $APPDATA\jubarte\font-index.tsv. Environment overrides change locations. Sources: src/convert/font.rs:2633-2652,3090-3107,3182-3229.
- Embedding-process availability and CPU/memory. Default admission permits 64 MiB compressed/per-part and 256 MiB total inflated; compare permits 512 MiB compressed/per-part and 2 GiB inflated. Both retain 10,000 entries and XML depth 256. Sources: src/admission.rs:63-99.
- Installed executable integrity and release credentials. Current updater trusts GitHub transport plus SHA256SUMS.txt, with no compiled signing key/threshold. Release signing uses ZIPSIGN_PRIVATE_KEY reference; GitHub uses GITHUB_TOKEN, local release also uses registry credentials. Sources: src/update.rs:25-48,198-257, .github/workflows/release.yml:261-309,356-369, scripts/release.sh:370-380,919-959,1069.
- Privacy of content intended for redaction: edit postconditions refuse retained copies through scrub::leaks (src/edit.rs:4724-4748; src/edit/prepared.rs:798-804,873). UTF-16 XML support and untouched-part preservation create a relevant encoding boundary (src/admission.rs:768-781; src/edit.rs:5381-5384).

## Trust Boundaries

- Untrusted document bytes -> in-process ZIP/OPC/XML/rendering. Admission precedes representative public parsing paths and applies active resource/XML limits. Inspection uses defaults; PDF/PNG layout uses admit_package and compare limits. Checked XML validates before DOM; infallible parse_xdocument deliberately does not. Sources: src/admission.rs:13-28, src/inspect.rs:604-622, src/convert/mod.rs:768-778, src/document_comparer.rs:4922-4928, src/xmllinq/parse.rs:84-135.
- ZIP central/local records -> shared production opener. open_archive runs zip_defects before returning; unchecked opening is distinct diagnostic/admission machinery. Sources: src/opc/archive.rs:15-20,34-51.
- Stdio MCP client -> filesystem host. --root defaults to cwd, is expanded/canonicalized, native _OutputRoot opens once. Python containment is diagnostic; actual reads/writes use held capability. Outputs additionally validate portable relative names, refuse symlinks, use atomic hard-link no-clobber without overwrite. Tool annotations are hints, not separate caller authorization. Sources: jubarte-python/python/jubarte_redlines/mcp_server.py:54-55,67-108,373-381, jubarte-python/src/lib.rs:1244-1278, src/publication.rs:146-199,228-254,448-463,558-570.
- Native CLI operator -> ambient input/output paths. std::fs::read inputs and publish_paths holds selected destination parents, stages .jubarte-<uuid>.partial/bytes. Atomic per file, not whole set. Distinct from MCP root confinement. Sources: src/bin/jubarte.rs:2740-2742,261-273, src/publication.rs:168-173,203-225,418-482.
- Python/Node CLI operators -> ambient files. Python Path.read_bytes/write_bytes and Node fs.readFileSync/writeFileSync are independent I/O. Shared grammar/core does not imply native staged publication. Sources: jubarte-python/python/jubarte_redlines/document.py:141-145, jubarte-python/python/jubarte_redlines/__main__.py:73-89,101-103,126-133, jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-79,106-111.
- Markdown images -> native CLI filesystem reads. Root is explicit --resource-path, otherwise Markdown parent/cwd. Names percent-decoded, URL schemes rejected, canonical containment checked, held OutputRoot performs reads. Library loader policy belongs to custom caller. Sources: src/bin/jubarte.rs:2811-2871,2936-2943.
- Native renderer -> ambient font discovery/cache. Document families drive installed-font lookup; cloud family requires one normal component. Index load/save and indexed face reads use ambient std::fs, independently of MCP capability. Sources: src/convert/font.rs:2219-2236,2258-2315,2633-2652,3154-3169,3182-3229,3395-3432.
- Browser/WASM host -> byte APIs. Native publication is non-wasm32 only, WASM engine defaults/self-update disabled. Node CLI adds ambient filesystem operations around WASM. Sources: src/lib.rs:147-149,163-165, jubarte-wasm/Cargo.toml:18-32, jubarte-wasm/src/lib.rs:86-104, jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-73.
- Updater metadata/assets -> executable. Compiled repository/platform, explicit --version may downgrade, --check avoids installation, noninteractive install requires --yes. SHA256SUMS.txt checked and extracted candidate regular-file required. Signature verification inactive: RELEASE_KEYS empty, FIRST_SIGNED_RELEASE None. Sources: src/update.rs:25-48,91-108,120-150,198-257,274-285,307-315.
- Maintainer tag/workflow -> publication/signing. v* triggers version checking/read-only builds; release job alone contents:write. Step-scoped signing reference decoded to temp files, archives signed and optionally checked with committed keys, checksums generated, dist/* published. Other registries via local script. Sources: .github/workflows/release.yml:15-21,40-89,230-322,356-369, scripts/release.sh:919-959,1069.
- Caller-supplied reference DOCX -> Markdown package assembly -> main/reference XML DOM. Python/WASM reference bytes are passed unchanged through shared options; bounded package reading is distinct from XML admission (jubarte-python/src/lib.rs:1206-1218; jubarte-wasm/src/lib.rs:1438-1449,1482-1488; src/markdown/mod.rs:260-282; src/markdown/package.rs:108-126,516-547).

## Attacker Capabilities

- Document supplier controls ZIP records, OPC/XML, embedded fonts/images, text, Markdown, legacy DOC supplied to an authorized interface. No initial environment, operator-root, executable, release-asset or credential control. Sources: SECURITY.md:38-41, src/admission.rs:13-28, src/legacy_doc.rs:13-50.
- MCP client invokes registered tools with path/plan/render/output/overwrite arguments within operator-granted root. No separate tenants or per-tool principals established. Authorized root reads and requested writes are normal capabilities. Sources: jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,220-277,373-381.
- Concurrent filesystem writer is conditional publication/resource actor; not assumed for ordinary remote document supplier. Held capabilities/no-clobber installation are relevant controls. Sources: src/publication.rs:146-165,558-570, src/bin/jubarte.rs:2858-2871.
- Release-asset attacker could cross install boundary upon operator update; checksum-only mode has no independent signer. Asset-write is an additional prerequisite, not gained by document input. Sources: docs/SELF_UPDATE.md:59-82, src/update.rs:44-48,198-257.
- Trusted startup environment or font-index control is distinct authority from document supply. Cache/font locations and indexed paths can be selected by that actor; escalation requires a less-privileged writer and useful new effect. Sources: src/convert/font.rs:2637-2652,3100-3107,3314-3358.

## Security Objectives

- Apply active package/XML budgets before permissive DOM, preserve typed refusal, explicitly gate unchecked APIs. Sources: src/inspect.rs:604-622, src/admission.rs:63-99, src/xmllinq/parse.rs:84-135.
- Prevent package data becoming ambient extraction, network fetching or macro execution. In-memory admission, external relationships retained as data; CLI image URLs forbidden separately. Sources: src/admission.rs:16-22, SECURITY.md:70-74, src/bin/jubarte.rs:2811-2871.
- Confine MCP explicit reads/artifacts to originally granted capability including concurrent pathname replacement; overwrite required. This does not sandbox whole renderer. Sources: jubarte-python/python/jubarte_redlines/mcp_server.py:75-108, src/publication.rs:146-199,448-463,558-570.
- Preserve previous native outputs on staging failures; report partial commits/durability; atomic per file. Python/Node distinct until verified. Sources: src/publication.rs:168-173,203-225, jubarte-python/python/jubarte_redlines/__main__.py:126-133, jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-111.
- Bind edits to intended content through optional hashes/exact anchors, not authentication. Sources: src/edit.rs:2081-2088,2480-2482, SECURITY.md:76-79.
- Explicit updater policy: current HTTPS/checksum, signature needs key/threshold, regular extracted file. Sources: Cargo.toml:137-148, src/update.rs:44-48,198-257,274-285,307-315, docs/SELF_UPDATE.md:59-82.
- Privileged maintainer workflows alone hold publication credentials; ordinary processing lacks signing/registry authority. Sources: .github/workflows/release.yml:19-21,230-265,356-369, scripts/release.sh:370-380.

## Assumptions

- No user deployment context, supplied threat model, or knowledge base. No public HTTP/multitenancy/auth layer/shared service/container mount mapping/production quotas established.
- Memory/CPU process limits are host obligations; input budgets do not prove all DoS prevention. Native/Python/Node input reads precede inspection of full bytes. Sources: src/admission.rs:39-40, SECURITY.md:70-74, src/bin/jubarte.rs:2740-2742, jubarte-python/python/jubarte_redlines/document.py:141-145, jubarte-wasm/cli/bin/jubarte-redlines.mjs:52-55.
- MCP build_server docstring claims only files under root, but docx_render invokes native ambient font discovery/index persistence. Explicit document/artifact capability remains enforced. Documentation discrepancy, no exploit established. Sources: jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,234-255, src/convert/font.rs:2637-2652,3090-3107,3182-3229.
- Library overview says no temporary files, but native cache save writes sibling .tsv.<pid>.tmp then renames. Returned document artifacts and ambient font side effects differ. Sources: src/lib.rs:7-11, src/convert/font.rs:3211-3229.
- Signed-release docs explicitly qualify staged checksum-only state. No trusted key/threshold in current code; signature dependency support is not enforcement. Sources: docs/SELF_UPDATE.md:59-82, src/update.rs:44-48,245-249.
- Native CLI staged publication and MCP root confinement are not universal: Python/Node direct I/O. Sources: src/bin/jubarte.rs:261-273, jubarte-python/python/jubarte_redlines/__main__.py:126-133, jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-111.
- Actual environment, cwd/root, OS ACLs, installed binary destination, updater internals, GitHub settings/secrets, registry permissions and generated WASM equivalence unverified offline. Deployment/artifact questions, not source-proved missing controls.
- Independent architecture mapping is not completed security audit coverage. No application execution, service contact, mutation or triggering inputs.
- Audit reconciliation: source-validated findings demonstrate exceptions to intended XML/resource/redaction invariants. Numbering/fontTable/theme/styles/settings, HTML altChunk representation changes, reference import, legacy decoding, embedded-font aliases, ZIP refusal diagnostics and the direct Strict classifier retain distinct broken controls. No generated model claim should imply those invariants are universally enforced. Final coverage remains partial with exact unresolved paths.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Model scope description: Current source in /Users/arthrod/temp/T/jubarte-redlines; offline architecture mapping only.
- Scan scope: .

This is a provisional model saved before successful completion.
