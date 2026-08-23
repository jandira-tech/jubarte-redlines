/* tslint:disable */
/* eslint-disable */

/**
 * Accept every tracked revision (package-wide) → clean DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::accept_revisions`.
 */
export function acceptRevisions(docx: Uint8Array): Uint8Array;

/**
 * Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
 *
 * Mirrors `jubarte::document_comparer::compare_documents`.
 */
export function compareDocuments(original: Uint8Array, modified: Uint8Array, author: string): Uint8Array;

/**
 * Render a DOCX package (bytes) → PDF bytes (Word-style layout).
 *
 * Mirrors `jubarte::convert::docx_to_pdf`. Fonts come from the embedded
 * Carlito / Liberation set; the native system/cloud font overrides are
 * no-ops under wasm (no filesystem), which only changes glyph sourcing,
 * never layout metrics.
 * `compress` (optional, default `false`) deflates the PDF's streams
 * (`/FlateDecode`): much smaller output, no longer plain text.
 */
export function docxToPdf(docx: Uint8Array, compress?: boolean | null): Uint8Array;

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
 * Number of pages in a PDF (cheap object scan; `0` if the bytes are not a
 * readable PDF).
 *
 * Mirrors `jubarte::convert::pdf_page_count`.
 */
export function pdfPageCount(pdf: Uint8Array): number;

/**
 * Reject every tracked revision (package-wide) → base DOCX bytes.
 *
 * Mirrors `jubarte::document_comparer::reject_revisions`.
 */
export function rejectRevisions(docx: Uint8Array): Uint8Array;
