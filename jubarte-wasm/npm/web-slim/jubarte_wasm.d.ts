/* tslint:disable */
/* eslint-disable */

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
     * The source compared against the clean copy (Word tracked changes);
     * `undefined` on refusal and for previews.
     */
    readonly redline: Uint8Array | undefined;
}

/**
 * Accept every tracked revision (package-wide) → clean DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::accept_revisions`.
 */
export function acceptRevisions(docx: Uint8Array): Uint8Array;

/**
 * Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline and
 * the per-operation report.
 *
 * Mirrors `jubarte::edit::apply_plan_json`.
 */
export function applyEditPlan(docx: Uint8Array, plan_json: string): EditOutput;

/**
 * What this build can do, as JSON (`runtime: "wasm"`): PDF only in the full
 * build, PNG never.
 *
 * Mirrors `jubarte::capabilities::capabilities`.
 */
export function capabilities(): string;

/**
 * Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
 *
 * Mirrors `jubarte::document_comparer::compare_documents`.
 */
export function compareDocuments(original: Uint8Array, modified: Uint8Array, author: string): Uint8Array;

/**
 * Body paragraphs as Markdown, each preceded by its `[body:p:N]` id: the
 * coordinates an edit plan uses.
 *
 * Mirrors `jubarte::inspect::markdown`.
 */
export function documentMarkdown(docx: Uint8Array): string;

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
 * serialized by the shared `revisions_to_json`.
 */
export function getRevisions(docx: Uint8Array): string;

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
 * Resolve every operation of an edit plan without producing documents.
 *
 * Mirrors `jubarte::edit::preview_plan`.
 */
export function previewEditPlan(docx: Uint8Array, plan_json: string): EditOutput;

/**
 * Reject every tracked revision (package-wide) → base DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::reject_revisions`.
 */
export function rejectRevisions(docx: Uint8Array): Uint8Array;

/**
 * SHA-256 (lowercase hex) of the bytes: the `source_sha256` guard an edit
 * plan carries.
 *
 * Mirrors `jubarte::inspect::source_sha256`.
 */
export function sourceSha256(docx: Uint8Array): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_editoutput_free: (a: number, b: number) => void;
    readonly acceptRevisions: (a: number, b: number, c: number) => void;
    readonly applyEditPlan: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly capabilities: (a: number) => void;
    readonly compareDocuments: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly documentMarkdown: (a: number, b: number, c: number) => void;
    readonly editReportJsonl: (a: number, b: number, c: number) => void;
    readonly editoutput_clean: (a: number, b: number) => void;
    readonly editoutput_json: (a: number, b: number) => void;
    readonly editoutput_ok: (a: number) => number;
    readonly editoutput_redline: (a: number, b: number) => void;
    readonly getRevisions: (a: number, b: number, c: number) => void;
    readonly inspectDocument: (a: number, b: number, c: number) => void;
    readonly previewEditPlan: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly rejectRevisions: (a: number, b: number, c: number) => void;
    readonly sourceSha256: (a: number, b: number, c: number) => void;
    readonly initPanicHook: () => void;
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
