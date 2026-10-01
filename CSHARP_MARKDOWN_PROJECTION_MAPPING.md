<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# C# Markdown Projection → Rust Mapping Study (Swarm Report)

Two-phase subagent swarm study, run 2026-10-01 against commit `b493cb58`.

**Sources mapped** (read-only, outside the repo):

- `WmlToMarkdownConverter.cs` (Docxodus, MIT — John Scrudato IV) — the *oracle*:
  anchor-addressed markdown projection over the live OOXML package.
- `IrMarkdownEmitter.cs` (Docxodus.Ir) — an IR-consuming re-implementation whose
  goal is byte-equivalence with the oracle.

**Target:** this repository (`jubarte-redlines`, lib `jubarte`).

**Method.** Phase 1: 17 mapper subagents, one per tiny fraction of the two C#
files (11 oracle fractions W1–W11, 6 IR fractions I1–I6). Each mapped every C#
construct to the Rust representation with `file:line` evidence on both sides and
scored **fidelity** (how faithfully Rust reproduces the fraction today) and
**enhancement** (how far Rust goes beyond the C#). Phase 2: 6 solution-proposer
subagents, one per gap cluster (A anchors, B scopes, C inline, D
classification/numbering, E tables, F architecture), each grounded in the
Phase-1 findings.

---

## 1. Executive summary

| Fraction | Scope | Fidelity | Enhancement |
| --- | --- | --- | --- |
| W1 | Settings & enums | 0.3 | 0.4 |
| W2 | Anchor & projection types | 0.2 | 0.3 |
| W3 | Entry points & scope enumeration | 0.2 | 0.4 |
| W4 | Anchor-index loop & id rendering | 0.4 | 0.2 |
| W5 | KindFor / IsHeading / IsListItem | 0.5 | 0.3 |
| W6 | Scope orchestration, notes & comments | 0.3 | 0.2 |
| W7 | Block & paragraph emission | 0.4 | 0.5 |
| W8 | Inline grouping & formatting | 0.55 | 0.8 |
| W9 | Hyperlinks, run text & escaping | 0.4 | 0.5 |
| W10 | List items & numbering markers | 0.5 | 0.4 |
| W11 | Tables | 0.3 | 0.5 |
| I1 | IR emitter structure & anchor index | 0.1 | 0.1 |
| I2 | Part-URI resolution, AnchorIdMap port, index walk | 0.2 | 0.2 |
| I3 | Auto-number resolver, previews, text predicates | 0.3 | 0.3 |
| I4 | IR markdown emission | 0.3 | 0.3 |
| I5 | IR inline grouping & run text | 0.5 | 0.5 |
| I6 | IR tables, heading level, anchor prefix | 0.2 | 0.2 |
| | **Average** | **0.33** | **0.36** |

Headline findings:

1. **The anchor core is absent.** No `{#kind:scope:unid}` tokens, no
   `AnchorIndex`, no `AnchorIdMap`/Abbreviated/Sequential/dual-keying anywhere
   in `src/`. `src/unid.rs` exists but serves the comparer only (counter path,
   no deterministic mode, no lookup-by-unid). Rust addresses content
   positionally (`{story}:p:{index}` in `src/inspect.rs` + SHA-256 snapshot
   guards in `src/edit.rs`), which is snapshot-scoped, not stable.
2. **Different lineage.** Rust's Word→Markdown path (`src/markdown/from_docx/`)
   derives from the anymd project (MIT), not from Docxodus. It is body-only
   (headers/footers never projected), CriticMarkup-oriented, with a 3-key
   inline format (no strike, no code), decimal-only list markers, and
   always-GFM tables with no opaque fallback.
3. **Byte-level divergences are pervasive**: minimal contextual escaping vs the
   oracle's escape-all; `"\t"` vs 4 spaces; `"\\\n"` vs `"  \n"` hard breaks;
   `_` vs `*` italic; compact `|a|b|` vs `| a | b |` tables; sequential `[^N]`
   vs unid-stable `[^fn-…]` note labels; heading clamp 1–6 vs 1–9.
4. **Where Rust is ahead**: style-chain emphasis resolution, CriticMarkup with
   author/date attribution, field/hyperlink machinery, media extraction,
   marker-width list indentation, revision-aware table cells, deterministic
   story ordering, and the inspect `Projection`/`Segment`/`Piece` text
   back-map.
5. **No projection IR exists** in Rust (the comparer's atom IR proves one is
   feasible). The C# project built its IR emitter specifically to reach
   byte-equivalence with the oracle via a second implementation — a strategy
   Phase 2 recommends adopting (Cluster F).

---

## 2. Phase 1 — per-fraction mapping reports

### W1: settings & enums

Summary: The C# settings surface is a ten-knob, anchor-centered projection
configuration; Rust has only a two-knob `MarkdownOptions` (tracked-changes mode
+ media dir) on the ANYMD lineage, with a separate positional-id projection in
`inspect.rs` that partially plays the anchor role.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `ProjectionScopes` [Flags] (WmlToMarkdownConverter.cs:19-29) | hardwired scopes: `src/markdown/from_docx/mod.rs:100-122` (body+notes+comments, no headers/footers); `src/inspect.rs:191-198,117-133` (body+header/footer/note stories, no comments) | missing | No [Flags]/bitflags selection anywhere; each projection fixes its own scope set; neither equals `All`. |
| `AnchorRenderMode` (:34-42) | `inspect::render_markdown` (`src/inspect.rs:215-236`) always emits block ids `[body:p:N]` | divergent | Block-style only, not toggleable; no inline span anchors, no None. |
| `TableRenderMode` (:47-55) | `markdown_table` (`src/markdown/from_docx/mod.rs:241-266`), flattening at :1029-1042 | missing | Always GFM with flattening/loss; no opaque-anchor fallback. |
| `TrackedChangeMode` (:60-68) | `markdown::TrackChanges` (`src/markdown/mod.rs:52-73`, All/Accept/Reject) → `from_docx::Revisions` (`from_docx/mod.rs:25-34`, Markup/Accept/Reject) | partial | `RenderInline`≈`All`/`Markup` (CriticMarkup `{+ins+}`/`{-del-}` plus author/date attribution — richer); `Accept`≈`Accept`; `StripDeletions` absent; Rust adds `Reject`. Defaults differ: C# `Accept`, Rust `All`. |
| `WmlToMarkdownConverterSettings` (:74-134) | `MarkdownOptions` (`src/markdown/mod.rs:146-156`) + `from_docx::Options` (`from_docx/mod.rs:36-42`) | partial | 2 of 10 knobs. |
| `HeadingLevelOffset` (:83) | — heading level fixed by style/outline (`from_docx/mod.rs:695-715`) | missing | No offset; clamped 1–6. |
| `AnchorMode` (:86) | — | missing | |
| `TableInlineCellMax` (:96) | — no cell-length cap | missing | |
| `ResolveNumbering` (:111, default true) | always-on: `Numbering::parse` (from_docx/mod.rs:94-99), `list_marker` (:737-746) | partial | Semantics = always true; no off-switch. |
| `ImageUriBuilder` (:118, never invoked) | `MarkdownOptions::extract_media` (mod.rs:152-155) → `media::Extracted` (`src/markdown/from_docx/media.rs:17-57`) | divergent | C# is a dead placeholder; Rust actually extracts bytes. No URI-shaping callback; no CLI flag. |
| `EmptyParagraphMode` (:128, :137-147) | from_docx: blank paragraphs skipped (`from_docx/mod.rs:707-727`); inspect: anchor-only line kept (`src/inspect.rs:903-906`) | divergent | Two hardwired behaviors ≈ Suppress vs AnchorOnly; no `MarkedEmpty` (∅). |
| `AnchorIdRendering` (:151-171) | positional ids `{story}:p:{index}` (`src/inspect.rs:35`; `unified.rs` `Locator::Paragraph`) | missing | Opposite philosophy: no Unids, no abbreviated/sequential token budgeting. |

- **Fidelity: 0.3** — TrackedChanges maps well; the anchor core (Scopes,
  AnchorRenderMode, AnchorIdRendering, TableRenderMode/TableInlineCellMax,
  HeadingLevelOffset) is absent.
- **Enhancement: 0.4** — real image extraction, pandoc CLI parity, `Reject`
  mode, inline author/date attribution, snapshot ids wired to executable edit
  plans.
- Confidence: high.

### W2: anchor & projection types

Summary: None of `Anchor`/`AnchorTarget`/`MarkdownProjection`/
`WmlDocument.ConvertToMarkdown` exists in Rust; jubarte addresses content by
story+index ids (`body:p:3`) instead of unid anchors, and its markdown
projection returns bare text with no addressable index.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `Anchor` record + `Token` (cs:179–191) | `Locator` enum (`src/markdown/unified.rs:64–85`); `inspect::Paragraph.id` (`src/inspect.rs:37`) | analogous | Index-based, no kind/scope split, no unid, no `{#...}` token anywhere. |
| `AnchorTarget` PartUri/Unid (cs:197–206) | `Story { id, kind, part, paragraphs }` (`src/inspect.rs:124–133`); `Opened::story_parts` (`src/inspect.rs:406–418`); `edit::StoryPart` (`src/edit.rs:1052–1059`) | partial | Part naming by stem+rels; no unid stored on the address; comments part not in `story_parts`. |
| `TextPreview` (~80 chars, cs:214) | — (inspect `Paragraph.text` is full text) | missing | |
| `AutoNumberPrefix` / `FullText` (cs:229, 240–245) | `list_marker` inlines the prefix into markdown, decimal-only (`from_docx/mod.rs:738–768`) | partial | Prefix rendered inline (the foot-gun C# documents), never captured as metadata. |
| `Resolve()` part-by-URI + Unid search (cs:251–274) | no by-unid lookup; unids only stamped inside the comparer (`src/comparer/mod.rs:452-453`; `src/unid.rs:31-62`); stability via `source_sha256` snapshot guard (`src/edit.rs:1070–1079`) | missing | Phase 2 builds on `PartFs` + `parse_part` + `PT::unid()`. |
| `MarkdownProjection` (cs:281–291) | `ReadDocx { markdown, media }` (`src/markdown/mod.rs:159–166`); `inspect::markdown` (`src/inspect.rs:188–236`) | missing/analogous | No type pairs markdown with an address map; `PageCitation` absent entirely. |
| `WmlDocument.ConvertToMarkdown` (cs:293–303) | free fn `docx_to_markdown` (`src/markdown/mod.rs:181–199`); `WmlDocument` (`src/wml_document.rs:15–88`) is comparer-only | missing | |

- **Fidelity: 0.2**; **Enhancement: 0.3** (text selectors, SHA-256 stale-plan
  rejection, deterministic content-independent unids improving on the TS
  random-Guid path).
- Confidence: high (absence verified by exhaustive rg).

### W3: entry points & scope enumeration

Summary: The Rust DOCX→Markdown path is a pure, immutable CriticMarkup
projection with none of this fraction's anchor machinery — no Unid assignment,
no write-back, no anchor index, no scope flags — and it drops headers/footers
entirely; the nearest scope enumeration lives in `inspect.rs` stories.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `Convert(WmlDocument)` round-trip persisting Unids (cs:321–335) | `docx_to_markdown(&[u8], …)` (`src/markdown/mod.rs:181`) → `from_docx::convert` (`from_docx/mod.rs:65`) | missing (analogue) | Pure projection over borrowed bytes; `PartFs::set_part`/`to_zip` (`src/opc/mod.rs:180,218`) is the Phase-2 base. |
| `Convert(WordprocessingDocument)` = BuildAnchorIndex + EmitMarkdown (:344–352) | `from_docx::convert` (`mod.rs:65–222`) | analogous | Different lineage. |
| `ScopeInfo` (:358–363) | `inspect::Story` (`src/inspect.rs:124–133`) | analogous | Not used by the markdown path. |
| `AnchorIdMap` (:372–381) | — | missing | |
| `ComputeTextPreview` 80-char (:383–391) | `src/bin/jubarte.rs:1026–1031` (`chars().take(80)` + "…") | analogous | CLI-only; UTF-8-char-safe vs C# UTF-16 `Substring`. |
| `BuildAnchorIndexOnly` (:401–412) | — (inspect re-parses all parts per call; edit path guards staleness with sha256) | missing | |
| Scope enumeration body/hdr{i}/ftr{i}/fn/en/cmt (:418–441) | Markdown visits document/styles/numbering/footnotes/endnotes/comments (`mod.rs:101–122`) — no headers/footers, no scope names; `inspect::story_parts` (`src/inspect.rs:406–418`) covers header/footer/notes (comments only counted) | partial/split | 4 of 6 scope kinds reach markdown output. |

Traps confirmed: Rust path never mutates source bytes; no per-part XDocument
cache/flush story; C# `hdr{i}` numbers by HeaderParts enumeration order while
Rust sorts by `(len, name)` with file stems; C# splits deterministic vs random
Unid consumers while Rust uses one process-wide counter.

- **Fidelity: 0.2**; **Enhancement: 0.4**.
- Confidence: high (medium only on C# `HeaderParts` ordering).

### W4: anchor-index loop & id rendering

Summary: The C# anchor-index loop (unid-keyed multi-kind index over per-part
scopes, with Abbreviated/Sequential id rendering and dual-key aliasing) has no
direct Rust counterpart; the nearest analogue is `src/inspect.rs`'s positional
`story:p:index` addressing, plus a deterministic-unid module used only by the
comparer lineage.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `AssignToAllElementsDeterministic` (:450) | `src/unid.rs:23-46` (`assign_to_all_elements`, monotonic counter) | analogous | Single merged path; Rust callers are comparer-only; counter is content-independent. |
| Root part annotation (:453–454) | — | missing | Rust threads parts explicitly; no annotation mechanism in `src/xmllinq/`. |
| Boilerplate skip-set (:460–472) | `src/inspect.rs:531-544`; `src/comparer/footnotes.rs:733-742` `is_structural_note` | partial | Rust enumerates 3 named types; C# tests `type != null && type != "normal"` (equivalent in practice given ECMA's closed enum). |
| Walk + `KindFor` kinds (:474–478) | `src/inspect.rs:522-528` (`w:p` only, textbox paragraphs excluded) | partial | Only kind `p`. |
| Suppress empty-paragraph drop (:484–488) | — (empty paragraphs keep ids) | missing | |
| id `kind:scope:unid` + first-wins (:489–490) | `src/inspect.rs:564` `format!("{story}:p:{index}")` | analogous | Positional 0-based; collision impossible by construction. |
| TextPreview / AutoNumberPrefix (:497–500) | full `text`; `numbered: bool` (`src/inspect.rs:561,567`) | partial | No truncation, no resolved number. |
| Abbreviated (:514–541) | — | missing | |
| Sequential insertion-order (:543–557) | enumerate over Vec (`src/inspect.rs:550-575`) | analogous | 0-based, p-only, per-story; order-safe by construction. |
| Dual-key aliasing (:559–572) | — | missing | `indexmap` only a transitive dep. |

- **Fidelity: 0.4**; **Enhancement: 0.2** (natural part ordering header2 <
  header10; deliberate textbox exclusion).
- Key order trap: any port of the Sequential/dual-key logic must use
  `Vec`/`IndexMap`, never `std::HashMap`.
- Confidence: high; medium on `AssignToAllElementsDeterministic`'s exact
  algorithm (its source is not in the provided files).

### W5: element classification

Summary: No anchor-kind projection at all — `KindFor` is missing; only partial
analogues of `IsHeading`/`IsListItem` inside the CriticMarkup emitter and the
coarse `inspect` flags.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `KindFor` (:581–595) | 3-way branch `from_docx/mod.rs:698–726`; `Paragraph{style, numbered, in_table}` `src/inspect.rs:32–56` | analogous | No kind strings anywhere. |
| `IsHeading` prefix/Title/Subtitle (:597–604) | `Styles::heading_level` (`from_docx/mod.rs:336–361`) + `outlineLvl` (:693–701) | partial | Rust matches style **name** through basedOn first, then id requiring a parseable digit; ASCII-lowercase only; drops Subtitle and digitless "Heading…" prefix. |
| `IsListItem` direct numPr + numId=0 sentinel (:611–613) | `num_pr` (`mod.rs:376–387`) + numId=="0" check (:738–741) | partial | numId never parsed as integer, so unparseable numIds short-circuit where C# falls through. |
| bare-numPr rule (:614) | none | missing | `<w:numPr/>` without numId falls to the style chain. |
| style chain basedOn, 16 hops, cycle guard (:618–636) | `Styles::chain` (`mod.rs:318–334`) | partial | 16-cap matches; no visited set (outcome-equivalent by find-first accident). |
| style-level numPr presence (:633) | `Style.num` (`mod.rs:308`) | divergent | Rust requires numId-with-val. |

- **Fidelity: 0.5**; **Enhancement: 0.3** (outlineLvl heading detection closer
  to real Word; heading inheritance; full counter math).
- Confidence: high.

### W6: scope orchestration, notes & comments emission

Summary: The whole scope-orchestration layer has no counterpart in Rust's
Word→Markdown path — `from_docx` emits body-only with sequential `[^N]`
footnote definitions and inline CriticMarkup comments.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `EmitContext` (cs:645–654) | `Writer` (`from_docx/mod.rs:564–604`) | analogous | No Scope/AnchorIdMap. |
| `# Document` / `# Headers` / `# Footers` sections (cs:670–718) | — | missing | from_docx never opens header/footer parts. |
| `---` dividers + anyScopeEmitted (cs:747–751) | — | missing | |
| `ScopeHasContent` (cs:757–764) | — | missing | |
| `EmitNoteDefinitions` (cs:766–797) | defs loop `mod.rs:179–216` | partial | `[^N]` sequential; only *referenced* notes emitted; no fn-/en- prefixes; no section headers. |
| `IsBoilerplateNote` (cs:812–816) | `is_structural_note` (`src/comparer/footnotes.rs:734–743`) | analogous | Narrower than "any type ≠ normal". |
| `EmitComments` (cs:818–844) | `note()` + `comment_note` (`mod.rs:1104–1127, 548–562`) | analogous | Inline `{>>Author (date): text<<}`; no `# Comments` section; author omitted when missing (C# writes "unknown"). |
| `ShortUnid`/`NoteLabelSuffix` (cs:846–858) | sequential index (`mod.rs:191`) | analogous | Not unid-derived. |

- **Fidelity: 0.3**; **Enhancement: 0.2** (note defs carry tracked-change
  marks; inline comment placement is editorially richer).
- Confidence: high.

### W7: block & paragraph emission

Summary: `from_docx` reproduces the block-dispatch skeleton, the tight-list
blank-line discipline, and basic ATX heading emission, but has no anchor
system, no section breaks, no empty-paragraph modes, and clamps headings at 6
instead of 9.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `EmitBlocks` dispatch + lookahead blank rule (cs:860-885) | `Writer::blocks` (`mod.rs:615-641`); separator rule (`ooxml.rs:786-790`) | partial | Lookbehind `is_list && last_was_list` — byte-equivalent joining; handles more wrappers. |
| Heading path (cs:891-915) | `mod.rs:712-715` | partial | No anchor/numbering prefix/offset. |
| Empty-paragraph modes (cs:924-953) | `mod.rs:708`; `ooxml.rs:778-780` | partial | Suppress-only, unconditional. |
| `HasVisibleInlineContent` (cs:960-970) | `Critic::is_blank` (`critic.rs:195-201`) | analogous | |
| `EmitInlineSectionBreak` (cs:978-991) | — | missing | No `sectPr` read in from_docx. |
| `AnchorPrefix` (cs:993-1001) | — | missing | `{#` appears nowhere in the markdown path. |
| `HeadingLevel` (cs:1003-1010) | `mod.rs:336-361` + `:693-701` | divergent | Clamp 1–6 vs 1–9; no Subtitle→2; digit-concat vs suffix-parse divergence; C# `char.IsDigit` Unicode vs Rust ASCII. |

- **Fidelity: 0.4**; **Enhancement: 0.5**.
- Confidence: high (medium on exact blank-line byte equivalence, reasoned not
  executed).

### W8: inline grouping & formatting

Summary: Rust has a full inline pipeline with the same envelope walk and
adjacent-run merging, but it is CriticMarkup-dialect and its formatting key is
only `(bold, italic, link)` — strike and code (2 of the C#'s 6 key fields) are
absent, and several merge/precedence rules differ.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `Revision` (cs:1018) | `Mark` (`critic.rs:35-53`) | analogous | |
| `RunFormatting` (cs:1020-1026) | `Leaf::Text` + `Span` (`ooxml.rs:466-473`) | partial | Key lacks Strike and Code. |
| Mode gating (cs:1028-1068) | `Revisions` + `revise.rs:19-92` + `critic.rs:391-461` | partial | Accept via tree rewrite; no distinct StripDeletions. |
| Hyperlink group (cs:1055-1061) | `Inline::render` (`ooxml.rs:521-552`) | diverges | Rust merges adjacent same-URL links; C# never merges. |
| Envelope Walk (cs:1128-1157) | `Writer::inline` (`mod.rs:816-867`) | partial | All five envelopes handled; ins>del nests `{++{--x--}++}` vs C# collapse; sdt carriers descend at all levels. |
| Flush/primed buffer (cs:1082-1110) | token stream + `join_text` (`critic.rs:226-337`) | analogous | |
| `ReadRunFormatting` (cs:1162-1172) | `Writer::run` (`mod.rs:869-898`) | partial+ | Rust resolves rStyle chain + paragraph base (C# direct rPr only). |
| `HasToggle` (cs:1174-1183) | `Element::toggle` (`ooxml.rs:323-326`) | near | Tri-state — richer; exact-case "false" vs C# case-insensitive. |
| `IsCodeRun` (cs:1185-1201) | missing (from_docx) | missing | Nearest: write-side VerbatimChar (`xml.rs:423,711`). |
| `MarkdownDelimiters` (cs:1203-1212) | `render_emphasis` (`ooxml.rs:555-607`) | partial | bold-outer/italic-inner matches; strike `~~` and code exclusivity absent. |

- **Fidelity: 0.55**; **Enhancement: 0.8** (style-chain resolution,
  author/date attribution, substitution composition, whitespace-safe
  delimiters, Reject mode, fldSimple URL extraction).
- Confidence: high.

### W9: hyperlinks, run text & escaping

Summary: The Rust pipeline covers this fraction's duties but with materially
different bytes everywhere: minimal contextual escaping instead of escape-all,
`"\t"` instead of 4 spaces, `"\\\n"` instead of `"  \n"`, sequential `[^N]`
labels, and no `w:anchor` internal links at all.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| r:id link (cs:1214–1228) | `mod.rs:828–831` → `ooxml.rs:521-552` | partial | |
| w:anchor → `#anchor` (cs:1229–1231) | — (test `mod.rs:2320`: plain text) | missing | `\l` anchors in field instructions also ignored (`mod.rs:1345`). |
| `LookupRelationshipUrl` (cs:1235–1246) | `Rels::link` (`ooxml.rs:213–218`) | analogous | Pre-resolved, external-only. |
| w:t/delText + escape (cs:1252–1255) | `mod.rs:913` | partial | |
| w:br → `"  \n"` (cs:1256–1257) | `"\n"` → `"\\\n"` (`mod.rs:915-919, 226-228`) | analogous | Rust skips page-type breaks. |
| w:tab → 4 spaces (cs:1258–1259) | `"\t"` (`mod.rs:914`) | divergent | |
| Note refs (cs:1260–1277) | `mod.rs:945–957` | analogous | Sequential labels; dangling `[^N]` on missing note vs C# silent. |
| `EscapeMarkdown` escape-all (cs:1279–1282) | `critic::escape` (11 CriticMarkup delimiters, `critic.rs:530-550`) | missing (as escape-all) | |

- **Fidelity: 0.4**; **Enhancement: 0.5** (field-based hyperlinks, autolinks +
  URL escaping, page-break suppression, noBreakHyphen/AlternateContent).
- Confidence: high.

### W10: list items & numbering markers

Summary: Rust has a real list-item pipeline with genuine Word counter math and
tight-list rules matching the oracle, but renders every non-bullet format as
decimal `{n}.`, ignores lvlText, uses marker-width-aligned nesting instead of
`ilvl*2`, and has no heading-numPr prefix or ResolveNumbering knob.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `EmitListItem` (cs:1290–1302) | `num_pr` (`mod.rs:376-387`); `paragraph()` num branch (`mod.rs:716-722`) | partial | ilvl clamped `min(8)`; no anchor. |
| indent `ilvl*2` (cs:1293) | `ListIndent::item` cumulative marker widths (`ooxml.rs:815-838`) | different | Rust nests under parent content (test asserts 3 spaces under "1."). |
| trailing blank rule (cs:1300–1301) | tight `"\n"` / loose `"\n\n"` (`ooxml.rs:786-790`) | match | Exact parity of rule. |
| `ResolveHeadingNumberPrefix` (cs:1304–1311) | heading branch ignores num (`mod.rs:712–715`) | missing | |
| `ResolveListMarker` (cs:1313–1321) | `list_marker` (`mod.rs:737-768`) | partial | Always `"{value}."` (test `mod.rs:1446-1476` asserts `"1. sub"` for lowerLetter); no glyph fallback; no knob. |

- **Fidelity: 0.5**; **Enhancement: 0.4**.
- Confidence: high; divergence pinned by Rust test at `mod.rs:1474`.

### W11: tables

Summary: The GFM-vs-opaque decision layer is absent — every table renders as
GFM, flattening nested tables, expanding gridSpan into empty columns, ignoring
vMerge — though the pipe emitter and cell escaping are reasonable analogues.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `EmitTable` dispatch (cs:1329-1338) | `mod.rs:619-627` | partial | Always pipe table. |
| `CanRenderAsGfm` (cs:1340-1356) | analogue `contains_merged` (`src/comparer/lcs_table.rs:120-123`); gridSpan → filler cells clamp 1..64 (`mod.rs:1020-1025`); vMerge ignored | missing in markdown | |
| Nested-table disqualify (cs:1348) | flatten with "; " (`mod.rs:1072-1084`) | divergent | |
| Cell cap 80 (cs:1349-1354) | — | missing | |
| `EmitGfmTable` (cs:1358-1373) | `markdown_table` (`mod.rs:242-266`); `table()` (`mod.rs:1007-1045`) | analogous | Compact `\|a\|b\|` + `\|-\|-\|`; pads ragged rows; drops empty rows/trailing columns. |
| `EmitOpaqueTable` (cs:1375-1390) | nearest `cell_span` (`src/convert/mod.rs:11341-11361`) | missing | |
| `CellTextForGfm` (cs:1395-1401) | `table_cell` (`mod.rs:231-239`) | partial | `\|` escape + newline collapse + trim match; empty→"" not " "; extra backslash escaping. |

- **Fidelity: 0.3**; **Enhancement: 0.5** (revision-aware cells, gridBefore
  padding, multi-paragraph `<br>` cells, round-trip through pulldown-cmark).
- Confidence: high.

### I1: IR emitter structure & anchor-index build

Summary: The Rust repo has no projection IR at all — markdown is a single-pass
stream over the package and inspection is a per-paragraph positional model.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| Byte-equivalence contract (cs:10–34) | — | missing | Nearest basis: `from_docx::convert`. |
| `IrMarkdownResult` (cs:39–46) | `Converted { markdown, media }` (`mod.rs:59-63`) | analogous | |
| `BuildAnchorIndex` scope walk (cs:64–98) | `Opened::story_parts()` (`src/inspect.rs:406-418`) | partial | Sorted by (len, name) — not oracle order; no anchors built. |
| `AddIndexEntry` (cs:154–173) | — | missing | Positional `body:p:{index}` instead. |
| `IndexNoteScope` (cs:179–195) | notes keyed `HashMap<(is_endnote, w:id), Element>` (`mod.rs:100–116`) | analogous | w:id keying exists; no Unid map. |
| `ProjectionAnchors` facts (cs:100–129) | — | missing | Retention is ownership in Rust; fact capture is a natural design. |
| (context) atom IR | `AtomBlock`/`ComparisonUnitAtom` (`src/comparer/atoms.rs:109/122/300`) | different purpose | Proof the repo can hold a retained IR. |

- **Fidelity: 0.1**; **Enhancement: 0.1** (inspect `Projection`/`Segment`/
  `Piece` back-map, limitations reporting, sha256 guards).
- Confidence: high.

### I2: part-URI resolution, AnchorIdMap port & index walk

Summary: The unid-based anchor index does not exist in Rust; only surrounding
primitives (rels-based part resolution, a document-order paragraph walk with
table cells and separator-note skipping, unid stamping in `src/unid.rs`).

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `ResolveScopePartUri` tiers (cs:214–220) | `main_part` / `Opened::related` (`src/inspect.rs:450-457, 436-447`) | analogous | Rels-authoritative + fallback. |
| `ResolveScopePartUriBySuffix` (cs:222–230) | `notes_part_names` (`src/document_comparer.rs:6057-6079`) | analogous+ | Rels-first; PR #51 explicitly rejects suffix hardcoding. |
| `AnchorIdMap` (cs:248–253) | — | missing | |
| `BuildAnchorIdMap` (cs:258–302) | — | missing | |
| `WalkAnchorsForIndex` (cs:312–372) | `body_paragraph_nodes` (`src/inspect.rs:523-528`) | partial | Same pre-order; only `w:p`. |
| Separator-note skip | `story_paragraph_nodes` (`src/inspect.rs:531-544`) | parity | Also skips continuationNotice. |
| `WalkTextboxAnchors` (cs:378–385) | — (inverse: excluded, flagged `text_box_omitted`, `src/inspect.rs:769-773`) | divergence | Opposite of oracle inline indexing. |
| sdt wrapper indexed-not-rendered asymmetry | — (sdt descended transparently) | missing | |
| Suppress-mode drop / 80-char preview | — | missing | |

- **Fidelity: 0.2**; **Enhancement: 0.2**.
- Confidence: high.

### I3: auto-number resolver, previews & text predicates

Summary: Rust has an exact 80-char + "…" preview cap in the CLI, but from a
glyph-substituting projection (not w:t-flat); the per-anchor AutoNumberResolver,
HeadingNumberPrefix rule, and index-walk text predicate have no counterpart.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `AutoNumberResolver` (cs:400-411) | `list_marker` (`mod.rs:738-768`); `numbered: bool` (`src/inspect.rs:43,566`) | missing | Markers computed at emit, never captured as data. |
| `BuildAutoNumberResolver` all-scope walk (cs:416-453) | `story_paragraph_nodes` (`src/inspect.rs:523-544`) | partial | No comments; textbox paragraphs deliberately omitted. |
| `HeadingNumberPrefix` (cs:458-465) | — | missing | |
| `ComputeTextPreview` (cs:471-479) | `src/bin/jubarte.rs:1026-1032` | partial | Cap exact; code points vs UTF-16 units; CLI-display only. |
| `AppendFlatText`/`AppendInlineText` (cs:485-542) | `walk_container`/`walk_run` (`src/inspect.rs:637-804`) | analogous | Inspect adds glyph substitutions and omits textbox text — opposite of C#. |
| `ParagraphHasVisibleText` (cs:550-565) | `Critic::is_blank` (`critic.rs:195-201`) | analogous | Rust counts a Comment as visible; includes fldSimple/sdt text the oracle drops. |
| `ParagraphHasVisibleTextOrTextbox` (cs:575-580) | — | missing | |

- **Fidelity: 0.3**; **Enhancement: 0.3** (byte-precise Projection back-map).
- Confidence: high.

### I4: IR markdown emission (blocks/paragraphs/lists)

Summary: `from_docx` has an analogous block dispatcher with tight-list
discipline, but the scope scaffolding, anchor tokens, unid-keyed note labels,
comments section, heading-number prefixes, and the reader-captured structural
list-item verdict are all absent.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `EmitMarkdown` scaffold (cs:586–655) | `convert()` (`mod.rs:65–222`) | missing (partial) | Body + note defs only. |
| `EmitBlocks` SDT skip + blank rule (cs:762–794) | `Writer::blocks` (`mod.rs:615-641`) + `push_paragraph` (`ooxml.rs:786-790`) | partial/divergent | Rust **descends and renders** sdtContent; blank rule keyed on resolved markers, not the structural verdict. |
| `IsListItemForBlankRule` passthrough (cs:796-803) | re-derived at emit | missing/divergent | Misses heading+numPr, bare numPr, unresolvable numId. |
| `EmitParagraph` clamp 1–9 + prefix + modes (cs:805-861) | `Writer::paragraph` (`mod.rs:687-735`) | partial | Clamp 1–6; empty paragraphs dropped unconditionally. |
| `EmitInlineSectionBreak` (cs:868-878) | — | missing | |
| `EmitListItem`/`ResolveListMarker` (cs:880-909) | `mod.rs:716-722` + `list_marker` | partial | Marker-width indent; decimal-only. |

- **Fidelity: 0.3**; **Enhancement: 0.3**.
- Confidence: high.

### I5: IR inline grouping & run text

Summary: `from_docx` performs the same job via a streaming two-layer pipeline
(Critic tokens → Inline spans), but its format key is 3-tuple vs the oracle's
5-tuple, and escaping/tab/note-label/SDT/textbox details all diverge.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `RunFmt` 5-key (cs:917) | `Span {bold, italic, link}` (`ooxml.rs:468-473`) | partial | |
| `EmitInlineRuns` (cs:919) | `Inline::render` + `render_emphasis` (`ooxml.rs:521-607`) | analogous | |
| `GroupInlineRuns` (cs:946) | `Critic::push` + `join_text` + `Inline::push` (`critic.rs:124, 315`; `ooxml.rs:500`) | analogous | |
| FromInlineSdt drop (cs:993-998) | sdt recursed (`mod.rs:845-847`) | divergent | Rust renders; oracle projects out. |
| `IrFieldRun` (cs:1000-1009) | fldSimple + complex-field state machine (`mod.rs:832, 909-944`) | faithful+ | Also extracts HYPERLINK links. |
| `IrTab` (cs:1010-1014) | tab with run format (`mod.rs:914`) | faithful | Text `"\t"` vs 4 spaces. |
| `IrBreak` (cs:1092-1098) | page breaks dropped; others `"\\\n"` | divergent | |
| `IrTextbox` drop (cs:1023-1028) | textbox → extra blocks + images (`mod.rs:979-1005`) | divergent+ | Rust preserves content. |
| `IsCodeRun` (cs:1046) | — | missing | |
| `EscapeMarkdown` (cs:1127-1130) | CriticMarkup delimiters only | divergent | |

- **Fidelity: 0.5**; **Enhancement: 0.5**.
- Confidence: high.

### I6: IR tables, heading level & anchor prefix

Summary: No anchor prefix, no GFM-vs-opaque decision, no opaque fallback, no
cell cap, no SDT two-depth discipline — Rust deliberately walks *through*
SDTs.

| C# construct | Rust counterpart | Status | Note |
| --- | --- | --- | --- |
| `HeadingLevel` (cs:1138) | `Styles::heading_level` (`mod.rs:336-361, 693-701`) | analogous | No Subtitle→2; clamp 1–6; adds outlineLvl. |
| `AnchorPrefix` (cs:1150) | — | missing | |
| `EmitTable` gate (cs:1163) | `Converter::table` (`mod.rs:1007-1045`) | divergent | Always GFM. |
| `CanRenderAsGfm` (cs:1181) | — | missing | vMerge never read in `src/markdown`. |
| `OracleVisibleRows`/`Cells` (cs:1210, 1217) | `table_rows`/`row_cells` walk through sdt/customXml/ins (`mod.rs:1130-1150`) | divergent | Renders the exact rows the oracle excludes. |
| `EmitGfmTable` (cs:1220) | `markdown_table` (`mod.rs:242-266`) | analogous | Compact pipes; normalization the oracle lacks. |
| `EmitOpaqueTable` (cs:1237) | — | missing | |
| `CellTextRaw`/`CellTextForGfm` (cs:1256, 1265) | `cell_text`/`table_cell` (`mod.rs:1049-1093, 231-239`) | divergent/partial | CriticMarkup-aware, not flat. |

- **Fidelity: 0.2**; **Enhancement: 0.2**.
- Confidence: high.

---

## 3. Phase 2 — solutions & best practices per cluster

### Cluster A: anchor infrastructure

**Recommendation.** Build a self-contained `src/markdown/projection/` module
(new, sibling of `from_docx`) owning: a deterministic `PtOpenXml.Unid`
assignment mode added to `src/unid.rs`, the
`Anchor`/`AnchorTarget`/`AnchorIndex`/`AnchorIdMap` types, and a
part-name+unid `resolve`. Keep the comparer's counter path untouched (same
split as the C#, WmlToMarkdownConverter.cs:446–450). Effort **M**, priority
**P1** — every later cluster consumes these ids.

- Deterministic assignment:
  `assign_to_all_elements_deterministic(dom, root, part_name) -> bool` — walk
  `descendants_and_self` in document order; for each element missing
  `PT::unid()` (never overwrite, matching `src/unid.rs:41`), build the chain of
  `(local-name, index-among-same-named-siblings)` from the part root and set
  `unid = hex(Sha256(part_name + "\0" + path))[..32]`, salting on intra-part
  collision. Stability contract: same part bytes + part name → same ids in any
  process/session/target; structural, so attribute order and zip entry order
  cannot change ids; idempotent.
- Types: `AnchorKind` enum (P/H/Li/Tbl/Tr/Tc/Col/Sec/Fn/En/Cmt/Sdt), `Anchor`
  with `id()`/`token()`, `AnchorTarget { part_uri, unid, text_preview,
  auto_number_prefix }` with `resolve(&PartFs)` (part by exact name, then
  unid attribute scan), `AnchorIndex` as a `Vec<(String, AnchorTarget)>`
  (**insertion order IS document order — never HashMap/BTreeMap; no direct
  indexmap dep**), `AnchorIdRendering` + `AnchorIdMap` with Abbreviated
  (shortest unique prefix, 4-char floor, 32 cap, per (kind,scope)) and
  Sequential (1-based per bucket) and dual-key aliasing.
- Integration: reuse `Opened`/`story_parts`/`parse_part` read-only; keep C#
  scope names (`hdr1`, `ftr1`, `fn`, `en`, `cmt`) for oracle parity; write-back
  via `pkg.set_part` + `to_zip`. Keep `{story}:p:{index}` as the edit-selector
  namespace; add `Selector::Unid` (untagged serde so old plans still parse) and
  optionally cross-register positional aliases in `AnchorIndex`.
- Tests: golden id/index snapshots; stability across re-runs and native vs
  WASM; idempotence; zip-entry-reordered package → identical ids; comparer
  output byte-identical (counter path untouched).
- Open questions: eager persistence vs returning stamped bytes (eager changes
  `source_sha256` and trips the edit stale guard); confirm `hdr1`/`fn` scope
  naming wins over inspect stems; positional↔unid alias now or later.

### Cluster B: scope orchestration, notes & comments

**Recommendation.** New `src/markdown/from_docx/scopes.rs` orchestrator behind
a separate public entry (e.g. `MarkdownOptions::projection` enum, default =
current Anymd), reusing the existing `Writer` per scope. Do **not** touch
`from_docx::convert`'s default output — `tests/docx_to_markdown_office_samples.rs:24-31`
and the doctest at `src/markdown/mod.rs:174-186` assert body-first bytes.

- Discovery: enumerate `rels.order` filtering `/header`, `/footer`
  (`ooxml.rs:99-133, 196-211`); number `hdr{i}`/`ftr{i}` by relationship
  order (oracle parity; deterministic for fixed bytes). Do not reuse inspect's
  stem sort.
- Suppression: `scope_has_text` = any descendant `t` with non-whitespace text
  (textbox text counts; PAGE-field-only headers suppressed — copy the oracle,
  comment it).
- Notes: `[^fn-{suffix}]` with suffix from `AnchorIdMap::render(note_unid)`
  (hard dependency on Cluster A); emit ALL non-boilerplate notes in document
  order; blank line between defs; silent no-op on missing refs.
- Shared predicate: `is_boilerplate_note_type(t) = t.is_some_and(|t| t !=
  "normal")` in a neutral module; refactor comparer's `is_structural_note`
  (delegate the type test) and inspect's `note_count`/`story_paragraph_nodes`
  (currently misses `continuationNotice` in counts).
- `# Comments` section: `- {#cmt:cmt:{rendered}} **{author}** (date): text`,
  author defaulting to `"unknown"`.
- Tests: 6+ header/footer variants mostly blank; PAGE-field-only header;
  textbox-only header; continuationNotice note; unreferenced notes;
  author-less comments; determinism.
- Risks: OpenXML `HeaderParts` order is rels insertion order in practice but
  undocumented — fallback to sectPr-reference-driven numbering if a fixture
  diverges. Effort ~2-3 days (P1 scaffold) + ~2 days (P2 notes/comments).

### Cluster C: inline formatting, escaping & run text

**Recommendation.** Port strike + code as a 4-field `Fmt` struct threaded
through `Span`/`Critic`; do NOT make escape-all the default — add an `Escape`
mode knob for parity runs only; adopt oracle byte conventions (tab 4 spaces,
`"  \n"` breaks, `*` italic, strike-outermost nesting, no link merging,
del-wins nesting, `#anchor` links); reject the inline-SDT render-drop (it
deletes visible Word content; record the divergence). Land the envelope fixes
first — independent and cheap.

- `Fmt { bold, italic, strike, code }` flows through the 6 tuple/bool sites
  (`ooxml.rs:466-473, 500`; `critic.rs:60-67, 124, 315-337, 381-389,
  427-440, 479-516`); rendering order code-exclusive, else `~~` outermost,
  `**`, `*`, closes reversed.
- `is_code_run`: rStyle ∈ {Code, HTMLCode, VerbatimChar} (ASCII
  case-insensitive) or `rFonts/@ascii` contains mono/courier/consolas; closes
  the round-trip with write-side VerbatimChar.
- `~~` de-conflict: extend `critic::escape` — a bare `~~` not preceded by `{`
  and not followed by `}` becomes `~\~`.
- Escape knob: `Escape { Minimal (default), Oracle }` on `Options`; Oracle =
  char-class escape of Text leaves (skip `fmt.code`), suppress label re-escape
  and table `\`-doubling; `pub(crate)` until differential tests validate.
- Byte fixes: del-inside-ins collapses to deletion (update test at
  `mod.rs:2122`); remove same-link merging in `Inline::render` (guard autolink
  to http/mailto so `#anchor` never becomes `<#anchor>`); `w:anchor` →
  `#anchor`; `hard_breaks` → `"  \n"` after `tidy_inline`; tab → 4 spaces
  (gate the space-run collapse); drop the page-break filter.
- Effort: envelopes/links ~0.5d (P1); Fmt+strike+code ~1d (P1); bytes ~0.5d
  (P2); escape knob ~0.5d (P2).

### Cluster D: classification, headings & numbering

**Recommendation.** One shared reader module `src/markdown/classify.rs`
(`Kind`/`kind_for`, reconciled `heading_level`, faithful `is_list_item`);
extract the PDF path's numbering math into shared `src/numbering.rs` and drive
markdown markers from it behind `resolve_numbering` (default on). Keep
name-based heading resolution + outlineLvl (Word parity), extend clamp to 1–9,
add Subtitle→2. Keep the marker-width indent (deliberate divergence; document
it).

- Heading precedence: direct `outlineLvl` → style-chain name
  ("title"→1, "subtitle"→2, "heading N") → chain outlineLvl → id fallback with
  oracle digit-concat, clamp 1–9. Add visited-set cycle guard to
  `Styles::chain`.
- `is_list_item` port (reader-captured fact, the `IsListItemForLayout`
  analogue): integer-parse numId (≠0 → li, =0 → not li); bare direct numPr
  without ilvl → li; else style-chain numPr **presence** ≤16 hops. Expose as
  `Class { kind, heading, list }` consumed by Writer, inspect, and the future
  anchor projection.
- Numbering: extract `NumFmt`/`parse_num_fmt`/`format_num`/lvlText `%n`
  substitution from `src/convert/mod.rs` (4915–5040) into `src/numbering.rs`
  (convert re-imports; PDF byte-identical); `list_marker` returns verbatim
  lvlText labels ("a.", "iv.", "1.1") with the single-glyph→"-" fallback;
  heading+numPr prefix (fixes dropped legal-heading numbers); knob + CLI flag.
- Predicates: name the three (render-visible vs DOM/index-visible vs
  preview-flat); `AutoNumberPrefix` + `full_text()` land on
  `inspect::Paragraph`.
- Effort: classify ~1d (P1, prerequisite for anchors); numbering ~2d (P1
  heading prefix, P2 markers); predicates ~0.5d (P3).

### Cluster E: tables

**Recommendation.** Add an oracle-faithful table pipeline beside the current
one, selected by `MarkdownOptions::table_mode` (`Auto` =
GfmWithOpaqueFallback / `AlwaysGfm` / `AlwaysOpaque`) +
`table_inline_cell_max` (80); parity mode dispatches through a ported
`can_render_as_gfm` (two-depth discipline) to byte-parity GFM or an opaque
```table block; the current flattening/padding pipeline remains the default
(`Native`) mode; record divergences WORD_DIFFERENCES-style.

- Two depths without model changes: merge checks descendants-wide
  (`find_all("gridSpan")`/`find_all("vMerge")` — note this is *stronger* than
  the IR twin, matching the oracle); nested/length checks via new
  `direct_named(el, name)` iterators mirroring LINQ `Elements` chains; keep
  `table_rows`/`row_cells` untouched for Native mode.
- Opaque block: rows = direct tr count; cols = max row Σ max(1, gridSpan)
  (math as in `cell_span`, no 64 clamp); anchor line behind a stub seam
  (`fn table_anchor(&self, _) -> String`) so Cluster A plugs in later.
- Cell text: `cell_text_raw` (flat w:t concat, for the length gate — counts
  UTF-16 units if byte parity on non-BMP matters) vs `cell_text_for_gfm`
  (newline→space, `|`→`\|`, trim, empty→`" "`, no backslash doubling, ragged
  rows kept).
- Tests: merges incl. nested-table and SDT-delivered cases; 79/81-char cells;
  SDT-wrapped rows (visible to merge check, invisible to length/nested); zero
  rows; AlwaysGfm/AlwaysOpaque overrides; golden byte-parity fixtures. Test
  the default mode alongside every parity assertion (AGENTS.md pattern).
- Effort 1.5–2.5 days; priority high (the enum/dispatch seam is on other
  clusters' critical path).

### Cluster F: architecture, settings, testing & roadmap

**Recommendation.** Build a new reader→IR→emitter projection pipeline in
`src/markdown/projection/` (`ir.rs`, `reader.rs`, `emit.rs`, `index.rs`),
leaving `from_docx` untouched as the pandoc-dialect reader; one public
`ProjectionOptions` + a dialect split at the API/CLI boundary; oracle goldens
checked in under `tests/fixtures/projection/` with the existing Docxodus MIT
attribution.

- Why IR: the anchor index is inherently two-pass; five sister clusters need a
  shared intermediate (anchors ride IR nodes; scopes are IR block lists;
  formats/markers/grid facts are reader-captured, exactly as
  `IrMarkdownEmitter.cs:796–803` demonstrates); the codebase already proves a
  retained IR works (`src/comparer/atoms.rs`).
- Public surface: `pub fn docx_to_projection(bytes, &ProjectionOptions) ->
  Projection { markdown, anchors }`; `#[non_exhaustive]` options with
  plain-struct scopes (no bitflags dep), small enums; projection
  `tracked_changes {Accept, RenderInline, StripDeletions}` kept **separate**
  from pandoc `TrackChanges`; one dialect enum at the read boundary, not
  per-knob flags; CLI `convert --to md` unchanged, `--projection anchored`
  for the new pipeline.
- Harness: `tests/fixtures/projection/NAME.docx` + `NAME.md` +
  `NAME.anchors.json` generated from the C# oracle (goldens pre-stamped with
  Unids, self-describing); property tests (determinism across cold runs,
  prefix uniqueness, dual-key resolution, resolve-back round-trip); Ring-1
  validity for Unid-stamped packages via `tests/common/validity.rs`.
- Roadmap: F0 scaffold (M) → A anchors (L) → B scopes (M) → C inline (M) →
  D numbering (L) → E tables (M). KPI: golden byte-parity fraction. **Do not
  build**: PageCitations, ImageUriBuilder, BlockAndInline anchor mode,
  RetainSources=false memory mode, projection↔redline wiring.
- C#→Rust engineering rules: (1) insertion-ordered `Vec`/IndexMap wherever
  order feeds output — never HashMap iteration; (2) `OrdinalIgnoreCase` →
  ASCII-case-insensitive helpers; (3) `char.IsLetterOrDigit`/`IsDigit` are
  Unicode → `is_alphanumeric`/`is_numeric`; (4) UTF-16 `Substring` → a
  `truncate_utf16` helper or recorded divergence (previews, ShortUnid); (5)
  `StringBuilder.Length--` → `String::pop()`; `Math.Clamp` bounds 1..9; (6)
  XDocument annotations → explicit reader context; (7) regex class → a
  char-class predicate fn; (8) lazy IEnumerable with mutating closures →
  explicit loops over materialized Vecs.
- Licensing: extend the existing `LICENSES/LicenseRef-Docxodus-MIT.txt` +
  REUSE.toml coverage to projection goldens; "Port of Docxodus
  `WmlToMarkdownConverter` (MIT)" notes in each new file's SPDX header; run
  `uv tool run --from 'reuse[charset-normalizer]' reuse lint`.

---

## 4. Consolidated open questions (for the human)

1. **Deterministic-unid algorithm**: `AssignToAllElementsDeterministic`'s
   source is not among the provided files. Must jubarte replicate its exact
   ids for unstamped documents (true byte parity), or is reading pre-stamped
   `pt:Unid` from oracle-generated goldens sufficient?
2. **Dialect strategy**: is the anchor-addressed projection a permanent second
   dialect (`--projection anchored`), or an eventual replacement for
   `docx_to_markdown`?
3. **Scope naming**: oracle `hdr{i}`/`fn`/`en`/`cmt` vs inspect file stems —
   confirm oracle parity wins inside the projection.
4. **Persistence**: should projection eagerly persist stamped bytes (changing
   `source_sha256` and tripping edit-plan stale guards), or return stamped
   bytes for the caller to adopt?
5. **Divergences to keep**: Rust's marker-width list indent, inline-SDT
   render-through, textbox content in markdown, revision-aware table cells —
   keep as documented better-than-oracle behavior, or gate behind parity mode?
6. **CLI surface**: flag on `convert`, or a dedicated `jubarte project`
   subcommand (the anchor index is structured output)?
