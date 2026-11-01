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
 * Check that every text change from `original` to `edited` is a revision
 * by `author`. Returns the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`,
 * `FOREIGN_COMMENT` for another author's comment gone, reworded, resolved
 * or reopened, or a new comment not signed `author`, and
 * `PERMISSION_RANGE_DROPPED` for a permission endpoint of `original` that
 * `edited` lost) as a JSON array.
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
 * Focused edit discovery; no filters returns this runtime's edit support matrix.
 */
export function capabilityQuery(operation?: string | null, field?: string | null): string;

/**
 * Render a focused query in the same text form as the native CLI.
 */
export function capabilityQueryText(operation?: string | null, field?: string | null): string;

/**
 * `filterJson` (as [`acceptChanges`](accept_changes) takes it) with each id
 * in its settled spelling (`r1+2` → `r1`, `r2`). An id that is no change
 * id, the expired `body:rev:1` among them, throws `jubarte-wasm: c5:
 * expected a change id …` (naming `r1` for `body:rev:1`), which the
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
 * id (`r0`), kind, target, author, paragraph and text (then `(N elements)`
 * for a change Word wrote as several marks), then `N change(s)`.
 * `authors` filters as in [`listChanges`](list_changes).
 *
 * Mirrors `jubarte::changes::changes_text`.
 */
export function changesText(docx: Uint8Array, authors?: string[] | null): string;

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
 * "max_uncompressed_bytes", "max_xml_depth", "max_xml_elements",
 * "max_story_units"}`. A package past the budget
 * throws with `INPUT_LIMIT`; an unknown key throws `invalid input limits`.
 * The default budget allows 2 GiB inflated, more than a 32-bit WASM heap
 * holds, so browser hosts should lower it.
 */
export function compareDocuments(original: Uint8Array, modified: Uint8Array, author: string, input_limits_json?: string | null): Uint8Array;

/**
 * The changes from `old` to `next` as a patch, JSON `{"text", "hunks":
 * [{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
 * with `[-old-]{+new+}` changes and CriticMarkup comments, at its
 * `pN` id in a Word document or `line:N` in Markdown. When a side is a
 * Word document, a replacement inside a paragraph reads insertion first,
 * `{+new+}[-old-]`, as Word Compare writes it.
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
 * Body paragraphs as Markdown, each preceded by its `[pN]` id: the
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
 * The JSON-lines form of a report (`load`, one `op` per operation,
 * `summary`), for agent logs.
 */
export function editReportJsonl(report_json: string): string;

/**
 * Why `--editing-mode` cannot apply `plan` to `docx`: the tracked changes
 * it keeps and does not itself accept or reject, or a reply to a comment
 * on a paragraph it deletes; `undefined` when neither.
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
 * The working-document JSON `jubarte read --for-edit` prints
 * (`jubarte-read-for-edit-v1`): source hash, paired target ids, exact
 * working text with `before_sha256`, and per-operation readiness. Options
 * (JSON, all optional): `paragraphs` (`"p3,p10-p20,t0"`), `head`, `tail`,
 * `find`/`regex` (+ `ignoreCase`) on the original final text, `changed`
 * with `by`, `context` (read-only original neighbours), `limit`,
 * `maxChars`, `all`, `cursor`, `acceptChange`/`rejectChange` (revision
 * ids), `existingRevisions` (`keep`, `accept`, `reject`, `refuse`,
 * `auto`) and `track` (`on`, `off`); `validityFile`, the path as the
 * command line spells it, adds `input_validity`. A `USAGE` failure throws
 * `jubarte-wasm: USAGE: …`, as the CLI exits 2; the other refusals throw
 * `jubarte-wasm: CODE: …`, as the CLI exits 3.
 *
 * Mirrors `jubarte::edit::for_edit::for_edit_json`.
 */
export function forEdit(docx: Uint8Array, options_json?: string | null): string;

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
 * The one stderr line `read` and `changes` give an input with validity
 * findings (`input has 2 validity warnings; jubarte validate FILE lists
 * them`), or `undefined` when there is none or the package cannot be
 * validated; `file` is the path as the command line spells it.
 *
 * Mirrors `jubarte::validate::input_notice`.
 */
export function inputNotice(docx: Uint8Array, file: string): string | undefined;

/**
 * The JSON status `changes --json` and `comments --json` print on stderr
 * unless quiet, with the input's validity findings counted (`{"input_validity":{"errors":0,"warnings":2,
 * "next":"jubarte validate FILE"}}`); `file` is the path as the command
 * line spells it. A package that cannot be validated counts none.
 *
 * Mirrors `jubarte::validate::input_validity_line`.
 */
export function inputValidityLine(docx: Uint8Array, file: string): string;

/**
 * The inspection snapshot as JSON: `schema_version`, `source_sha256`,
 * `summary`, `paragraphs` (ids, text, style, formatting spans,
 * limitations) and top-level `tables` with recursive `nested` entries.
 * Snapshot schema 3 lists logical changes (one entry, with `members`, for
 * a change Word wrote as several marks), counts top-level tables, and gives
 * nested entries `parent` cell paths and no `index`. Oversized or malformed packages are refused before parsing.
 *
 * Mirrors `jubarte::inspect::inspect_json`.
 */
export function inspectDocument(docx: Uint8Array): string;

/**
 * List the tracked changes as a JSON array string, one object per
 * logical change: a change Word wrote as several marks is one object
 * with a composite id (`r1+2`) and its `members`, and an author's
 * deletion followed by their insertion is one `replacement`. Each `id`
 * is one `acceptChanges` / `rejectChanges` select by. The objects are
 * the ones `jubarte changes --json` prints: `id`, `ref` (the same id),
 * `kind` (`insertion`, `deletion`, `move`, `formatting` or
 * `replacement`), `target`, `author`, `date`, `text`, `paragraph`,
 * `move_name`, `move_side`, `inside`, `holds_comments` when the change
 * holds comment marks, and `members` (each mark's `id`, `kind`,
 * `target`, `date` and `text`) for a change of several marks.
 * `authors` keeps those authors' changes (exact names), as `jubarte
 * changes --author`; omitted or empty, every change.
 *
 * Mirrors `jubarte::changes::list_logical_changes` and `by_authors`.
 */
export function listChanges(docx: Uint8Array, authors?: string[] | null): string;

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
 * `source` names the document; `trackChanges` `accept` or `reject`
 * settles the text and names the choice in `track_changes`;
 * `validityFile`, the path as the command line spells it, adds
 * `summary.input_validity`, the input's validity findings counted. The
 * snapshot keeps the selected paragraphs and their changes and comments.
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
 * `dryRun`, `rewriteOthersComments`.
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
 * The package with every repairable finding fixed, with the findings it
 * fixed and could not fix in `json`.
 *
 * Mirrors `jubarte::validate::repair`.
 */
export function repairDocument(docx: Uint8Array): RepairOutput;

/**
 * The lines `edit -p rN --accept/--reject` prints for a pure resolution:
 * one per resolved change from `source` (the bytes before resolution) and
 * `resolvedJson` (`{"accepted": [...], "rejected": [...]}` short ids).
 *
 * Mirrors `jubarte::edit::resolution::resolution_text`.
 */
export function resolutionText(docx: Uint8Array, resolved_json: string): string;

/**
 * The line an error prints on stdout under `--json`:
 * `{"error":{"code":…,"message":…}}`, its code the message's leading
 * `CODE:`, else `USAGE` for an exit 2 and `ERROR` for any other.
 *
 * Mirrors `jubarte::cli::runtime_json_error`.
 */
export function runtimeJsonError(message: string, exit: number): string;

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
 * top-level body table, then a line per row of `ids=text` cells; nested
 * tables follow their cell row, indented two spaces per level and labelled
 * `nested in CELL_PATH`. `no tables` prints when there are none. Options (JSON, all optional): `trackChanges` (`reject`
 * reads the rejected text).
 *
 * Mirrors `jubarte::markdown::read_tables` without a selection.
 */
export function tablesText(docx: Uint8Array, options_json?: string | null): string;

/**
 * What `jubarte read --tables` prints, as JSON `{markdown, warnings}`:
 * the grids (`tablesText`), or with `paragraphs` (`"t0"`, `"p11"`,
 * `"p3-p20"`) the grids of the tables the view's selection reaches (a
 * `pN` inside a table, that table), each with the tables nested in it,
 * then an empty line and the view `readView` prints of that selection.
 * Options (JSON, all optional): `trackChanges`, `comments`, `dates`,
 * `pageMarkers`, `paragraphs`, `source`, as `readView` reads them. A pick
 * that reaches no table throws `jubarte-wasm: USAGE: …`, as the CLI
 * exits 2.
 *
 * Mirrors `jubarte::markdown::read_tables`.
 */
export function tablesView(docx: Uint8Array, options_json?: string | null): string;

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
 * Whether a command line (without the program name) asks for JSON: a
 * real `--json` flag before any `--`, read as the binary reads it (the
 * `--json` in `--text --json` is a value). An error then prints its
 * [`runtime_json_error`] line on stdout too.
 *
 * Mirrors `jubarte::cli::wants_json`.
 */
export function wantsJson(_arguments: string[]): boolean;

/**
 * The line a writing task prints on stderr for the file it wrote:
 * `wrote r.pdf (pdf, 48213 bytes, 3 pages)`. `format` is `docx`, `md`,
 * `pdf`, `png` or `json`; `pages` is a PDF's.
 *
 * Mirrors `jubarte::cli::wrote_line`.
 */
export function wroteLine(path: string, format: string, size: number, pages?: number | null): string;
