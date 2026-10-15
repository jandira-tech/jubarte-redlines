// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2024-2026 SylphxAI
//
// SPDX-License-Identifier: AGPL-3.0-only

//! CriticMarkup for tracked changes and comments.
//!
//! Converters record insertions, deletions, highlights and comments as flat
//! tokens in document order. [`Critic::into_inline`] then turns them into
//! markup that nests cleanly, so the simple regular expressions CriticMarkup is
//! designed for can parse it:
//!
//! - a span that crosses another is split so every span closes inside its parent;
//! - a span inside another of the same kind is merged into it (`{--{--x--}--}`
//!   would end at the first `--}`);
//! - spans with no text are dropped, keeping any comments they held;
//! - neighbouring spans of the same kind are joined;
//! - comments inside a highlight move to right after it (`{==text==}{>>note<<}`);
//! - a deletion next to an insertion becomes a substitution (`{~~old~>new~~}`).
//!
//! An insertion or deletion can carry who made it and when. That attribution
//! follows the change as a comment, which is how the CriticMarkup spec tracks
//! several authors: `{++new++}{>>Ana Lima (2026-09-29T14:05:00Z)<<}`. Neighbours
//! join only when their attributions match.
//!
//! Document text that happens to contain a CriticMarkup delimiter is escaped
//! with a Markdown backslash (`{\++`), which renders as the original characters
//! but no longer matches the delimiter.

use super::ooxml::{Inline, Marks};

/// A CriticMarkup span kind. Moves are an insertion at their destination and a
/// deletion at their source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    Insertion,
    Deletion,
    Highlight,
}

impl Mark {
    pub(crate) fn delimiters(self) -> (&'static str, &'static str) {
        match self {
            Self::Insertion => ("{++", "++}"),
            Self::Deletion => ("{--", "--}"),
            Self::Highlight => ("{==", "==}"),
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// A tracked change: its kind and who made it when, as the inside of its
/// `{>>…<<}` note (`Ana Lima (2026-09-29T14:05:00Z)`), already escaped.
pub(crate) type Change = (Mark, Option<String>);

/// Agent-view attribution: an internal tag (`0@AC`, `1+2@AC`,
/// `header1#1@AC`, optionally `0@AC 2026-10-03T14:05:00Z`) behind this
/// sentinel, so that it is never
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
/// author in one story (their ids differ by design).
fn same_author(a: Option<&str>, b: Option<&str>) -> bool {
    use super::agent::{handle_of, split_tag};
    match (tag(a), tag(b)) {
        (Some(a), Some(b)) => {
            let (a, b) = (tag_parts(a).0, tag_parts(b).0);
            handle_of(a) == handle_of(b) && split_tag(a).0 == split_tag(b).0
        }
        _ => a == b,
    }
}

/// The tags of two neighbours or of a substitution's two sides, joined,
/// keeping the first inline timestamp.
fn join_tagged(a: &str, b: &str) -> String {
    let (ta, da) = tag_parts(a);
    let (tb, db) = tag_parts(b);
    let mut out = format!(
        "{TAG}{}",
        super::agent::join_tags(&[ta.to_string(), tb.to_string()])
    );
    if let Some(date) = da.or(db) {
        out.push(' ');
        out.push_str(date);
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Leaf {
    /// Document text: escaped when rendered.
    Text {
        text: String,
        marks: Marks,
        link: Option<String>,
    },
    /// Markdown built by the converter (images, math, note references).
    Raw(String),
    /// The inside of a `{>>…<<}` comment, already escaped.
    Comment(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Leaf(Leaf),
    Open(Mark, Option<String>),
    Close(Mark),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Leaf(Leaf),
    Span(Mark, Option<String>, Vec<Node>),
}

impl Node {
    fn has_content(&self) -> bool {
        match self {
            Self::Leaf(Leaf::Comment(_)) => false,
            Self::Leaf(_) => true,
            Self::Span(_, _, children) => children.iter().any(Self::has_content),
        }
    }
}

/// Inline content of one paragraph with CriticMarkup spans.
#[derive(Debug, Default)]
pub(crate) struct Critic {
    tokens: Vec<Token>,
    /// Text written as is, for a document with no tracked change or comment:
    /// nothing can open a span there, so nothing needs escaping.
    plain: bool,
    /// The paragraph is joined to more text on the same line (a flattened
    /// table's cells): see [`space_into_last_change`].
    inline_end: bool,
    /// Agent view: runs of spaces stay as the file holds them, and Markdown
    /// syntax in the text is escaped once neighbouring runs are joined.
    agent: bool,
    /// A code block's paragraph: its text is written as is, spaces and all.
    code_block: bool,
}

impl Critic {
    /// A paragraph of a document with no tracked change or comment: its text
    /// is written unescaped.
    pub(crate) fn plain() -> Self {
        Self {
            plain: true,
            ..Self::default()
        }
    }

    /// Agent view: keeps runs of spaces as the file holds them and escapes
    /// Markdown syntax in the text. Word splits runs anywhere, so `==` or
    /// `~~` can straddle two runs: the escaping waits for the joined text.
    pub(crate) fn set_agent(&mut self) {
        self.agent = true;
    }

    /// A code block's paragraph (`SourceCode`): its text goes in a fence
    /// unescaped, with its runs of spaces.
    pub(crate) fn set_code_block(&mut self) {
        self.code_block = true;
    }

    /// Marks the paragraph as joined to more text on the same line.
    pub(crate) fn set_inline_end(&mut self) {
        self.inline_end = true;
    }

    pub(crate) fn push(&mut self, text: &str, bold: bool, italic: bool, link: Option<&str>) {
        let marks = Marks {
            bold,
            italic,
            ..Marks::default()
        };
        self.push_styled(text, marks, link);
    }

    /// [`Critic::push`] with underline, which only the agent view sets.
    pub(crate) fn push_styled(&mut self, text: &str, marks: Marks, link: Option<&str>) {
        if !text.is_empty() {
            self.tokens.push(Token::Leaf(Leaf::Text {
                text: text.to_string(),
                marks,
                link: link.map(str::to_string),
            }));
        }
    }

    pub(crate) fn raw(&mut self, markdown: &str) {
        if !markdown.is_empty() {
            self.tokens
                .push(Token::Leaf(Leaf::Raw(markdown.to_string())));
        }
    }

    /// Converter-built Markdown at token position `at` (see [`Critic::len`]),
    /// as the start of the paragraph: text right after it loses its leading
    /// space, up to the first marker, as the paragraph's own start would.
    pub(crate) fn insert_raw(&mut self, at: usize, markdown: &str) {
        for token in &mut self.tokens[at..] {
            match token {
                Token::Leaf(Leaf::Text { text, .. }) => {
                    let trimmed = text.trim_start().to_string();
                    let done = !trimmed.is_empty();
                    *text = trimmed;
                    if done {
                        break;
                    }
                }
                _ => break,
            }
        }
        self.tokens
            .insert(at, Token::Leaf(Leaf::Raw(markdown.to_string())));
    }

    /// Drops the last token when it is the raw `markdown`: a tag the next
    /// run reopens (`</sup><sup>`) stays open across both instead.
    pub(crate) fn reopen(&mut self, markdown: &str) -> bool {
        let last =
            matches!(self.tokens.last(), Some(Token::Leaf(Leaf::Raw(raw))) if raw == markdown);
        if last {
            self.tokens.pop();
        }
        last
    }

    /// The number of tokens so far, a position for [`Critic::insert_raw`].
    pub(crate) fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Drop the tokens pushed since [`Critic::len`] was `len`.
    pub(crate) fn truncate(&mut self, len: usize) {
        self.tokens.truncate(len);
    }

    pub(crate) fn comment(&mut self, note: &str) {
        self.tokens
            .push(Token::Leaf(Leaf::Comment(note.to_string())));
    }

    /// Opens a span; `by` is the attribution of an insertion or deletion.
    pub(crate) fn open(&mut self, mark: Mark, by: Option<&str>) {
        self.tokens.push(Token::Open(mark, by.map(str::to_string)));
    }

    pub(crate) fn close(&mut self, mark: Mark) {
        self.tokens.push(Token::Close(mark));
    }

    /// Drops every attribution. Inside a comment's own text an attribution would
    /// put a `{>>…<<}` inside another, and the outer one would end at the inner
    /// closer.
    pub(crate) fn unattributed(mut self) -> Self {
        for token in &mut self.tokens {
            if let Token::Open(_, by) = token {
                *by = None;
            }
        }
        self
    }

    /// True when there is no visible text and no comment.
    pub(crate) fn is_blank(&self) -> bool {
        self.tokens.iter().all(|token| match token {
            Token::Leaf(Leaf::Text { text, .. } | Leaf::Raw(text)) => text.trim().is_empty(),
            Token::Leaf(Leaf::Comment(_)) => false,
            Token::Open(..) | Token::Close(_) => true,
        })
    }

    pub(crate) fn into_inline(self) -> Inline {
        let mut nodes = tidy(tree(self.tokens));
        if self.inline_end {
            space_into_last_change(&mut nodes);
        }
        let escaping = Escaping {
            plain: self.plain,
            agent: self.agent,
            code_block: self.code_block,
        };
        let mut pieces = Vec::new();
        render(&nodes, &mut pieces, escaping, &mut 0);
        wrap_markers(&mut pieces);
        let mut inline = Inline::default();
        if self.code_block {
            inline.verbatim();
        } else if self.agent {
            inline.keep_spaces();
        }
        for piece in pieces {
            inline.push_styled(&piece.text, piece.marks, piece.link.as_deref());
        }
        inline
    }
}

/// A span still open while the tree is built.
type Open = (Mark, Option<String>, Vec<Node>);

/// Builds well-nested spans from flat tokens. A span of a kind that is already
/// open is merged into the open one, which keeps its attribution; a close that
/// would cross other spans closes them first and opens them again after it,
/// with the same attribution. Spans still open at the end close.
fn tree(tokens: Vec<Token>) -> Vec<Node> {
    let mut root = Vec::new();
    let mut open: Vec<Open> = Vec::new();
    let mut merged = [0usize; 3];
    for token in tokens {
        match token {
            Token::Leaf(leaf) => children(&mut root, &mut open).push(Node::Leaf(leaf)),
            Token::Open(mark, _) if open.iter().any(|(m, ..)| *m == mark) => {
                merged[mark.index()] += 1;
            }
            Token::Open(mark, by) => open.push((mark, by, Vec::new())),
            Token::Close(mark) if merged[mark.index()] > 0 => merged[mark.index()] -= 1,
            Token::Close(mark) => {
                let Some(at) = open.iter().position(|(m, ..)| *m == mark) else {
                    continue;
                };
                let reopen: Vec<(Mark, Option<String>)> = open[at + 1..]
                    .iter()
                    .map(|(m, by, _)| (*m, by.clone()))
                    .collect();
                close_from(&mut root, &mut open, at);
                open.extend(reopen.into_iter().map(|(m, by)| (m, by, Vec::new())));
            }
        }
    }
    close_from(&mut root, &mut open, 0);
    root
}

fn children<'a>(root: &'a mut Vec<Node>, open: &'a mut [Open]) -> &'a mut Vec<Node> {
    match open.last_mut() {
        Some((_, _, nodes)) => nodes,
        None => root,
    }
}

/// Closes the open spans from index `at` up, innermost first, into their parent.
fn close_from(root: &mut Vec<Node>, open: &mut Vec<Open>, at: usize) {
    let closed = open.split_off(at);
    let outermost = closed
        .into_iter()
        .rev()
        .fold(None, |inner, (mark, by, mut nodes)| {
            nodes.extend(inner);
            Some(Node::Span(mark, by, nodes))
        });
    children(root, open).extend(outermost);
}

/// Drops empty spans, joins neighbours of the same kind and attribution, and
/// moves comments out of highlights, at every level.
fn tidy(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for node in nodes {
        let Node::Span(mark, by, nodes) = node else {
            out.push(node);
            continue;
        };
        let mut comments = Vec::new();
        let nodes = if mark == Mark::Highlight {
            lift_comments(nodes, &mut comments)
        } else {
            nodes
        };
        if !nodes.iter().any(Node::has_content) {
            lift_comments(nodes, &mut out);
        } else if let Some(Node::Span(_, before_by, before)) = out.last_mut().filter(|last| {
            matches!(last, Node::Span(m, b, _) if *m == mark && same_author(b.as_deref(), by.as_deref()))
        }) {
            if let (Some(a), Some(b)) = (tag(before_by.as_deref()), tag(by.as_deref())) {
                *before_by = Some(join_tagged(a, b));
            }
            before.extend(nodes);
        } else {
            out.push(Node::Span(mark, by, nodes));
        }
        out.extend(comments);
    }
    join_text(
        out.into_iter()
            .map(|node| match node {
                Node::Span(mark, by, nodes) => Node::Span(mark, by, tidy(nodes)),
                leaf => leaf,
            })
            .collect(),
    )
}

/// Joins neighbouring text of the same formatting. Word splits runs anywhere,
/// so a delimiter can arrive in two pieces (`a -` and `-} b`); escaping the
/// joined text sees it whole.
fn join_text(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::with_capacity(nodes.len());
    for node in nodes {
        match (out.last_mut(), node) {
            (
                Some(Node::Leaf(Leaf::Text {
                    text: before,
                    marks: m,
                    link: l,
                })),
                Node::Leaf(Leaf::Text { text, marks, link }),
            ) if *m == marks && *l == link => {
                before.push_str(&text);
            }
            (_, node) => out.push(node),
        }
    }
    out
}

/// Moves the space before an insertion or deletion that ends the paragraph
/// into it: `in{++ nested++}`, not `in {++nested++}`. Both read the same once
/// accepted, but rejected the second leaves a trailing space, which shows
/// when more text follows on the same line (`in ; gone` in a flattened table).
fn space_into_last_change(nodes: &mut [Node]) {
    let [
        ..,
        Node::Leaf(Leaf::Text { text, .. }),
        Node::Span(Mark::Insertion | Mark::Deletion, by, inner),
    ] = nodes
    else {
        return;
    };
    // A tagged change (agent view) keeps its text exactly as the file holds it.
    if tag(by.as_deref()).is_some() {
        return;
    }
    // Only into text the change starts with, so the space keeps a format.
    let Some(Node::Leaf(Leaf::Text { text: first, .. })) = inner.first_mut() else {
        return;
    };
    let kept = text.trim_end().len();
    if kept == text.len() || kept == 0 {
        return;
    }
    first.insert_str(0, &text[kept..]);
    text.truncate(kept);
}

/// Removes every comment below `nodes` into `comments`, in order.
fn lift_comments(nodes: Vec<Node>, comments: &mut Vec<Node>) -> Vec<Node> {
    let mut kept = Vec::new();
    for node in nodes {
        match node {
            Node::Leaf(Leaf::Comment(_)) => comments.push(node),
            Node::Span(mark, by, nodes) => {
                kept.push(Node::Span(mark, by, lift_comments(nodes, comments)));
            }
            leaf => kept.push(leaf),
        }
    }
    kept
}

/// One piece of rendered inline text. Markers are CriticMarkup delimiters and
/// notes, which take a link only from the text around them.
struct Piece {
    text: String,
    marks: Marks,
    link: Option<String>,
    /// For a marker, the change or comment it belongs to: all of one change's
    /// markers join a link, or none do, so brackets never cross.
    marker: Option<usize>,
}

/// How [`render`] escapes document text.
#[derive(Clone, Copy)]
struct Escaping {
    /// No tracked change or comment: nothing can open a CriticMarkup span.
    plain: bool,
    /// Agent view: Markdown syntax is escaped too.
    agent: bool,
    /// A code block: nothing is escaped.
    code_block: bool,
}

impl Escaping {
    /// `text` escaped; a code span's or a code block's text is written as
    /// is, since a backslash shows there.
    fn text(self, text: &str, marks: Marks) -> String {
        if self.code_block || marks.code {
            return text.to_string();
        }
        match (self.agent, self.plain) {
            (true, true) => super::agent::escape_markdown(text),
            (true, false) => escape(&super::agent::escape_markdown(text)),
            (false, true) => text.to_string(),
            (false, false) => escape(text),
        }
    }
}

fn render(nodes: &[Node], out: &mut Vec<Piece>, escaping: Escaping, ids: &mut usize) {
    let mut index = 0;
    while index < nodes.len() {
        match (&nodes[index], nodes.get(index + 1)) {
            // The new text, then the old it replaces: Word's Compare
            // writes a substitution so, and so the writer puts
            // `{~~old~>new~~}` back. A deletion before an insertion stays
            // two changes, in their order.
            (
                Node::Span(Mark::Insertion, new_by, new),
                Some(Node::Span(Mark::Deletion, old_by, old)),
            ) => {
                *ids += 1;
                let id = *ids;
                marker(out, "{~~", id);
                render(old, out, escaping, ids);
                marker(out, "~>", id);
                render(new, out, escaping, ids);
                marker(out, "~~}", id);
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
                index += 2;
                continue;
            }
            (Node::Span(mark, by, nodes), _) => {
                *ids += 1;
                let id = *ids;
                let (open, close) = mark.delimiters();
                marker(out, open, id);
                render(nodes, out, escaping, ids);
                marker(out, close, id);
                attribution(out, by.as_deref(), id);
            }
            (Node::Leaf(Leaf::Text { text, marks, link }), _) => {
                out.push(Piece {
                    text: escaping.text(text, *marks),
                    marks: *marks,
                    link: link.clone(),
                    marker: None,
                });
            }
            (Node::Leaf(Leaf::Raw(markdown)), _) => out.push(Piece {
                text: math_changes(&if escaping.plain {
                    markdown.clone()
                } else {
                    defuse(markdown)
                }),
                marks: Marks::default(),
                link: None,
                marker: None,
            }),
            (Node::Leaf(Leaf::Comment(inner)), _) => {
                *ids += 1;
                marker(out, &note(inner), *ids);
            }
        }
        index += 1;
    }
}

fn marker(out: &mut Vec<Piece>, text: &str, id: usize) {
    out.push(Piece {
        text: text.to_string(),
        marks: Marks::default(),
        link: None,
        marker: Some(id),
    });
}

/// Markers between two pieces of the same link or emphasis take it on, so a
/// change inside a link's text or a bold run keeps one link or one run around
/// it: `[the {++new ++}page](url)`, `**around {++this++} inside**`.
/// CriticMarkup is read before Markdown, so the link and the emphasis still
/// resolve. A change joins only if all its markers do; one that starts inside
/// the link and ends after it stays outside.
fn wrap_markers(pieces: &mut [Piece]) {
    type Format = (Marks, Option<String>);
    let format = |piece: &Piece| -> Format { (piece.marks, piece.link.clone()) };
    let mut joined: Vec<Option<Format>> = vec![None; pieces.len()];
    let mut start = 0;
    while start < pieces.len() {
        if pieces[start].marker.is_none() {
            start += 1;
            continue;
        }
        let end = (start..pieces.len())
            .find(|&at| pieces[at].marker.is_none())
            .unwrap_or(pieces.len());
        let before = start.checked_sub(1).map(|at| format(&pieces[at]));
        let after = pieces.get(end).map(format);
        if let Some(around) =
            before.filter(|f| *f != (Marks::default(), None) && Some(f) == after.as_ref())
        {
            joined[start..end].fill(Some(around));
        }
        start = end;
    }
    let mut refused: std::collections::HashSet<usize> = pieces
        .iter()
        .zip(&joined)
        .filter(|(piece, around)| around.is_none() && piece.marker.is_some())
        .filter_map(|(piece, _)| piece.marker)
        .collect();
    // Markers next to each other join all or none: one that stays outside
    // (`==}` of a highlight begun before the bold) closes the emphasis, and
    // a note after it would reopen it on text that reads back plain.
    loop {
        let mut more = false;
        let mut start = 0;
        while start < pieces.len() {
            let end = (start..pieces.len())
                .find(|&at| pieces[at].marker.is_none())
                .unwrap_or(pieces.len());
            let run = || pieces[start..end].iter().filter_map(|piece| piece.marker);
            if run().any(|id| refused.contains(&id)) {
                for id in run() {
                    more |= refused.insert(id);
                }
            }
            start = end + 1;
        }
        if !more {
            break;
        }
    }
    for (piece, around) in pieces.iter_mut().zip(joined) {
        if let Some((marks, link)) =
            around.filter(|_| piece.marker.is_some_and(|id| !refused.contains(&id)))
        {
            piece.marks = marks;
            piece.link = link;
        }
    }
}

fn attribution(out: &mut Vec<Piece>, by: Option<&str>, id: usize) {
    if let Some(by) = by {
        marker(out, &note(&display(by)), id);
    }
}

fn note(inner: &str) -> String {
    format!("{{>>{inner}<<}}")
}

/// Breaks every CriticMarkup delimiter in document text with a Markdown
/// backslash escape, so it renders unchanged but cannot open or close a span.
pub(crate) fn escape(text: &str) -> String {
    const DELIMITERS: [(&str, &str); 11] = [
        ("{++", "{\\++"),
        ("{--", "{\\--"),
        ("{~~", "{\\~~"),
        ("{>>", "{\\>>"),
        ("{==", "{\\=="),
        ("++}", "++\\}"),
        ("--}", "--\\}"),
        ("~~}", "~~\\}"),
        ("<<}", "<<\\}"),
        ("==}", "==\\}"),
        ("~>", "~\\>"),
    ];
    if !text.contains(['{', '}', '~']) {
        return text.to_string();
    }
    DELIMITERS
        .iter()
        .fold(text.to_string(), |text, (from, to)| text.replace(from, to))
}

/// Breaks every CriticMarkup delimiter in Markdown the converter built
/// (equations, image descriptions). Outside an equation a space does it, and
/// an image description reads the same. Inside one, a space would be part of
/// the equation's text and the next write would draw it. An empty group
/// breaks the delimiter and the writer drops it (`$x_{--}$` is `$x_{{}--{}}$`,
/// still the text `--`).
fn defuse(markdown: &str) -> String {
    const SPACED: [(&str, &str); 11] = [
        ("{++", "{ ++"),
        ("{--", "{ --"),
        ("{~~", "{ ~~"),
        ("{>>", "{ >>"),
        ("{==", "{ =="),
        ("++}", "++ }"),
        ("--}", "-- }"),
        ("~~}", "~~ }"),
        ("<<}", "<< }"),
        ("==}", "== }"),
        ("~>", "~ >"),
    ];
    // Openers before closers, so `{--` broken as `{{}--` still has a `--}`
    // for the closer to break.
    const GROUPED: [(&str, &str); 11] = [
        ("{++", "{{}++"),
        ("{--", "{{}--"),
        ("{~~", "{{}~~"),
        ("{>>", "{{}>>"),
        ("{==", "{{}=="),
        ("++}", "++{}"),
        ("--}", "--{}"),
        ("~~}", "~~{}"),
        ("<<}", "<<{}"),
        ("==}", "=={}"),
        ("~>", "~{}>"),
    ];
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while !rest.is_empty() {
        let Some(start) = super::unescaped_dollar(rest) else {
            out.push_str(&break_delimiters(rest, &SPACED));
            break;
        };
        out.push_str(&break_delimiters(&rest[..start], &SPACED));
        let fence = if rest[start..].starts_with("$$") {
            "$$"
        } else {
            "$"
        };
        let body = start + fence.len();
        let Some(len) = super::math_body(&rest[body..], fence) else {
            out.push_str(&break_delimiters(&rest[start..], &SPACED));
            break;
        };
        let end = body + len + fence.len();
        out.push_str(fence);
        out.push_str(&break_delimiters(&rest[body..body + len], &GROUPED));
        out.push_str(fence);
        rest = &rest[end..];
    }
    out
}

fn break_delimiters(text: &str, marks: &[(&str, &str)]) -> String {
    marks
        .iter()
        .fold(text.to_string(), |text, (from, to)| text.replace(from, to))
}

/// An equation's changes (`$A={--π--}r^2$`), which the converter marks
/// with [`math::INSERT_START`] and its kin so `defuse` leaves them whole.
fn math_changes(markdown: &str) -> String {
    use crate::markdown::math;
    if !markdown.contains([
        math::INSERT_START,
        math::INSERT_END,
        math::DELETE_START,
        math::DELETE_END,
    ]) {
        return markdown.to_string();
    }
    markdown
        .replace(math::INSERT_START, "{++")
        .replace(math::INSERT_END, "++}")
        .replace(math::DELETE_START, "{--")
        .replace(math::DELETE_END, "--}")
}

/// Appends `prefix` and `body` to `out` after `separator`. When the separator is
/// itself tracked (a paragraph mark that was inserted or deleted), it is marked
/// the way the CriticMarkup spec marks a paragraph break (`{++\n\n++}`), with
/// its attribution after it. An adjoining span of the same kind and attribution
/// is extended rather than opened twice.
///
/// `space` says the text on either side of a tracked break had a space there,
/// which rendering trimmed: once the break is accepted or rejected away the two
/// paragraphs join with that space, so it goes after the marked break.
pub(crate) fn splice(
    out: &mut String,
    separator: &str,
    prefix: &str,
    body: &str,
    change: Option<&Change>,
    space: bool,
) {
    let Some((mark, by)) = change else {
        out.push_str(separator);
        out.push_str(prefix);
        out.push_str(body);
        return;
    };
    let (open, close) = mark.delimiters();
    let by = by.as_deref().map(|b| note(&display(b))).unwrap_or_default();
    let end = format!("{close}{by}");
    match out.strip_suffix(end.as_str()) {
        Some(kept) => out.truncate(kept.len()),
        None => out.push_str(open),
    }
    out.push_str(separator);
    out.push_str(prefix);
    // The body's first span continues this one when it is of the same kind and
    // its closer carries the same attribution. Document text is escaped and
    // converter-built Markdown is defused, so neither contains the closer, and a
    // span never nests one of its own kind: the first closer after the opener
    // is that span's own.
    let continues = body.strip_prefix(open).filter(|rest| {
        rest.find(close)
            .is_some_and(|at| rest[at + close.len()..].starts_with(by.as_str()))
    });
    match continues {
        Some(rest) => out.push_str(rest),
        None => {
            out.push_str(&end);
            if space {
                out.push(' ');
            }
            out.push_str(body);
        }
    }
}

/// The change a rendered part is wholly inside, with its attribution:
/// `{--gone--}{>>Ana<<}` is `(Deletion, Some("Ana"))`. Document text is
/// escaped, so the first closer after the opener is the span's own.
pub(crate) fn whole_change(part: &str) -> Option<Change> {
    [Mark::Insertion, Mark::Deletion]
        .into_iter()
        .find_map(|mark| {
            let (open, close) = mark.delimiters();
            let inner = part.strip_prefix(open)?;
            let after = &inner[inner.find(close)? + close.len()..];
            match after {
                "" => Some((mark, None)),
                _ => {
                    let by = after.strip_prefix("{>>")?.strip_suffix("<<}")?;
                    (!by.contains("<<}")).then(|| (mark, Some(by.to_string())))
                }
            }
        })
}

/// Joins parts that sit side by side, such as the cells of a flattened
/// table. A part that is wholly one change takes the separator before it into
/// that change (the first part, the separator after it), so accepting or
/// rejecting the change leaves no stray separator.
pub(crate) fn join_changed(parts: Vec<String>, separator: &str) -> String {
    let mut parts = parts.into_iter();
    let mut out = parts.next().unwrap_or_default();
    let first = whole_change(&out);
    for (index, part) in parts.enumerate() {
        let change = whole_change(&part).or_else(|| first.clone().filter(|_| index == 0));
        splice(&mut out, separator, "", &part, change.as_ref(), false);
    }
    out
}

/// A rendered paragraph, the tracked change on the mark that ends it, and
/// whether its text started and ended with a space before rendering trimmed it.
pub(crate) type Part = (String, Option<Change>, (bool, bool));

/// Joins paragraphs, each paired with the tracked change on the paragraph mark
/// that ends it. A blank paragraph drops the pending change: its own break stays.
pub(crate) fn join_marked(parts: Vec<Part>, separator: &str) -> String {
    join_paragraphs(parts, separator, false)
}

/// [`join_marked`], keeping a separator for each empty paragraph when
/// `keep_empty` (the agent view, whose table line counts every paragraph).
pub(crate) fn join_paragraphs(parts: Vec<Part>, separator: &str, keep_empty: bool) -> String {
    let mut out = String::new();
    let mut pending = None;
    let mut trailing = false;
    // The first paragraph, while it is the only one and wholly one change.
    let mut first = None;
    for (i, (part, mark, (leading, ends_in_space))) in parts.into_iter().enumerate() {
        if part.trim().is_empty() {
            // The agent view keeps an empty paragraph, and its mark's change
            // goes with the break after it.
            pending = if keep_empty { mark } else { None };
            if keep_empty && i > 0 {
                first = None;
                out.push_str(separator);
            }
            continue;
        }
        if out.is_empty() {
            first = whole_change(&part);
            if keep_empty && i > 0 {
                out.push_str(separator);
            }
            out.push_str(&part);
        } else {
            // A break with no change of its own goes with a paragraph that is
            // wholly one change, which leaves no stray break once resolved.
            // The agent view (`keep_empty`) prints the break as its caller
            // marks it: in a change only when the paragraph's mark is.
            let first = first.take();
            let change = if keep_empty {
                pending.take()
            } else {
                pending.take().or_else(|| whole_change(&part)).or(first)
            };
            // The agent view already wrote a leading space as `&#32;`.
            // Another space here would double it.
            let space = trailing || (leading && !part.starts_with("&#32;"));
            splice(&mut out, separator, "", &part, change.as_ref(), space);
        }
        pending = mark;
        trailing = ends_in_space;
    }
    out
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use Mark::{Deletion as Del, Highlight as Hl, Insertion as Ins};

    fn text(critic: &mut Critic, t: &str) {
        critic.push(t, false, false, None);
    }

    fn md(critic: Critic) -> String {
        critic.into_inline().render(true)
    }

    /// Builds a paragraph from `(op, value)` steps: `open`/`close` take `+`,
    /// `-` or `=`; `note` is a comment, `raw` raw Markdown, anything else text.
    fn script(steps: &[(&str, &str)]) -> String {
        let mut critic = Critic::default();
        for (op, value) in steps {
            match *op {
                "open" => critic.open(kind(value), None),
                // `by+Ana (d)`: an insertion or deletion made by Ana at d.
                op if op.starts_with("by") => critic.open(kind(&op[2..]), Some(value)),
                "close" => critic.close(kind(value)),
                "note" => critic.comment(value),
                "raw" => critic.raw(value),
                _ => text(&mut critic, value),
            }
        }
        md(critic)
    }

    fn kind(value: &str) -> Mark {
        match value {
            "+" => Ins,
            "-" => Del,
            _ => Hl,
        }
    }

    #[test]
    fn plain_marks_render_with_their_delimiters() {
        assert_eq!(
            script(&[
                ("open", "+"),
                ("t", "a"),
                ("close", "+"),
                ("t", " "),
                ("open", "="),
                ("t", "b"),
                ("close", "=")
            ]),
            "{++a++} {==b==}"
        );
        assert_eq!(
            script(&[("open", "-"), ("t", "gone"), ("close", "-")]),
            "{--gone--}"
        );
    }

    #[test]
    fn an_insertion_then_a_deletion_is_a_substitution_and_the_reverse_two_changes() {
        let del_ins = [
            ("open", "-"),
            ("t", "old"),
            ("close", "-"),
            ("open", "+"),
            ("t", "new"),
            ("close", "+"),
        ];
        let ins_del = [
            ("open", "+"),
            ("t", "new"),
            ("close", "+"),
            ("open", "-"),
            ("t", "old"),
            ("close", "-"),
        ];
        // Word's Compare writes the new text first; the writer puts
        // `{~~old~>new~~}` back so. The other order keeps its two changes.
        assert_eq!(script(&ins_del), "{~~old~>new~~}");
        assert_eq!(script(&del_ins), "{--old--}{++new++}");
        // Only one pair: a span before it stays on its own.
        let triple = [
            ("open", "-"),
            ("t", "a"),
            ("close", "-"),
            ("open", "+"),
            ("t", "b"),
            ("close", "+"),
            ("open", "-"),
            ("t", "c"),
            ("close", "-"),
        ];
        assert_eq!(script(&triple), "{--a--}{~~c~>b~~}");
    }

    #[test]
    fn same_kind_nesting_is_merged_into_one_span() {
        let nested = [
            ("open", "-"),
            ("t", "a"),
            ("open", "-"),
            ("t", "b"),
            ("close", "-"),
            ("t", "c"),
            ("close", "-"),
            ("t", "d"),
        ];
        // The outer close still ends the span: `d` is outside it.
        assert_eq!(script(&nested), "{--abc--}d");
    }

    #[test]
    fn merged_spans_are_counted_per_kind() {
        // Two overlapping highlights, and an insertion inside them: closing the
        // insertion must not use up the highlights' count.
        let steps = [
            ("open", "="),
            ("open", "="),
            ("t", "a"),
            ("open", "+"),
            ("t", "b"),
            ("close", "+"),
            ("t", "c"),
            ("close", "="),
            ("close", "="),
        ];
        assert_eq!(script(&steps), "{==a{++b++}c==}");
    }

    #[test]
    fn different_kinds_may_nest() {
        // Inserted by one author, deleted by another: gone either way.
        let nested = [
            ("open", "+"),
            ("open", "-"),
            ("t", "x"),
            ("close", "-"),
            ("close", "+"),
        ];
        assert_eq!(script(&nested), "{++{--x--}++}");
    }

    #[test]
    fn crossing_spans_are_split_so_each_closes_inside_its_parent() {
        let crossing = [
            ("open", "="),
            ("t", "a"),
            ("open", "+"),
            ("t", "b"),
            ("close", "="),
            ("t", "c"),
            ("close", "+"),
        ];
        assert_eq!(script(&crossing), "{==a{++b++}==}{++c++}");
    }

    #[test]
    fn neighbours_of_the_same_kind_join() {
        let split = [
            ("open", "+"),
            ("t", "a"),
            ("close", "+"),
            ("open", "+"),
            ("t", "b"),
            ("close", "+"),
        ];
        assert_eq!(script(&split), "{++ab++}");
        // Joining exposes inner neighbours, which join too.
        let deep = [
            ("open", "-"),
            ("open", "+"),
            ("t", "a"),
            ("close", "+"),
            ("close", "-"),
            ("open", "-"),
            ("open", "+"),
            ("t", "b"),
            ("close", "+"),
            ("close", "-"),
        ];
        assert_eq!(script(&deep), "{--{++ab++}--}");
    }

    #[test]
    fn empty_spans_vanish_but_keep_their_comments() {
        assert_eq!(
            script(&[("t", "a"), ("open", "+"), ("close", "+"), ("t", "b")]),
            "ab"
        );
        assert_eq!(
            script(&[
                ("t", "a"),
                ("open", "+"),
                ("open", "-"),
                ("note", "c"),
                ("close", "-"),
                ("close", "+")
            ]),
            "a{>>c<<}"
        );
        // A whitespace insertion is real content.
        assert_eq!(
            script(&[
                ("t", "a"),
                ("open", "+"),
                ("t", " "),
                ("close", "+"),
                ("t", "b")
            ]),
            "a{++ ++}b"
        );
    }

    #[test]
    fn comments_inside_a_highlight_follow_it() {
        let steps = [
            ("open", "="),
            ("t", "a"),
            ("note", "one"),
            ("open", "+"),
            ("t", "b"),
            ("note", "two"),
            ("close", "+"),
            ("close", "="),
            ("t", "."),
        ];
        assert_eq!(script(&steps), "{==a{++b++}==}{>>one<<}{>>two<<}.");
        // Two highlights separated by a comment stay separate.
        let steps = [
            ("open", "="),
            ("t", "a"),
            ("close", "="),
            ("note", "x"),
            ("open", "="),
            ("t", "b"),
            ("close", "="),
        ];
        assert_eq!(script(&steps), "{==a==}{>>x<<}{==b==}");
    }

    #[test]
    fn overlapping_highlights_become_their_union() {
        let steps = [
            ("open", "="),
            ("t", "a"),
            ("open", "="),
            ("t", "b"),
            ("close", "="),
            ("note", "A"),
            ("t", "c"),
            ("close", "="),
            ("note", "B"),
        ];
        assert_eq!(script(&steps), "{==abc==}{>>A<<}{>>B<<}");
    }

    #[test]
    fn unclosed_spans_close_at_the_end_and_stray_closes_are_ignored() {
        assert_eq!(
            script(&[("close", "+"), ("open", "-"), ("t", "a")]),
            "{--a--}"
        );
    }

    #[test]
    fn raw_markdown_is_content_whose_delimiters_are_spaced_apart() {
        // A backslash would change the LaTeX. Outside an equation a space
        // keeps a delimiter from closing the deletion around it; inside one
        // an empty group does, and the writer drops it.
        assert_eq!(
            script(&[
                ("open", "-"),
                ("raw", "$x_{--}$ {++ ++} ~> {== ==} {>> <<} {~~ ~~}"),
                ("close", "-"),
                ("raw", "")
            ]),
            "{--$x_{{}--{}}$ { ++ ++ } ~ > { == == } { >> << } { ~~ ~~ }--}"
        );
        assert_eq!(defuse("$a+b$"), "$a+b$");
    }

    #[test]
    fn an_equation_breaks_delimiters_on_an_empty_group() {
        assert_eq!(defuse("$x_{--}$"), "$x_{{}--{}}$");
        assert_eq!(defuse("$y^{++}$"), "$y^{{}++{}}$");
        assert_eq!(defuse("$$x_{--}$$"), "$$x_{{}--{}}$$");
        assert_eq!(defuse("$a~>$"), "$a~{}>$");
        // The broken form has no delimiter left, so a second pass keeps it.
        assert_eq!(defuse("$x_{{}--{}}$"), "$x_{{}--{}}$");
        // A backslash before `$` is the character, not a fence.
        assert_eq!(defuse(r"cost \$5 $x_{--}$"), r"cost \$5 $x_{{}--{}}$");
        // An unclosed dollar is text, and stays broken with a space.
        assert_eq!(defuse("cost $5 and {--x--}"), "cost $5 and { --x-- }");
    }

    #[test]
    fn a_tracked_break_finds_the_real_closer_past_an_equation() {
        let mut critic = Critic::default();
        critic.open(Del, Some("Ana"));
        critic.raw("$x_{--}$");
        critic.close(Del);
        let body = critic.into_inline().render(true);
        let mut out = "{--A--}{>>Ana<<}".to_string();
        splice(
            &mut out,
            "\n\n",
            "",
            &body,
            Some(&(Del, Some("Ana".into()))),
            false,
        );
        assert_eq!(out, "{--A\n\n$x_{{}--{}}$--}{>>Ana<<}");
    }

    #[test]
    fn formatting_and_links_stay_inside_the_markers() {
        let mut critic = Critic::default();
        critic.open(Ins, None);
        critic.push("bold", true, false, None);
        critic.close(Ins);
        critic.push(" ", false, false, None);
        critic.open(Del, None);
        critic.push("site", false, false, Some("https://x.test"));
        critic.close(Del);
        critic.push("", false, false, None);
        assert_eq!(md(critic), "{++**bold**++} {--[site](https://x.test)--}");
    }

    #[test]
    fn blankness_ignores_markers_but_not_comments() {
        let mut critic = Critic::default();
        critic.open(Ins, None);
        text(&mut critic, "  ");
        critic.raw(" ");
        critic.close(Ins);
        assert!(critic.is_blank());
        critic.comment("note");
        assert!(!critic.is_blank());
        let mut raw = Critic::default();
        raw.raw("![x](y)");
        assert!(!raw.is_blank());
    }

    #[test]
    fn delimiters_in_document_text_are_escaped() {
        assert_eq!(escape("plain"), "plain");
        assert_eq!(escape("a{b}c"), "a{b}c");
        assert_eq!(
            escape("{++ {-- {~~ {>> {== ++} --} ~~} <<} ==} ~>"),
            "{\\++ {\\-- {\\~~ {\\>> {\\== ++\\} --\\} ~~\\} <<\\} ==\\} ~\\>"
        );
        assert_eq!(escape("{+++}"), "{\\+++\\}");
        assert_eq!(
            script(&[("open", "+"), ("t", "a --} b"), ("close", "+")]),
            "{++a --\\} b++}"
        );
    }

    #[test]
    fn splice_marks_a_tracked_break_and_extends_adjacent_spans() {
        let join = |out: &str, body: &str, mark: Option<Mark>| {
            let mut out = out.to_string();
            splice(
                &mut out,
                "\n\n",
                "",
                body,
                mark.map(|m| (m, None)).as_ref(),
                false,
            );
            out
        };
        assert_eq!(join("A", "B", None), "A\n\nB");
        assert_eq!(join("A", "B", Some(Del)), "A{--\n\n--}B");
        assert_eq!(join("A", "B", Some(Ins)), "A{++\n\n++}B");
        assert_eq!(join("{--A--}", "{--B--}", Some(Del)), "{--A\n\nB--}");
        assert_eq!(
            join("Keep {--old--}", "B", Some(Del)),
            "Keep {--old\n\n--}B"
        );
        assert_eq!(join("A", "{++B++} rest", Some(Ins)), "A{++\n\nB++} rest");
        // An escaped delimiter in the text is not a span to extend.
        assert_eq!(join("a --\\}", "b", Some(Del)), "a --\\}{--\n\n--}b");
        let mut heading = "A".to_string();
        splice(&mut heading, "\n\n", "## ", "B", Some(&(Ins, None)), false);
        assert_eq!(heading, "A{++\n\n## ++}B");
    }

    #[test]
    fn join_marked_uses_each_paragraph_mark_and_resets_on_blanks() {
        let bare = (false, false);
        let parts = vec![
            ("a".to_string(), Some((Del, None)), bare),
            ("b".to_string(), None, bare),
            ("c".to_string(), Some((Ins, None)), bare),
            ("  ".to_string(), Some((Del, None)), bare),
            ("d".to_string(), Some((Del, None)), bare),
        ];
        assert_eq!(join_marked(parts, " "), "a{-- --}b c d");
        assert_eq!(join_marked(Vec::new(), " "), "");
    }

    #[test]
    fn a_space_before_a_change_that_ends_the_paragraph_goes_inside_it() {
        let joined = |steps: &[(&str, &str)]| {
            let mut critic = Critic::default();
            critic.set_inline_end();
            for (op, value) in steps {
                match *op {
                    "open" => critic.open(kind(value), None),
                    "close" => critic.close(kind(value)),
                    _ => text(&mut critic, value),
                }
            }
            md(critic)
        };
        assert_eq!(
            joined(&[("t", "in "), ("open", "+"), ("t", "nested"), ("close", "+")]),
            "in{++ nested++}"
        );
        assert_eq!(
            joined(&[("t", "kept "), ("open", "-"), ("t", "gone"), ("close", "-")]),
            "kept{-- gone--}"
        );
        // Not at the end, or not a change: left as it is.
        assert_eq!(
            joined(&[
                ("t", "a "),
                ("open", "+"),
                ("t", "b"),
                ("close", "+"),
                ("t", " c")
            ]),
            "a {++b++} c"
        );
        assert_eq!(
            joined(&[("t", "a "), ("open", "="), ("t", "b"), ("close", "=")]),
            "a {==b==}"
        );
        // A paragraph of its own keeps the space where it was.
        assert_eq!(
            script(&[("t", "in "), ("open", "+"), ("t", "nested"), ("close", "+")]),
            "in {++nested++}"
        );
    }

    #[test]
    fn a_wholly_changed_paragraph_takes_the_break_before_it() {
        let bare = (false, false);
        let parts = vec![
            ("Para".to_string(), None, bare),
            ("{++Added para++}{>>Ana<<}".to_string(), None, bare),
        ];
        assert_eq!(
            join_marked(parts, "<br>"),
            "Para{++<br>Added para++}{>>Ana<<}"
        );
        let parts = vec![
            ("{--Gone--}".to_string(), None, bare),
            ("Kept".to_string(), None, bare),
        ];
        assert_eq!(join_marked(parts, "<br>"), "{--Gone<br>--}Kept");
    }

    #[test]
    fn a_space_trimmed_at_a_tracked_break_comes_back_after_it() {
        // `notice.` + ` The` with the break deleted reads `notice. The` once
        // accepted; the space stays outside the change.
        let parts = vec![
            ("notice.".to_string(), Some((Del, None)), (false, false)),
            ("The".to_string(), None, (true, false)),
            ("before".to_string(), Some((Ins, None)), (false, true)),
            ("accepting".to_string(), None, (false, false)),
        ];
        assert_eq!(
            join_marked(parts, "<br>"),
            "notice.{--<br>--} The<br>before{++<br>++} accepting"
        );
        // An untracked break needs no space: it stays a break.
        let parts = vec![
            ("a".to_string(), None, (false, true)),
            ("b".to_string(), None, (true, false)),
        ];
        assert_eq!(join_marked(parts, "<br>"), "a<br>b");
        // A space the text already keeps (`&#32;`) is not written again.
        let parts = vec![
            ("notice.".to_string(), Some((Del, None)), (false, false)),
            ("&#32;The".to_string(), None, (true, false)),
        ];
        assert_eq!(join_marked(parts, "<br>"), "notice.{--<br>--}&#32;The");
    }

    #[test]
    fn each_change_is_followed_by_its_author_and_date() {
        let steps = [
            ("by+", "Ana (2026-01-02T03:04:00Z)"),
            ("t", "new"),
            ("close", "+"),
            ("t", " and "),
            ("by-", "Bo"),
            ("t", "old"),
            ("close", "-"),
            ("t", "."),
        ];
        assert_eq!(
            script(&steps),
            "{++new++}{>>Ana (2026-01-02T03:04:00Z)<<} and {--old--}{>>Bo<<}."
        );
    }

    #[test]
    fn a_substitution_names_one_author_or_both() {
        let same = [
            ("by+", "Ana"),
            ("t", "new"),
            ("close", "+"),
            ("by-", "Ana"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&same), "{~~old~>new~~}{>>Ana<<}");
        // Each author follows in the order of old then new.
        let different = [
            ("by+", "Bo"),
            ("t", "new"),
            ("close", "+"),
            ("by-", "Ana"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&different), "{~~old~>new~~}{>>Ana<<}{>>Bo<<}");
        let one_side = [
            ("by+", "Bo"),
            ("t", "new"),
            ("close", "+"),
            ("open", "-"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&one_side), "{~~old~>new~~}{>>Bo<<}");
        let other_side = [
            ("open", "+"),
            ("t", "new"),
            ("close", "+"),
            ("by-", "Ana"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&other_side), "{~~old~>new~~}{>>Ana<<}");
    }

    #[test]
    fn neighbours_join_only_when_made_by_the_same_person_at_the_same_time() {
        let same = [
            ("by+", "Ana"),
            ("t", "a"),
            ("close", "+"),
            ("by+", "Ana"),
            ("t", "b"),
            ("close", "+"),
        ];
        assert_eq!(script(&same), "{++ab++}{>>Ana<<}");
        let different = [
            ("by+", "Ana"),
            ("t", "a"),
            ("close", "+"),
            ("by+", "Bo"),
            ("t", "b"),
            ("close", "+"),
        ];
        assert_eq!(script(&different), "{++a++}{>>Ana<<}{++b++}{>>Bo<<}");
    }

    #[test]
    fn attributions_stay_with_their_change_inside_a_highlight() {
        // A Word comment moves after the highlight; the attribution does not.
        let steps = [
            ("open", "="),
            ("t", "a"),
            ("by+", "Bo"),
            ("t", "b"),
            ("close", "+"),
            ("note", "Ana: why?"),
            ("close", "="),
        ];
        assert_eq!(script(&steps), "{==a{++b++}{>>Bo<<}==}{>>Ana: why?<<}");
    }

    #[test]
    fn merged_nesting_keeps_the_outer_author_and_split_spans_keep_theirs() {
        let nested = [
            ("by-", "Row"),
            ("t", "a"),
            ("by-", "Run"),
            ("t", "b"),
            ("close", "-"),
            ("close", "-"),
        ];
        assert_eq!(script(&nested), "{--ab--}{>>Row<<}");
        let crossing = [
            ("open", "="),
            ("t", "a"),
            ("by+", "Bo"),
            ("t", "b"),
            ("close", "="),
            ("t", "c"),
            ("close", "+"),
        ];
        assert_eq!(script(&crossing), "{==a{++b++}{>>Bo<<}==}{++c++}{>>Bo<<}");
        // An empty change leaves no attribution behind.
        assert_eq!(script(&[("t", "x"), ("by+", "Bo"), ("close", "+")]), "x");
    }

    #[test]
    fn a_tracked_break_extends_spans_only_by_the_same_person() {
        let join = |out: &str, body: &str, change: Option<Change>| {
            let mut out = out.to_string();
            splice(&mut out, "\n\n", "", body, change.as_ref(), false);
            out
        };
        let ana = || Some((Del, Some("Ana".to_string())));
        assert_eq!(join("A", "B", ana()), "A{--\n\n--}{>>Ana<<}B");
        assert_eq!(
            join("{--A--}{>>Ana<<}", "{--B--}{>>Ana<<} rest", ana()),
            "{--A\n\nB--}{>>Ana<<} rest"
        );
        // Someone else's deletion on either side stays its own span.
        assert_eq!(
            join("{--A--}{>>Bo<<}", "{--B--}{>>Bo<<}", ana()),
            "{--A--}{>>Bo<<}{--\n\n--}{>>Ana<<}{--B--}{>>Bo<<}"
        );
        // A body span of the same kind whose closer is followed by nothing
        // attributed is not the same change.
        assert_eq!(
            join("A", "{--B--} rest", ana()),
            "A{--\n\n--}{>>Ana<<}{--B--} rest"
        );
        assert_eq!(join("A", "{--B", ana()), "A{--\n\n--}{>>Ana<<}{--B");
    }

    #[test]
    fn tree_ignores_a_close_with_nothing_open() {
        assert_eq!(tree(vec![Token::Close(Ins)]), Vec::<Node>::new());
    }

    #[test]
    fn a_separator_next_to_a_wholly_changed_part_joins_its_change() {
        let by = "{>>Ana<<}";
        let joined = |parts: &[&str]| {
            join_changed(parts.iter().map(|part| part.to_string()).collect(), "; ")
        };
        assert_eq!(
            joined(&["in {++nested++}", &format!("{{--gone--}}{by}")]),
            format!("in {{++nested++}}{{--; gone--}}{by}")
        );
        assert_eq!(
            joined(&[&format!("{{--gone--}}{by}"), "kept", "{++new++}"]),
            format!("{{--gone; --}}{by}kept{{++; new++}}")
        );
        // Text around the change keeps the separator outside it.
        assert_eq!(
            joined(&["a {--b--}", "c {++d++} e"]),
            "a {--b--}; c {++d++} e"
        );
    }

    #[test]
    fn a_change_inside_bold_text_keeps_one_bold_run() {
        let mut critic = Critic::default();
        critic.push("around ", true, false, None);
        critic.open(Ins, Some("Ana"));
        critic.push("an insertion", true, false, None);
        critic.close(Ins);
        critic.push(" inside", true, false, None);
        critic.push(" end", false, false, None);
        assert_eq!(
            md(critic),
            "**around {++an insertion++}{>>Ana<<} inside** end"
        );
        // Bold text next to a plain change stays apart from it.
        let mut critic = Critic::default();
        critic.push("bold", true, false, None);
        critic.open(Del, None);
        critic.push(" plain", false, false, None);
        critic.close(Del);
        assert_eq!(md(critic), "**bold**{-- plain--}");
    }

    #[test]
    fn markers_side_by_side_join_the_emphasis_together_or_not_at_all() {
        // file_170: a highlight begun before the bold ends with the bold's
        // last run; the notes after its end stay outside the bold too.
        let mut critic = Critic::default();
        critic.open(Hl, None);
        critic.push("varied ", false, false, None);
        critic.push("Emphasis lives here. ", true, true, None);
        critic.close(Hl);
        critic.comment("c2 @AS: Testing.");
        critic.push("See note.", true, true, None);
        assert_eq!(
            md(critic),
            "{==varied **_Emphasis lives here._** ==}{>>c2 @AS: Testing.<<}**_See note._**"
        );
        // A note alone between two bold runs still joins them.
        let mut critic = Critic::default();
        critic.push("bold ", true, false, None);
        critic.comment("c0 @AS: Here.");
        critic.push("text", true, false, None);
        assert_eq!(md(critic), "**bold {>>c0 @AS: Here.<<}text**");
    }

    #[test]
    fn a_change_inside_link_text_keeps_one_link() {
        let mut critic = Critic::default();
        let url = Some("https://example.com");
        critic.push("the ", false, false, url);
        critic.open(Ins, Some("Ana"));
        critic.push("new ", false, false, url);
        critic.close(Ins);
        critic.push("page", false, false, url);
        assert_eq!(
            md(critic),
            "[the {++new ++}{>>Ana<<}page](https://example.com)"
        );
        // A change around a whole link, or between two different links,
        // stays outside them.
        let mut critic = Critic::default();
        critic.push("see ", false, false, None);
        critic.open(Del, None);
        critic.push("old", false, false, Some("https://a.example"));
        critic.close(Del);
        critic.push(" and ", false, false, Some("https://b.example"));
        assert_eq!(
            md(critic),
            "see {--[old](https://a.example)--} [and](https://b.example)"
        );
    }

    #[test]
    fn a_delimiter_split_across_runs_is_escaped() {
        // Word splits runs anywhere; `a -` + `-} b` must not close the deletion.
        assert_eq!(
            script(&[("open", "-"), ("t", "a -"), ("t", "-} b"), ("close", "-")]),
            "{--a --\\} b--}"
        );
        // An empty span between the two pieces is dropped, so they still meet.
        assert_eq!(
            script(&[
                ("open", "-"),
                ("t", "x {"),
                ("open", "+"),
                ("close", "+"),
                ("t", "++ y"),
                ("close", "-")
            ]),
            "{--x {\\++ y--}"
        );
    }
}
