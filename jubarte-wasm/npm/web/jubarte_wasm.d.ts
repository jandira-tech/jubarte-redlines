/* tslint:disable */
/* eslint-disable */

export function diffDocumentsUnified(old: Uint8Array, next: Uint8Array, oldName?: string, newName?: string, context?: number): string;



/**
 * What [`appendDocuments`](append_documents) returns.
 */
export class AppendOutput {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The joined document.
     */
    readonly docx: Uint8Array;
    /**
     * What was not carried, as a JSON array of `CODE: message` strings
     * (`COMMENTS_DROPPED: ...`).
     */
    readonly warnings: string;
}

/**
 * What [`applyEditPlan`](apply_edit_plan) and
 * [`previewEditPlan`](preview_edit_plan) return. A refused plan is data, not
 * an exception, so every operation's outcome stays readable.
 */
export class EditOutput {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The edited document without tracked changes; `undefined` on refusal
     * and for previews.
     */
    readonly clean: Uint8Array | undefined;
    /**
     * The report JSON when `ok`, else the error JSON (`code`, `operation`,
     * `message`, `outcomes`).
     */
    readonly json: string;
    /**
     * `true` when the plan was applied (or resolved, for a preview).
     */
    readonly ok: boolean;
    /**
     * The changes the redline tracks as a patch (see
     * [`diffDocuments`](diff_documents)), by the plan's author and date;
     * `undefined` on refusal and for previews.
     */
    readonly patch: string | undefined;
    /**
     * The source compared against the clean copy (Word tracked changes);
     * `undefined` on refusal and for previews.
     */
    readonly redline: Uint8Array | undefined;
}

/**
 * What [`updateFields`](update_fields) returns.
 */
export class FieldsOutput {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The document with refreshed field results.
     */
    readonly docx: Uint8Array;
    /**
     * `{"page_count", "fields": [{"kind", "code", "paragraph", "old", "new"}]}`.
     */
    readonly json: string;
}

/**
 * What [`renderPdf`](render_pdf) returns.
 */
export class RenderOutput {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * `{"page_count", "page_starts"}`: the pages written and each one's
     * first line (60 characters at most), as an edit bundle's `render`
     * line lists them.
     */
    readonly json: string;
    /**
     * The PDF.
     */
    readonly pdf: Uint8Array;
}

/**
 * Output of [`repairDocument`](repair_document).
 */
export class RepairOutput {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The package with every repairable finding fixed.
     */
    readonly docx: Uint8Array;
    /**
     * `{"repaired": [...], "remaining": [...]}`: the findings fixed and the
     * ones the output still has.
     */
    readonly json: string;
}

/**
 * Accept the changes `filterJson` selects and keep the rest tracked, as
 * Word's Accept This Change does. The filter is `{"ids": [...],
 * "authors": [...], "kinds": [...]}`: a change is selected when it matches
 * every list given (`{}` selects every change; an empty list, none).
 *
 * Mirrors `jubarte::changes::accept_changes`.
 */
export function acceptChanges(docx: Uint8Array, filter_json: string): Uint8Array;

/**
 * Accept every tracked revision (package-wide) → clean DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::accept_revisions`.
 */
export function acceptRevisions(docx: Uint8Array): Uint8Array;

/**
 * Append B after A, carrying B's images, links, headers, styles, lists and
 * notes. `optionsJson` is `{"section_break": "next_page" | "continuous" |
 * "none", "keep_sections": bool, "comments": "drop" | "carry"}`, each
 * optional; B's comments are dropped (warned) unless `"carry"`.
 *
 * Mirrors `jubarte::append::append_documents`.
 */
export function appendDocuments(a: Uint8Array, b: Uint8Array, options_json?: string | null): AppendOutput;

/**
 * Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline,
 * the patch of the plan's changes and the per-operation report. `name`
 * (default `document.docx`) is the file the patch names; under
 * `existing_revisions: keep` the patch shows the plan's own changes, not
 * the other party's the redline also holds.
 *
 * Mirrors `jubarte::edit::apply_plan_json`.
 */
export function applyEditPlan(docx: Uint8Array, plan_json: string, name?: string | null): EditOutput;

/**
 * Audit findings as JSON `{findings, rules, layout}` (see `jubarte validate --rules`).
 * `rules` is a comma-separated list of rule sets (`a11y`, `style`,
 * `structure`) or codes; omitted or empty runs every rule. The slim build
 * has no layout pass: it leaves `FONT_SUBSTITUTED` out (naming it is an
 * error) and does not compare `NUMPAGES` caches with a page count.
 */
export function auditDocument(docx: Uint8Array, rules?: string | null): string;

/**
 * Every text change from `original` to `edited` must be a revision by
 * `author`; the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`) as a JSON
 * array.
 *
 * Mirrors `jubarte::validate::audit_tracked`.
 */
export function auditTracked(original: Uint8Array, edited: Uint8Array, author: string): string;

/**
 * What this build can do, as JSON (`runtime: "wasm"`): PDF and field
 * refresh only in the full build, PNG never.
 *
 * Mirrors `jubarte::capabilities::capabilities`.
 */
export function capabilities(): string;

/**
 * [`capabilities`] as the text summary `jubarte capabilities` prints; of
 * `manifestJson` when given (a manifest a host narrowed to what it does),
 * else of this build's.
 *
 * Mirrors `jubarte::capabilities::capabilities_text`.
 */
export function capabilitiesText(manifest_json?: string | null): string;

/**
 * `filterJson` (as [`acceptChanges`](accept_changes) takes it) with each id
 * in its long form (`r1+2` → `body:rev:1`, `body:rev:2`). An id that is no
 * change id throws `jubarte-wasm: c5: expected a change id …`, which the
 * binary reports as a usage error before reading the document.
 *
 * Mirrors `jubarte::changes::ChangeFilter::normalized`.
 */
export function changeFilter(filter_json: string): string;

/**
 * The blocks of `docx` (a redline) carrying `author`'s marks, as the agent
 * view `edit` and `add` print (every changed block when none does);
 * `accepted` reads its accepted text, `source` is the name printed as
 * `source:`.
 *
 * Mirrors `jubarte::markdown::changed_view`.
 */
export function changedView(docx: Uint8Array, author: string, accepted: boolean, source?: string | null): string;

/**
 * The listing `jubarte changes` prints: a line per change with its short
 * id (`r0`), kind, target, author, paragraph and text, then `N change(s)`.
 *
 * Mirrors `jubarte::changes::changes_text`.
 */
export function changesText(docx: Uint8Array): string;

/**
 * The listing `jubarte comments` prints: a line per comment with its short
 * id (`c5`), paragraph, author, text and anchor, then `N comment(s)`.
 * `author` and `latest` filter as in [`listComments`](list_comments).
 *
 * Mirrors `jubarte::comments::comments_text`.
 */
export function commentsText(docx: Uint8Array, author?: string | null, latest?: boolean | null): string;

/**
 * Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
 *
 * Mirrors `jubarte::document_comparer::compare_documents`.
 * `inputLimitsJson` (optional) overrides the admission budget key by key:
 * `{"max_compressed_bytes", "max_entries", "max_part_bytes",
 * "max_uncompressed_bytes", "max_xml_depth"}`. A package past the budget
 * throws with `INPUT_LIMIT`; an unknown key throws `invalid input limits`.
 * The default budget allows 2 GiB inflated, more than a 32-bit WASM heap
 * holds, so browser hosts should lower it.
 */
export function compareDocuments(original: Uint8Array, modified: Uint8Array, author: string, input_limits_json?: string | null): Uint8Array;

/**
 * A Word document as `convert --to md` writes it: Markdown with its
 * changes kept, accepted or rejected (`trackChanges`: `all`, `accept`,
 * `reject`), and unless `pageMarkers` is false `<!-- page N of M -->` lines
 * from the layout, its changes painted as `revisions`. JSON
 * `{"markdown", "warning"}`: the warning says why the markers are missing.
 * Full build only.
 *
 * Mirrors `jubarte::markdown::with_page_markers`.
 */
export function convertMarkdown(docx: Uint8Array, track_changes: string, page_markers: boolean, revisions?: string | null, revision_palette?: string | null): string;

/**
 * The changes from `old` to `next` as a patch, JSON `{"text", "hunks":
 * [{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
 * with `[-old-]{+new+}` changes and CriticMarkup comments, at its
 * `body:p:N` id in a Word document or `line:N` in Markdown.
 *
 * Each side is a `.docx` package or UTF-8 Markdown
 * (`new TextEncoder().encode(text)`). `author` and `date` (ISO 8601) own
 * the changes; `columns` wraps the lines (72 by default, 0 does not);
 * the names default to `old.docx`/`old.md` and `new.docx`/`new.md`.
 *
 * Mirrors `jubarte::markdown::patch_documents`.
 */
export function diffDocuments(old: Uint8Array, next: Uint8Array, author: string, date: string, columns?: number | null, old_name?: string | null, new_name?: string | null): string;

/**
 * The complete document as CriticMarkup; existing paragraph patches stay separate.
 */
export function diffDocumentsCritic(old: Uint8Array, next: Uint8Array, author?: string | null, date?: string | null): string;

/**
 * Document review view. `optionsJson` is a strict camelCase object with
 * `format` (github, word, normal, context, side-by-side), `oldName`,
 * `newName`, `context` (u32), `acceptChanges`, `fullLines`, `oldFormat`
 * and `newFormat` (docx/md). Defaults use the core display window; Word
 * always accepts both inputs' revisions before creating new CriticMarkup.
 */
export function diffDocumentsView(old: Uint8Array, next: Uint8Array, options_json?: string | null): string;

/**
 * Body paragraphs as Markdown, each preceded by its `[body:p:N]` id: the
 * coordinates an edit plan uses.
 *
 * Mirrors `jubarte::inspect::markdown`.
 */
export function documentMarkdown(docx: Uint8Array): string;

/**
 * Markdown without paragraph ids, with tracked changes kept or resolved.
 */
export function documentMarkdownWithChanges(docx: Uint8Array, track_changes: string): string;

/**
 * Render a DOCX package (bytes) → PDF bytes (Word-style layout).
 *
 * Mirrors `jubarte::convert::docx_to_pdf`. Fonts come from the embedded
 * Carlito / Liberation set; the native system/cloud font overrides are
 * no-ops under wasm (no filesystem), which only changes glyph sourcing,
 * never layout metrics.
 * `compress` (optional, default `false`) deflates the PDF's streams
 * (`/FlateDecode`): much smaller output, no longer plain text.
 * `revisions` (optional, default `"conventional"`) paints tracked changes:
 * `"conventional"`, `"word"` (Microsoft Word's markup) or `"custom"` with
 * `revisionPalette` (`"deleted=#AA0000:strike,..."`).
 * `moveComments` (optional, default `false`) lists the comments after the
 * last page instead of in balloons beside the text; `changedOnly`
 * (optional, default `false`) keeps only the pages a tracked change
 * touches, the pages a comment is anchored on and each section's first
 * page on which a revised header or footer paints (else the first page).
 */
export function docxToPdf(docx: Uint8Array, compress?: boolean | null, revisions?: string | null, revision_palette?: string | null, move_comments?: boolean | null, changed_only?: boolean | null): Uint8Array;

/**
 * The JSON-lines form of a report (`load`, one `op` per operation,
 * `summary`), for agent logs.
 */
export function editReportJsonl(report_json: string): string;

/**
 * Why `--editing-mode` cannot apply `plan` to `docx`: the tracked changes
 * it keeps and does not itself accept or reject; `undefined` when none.
 *
 * Mirrors `jubarte::edit::flags::editing_mode_conflict`.
 */
export function editingModeConflict(plan: string, docx: Uint8Array): string | undefined;

/**
 * The plan the `edit`/`add` operation flags describe, as JSON `{plan,
 * notes}`: `plan` is the plan's JSON for [`applyEditPlan`](apply_edit_plan),
 * `notes` the `op-N: …` lines to print. `verb` is `edit` or `add`;
 * `operationsJson` the parsed command's `operations` array. Options (JSON,
 * all optional): `author` (default `Redline`), `date`,
 * `existingRevisions` (auto, keep, accept, reject, refuse; default auto).
 * An error is a usage error.
 *
 * Mirrors `jubarte::edit::flags::plan_from_flags`.
 */
export function flagPlan(verb: string, operations_json: string, docx: Uint8Array, options_json?: string | null): string;

/**
 * List the tracked revisions in a DOCX as a JSON array string, grouped as
 * Docxodus's `GetRevisions` groups them; each object has
 * (`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).
 *
 * Mirrors `jubarte::document_comparer::get_revisions` with default settings,
 * serialized by the shared `revisions_to_json`. `inputLimitsJson` as in
 * `compareDocuments`.
 */
export function getRevisions(docx: Uint8Array, input_limits_json?: string | null): string;

/**
 * One-shot init: panic hook → `console.error`. Safe to call multiple times.
 */
export function initPanicHook(): void;

/**
 * The inspection snapshot as JSON: `schema_version`, `source_sha256`,
 * `summary` and `paragraphs` (ids, text, style, formatting spans,
 * limitations). Oversized or malformed packages are refused before parsing.
 *
 * Mirrors `jubarte::inspect::inspect_json`.
 */
export function inspectDocument(docx: Uint8Array): string;

/**
 * List the tracked changes one by one as a JSON array string, each with the
 * id `acceptChanges` / `rejectChanges` select by (the same objects as
 * `jubarte changes --json`: `id`, `kind`, `target`, `author`, `date`,
 * `text`, `move_name`, `move_side`, `inside`).
 *
 * Mirrors `jubarte::changes::list_changes`.
 */
export function listChanges(docx: Uint8Array): string;

/**
 * List every comment as a JSON array string (the objects `jubarte comments
 * --json` prints: `id`, `author`, `initials`, `date`, `text`, `parent`,
 * `done`, `paragraph`, `last_paragraph` when the range ends in a later
 * paragraph, `anchor_text`, `before`, `after`). `author` keeps one
 * author's comments; `latest` keeps the newest comment of each thread.
 *
 * Mirrors `jubarte::comments::list_comments` and `select_comments`.
 */
export function listComments(docx: Uint8Array, author?: string | null, latest?: boolean | null): string;

/**
 * The author and timestamp a Markdown document gives the changes and
 * comments that name none, as JSON `[author, date]` (`null` for each it
 * does not tell): its front matter, else the reference `.docx`'s creator
 * and creation time. The CLI falls back to `Redline` and now.
 */
export function markdownStamp(text: string, reference?: Uint8Array | null): string;

/**
 * Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.
 *
 * `optionsJson` (every field optional): `page` (`"letter"` default, or
 * `"a4"`), `author` and `date` (for a change or comment that names none;
 * given, they beat the front matter's `author:` and `last_modified:`,
 * `modified:`, `date:`, `created:`, which beat `"Redline"` and the fixed
 * epoch, so the same Markdown writes the same bytes; the CLI dates them now
 * instead), `critic` (`true`: CriticMarkup becomes tracked changes and
 * comments), `track_changes` (or `trackChanges`: `"all"`,
 * `"accept"`, `"reject"`), `tracking` (`true`/`false`: Word's Track
 * Changes switch in the output, whatever the front matter's
 * `track_changes:` says) and `scholar` (`true`: set as LaTeX's article
 * class, page numbers from page 2, as `--scholar`). An unknown field is
 * an error. `reference`, a
 * `.docx`, lends its styles and page setup, and then `page` is ignored.
 * Images are written as their alt text, and the engine's warnings are not
 * returned.
 */
export function markdownToDocx(text: string, options_json?: string | null, reference?: Uint8Array | null): Uint8Array;

/**
 * Shared clap parsing, with no filesystem, clock or process access.
 */
export function parseCli(arguments_json: string, program?: string | null, supported_json?: string | null): string;

/**
 * Number of pages in a PDF (cheap object scan; `0` if the bytes are not a
 * readable PDF).
 *
 * Mirrors `jubarte::convert::pdf_page_count`.
 */
export function pdfPageCount(pdf: Uint8Array): number;

/**
 * Why `planJson` is no edit plan, as the binary prints it (`INVALID_PLAN:
 * …`); `undefined` for a plan. A plan that parses can still be refused when
 * applied; that refusal carries the per-operation report.
 *
 * Mirrors `jubarte::edit::EditPlan::from_json`.
 */
export function planError(plan_json: string): string | undefined;

/**
 * Resolve every operation of an edit plan without producing documents.
 *
 * Mirrors `jubarte::edit::preview_plan`.
 */
export function previewEditPlan(docx: Uint8Array, plan_json: string): EditOutput;

/**
 * The snapshot `jubarte read --json` prints, with its `changes` and
 * `comments`. Options (JSON, all optional) select as in
 * [`readView`](read_view): `paragraphs`, `head`, `tail`, `changed`, `by`;
 * `source` names the document. The snapshot keeps the selected paragraphs
 * and their changes and comments.
 *
 * Mirrors `jubarte::markdown::read_snapshot`.
 */
export function readJson(docx: Uint8Array, options_json?: string | null): string;

/**
 * The agent view (`read`) as JSON `{markdown, warnings}`: a YAML header,
 * then Markdown with `<!-- pN -->` id lines and every change and comment
 * with its id. Options (JSON, all optional): `trackChanges` (all, accept,
 * reject), `comments` (inline, none), `dates`, `pageMarkers` (default
 * true: the layout pass numbers the pages), `paragraphs` (`"p3,p10-p20,t0"`),
 * `head`, `tail`, `changed` (only the blocks with marks) with `by` (one
 * author's: a handle or a full name), `source` (the name printed as
 * `source:`).
 *
 * Mirrors `jubarte::markdown::read`.
 */
export function readView(docx: Uint8Array, options_json?: string | null): string;

/**
 * `edit`/`add --plan PLAN.json` with operation flags: the operations
 * recorded into `held` (PLAN.json's text, absent before the first
 * recording), as JSON `{ok: true, recorded: {plan, json, notes,
 * summary}}`: preview `plan`, write `json`, print `notes` and `summary`.
 * On failure `{ok: false, usage, message}`: a usage error (exit 2), or
 * `held` is not a plan (exit 3). Options (JSON): `author`, `date`,
 * `existingRevisions`, `file`, `plan`, `defaultPlan` (a bare `--plan`),
 * `dryRun`.
 *
 * Mirrors `jubarte::edit::record::record`.
 */
export function recordPlan(verb: string, operations_json: string, docx: Uint8Array, options_json: string, held?: string | null): string;

/**
 * DOCX/Markdown comparison written as a Word redline, for host CLI I/O.
 */
export function redlineDocuments(old: Uint8Array, next: Uint8Array, author: string, date: string): Uint8Array;

/**
 * The error line for an input no task reads, as the binary prints it:
 * `LEGACY_DOC: a.docx is a Word 97-2003 …`, or (`word`) `INVALID_PACKAGE:
 * a.docx is not a ZIP file: …`; `undefined` when the bytes may be read.
 *
 * Mirrors `jubarte::cli::refused_input`.
 */
export function refusedInput(path: string, bytes: Uint8Array, word: boolean): string | undefined;

/**
 * Reject the changes `filterJson` selects and keep the rest tracked
 * (filter as in `acceptChanges`).
 *
 * Mirrors `jubarte::changes::reject_changes`.
 */
export function rejectChanges(docx: Uint8Array, filter_json: string): Uint8Array;

/**
 * Reject every tracked revision (package-wide) → base DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::reject_revisions`.
 */
export function rejectRevisions(docx: Uint8Array): Uint8Array;

/**
 * An edited document's PDF, compressed, its changes painted as
 * `revisions` (`conventional`, `word` or `custom` with
 * `revisionPalette`), with the page facts `edit --pdf` reports. Full build
 * only.
 *
 * Mirrors `jubarte::convert::render`.
 */
export function renderPdf(docx: Uint8Array, revisions?: string | null, revision_palette?: string | null): RenderOutput;

/**
 * The package with every repairable finding fixed, with the findings it
 * fixed and could not fix in `json`.
 *
 * Mirrors `jubarte::validate::repair`.
 */
export function repairDocument(docx: Uint8Array): RepairOutput;

/**
 * Remove who touched a document: author names (as one alias), rsids, the
 * people and dates in the document properties, and comments.
 * `optionsJson` is `{"author_alias": string, "rsids": bool, "docprops":
 * bool, "comments": bool}`, a field left out off; without it, everything
 * goes under the alias `Author`.
 *
 * Mirrors `jubarte::scrub::scrub`.
 */
export function scrubDocument(docx: Uint8Array, options_json?: string | null): Uint8Array;

/**
 * SHA-256 (lowercase hex) of the bytes: the `source_sha256` guard an edit
 * plan carries.
 *
 * Mirrors `jubarte::inspect::source_sha256`.
 */
export function sourceSha256(docx: Uint8Array): string;

/**
 * The grids `jubarte read --tables` prints: a `table N: RxC …` line per
 * body table, then a line per row of `ids=text` cells; `no tables` when
 * there are none.
 *
 * Mirrors `jubarte::inspect::tables_text`.
 */
export function tablesText(docx: Uint8Array): string;

/**
 * Refresh the cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and
 * `TOC` fields from jubarte's layout (page numbers are jubarte's, not
 * Word's). Full build only: it needs the layout the PDF export links.
 *
 * Mirrors `jubarte::fields::update_fields`.
 */
export function updateFields(docx: Uint8Array): FieldsOutput;

/**
 * Word-validity findings beyond the schema as a JSON array (`code`, `set`,
 * `severity`, `part`, `location`, `ref`, `message`, `word_fatal`,
 * `repairable`; `ref` is the short id the CLI prints for `location`);
 * `[]` is a pass.
 *
 * Mirrors `jubarte::validate::validate`.
 */
export function validateDocument(docx: Uint8Array): string;

/**
 * The line a writing task prints on stderr for the file it wrote:
 * `wrote r.pdf (pdf, 48213 bytes, 3 pages)`. `format` is `docx`, `md`,
 * `pdf`, `png` or `json`; `pages` is a PDF's.
 *
 * Mirrors `jubarte::cli::wrote_line`.
 */
export function wroteLine(path: string, format: string, size: number, pages?: number | null): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_appendoutput_free: (a: number, b: number) => void;
    readonly __wbg_editoutput_free: (a: number, b: number) => void;
    readonly __wbg_fieldsoutput_free: (a: number, b: number) => void;
    readonly __wbg_renderoutput_free: (a: number, b: number) => void;
    readonly __wbg_repairoutput_free: (a: number, b: number) => void;
    readonly acceptChanges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly acceptRevisions: (a: number, b: number, c: number) => void;
    readonly appendDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly appendoutput_docx: (a: number, b: number) => void;
    readonly appendoutput_warnings: (a: number, b: number) => void;
    readonly applyEditPlan: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly auditDocument: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly auditTracked: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly capabilities: (a: number) => void;
    readonly capabilitiesText: (a: number, b: number, c: number) => void;
    readonly changeFilter: (a: number, b: number, c: number) => void;
    readonly changedView: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly changesText: (a: number, b: number, c: number) => void;
    readonly commentsText: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly compareDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly convertMarkdown: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly diffDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
    readonly diffDocumentsCritic: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly diffDocumentsUnified: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly diffDocumentsView: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly documentMarkdown: (a: number, b: number, c: number) => void;
    readonly documentMarkdownWithChanges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly docxToPdf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly editReportJsonl: (a: number, b: number, c: number) => void;
    readonly editingModeConflict: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly editoutput_clean: (a: number, b: number) => void;
    readonly editoutput_json: (a: number, b: number) => void;
    readonly editoutput_ok: (a: number) => number;
    readonly editoutput_patch: (a: number, b: number) => void;
    readonly editoutput_redline: (a: number, b: number) => void;
    readonly fieldsoutput_docx: (a: number, b: number) => void;
    readonly fieldsoutput_json: (a: number, b: number) => void;
    readonly flagPlan: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly getRevisions: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly initPanicHook: () => void;
    readonly inspectDocument: (a: number, b: number, c: number) => void;
    readonly listChanges: (a: number, b: number, c: number) => void;
    readonly listComments: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly markdownStamp: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly markdownToDocx: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly parseCli: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly pdfPageCount: (a: number, b: number) => number;
    readonly planError: (a: number, b: number, c: number) => void;
    readonly previewEditPlan: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly readJson: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly readView: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly recordPlan: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly redlineDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly refusedInput: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly rejectChanges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly rejectRevisions: (a: number, b: number, c: number) => void;
    readonly renderPdf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly renderoutput_json: (a: number, b: number) => void;
    readonly renderoutput_pdf: (a: number, b: number) => void;
    readonly repairDocument: (a: number, b: number, c: number) => void;
    readonly repairoutput_docx: (a: number, b: number) => void;
    readonly repairoutput_json: (a: number, b: number) => void;
    readonly scrubDocument: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly sourceSha256: (a: number, b: number, c: number) => void;
    readonly tablesText: (a: number, b: number, c: number) => void;
    readonly updateFields: (a: number, b: number, c: number) => void;
    readonly validateDocument: (a: number, b: number, c: number) => void;
    readonly wroteLine: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
