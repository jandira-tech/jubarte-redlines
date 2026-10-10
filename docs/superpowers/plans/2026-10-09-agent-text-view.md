# Agent Text View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `jubarte read FILE` (alias `text`, and plain `jubarte FILE`) print the agent view: a YAML header, then a CriticMarkup body in which every paragraph, table, revision and comment carries the id that `edit`, `accept`, `reject` and the comment operations already take, with selection flags (`-p`, `--head`, `--tail`) for long documents.

**Architecture:** The view is the existing DOCX to Markdown converter (`src/markdown/from_docx`) behind one new option, `ids`. With `ids` on, paragraphs are numbered before accept/reject resolution so indices match `inspect` and `edit`; an `<!-- pN … -->` line precedes each block; every tracked change is followed by a CriticMarkup attribution note carrying Word's `w:id` and the author's handle (`{++quarterly++}{>>#0 @AC<<}`), the position the spec and `write.rs` already reserve for authorship; comments carry their `w:id`, handle and thread parent; page markers come from the same layout pass `convert -t md` uses; a header module renders document facts as YAML. With `ids` off, output stays byte-identical to today, and the existing suite is the regression test for that.

**Tech Stack:** Rust; the crate's own `ooxml::Element` tree, `critic` renderer, `Blocks` joiner and `markdown::paginate`; `convert::render` for page texts; clap CLI; integration tests in `tests/` with `tests/common/docx.rs` builders and the `CARGO_BIN_EXE_jubarte` binary.

---

## Scope

In: tracked, no-comments, accept-all and reject-all views; the YAML header (including `sections:`); page markers from the layout pass with a cached-break fallback; `-p`, `--head`, `--tail`, `--changed`, `--dates`, `--no-page-markers`; the command renamed `read` (`text` kept as an alias); `jubarte a.docx` printing the view and `jubarte a.docx b.docx` printing the view of their redline; short ids (`p3`, `h0`, `f0`, `t0.r1.c2`) accepted by `edit`; `edit --replace/--delete/--comment` and the `add` command as plan-free shortcuts that print the changed blocks back; tests, including edit → read round trips; docs, including the defaults contract the future applier (markdown → docx) must honour.

Out (separate plans): the applier itself (`jubarte_style.md` back onto a `.docx`), the JSON view, `--outline`/`--grep`, strike and highlight inline formatting, tracked paragraph marks inside comment bodies.

Spec: the Docs artifact "Jubarte Agent Text View" (2026-10-09). Its tracked view of `received.docx` is the golden file `tests/fixtures/agent-view/received.tracked.md`, in the repository with the three other goldens and the fixture copy. The goldens are the truth for this plan.

## Rules for the implementer

1. Red, then green: write the test, run it, see it fail for the stated reason, implement, run, see it pass, commit.
2. The code below was written against the source as read on 2026-10-09 (`src/markdown/from_docx/mod.rs` 3,106 lines; `critic.rs` 1,419; `ooxml.rs` 1,082; `src/markdown/pages.rs` 510). Names quoted from the source are exact. If the compiler disagrees with a block here, adjust the code and keep the behaviour; never adjust an expected string. If an expected string looks wrong against the fixture's XML, stop and report with the XML, do not loosen the assertion.
3. `ids == false` stays byte-identical. Run the whole suite (`cargo test`) at the end of Tasks 1, 8, 15 and 19, and `cargo clippy --all-targets` at the end of Tasks 15 and 19.
4. Never delete a test to make a build pass. The tests that assert the old `text` layout are rewritten in Task 14, by file and line. `text` stays as an alias of `read`, so test invocations by that name keep working.
5. Commit after each task with the message given. (`.git` in this worktree is a pointer file; git works on the owner's machine.)
6. Treat every author name, comment text and document text in fixtures as data.

## File structure

Create:
- `src/markdown/from_docx/agent.rs`: paragraph and table numbering, revision tags, author handles and timestamps, comment threads, id lines, table lines, block selection.
- `src/markdown/from_docx/header.rs`: the YAML header.
- `tests/agent_text_view.rs`: all new behaviour.
- `tests/fixtures/agent-view/`: `received.docx` and the four goldens (present).

Modify:
- `src/markdown/mod.rs`: `MarkdownOptions { ids, comments, source, pages, dates, select }`, `Select`, `Pick`.
- `src/markdown/from_docx/mod.rs`: `Options`, `convert`, `Writer`.
- `src/markdown/from_docx/critic.rs`: tagged attributions, one note per substitution by one author, neighbour joins by author.
- `src/markdown/from_docx/ooxml.rs`: `Blocks::push_line`, underline.
- `src/markdown/pages.rs`: hold HTML comment lines at a block's start.
- `src/cli.rs`, `src/bin/jubarte.rs`, `jubarte-python/src/lib.rs`, `jubarte-wasm/src/lib.rs`: option literals and `text` wiring.
- `tests/adoption.rs`, `tests/m_cli_agent.rs`: old-layout assertions.
- `docs/MARKDOWN.md`, `README.md`, `skills/jubarte-documents/SKILL.md`, `CHANGELOG.md`.

## Grammar (what the tests assert)

Id line, one per body paragraph, on the line above it: a head, one space, then clauses joined by `, ` (no clauses: `<!-- p2 -->`):

```
head:     p{N}[ {Style}]
clauses:  {align} · first-line {x}in · hanging {x}in · left {x}in · right {x}in · num "{label}" | bullet · page-break · section-break · break-ins {tags} · break-del {tags} · fmt {tags} · rev {tags} · comments #c{ids}
examples: <!-- p0 center -->   <!-- p18 first-line 0.5in, comments #c11 -->   <!-- p0 Quote justify, hanging 0.25in, left 0.5in -->   <!-- p3 rev #0 @AC; #1+2 @AC -->
```

- `N` is the paragraph's index among every `w:p` under `w:body` in document order, table cells included, text boxes (`w:txbxContent`) excluded: the same rule as `inspect::body_paragraph_nodes`, so `pN` is `body:p:N`.
- `Style` prints when it is not the default paragraph style and, for a heading, not `Heading{level}`.
- `align` prints only when the paragraph sets `w:jc` itself: `center`, `right` (`right`, `end`), `justify` (`both`, `distribute`), else `left`.
- Indents print from the paragraph's own `w:ind`, twips to inches with up to two decimals (`720` is `0.5in`, `1440` is `1in`).
- `num "1."` prints the auto-number label the converter computed; `bullet` for a bulleted item. The label is not document text.
- `page-break`: the paragraph holds `w:br w:type="page"`. `section-break`: its `w:pPr` holds `w:sectPr`.
- `break-ins`/`break-del`: a tracked paragraph mark (`w:pPr/w:rPr/w:ins|w:del`), as tags.
- `fmt`: `w:rPrChange`/`w:pPrChange` in the paragraph, as tags.
- `rev`: only in accept-all and reject-all views, the tags of the revisions the paragraph held before resolution.
- `comments`: only with `--comments none`, the comment ids whose ranges or references the paragraph holds.
- Empty paragraphs: `<!-- p19 empty -->`, consecutive ones `<!-- p19-p23 empty -->`.
- Page markers: `<!-- page N of M -->` followed by a blank line, before the first block that starts on page N, `<!-- page 1 of M -->` first. They come from the layout pass (`convert::render` page texts through `markdown::paginate`); without one, from `w:lastRenderedPageBreak`.

A **tag** is `#<ids> @<handle>`: `#0 @AC`, `#1+2 @AC` (two marks of one logical change), and in a tag list on an id line, `; ` between tags of different authors (`rev #1 @AC; #2 @JD`). The ids are `w:id` values, so `#1` in the body is `body:rev:1` for `accept --id`, `reject --id` and `resolve_revisions`. The handle is always present. Internally (attributes, `critic` attribution strings) a tag is `ids@HH`, several authors joined by `|` (`1@AC|2@JD`); `agent::format_tag` prints it.

Table line: head `t{T}[ center|right] {rows}x{cols}`, then clauses `cells p{a}-p{b} by row` (or `cells r0 p8-p10 r1 p11-p14 …` when a cell holds other than one paragraph), `header row repeats`, `merged cells`, `break-ins {tag} in p{N}`, `break-del {tag} in p{N}`, `rev {tag} in p{N}` (resolved views), several of the last three joined by `; `. `T` counts top-level body tables. The pipe table itself keeps the converter's shape (`|a|b|`, `|-|-|`).

Revision in the body: the change in plain CriticMarkup, then its attribution note:

```
{++quarterly++}{>>#0 @AC<<}
{--ninety days--}{>>#7 @AC<<}
{~~weekly~>monthly~~}{>>#1+2 @AC<<}          one author: one note, ids joined
{~~old~>new~~}{>>#1 @AC<<}{>>#2 @JD<<}        two authors: deleted side first, then inserted side
{++bold words++}{>>#7+8 @AC<<}                 neighbouring marks of one kind by one author
{++quarterly++}{>>#0 @AC 2026-10-03T14:05:00Z<<}   with --dates, for an author with several timestamps
```

Logical changes: neighbouring marks of one kind by one author join; then a deletion directly followed by an insertion, or the reverse, pairs as a substitution, greedily left to right, with bookmarks, proofing marks and comment markers ignored for adjacency. This is the order `critic::tidy` and `critic::render` apply, so `rev` clauses and notes agree. The text inside the delimiters is exactly the document's text: no tag, no moved space.

Comment: `{==anchored text==}{>>#c5 @AC: text<<}`; a reply is its own note naming the thread root, `{>>#c6 @AS re #c5: Disagree.<<}`; a resolved root reads `{>>#c5 @AC resolved: text<<}`; an empty range gives `{>>#c11 @AC: text<<}` with no `{==…==}`. Head grammar shared with revisions: `#<ids>[ @<handle>][ <timestamp>][ re #c<root>][ resolved]`, then `: text` for comments only.

Handles: the comment's `w:initials` when that author wrote one, else the uppercase initials of the name's words; a collision appends a digit. Timestamps: the header's `authors:` line prints an author's timestamp in full when all of that author's marks and comments share one, else the day range (`2026-10-01..2026-10-03`); with `--dates`, notes of such an author carry their timestamp inline.

Header: see `tests/fixtures/agent-view/received.tracked.md`. A `key: value` line with a comment pads the key/value to 35 columns, then `# comment`; a key/value longer than 33 characters takes two spaces instead. `range:` follows `body:` when a selection is active; `sections:` follows `footers:` when the document has more than one section.

Short ids outside the body: `h{N}` is the part `header{N+1}.xml` (its first text paragraph; `h1.p2` its paragraph 2), `f{N}` is `footer{N+1}.xml`, `t{N}` the N-th top-level body table, `t0.r1.c2` its cell. Word numbers header and footer parts arbitrarily, so `h0` says nothing about the role; the `headers:` block does. `edit` accepts these ids (Task 16) beside the long `header2:p:0` form.

---

### Task 1: Fixture, goldens and the options plumbing

**Files:**
- Present: `tests/fixtures/agent-view/received.docx`, `received.tracked.md`, `received.no-comments.md`, `received.accept.md`, `received.reject.md`
- Modify: `src/markdown/mod.rs:199-208` (`MarkdownOptions`), `src/markdown/from_docx/mod.rs:37` (`Options`)
- Modify: `src/bin/jubarte.rs:752-757`, `jubarte-python/src/lib.rs:678-682`, `jubarte-wasm/src/lib.rs:271-275` and `:926-929` (struct literals)
- Create: `tests/agent_text_view.rs`

- [ ] **Step 1: Confirm the fixtures are in place**

Run: `ls tests/fixtures/agent-view/ && cmp tests/fixtures/agent-view/received.docx examples/cli/visualize/out/received.docx && echo same`
Expected: five files listed and `same`. If `received.docx` is missing: `cp examples/cli/visualize/out/received.docx tests/fixtures/agent-view/`.

- [ ] **Step 2: Write the failing test for the default options**

Create `tests/agent_text_view.rs`:

```rust
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The agent view of `jubarte text`: id lines, attribution notes, comment
//! ids, the YAML header and block selection. Goldens in
//! `tests/fixtures/agent-view/`.
mod common;
use common::docx::{docx, para, Part, W_NS};
use jubarte::markdown::{MarkdownOptions, Select, TrackChanges, docx_to_markdown};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/agent-view")
        .join(name)
}

fn golden(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).unwrap()
}

fn agent(docx: &[u8]) -> String {
    agent_with(docx, TrackChanges::All, true)
}

fn agent_with(docx: &[u8], track_changes: TrackChanges, comments: bool) -> String {
    agent_options(docx, MarkdownOptions { track_changes, comments, ..agent_defaults() })
}

fn agent_defaults() -> MarkdownOptions {
    MarkdownOptions {
        ids: true,
        comments: true,
        source: Some("sample.docx".into()),
        ..MarkdownOptions::default()
    }
}

fn agent_options(docx: &[u8], options: MarkdownOptions) -> String {
    docx_to_markdown(docx, &options).unwrap().markdown
}

/// The body of an agent view: what follows the YAML header.
fn body(markdown: &str) -> &str {
    markdown.splitn(3, "---\n").nth(2).unwrap_or(markdown)
}

/// The header's lines, without the `---` fences.
fn header_lines(markdown: &str) -> Vec<&str> {
    markdown.splitn(3, "---\n").nth(1).unwrap().lines().collect()
}

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run jubarte")
}

#[track_caller]
fn ok(args: &[&str], dir: &Path) -> String {
    let out = jubarte(args, dir);
    assert!(
        out.status.success(),
        "jubarte {args:?} exited {:?}\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8 stdout")
}

#[test]
fn options_default_to_the_plain_conversion() {
    let options = MarkdownOptions::default();
    assert!(!options.ids);
    assert!(options.comments);
    assert_eq!(options.source, None);
    assert_eq!(options.pages, None);
    assert!(options.page_markers);
    assert!(!options.dates);
    assert!(options.select.is_none());
    let bytes = docx(&para("Plain text."));
    let plain = docx_to_markdown(&bytes, &options).unwrap().markdown;
    assert_eq!(plain, "Plain text.\n");
}

#[test]
fn select_parses_single_paragraphs_ranges_lists_and_tables() {
    use jubarte::markdown::Pick;
    assert_eq!(
        Select::parse("p2, p5-p7,12,p17-,-p1,t0").unwrap(),
        Select::Picks(vec![
            Pick::Paragraphs { from: 2, to: Some(2) },
            Pick::Paragraphs { from: 5, to: Some(7) },
            Pick::Paragraphs { from: 12, to: Some(12) },
            Pick::Paragraphs { from: 17, to: None },
            Pick::Paragraphs { from: 0, to: Some(1) },
            Pick::Table(0),
        ])
    );
    assert_eq!(Select::parse("p7-p5").unwrap_err(), "p7-p5: the range runs backwards");
    assert_eq!(Select::parse("x3").unwrap_err(), "x3: expected pN, pN-pM or tN");
    assert_eq!(Select::parse(" , ").unwrap_err(), "no paragraphs selected");
}
```

- [ ] **Step 3: Run them to see them fail**

Run: `cargo test --test agent_text_view options_default select_parses`
Expected: compile error, `no field 'ids' on type MarkdownOptions`, `cannot find type Select`.

- [ ] **Step 4: Add the fields and the selection types**

In `src/markdown/mod.rs`, `MarkdownOptions` (around line 199). Keep the existing fields and doc comments; add seven fields and give the struct a hand-written `impl Default` (two of the new fields default to `true`, so a derive will not do): `track_changes: TrackChanges::All, extract_media: None, ids: false, comments: true, source: None, pages: None, page_markers: true, dates: false, select: None`. If a derived `Default` exists today, replace it with that impl.

```rust
    /// Agent view: a YAML header, an id line before every block, Word's
    /// ids on revisions and comments. Off, the plain conversion, unchanged.
    pub ids: bool,
    /// With `ids`: comments inline (`true`), or hidden with their ids on the
    /// id line of the paragraph that holds them.
    pub comments: bool,
    /// With `ids`: the name printed as `source:` in the header; `None`
    /// prints `(bytes)`.
    pub source: Option<String>,
    /// With `ids`: the text painted on each page by the layout pass
    /// (`convert::RenderReport::pages`), for `<!-- page N of M -->` lines.
    /// `None` falls back to Word's cached page breaks.
    pub pages: Option<Vec<String>>,
    /// With `ids` and no `pages`: write `<!-- page N of M -->` lines from
    /// Word's cached breaks (`true`, the default), or no page lines at all;
    /// the header's page count is unaffected.
    pub page_markers: bool,
    /// With `ids`: timestamps inline on the notes of an author whose marks
    /// and comments do not all share one timestamp.
    pub dates: bool,
    /// With `ids`: which blocks of the body to print.
    pub select: Option<Select>,
```

Add after `MarkdownOptions`:

```rust
/// Which blocks of the agent view to print (`-p`, `--head`, `--tail`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Select {
    /// The first `n` blocks (a table is one block).
    Head(usize),
    /// The last `n` blocks.
    Tail(usize),
    /// Paragraphs and tables by id, in document order.
    Picks(Vec<Pick>),
}

/// One item of a `-p` selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// `pN`, `pN-pM`, `pN-` (to the end, `to: None`), `-pM` (`from: 0`).
    Paragraphs { from: usize, to: Option<usize> },
    /// `tN`: the whole table.
    Table(usize),
}

impl Select {
    /// `p2, p5-p7, 12, p17-, -p1, t0`: comma-separated picks; a bare number
    /// is a paragraph.
    pub fn parse(spec: &str) -> Result<Self, String> {
        fn number(item: &str, text: &str) -> Result<usize, String> {
            let text = text.trim();
            text.strip_prefix('p')
                .unwrap_or(text)
                .parse()
                .map_err(|_| format!("{item}: expected pN, pN-pM or tN"))
        }
        let mut picks = Vec::new();
        for item in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(table) = item.strip_prefix('t') {
                let n = table
                    .parse()
                    .map_err(|_| format!("{item}: expected pN, pN-pM or tN"))?;
                picks.push(Pick::Table(n));
                continue;
            }
            let (from, to) = match item.split_once('-') {
                None => {
                    let n = number(item, item)?;
                    (n, Some(n))
                }
                Some((a, b)) => (
                    if a.trim().is_empty() { 0 } else { number(item, a)? },
                    if b.trim().is_empty() { None } else { Some(number(item, b)?) },
                ),
            };
            if to.is_some_and(|to| to < from) {
                return Err(format!("{item}: the range runs backwards"));
            }
            picks.push(Pick::Paragraphs { from, to });
        }
        if picks.is_empty() {
            return Err("no paragraphs selected".to_string());
        }
        Ok(Self::Picks(picks))
    }
}
```

In `src/markdown/from_docx/mod.rs`, `Options` (line 37, derives `Default`): add

```rust
    /// The agent view (see `agent.rs` and `header.rs`).
    pub(crate) ids: bool,
    /// With `ids`: comments inline, or hidden and listed on id lines.
    pub(crate) comments: bool,
    /// With `ids`: the `source:` name in the header.
    pub(crate) source: Option<String>,
    /// With `ids`: painted page texts for the page markers.
    pub(crate) pages: Option<Vec<String>>,
    /// With `ids` and no `pages`: cached-break page lines, or none.
    pub(crate) page_markers: bool,
    /// With `ids`: inline timestamps on notes.
    pub(crate) dates: bool,
    /// With `ids`: which blocks to print.
    pub(crate) select: Option<super::Select>,
```

`Options` derives `Default`, which makes `page_markers` false; that is fine, since `docx_to_markdown` always sets it and the in-module tests use the plain conversion. In `docx_to_markdown` (`src/markdown/mod.rs:233`), pass them through inside the `from_docx::Options { … }` literal: `ids: options.ids, comments: options.comments, source: options.source.clone(), pages: options.pages.clone(), page_markers: options.page_markers, dates: options.dates, select: options.select.clone()`.

Update every `MarkdownOptions { … }` literal that does not already use `..Default::default()`: `src/bin/jubarte.rs:754` (`run_text`), `jubarte-python/src/lib.rs:679`, `jubarte-wasm/src/lib.rs:272` and `:927`. Add `..MarkdownOptions::default()` as the last member of each (full path if the type is not imported there).

- [ ] **Step 5: Run the tests and the whole suite**

Run: `cargo test --test agent_text_view options_default select_parses`
Expected: PASS.
Run: `cargo test`
Expected: everything passes; no output changed because `ids` is off everywhere.

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/agent-view tests/agent_text_view.rs src/markdown/mod.rs src/markdown/from_docx/mod.rs src/bin/jubarte.rs jubarte-python/src/lib.rs jubarte-wasm/src/lib.rs
git commit -m "feat(markdown): agent view options and Select, off by default"
```

---

### Task 2: Numbering, tags, handles and timestamps (the stamp)

**Files:**
- Create: `src/markdown/from_docx/agent.rs`
- Modify: `src/markdown/from_docx/mod.rs:16-19` (module list)

- [ ] **Step 1: Write the failing unit tests**

Create `src/markdown/from_docx/agent.rs` with the tests first (the functions come in Step 3):

```rust
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The agent view: numbering that matches `inspect` and `edit`, revision
//! tags, author handles and timestamps, comment threads, id lines, table
//! lines and block selection.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::ooxml::{Element, Node};

/// Attribute stamped on every numbered `w:p`: its `body:p:N` index.
pub(crate) const INDEX: &str = "jubarteIndex";
/// Attribute stamped on every top-level `w:tbl`: its `t{N}` number.
pub(crate) const TABLE: &str = "jubarteTable";
/// Attribute stamped on a `w:p` that holds revisions: `kind:tag` entries
/// separated by spaces (`ins:0@AC sub:1+2@AC`), recorded before resolution.
pub(crate) const REVS: &str = "jubarteRevs";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::from_docx::ooxml::parse_xml;

    const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;

    fn body(inner: &str) -> Element {
        parse_xml(format!("<w:body {W}>{inner}</w:body>").as_bytes()).unwrap()
    }

    fn doc(inner: &str) -> Element {
        parse_xml(format!("<w:document {W}><w:body>{inner}</w:body></w:document>").as_bytes()).unwrap()
    }

    fn indices(e: &Element, out: &mut Vec<String>) {
        if e.is("p") {
            out.push(e.attr(INDEX).unwrap_or("-").to_string());
        }
        for child in e.elements() {
            indices(child, out);
        }
    }

    #[test]
    fn stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded() {
        let mut body = body(
            r#"<w:p/><w:tbl><w:tr><w:tc><w:p/><w:p/></w:tc></w:tr></w:tbl><w:p><w:r><w:pict><w:txbxContent><w:p/></w:txbxContent></w:pict></w:r></w:p><w:sdt><w:sdtContent><w:p/></w:sdtContent></w:sdt>"#,
        );
        let (paragraphs, tables) = stamp(&mut body, &Handles::default());
        assert_eq!((paragraphs, tables), (5, 1));
        let mut seen = Vec::new();
        indices(&body, &mut seen);
        assert_eq!(seen, ["0", "1", "2", "3", "-", "4"]);
        assert_eq!(body.child("tbl").unwrap().attr(TABLE), Some("0"));
    }

    #[test]
    fn nested_tables_are_not_numbered() {
        let mut body = body(r#"<w:tbl><w:tr><w:tc><w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl></w:tc></w:tr></w:tbl><w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#);
        let (_, tables) = stamp(&mut body, &Handles::default());
        assert_eq!(tables, 2);
        let mut all = Vec::new();
        body.find_all("tbl", &mut all);
        let numbered: Vec<Option<&str>> = all.iter().map(|t| t.attr(TABLE)).collect();
        assert_eq!(numbered, [Some("0"), None, Some("1")]);
    }

    #[test]
    fn revision_tags_join_neighbours_then_pair_a_deletion_with_the_insertion_beside_it() {
        let d = doc(r#"<w:p><w:ins w:id="0" w:author="Ann Counsel"><w:r><w:t>x</w:t></w:r></w:ins><w:r><w:t> t </w:t></w:r><w:del w:id="1" w:author="Ann Counsel"><w:r><w:delText>a</w:delText></w:r></w:del><w:bookmarkStart w:id="9" w:name="_b"/><w:ins w:id="2" w:author="Ann Counsel"><w:r><w:t>b</w:t></w:r></w:ins><w:r><w:t> u </w:t></w:r><w:del w:id="3" w:author="Ann Counsel"><w:r><w:delText>c</w:delText></w:r></w:del><w:ins w:id="7" w:author="Ann Counsel"><w:r><w:t>d</w:t></w:r></w:ins><w:ins w:id="8" w:author="Ann Counsel"><w:r><w:t>e</w:t></w:r></w:ins></w:p>"#);
        let handles = handles(&d, None);
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles).into_iter().map(|t| format!("{}:{}", t.kind, t.tag)).collect();
        assert_eq!(tags, ["ins:0@AC", "sub:1+2@AC", "sub:3+7+8@AC"]);
    }

    #[test]
    fn tags_always_carry_the_handle_and_format_for_printing() {
        let d = doc(r#"<w:p><w:ins w:id="0" w:author="Ann Counsel"><w:r><w:t>a</w:t></w:r></w:ins><w:r><w:t> </w:t></w:r><w:del w:id="1" w:author="John Doe"><w:r><w:delText>b</w:delText></w:r></w:del></w:p>"#);
        let handles = handles(&d, None);
        assert_eq!(handles.by_author["Ann Counsel"], "AC");
        assert_eq!(handles.by_author["John Doe"], "JD");
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles).into_iter().map(|t| t.tag).collect();
        assert_eq!(tags, ["0@AC", "1@JD"]);
        assert_eq!(join_tags(&["1@AC".into(), "2@AC".into()]), "1+2@AC");
        assert_eq!(join_tags(&["1@AC".into(), "2@JD".into()]), "1@AC|2@JD");
        assert_eq!(join_tags(&["1+2@AC".into(), "3@AC".into()]), "1+2+3@AC");
        assert_eq!(format_tag("0@AC"), "#0 @AC");
        assert_eq!(format_tag("1+2@AC"), "#1+2 @AC");
        assert_eq!(format_tag("1@AC|2@JD"), "#1 @AC; #2 @JD");
        assert_eq!(handle_of("1+2@AC"), Some("AC"));
    }

    #[test]
    fn handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps() {
        let d = doc(r#"<w:p><w:ins w:id="0" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/><w:ins w:id="1" w:author="Al Cooper" w:date="2026-10-02T08:00:00Z"/><w:ins w:id="2" w:author="Al Cooper" w:date="2026-10-03T08:30:00Z"/></w:p>"#);
        let comments = parse_xml(format!(r#"<w:comments {W}><w:comment w:id="5" w:author="Arthur Souza Rodrigues" w:initials="AS" w:date="2026-10-09T16:13:00Z"/><w:comment w:id="6" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:comments>"#).as_bytes()).unwrap();
        let handles = handles(&d, Some(&comments));
        assert_eq!(handles.order, ["Ann Counsel", "Al Cooper", "Arthur Souza Rodrigues"]);
        assert_eq!(handles.by_author["Ann Counsel"], "AC");
        assert_eq!(handles.by_author["Al Cooper"], "AC2");
        assert_eq!(handles.by_author["Arthur Souza Rodrigues"], "AS");
        assert_eq!(handles.unique_date("Ann Counsel"), Some("2026-10-01T09:00:00Z"));
        assert_eq!(handles.unique_date("Al Cooper"), None);
        assert_eq!(handles.date_range("Al Cooper"), Some("2026-10-02..2026-10-03".to_string()));
        assert_eq!(handles.unique_date("Nobody"), None);
    }

    #[test]
    fn index_runs_collapse_consecutive_indices() {
        assert_eq!(runs(&[3, 4, 5, 9, 11, 12]), [(3, 5), (9, 9), (11, 12)]);
        assert!(runs(&[]).is_empty());
    }
}
```

Add `mod agent;` next to `mod critic;` at `src/markdown/from_docx/mod.rs:16`.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --lib markdown::from_docx::agent`
Expected: compile errors, `cannot find function stamp` and the rest.

- [ ] **Step 3: Implement the stamp, tags, handles and runs**

Insert above the `#[cfg(test)]` module in `agent.rs`:

```rust
/// Author handles, order and timestamps for tags, notes and the header.
#[derive(Debug, Default, Clone)]
pub(crate) struct Handles {
    /// Author name as stored → handle (`Ann Counsel` → `AC`).
    pub by_author: HashMap<String, String>,
    /// Authors in first-appearance order: revision authors in document
    /// order, then comment-only authors in part order.
    pub order: Vec<String>,
    /// Author → every `w:date` on their marks and comments.
    pub dates: HashMap<String, BTreeSet<String>>,
}

impl Handles {
    pub(crate) fn of(&self, author: Option<&str>) -> Option<&str> {
        author.and_then(|a| self.by_author.get(a)).map(String::as_str)
    }

    /// The one timestamp all of the author's marks and comments share.
    pub(crate) fn unique_date(&self, author: &str) -> Option<&str> {
        let dates = self.dates.get(author)?;
        (dates.len() == 1).then(|| dates.iter().next().map(String::as_str)).flatten()
    }

    /// `first..last` days of an author with several timestamps.
    pub(crate) fn date_range(&self, author: &str) -> Option<String> {
        let dates = self.dates.get(author)?;
        let day = |d: &String| d.get(..10).unwrap_or(d).to_string();
        match (dates.iter().next(), dates.iter().next_back()) {
            (Some(first), Some(last)) if dates.len() > 1 => Some(format!("{}..{}", day(first), day(last))),
            _ => None,
        }
    }

    /// Whether a note of this author needs its timestamp inline (`--dates`).
    pub(crate) fn needs_date(&self, author: Option<&str>) -> bool {
        author.is_some_and(|a| self.dates.get(a).is_some_and(|d| d.len() > 1))
    }
}

/// One logical revision as the id line and the header count it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RevTag {
    /// `ins`, `del`, `sub`, `mark` (paragraph mark), `row`, `cell`.
    pub kind: &'static str,
    /// Internal form: `0@AC`, `1+2@AC`, `1@AC|2@JD`.
    pub tag: String,
    pub author: Option<String>,
    pub date: Option<String>,
}

fn is_revision(local: &str) -> bool {
    matches!(local, "ins" | "del" | "moveTo" | "moveFrom" | "cellIns" | "cellDel")
}

fn kind_of(local: &str) -> Option<&'static str> {
    match local {
        "ins" | "moveTo" => Some("ins"),
        "del" | "moveFrom" => Some("del"),
        _ => None,
    }
}

/// `12@AC`: the element's `w:id` and its author's handle (`??` for an
/// author the legend does not know, which only a malformed file produces).
pub(crate) fn tag_of(element: &Element, handles: &Handles) -> String {
    let id = element.attr("id").unwrap_or("?");
    let handle = handles.of(element.attr("author")).unwrap_or("??");
    format!("{id}@{handle}")
}

/// The handle of a one-author tag (`1+2@AC` → `AC`); `None` for several.
pub(crate) fn handle_of(tag: &str) -> Option<&str> {
    if tag.contains('|') {
        return None;
    }
    tag.rsplit_once('@').map(|(_, h)| h)
}

/// Joins the tags of one logical change: `1@AC` + `2@AC` is `1+2@AC`,
/// `1@AC` + `2@JD` is `1@AC|2@JD`, `1+2@AC` + `3@AC` is `1+2+3@AC`.
pub(crate) fn join_tags(parts: &[String]) -> String {
    let handles: Vec<Option<&str>> = parts.iter().map(|p| handle_of(p)).collect();
    if handles.iter().all(|h| h.is_some() && *h == handles[0]) {
        let ids: Vec<&str> = parts
            .iter()
            .map(|p| p.rsplit_once('@').map_or(p.as_str(), |(ids, _)| ids))
            .collect();
        format!("{}@{}", ids.join("+"), handles[0].unwrap_or("??"))
    } else {
        parts.join("|")
    }
}

/// `0@AC` → `#0 @AC`; `1+2@AC` → `#1+2 @AC`; `1@AC|2@JD` → `#1 @AC; #2 @JD`.
pub(crate) fn format_tag(tag: &str) -> String {
    tag.split('|')
        .map(|one| match one.rsplit_once('@') {
            Some((ids, handle)) => format!("#{ids} @{handle}"),
            None => format!("#{one}"),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// A list of tags for an id line: `#0 @AC; #1+2 @AC`.
pub(crate) fn format_tags(tags: &[String]) -> String {
    tags.iter().map(|t| format_tag(t)).collect::<Vec<_>>().join("; ")
}

/// The revisions among a paragraph's direct children, in order, as the
/// renderer shows them: neighbouring marks of one kind by one author join
/// (`7+8@AC`), then a deletion directly followed by an insertion (or the
/// reverse) pairs as a substitution, greedily left to right. Bookmarks,
/// proofing marks and comment markers do not break adjacency; a run does.
pub(crate) fn revision_tags(p: &Element, handles: &Handles) -> Vec<RevTag> {
    // One slot per child: a revision, or `None` for anything else that
    // breaks adjacency.
    let mut slots: Vec<Option<RevTag>> = Vec::new();
    for e in p.elements() {
        if matches!(
            e.local(),
            "bookmarkStart" | "bookmarkEnd" | "proofErr" | "commentRangeStart" | "commentRangeEnd"
        ) {
            continue;
        }
        slots.push(kind_of(e.local()).map(|kind| RevTag {
            kind,
            tag: tag_of(e, handles),
            author: e.attr("author").map(str::to_string),
            date: e.attr("date").map(str::to_string),
        }));
    }
    // 1. Join neighbours of one kind by one author.
    let mut joined: Vec<Option<RevTag>> = Vec::new();
    for slot in slots {
        match (joined.last_mut(), slot) {
            (Some(Some(last)), Some(next)) if last.kind == next.kind && last.author == next.author => {
                last.tag = join_tags(&[last.tag.clone(), next.tag]);
            }
            (_, slot) => joined.push(slot),
        }
    }
    // 2. Pair a deletion with the insertion beside it.
    let mut out = Vec::new();
    let mut i = 0;
    while i < joined.len() {
        let Some(current) = joined[i].take() else {
            i += 1;
            continue;
        };
        let next_kind = joined.get(i + 1).and_then(|n| n.as_ref()).map(|n| n.kind);
        let pair = matches!((current.kind, next_kind), ("del", Some("ins")) | ("ins", Some("del")));
        if pair {
            let next = joined[i + 1].take().expect("checked above");
            let (old, new) = if current.kind == "del" { (current, next) } else { (next, current) };
            out.push(RevTag {
                kind: "sub",
                tag: join_tags(&[old.tag, new.tag]),
                author: old.author,
                date: old.date,
            });
            i += 2;
        } else {
            out.push(current);
            i += 1;
        }
    }
    out
}

/// Tracked paragraph mark: the `w:ins` / `w:del` under `w:pPr/w:rPr`.
pub(crate) fn mark_tags(p: &Element, handles: &Handles) -> (Vec<String>, Vec<String>) {
    let mut ins = Vec::new();
    let mut del = Vec::new();
    if let Some(rpr) = p.path(&["pPr", "rPr"]) {
        for e in rpr.elements() {
            match kind_of(e.local()) {
                Some("ins") => ins.push(tag_of(e, handles)),
                Some("del") => del.push(tag_of(e, handles)),
                _ => {}
            }
        }
    }
    (ins, del)
}

/// Tags of the formatting changes recorded in a paragraph.
pub(crate) fn format_change_tags(p: &Element, handles: &Handles) -> Vec<String> {
    let mut found = Vec::new();
    p.find_all("rPrChange", &mut found);
    p.find_all("pPrChange", &mut found);
    found.iter().map(|e| tag_of(e, handles)).collect()
}

struct Counter {
    p: usize,
    t: usize,
}

/// Numbers every `w:p` under `body` as `inspect::body_paragraph_nodes`
/// does (document order, text boxes excluded), numbers top-level tables,
/// and records each paragraph's revision tags. Returns (paragraphs, tables).
pub(crate) fn stamp(body: &mut Element, handles: &Handles) -> (usize, usize) {
    let mut counter = Counter { p: 0, t: 0 };
    stamp_in(body, &mut counter, handles, false);
    (counter.p, counter.t)
}

fn stamp_in(element: &mut Element, c: &mut Counter, handles: &Handles, in_cell: bool) {
    if element.is("p") {
        let revs: Vec<String> = revision_tags(element, handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .collect();
        element.attrs.push((INDEX.to_string(), c.p.to_string()));
        c.p += 1;
        if !revs.is_empty() {
            element.attrs.push((REVS.to_string(), revs.join(" ")));
        }
    }
    if element.is("tbl") && !in_cell {
        element.attrs.push((TABLE.to_string(), c.t.to_string()));
        c.t += 1;
    }
    let in_cell = in_cell || element.is("tc");
    for child in element.children.iter_mut() {
        if let Node::Element(child) = child {
            if child.is("txbxContent") {
                continue;
            }
            stamp_in(child, c, handles, in_cell);
        }
    }
}

/// The `kind:tag` entries stamped on a paragraph, as (kind, tag).
pub(crate) fn stamped_revs(p: &Element) -> Vec<(String, String)> {
    p.attr(REVS)
        .map(|revs| {
            revs.split(' ')
                .filter_map(|r| r.split_once(':').map(|(k, t)| (k.to_string(), t.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn walk_revision_authors(e: &Element, out: &mut Vec<String>, dates: &mut HashMap<String, BTreeSet<String>>) {
    if is_revision(e.local()) || e.local().ends_with("PrChange") {
        if let Some(author) = e.attr("author") {
            if !out.iter().any(|a| a == author) {
                out.push(author.to_string());
            }
            if let Some(date) = e.attr("date") {
                dates.entry(author.to_string()).or_default().insert(date.to_string());
            }
        }
    }
    for child in e.elements() {
        walk_revision_authors(child, out, dates);
    }
}

/// Authors of revisions (document order) then of comments (comment order),
/// each with a handle: the comment `w:initials` that author wrote, else the
/// uppercase initials of the name's words; a collision appends 2, 3, ….
pub(crate) fn handles(document: &Element, comments: Option<&Element>) -> Handles {
    let mut order = Vec::new();
    let mut dates: HashMap<String, BTreeSet<String>> = HashMap::new();
    walk_revision_authors(document, &mut order, &mut dates);
    let mut initials: HashMap<String, String> = HashMap::new();
    if let Some(comments) = comments {
        for c in comments.children_named("comment") {
            let Some(author) = c.attr("author") else { continue };
            if !order.iter().any(|a| a == author) {
                order.push(author.to_string());
            }
            if let Some(date) = c.attr("date") {
                dates.entry(author.to_string()).or_default().insert(date.to_string());
            }
            if let Some(i) = c.attr("initials").map(str::trim).filter(|i| !i.is_empty()) {
                initials.entry(author.to_string()).or_insert_with(|| i.to_string());
            }
        }
    }
    let mut by_author = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    for author in &order {
        let base = initials.get(author).cloned().unwrap_or_else(|| {
            let letters: String = author
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect::<String>()
                .to_uppercase();
            if letters.is_empty() { "??".to_string() } else { letters }
        });
        let mut handle = base.clone();
        let mut n = 2;
        while !taken.insert(handle.clone()) {
            handle = format!("{base}{n}");
            n += 1;
        }
        by_author.insert(author.clone(), handle);
    }
    Handles { by_author, order, dates }
}

/// Consecutive runs of sorted indices: `[3,4,5,9]` → `[(3,5),(9,9)]`.
pub(crate) fn runs(indices: &[usize]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for &i in indices {
        match out.last_mut() {
            Some((_, end)) if *end + 1 == i => *end = i,
            _ => out.push((i, i)),
        }
    }
    out
}
```

`parse_xml` is `pub(crate) fn parse_xml(bytes: &[u8]) -> Result<Element, String>` at `ooxml.rs:330`; `Element` has `pub attrs: Vec<(String, String)>` and `pub children: Vec<Node>` (`ooxml.rs:223`); `attr` matches by local name.

- [ ] **Step 4: Run the tests**

Run: `cargo test --lib markdown::from_docx::agent`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
git add src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs
git commit -m "feat(markdown): agent view numbering, tags, handles and timestamps"
```

---

### Task 3: Id lines, empties and page markers

**Files:**
- Modify: `src/markdown/from_docx/ooxml.rs:734-812` (`Blocks`)
- Modify: `src/markdown/pages.rs:34-125` (`paginate`)
- Modify: `src/markdown/from_docx/mod.rs` (`Writer` fields and the literal in `convert`, `paragraph`, `blocks`)
- Modify: `src/markdown/from_docx/agent.rs` (id line builder)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

Append to `tests/agent_text_view.rs`:

```rust
#[test]
fn id_lines_precede_every_paragraph_and_the_header_opens_the_output() {
    let bytes = docx(&format!(
        "{}{}",
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#,
        para("Body text.")
    ));
    let out = agent(&bytes);
    assert!(out.starts_with("---\nsource: sample.docx\n"), "{out}");
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p0 center -->\n# Title\n\n<!-- p1 -->\nBody text.\n"
    );
}

#[test]
fn empty_paragraphs_collapse_and_cached_breaks_number_the_pages() {
    let bytes = docx(&format!(
        "{}<w:p/><w:p/><w:p/>{}<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p><w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:lastRenderedPageBreak/><w:t>Second page</w:t></w:r></w:p>",
        para("One"),
        para("Five")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nOne\n\n<!-- p1-p3 empty -->\n\n<!-- p4 -->\nFive\n\n<!-- p5 page-break -->\n\n<!-- page 2 of 2 -->\n\n<!-- p6 center -->\nSecond page\n"
    );
}

#[test]
fn trailing_empty_paragraphs_are_still_listed() {
    let bytes = docx(&format!("{}<w:p/>", para("Only")));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nOnly\n\n<!-- p1 empty -->\n"
    );
}

#[test]
fn layout_page_texts_place_the_markers_above_the_id_lines() {
    let bytes = docx(&format!("{}{}{}", para("Alpha text here"), para("Beta text here"), para("Gamma text here")));
    let out = agent_options(
        &bytes,
        MarkdownOptions {
            pages: Some(vec!["Alpha text here".into(), "Beta text here Gamma text here".into()]),
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&out),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nAlpha text here\n\n<!-- page 2 of 2 -->\n\n<!-- p1 -->\nBeta text here\n\n<!-- p2 -->\nGamma text here\n"
    );
    assert!(out.contains("\nbody: p0-p2, 0 tables, 2 pages     # pages from layout\n"), "{out}");
}

#[test]
fn paginate_holds_comment_lines_at_a_block_start() {
    let md = "<!-- p0 -->\nAlpha text here\n\n<!-- p1 page-break -->\n\n<!-- t0 1x1, cells p2-p2 by row -->\n|Beta text here|\n|-|\n";
    assert_eq!(
        jubarte::markdown::paginate(md, &["alpha text here", "beta text here"]),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nAlpha text here\n\n<!-- p1 page-break -->\n\n<!-- page 2 of 2 -->\n\n<!-- t0 1x1, cells p2-p2 by row -->\n|Beta text here|\n|-|\n"
    );
}
```

The `body:` line's comment is settled in Task 10; the fourth test's assertion on it is written now so it fails for the right reason then.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view id_lines empty_paragraphs trailing_empty layout_page paginate_holds`
Expected: the first four FAIL (no `<!-- ` lines); `paginate_holds_comment_lines_at_a_block_start` FAILS with the second marker between `<!-- t0 … -->` and `|Beta text here|`, or missing.

- [ ] **Step 3: `Blocks::push_line`**

In `ooxml.rs`, add a field to `Blocks` and initialise it in `new()`:

```rust
    /// The next block follows after one newline: an id line was just
    /// written (agent view).
    tight: bool,
```

Add the method after `push_prefixed`:

```rust
    /// An id or table line of the agent view. It stands on its own line and
    /// the next block follows it directly, without a blank line.
    pub(crate) fn push_line(&mut self, line: &str) {
        self.separator = None;
        if !self.out.is_empty() {
            self.out.push_str(if self.tight { "\n" } else { "\n\n" });
        }
        self.out.push_str(line);
        self.last_was_list = false;
        self.tight = true;
    }
```

In `push_paragraph`, replace the separator choice with

```rust
            let separator = if self.tight {
                "\n"
            } else if is_list && self.last_was_list {
                "\n"
            } else {
                "\n\n"
            };
```

and set `self.tight = false;` right after `self.last_was_list = is_list;`.

- [ ] **Step 4: `paginate` holds comment lines at a block start**

In `src/markdown/pages.rs`, `paginate` (line 34). Declare `let mut held = String::new();` before the `for line in markdown.split_inclusive('\n')` loop. As the first statements inside the loop, after `let trimmed = line.trim();`:

```rust
        // The agent view's id and table lines open a block as HTML
        // comments. They are held back: the block's key is the text under
        // them, and a marker due for the block goes above them.
        if block_start && fence.is_none() && trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            held.push_str(line);
            continue;
        }
```

Right before `out.push_str(line);` (after the marker may have been pushed), insert:

```rust
        out.push_str(&held);
        held.clear();
```

After the loop, before `out`, add `out.push_str(&held);`. `continue` skips the `block_start = …` assignment at the loop's end, so the line under the comment is still read as the block start. Empty lines keep their current behaviour.

- [ ] **Step 5: The id line builder**

Append to `agent.rs` (above the tests):

```rust
/// What the id line of a paragraph says beyond its index.
pub(crate) struct LineFacts<'a> {
    pub index: usize,
    pub style: Option<&'a str>,
    pub default_style: Option<&'a str>,
    pub heading: Option<usize>,
    /// The list marker the converter computed (`1.`, `(a)`, `-`).
    pub marker: Option<&'a str>,
    pub page_break: bool,
    /// Accept-all or reject-all view: print the `rev` tags.
    pub resolved: bool,
    /// Comment ids to list (comments hidden).
    pub comments: &'a [String],
}

/// Twips as inches with up to two decimals: `720` → `0.5in`, `1440` → `1in`.
pub(crate) fn inches(twips: f64) -> String {
    let value = format!("{:.2}", twips / 1440.0);
    let value = value.trim_end_matches('0').trim_end_matches('.');
    format!("{value}in")
}

pub(crate) fn has_page_break(p: &Element) -> bool {
    let mut breaks = Vec::new();
    p.find_all("br", &mut breaks);
    breaks.iter().any(|b| b.attr("type") == Some("page"))
}

pub(crate) fn has_rendered_page_break(p: &Element) -> bool {
    let mut found = Vec::new();
    p.find_all("lastRenderedPageBreak", &mut found);
    !found.is_empty()
}

pub(crate) fn has_section_break(p: &Element) -> bool {
    p.path(&["pPr", "sectPr"]).is_some()
}

/// `<!-- head -->` or `<!-- head clause, clause -->`.
pub(crate) fn line(head: &str, clauses: &[String]) -> String {
    if clauses.is_empty() {
        format!("<!-- {head} -->")
    } else {
        format!("<!-- {head} {} -->", clauses.join(", "))
    }
}

/// `<!-- p3 justify, first-line 0.5in, comments #c5 -->` (the plan's grammar).
pub(crate) fn id_line(p: &Element, f: &LineFacts, handles: &Handles) -> String {
    let mut head = format!("p{}", f.index);
    if let Some(style) = f.style {
        let implied = match f.heading {
            Some(level) => style == format!("Heading{level}"),
            None => Some(style) == f.default_style || style == "Normal",
        };
        if !implied {
            head.push(' ');
            head.push_str(style);
        }
    }
    let ppr = p.child("pPr");
    let mut clauses: Vec<String> = Vec::new();
    if let Some(jc) = ppr.and_then(|pr| pr.child("jc")).and_then(|j| j.attr("val")) {
        clauses.push(
            match jc {
                "center" => "center",
                "right" | "end" => "right",
                "both" | "distribute" => "justify",
                _ => "left",
            }
            .to_string(),
        );
    }
    if let Some(ind) = ppr.and_then(|pr| pr.child("ind")) {
        for (attr, name) in [
            ("firstLine", "first-line"),
            ("hanging", "hanging"),
            ("left", "left"),
            ("start", "left"),
            ("right", "right"),
            ("end", "right"),
        ] {
            if let Some(v) = ind.attr(attr).and_then(|v| v.parse::<f64>().ok()) {
                clauses.push(format!("{name} {}", inches(v)));
            }
        }
    }
    match f.marker {
        Some("-") => clauses.push("bullet".to_string()),
        Some(label) => clauses.push(format!("num \"{label}\"")),
        None => {}
    }
    if f.page_break {
        clauses.push("page-break".to_string());
    }
    if has_section_break(p) {
        clauses.push("section-break".to_string());
    }
    let (ins, del) = mark_tags(p, handles);
    if !ins.is_empty() {
        clauses.push(format!("break-ins {}", format_tags(&ins)));
    }
    if !del.is_empty() {
        clauses.push(format!("break-del {}", format_tags(&del)));
    }
    let fmt = format_change_tags(p, handles);
    if !fmt.is_empty() {
        clauses.push(format!("fmt {}", format_tags(&fmt)));
    }
    if f.resolved {
        let tags: Vec<String> = stamped_revs(p).into_iter().map(|(_, tag)| tag).collect();
        if !tags.is_empty() {
            clauses.push(format!("rev {}", format_tags(&tags)));
        }
    }
    if !f.comments.is_empty() {
        let ids: Vec<String> = f.comments.iter().map(|c| format!("#c{c}")).collect();
        clauses.push(format!("comments {}", ids.join(" ")));
    }
    line(&head, &clauses)
}

/// `<!-- p19 empty -->` / `<!-- p19-p23 empty -->` lines for pending empties.
pub(crate) fn empty_lines(indices: &[usize]) -> Vec<String> {
    runs(indices)
        .into_iter()
        .map(|(a, b)| {
            if a == b { format!("<!-- p{a} empty -->") } else { format!("<!-- p{a}-p{b} empty -->") }
        })
        .collect()
}

/// The page marker `paginate` writes, for the cached-break fallback.
pub(crate) fn page_marker(page: usize, total: usize) -> String {
    format!("<!-- page {page} of {total} -->")
}
```

- [ ] **Step 6: Wire the Writer**

In `mod.rs`, add fields to `struct Writer` (line 564) and initialise them in the `Writer { … }` literal inside `convert` (line 120 area):

```rust
    /// Agent view (`Options::ids`).
    agent: bool,
    /// Agent view: comments inline, or hidden and listed on id lines.
    comments_inline: bool,
    /// Agent view: accept-all or reject-all (revisions already resolved).
    resolved: bool,
    /// Agent view: inline timestamps on notes (`Options::dates`).
    dates: bool,
    handles: agent::Handles,
    /// Agent view: the default paragraph style id (`w:default="1"`).
    default_style: Option<String>,
    /// Agent view: empty paragraphs not yet written as `<!-- pN empty -->`.
    pending_empty: Vec<usize>,
    /// Agent view, cached-break fallback: `Some(total)` makes the writer
    /// emit `<!-- page N of total -->` itself; `None` leaves markers to
    /// `paginate`.
    cached_pages: Option<usize>,
    /// Agent view: pages announced so far by the writer.
    page: usize,
    /// Agent view: comment ids met since the last id line.
    para_comments: Vec<String>,
```

Initial values: `agent: options.ids, comments_inline: options.comments, resolved: accept.is_some(), dates: options.dates, handles: handles.clone(), default_style, pending_empty: Vec::new(), cached_pages, page: 0, para_comments: Vec::new()`, where the last three come from the block below.

Still in `convert`, change `let document = …` (line 66) to `let mut document`, move `let part = …` and `let rels = …` (lines 80-85) above the resolution, and insert before `let document = match accept { … }`:

```rust
    let comments_root = package
        .xml(&part("/comments", "word/comments.xml"))
        .ok()
        .flatten();
    let handles = if options.ids {
        agent::handles(&document, comments_root.as_ref())
    } else {
        agent::Handles::default()
    };
    // (paragraphs, tables) numbered; the header reads the counts in Task 10.
    let stamped = if options.ids {
        document
            .children
            .iter_mut()
            .find_map(|n| match n {
                ooxml::Node::Element(e) if e.is("body") => Some(e),
                _ => None,
            })
            .map(|body| agent::stamp(body, &handles))
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };
    // The body before resolution, for the header's counts and page facts.
    let original = options.ids.then(|| document.clone());
    // Cached-break page lines: only without layout pages, and only when
    // page lines are wanted at all (`--no-page-markers` turns them off).
    let cached_pages = match (&options.pages, &original) {
        (Some(_), _) | (None, None) => None,
        (None, Some(_)) if !options.page_markers => None,
        (None, Some(original)) => {
            let body = original.child("body").unwrap_or(original);
            let (rendered, hard) = agent::page_counts(body);
            Some(if rendered > 0 { 1 + rendered } else { 1 + hard })
        }
    };
```

Replace the later `let comments = package.xml(…).ok().flatten().filter(|_| accept.is_none());` (line 115) with `let comments = comments_root.clone().filter(|_| accept.is_none() || options.ids);`.

`default_style`: keep the raw styles root in a variable before it is parsed (`let styles_root = package.xml(&part("/styles", "word/styles.xml")).ok().flatten();` then `Styles::parse` from `styles_root.as_ref()`), and set

```rust
    let default_style = styles_root.as_ref().and_then(|root| {
        root.children_named("style")
            .find(|s| s.attr("type") == Some("paragraph") && s.attr("default").is_some_and(|d| d == "1" || d == "true"))
            .and_then(|s| s.attr("styleId"))
            .map(str::to_string)
    });
```

Add `page_counts` to `agent.rs`:

```rust
/// Paragraphs (text boxes excluded) holding `w:lastRenderedPageBreak`, and
/// hard page and section breaks, for the cached-break page count.
pub(crate) fn page_counts(e: &Element) -> (usize, usize) {
    if e.is("txbxContent") {
        return (0, 0);
    }
    let mut rendered = 0;
    let mut hard = 0;
    if e.is("p") {
        rendered += usize::from(has_rendered_page_break(e));
        hard += usize::from(has_page_break(e)) + usize::from(has_section_break(e));
    }
    for child in e.elements() {
        let (r, h) = page_counts(child);
        rendered += r;
        hard += h;
    }
    (rendered, hard)
}
```

Add two Writer helpers (inside `impl Writer<'_>`):

```rust
    /// Agent view: writes the pending `<!-- pN empty -->` lines.
    fn flush_empty(&mut self, blocks: &mut Blocks) {
        for line in agent::empty_lines(&std::mem::take(&mut self.pending_empty)) {
            blocks.push_line(&line);
        }
    }

    /// Agent view, cached-break fallback: the page markers due before a
    /// block, page 1 included. With layout pages, `paginate` writes them.
    fn page_lines(&mut self, blocks: &mut Blocks, rendered_break: bool) {
        let Some(total) = self.cached_pages else { return };
        if self.page == 0 {
            self.page = 1;
            blocks.push(&agent::page_marker(1, total), false);
        }
        if rendered_break && self.page < total {
            self.page += 1;
            blocks.push(&agent::page_marker(self.page, total), false);
        }
    }
```

`blocks.push` (not `push_line`) so that a blank line follows the marker, as `paginate` writes it. The `self.page < total` guard covers a paragraph that holds both a hard break and a cached break.

In `Writer::paragraph`, after `let (inline, extra) = self.paragraph_inline(p);` and `let written = !inline.is_blank();`, insert the agent block. The markers come first, computed exactly once and only for a written paragraph, which is when the plain conversion computes them today (so Word's list counters advance identically):

```rust
        let list_marker: Option<(String, usize)> = if written && heading.is_none() {
            num.clone()
                .and_then(|(id, ilvl)| self.list_marker(&id, ilvl.min(8)).map(|m| (m, ilvl)))
        } else {
            None
        };
        let heading_marker: Option<String> = if self.agent && written && heading.is_some() {
            num.clone().and_then(|(id, ilvl)| self.list_marker(&id, ilvl.min(8)))
        } else {
            None
        };
        let index = if self.agent {
            p.attr(agent::INDEX).and_then(|v| v.parse::<usize>().ok())
        } else {
            None
        };
        if let Some(index) = index {
            let page_break = agent::has_page_break(p);
            if !written && extra.is_empty() && !page_break && !agent::has_section_break(p) {
                self.pending_empty.push(index);
                self.para_comments.clear();
                return;
            }
            self.flush_empty(blocks);
            self.page_lines(blocks, agent::has_rendered_page_break(p));
            let marker = heading_marker
                .as_deref()
                .or(list_marker.as_ref().map(|(m, _)| m.as_str()));
            let comments = if self.comments_inline { Vec::new() } else { std::mem::take(&mut self.para_comments) };
            self.para_comments.clear();
            let facts = agent::LineFacts {
                index,
                style: style.as_deref(),
                default_style: self.default_style.as_deref(),
                heading,
                marker,
                page_break,
                resolved: self.resolved,
                comments: &comments,
            };
            blocks.push_line(&agent::id_line(p, &facts, &self.handles));
        }
```

`num` is `Option<(String, usize)>`; `num.clone()` keeps it available. In the heading branch, prefix the label when present:

```rust
                let mut text = inline.render(false).replace('\n', " ");
                if let Some(label) = &heading_marker {
                    text = format!("{label} {text}");
                }
```

In the list branch, use the precomputed value instead of recomputing it: `} else if let Some((marker, ilvl)) = list_marker {` (the branch body stays as it is). Because `list_marker` is computed only when `written && heading.is_none()`, which is exactly the condition under which the old code reached that call, the plain conversion's list numbering is unchanged.

At the end of `paragraph`, make the separator agent-aware:

```rust
        blocks.set_separator(if self.agent { None } else { self.paragraph_mark(p).filter(|_| written && !boxed) });
```

In `convert`, after `writer.blocks(body, &mut blocks, &mut list);` add `writer.flush_empty(&mut blocks);`. Where the markdown is assembled (`let mut markdown = blocks.finish(); …`), run the layout markers in agent mode:

```rust
    if let Some(pages) = options.pages.as_ref().filter(|_| options.ids) {
        let pages: Vec<&str> = pages.iter().map(String::as_str).collect();
        markdown = crate::markdown::paginate(&markdown, &pages);
    }
```

(this stays before the header is prepended in Task 10, so the first marker precedes the first block, not the YAML).

- [ ] **Step 7: Run the tests**

Run: `cargo test --test agent_text_view`
Expected: the five new tests PASS except the `body:` assertion in `layout_page_texts_place_the_markers_above_the_id_lines` (header not built yet; mark that single assertion `// Task 10` and move it there if you prefer a green suite between tasks).
Run: `cargo test --test docx_to_markdown_critic_markup --test docx_markdown_round_trip --test markdown_cli --test markdown_cli_page`
Expected: PASS (legacy path and `convert -t md` markers untouched; a document without HTML comment lines never enters the held branch).

- [ ] **Step 8: Commit**

```bash
git add src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs src/markdown/from_docx/ooxml.rs src/markdown/pages.rs tests/agent_text_view.rs
git commit -m "feat(markdown): agent view id lines, empty runs and page markers"
```

---

### Task 4: Id line annotations

**Files:**
- Test: `tests/agent_text_view.rs`
- Modify: `src/markdown/from_docx/agent.rs` only if a test fails

- [ ] **Step 1: Write the tests**

```rust
#[test]
fn id_line_shows_direct_alignment_indent_style_and_number() {
    let numbering = Part {
        name: "word/numbering.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
        xml: &format!(r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#),
    };
    let body_xml = concat!(
        r#"<w:p><w:pPr><w:pStyle w:val="Quote"/><w:jc w:val="both"/><w:ind w:left="720" w:hanging="360"/></w:pPr><w:r><w:t>Quoted.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>First item</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>Numbered heading</w:t></w:r></w:p>"#,
    );
    let styles = Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &format!(r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr></w:style></w:styles>"#),
    };
    let bytes = common::docx::docx_with(body_xml, &[numbering, styles]);
    let out = body(&agent(&bytes)).to_string();
    assert!(out.contains("<!-- p0 Quote justify, hanging 0.25in, left 0.5in -->\nQuoted."), "{out}");
    assert!(out.contains("<!-- p1 num \"1.\" -->\n1. First item"), "{out}");
    assert!(out.contains("<!-- p2 num \"2.\" -->\n## 2. Numbered heading"), "{out}");
}

#[test]
fn id_line_names_tracked_paragraph_marks_and_formatting_changes() {
    let body_xml = concat!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="4" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr><w:r><w:t>Split here</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>Now bold</w:t></w:r></w:p>"#,
    );
    let out = body(&agent(&docx(body_xml))).to_string();
    assert!(out.contains("<!-- p0 break-ins #4 @AC -->\nSplit here"), "{out}");
    assert!(out.contains("<!-- p1 fmt #5 @AC -->\n**Now bold**"), "{out}");
}
```

The list item `1. First item` is what the converter writes for a decimal `%1.` level today; the id line adds `num "1."` so an agent knows the label is not text.

- [ ] **Step 2: Run them**

Run: `cargo test --test agent_text_view id_line_`
Expected: PASS if Task 3 was implemented as written. If `num` or `fmt` are missing, fix `id_line`/`paragraph`; the indent clause order is `first-line, hanging, left, right`.

- [ ] **Step 3: Commit**

```bash
git add tests/agent_text_view.rs src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs
git commit -m "test(markdown): id line annotations"
```

---

### Task 5: Table lines

**Files:**
- Modify: `src/markdown/from_docx/agent.rs` (`table_line`), `src/markdown/from_docx/mod.rs` (`blocks`, `"tbl"` arm at line 618)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
const TABLE_3X2: &str = r#"<w:tbl><w:tblPr><w:jc w:val="center"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:p><w:r><w:t>Item</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Due</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Report</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Day 10</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Call</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Monthly</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

#[test]
fn table_line_gives_grid_cells_and_header_row() {
    let bytes = docx(&format!("{}{TABLE_3X2}{}", para("Before"), para("After")));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nBefore\n\n<!-- t0 center 3x2, cells p1-p6 by row, header row repeats -->\n|Item|Due|\n|-|-|\n|Report|Day 10|\n|Call|Monthly|\n\n<!-- p7 -->\nAfter\n"
    );
}

#[test]
fn table_line_lists_rows_when_a_cell_holds_two_paragraphs_and_notes_merges_and_marks() {
    let tbl = r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>Head</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:pPr><w:rPr><w:ins w:id="12" w:author="Ann Counsel"/></w:rPr></w:pPr><w:r><w:t>a</w:t></w:r></w:p><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let out = body(&agent(&docx(tbl))).to_string();
    assert!(out.starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 2x2, cells r0 p0 r1 p1-p3, merged cells, break-ins #12 @AC in p1 -->\n"), "{out}");
}
```

The pipe table shape (`|Item|Due|` without spaces, `|-|-|`) is what `markdown_table` writes today (`mod.rs:242`); the agent view does not change it, and the goldens use that shape.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view table_line`
Expected: FAIL, no `<!-- t0` line.

- [ ] **Step 3: Implement `table_line` and emit it**

Append to `agent.rs`:

```rust
/// `<!-- t0 center 3x3, cells p8-p16 by row, header row repeats -->`;
/// `None` for a nested table (not numbered).
pub(crate) fn table_line(tbl: &Element, resolved: bool, handles: &Handles) -> Option<String> {
    let t = tbl.attr(TABLE)?;
    let rows: Vec<&Element> = tbl.children_named("tr").collect();
    let cols = tbl
        .child("tblGrid")
        .map(|g| g.children_named("gridCol").count())
        .filter(|&c| c > 0)
        .unwrap_or_else(|| rows.iter().map(|r| r.children_named("tc").count()).max().unwrap_or(0));
    let mut head = format!("t{t}");
    if let Some(jc) = tbl.path(&["tblPr", "jc"]).and_then(|j| j.attr("val")) {
        match jc {
            "center" => head.push_str(" center"),
            "right" | "end" => head.push_str(" right"),
            _ => {}
        }
    }
    head.push_str(&format!(" {}x{}", rows.len(), cols));
    let mut clauses: Vec<String> = Vec::new();
    let mut per_row: Vec<(usize, usize)> = Vec::new();
    let mut uniform = true;
    let mut merged = false;
    let mut break_ins: Vec<String> = Vec::new();
    let mut break_del: Vec<String> = Vec::new();
    let mut revs: Vec<String> = Vec::new();
    for tr in &rows {
        let mut range: Option<(usize, usize)> = None;
        for tc in tr.children_named("tc") {
            if tc.path(&["tcPr", "gridSpan"]).is_some() || tc.path(&["tcPr", "vMerge"]).is_some() {
                merged = true;
            }
            let mut ps = Vec::new();
            tc.find_all("p", &mut ps);
            let idx: Vec<usize> = ps.iter().filter_map(|p| p.attr(INDEX)?.parse().ok()).collect();
            if idx.len() != 1 {
                uniform = false;
            }
            for p in &ps {
                let Some(i) = p.attr(INDEX) else { continue };
                let (ins, del) = mark_tags(p, handles);
                break_ins.extend(ins.iter().map(|t| format!("{} in p{i}", format_tag(t))));
                break_del.extend(del.iter().map(|t| format!("{} in p{i}", format_tag(t))));
                if resolved {
                    revs.extend(stamped_revs(p).into_iter().map(|(_, tag)| format!("{} in p{i}", format_tag(&tag))));
                }
            }
            if let (Some(&a), Some(&b)) = (idx.first(), idx.last()) {
                range = Some(match range {
                    None => (a, b),
                    Some((start, _)) => (start, b),
                });
            }
        }
        if let Some(r) = range {
            per_row.push(r);
        }
    }
    if uniform {
        if let (Some(&(a, _)), Some(&(_, b))) = (per_row.first(), per_row.last()) {
            clauses.push(format!("cells p{a}-p{b} by row"));
        }
    } else {
        let rows: Vec<String> = per_row
            .iter()
            .enumerate()
            .map(|(i, (a, b))| if a == b { format!("r{i} p{a}") } else { format!("r{i} p{a}-p{b}") })
            .collect();
        clauses.push(format!("cells {}", rows.join(" ")));
    }
    if rows.first().is_some_and(|tr| tr.path(&["trPr", "tblHeader"]).is_some()) {
        clauses.push("header row repeats".to_string());
    }
    if merged {
        clauses.push("merged cells".to_string());
    }
    if !break_ins.is_empty() {
        clauses.push(format!("break-ins {}", break_ins.join("; ")));
    }
    if !break_del.is_empty() {
        clauses.push(format!("break-del {}", break_del.join("; ")));
    }
    if !revs.is_empty() {
        clauses.push(format!("rev {}", revs.join("; ")));
    }
    Some(line(&head, &clauses))
}
```

In `Writer::blocks`, the `"tbl"` arm becomes:

```rust
                "tbl" => {
                    list.reset();
                    let notes = self.take_notes();
                    blocks.push_prefixed("", &notes, false);
                    if self.agent {
                        if let Some(line) = agent::table_line(child, self.resolved, &self.handles) {
                            self.flush_empty(blocks);
                            let mut breaks = Vec::new();
                            child.find_all("lastRenderedPageBreak", &mut breaks);
                            self.page_lines(blocks, !breaks.is_empty());
                            blocks.push_line(&line);
                        }
                    }
                    let table = self.table(child);
                    blocks.push(&table, false);
                }
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test agent_text_view table_line`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs tests/agent_text_view.rs
git commit -m "feat(markdown): agent view table lines"
```

---

### Task 6: Attribution notes with ids and handles

**Files:**
- Modify: `src/markdown/from_docx/critic.rs` (`render` at 396, `tidy` at 282, `space_into_last_change` at 348, `attribution` at 523)
- Modify: `src/markdown/from_docx/mod.rs` (`Writer::change_of`, `Writer::revision_marks`, call sites, `paragraph_mark`)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
fn ins(id: u32, author: &str, text: &str) -> String {
    ins_at(id, author, "2026-10-01T09:00:00Z", text)
}

fn ins_at(id: u32, author: &str, date: &str, text: &str) -> String {
    format!(r#"<w:ins w:id="{id}" w:author="{author}" w:date="{date}"><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:ins>"#)
}

fn del(id: u32, author: &str, text: &str) -> String {
    format!(r#"<w:del w:id="{id}" w:author="{author}" w:date="2026-10-01T09:00:00Z"><w:r><w:delText xml:space="preserve">{text}</w:delText></w:r></w:del>"#)
}

fn run(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

#[test]
fn attribution_notes_follow_each_change_with_id_and_handle() {
    let p = format!(
        "<w:p>{}{}{}{}{}{}{}</w:p>",
        run("Pay within "), del(3, "Ann Counsel", "thirty"), ins(4, "Ann Counsel", "forty-five"),
        run(" days, "), ins(5, "Ann Counsel", "quarterly"), run(" reports"), del(6, "Ann Counsel", ", nothing else")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nPay within {~~thirty~>forty-five~~}{>>#3+4 @AC<<} days, {++quarterly++}{>>#5 @AC<<} reports{--, nothing else--}{>>#6 @AC<<}\n"
    );
}

#[test]
fn neighbouring_marks_by_one_author_share_one_note_and_keep_their_text() {
    let p = format!("<w:p>{}{}{}</w:p>", run("a "), ins(7, "Ann Counsel", "bold"), ins(8, "Ann Counsel", " words"));
    assert_eq!(body(&agent(&docx(&p))), "<!-- page 1 of 1 -->\n\n<!-- p0 -->\na {++bold words++}{>>#7+8 @AC<<}\n");
}

#[test]
fn a_substitution_by_two_authors_gets_two_notes_deleted_side_first() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("x "), del(1, "Ann Counsel", "old"), ins(2, "John Doe", "new"), ins(3, "Ann Counsel", "!")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nx {~~old~>new~~}{>>#1 @AC<<}{>>#2 @JD<<}{++!++}{>>#3 @AC<<}\n"
    );
}

#[test]
fn dates_go_inline_only_when_asked_and_only_for_an_author_with_several() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("a "), ins_at(1, "Ann Counsel", "2026-10-01T09:00:00Z", "b"), run(" c "), ins_at(2, "Ann Counsel", "2026-10-03T14:05:00Z", "d")
    );
    let bytes = docx(&p);
    assert!(body(&agent(&bytes)).contains("{++b++}{>>#1 @AC<<} c {++d++}{>>#2 @AC<<}"), "{}", agent(&bytes));
    let dated = agent_options(&bytes, MarkdownOptions { dates: true, ..agent_defaults() });
    assert!(body(&dated).contains("{++b++}{>>#1 @AC 2026-10-01T09:00:00Z<<} c {++d++}{>>#2 @AC 2026-10-03T14:05:00Z<<}"), "{dated}");
    let single = docx(&format!("<w:p>{}{}</w:p>", run("a "), ins(1, "Ann Counsel", "b")));
    let dated = agent_options(&single, MarkdownOptions { dates: true, ..agent_defaults() });
    assert!(body(&dated).contains("{++b++}{>>#1 @AC<<}"), "{dated}");
}

#[test]
fn legacy_output_keeps_author_notes() {
    let p = format!("<w:p>{}{}</w:p>", run("a "), ins(7, "Ann Counsel", "b"));
    let legacy = docx_to_markdown(&docx(&p), &MarkdownOptions::default()).unwrap().markdown;
    assert_eq!(legacy, "a{++ b++}{>>Ann Counsel (2026-10-01T09:00:00Z)<<}\n");
}
```

The last expectation is today's output (`space_into_last_change` moves the space into a change that ends the paragraph); run it first and, if the plain converter prints `a {++b++}…` instead, keep whatever it prints: that test pins the legacy behaviour, whatever it is.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view attribution_notes neighbouring a_substitution dates_go legacy_output`
Expected: the first four FAIL with `{>>Ann Counsel (…)<<}` notes; `legacy_output_keeps_author_notes` passes.

- [ ] **Step 3: `critic.rs`: tagged attributions**

Add near `Change` (line 57):

```rust
/// Agent-view attribution: an internal tag (`0@AC`, `1+2@AC`, optionally
/// `0@AC 2026-10-03T14:05:00Z`) behind this sentinel, so that it is never
/// mistaken for an author's note. Printed through `agent::format_tag`;
/// never written to the output.
pub(crate) const TAG: &str = "\u{E000}";

fn tag(by: Option<&str>) -> Option<&str> {
    by?.strip_prefix(TAG)
}

/// The tag proper and its optional inline timestamp.
fn tag_parts(tagged: &str) -> (&str, Option<&str>) {
    match tagged.split_once(' ') {
        Some((t, date)) => (t, Some(date)),
        None => (tagged, None),
    }
}

/// A note's text: the formatted tag, or the plain attribution.
fn display(by: &str) -> String {
    match by.strip_prefix(TAG) {
        Some(tagged) => {
            let (t, date) = tag_parts(tagged);
            let mut out = super::agent::format_tag(t);
            if let Some(date) = date {
                out.push(' ');
                out.push_str(date);
            }
            out
        }
        None => by.to_string(),
    }
}

/// Two attributions join as neighbours: equal notes, or two tags of one
/// author (their ids differ by design).
fn same_author(a: Option<&str>, b: Option<&str>) -> bool {
    match (tag(a), tag(b)) {
        (Some(a), Some(b)) => {
            super::agent::handle_of(tag_parts(a).0) == super::agent::handle_of(tag_parts(b).0)
        }
        _ => a == b,
    }
}

/// The tags of two neighbours or of a substitution's two sides, joined,
/// keeping the first inline timestamp.
fn join_tagged(a: &str, b: &str) -> String {
    let (ta, da) = tag_parts(a);
    let (tb, db) = tag_parts(b);
    let mut out = format!("{TAG}{}", super::agent::join_tags(&[ta.to_string(), tb.to_string()]));
    if let Some(date) = da.or(db) {
        out.push(' ');
        out.push_str(date);
    }
    out
}
```

Change `attribution` (line 523) to print the display form:

```rust
fn attribution(out: &mut Vec<Piece>, by: Option<&str>, id: usize) {
    if let Some(by) = by {
        marker(out, &note(&display(by)), id);
    }
}
```

In `render` (line 396), the substitution arm: replace the two `attribution(…)` calls after `marker(out, "~~}", id);` with

```rust
                match (tag(old_by.as_deref()), tag(new_by.as_deref())) {
                    (Some(a), Some(b)) if same_author(old_by.as_deref(), new_by.as_deref()) => {
                        let joined = join_tagged(a, b);
                        attribution(out, Some(&joined), id);
                    }
                    _ => {
                        attribution(out, old_by.as_deref(), id);
                        if new_by != old_by {
                            attribution(out, new_by.as_deref(), id);
                        }
                    }
                }
```

The single-span arm is unchanged (`attribution` now formats tags). In `tidy` (line 282), the neighbour join:

```rust
        } else if let Some(Node::Span(_, before_by, before)) = out
            .last_mut()
            .filter(|last| matches!(last, Node::Span(m, b, _) if *m == mark && same_author(b.as_deref(), by.as_deref())))
        {
            if let (Some(a), Some(b)) = (tag(before_by.as_deref()), tag(by.as_deref())) {
                *before_by = Some(join_tagged(a, b));
            }
            before.extend(nodes);
        } else {
```

In `space_into_last_change` (line 348), a tagged change keeps the text exactly as the file holds it. Name the attribution in the destructuring (`Node::Span(Mark::Insertion | Mark::Deletion, by, inner)`, it is `_` today) and add right after the `else { return; };`:

```rust
    if tag(by.as_deref()).is_some() {
        return;
    }
```

`whole_change`, `splice`, `join_marked` and `join_changed` need no change: they read the rendered note text (`#3+4 @AC`), which is the same for the same change.

- [ ] **Step 4: `mod.rs`: tag attributions in agent mode**

Add to `impl Writer<'_>`:

```rust
    /// The agent tag of a revision element, with its timestamp when
    /// `--dates` asks for one and the author has several.
    fn agent_tag(&self, element: &Element) -> String {
        let mut tagged = format!("{}{}", critic::TAG, agent::tag_of(element, &self.handles));
        if self.dates && self.handles.needs_date(element.attr("author")) {
            if let Some(date) = element.attr("date") {
                tagged.push(' ');
                tagged.push_str(date);
            }
        }
        tagged
    }

    /// A tracked change's mark with its attribution: the agent tag, or the
    /// `Author (date)` note of the plain conversion.
    fn change_of(&self, element: &Element) -> Change {
        let (mark, by) = change_of(element);
        if self.agent { (mark, Some(self.agent_tag(element))) } else { (mark, by) }
    }

    /// [`revision_marks`] with agent tags when the agent view is on.
    fn revision_marks(&self, properties: &Element) -> Vec<Change> {
        if !self.agent {
            return revision_marks(properties);
        }
        properties
            .elements()
            .filter_map(|p| {
                let mark = match p.local() {
                    "ins" | "moveTo" | "cellIns" => Mark::Insertion,
                    "del" | "moveFrom" | "cellDel" => Mark::Deletion,
                    _ => return None,
                };
                Some((mark, Some(self.agent_tag(p))))
            })
            .collect()
    }
```

Then, inside `impl Writer<'_>` only, replace every call of the free functions with the methods: `change_of(child)` → `self.change_of(child)` (in `blocks`, `cell_parts`, and the `ins`/`del` arms of `inline`/`run`; `grep -n 'change_of(' src/markdown/from_docx/mod.rs` lists them), and `revision_marks(` → `self.revision_marks(` in `table`, `cell_text` and `Writer::paragraph_mark`. Leave the free functions and their other callers alone. `self.revised(self.change_of(child), …)` compiles (two-phase borrow).

Make `Writer::paragraph_mark` (line 1124) return `None` when `self.agent`: paragraph marks are printed on id and table lines.

- [ ] **Step 5: Run the tests**

Run: `cargo test --test agent_text_view`
Expected: all PASS.
Run: `cargo test --lib markdown::from_docx`
Expected: PASS (the legacy arms are unchanged).

- [ ] **Step 6: Commit**

```bash
git add src/markdown/from_docx/critic.rs src/markdown/from_docx/mod.rs tests/agent_text_view.rs
git commit -m "feat(markdown): attribution notes with ids and handles in the agent view"
```

---

### Task 7: Comments with ids, handles, threads and the hidden mode

**Files:**
- Modify: `src/markdown/from_docx/agent.rs` (`Threads`, `comment_head`), `src/markdown/from_docx/mod.rs` (`convert`, `note`, `comment_range`, `run_child`)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
const COMMENTS_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
const EXTENDED_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml";
const EXTENDED_REL: &str = "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";
const W14: &str = r#"xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
const W15: &str = r#"xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml""#;

fn comment(id: u32, author: &str, initials: &str, date: &str, para_id: &str, text: &str) -> String {
    format!(r#"<w:comment w:id="{id}" w:author="{author}" w:initials="{initials}" w:date="{date}"><w:p w14:paraId="{para_id}"><w:r><w:annotationRef/></w:r><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#)
}

fn reference(id: u32) -> String {
    format!(r#"<w:r><w:commentReference w:id="{id}"/></w:r>"#)
}

fn commented_docx(extended: &str) -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}{}{}</w:comments>"#,
        comment(5, "Ann Counsel", "AC", "2026-10-01T09:00:00Z", "11A5D0F2", "Cap in Delaware?"),
        comment(6, "Arthur Souza Rodrigues", "AS", "2026-10-09T16:13:00Z", "33767091", "Disagree."),
        comment(11, "Ann Counsel", "AC", "2026-10-01T09:00:00Z", "214DA01E", "Add survival?")
    );
    let extended_xml = format!(r#"<w15:commentsEx xmlns:w="{W_NS}" {W15}>{extended}</w15:commentsEx>"#);
    let body_xml = format!(
        r#"<w:p>{}<w:commentRangeStart w:id="5"/><w:commentRangeStart w:id="6"/>{}<w:commentRangeEnd w:id="5"/>{}<w:commentRangeEnd w:id="6"/>{}</w:p><w:p>{}<w:commentRangeStart w:id="11"/><w:commentRangeEnd w:id="11"/>{}{}</w:p>"#,
        run("Fee. "), run("Late amounts accrue interest."), reference(5), reference(6),
        run("Keep it secret"), reference(11), run(".")
    );
    common::docx::docx_with(
        &body_xml,
        &[
            Part { name: "word/comments.xml", content_type: COMMENTS_CT, rel_type: COMMENTS_REL, xml: &comments },
            Part { name: "word/commentsExtended.xml", content_type: EXTENDED_CT, rel_type: EXTENDED_REL, xml: &extended_xml },
        ],
    )
}

const THREADED: &str = r#"<w15:commentEx w15:paraId="11A5D0F2" w15:done="0"/><w15:commentEx w15:paraId="33767091" w15:paraIdParent="11A5D0F2" w15:done="0"/><w15:commentEx w15:paraId="214DA01E" w15:done="0"/>"#;

#[test]
fn comments_carry_ids_handles_and_thread_parents() {
    assert_eq!(
        body(&agent(&commented_docx(THREADED))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nFee. {==Late amounts accrue interest.==}{>>#c5 @AC: Cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}\n\n<!-- p1 -->\nKeep it secret{>>#c11 @AC: Add survival?<<}.\n"
    );
}

#[test]
fn two_independent_comments_on_one_anchor_are_not_a_thread() {
    let independent = THREADED.replace(r#" w15:paraIdParent="11A5D0F2""#, "");
    let out = body(&agent(&commented_docx(&independent))).to_string();
    assert!(out.contains("{>>#c5 @AC: Cap in Delaware?<<}{>>#c6 @AS: Disagree.<<}"), "{out}");
}

#[test]
fn a_resolved_thread_says_so_on_its_root() {
    let resolved = THREADED.replacen(r#"w15:paraId="11A5D0F2" w15:done="0""#, r#"w15:paraId="11A5D0F2" w15:done="1""#, 1);
    let out = body(&agent(&commented_docx(&resolved))).to_string();
    assert!(out.contains("{>>#c5 @AC resolved: Cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}"), "{out}");
}

#[test]
fn hidden_comments_are_listed_on_the_id_line() {
    assert_eq!(
        body(&agent_with(&commented_docx(THREADED), TrackChanges::All, false)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 comments #c5 #c6 -->\nFee. Late amounts accrue interest.\n\n<!-- p1 comments #c11 -->\nKeep it secret.\n"
    );
}
```

`Part`'s `rel_type` is added to `word/_rels/document.xml.rels` by `docx_with` (`tests/common/docx.rs:95-104`), which is how `convert` finds `word/comments.xml` and `word/commentsExtended.xml` (`rels.first_of_type("/comments")`, `first_of_type("/commentsExtended")`).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view comments_carry two_independent a_resolved hidden_comments`
Expected: FAIL; the notes read `Ann Counsel (2026-10-01T09:00:00Z): …`.

- [ ] **Step 3: Threads and heads in `agent.rs`**

```rust
/// Comment threads from `commentsExtended.xml`.
#[derive(Debug, Default, Clone)]
pub(crate) struct Threads {
    /// Reply id → root id (`w15:paraIdParent`; Word threads are flat).
    pub reply_of: HashMap<String, String>,
    /// Ids marked `w15:done="1"`.
    pub done: HashSet<String>,
}

/// `commentEx` names a comment by the `w14:paraId` of its last paragraph.
pub(crate) fn threads(comments: Option<&Element>, extended: Option<&Element>) -> Threads {
    let mut t = Threads::default();
    let (Some(comments), Some(extended)) = (comments, extended) else {
        return t;
    };
    let by_para: HashMap<String, String> = comments
        .children_named("comment")
        .filter_map(|c| {
            let id = c.attr("id")?;
            let para = c.children_named("p").last()?.attr("paraId")?;
            Some((para.to_string(), id.to_string()))
        })
        .collect();
    for ex in extended.children_named("commentEx") {
        let Some(id) = ex.attr("paraId").and_then(|p| by_para.get(p)) else {
            continue;
        };
        if ex.attr("done").is_some_and(|d| d == "1" || d == "true") {
            t.done.insert(id.clone());
        }
        if let Some(parent) = ex.attr("paraIdParent").and_then(|p| by_para.get(p)) {
            t.reply_of.insert(id.clone(), parent.clone());
        }
    }
    t
}

/// `#c5 @AC: `, `#c5 @AC resolved: `, `#c6 @AS re #c5: `, or `#c5: ` with
/// no author. `date` is the inline timestamp, when wanted.
pub(crate) fn comment_head(
    id: &str,
    author: Option<&str>,
    date: Option<&str>,
    handles: &Handles,
    threads: &Threads,
) -> String {
    let mut head = format!("#c{id}");
    if let Some(handle) = handles.of(author) {
        head.push_str(&format!(" @{handle}"));
    }
    if let Some(date) = date {
        head.push(' ');
        head.push_str(date);
    }
    if let Some(root) = threads.reply_of.get(id) {
        head.push_str(&format!(" re #c{root}"));
    } else if threads.done.contains(id) {
        head.push_str(" resolved");
    }
    head.push_str(": ");
    head
}
```

- [ ] **Step 4: Wire the Writer**

Add a field `threads: agent::Threads` to `Writer`. In `convert`, after `comments_root`:

```rust
    let extended = if options.ids {
        package
            .xml(&part("/commentsExtended", "word/commentsExtended.xml"))
            .ok()
            .flatten()
    } else {
        None
    };
    let threads = agent::threads(comments_root.as_ref(), extended.as_ref());
```

and `threads: threads.clone()` in the literal.

In `Writer::note` (line 1131), the note's text is built from `comment_note(…)` at the end; replace that line with:

```rust
        let note = if self.agent {
            let date = (self.dates && self.handles.needs_date(comment.attr("author")))
                .then(|| comment.attr("date"))
                .flatten();
            let mut inner = agent::comment_head(id, comment.attr("author"), date, &self.handles, &self.threads);
            inner.push_str(&text);
            inner
        } else {
            comment_note(comment.attr("author"), comment.attr("date"), &text)
        };
```

Replies keep their own notes: the plain conversion already writes one note per comment at its reference, and that is the agent view's form too. Nothing is folded or suppressed.

In `comment_range` (line 653), before the existing `let Some(id) = …`, add:

```rust
        if self.agent {
            if let Some(id) = range.attr("id").filter(|id| self.comments.contains_key(*id)) {
                if range.is("commentRangeStart") && !self.para_comments.iter().any(|c| c == id) {
                    self.para_comments.push(id.to_string());
                }
                if !self.comments_inline {
                    return;
                }
            }
        }
```

In `run_child`, the `"commentReference"` arm (line 987 area):

```rust
            "commentReference" if !hidden && !self.in_comment => {
                if let Some(id) = child.attr("id").filter(|id| self.comments.contains_key(*id)) {
                    if self.agent {
                        if !self.para_comments.iter().any(|c| c == id) {
                            self.para_comments.push(id.to_string());
                        }
                        if !self.comments_inline {
                            return;
                        }
                    }
                    if let Some(note) = self.note(id) {
                        out.comment(&note);
                    }
                }
            }
```

(If the arm's body is an expression inside a `match` rather than a statement block, keep its shape; the behaviour is: record the id, then skip the note when comments are hidden.)

- [ ] **Step 5: Run the tests**

Run: `cargo test --test agent_text_view`
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs tests/agent_text_view.rs
git commit -m "feat(markdown): comment ids, handles, thread parents and hidden mode in the agent view"
```

---

### Task 8: Accept-all and reject-all views keep ids, comments and `rev` tags

**Files:**
- Modify: `src/markdown/from_docx/mod.rs` (`convert`), possibly `src/markdown/from_docx/revise.rs`
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn accept_all_keeps_indices_comments_and_lists_the_revisions_applied() {
    let p0 = format!("<w:p>{}{}{}{}</w:p>", run("Deliver "), ins(0, "Ann Counsel", "quarterly"), run(" reports "), del(1, "Ann Counsel", "weekly"));
    let p1 = format!("<w:p>{}{}{}</w:p>", run("Within "), del(3, "Ann Counsel", "thirty"), ins(4, "Ann Counsel", "forty-five"));
    let bytes = docx(&format!("{p0}{p1}"));
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Accept, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev #0 @AC; #1 @AC -->\nDeliver quarterly reports\n\n<!-- p1 rev #3+4 @AC -->\nWithin forty-five\n"
    );
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Reject, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev #0 @AC; #1 @AC -->\nDeliver  reports weekly\n\n<!-- p1 rev #3+4 @AC -->\nWithin thirty\n"
    );
}

#[test]
fn accept_all_keeps_comments_in_the_agent_view() {
    let out = body(&agent_with(&commented_docx(THREADED), TrackChanges::Accept, true)).to_string();
    assert!(out.contains("{>>#c5 @AC: Cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}"), "{out}");
}
```

The trailing space after `reports` in the accepted p0 is trimmed by `Blocks::push_paragraph` (`body.trim_end()`). The two spaces in `Deliver  reports` of the rejected view are what the file holds once the insertion is gone; the view does not tidy them. In p0 the insertion and the deletion are not adjacent (a run sits between), so they stay two tags, joined with `; ` on the id line.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view accept_all`
Expected: FAIL. Likely causes, in order: no `rev` clause (stamps run after resolution), comments dropped in accept mode, or comment markers lost by `revise::resolve`.

- [ ] **Step 3: Make it pass**

In `convert`, the order must be: load `document` → `handles` → `stamp` → `original = document.clone()` → `revise::resolve`. Verify the Task 3 block sits above `let document = match accept { … }`; if not, move it. `resolved: accept.is_some()` is already on the Writer, and `id_line` prints `rev` from the stamped `REVS` attribute, which survives `resolve_in` (it edits children, not the paragraph's attributes). Comments: Task 3 changed the filter to `accept.is_none() || options.ids`.

If `accept_all_keeps_comments_in_the_agent_view` still fails, `revise::resolve` is dropping `commentRangeStart`/`commentRangeEnd`/`commentReference`; open `revise.rs:58-170` and make `resolve_in` keep those three elements as it keeps bookmarks. Add a unit test there mirroring the existing ones.

- [ ] **Step 4: Run the tests and the whole suite**

Run: `cargo test --test agent_text_view` then `cargo test`
Expected: PASS everywhere.

- [ ] **Step 5: Commit**

```bash
git add src/markdown/from_docx tests/agent_text_view.rs
git commit -m "feat(markdown): accept-all and reject-all agent views keep ids and comments"
```

---

### Task 9: Underline

**Files:**
- Modify: `src/markdown/from_docx/mod.rs` (`run` at 869, `run_child` at 927), `critic.rs` (`Leaf::Text`, `Piece`, `render`, `Critic::push`), `ooxml.rs` (`Span`, `Inline::push`, `render_emphasis` at 555)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn underline_renders_inside_bold_in_the_agent_view_only() {
    let p = r#"<w:p><w:r><w:t xml:space="preserve">keep it </w:t></w:r><w:r><w:rPr><w:b/><w:u w:val="single"/></w:rPr><w:t>secret</w:t></w:r><w:r><w:rPr><w:u w:val="none"/></w:rPr><w:t>.</w:t></w:r></w:p>"#;
    assert_eq!(body(&agent(&docx(p))), "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nkeep it **<u>secret</u>**.\n");
    let legacy = docx_to_markdown(&docx(p), &MarkdownOptions::default()).unwrap().markdown;
    assert_eq!(legacy, "keep it **secret**.\n");
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test --test agent_text_view underline`
Expected: FAIL, `**secret**` without `<u>`.

- [ ] **Step 3: Thread `underline` next to `bold` and `italic`**

Every place that carries `bold, italic` as a pair carries a third flag, `underline`, in the same order (`grep -n "bold" src/markdown/from_docx/{mod.rs,critic.rs,ooxml.rs}` lists them; expect about twenty sites):

- `mod.rs` `run()`: compute `let underline = self.agent && rpr.and_then(|r| r.child("u")).is_some_and(|u| u.attr("val").is_none_or(|v| v != "none"));` right after `italic`; pass `(bold, italic, underline)` to `run_child`, which passes it to every `out.push(…)`.
- `critic.rs`: `Critic::push(&mut self, text, bold, italic, underline, link)`; `Leaf::Text { text, bold, italic, underline, link }`; `Piece { underline }`; `render` copies it into the `Piece`; markers get `underline: false`.
- `ooxml.rs`: `Span { underline }`; `Inline::push(text, bold, italic, underline, link)` merges on `(bold, italic, underline)` equality; `render_emphasis` merges on the triple and wraps `<u>…</u>` innermost: `**<u>secret</u>**`, `*<u>x</u>*`, `***<u>x</u>***`. The `emphasis == false` path (headings) drops all three.
- `Critic::into_inline` carries the flag from `Piece` to `Inline::push`.

Only `run()` ever sets `underline = true`, and only when `self.agent`, so the plain conversion cannot change.

- [ ] **Step 4: Run the test, the module tests and the round-trip tests**

Run: `cargo test --test agent_text_view underline && cargo test --lib markdown && cargo test --test docx_markdown_round_trip --test docx_to_markdown_office_samples`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/markdown/from_docx
git commit -m "feat(markdown): underline as <u> in the agent view"
```

---

### Task 10: The header, part 1 (counts, authors, owner, body)

**Files:**
- Create: `src/markdown/from_docx/header.rs`
- Modify: `src/markdown/from_docx/mod.rs` (module list, `convert`), `src/markdown/from_docx/agent.rs` (collectors)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
const CORE_CT: &str = "application/vnd.openxmlformats-package.core-properties+xml";

fn core(xml: &str) -> Part<'_> {
    Part { name: "docProps/core.xml", content_type: CORE_CT, rel_type: "", xml }
}

#[test]
fn header_counts_revisions_comments_and_authors() {
    let p = format!("<w:p>{}{}{}{}</w:p>", run("a "), del(1, "Ann Counsel", "b"), ins(2, "Ann Counsel", "c"), ins(3, "John Doe", "d"));
    let bytes = common::docx::docx_with(&p, &[core(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:creator>Jane Owner</dc:creator><cp:lastModifiedBy>Someone Else</cp:lastModifiedBy></cp:coreProperties>"#)]);
    let out = agent(&bytes);
    let lines = header_lines(&out);
    assert_eq!(lines[0], "source: sample.docx");
    assert_eq!(lines[1], "view: tracked                      # revisions as CriticMarkup, comments inline");
    assert_eq!(lines[2], "track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it");
    assert_eq!(lines[3], "revisions: 2                       # 1 insertion, 1 substitution (3 Word marks)");
    assert_eq!(lines[4], "comments: 0");
    assert_eq!(lines[5], "authors:");
    assert_eq!(lines[6], "  document_owner: Jane Owner       # dc:creator");
    assert_eq!(lines[7], "  AC: Ann Counsel                  # 1 revision, 2026-10-01T09:00:00Z");
    assert_eq!(lines[8], "  JD: John Doe                     # 1 revision, 2026-10-01T09:00:00Z");
    assert_eq!(lines[9], "body: p0-p0, 0 tables, 1 page      # page count estimated from breaks");
}

#[test]
fn header_without_core_properties_names_no_owner_and_reads_tracking_from_settings() {
    let settings = Part {
        name: "word/settings.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
        xml: &format!(r#"<w:settings xmlns:w="{W_NS}"><w:trackRevisions/></w:settings>"#),
    };
    let out = agent(&common::docx::docx_with(&para("x"), &[settings]));
    let lines = header_lines(&out);
    assert_eq!(lines[2], "track_changes: on                  # w:trackRevisions set; edits are tracked");
    assert_eq!(lines[3], "revisions: 0");
    assert_eq!(lines[5], "authors:");
    assert_eq!(lines[6], "  document_owner: none             # no docProps/core.xml");
}

#[test]
fn header_view_lines_for_hidden_comments_accept_and_reject() {
    let bytes = commented_docx(THREADED);
    assert_eq!(header_lines(&agent_with(&bytes, TrackChanges::All, false))[1], "view: tracked, comments hidden     # 2 threads open (3 comments); carrying paragraphs are marked");
    let p = format!("<w:p>{}{}</w:p>", run("a "), ins(0, "Ann Counsel", "b"));
    assert_eq!(header_lines(&agent_with(&docx(&p), TrackChanges::Accept, true))[1], "view: accept-all                   # 1 revision by AC shown as accepted; file unchanged");
    assert_eq!(header_lines(&agent_with(&docx(&p), TrackChanges::Reject, true))[1], "view: reject-all                   # 1 revision by AC shown as rejected; file unchanged");
    assert_eq!(header_lines(&agent(&bytes))[4], "comments: 2 threads open           # 3 comments: c5 (+ reply c6), c11");
}

#[test]
fn header_prints_a_day_range_for_an_author_with_several_timestamps() {
    let p = format!("<w:p>{}{}{}{}</w:p>", run("a "), ins_at(1, "Ann Counsel", "2026-10-01T09:00:00Z", "b"), run(" c "), ins_at(2, "Ann Counsel", "2026-10-03T14:05:00Z", "d"));
    let lines = header_lines(&agent(&docx(&p)));
    assert_eq!(lines[6], "  AC: Ann Counsel                  # 2 revisions, 2026-10-01..2026-10-03");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view header_`
Expected: FAIL (no header yet).

- [ ] **Step 3: `header.rs`, part 1**

Create `src/markdown/from_docx/header.rs`:

```rust
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The YAML header of the agent view.

use std::collections::HashSet;

use super::agent::{Handles, RevTag};
use super::ooxml::Element;

/// Where the owner's name came from.
pub(crate) enum Owner {
    Creator(String),
    LastModifiedBy(String),
    None,
}

/// Where the page count and markers came from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PagesSource {
    /// `Options::pages` from the layout pass.
    Layout,
    /// `w:lastRenderedPageBreak` in the file.
    Cached,
    /// Hard page and section breaks only.
    Estimated,
}

/// A header or footer of one section.
#[derive(Clone)]
pub(crate) struct StoryFact {
    /// `header` or `footer`.
    pub kind: &'static str,
    /// `first`, `default`, `even`.
    pub ty: &'static str,
    /// `h1` (`header2.xml`), `f0`, `h1.p2`.
    pub id: String,
    pub text: String,
    /// `center`, `right`, or `None` for left.
    pub align: Option<&'static str>,
    pub active: bool,
}

/// A section of the body (Task 11 fills these).
pub(crate) struct SectionFact {
    /// Paragraph range, inclusive.
    pub first: usize,
    pub last: usize,
    pub sect_pr: Option<Element>,
    pub stories: Vec<StoryFact>,
    pub title_page: bool,
    pub columns: usize,
}

pub(crate) struct CommentFact {
    pub id: String,
    pub author: Option<String>,
    pub date: Option<String>,
    pub parent: Option<String>,
}

pub(crate) struct Facts<'a> {
    pub source: &'a str,
    /// `None` tracked, `Some(true)` accept-all, `Some(false)` reject-all.
    pub resolved: Option<bool>,
    pub comments_inline: bool,
    pub tracking_on: bool,
    pub tags: Vec<RevTag>,
    pub marks: usize,
    pub format_changes: usize,
    pub comments: Vec<CommentFact>,
    pub handles: &'a Handles,
    /// Resolved comment ids (`w15:done`), from `agent::Threads`.
    pub done: &'a HashSet<String>,
    pub owner: Owner,
    pub paragraphs: usize,
    pub tables: usize,
    pub pages: usize,
    pub pages_source: PagesSource,
    /// `range:` line after `body:` when a selection is active (Task 12).
    pub range: Option<String>,
    pub styles: Option<&'a Element>,
    pub theme: Option<&'a Element>,
    pub default_style: Option<&'a str>,
    /// (level, style id) for every heading level used, lowest first.
    pub heading_styles: Vec<(usize, String)>,
    pub table_styles: Vec<String>,
    /// The body's sections in order; the first one is described in full.
    pub sections: Vec<SectionFact>,
}

/// `key: value` padded to 35 columns then `# comment`; a longer key/value
/// takes two spaces.
fn kv(out: &mut String, key_value: &str, comment: Option<&str>) {
    match comment {
        Some(c) if key_value.len() <= 33 => out.push_str(&format!("{key_value:<35}# {c}\n")),
        Some(c) => out.push_str(&format!("{key_value}  # {c}\n")),
        None => {
            out.push_str(key_value);
            out.push('\n');
        }
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 { format!("1 {one}") } else { format!("{n} {many}") }
}

fn join(items: &[String]) -> String {
    items.join(", ")
}

pub(crate) fn render(f: &Facts) -> String {
    let mut out = String::from("---\n");
    kv(&mut out, &format!("source: {}", f.source), None);
    let handle_list: Vec<String> = f
        .handles
        .order
        .iter()
        .filter(|a| f.tags.iter().any(|t| t.author.as_deref() == Some(a.as_str())))
        .filter_map(|a| f.handles.by_author.get(a).cloned())
        .collect();
    let by = if handle_list.is_empty() { String::new() } else { format!(" by {}", join(&handle_list)) };
    let threads: Vec<&CommentFact> = f.comments.iter().filter(|c| c.parent.is_none()).collect();
    match (f.resolved, f.comments_inline) {
        (None, true) => kv(&mut out, "view: tracked", Some("revisions as CriticMarkup, comments inline")),
        (None, false) => kv(
            &mut out,
            "view: tracked, comments hidden",
            Some(&format!(
                "{} ({}); carrying paragraphs are marked",
                plural(threads.len(), "thread open", "threads open"),
                plural(f.comments.len(), "comment", "comments")
            )),
        ),
        (Some(accept), _) => kv(
            &mut out,
            &format!("view: {}", if accept { "accept-all" } else { "reject-all" }),
            Some(&format!(
                "{}{by} shown as {}; file unchanged",
                plural(f.tags.len(), "revision", "revisions"),
                if accept { "accepted" } else { "rejected" }
            )),
        ),
    }
    if f.tracking_on {
        kv(&mut out, "track_changes: on", Some("w:trackRevisions set; edits are tracked"));
    } else {
        kv(&mut out, "track_changes: off", Some("w:trackRevisions not set; new edits are not tracked unless edit sets it"));
    }
    if f.tags.is_empty() {
        kv(&mut out, "revisions: 0", None);
    } else {
        let count = |kind: &str| f.tags.iter().filter(|t| t.kind == kind).count();
        let mut parts = Vec::new();
        for (kind, one, many) in [
            ("ins", "insertion", "insertions"),
            ("del", "deletion", "deletions"),
            ("sub", "substitution", "substitutions"),
            ("mark", "paragraph mark", "paragraph marks"),
            ("row", "table row", "table rows"),
            ("cell", "table cell", "table cells"),
        ] {
            let n = count(kind);
            if n > 0 {
                parts.push(plural(n, one, many));
            }
        }
        let mut marks = plural(f.marks, "Word mark", "Word marks");
        if f.format_changes > 0 {
            marks.push_str(&format!(", {} formatting", f.format_changes));
        }
        kv(&mut out, &format!("revisions: {}", f.tags.len()), Some(&format!("{} ({marks})", join(&parts))));
    }
    if f.comments.is_empty() {
        kv(&mut out, "comments: 0", None);
    } else {
        let open = threads.iter().filter(|c| !f.done.contains(&c.id)).count();
        let mut key = format!("comments: {}", plural(open, "thread open", "threads open"));
        if open < threads.len() {
            key.push_str(&format!(", {} resolved", threads.len() - open));
        }
        let list: Vec<String> = threads
            .iter()
            .map(|t| {
                let replies: Vec<String> = f.comments.iter().filter(|c| c.parent.as_deref() == Some(&t.id)).map(|c| format!("c{}", c.id)).collect();
                match replies.len() {
                    0 => format!("c{}", t.id),
                    1 => format!("c{} (+ reply {})", t.id, replies[0]),
                    _ => format!("c{} (+ replies {})", t.id, join(&replies)),
                }
            })
            .collect();
        kv(&mut out, &key, Some(&format!("{}: {}", plural(f.comments.len(), "comment", "comments"), join(&list))));
    }
    out.push_str("authors:\n");
    match &f.owner {
        Owner::Creator(name) => kv(&mut out, &format!("  document_owner: {name}"), Some("dc:creator")),
        Owner::LastModifiedBy(name) => kv(&mut out, &format!("  document_owner: {name}"), Some("cp:lastModifiedBy; no dc:creator")),
        Owner::None => kv(&mut out, "  document_owner: none", Some("no docProps/core.xml")),
    }
    for author in &f.handles.order {
        let Some(handle) = f.handles.by_author.get(author) else { continue };
        let revisions = f.tags.iter().filter(|t| t.author.as_deref() == Some(author.as_str())).count();
        let comments = f.comments.iter().filter(|c| c.author.as_deref() == Some(author.as_str())).count();
        let mut parts = Vec::new();
        if revisions > 0 {
            parts.push(plural(revisions, "revision", "revisions"));
        }
        if comments > 0 {
            parts.push(plural(comments, "comment", "comments"));
        }
        if let Some(date) = f.handles.unique_date(author) {
            parts.push(date.to_string());
        } else if let Some(range) = f.handles.date_range(author) {
            parts.push(range);
        }
        kv(&mut out, &format!("  {handle}: {author}"), Some(&join(&parts)));
    }
    let last = f.paragraphs.saturating_sub(1);
    kv(
        &mut out,
        &format!("body: p0-p{last}, {}, {}", plural(f.tables, "table", "tables"), plural(f.pages, "page", "pages")),
        Some(match f.pages_source {
            PagesSource::Layout => "pages from layout",
            PagesSource::Cached => "pages from Word's cached layout",
            PagesSource::Estimated => "page count estimated from breaks",
        }),
    );
    if let Some(range) = &f.range {
        kv(&mut out, &format!("range: {range} of p0-p{last}"), None);
    }
    part_two(&mut out, f);
    out.push_str("---\n");
    out
}

/// Page setup, styles, headers, footers and sections (Task 11).
fn part_two(_out: &mut String, _f: &Facts) {}
```

- [ ] **Step 4: Collectors in `agent.rs` and the facts block in `convert`**

Add `mod header;` to the module list. In `agent.rs` add:

```rust
/// Every logical revision in the stamped body: inline tags per paragraph,
/// tracked paragraph marks, tracked rows and cells.
pub(crate) fn collect_revisions(body: &Element, handles: &Handles) -> Vec<RevTag> {
    let mut out = Vec::new();
    collect_in(body, handles, &mut out);
    out
}

fn collect_in(e: &Element, handles: &Handles, out: &mut Vec<RevTag>) {
    if e.is("txbxContent") {
        return;
    }
    let attribution = |m: &Element| (m.attr("author").map(str::to_string), m.attr("date").map(str::to_string));
    if e.is("p") {
        out.extend(revision_tags(e, handles));
        if let Some(rpr) = e.path(&["pPr", "rPr"]) {
            for m in rpr.elements().filter(|m| kind_of(m.local()).is_some()) {
                let (author, date) = attribution(m);
                out.push(RevTag { kind: "mark", tag: tag_of(m, handles), author, date });
            }
        }
    }
    if e.is("tr") {
        if let Some(trpr) = e.child("trPr") {
            for m in trpr.elements().filter(|m| kind_of(m.local()).is_some()) {
                let (author, date) = attribution(m);
                out.push(RevTag { kind: "row", tag: tag_of(m, handles), author, date });
            }
        }
    }
    if e.is("tc") {
        if let Some(tcpr) = e.child("tcPr") {
            for m in tcpr.elements().filter(|m| matches!(m.local(), "cellIns" | "cellDel")) {
                let (author, date) = attribution(m);
                out.push(RevTag { kind: "cell", tag: tag_of(m, handles), author, date });
            }
        }
    }
    for child in e.elements() {
        collect_in(child, handles, out);
    }
}

/// Count of revision elements (`w:ins`, `w:del`, moves, cell marks) and of
/// formatting changes (`*PrChange`) under `body`, text boxes excluded.
pub(crate) fn count_marks(e: &Element) -> (usize, usize) {
    if e.is("txbxContent") {
        return (0, 0);
    }
    let mut marks = usize::from(is_revision(e.local()));
    let mut formats = usize::from(e.local().ends_with("PrChange"));
    for child in e.elements() {
        let (m, f) = count_marks(child);
        marks += m;
        formats += f;
    }
    (marks, formats)
}
```

In `convert`, after the markdown is assembled and paginated (end of Task 3's block), prepend the header:

```rust
    if let Some(original) = &original {
        let body = original.child("body").unwrap_or(original);
        let tags = agent::collect_revisions(body, &handles);
        let (marks, format_changes) = agent::count_marks(body);
        let (rendered, hard) = agent::page_counts(body);
        let settings = package.xml("word/settings.xml").ok().flatten();
        let core = package.xml("docProps/core.xml").ok().flatten();
        let owner = match &core {
            None => header::Owner::None,
            Some(core) => match (
                core.child("creator").map(Element::text).filter(|t| !t.trim().is_empty()),
                core.child("lastModifiedBy").map(Element::text).filter(|t| !t.trim().is_empty()),
            ) {
                (Some(name), _) => header::Owner::Creator(name.trim().to_string()),
                (None, Some(name)) => header::Owner::LastModifiedBy(name.trim().to_string()),
                (None, None) => header::Owner::None,
            },
        };
        let comment_facts: Vec<header::CommentFact> = comments_root
            .as_ref()
            .map(|root| {
                root.children_named("comment")
                    .filter_map(|c| {
                        let id = c.attr("id")?.to_string();
                        Some(header::CommentFact {
                            parent: threads.reply_of.get(&id).cloned(),
                            author: c.attr("author").map(str::to_string),
                            date: c.attr("date").map(str::to_string),
                            id,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let (pages, pages_source) = match (&options.pages, rendered) {
            (Some(pages), _) => (pages.len().max(1), header::PagesSource::Layout),
            (None, n) if n > 0 => (1 + n, header::PagesSource::Cached),
            (None, _) => (1 + hard, header::PagesSource::Estimated),
        };
        let facts = header::Facts {
            source: options.source.as_deref().unwrap_or("(bytes)"),
            resolved: accept,
            comments_inline: options.comments,
            tracking_on: settings.as_ref().is_some_and(|s| s.child("trackRevisions").is_some()),
            tags,
            marks,
            format_changes,
            comments: comment_facts,
            handles: &handles,
            done: &threads.done,
            owner,
            paragraphs: stamped.0,
            tables: stamped.1,
            pages,
            pages_source,
            range: None,
            styles: styles_root.as_ref(),
            theme: None,
            default_style: default_style.as_deref(),
            heading_styles: Vec::new(),
            table_styles: Vec::new(),
            sections: Vec::new(),
        };
        markdown = format!("{}\n{markdown}", header::render(&facts));
    }
```

`Element::text()` exists (`ooxml.rs:307`). `handles`, `threads`, `stamped`, `styles_root`, `default_style` and `comments_root` were declared before the writer in Task 3 and remain in scope; the writer holds clones.

- [ ] **Step 5: Run the tests**

Run: `cargo test --test agent_text_view header_ layout_page`
Expected: PASS, including the `body:` assertion deferred from Task 3.

- [ ] **Step 6: Commit**

```bash
git add src/markdown/from_docx tests/agent_text_view.rs
git commit -m "feat(markdown): agent view header: counts, authors, owner, body"
```

---

### Task 11: The header, part 2 (page, styles, headers, footers, sections)

**Files:**
- Modify: `src/markdown/from_docx/header.rs` (`part_two` and helpers), `src/markdown/from_docx/mod.rs` (`convert`: theme, heading styles, table styles, sections)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const FOOTER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const HEADER_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const FOOTER_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";

fn hdr(text: &str, jc: Option<&str>) -> String {
    let ppr = jc.map_or(String::new(), |j| format!(r#"<w:pPr><w:jc w:val="{j}"/></w:pPr>"#));
    format!(r#"<w:hdr xmlns:w="{W_NS}"><w:p>{ppr}<w:r><w:t>{text}</w:t></w:r></w:p></w:hdr>"#)
}

#[test]
fn header_describes_page_setup_styles_headers_and_footers() {
    let styles = format!(r#"<w:styles xmlns:w="{W_NS}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style><w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/><w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/></w:tblBorders></w:tblPr></w:style></w:styles>"#);
    let header1 = hdr("SIGNATURE PAGE", Some("right"));
    let header2 = format!(r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>DRAFT</w:t></w:r></w:p><w:p/></w:hdr>"#);
    let footer1 = format!(r#"<w:ftr xmlns:w="{W_NS}"><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p></w:ftr>"#);
    let footer2 = format!(r#"<w:ftr xmlns:w="{W_NS}"><w:p><w:fldSimple w:instr=" PAGE "><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p></w:ftr>"#);
    let body_xml = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>{}<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/></w:tblPr><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        para("x")
    );
    // Part relationship ids are rIdX0.. in order (tests/common/docx.rs:101).
    let sect = r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX1"/><w:footerReference w:type="even" r:id="rIdX3"/><w:footerReference w:type="default" r:id="rIdX4"/><w:headerReference w:type="first" r:id="rIdX2"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/><w:titlePg/></w:sectPr>"#;
    let bytes = common::docx::docx_with_sect_pr(
        &body_xml,
        &[
            Part { name: "word/styles.xml", content_type: STYLES_CT, rel_type: STYLES_REL, xml: &styles },
            Part { name: "word/header1.xml", content_type: HEADER_CT, rel_type: HEADER_REL, xml: &header1 },
            Part { name: "word/header2.xml", content_type: HEADER_CT, rel_type: HEADER_REL, xml: &header2 },
            Part { name: "word/footer1.xml", content_type: FOOTER_CT, rel_type: FOOTER_REL, xml: &footer1 },
            Part { name: "word/footer2.xml", content_type: FOOTER_CT, rel_type: FOOTER_REL, xml: &footer2 },
        ],
        sect,
    );
    let out = agent(&bytes);
    let header = out.splitn(3, "---\n").nth(1).unwrap();
    let expected = "\
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  \"#\": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next
  table: TableGrid, all borders 0.5pt
headers:
  first: {id: h1, text: DRAFT}     # page 1 only (different first page)
  default: {id: h0, text: SIGNATURE PAGE, right}
footers:
  first: none                      # page 1 shows no page number
  default: {id: f1, text: \"{PAGE}\"}
  even: {id: f0, text: \"{PAGE}\", inactive}  # defined, but even/odd headers are off
";
    assert!(header.ends_with(expected), "header:\n{header}");
}

#[test]
fn header_page_line_for_a4_landscape_with_uneven_margins_and_a_gutter() {
    let sect = r#"<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="709" w:footer="709" w:gutter="720"/></w:sectPr>"#;
    let out = agent(&common::docx::docx_with_sect_pr(&para("x"), &[], sect));
    assert!(out.contains("\npage: A4 landscape, margins top 1in, right 1.25in, bottom 1in, left 1.25in, header/footer 0.49in, gutter 0.5in\n"), "{out}");
}

#[test]
fn later_sections_list_their_range_and_what_differs_from_the_first() {
    let main = hdr("MAIN", None);
    let schedule = hdr("SCHEDULE A", None);
    let margins = r#"<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>"#;
    let body_xml = format!(
        r#"{}<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/>{margins}</w:sectPr></w:pPr><w:r><w:t>End of part one</w:t></w:r></w:p>{}{}"#,
        para("Intro"),
        para("Schedule"),
        para("Rows")
    );
    let sect = format!(r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX1"/><w:cols w:num="2"/>{margins}</w:sectPr>"#);
    let bytes = common::docx::docx_with_sect_pr(
        &body_xml,
        &[
            Part { name: "word/header1.xml", content_type: HEADER_CT, rel_type: HEADER_REL, xml: &main },
            Part { name: "word/header2.xml", content_type: HEADER_CT, rel_type: HEADER_REL, xml: &schedule },
        ],
        &sect,
    );
    let out = agent(&bytes);
    let header = out.splitn(3, "---\n").nth(1).unwrap();
    assert!(header.ends_with("headers:\n  default: {id: h0, text: MAIN}\nsections:\n  2: {p2-p3, headers: {default: {id: h1, text: SCHEDULE A}}, columns: 2}\n"), "header:\n{header}");
    assert!(body(&out).contains("<!-- p1 section-break -->\nEnd of part one\n"), "{out}");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view header_describes header_page_line later_sections`
Expected: FAIL (no `page:` line).

- [ ] **Step 3: Implement `part_two` and its helpers**

Replace the stub in `header.rs`:

```rust
fn twips(e: &Element, attr: &str) -> Option<f64> {
    e.attr(attr).and_then(|v| v.parse::<f64>().ok())
}

/// `Letter portrait, margins 1in, header/footer 0.5in` for a `w:sectPr`.
fn page_setup(sect: &Element) -> String {
    let size = sect.child("pgSz");
    let (w, h) = (size.and_then(|s| twips(s, "w")).unwrap_or(12240.0), size.and_then(|s| twips(s, "h")).unwrap_or(15840.0));
    let landscape = size.and_then(|s| s.attr("orient")) == Some("landscape") || w > h;
    let (short, long) = if w < h { (w, h) } else { (h, w) };
    let name = match (short as i64, long as i64) {
        (12240, 15840) => "Letter".to_string(),
        (12240, 20160) => "Legal".to_string(),
        (11906, 16838) => "A4".to_string(),
        _ => format!("{}x{}", super::agent::inches(w).trim_end_matches("in"), super::agent::inches(h)),
    };
    let mut out = format!("{name} {}", if landscape { "landscape" } else { "portrait" });
    if let Some(m) = sect.child("pgMar") {
        let side = |a: &str| twips(m, a).unwrap_or(1440.0);
        let (t, r, b, l) = (side("top"), side("right"), side("bottom"), side("left"));
        let inch = super::agent::inches;
        if t == r && r == b && b == l {
            out.push_str(&format!(", margins {}", inch(t)));
        } else {
            out.push_str(&format!(", margins top {}, right {}, bottom {}, left {}", inch(t), inch(r), inch(b), inch(l)));
        }
        let (hd, ft) = (twips(m, "header").unwrap_or(720.0), twips(m, "footer").unwrap_or(720.0));
        if hd == ft {
            out.push_str(&format!(", header/footer {}", inch(hd)));
        } else {
            out.push_str(&format!(", header {}, footer {}", inch(hd), inch(ft)));
        }
        if let Some(g) = twips(m, "gutter").filter(|&g| g > 0.0) {
            out.push_str(&format!(", gutter {}", inch(g)));
        }
    }
    out
}

/// Resolved paragraph/run properties of a style through its `basedOn`
/// chain and the document defaults.
#[derive(Default)]
struct Resolved {
    font: Option<String>,
    size_half_points: Option<f64>,
    bold: bool,
    italic: bool,
    before: Option<f64>,
    after: Option<f64>,
    line: Option<(f64, String)>,
    keep_next: bool,
    align: Option<String>,
}

fn style_by_id<'a>(styles: &'a Element, id: &str) -> Option<&'a Element> {
    styles.children_named("style").find(|s| s.attr("styleId") == Some(id))
}

fn font_of(rpr: &Element, theme: Option<&Element>) -> Option<String> {
    let fonts = rpr.child("rFonts")?;
    if let Some(name) = fonts.attr("ascii") {
        return Some(name.to_string());
    }
    let which = fonts.attr("asciiTheme").or_else(|| fonts.attr("hAnsiTheme"))?;
    let theme = theme?;
    let mut scheme = Vec::new();
    theme.find_all(if which.starts_with("major") { "majorFont" } else { "minorFont" }, &mut scheme);
    scheme.first()?.child("latin")?.attr("typeface").map(str::to_string)
}

fn apply(r: &mut Resolved, ppr: Option<&Element>, rpr: Option<&Element>, theme: Option<&Element>) {
    if let Some(rpr) = rpr {
        if r.font.is_none() {
            r.font = font_of(rpr, theme);
        }
        if r.size_half_points.is_none() {
            r.size_half_points = rpr.child("sz").and_then(|s| twips(s, "val"));
        }
        r.bold |= rpr.toggle("b").unwrap_or(false);
        r.italic |= rpr.toggle("i").unwrap_or(false);
    }
    if let Some(ppr) = ppr {
        if let Some(sp) = ppr.child("spacing") {
            if r.before.is_none() {
                r.before = twips(sp, "before");
            }
            if r.after.is_none() {
                r.after = twips(sp, "after");
            }
            if r.line.is_none() {
                if let Some(line) = twips(sp, "line") {
                    r.line = Some((line, sp.attr("lineRule").unwrap_or("auto").to_string()));
                }
            }
        }
        r.keep_next |= ppr.child("keepNext").is_some();
        if r.align.is_none() {
            r.align = ppr.child("jc").and_then(|j| j.attr("val")).map(str::to_string);
        }
    }
}

fn resolve(styles: Option<&Element>, theme: Option<&Element>, id: &str) -> Resolved {
    let mut r = Resolved::default();
    let Some(styles) = styles else { return r };
    let mut current = Some(id.to_string());
    let mut hops = 0;
    while let Some(sid) = current.take() {
        hops += 1;
        if hops > 16 {
            break;
        }
        let Some(style) = style_by_id(styles, &sid) else { break };
        apply(&mut r, style.child("pPr"), style.child("rPr"), theme);
        current = style.child("basedOn").and_then(|b| b.attr("val")).map(str::to_string);
    }
    if let Some(defaults) = styles.child("docDefaults") {
        apply(&mut r, defaults.path(&["pPrDefault", "pPr"]), defaults.path(&["rPrDefault", "rPr"]), theme);
    }
    r
}

fn points(twips: f64) -> String {
    let v = format!("{:.1}", twips / 20.0);
    v.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn align_word(align: Option<&str>) -> &'static str {
    match align {
        Some("center") => "center",
        Some("right" | "end") => "right",
        Some("both" | "distribute") => "justify",
        _ => "left",
    }
}

/// The default style's line prints everything; a heading line always prints
/// its font and size, then only what differs from `base` (the default
/// style), so the legend reads as deviations.
fn describe(r: &Resolved, base: Option<&Resolved>) -> String {
    let differs = |pick: fn(&Resolved) -> String| base.is_none_or(|b| pick(b) != pick(r));
    let mut parts = Vec::new();
    let mut font = r.font.clone().unwrap_or_else(|| "Times New Roman".to_string());
    if r.bold {
        font.push_str(" bold");
    }
    if r.italic {
        font.push_str(" italic");
    }
    let size = r.size_half_points.unwrap_or(20.0) / 2.0;
    let size = format!("{size:.1}");
    parts.push(format!("{font} {}pt", size.trim_end_matches('0').trim_end_matches('.')));
    if differs(|x| format!("{:?}", x.before)) {
        if let Some(b) = r.before.filter(|&b| b > 0.0) {
            parts.push(format!("before {}pt", points(b)));
        }
    }
    if differs(|x| format!("{:?}", x.after)) {
        if let Some(a) = r.after.filter(|&a| a > 0.0) {
            parts.push(format!("after {}pt", points(a)));
        }
    }
    if differs(|x| format!("{:?}", x.line)) {
        if let Some((line, rule)) = &r.line {
            if rule == "auto" {
                let ratio = format!("{:.2}", line / 240.0);
                let ratio = ratio.trim_end_matches('0').trim_end_matches('.');
                if ratio != "1" {
                    parts.push(format!("line {ratio}"));
                }
            } else {
                parts.push(format!("line {}pt {rule}", points(*line)));
            }
        }
    }
    if r.keep_next && differs(|x| x.keep_next.to_string()) {
        parts.push("keep-next".to_string());
    }
    match base {
        None => parts.push(align_word(r.align.as_deref()).to_string()),
        Some(b) if align_word(r.align.as_deref()) != align_word(b.align.as_deref()) => {
            parts.push(align_word(r.align.as_deref()).to_string());
        }
        Some(_) => {}
    }
    parts.join(", ")
}

fn table_style_line(styles: Option<&Element>, id: &str) -> String {
    let borders = styles
        .and_then(|s| style_by_id(s, id))
        .and_then(|s| s.path(&["tblPr", "tblBorders"]));
    let Some(borders) = borders else { return id.to_string() };
    let sides = ["top", "left", "bottom", "right", "insideH", "insideV"];
    let sizes: Vec<Option<(String, String)>> = sides
        .iter()
        .map(|side| borders.child(side).map(|b| (b.attr("val").unwrap_or("").to_string(), b.attr("sz").unwrap_or("").to_string())))
        .collect();
    match sizes.first().cloned().flatten() {
        Some((val, sz)) if val == "single" && sizes.iter().all(|s| s.as_ref() == Some(&(val.clone(), sz.clone()))) => {
            let pt = sz.parse::<f64>().map(|s| s / 8.0).unwrap_or(0.5);
            let pt = format!("{pt:.2}");
            format!("{id}, all borders {}pt", pt.trim_end_matches('0').trim_end_matches('.'))
        }
        _ => format!("{id}, borders vary"),
    }
}

fn entry(s: &StoryFact) -> String {
    let text = if s.text.contains('{') || s.text.contains(':') || s.text.is_empty() {
        format!("\"{}\"", s.text.replace('"', "\\\""))
    } else {
        s.text.clone()
    };
    let mut inner = format!("id: {}, text: {text}", s.id);
    if let Some(align) = s.align {
        inner.push_str(&format!(", {align}"));
    }
    if !s.active {
        inner.push_str(", inactive");
    }
    format!("{{{inner}}}")
}

/// The `headers:` / `footers:` blocks of the first section.
fn first_section_stories(out: &mut String, section: &SectionFact) {
    for kind in ["header", "footer"] {
        let stories: Vec<&StoryFact> = section.stories.iter().filter(|s| s.kind == kind).collect();
        if stories.is_empty() && !section.title_page {
            continue;
        }
        out.push_str(&format!("{kind}s:\n"));
        let find = |ty: &str| stories.iter().find(|s| s.ty == ty);
        if section.title_page {
            match find("first") {
                Some(s) => kv(out, &format!("  first: {}", entry(s)), Some("page 1 only (different first page)")),
                None => kv(out, "  first: none", Some(if kind == "footer" { "page 1 shows no page number" } else { "page 1 shows no header" })),
            }
        }
        match find("default") {
            Some(s) => kv(out, &format!("  default: {}", entry(s)), None),
            None => kv(out, "  default: none", None),
        }
        if let Some(s) = find("even") {
            kv(out, &format!("  even: {}", entry(s)), if s.active { None } else { Some("defined, but even/odd headers are off") });
        }
    }
}

/// `{p2-p3, headers: {default: {…}}, columns: 2}`: what a later section
/// changes against the one before it (page setup against the first).
fn section_entry(section: &SectionFact, previous: &SectionFact, first: &SectionFact) -> String {
    let mut parts = vec![format!("p{}-p{}", section.first, section.last)];
    let setup = |s: &SectionFact| s.sect_pr.as_ref().map(page_setup);
    if setup(section) != setup(first) {
        if let Some(page) = setup(section) {
            parts.push(format!("page: {page}"));
        }
    }
    for kind in ["header", "footer"] {
        let changed: Vec<String> = section
            .stories
            .iter()
            .filter(|s| s.kind == kind)
            .filter(|s| !previous.stories.iter().any(|p| p.kind == s.kind && p.ty == s.ty && p.id == s.id))
            .map(|s| format!("{}: {}", s.ty, entry(s)))
            .collect();
        if !changed.is_empty() {
            parts.push(format!("{kind}s: {{{}}}", changed.join(", ")));
        }
    }
    if section.columns != previous.columns {
        parts.push(format!("columns: {}", section.columns));
    }
    format!("{{{}}}", parts.join(", "))
}

fn part_two(out: &mut String, f: &Facts) {
    if let Some(sect) = f.sections.first().and_then(|s| s.sect_pr.as_ref()) {
        kv(out, &format!("page: {}", page_setup(sect)), None);
    }
    out.push_str("styles:\n");
    let default = f.default_style.unwrap_or("Normal");
    let base = resolve(f.styles, f.theme, default);
    kv(out, &format!("  {default}: {}", describe(&base, None)), Some("default; unannotated paragraphs use it"));
    for (level, style) in &f.heading_styles {
        kv(out, &format!("  \"{}\": {style}, {}", "#".repeat(*level), describe(&resolve(f.styles, f.theme, style), Some(&base))), None);
    }
    match f.table_styles.as_slice() {
        [] => {}
        [one] => kv(out, &format!("  table: {}", table_style_line(f.styles, one)), None),
        many => kv(out, &format!("  tables: {}", many.join(", ")), None),
    }
    if let Some(first) = f.sections.first() {
        first_section_stories(out, first);
        if f.sections.len() > 1 {
            out.push_str("sections:\n");
            for (i, section) in f.sections.iter().enumerate().skip(1) {
                kv(out, &format!("  {}: {}", i + 1, section_entry(section, &f.sections[i - 1], first)), None);
            }
        }
    }
}

/// A header/footer paragraph's text with fields as `{PAGE}`: field codes
/// print, cached results do not.
pub(crate) fn story_text(p: &Element) -> String {
    let mut out = String::new();
    let mut instr: Option<String> = None;
    let mut in_result = false;
    for run in p.elements() {
        match run.local() {
            "fldSimple" => {
                let code = run.attr("instr").unwrap_or("").split_whitespace().next().unwrap_or("FIELD").to_uppercase();
                out.push_str(&format!("{{{code}}}"));
            }
            "r" => {
                for child in run.elements() {
                    match child.local() {
                        "fldChar" => match child.attr("fldCharType") {
                            Some("begin") => instr = Some(String::new()),
                            Some("separate") => in_result = true,
                            Some("end") => {
                                if let Some(code) = instr.take() {
                                    let code = code.split_whitespace().next().unwrap_or("FIELD").to_uppercase();
                                    out.push_str(&format!("{{{code}}}"));
                                }
                                in_result = false;
                            }
                            _ => {}
                        },
                        "instrText" => {
                            if let Some(i) = instr.as_mut() {
                                i.push_str(&child.text());
                            }
                        }
                        "t" if instr.is_none() && !in_result => out.push_str(&child.text()),
                        "tab" => out.push('\t'),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    out.trim().to_string()
}
```

- [ ] **Step 4: Gather theme, styles and sections in `convert`**

In the facts block of Task 10, before `let facts = …`, add:

```rust
        let theme = package.xml("word/theme/theme1.xml").ok().flatten();
        let mut heading_styles: Vec<(usize, String)> = Vec::new();
        {
            let mut ps = Vec::new();
            body.find_all("p", &mut ps);
            let mut uses: std::collections::BTreeMap<(usize, String), usize> = std::collections::BTreeMap::new();
            for p in &ps {
                let Some(style) = p.path(&["pPr", "pStyle"]).and_then(|s| s.attr("val")) else { continue };
                if let Some(level) = writer.styles.heading_level(style) {
                    *uses.entry((level, style.to_string())).or_default() += 1;
                }
            }
            for level in 1..=6 {
                if let Some(((_, style), _)) = uses.iter().filter(|((l, _), _)| *l == level).max_by_key(|(_, n)| **n) {
                    heading_styles.push((level, style.clone()));
                }
            }
        }
        let mut table_styles: Vec<String> = Vec::new();
        {
            let mut tables = Vec::new();
            body.find_all("tbl", &mut tables);
            for t in tables {
                if let Some(s) = t.path(&["tblPr", "tblStyle"]).and_then(|s| s.attr("val")) {
                    if !table_styles.iter().any(|x| x == s) {
                        table_styles.push(s.to_string());
                    }
                }
            }
        }
        let even_odd = settings.as_ref().is_some_and(|s| s.child("evenAndOddHeaders").is_some());
        let sections = {
            // A section's properties sit at its end: in the last paragraph's
            // `w:pPr/w:sectPr`, or in `w:body` for the final section.
            let mut ps = Vec::new();
            body.find_all("p", &mut ps);
            let mut ends: Vec<(usize, Element)> = ps
                .iter()
                .filter_map(|p| {
                    let index: usize = p.attr(agent::INDEX)?.parse().ok()?;
                    Some((index, p.path(&["pPr", "sectPr"])?.clone()))
                })
                .collect();
            if let Some(last) = body.child("sectPr") {
                ends.push((stamped.0.saturating_sub(1), last.clone()));
            }
            let mut sections: Vec<header::SectionFact> = Vec::new();
            let mut first = 0;
            for (last, sect) in ends {
                let title_page = sect.child("titlePg").is_some();
                let columns = sect.child("cols").and_then(|c| c.attr("num")).and_then(|n| n.parse().ok()).unwrap_or(1);
                let mut stories = Vec::new();
                for reference in sect.elements() {
                    let kind = match reference.local() {
                        "headerReference" => "header",
                        "footerReference" => "footer",
                        _ => continue,
                    };
                    let ty = match reference.attr("type") {
                        Some("first") => "first",
                        Some("even") => "even",
                        _ => "default",
                    };
                    let Some(target) = reference.attr("id").and_then(|id| rels.get(id)).map(|r| r.target.clone()) else { continue };
                    let path = if target.starts_with("word/") { target.clone() } else { format!("word/{}", target.trim_start_matches('/')) };
                    let stem = std::path::Path::new(&target).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                    let Ok(Some(root)) = package.xml(&path) else { continue };
                    let paragraphs: Vec<&Element> = root.children_named("p").collect();
                    let (index, text, align) = paragraphs
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let align = p.path(&["pPr", "jc"]).and_then(|j| j.attr("val")).and_then(|v| match v {
                                "center" => Some("center"),
                                "right" | "end" => Some("right"),
                                _ => None,
                            });
                            (i, header::story_text(p), align)
                        })
                        .find(|(_, text, _)| !text.is_empty())
                        .unwrap_or((0, String::new(), None));
                    // `header2.xml` is `h1`, `footer1.xml` is `f0`; a paragraph
                    // other than the first adds `.pN`.
                    let number: usize = stem
                        .trim_start_matches(|c: char| c.is_ascii_alphabetic())
                        .parse::<usize>()
                        .map_or(0, |n| n.saturating_sub(1));
                    let short = format!("{}{number}", if kind == "header" { 'h' } else { 'f' });
                    stories.push(header::StoryFact {
                        kind,
                        ty,
                        id: if index == 0 { short } else { format!("{short}.p{index}") },
                        text,
                        align,
                        active: match ty { "first" => title_page, "even" => even_odd, _ => true },
                    });
                }
                // A type a section does not name is inherited from the one before.
                if let Some(previous) = sections.last() {
                    for inherited in &previous.stories {
                        if !stories.iter().any(|s| s.kind == inherited.kind && s.ty == inherited.ty) {
                            let mut s = inherited.clone();
                            s.active = match s.ty { "first" => title_page, "even" => even_odd, _ => true };
                            stories.push(s);
                        }
                    }
                }
                sections.push(header::SectionFact { first, last, sect_pr: Some(sect), stories, title_page, columns });
                first = last + 1;
            }
            sections
        };
```

and set `theme: theme.as_ref(), heading_styles, table_styles, sections` in `Facts`. `writer.styles` is the parsed `Styles` (private field of `Writer`, same module); `rels.get(id)` returns `Option<&Rel>` with `pub target: String` (`ooxml.rs:189-207`). `package` must still be mutable here (`xml` takes `&mut self`); if the borrow checker objects because `writer` borrows `&rels`, compute `sections` before the writer is created and keep it in a local.

- [ ] **Step 5: Run the tests**

Run: `cargo test --test agent_text_view`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/markdown/from_docx tests/agent_text_view.rs
git commit -m "feat(markdown): agent view header: page setup, styles, headers, footers, sections"
```

---

### Task 12: Block selection (`-p`, `--head`, `--tail`)

**Files:**
- Modify: `src/markdown/from_docx/agent.rs` (`select_blocks`), `src/markdown/from_docx/mod.rs` (`convert`), `src/markdown/mod.rs` (`MarkdownError` variant if needed)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

```rust
fn four_paragraphs_and_a_table() -> Vec<u8> {
    docx(&format!("{}{}{TABLE_3X2}{}{}", para("Zero"), para("One"), para("Eight"), para("Nine")))
}

fn selected(select: Select) -> String {
    agent_options(&four_paragraphs_and_a_table(), MarkdownOptions { select: Some(select), ..agent_defaults() })
}

#[test]
fn head_prints_the_first_blocks_and_a_range_line() {
    let out = selected(Select::Head(2));
    assert!(out.contains("\nbody: p0-p9, 1 table, 1 page      # page count estimated from breaks\nrange: head 2 (p0-p1) of p0-p9\n"), "{out}");
    assert_eq!(body(&out), "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nZero\n\n<!-- p1 -->\nOne\n");
}

#[test]
fn tail_prints_the_last_blocks_with_the_page_they_are_on() {
    let out = selected(Select::Tail(2));
    assert!(out.contains("\nrange: tail 2 (p8-p9) of p0-p9\n"), "{out}");
    assert_eq!(body(&out), "<!-- page 1 of 1 -->\n\n<!-- p8 -->\nEight\n\n<!-- p9 -->\nNine\n");
}

#[test]
fn picks_print_paragraphs_ranges_and_whole_tables_in_document_order() {
    let out = selected(Select::parse("p9, p1, p4").unwrap());
    assert!(out.contains("\nrange: p1, p4, p9 of p0-p9\n"), "{out}");
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p1 -->\nOne\n\n<!-- t0 center 3x2, cells p2-p7 by row, header row repeats -->\n|Item|Due|\n|-|-|\n|Report|Day 10|\n|Call|Monthly|\n\n<!-- p9 -->\nNine\n"
    );
    let out = selected(Select::parse("t0,p8-").unwrap());
    assert!(out.contains("\nrange: t0, p8-p9 of p0-p9\n"), "{out}");
    assert!(body(&out).starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 center 3x2"), "{out}");
}

#[test]
fn a_pick_past_the_end_is_an_error() {
    let err = docx_to_markdown(
        &four_paragraphs_and_a_table(),
        &MarkdownOptions { select: Some(Select::parse("p12").unwrap()), ..agent_defaults() },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("p12 is past the last paragraph p9"), "{err}");
    let err = docx_to_markdown(&four_paragraphs_and_a_table(), &MarkdownOptions { select: Some(Select::Head(0)), ..agent_defaults() })
        .unwrap_err()
        .to_string();
    assert!(err.contains("head needs a count above 0"), "{err}");
}

#[test]
fn a_selection_keeps_the_page_marker_of_a_later_page() {
    let bytes = docx(&format!(
        "{}{}<w:p><w:r><w:lastRenderedPageBreak/><w:t>Two</w:t></w:r></w:p>{}",
        para("Zero"), para("One"), para("Three")
    ));
    let out = agent_options(&bytes, MarkdownOptions { select: Some(Select::parse("p3").unwrap()), ..agent_defaults() });
    assert_eq!(body(&out), "<!-- page 2 of 2 -->\n\n<!-- p3 -->\nThree\n");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test --test agent_text_view head_prints tail_prints picks_print a_pick_past a_selection_keeps`
Expected: FAIL (the whole body prints, no `range:` line).

- [ ] **Step 3: Implement `select_blocks`**

Append to `agent.rs`:

```rust
use crate::markdown::{Pick, Select};

/// One block of the rendered body: its lines, the paragraph span it covers
/// and, for a table, its number.
struct Block {
    text: String,
    span: Option<(usize, usize)>,
    table: Option<usize>,
    /// The `<!-- page N of M -->` line that preceded it, if any.
    page: Option<String>,
}

/// Every `pN` in a table or empty-run line, for its span.
fn numbers_after(line: &str, key: &str) -> Vec<usize> {
    let Some(at) = line.find(key) else { return Vec::new() };
    line[at + key.len()..]
        .split(|c: char| !c.is_ascii_digit() && c != 'p')
        .filter_map(|piece| piece.strip_prefix('p')?.parse().ok())
        .collect()
}

/// Splits a rendered body on blank lines into blocks keyed by their id
/// lines; a page marker attaches to the block after it; a block with no id
/// line (notes, footnote definitions) attaches to the block before it.
fn blocks_of(body: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut page: Option<String> = None;
    for chunk in body.split("\n\n").filter(|c| !c.trim().is_empty()) {
        let first = chunk.lines().next().unwrap_or("");
        if first.starts_with("<!-- page ") {
            page = Some(first.to_string());
            continue;
        }
        let head = first.strip_prefix("<!-- ").unwrap_or("");
        if let Some(rest) = head.strip_prefix('p') {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(start) = digits.parse::<usize>() {
                let end = rest
                    .strip_prefix(&digits)
                    .and_then(|r| r.strip_prefix("-p"))
                    .map(|r| r.chars().take_while(char::is_ascii_digit).collect::<String>())
                    .and_then(|d| d.parse::<usize>().ok())
                    .unwrap_or(start);
                blocks.push(Block { text: chunk.to_string(), span: Some((start, end)), table: None, page: page.take() });
                continue;
            }
        }
        if let Some(rest) = head.strip_prefix('t') {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(table) = digits.parse::<usize>() {
                let cells = numbers_after(first, "cells ");
                let span = match (cells.iter().min(), cells.iter().max()) {
                    (Some(&a), Some(&b)) => Some((a, b)),
                    _ => None,
                };
                blocks.push(Block { text: chunk.to_string(), span, table: Some(table), page: page.take() });
                continue;
            }
        }
        match blocks.last_mut() {
            Some(last) => {
                last.text.push_str("\n\n");
                last.text.push_str(chunk);
            }
            None => blocks.push(Block { text: chunk.to_string(), span: None, table: None, page: page.take() }),
        }
    }
    blocks
}

fn span_text(a: usize, b: usize) -> String {
    if a == b { format!("p{a}") } else { format!("p{a}-p{b}") }
}

/// The selected blocks of `body` joined back, and the `range:` text.
pub(crate) fn select_blocks(body: &str, select: &Select, last: usize) -> Result<(String, String), String> {
    let blocks = blocks_of(body);
    let keep: Vec<bool>;
    let range: String;
    match select {
        Select::Head(n) | Select::Tail(n) if *n == 0 => {
            return Err(format!("{} needs a count above 0", if matches!(select, Select::Head(_)) { "head" } else { "tail" }));
        }
        Select::Head(n) => {
            keep = (0..blocks.len()).map(|i| i < *n).collect();
            let spans: Vec<(usize, usize)> = blocks.iter().take(*n).filter_map(|b| b.span).collect();
            range = format!("head {n} ({})", span_text(spans.first().map_or(0, |s| s.0), spans.last().map_or(0, |s| s.1)));
        }
        Select::Tail(n) => {
            let skip = blocks.len().saturating_sub(*n);
            keep = (0..blocks.len()).map(|i| i >= skip).collect();
            let spans: Vec<(usize, usize)> = blocks.iter().skip(skip).filter_map(|b| b.span).collect();
            range = format!("tail {n} ({})", span_text(spans.first().map_or(0, |s| s.0), spans.last().map_or(0, |s| s.1)));
        }
        Select::Picks(picks) => {
            let mut wanted: Vec<(usize, usize)> = Vec::new();
            let mut tables: Vec<usize> = Vec::new();
            let mut names: Vec<(usize, String)> = Vec::new();
            for pick in picks {
                match pick {
                    Pick::Paragraphs { from, to } => {
                        let to = to.unwrap_or(last);
                        for n in [*from, to] {
                            if n > last {
                                return Err(format!("p{n} is past the last paragraph p{last}"));
                            }
                        }
                        wanted.push((*from, to));
                        names.push((*from, span_text(*from, to)));
                    }
                    Pick::Table(t) => {
                        let Some(block) = blocks.iter().find(|b| b.table == Some(*t)) else {
                            return Err(format!("t{t} is not a table of this document"));
                        };
                        tables.push(*t);
                        names.push((block.span.map_or(0, |s| s.0), format!("t{t}")));
                    }
                }
            }
            keep = blocks
                .iter()
                .map(|b| {
                    b.table.is_some_and(|t| tables.contains(&t))
                        || b.span.is_some_and(|(a, z)| wanted.iter().any(|&(from, to)| a <= to && from <= z))
                })
                .collect();
            names.sort_by_key(|(at, _)| *at);
            range = names.into_iter().map(|(_, name)| name).collect::<Vec<_>>().join(", ");
        }
    }
    let mut out = String::new();
    let mut page_due: Option<&str> = None;
    let mut page_written: Option<&str> = None;
    for (block, keep) in blocks.iter().zip(&keep) {
        if let Some(page) = &block.page {
            page_due = Some(page.as_str());
        }
        if !keep {
            continue;
        }
        if let Some(page) = page_due.take() {
            if page_written != Some(page) {
                out.push_str(page);
                out.push_str("\n\n");
                page_written = Some(page);
            }
        }
        out.push_str(&block.text);
        out.push_str("\n\n");
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.is_empty() {
        out.push('\n');
    }
    Ok((out, range))
}
```

In `convert`, between pagination and the header (both inside the agent branch), apply the selection and keep its range for the header:

```rust
    let mut range = None;
    if let Some(select) = options.select.as_ref().filter(|_| options.ids) {
        let (selected, described) = agent::select_blocks(&markdown, select, stamped.0.saturating_sub(1))
            .map_err(|message| ooxml::invalid(message))?;
        markdown = selected;
        range = Some(described);
    }
```

and `range,` in `Facts`. `ooxml::invalid` builds the `ConvertError` the module already uses for bad input (`mod.rs:68`); `docx_to_markdown` maps it to `MarkdownError::Docx`, whose `Display` carries the message.

- [ ] **Step 4: Run the tests**

Run: `cargo test --test agent_text_view`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs tests/agent_text_view.rs
git commit -m "feat(markdown): -p, --head and --tail block selection in the agent view"
```

---

### Task 13: CLI wiring (`read`, alias `text`)

**Files:**
- Modify: `src/cli.rs:373-382` (`Text`), `src/bin/jubarte.rs:743-762` (`run_text`), its dispatch site, and the page-text helper around `src/bin/jubarte.rs:1515-1540`
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn cli_read_prints_the_agent_view_with_flags() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    let tracked = ok(&["read", "received.docx"], dir.path());
    assert_eq!(ok(&["text", "received.docx"], dir.path()), tracked, "text is an alias of read");
    assert!(tracked.starts_with("---\nsource: received.docx\nview: tracked "), "{tracked}");
    assert!(tracked.contains("# pages from layout\n"), "{tracked}");
    let hidden = ok(&["read", "received.docx", "--comments", "none"], dir.path());
    assert!(hidden.contains("view: tracked, comments hidden"), "{hidden}");
    let accepted = ok(&["read", "received.docx", "--track-changes", "accept"], dir.path());
    assert!(accepted.contains("view: accept-all"), "{accepted}");
    let rejected = ok(&["read", "received.docx", "--track-changes", "reject"], dir.path());
    assert!(rejected.contains("view: reject-all"), "{rejected}");
    let fast = ok(&["read", "received.docx", "--no-page-markers"], dir.path());
    assert!(fast.contains("# pages from Word's cached layout\n"), "{fast}");
    assert!(!fast.contains("<!-- page "), "{fast}");
    let head = ok(&["read", "received.docx", "--head", "3"], dir.path());
    assert!(head.contains("\nrange: head 3 (p0-p2) of p0-p20\n"), "{head}");
    let picked = ok(&["read", "received.docx", "-p", "p5,t0"], dir.path());
    assert!(picked.contains("\nrange: p5, t0 of p0-p20\n"), "{picked}");
    let dated = ok(&["read", "received.docx", "--dates"], dir.path());
    assert!(dated.contains("{>>#0 @AC<<}"), "{dated}");
    let bad = jubarte(&["read", "received.docx", "-p", "p99"], dir.path());
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("p99 is past the last paragraph p20"));
    let both = jubarte(&["read", "received.docx", "--head", "2", "--tail", "2"], dir.path());
    assert!(!both.status.success());
}
```

(`tempfile` is a dev-dependency already.) With `--no-page-markers` the cached fallback still counts pages but writes no markers: that is the `cached_pages: None` plus `pages_source: Cached` combination (see Step 3).

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test --test agent_text_view cli_read`
Expected: FAIL, `read` is not a command yet.

- [ ] **Step 3: Wire it**

`src/cli.rs`, the `Text` variant becomes `Read`, with the old name as a visible alias (clap: `#[command(visible_alias = "text")]` on the variant), and the `Command::Text { … }` arm in `cli_main` becomes `Command::Read { … }`:

```rust
    /// Read a document as the agent view: a YAML header, then Markdown with
    /// an `<!-- pN -->` id line before every paragraph, tracked changes as
    /// CriticMarkup followed by their ids (`{++text++}{>>#12 @AC<<}`) and
    /// comments with theirs (`{>>#c5 @AC: …<<}`). `jubarte FILE` is the same.
    #[command(visible_alias = "text")]
    Read {
        /// The document (.docx) to read.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// The view's options.
        #[command(flatten)]
        #[serde(flatten)]
        args: ReadArgs,
    },
```

The options live in their own `clap::Args` struct so that the one-file shorthand `jubarte FILE` (Task 19) can flatten the same struct into `Cli`; put it next to `CompareArgs`:

```rust
/// Options of `read`, also accepted by the one-file shorthand `jubarte FILE`.
#[derive(clap::Args, Debug, Default, PartialEq, Serialize)]
#[group(id = "read_options", multiple = true)]
pub struct ReadArgs {
    /// Tracked changes inline (all, the default), or the text with every
    /// change accepted or rejected; the id lines then list what changed.
    #[arg(long, value_enum, value_name = "CHOICE", help_heading = "Read options")]
    pub track_changes: Option<TrackChanges>,
    /// Comments inline (default) or hidden, with their ids on the id
    /// line of the paragraph that holds them.
    #[arg(long, value_enum, default_value_t = CommentsArg::Inline, value_name = "MODE", help_heading = "Read options")]
    pub comments: CommentsArg,
    /// Timestamps on the notes of an author whose changes do not all
    /// share one (the header shows an author's single timestamp).
    #[arg(long, help_heading = "Read options")]
    pub dates: bool,
    /// Skip the layout pass; page count from Word's cached breaks, no
    /// `<!-- page N of M -->` lines.
    #[arg(long, help_heading = "Read options")]
    pub no_page_markers: bool,
    /// Only these blocks: `p5`, `p4-p7`, `p17-`, `-p3`, `t0`, comma-separated.
    #[arg(short = 'p', long = "paragraphs", value_name = "SPEC", conflicts_with_all = ["head", "tail"], help_heading = "Read options")]
    pub paragraphs: Option<String>,
    /// Only the first N blocks (a table is one block).
    #[arg(long, value_name = "N", conflicts_with = "tail", help_heading = "Read options")]
    pub head: Option<usize>,
    /// Only the last N blocks.
    #[arg(long, value_name = "N", help_heading = "Read options")]
    pub tail: Option<usize>,
}


Add next to `TrackChanges` (line 902):

```rust
/// `--comments` of `read`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommentsArg {
    #[default]
    Inline,
    None,
}
```

`src/bin/jubarte.rs`: factor the page-text block of `convert -t md` (lines 1515-1540, the `render(…)` call and `rendered.report.pages[].text`) into a helper both commands call:

```rust
/// The text painted on each page by the layout pass, for page markers;
/// `None` (with a warning) when the layout fails.
fn page_texts(bytes: &[u8], revisions: jubarte::convert::Revisions) -> Option<Vec<String>> {
    match jubarte::convert::render(
        bytes,
        jubarte::convert::PdfOptions { revisions, ..jubarte::convert::PdfOptions::default() },
        jubarte::convert::RenderRequest::default(),
    ) {
        Ok(rendered) => Some(rendered.report.pages.iter().map(|p| p.text.clone()).collect()),
        Err(e) => {
            eprintln!("warning: no page markers: {e}");
            None
        }
    }
}
```

(keep the exact `render` signature the existing call uses; the existing `convert -t md` branch becomes a call to this helper plus `paginate`). Then `run_text`:

```rust
fn run_text(file: &Path, args: &ReadArgs) -> Result<(), String> {
    let ReadArgs { track_changes, comments, dates, no_page_markers, paragraphs, head, tail } = args;
    let (comments, dates, no_page_markers, head, tail) = (*comments, *dates, *no_page_markers, *head, *tail);
    let paragraphs = paragraphs.as_deref();
    let bytes = read_document(file)?;
    let select = match (paragraphs, head, tail) {
        (Some(spec), _, _) => Some(jubarte::markdown::Select::parse(spec)?),
        (None, Some(n), _) => Some(jubarte::markdown::Select::Head(n)),
        (None, None, Some(n)) => Some(jubarte::markdown::Select::Tail(n)),
        (None, None, None) => None,
    };
    let revisions = track_changes.unwrap_or(TrackChanges::All);
    let pages = if no_page_markers { None } else { page_texts(&bytes, revisions.into()) };
    let read = jubarte::markdown::docx_to_markdown(
        &bytes,
        &jubarte::markdown::MarkdownOptions {
            track_changes: revisions.into(),
            extract_media: None,
            ids: true,
            comments: comments == CommentsArg::Inline,
            source: file.file_name().map(|n| n.to_string_lossy().into_owned()),
            pages,
            page_markers: !no_page_markers,
            dates,
            select,
        },
    )
    .map_err(|e| e.to_string())?;
    print!("{}", read.markdown);
    Ok(())
}
```

`convert -t md` passes its `--track-changes` choice to the layout as `jubarte::convert::Revisions`; use the same conversion (`revisions.into()` or the match that branch uses today). Import `CommentsArg` and `ReadArgs` where `TrackChanges` is imported in the binary; the dispatch arm is `Some(Command::Read { file, args }) => run_text(&file, &args)` (`grep -n "run_text(" src/bin/jubarte.rs` for the old call). `jubarte::inspect::markdown` stays as a library function (the Python and WASM bindings keep calling it).

- [ ] **Step 4: Run the test and the CLI parser tests**

Run: `cargo test --test agent_text_view cli_read && cargo test --test cli_parser`
Expected: PASS. If `cli_parser` snapshots the `text --help` text or the command list, update it: the command is `read`, listed with its alias.

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/bin/jubarte.rs tests/agent_text_view.rs tests/cli_parser.rs
git commit -m "feat(cli): read (alias text) prints the agent view; -p, --head, --tail, --comments, --dates, --no-page-markers"
```

---

### Task 14: Tests that asserted the old layout

**Files:**
- Modify: `tests/adoption.rs:131,283-284,601-602,688-689`, `tests/m_cli_agent.rs:68-71`

- [ ] **Step 1: Run them to see which fail**

Run: `cargo test --test adoption --test m_cli_agent`
Expected: failures in `read_text_has_paragraph_ids_and_inspect_has_the_source_hash`, the accept/clean equality test (line 283), the legacy `.doc` test (601), the long-document round trip (688) and `text_prints_markdown_with_paragraph_ids`.

- [ ] **Step 2: Rewrite each assertion against the new layout, keeping its intent**

`tests/adoption.rs`: add once, near `ok`:

```rust
/// The body of a `text` output: what follows the YAML header, which names
/// the file and so differs between two copies of the same text.
fn body(text: &str) -> String {
    text.splitn(3, "---\n").nth(2).unwrap_or(text).to_string()
}
```

- line 131: `assert!(text.contains("<!-- p0"), "{text}");`
- lines 283-284: `assert_eq!(body(&ok(&["text", "check.docx"], &dir)), body(&ok(&["text", "review/clean.docx"], &dir)));`
- line 602: `assert!(text.contains("# Services Agreement"), "{text}");`
- lines 688-689: `assert_eq!(body(&ok(&["text", "back.docx"], dir.path())), body(&ok(&["text", "long.docx"], dir.path())));`

`tests/m_cli_agent.rs:68-71`, keeping the test name:

```rust
    assert!(stdout.starts_with("---\nsource: "), "{stdout}");
    assert!(stdout.contains("<!-- p0"), "{stdout}");
    assert!(stdout.contains("The individual"), "{stdout}");
```

If `write_fixture` styles its first paragraph as a heading, `<!-- p0` still matches; do not assert the heading marker here. If the equality tests at 283 and 688 differ only in page markers (the layout of two equal texts can differ by a soft break when their XML differs), compare with `--no-page-markers` on both sides.

- [ ] **Step 3: Run them**

Run: `cargo test --test adoption --test m_cli_agent`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/adoption.rs tests/m_cli_agent.rs
git commit -m "test: adoption and agent CLI tests read the agent view"
```

---

### Task 15: Goldens, edit → text round trips, docs

**Files:**
- Test: `tests/agent_text_view.rs`
- Modify: `Cargo.toml` (`[dev-dependencies]`), `docs/MARKDOWN.md`, `README.md:380,502-511`, `skills/jubarte-documents/SKILL.md:17,34-43`, `CHANGELOG.md`

- [ ] **Step 1: Write the golden tests**

```rust
#[test]
fn received_docx_tracked_view_matches_the_golden() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    assert_eq!(ok(&["read", "received.docx"], dir.path()), golden("received.tracked.md"));
}

#[test]
fn received_docx_other_views_match_their_goldens() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    assert_eq!(ok(&["read", "received.docx", "--comments", "none"], dir.path()), golden("received.no-comments.md"));
    assert_eq!(ok(&["read", "received.docx", "--track-changes", "accept"], dir.path()), golden("received.accept.md"));
    assert_eq!(ok(&["read", "received.docx", "--track-changes", "reject"], dir.path()), golden("received.reject.md"));
}
```

- [ ] **Step 2: Run them**

Run: `cargo test --test agent_text_view received_docx`
Expected: PASS. On a mismatch, read the diff line by line against the grammar and the fixture's XML (`unzip -o received.docx -d x && xmllint --format x/word/document.xml`); the usual suspects are header padding (`kv`), the `table:` line, `first: none` under `footers:`, the `<u>` on `secret`, the comment c11 placed before the final period, and the second page marker (the layout must put `[To be included]` on page 2, which the hard break at p19 forces). Fix the implementation.

- [ ] **Step 3: Write the edit → text tests**

These prove the ids printed are the ids the other commands take. Revision ids come from the comparer, so the tests read them from the output and check them against `jubarte changes`.

```rust
const BASE: &str = r#"<w:p><w:r><w:t>Fees</w:t></w:r></w:p><w:p><w:r><w:t>Client shall pay each invoice within thirty days of receipt.</w:t></w:r></w:p><w:p><w:r><w:t>Late amounts accrue interest at one percent per month.</w:t></w:r></w:p>"#;

fn edited(dir: &Path, plan: &str, out: &str) -> String {
    std::fs::write(dir.join("plan.json"), plan).unwrap();
    ok(&["edit", "base.docx", "--plan", "plan.json", "--out-dir", out], dir);
    ok(&["read", &format!("{out}/redline.docx"), "--no-page-markers"], dir)
}

fn captures<'a>(text: &'a str, pattern: &str) -> Vec<&'a str> {
    let re = regex::Regex::new(pattern).unwrap();
    let caps = re.captures(text).unwrap_or_else(|| panic!("no match for {pattern} in:\n{text}"));
    (1..caps.len()).map(|i| caps.get(i).unwrap().as_str()).collect()
}

#[test]
fn an_edit_shows_up_with_the_ids_that_changes_and_reject_take() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let plan = r#"{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":[
        {"kind":"replace","paragraph":{"index":1},"find":"thirty","replacement":"forty-five"},
        {"kind":"comment","paragraph":{"index":2},"find":"one percent","text":"Is one percent the statutory cap?"}]}"#;
    let text = edited(dir.path(), plan, "review");
    let ids = captures(&text, r"<!-- p1 -->\nClient shall pay each invoice within \{~~thirty~>forty-five~~\}\{>>#(\d+)\+(\d+) @AC<<\} days of receipt\.\n");
    let (del_id, ins_id) = (ids[0], ids[1]);
    assert!(text.contains("<!-- p2 -->\nLate amounts accrue interest at {==one percent==}{>>#c0 @AC: Is one percent the statutory cap?<<} per month.\n"), "{text}");
    assert!(text.contains("revisions: 1                       # 1 substitution (2 Word marks)"), "{text}");
    assert!(text.contains("comments: 1 thread open            # 1 comment: c0"), "{text}");
    assert!(text.contains("  AC: Ann Counsel                  # 1 revision, 1 comment, 2026-10-01T09:00:00Z"), "{text}");

    let changes = ok(&["changes", "review/redline.docx", "--json"], dir.path());
    let rows: Vec<serde_json::Value> = changes.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let by_id = |id: &str| rows.iter().find(|r| r["id"] == format!("body:rev:{id}")).unwrap_or_else(|| panic!("no change body:rev:{id} in {changes}"));
    assert_eq!(by_id(del_id)["kind"], "deletion");
    assert_eq!(by_id(del_id)["text"], "thirty");
    assert_eq!(by_id(ins_id)["kind"], "insertion");
    assert_eq!(by_id(ins_id)["text"], "forty-five");

    ok(&["reject", "review/redline.docx", "-o", "rejected.docx", "--id", &format!("body:rev:{del_id}"), "--id", &format!("body:rev:{ins_id}")], dir.path());
    let rejected = ok(&["read", "rejected.docx", "--no-page-markers"], dir.path());
    assert!(rejected.contains("<!-- p1 -->\nClient shall pay each invoice within thirty days of receipt.\n"), "{rejected}");
    assert!(rejected.contains("\nrevisions: 0\n"), "{rejected}");
    assert!(rejected.contains("{>>#c0 @AC: Is one percent the statutory cap?<<}"), "{rejected}");
}

#[test]
fn a_reply_and_a_resolution_by_a_second_author_thread_under_the_root() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let first = r#"{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":[
        {"kind":"comment","paragraph":{"index":2},"find":"one percent","text":"Cap?"}]}"#;
    edited(dir.path(), first, "one");
    std::fs::copy(dir.path().join("one/redline.docx"), dir.path().join("base.docx")).unwrap();
    let second = r#"{"schema_version":1,"author":"Bob Lee","date":"2026-10-02T10:00:00Z","operations":[
        {"kind":"reply_comment","comment_id":0,"text":"Yes, in Delaware."},
        {"kind":"resolve_comment","comment_id":0,"done":true}]}"#;
    let text = edited(dir.path(), second, "two");
    assert!(text.contains("{==one percent==}{>>#c0 @AC resolved: Cap?<<}{>>#c1 @BL re #c0: Yes, in Delaware.<<}"), "{text}");
    assert!(text.contains("comments: 0 threads open, 1 resolved  # 2 comments: c0 (+ reply c1)"), "{text}");
    assert!(text.contains("  BL: Bob Lee                      # 1 comment, 2026-10-02T10:00:00Z"), "{text}");
}

#[test]
fn a_second_author_editing_a_redline_gets_their_own_notes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let first = r#"{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":[
        {"kind":"replace","paragraph":{"index":1},"find":"thirty","replacement":"forty-five"}]}"#;
    edited(dir.path(), first, "one");
    std::fs::copy(dir.path().join("one/redline.docx"), dir.path().join("base.docx")).unwrap();
    let second = r#"{"schema_version":1,"author":"John Doe","date":"2026-10-02T10:00:00Z","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":{"index":1},"find":"receipt","replacement":"the invoice"}]}"#;
    let text = edited(dir.path(), second, "two");
    captures(&text, r"\{~~thirty~>forty-five~~\}\{>>#\d+\+\d+ @AC<<\} days of \{~~receipt~>the invoice~~\}\{>>#\d+\+\d+ @JD<<\}\.");
    assert!(text.contains("revisions: 2                       # 2 substitutions (4 Word marks)"), "{text}");
}
```

`serde_json` is a regular dependency (`Cargo.toml:112`), so tests can use it; `regex` is not in the manifest: add `regex = "1"` under `[dev-dependencies]`.

The comment id `c0` and the reply `c1`: `edit` allocates the next comment id as max existing + 1, 0 on a clean document (`src/edit.rs:1775`); the reply's initials derive from the author (`BL`) and are written to the comment (`src/edit.rs:1767-1773`); `resolve_comment` writes `w15:done` and the reply writes `w15:paraIdParent` (`src/edit.rs:315-322`, `reply_parents` in `src/edit/tracked.rs:180`). `keep` is required in the third test because `existing_revisions` defaults to `refuse` (`src/edit.rs:122-124`).

- [ ] **Step 4: Run them**

Run: `cargo test --test agent_text_view an_edit a_reply a_second_author`
Expected: PASS. If the substitution in the first test renders as two separate changes (`{--thirty--}{>>…<<}{++forty-five++}{>>…<<}`), the comparer emitted the insertion before the deletion with something between them; read `review/redline.docx`'s XML, and if the two marks are adjacent siblings, fix the pairing in `revision_tags`/`render`; if they are not, report the XML rather than loosening the regex.

- [ ] **Step 5: Docs**

`docs/MARKDOWN.md`: add a section `## Agent view` after `## CriticMarkup` with:

1. The grammar block from this plan's "Grammar" section, verbatim minus the test remarks.
2. A 15-line excerpt of `tests/fixtures/agent-view/received.tracked.md` (header through `<!-- p3 -->`).
3. "Reading back" (the contract for the applier plan). Write it as it stands here:

   > An agent view is CriticMarkup plus id lines, so it reads back as follows. A change followed by a tagged note (`{>>#12 @AC<<}`) is the document's revision `body:rev:12` (`footnotes:rev:12` under a footnote id line): kept as it is when unchanged, accepted or rejected when the agent removed its text. A change with no note is new, by the plan author. A tagged comment note is the document's comment; `re #cN` names its thread root; `resolved` resolves the thread; a note with no tag is a new comment by the plan author, a reply when it follows a tagged note on the same anchor. `<!-- pN -->` lines are alignment keys and are never text. Defaults when the markdown is incomplete, in order: `source:` names a base document, and everything not stated comes from it by id; a note's own `@handle` and timestamp; the handle's single timestamp from `authors:`; if `authors:` lists exactly one non-owner author, that author; else the author `Modified User`; a missing date is the header's `date:` line, else the conversion time in UTC, with a warning; `document_owner` missing is `Original User`; with no header at all, Letter portrait, one-inch margins, Normal Calibri 11pt and the writer's other defaults. A header without `source:` is honoured for `page:`, the `styles:` lines it names, and `headers:`/`footers:` text, alignment and `{PAGE}`/`{NUMPAGES}`/`{DATE}` fields.

4. "Limits": tracked paragraph marks inside comment bodies are not shown; page markers need the layout pass (`--no-page-markers` skips it, and the count then comes from Word's cached breaks); a table is one block for `-p`, `--head` and `--tail`.

`README.md:380`: `| \`jubarte text\` | Read the agent view: YAML header, \`<!-- pN -->\` id lines, tracked changes and comments with their ids |`. Lines 502-511: replace the `[body:p:12] …` sample with the golden body from `<!-- p0 center -->` through the `<!-- p3 -->` paragraph.

`skills/jubarte-documents/SKILL.md:17`: `Read | \`jubarte text file.docx\` (header + Markdown with \`<!-- pN -->\` ids; \`pN\` is \`body:p:N\`) or \`jubarte inspect file.docx --json\``; lines 34-43: show the id-line form, `-p p12,p40-p48`, and say that `{++…++}{>>#12 @AC<<}` is `body:rev:12` for `accept --id`/`reject --id` and `{>>#c5 @AC: …<<}` is `comment_id: 5`.

`CHANGELOG.md`, under the unreleased heading, `### Changed`: `jubarte text prints the agent view: a YAML header (authors with handles and timestamps, owner, page setup, styles, headers, footers, sections), an <!-- pN --> id line before every paragraph (same numbering as edit), tracked changes as CriticMarkup followed by a note with their w:id and author handle, comments with their ids and thread parents, page markers from the layout pass, and -p / --head / --tail / --comments none / --dates / --no-page-markers. The previous [body:p:N] projection remains available as inspect::markdown in the library bindings.`

- [ ] **Step 6: Whole suite, clippy, commit**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: clean.

```bash
git add tests/agent_text_view.rs Cargo.toml docs/MARKDOWN.md README.md skills/jubarte-documents/SKILL.md CHANGELOG.md
git commit -m "test(markdown): agent view goldens and edit round trips; docs"
```

---

## Self-review of Tasks 1-15 (Tasks 16-19 follow it)

Spec coverage: header lines (Tasks 10, 11), id lines and empties and page markers from both sources (3), annotations (4), tables with alignment and cell marks (5), attribution notes with ids, handles, joins and inline dates (6), comments with ids, `re`, `resolved`, hidden mode (7), accept/reject with `rev` and kept comments (8), `<u>` (9), `sections:` (11), `-p`/`--head`/`--tail` (12), CLI with layout pass and flags (13), goldens and edit round trips and the read-back contract (15). Not covered by design: the applier, JSON view, outline/grep, `edit` locator aliases.

Type consistency: `Handles { by_author, order, dates }` (Task 2) is what `tag_of`, `comment_head`, `header::render` read; `RevTag { kind, tag, author, date }` is produced by `revision_tags`/`collect_revisions` and consumed by `header::render`; `LineFacts` fields match the `paragraph` call; `Blocks::push_line` is used by `paragraph`, `flush_empty`, `blocks`; `Facts` is filled in Tasks 10, 11 and 12 (`range`); `Select`/`Pick` (Task 1) are read by `select_blocks` (Task 12) and built by `run_text` (Task 13); `page_marker` (Task 3) matches `pages::marker` minus its trailing blank line, which `Blocks::push` supplies.

Known risks, in the order they are likely to bite: `paginate`'s empty-key behaviour for a held line followed by a blank line (Task 3, Step 4; the unit test covers the id line and the page-break line); borrow of `package` versus `writer` when loading header and footer parts (Task 11, move the loading before the writer); `revise::resolve` dropping comment markers (Task 8); `cli_parser` help snapshots (Task 13); the comparer's mark order in the edit round trip (Task 15).

---

## Tasks 16-19: short ids, `--changed`, plan-free `edit` and `add`, implicit commands

These four tasks were added after Tasks 1-15 were written. They depend on Tasks 1, 12 and 13 (the `Select` enum, `select_blocks`, `ReadArgs`) and on nothing else in the plan, so they can be implemented after Task 15 in order. Source references are to the files as read on 2026-10-09: `src/edit.rs` 7,100 lines (`select` at 3134, `Selector` at 738, `Transaction` at 1596, `apply_plan` at 1263), `src/cli.rs` 1,530 lines (`CompareArgs` at 40, `Edit` at 384, `parse_json` at 1411), `src/bin/jubarte.rs` 2,500 lines (`run_edit` at 782, `resolve_compare` at 1022, `run` at 1089, `cli_main` at 1896).

### Task 16: Short ids in `edit` selectors

`edit` plans and the Task 18 shortcuts take the ids the view prints: `p12`, `h0`, `h0.p1`, `f1`, `t0.r1.c2`, `t0.r1.c2.p1`. They expand to the long ids the report keeps printing (`body:p:12`, `header1:p:0`, `footer2:p:0`), so nothing on the wire changes.

**Files:**
- Modify: `src/edit.rs` (`select`, two new methods on `Transaction`)
- Create: `tests/edit_short_ids.rs`

- [ ] **Step 1: Write the failing tests**

```rust
//! Short paragraph ids (`p3`, `h0.p1`, `t0.r1.c2`) in edit selectors.

mod common;

use common::docx::{docx_with_sect, para, Part, W_NS};
use jubarte::edit::{apply_plan, EditPlan, EditResult};

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
/// Two rows, two cells; the last cell holds two paragraphs. Body numbering:
/// p0 Zero, p1 Item, p2 Due, p3 Report, p4 Day 10, p5 or later, p6 Six.
const TABLE: &str = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Item</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Due</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Report</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Day 10</w:t></w:r></w:p><w:p><w:r><w:t>or later</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

fn source() -> Vec<u8> {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>DRAFT</w:t></w:r></w:p><w:p><w:r><w:t>Confidential</w:t></w:r></w:p></w:hdr>"#
    );
    docx_with_sect(
        &format!("{}{TABLE}{}", para("Zero"), para("Six")),
        &[Part { name: "word/header1.xml", content_type: HEADER_CT, rel_type: HEADER_REL, xml: &header }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

fn at(result: &EditResult, i: usize) -> &str {
    result.report.operations[i].paragraph.as_deref().unwrap()
}

#[test]
fn short_ids_resolve_to_the_long_ids_the_report_prints() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[
        {"kind":"replace","paragraph":"p6","find":"Six","replacement":"Seven"},
        {"kind":"replace","paragraph":"t0.r1.c1","find":"Day 10","replacement":"Day 12"},
        {"kind":"replace","paragraph":"t0.r1.c1.p1","find":"later","replacement":"earlier"},
        {"kind":"replace","paragraph":"h0","find":"DRAFT","replacement":"FINAL"},
        {"kind":"replace","paragraph":{"id":"h0.p1"},"find":"Confidential","replacement":"Public"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.ok, "{:?}", out.report.operations);
    assert_eq!(at(&out, 0), "body:p:6");
    assert_eq!(at(&out, 1), "body:p:4");
    assert_eq!(at(&out, 2), "body:p:5");
    assert_eq!(at(&out, 3), "header1:p:0");
    assert_eq!(at(&out, 4), "header1:p:1");
}

#[test]
fn a_short_id_that_points_nowhere_is_refused_and_named() {
    for (short, message) in [
        ("p7", "paragraph index 7 does not exist in body (7 paragraphs)"),
        ("h1", "h1 is header2, which this document does not have"),
        ("f0", "f0 is footer1, which this document does not have"),
        ("t1.r0.c0", "t1 is not a table of this document (1 table)"),
        ("t0.r2.c0", "t0 has 2 rows, no row 2"),
        ("t0.r0.c2", "t0.r0 has 2 cells, no cell 2"),
        ("t0.r0.c0.p1", "t0.r0.c0 has 1 paragraph, no p1"),
        ("t0", "t0: a table id needs a row and a cell, as t0.r1.c2"),
        ("px", "unknown paragraph id px"),
    ] {
        let e = apply_plan(
            &source(),
            &plan(&format!(
                r#"[{{"kind":"replace","paragraph":"{short}","find":"x","replacement":"y"}}]"#
            )),
        )
        .unwrap_err();
        assert_eq!(e.code, "ANCHOR_NOT_FOUND", "{short}: {e}");
        assert!(e.to_string().contains(message), "{short}: {e}");
    }
}
```

- [ ] **Step 2: Run them**

Run: `cargo test --test edit_short_ids`
Expected: FAIL; today every short id is "unknown paragraph id".

- [ ] **Step 3: Expand short ids in `select`**

In `Transaction::select` (`src/edit.rs:3134`), the `Selector::Name(id) | Selector::Id { id }` arm first expands a short id:

```rust
            Selector::Name(id) | Selector::Id { id } => {
                let long = match self.long_id(id) {
                    Some(Ok(long)) => long,
                    Some(Err(message)) => return not_found(message),
                    None => id.clone(),
                };
                match long
                    .rsplit_once(":p:")
                    .and_then(|(story, n)| Some((story.to_string(), n.parse::<usize>().ok()?)))
                {
                    Some((story, n)) => (story, Some(n)),
                    None => return not_found(format!("unknown paragraph id {id}")),
                }
            }
```

(`story` becomes an owned `String` in this arm; make the other arms produce `String` too, `BODY_STORY.to_string()`, and compare with `s.id == story` as before.) Add the two methods to `impl Transaction`:

```rust
    /// The long id of a short one: `p3` → `body:p:3`; `h0` and `h0.p1` →
    /// `header1:p:0` and `header1:p:1` (`hN` is the part `header{N+1}.xml`,
    /// as the agent view numbers them); `fN` likewise for footers;
    /// `t0.r1.c2` and `t0.r1.c2.p1` → the first (or the K-th) paragraph of
    /// that cell. `None` when `id` is not a short id.
    fn long_id(&self, id: &str) -> Option<Result<String, String>> {
        fn number(text: &str) -> Option<usize> {
            (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
                .then(|| text.parse().ok())
                .flatten()
        }
        let mut parts = id.split('.');
        let head = parts.next()?;
        let kind = head.chars().next()?;
        let n = number(&head[kind.len_utf8()..])?;
        let rest: Vec<&str> = parts.collect();
        let paragraph = |rest: &[&str]| -> Option<usize> {
            match rest {
                [] => Some(0),
                [p] => number(p.strip_prefix('p')?),
                _ => None,
            }
        };
        Some(match kind {
            'p' if rest.is_empty() => Ok(format!("body:p:{n}")),
            'h' | 'f' => {
                let index = paragraph(&rest)?;
                let story = format!("{}{}", if kind == 'h' { "header" } else { "footer" }, n + 1);
                if self.stories.iter().any(|s| s.id == story) {
                    Ok(format!("{story}:p:{index}"))
                } else {
                    Err(format!("{head} is {story}, which this document does not have"))
                }
            }
            't' => self.table_paragraph(head, n, &rest),
            _ => return None,
        })
    }

    /// `t{n}.r{R}.c{C}[.p{K}]`: the long id of that cell's K-th own
    /// paragraph. Tables are the body's top-level `w:tbl` elements in
    /// document order (nested tables are not numbered, as in the agent
    /// view); rows and cells count as they appear in the XML, a merged cell
    /// once.
    fn table_paragraph(&self, head: &str, n: usize, rest: &[&str]) -> Result<String, String> {
        let dom = &self.opened.dom;
        let (tc, tr, tbl, p) = (W::tc(), W::name("tr"), W::tbl(), W::p());
        let nearest = |node: NodeId, name: &crate::xmllinq::XName| dom.ancestors(node, Some(name)).first().copied();
        let tables: Vec<NodeId> = dom
            .descendants(self.opened.body, Some(&tbl))
            .into_iter()
            .filter(|&t| nearest(t, &tc).is_none() && nearest(t, &W::txbx_content()).is_none())
            .collect();
        let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
        let Some(&table) = tables.get(n) else {
            return Err(format!("{head} is not a table of this document ({})", plural(tables.len(), "table")));
        };
        let number = |text: &str, prefix: char| -> Option<usize> {
            let digits = text.strip_prefix(prefix)?;
            (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then(|| digits.parse().ok()).flatten()
        };
        let (row, cell, index) = match rest {
            [r, c] => (number(r, 'r'), number(c, 'c'), Some(0)),
            [r, c, k] => (number(r, 'r'), number(c, 'c'), number(k, 'p')),
            _ => (None, None, None),
        };
        let (Some(row), Some(cell), Some(index)) = (row, cell, index) else {
            return Err(format!("{head}: a table id needs a row and a cell, as t0.r1.c2"));
        };
        let rows: Vec<NodeId> = dom.descendants(table, Some(&tr)).into_iter().filter(|&r| nearest(r, &tbl) == Some(table)).collect();
        let Some(&row_node) = rows.get(row) else {
            return Err(format!("{head} has {}, no row {row}", plural(rows.len(), "row")));
        };
        let cells: Vec<NodeId> = dom.descendants(row_node, Some(&tc)).into_iter().filter(|&c| nearest(c, &tr) == Some(row_node)).collect();
        let Some(&cell_node) = cells.get(cell) else {
            return Err(format!("{head}.r{row} has {}, no cell {cell}", plural(cells.len(), "cell")));
        };
        let own: Vec<NodeId> = dom.descendants(cell_node, Some(&p)).into_iter().filter(|&q| nearest(q, &tc) == Some(cell_node)).collect();
        let Some(&node) = own.get(index) else {
            return Err(format!("{head}.r{row}.c{cell} has {}, no p{index}", plural(own.len(), "paragraph")));
        };
        let global = self
            .paragraph_nodes
            .iter()
            .position(|&q| q == node)
            .ok_or_else(|| format!("{head}: that paragraph is inside a text box and cannot be edited"))?;
        let (story, in_story) = self.paragraph_story[global];
        Ok(format!("{}:p:{in_story}", self.stories[story].id))
    }
```

`self.opened.body` is the body `NodeId` the transaction already uses as `stories[0].root` (`src/edit.rs:1725`); `W::tc()`, `W::tbl()`, `W::p()`, `W::txbx_content()` and `W::name("tr")` exist in `src/namespaces.rs` (lines 96-124). The same nearest-ancestor filtering is what `inspect::tables` does (`src/inspect/tables.rs:60-98`), so `t0.r1.c1` here is the cell `inspect --json` lists at `tables[0].rows[1][1]` when no table is nested. Error messages: "paragraph index 7 does not exist in body (7 paragraphs)" is the existing one at `src/edit.rs:3165`, reached through `body:p:7`.

Also extend the `Selector` doc comment (`src/edit.rs:733-736`): "Ids name their story (`body:p:3`, `header1:p:0`) or use the agent view's short forms (`p3`, `h0`, `h0.p1`, `f1`, `t0.r1.c2`, `t0.r1.c2.p1`); the report prints the long form."

- [ ] **Step 4: Run them**

Run: `cargo test --test edit_short_ids && cargo test --lib edit::`
Expected: PASS, and the edit unit tests unchanged.

- [ ] **Step 5: Commit**

```bash
git add src/edit.rs tests/edit_short_ids.rs
git commit -m "feat(edit): short paragraph ids (p3, h0.p1, f1, t0.r1.c2) in selectors"
```

---

### Task 17: `--changed` and `--by`: only the blocks with revisions or comments

An agent reading a redline usually wants the changed paragraphs, not the document. `read --changed` keeps the blocks that carry a tracked change or a comment; `--by NAME` (a handle or a full name) keeps those with that author's marks. Task 18's `edit` and `add` print their redline this way.

**Files:**
- Modify: `src/markdown/mod.rs` (`Select::Changed`), `src/markdown/from_docx/agent.rs` (`select_blocks`, `has_handle`), `src/markdown/from_docx/mod.rs` (`convert`: handle resolution), `src/cli.rs` (`ReadArgs`), `src/bin/jubarte.rs` (`run_text`)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

Append to `tests/agent_text_view.rs` (the `ins`/`del` helpers are Task 6's):

```rust
fn marked_docx() -> Vec<u8> {
    docx(&format!(
        r#"{}<w:p><w:r><w:t xml:space="preserve">Pay in </w:t></w:r>{}<w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>{}<w:p><w:r><w:t xml:space="preserve">Fee: </w:t></w:r>{}</w:p>"#,
        para("Quiet"),
        ins(0, "Ann Counsel", "ten"),
        para("Also quiet"),
        del(1, "Bob Lee", "waived"),
    ))
}

#[test]
fn changed_keeps_the_blocks_with_marks_by_anyone_or_by_one_author() {
    let bytes = marked_docx();
    let all = agent_options(&bytes, MarkdownOptions { select: Some(Select::Changed { by: None }), ..agent_defaults() });
    assert!(all.contains("\nrange: changed (p1, p3) of p0-p3\n"), "{all}");
    assert_eq!(
        body(&all),
        "<!-- page 1 of 1 -->\n\n<!-- p1 -->\nPay in {++ten++}{>>#0 @AC<<} days.\n\n<!-- p3 -->\nFee: {--waived--}{>>#1 @BL<<}\n"
    );
    let bob = agent_options(&bytes, MarkdownOptions { select: Some(Select::Changed { by: Some("Bob Lee".into()) }), ..agent_defaults() });
    assert!(bob.contains("\nrange: changed by @BL (p3) of p0-p3\n"), "{bob}");
    assert_eq!(body(&bob), "<!-- page 1 of 1 -->\n\n<!-- p3 -->\nFee: {--waived--}{>>#1 @BL<<}\n");
    let by_handle = agent_options(&bytes, MarkdownOptions { select: Some(Select::Changed { by: Some("BL".into()) }), ..agent_defaults() });
    assert_eq!(body(&by_handle), body(&bob));
    let nobody = agent_options(&bytes, MarkdownOptions { select: Some(Select::Changed { by: Some("Cy Young".into()) }), ..agent_defaults() });
    assert!(nobody.contains("\nrange: changed by Cy Young (none) of p0-p3\n"), "{nobody}");
    assert!(body(&nobody).trim().is_empty(), "{nobody}");
}

#[test]
fn cli_read_changed_prints_only_the_changed_blocks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    let changed = ok(&["read", "received.docx", "--changed", "--no-page-markers"], dir.path());
    assert!(changed.contains("\nrange: changed (p3, p5, p7, t0, p18) of p0-p20\n"), "{changed}");
    assert!(!changed.contains("<!-- p0 ") && !changed.contains("<!-- p1 "), "{changed}");
    let by = ok(&["read", "received.docx", "--changed", "--by", "AS", "--no-page-markers"], dir.path());
    assert!(by.contains("\nrange: changed by @AS (p5) of p0-p20\n"), "{by}");
    assert!(by.contains("{>>#c6 @AS re #c5: Disagree.<<}") && !by.contains("<!-- p3 "), "{by}");
    let bad = jubarte(&["read", "received.docx", "--changed", "-p", "p1"], dir.path());
    assert!(!bad.status.success(), "--changed conflicts with -p");
    let bad = jubarte(&["read", "received.docx", "--by", "AS"], dir.path());
    assert!(!bad.status.success(), "--by needs --changed");
}
```

The golden `received.tracked.md` has marks in p3, p5, p7, the table (cell p12) and p18; p5 holds the only `@AS` note (the reply c6).

- [ ] **Step 2: Run them**

Run: `cargo test --test agent_text_view changed`
Expected: FAIL to compile (`Select::Changed` does not exist).

- [ ] **Step 3: The variant, the matcher and the handle resolution**

`src/markdown/mod.rs`, in `Select` (Task 1):

```rust
    /// The blocks that carry a tracked change or a comment; with `by`, only
    /// those with that author's marks (`by` is a handle such as `AC` or the
    /// author's full name).
    Changed { by: Option<String> },
```

`src/markdown/from_docx/agent.rs`, a new arm in `select_blocks` before the `Select::Picks` arm (`by` arrives here already resolved to a handle when the author is known):

```rust
        Select::Changed { by } => {
            keep = blocks
                .iter()
                .map(|block| {
                    let first = block.text.lines().next().unwrap_or("");
                    let marked = block.text.contains("{>>#")
                        || [" rev #", " comments #", " break-ins #", " break-del #", " fmt #"].iter().any(|key| first.contains(key));
                    marked && by.as_deref().is_none_or(|handle| has_handle(&block.text, handle))
                })
                .collect();
            let names: Vec<String> = blocks
                .iter()
                .zip(&keep)
                .filter(|(_, keep)| **keep)
                .filter_map(|(block, _)| match (block.table, block.span) {
                    (Some(t), _) => Some(format!("t{t}")),
                    (None, Some((a, z))) => Some(span_text(a, z)),
                    _ => None,
                })
                .collect();
            range = format!(
                "changed{} ({})",
                by.as_deref().map_or(String::new(), |b| format!(" by {}", display_by(b))),
                if names.is_empty() { "none".to_string() } else { names.join(", ") }
            );
        }
```

with, next to `span_text`:

```rust
/// `@HH` in `text` where the handle ends (next char not alphanumeric).
fn has_handle(text: &str, handle: &str) -> bool {
    let key = format!("@{handle}");
    text.match_indices(&key)
        .any(|(at, _)| text[at + key.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric()))
}

/// A resolved handle prints as `@AC`; an unresolved name as given.
fn display_by(by: &str) -> String {
    if by.len() <= 4 && by.chars().all(|c| c.is_ascii_alphanumeric()) { format!("@{by}") } else { by.to_string() }
}
```

(`Option::is_none_or` needs Rust 1.82; if the toolchain is older use `map_or(true, …)`.)

`src/markdown/from_docx/mod.rs`, in `convert` where Task 12 applies the selection, resolve a name to its handle first; `handles` is the `agent::Handles` built in Task 2 (`by_author: HashMap<String, String>`):

```rust
    let select = options.select.clone().map(|select| match select {
        Select::Changed { by: Some(by) } => Select::Changed {
            by: Some(
                handles
                    .by_author
                    .get(&by)
                    .cloned()
                    .or_else(|| handles.by_author.values().find(|h| **h == by).cloned())
                    .unwrap_or(by),
            ),
        },
        other => other,
    });
```

and pass `select.as_ref()` to `select_blocks`. `Select` needs `Clone` (it has it from Task 1's derive; add it if not).

`src/cli.rs`, `ReadArgs` (Task 13) gains two fields:

```rust
    /// Only the blocks with a tracked change or a comment.
    #[arg(long, conflicts_with_all = ["paragraphs", "head", "tail"], help_heading = "Read options")]
    pub changed: bool,
    /// With --changed: only the blocks with this author's marks (a handle
    /// such as AC, or the full name).
    #[arg(long, value_name = "AUTHOR", requires = "changed", help_heading = "Read options")]
    pub by: Option<String>,
```

`src/bin/jubarte.rs`, `run_text`: destructure the two new fields and extend the `select` match:

```rust
    let select = match (paragraphs, head, tail, *changed) {
        (Some(spec), _, _, _) => Some(jubarte::markdown::Select::parse(spec)?),
        (None, Some(n), _, _) => Some(jubarte::markdown::Select::Head(n)),
        (None, None, Some(n), _) => Some(jubarte::markdown::Select::Tail(n)),
        (None, None, None, true) => Some(jubarte::markdown::Select::Changed { by: by.clone() }),
        (None, None, None, false) => None,
    };
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test agent_text_view`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/markdown/mod.rs src/markdown/from_docx/agent.rs src/markdown/from_docx/mod.rs src/cli.rs src/bin/jubarte.rs tests/agent_text_view.rs
git commit -m "feat(read): --changed and --by keep only the blocks with marks"
```

---

### Task 18: `edit` without a plan, and `add`

Two command-line forms that build an `EditPlan` in memory and run it through the same code as `--plan`:

```
jubarte edit FILE --replace WHERE FIND WITH [--replace …] [--delete WHERE FIND] [--comment WHERE FIND TEXT]
jubarte add  FILE WHERE TEXT [--after | --before | --start | --end | --after-text ANCHOR | --before-text ANCHOR]
```

`WHERE` is any selector id, short or long (Task 16). `--comment WHERE "" TEXT` comments the whole paragraph. `add --after` (the default) and `--before` insert a new paragraph that copies `WHERE`'s paragraph properties (`insert_paragraph`); `--start`, `--end`, `--after-text` and `--before-text` insert inside `WHERE` (`insert`). Both commands take `--author NAME` (default `Modified User`, the defaults contract's MU), `--date ISO8601` (default: now, in UTC, as a plan without `date`), `--existing-revisions auto|keep|accept|reject|refuse` (default `auto`: `keep` when the source already has tracked changes, so the other party's marks stay and the new ones sit beside them, as typing on a received redline does in Word; otherwise the comparer path, which gives word-level marks), `--out-dir DIR` (default `<FILE's directory>/<stem>.edit`, refused when it exists unless `--force`), `--force`, `-q`. After writing `clean.docx`, `redline.docx`, `patch.diff` and `report.jsonl`, both commands (and `edit --plan`) print the agent view of `redline.docx` selected with `--changed --by <author>` and no layout pass, in place of the patch that `edit` printed before (the patch is still in `patch.diff`).

**Files:**
- Modify: `src/cli.rs` (`Edit`, new `Add`, `ExistingArg`), `src/bin/jubarte.rs` (`EditJob`, `run_edit`, plan builders, dispatch)
- Test: `tests/agent_text_view.rs`

- [ ] **Step 1: Write the failing tests**

Append to `tests/agent_text_view.rs` (`BASE`, `captures`, `ok`, `jubarte` are Task 15's; `BASE` is p0 `Fees`, p1 `Client shall pay each invoice within thirty days of receipt.`, p2 `Late amounts accrue interest at one percent per month.`):

```rust
#[test]
fn edit_flags_need_no_plan_and_print_the_changed_blocks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let out = ok(
        &[
            "edit", "base.docx",
            "--replace", "p1", "thirty", "forty-five",
            "--delete", "p2", " per month",
            "--comment", "p0", "", "Add a fee schedule.",
            "--date", "2026-10-01T09:00:00Z",
        ],
        dir.path(),
    );
    for name in ["clean.docx", "redline.docx", "patch.diff", "report.jsonl"] {
        assert!(dir.path().join("base.edit").join(name).is_file(), "default out dir holds {name}");
    }
    assert!(out.contains("\nsource: base.edit/redline.docx\n"), "{out}");
    assert!(out.contains("\nrange: changed by @MU (p0, p1, p2) of p0-p2\n"), "{out}");
    assert!(out.contains("MU: Modified User") && out.contains("# 2 revisions, 1 comment, 2026-10-01T09:00:00Z"), "{out}");
    assert!(out.contains("<!-- p0 -->\n{==Fees==}{>>#c0 @MU: Add a fee schedule.<<}\n"), "{out}");
    captures(&out, r"<!-- p1 -->\nClient shall pay each invoice within \{~~thirty~>forty-five~~\}\{>>#\d+\+\d+ @MU<<\} days of receipt\.\n");
    captures(&out, r"<!-- p2 -->\nLate amounts accrue interest at one percent ?\{-- ?per month--\}\{>>#\d+ @MU<<\} ?\.\n");
    assert!(!out.contains("\n@@ "), "the patch is not printed: {out}");
    let again = jubarte(&["edit", "base.docx", "--replace", "p1", "thirty", "sixty"], dir.path());
    assert!(!again.status.success(), "base.edit exists: --force or --out-dir");
    let neither = jubarte(&["edit", "base.docx"], dir.path());
    assert!(!neither.status.success(), "a plan or an operation flag is required");
    let both = jubarte(&["edit", "base.docx", "--plan", "x.json", "--replace", "p1", "a", "b"], dir.path());
    assert!(!both.status.success(), "--plan and --replace conflict");
}

#[test]
fn edit_flags_on_a_redline_keep_the_other_party_s_marks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    ok(&["edit", "base.docx", "--replace", "p1", "thirty", "forty-five", "--author", "Ann Counsel", "--date", "2026-10-01T09:00:00Z", "--out-dir", "one"], dir.path());
    let out = ok(&["edit", "one/redline.docx", "--replace", "p1", "receipt", "the invoice", "--author", "John Doe", "--date", "2026-10-02T10:00:00Z", "--out-dir", "two"], dir.path());
    assert!(out.contains("\nrange: changed by @JD (p1) of p0-p2\n"), "{out}");
    captures(&out, r"\{~~thirty~>forty-five~~\}\{>>#\d+\+\d+ @AC<<\} days of \{~~receipt~>the invoice~~\}\{>>#\d+\+\d+ @JD<<\}\.");
    let report = std::fs::read_to_string(dir.path().join("two/report.jsonl")).unwrap();
    assert!(report.contains(r#""existing_revisions":"keep""#), "auto picked keep: {report}");
    let refused = jubarte(&["edit", "one/redline.docx", "--replace", "p1", "receipt", "the invoice", "--existing-revisions", "refuse", "--out-dir", "three"], dir.path());
    assert_eq!(refused.status.code(), Some(3), "refuse is still available");
}

#[test]
fn add_places_a_paragraph_or_text_where_asked() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let after = ok(&["add", "base.docx", "p1", "Invoices are due in full.", "--out-dir", "after"], dir.path());
    assert!(after.contains("range: changed by @MU (") && after.contains("p2"), "{after}");
    captures(&after, r"<!-- p2[^>]*-->\n\{\+\+Invoices are due in full\.\+\+\}\{>>#\d+ @MU<<\}\n");
    let before = ok(&["add", "base.docx", "p1", "Payment", "--before", "--out-dir", "before"], dir.path());
    captures(&before, r"<!-- p1[^>]*-->\n\{\+\+Payment\+\+\}\{>>#\d+ @MU<<\}\n");
    // Direct emission (keep) places inline text exactly where asked.
    let keep = ["--existing-revisions", "keep", "--date", "2026-10-01T09:00:00Z"];
    let end = ok(&[&["add", "base.docx", "p0", " and Expenses", "--end", "--out-dir", "end"][..], &keep[..]].concat(), dir.path());
    captures(&end, r"<!-- p0 -->\nFees\{\+\+ and Expenses\+\+\}\{>>#\d+ @MU<<\}\n");
    let start = ok(&[&["add", "base.docx", "p2", "Note: ", "--start", "--out-dir", "start"][..], &keep[..]].concat(), dir.path());
    captures(&start, r"<!-- p2 -->\n\{\+\+Note: \+\+\}\{>>#\d+ @MU<<\}Late amounts");
    let after_text = ok(&[&["add", "base.docx", "p1", " (30)", "--after-text", "thirty", "--out-dir", "at"][..], &keep[..]].concat(), dir.path());
    captures(&after_text, r"within thirty\{\+\+ \(30\)\+\+\}\{>>#\d+ @MU<<\} days");
    let before_text = ok(&[&["add", "base.docx", "p1", "calendar ", "--before-text", "days", "--out-dir", "bt"][..], &keep[..]].concat(), dir.path());
    captures(&before_text, r"within thirty \{\+\+calendar \+\+\}\{>>#\d+ @MU<<\}days of receipt");
    let two = jubarte(&["add", "base.docx", "p1", "x", "--before", "--end"], dir.path());
    assert!(!two.status.success(), "one placement only");
}
```

The first two `add` cases run through the comparer, whose placement of the inserted paragraph mark (on the new paragraph or on its neighbour) is its own; the test pins only the new paragraph's number and text. The inline cases pin exact strings under `keep`, where the text is emitted where the operation put it.

- [ ] **Step 2: Run them**

Run: `cargo test --test agent_text_view edit_flags add_places`
Expected: FAIL, unknown arguments.

- [ ] **Step 3: CLI**

`src/cli.rs`, the `Edit` variant: `plan` and `out_dir` become optional and the operation flags arrive:

```rust
    /// Edit a document: a JSON plan, or --replace / --delete / --comment
    /// flags; writes clean copy, redline, patch and report (refusal: exit 3)
    /// and prints the changed paragraphs as the agent view.
    #[command(after_help = "Examples:\n  \
        jubarte edit a.docx --replace p12 \"thirty days\" \"forty-five days\"\n  \
        jubarte edit a.docx --delete p7 \"at its sole discretion\" --comment p7 \"\" \"Removed; see call notes.\"\n  \
        jubarte edit a.docx --plan plan.json --out-dir review\n\n\
        WHERE is a paragraph id from `jubarte read`: p12, h0, f0.p1, t0.r1.c2 (or body:p:12).")]
    Edit {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Edit plan JSON (see `jubarte capabilities --json` for the kinds).
        #[arg(long, value_name = "PLAN.json", required_unless_present_any = ["replace", "delete", "comment"], conflicts_with_all = ["replace", "delete", "comment", "author", "date", "existing_revisions"])]
        plan: Option<PathBuf>,
        /// Replace FIND (once in WHERE) with WITH; repeatable.
        #[arg(long, num_args = 3, value_names = ["WHERE", "FIND", "WITH"], action = clap::ArgAction::Append)]
        replace: Vec<String>,
        /// Delete FIND (once in WHERE); repeatable.
        #[arg(long, num_args = 2, value_names = ["WHERE", "FIND"], action = clap::ArgAction::Append)]
        delete: Vec<String>,
        /// Comment TEXT on FIND in WHERE; an empty FIND ("") comments the
        /// whole paragraph; repeatable.
        #[arg(long, num_args = 3, value_names = ["WHERE", "FIND", "TEXT"], action = clap::ArgAction::Append)]
        comment: Vec<String>,
        /// Author of the flags' changes and comments.
        #[arg(long, value_name = "NAME", default_value = "Modified User")]
        author: String,
        /// Their timestamp (ISO 8601) [default: now, UTC].
        #[arg(long, value_name = "ISO8601")]
        date: Option<String>,
        /// What to do when FILE already holds tracked changes: auto keeps
        /// them and tracks the new edits beside them; a clean file goes
        /// through the comparer.
        #[arg(long, value_enum, value_name = "MODE", default_value_t = ExistingArg::Auto)]
        existing_revisions: ExistingArg,
        /// Directory to create for clean.docx, redline.docx, patch.diff,
        /// report.jsonl [default: <FILE's directory>/<stem>.edit].
        #[arg(long, value_name = "DIR")]
        out_dir: Option<PathBuf>,
        … dry_run, force, pdf, png, dpi, revisions, revision_palette, quiet unchanged …
    },
    /// Add text: a new paragraph after or before WHERE, or text inside it.
    #[command(after_help = "Examples:\n  \
        jubarte add a.docx p12 \"Time is of the essence.\"              new paragraph after p12\n  \
        jubarte add a.docx p12 \"Recitals\" --before\n  \
        jubarte add a.docx p5 \" (the \\\"Fee\\\")\" --after-text \"monthly fee\"\n  \
        jubarte add a.docx t0.r1.c1 \" or later\" --end")]
    Add {
        /// The source document (.docx). Never modified.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Paragraph id from `jubarte read`: p12, h0, f0.p1, t0.r1.c2.
        #[arg(value_name = "WHERE")]
        at: String,
        /// The text to add.
        #[arg(value_name = "TEXT")]
        text: String,
        /// A new paragraph after WHERE (the default).
        #[arg(long, group = "place")]
        after: bool,
        /// A new paragraph before WHERE.
        #[arg(long, group = "place")]
        before: bool,
        /// Inside WHERE, at its start.
        #[arg(long, group = "place")]
        start: bool,
        /// Inside WHERE, at its end.
        #[arg(long, group = "place")]
        end: bool,
        /// Inside WHERE, right after ANCHOR (which must occur once).
        #[arg(long, value_name = "ANCHOR", group = "place")]
        after_text: Option<String>,
        /// Inside WHERE, right before ANCHOR (which must occur once).
        #[arg(long, value_name = "ANCHOR", group = "place")]
        before_text: Option<String>,
        /// Author of the change.
        #[arg(long, value_name = "NAME", default_value = "Modified User")]
        author: String,
        /// Its timestamp (ISO 8601) [default: now, UTC].
        #[arg(long, value_name = "ISO8601")]
        date: Option<String>,
        /// What to do when FILE already holds tracked changes (see edit).
        #[arg(long, value_enum, value_name = "MODE", default_value_t = ExistingArg::Auto)]
        existing_revisions: ExistingArg,
        /// Directory to create for the outputs [default: <FILE's directory>/<stem>.edit].
        #[arg(long, value_name = "DIR")]
        out_dir: Option<PathBuf>,
        /// Replace an existing output directory's files.
        #[arg(long)]
        force: bool,
        /// Print nothing on success.
        #[arg(short = 'q', long)]
        quiet: bool,
    },
```

and, next to `TrackChanges`:

```rust
/// `--existing-revisions` of `edit` and `add`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExistingArg {
    /// `keep` when the file has tracked changes, else the comparer path.
    #[default]
    Auto,
    Keep,
    Accept,
    Reject,
    Refuse,
}
```

`group = "place"` on the six placement flags makes them mutually exclusive (clap creates the group); `cli_definition_is_valid` (`Cli::command().debug_assert()`) catches a mistake in these attributes. The `Serialize` derive on `Command` is for `parse_json`; `Vec<String>` and the new enum serialize as they are.

- [ ] **Step 4: The binary**

`src/bin/jubarte.rs`. `EditJob` loses `plan`; `run_edit(job, plan: jubarte::edit::EditPlan)` takes the plan from its caller. Add:

```rust
/// `<dir>/<stem>.edit` next to the source.
fn default_out_dir(file: &Path) -> PathBuf {
    let stem = file.file_stem().map_or_else(|| "document".to_string(), |s| s.to_string_lossy().into_owned());
    match file.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) => dir.join(format!("{stem}.edit")),
        None => PathBuf::from(format!("{stem}.edit")),
    }
}

/// `auto` is `keep` for a source with tracked changes, else `refuse` (which
/// a clean source never triggers, so the comparer path runs).
fn existing_revisions(mode: ExistingArg, source: &[u8]) -> jubarte::edit::ExistingRevisions {
    use jubarte::edit::ExistingRevisions as E;
    match mode {
        ExistingArg::Auto => {
            if jubarte::changes::list_changes(source).is_ok_and(|c| !c.is_empty()) { E::Keep } else { E::Refuse }
        }
        ExistingArg::Keep => E::Keep,
        ExistingArg::Accept => E::Accept,
        ExistingArg::Reject => E::Reject,
        ExistingArg::Refuse => E::Refuse,
    }
}

fn flag_plan(author: String, date: Option<String>, existing: jubarte::edit::ExistingRevisions, operations: Vec<jubarte::edit::OperationKind>) -> jubarte::edit::EditPlan {
    jubarte::edit::EditPlan {
        schema_version: jubarte::edit::SCHEMA_VERSION,
        source_sha256: None,
        author,
        date,
        initials: None,
        resolve_revisions: None,
        existing_revisions: existing,
        operations: operations.into_iter().map(|kind| jubarte::edit::Operation { id: None, kind }).collect(),
        update_fields: false,
    }
}

/// `--replace`, `--delete`, `--comment` triples and pairs, in that order.
fn edit_flag_operations(replace: &[String], delete: &[String], comment: &[String]) -> Vec<jubarte::edit::OperationKind> {
    use jubarte::edit::{OperationKind, Selector};
    let at = |id: &String| Selector::Name(id.clone());
    let mut ops = Vec::new();
    for r in replace.chunks_exact(3) {
        ops.push(OperationKind::Replace { paragraph: at(&r[0]), find: r[1].clone(), replacement: r[2].clone(), format: None, comment: None, whole: false, occurrence: None });
    }
    for d in delete.chunks_exact(2) {
        ops.push(OperationKind::Delete { paragraph: at(&d[0]), find: d[1].clone(), occurrence: None });
    }
    for c in comment.chunks_exact(3) {
        ops.push(OperationKind::Comment { paragraph: at(&c[0]), find: (!c[1].is_empty()).then(|| c[1].clone()), text: c[2].clone(), through: None, occurrence: None });
    }
    ops
}

fn add_operation(at: &str, text: &str, before: bool, start: bool, end: bool, after_text: Option<&str>, before_text: Option<&str>) -> jubarte::edit::OperationKind {
    use jubarte::edit::{Edge, OperationKind, RunSpec, Selector, Side};
    let paragraph = Selector::Name(at.to_string());
    let inline = |after: Option<String>, before: Option<String>, position: Option<Edge>| OperationKind::Insert {
        paragraph: paragraph.clone(), after, before, position, text: text.to_string(), format: None, comment: None, occurrence: None,
    };
    match (start, end, after_text, before_text) {
        (true, _, _, _) => inline(None, None, Some(Edge::Start)),
        (_, true, _, _) => inline(None, None, Some(Edge::End)),
        (_, _, Some(anchor), _) => inline(Some(anchor.to_string()), None, None),
        (_, _, _, Some(anchor)) => inline(None, Some(anchor.to_string()), None),
        _ => OperationKind::InsertParagraph {
            paragraph,
            position: if before { Side::Before } else { Side::After },
            runs: vec![RunSpec { text: text.to_string(), ..RunSpec::default() }],
            like: None,
            style: None,
            comment: None,
        },
    }
}
```

`OperationKind`, `Operation`, `Selector`, `RunSpec`, `Edge`, `Side`, `SCHEMA_VERSION` are public in `jubarte::edit` (`src/edit.rs:50-320, 712-800`); if a field was added to `Replace`/`Insert`/`Comment`/`InsertParagraph` since, the compiler names it, fill it with `None`/`false`.

In `cli_main`, the `Edit` arm keeps its exit-code mapping (`Ok(()) => ExitCode::SUCCESS`, `Err((code, message))` as today) around a closure that loads or builds the plan; `run_edit(job, plan, source)` no longer reads the file or the plan itself:

```rust
            let outcome = (|| -> Result<(), (u8, String)> {
                let fail = |m: String| (1u8, m);
                let source = read_document(&file).map_err(fail)?;
                let plan = match &plan {
                    Some(path) => {
                        let json = std::fs::read_to_string(path)
                            .map_err(|e| fail(format!("reading {}: {e}", path.display())))?;
                        jubarte::edit::EditPlan::from_json(&json).map_err(|e| (EXIT_PLAN_REFUSED, e.to_string()))?
                    }
                    None => flag_plan(
                        author.clone(),
                        date.clone(),
                        existing_revisions(existing_revisions, &source),
                        edit_flag_operations(&replace, &delete, &comment),
                    ),
                };
                let out_dir = out_dir.clone().unwrap_or_else(|| default_out_dir(&file));
                run_edit(
                    &EditJob { file: &file, out_dir: &out_dir, dry_run, force, pdf, png, dpi, revisions: style, quiet },
                    plan,
                    source,
                )
            })();
```

The `Add` arm does the same with `flag_plan(author, date, existing, vec![add_operation(&at, &text, before, start, end, after_text.as_deref(), before_text.as_deref())])` and an `EditJob` whose `dry_run`, `pdf` and `png` are false, `dpi` is 96.0 and `revisions` is `revision_style(Revisions::Conventional, None)` (the conventional style; `quiet` and `force` from the flags).

In `run_edit`, replace the final `print!("{patch}")` (line 924) with the view:

```rust
    let view = jubarte::markdown::docx_to_markdown(
        &result.redline,
        &jubarte::markdown::MarkdownOptions {
            ids: true,
            comments: true,
            source: Some(job.out_dir.join("redline.docx").display().to_string()),
            page_markers: false,
            select: Some(jubarte::markdown::Select::Changed { by: Some(result.report.author.clone()) }),
            ..Default::default()
        },
    )
    .map_err(|e| fail(format!("reading the redline back: {e}")))?;
    print!("{}", view.markdown);
```

and change the `wrote …` line's wording only if it names the patch as printed. `-q` suppresses the view with the rest.

- [ ] **Step 5: Run the tests, the CLI parser tests and the edit tests**

Run: `cargo test --test agent_text_view && cargo test --test cli_parser && cargo test --test adoption && cargo test --test m_cli_agent && cargo test --test cli_failure_paths`
Expected: PASS. `cli_parser` parses `["edit", "a", "--plan", "p", "--out-dir", "d"]` (line 186-188); if it compares the whole `args` object, add `replace: []`, `delete: []`, `comment: []`, `author: "Modified User"`, `date: null`, `existing_revisions: "auto"`. `adoption.rs` and `m_cli_agent.rs` read `patch.diff` from disk, not from stdout, so they pass as they are; if one asserts the patch on stdout, assert it from the file instead (the plan's rule 4 applies: no test is deleted).

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/bin/jubarte.rs tests/agent_text_view.rs
git commit -m "feat(cli): edit --replace/--delete/--comment and add, plan-free; edit prints the changed blocks"
```

---

### Task 19: `jubarte FILE` reads, `jubarte A B` prints the redline; docs

- `jubarte FILE [read options]` is `jubarte read FILE [read options]`.
- `jubarte A B` compares (the comparer already accepts both sides' tracked changes before comparing in the default Word mode, `src/document_comparer.rs:6433-6442`) and, with no `-o`, prints the redline's agent view instead of writing a file; the read options apply to that view (`jubarte old.docx new.docx --changed`). With `-o FILE` it writes the file and prints `wrote …` as today, and read options are refused. The explicit `jubarte compare A B` keeps today's behaviour (default output `<A>_v_<B>.docx`).
- `parse_json` (the Python and WASM facade) reports a one-file shorthand as `{"command": "read", "args": {"file": …, …read options…}}`, and refuses it when `read` is not among the adapter's supported tasks.

**Files:**
- Modify: `src/cli.rs` (`CompareArgs`, `Cli`, `validate_matches`, `parse_json`, help), `src/bin/jubarte.rs` (`Job`, `resolve_compare`, `run`, `cli_main`, unit tests), `tests/cli_parser.rs`
- Test: `tests/agent_text_view.rs`
- Docs: `docs/MARKDOWN.md`, `README.md`, `skills/jubarte-documents/SKILL.md`, `CHANGELOG.md`

- [ ] **Step 1: Write the failing tests**

Append to `tests/agent_text_view.rs`:

```rust
#[test]
fn one_file_prints_the_agent_view_and_two_files_print_their_redline() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    assert_eq!(ok(&["received.docx"], dir.path()), golden("received.tracked.md"));
    assert_eq!(
        ok(&["received.docx", "--head", "2", "--no-page-markers"], dir.path()),
        ok(&["read", "received.docx", "--head", "2", "--no-page-markers"], dir.path())
    );

    std::fs::write(dir.path().join("old.docx"), docx(BASE)).unwrap();
    std::fs::write(dir.path().join("new.docx"), docx(&BASE.replace("thirty", "forty-five"))).unwrap();
    let out = ok(&["old.docx", "new.docx"], dir.path());
    assert!(out.starts_with("---\nsource: old_v_new.docx (not written; -o keeps it)\n"), "{out}");
    captures(&out, r"<!-- p1 -->\nClient shall pay each invoice within \{~~thirty~>forty-five~~\}\{>>#\d+\+\d+ @R<<\} days of receipt\.\n");
    assert!(out.contains("R: Redline"), "{out}");
    assert!(!dir.path().join("old_v_new.docx").exists(), "nothing written without -o");
    let written = ok(&["old.docx", "new.docx", "-o", "r.docx"], dir.path());
    assert!(written.starts_with("wrote r.docx"), "{written}");
    assert!(dir.path().join("r.docx").is_file());
    let changed = ok(&["old.docx", "new.docx", "--changed"], dir.path());
    assert!(changed.contains("\nrange: changed (p1) of p0-p2\n"), "{changed}");
    let bad = jubarte(&["old.docx", "new.docx", "-o", "x.docx", "--head", "1"], dir.path());
    assert!(!bad.status.success(), "read options go with the printed view, not with -o");
    let bad = jubarte(&["compare", "old.docx"], dir.path());
    assert!(!bad.status.success(), "compare still needs two documents");
}
```

`tests/cli_parser.rs:36-43`: take `&["a.docx"][..]` out of the exit-2 list and add, in the same test:

```rust
    let read = parse(&["a.docx"], &[]);
    assert_eq!(read["exit_code"], 0, "{read}");
    assert_eq!(read["command"], "read");
    assert_eq!(read["args"]["file"], "a.docx");
    assert!(read["args"]["head"].is_null() && read["args"]["changed"] == false);
    assert_eq!(parse(&["a.docx", "--head", "2"], &[])["args"]["head"], 2);
    assert_eq!(parse(&["a.docx"], &["compare"])["exit_code"], 2, "read not supported");
    assert_eq!(parse(&["compare", "a.docx"], &[])["exit_code"], 2);
    assert_eq!(parse(&["a.docx", "b.docx", "--head", "2"], &[])["args"]["view"]["head"], 2);
    assert_eq!(parse(&["a.docx", "b.docx", "-o", "x.docx", "--head", "2"], &[])["exit_code"], 2);
```

`src/bin/jubarte.rs` unit test `positional_args_and_default_output` (line 2464): `j.output` becomes `None` for the shorthand; add `let j = job_of(&["jubarte", "compare", "a.docx", "b.docx"]); assert_eq!(j.output, Some(PathBuf::from("a_v_b.docx")));`.

- [ ] **Step 2: Run them**

Run: `cargo test --test agent_text_view one_file && cargo test --test cli_parser && cargo test --bin jubarte positional`
Expected: FAIL (`MODIFIED` is required; output path is not optional).

- [ ] **Step 3: CLI**

`src/cli.rs`:

1. `CompareArgs::modified_pos` (line 46): drop `required_unless_present = "modified"`; doc: `/// The modified document (.docx or Markdown). With ORIGINAL alone, the agent view of that one document is printed (same as \`read\`).`
2. `Cli` gains the read options after `compare`:

```rust
    /// Options of the one-file shorthand (`jubarte FILE`).
    #[command(flatten)]
    #[serde(flatten)]
    pub read: ReadArgs,
```

3. `after_help` (line 24): add `  jubarte contract.docx                    print the agent view (read)` as the second example and reword the shorthand line to `  jubarte old.docx new.docx                redline printed as the agent view (-o writes it)`.
4. `validate_matches` (line 1202) returns `Ok(())` as soon as there is no subcommand. Before that return, add the shorthand check: with no subcommand, when `output` is present and any read option (`track_changes`, `comments`, `dates`, `no_page_markers`, `paragraphs`, `head`, `tail`, `changed`, `by`) has `ValueSource::CommandLine`, return `command.error(ErrorKind::ArgumentConflict, "read options apply to the printed view; drop -o to print it")`. In the subcommand branch, when `name == "compare"` and neither `modified` nor `modified_pos` is present, return `task.error(ErrorKind::MissingRequiredArgument, "compare needs MODIFIED (or -m FILE)")`.
5. `parse_json` (line 1495-1505), in the no-subcommand branch after the unsupported-task check:

```rust
            if compare.modified.is_none() && compare.modified_pos.is_none() {
                if !accepts("read") {
                    return Err(command.error(clap::error::ErrorKind::InvalidSubcommand, "this task is not supported by this adapter"));
                }
                let read = ReadArgs::from_arg_matches(&matches)?;
                let mut args = serde_json::to_value(&read).expect("UTF-8 CLI arguments");
                args["file"] = serde_json::to_value(compare.original.as_ref().or(compare.original_pos.as_ref())).expect("UTF-8 path");
                return Ok(serde_json::json!({"exit_code": 0, "command": "read", "args": args}).to_string());
            }
```

(`clap::FromArgMatches` is already imported there for `CompareArgs::from_arg_matches`.) In the two-input case of the same branch, `compare_json(&compare)` gains the view options: `args["view"] = serde_json::to_value(&ReadArgs::from_arg_matches(&matches)?)`, so an adapter that prints the view can honour them. Add `("read", "contract.docx")` to the facade's examples list.

- [ ] **Step 4: The binary**

`src/bin/jubarte.rs`:

1. `Job.output: Option<PathBuf>` and `Job.read: ReadArgs`; `resolve_compare(compare: CompareArgs, read: ReadArgs, explicit: bool)`: `output: compare.output.or_else(|| explicit.then(|| default_output(&original, &modified)))`, `read`; the `Command::Compare(args)` dispatch passes `ReadArgs::default(), true`, the `None` arm `cli.read, false`. (`job_of` in the unit tests passes `ReadArgs::default()` for the explicit form.)
2. `cli_main`'s `None` arm, before resolving the comparison:

```rust
        None => {
            let modified = cli.compare.modified.as_ref().or(cli.compare.modified_pos.as_ref());
            if modified.is_none() {
                let file = cli.compare.original.clone().or(cli.compare.original_pos.clone()).expect("clap requires ORIGINAL");
                return exit_code(run_text(&file, &cli.read));
            }
            resolve_compare(cli.compare, cli.read, false)
        }
```

3. `run(job)`: `ensure_writable` only when `job.output` is `Some`; after `out` is computed, when `job.output` is `None`, print the view and return:

```rust
    let Some(output) = &job.output else {
        let name = default_output(&job.original, &job.modified);
        return print_agent_view(&out, format!("{} (not written; -o keeps it)", name.display()), &job.read);
    };
```

where `print_agent_view(bytes: &[u8], source: String, args: &ReadArgs) -> Result<(), String>` is `run_text`'s body from `let select = …` on, with `source` passed in instead of taken from the path (`run_text` becomes `read_document` + `print_agent_view(&bytes, file name, args)`). A `.md` pair with no `-o` compares to DOCX (`Format::of_path(None)` falls to `Docx`, as the existing `unwrap_or(Format::Docx)` does) and the view reads that.
4. Nothing else in `run` changes: `-o` keeps writing and printing `wrote …`.

- [ ] **Step 5: Run everything that touches the CLI**

Run: `cargo test --test agent_text_view && cargo test --test cli_parser && cargo test --bin jubarte && cargo test --test adoption && cargo test --test m_cli_agent && cargo test --test cli_failure_paths`
Expected: PASS. `cli_parser::facade_restricts_help_acceptance_aliases_and_shorthand` and `supported_compare_never_accepts_a_disabled_task_as_filenames` keep passing because the disabled-task check runs before the read branch; if a help snapshot lists the top-level options, it now shows the "Read options" heading.

- [ ] **Step 6: Docs**

`docs/MARKDOWN.md`, in the `## Agent view` section Task 15 added, a `### Commands` subsection:

```
jubarte read FILE                 the view (alias: text; `jubarte FILE` is the same)
  -p p5,p12-p20,t0  --head N  --tail N  --changed [--by AC]
  --track-changes accept|reject   --comments none   --dates   --no-page-markers
jubarte A B                       accept both sides' changes, compare, print the redline's view (-o FILE writes it)
jubarte edit FILE --replace WHERE FIND WITH  --delete WHERE FIND  --comment WHERE FIND TEXT
jubarte add FILE WHERE TEXT [--after|--before|--start|--end|--after-text X|--before-text X]
```

with one paragraph after it: ids (`p12`, `h0`, `f0.p1`, `t0.r1.c2`) are the ones the view prints and `edit` plans take; `edit` and `add` write `<stem>.edit/` and print the changed blocks (`read redline.docx --changed --by <author>`); defaults `Modified User`, now, `--existing-revisions auto`.

`README.md:380` row: `| \`jubarte read\` (\`jubarte FILE\`) | The agent view: YAML header, \`<!-- pN -->\` id lines, tracked changes and comments with their ids; \`-p\`, \`--head\`, \`--tail\`, \`--changed\` |`; add rows for `jubarte edit FILE --replace …` and `jubarte add`; in the compare section note that `jubarte A B` without `-o` prints the redline's agent view and `compare A B` writes `<A>_v_<B>.docx`.

`skills/jubarte-documents/SKILL.md`: the Read row becomes `jubarte FILE.docx` (or `read`), the Edit row shows the flag form first and the plan form second, and a new Add row; the id note says `p12`, `h0`, `f0.p1`, `t0.r1.c2`.

`CHANGELOG.md`, unreleased, `### Added`: `jubarte FILE prints the agent view; jubarte A B prints the agent view of their redline (write it with -o; compare A B still writes <A>_v_<B>.docx). read --changed [--by AUTHOR]. edit --replace/--delete/--comment and the add command edit without a plan (author Modified User, --out-dir <stem>.edit, --existing-revisions auto). Short ids p12, h0, f0.p1, t0.r1.c2 in edit selectors.` `### Changed`: `edit prints the redline's changed blocks as the agent view instead of the patch (patch.diff is still written). jubarte A B no longer writes a file unless -o is given.`

- [ ] **Step 7: Whole suite, clippy, commit**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: clean.

```bash
git add src/cli.rs src/bin/jubarte.rs tests/cli_parser.rs tests/agent_text_view.rs docs/MARKDOWN.md README.md skills/jubarte-documents/SKILL.md CHANGELOG.md
git commit -m "feat(cli): jubarte FILE reads, jubarte A B prints the redline view; docs for read, edit flags and add"
```

---

## Self-review of Tasks 16-19

Consistency with Tasks 1-15: `Select::Changed` extends the Task 1 enum and the Task 12 matcher without touching `Head`, `Tail` or `Picks`; `ReadArgs` is Task 13's struct, flattened into `Cli` in Task 19 and extended in Task 17; `run_text` keeps its signature `(file, &ReadArgs)` and loses its body to `print_agent_view`, which `run` (compare) also calls; `edit`'s stdout changes in Task 18 and Task 15's round-trip tests ignore it (they re-read with `read`). Short ids in Task 16 resolve to the long ids every report, `--id` filter and golden already use; the view never prints long ids, and `edit` never prints short ones.

Risks: clap's `num_args = 3` with `ArgAction::Append` flattens values (one `Vec<String>`), which `chunks_exact(3)` relies on; the implicit `place` group on `add`; `required_unless_present_any` with `conflicts_with_all` on `plan` (the defaulted `author`/`existing_revisions` do not count as present); the comparer's placement of an inserted paragraph mark (the `add` test pins only the paragraph's number and text); `Option::is_none_or` on an older toolchain.
