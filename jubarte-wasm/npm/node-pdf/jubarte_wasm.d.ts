/* tslint:disable */
/* eslint-disable */

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
export function docxToPdf(docx: Uint8Array | number[], compress?: boolean | null, revisions?: string | null, revision_palette?: string | null, move_comments?: boolean | null, changed_only?: boolean | null): Uint8Array;

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
export function pdfPageCount(pdf: Uint8Array | number[]): number;

/**
 * An edited document's PDF, compressed, its changes painted as
 * `revisions` (`conventional`, `word` or `custom` with
 * `revisionPalette`), with the page facts `edit --pdf` reports. Full build
 * only.
 *
 * Mirrors `jubarte::convert::render`.
 */
export function renderPdf(docx: Uint8Array | number[], revisions?: string | null, revision_palette?: string | null): RenderOutput;
