<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Plan: spec §8 items 1, 2 (slug fixes only), 3, 4, 5 (track_changes only), 6

No code changed. All change points were read in the tree on branch `cli-std/25a-reader-threads`. Ordered smallest-safe-first; each is a PR over the previous (spec §7 discipline).

Repo constraints honored throughout: test-first (red then green); no `#[allow]`; `clippy -D warnings`; `cargo nextest`, one cargo process at a time; Word only via the bench scripts; Microsoft Word parity, never LibreOffice.

## Arthur's decisions (2026-10-13)

Asked as eight choices with three alternatives each; his answer: "1a 2a but the cli should be able to override 3a 4a 5a 6a 7c 8c (probably, investigate, i dropped a few fixtures with the scenarios)".

1. **Columns** (A): a `page:` clause on the section line (`columns 2, column gap 0.5in`). Done (48f667ab).
2. **`track_changes`** (A + override): the writer honours the YAML key; a CLI flag on `jubarte convert … -t docx` overrides it (on or off) whatever the header says. Task 2 carries the flag.
3. **Checklist mixed with plain bullets** (A): numbering per paragraph. Task items take the checklist numbering, plain items the bullet numbering, in the same list.
4. **Intraword `H~2~O`** (A): left as pulldown-cmark parses it (no subscript inside a word); documented in `docs/MARKDOWN.md`.
5. **Paragraph spacing** (A): a spacing clause on the id line only when the paragraph differs from its style.
6. **A footnote referenced twice** (A): two notes, as today (Word ties one reference to one note).
7. **`#link` with no target** (C): the writer warns on stderr and writes the link anyway.
8. **`cN.text`** (C, pending investigation): deprecate it; any edit may change commented text. As Arthur first put it: the anchor moves when the range's ends survive; the comment is dropped, and reported on stderr, when they do not. The fixtures below refine this. The Word evidence is in `jubarte-loop/cli-std/comment-scenarios/` (Arthur's fixtures); the rule is settled from them before any code.

   **What Word did with Arthur's fixtures** (word_pdfs/ rendered by Word; report `comment-scenarios/empryo_comments.md`). This corrects the "drop" half of 8C:
   - *Tracked* deletion never removes a comment. The start, the end, both ends, or all of the covered text: `commentRangeStart`, `commentRangeEnd` and the reference run stay outside every `w:del`. A deletion that crosses a marker is split there (`deletedendcomment.docx`: one `w:del` before `commentRangeEnd`, one after the reference run). Deleting everything leaves `commentRangeStart w:del… commentRangeEnd`, and Word still draws both balloons.
   - *Untracked* deletion of all the covered text (`deletedallcomment(2).docx`) removes the markers, the comments and the whole comment-part family.
   - The markers never move. A tracked partial deletion leaves `commentRangeStart` and `commentRangeEnd` where they were, with the deleted text still between them (`deletedstartcomment.docx`: `commentRangeStart`, `w:del("<start>lorem i")`, `"psum …"`, `commentRangeEnd`). The range shrinks only when the deletion is accepted; rejecting restores it.
   - Not yet evidenced: an *untracked* partial deletion (expected: markers stay, the range covers less), and insertions typed at either edge (inside or outside the range?). Both need a fixture from Arthur.
   - So jubarte keeps the comment on every tracked edit, with markers outside the `w:del`. It drops the comment, and reports that on stderr, only when an untracked edit empties the range. The Task 23 comparer bug (`w:del` escaping the range on a whole-range replace, `src/comparer/comments.rs` `map_range`) is the one tracked shape that is wrong today.

## Task 0 — Crate survey for item 1 (decision, no code)

**Decision: write a small hand parser; depend on nothing.**

- `pulldown-cmark 0.13` (`Cargo.toml:116`) already emits `Event::InlineMath`/`DisplayMath`, gated by `Options::ENABLE_MATH`. The writer never enables it (`src/markdown/write.rs:262-269`) and dumps math as text (`src/markdown/write.rs:514`).
- Surveyed on crates.io: `latex2mathml` / `latex2mmlc` (pure-Rust LaTeX→**MathML**, e.g. [latex2mathml](https://crates.io/crates/latex2mathml)). Wrong target: we need LaTeX→**OMML**, so it buys a parser but still needs a full MathML→OMML second stage plus version/supply-chain drift. No Rust crate writing OMML (`m:oMath`, `m:f`, `m:rad`, `m:nary`…) was found; `pulldown-latex`/`katex-rs` have no such crate pages (search returned only bindings/docs tooling). Haskell `texmath` writes OMML but is not a Rust dependency option.
- The contract is finite and already enumerated: the reader's OMML→LaTeX table (`src/markdown/from_docx/mod.rs:2291-2470`: `f` at :2321, scripts :2322-2335, `rad` :2336-2343, `d` :2344-2363, `func` :2364-2368, `nary` :2369-2395, `acc` :2396-2406, `bar` :2407-2413, `limLow/limUpp` :2415-2416, `m` :2417-2428, `eqArr` :2429-2435, `MATH_FUNCTIONS` :2453-2457, `group`/`base` :2444-2469). Every LaTeX the reader prints must parse back to the OMML it came from — a hand parser closed over exactly this table is smaller and round-trip-exact where a general crate would drift (brace normalization, `sSubSup` vs sequential scripts, `m:nary` limits).

## Task 1 — github-slugger fixes (item 2 scope only)

**Goal.** Duplicate headings follow github-slugger retry semantics; combining marks stay; only U+0020 becomes `-` (NBSP dropped); empty headings get no bookmark.

- Markdown in: `# Intro\n\n# Intro\n\n# Intro 1\n` + `[a](#intro) [b](#intro-1) [c](#intro-1-1)` (exact shape in `tests/markdown_to_docx.rs:477-505`).
- XML out: bookmarks `intro`, `intro-1`, `intro-1-1` (`<w:bookmarkStart w:id="0" w:name="intro"/>` …), links rewritten to match (`<w:hyperlink w:anchor="intro-1-1"` …). Combining-mark heading `e\u{301}cole math` → `e\u{301}cole-math`; `Foo\u{a0}Bar` → `foobar--emdash` shape; `#` (empty) → no bookmark.
- Reader/view out: unchanged (writer-only fix).

**Failing tests first.**

- `tests/markdown_to_docx.rs::heading_slugs_follow_github_slugger` — asserts exactly the retry/combining/nbsp/empty behavior; current slug filter (`src/markdown/write.rs:1828-1837`: `is_alphanumeric` drops combining marks, `is_whitespace` turns NBSP into `-`) and repeat counter (`src/markdown/write.rs:1838-1843`: per-slug counter, never retries a suffixed name against `used`/cut collisions) do not implement it.
- Add `tests/markdown_to_docx.rs::duplicate_slug_retries_against_cut_names` (e.g. `Intro`, `Intro`, `Intro 1` → `intro`, `intro-1`, `intro-1-1` per spec §8.2), asserting `bookmark_name` collision path (`src/markdown/write.rs:1773-1797`).

**Change points.**

- `src/markdown/write.rs:1828-1837` (slug char map: keep combining marks, drop NBSP, only U+0020 → `-`).
- `src/markdown/write.rs:1838-1848` (duplicate handling: github-slugger retry against all names given out, not per-slug counter).
- `src/markdown/write.rs:1760-1797` (`MOST_BOOKMARK`, `word_takes`, `bookmark_name` cut/`-N` interaction with retried names).
- `src/markdown/write.rs:1702-1732` (rename propagation to `Link::Anchor`; empty slug → no bookmark, consistent with `word_takes`).

**Word checks.** `tools/validate-docx` (validator CLI, `tools/validate-docx/Program.cs`) on the written file; Word-validity + no repair prompt via `neurotic_docx_bench/scripts/word_pdf.py`; visual: links jump to the right heading in the exported PDF (no dedicated Word PDF; self-made docx→PDF).

**Gate/fixtures.** None expected: `tests/md_fixed_point.rs` has no slug fixtures; no committed views change. If a fixture gains a bookmark, regenerate only that view with `jubarte read SRC --no-page-markers`.

**Risks.** Cut-then-suffix collisions (`long` cut to 40 chars then `-1` at `src/markdown/write.rs:1785-1796`); digit-leading slugs (`_` prefix, `tests/markdown_to_docx.rs:507-521`); Word's 40-char cap and letter-start rule.

**Commit (1, ≤5 files).** `src/markdown/write.rs`, `tests/markdown_to_docx.rs`.

## Task 2 — Honor `track_changes` (item 5, track_changes only; columns already done)

**Goal.** The YAML key the reader prints controls `w:trackRevisions`; revisions alone never turn tracking on. A CLI flag overrides the key (decision 2).

- Markdown in: `---\ntrack_changes: on\n---\n\nText.\n` (and `off`).
- XML out (`word/settings.xml`): on → `<w:trackRevisions/>` present (same element the reader keys on, `src/markdown/from_docx/header.rs:299-310`); off → absent. With a reference doc, its settings kept (`src/markdown/package.rs:52-67` `KEPT`) then forced to the key.
- View out: `jubarte read` prints `track_changes: on|off` unchanged (`src/markdown/from_docx/header.rs:299-310`, `tracking_on` at `src/markdown/from_docx/header.rs:75-80`; tests at `tests/agent_text_view.rs:1550-1600`).

**Failing tests first.**

- `tests/markdown_to_docx.rs::track_changes_key_writes_track_revisions` — `track_changes: on` → `settings.xml` contains `<w:trackRevisions/>`; `off` → absent; revisions present + key absent → absent (no auto-on).
- `tests/markdown_to_docx.rs::track_changes_key_overrides_reference_settings` — reference with/without the element gets forced to the key.

**Change points.**

- `src/markdown/write.rs:1575-1618` (`metadata()`: parse `track_changes:` today ignored).
- `src/markdown/xml.rs:206-260` (`Document`: new `tracking: Option<bool>` field beside `title`/`author` at :210-211).
- `src/markdown/package.rs:775-781` (default-template `settings.xml`: insert `<w:trackRevisions/>` when on).
- `src/markdown/package.rs:120-200` (`assemble()`: apply the key to a reference's kept settings; cf. edit path `src/settings.rs:185,258,380` for the canonical element position).
- `src/markdown/mod.rs:144-180` (`DocxOptions` docs if the key needs surfacing; writer behavior stays YAML-driven).

**Word checks.** `tools/validate-docx`; `word_pdf.py` round trip (Word must not repair; new edits track only when on). Visual: make one edit in Word in each file, confirm tracking behavior matches the key.

**Gate/fixtures.** `tests/md_fixed_point.rs` fixtures with `track_changes: on` (e.g. `potpourri.md`) keep passing; `docx_verdicts` (`tests/md_fixed_point.rs:118-140`, `debug --check render`) must show no new `settings differ`. Committed fixtures unchanged unless one names the key; regenerate via `jubarte read SRC --no-page-markers` if so.

**Risks.** Wrong element name (`w:trackChanges` does not exist — spec §8.5); element ordering inside `w:settings` (follow `src/settings.rs:380-385`); reference docs whose settings lack the element.

**Commit (1).** `src/markdown/write.rs`, `src/markdown/xml.rs`, `src/markdown/package.rs`, `tests/markdown_to_docx.rs`.

## Task 3 — Inline HTML breadth (item 4)

**Goal.** `<b>/<strong>`, `<i>/<em>`, `<s>/<del>`, `<code>`, `<mark>` work; anything still dropped warns.

- Markdown in: `<b>b</b> <i>i</i> <s>s</s> <code>c</code> <mark>m</mark> <span>kept-as-text?</span>`.
- XML out: `<w:b/><w:bCs/>`, `<w:i/><w:iCs/>`, `<w:strike/>` (`src/markdown/xml.rs:640-648`), `VerbatimChar` (`src/markdown/xml.rs:631-636`), `<w:highlight w:val="yellow"/>` (`src/markdown/xml.rs:649-651`); dropped tags → entry in `document.warnings` (`src/markdown/xml.rs:212`, surfaced via `src/markdown/write.rs:39-45`), never silent.
- View out: unchanged shapes (`**b**`, `*i*`, `~~s~~`, `` `c` ``, `{==m==}`); warn path is writer-only.

**Failing tests first.**

- `tests/markdown_to_docx.rs::inline_html_bold_italic_strike_code_mark` — each new tag asserts its `w:rPr` element in `document.xml`.
- `tests/markdown_to_docx.rs::dropped_html_warns` — e.g. `<span>`/`<div>` produce warning text naming the tag and write no formatting.
- Reader-side (spec §5.2 “warn when HTML is dropped” is writer warnings; no reader change): no new `agent_text_view` test unless underline handling moves (it stays agent-only, `src/markdown/from_docx/mod.rs:1871-1881`).

**Change points.**

- `src/markdown/write.rs:1520-1547` (`inline_html()`: add arms on existing counters `bold`/`italic`/`strike`/`code` at :664-668/:764-766, highlight group at :415-418; replace silent `_ => {}` at :1545 with a warning push).
- `src/markdown/write.rs:1149-1160` (`format()`) and `src/markdown/write.rs:1162-1182` (`push()`, `VerbatimChar` style insert at :1169-1171) — verify `<code>` inside HTML sets `code` counter.
- `src/markdown/xml.rs:628-665` (`run_properties()`: strike/highlight/code emission; **not** mdast2docx's `emphasisMark` — spec §8 “Not adopted”).
- `src/markdown/from_docx/ooxml.rs:477-547` (`Span`/`push_styled` merge keys on b/i/u only at :531-536 — read back unchanged; no change unless warn parity needs it).

**Word checks.** `tools/validate-docx`; `word_pdf.py`; visual: bold/italic/strike/code/highlight runs render distinctly in the PDF. No reference Word PDF for this one.

**Gate/fixtures.** `kitchen-sink.md`/`potpourri.md` may gain warnings only (warnings are not in the compared view); `tests/md_fixed_point.rs` comparable view (`comparable()` strips only `source:`/`sha256:`) must stay byte-equal. No fixture regen expected.

**Risks.** `<mark>` must be highlight-only (Word writes `w:highlight` alone); case/whitespace variants (`<BR/>`, `< b >` — current normalizer at :1521-1526 lowercases/strips whitespace); nested tags (`<b><i>` counters already nest); block-level HTML stays view-plumbing (`src/markdown/write.rs:781-785`).

**Commit (1).** `src/markdown/write.rs`, `src/markdown/xml.rs`, `tests/markdown_to_docx.rs`.

## Task 4 — Small wins (item 6, split in two commits)

**Goal.** Six remark-docx gaps, each tiny, each Word-exact.

1. `wp:docPr title` for a picture's title — Markdown in `![alt](f.png "title")`; XML out `<wp:docPr … descr="alt" title="title"/>` (today `descr` twice, no `title`: `src/markdown/xml.rs:704-724`).
2. Linked `FootnoteTextChar` — character style linked to `FootnoteText` (today `FootnoteText` at `src/markdown/xml.rs:1001-1009` has no link; cf. `FootnoteReference` char style nearby).
3. Code-in-link keeps code face — `` [`c`](url) `` keeps `VerbatimChar` and `Hyperlink` (today `link` masks `code`: `src/markdown/xml.rs:631-636`).
4. Fuller core properties from front matter — beyond `title:`/`author:` (`src/markdown/write.rs:1606-1608`, `src/markdown/package.rs:634-670`); scope: `description`, `keywords`, `category` (front-matter keys to agree; Word shows them in File → Info).
5. `fontTable` entries for legend fonts — default template (`src/markdown/package.rs:735-808`) has no `fontTable`; legend fonts (`src/markdown/view_styles.rs:153-193`, applied at `src/markdown/package.rs:122-130`) must have `w:fonts` entries or Word substitutes.
6. Fixture shapes as new tests (`tests/markdown_to_docx.rs` + `tests/fixtures/md-roundtrip/`): headings in lists/quotes, notes inside links/bold/quotes, reference links, fence info strings, nested tasks, HTML zoo (exact list in `crush_capabilities.md:195-211`).

**Failing tests first.**

- `tests/markdown_to_docx.rs::picture_title_writes_docpr_title` (descr vs title split; `read_picture` at `src/markdown/write.rs:2151-2172`, cap at :276-278/:2170-2172).
- `tests/markdown_to_docx.rs::code_in_link_keeps_code_face` (assert both `Hyperlink` and `VerbatimChar` markers).
- `tests/markdown_to_docx.rs::footnote_text_char_is_linked` (styles part).
- `tests/markdown_to_docx.rs::front_matter_writes_full_core_properties`.
- `tests/markdown_to_docx.rs::legend_fonts_have_font_table_entries`.
- New fixtures: `tests/fixtures/md-roundtrip/remark-shapes.md` (+ entries if docx-backed: `docx.txt`/`views.txt`).

**Change points.**

- `src/markdown/write.rs:682-686` (`Tag::Image` keeps `(url, alt)` only — extend with title) + `src/markdown/write.rs:1549-1572` (`picture()`) + `src/markdown/write.rs:2151-2172` (`read_picture`) + `src/markdown/xml.rs:158-166` (`Picture`) + `src/markdown/xml.rs:704-724` (`drawing_xml`) + `src/markdown/package.rs:621-633` (`max_drawing_id` untouched, cited as adjacent).
- `src/markdown/xml.rs:631-636` (code-in-link) + style defs `src/markdown/xml.rs:904-1040` + `src/markdown/write.rs:1169-1171`.
- `src/markdown/package.rs:634-670` + `src/markdown/package.rs:782-789` (core template) + `src/markdown/write.rs:1575-1618` (front-matter keys).
- `src/markdown/package.rs:735-808` (fontTable part + content-type) + legend source `src/markdown/view_styles.rs:153-193`.
- Tests/fixtures only for the shapes.

**Word checks.** Per sub-item: `tools/validate-docx` each; `word_pdf.py` each; visuals: picture title in Format Picture → Alt Text; footnote text in its style; code-in-link renders mono+blue-underline; File → Info shows props; legend-font doc shows no substitution in PDF.

**Gate/fixtures.** Core-props/fontTable changes can shift `docx_verdicts` render diff (fonts!) — run `tests/md_fixed_point.rs` after each sub-item; regenerate affected committed views with `jubarte read SRC --no-page-markers` and record in the commit.

**Risks.** `title` vs `descr` split must not break alt-text readers (`from_docx` reads `descr||title`, `mod.rs:2016-2044` — cite, do not change); linked char styles require matching `w:link` both ways; core-prop keys must not collide with view YAML keys (`view:`, `page:`, `styles:` at `src/markdown/write.rs:1609-1614`).

**Commits (2).** 4a: `src/markdown/xml.rs`, `src/markdown/write.rs`, `tests/markdown_to_docx.rs` (docPr title, code-in-link, FootnoteTextChar). 4b: `src/markdown/package.rs`, `src/markdown/write.rs`, `src/markdown/view_styles.rs`, `tests/fixtures/md-roundtrip/remark-shapes.md`, `tests/markdown_to_docx.rs` (core props, fontTable, fixture shapes). Keep each ≤5 files (split further if needed).

## Task 5 — Checklist as Word's own list (item 3)

**Goal.** `- [ ]`/`- [x]` become Word's checklist shape from Arthur's sample, not `☒` text or `w14:checkbox` SDTs.

- Markdown in:
  ```markdown
  - [x] Travel in the morning.
  - [ ] Wash my towels.
    - [ ] Nested level
  - Plain bullet
  ```
- XML out (mimic `tests/fixtures/agent-view/word-checklist.docx` exactly): a dedicated bullet `w:abstractNum` whose nine levels are all `w:lvlText` U+F0A8 (Wingdings; sample `numbering.xml`: `abstractNumId="3"`, `nsid 541df9e6`, all nine `w:lvl` U+F0A8, `w:ind left 720·(level+1), hanging 360`), `ListParagraph` items (`document.xml` sample: `w:numId 3`, `ilvl 0`); checked item: `<w:strike/>` on the paragraph mark's `w:rPr` (`document.xml` p0/p3: `w:pPr/w:rPr/w:strike w:val="1"`) **and** on every run — the box takes the mark's formatting so it is struck too. No `w:sdt`, no ☐/☑ text.
- View out: `jubarte read --no-page-markers` prints `- [x] Travel in the morning.` / `- [ ] Wash my towels.`; a checked item's strikes print no `~~` (reader suppresses mark+run strike on checklist items).

**Failing tests first.**

- `tests/markdown_to_docx.rs::task_items_use_word_checklist_numbering` — abstractNum has 9× U+F0A8 Wingdings levels, indents `720·(level+1)`/360; items are `ListParagraph` on a numId distinct from the plain-bullet one.
- `tests/markdown_to_docx.rs::checked_items_strike_mark_and_runs` — checked: mark `w:rPr` strike + every run strike; unchecked: neither.
- `tests/agent_text_view.rs::word_checklist_reads_as_tasks` (or `tests/md_reader_id_lines.rs::…`): read `tests/fixtures/agent-view/word-checklist.docx` → `- [x]`/`- [ ]` lines; today it reads as plain bullets dropping the strike (`w:val="1"`).
- `tests/md_reader_id_lines.rs::checked_item_strikes_do_not_print_tildes` — checked item round-trips with no `~~`.

**Change points.**

- Writer: `src/markdown/write.rs:539-546` (replace ☒/☐ text + `prefix_runs` bookkeeping with a checked flag on the paragraph/item); `src/markdown/write.rs:570-602` (`Tag::List`/`Tag::Item` instance/level plumbing — checklist needs its own instance); `src/markdown/write.rs:922-937` (paragraph style `ListParagraph` + `numbering` assignment); `src/markdown/xml.rs:148-157` (`List`: checked/task variant or parallel list kind); `src/markdown/xml.rs:797-863` (`abstract_num` Wingdings-U+F0A8 arm + `numbering()` wiring); `src/markdown/package.rs:473-532` (append the new abstractNum/num beside existing bullets).
- Reader: `src/markdown/from_docx/mod.rs:1680-1745` (`list_marker()`: detect the U+F0A8 abstractNum → `- [ ]`/`- [x]` marker instead of `"-"` at :1688); run/mark strike read — mark `w:rPr` (`agent.rs` id-line facts cited via `src/markdown/from_docx/agent.rs:621,801-832`) + run `w:strike` → task-checked, suppressing `~~`; `src/markdown/from_docx/ooxml.rs:477-547` (`Span`: strike is not a merge key today — checklist path must not leak into `~~` rendering at :584-640).
- Today's behavior pins: writer ☒ at `src/markdown/write.rs:539-546` + flat-task test at `tests/markdown_to_docx.rs:74-101`; reader bullet arm at `src/markdown/from_docx/mod.rs:1686-1689`.

**Word checks.** `tools/validate-docx`; `word_pdf.py` with no repair prompt; **the** visual: export our checklist docx to PDF and place beside Arthur's `jubarte-loop/cli-std/remarkdocx/Document 12(1).pdf` — same box glyph, same indent ladder, checked items struck including the box.

**Gate/fixtures.** High impact: every `- [ ]` fixture changes shape. `tests/md_fixed_point.rs` will go red on task fixtures until the reader half lands — land writer+reader in one PR, then regenerate affected committed views (`jubarte read SRC --no-page-markers`) and record `docx_verdicts` deltas (numbering part gains an abstractNum).

**Risks (Word-refuses / must-handle).**

- `abstractNum nsid`: Word tolerates missing `nsid` but two abstractNums sharing ids collide — allocate past existing ids (`src/markdown/package.rs:508-532` pattern).
- Numbering ids beside existing bullet lists: checklist needs its **own** abstractNum+num (never reuse the plain-bullet one), or plain bullets become boxes.
- Nested task levels: all nine levels U+F0A8 with `left 720·(level+1)`; nested `- [ ]` must set `ilvl` (writer `level` at `src/markdown/write.rs:572,591-595`; reader `ilvl.min(8)` at `src/markdown/from_docx/mod.rs:1414-1416`).
- Task mixed with plain bullets: same list block, two numIds — writer must split instances per item kind; reader must not merge them into one marker kind.
- Strike scope: mark-only strike leaves runs unstruck (box unstruck); run-only strike leaves the box unstruck — both required, per sample.

**Commits (3, in order).** 5a writer numbering: `src/markdown/xml.rs`, `src/markdown/package.rs`, `tests/markdown_to_docx.rs`. 5b reader: `src/markdown/from_docx/mod.rs`, `src/markdown/from_docx/ooxml.rs`, `tests/agent_text_view.rs` (+ `tests/md_reader_id_lines.rs` if the tilde-suppression test lives there — still ≤5). 5c fixtures/gate: regenerated views + `tests/fixtures/md-roundtrip/*.md`, `tests/md_fixed_point.rs` only if the harness needs task-aware comparison (avoid touching the harness if possible).

## Task 6 — LaTeX→OMML writer (item 1)

**Goal.** `$…$`/`$$…$$` become real equations; `math.md` turns green and round-trips.

- Markdown in: `` $x^2$ `` and `$$\\frac{−b±\\sqrt{b^2−4ac}}{2a}$$` (exact `tests/fixtures/md-roundtrip/math.md:20` display line).
- XML out: inline → `<m:oMath>…</m:oMath>` in the run flow; display → `<m:oMathPara>` wrapping one or more `<m:oMath>` (both TS libraries get this wrong per `CONSOLIDATED.md:42-43,54-57`; the reader proves the expected shape at `src/markdown/from_docx/mod.rs:1824-1839`). Structures: `m:f`/`m:num`/`m:den`, `m:rad` (+ empty vs filled `m:deg`), `m:sSup`/`m:e`/`m:sup` (+ `sSub`, combined `sSubSup`, `sPre`), literal `m:r`/`m:t` runs, `±` passthrough — the inverse of `math_inner` arms (`src/markdown/from_docx/mod.rs:2321-2343`).
- View out: `jubarte read --no-page-markers` prints `$x^2$` / `$${…}$$` back (reader already does: `src/markdown/from_docx/mod.rs:1830-1839`, `math()` at :2292-2294, tests at :2675+).

**Failing tests first.**

- `tests/markdown_to_docx.rs::inline_math_writes_omath` — `` $x^2$ `` → `<m:oMath>` with `m:sSup`, no literal `$` runs.
- `tests/markdown_to_docx.rs::display_math_writes_omathpara` — `$$…$$` → `<m:oMathPara>` (centered display paragraph), multi-`\\\\` → one para with N `m:oMath` (reader joins with `" \\\\ "`, :1831).
- `tests/markdown_to_docx.rs::quadratic_formula_round_trips` — full walk from the math report (`muse_m2d_math.md` §1c): frac + `±` + rad + `sSup` radicand.
- Reader-table coverage test per arm (`f`, `sSup`/`sSub`/`sSubSup`/`sPre`, `rad`±degree, `d`, `func`, `nary`, `acc`, `bar`, `limLow`/`limUpp`, `m`, `eqArr`, functions): each LaTeX the reader prints parses back to its OMML.

**Change points.**

- `src/markdown/write.rs:262-269` (enable `ENABLE_MATH` in `parser_options()`).
- `src/markdown/write.rs:514` (route `InlineMath`/`DisplayMath` to the new parser/emitter instead of `self.text()`).
- `src/markdown/xml.rs:35-57` (`Content`: new math arm; `Run`/`RunFormat` interaction — math runs carry no b/i/u).
- `src/markdown/xml.rs:704-724` (new OMML serializer beside `drawing_xml()`; display `m:oMathPara` + justification, inline `m:oMath`).
- New small parser module (e.g. `src/markdown/math.rs`, wired through `src/markdown/mod.rs`): grammar closed over the reader table (`src/markdown/from_docx/mod.rs:2296-2442`, `MATH_FUNCTIONS` :2454-2457, `group`/`base` :2444-2469); unknown commands → literal `m:t` text (never drop), keeping the gate green.
- `src/markdown/package.rs:735-808` only if the math namespace needs declaring on the default template (check validator output first).

**Word checks.** `tools/validate-docx` on every new OMML shape (namespace errors are the classic failure); `word_pdf.py` with no repair prompt; visual: our `math.md` PDF shows a real centered display equation matching Word's equation-editor rendering (quadratic formula), inline math sits on the text baseline.

**Gate/fixtures.** `tests/md_fixed_point.rs`: `math.md` (currently text-round-trip) becomes equation-round-trip — update the fixture view if the reader normalizes braces (`group()` leaves single chars unbraced, :2444-2451); `potpourri.md` docx-side `debug --check render` must show no new losses. No `docx.txt` additions unless a docx equation fixture is wanted (views stay generated via `jubarte read SRC --no-page-markers`).

**Risks (shapes Word refuses).**

- `m:` elements outside `m:oMath`/`m:oMathPara`, or display math as bare `m:oMath` in `w:p` (the TS-library bug) — Word may show text or repair.
- `m:ctrlPr` required in some contexts (Word emits it; missing is usually tolerated but validator-flagged — obey `tools/validate-docx`).
- Run properties inside `m:r`: only `m:rPr` belongs there — a `w:rPr` (or `w:strike`/`w:highlight` leaking from Task 5 state) invalidates the run.
- `m:oMathPara` justification: display centering is the para's, not the math's — set it where Word does or the equation left-aligns.
- Parser asymmetry: reader covers `d`/`func`/`nary`/`acc`/`bar`/`limLow`/`limUpp`/`m`/`eqArr` (all absent from mdast2docx's writer) — copying mdast2docx verbatim caps fidelity; the emitter must cover the whole reader table or the round trip silently narrows.

**Commits (4, in order).** 6a parser: new `src/markdown/math.rs`, `src/markdown/mod.rs`, unit tests in-file. 6b emitter+plumbing: `src/markdown/xml.rs`, `src/markdown/write.rs`, `tests/markdown_to_docx.rs`. 6c display wrappers + `ENABLE_MATH`: `src/markdown/write.rs`, `src/markdown/xml.rs`, `tests/markdown_to_docx.rs`. 6d fixtures/gate: `tests/fixtures/md-roundtrip/math.md`, regenerated views, `docs/DIALECT.md:20` (`$x$` “text today” line) + `docs/MARKDOWN.md:29-65` table.

## Validation plan (every task)

1. `cargo nextest run <affected-test>` red first, then green; full `cargo nextest run tests/md_fixed_point.rs tests/markdown_to_docx.rs tests/md_reader_id_lines.rs tests/agent_text_view.rs` before each PR.
2. `cargo clippy -- -D warnings` (no `#[allow]` anywhere).
3. `tools/validate-docx` on every written docx (Office2019 validator, exit 1 on any finding).
4. `neurotic_docx_bench/scripts/word_pdf.py` on every Word-visible change (no repair prompt = Word-valid); checklist PDF beside Arthur's `Document 12(1).pdf`.
5. `tests/md_fixed_point.rs` strict gate + `debug A B --check render` docx-side verdicts; regenerate committed views only with `jubarte read SRC --no-page-markers`.

## Risks / open questions

- Checklist + math both touch `numbering`/paragraph paths; land checklist before math so math PDFs diff against a stable list numbering.
- `fontTable` addition (Task 4b) can shift `docx_verdicts` font lines across many fixtures — run the gate per sub-item, not at the end.
- No open spec questions: §8 settles shapes (checklist sample, no-SDT, no `emphasisMark`, `w:trackRevisions` name, github-slugger retry).


