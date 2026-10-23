# Threat Model

Jubarte is a local Word document processing engine exposed through Rust byte APIs, a native CLI, Python/PyO3 APIs and CLI, a Python stdio MCP server, and JavaScript/WASM APIs plus a Node CLI. It admits DOCX packages, parses XML into an arena DOM, compares or edits documents, and produces DOCX, Markdown, inspection reports, PDF and PNG output. Legacy DOC conversion is a separate OLE parser. Rendering uses bundled, embedded and native installed fonts. Filesystem authority belongs to host adapters: the MCP adapter grants a persistent directory capability, whereas local CLIs use ambient process authority. The native CLI additionally offers an explicit GitHub self-update operation. Tag-triggered CI publishes release artifacts; a separate maintainer script publishes package registries. Sources: src/lib.rs:7-35,96-165; src/legacy_doc.rs:13-50; jubarte-python/pyproject.toml:42-49; jubarte-python/python/jubarte_redlines/mcp_server.py:67-108,373-381; jubarte-wasm/cli/bin/jubarte-redlines.mjs:16-21; src/update.rs:186-257; .github/workflows/release.yml:15-21,230-369.

## Assets

- Confidential document text, authors, comments, tracked changes, embedded media and derived reports. Inspection/rendering intentionally disclose selected content to the invoking caller; hosts protect reports, patches and storage/client access. src/lib.rs:15-29; jubarte-python/python/jubarte_redlines/mcp_server.py:131-150,185-217.
- Integrity of source documents, requested edits and produced artifacts. V1 plans optionally pin source_sha256; mismatches refuse STALE_SOURCE. Preparation binds source, normalized request and actual primary output with separate SHA-256 digests. src/edit.rs:97-106,2073-2089; src/edit/prepared.rs:202-234.
- Host filesystem authority: MCP document I/O uses an opened root directory; native CLI selected output paths carry ambient authority. Native installed-font reads and font-index cache have independent process authority. src/publication.rs:146-205; jubarte-python/python/jubarte_redlines/mcp_server.py:69-108; src/convert/font.rs:2258-2276,2633-2652,3090-3107,3213-3227.
- Embedding process/service/agent availability. Default limits: 64 MiB compressed/per part,256 MiB inflated,10000 entries,depth256; comparer/renderer:512 MiB compressed/per part,2GiB inflated,same count/depth. src/admission.rs:63-99; src/document_comparer.rs:4926-4927; src/convert/mod.rs:769-778.
- Installed executable and distribution integrity. Updater effective trust is HTTPS plus co-located SHA256SUMS.txt, no active keys. Release job has contents-write token; optional signing uses secret reference ZIPSIGN_PRIVATE_KEY. src/update.rs:25-48,198-210,245-257; Cargo.toml:self_update features; .github/workflows/release.yml:230-264,277-309,356-369.

## Trust Boundaries

- Untrusted document producer -> engine bytes: representative comparison/rendering admits container budgets, safe part identities, compression, inflated bytes and XML policy; normal opens check ZIP structure. Debug intentionally uses unchecked opener for diagnostic reporting. src/admission.rs:13-28,399-424,439-569; src/document_comparer.rs:4903-4918,6530-6567; src/convert/mod.rs:769-778; src/opc/archive.rs:15-51.
- Raw XML -> public parser: parse_xdocument is permissive, no input policy; try_parse_xdocument_with_limits validates before DOM. Callers must choose checked API for untrusted XML. src/xmllinq/parse.rs:19-36,84-135.
- Agent edit requests -> document mutation: optional source digest mismatch refuses, operations resolve before application, candidate checked, preview/application share preparation. These are integrity controls, not authentication or human approval. src/edit.rs:5-24,88-106,2073-2094; src/edit/prepared.rs:202-234.
- MCP stdio client -> held root: --root defaults cwd, expanduser/resolve then opens _OutputRoot once; cap-std actual reads/publication back Python diagnostics. All clients share server root; annotations are descriptions rather than independent authorization. jubarte-python/python/jubarte_redlines/mcp_server.py:54-55,67-108,373-381; jubarte-python/src/lib.rs:1244-1278; src/publication.rs:146-199.
- Artifacts -> filesystem: OutputRoot validates portable relative names and rejects output symlinks; CLI publish_paths grants ambient selected parents. Both stage,flush,sync,readback before per-file commits; no-clobber hard-link or overwrite rename. Sets can partially commit and errors report committed paths; Windows parent sync is no-op. src/publication.rs:83-143,168-270,387-405,419-505,533-565.
- Markdown images -> native local reads: --resource-path else Markdown parent else '.'; percent-decoded names reject schemes and must remain canonical-root-contained; actual OutputRoot read preserves confinement. Library ImageLoader callback has caller authority and default is absent. src/bin/jubarte.rs:2835-2876,2936-2943; src/markdown/mod.rs:159-170,196-198.
- Document font requests -> native host: system/Word/cloud/user font lookup and chosen-path index; JUBARTE_FONT_DIR overrides user fonts; JUBARTE_FONT_INDEX direct path or empty/off disables, default parent(user_font_dir)/font-index.tsv, PID-qualified sibling temporary file then rename. Applies to MCP native rendering independently of document root. src/convert/font.rs:2225-2235,2258-2315,2633-2652,2855-2862,3090-3107,3154-3188,3213-3247,3395-3432; jubarte-python/python/jubarte_redlines/mcp_server.py:185-188,234-255.
- WASM bytes -> JS host: exported bytes/JSON APIs have no native publication; Node wrapper adds ambient fs.readFileSync/fs.writeFileSync, ensureWritable existence checks and native-only-option refusals. No inferred native cap-std parity. src/lib.rs:163-165; jubarte-wasm/src/lib.rs:233-272; jubarte-wasm/cli/bin/jubarte-redlines.mjs:16-21,52-79,106-110,712-740.
- Operator -> self-update: explicit preflight terminal/--yes/--check, supported platform/GitHub release selection, confirmation unless yes; check stops before installation. self_update handles HTTPS/checksum/binary replacement with regular-file hook. RELEASE_KEYS=[] and FIRST_SIGNED_RELEASE=None disable signature requirement. src/update.rs:25-48,87-125,186-257.
- Maintainer/tag -> distribution: v* tag release builds have contents-read, publishing contents-write, no checkout credential persistence; dist artifacts optionally signed from RUNNER_TEMP/zipsign.XXXXXX private key directory with EXIT cleanup, checksummed and gh-published with secrets.GITHUB_TOKEN. Separate scripts/release.sh uses authenticated local registries. .github/workflows/release.yml:15-21,40-89,230-321,356-369; scripts/release.sh:72-105,150-168.
- Untrusted fork PR -> self-hosted execution: CI/docs/adoption/roundtrip check out PR code and execute local actions/builds/scripts; runner network has documented mbx-cache:7543, with reusable checkout/cache behavior. GitHub approval policy, runner isolation/lifetime and server-side cache authority unknown. .github/workflows/ci.yml:13-48; docs.yml:42-99; adoption.yml:19-100; md-roundtrip.yml:13-39; .github/actions/mbx/action.yml:8-38.

## Attacker Capabilities

- Document producer chooses ZIP records,XML,relationships,images,embedded fonts,Markdown and legacy DOC, without authority over host environment,MCP startup root,installed fonts,release secrets or operator-selected paths. SECURITY.md:38-85; src/admission.rs:13-28; src/legacy_doc.rs:13-50.
- Connected MCP client invokes registered read/mutation tools and chooses paths,plans,outputs,overwrite within the granted root. No per-user ownership or human gate established; stdio launcher/client is boundary. Authority outside root cannot be assumed. jubarte-python/python/jubarte_redlines/mcp_server.py:54-108,152-168,220-277,373-381.
- Concurrent hostile filesystem writer requires write access to relevant parent,staging location or granted root; document control alone gives no such ability. src/publication.rs:146-148,353-381,448-505,558-565.
- GitHub release-asset controller supplies archives/matching checksums under current documented checksum-only trust; ordinary document author,MCP client or unauthenticated network peer lacks that authority. src/update.rs:25-48,198-210,245-257; docs/SELF_UPDATE.md:59-80.
- Repository/tag/package publishers are privileged operators; their credential access is not an attacker power without compromise evidence. Benchmark stdin uses ambient path authority as developer harness, not established network service. .github/workflows/release.yml:15-21,230-264; scripts/release.sh:98-105; jubarte-rust-inproc/src/main.rs:5-20,91-98.
- External fork contributor may submit PR code, but workflow execution depends on actual Actions approval policy. Private-network or persistence impact additionally depends on runner provisioning/isolation.

## Security Objectives

- Admit untrusted packages before processing, preserve typed refusal, reject inconsistent ZIP, enforce active XML limits before checked DOM construction; host CPU/memory isolation remains required. SECURITY.md:38-74; src/admission.rs:399-569; src/opc/archive.rs:34-51; src/xmllinq/parse.rs:111-135.
- No external relationship fetch or embedded execution; inert metadata differs from explicitly granted Markdown loaders and local font discovery. src/admission.rs:16-22; SECURITY.md:70-85; src/markdown/mod.rs:159-170; src/convert/font.rs:2258-2315.
- MCP document I/O confined to held root, honor overwrite=false, refuse unsafe export paths, report partial sets; native font I/O is outside document capability. mcp_server.py:67-108 in jubarte-python/python/jubarte_redlines; src/publication.rs:83-199,228-270; src/convert/font.rs:3090-3107,3213-3227.
- Apply resolved edits to supplied source, honor optional SHA256 and preparation-output binding; author labels/hashes do not authenticate callers. src/edit.rs:5-24,97-108,2073-2094; src/edit/prepared.rs:202-234.
- Executable replacement only through explicit configured checksum/signature workflow with correct platform member. Do not claim signatures active with no keys. src/update.rs:25-48,87-125,186-257.
- Limit release secrets to intended steps/jobs and bind artifacts to tag/version/checksums; never reproduce credentials in analysis. .github/workflows/release.yml:40-49,230-321,356-369.

## Assumptions

- Exact authorized live target reviewed, no historical git-show: HEAD observed 4e10eca69c42211651ca907f38b74b6d19e6928c, scan folder labels9a0423c22ce6e195c32b886a3f45ccc48087d1dc. Findings must not be silently attributed to labeled historical revision.
- src/lib.rs:7-11 broadly promises byte APIs/no temporary files; native renderer writes font-index.tsv and PID-qualified temporary sibling when lookup records entries (src/convert/font.rs:3090-3107,3192-3229). This is auxiliary font state rather than package extraction.
- MCP tools-only-under-root claim (jubarte-python/python/jubarte_redlines/mcp_server.py:67-108) covers capability document I/O but rendering additionally accesses native font folders/index (src/convert/font.rs:2258-2315,3090-3107,3213-3247). Impact depends on configured paths,cache,permissions and lookups; no arbitrary path attack inferred.
- Checksum-only current state agrees with docs/SELF_UPDATE.md:69-80 and empty keys/start-version src/update.rs:44-48. Signing capability does not prove enforcement.
- Native/Python/Node CLIs use operator ambient filesystem authority. Python _write uses Path.write_bytes (jubarte-python/python/jubarte_redlines/__main__.py:126-133), Node fs.writeFileSync (jubarte-wasm/cli/bin/jubarte-redlines.mjs:106-110); command parity does not prove staged-write parity.
- No deployed HTTP listener,tenant population,mounts,effective env,ACLs or client authentication supplied. MCP uses stdio (jubarte-python/python/jubarte_redlines/mcp_server.py:373-381); remote exposure needs another wrapper.
- Root capability and input budgets are distinct controls, not whole-process sandbox; SECURITY.md:70-74 requires CPU/memory host limits.
- Architecture mapping alone is not completed security coverage. self_update extraction/replacement internals,cap-std platform internals and external pinned action implementations were not fully inspected.
- Normal CI optionally signs and requires secret if committed public keys exist (.github/workflows/release.yml:252-321); fallback release_from_artifacts (scripts/release.sh:976-1012) checksums/publishes without same signing step. Current empty trusted keys make this conditional future signing discrepancy rather than active bypass.
- Fork approvals,self-hosted worker isolation/lifetime,private-network policy and independent cache-server authorization were not supplied. No host-root,Docker socket,cache-write,secret theft or persistence inferred.

---

## Saved Model Context

- Source: worker
- Scan: f695b239-71c7-4b01-b1c7-bb7c6b33c706
- Target: /Users/arthrod/temp/T/jubarte-redlines
- Result status: running
- Model origin: generated
- Model scope: .
- Model scope description: Architecture of authorized live repository /Users/arthrod/temp/T/jubarte-redlines; live HEAD observed 4e10eca69c42211651ca907f38b74b6d19e6928c, distinct from scan-directory label 9a0423c22ce6e195c32b886a3f45ccc48087d1dc.
- Scan scope: .

This is a provisional model saved before successful completion.
