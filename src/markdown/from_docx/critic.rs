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
//! - untagged neighbouring spans of the same kind are joined;
//! - comments inside a highlight move to right after it (`{==text==}{>>note<<}`);
//! - a deletion next to an insertion becomes a substitution (`{~~old~>new~~}`).
//!
//! An insertion or deletion carries its attribution after the closer in
//! braces: `{++new++}{r1 Alice (2026-09-29T14:05:00Z)}`. Distinct revision
//! ids keep separate marks. Only a deletion then a live insertion of one
//! change (as `jubarte changes` lists it) becomes a substitution, with old
//! ids before new ids; two changes print apart (#1363).
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
    /// A comment's scope, `{==…==}`.
    Scope,
}

impl Mark {
    pub(crate) fn delimiters(self) -> (&'static str, &'static str) {
        match self {
            Self::Insertion => ("{++", "++}"),
            Self::Deletion => ("{--", "--}"),
            Self::Scope => ("{==", "==}"),
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// A tracked change: its kind and who made it when, as the inside of its
/// brace tag (`r1 Alice (2026-09-29T14:05:00Z)`), already escaped.
pub(crate) type Change = (Mark, Option<String>);

/// Agent-view attribution: an internal tag (`0@AC`, `1+2@AC`,
/// `header1#1@AC`, optionally `0@AC 2026-10-03T14:05:00Z`) behind this
/// sentinel, so that it is never
/// mistaken for an author's note. Printed through `agent::format_tag`;
/// never written to the output.
pub(crate) const TAG: &str = "\u{E000}";

/// Plain-view attribution: the change's ids and its `Author (date)` note
/// as an internal tag (`agent::plain_tag_of`) behind this sentinel, so that
/// distinct neighbouring ids remain separate. Printed through
/// `agent::format_plain_tag`: `r1+2 Ana Lima (2026-09-29T14:05:00Z)`.
pub(crate) const PLAIN: &str = "\u{E001}";

/// The internal tag of an attribution, agent or plain.
fn tag(by: Option<&str>) -> Option<&str> {
    let by = bare(by?);
    by.strip_prefix(TAG)
        .or_else(|| by.strip_prefix(PLAIN))
        // A note that names the change it sits inside (`3@BO` re `1@AL`)
        // carries that one after [`ON`]: the tag is the change's own.
        .map(|tagged| tagged.split_once(ON).map_or(tagged, |(own, _)| own))
}

/// The change a nested note names after [`ON`], as its tag reads (no
/// sentinel): the change the note's own sits inside.
fn outer_of(by: Option<&str>) -> Option<&str> {
    let by = bare(by?);
    by.strip_prefix(TAG)
        .or_else(|| by.strip_prefix(PLAIN))?
        .split_once(ON)
        .map(|(_, outer)| outer)
}

/// Whether `new_by`'s change is the one `old_by`'s sits in: a deletion
/// in one mark of an insertion never pairs with the insertion's next mark
/// (`{--old--}{r5 @B re r3+4 @B}{++new++}{r3+4 @B}`), for the
/// substitution's new side would be its own parent.
fn holds(new_by: Option<&str>, old_by: Option<&str>) -> bool {
    fn first(tagged: &str) -> &str {
        tag_parts(tagged.split(ON).next().unwrap_or(tagged)).0
    }
    match (tag(new_by), outer_of(old_by)) {
        (Some(own), Some(outer)) => super::agent::same_change(first(own), first(outer)),
        _ => false,
    }
}

/// The changes a note names sit behind this sentinel after the change's
/// own tag (`\u{E000}3@BO\u{E002}1@AL`), with each tag's optional
/// timestamp before it, exactly as [`display`] prints them: `r3 @BO re
/// r1 @AL`.
const ON: &str = "\u{E002}";

/// Marks an attribution the container carries (a row's or cell's own
/// mark, opened around each paragraph it covers) rather than a change on
/// the content: a sole change under one is not folded into a note, for
/// the writer rebuilds container marks itself, not as a wrap.
pub(crate) const CARRIED: &str = "\u{E003}";

/// An attribution without [`CARRIED`].
fn bare(by: &str) -> &str {
    by.strip_suffix(CARRIED).unwrap_or(by)
}

/// The sentinel an attribution's tag sits behind.
fn sentinel(by: &str) -> &'static str {
    if bare(by).starts_with(PLAIN) {
        PLAIN
    } else {
        TAG
    }
}

/// The tag proper and its optional inline timestamp.
fn tag_parts(tagged: &str) -> (&str, Option<&str>) {
    match tagged.split_once(' ') {
        Some((t, date)) => (t, Some(date)),
        None => (tagged, None),
    }
}

/// A note's text: the formatted tag, or the plain attribution. A note
/// that names the changes it sits inside prints ` re ` between them,
/// one part per level, its own change first. Each plain part's author
/// spells its parentheses percent-wise (`Ana %28consultant%29`), so its
/// date's are the only bare ones and the writer's parser can split the
/// note.
pub(super) fn display(by: &str) -> String {
    let by = bare(by);
    let Some(tagged) = by.strip_prefix(PLAIN).or_else(|| by.strip_prefix(TAG)) else {
        return by.to_string();
    };
    let fold = tagged.contains(ON);
    let one = |part: &str| {
        if by.starts_with(PLAIN) {
            let printed = super::agent::format_plain_tag(part);
            return if fold {
                escaped_author(&printed)
            } else {
                printed
            };
        }
        let (t, date) = tag_parts(part);
        let mut out = super::agent::format_tag(t);
        if let Some(date) = date {
            out.push(' ');
            out.push_str(date);
        }
        out
    };
    {
        let mut out = String::with_capacity(tagged.len() + 32);
        let mut first = true;
        for part in tagged.split(ON).map(one) {
            if !first {
                out.push_str(" re ");
            }
            out.push_str(&part);
            first = false;
        }
        out
    }
}

/// A printed plain part (`r1+2 Name (date)`) with the name's parentheses,
/// backslashes and `%` percent-spelled (`%28` and its kin, `%` itself
/// `%25`, the spelling the internal tags use): only the date's stay bare.
fn escaped_author(part: &str) -> String {
    let Some((ids, rest)) = part.split_once(' ') else {
        return part.to_string();
    };
    let author_end = super::super::view_notes::trailing_bare_group(rest)
        .filter(|&at| super::super::write::is_date(&rest[at + 1..rest.len() - 1]))
        .unwrap_or(rest.len());
    let mut out = String::with_capacity(part.len());
    out.push_str(ids);
    out.push(' ');
    for c in rest[..author_end].chars() {
        match c {
            '%' => out.push_str("%25"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '\\' => out.push_str("%5C"),
            c => out.push(c),
        }
    }
    out.push_str(&rest[author_end..]);
    out
}

/// Two attributions can pair: equal notes, or two tags of one author in
/// one story with equal inline dates (their ids differ by design). A single
/// substitution tag cannot represent two different timestamps.
fn pairable_attribution(a: Option<&str>, b: Option<&str>) -> bool {
    use super::agent::{handle_of, split_tag};
    match (tag(a), tag(b)) {
        (Some(ta), Some(tb)) if a.map(sentinel) == b.map(sentinel) => {
            let (own_a, own_b) = (tag_parts(ta).0, tag_parts(tb).0);
            let same = if a.is_some_and(|by| sentinel(by) == PLAIN) {
                let attribution = |tag| {
                    crate::markdown::view_notes::plain_revision(&super::agent::format_plain_tag(
                        tag,
                    ))
                    .map(|part| (part.author, part.date))
                };
                attribution(own_a)
                    .zip(attribution(own_b))
                    .is_some_and(|(a, b)| a == b)
            } else {
                handle_of(own_a)
                    .zip(handle_of(own_b))
                    .is_some_and(|(a, b)| a == b)
            };
            same && tag_parts(ta).1 == tag_parts(tb).1 && split_tag(own_a).0 == split_tag(own_b).0
        }
        _ => a == b,
    }
}

/// Whether two sides' changes are one: a substitution prints one change,
/// as `jubarte changes` lists it (#1363). A deletion and the insertion
/// after it with content between them, printed or not, are two.
fn one_change(old_by: Option<&str>, new_by: Option<&str>) -> bool {
    match (tag(old_by), tag(new_by)) {
        (Some(old), Some(new)) => super::agent::one_replacement(tag_parts(old).0, tag_parts(new).0),
        _ => true,
    }
}

/// The tags of two neighbours or of a substitution's two sides (whole
/// attributions, sentinel and all), joined, keeping their shared inline
/// timestamp. The pairing guard requires equal dates.
fn join_tagged(a: &str, b: &str) -> String {
    let kind = sentinel(a);
    let (ta, da) = tag_parts(tag(Some(a)).unwrap_or(a));
    let (tb, db) = tag_parts(tag(Some(b)).unwrap_or(b));
    let mut out = format!(
        "{kind}{}",
        super::agent::join_tags(&[ta.to_string(), tb.to_string()])
    );
    if let Some(date) = da.or(db) {
        out.push(' ');
        out.push_str(date);
    }
    if let Some(outer) = outer_of(Some(a)) {
        out.push_str(ON);
        out.push_str(outer);
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
    /// The inside of a `{>>…<<}` comment, in each of the forms its
    /// contexts print.
    Comment(Note),
}

/// A comment's note, as each of its contexts prints it: the prose around
/// its anchor, and the code block it is anchored in, whose fence unwinds
/// only the escape that form's text uses (#518).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Note {
    pub(crate) prose: String,
    pub(crate) code: String,
}

impl Note {
    /// A note whose text no context changes: it holds nothing either
    /// escape touches.
    #[cfg(test)]
    pub(crate) fn same(text: &str) -> Self {
        Self {
            prose: text.to_string(),
            code: text.to_string(),
        }
    }
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
    /// A change's pieces around the changes it held, with the outer
    /// change's mark and attribution: each piece keeps the attribution
    /// too. CriticMarkup cannot nest a change in a change, so an insertion
    /// holding a deletion prints `{++a++}…{++b++}`, and the pieces never
    /// pair as a substitution.
    Split(Mark, Option<String>, Vec<Node>),
}

impl Node {
    fn has_content(&self) -> bool {
        match self {
            Self::Leaf(Leaf::Comment(_)) => false,
            Self::Leaf(_) => true,
            Self::Span(_, _, children) | Self::Split(.., children) => {
                children.iter().any(Self::has_content)
            }
        }
    }
}

/// Inline content of one paragraph with CriticMarkup spans.
#[derive(Debug, Default)]
pub(crate) struct Critic {
    tokens: Vec<Token>,
    /// The paragraph is joined to more text on the same line (a flattened
    /// table's cells): see [`space_into_last_change`].
    inline_end: bool,
    /// Agent view: runs of spaces stay as the file holds them.
    agent: bool,
    /// A code block's paragraph: its text goes in a fence, its runs of
    /// spaces kept and its CriticMarkup delimiters escaped for the writer
    /// to unwind there.
    code_block: bool,
    /// Source dates, kept separately from the optional printed timestamps.
    source_dates: std::collections::HashMap<String, Option<String>>,
}

impl Critic {
    /// Agent view: keeps runs of spaces as the file holds them.
    pub(crate) fn set_agent(&mut self) {
        self.agent = true;
    }

    /// A code block's paragraph (`SourceCode`): its text goes in a fence
    /// with its runs of spaces, its CriticMarkup delimiters escaped.
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

    pub(crate) fn comment(&mut self, note: &Note) {
        self.tokens.push(Token::Leaf(Leaf::Comment(note.clone())));
    }

    /// Opens a span; `by` is the attribution of an insertion or deletion.
    pub(crate) fn open(&mut self, mark: Mark, by: Option<&str>) {
        self.tokens.push(Token::Open(mark, by.map(str::to_string)));
    }

    /// Open a source revision without letting timestamp elision affect pairing.
    pub(crate) fn open_dated(&mut self, mark: Mark, by: Option<&str>, date: Option<String>) {
        if let Some(by) = by {
            self.source_dates.insert(by.to_string(), date);
        }
        self.open(mark, by);
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

    /// True when there is no visible text and no comment. Text inside an
    /// insertion or deletion counts even when it is only a space: Word shows
    /// it (`{++ ++}`), and dropping it would lose the change's run. So do
    /// spaces carrying underline, highlight or strike, which show on a
    /// space: a signature line of underlined spaces is not a blank
    /// paragraph. Other whitespace (a tab) renders to nothing the marks
    /// could hold, so it stays blank.
    pub(crate) fn is_blank(&self) -> bool {
        let mut changed = 0usize;
        self.tokens.iter().all(|token| match token {
            Token::Leaf(Leaf::Text { text, marks, .. }) => {
                let marked = marks.underline || marks.highlight.is_some() || marks.strike;
                let spaces = !text.is_empty() && text.chars().all(|c| c == ' ');
                let visible = !text.trim().is_empty() || (spaces && marked);
                changed == 0 && !visible
            }
            Token::Leaf(Leaf::Raw(text)) => changed == 0 && text.trim().is_empty(),
            Token::Leaf(Leaf::Comment(_)) => false,
            Token::Open(Mark::Insertion | Mark::Deletion, _) => {
                changed += 1;
                true
            }
            Token::Close(Mark::Insertion | Mark::Deletion) => {
                changed = changed.saturating_sub(1);
                true
            }
            Token::Open(..) | Token::Close(_) => true,
        })
    }

    /// A header/footer scalar: fold revisions as in the body, preserving
    /// the story walker's escaped text, fields and drawings byte for byte.
    pub(crate) fn into_story(self) -> String {
        let nodes = tidy(unnest(tree(self.tokens)));
        let mut pieces = Vec::new();
        render(
            &nodes,
            &mut pieces,
            Escaping {
                code_block: false,
                story: true,
                source_dates: &self.source_dates,
            },
            &mut 0,
        );
        pieces.into_iter().map(|piece| piece.text).collect()
    }

    pub(crate) fn into_inline(self) -> Inline {
        let mut nodes = tidy(unnest(tree(self.tokens)));
        if self.inline_end {
            space_into_last_change(&mut nodes);
        }
        let escaping = Escaping {
            code_block: self.code_block,
            story: false,
            source_dates: &self.source_dates,
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
            Token::Open(mark, _)
                if open
                    .iter()
                    .rposition(|(m, ..)| *m == mark)
                    .is_some_and(|at| {
                        // A span of a kind already open merges into it when only a
                        // comment's scope lies between: a change of the other kind
                        // between them makes a nest the reader must keep (`w:ins >
                        // w:del > w:ins`), not a span to absorb.
                        open[at + 1..]
                            .iter()
                            .all(|(m, ..)| matches!(m, Mark::Scope))
                    }) =>
            {
                merged[mark.index()] += 1;
            }
            Token::Open(mark, by) => open.push((mark, by, Vec::new())),
            Token::Close(mark) if merged[mark.index()] > 0 => merged[mark.index()] -= 1,
            Token::Close(mark) => {
                let Some(at) = open.iter().rposition(|(m, ..)| *m == mark) else {
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

/// Splits a change around every change inside it: CriticMarkup cannot nest
/// one, so `w:ins` holding `w:del` prints `{++a++}…{--x--}…{++b++}` and the
/// writer puts the marks back one inside the other. Each piece keeps the
/// outer change's attribution. A change is a span's direct child; inside a
/// highlight or a comment it may stay.
/// A node is a change, holds one (a comment's scope), or is the pieces
/// one split into: anything the outer change must break around.
fn is_change(node: &Node) -> bool {
    match node {
        Node::Split(..) => true,
        Node::Span(Mark::Scope, _, children) => children.iter().any(is_change),
        Node::Span(..) => true,
        _ => false,
    }
}

/// A piece of a split `mark` made is one of the changes it held, not a
/// piece of the change itself: the writer nests it back inside, so the
/// outer may fold into its note. Same-kind changes merge in [`tree`],
/// so a held change never shares the outer's mark — save a level a
/// deeper fold already dissolved into its sole change, whose note then
/// names the change it sits inside.
fn held_change(piece: &Node, outer: Mark) -> bool {
    match piece {
        Node::Split(..) => true,
        Node::Span(Mark::Scope, _, children) => children.iter().any(is_change),
        Node::Span(mark, by, _) => outer_of(by.as_deref()).is_some() || *mark != outer,
        _ => false,
    }
}

/// Splits a change around every change inside it: CriticMarkup cannot nest
/// one, so `w:ins` holding `w:del` prints `{++a++}…{--x--}…{++b++}` and the
/// writer puts the marks back one inside the other. Each piece keeps the
/// outer change's attribution. A change is a span's direct child; inside a
/// highlight or a comment it may stay.
fn unnest(nodes: Vec<Node>) -> Vec<Node> {
    fn pieces(mark: Mark, by: &Option<String>, children: Vec<Node>) -> Vec<Node> {
        let mut out = Vec::new();
        let mut live = Vec::new();
        for child in children {
            if is_change(&child) {
                if !live.is_empty() {
                    out.push(Node::Span(mark, by.clone(), std::mem::take(&mut live)));
                }
                out.push(match child {
                    Node::Span(Mark::Scope, note, children) => {
                        Node::Span(Mark::Scope, note, pieces(mark, by, children))
                    }
                    child => child,
                });
            } else {
                live.push(child);
            }
        }
        if !live.is_empty() {
            out.push(Node::Span(mark, by.clone(), live));
        }
        out
    }
    nodes
        .into_iter()
        .map(|node| {
            let Node::Span(mark, by, children) = node else {
                return node;
            };
            let children = unnest(children);
            if matches!(mark, Mark::Insertion | Mark::Deletion) && children.iter().any(is_change) {
                Node::Split(mark, by.clone(), pieces(mark, &by, children))
            } else {
                Node::Span(mark, by, children)
            }
        })
        .collect()
}

/// Names `outer` in the note of the change `node` is: `A` inserted what
/// `B` deleted, and the note reads `r3 @B re r1 @A`, so the writer nests
/// them again. Both must be tagged attributions of one view.
fn fold_on(node: &mut Node, outer: Option<&str>) {
    let Some(outer) = outer else {
        return;
    };
    // A row's or cell's own mark wraps the content the container covers;
    // the writer rebuilds it there, not as a wrap around the sole change.
    if outer.ends_with(CARRIED) {
        return;
    }
    let Some(tagged) = outer
        .strip_prefix(TAG)
        .or_else(|| outer.strip_prefix(PLAIN))
    else {
        return;
    };
    match node {
        Node::Span(Mark::Insertion | Mark::Deletion, by, _) => {
            if let Some(own) = by.as_ref()
                && sentinel(own) == sentinel(outer)
                && tag(Some(own)) != tag(Some(outer))
            {
                *by = Some(format!("{own}{ON}{tagged}"));
            }
        }
        Node::Span(Mark::Scope, _, children) | Node::Split(_, _, children) => {
            for child in children {
                fold_on(child, Some(outer));
            }
        }
        _ => {}
    }
}

/// Drops empty spans, joins neighbours of the same kind and attribution, and
/// moves comments out of highlights, at every level.
fn tidy(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for node in nodes {
        let mut comments = Vec::new();
        let node = if let Node::Split(mark, outer, pieces) = node {
            // The pieces tidy as one list: what emptied out of a child change
            // leaves its neighbours adjacent, and they join again. A piece
            // with nothing in it can only hold comments: they move out and
            // the piece goes, so where the outer's changes stand decides
            // its fate, not where a comment chanced to sit.
            let mut pieces = tidy(pieces);
            pieces.retain(|piece| {
                if piece.has_content() {
                    true
                } else {
                    lift_comments(std::slice::from_ref(piece).to_vec(), &mut comments);
                    false
                }
            });
            if pieces.is_empty() {
                out.extend(comments);
                continue;
            }
            // Each held change names its outer, even when live pieces of
            // that outer remain beside it.
            for piece in &mut pieces {
                if held_change(piece, mark) {
                    fold_on(piece, outer.as_deref());
                }
            }
            if pieces.len() == 1 {
                pieces.remove(0)
            } else {
                out.push(Node::Split(mark, outer, pieces));
                out.extend(comments);
                continue;
            }
        } else {
            node
        };
        let Node::Span(mark, by, nodes) = node else {
            out.push(node);
            out.extend(comments);
            continue;
        };
        let nodes = if mark == Mark::Scope {
            lift_comments(nodes, &mut comments)
        } else {
            nodes
        };
        if !nodes.iter().any(Node::has_content) {
            lift_comments(nodes, &mut out);
        } else if let Some(Node::Span(_, _, before)) = out
            .last_mut()
            .filter(|last| matches!(last, Node::Span(m, b, _) if *m == mark && b == &by && (mark == Mark::Scope || tag(b.as_deref()).is_none())))
        {
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
    if by.as_deref().is_some_and(|by| by.starts_with(TAG)) {
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
            Node::Split(mark, outer, pieces) => {
                kept.push(Node::Split(mark, outer, lift_comments(pieces, comments)));
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
struct Escaping<'a> {
    /// A code block: its delimiters are broken the way its fence unwinds
    /// ([`escape_code`]), and nothing else is, since a fence holds the rest
    /// of the text as written.
    code_block: bool,
    /// Story pieces are already escaped and carry complete field/drawing syntax.
    story: bool,
    source_dates: &'a std::collections::HashMap<String, Option<String>>,
}

impl Escaping<'_> {
    fn pairable(self, a: Option<&str>, b: Option<&str>) -> bool {
        let date = |by: Option<&str>| {
            by.and_then(|by| {
                self.source_dates
                    .get(bare(by).split(ON).next().unwrap_or_default())
            })
        };
        pairable_attribution(a, b)
            && match (date(a), date(b)) {
                (Some(a), Some(b)) => a == b,
                _ => true,
            }
    }

    /// `text` escaped: Markdown's marks and CriticMarkup's delimiters both,
    /// so nothing reads back as what it is not. A code span's text is
    /// written as is, since a backslash shows there, and a code block's
    /// delimiters are broken the way its fence unwinds.
    fn text(self, text: &str, marks: Marks) -> String {
        if self.code_block {
            escape_code(text)
        } else if marks.code {
            text.to_string()
        } else {
            escape(&super::agent::escape_markdown(text))
        }
    }
}

fn render(nodes: &[Node], out: &mut Vec<Piece>, escaping: Escaping<'_>, ids: &mut usize) {
    fn flatten(nodes: &[Node], out: &mut Vec<Node>) {
        for node in nodes {
            if let Node::Split(.., pieces) = node {
                flatten(pieces, out);
            } else {
                out.push(node.clone());
            }
        }
    }
    let mut flat = Vec::new();
    flatten(nodes, &mut flat);
    let nodes = flat.as_slice();
    let mut index = 0;
    while index < nodes.len() {
        match (&nodes[index], nodes.get(index + 1)) {
            // Pair a deletion then a live insertion of one change.
            // The old side keeps its parent; a parent on the new side
            // prevents pairing. Insertion then deletion stays in order.
            // A rendered bookmark, comment range or field (w:fldChar)
            // between the marks separates them; proofing, unreferenced
            // bookmarks and empty runs are transparent.
            (
                Node::Span(Mark::Deletion, old_by, old),
                Some(Node::Span(Mark::Insertion, new_by, new)),
            ) if outer_of(new_by.as_deref()).is_none()
                && !holds(new_by.as_deref(), old_by.as_deref())
                && one_change(old_by.as_deref(), new_by.as_deref())
                && escaping.pairable(old_by.as_deref(), new_by.as_deref())
                && new.iter().any(Node::has_content) =>
            {
                *ids += 1;
                let id = *ids;
                marker(out, "{~~", id);
                render(old, out, escaping, ids);
                marker(out, "~>", id);
                render(new, out, escaping, ids);
                marker(out, "~~}", id);
                match (old_by.as_deref(), new_by.as_deref()) {
                    (Some(a), Some(b))
                        if tag(Some(a)).is_some()
                            && escaping.pairable(old_by.as_deref(), new_by.as_deref()) =>
                    {
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
            // The pieces of a split change render side by side, never
            // paired as a substitution: their order is the document's.
            (Node::Split(.., pieces), _) => {
                for piece in pieces {
                    render(std::slice::from_ref(piece), out, escaping, ids);
                }
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
                text: if escaping.story {
                    markdown.clone()
                } else {
                    math_changes(&if escaping.code_block {
                        escape_code(markdown)
                    } else {
                        defuse(markdown)
                    })
                },
                marks: Marks::default(),
                link: None,
                marker: None,
            }),
            (Node::Leaf(Leaf::Comment(comment)), _) => {
                *ids += 1;
                let inner = if escaping.code_block {
                    &comment.code
                } else {
                    &comment.prose
                };
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
/// Code spans are literal to the reader: their marks must stay outside.
fn wrap_markers(pieces: &mut [Piece]) {
    type Format = (Marks, Option<String>);
    let format = |piece: &Piece| -> Format {
        // A marker can remain inside the shared link, but never inside
        // code: the label is parsed Markdown and code contents are literal.
        let marks = if piece.marks.code {
            Marks::default()
        } else {
            piece.marks
        };
        (marks, piece.link.clone())
    };
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
        marker(out, &attribution_note(&display(by)), id);
    }
}

pub(super) fn attribution_note(inner: &str) -> String {
    let mut out = String::from("{");
    let mut escaped = false;
    for c in inner.chars() {
        if matches!(c, '{' | '}') && !escaped {
            out.push('\\');
        }
        out.push(c);
        escaped = c == '\\' && !escaped;
    }
    out.push('}');
    out
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
    let text = DELIMITERS
        .iter()
        .fold(text.to_string(), |text, (from, to)| text.replace(from, to));
    let mut out = String::new();
    for (at, c) in text.char_indices() {
        if c == '{' && crate::markdown::critic::attribution_tag(&text[at..]).is_some() {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The text of a code block, as its fence holds it: a delimiter of
/// [`escape`] with a run of backslashes at its break point (`{` + the
/// run + `++`, `++` + the run + `}`, `~` + the run + `>`) gains enough of
/// them that the run is odd, and every other backslash is left alone. A
/// fence reads no Markdown escape, so the writer unwinds exactly those
/// odd runs there (see `crate::markdown::critic`), and hand-written code
/// (`C:\path`, a regex `\\d+`) passes through as written.
pub(crate) fn escape_code(text: &str) -> String {
    /// What can follow the backslash run an opener holds (`{` + run).
    const OPENER: [&str; 5] = ["++", "--", "~~", ">>", "=="];
    /// What can hold the run a closer escapes (`++` + run + `}`).
    const CLOSER: [&str; 5] = ["++", "--", "~~", "<<", "=="];
    let bytes = text.as_bytes();
    // The delimiter shape at `at`, if one sits there: its head, the
    // backslash run at its break point, and its tail.
    let shape = |at: usize| -> Option<(usize, usize, usize)> {
        let shape = |head: usize, tails: &[&str]| {
            let mut run = at + head;
            while bytes.get(run) == Some(&b'\\') {
                run += 1;
            }
            let tail = tails
                .iter()
                .find(|tail| text[run..].starts_with(*tail))?
                .len();
            Some((head, run - (at + head), tail))
        };
        if bytes.get(at) == Some(&b'{') {
            shape(1, &OPENER)
        } else if CLOSER.iter().any(|head| text[at..].starts_with(head)) {
            shape(2, &["}"])
        } else if bytes.get(at) == Some(&b'~') {
            shape(1, &[">"])
        } else {
            None
        }
    };
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        // Tags are metadata only after a closer or a pilcrow, but a code
        // fence must preserve the same literal text in either position.
        let mut tag_at = at;
        while bytes.get(tag_at) == Some(&b'\\') {
            tag_at += 1;
        }
        if let Some((_, width)) = crate::markdown::critic::attribution_tag(&text[tag_at..]) {
            out.push_str(&"\\".repeat((tag_at - at) * 2 + 1));
            out.push_str(&text[tag_at..tag_at + width]);
            at = tag_at + width;
            continue;
        }

        if let Some((head, run, tail)) = shape(at) {
            out.push_str(&text[at..at + head]);
            for _ in 0..run * 2 + 1 {
                out.push('\\');
            }
            out.push_str(&text[at + head + run..at + head + run + tail]);
            at += head + run + tail;
        } else {
            let c = text[at..].chars().next().unwrap_or_default();
            out.push(c);
            at += c.len_utf8();
        }
    }
    out
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
        ("++}", "++{}}"),
        ("--}", "--{}}"),
        ("~~}", "~~{}}"),
        ("<<}", "<<{}}"),
        ("==}", "=={}}"),
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
/// with [`crate::markdown::math::INSERT_START`] and its kin so `defuse` leaves them whole.
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

/// Whether `out` ends with a line of backticks: a code fence, which must end
/// its line. A CriticMarkup opener written after it (` ```{++ `) reads as the
/// fence's info string, so the fence stops closing its block and the block
/// swallows what follows (#587).
fn ends_with_fence(out: &str) -> bool {
    out.lines().next_back().is_some_and(|line| {
        let line = line.trim();
        line.len() >= 3 && line.bytes().all(|b| b == b'`')
    })
}

/// Appends `prefix` and `body` to `out` after `separator`. When the separator is
/// itself tracked (a paragraph mark that was inserted or deleted), it is marked
/// the way the CriticMarkup spec marks a paragraph break (`{++\n\n++}`), with
/// its attribution after it. An adjoining span of the same kind and attribution
/// is extended rather than opened twice. Before a code fence the opener cannot
/// share the fence's line ([`ends_with_fence`]): the break is written first and
/// the span opens on the next block's line, or is dropped when it would wrap
/// nothing but the break.
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
    let shown = by.as_deref().map(display);
    let by = shown.as_deref().map(attribution_note).unwrap_or_default();
    let end = format!("{close}{by}");
    let continues = body.strip_prefix(open).filter(|rest| {
        rest.find(close)
            .is_some_and(|at| rest[at + close.len()..].starts_with(&by))
    });
    let fence = ends_with_fence(out);
    if let Some(kept) = out.strip_suffix(&end) {
        out.truncate(kept.len());
    } else if fence {
        out.push_str(separator);
        out.push_str(prefix);
        if let Some(rest) = continues {
            out.push_str(open);
            out.push_str(rest);
        } else {
            out.push_str(body);
        }
        return;
    } else {
        out.push_str(open);
    }
    out.push_str(separator);
    out.push_str(prefix);
    if let Some(rest) = continues {
        out.push_str(rest);
    } else {
        out.push_str(&end);
        if space {
            out.push(' ');
        }
        out.push_str(body);
    }
}

/// `{--gone--}{r1 @A}` is `(Deletion, Some("r1 @A"))`. Document text is
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
                    let (by, width) = crate::markdown::critic::attribution_tag(after)?;
                    (width == after.len()).then(|| (mark, Some(by.to_string())))
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use Mark::{Deletion as Del, Insertion as Ins, Scope as Sc};

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
                "note" => critic.comment(&Note::same(value)),
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
            _ => Sc,
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
    fn a_deletion_then_an_insertion_is_a_substitution_and_the_reverse_two_changes() {
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
        assert_eq!(script(&ins_del), "{++new++}{--old--}");
        assert_eq!(script(&del_ins), "{~~old~>new~~}");
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
        assert_eq!(script(&triple), "{~~a~>b~~}{--c--}");
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
    fn different_kinds_cannot_nest() {
        // CriticMarkup cannot nest a change in a change: an insertion that
        // is only the deletion splits away entirely, each piece keeping its
        // attribution.
        let nested = [
            ("open", "+"),
            ("open", "-"),
            ("t", "x"),
            ("close", "-"),
            ("close", "+"),
        ];
        assert_eq!(script(&nested), "{--x--}");
    }

    #[test]
    fn a_change_inside_a_change_splits_the_outer_one() {
        let nested = [
            ("by+", "Ana"),
            ("t", "one "),
            ("by-", "Ben"),
            ("t", "two"),
            ("close", "-"),
            ("t", " three"),
            ("close", "+"),
        ];
        assert_eq!(
            script(&nested),
            "{++one ++}{Ana}{--two--}{Ben}{++ three++}{Ana}"
        );
    }

    #[test]
    fn a_change_that_is_another_s_only_content_names_it_in_the_note() {
        // `A` inserted what `B` deleted: nothing of the insertion is left
        // to split around the deletion, so the note names it after ` re `.
        let nested = [
            ("by+", "\u{E000}1@AL"),
            ("by-", "\u{E000}3@BO"),
            ("t", "x"),
            ("close", "-"),
            ("close", "+"),
        ];
        assert_eq!(script(&nested), "{--x--}{r3 @BO re r1 @AL}");
        // Each tag keeps its own timestamp, as notes print them.
        let dated = [
            ("by+", "\u{E000}1@AL 2026-01-01T09:00:00Z"),
            ("by-", "\u{E000}3@BO 2026-01-02T10:00:00Z"),
            ("t", "x"),
            ("close", "-"),
            ("close", "+"),
        ];
        assert_eq!(
            script(&dated),
            "{--x--}{r3 @BO 2026-01-02T10:00:00Z re r1 @AL 2026-01-01T09:00:00Z}"
        );
        // The plain view names the same change by its author and date.
        let plain = [
            ("by+", "\u{E001}1@Ana%20Lima%20(2026-01-01T09:00:00Z)"),
            ("by-", "\u{E001}3@Ben%20Ode%20(2026-01-02T10:00:00Z)"),
            ("t", "x"),
            ("close", "-"),
            ("close", "+"),
        ];
        assert_eq!(
            script(&plain),
            "{--x--}{r3 Ben Ode (2026-01-02T10:00:00Z) \
             re r1 Ana Lima (2026-01-01T09:00:00Z)}"
        );
    }

    #[test]
    fn a_note_that_names_no_change_keeps_the_sole_one() {
        // Untagged changes have no note to name the outer one in, so the
        // outer stays gone, as joined crossings always were.
        let nested = [
            ("by+", "Ana"),
            ("by-", "Ben"),
            ("t", "x"),
            ("close", "-"),
            ("close", "+"),
        ];
        assert_eq!(script(&nested), "{--x--}{Ben}");
    }

    #[test]
    fn an_insertion_inside_a_deletion_splits_it_too() {
        let nested = [
            ("by-", "Ben"),
            ("t", "one "),
            ("by+", "Ana"),
            ("t", "two"),
            ("close", "+"),
            ("t", " three"),
            ("close", "-"),
        ];
        assert_eq!(
            script(&nested),
            "{--one --}{Ben}{++two++}{Ana}{-- three--}{Ben}"
        );
    }

    #[test]
    fn split_pieces_do_not_pair_as_a_substitution() {
        // `one` was inserted before `two`, not instead of it: the pieces
        // the nested change split keep their order.
        let nested = [
            ("by+", "Ana"),
            ("t", "one "),
            ("by-", "Ben"),
            ("t", "two"),
            ("close", "-"),
            ("t", " three"),
            ("close", "+"),
            ("by-", "Ben"),
            ("t", "four"),
            ("close", "-"),
        ];
        assert_eq!(
            script(&nested),
            "{++one ++}{Ana}{--two--}{Ben}{++ three++}{Ana}\
             {--four--}{Ben}"
        );
    }

    #[test]
    fn a_nest_inside_a_nest_splits_at_every_level() {
        // Two changes inside one: the outer change splits around each.
        let nested = [
            ("by+", "Ana"),
            ("t", "a"),
            ("by-", "Ben"),
            ("t", "x"),
            ("close", "-"),
            ("t", "y"),
            ("by-", "Cid"),
            ("t", "z"),
            ("close", "-"),
            ("t", "b"),
            ("close", "+"),
        ];
        assert_eq!(
            script(&nested),
            "{++a++}{Ana}{--x--}{Ben}{++y++}{Ana}{--z--}{Cid}{++b++}{Ana}"
        );
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
        // Joining exposes inner neighbours, which join too. The deletions
        // around them split away: a change cannot hold a change.
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
        assert_eq!(script(&deep), "{++ab++}");
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
        let mut out = "{--A--}{Ana}".to_string();
        splice(
            &mut out,
            "\n\n",
            "",
            &body,
            Some(&(Del, Some("Ana".into()))),
            false,
        );
        assert_eq!(out, "{--A\n\n$x_{{}--{}}$--}{Ana}");
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
    fn blankness_counts_changed_text_but_ignores_markers() {
        // An inserted or deleted space is text Word shows (`{++ ++}`), so it
        // keeps the paragraph's line; a space only highlighted does not.
        let mut critic = Critic::default();
        critic.open(Ins, None);
        text(&mut critic, "  ");
        critic.raw(" ");
        critic.close(Ins);
        assert!(!critic.is_blank());
        let mut scope = Critic::default();
        scope.open(Mark::Scope, None);
        text(&mut scope, "  ");
        scope.close(Mark::Scope);
        assert!(scope.is_blank());
        scope.comment(&Note::same("note"));
        assert!(!scope.is_blank());
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
    fn a_span_after_a_code_fence_opens_on_the_next_block_s_line() {
        // The closing fence of a code block must end its line: an opener
        // pushed after its backticks (` ```{++ `) reads as the fence's info
        // string, the block never closes, and it swallows the span and the
        // blocks after it (#587).
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
        let fence = "```\nsome code\n```";
        // The paragraph's own inserted runs keep their span, on the line
        // after the fence: the reader marks the new paragraph's break again
        // from the span alone.
        assert_eq!(
            join(fence, "{++B++}", Some(Ins)),
            "```\nsome code\n```\n\n{++B++}"
        );
        // A mark whose span would cover only the break is dropped, as a
        // tracked break before a table is: written after the fence it wraps
        // nothing (`{++++}`), and written into the fence line it reopens it.
        assert_eq!(join(fence, "B", Some(Ins)), "```\nsome code\n```\n\nB");
        assert_eq!(join(fence, "B", Some(Del)), "```\nsome code\n```\n\nB");
        // The fence's length or indentation does not matter.
        assert_eq!(
            join("  ````\ncode\n````", "{++B++}", Some(Ins)),
            "  ````\ncode\n````\n\n{++B++}"
        );
        // A heading keeps its `## ` outside the span, so it stays a heading.
        let mut heading = "```\ncode\n```".to_string();
        splice(
            &mut heading,
            "\n\n",
            "## ",
            "{++B++}",
            Some(&(Ins, None)),
            false,
        );
        assert_eq!(heading, "```\ncode\n```\n\n## {++B++}");
        // After a block that is not a fence, the span still covers the break.
        assert_eq!(join("A", "B", Some(Ins)), "A{++\n\n++}B");
    }

    #[test]
    fn a_plain_span_across_paragraphs_names_every_piece_s_id() {
        let mark = |id: &str| Some(format!("{PLAIN}{id}@Bo%20Chen"));
        let join = |out: &str, body: &str, id: &str| {
            let mut out = out.to_string();
            splice(&mut out, "\n\n", "", body, Some(&(Del, mark(id))), false);
            out
        };
        // The deleted text before the break, the break, the text after it.
        assert_eq!(
            join("a{--A--}{r8 Bo Chen}", "{--B--}{r10 Bo Chen} b", "9"),
            "a{--A--}{r8 Bo Chen}{--\n\n--}{r9 Bo Chen}{--B--}{r10 Bo Chen} b"
        );
        assert_eq!(
            join("{--A--}{r8 Bo Chen}", "B", "9"),
            "{--A--}{r8 Bo Chen}{--\n\n--}{r9 Bo Chen}B"
        );
        assert_eq!(
            join("A", "{--B--}{r10 Bo Chen}", "9"),
            "A{--\n\n--}{r9 Bo Chen}{--B--}{r10 Bo Chen}"
        );
        let notes = |id: &str| Some(format!("{PLAIN}footnotes#{id}@Bo"));
        let mut out = "{--A--}{footnotes.r2 Bo}".to_string();
        splice(&mut out, "\n\n", "", "B", Some(&(Del, notes("3"))), false);
        assert_eq!(out, "{--A--}{footnotes.r2 Bo}{--\n\n--}{footnotes.r3 Bo}B");
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
            ("{++Added para++}{noid Alice}".to_string(), None, bare),
        ];
        assert_eq!(
            join_marked(parts, "<br>"),
            "Para{++<br>Added para++}{noid Alice}"
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
            "{++new++}{Ana (2026-01-02T03:04:00Z)} and {--old--}{Bo}."
        );
    }

    #[test]
    fn a_story_substitution_keeps_each_inline_date_when_written_back() {
        let mut critic = Critic::default();
        for (mark, tag, text) in [
            (Ins, "header1#1@BO 2026-09-30T09:00:00Z", "new"),
            (Del, "header1#2@AL 2026-09-29T14:05:00Z", "old"),
        ] {
            critic.open(mark, Some(&format!("{TAG}{tag}")));
            critic.raw(text);
            critic.close(mark);
        }
        let story = critic.into_story();
        let view = format!(
            "---\nauthors:\n  AL: Alice\n  BO: Bob\nheaders:\n  default: {{id: header1, text: \"{story}\"}}\n---\nBody.\n"
        );
        let written =
            crate::markdown::markdown_to_docx(&view, &crate::markdown::DocxOptions::default())
                .unwrap();
        let package = crate::opc::PartFs::open(&written.docx).unwrap();
        let xml = package.part_string("word/header1.xml").unwrap();
        let header = super::super::ooxml::parse_xml(xml.as_bytes()).unwrap();
        let paragraph = header
            .elements()
            .find(|element| element.local() == "p")
            .unwrap();
        let revisions: Vec<_> = paragraph
            .elements()
            .filter(|element| matches!(element.local(), "ins" | "del"))
            .map(|element| {
                (
                    element.local(),
                    element.attr("author"),
                    element.attr("date"),
                )
            })
            .collect();
        assert_eq!(
            revisions,
            [
                ("ins", Some("Bob"), Some("2026-09-30T09:00:00Z")),
                ("del", Some("Alice"), Some("2026-09-29T14:05:00Z")),
            ],
            "{view}\n{xml}"
        );
    }

    #[test]
    fn adjacent_changes_keep_each_author_in_document_order() {
        let same = [
            ("by+", "Ana"),
            ("t", "new"),
            ("close", "+"),
            ("by-", "Ana"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&same), "{++new++}{Ana}{--old--}{Ana}");
        // Each author follows in the order of old then new.
        let different = [
            ("by+", "Bo"),
            ("t", "new"),
            ("close", "+"),
            ("by-", "Ana"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&different), "{++new++}{Bo}{--old--}{Ana}");
        let one_side = [
            ("by+", "Bo"),
            ("t", "new"),
            ("close", "+"),
            ("open", "-"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&one_side), "{++new++}{Bo}{--old--}");
        let other_side = [
            ("open", "+"),
            ("t", "new"),
            ("close", "+"),
            ("by-", "Ana"),
            ("t", "old"),
            ("close", "-"),
        ];
        assert_eq!(script(&other_side), "{++new++}{--old--}{Ana}");
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
        assert_eq!(script(&same), "{++ab++}{Ana}");
        let different = [
            ("by+", "Ana"),
            ("t", "a"),
            ("close", "+"),
            ("by+", "Bo"),
            ("t", "b"),
            ("close", "+"),
        ];
        assert_eq!(script(&different), "{++a++}{Ana}{++b++}{Bo}");
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
        assert_eq!(script(&steps), "{==a{++b++}{Bo}==}{>>Ana: why?<<}");
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
        assert_eq!(script(&nested), "{--ab--}{Row}");
        let crossing = [
            ("open", "="),
            ("t", "a"),
            ("by+", "Bo"),
            ("t", "b"),
            ("close", "="),
            ("t", "c"),
            ("close", "+"),
        ];
        assert_eq!(script(&crossing), "{==a{++b++}{Bo}==}{++c++}{Bo}");
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
        assert_eq!(join("A", "B", ana()), "A{--\n\n--}{Ana}B");
        assert_eq!(
            join("{--A--}{Ana}", "{--B--}{Ana} rest", ana()),
            "{--A\n\nB--}{Ana} rest"
        );
        // Someone else's deletion on either side stays its own span.
        assert_eq!(
            join("{--A--}{Bo}", "{--B--}{Bo}", ana()),
            "{--A--}{Bo}{--\n\n--}{Ana}{--B--}{Bo}"
        );
        // A body span of the same kind whose closer is followed by nothing
        // attributed is not the same change.
        assert_eq!(
            join("A", "{--B--} rest", ana()),
            "A{--\n\n--}{Ana}{--B--} rest"
        );
        assert_eq!(join("A", "{--B", ana()), "A{--\n\n--}{Ana}{--B");
    }

    #[test]
    fn tree_ignores_a_close_with_nothing_open() {
        assert_eq!(tree(vec![Token::Close(Ins)]), Vec::<Node>::new());
    }

    #[test]
    fn review_5_literal_braces_are_not_whole_change_attributions() {
        for literal in ["{X Y}", "{>>c0 @A: note<<}", "{r1 @A}{literal}"] {
            assert_eq!(
                whole_change(&format!("{{--a--}}{literal}")),
                None,
                "{literal}"
            );
        }
        assert_eq!(
            whole_change("{--a--}{r1 @A}"),
            Some((Del, Some("r1 @A".into())))
        );
    }

    #[test]
    fn a_separator_next_to_a_wholly_changed_part_joins_its_change() {
        let by = "{noid Alice}";
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
        assert_eq!(md(critic), "**around {++an insertion++}{Ana} inside** end");
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
        critic.open(Sc, None);
        critic.push("varied ", false, false, None);
        critic.push("Emphasis lives here. ", true, true, None);
        critic.close(Sc);
        critic.comment(&Note::same("c2 @AS: Testing."));
        critic.push("See note.", true, true, None);
        assert_eq!(
            md(critic),
            "{==varied **_Emphasis lives here._** ==}{>>c2 @AS: Testing.<<}**_See note._**"
        );
        // A note alone between two bold runs still joins them.
        let mut critic = Critic::default();
        critic.push("bold ", true, false, None);
        critic.comment(&Note::same("c0 @AS: Here."));
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
        assert_eq!(md(critic), "[the {++new ++}{Ana}page](https://example.com)");
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

    #[test]
    fn a_note_prints_the_form_its_context_escapes() {
        // A note anchored in a code block is written inside its fence, where
        // the writer unwinds only the fence's escape (#518): the prose form
        // would double its backslashes on every round trip.
        let note = Note {
            prose: r"prose \ and \* form".to_string(),
            code: r"code \ and * form".to_string(),
        };
        let mut prose = Critic::default();
        prose.comment(&note);
        assert_eq!(md(prose), r"{>>prose \ and \* form<<}");
        let mut code = Critic::default();
        code.set_code_block();
        code.comment(&note);
        assert_eq!(md(code), r"{>>code \ and * form<<}");
    }

    #[test]
    fn a_code_block_breaks_only_the_delimiter_shapes_it_holds() {
        // Inside a fence Markdown reads no backslash escape, so the pair
        // has to say itself what is text: a delimiter shape with a run of
        // backslashes at its break point gains enough of them that the run
        // is odd, and every other backslash the document holds is left
        // alone. The writer unwinds exactly those runs there, so code a
        // human wrote (`C:\path`, a regex `\\d+`) is not touched either.
        let code = |t: &str| {
            let mut critic = Critic::default();
            critic.set_code_block();
            text(&mut critic, t);
            md(critic)
        };
        assert_eq!(code("fenced {++x++} code"), "fenced {\\++x++\\} code");
        for (document, markdown) in [
            (r"{\++x++}", r"{\\\++x++\}"),
            (r"{\\++x++}", r"{\\\\\++x++\}"),
            (r"\{++", r"\{\++"),
            ("a~>b", r"a~\>b"),
            (r"a\~>b", r"a\~\>b"),
            (r"back\slash", r"back\slash"),
            (r"C:\path\x", r"C:\path\x"),
            (r"\\d+", r"\\d+"),
            (r"a\\b", r"a\\b"),
        ] {
            assert_eq!(code(document), markdown, "{document}");
        }
        // A tracked change keeps its delimiters: the fence still reads
        // CriticMarkup, so only the text inside is escaped, even where
        // that text ends in a backslash right at a delimiter.
        let mut critic = Critic::default();
        critic.set_code_block();
        critic.open(Ins, None);
        text(&mut critic, "new {++x++}");
        critic.close(Ins);
        assert_eq!(md(critic), "{++new {\\++x++\\}++}");
        let mut critic = Critic::default();
        critic.set_code_block();
        text(&mut critic, "old\\");
        critic.open(Ins, None);
        text(&mut critic, "new");
        critic.close(Ins);
        assert_eq!(md(critic), "old\\{++new++}");
        let mut critic = Critic::default();
        critic.set_code_block();
        critic.open(Ins, None);
        text(&mut critic, "new\\");
        critic.close(Ins);
        assert_eq!(md(critic), "{++new\\++}");
        let mut critic = Critic::default();
        critic.set_code_block();
        critic.open(Ins, None);
        text(&mut critic, "n");
        critic.close(Ins);
        critic.open(Del, None);
        text(&mut critic, "old\\");
        critic.close(Del);
        assert_eq!(md(critic), "{++n++}{--old\\--}");
        // Converter-built Markdown (an equation, an image) is fence text
        // too, and comes back the same way.
        let mut critic = Critic::default();
        critic.set_code_block();
        critic.raw("$x_{--}$ and ![a {++b++}](u\\v)");
        assert_eq!(md(critic), "$x_{\\--}$ and ![a {\\++b++\\}](u\\v)");
    }

    #[test]
    fn the_writer_unwinds_the_code_escape_inside_a_fence() {
        // `encode` reads a fence's escaped text back as the document held
        // it, and still opens the changes the reader wrote there.
        let fence = |code: &str| format!("```\n{code}\n```\n");
        for document in [
            "fenced {++x++} code",
            "{\\++x++}",
            "{\\\\++x++}",
            "\\{++",
            "a~>b",
            "a\\~>b",
            "back\\slash",
            "C:\\path\\x",
            "\\\\d+",
            "a\\\\b",
        ] {
            let source = fence(&escape_code(document));
            assert_eq!(
                crate::markdown::critic::encode(&source),
                fence(document),
                "{document}"
            );
        }
        let mut critic = Critic::default();
        critic.set_code_block();
        critic.open(Ins, None);
        text(&mut critic, "new");
        critic.close(Ins);
        assert_eq!(md(critic), "{++new++}");
    }

    #[test]
    fn the_fence_escape_pair_is_exact() {
        // Over every mix of backslash runs and delimiter shapes:
        // unwind(escape_code(t)) == t, and escape_code(unwind(s)) == s for
        // every s escape_code can produce, so no code text can reach Word
        // changed or name another document's fence.
        let fence = |code: &str| format!("```\n{code}\n```\n");
        let unwind = |escaped: &str| {
            let whole = crate::markdown::critic::encode(&fence(escaped));
            whole["```\n".len()..whole.len() - "\n```\n".len()].to_string()
        };
        let shapes = [
            ("{", "++"),
            ("{", "--"),
            ("{", "~~"),
            ("{", ">>"),
            ("{", "=="),
            ("++", "}"),
            ("--", "}"),
            ("~~", "}"),
            ("<<", "}"),
            ("==", "}"),
            ("~", ">"),
        ];
        let mut texts: Vec<String> = Vec::new();
        for run in 0..5 {
            let backslashes = "\\".repeat(run);
            for (head, tail) in shapes {
                texts.push(format!("a{head}{backslashes}{tail}b"));
                texts.push(format!("{head}{backslashes}{tail}"));
                texts.push(format!("x{backslashes} {head}{backslashes}{tail}"));
            }
            texts.push(format!("u{backslashes}v"));
        }
        texts.extend([
            "{++x++}".to_string(),
            "{\\\\}".to_string(),
            "~~}".to_string(),
            "{~~a~>b~~}".to_string(),
            "C:\\path\\x".to_string(),
            "\\\\d+".to_string(),
        ]);
        for text in &texts {
            let escaped = escape_code(text);
            assert_eq!(unwind(&escaped), *text, "unwind(escape_code({text:?}))");
            assert_eq!(
                escape_code(&unwind(&escaped)),
                escaped,
                "escape_code(unwind({escaped:?}))"
            );
        }
    }
}
