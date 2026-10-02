// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The items a Markdown document becomes, and their WordprocessingML.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
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
    pub code: bool,
    pub link: bool,
}

/// What a run holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Content {
    Text(String),
    Break,
    Tab,
    /// A footnote reference, by the footnote's position (1-based).
    NoteReference(u32),
    /// A picture, by its index in [`Document::pictures`].
    Picture(usize),
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
    pub style: Option<&'static str>,
    /// A numbering instance (index into [`Document::lists`]) and level.
    pub numbering: Option<(usize, u32)>,
    pub indent: Option<u32>,
    pub align: Option<&'static str>,
    /// A thematic break: a bottom border on an empty paragraph.
    pub rule: bool,
    /// The paragraph mark's tracked change.
    pub mark: Option<Kind>,
    /// The serial of the change `mark` belongs to, for its attribution.
    pub mark_by: Option<u32>,
    /// `mark` was set because the paragraph's whole text is one change.
    pub whole: bool,
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
    LinkStart(Link),
    LinkEnd,
    Table {
        columns: Vec<Option<&'static str>>,
    },
    Row {
        header: bool,
        change: Option<Change>,
    },
    Cell(Option<&'static str>),
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
pub(super) struct Picture {
    pub bytes: Vec<u8>,
    pub extension: &'static str,
    pub content_type: &'static str,
    pub width: u64,
    pub height: u64,
    pub alt: String,
}

/// A footnote: its items and the change of its reference, which the note
/// shares when its own text has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Note {
    pub items: Vec<Item>,
    pub change: Option<Change>,
}

/// A comment's paragraphs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Comment {
    pub id: u32,
    pub paragraphs: Vec<String>,
    /// Author and date the comment names (`Ana (2026-01-02T03:04:00Z): ...`).
    pub by: Option<(String, String)>,
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
    /// Highlight groups that became a comment's range.
    pub commented: HashSet<usize>,
    pub pictures: Vec<Picture>,
    pub lists: Vec<List>,
    pub styles: HashSet<&'static str>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub warnings: Vec<String>,
    /// Author and date of changes that name theirs, by serial.
    pub attributions: HashMap<u32, (String, String)>,
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
    pub next_revision: u32,
    /// `w:numId` of each numbering instance.
    pub num_ids: Vec<u32>,
    /// Added to a footnote's position to make its `w:id`.
    pub note_base: u32,
    pub next_drawing: u32,
    /// Width available to a table, in twentieths of a point.
    pub text_width: u32,
}

impl Context<'_> {
    /// A `w:ins` or `w:del` start tag's name and attributes, with the
    /// author and date change `by` names, or the document's.
    fn revision(&mut self, kind: Kind, by: Option<u32>) -> String {
        let id = self.next_revision;
        self.next_revision += 1;
        let name = match kind {
            Kind::Insert => "w:ins",
            Kind::Delete => "w:del",
        };
        match by.and_then(|serial| self.document.attributions.get(&serial)) {
            Some((author, date)) => format!(
                "{name} w:id=\"{id}\" w:author=\"{}\" w:date=\"{}\"",
                escape(author),
                escape(date)
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
    let mut out = String::new();
    // The open w:ins / w:del and the change it holds.
    let mut open: Option<Change> = None;
    let mut first_paragraph = note.is_some();
    let mut columns: Vec<Option<&'static str>> = Vec::new();
    let close = |out: &mut String, open: &mut Option<Change>| {
        if let Some((kind, _)) = open.take() {
            out.push_str(match kind {
                Kind::Insert => "</w:ins>",
                Kind::Delete => "</w:del>",
            });
        }
    };
    for item in items {
        if !matches!(item, Item::Run(_)) {
            close(&mut out, &mut open);
        }
        match item {
            Item::Paragraph(paragraph) => {
                out.push_str("<w:p>");
                paragraph_properties(context, paragraph, &mut out);
                if first_paragraph {
                    first_paragraph = false;
                    let change = note.flatten();
                    let wrap = change.map(|(kind, by)| context.revision(kind, Some(by)));
                    if let Some(wrap) = &wrap {
                        let _ = write!(out, "<{wrap}>");
                    }
                    out.push_str(
                        "<w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr>\
                         <w:footnoteRef/></w:r>",
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
                    if let Some((kind, _)) = change {
                        out.push_str(match kind {
                            Kind::Insert => "</w:ins>",
                            Kind::Delete => "</w:del>",
                        });
                    }
                }
            }
            Item::ParagraphEnd => out.push_str("</w:p>"),
            Item::Run(run) => {
                if open != run.change {
                    close(&mut out, &mut open);
                    if let Some((kind, by)) = run.change {
                        let wrap = context.revision(kind, Some(by));
                        let _ = write!(out, "<{wrap}>");
                    }
                    open = run.change;
                }
                write_run(context, run, relate, &mut out);
            }
            Item::RangeStart(slot) => {
                if let Some(Some(id)) = context.document.slots.get(*slot) {
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
            Item::LinkStart(Link::External(url)) => {
                let id = relate.hyperlink(url);
                let _ = write!(out, "<w:hyperlink r:id=\"{id}\" w:history=\"1\">");
            }
            Item::LinkStart(Link::Anchor(name)) => {
                let _ = write!(
                    out,
                    "<w:hyperlink w:anchor=\"{}\" w:history=\"1\">",
                    escape(name)
                );
            }
            Item::LinkEnd => out.push_str("</w:hyperlink>"),
            Item::Table { columns: these } => {
                columns.clone_from(these);
                let count = u32::try_from(columns.len().max(1)).unwrap_or(1);
                out.push_str(
                    "<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/>\
                     <w:tblW w:w=\"5000\" w:type=\"pct\"/>\
                     <w:tblLook w:val=\"04A0\" w:firstRow=\"1\" w:lastRow=\"0\" \
                     w:firstColumn=\"1\" w:lastColumn=\"0\" w:noHBand=\"0\" w:noVBand=\"1\"/>\
                     </w:tblPr><w:tblGrid>",
                );
                for _ in 0..count {
                    let _ = write!(out, "<w:gridCol w:w=\"{}\"/>", context.text_width / count);
                }
                out.push_str("</w:tblGrid>");
            }
            Item::Row { header, change } => {
                out.push_str("<w:tr>");
                if *header || change.is_some() {
                    out.push_str("<w:trPr>");
                    if *header {
                        out.push_str("<w:tblHeader/>");
                    }
                    if let Some((kind, by)) = change {
                        let mark = context.revision(*kind, Some(*by));
                        let _ = write!(out, "<{mark}/>");
                    }
                    out.push_str("</w:trPr>");
                }
            }
            Item::Cell(_) => {
                let count = u32::try_from(columns.len().max(1)).unwrap_or(1);
                let _ = write!(
                    out,
                    "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/></w:tcPr>",
                    context.text_width / count
                );
            }
            Item::CellEnd => out.push_str("</w:tc>"),
            Item::RowEnd => out.push_str("</w:tr>"),
            Item::TableEnd => out.push_str("</w:tbl>"),
        }
    }
    close(&mut out, &mut open);
    out
}

fn paragraph_properties(context: &mut Context<'_>, paragraph: &Paragraph, out: &mut String) {
    let mut properties = String::new();
    if let Some(style) = paragraph.style {
        let _ = write!(properties, "<w:pStyle w:val=\"{style}\"/>");
    }
    if let Some((instance, level)) = paragraph.numbering {
        let num = context.num_ids.get(instance).copied().unwrap_or_default();
        let _ = write!(
            properties,
            "<w:numPr><w:ilvl w:val=\"{level}\"/><w:numId w:val=\"{num}\"/></w:numPr>"
        );
    }
    if paragraph.rule {
        properties.push_str(
            "<w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"auto\"/></w:pBdr>",
        );
    }
    if let Some(indent) = paragraph.indent {
        let _ = write!(properties, "<w:ind w:left=\"{indent}\"/>");
    }
    if let Some(align) = paragraph.align {
        let _ = write!(properties, "<w:jc w:val=\"{align}\"/>");
    }
    if let Some(kind) = paragraph.mark {
        let mark = context.revision(kind, paragraph.mark_by);
        let _ = write!(properties, "<w:rPr><{mark}/></w:rPr>");
    }
    if !properties.is_empty() {
        let _ = write!(out, "<w:pPr>{properties}</w:pPr>");
    }
}

fn run_properties(context: &Context<'_>, run: &Run) -> String {
    let format = run.format;
    let mut out = String::new();
    let style = match &run.content {
        Content::NoteReference(_) => Some("FootnoteReference"),
        _ if format.link => Some("Hyperlink"),
        _ if format.code => Some("VerbatimChar"),
        _ => None,
    };
    if let Some(style) = style {
        let _ = write!(out, "<w:rStyle w:val=\"{style}\"/>");
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
    if context.highlighted(run) {
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
        Content::Tab => out.push_str("<w:tab/>"),
        Content::NoteReference(position) => {
            let _ = write!(
                out,
                "<w:footnoteReference w:id=\"{}\"/>",
                context.note_base + position
            );
        }
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

fn drawing_xml(picture: &Picture, rel: &str, id: u32, out: &mut String) {
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
                let initials: String = author
                    .split_whitespace()
                    .filter_map(|word| word.chars().next())
                    .collect();
                let _ = write!(
                    out,
                    "<w:comment w:id=\"{}\" w:author=\"{}\" w:date=\"{}\" w:initials=\"{}\">",
                    comment.id,
                    escape(author),
                    escape(date),
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

/// The definition of a style this writer uses, for a document that lacks it.
pub(crate) fn style_definition(id: &str) -> Option<String> {
    const HEADINGS: [(&str, &str); 6] = [
        ("32", ""),
        ("28", ""),
        ("24", ""),
        ("22", "<w:i/><w:iCs/>"),
        ("22", ""),
        ("22", "<w:i/><w:iCs/>"),
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
        "Heading1" | "Heading2" | "Heading3" | "Heading4" | "Heading5" | "Heading6" => {
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
        "FootnoteText" => paragraph(
            "FootnoteText",
            "footnote text",
            &format!(
                "<w:uiPriority w:val=\"99\"/><w:unhideWhenUsed/><w:pPr>{tight}</w:pPr>\
                 <w:rPr><w:sz w:val=\"20\"/><w:szCs w:val=\"20\"/></w:rPr>"
            ),
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
        "FootnoteReference" => character(
            "FootnoteReference",
            "footnote reference",
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

#[cfg(test)]
mod tests {
    use super::*;

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
            "Quote",
            "ListParagraph",
            "SourceCode",
            "FootnoteText",
            "CommentText",
            "VerbatimChar",
            "Hyperlink",
            "FootnoteReference",
            "CommentReference",
            "TableGrid",
        ] {
            let definition = style_definition(id).unwrap();
            assert!(definition.contains(&format!("w:styleId=\"{id}\"")), "{id}");
        }
        assert!(style_definition("Heading7").is_none());
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
