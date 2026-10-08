/* tslint:disable */
/* eslint-disable */

export function diffDocumentsUnified(old: Uint8Array, new: Uint8Array, oldName?: string, newName?: string, context?: number): string;



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
 * Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline and
 * the per-operation report.
 *
 * Mirrors `jubarte::edit::apply_plan_json`.
 */
export function applyEditPlan(docx: Uint8Array, plan_json: string): EditOutput;

/**
 * Audit findings as JSON `{findings, rules, layout}` (see `jubarte audit`).
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
 * The changes from `old` to `new` as a patch, JSON `{"text", "hunks":
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
export function diffDocuments(old: Uint8Array, _new: Uint8Array, author: string, date: string, columns?: number | null, old_name?: string | null, new_name?: string | null): string;

/**
 * The complete document as CriticMarkup; existing paragraph patches stay separate.
 */
export function diffDocumentsCritic(old: Uint8Array, _new: Uint8Array, author?: string | null, date?: string | null): string;

/**
 * Document review view. `optionsJson` is a strict camelCase object with
 * `format` (github, word, normal, context, side-by-side), `oldName`,
 * `newName`, `context` (u32), `acceptChanges`, `fullLines`, `oldFormat`
 * and `newFormat` (docx/md). Defaults use the core display window; Word
 * always accepts both inputs' revisions before creating new CriticMarkup.
 */
export function diffDocumentsView(old: Uint8Array, _new: Uint8Array, options_json?: string | null): string;

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
 */
export function docxToPdf(docx: Uint8Array, compress?: boolean | null, revisions?: string | null, revision_palette?: string | null): Uint8Array;

/**
 * The JSON-lines form of a report (`load`, one `op` per operation,
 * `summary`), for agent logs.
 */
export function editReportJsonl(report_json: string): string;

/**
 * List the tracked revisions in a DOCX as a JSON array string — the same
 * object shape as the CLI `jubarte revisions --json` lines
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
 * `done`, `paragraph`, `anchor_text`, `before`, `after`). `author` keeps
 * one author's comments; `latest` keeps the newest comment of each thread.
 *
 * Mirrors `jubarte::comments::list_comments` and `select_comments`.
 */
export function listComments(docx: Uint8Array, author?: string | null, latest?: boolean | null): string;

/**
 * Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.
 *
 * `optionsJson` (every field optional): `page` (`"letter"` default, or
 * `"a4"`), `author` (`"Redline"`), `date` (fixed epoch, so the same Markdown
 * writes the same bytes), `critic` (`true`: CriticMarkup becomes tracked
 * changes and comments) and `track_changes` (or `trackChanges`: `"all"`,
 * `"accept"`, `"reject"`). An unknown field is an error. `reference`, a
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
 * Resolve every operation of an edit plan without producing documents.
 *
 * Mirrors `jubarte::edit::preview_plan`.
 */
export function previewEditPlan(docx: Uint8Array, plan_json: string): EditOutput;

/**
 * DOCX/Markdown comparison written as a Word redline, for host CLI I/O.
 */
export function redlineDocuments(old: Uint8Array, _new: Uint8Array, author: string, date: string): Uint8Array;

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
 * Refresh the cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and
 * `TOC` fields from jubarte's layout (page numbers are jubarte's, not
 * Word's). Full build only: it needs the layout the PDF export links.
 *
 * Mirrors `jubarte::fields::update_fields`.
 */
export function updateFields(docx: Uint8Array): FieldsOutput;

/**
 * Word-validity findings beyond the schema as a JSON array (`code`,
 * `part`, `path`, `message`, `word_fatal`, `repairable`); `[]` is a pass.
 *
 * Mirrors `jubarte::validate::validate`.
 */
export function validateDocument(docx: Uint8Array): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_appendoutput_free: (a: number, b: number) => void;
    readonly __wbg_editoutput_free: (a: number, b: number) => void;
    readonly __wbg_fieldsoutput_free: (a: number, b: number) => void;
    readonly __wbg_repairoutput_free: (a: number, b: number) => void;
    readonly acceptChanges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly acceptRevisions: (a: number, b: number, c: number) => void;
    readonly appendDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly appendoutput_docx: (a: number, b: number) => void;
    readonly appendoutput_warnings: (a: number, b: number) => void;
    readonly applyEditPlan: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly auditDocument: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly auditTracked: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly capabilities: (a: number) => void;
    readonly compareDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly diffDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number) => void;
    readonly diffDocumentsCritic: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly diffDocumentsUnified: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly diffDocumentsView: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly documentMarkdown: (a: number, b: number, c: number) => void;
    readonly documentMarkdownWithChanges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly docxToPdf: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly editReportJsonl: (a: number, b: number, c: number) => void;
    readonly editoutput_clean: (a: number, b: number) => void;
    readonly editoutput_json: (a: number, b: number) => void;
    readonly editoutput_ok: (a: number) => number;
    readonly editoutput_patch: (a: number, b: number) => void;
    readonly editoutput_redline: (a: number, b: number) => void;
    readonly fieldsoutput_docx: (a: number, b: number) => void;
    readonly fieldsoutput_json: (a: number, b: number) => void;
    readonly getRevisions: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly initPanicHook: () => void;
    readonly inspectDocument: (a: number, b: number, c: number) => void;
    readonly listChanges: (a: number, b: number, c: number) => void;
    readonly listComments: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly markdownToDocx: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly parseCli: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly pdfPageCount: (a: number, b: number) => number;
    readonly previewEditPlan: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly redlineDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly rejectChanges: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly rejectRevisions: (a: number, b: number, c: number) => void;
    readonly repairDocument: (a: number, b: number, c: number) => void;
    readonly repairoutput_docx: (a: number, b: number) => void;
    readonly repairoutput_json: (a: number, b: number) => void;
    readonly scrubDocument: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly sourceSha256: (a: number, b: number, c: number) => void;
    readonly updateFields: (a: number, b: number, c: number) => void;
    readonly validateDocument: (a: number, b: number, c: number) => void;
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
