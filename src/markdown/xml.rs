// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The items a Markdown document becomes, and their WordprocessingML.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;

/// An inserted or a deleted tracked change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Insert,
    Delete,
}

/// A change's kind and its serial number, so two changes side by side stay
/// two revisions.
pub(super) type Change = (Kind, u32);

/// Direct run formatting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RunFormat {
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub underline: bool,
    pub superscript: bool,
    pub subscript: bool,
    /// `<mark>`: a yellow highlight of its own.
    pub mark: bool,
    pub code: bool,
    pub link: bool,
}

/// What a run holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Content {
    Text(String),
    Break,
    /// A page break (`w:br w:type="page"`).
    PageBreak,
    Tab,
    /// A footnote reference, by the footnote's position (1-based).
    NoteReference(u32),
    /// A picture, by its index in [`Document::pictures`].
    Picture(usize),
    /// An equation's LaTeX: display math (`$$…$$`) or inline (`$…$`).
    Math {
        latex: String,
        display: bool,
    },
    /// A body field's start: its code (`w:instrText`) between the begin and
    /// separate characters; its result runs follow.
    FieldBegin(String),
    /// A body field's end character.
    FieldEnd,
}

/// A run: content, formatting, the change it belongs to and the highlight
/// group it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Run {
    pub content: Content,
    pub format: RunFormat,
    pub change: Option<Change>,
    pub highlight: Option<usize>,
}

/// Paragraph properties.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Paragraph {
    /// Its style: a built-in one the reader defines when the docx lacks it
    /// (`Heading1`), or the one an id line names (`<!-- p0 TitlePage -->`).
    pub style: Option<Cow<'static, str>>,
    /// A numbering instance (index into [`Document::lists`]) and level.
    pub numbering: Option<(usize, u32)>,
    pub indent: Option<u32>,
    /// The indents an id line names, in twips (`first-line 0.5in`); its
    /// `left` wins over `indent`.
    pub ind: Indent,
    pub align: Option<&'static str>,
    /// A heading level (`w:outlineLvl`, 0 for `#`) the paragraph carries
    /// itself, when its style is one an id line names.
    pub outline: Option<usize>,
    /// A thematic break: a bottom border on an empty paragraph.
    pub rule: bool,
    /// The paragraph mark's tracked change.
    pub mark: Option<Kind>,
    /// The serial of the change `mark` belongs to, for its attribution.
    pub mark_by: Option<u32>,
    /// `mark` was set because the paragraph's whole text is one change.
    pub whole: bool,
    /// The label an id line names (`num "(a)"`): an index into
    /// `Document::label_uses`.
    pub label: Option<usize>,
    /// An id line said what the mark is (`break-ins r4 @AC`, or nothing):
    /// the text does not change it.
    pub mark_fixed: bool,
    /// The paragraph ends a section (an id line's `section-break`).
    pub section_break: bool,
    /// The paragraph is a `#` heading: a link may name it by its slug.
    pub heading: bool,
    /// The bookmark a link to the heading jumps to (`the-intro`).
    pub bookmark: Option<String>,
    /// The formatting changes its id line names (`fmt r10 @AL`).
    pub formats: Vec<FormatMark>,
    /// The marks its id line names that its text shows nothing of (`rev
    /// r9 @AL`).
    pub unshown: Vec<FormatMark>,
}

/// A formatting change, or a mark whose text the view does not show, as
/// the view names it: its `w:id`, who made it and when. The view says who
/// changed the formatting, not what it was: the change keeps what is
/// there, so rejecting it changes nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FormatMark {
    pub id: Option<u32>,
    pub author: String,
    pub date: String,
}

/// The formatting changes a table line names: the table's own, a row's
/// (`in t0.row1`) and a cell's (`in t0.row1.cell0`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TableFormats {
    pub table: Option<FormatMark>,
    /// The grid's change (`w:tblGridChange`): an id, no author.
    pub grid: Option<u32>,
    /// A row's own insertion or deletion and a cell's own change, which no
    /// text shows.
    pub row_marks: HashMap<usize, (Kind, FormatMark)>,
    pub cell_marks: HashMap<(usize, usize), (super::view_notes::CellMark, FormatMark)>,
    pub rows: HashMap<usize, FormatMark>,
    pub cells: HashMap<(usize, usize), FormatMark>,
}

/// A paragraph's `w:ind`, side by side, in twips.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Indent {
    pub first_line: Option<i64>,
    pub hanging: Option<i64>,
    pub left: Option<i64>,
    pub right: Option<i64>,
}

/// A hyperlink's target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Link {
    External(String),
    Anchor(String),
}

/// One element of a story, in document order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Item {
    Paragraph(Paragraph),
    ParagraphEnd,
    Run(Run),
    /// Where a comment's range starts, by comment slot: most slots never
    /// get a comment and write nothing.
    RangeStart(usize),
    RangeEnd(u32),
    CommentReference(u32),
    /// A link and its title (`w:tooltip`).
    LinkStart(Link, Option<String>),
    /// A bookmark where it stands (`<a id="name"></a>`).
    Bookmark(String),
    LinkEnd,
    Table {
        columns: Vec<Option<&'static str>>,
        /// The table's own alignment (`jc`).
        align: Option<&'static str>,
        /// Its style (`tblStyle`); none is Word's Normal Table.
        style: Option<String>,
        formats: TableFormats,
    },
    Row {
        header: bool,
        change: Option<Change>,
        /// The grid columns the row leaves out before and after its cells.
        gaps: (usize, usize),
    },
    Cell {
        align: Option<&'static str>,
        /// The grid columns the cell spans (`gridSpan`), at least one.
        across: usize,
        /// A vertical merge: `Some(true)` where it starts, `Some(false)`
        /// where it continues (`vMerge`).
        merge: Option<bool>,
    },
    CellEnd,
    RowEnd,
    TableEnd,
}

/// A numbering instance: a bullet list, or an ordered list and where it
/// starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct List {
    pub ordered: bool,
    pub level: u32,
    pub start: u32,
}

/// An embedded picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Picture {
    pub bytes: Vec<u8>,
    pub extension: &'static str,
    pub content_type: &'static str,
    pub width: u64,
    pub height: u64,
    pub alt: String,
}

/// A footnote or an endnote: its items and the change of its reference,
/// which the note shares when its own text has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Note {
    pub items: Vec<Item>,
    pub change: Option<Change>,
    /// The view names it an endnote (`<!-- endnotes.p0 -->`).
    pub endnote: bool,
}

/// A comment's paragraphs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Comment {
    pub id: u32,
    pub paragraphs: Vec<String>,
    /// Author and date the comment names (`Ana (2026-01-02T03:04:00Z): ...`).
    pub by: Option<(String, String)>,
    /// The initials the view names the author by (`c5 @AC: ...`).
    pub initials: Option<String>,
    /// The number the view gives it (`c5`), kept as its `w:id`.
    pub number: Option<u32>,
    /// The comment it replies to (`c6 @AS re c5`): the view's number while
    /// reading, its `w:id` once read.
    pub parent: Option<u32>,
    /// Resolved (`c5 @AC resolved`).
    pub done: bool,
}

/// A Markdown document read into Word items.
#[derive(Clone, Debug, Default)]
pub(super) struct Document {
    pub body: Vec<Item>,
    /// Footnotes in reference order.
    pub notes: Vec<Note>,
    pub comments: Vec<Comment>,
    /// Comment slots: the comment each got, if any.
    pub slots: Vec<Option<u32>>,
    /// More comments on a slot's range, by slot: those that follow the
    /// first on the same text (`{==text==}{>>c1<<}{>>c2<<}`).
    pub shared: HashMap<usize, Vec<u32>>,
    /// Highlight groups that became a comment's range.
    pub commented: HashSet<usize>,
    pub pictures: Vec<Picture>,
    pub lists: Vec<List>,
    pub styles: HashSet<&'static str>,
    /// Styles id lines name (`<!-- p0 TitlePage -->`), by id.
    pub named_styles: BTreeSet<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    /// `authors: document_owner:`, the core properties' owner.
    pub owner: Option<super::view_notes::Owner>,
    /// The legend's `table:` style and its border, eighths of a point.
    pub table_style: Option<(String, Option<u32>)>,
    /// Table styles Word has no definition of here, each with the border
    /// the legend gives it: defined by name.
    pub table_styles: BTreeMap<String, Option<u32>>,
    /// The `track_changes:` key: Word's Track Changes switch.
    pub tracking: Option<bool>,
    pub warnings: Vec<String>,
    /// Author and date of changes that name theirs, by serial.
    pub attributions: HashMap<u32, (String, String)>,
    /// The `w:id`s of the marks of changes the view numbers, by serial, in
    /// order (`{>>r5+6 @AB<<}` on a change of two runs).
    pub revision_ids: HashMap<u32, Vec<u32>>,
    /// The date the last mark of a change takes, when its author's view
    /// dates them with a range (`2026-03-02..2026-03-04`).
    pub range_ends: HashMap<u32, String>,
    /// The formatting change each section's page names (`page: …; fmt r17
    /// @AL`), by section number from one.
    pub section_formats: HashMap<usize, FormatMark>,
    /// The `w:id`s formatting changes take: no fresh id takes them.
    pub reserved_ids: BTreeSet<u32>,
    /// The list labels id lines name (`num "(a)"`, `bullet`), each once.
    pub labels: Vec<Label>,
    /// Each paragraph's use of a label: one `w:num` each.
    pub label_uses: Vec<LabelUse>,
    /// Each section's page the view's YAML header names, by section number
    /// from 1 (`page:` the first, `sections:` the others).
    pub pages: BTreeMap<usize, super::view_notes::PageSetup>,
    /// Each section's headers and footers the view's YAML header names, by
    /// section number from 1.
    pub stories: BTreeMap<usize, super::view_stories::Stories>,
    /// Each handle of the view's `authors:` table, as the headers' and
    /// footers' changes name it: its name and date.
    pub people: HashMap<String, (String, String)>,
    /// The default and heading styles the view's `styles:` legend defines.
    pub style_specs: Vec<super::view_styles::StyleSpec>,
}

impl Document {
    /// The `w:sectPr` of section `number` (from 1) as the view names its
    /// page: its own, else the first section's, else `fallback`.
    pub(super) fn section(&self, number: usize, fallback: &str) -> String {
        let section = self
            .pages
            .get(&number)
            .or_else(|| self.pages.get(&1))
            .map_or_else(|| fallback.to_string(), |page| page.sect_pr());
        match self.section_formats.get(&number) {
            Some(mark) => with_section_change(&section, mark),
            None => section,
        }
    }

    /// How many paragraphs end a section.
    pub(super) fn section_breaks(&self) -> usize {
        self.body
            .iter()
            .filter(|item| matches!(item, Item::Paragraph(p) if p.section_break))
            .count()
    }
}

/// A paragraph numbered with a label: which one, at which list level, and
/// the number its list counts it (`2.` in `2. Text`), when it says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LabelUse {
    pub label: usize,
    pub level: u32,
    pub start: Option<u32>,
}

/// A list label an id line names, written as it reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Label {
    Bullet,
    Text(String),
}

/// Text for XML content: markup escaped, characters XML 1.0 forbids dropped.
pub(super) fn escape(text: &str) -> Cow<'_, str> {
    let clean = |c: char| {
        matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}')
            || c > '\u{FFFF}'
    };
    if !text
        .chars()
        .any(|c| matches!(c, '&' | '<' | '>' | '"') || !clean(c))
    {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars().filter(|c| clean(*c)) {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    Cow::Owned(out)
}

/// What a story needs from its package while it is written.
pub(super) trait Relate {
    /// The relationship id of an external hyperlink from this story's part.
    fn hyperlink(&mut self, url: &str) -> String;
    /// The relationship id of a picture from this story's part.
    fn picture(&mut self, index: usize) -> String;
}

/// Numbers shared by every story of one document.
pub(super) struct Context<'d> {
    pub document: &'d Document,
    pub author: String,
    pub date: String,
    /// The next fresh `w:id`, above every id the view names.
    pub next_revision: u32,
    /// How many marks of each numbered change are written, by serial.
    pub cursors: HashMap<u32, usize>,
    /// Every revision `w:id` written so far: none is written twice, not
    /// even one the view names twice (`r43+43`), which Word's own files
    /// never repeat (see `validate::ring1`).
    pub used_revisions: HashSet<u32>,
    /// `w:numId` of each numbering instance.
    pub num_ids: Vec<u32>,
    /// `w:numId` of each label use, by its index in `Document::label_uses`.
    pub label_nums: Vec<u32>,
    /// Each note's `w:id` in its part, by position (a reference's, less
    /// one).
    pub note_ids: Vec<u32>,
    /// The story written is an endnote's.
    pub in_endnote: bool,
    pub next_drawing: u32,
    /// The next bookmark's `w:id`.
    pub next_bookmark: u32,
    /// Width available to a table, in twentieths of a point.
    pub text_width: u32,
    /// The body's sections ended so far, and the section a paragraph
    /// ending one takes when the view names no page.
    pub sections_ended: usize,
    pub default_section: String,
    /// The headers and footers each section's `w:sectPr` references.
    pub section_stories: super::view_stories::SectionStories,
}

impl Context<'_> {
    /// A `w:ins` or `w:del` start tag's name and attributes, with the
    /// author and date change `by` names, or the document's.
    /// The `w:id` of the next mark of change `by`: the ids the view names
    /// for it, in turn, while one is left that no mark has taken; else a
    /// fresh one above every id the view names.
    fn revision_id(&mut self, by: Option<u32>) -> u32 {
        if let Some(serial) = by
            && let Some(ids) = self.document.revision_ids.get(&serial)
        {
            let at = self.cursors.entry(serial).or_insert(0);
            if let Some(&id) = ids.get(*at) {
                *at += 1;
                if self.used_revisions.insert(id) {
                    return id;
                }
            }
        }
        // Fresh ids stay within the `int` Word reads, past the view's own.
        loop {
            if self.next_revision > super::view_notes::MOST_ID {
                self.next_revision = 0;
            }
            let id = self.next_revision;
            self.next_revision += 1;
            if self.used_revisions.insert(id) {
                return id;
            }
        }
    }

    /// A formatting change's start tag: `name` with its id (the view's,
    /// unless a mark took it) and its author and date.
    fn format_mark(&mut self, name: &str, mark: &FormatMark) -> String {
        let id = mark
            .id
            .filter(|&id| self.used_revisions.insert(id))
            .unwrap_or_else(|| self.revision_id(None));
        format!(
            "{name} w:id=\"{id}\" w:author=\"{}\"{}",
            escape(&mark.author),
            date_attribute(&mark.date)
        )
    }

    /// How many of the ids the view names for change `by` no mark has
    /// taken yet.
    fn ids_left(&self, by: u32) -> usize {
        let named = self.document.revision_ids.get(&by).map_or(0, Vec::len);
        named.saturating_sub(self.cursors.get(&by).copied().unwrap_or(0))
    }

    fn revision(&mut self, kind: Kind, by: Option<u32>) -> String {
        let id = self.revision_id(by);
        let name = match kind {
            Kind::Insert => "w:ins",
            Kind::Delete => "w:del",
        };
        let last_day = by
            .filter(|&serial| self.ids_left(serial) == 0)
            .and_then(|serial| self.document.range_ends.get(&serial));
        match by.and_then(|serial| self.document.attributions.get(&serial)) {
            Some((author, date)) => format!(
                "{name} w:id=\"{id}\" w:author=\"{}\"{}",
                escape(author),
                date_attribute(last_day.unwrap_or(date))
            ),
            None => format!(
                "{name} w:id=\"{id}\" w:author=\"{}\" w:date=\"{}\"",
                self.author, self.date
            ),
        }
    }

    fn highlighted(&self, run: &Run) -> bool {
        run.highlight
            .is_some_and(|group| !self.document.commented.contains(&group))
    }
}

/// Writes a story's items. `note` is the change of the footnote whose first
/// paragraph gets the note's reference mark, when this story is a note.
pub(super) fn story(
    context: &mut Context<'_>,
    items: &[Item],
    relate: &mut dyn Relate,
    note: Option<Option<Change>>,
) -> String {
    let items = &*split_for_ids(context, items);
    let mut out = String::new();
    // The open w:ins / w:del and the change it holds.
    let mut open: Option<Change> = None;
    let mut first_paragraph = note.is_some();
    let mut columns: Vec<Option<&'static str>> = Vec::new();
    // The bookmark the open paragraph ends.
    let mut bookmark: Option<u32> = None;
    // The marks the open paragraph's text shows nothing of.
    let mut unshown: Vec<FormatMark> = Vec::new();
    // Its formatting changes past the mark's and its own: its runs'.
    let mut run_formats: Vec<FormatMark> = Vec::new();
    // The open tables' formatting changes, and the row and cell each is in.
    let mut tables: Vec<(TableFormats, usize, usize)> = Vec::new();
    // The marks each change still opens on its own; a change the view
    // gives more ids splits at run edges until the counts agree
    // (`{--plain _italic_--}{>>r43+44 @AL<<}` is two marks).
    let mut natural = natural_marks(items);
    // The open row's own change: its mark carries the text the view tags
    // with the row's id alone.
    let mut row_change: Option<Change> = None;
    // The open cell's own insertion or deletion, and its id; likewise the
    // open row's, when the table line names it.
    let mut cell_change: Option<(Kind, u32)> = None;
    let mut row_own: Option<(Kind, u32)> = None;
    for (at, item) in items.iter().enumerate() {
        // A range on just a substitution's inserted side starts inside the
        // insertion as it ends there (`ends_inserted_side`): Word's shape,
        // which keeps the two sides together.
        if let Item::RangeStart(_) = item
            && row_change.is_none()
            && cell_change.is_none()
            && row_own.is_none()
            && let Some(side) = starts_inserted_side(&items[at + 1..])
            && open != Some(side)
        {
            close_mark(context, &natural, &mut out, &mut open);
            let (kind, by) = side;
            spend(&mut natural, by);
            let wrap = context.revision(kind, Some(by));
            let _ = write!(out, "<{wrap}>");
            open = Some(side);
        }
        // A bookmark, a comment's range marks and its reference sit inside
        // a `w:ins`/`w:del` when the change goes on after them
        // (`{--cut {==by a==}{>>c0<<} comment--}`); between two changes
        // they are in neither. A range that ends a substitution's inserted
        // side, its deleted side next, ends inside the insertion, as Word
        // writes it: between the two it would part them.
        let inside = match item {
            Item::Run(_) => true,
            Item::RangeEnd(_) if ends_inserted_side(&items[at + 1..], open) => true,
            Item::Bookmark(_)
            | Item::RangeStart(_)
            | Item::RangeEnd(_)
            | Item::CommentReference(_) => continues(&items[at + 1..], open),
            _ => false,
        };
        if !inside {
            close_mark(context, &natural, &mut out, &mut open);
        }
        match item {
            Item::Bookmark(name) => {
                let id = context.next_bookmark;
                context.next_bookmark += 1;
                let _ = write!(
                    out,
                    "<w:bookmarkStart w:id=\"{id}\" w:name=\"{}\"/><w:bookmarkEnd w:id=\"{id}\"/>",
                    escape(name)
                );
            }
            Item::Paragraph(paragraph) => {
                out.push_str("<w:p>");
                paragraph_properties(context, paragraph, &mut out);
                unshown.clone_from(&paragraph.unshown);
                run_formats = paragraph.formats.get(2..).unwrap_or_default().to_vec();
                if let Some(name) = &paragraph.bookmark {
                    let id = context.next_bookmark;
                    context.next_bookmark += 1;
                    let _ = write!(
                        out,
                        "<w:bookmarkStart w:id=\"{id}\" w:name=\"{}\"/>",
                        escape(name)
                    );
                    bookmark = Some(id);
                }
                if paragraph.mark.is_some()
                    && let Some(by) = paragraph.mark_by
                {
                    spend(&mut natural, by);
                }
                if first_paragraph {
                    first_paragraph = false;
                    let change = note.flatten();
                    let wrap = change.map(|(kind, by)| context.revision(kind, Some(by)));
                    if let Some(wrap) = &wrap {
                        let _ = write!(out, "<{wrap}>");
                    }
                    let (style, element) = if context.in_endnote {
                        ("EndnoteReference", "endnoteRef")
                    } else {
                        ("FootnoteReference", "footnoteRef")
                    };
                    let _ = write!(
                        out,
                        "<w:r><w:rPr><w:rStyle w:val=\"{style}\"/></w:rPr><w:{element}/></w:r>"
                    );
                    let text = if matches!(change, Some((Kind::Delete, _))) {
                        "delText"
                    } else {
                        "t"
                    };
                    let _ = write!(
                        out,
                        "<w:r><w:{text} xml:space=\"preserve\"> </w:{text}></w:r>"
                    );
                    // Word's one mark for the reference mark and the text
                    // after it, when the text is in the same change.
                    let text_change = items[at + 1..].iter().find_map(|item| match item {
                        Item::Run(run) => Some(run.change),
                        _ => None,
                    });
                    if let Some((kind, by)) = change {
                        if text_change == Some(change) {
                            spend(&mut natural, by);
                            open = change;
                        } else {
                            out.push_str(match kind {
                                Kind::Insert => "</w:ins>",
                                Kind::Delete => "</w:del>",
                            });
                        }
                    }
                }
            }
            Item::ParagraphEnd => {
                if let Some(id) = bookmark.take() {
                    let _ = write!(out, "<w:bookmarkEnd w:id=\"{id}\"/>");
                }
                // The view says who changed a run's formatting, not which
                // run: each change past the paragraph's own is a run of its
                // own, in order, and reads back in that order.
                for format in std::mem::take(&mut run_formats) {
                    let change = context.format_mark("w:rPrChange", &format);
                    let _ = write!(
                        out,
                        "<w:r><w:rPr><{change}><w:rPr/></w:rPrChange></w:rPr></w:r>"
                    );
                }
                // An empty insertion: Word keeps the mark, and shows no text.
                for mark in std::mem::take(&mut unshown) {
                    let mark = context.format_mark("w:ins", &mark);
                    let _ = write!(out, "<{mark}/>");
                }
                out.push_str("</w:p>");
            }
            Item::Run(Run {
                content: Content::Math { latex, display },
                change,
                ..
            }) => {
                // Word marks a change inside the math, run by run.
                close_mark(context, &natural, &mut out, &mut open);
                // A change inside the math has no serial: the document's
                // author and date.
                let change = *change;
                let mut mark = |kind: Kind| {
                    let by = change.filter(|(own, _)| *own == kind).map(|(_, by)| by);
                    context.revision(kind, by)
                };
                let kind = change.map(|(kind, _)| kind);
                out.push_str(&super::math::omml(latex, *display, kind, &mut mark));
            }
            Item::Run(run) => {
                // Text whose ids the view spent on the row's mark: the
                // row's mark is the change, and the text has none of its
                // own. A change the view names no ids for is Word's shape,
                // the row's mark and the text's.
                // Each cell's text is its own change (`{++x1++}{>>r108<<}`
                // twice): one that names only the row's ids is carried too.
                let carried = match (row_change, run.change) {
                    (Some((row_kind, row_by)), Some((kind, by))) if row_kind == kind => {
                        let named = |serial: u32| {
                            context
                                .document
                                .revision_ids
                                .get(&serial)
                                .filter(|ids| !ids.is_empty())
                        };
                        match (named(row_by), named(by)) {
                            (Some(_), Some(_)) if by == row_by => context.ids_left(by) == 0,
                            (Some(row_ids), Some(ids)) => ids.iter().all(|id| row_ids.contains(id)),
                            _ => false,
                        }
                    }
                    _ => false,
                } || [cell_change, row_own].into_iter().any(|own| {
                    // Likewise a cell's or a row's own insertion or deletion.
                    match (own, run.change) {
                        (Some((own_kind, id)), Some((kind, by))) if own_kind == kind => context
                            .document
                            .revision_ids
                            .get(&by)
                            .is_some_and(|ids| ids.as_slice() == [id]),
                        _ => false,
                    }
                });
                if carried {
                    close_mark(context, &natural, &mut out, &mut open);
                } else if open != run.change {
                    close_mark(context, &natural, &mut out, &mut open);
                    if let Some((kind, by)) = run.change {
                        spend(&mut natural, by);
                        let wrap = context.revision(kind, Some(by));
                        let _ = write!(out, "<{wrap}>");
                    }
                    open = run.change;
                } else if let Some((kind, by)) = run.change
                    && context.ids_left(by) > natural.get(&by).copied().unwrap_or(0)
                {
                    // The change goes on in a new mark: no empty marks yet.
                    out.push_str(match kind {
                        Kind::Insert => "</w:ins>",
                        Kind::Delete => "</w:del>",
                    });
                    let wrap = context.revision(kind, Some(by));
                    let _ = write!(out, "<{wrap}>");
                    open = run.change;
                }
                if carried {
                    // Plain text: the row's own mark says it is changed.
                    let plain = Run {
                        change: None,
                        ..run.clone()
                    };
                    write_run(context, &plain, relate, &mut out);
                } else {
                    write_run(context, run, relate, &mut out);
                }
            }
            Item::RangeStart(slot) => {
                if let Some(Some(id)) = context.document.slots.get(*slot) {
                    let _ = write!(out, "<w:commentRangeStart w:id=\"{id}\"/>");
                }
                for id in context.document.shared.get(slot).into_iter().flatten() {
                    let _ = write!(out, "<w:commentRangeStart w:id=\"{id}\"/>");
                }
            }
            Item::RangeEnd(id) => {
                let _ = write!(out, "<w:commentRangeEnd w:id=\"{id}\"/>");
            }
            Item::CommentReference(id) => {
                let _ = write!(
                    out,
                    "<w:r><w:rPr><w:rStyle w:val=\"CommentReference\"/></w:rPr>\
                     <w:commentReference w:id=\"{id}\"/></w:r>"
                );
            }
            Item::LinkStart(link, title) => {
                let tooltip = title
                    .as_deref()
                    .map(|t| format!(" w:tooltip=\"{}\"", escape(t)))
                    .unwrap_or_default();
                match link {
                    Link::External(url) => {
                        let id = relate.hyperlink(url);
                        let _ = write!(out, "<w:hyperlink r:id=\"{id}\"{tooltip} w:history=\"1\">");
                    }
                    Link::Anchor(name) => {
                        let _ = write!(
                            out,
                            "<w:hyperlink w:anchor=\"{}\"{tooltip} w:history=\"1\">",
                            escape(name)
                        );
                    }
                }
            }
            Item::LinkEnd => out.push_str("</w:hyperlink>"),
            Item::Table {
                columns: these,
                align,
                style,
                formats,
            } => {
                columns.clone_from(these);
                let count = u32::try_from(columns.len().max(1)).unwrap_or(1);
                let mut properties = String::new();
                if let Some(style) = style {
                    let _ = write!(properties, "<w:tblStyle w:val=\"{}\"/>", escape(style));
                }
                properties.push_str("<w:tblW w:w=\"5000\" w:type=\"pct\"/>");
                if let Some(align) = align {
                    let _ = write!(properties, "<w:jc w:val=\"{align}\"/>");
                }
                properties.push_str(
                    "<w:tblLook w:val=\"04A0\" w:firstRow=\"1\" w:lastRow=\"0\" \
                     w:firstColumn=\"1\" w:lastColumn=\"0\" w:noHBand=\"0\" w:noVBand=\"1\"/>",
                );
                let _ = write!(out, "<w:tbl><w:tblPr>{properties}");
                if let Some(format) = &formats.table {
                    let change = context.format_mark("w:tblPrChange", format);
                    let _ = write!(
                        out,
                        "<{change}><w:tblPr>{properties}</w:tblPr></w:tblPrChange>"
                    );
                }
                out.push_str("</w:tblPr><w:tblGrid>");
                tables.push((formats.clone(), 0, 0));
                let mut grid = String::new();
                for _ in 0..count {
                    let _ = write!(grid, "<w:gridCol w:w=\"{}\"/>", context.text_width / count);
                }
                out.push_str(&grid);
                // The grid's change keeps the grid as it is, and names no
                // author: Word's `w:tblGridChange` has none.
                if let Some(id) = formats.grid {
                    let id = Some(id)
                        .filter(|&id| context.used_revisions.insert(id))
                        .unwrap_or_else(|| context.revision_id(None));
                    let _ = write!(
                        out,
                        "<w:tblGridChange w:id=\"{id}\"><w:tblGrid>{grid}</w:tblGrid></w:tblGridChange>"
                    );
                }
                out.push_str("</w:tblGrid>");
            }
            Item::Row {
                header,
                change,
                gaps: (before, after),
            } => {
                out.push_str("<w:tr>");
                row_change = *change;
                let (format, own) = tables
                    .last_mut()
                    .map(|(formats, row, cell)| {
                        let found = (
                            formats.rows.get(row).cloned(),
                            formats.row_marks.get(row).cloned(),
                        );
                        *row += 1;
                        *cell = 0;
                        found
                    })
                    .unwrap_or_default();
                // A row's own mark the text does not show: its change, when
                // the text shows one, is that.
                let own = own.filter(|_| change.is_none());
                row_own = own
                    .as_ref()
                    .and_then(|(kind, mark)| mark.id.map(|id| (*kind, id)));
                let gapped = *before > 0 || *after > 0;
                if *header || change.is_some() || own.is_some() || format.is_some() || gapped {
                    // What the row has, which its formatting change keeps.
                    let mut base = String::new();
                    let count = u32::try_from(columns.len().max(1)).unwrap_or(1);
                    let width =
                        |n: usize| context.text_width / count * u32::try_from(n).unwrap_or(0);
                    if *before > 0 {
                        let _ = write!(base, "<w:gridBefore w:val=\"{before}\"/>");
                    }
                    if *after > 0 {
                        let _ = write!(base, "<w:gridAfter w:val=\"{after}\"/>");
                    }
                    if *before > 0 {
                        let _ = write!(
                            base,
                            "<w:wBefore w:w=\"{}\" w:type=\"dxa\"/>",
                            width(*before)
                        );
                    }
                    if *after > 0 {
                        let _ =
                            write!(base, "<w:wAfter w:w=\"{}\" w:type=\"dxa\"/>", width(*after));
                    }
                    if *header {
                        base.push_str("<w:tblHeader/>");
                    }
                    let _ = write!(out, "<w:trPr>{base}");
                    if let Some((kind, by)) = change {
                        spend(&mut natural, *by);
                        let mark = context.revision(*kind, Some(*by));
                        let _ = write!(out, "<{mark}/>");
                    }
                    if let Some((kind, mark)) = &own {
                        let name = match kind {
                            Kind::Insert => "w:ins",
                            Kind::Delete => "w:del",
                        };
                        let written = context.format_mark(name, mark);
                        let _ = write!(out, "<{written}/>");
                    }
                    if let Some(format) = &format {
                        let change = context.format_mark("w:trPrChange", format);
                        let _ = write!(out, "<{change}><w:trPr>{base}</w:trPr></w:trPrChange>");
                    }
                    out.push_str("</w:trPr>");
                }
            }
            Item::Cell { across, merge, .. } => {
                let count = u32::try_from(columns.len().max(1)).unwrap_or(1);
                let across = u32::try_from(*across).unwrap_or(1).max(1);
                let _ = write!(
                    out,
                    "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/>",
                    context.text_width / count * across
                );
                if across > 1 {
                    let _ = write!(out, "<w:gridSpan w:val=\"{across}\"/>");
                }
                match merge {
                    Some(true) => out.push_str("<w:vMerge w:val=\"restart\"/>"),
                    Some(false) => out.push_str("<w:vMerge/>"),
                    None => {}
                }
                let (format, own) = tables
                    .last_mut()
                    .map(|(formats, row, cell)| {
                        let at = (row.saturating_sub(1), *cell);
                        *cell += 1;
                        (
                            formats.cells.get(&at).cloned(),
                            formats.cell_marks.get(&at).cloned(),
                        )
                    })
                    .unwrap_or_default();
                // The cell's properties as they are, without its changes.
                let open = out
                    .rfind("<w:tcPr>")
                    .map_or(out.len(), |at| at + "<w:tcPr>".len());
                let old = out[open..].to_string();
                cell_change = None;
                if let Some((kind, mark)) = &own {
                    use super::view_notes::CellMark;
                    let name = match kind {
                        CellMark::Insert => "w:cellIns",
                        CellMark::Delete => "w:cellDel",
                        CellMark::Merge => "w:cellMerge",
                    };
                    let written = context.format_mark(name, mark);
                    let _ = write!(out, "<{written}/>");
                    cell_change = match kind {
                        CellMark::Insert => mark.id.map(|id| (Kind::Insert, id)),
                        CellMark::Delete => mark.id.map(|id| (Kind::Delete, id)),
                        CellMark::Merge => None,
                    };
                }
                if let Some(format) = &format {
                    let change = context.format_mark("w:tcPrChange", format);
                    let _ = write!(out, "<{change}><w:tcPr>{old}</w:tcPr></w:tcPrChange>");
                }
                out.push_str("</w:tcPr>");
            }
            Item::CellEnd => {
                cell_change = None;
                out.push_str("</w:tc>");
            }
            Item::RowEnd => {
                row_change = None;
                row_own = None;
                out.push_str("</w:tr>");
            }
            Item::TableEnd => {
                tables.pop();
                out.push_str("</w:tbl>");
            }
        }
    }
    close_mark(context, &natural, &mut out, &mut open);
    out
}

/// Whether the next run after a comment's marks continues the open change.
fn continues(rest: &[Item], open: Option<Change>) -> bool {
    open.is_some()
        && rest
            .iter()
            .find(|item| {
                !matches!(
                    item,
                    Item::Bookmark(_)
                        | Item::RangeStart(_)
                        | Item::RangeEnd(_)
                        | Item::CommentReference(_)
                )
            })
            .is_some_and(|item| matches!(item, Item::Run(run) if run.change == open))
}

/// Ends the open mark. A change the view gives more ids than it has pieces
/// left writes the rest as empty marks after it: `{++a++}{>>r4+5+6 @AL<<}`
/// is `a` in mark 4, then empty marks 5 and 6.
/// The insertion `rest` starts with, when its runs are all a comment range
/// holds before it ends (`ends_inserted_side`) and a deleted run follows.
fn starts_inserted_side(rest: &[Item]) -> Option<Change> {
    let Some(Item::Run(Run {
        change: Some(side @ (Kind::Insert, _)),
        ..
    })) = rest.first()
    else {
        return None;
    };
    let runs = rest
        .iter()
        .take_while(|item| matches!(item, Item::Run(run) if run.change == Some(*side)))
        .count();
    (matches!(rest.get(runs), Some(Item::RangeEnd(_)))
        && ends_inserted_side(&rest[runs + 1..], Some(*side)))
    .then_some(*side)
}

/// Whether `open` is an insertion and the next item a deleted run.
fn ends_inserted_side(rest: &[Item], open: Option<Change>) -> bool {
    matches!(open, Some((Kind::Insert, _)))
        && matches!(
            rest.first(),
            Some(Item::Run(Run {
                change: Some((Kind::Delete, _)),
                ..
            }))
        )
}

fn close_mark(
    context: &mut Context<'_>,
    natural: &HashMap<u32, usize>,
    out: &mut String,
    open: &mut Option<Change>,
) {
    let Some((kind, by)) = open.take() else {
        return;
    };
    out.push_str(match kind {
        Kind::Insert => "</w:ins>",
        Kind::Delete => "</w:del>",
    });
    if natural.get(&by).copied().unwrap_or(0) == 0 {
        for _ in 0..context.ids_left(by) {
            let mark = context.revision(kind, Some(by));
            let _ = write!(out, "<{mark}/>");
        }
    }
}

/// The marks each numbered change opens in `items` when no change splits:
/// a run that does not continue the open mark, a paragraph mark, a row.
fn natural_marks(items: &[Item]) -> HashMap<u32, usize> {
    let mut marks = HashMap::new();
    let mut open: Option<Change> = None;
    for (at, item) in items.iter().enumerate() {
        match item {
            Item::Bookmark(_) => {}
            Item::RangeStart(_) | Item::RangeEnd(_) | Item::CommentReference(_)
                if continues(&items[at + 1..], open) => {}
            Item::Run(Run {
                content: Content::Math { .. },
                ..
            }) => open = None,
            Item::Run(run) => {
                if open != run.change
                    && let Some((_, by)) = run.change
                {
                    *marks.entry(by).or_default() += 1;
                }
                open = run.change;
            }
            Item::Paragraph(paragraph) => {
                open = None;
                if paragraph.mark.is_some()
                    && let Some(by) = paragraph.mark_by
                {
                    *marks.entry(by).or_default() += 1;
                }
            }
            Item::Row {
                change: Some((_, by)),
                ..
            } => {
                open = None;
                *marks.entry(*by).or_default() += 1;
            }
            _ => open = None,
        }
    }
    marks
}

/// `items` with text runs split until each change has a piece for every id
/// the view names it: `{++one two++}{>>r70+71 @AL<<}` is two marks, `one `
/// and `two`. A piece is a run, a paragraph mark or a row; a run splits
/// after its last inner space, else before its last character.
fn split_for_ids<'i>(context: &Context<'_>, items: &'i [Item]) -> Cow<'i, [Item]> {
    let mut pieces: HashMap<u32, usize> = HashMap::new();
    for item in items {
        let by = match item {
            Item::Run(Run {
                change: Some((_, by)),
                ..
            })
            | Item::Row {
                change: Some((_, by)),
                ..
            } => *by,
            Item::Paragraph(paragraph) if paragraph.mark.is_some() => {
                let Some(by) = paragraph.mark_by else {
                    continue;
                };
                by
            }
            _ => continue,
        };
        *pieces.entry(by).or_default() += 1;
    }
    let mut short: Vec<(u32, usize)> = pieces
        .into_iter()
        .filter_map(|(by, have)| {
            let want = context.ids_left(by);
            (want > have).then(|| (by, want - have))
        })
        .collect();
    if short.is_empty() {
        return Cow::Borrowed(items);
    }
    short.sort_unstable();
    let mut out = items.to_vec();
    for (by, missing) in short {
        for _ in 0..missing {
            let found = out
                .iter()
                .enumerate()
                .rev()
                .find_map(|(at, item)| match item {
                    Item::Run(Run {
                        change: Some((_, own)),
                        content: Content::Text(text),
                        ..
                    }) if *own == by => split_point(text).map(|cut| (at, cut)),
                    _ => None,
                });
            let Some((at, cut)) = found else {
                break;
            };
            let Item::Run(run) = &out[at] else {
                break;
            };
            let Content::Text(text) = &run.content else {
                break;
            };
            let (head_text, tail_text) = text.split_at(cut);
            let mut head = run.clone();
            let mut tail = run.clone();
            head.content = Content::Text(head_text.to_string());
            tail.content = Content::Text(tail_text.to_string());
            out[at] = Item::Run(head);
            out.insert(at + 1, Item::Run(tail));
        }
    }
    Cow::Owned(out)
}

/// Where a run's text splits in two: after its last space with text after
/// it, else before its last character; `None` for one character.
pub(super) fn split_point(text: &str) -> Option<usize> {
    text.char_indices()
        .rev()
        .find(|&(at, c)| c == ' ' && at + 1 < text.len())
        .map(|(at, _)| at + 1)
        .or_else(|| {
            text.char_indices()
                .last()
                .map(|(at, _)| at)
                .filter(|&at| at > 0)
        })
}

/// One natural mark of change `by` is written.
fn spend(natural: &mut HashMap<u32, usize>, by: u32) {
    if let Some(left) = natural.get_mut(&by) {
        *left = left.saturating_sub(1);
    }
}

fn paragraph_properties(context: &mut Context<'_>, paragraph: &Paragraph, out: &mut String) {
    let mut properties = String::new();
    if let Some(style) = &paragraph.style {
        let _ = write!(properties, "<w:pStyle w:val=\"{}\"/>", escape(style));
    }
    if let Some((instance, level)) = paragraph.numbering {
        let num = context.num_ids.get(instance).copied().unwrap_or_default();
        let _ = write!(
            properties,
            "<w:numPr><w:ilvl w:val=\"{level}\"/><w:numId w:val=\"{num}\"/></w:numPr>"
        );
    } else if let Some(i) = paragraph.label
        && let (Some(num), Some(using)) = (
            context.label_nums.get(i),
            context.document.label_uses.get(i),
        )
    {
        let _ = write!(
            properties,
            "<w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"{num}\"/></w:numPr>",
            using.level
        );
    }
    if paragraph.rule {
        properties.push_str(
            "<w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"auto\"/></w:pBdr>",
        );
    }
    let ind = paragraph.ind;
    let left = ind.left.or(paragraph.indent.map(i64::from));
    if left.is_some() || ind.right.is_some() || ind.first_line.is_some() || ind.hanging.is_some() {
        properties.push_str("<w:ind");
        for (name, value) in [
            ("left", left),
            ("right", ind.right),
            ("firstLine", ind.first_line),
            ("hanging", ind.hanging),
        ] {
            if let Some(value) = value {
                let _ = write!(properties, " w:{name}=\"{value}\"");
            }
        }
        properties.push_str("/>");
    }
    if let Some(align) = paragraph.align {
        let _ = write!(properties, "<w:jc w:val=\"{align}\"/>");
    }
    if let Some(level) = paragraph.outline {
        let _ = write!(properties, "<w:outlineLvl w:val=\"{level}\"/>");
    }
    // What the paragraph has, which its formatting change keeps.
    let base = properties.clone();
    // Word reads the mark's change before the paragraph's: with two, the
    // first is the mark's.
    let (mark_format, paragraph_format) = match paragraph.formats.as_slice() {
        [] => (None, None),
        [one] => (None, Some(one)),
        [first, second, ..] => (Some(first), Some(second)),
    };
    let mut mark_properties = String::new();
    if let Some(kind) = paragraph.mark {
        let mark = context.revision(kind, paragraph.mark_by);
        let _ = write!(mark_properties, "<{mark}/>");
    }
    if let Some(format) = mark_format {
        let change = context.format_mark("w:rPrChange", format);
        let _ = write!(mark_properties, "<{change}><w:rPr/></w:rPrChange>");
    }
    if !mark_properties.is_empty() {
        let _ = write!(properties, "<w:rPr>{mark_properties}</w:rPr>");
    }
    if paragraph.section_break {
        context.sections_ended += 1;
        let number = context.sections_ended;
        let section = context.document.section(number, &context.default_section);
        properties.push_str(&context.section_stories.apply(number, &section));
    }
    if let Some(format) = paragraph_format {
        let change = context.format_mark("w:pPrChange", format);
        let _ = write!(properties, "<{change}><w:pPr>{base}</w:pPr></w:pPrChange>");
    }
    if !properties.is_empty() {
        let _ = write!(out, "<w:pPr>{properties}</w:pPr>");
    }
}

/// Whether the note a reference at `position` (1-based) points to is an
/// endnote.
fn is_endnote(context: &Context<'_>, position: u32) -> bool {
    usize::try_from(position.saturating_sub(1))
        .ok()
        .and_then(|at| context.document.notes.get(at))
        .is_some_and(|note| note.endnote)
}

fn run_properties(context: &Context<'_>, run: &Run) -> String {
    let format = run.format;
    let mut out = String::new();
    let style = match &run.content {
        Content::NoteReference(position) if is_endnote(context, *position) => {
            Some("EndnoteReference")
        }
        Content::NoteReference(_) => Some("FootnoteReference"),
        _ if format.link => Some("Hyperlink"),
        _ if format.code => Some("VerbatimChar"),
        _ => None,
    };
    if let Some(style) = style {
        let _ = write!(out, "<w:rStyle w:val=\"{style}\"/>");
    }
    // A run has one character style: code in a link takes the face
    // `VerbatimChar` gives, so it still reads as code.
    if format.code && style != Some("VerbatimChar") {
        out.push_str("<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/>");
    }
    if format.bold {
        out.push_str("<w:b/><w:bCs/>");
    }
    if format.italic {
        out.push_str("<w:i/><w:iCs/>");
    }
    if format.strike {
        out.push_str("<w:strike/>");
    }
    if format.mark || context.highlighted(run) {
        out.push_str("<w:highlight w:val=\"yellow\"/>");
    }
    if format.underline {
        out.push_str("<w:u w:val=\"single\"/>");
    }
    if format.superscript {
        out.push_str("<w:vertAlign w:val=\"superscript\"/>");
    } else if format.subscript {
        out.push_str("<w:vertAlign w:val=\"subscript\"/>");
    }
    if out.is_empty() {
        out
    } else {
        format!("<w:rPr>{out}</w:rPr>")
    }
}

fn write_run(context: &mut Context<'_>, run: &Run, relate: &mut dyn Relate, out: &mut String) {
    let properties = run_properties(context, run);
    let deleted = matches!(run.change, Some((Kind::Delete, _)));
    let _ = write!(out, "<w:r>{properties}");
    match &run.content {
        Content::Text(text) => {
            let element = if deleted { "w:delText" } else { "w:t" };
            let _ = write!(
                out,
                "<{element} xml:space=\"preserve\">{}</{element}>",
                escape(text)
            );
        }
        Content::Break => out.push_str("<w:br/>"),
        Content::PageBreak => out.push_str("<w:br w:type=\"page\"/>"),
        Content::Tab => out.push_str("<w:tab/>"),
        Content::FieldBegin(code) => {
            let element = if deleted {
                "w:delInstrText"
            } else {
                "w:instrText"
            };
            let _ = write!(
                out,
                "<w:fldChar w:fldCharType=\"begin\"/></w:r><w:r>{properties}\
                 <{element} xml:space=\"preserve\"> {} </{element}></w:r><w:r>{properties}\
                 <w:fldChar w:fldCharType=\"separate\"/>",
                escape(code)
            );
        }
        Content::FieldEnd => out.push_str("<w:fldChar w:fldCharType=\"end\"/>"),
        Content::NoteReference(position) => {
            let at = usize::try_from(position.saturating_sub(1)).unwrap_or(usize::MAX);
            let element = if is_endnote(context, *position) {
                "endnoteReference"
            } else {
                "footnoteReference"
            };
            let id = context.note_ids.get(at).copied().unwrap_or(*position);
            let _ = write!(out, "<w:{element} w:id=\"{id}\"/>");
        }
        // The story writes math outside any run.
        Content::Math { .. } => {}
        Content::Picture(index) => {
            let id = relate.picture(*index);
            let drawing = context.next_drawing;
            context.next_drawing += 1;
            if let Some(picture) = context.document.pictures.get(*index) {
                drawing_xml(picture, &id, drawing, out);
            }
        }
    }
    out.push_str("</w:r>");
}

/// A `w:drawing` holding `picture` inline, embedded through `rel`, with
/// drawing id `id`. The caller declares the `w`, `wp` and `r` prefixes.
pub(crate) fn drawing_xml(picture: &Picture, rel: &str, id: u32, out: &mut String) {
    let alt = escape(&picture.alt);
    let (cx, cy) = (picture.width, picture.height);
    let _ = write!(
        out,
        "<w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
         <wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>\
         <wp:docPr id=\"{id}\" name=\"Picture {id}\" descr=\"{alt}\"/>\
         <wp:cNvGraphicFramePr><a:graphicFrameLocks \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" noChangeAspect=\"1\"/>\
         </wp:cNvGraphicFramePr>\
         <a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
         <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <pic:nvPicPr><pic:cNvPr id=\"0\" name=\"Picture {id}\" descr=\"{alt}\"/><pic:cNvPicPr/></pic:nvPicPr>\
         <pic:blipFill><a:blip r:embed=\"{rel}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
         <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>\
         </a:graphicData></a:graphic></wp:inline></w:drawing>"
    );
}

/// A comments part holding `comments`.
pub(super) fn comments_part(context: &Context<'_>, comments: &[Comment]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <w:comments xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
    );
    let initials: String = context
        .author
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect();
    for comment in comments {
        match &comment.by {
            Some((author, date)) => {
                let initials: String = comment.initials.clone().unwrap_or_else(|| {
                    author
                        .split_whitespace()
                        .filter_map(|word| word.chars().next())
                        .collect()
                });
                let _ = write!(
                    out,
                    "<w:comment w:id=\"{}\" w:author=\"{}\"{} w:initials=\"{}\">",
                    comment.id,
                    escape(author),
                    date_attribute(date),
                    escape(&initials)
                );
            }
            None => {
                let _ = write!(
                    out,
                    "<w:comment w:id=\"{}\" w:author=\"{}\" w:date=\"{}\" w:initials=\"{}\">",
                    comment.id, context.author, context.date, initials
                );
            }
        }
        let mut paragraphs = comment.paragraphs.iter().map(|p| p.trim());
        let first = paragraphs.next().unwrap_or_default();
        let _ = write!(
            out,
            "<w:p><w:pPr><w:pStyle w:val=\"CommentText\"/></w:pPr>\
             <w:r><w:rPr><w:rStyle w:val=\"CommentReference\"/></w:rPr><w:annotationRef/></w:r>\
             <w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
            escape(first)
        );
        for paragraph in paragraphs.filter(|p| !p.is_empty()) {
            let _ = write!(
                out,
                "<w:p><w:pPr><w:pStyle w:val=\"CommentText\"/></w:pPr>\
                 <w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                escape(paragraph)
            );
        }
        out.push_str("</w:comment>");
    }
    out.push_str("</w:comments>");
    out
}

/// The format of every level of a list definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListFormat {
    /// `•`, `◦`, `▪` by level.
    Bullet,
    /// `1.`, with the level's own counter.
    Decimal,
    /// `a.`, with the level's own counter.
    LowerLetter,
}

/// A `w:abstractNum` with nine levels of `format`, each indented a
/// further half inch.
pub(crate) fn abstract_num(id: u32, format: ListFormat) -> String {
    const GLYPHS: [&str; 3] = ["\u{2022}", "\u{25E6}", "\u{25AA}"];
    let mut out = format!(
        "<w:abstractNum w:abstractNumId=\"{id}\"><w:multiLevelType w:val=\"hybridMultilevel\"/>"
    );
    for level in 0..9u32 {
        let (format, text) = match format {
            ListFormat::Bullet => ("bullet", GLYPHS[level as usize % GLYPHS.len()].to_string()),
            ListFormat::Decimal => ("decimal", format!("%{}.", level + 1)),
            ListFormat::LowerLetter => ("lowerLetter", format!("%{}.", level + 1)),
        };
        let _ = write!(
            out,
            "<w:lvl w:ilvl=\"{level}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{format}\"/>\
             <w:lvlText w:val=\"{text}\"/><w:lvlJc w:val=\"left\"/>\
             <w:pPr><w:ind w:left=\"{}\" w:hanging=\"360\"/></w:pPr></w:lvl>",
            720 * (level + 1)
        );
    }
    out.push_str("</w:abstractNum>");
    out
}

/// A `w:num` instance of `abstract_id`, optionally starting `level` at
/// `start`.
pub(crate) fn num(num_id: u32, abstract_id: u32, start: Option<(u32, u32)>) -> String {
    let mut out = format!("<w:num w:numId=\"{num_id}\"><w:abstractNumId w:val=\"{abstract_id}\"/>");
    if let Some((level, start)) = start {
        let _ = write!(
            out,
            "<w:lvlOverride w:ilvl=\"{level}\"><w:startOverride w:val=\"{start}\"/></w:lvlOverride>"
        );
    }
    out.push_str("</w:num>");
    out
}

/// The `w:abstractNum` elements for bullets (`bullets`) and for decimal
/// numbering (`decimal`), and one `w:num` per instance.
pub(super) fn numbering(
    lists: &[List],
    bullets: u32,
    decimal: u32,
    first_num: u32,
) -> (String, String) {
    let mut abstracts = String::new();
    for (id, ordered) in [(bullets, false), (decimal, true)] {
        if lists.iter().any(|list| list.ordered == ordered) {
            let format = if ordered {
                ListFormat::Decimal
            } else {
                ListFormat::Bullet
            };
            abstracts.push_str(&abstract_num(id, format));
        }
    }
    let mut nums = String::new();
    for (index, list) in lists.iter().enumerate() {
        let id = first_num + u32::try_from(index).unwrap_or(0);
        let abstract_id = if list.ordered { decimal } else { bullets };
        let start = list.ordered.then_some((list.level, list.start));
        nums.push_str(&num(id, abstract_id, start));
    }
    (abstracts, nums)
}

/// One list per label, every level's text the label itself (no `%1`),
/// so Word prints it as written; and one `w:num` per use, restarting its
/// level at the number the view gives it.
pub(super) fn label_numbering(
    labels: &[Label],
    uses: &[LabelUse],
    first_abstract: u32,
    first_num: u32,
) -> (String, String) {
    let mut abstracts = String::new();
    for (index, label) in (0u32..).zip(labels) {
        let (format, text) = match label {
            Label::Bullet => ("bullet", "\u{2022}"),
            Label::Text(text) => ("decimal", text.as_str()),
        };
        let _ = write!(
            abstracts,
            "<w:abstractNum w:abstractNumId=\"{}\"><w:multiLevelType w:val=\"multilevel\"/>",
            first_abstract + index
        );
        for level in 0..9 {
            let _ = write!(
                abstracts,
                "<w:lvl w:ilvl=\"{level}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{format}\"/>\
                 <w:lvlText w:val=\"{}\"/><w:lvlJc w:val=\"left\"/></w:lvl>",
                escape(text)
            );
        }
        abstracts.push_str("</w:abstractNum>");
    }
    let mut nums = String::new();
    for (index, using) in (0u32..).zip(uses) {
        let abstract_id = first_abstract + u32::try_from(using.label).unwrap_or(0);
        let start = using.start.map(|start| (using.level, start));
        nums.push_str(&num(first_num + index, abstract_id, start));
    }
    (abstracts, nums)
}

/// The definition of a style this writer uses, for a document that lacks it.
/// A paragraph style an id line names and the reader has no definition
/// for: on Normal, named by its id, so the view prints the id back.
pub(super) fn named_style_definition(id: &str) -> String {
    let id = escape(id);
    format!(
        "<w:style w:type=\"paragraph\" w:customStyle=\"1\" w:styleId=\"{id}\">\
         <w:name w:val=\"{id}\"/><w:basedOn w:val=\"Normal\"/><w:qFormat/></w:style>"
    )
}

pub(crate) fn style_definition(id: &str) -> Option<String> {
    // Heading 7-9 and Title are not written from Markdown; a `.doc`
    // converted through it restores them (`legacy_doc`).
    const HEADINGS: [(&str, &str); 9] = [
        ("32", ""),
        ("28", ""),
        ("24", ""),
        ("22", "<w:i/><w:iCs/>"),
        ("22", ""),
        ("22", "<w:i/><w:iCs/>"),
        ("22", ""),
        ("22", "<w:i/><w:iCs/>"),
        ("22", ""),
    ];
    let paragraph = |id: &str, name: &str, body: &str| {
        format!(
            "<w:style w:type=\"paragraph\" w:styleId=\"{id}\"><w:name w:val=\"{name}\"/>\
             <w:basedOn w:val=\"Normal\"/>{body}</w:style>"
        )
    };
    let character = |id: &str, name: &str, body: &str| {
        format!(
            "<w:style w:type=\"character\" w:styleId=\"{id}\"><w:name w:val=\"{name}\"/>\
             <w:basedOn w:val=\"DefaultParagraphFont\"/>{body}</w:style>"
        )
    };
    let mono = "<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\"/>";
    let tight = "<w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/>";
    Some(match id {
        "Normal" => "<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
                     <w:name w:val=\"Normal\"/><w:qFormat/></w:style>"
            .to_string(),
        "DefaultParagraphFont" => "<w:style w:type=\"character\" w:default=\"1\" \
                                   w:styleId=\"DefaultParagraphFont\"><w:name w:val=\"Default Paragraph Font\"/>\
                                   <w:uiPriority w:val=\"1\"/><w:semiHidden/><w:unhideWhenUsed/></w:style>"
            .to_string(),
        "TableNormal" => "<w:style w:type=\"table\" w:default=\"1\" w:styleId=\"TableNormal\">\
                          <w:name w:val=\"Normal Table\"/><w:uiPriority w:val=\"99\"/><w:semiHidden/>\
                          <w:unhideWhenUsed/><w:tblPr><w:tblInd w:w=\"0\" w:type=\"dxa\"/><w:tblCellMar>\
                          <w:top w:w=\"0\" w:type=\"dxa\"/><w:left w:w=\"108\" w:type=\"dxa\"/>\
                          <w:bottom w:w=\"0\" w:type=\"dxa\"/><w:right w:w=\"108\" w:type=\"dxa\"/>\
                          </w:tblCellMar></w:tblPr></w:style>"
            .to_string(),
        "NoList" => "<w:style w:type=\"numbering\" w:default=\"1\" w:styleId=\"NoList\">\
                     <w:name w:val=\"No List\"/><w:uiPriority w:val=\"99\"/><w:semiHidden/>\
                     <w:unhideWhenUsed/></w:style>"
            .to_string(),
        "Heading1" | "Heading2" | "Heading3" | "Heading4" | "Heading5" | "Heading6"
        | "Heading7" | "Heading8" | "Heading9" => {
            let level: usize = id[7..].parse().ok()?;
            let (size, extra) = HEADINGS[level - 1];
            format!(
                "<w:style w:type=\"paragraph\" w:styleId=\"{id}\"><w:name w:val=\"heading {level}\"/>\
                 <w:basedOn w:val=\"Normal\"/><w:next w:val=\"Normal\"/><w:uiPriority w:val=\"9\"/>\
                 <w:qFormat/><w:pPr><w:keepNext/><w:keepLines/>\
                 <w:spacing w:before=\"240\" w:after=\"80\"/><w:outlineLvl w:val=\"{}\"/></w:pPr>\
                 <w:rPr><w:b/><w:bCs/>{extra}<w:sz w:val=\"{size}\"/><w:szCs w:val=\"{size}\"/></w:rPr></w:style>",
                level - 1
            )
        }
        "Title" => paragraph(
            "Title",
            "Title",
            "<w:next w:val=\"Normal\"/><w:uiPriority w:val=\"10\"/><w:qFormat/><w:pPr>\
             <w:spacing w:after=\"80\" w:line=\"240\" w:lineRule=\"auto\"/><w:contextualSpacing/>\
             </w:pPr><w:rPr><w:spacing w:val=\"-10\"/><w:kern w:val=\"28\"/>\
             <w:sz w:val=\"56\"/><w:szCs w:val=\"56\"/></w:rPr>",
        ),
        "Quote" => paragraph(
            "Quote",
            "Quote",
            "<w:next w:val=\"Normal\"/><w:uiPriority w:val=\"29\"/><w:qFormat/><w:pPr>\
             <w:spacing w:before=\"200\" w:after=\"160\"/><w:ind w:left=\"864\" w:right=\"864\"/></w:pPr>\
             <w:rPr><w:i/><w:iCs/></w:rPr>",
        ),
        "ListParagraph" => paragraph(
            "ListParagraph",
            "List Paragraph",
            "<w:uiPriority w:val=\"34\"/><w:qFormat/><w:pPr><w:ind w:left=\"720\"/>\
             <w:contextualSpacing/></w:pPr>",
        ),
        "SourceCode" => paragraph(
            "SourceCode",
            "Source Code",
            &format!("<w:pPr>{tight}</w:pPr><w:rPr>{mono}<w:sz w:val=\"20\"/><w:szCs w:val=\"20\"/></w:rPr>"),
        ),
        "FootnoteText" | "EndnoteText" => paragraph(
            id,
            if id == "FootnoteText" {
                "footnote text"
            } else {
                "endnote text"
            },
            &format!(
                "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/><w:pPr>{tight}</w:pPr>\
                 <w:rPr><w:sz w:val=\"20\"/><w:szCs w:val=\"20\"/></w:rPr>"
            ),
        ),
        // Word's own: tabs at the middle and the end of a Letter line,
        // single spacing.
        "Header" | "Footer" => paragraph(
            id,
            if id == "Header" { "header" } else { "footer" },
            "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/>\
             <w:pPr><w:tabs><w:tab w:val=\"center\" w:pos=\"4680\"/>\
             <w:tab w:val=\"right\" w:pos=\"9360\"/></w:tabs>\
             <w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>",
        ),
        "CommentText" => paragraph(
            "CommentText",
            "annotation text",
            "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/>\
             <w:rPr><w:sz w:val=\"20\"/><w:szCs w:val=\"20\"/></w:rPr>",
        ),
        "VerbatimChar" => character("VerbatimChar", "Verbatim Char", &format!("<w:rPr>{mono}</w:rPr>")),
        "Hyperlink" => character(
            "Hyperlink",
            "Hyperlink",
            "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/>\
             <w:rPr><w:color w:val=\"0563C1\"/><w:u w:val=\"single\"/></w:rPr>",
        ),
        "FootnoteReference" | "EndnoteReference" => character(
            id,
            if id == "FootnoteReference" {
                "footnote reference"
            } else {
                "endnote reference"
            },
            "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/>\
             <w:rPr><w:vertAlign w:val=\"superscript\"/></w:rPr>",
        ),
        "CommentReference" => character(
            "CommentReference",
            "annotation reference",
            "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/>\
             <w:rPr><w:sz w:val=\"16\"/><w:szCs w:val=\"16\"/></w:rPr>",
        ),
        "TableGrid" => {
            let border = |side: &str| {
                format!("<w:{side} w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>")
            };
            let borders: String = ["top", "left", "bottom", "right", "insideH", "insideV"]
                .iter()
                .map(|side| border(side))
                .collect();
            format!(
                "<w:style w:type=\"table\" w:styleId=\"TableGrid\"><w:name w:val=\"Table Grid\"/>\
                 <w:basedOn w:val=\"TableNormal\"/><w:uiPriority w:val=\"39\"/>\
                 <w:pPr>{tight}</w:pPr><w:tblPr><w:tblBorders>{borders}</w:tblBorders></w:tblPr></w:style>"
            )
        }
        _ => return None,
    })
}

/// ` w:date="…"`, or nothing for an empty date: the attribute is optional,
/// and a note the view gave no exact date (an author with several, the
/// `authors:` table only spanning days) is written undated, never with an
/// invented one.
/// `section` with a `w:sectPrChange` that keeps its page as it is.
fn with_section_change(section: &str, mark: &FormatMark) -> String {
    let (Some(open), Some(close)) = (
        section
            .find("<w:sectPr")
            .and_then(|at| section[at..].find('>').map(|end| at + end + 1)),
        section.rfind("</w:sectPr>"),
    ) else {
        return section.to_string();
    };
    if open > close || section[..open].ends_with("/>") {
        return section.to_string();
    }
    // The page alone: a section's headers are no part of its change.
    let old: String = section[open..close]
        .split_inclusive("/>")
        .filter(|element| {
            !element.contains("<w:headerReference") && !element.contains("<w:footerReference")
        })
        .collect();
    format!(
        "{}<w:sectPrChange w:id=\"{}\" w:author=\"{}\"{}><w:sectPr>{old}</w:sectPr></w:sectPrChange>{}",
        &section[..close],
        mark.id.unwrap_or(super::view_notes::MOST_ID),
        escape(&mark.author),
        date_attribute(&mark.date),
        &section[close..]
    )
}

fn date_attribute(date: &str) -> String {
    if date.is_empty() {
        String::new()
    } else {
        format!(" w:date=\"{}\"", escape(date))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_run_splits_after_its_last_inner_space_else_before_its_last_character() {
        assert_eq!(split_point("one two"), Some(4));
        assert_eq!(split_point("a b c"), Some(4));
        // A trailing space has no text after it.
        assert_eq!(split_point("one "), Some(3));
        assert_eq!(split_point("née"), Some(3));
        assert_eq!(split_point("x"), None);
        assert_eq!(split_point(""), None);
    }

    #[test]
    fn escape_drops_characters_xml_forbids() {
        assert_eq!(escape("a < b & \"c\""), "a &lt; b &amp; &quot;c&quot;");
        assert_eq!(escape("bell\u{7}\u{FFFE}ok"), "bellok");
        assert!(matches!(escape("plain"), Cow::Borrowed(_)));
    }

    #[test]
    fn every_style_the_writer_names_has_a_definition() {
        for id in [
            "Normal",
            "DefaultParagraphFont",
            "TableNormal",
            "NoList",
            "Heading1",
            "Heading6",
            "Heading9",
            "Title",
            "Quote",
            "ListParagraph",
            "SourceCode",
            "FootnoteText",
            "EndnoteText",
            "CommentText",
            "VerbatimChar",
            "Hyperlink",
            "FootnoteReference",
            "EndnoteReference",
            "CommentReference",
            "TableGrid",
        ] {
            let definition = style_definition(id).unwrap();
            assert!(definition.contains(&format!("w:styleId=\"{id}\"")), "{id}");
        }
        assert!(style_definition("Heading10").is_none());
        assert!(style_definition("Unknown").is_none());
    }

    #[test]
    fn numbering_restarts_each_ordered_list_at_its_start() {
        let lists = [
            List {
                ordered: false,
                level: 0,
                start: 1,
            },
            List {
                ordered: true,
                level: 1,
                start: 4,
            },
        ];
        let (abstracts, nums) = numbering(&lists, 10, 11, 20);
        assert_eq!(abstracts.matches("<w:abstractNum ").count(), 2);
        assert!(nums.contains("<w:num w:numId=\"20\"><w:abstractNumId w:val=\"10\"/></w:num>"));
        assert!(nums.contains(
            "<w:num w:numId=\"21\"><w:abstractNumId w:val=\"11\"/>\
             <w:lvlOverride w:ilvl=\"1\"><w:startOverride w:val=\"4\"/></w:lvlOverride></w:num>"
        ));
        let (only_bullets, _) = numbering(&lists[..1], 10, 11, 20);
        assert_eq!(only_bullets.matches("<w:abstractNum ").count(), 1);
    }
}
