<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

**Contract version 3 · Review amendments · 17 October 2026 UTC · Implementation specification**

**Agent view always emits short IDs. `--json` always emits both `short_id` and `long_id` for every document reference. No third identity representation is permitted.**

This contract covers document addresses in the native CLI, agent Markdown, JSON and JSONL, edit and resolution reports, diagnostics, Python, WASM, npm, and MCP. It fixes the vocabulary, the meaning of each address, the exact JSON shape, source-key edge cases, and the migration gates. The complete short grammar and a substantial long-ID catalog are included below.

The governing output rule is the maintainer’s instruction. This revision adopts the requested `psN` spelling for structurally scoped paragraphs while preserving `pN` for story-indexed paragraphs, and recommends explicit `cell.ps0` for the first own cell paragraph. The four-field long format and paired JSON representation remain the contract. The rules below close identity collisions, define scope, and prohibit fallback address families.

**Implementation status:** this document landed documentation-only in `4612a155` (merged by `b5bdccbd`, PR #570). No code implements the contract: `main` at `aba48bd5` is still pre-contract. The code on `main` implements the 13 October settlement's transitional grammar — `rN`, `cN`, `pN`, `tN.rowM.cellK`, `.ccN`, closed `pN-pM` and `tN-tM` ranges, old nested `.pN` cell paragraphs (`t0.row1.cell2.p1`, not the contract's `.ps1`), the `#12`-era spellings retired — plus these known divergences: `ChangeId::short()` still falls back to `story:rev:wid`, the view still prints `noid` for unnumbered revisions, paragraph zero of a story still prints as the bare story alias (`header1`, not `header1.p0`), `r43+43` still prints with a repeated number, JSON still emits colon long IDs (`body:rev:12`, `body:p:5`) beside a `ref` short field instead of the paired `short_id`/`long_id`, `read -p` still accepts comma lists and open ranges through `Select::parse`, block steps are still `usize`, and no typed `LongId`, `j:short` key, `IdRef` pair or `--long-id` input exists. Version 2 was written against [`d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd`](https://github.com/jandira-tech/jubarte-redlines/commit/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd) on `fix/md-roundtrip-fidelity`; version 3 was checked against `feat/md-table-grid-default` at `7c996819`, where every piece of current code it names reads the same, and against a 17-document docx → md → docx → md probe recorded in `docs/superpowers/specs/2026-10-16-id-contract-review.md`. This document supersedes conflicting ID clauses in the [13 October settlement](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/docs/superpowers/specs/2026-10-13-ids-and-markdown-roundtrip.md), including its §4.3 dependent keys (a table is no longer its first row's key, a cell no longer its row's key plus `.cellN`) and its §5.3 anchored mode, which now lives in section 7 here. It does not supersede unrelated Markdown fidelity rules. Plan: `docs/superpowers/plans/2026-10-16-id-contract-implementation.md`.

**Changes in version 3 (Arthur's decisions of 2026-10-16 on the review of version 2):**

1. The writer's rule is contractual: `rN` and `cN` labels write back as `w:id="N"`, story aliases as their conventional part URIs; the writer never renumbers a label (section 7). This is what makes the Markdown a fixed point for identity.
2. `ps` is always an index into a scope's own paragraphs. In a cell it is an accepted alias that resolves to the story `pN` the view prints; in a text box it is the printed id, because box paragraphs form their own inventory (section 2).
3. Whatever number Word gave is the key. A keyless target's long ID is `PART.ELEMENT.j:short.SHORT`, its short entity id verbatim. Version 2's SHA-256 `j:key`, its framing, collision counter and vectors are withdrawn. Native keys keep their ranges, Microsoft's for `paraId` and Word's `int` for every `w:id`; a value outside them is not a number Word would write: the long ID takes `j:short` and the short ID is allocated (sections 4, 5 and 6).
4. Long IDs ship with their first consumer (`--long-id` input, paired JSON), after the short grammar, the inventory, the presenters and the writer's rule (section 9).
5. Text-box forms are accepted syntax that resolves to `UNSUPPORTED_TARGET` until the text-box stage lands; the inventory's text-box test covers `w:txbxContent` and `wne:txbxContent` (sections 7 and 9).
6. Carriers that shared one source `w:id` print as their distinct allocated labels joined (`r0+1`); a printed id never repeats a number, so `r43+43` is retired (sections 3 and 4).
7. Anchored Markdown edits are specified here, not only in the 13 October settlement (section 7).
8. A comma in `read -p` is an option-level separator, never part of an id; open ranges are retired (section 2).
9. Every catalog row is checkable by eye: a `j:short` value is the short id beside it (section 11).

# 1 Output contract

## Two representations

| Output surface | Required identity representation |
| - | - |
| Agent Markdown, text listings, YAML ID values, notes and human diagnostics | Canonical short entity IDs, or the defined short selector grammar where a selection is intended |
| `--json`, including JSONL events and structured failures | Every resolved entity reference contains exactly `short_id` and `long_id` |
| Python, WASM, npm and MCP structured results | The same paired representation produced by Rust |
| Edit input, including structured plans | A settled short ID or selector, or a settled long ID where that target kind is accepted |
| Document content and nonidentity metadata | Their native content types; they must never be offered as alternative document addresses |

“Always” applies to every emitted reference, not to whether every view must list every object. A compact table view may print a cell ID without separately listing all of its paragraph IDs. If it prints an identity, that identity must be short. If JSON exposes the cell and its paragraphs, each receives the appropriate pair.

There is no `--long-id` output mode, PID annotation mode, legacy output mode, environment override, or diagnostic exception. A future input option named `--long-id` may select a target using a long ID; it must not alter the output policy. Ordinary ID-taking options should accept the same typed input union instead of requiring another flag.

## The one JSON reference shape

```json
{
  "short_id": "p12",
  "long_id": "document.w:p.w14:paraId.3F2A1B7C"
}
```

These field names are mandatory. A primary object may flatten this pair beside its other properties. A reference to another object is a nested object with the same two fields. Arrays contain these objects. Do not output a string reference and an auxiliary lookup table that the caller must join to recover its other ID.

```json
{
  "schema_version": 2,
  "id_contract": "jubarte.ids/2",
  "identity_profile": "canonical-wml-v1",
  "source_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "changes": [
    {
      "short_id": "r12",
      "long_id": "document.w:ins.w:id.12",
      "kind": "insertion",
      "paragraph": {
        "short_id": "p12",
        "long_id": "document.w:p.w14:paraId.3F2A1B7C"
      }
    }
  ]
}
```

The digest above is illustrative. A real response uses the digest of the inspected bytes.

The following are forbidden as additional document identity fields: `id`, `ref`, `pid`, `paragraph_id`, `comment_id`, `legacy_id`, `display_id`, or a string-valued `at`/`location` standing in for the pair. Semantically named fields such as `paragraph`, `parent`, `inside`, `from`, `to` and `target` remain useful, but their resolved entity values must be paired references.

An `IdRef` has two nonempty strings. It cannot contain a null half, an empty half, `unknown`, `noid`, or a raw number; a keyless target's long half is its complete four-field `j:short` long ID (section 5), never a bare short id or a path string in the long field. If there is no resolved entity, use no reference: for example `"target": null`, with a structured diagnostic. An absent reference is not permission to create an invalid pair.

## Scope without loopholes

Operation correlation labels, source digests, counts, character offsets, file names, style names, bookmark names, relationship attributes and Markdown line numbers are not alternative document addresses. Give them explicit names such as `operation_label`, `source_sha256`, `start`, `end`, `part_uri`, `style_name`, or `line_number`. Do not place them in a document-ID field, advertise them as selectors, or synthesize a public ID from them.

User-authored text can contain `body:p:12`, a UUID or any other string. Preserving that content does not violate this contract. The ban applies to IDs that Jubarte generates as document addresses. It must be checked structurally, not by deleting suspicious strings from document text.

Source XML attributes remain source data. A raw `w:id`, `w14:paraId`, `w14:textId`, `r:id`, or bookmark name must never be promoted into a third public address. A native key contributes to a long ID only under the explicit key rules below.

## No fallback

The implementation must never do any equivalent of:

```rust
short_id.unwrap_or(long_id)
short_id.unwrap_or(raw_xml_id)
short_id.unwrap_or("noid")
```

If the source key is missing, unusual or duplicated, the inventory assigns a valid short ID and an appropriate long ID. If the engine cannot inventory the requested structure safely, it returns a typed failure or explicitly incomplete coverage. It preserves the source content. It does not emit an address outside the grammar or silently omit a supported entity.

# 2 Complete short ID grammar

## Story and scoped paragraph names

Use `pN` for an index in a story paragraph inventory and `psN` for an index within a named structural paragraph scope. `ps` means scoped paragraph, not subparagraph: the target is a complete paragraph, not a textual or legal subdivision.

| Address | Meaning |
| - | - |
| `p12` | Paragraph 12 in the main-story inventory |
| `header1.p0`, `footer2.p7`, `footnotes.p3`, `comments.p4` | Paragraph in the named story inventory; spellings stay unchanged |
| `t0.row1.cell2.ps1` | Own paragraph 1 in the named cell |
| `p14.box0.ps0` | Paragraph 0 in the named text-box inventory |
| `t0.row1.cell2.ps1-ps3` | Cell-scoped paragraphs 1 through 3 |
| `p14.box0.ps1.box0.ps0` | A scoped paragraph inside a nested text box |

The word “story” includes the body and named nonbody stories. Under this contract, the story-wide paragraph inventory still includes table-cell paragraphs and excludes separate text-box inventories. This amendment changes the scoped spelling; it does not renumber story paragraphs or make `pN` mean only paragraphs physically outside tables. Cell-scoped paths remain selectors that normalize to the containing inventory’s canonical paragraph identity. Text-box paragraphs retain their canonical structural path, now using `psN`.

`ps` has one meaning everywhere: an index into the named scope's own paragraph inventory (the cell's own paragraphs, the box's own paragraphs), excluding paragraphs that belong to a nested table or a deeper box. Which spelling the view prints follows from inventory membership, not from the token. A cell's paragraphs belong to the story inventory, so `t0.row0.cell0.ps1` is an accepted alias for the second paragraph inside that cell and resolves and prints as that paragraph's story id (`p23`); a text box's paragraphs form their own inventory, so `p14.box0.ps1` is the printed id (Arthur, 2026-10-16: "in a cell, `pN` corresponds to that specific cell, but we also read `cell.psN`; it works as an alias").

The recommended additional simplification is to accept explicit `cell.ps0`. Every scoped index then starts at zero, and the cell itself remains a separate entity. A paragraph-edit command can retain an explicit cell-to-first-paragraph coercion, but the grammar no longer needs a special ban on the first scoped paragraph.

| Candidate | Meaning and tradeoff | Decision |
| - | - | - |
| `psN` | Scoped paragraph; retains the paragraph-leading `p` vocabulary | Use this one spelling |
| `spN` | More conventional initialism for scoped paragraph; no additional parser capability | Do not introduce as an alias |
| `lpN` | Readable as local paragraph; another vocabulary change with no substantive parser advantage | Do not introduce as an alias |

Parse the complete alphabetic step name before its numeric suffix. Represent `p` and `ps` as different syntax variants, validate their permitted owners, and match range endpoints by typed step kind. The current parser already has an unambiguous structural state machine; the benefit is making scope explicit in the type system and removing the first-cell-paragraph exception, rather than changing the underlying XML paragraph type. [Current step type and parser](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids/short.rs#L48-L108) · [Current owner transitions](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids/short.rs#L198-L247).

## All literals and numeric rules

There are exactly **19 keyword literals**:

```text
p  ps  t  row  cell  box  cc  r  c  s  toc
header  footer  footnotes  endnotes  comments  styles  numbering  text
```

There are exactly **three structural separators**: `.`, `+`, and `-`. Decimal digits are `0` through `9`.

For the portable wire contract, **every numeric component is a `u32`**, written `0` or `[1-9][0-9]*`, within 0 through 4,294,967,295. No leading zeros, signs, whitespace, Unicode digits or case folding are accepted. All positional indices, including scoped paragraph positions, start at zero; there is no first-paragraph exception. The engine may impose smaller resource limits; it must report those limits without changing syntax or wrapping an integer.

This deliberately replaces host-width-dependent `usize` syntax. The current parser uses `u32` for revisions, comments and header/footer suffixes, but `usize` for block positions; that can accept different strings in native and WASM builds. Use checked conversion from the wire number to a host index. [Current number handling](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids.rs#L199-L217) · [Current steps](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids/short.rs#L49-L105).

## Complete grammar

This is the complete set of accepted short forms. Numerical instances and recursive nesting cannot usefully be enumerated as a finite list; the productions define all of them.

```ebnf
short         = block | range | change | comment | covered | section | toc ;

part          = "header" N | "footer" N
              | "footnotes" | "endnotes" | "comments" ;
meta          = "styles" | "numbering" ;
story         = part | meta ;

block         = item | part | part "." item | meta ;
item          = story_para | table | "cc" N ;

story_para    = "p" N [ "." para_tail ] ;
scoped_para   = "ps" N [ "." para_tail ] ;
para_tail     = "box" N "." box_item | "cc" N ;
box_item      = scoped_para | table ;

table         = "t" N [ "." "row" N
                         [ "." "cell" N [ "." cell_tail ] ] ] ;
cell_tail     = scoped_para
              | table
              | "box" N "." box_item
              | "cc" N ;

change        = [ story "." ] "r" N { "+" N } ;
comment       = "c" N ;
covered       = "c" N ".text" ;
section       = "s" N ;
toc           = "toc" N ;

range         = ranged_block "-" W N ;
```

A `ranged_block` is a `block` with a nonempty structural path. `W` is exactly the terminal step word of that path: `p`, `ps`, `t`, `row`, `cell`, or `cc`. The final number is at least the first endpoint’s final number. The story and every parent step remain unchanged. A range is inclusive. A `p` endpoint cannot end in `ps`, or vice versa; validate the typed step kind and the shared owner before expanding a range.

For example, `t0.row1.cell2.ps1-ps3` selects the cell’s own paragraphs at indices 1 through 3; the abbreviated endpoint inherits the entire cell owner path. It does not refer to story-wide paragraphs 1 through 3. `p14.box0.ps0-ps3` uses that box’s paragraph inventory. `p4-p4` is an accepted singleton selection. `header1.p0-header1.p2` is not an accepted spelling; use `header1.p0-p2`. Refuse mixed kinds such as `t0.row1.cell2.ps1-p3` and `p3-ps7`.

## Complete catalog of forms

| Kind | Every permitted form or production | Representative spellings |
| - | - | - |
| Main-story paragraph | `pN` | `p0`, `p12` |
| Header story | `headerN` | `header0`, `header1` |
| Footer story | `footerN` | `footer0`, `footer2` |
| Other named stories | `footnotes`, `endnotes`, `comments`, `styles`, `numbering` | Exactly those five literals |
| Nonbody story paragraph | `part.pN` | `header1.p0`, `footer2.p7`, `footnotes.p3`, `endnotes.p0`, `comments.p4` |
| Table | `tN` with an optional `part.` prefix or a permitted recursive cell/box parent | `t0`, `header1.t0`, `t0.row1.cell2.t0`, `p14.box0.t0` |
| Row | A table path followed by `.rowN` | `t0.row1`, `header1.t0.row0` |
| Cell | A row path followed by `.cellN` | `t0.row1.cell2`, `footer2.t0.row0.cell0` |
| Scoped cell paragraph selector | A cell path followed by `.psN`, including zero | `t0.row1.cell2.ps0`, `t0.row1.cell2.ps1`, `t0.row1.cell2.ps3` |
| Nested table and descendants | The `table` production under `cell_tail`, recursively | `t0.row1.cell2.t0.row0.cell0` |
| Text box paragraph | A paragraph or cell path, then `.boxN.`, then `scoped_para` | `p14.box0.ps0`, `t0.row1.cell2.box0.ps0` |
| Text box table and descendants | A paragraph or cell path, then `.boxN.`, then `table` | `p14.box0.t0.row0.cell0` |
| Nested text boxes | Repeat `para_tail → boxN.box_item` | `p14.box0.ps1.box0.ps0` |
| Block content control | `ccN`, optionally prefixed by `part.` | `cc0`, `header1.cc0`, `comments.cc1` |
| Inline content control | A paragraph or cell path followed by `.ccN` | `p12.cc0`, `t0.row1.cell2.ps0.cc0`, `p14.box0.ps1.cc0` |
| One body revision | `rN` | `r0`, `r12` |
| One nonbody revision | `story.rN` | `header1.r1`, `footnotes.r3`, `styles.r0`, `numbering.r4` |
| Revision group selector | One revision prefix followed by one or more `+N` components | `r1+2`, `r1+2+3`, `header1.r1+4` |
| Comment | `cN` | `c0`, `c5` |
| Comment coverage selector | `cN.text` | `c5.text` |
| Section | `sN` | `s0`, `s1` |
| Table of contents | `tocN` | `toc0`, `toc1` |
| Story paragraph range | A story-paragraph-ending block path followed by `-pN` | `p3-p7`, `footnotes.p2-p3`, `header1.p0-p2` |
| Scoped paragraph range | A scoped-paragraph-ending block path followed by `-psN` | `t0.row1.cell2.ps1-ps3`, `p14.box0.ps0-ps3` |
| Table range | A table-ending block path followed by `-tN` | `t0-t2`, `header1.t0-t1` |
| Row range | A row-ending block path followed by `-rowN` | `t0.row1-row3` |
| Cell range | A cell-ending block path followed by `-cellN` | `t0.row1.cell0-cell2` |
| Content control range | A control-ending block path followed by `-ccN` | `cc0-cc2`, `p12.cc0-cc1` |

These rows are compositions of one grammar, not separate parsers. Every permitted table, paragraph, cell and box recursion inherits the same number, prefix and suffix rules. `styles` and `numbering` accept only their bare story name and revision forms; they do not have paragraph, table or control forms.

## Forms that do not exist

There is no short entity or selector named `body`, `body.p0`, `body.r0`, `document`, `h0`, `f0`, `textbox1`, `box0`, `row1`, `cell2`, or `ps0` by itself. There is no terminal `.boxN`, `.ccN.pN`, `.ccN.psN`, `p0.ps0`, `header1.ps0`, `c5.p0`, `header1.c5`, `header1.s0`, or `header1.toc0`. The old scoped spellings `t0.row1.cell2.p1` and `p14.box0.p0` are refused after cutover; their new spellings are `t0.row1.cell2.ps1` and `p14.box0.ps0`. `lpN` and `spN` are not aliases.

There are no short families for runs, pictures, bookmarks, individual note containers, ordinary fields, relationships, style definitions, numbering definitions, package parts in general, or a whole body/document. Those remain content or context owned by named entities. Do not invent `img0`, `b0`, `fn2`, `run3`, `part0`, `pid:...`, a UUID or a bare hexadecimal address to expose them.

An inline picture ordinal, a Markdown bookmark anchor or a footnote label can still be part of the Markdown dialect. It is not a Jubarte document address and must not be accepted as one.

A comma list, open range or whitespace-wrapped token is not a short ID. A command that takes several ids may split its option value on commas before parsing (`read -p p0,p1,p5-p7`): the comma is an option-level separator, as a repeated option or a JSON array is, never a character of an id, and each piece goes through the one grammar. Open ranges (`p17-`, `-p3`) are retired; spell the end (`p17-p40`). The current separate parser accepts them; retaining that second grammar would reopen this contract. [Current selection parser](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/markdown/mod.rs#L446-L557).

# 3 Identity and selection have different meanings

## A canonical entity reference names one logical target

For one inspected snapshot, the inventory must have exactly one canonical short entity ID and one canonical long ID per exposed logical target. Resolving either must return the same target. Different targets cannot share either member of the pair.

“Logical target” permits a proven group of physical XML carriers, such as a supported style formatting pair or verified AlternateContent twins. It does not permit arbitrary nodes to be merged because their raw `w:id` happens to match. Grouping belongs to the document model before IDs are assigned. [Existing twin and style grouping](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/changes.rs#L559-L661).

The grammar also contains selectors. Selectors can find an entity through structure, coerce a container to an operation’s target, or select several targets. They are accepted inputs; they are not all valid values of an entity’s serialized `short_id`.

## Settled meanings for the collisions

| Input | Identity or selection meaning | Canonical output after resolution |
| - | - | - |
| `header1` | The header story | Story pair with `short_id: "header1"` |
| `header1.p0` | Its first paragraph | Paragraph pair with `short_id: "header1.p0"` |
| `edit -p header1` | Explicit shorthand for paragraph zero of that story; if absent or unsupported, fail without trying a later paragraph | The actual paragraph pair; never relabel the paragraph as the story |
| `t0.row1.cell2` | The cell entity | Cell pair; its long element is `w:tc` |
| A paragraph-edit command targeting that cell | Explicit shorthand for its first own paragraph | The paragraph’s canonical pair; fail rather than skip if that paragraph cannot be used |
| `t0.row1.cell2.ps0` | Explicitly select the cell’s first own paragraph | Its canonical story-wide pair, for example `p22` |
| `t0.row1.cell2.ps1` | The cell’s second own paragraph | The canonical story-wide paragraph pair, for example `p23` |
| `p14.box0.ps1` | A paragraph in a text-box inventory | The box paragraph pair with canonical short ID `p14.box0.ps1` |
| `p14.box0.t0.row0.cell0.ps0` | First own paragraph of a cell inside that box | The corresponding box-wide paragraph pair, for example `p14.box0.ps4` |
| `p4-p4` | A one-member selection | One paragraph pair for `p4` |
| `r1+2` | A revision selection | Two revision pairs |
| `r43+43` | The same uniquely identified revision selected twice | One pair after deduplication; never two unrelated XML holders |
| `c5.text` | The text covered by comment `c5` | A paired comment reference and paired paragraph/span endpoints |
| `s0` | The first body section | A section pair, distinct from the paragraph holding its section properties |
| `toc0` | The first body TOC as a semantic object | A TOC pair, distinct from a wrapping content control’s pair |

The recommended contract accepts explicit `cell.ps0`; `cell.ps1` is the second own paragraph and so on. This removes the previous positive-only first-paragraph exception. A bare cell remains a cell entity; a paragraph operation may support the documented first-own-paragraph coercion. Explicit `.ps0` and the coercion must select the same paragraph and must fail if it is absent or unsupported, without skipping to a later paragraph. Paragraph identity remains the corresponding story-wide or box-wide canonical paragraph identity.

## Numbering and canonical paragraph paths

Body and nonbody story paragraph indices count actual logical paragraphs in document order, including table-cell paragraphs and empty paragraphs, excluding separate text-box paragraph inventories and saved formatting-history copies. The traversal must have one defined AlternateContent policy and must not double-count proven twin content. Filtering by author, change state, selected paragraphs, pagination or output format occurs after inventory construction and does not renumber IDs.

A paragraph that belongs to the story-wide sequence is canonically `pN` or `part.pN`, even inside a table. A cell-relative `...cellN.psM` is a structural selector that resolves to that paragraph’s canonical pair. For example, `t0.row1.cell2.ps1` may resolve to `p23`; it does not create a second canonical entity ID. The scoped-token amendment preserves these story indices and the established numbering policy. Global indices may leave visible gaps in a filtered or compact view.

Each text-box paragraph inventory likewise includes its descendant table-cell paragraphs and excludes deeper text boxes. Its paragraph spelling is `...boxN.psM`. A cell-relative selector inside a box normalizes to that box-wide paragraph inventory, for example `p14.box0.t0.row0.cell0.ps0` to `p14.box0.ps4`. Thus `ps` means scoped indexing; it does not mean direct XML child in every context.

A text-box paragraph is canonically rooted at its owning paragraph’s canonical path: `p14.box0.ps0`, for example. Its nested tables and text boxes follow the grammar recursively. A cell-based box shortcut is normalized through the first own paragraph of the cell before a canonical entity ID is printed.

Tables count within their actual structural container: top-level tables within the story, nested tables within their cell, and box tables within their box. Rows and cells count actual direct logical children, including physical cells participating in merges. A cell’s own paragraph selector must exclude paragraphs inside nested tables or nested boxes. It must not treat rendered grid columns as XML cell ordinals.

Block-level and other non-inline content controls use a story-wide `ccN` inventory in document order. An inline control belongs to the containing paragraph in the same flow and text-box inventory; ownership lookup must not cross a text-box-content or block/cell boundary to an outer anchor paragraph. Cell-direct and other non-inline controls use the story-wide counter even when an outer anchor paragraph is an XML ancestor. Inline controls use their owning paragraph’s canonical path plus `.ccN`, with nested inline controls in that same paragraph each counted once in document order. Controls are leaves in the short grammar; nesting uses ownership metadata rather than invented `.ccN.ccM` syntax.

Sections are semantic body sections, numbered from zero and always including the final section. An explicit section uses its own `w:sectPr` node as its physical locator. If the final section has no explicit `w:sectPr`, use the `w:body` node as the locator and a `j:short` key; do not create an XML node during read. Both explicit and implicit sections use the registered ELEMENT `w:sectPr` and semantic role `section`. TOCs count body TOC objects from zero; a field plus its wrapping TOC control counts as one TOC. If the wrapper is separately exposed as a content control, it is a distinct logical target with its own `ccN` and long ID.

The current table inspection and edit selectors use different counting conventions: inspection assigns a paragraph to its nearest cell, while the edit resolver’s cell scan includes nested-table descendants. Implement the contract’s own-cell paragraph inventory once and share it. Renaming the token alone does not fix that disagreement. [Inspection](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/inspect/tables.rs#L35-L58) · [Current edit scan](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/edit/origin.rs#L142-L153).

## Canonical entity subset

The broader grammar above is the accepted short-address language. The emitted entity `short_id` subset is narrower, so structural shortcuts cannot become competing primary identities:

```ebnf
cp          = [ part "." ] "p" N { "." "box" N "." "ps" N } ;
table_base  = [ part "." ] "t" N
            | cp "." "box" N "." "t" N ;
ctable      = table_base { "." "row" N "." "cell" N "." "t" N } ;
crow        = ctable "." "row" N ;
ccell       = crow "." "cell" N ;
control     = [ part "." ] "cc" N | cp "." "cc" N ;
revision    = [ story "." ] "r" N ;
entity      = cp | ctable | crow | ccell | control | story
            | revision | "c" N | "s" N | "toc" N ;
```

Here `part` and `story` are the definitions in section 2. No range, revision group or comment coverage selector is an entity-field value. A path through a cell's paragraph or box is normalized through the canonical paragraph inventory before the entity pair is constructed.

## Groups and ranges in JSON

Do not concatenate long IDs, append range syntax to a long ID, or assign one member’s long ID to a whole group.

```json
{
  "selection": [
    {"short_id": "r1", "long_id": "document.w:del.w:id.1"},
    {"short_id": "r2", "long_id": "document.w:ins.w:id.2"}
  ]
}
```

Selection arrays are ordered by canonical inventory order and deduplicated by target identity. Repeating a member does not create a second target. The agent presenter may compact suitable selections as `r1+2` or `p3-p7`; JSON always keeps the individual paired references.

The agent presenter never prints a repeated number. Carriers that shared one source `w:id` have distinct allocated labels (section 4), and a visible change made of several carriers prints them joined, `r0+1`. The former `r43+43` spelling is retired: it was an entity spelling in disguise, and it survived no round trip, since the writer could give only one carrier the number and invented another for the second (`5331b981b7_redline_cicerodo_v_plate_30.docx`: md1 `r43+43`, md2 `r43` on the text and `rev r66` on the id line).

Comment coverage is a span selection, not a second form of the comment’s long ID:

```json
{
  "kind": "comment_text",
  "comment": {"short_id": "c5", "long_id": "comments.w:comment.w:id.5"},
  "spans": [
    {
      "paragraph": {"short_id": "p12", "long_id": "document.w:p.w14:paraId.3F2A1B7C"},
      "start": 4,
      "end": 11
    }
  ]
}
```

Offsets use the engine’s documented Unicode-character coordinate convention, with an inclusive start and exclusive end. They are coordinates, not IDs. A missing or ambiguous comment anchor must be reported; it cannot be replaced with an invented range or an arbitrary first matching paragraph.

# 4 Assigning IDs without changing the source

## One inventory before every view

Build one typed identity inventory from the complete admitted source snapshot. It owns logical entities, their structural positions, native-key candidates, canonical short IDs, canonical long IDs, and the resolution maps. Every presenter and operation uses it.

Build the inventory before filtering, pagination, author selection, `--changed`, or an accepted/rejected rendering projection. Reuse the source identities through those projections. Two reads of the same bytes under the same identity profile must assign the same IDs regardless of ZIP entry order in the reader, hash-map iteration, thread scheduling, requested output format, or selection.

Traversal is fixed: main document; headers by canonical part URI byte order; footers by canonical part URI byte order; footnotes; endnotes; comments; styles; numbering. Within a part use canonical XML document order. Resolve repeated relationships to the same part once. Detect conflicting role assignment and unsupported multiple singleton-role parts.

The logical-content walker must choose AlternateContent according to a fixed, versioned support profile. Do not count verified Choice/Fallback twins twice. Preserve inactive source bytes, unmatched fallback content, and separately listed revision carriers. Do not drop an entity merely because a duplicate-key heuristic incorrectly classified it as a twin.

The body/document and bare text-box containers are context objects, since the closed short grammar gives them no standalone entity ID. Their addressable descendants still have complete pairs. Unsupported XML parts remain preserved package data. A request that promises a complete inventory of unsupported structures must fail or explicitly report incomplete coverage; it must never return long-only entities.

## Header and footer aliases

A short story ID describes the resolved story role. It is not a lossy basename of an arbitrary part path.

For header and footer roles, reserve every canonical suffix belonging to an exact conventional URI such as `/word/header1.xml` or `/word/footer2.xml` before assigning anything else. Those conventional parts retain `header1` and `footer2`. Sort other related parts by their full canonical URI and give them the lowest free `headerN` or `footerN` slot. Their long IDs retain their actual physical part identity.

For example, a header physically stored at `/word/custom/Header_A.xml` can receive the short story alias `header0`; its long PART is the encoded full URI described below. A literal part name `p12.xml` must never become the short story `p12`.

The singleton roles `footnotes`, `endnotes`, `comments`, `styles` and `numbering` each represent one admitted logical part. Multiple references to the same URI are not multiple stories. Multiple distinct parts competing for a singleton role require an explicit ambiguity or coverage diagnostic; do not relabel them as headers or invent `footnotes2`.

The current `part_story()` uses only the last path segment, and the paragraph shortening helper can mistake a part stem such as `p12` for a body paragraph. Using the full part binding is necessary to remove this collision. [Current helpers](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids.rs#L376-L440).

## Revision and comment labels

A *label* is the number in a revision's or comment's short ID, the `N` of `rN` or `cN`. It is a component of the short ID, not a third identity; this section says where that number comes from. Treat raw XML IDs as input data, not ready-made output strings. Preserve missing, empty, malformed and noncanonical source values distinctly in the internal source model.

For revisions, allocate within each short story namespace. For comments, allocate within the comment-definition inventory. First perform the engine’s proven logical grouping. Each logical target then has exactly one primary physical carrier, the canonical first carrier in inventory order, and exactly one possible preserved short-label candidate: that primary carrier’s raw value. For a comment definition the definition itself is the primary carrier. Then:

1. Collect canonical decimal raw values within Word's `int` (`0..=2147483647`, the ceiling the Markdown writer itself keeps as `MOST_ID`) from all physical carriers of every logical target. Reserve every such number before assigning any aliases, including numbers found only on secondary carriers.

2. Preserve the primary carrier’s `rN` or `cN` candidate only when it is canonical, within that range, and every carrier holding that number belongs to this one logical target. A target can preserve at most one label.

3. A number held by different logical targets remains reserved and unassigned. A secondary-only number also remains reserved and unassigned; it is not another alias for its logical target. No ambiguous or secondary label may silently select a first holder.

4. Give each target without a preserved primary label the lowest unused number in canonical inventory order. This includes a noncanonical, missing, empty, ambiguous, negative or oversized primary value; `4294967295` is oversized, since it is above Word's `int`, even though the short grammar can spell it. A canonical value on a secondary carrier does not override that rule.

5. Detect exhaustion and return a typed allocation/resource error. Do not overflow, reuse a number or fall back to a long ID.

For a body inventory encountered in the order shown below:

| Source value | Why | Assigned short ID |
| - | - | - |
| `12` | Unique canonical value | `r12` |
| `43` on target A | Duplicate canonical value | `r0` |
| `43` on target B | Duplicate canonical value | `r1` |
| `007` | Noncanonical decimal spelling | `r2` |
| `1+2` | One literal source value, not a two-change selector | `r3` |
| Missing | No source value | `r4` |
| Empty string | Present but empty | `r5` |
| `4294967295` | Above Word's `int`; jubarte never writes one | `r6` |

Here `r43` is not assigned. An old ambiguous reference to it must not silently select the first target. The agent can show the two actual targets as `r0+1`. If two physical carriers were proven to be one logical revision before allocation, they receive one ID instead.

The Markdown writer already refuses to copy a number above that ceiling back into a file; allocating it at read time is what makes the two reads agree.

A label does not imply that the source XML carries the same number. Every edit resolves the label through the inventory to the exact target. Never turn `r2` into a raw `w:id="2"` query or rewrite `w:id="007"` merely to make the display shorter.

The same rules apply to comments. Enumerate comment nodes before numeric coercion. Preserve ambiguous anchor/thread relationships as diagnostics; do not choose a duplicate definition by first- or last-write wins. Reading and displaying malformed input does not make that input valid, and an operation requiring ambiguous linkage must refuse that operation.

This replaces three current failure modes: legacy long fallback for unusual revision IDs, `noid` for missing revision IDs, and divergent comment inventories caused by raw string formatting in Markdown but `u32` coercion in the comment index. [Revision fallback](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids.rs#L141-L173) · [Missing ID output](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/markdown/from_docx/agent.rs#L268-L321) · [Comment index](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/comments.rs#L479-L519).

## Reading is not stamping

Constructing IDs must not mutate the DOCX. Missing or repeated source keys do not authorize renumbering revisions, rewriting comments, renaming bookmarks, adding XML attributes or repairing a package during a read.

This contract uses `j:short` long keys where the source cannot provide a unique native key (section 5). They are computed metadata, not fictitious attributes claimed to exist in the file.

The earlier proposal to stamp synthetic `w14:paraId` values into every written document is not a prerequisite for this output contract. Any future persistence/stamping feature is separate work with explicit key migration, reference updates, OOXML validation and Word fidelity gates. Existing comment-family integrity stamping and temporary origin tracking do not establish a general public PID mode. [Temporary origin tags](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/edit/origin.rs#L253-L319) · [Comment-family handling](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/comments.rs#L940-L978).

# 5 Long ID grammar and keys

## Exactly four fields

The only long-ID family is:

```text
PART.ELEMENT.ATTRIBUTE.VALUE
```

Examples:

```text
document.w:p.w14:paraId.3F2A1B7C
header1.w:p.w14:paraId.1A2B3C4D
document.w:ins.w:id.12
comments.w:comment.w:id.5
document.w:sdt.w:id.-2D1031330684
document.w:tbl.j:short.t0
document.w:p.j:short.p14.box0.ps0
```

Parse on the first three literal dots. The remainder is the encoded VALUE. ELEMENT and ATTRIBUTE use canonical registered names, independent of the XML file’s chosen namespace prefixes. A syntactically valid address still must resolve to a registered target in the supplied snapshot.

`body:p:12`, `body:rev:12`, `header1:p:0` and `pid:3F2A1B7C` are not long IDs under this contract. They must not be emitted or accepted after cutover.

The four-field family was selected in the prior settlement, but it has no implemented `LongId` type in the reviewed source. The current colon forms are transitional code behavior, not the final grammar. [Prior long specification](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/docs/superpowers/specs/2026-10-13-ids-and-markdown-roundtrip.md#L159-L225) · [Current ID module](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids.rs).

## PART must preserve the whole physical part identity

Use a friendly PART token only as an exact, reversible abbreviation of a conventional physical URI:

| PART | Exactly this URI |
| - | - |
| `document` | `/word/document.xml` |
| `headerN` | `/word/headerN.xml`, with canonical decimal N |
| `footerN` | `/word/footerN.xml`, with canonical decimal N |
| `footnotes` | `/word/footnotes.xml` |
| `endnotes` | `/word/endnotes.xml` |
| `comments` | `/word/comments.xml` |
| `styles` | `/word/styles.xml` |
| `numbering` | `/word/numbering.xml` |

These are physical-URI abbreviations, never dynamically allocated story aliases. A main document at another URI does not get the long PART `document`; a nonstandard header allocated short alias `header0` does not get the long PART `header0`.

For every other admitted part URI, PART is `pkg:` followed by the full canonical URI without its leading slash, escaped byte by byte. Within this payload only ASCII letters, digits and underscore remain literal; every other UTF-8 byte, including dot and hyphen, is `-XX`.

```text
/word/custom/Header_A.xml
  → pkg:word-2Fcustom-2FHeader_A-2Exml

/other/header1.xml
  → pkg:other-2Fheader1-2Exml
```

The second example cannot collide with `header1`, which means only `/word/header1.xml`. Re-encoding a decoded PART must reproduce it exactly. Refuse the `pkg:` spelling of a URI that has a friendly abbreviation, so `pkg:word-2Fdocument-2Exml` is noncanonical.

Canonical OPC URI validation belongs to package admission. A long-ID parser decodes this component once, then looks it up in the already admitted package map. It must never treat it as a filesystem path or an external URL to fetch.

## VALUE escaping

Within VALUE, retain exactly ASCII letters, digits, dot, underscore and colon. Encode every other UTF-8 byte as a hyphen and two uppercase hexadecimal digits. A literal hyphen is always `-2D`.

| Source bytes or value | Encoded VALUE |
| - | - |
| `3F2A1B7C` | `3F2A1B7C` |
| `007` | `007` |
| `-1031330684` | `-2D1031330684` |
| `1+2` | `1-2B2` |
| `A B` | `A-20B` |
| `a.b:c_d` | `a.b:c_d` |
| `å` | `-C3-A5` |
| `%2F` | `-252F` |
| `-2D` | `-2D2D` |

The codec table illustrates encoding, not which values are valid for a particular XML key. For example, `1-2B2` is a valid encoding of a string, but a literal `1+2` is not automatically valid for a registered numeric `w:id` key. Such a malformed source target receives a `j:short` long key.

Reject incomplete escapes, nonhex escapes, lowercase escape hex, invalid UTF-8, unnecessary escapes of characters allowed literally, and any string whose decode/re-encode differs. Decode once. Do not interpret a dot inside VALUE as a cell suffix or another path component.

## Native key whitelist

A native key is usable only when its encoded VALUE is at most 1,024 ASCII bytes, its registered getter yields exactly one eligible value under the policy below, and that value identifies exactly one logical target in the key’s uniqueness bucket. The bucket is the full physical part URI plus registered ELEMENT and ATTRIBUTE. Compare eligible numeric values for conflicts across all physical carriers, and collapse carriers only through proven logical grouping. Preserve the eligible primary source lexical value in the emitted long VALUE.

| Logical target | ELEMENT | ATTRIBUTE | Getter or rule |
| - | - | - | - |
| Paragraph | `w:p` | `w14:paraId` | Paragraph’s own attribute; 8 hex digits |
| Row | `w:tr` | `w14:paraId` | Row’s own attribute; 8 hex digits |
| Insertion/deletion or move carrier | Registered revision element listed below | `w:id` | Its own annotation attribute |
| Formatting/table-structure revision | Registered revision element listed below | `w:id` | Its own annotation attribute |
| Comment definition | `w:comment` | `w:id` | The definition’s own attribute |
| Content control | `w:sdt` | `w:id` | Explicit getter: `w:sdtPr/w:id/@w:val` |

The revision ELEMENT allowlist is exactly:

```text
w:ins             w:del
w:moveFrom        w:moveTo
w:cellIns         w:cellDel
w:rPrChange       w:pPrChange
w:sectPrChange    w:tblPrChange
w:tblPrExChange   w:tblGridChange
w:trPrChange      w:tcPrChange
w:numberingChange w:cellMerge
```

These are key kinds, not a promise that every operation already supports every element in every story. Capability checks remain separate. A move range marker, ordinary field, bookmark, note container or style definition does not gain a new public address merely because the schema has an attribute called `id`.

The content-control spelling retains the earlier selected `w:sdt.w:id` form. It is a registered derived getter, not a claim that `w:sdt` literally carries a `w:id` attribute. Microsoft’s schema documentation distinguishes that child `w:id` element and its `w:val` value. Paragraph and row `w14:paraId` are genuine attributes. [Content control key](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.sdtid?view=openxml-3.0.1) · [Row paragraph key](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.tablerow.paragraphid?view=openxml-3.0.1) · [Annotation identifier](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.trackchangetype.id?view=openxml-3.0.1).

Do not use `w14:textId`, `wp14:editId`, `w:rsid*`, timestamps, author names or text content as identity keys.

## Native key eligibility

Arthur, 2026-10-16: "whatever number Word has given is the number, and if none was given, then we use the short id." Identity therefore takes the file's own key whenever it is lexically a key of its type, lies in the range Word itself would write, and names exactly one logical target in its uniqueness bucket. The ranges are Microsoft's, not this contract's: MS-DOCX says of `paraId` that "values MUST be greater than 0 and less than 0x80000000", and MS-OI29500 says Word restricts `ST_DecimalNumber` to the XML Schema `int` type, signed 32-bit (both checked 2026-10-16). A value outside its range, or otherwise excluded (malformed, duplicated, ambiguous getter, oversized), remains preserved in the document, `jubarte validate` reports it, and its target takes a `j:short` key; exclusion does not by itself declare the whole DOCX invalid. [Paragraph identifier range](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/a0e7d2e2-2246-44c6-96e8-1cf009823615) · [Standard and Word decimal types](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/f22c45b2-8667-41ce-93a3-b1451c17790a).

| Registered key | Exact native-key eligibility |
| - | - |
| Paragraph or row `w14:paraId` | Exactly eight ASCII hex digits; numeric value `1..=0x7FFFFFFF`. Preserve source letter case; compare numeric values for duplicate detection. |
| Registered revision or comment `w:id` | XML-decoded text matches `[+-]?[0-9]+`, numeric value `-2147483648..=2147483647`, no whitespace. Leading sign and zeros preserved in the emitted VALUE. |
| Derived SDT `w:id` | Exactly one applicable `w:sdtPr/w:id/@w:val`; XML-decoded text matches `[+-]?[0-9]+`, numeric value `-2147483648..=2147483647`, no whitespace. Missing or multiple applicable getters make the native key unusable. |

Getter text means the XML-decoded attribute value, not the raw entity-reference spelling in the source bytes. Numeric equivalence is used for the range check and for conflict detection: `7`, `007` and `+7` conflict as native candidates in the same bucket, as do `0`, `+0` and `-0`, and `3F2A1B7C` conflicts with `3f2a1b7c`. Long-ID resolution accepts only the inventory's exact registered spelling. A source key `007` does not create additional accepted long aliases ending in `7` or `+7`.

Version 2 floored revision and comment `w:id` at `0` as "the contract's conservative native-key policy" while giving SDT `w:id` the full signed range. Version 3 uses Word's `int` range for every `w:id` key, because the floor had no source in the specification (Arthur, 2026-10-16: keep the limits that are truthful). A `paraId` above `7FFFFFFF` is the main population this rule sends to `j:short`: the 13 October spec counts 518 of them in 193 corpus files, mostly Google Docs exports. A `w:id` above Word's `int` is no number on either side: section 4 allocates its short ID (`r0`) and its long ID is `document.w:ins.j:short.r0`. jubarte never writes such a number itself (the comparer counts from 1; the Markdown writer's fresh ids stay within `0x7FFFFFFF`, `MOST_ID`, and it drops a view number above that rather than copy it), so the case comes only from other producers. An in-range value the short grammar cannot spell (`007`, `+7`, `-5`) keeps its native long key while its short ID is allocated, so that pair, `r0` and `document.w:ins.w:id.-2D5`, states two facts. The short label rule of section 4 is unchanged: it preserves a canonical `u32` label and allocates for everything else, so `r4294967295` and `document.w:ins.w:id.4294967295` now agree.

## Keyless targets take their short id

If a target has no usable unique native key, ATTRIBUTE is the reserved virtual name `j:short` and VALUE is the target's canonical short entity ID, byte for byte:

```text
PART.ELEMENT.j:short.SHORT
```

```text
document.w:tbl.j:short.t0
document.w:tc.j:short.t0.row1.cell2
document.w:p.j:short.p7
document.w:p.j:short.p14.box0.ps0
document.w:sectPr.j:short.s2
document.j:toc.j:short.toc0
header1.w:hdr.j:short.header1
pkg:word-2Fcustom-2FHeader_A-2Exml.w:hdr.j:short.header0
```

`j:short` is an internal contract key type, not an XML attribute, namespace declaration or third ID family. It is never printed outside a complete long ID in a `long_id` field, and never stamped into the source XML. The VALUE is exactly the string in the pair's `short_id`: a canonical entity spelling from the subset in section 3, never a range, group, coverage or unnormalised cell selector. Entity short IDs contain only ASCII letters, digits and dots, so the VALUE needs no escaping; the dots inside it are not path components to the long-ID parser, which splits on the first three dots only. Optional metadata may describe `key_origin: "short"`; it must not repeat the key as another public ID.

A `j:short` long ID is noncanonical for a target whose native key is usable: resolution refuses it with `ID_NOT_FOUND` and offers the target's real pair as a candidate reference. The two halves of a pair are therefore two different facts when the file gave a key, and one fact written twice when it did not: the long form then adds only the physical part and element to the short form.

Use each object's registered ELEMENT. Physical targets use their own tag; the implicit final section uses the semantic `w:sectPr` convention, and a semantic TOC uses `j:toc`:

| Target without a usable native key | ELEMENT |
| - | - |
| Paragraph, row, table, cell | `w:p`, `w:tr`, `w:tbl`, `w:tc` respectively |
| Revision | Its registered revision ELEMENT |
| Comment, content control | `w:comment`, `w:sdt` |
| Section | `w:sectPr` |
| Header/footer story | `w:hdr`, `w:ftr` |
| Note/comment story | `w:footnotes`, `w:endnotes`, `w:comments` |
| Metadata story | `w:styles`, `w:numbering` |
| Semantic TOC | Reserved virtual ELEMENT `j:toc` |

A table is not its first row. A cell is not its row plus a string suffix. A section is not its owning paragraph. A TOC semantic object is not its wrapping SDT. They have distinct target types and distinct long IDs.

For a logical revision spanning several verified physical carriers, use the same canonical primary carrier for the short-label candidate, native long-key candidate and physical locator. Native uniqueness checks consider eligible candidates on all carriers before collapsing proven members of the same logical target. Secondary carriers do not receive additional public aliases or long-only member addresses. The resulting pair identifies the logical revision, and the model resolves its internal membership.

There is no public `j:path`, `j:key`, UUID form, bare hash form, old colon form, or `.cellN` extension to a long ID. The only synthetic representation is the same four-field long family using `j:short`. Contract version 2 proposed a SHA-256 `j:key` over the file digest and the element's child-ordinal path; it was never implemented and is withdrawn: within one snapshot the hash encoded nothing the structural position did not, it changed on a re-zip that changed no content, and it cost 64 characters an agent cannot read.

# 6 Determinism and stability

## Keyless targets carry no hash

The VALUE of a `j:short` long ID is the short entity ID the inventory assigned. It is produced by the same fixed traversal, the same AlternateContent policy, the same structural numbering and the same alias allocation as the short ID (section 4), so it inherits their determinism and needs no digest, no framing, no collision counter and no collision registry. Two reads of the same bytes under the same identity profile assign the same `j:short` keys; so do two reads of a re-zipped file whose parts are byte-identical, since no file digest enters the key.

Delimited prose paths, host memory addresses, DOM arena indices, random values, wall-clock time and hash-map iteration order must not enter a short ID, and therefore cannot enter a `j:short` key.

## What long ID stability means

| Key origin | Stability guarantee |
| - | - |
| Unique native XML key | Remains the same while its physical part URI, key type, lexical value, uniqueness and logical-target interpretation remain the same |
| `j:short` | Valid in its declared source snapshot, exactly like the short ID it carries; positions and allocated aliases can change after edits; unchanged by a re-zip that leaves the part bytes alone |
| Any short entity ID | Valid in its declared source snapshot; positions and allocated aliases can change after edits |

A `j:short` key is not a persistent cross-write identifier: it moves with the structure it indexes, after an insertion before it or a change of traversal profile. Native keys can also change or become ambiguous after external edits. Do not advertise universal permanence. The long form's value over the short form is the physical part and element it names and, when the file gave a key, that key.

Bind each inventory to its source digest, contract and traversal profile. A complete JSON result or JSONL event carries `schema_version: 2`, `id_contract: "jubarte.ids/2"`, `identity_profile: "canonical-wml-v1"` and `source_sha256`. Every resolved pair inherits this envelope unless an explicit nested input/output snapshot context supplies the digest for that pair. Each standalone JSON/JSONL record is self-contained; it never inherits context from an earlier record. For a failure before a source snapshot is available, `source_sha256` is null; a null digest cannot scope a resolved `IdRef`. The wire examples in sections 1, 3, 6 and 7, other than the complete response in section 1, are payload fragments within this envelope. Section 10 JSON blocks are conformance fixture data, not emitted application responses. Agent Markdown retains its source/digest header; other text views carry source context as metadata, never as another document address. If traversal policy changes, version the profile and affected fixtures. View filters do not change the identity profile.

Apply `STALE_SOURCE` checks to source-bound edits. Resolving a long ID in a different snapshot is an explicit lookup against that new snapshot; it is not permission to ignore an edit plan’s source pin.

## Before and after an edit

An edit report must distinguish the input and output snapshots. A reference in the source inventory and a reference in the output inventory can have the same short spelling while naming different positions.

```json
{
  "operation_label": "op-1",
  "from": {
    "source_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "target": {
      "short_id": "p12",
      "long_id": "document.w:p.w14:paraId.3F2A1B7C"
    }
  },
  "to": {
    "source_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "target": {
      "short_id": "p13",
      "long_id": "document.w:p.w14:paraId.3F2A1B7C"
    }
  }
}
```

A removed target has a source pair and `to: null`. A created target has an output pair and no source target. `j:short` long IDs move with their short IDs across the mapping; do not force equality. For a split or merge, use source and output arrays rather than inventing a one-to-one correspondence.

The agent equivalent uses only short IDs and states the side: `note: source p12 is now output p13`. Compact shift notes such as `source p5-p40 are now output p6-p41` are allowed short selector syntax. The JSON report carries the actual individual pairs and snapshot context.

Origin tracking must follow entities through the edit; it must not reconstruct old references from post-edit indices. Python and WASM must preserve these mappings, including deletion and split/merge cases, rather than leaving them accessible only in raw JSON.

# 7 Errors capabilities and Markdown round trips

## Errors must obey the same output policy

Resolve errors as typed data and render them through the same presenters as successful results. A lower-level error string must not leak a legacy ID, raw XML key or long ID into agent output.

| Condition | Structured code | Identity treatment |
| - | - | - |
| Token outside the settled grammar | `INVALID_ID` | No target pair; identify the argument/field and expected kind |
| Valid grammar but absent target | `ID_NOT_FOUND` | No invented pair; optional real candidate references are paired |
| Recognized target kind unsupported by an operation | `UNSUPPORTED_TARGET` | Include its real pair when already resolved |
| Ambiguous source relationship or logical ownership | `AMBIGUOUS_TARGET` | Pair every known candidate; do not pick one |
| Source digest/profile mismatch | `STALE_SOURCE` | Preserve source context; do not silently retarget |
| Bounds or alias space exhausted | `ID_ALLOCATION_FAILED` or the configured resource-limit code | No fallback ID |
| Requested inventory cannot be complete | `UNSUPPORTED_STRUCTURE` | Explicit coverage/failure, with paired known owners where possible |

In agent errors, show only canonical short references to real targets. Do not echo a rejected raw long/legacy token as if it were an address. Name its input position instead. For example:

```text
error: invalid ID in --id; expected a revision address
error: comment coverage is ambiguous; candidate comments: c2, c3
error: this operation cannot edit t0.row1
```

In JSON, put references in fields rather than interpolating an unpaired string into the message:

```json
{
  "event": "error",
  "code": "UNSUPPORTED_TARGET",
  "message": "This operation does not support a row target.",
  "target": {
    "short_id": "t0.row1",
    "long_id": "document.w:tr.w14:paraId.1A2B3C4D"
  }
}
```

A syntax error can have `target: null`; that is not a half-populated ID. If a suggestion is an actual document target, JSON gives the complete pair. If it is grammar guidance, make it an expected-kind/grammar field rather than a fake entity record.

Parsing invalid public ID syntax exits 2 consistently in native and npm CLIs. Preserve the existing command-level taxonomy for operational failures unless it is changed in the same versioned API work. No wrapper may reformat the identity payload independently. In JSON mode every command-result/error record is structured; any separate human progress stream also obeys short-only identity rendering.

## Valid syntax does not promise edit support

One parser recognizes the whole grammar. A separate capability check says whether a command can operate on that kind of target. Do not make a valid ID into an “invalid spelling” merely because an edit operation is not implemented.

Generate capability declarations from the same typed target kinds. An operation that accepts paragraph edits can explicitly allow a story/cell-to-first-paragraph coercion. A structural table operation can use the table/cell target directly. These are different operation signatures over the same inventory, not different meanings assigned by a printer.

Text-box forms (`…boxN.…`) are accepted syntax from the first release of this grammar and resolve to `UNSUPPORTED_TARGET` until the text-box stage of section 9 lands. Today the view prints a box's paragraphs as unnamed blocks under the host paragraph, `edit` refuses them, and the writer has no text-box output, so a boxed document is not a Markdown fixed point (`Strict01.docx` gains a paragraph on the second pass and every later `pN` shifts by one). The inventory's text-box test must match both `w:txbxContent` and `wne:txbxContent` (`inspect::text_box_names()` already does; the comparer's own walk in `revision_processor.rs` matches `w:` only and must share it); `Strict01.docx` holds the `wne:` form.

Do not claim sections, TOCs, nested tables, text boxes or all nonbody changes are already supported merely because their grammar parses. The current `parse_location` intentionally refuses many of those operations. [Current operation adapter](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/ids.rs#L312-L373).

## Round trips without long IDs in Markdown

Agent Markdown carries short IDs and the existing source/digest context. An anchored Markdown edit resolves those short IDs through the inventory of the pinned DOCX (next section). It must never need a long-ID annotation or `pid:` suffix in the agent text.

A standalone Markdown-to-DOCX conversion regenerates the XML keys the Markdown does not carry (`w14:paraId`, `w:rsid*`) while preserving the represented content and relationships. It cannot promise preservation of an arbitrary original native key that the Markdown does not carry. Native-key fidelity requires the pinned source or an explicit paired-ID JSON artifact, not a hidden third identifier inside Markdown.

**The writer's rule (Arthur, 2026-10-16).** The labels the Markdown names are the one identity it carries, and the standalone writer writes them back as the numbers they are:

- `rN` becomes `w:id="N"` on its carrier, and `story.rN` the same in that part, in document order. A label the Markdown names more than once (one logical change the view marks on several blocks) is written once as its number; its other carriers take the lowest fresh id above the Markdown's largest label, within Word's `int` (today's `MOST_ID` ceiling). The writer never renumbers a label and never allocates below one.
- `cN` becomes the comment's `w:id="N"`; a comment with no number, or with a number an earlier comment took, takes the lowest free number; replies name their parent by the written `w:id`.
- `headerN` and `footerN` write to `/word/headerN.xml` and `/word/footerN.xml`; `footnotes`, `endnotes`, `comments`, `styles` and `numbering` to their conventional URIs. `sN` is section order; `tN`, `pN` and every structural step are positions the written structure reproduces.

This rule is what makes docx → md → docx → md a fixed point for identity. The first read's labels are, by section 4, canonical decimal numbers unique within their story, so the written file holds them as unique native `w:id`s and the second read preserves every one. Two carriers that shared a number in the source hold two labels after the first read (`r0`, `r1`) and two numbers after the first write; the second read agrees. The reviewed code already behaves this way ([revision ids](https://github.com/jandira-tech/jubarte-redlines/blob/7c996819/src/markdown/xml.rs#L449-L470) · [comment ids](https://github.com/jandira-tech/jubarte-redlines/blob/7c996819/src/markdown/write.rs#L3879-L3925)); the rule is contractual from version 3 so that no later writer may regenerate labels `0..n` in document order, which would turn every `r12` into `r0` on the second pass.

`tests/md_fixed_point.rs` and `committed_views_match_the_view` are cutover gates (section 10): every id verdict of every listed document is `correct` before schema 2 is released, and the committed views are regenerated in the same change as the id spellings they print.

A structural edit reports shifts and source/output mapping. The Markdown reader must treat ID-bearing comments and YAML values according to the same grammar as the CLI. It must not interpret a revision alias by copying its digits directly into a source lookup.

Inline bookmark names, hyperlinks, note labels and image references remain Markdown content. Preserve them according to the dialect's fidelity rules; do not turn them into independent long-only identity objects.

## Anchored Markdown edits

Moved here from the 13 October settlement §5.3 (Arthur, 2026-10-16), because the resolver this contract defines is what anchored mode consumes. Anchored mode is the operation goal 2 of the review names, "modify the docx precisely"; it is not implemented on the reviewed branch and is staged in section 9.

- **Two modes.** Standalone (`convert X.md -o X.docx --standalone`) rebuilds from the Markdown alone; the writer's rule above is its identity promise. Anchored applies the Markdown's differences from the source's own view as edits to the source, so everything the Markdown does not express survives byte for byte.
- **Anchored is automatic.** When the YAML `source:` and `sha256:` name a file found beside the Markdown and the digest matches, the conversion anchors on it and prints `note: anchored on SOURCE.docx`; `--standalone` opts out. A digest mismatch is `STALE_SOURCE`, never a silent fall back to standalone.
- **Id lines are the anchors.** Each id line ties a Markdown block to the source entity its short ID resolves to through the pinned inventory, with the same grammar, coercions and capability checks as `edit -p`. A block with no id line is new. A source block whose id line is absent from the Markdown is deleted. Text, notes and YAML entries under an id line are compared with the source's own view of that entity, and only the differences become operations.
- **Everything else is untouched.** Crops, effects, rotation, exact anchor offsets, OLE payloads, `w:rsid*`, native keys and every part the dialect does not describe stay as the source holds them. The native keys of untouched entities are therefore preserved by anchored mode, which is why the Markdown needs no long IDs.
- **The report.** An anchored conversion emits the same edit report as `edit` (section 6): input and output snapshots, paired source and output references, shifts, splits and merges, and typed diagnostics through the shared presenters.

# 8 Rust implementation ownership

## One source of truth

Rust admits the package and XML, builds the logical entity inventory, and assigns typed paired references. The shared agent presenter or JSON serializer renders those references. Native, Python, WASM, npm and MCP transports carry the resulting representation.

The ownership boundaries are mandatory; exact file splitting can follow repository conventions.

| Owner | Responsibility |
| - | - |
| `ids/short` | Closed syntax, portable numeric types, canonical short formatting, entity-versus-selector types |
| `ids/long` | Four-field syntax, registered getters, URI/value codecs, native-key eligibility, `j:short` construction from the canonical short id |
| Identity inventory/index | Structural numbering, role bindings, logical revision grouping, short alias assignment, uniqueness and source scope |
| Command resolver | Convert accepted selectors to actual targets, apply explicit coercions, check capability and staleness |
| Shared wire DTOs | Paired fields for objects and nested references, versioned schemas, reports and diagnostics |
| Agent/diagnostic presenter | Print only typed canonical short entities or valid short selection expressions |
| Markdown writer | Labels to `w:id`, story aliases to conventional part URIs, fresh ids only above the Markdown's largest label; the anchored conversion (section 7) |
| Bindings | Pass through or decode the shared representations, preserving all fields and context |
| Documentation/test generation | Produce spelling catalogs, schema examples and conformance fixtures from the typed contract |

Do not construct long IDs in Python or JavaScript; pass Rust's through. Do not fix npm output by editing a checked-in generated binary or declaration while the Rust source still behaves differently.

## Make invalid combinations difficult to represent

An indicative type boundary is:

```rust
// Sketch of ownership, not a drop-in patch.
pub enum Step {
    StoryParagraph(u32),  // pN at the story root
    ScopedParagraph(u32), // psN after a cell or text box
    Table(u32),
    Row(u32),
    Cell(u32),
    TextBox(u32),
    ContentControl(u32),
}
pub struct ShortEntityId { /* private validated representation */ }
pub struct LongId { /* private validated representation */ }

pub struct IdRef {
    short_id: ShortEntityId,
    long_id: LongId,
}

pub enum Selection {
    Entities(Vec<EntityKey>),
    CommentText {
        comment: EntityKey,
        spans: Vec<TextSpan>,
    },
}

pub struct IdentityIndex {
    // Owns source/profile context, target records and both lookup maps.
    // Only this layer constructs IdRef values for resolved entities.
}
```

A grammar parser may return a broader short-address enum containing ranges/groups/coverage selectors. It must not deserialize those directly into an entity’s `short_id`. Resolution produces validated canonical entity references. `StoryParagraph` and `ScopedParagraph` are different address-step variants, but both resolve to the same underlying paragraph entity type. Parse the complete alphabetic tag before the canonical decimal number, and update `number`, `renumbered`, `Display`, paragraph constructors and every exhaustive match together. A formatter must not infer scope by searching an already-formatted string.

Keep raw source values in dedicated internal types, including missing versus empty. Keep DOM handles, source attributes and operation labels separate from public IDs. A formatter named `short()` must return a short-ID type, not an unconstrained `String` that can hold a long fallback.

The wire serializer emits both fields as one unit. The agent presenter accepts a short-ID value or extracts it from an `IdRef`; it does not receive an arbitrary raw string. Diagnostic rendering must not fall through to a generic `Debug` or `Display` implementation that prints raw source keys.

Use checked integer conversion and arithmetic. The default contract limits are 4,096 ASCII bytes per public ID token, 128 structural steps, and 256 revision members in one group selector. A native encoded VALUE longer than 1,024 bytes uses a `j:short` key. Lower configured limits are allowed when reported consistently; no limit permits truncation or another spelling. Check ranges against actual owner bounds and configured selection budgets before expanding them. Bound allocations before allocating or iterating. Avoid `unwrap`/panics on untrusted syntax. Do not add unsafe code for parsing or formatting IDs. Validate the complete generated short and long tokens against active limits before exposing a target, including the escaped PART and all canonical structural steps. If either representation cannot fit, return a typed resource failure or explicitly permitted incomplete coverage; never truncate, change PART, or emit one half of a pair. Lower runtime resource limits may reject an inventory but must not change key selection, alias allocation or the fixed 1,024-byte native-key eligibility threshold.

Use canonical ordered vectors for traversal/output. Hash maps can accelerate lookup, but their iteration order must not influence aliases, primary carriers, collision counters or serialized order. Build all uniqueness checks before exposing results.

## Required invariants

For each exposed target `e` in one snapshot:

```text
resolve_short(canonical_short(e)) = e
resolve_long(canonical_long(e))   = e
serialize_json(e) contains both, and only those two identity fields
render_agent_ref(e)              = canonical_short(e)
```

Across distinct targets, both canonical short IDs and canonical long IDs are unique. A selector may resolve to an entity whose canonical short spelling differs from the selector; that normalization is deliberate and occurs before serialization.

For accepted canonical syntax, `parse(display(value)) == value`. For printed IDs, `display(parse(printed)) == printed`. A document resolver additionally checks existence, source context and canonical bindings. A string being syntactically valid does not make it an emitted canonical address for that document.

# 9 Migration and required source changes

## Remove the conflicting policies in one cutover

The reviewed code and earlier design contain the following conflicts. The right-hand column is the rule after this contract lands.

| Existing behavior or proposal | Required replacement |
| - | - |
| One `Step::Paragraph` and nested `.pN` spellings | Distinct `StoryParagraph` and `ScopedParagraph` variants; `pN` at story scope and `psN` at cell/box scope |
| First cell paragraph requires implicit cell shorthand | Explicit `.ps0` is valid; the cell remains an entity, and paragraph coercion stays operation-specific |
| Cell paragraph lookup includes nested-table descendants | Shared own-cell paragraph inventory, matching nearest-cell ownership |
| `ChangeId::short()` falls back to `story:rev:raw` | Inventory-assigned `rN` or `story.rN`, always |
| `short_paragraph()` returns legacy/raw strings | Typed canonical paragraph reference; unsupported structure is a diagnostic |
| Missing revision prints `noid` | Allocated short label plus `j:short` long key |
| Paragraph zero prints as `header1` | Paragraph is `header1.p0`; story remains `header1` |
| A cell and its first paragraph share an identity | Cell has its own pair; paragraph has its canonical global pair; edits may coerce explicitly |
| `r43+43` stands for two unrelated raw-ID holders | Distinct allocated labels, printed joined (`r0+1`); a printed id never repeats a number; a repeated member in a selection means repeated selection of one target |
| Table long ID equals its first row long ID | Own `w:tbl` long key |
| Cell long ID appends `.cellN` to a row key | Own `w:tc` long key |
| PART is an arbitrary basename | Exact conventional-URI abbreviation or injective `pkg:` encoding |
| JSON aliases `id`/`ref`, numeric comment IDs and string references | One `short_id`/`long_id` pair everywhere |
| Native/Python/npm construct errors independently | Shared typed diagnostics and serializer |
| `--pids` or `pid:HEX` proposal | No third address family or agent annotation mode |
| Long IDs in machine-hint strings | Paired references in structured hints; short-only references in human hints |
| Automatic source repair needed to make IDs unique | Read-only inventory, aliases and `j:short` long keys |
| Generic inline objects exposed only by native long keys | Content/metadata or explicitly supported paired entities; no long-only records |
| Read has comma/open-range syntax separate from other commands | One closed ID grammar; a comma in an option value is an option-level separator and each piece parses with that grammar; open ranges retired |
| Block numbers use host `usize` width | Portable `u32` wire components with checked host conversion |
| Version 2's SHA-256 `j:key` (never shipped) | `j:short` with the canonical short entity ID; no hash, no counter, no vectors |
| Version 2's `0` floor on revision and comment `w:id` | Word's full `int` range for every `w:id` key; `paraId` keeps Microsoft's `1..=0x7FFFFFFF` |
| Writer free to regenerate revision ids | Labels round-trip: `rN` → `w:id="N"`, `cN` → `w:id="N"`, story aliases → conventional URIs (section 7) |
| Anchored mode governed only by the 13 October spec | Section 7 of this contract; staged below |
| Two text-box tests: `inspect::text_box_names()` matches `w:` and `wne:txbxContent`, the comparer's `revision_processor.rs` matches `w:` only | One shared text-box test, `inspect::text_box_names()`, in every walk |

The absence of `--pids` and `--long-id` in the current flags is intentional context: these are earlier proposals, not already shipped options that need to be removed. Do not accidentally implement them as competing output modes. [Current read flags](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/cli.rs#L55-L142) · [PID proposal](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/docs/2026-10-09-agent-edit-follow-ups.md#L181-L247).

## Concrete implementation sequence

Reordered in version 3 (Arthur, 2026-10-16): the short grammar, the inventory, the presenters and the writer's rule come first, because goals 1 to 3 run on them; long IDs ship with their first consumer.

| Stage | Work and completion condition |
| - | - |
| Urgent 1 | Revised short syntax: split story `pN` from scoped `psN`; parse full tags; portable `u32` steps; explicit cell `ps0`; range endpoint kinds; update `number`, `renumbered`, `Display` and constructors. Grammar vectors pass without old nested `.pN` aliases. |
| Urgent 2 | The source inventory and alias allocator. Preserve established story paragraph numbering, define the cell-own and box paragraph inventories once (shared by read, inspect and edit), use the one text-box test (`inspect::text_box_names()`, already both namespaces) in every walk, and resolve the nested-table counting disagreement. Prove canonicalization, uniqueness, source scope and deterministic allocation. |
| Urgent 3 | Convert inspection and every agent presenter to typed inventory references: `header1.p0`, `sN` from zero, allocated labels joined (`r0+1`, never `r43+43`), `ps` in canonical box paths; remove all string fallbacks (`noid`, `:rev:`, `:p:`). |
| Urgent 4 | The writer's rule (section 7) as code and test; regenerate the committed views; `tests/md_fixed_point.rs` id verdicts all `correct` on its list. |
| ASAP 1 | Move CLI, `read -p` (comma as option separator), Markdown read selection and edit adapters to the shared resolver; explicit `ps0` and the cell coercion agree; truthful capability refusals for boxes, controls, sections, TOCs and nested cells. |
| ASAP 2 | Anchored Markdown edits (section 7): automatic anchoring on `source:`/`sha256:`, `--standalone`, id lines resolved through the pinned inventory, byte-for-byte preservation of everything else, the shared edit report. |
| ASAP 3 | Long IDs with their first consumer: the typed `LongId` (native keys, `j:short`, `pkg:` parts, VALUE codec), the paired `IdRef` in every JSON/JSONL object and nested reference, `--long-id` input on the id-taking options, typed diagnostics with pairs. Nothing in Urgent 1 to 4 or ASAP 1 to 2 depends on this stage. |
| ASAP 4 | Python, WASM, npm and MCP models, parsers, errors, generated declarations, capabilities, help text, saved-plan examples and round-trip fixtures from the same source release. No binding may continue accepting or printing old nested `.pN`. |
| ASAP 5 | Run syntax, owner-resolution, canonicalization, paired-wire, writer-label, anchored and binding-parity gates, then the coordinated schema-2 cutover. Delete retired fallback paths, first-cell special-case suggestions and superseded output fixtures. |
| Future | Text boxes as supported targets: the view prints a box's paragraphs under their own `…boxN.psM` id lines, the writer writes `w:txbxContent`, the fixed-point fixtures gain boxed documents (`tests/fixtures/strict/Strict01.docx`, `ac6cd8d92f.docx`); only then do box ids leave `UNSUPPORTED_TARGET`. Optional persistent stamping with explicit mappings and Word/OOXML validation. |
| Maybe | Additional addressable inline/package entities require an explicit versioned extension. Any future repartition of canonical paragraph ownership must be a separate proposal with renumbering and migration analysis; it is not part of the `ps` amendment. |

These are implementation stages, not permission to ship a half-converted public mode. After the schema-2 cutover, every affected output path must obey the rule. Until then, the release must not claim conformance.

## Source ownership map

| Source area | Specific work |
| - | - |
| `src/ids.rs`, `src/ids/short.rs` | Split paragraph step variants; update tokenization, owner transitions, range kinds, formatting and suggestions; portable bounds; long types and registry in ASAP 3 |
| `src/markdown/xml.rs`, `src/markdown/write.rs` | The writer's rule: labels to `w:id`, aliases to conventional URIs, fresh ids only above the Markdown's largest; the anchored conversion |
| `src/inspect.rs` `text_box_names()`, `src/revision_processor.rs` | The one text-box test (`w:` and `wne:txbxContent`), shared by the inventory, the edit origin and the comparer |
| `src/inspect.rs`, `src/inspect/tables.rs` | Paired entity DTOs, unchanged story counters, structural table/row/cell inventory and shared cell-own/box paragraph ownership |
| `src/changes.rs`, `src/comments.rs` | Exact logical targets, raw key preservation, alias lookup, paired parents/anchors/inside references |
| `src/markdown/from_docx/agent.rs`, `header.rs` | Typed short-only markers and recursive ps paths; preserve global p references, source text and XML names; update notes, comments and section YAML |
| `src/markdown/mod.rs`, Markdown reader modules | Shared p/ps selector parsing and source-bound resolution; remove the separate body-only Pick restriction when adding scoped reads |
| `src/edit.rs`, `src/edit/origin.rs`, edit adapters | Separate story/scoped target matches; shared own-cell enumeration; explicit ps0/coercion parity; paired outcomes and diagnostics |
| `src/validate.rs`, `src/markdown/unified.rs` | Separate XML/line coordinates from document IDs and identify snapshot side |
| `src/bin/jubarte.rs`, `src/cli.rs` | Typed options, consistent syntax exits, shared text/JSON error dispatch |
| `jubarte-python/python/jubarte_redlines/models.py`, MCP server | Versioned paired models, preserve move mappings, remove numeric/string identity assumptions |
| `jubarte-wasm` and npm CLI | Shared serialized output, error parity, regenerate artifacts from matching source |
| ID docs, generated API docs, capabilities and fixture expectations | One generated vocabulary and one wire schema |

The current wire records illustrate why changing only the printer is insufficient: inspection uses `id`, changes use duplicate `id`/`ref`, comments include numeric references, and JSONL renames a paragraph to `at`. Python’s typed edit report also omits the paragraph move mapping. [Inspection DTOs](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/inspect.rs#L45-L244) · [Change DTO](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/changes.rs#L53-L94) · [Comment DTO](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/comments.rs#L43-L78) · [JSONL events](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/edit.rs#L987-L1054) · [Python report](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/jubarte-python/python/jubarte_redlines/models.py#L1264-L1407).

## Compatibility rule

This is a versioned wire change. Schema 2 emits no `id`/`ref` compatibility aliases, old colon IDs, numeric document-ID fields, legacy-ID side channel or opt-in old output format. Numeric comment inputs and bare paragraph-index inputs in structured public plans must migrate to typed short/long address fields as part of that version change. Counts and coordinates remain numbers. Fold the scoped-token amendment into that coordinated schema-2 cutover. Old nested `.pN` spellings are refused, not accepted as permanent aliases. A migration tool may rewrite only structurally identified old paragraph steps and their range-end kinds; it must preserve story `.pN`, XML `w:p`, long IDs, and user-authored text. A parse-error hint may show the new grammar, but must not pretend an unresolved example is a real paired target.

Version 2's `j:key` never shipped, so it needs no migration; a `j:key` long ID is refused like any other retired family.

Version-1 plans receive an explicit unsupported-version/migration error at the version boundary. If maintainers need an offline converter for old plans, make it a separate conversion step requiring the correct source snapshot; it must produce schema-2 references and must not create a permanent permissive parser inside the engine.

Update old tests that intentionally require fallback output. Do not keep those expectations alongside new tests and call both modes supported. Current npm tests still contain retired `body:p:N` expectations, so every binding’s fixtures must be checked against artifacts built from the same commit. [Existing fallback expectations](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/src/markdown/from_docx/agent.rs#L2822-L2852) · [npm expectations](https://github.com/jandira-tech/jubarte-redlines/blob/d6fce6e7ed5f09527ebb3ab22e4c2c9f86165cbd/jubarte-wasm/cli/test/read-view.test.mjs#L125-L143).

# 10 Conformance and release gates

These cases are the completion criteria for the implementation. The grammar/catalog checks accompanying this specification validate the specification's examples; they are not a claim that the current engine or Microsoft Word passed the new contract.

| Case | Required result |
| - | - |
| Envelope and context | Every standalone JSON/JSONL record has the version, contract, profile and source fields; each resolved pair has a known snapshot context; input/output overrides are correct; records need no earlier context line. |
| Output fields | Every resolved JSON/JSONL reference has both mandatory strings; id/ref/pid/numeric aliases are absent. |
| Agent output | Every generated identity comes from the short presenter, including YAML, notes, unplaced changes and errors. |
| Source text preservation | A paragraph containing literal body\:p:12 or a UUID remains intact; the guard checks generated addresses, not document text. |
| Canonical syntax | All accepted short/long strings round-trip through the central types; forbidden forms fail in native and WASM alike. |
| Paragraph scope tokens | Story p and scoped ps are distinct syntax variants; only cells/boxes accept ps; all preserved story examples keep p; old nested p forms and lp/sp aliases are refused. |
| Scoped ranges | Terminal kinds match exactly; cell.ps1-ps3 and box.ps0-ps3 are accepted; mixed p/ps endpoints and cross-owner full endpoints are refused. |
| Short spelling and long stability | The amendment preserves story indices and all long IDs for the same target/source/profile; XML w is unchanged; nested short paths consistently use ps. |
| Entity subset | A range, revision group, coverage selector or unnormalized cell-paragraph selector cannot be serialized as an entity short_id. |
| Story paragraph zero | header1 and header1.p0 resolve to different logical targets; edit coercion reports the paragraph. |
| Cell paragraph zero | Cell and paragraph pairs differ; explicit cell.ps0 and permitted cell edit shorthand resolve to the same first own paragraph; neither skips an absent or unsupported first paragraph. |
| Nested tables | Native read, agent read, inspect tables and edit selectors use the same structural table/row/cell inventory. |
| Nested paragraph selection | A cell's ps1 excludes paragraphs belonging to a nested table; read, inspect and edit resolve it through one owner inventory to the correct canonical story/box paragraph. |
| Boxes | Box containers have no standalone ID; descendants resolve through canonical owning paragraph paths; box forms parse and resolve to `UNSUPPORTED_TARGET` until the text-box stage; both `txbxContent` namespaces are detected. |
| Controls | Block and inline controls have defined counters; nested controls do not invent cc.cc syntax. |
| Sections and TOCs | Section YAML uses sN; explicit/implicit sections use role section with the specified locator; TOCs have distinct j long keys and are not counted twice because of a wrapper. |
| Unusual revisions | 007, literal 1+2, negative, oversized, missing and empty source keys each receive a valid short label. |
| Duplicate revisions | Unrelated repeated numeric holders get distinct aliases; the ambiguous original number stays unassigned. |
| Semantic revision groups | Proven MC twins/style pairs remain one target with one primary short-label candidate; secondary carrier numbers stay reserved; independent duplicates are not collapsed. |
| Comment preservation | Raw comment nodes are inventoried before numeric parsing; no leading-zero collapse, omission or last-write overwrite. |
| Ambiguous comment links | Display can preserve distinct definitions; an edit requiring ambiguous coverage/thread linkage refuses safely. |
| Filtering invariance | Full, selected, changed-only, author-filtered, paged and alternate output views retain identical IDs for retained targets. |
| Part collisions | Conventional header1, another directory's header1.xml, Header_1.xml and p12.xml never collide. |
| Singleton roles | Multiple distinct parts for one singleton role have an explicit coverage/refusal result, not another ID spelling. |
| Long key type | Wrong ELEMENT/ATTRIBUTE pairs, malformed values, unofficial inline keys and appended .cellN suffixes fail. |
| Escaping | All codec vectors pass; lowercase/unnecessary/truncated escapes and invalid UTF-8 fail; decoding occurs once. |
| Key uniqueness | Missing, out-of-range, malformed, ambiguous-getter, duplicate or oversized native keys use `j:short`; distinct target collisions are detected. |
| Short keys | Every keyless target's long ID is `PART.ELEMENT.j:short.` plus its canonical short id; native and WASM agree; a `j:short` for a natively keyed target is refused with the real pair as candidate. |
| Writer labels | docx → md → docx → md preserves every `rN`, `cN`, story alias and `sN`; `tests/md_fixed_point.rs` id verdicts are all `correct` for its list; a document with two carriers on one source `w:id` is a fixed point. |
| Anchored conversion | With a matching `source:`/`sha256:` beside the Markdown the conversion anchors and changes nothing the Markdown did not; a mismatch is `STALE_SOURCE`; `--standalone` rebuilds. |
| Bounded behavior | Overlong tokens, deep paths, excessive group members, past-end ranges and exhausted allocation return typed errors. |
| Source scope | Wrong source hash/profile refuses a pinned edit; selectors are not reapplied to a changed candidate. |
| Edit mapping | Creation, deletion, shifts, splits and merges have paired endpoints in their correct source/output contexts. |
| Read-only identity | Reading and assigning aliases/keys does not change source package bytes or XML attributes. |
| Binding parity | Rust, Python typed/raw results, WASM, npm and MCP agree on all fields, mappings and error payloads. |
| Anchored round trip | Short IDs resolve through the original snapshot; arbitrary native XML IDs need no hidden Markdown annotation. |
| Cutover | Old schema/output expectations are removed together; no compatibility output mode remains. |

Use focused unit/property tests for typed syntax and codecs; fixture tests for inventory semantics, source preservation and edits; and shared golden-wire tests across bindings. After source implementation, run the repository-required formatting, clippy, targeted tests and CLI help smoke gate. Word/OOXML fidelity gates apply to changes that write or alter document structures. No full build or document mutation was needed to prepare this contract.

## Complete short syntax vectors

The following **166 accepted** and **127 refused** strings are portable parser cases. They include the scoped `psN` amendment, explicit cell `ps0`, nested paths, mixed-range refusals and retired nested `.pN` forms. Accepted means the grammar recognizes a spelling, not that a document contains it or every operation supports it.

```json
{
  "accepted": [
    "p0",
    "p12",
    "p123",
    "header0.p0",
    "header1.p0",
    "header1.p2",
    "header4.p7",
    "footer0.p0",
    "footer2.p1",
    "footnotes.p0",
    "footnotes.p3",
    "endnotes.p0",
    "endnotes.p8",
    "comments.p0",
    "comments.p4",
    "p14.box0.ps0",
    "p14.box0.ps1.box0.ps0",
    "header1.p2.box1.ps3",
    "r12",
    "r13",
    "r14",
    "r15",
    "r16",
    "r17",
    "r18",
    "r19",
    "r20",
    "r21",
    "r22",
    "r23",
    "r24",
    "r25",
    "r26",
    "r27",
    "header1.r1",
    "header1.r2",
    "footer2.r3",
    "footer2.r4",
    "footnotes.r5",
    "endnotes.r6",
    "comments.r7",
    "styles.r0",
    "styles.r1",
    "numbering.r4",
    "header4.r8",
    "footnotes.r9",
    "r0",
    "c0",
    "c5",
    "c27",
    "c100",
    "c999",
    "c2147483647",
    "cc0",
    "cc1",
    "p12.cc0",
    "p12.cc1",
    "header1.cc0",
    "footer2.p0.cc0",
    "p14.box0.ps1.cc0",
    "comments.cc0",
    "t0.row0",
    "t0.row1",
    "t2.row3",
    "t0.row1.cell2.t0.row0",
    "t0.row1.cell2.t1.row4",
    "p14.box0.t0.row0",
    "p14.box0.ps1.box0.t0.row1",
    "header1.t0.row0",
    "footer2.t0.row2",
    "footnotes.t0.row0",
    "endnotes.t0.row0",
    "comments.t0.row0",
    "header0.r4",
    "header0.p1",
    "header0.p2",
    "p7",
    "footer2.p0",
    "footnotes.p1",
    "comments.p2",
    "t0",
    "t1",
    "t0.row1.cell2.t0",
    "t0.row1.cell2.t0.row0.cell0.t0",
    "p14.box0.t0",
    "p14.box0.ps1.box0.t0",
    "header1.t0",
    "footer2.t0",
    "footnotes.t0",
    "endnotes.t0",
    "comments.t0",
    "header0.t0",
    "t0.row0.cell0",
    "t0.row0.cell1",
    "t0.row1.cell2.t0.row0.cell0",
    "p14.box0.t0.row0.cell0",
    "header1.t0.row0.cell0",
    "footnotes.t0.row0.cell0",
    "c1",
    "r1",
    "footnotes.r0",
    "header0.r0",
    "s0",
    "s1",
    "s2",
    "toc0",
    "toc1",
    "header1",
    "footer2",
    "footnotes",
    "endnotes",
    "comments",
    "styles",
    "numbering",
    "header0",
    "p4294967295",
    "r4294967295",
    "c4294967295",
    "header4294967295",
    "t4294967295.row0.cell0",
    "p3-p7",
    "p4-p4",
    "footnotes.p2-p3",
    "header1.p0-p2",
    "t0-t2",
    "t0.row1-row3",
    "t0.row1.cell0-cell2",
    "cc0-cc2",
    "p12.cc0-cc1",
    "t0.row1.cell2.ps1-ps3",
    "r1+2",
    "r1+2+3",
    "r43+43",
    "header1.r1+4",
    "styles.r0+1",
    "numbering.r2+3",
    "c5.text",
    "c0.text",
    "t0.row1.cell2.ps1",
    "t0.row1.cell2.box0.ps0",
    "t0.row1.cell2.cc0",
    "t0.row1.cell2.ps1.cc0",
    "t0.row1.cell2.ps1.box0.t0.row0.cell0",
    "header1.t0.row0.cell0.ps1",
    "p14.box0.ps1-ps3",
    "p14.box0.t0.row0.cell0.box0.ps0",
    "footer2.p7",
    "t0.row1.cell2.ps0",
    "t0.row1.cell2.ps0-ps3",
    "t0.row1.cell2.ps0-ps0",
    "t0.row1.cell2.ps0.cc0",
    "t0.row1.cell2.ps0.box0.ps0",
    "p14.box0.ps0-ps3",
    "p14.box0.ps1.box0.t0.row0.cell0.ps0",
    "header1.p0.box0.ps0",
    "header1.p2.box1.ps3.cc0",
    "header1.t0.row0.cell0.ps0",
    "footer2.t0.row0.cell0.ps0",
    "footnotes.t0.row0.cell0.ps0",
    "endnotes.t0.row0.cell0.ps0",
    "comments.t0.row0.cell0.ps0",
    "t0.row1.cell2.t0.row0.cell0.ps0",
    "t0.row1.cell2.ps1.box0.ps0",
    "t0.row1.cell2.ps4294967295",
    "p14.box0.ps4294967295",
    "p14.box0.ps0-ps4294967295"
  ],
  "refused": [
    "",
    " ",
    " p12",
    "p12 ",
    "p01",
    "p+1",
    "p-1",
    "P1",
    "p١",
    "p１２",
    "p4294967296",
    "r4294967296",
    "c4294967296",
    "header4294967296",
    "12",
    "#12",
    "#c5",
    "#c5.text",
    "1+2",
    "t0.r1.c2",
    "body",
    "body.p0",
    "body.r0",
    "document",
    "h0",
    "f0",
    "header01",
    "Header1",
    "header-1",
    "header1.header2.p0",
    "textbox1.p0",
    "p14.box0",
    "box0.p0",
    "row1",
    "cell2",
    "p0.row0",
    "t0.cell0",
    "t0.row0.p0",
    "t0.row0.cell0.p0",
    "t0.row0.cell0.p0.cc0",
    "p0.p1",
    "cc0.p0",
    "p0.cc0.p1",
    "p0.box0.cc0",
    "styles.p0",
    "styles.t0",
    "numbering.p0",
    "comments.c5",
    "header1.c5",
    "header1.s0",
    "header1.toc0",
    "c5.p0",
    "c5.text.p0",
    "c05",
    "r01",
    "r1+r2",
    "r1++2",
    "r1+02",
    "header1.r1+footer2.r2",
    "r1-r3",
    "c1-c2",
    "s0-s1",
    "toc0-toc1",
    "p7-p3",
    "p3-t7",
    "p3-p7-p9",
    "header1.p0-header1.p2",
    "header1-header2",
    "p17-",
    "-p3",
    "p0,p1",
    "body:p:12",
    "body:rev:12",
    "body:rev:007",
    "header1:p:0",
    "pid:3F2A1B7C",
    "3F2A1B7C",
    "p12~3F2A",
    "noid",
    "footnotes.noid",
    "p14.box0.p0",
    "p14.box0.p1.box0.p0",
    "header1.p2.box1.p3",
    "p14.box0.p1.cc0",
    "p14.box0.p1.box0.t0.row1",
    "p14.box0.p1.box0.t0",
    "t0.row1.cell2.p1-p3",
    "t0.row1.cell2.p1",
    "t0.row1.cell2.box0.p0",
    "t0.row1.cell2.p1.cc0",
    "t0.row1.cell2.p1.box0.t0.row0.cell0",
    "header1.t0.row0.cell0.p1",
    "p14.box0.p1-p3",
    "p14.box0.t0.row0.cell0.box0.p0",
    "ps0",
    "ps1",
    "ps4294967295",
    "header1.ps0",
    "footer2.ps7",
    "footnotes.ps3",
    "endnotes.ps0",
    "comments.ps4",
    "styles.ps0",
    "numbering.ps0",
    "p14.ps0",
    "p0.ps1",
    "p0.box0.ps0.ps1",
    "t0.ps0",
    "t0.row0.ps0",
    "t0.row1.cell2.ps1-p3",
    "p14.box0.ps1-p3",
    "p3-ps7",
    "t0.row1.cell2.ps01",
    "t0.row1.cell2.PS1",
    "t0.row1.cell2.pss1",
    "t0.row1.cell2.ps-1",
    "t0.row1.cell2.ps4294967296",
    "p14.box0.ps4294967296",
    "t0.row1.cell2.ps3-ps1",
    "p14.box0.ps1-ps3-ps5",
    "p14.box0.ps1-p14.box0.ps3",
    "p14.box0.ps1-t3",
    "p14.box0.p1-ps3",
    "t0.row1.cell2.p1-ps3",
    "t0.row1.cell2.lp0",
    "t0.row1.cell2.sp0",
    "ps1-ps3"
  ]
}
```

## Long syntax refusals

These 48 cases cover retired families (`body:`, `pid:`, `j:path`, `j:key`), unsupported key kinds, malformed, out-of-range or noncanonical values and escaping, and `j:short` values that are not canonical entity ids. A native VALUE outside its registered range (`paraId` outside `1..=0x7FFFFFFF`, `w:id` outside Word's `int`) is refused as syntax, since no inventory can hold it as a key. `document.w:ins.w:id.-2D1` is deliberately not in the list: `-1` is within Word's `int`, so it parses and resolves when the file holds that unique value. All 130 positive catalog examples appear in section 11.

```json
[
  "body:p:12",
  "body:rev:12",
  "body:rev:007",
  "header1:p:0",
  "pid:3F2A1B7C",
  "document.w:p.w14:paraId.",
  "document.w:p.w14:paraId.1234",
  "document.w:p.w14:paraId.G123ABCD",
  "document.w:p.w14:paraId.3F2A1B7C.cell0",
  "document.w:tbl.w14:paraId.3F2A1B7C",
  "document.w:p.w14:textId.3F2A1B7C",
  "document.w:r.w:id.12",
  "document.w:bookmarkStart.w:name._Ref123",
  "footnotes.w:footnote.w:id.2",
  "styles.w:style.w:styleId.Heading1",
  "document.w:p.j:path.body.p0",
  "document.w:p.j:key.1234",
  "document.w:p.j:key.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "document.w:p.j:short.",
  "document.w:p.j:short.p3-p7",
  "document.w:p.j:short.t0.row1.cell2.ps1",
  "document.w:tbl.j:short.t0.row1",
  "document.w:p.j:short.body.p0",
  "document.w:p.j:short.P12",
  "document.w:p.j:short.p012",
  "header1.w:p.j:short.p0",
  "document.w:ins.w:id.1-2B2",
  "document.w:ins.w:id.x9",
  "document.w:ins.w:id.12-",
  "document.w:ins.w:id.12-2",
  "document.w:sdt.w:id.-2d1",
  "document.w:sdt.w:id.-321",
  "pkg:word-2Fdocument-2Exml.w:p.w14:paraId.3F2A1B7C",
  "pkg:word-2fcustom-2FHeader_A-2Exml.w:p.w14:paraId.3F2A1B7C",
  "pkg:word/custom/Header_A.xml.w:p.w14:paraId.3F2A1B7C",
  "Header_A.w:p.w14:paraId.3F2A1B7C",
  "header01.w:p.w14:paraId.3F2A1B7C",
  "document.w:p.w14:paraId.-331234567",
  "document.w:p.w14:paraId.3F2A1B7C ",
  "document.w:p.w14:paraId.00000000",
  "document.w:p.w14:paraId.80000000",
  "document.w:tr.w14:paraId.FFFFFFFF",
  "document.w:ins.w:id.2147483648",
  "document.w:ins.w:id.-2D2147483649",
  "comments.w:comment.w:id.2147483648",
  "document.w:sdt.w:id.2147483648",
  "document.w:sdt.w:id.-2D2147483649",
  "document.w:ins.w:id.12-20"
]
```

The `j:short` refusals: an empty value; a range or an unnormalised cell selector (neither is an entity spelling); a short id whose kind disagrees with ELEMENT (`t0.row1` is a row, not a `w:tbl`); the nonexistent `body` story; a case or leading-zero variant; and a story paragraph without its story (`header1.p0` is the short id, so the VALUE carries it even though PART already says `header1`). `document.w:tbl.j:short.t0` resolves only while `t0` has no native key, which for a `w:tbl` is always.

## Alias and codec vectors

Each ordinary alias case inventories distinct logical source targets in the shown order. JSON null means a missing source attribute; an empty string means a present empty attribute. In each `grouped_alias_cases` entry, a nested list contains all raw carrier values for one already-proven logical target, with its canonical primary carrier first. `expected` has exactly one short ID per logical target. `reserved_unassigned` lists source numbers that must not resolve as additional aliases.

```json
{
  "alias_cases": [
    {
      "raw": [
        "12",
        "43",
        "43",
        "007",
        "1+2",
        null,
        ""
      ],
      "prefix": "r",
      "expected": [
        "r12",
        "r0",
        "r1",
        "r2",
        "r3",
        "r4",
        "r5"
      ]
    },
    {
      "raw": [
        "0",
        "2",
        "7",
        "007",
        "1+2",
        null,
        "",
        "4294967296"
      ],
      "prefix": "r",
      "expected": [
        "r0",
        "r2",
        "r7",
        "r1",
        "r3",
        "r4",
        "r5",
        "r6"
      ]
    },
    {
      "raw": [
        "5",
        "005",
        "5",
        null,
        ""
      ],
      "prefix": "c",
      "expected": [
        "c0",
        "c1",
        "c2",
        "c3",
        "c4"
      ]
    },
    {
      "raw": [
        "4294967295",
        "007",
        "0"
      ],
      "prefix": "r",
      "expected": [
        "r1",
        "r2",
        "r0"
      ]
    },
    {
      "raw": [
        "2147483647",
        "2147483648"
      ],
      "prefix": "r",
      "expected": [
        "r2147483647",
        "r0"
      ]
    }
  ],
  "grouped_alias_cases": [
    {
      "carrier_values": [
        [
          "10",
          "11"
        ],
        [
          "20"
        ]
      ],
      "prefix": "r",
      "expected": [
        "r10",
        "r20"
      ],
      "reserved_unassigned": [
        11
      ]
    },
    {
      "carrier_values": [
        [
          "10",
          "11"
        ],
        [
          "11"
        ],
        [
          null,
          "21"
        ]
      ],
      "prefix": "r",
      "expected": [
        "r10",
        "r0",
        "r1"
      ],
      "reserved_unassigned": [
        11,
        21
      ]
    }
  ],
  "codec_cases": [
    {
      "raw": "3F2A1B7C",
      "encoded": "3F2A1B7C"
    },
    {
      "raw": "007",
      "encoded": "007"
    },
    {
      "raw": "-1031330684",
      "encoded": "-2D1031330684"
    },
    {
      "raw": "1+2",
      "encoded": "1-2B2"
    },
    {
      "raw": "A B",
      "encoded": "A-20B"
    },
    {
      "raw": "a.b:c_d",
      "encoded": "a.b:c_d"
    },
    {
      "raw": "å",
      "encoded": "-C3-A5"
    },
    {
      "raw": "%2F",
      "encoded": "-252F"
    },
    {
      "raw": "-2D",
      "encoded": "-2D2D"
    }
  ]
}
```

## Short key vectors

These replace version 2's hash vectors. Each is checkable by eye: the long ID is PART, ELEMENT, `j:short` and the short id, joined by dots.

```json
{
  "vectors": [
    {"short_id": "t0", "part_uri": "/word/document.xml", "element": "w:tbl", "expected_long_id": "document.w:tbl.j:short.t0"},
    {"short_id": "t0.row1.cell2", "part_uri": "/word/document.xml", "element": "w:tc", "expected_long_id": "document.w:tc.j:short.t0.row1.cell2"},
    {"short_id": "p14.box0.ps0", "part_uri": "/word/document.xml", "element": "w:p", "expected_long_id": "document.w:p.j:short.p14.box0.ps0"},
    {"short_id": "s2", "part_uri": "/word/document.xml", "element": "w:sectPr", "expected_long_id": "document.w:sectPr.j:short.s2"},
    {"short_id": "toc0", "part_uri": "/word/document.xml", "element": "j:toc", "expected_long_id": "document.j:toc.j:short.toc0"},
    {"short_id": "header0", "part_uri": "/word/custom/Header_A.xml", "element": "w:hdr", "expected_long_id": "pkg:word-2Fcustom-2FHeader_A-2Exml.w:hdr.j:short.header0"},
    {"short_id": "r4", "part_uri": "/word/document.xml", "element": "w:ins", "expected_long_id": "document.w:ins.j:short.r4"},
    {"short_id": "footnotes.p1", "part_uri": "/word/footnotes.xml", "element": "w:p", "expected_long_id": "footnotes.w:p.j:short.footnotes.p1"}
  ]
}
```

A test must also check the negative direction: `parse(long).value == short_id` for every row, and a `j:short` whose value is not the canonical short id of a target of that ELEMENT in that PART is `ID_NOT_FOUND`.

# 11 Long ID catalog

This is a **non-exhaustive list of 130 complete long IDs**, paired with their short entity IDs: **82 native-key examples and 48 short-key examples**. Together with the closed registries and grammar, these cover the relevant shapes without pretending that a finite list can enumerate every document.

Each row is an independent illustrative case. Repeated short spellings across different rows do not refer to one shared snapshot. Native-key rows assume the key exists, is lexically a key of its type within its registered range, and uniquely identifies the logical target. Short-key rows need no tuple: the VALUE is the short id in the first column, so every row is checkable by eye.

Story-prefixed examples include grammar/key combinations whose operation support still requires the capability checks in section 7. A nonstandard physical part can have an allocated short role alias while its long PART preserves its actual URI.

## Paragraphs and text box paragraphs

| Short entity ID | Long ID |
| - | - |
| `p0` | `document.w:p.w14:paraId.10000001` |
| `p12` | `document.w:p.w14:paraId.10000002` |
| `p123` | `document.w:p.w14:paraId.10000003` |
| `header0.p0` | `header0.w:p.w14:paraId.10000004` |
| `header1.p0` | `header1.w:p.w14:paraId.10000005` |
| `header1.p2` | `header1.w:p.w14:paraId.10000006` |
| `header4.p7` | `header4.w:p.w14:paraId.10000007` |
| `footer0.p0` | `footer0.w:p.w14:paraId.10000008` |
| `footer2.p1` | `footer2.w:p.w14:paraId.10000009` |
| `footnotes.p0` | `footnotes.w:p.w14:paraId.1000000A` |
| `footnotes.p3` | `footnotes.w:p.w14:paraId.1000000B` |
| `endnotes.p0` | `endnotes.w:p.w14:paraId.1000000C` |
| `endnotes.p8` | `endnotes.w:p.w14:paraId.1000000D` |
| `comments.p0` | `comments.w:p.w14:paraId.1000000E` |
| `comments.p4` | `comments.w:p.w14:paraId.1000000F` |
| `p14.box0.ps0` | `document.w:p.w14:paraId.10000010` |
| `p14.box0.ps1.box0.ps0` | `document.w:p.w14:paraId.10000011` |
| `header1.p2.box1.ps3` | `header1.w:p.w14:paraId.10000012` |

## Body revisions

| Short entity ID | Long ID |
| - | - |
| `r12` | `document.w:ins.w:id.12` |
| `r13` | `document.w:del.w:id.13` |
| `r14` | `document.w:moveFrom.w:id.14` |
| `r15` | `document.w:moveTo.w:id.15` |
| `r16` | `document.w:cellIns.w:id.16` |
| `r17` | `document.w:cellDel.w:id.17` |
| `r18` | `document.w:rPrChange.w:id.18` |
| `r19` | `document.w:pPrChange.w:id.19` |
| `r20` | `document.w:sectPrChange.w:id.20` |
| `r21` | `document.w:tblPrChange.w:id.21` |
| `r22` | `document.w:tblPrExChange.w:id.22` |
| `r23` | `document.w:tblGridChange.w:id.23` |
| `r24` | `document.w:trPrChange.w:id.24` |
| `r25` | `document.w:tcPrChange.w:id.25` |
| `r26` | `document.w:numberingChange.w:id.26` |
| `r27` | `document.w:cellMerge.w:id.27` |

## Revisions in other stories

| Short entity ID | Long ID |
| - | - |
| `header1.r1` | `header1.w:ins.w:id.1` |
| `header1.r2` | `header1.w:del.w:id.2` |
| `footer2.r3` | `footer2.w:moveFrom.w:id.3` |
| `footer2.r4` | `footer2.w:moveTo.w:id.4` |
| `footnotes.r5` | `footnotes.w:rPrChange.w:id.5` |
| `endnotes.r6` | `endnotes.w:pPrChange.w:id.6` |
| `comments.r7` | `comments.w:ins.w:id.7` |
| `styles.r0` | `styles.w:pPrChange.w:id.0` |
| `styles.r1` | `styles.w:rPrChange.w:id.1` |
| `numbering.r4` | `numbering.w:numberingChange.w:id.4` |
| `header4.r8` | `header4.w:tblPrChange.w:id.8` |
| `footnotes.r9` | `footnotes.w:cellMerge.w:id.9` |

## Unusual native lexical values

| Short entity ID | Long ID |
| - | - |
| `r0` | `document.w:ins.w:id.007` |
| `c0` | `comments.w:comment.w:id.007` |

## Comment definitions

| Short entity ID | Long ID |
| - | - |
| `c0` | `comments.w:comment.w:id.0` |
| `c5` | `comments.w:comment.w:id.5` |
| `c27` | `comments.w:comment.w:id.27` |
| `c100` | `comments.w:comment.w:id.100` |
| `c999` | `comments.w:comment.w:id.999` |
| `c2147483647` | `comments.w:comment.w:id.2147483647` |

## Content controls

| Short entity ID | Long ID |
| - | - |
| `cc0` | `document.w:sdt.w:id.42` |
| `cc1` | `document.w:sdt.w:id.-2D1031330684` |
| `p12.cc0` | `document.w:sdt.w:id.-2D1` |
| `p12.cc1` | `document.w:sdt.w:id.2147483647` |
| `header1.cc0` | `header1.w:sdt.w:id.12` |
| `footer2.p0.cc0` | `footer2.w:sdt.w:id.27` |
| `p14.box0.ps1.cc0` | `document.w:sdt.w:id.314` |
| `comments.cc0` | `comments.w:sdt.w:id.9` |

## Native row keys

| Short entity ID | Long ID |
| - | - |
| `t0.row0` | `document.w:tr.w14:paraId.20000001` |
| `t0.row1` | `document.w:tr.w14:paraId.20000002` |
| `t2.row3` | `document.w:tr.w14:paraId.20000003` |
| `t0.row1.cell2.t0.row0` | `document.w:tr.w14:paraId.20000004` |
| `t0.row1.cell2.t1.row4` | `document.w:tr.w14:paraId.20000005` |
| `p14.box0.t0.row0` | `document.w:tr.w14:paraId.20000006` |
| `p14.box0.ps1.box0.t0.row1` | `document.w:tr.w14:paraId.20000007` |
| `header1.t0.row0` | `header1.w:tr.w14:paraId.20000008` |
| `footer2.t0.row2` | `footer2.w:tr.w14:paraId.20000009` |
| `footnotes.t0.row0` | `footnotes.w:tr.w14:paraId.2000000A` |
| `endnotes.t0.row0` | `endnotes.w:tr.w14:paraId.2000000B` |
| `comments.t0.row0` | `comments.w:tr.w14:paraId.2000000C` |

## Nonconventional physical parts

| Short entity ID | Long ID |
| - | - |
| `header0.p0` | `pkg:word-2Fcustom-2FHeader_A-2Exml.w:p.w14:paraId.3F2A1B7C` |
| `footer0.p0` | `pkg:other-2Ffooter1-2Exml.w:p.w14:paraId.1A2B3C4D` |
| `p0` | `pkg:custom-2Fmain-2Exml.w:p.w14:paraId.1234ABCD` |
| `header0.r4` | `pkg:word-2FHeader_1-2Exml.w:ins.w:id.4` |
| `header0.p1` | `pkg:other-2Fheader1-2Exml.w:p.w14:paraId.11223344` |
| `header0.p2` | `pkg:word-2Fheader01-2Exml.w:p.w14:paraId.55667788` |
| `c5` | `pkg:review-2Fcomments-2Exml.w:comment.w:id.5` |
| `styles.r1` | `pkg:theme-2Fstyles-2Exml.w:rPrChange.w:id.1` |

## Short paragraph keys

| Short entity ID | Long ID |
| - | - |
| `p7` | `document.w:p.j:short.p7` |
| `header1.p0` | `header1.w:p.j:short.header1.p0` |
| `footer2.p0` | `footer2.w:p.j:short.footer2.p0` |
| `footnotes.p1` | `footnotes.w:p.j:short.footnotes.p1` |
| `comments.p2` | `comments.w:p.j:short.comments.p2` |
| `p14.box0.ps0` | `document.w:p.j:short.p14.box0.ps0` |

## Table keys

| Short entity ID | Long ID |
| - | - |
| `t0` | `document.w:tbl.j:short.t0` |
| `t1` | `document.w:tbl.j:short.t1` |
| `t0.row1.cell2.t0` | `document.w:tbl.j:short.t0.row1.cell2.t0` |
| `t0.row1.cell2.t0.row0.cell0.t0` | `document.w:tbl.j:short.t0.row1.cell2.t0.row0.cell0.t0` |
| `p14.box0.t0` | `document.w:tbl.j:short.p14.box0.t0` |
| `p14.box0.ps1.box0.t0` | `document.w:tbl.j:short.p14.box0.ps1.box0.t0` |
| `header1.t0` | `header1.w:tbl.j:short.header1.t0` |
| `footer2.t0` | `footer2.w:tbl.j:short.footer2.t0` |
| `footnotes.t0` | `footnotes.w:tbl.j:short.footnotes.t0` |
| `endnotes.t0` | `endnotes.w:tbl.j:short.endnotes.t0` |
| `comments.t0` | `comments.w:tbl.j:short.comments.t0` |
| `header0.t0` | `pkg:word-2Fcustom-2FHeader_A-2Exml.w:tbl.j:short.header0.t0` |

## Short row keys

| Short entity ID | Long ID |
| - | - |
| `t0.row0` | `document.w:tr.j:short.t0.row0` |
| `t0.row1.cell2.t0.row0` | `document.w:tr.j:short.t0.row1.cell2.t0.row0` |
| `header1.t0.row0` | `header1.w:tr.j:short.header1.t0.row0` |

## Cell keys

| Short entity ID | Long ID |
| - | - |
| `t0.row0.cell0` | `document.w:tc.j:short.t0.row0.cell0` |
| `t0.row0.cell1` | `document.w:tc.j:short.t0.row0.cell1` |
| `t0.row1.cell2.t0.row0.cell0` | `document.w:tc.j:short.t0.row1.cell2.t0.row0.cell0` |
| `p14.box0.t0.row0.cell0` | `document.w:tc.j:short.p14.box0.t0.row0.cell0` |
| `header1.t0.row0.cell0` | `header1.w:tc.j:short.header1.t0.row0.cell0` |
| `footnotes.t0.row0.cell0` | `footnotes.w:tc.j:short.footnotes.t0.row0.cell0` |

## Missing malformed or duplicate native keys

| Short entity ID | Long ID |
| - | - |
| `cc0` | `document.w:sdt.j:short.cc0` |
| `p12.cc0` | `document.w:sdt.j:short.p12.cc0` |
| `c0` | `comments.w:comment.j:short.c0` |
| `c1` | `comments.w:comment.j:short.c1` |
| `r0` | `document.w:ins.j:short.r0` |
| `r1` | `document.w:ins.j:short.r1` |
| `footnotes.r0` | `footnotes.w:del.j:short.footnotes.r0` |
| `header0.r0` | `pkg:word-2Fcustom-2FHeader_A-2Exml.w:rPrChange.j:short.header0.r0` |

## Sections

Explicit and implicit sections alike are `document.w:sectPr.j:short.sN`; `s2` here is an implicit final section, which has no `w:sectPr` node of its own, and none is created.

| Short entity ID | Long ID |
| - | - |
| `s0` | `document.w:sectPr.j:short.s0` |
| `s1` | `document.w:sectPr.j:short.s1` |
| `s2` | `document.w:sectPr.j:short.s2` |

## Tables of contents

| Short entity ID | Long ID |
| - | - |
| `toc0` | `document.j:toc.j:short.toc0` |
| `toc1` | `document.j:toc.j:short.toc1` |

## Whole nonbody stories

| Short entity ID | Long ID |
| - | - |
| `header1` | `header1.w:hdr.j:short.header1` |
| `footer2` | `footer2.w:ftr.j:short.footer2` |
| `footnotes` | `footnotes.w:footnotes.j:short.footnotes` |
| `endnotes` | `endnotes.w:endnotes.j:short.endnotes` |
| `comments` | `comments.w:comments.j:short.comments` |
| `styles` | `styles.w:styles.j:short.styles` |
| `numbering` | `numbering.w:numbering.j:short.numbering` |
| `header0` | `pkg:word-2Fcustom-2FHeader_A-2Exml.w:hdr.j:short.header0` |

## Deliberately absent public long IDs

A native-looking key is not automatically a public long ID. Under this closed short grammar, the following objects remain content/context and do not receive an unpaired public address: bookmark endpoints, pictures, runs, individual note containers, ordinary fields, style definitions, numbering definitions, relationship entries, the body/document root and bare text-box containers.

Therefore examples such as `document.w:bookmarkStart.w:name._Ref123`, `footnotes.w:footnote.w:id.2`, or `styles.w:style.w:styleId.Heading1` from broader schema-oriented proposals are not public entity IDs in this contract. Their content and names are preserved; their containing paragraph/story can be referenced with a complete pair. Never pair a paragraph's short ID with a bookmark's long key.

Ranges, revision groups and `cN.text` likewise have no fabricated single long ID. Their JSON representation contains the actual paired members or span endpoints defined in section 3.
