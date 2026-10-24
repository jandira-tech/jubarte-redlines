// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2024-2026 SylphxAI
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word to Markdown: headings, emphasis, links, lists, tables, footnotes,
//! equations, and tracked changes and comments as CriticMarkup.
//!
//! Copied from anymd's `anymd-formats` crate (MIT, see
//! `LICENSES/LicenseRef-anymd-MIT.txt`) and adapted: pictures are collected
//! for the caller to write instead of going to anymd's cache, and document
//! properties are not read.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;

mod agent;
mod critic;
pub(crate) mod drawing;
mod header;
mod media;
mod mermaid;
mod ooxml;
mod revise;

use critic::{Change, Critic, Mark};
use ooxml::{Blocks, ListIndent, Marks, Media, Package, Rels};
pub(crate) use ooxml::{Element, parse_xml};

/// How tracked changes and comments come out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Revisions {
    /// Tracked changes and comments as CriticMarkup.
    #[default]
    Markup,
    /// The text with every change accepted; comments are left out.
    Accept,
    /// The text with every change rejected; comments are left out.
    Reject,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Options {
    pub(crate) revisions: Revisions,
    /// Collect raster pictures and name them under this directory. `None`
    /// writes pictures as their alt text.
    pub(crate) media_dir: Option<String>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConvertError {
    /// The bytes are not a readable `.docx`.
    Invalid(String),
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Converted {
    pub(crate) markdown: String,
    /// The body paragraphs a selection kept (`pN` numbers, ascending);
    /// `None` without [`Options::select`].
    pub(crate) selected: Option<Vec<usize>>,
    /// The header, footer and note stories a `--changed` selection keeps.
    pub(crate) selected_stories: Vec<String>,
    /// Pictures by file name, under [`Options::media_dir`].
    pub(crate) media: BTreeMap<String, Vec<u8>>,
}

/// A part's file stem: `word/footnotes.xml` → `footnotes`, the story
/// `jubarte changes` names its changes by.
fn part_stem(path: &str) -> String {
    crate::ids::part_story(path)
}

pub(crate) fn convert(bytes: &[u8], options: &Options) -> Result<Converted, ConvertError> {
    let mut package = Package::open(bytes, "DOCX")?;
    let main = package.main_part("word/document.xml")?;
    let mut document = package
        .xml(&main)?
        .ok_or_else(|| ooxml::invalid("DOCX has no word/document.xml part"))?;
    let accept = match options.revisions {
        Revisions::Markup => None,
        Revisions::Accept => Some(true),
        Revisions::Reject => Some(false),
    };
    let rels = package.rels(&main)?;
    let part = |kind: &str, fallback: &str| {
        rels.first_of_type(kind)
            .map(|r| r.target.clone())
            .unwrap_or_else(|| fallback.to_string())
    };
    let comments_root = package
        .xml(&part("/comments", "word/comments.xml"))
        .ok()
        .flatten();
    let extended = if options.ids {
        package
            .xml(&part("/commentsExtended", "word/commentsExtended.xml"))
            .ok()
            .flatten()
    } else {
        None
    };
    let threads = agent::threads(comments_root.as_ref(), extended.as_ref());
    let mut note_roots = Vec::new();
    // The notes parts' stems (`footnotes`), which name their changes' story.
    let mut note_stories: HashMap<bool, String> = HashMap::new();
    // Each notes part's name and `xmlns` scope, for drawings it holds.
    let mut note_scopes: HashMap<bool, (String, drawing::ScopeMap)> = HashMap::new();
    for (kind, fallback, name) in [
        ("/footnotes", "word/footnotes.xml", "footnote"),
        ("/endnotes", "word/endnotes.xml", "endnote"),
    ] {
        let path = part(kind, fallback);
        if let Ok(Some(root)) = package.xml(&path) {
            note_stories.insert(name == "endnote", part_stem(&path));
            note_scopes.insert(name == "endnote", (path, drawing::ScopeMap::of(&root)));
            note_roots.push((name, root));
        }
    }
    // Header and footer parts, for their comment anchors and changes.
    let mut story_roots = Vec::new();
    let mut story_stems = Vec::new();
    for target in rels
        .targets_of_type("/header")
        .into_iter()
        .chain(rels.targets_of_type("/footer"))
    {
        if let Ok(Some(root)) = package.xml(&target) {
            story_roots.push(root);
            story_stems.push(part_stem(&target));
        }
    }
    // Auxiliary parts are best-effort: a broken styles part should not lose the text.
    let styles_root = package
        .xml(&part("/styles", "word/styles.xml"))
        .ok()
        .flatten();
    let styles = styles_root.as_ref().map(Styles::parse).unwrap_or_default();
    let mut handles = if options.ids {
        // An author whose only changes sit in a header or the styles gets a
        // handle too, so `--by` takes it; the styles' come after the comment
        // authors', whose handles stay as they were.
        let stories: Vec<&Element> = note_roots
            .iter()
            .map(|(_, root)| root)
            .chain(&story_roots)
            .collect();
        agent::handles(
            &document,
            &stories,
            comments_root.as_ref(),
            styles_root.as_ref(),
        )
    } else {
        agent::Handles::default()
    };
    if options.ids {
        // What separates two marks in the text: a printed comment range, a
        // printed bookmark.
        if options.comments
            && let Some(root) = &comments_root
        {
            handles.shown_ranges = root
                .children_named("comment")
                .filter_map(|c| c.attr("id").map(str::to_string))
                .collect();
        }
        handles.hidden_styles = styles.hidden();
        handles.shown_bookmarks = bookmark_targets(
            std::iter::once(&document)
                .chain(note_roots.iter().map(|(_, root)| root))
                .chain(comments_root.as_ref()),
            true,
        );
        // A heading's own slug bookmark prints no anchor (the link reaches
        // the heading by its slug): it keeps no two marks apart.
        let mut slugger = super::write::Slugger::default();
        for p in body_headings(&document, &styles) {
            let own = own_slug(&mut slugger, p);
            let mut starts = Vec::new();
            p.find_all("bookmarkStart", &mut starts);
            for start in starts {
                if let Some(name) = start.attr("name")
                    && is_own_slug(name, own.as_deref())
                {
                    handles.shown_bookmarks.remove(name);
                }
            }
        }
    }
    // Agent view: paragraphs and tables numbered before accept/reject
    // resolution, so indices match `inspect` and `edit`.
    // (paragraphs, tables) numbered, for the header.
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
    // Cached-break page lines: only in the agent view, without layout pages,
    // and only when page lines are wanted at all.
    let cached_pages =
        (options.ids && options.pages.is_none() && options.page_markers).then(|| {
            let body = document.child("body").unwrap_or(&document);
            let (rendered, hard) = agent::page_counts(body);
            let total = 1 + if rendered > 0 { rendered } else { hard };
            // Without cached breaks the writer turns pages on hard breaks.
            let sections = (rendered == 0).then(|| agent::page_sections(body));
            (total, sections)
        });
    let (cached_pages, page_sections) = match cached_pages {
        Some((total, sections)) => (Some(total), sections),
        None => (None, None),
    };
    let document = match accept {
        Some(accept) => revise::resolve(&document, accept),
        None => document,
    };

    let default_style = styles_root.as_ref().and_then(|root| {
        root.children_named("style")
            .find(|s| {
                s.attr("type") == Some("paragraph")
                    && s.attr("default").is_some_and(|d| d == "1" || d == "true")
            })
            .and_then(|s| s.attr("styleId"))
            .map(str::to_string)
    });
    let numbering = package
        .xml(&part("/numbering", "word/numbering.xml"))
        .ok()
        .flatten()
        .map(|e| Numbering::parse(&e))
        .unwrap_or_default();
    let mut notes = HashMap::new();
    for (name, root) in &note_roots {
        for note in root.children_named(name) {
            if let Some(id) = note.attr("id") {
                let note = match accept {
                    Some(accept) => revise::resolve(note, accept),
                    None => note.clone(),
                };
                notes.insert((*name == "endnote", id.to_string()), note);
            }
        }
    }
    // Notes with no surviving reference are pruned from the settled file
    // (`prune_orphan_notes`), so settled numbering skips them too. In the
    // tracked view every reference survives.
    let referenced: HashSet<(bool, String)> = if let Some(accept) = accept {
        // Headers and footers must be settled too: a rejected insertion
        // (or accepted deletion) there no longer keeps its note alive.
        let settled_stories: Vec<_> = story_roots
            .iter()
            .map(|root| revise::resolve(root, accept))
            .collect();
        let mut out = HashSet::new();
        let mut stack: Vec<&Element> = vec![&document];
        stack.extend(settled_stories.iter());
        stack.extend(notes.values());
        while let Some(next) = stack.pop() {
            for child in next.elements() {
                if (child.is("footnoteReference") || child.is("endnoteReference"))
                    && let Some(id) = child.attr("id")
                {
                    out.insert((child.is("endnoteReference"), id.to_string()));
                }
                stack.push(child);
            }
        }
        out
    } else {
        HashSet::new()
    };
    // Each note's paragraphs as `edit` numbers its story
    // (`inspect::story_paragraph_nodes`): the first index and how many;
    // separator notes are not numbered. Counted after accept/reject
    // resolution, so the ids name the paragraphs of the view they print in.
    let mut note_paragraphs: HashMap<(bool, String), (usize, usize)> = HashMap::new();
    for (name, root) in &note_roots {
        let mut first = 0;
        for note in root.children_named(name) {
            if note.attr("type").is_some_and(|t| {
                matches!(
                    t,
                    "separator" | "continuationSeparator" | "continuationNotice"
                )
            }) {
                continue;
            }
            let key = note
                .attr("id")
                .map(|id| (*name == "endnote", id.to_string()));
            if accept.is_some() && key.as_ref().is_none_or(|key| !referenced.contains(key)) {
                continue;
            }
            let count = key
                .as_ref()
                .and_then(|key| notes.get(key))
                .map(agent::paragraphs)
                .map_or_else(|| agent::paragraphs(note).len(), |v| v.len());
            if let Some(key) = key {
                note_paragraphs.insert(key, (first, count));
            }
            first += count;
        }
    }
    // Accepting or rejecting the changes leaves comments out as well, except
    // in the agent view, which keeps them.
    let comments = comments_root
        .clone()
        .filter(|_| accept.is_none() || options.ids);
    let media = Media::load(
        &mut package,
        &rels,
        options.media_dir.as_deref().map(media::Extracted::new),
    );
    let hidden_styles = styles.hidden();
    let mut writer = Writer {
        rels: &rels,
        media,
        hidden_styles,
        styles,
        numbering,
        counters: HashMap::new(),
        started_nums: Vec::new(),
        note_refs: Vec::new(),
        note_changes: Vec::new(),
        lead: None,
        flattening: 0,
        heading_slug: None,
        slugger: super::write::Slugger::default(),
        tables_differ: {
            let (styles, unstyled) = table_styles(
                document.child("body").unwrap_or(&document),
                custom_table_grid(styles_root.as_ref()),
            );
            styles.len() > 1 || (!styles.is_empty() && unstyled)
        },
        custom_grid: custom_table_grid(styles_root.as_ref()),
        bookmarks: bookmark_targets(
            std::iter::once(&document)
                .chain(note_roots.iter().map(|(_, root)| root))
                .chain(comments_root.as_ref()),
            options.ids,
        ),
        monospace_paragraph: false,
        closing_fields: HashSet::new(),
        spanning_fields: HashSet::new(),
        open_fields: Vec::new(),
        revisions: Vec::new(),
        comments: HashMap::new(),
        notes: HashMap::new(),
        referenced: HashSet::new(),
        noted_at_end: HashSet::new(),
        noted_at_reference: HashSet::new(),
        open_comments: Vec::new(),
        hidden_ranges: Vec::new(),
        pending_notes: Vec::new(),
        in_comment: false,
        in_box: false,
        agent: options.ids,
        comments_inline: options.comments,
        resolved: accept.is_some(),
        dates: options.dates,
        handles: handles.clone(),
        pending_empty: Vec::new(),
        line_indent: 0,
        cached_pages,
        page_sections,
        break_due: false,
        announce: false,
        page: 0,
        para_comments: Vec::new(),
        threads: threads.clone(),
        story: None,
        part: main.clone(),
        scopes: drawing::ScopeMap::of(&document),
        pendings: Vec::new(),
    };
    if let Some(root) = &comments {
        writer.comments = root
            .children_named("comment")
            .filter_map(|c| c.attr("id").map(|id| (id.to_string(), c.clone())))
            .collect();
    }
    let mut references = Vec::new();
    document.find_all("commentReference", &mut references);
    for note in notes.values() {
        note.find_all("commentReference", &mut references);
    }
    writer.referenced = references
        .iter()
        .filter_map(|r| r.attr("id"))
        .map(str::to_string)
        .collect();
    let body = document.child("body").unwrap_or(&document);
    let mut blocks = Blocks::new();
    let mut list = ListIndent::default();
    writer.blocks(body, &mut blocks, &mut list);
    writer.flush_empty(&mut blocks);
    // Notes of ranges that ended after the last paragraph.
    let trailing = writer.take_notes();
    blocks.push(&trailing, false);

    // Footnotes and endnotes, numbered in reference order. A comment range the
    // body never closed does not run on into them.
    writer.open_comments.clear();
    writer.hidden_ranges.clear();
    let mut index = 0;
    let mut defs = Vec::new();
    // The id line of each definition, by label: `footnotes.p2`, or
    // `footnotes.p2-p3` for a note of two paragraphs.
    let mut note_ids: HashMap<String, String> = HashMap::new();
    // The style of each note whose first paragraph is not in Word's own
    // (`footnote text`, `endnote text`), by label: its id line names it.
    let mut note_styles: HashMap<String, Vec<String>> = HashMap::new();
    // The notes a definition line shows.
    let mut defined: HashSet<(bool, String)> = HashSet::new();
    while index < writer.note_refs.len() {
        let key = writer.note_refs[index].clone();
        // A note whose reference was inserted or deleted was inserted or
        // deleted with it, label and all.
        let changes = writer.note_changes[index].clone();
        index += 1;
        let Some(note) = notes.get(&key) else {
            continue;
        };
        // Word inserts or deletes a note with its reference, its text in a
        // mark of that kind. In the agent view a note whose text stays live
        // is not wrapped: its own marks keep their ids there, and a wrap
        // would claim text the reference's change never touched.
        let changes = if !writer.agent || text_follows_reference(note, &changes) {
            changes
        } else {
            Vec::new()
        };
        let around = std::mem::replace(&mut writer.revisions, changes);
        writer.lead = Some(format!("[^{index}]: "));
        writer.story = note_stories.get(&key.0).cloned();
        // Drawings a note prints name the notes part's relationships.
        let story_part = note_scopes.get(&key.0).map(|(part, scopes)| {
            (
                std::mem::replace(&mut writer.part, part.clone()),
                std::mem::replace(&mut writer.scopes, scopes.clone()),
            )
        });
        // Every paragraph the id line counts, a table's cells' too: the
        // view has no table in a note, so they are the note's paragraphs.
        let paragraphs = agent::paragraphs(note);
        // Each paragraph's style, Word's own as `own`; `None` when every
        // paragraph is in Word's own, a run of the last style left out.
        let note_style = {
            let own = if key.0 { "EndnoteText" } else { "FootnoteText" };
            let own_name = if key.0 {
                "endnote text"
            } else {
                "footnote text"
            };
            let styles: Vec<Option<String>> = paragraphs
                .iter()
                .map(|p| {
                    let id = p
                        .path(&["pPr", "pStyle"])
                        .and_then(|s| s.attr("val"))
                        .filter(|id| Some(*id) != default_style.as_deref());
                    match id {
                        None => Some(default_style.as_deref().unwrap_or("Normal").to_string()),
                        // Word's own, under any id (`a5` in a localized
                        // document), spelled as Word spells it.
                        Some(id)
                            if writer
                                .styles
                                .by_id
                                .get(id)
                                .map_or(id.eq_ignore_ascii_case(own), |s| {
                                    s.exact_name == own_name
                                }) =>
                        {
                            None
                        }
                        Some(id) => Some(id.to_string()),
                    }
                })
                .collect();
            if styles.iter().all(Option::is_none) {
                None
            } else {
                let mut list: Vec<String> = styles
                    .into_iter()
                    .map(|s| s.unwrap_or_else(|| own.to_string()))
                    .collect();
                while list.len() > 1 && list[list.len() - 1] == list[list.len() - 2] {
                    list.pop();
                }
                Some(list)
            }
        };
        let many = writer.agent && paragraphs.len() > 1;
        let parts: Vec<critic::Part> = paragraphs
            .into_iter()
            .enumerate()
            .map(|(at, p)| {
                let inline = writer.paragraph_inline(p).0.into_inline();
                // A line break is `<br/>` in the agent view, as in a cell:
                // `<br>` ends one of the note's paragraphs.
                let line_break = if writer.agent { "<br/>" } else { " " };
                let mut text = inline.render(true).replace('\n', line_break);
                // A first paragraph holding the reference mark alone still
                // opens the definition, so the label leads its `<br>`s.
                if at == 0
                    && many
                    && let Some(lead) = writer.lead.take()
                {
                    text = format!("{}{text}", lead.trim_end());
                }
                // In a note wrapped in its reference's change, an untracked
                // mark or one of that kind (Word's own, its tag on the
                // clause) joins the wrap: resolving the reference resolves
                // the whole note.
                let mark = writer.note_mark(p).or_else(|| {
                    writer
                        .revisions
                        .last()
                        .filter(|(kind, _)| paragraph_mark_kind(p).is_none_or(|own| own == *kind))
                        .cloned()
                });
                (text, mark, inline.edges())
            })
            .collect();
        writer.revisions = around;
        if let Some((part, scopes)) = story_part {
            writer.part = part;
            writer.scopes = scopes;
        }
        let story = writer.story.take();
        // Still set: no paragraph had text, so there is no note.
        if writer.lead.take().is_none() {
            // The agent view joins a note's paragraphs with `<br>`, which
            // its id line counts (`footnotes.p2-p3`), as a table's cells.
            let mut def = if writer.agent && parts.len() > 1 {
                critic::join_paragraphs(parts, "<br>", true)
            } else {
                critic::join_marked(parts, " ")
            };
            // The label's space stands for the one Word's text starts
            // with, also when a change opens before the label.
            let lead = format!("[^{index}]: ");
            if let Some(at) = def.find(&format!("{lead} ")) {
                def.remove(at + lead.len());
            }
            // The tracked view names the note's changes its text cannot
            // show: formatting, a deleted reference mark, an empty run.
            if writer.agent
                && !writer.resolved
                && let Some(story) = story
            {
                let (rev, fmt) =
                    agent::story_unshown(note, &story, &def, &HashSet::new(), &writer.handles);
                let mut clauses = Vec::new();
                for inserted in [true, false] {
                    let tags: Vec<&str> = rev
                        .iter()
                        .filter(|(ins, _)| *ins == inserted)
                        .map(|(_, tag)| tag.as_str())
                        .collect();
                    if !tags.is_empty() {
                        let word = if inserted { "rev" } else { "rev-del" };
                        clauses.push(format!("{word} {}", tags.join("; ")));
                    }
                }
                if !fmt.is_empty() {
                    let tags: Vec<&str> = fmt.iter().map(|(_, tag)| tag.as_str()).collect();
                    clauses.push(format!("fmt {}", tags.join("; ")));
                }
                if !clauses.is_empty() {
                    def.push_str(&format!(" <!-- {} -->", clauses.join(", ")));
                }
            }
            if let (Some(story), Some(&(first, count))) =
                (note_stories.get(&key.0), note_paragraphs.get(&key))
            {
                let id = match count {
                    0 | 1 => format!("{story}.p{first}"),
                    _ => format!("{story}.p{first}-p{}", first + count - 1),
                };
                note_ids.insert(index.to_string(), id);
                if let Some(style) = note_style {
                    note_styles.insert(index.to_string(), style);
                }
            }
            defs.push(def);
            defined.insert(key);
        }
    }
    let mut markdown = blocks.finish();
    if let Some(pages) = options.pages.as_ref().filter(|_| options.ids) {
        let pages: Vec<&str> = pages.iter().map(String::as_str).collect();
        markdown = crate::markdown::paginate(&markdown, &pages);
    }
    let mut range = None;
    let mut kept = None;
    let mut stories_changed = false;
    let mut selected_stories = Vec::new();
    if let Some(select) = options.select.as_ref().filter(|_| options.ids) {
        let select = match select {
            super::Select::Changed { by: Some(by) } => super::Select::Changed {
                by: Some(resolve_author(by, &handles)),
            },
            other => other.clone(),
        };
        if let super::Select::Changed { by } = &select {
            let revised = |root: &Element| agent::revised_by(root, by.as_deref(), &handles);
            stories_changed = story_roots.iter().any(revised);
            selected_stories = story_stems
                .iter()
                .zip(&story_roots)
                .filter(|(_, root)| revised(root))
                .map(|(stem, _)| stem.clone())
                .chain(
                    note_roots
                        .iter()
                        .filter(|(_, root)| revised(root))
                        .filter_map(|(name, _)| note_stories.get(&(*name == "endnote")).cloned()),
                )
                .collect();
            selected_stories.dedup();
        }
        // Hidden comments print ids only; `--by` reads their authors here.
        let comment_handles: HashMap<String, String> = writer
            .comments
            .iter()
            .filter_map(|(id, c)| Some((id.clone(), handles.of(c.attr("author"))?.to_string())))
            .collect();
        let (selected, described, paragraphs) = agent::select_blocks(
            &markdown,
            &mut defs,
            &select,
            stamped.0.checked_sub(1),
            &comment_handles,
        )
        .map_err(ooxml::invalid)?;
        markdown = selected;
        range = Some(described);
        kept = Some(paragraphs);
    }
    if let Some(original) = &original {
        let body = original.child("body").unwrap_or(original);
        let tags = agent::collect_revisions(body, &handles);
        let (rendered, hard) = agent::page_counts(body);
        let settings = package.xml("word/settings.xml").ok().flatten();
        let core = package.xml("docProps/core.xml").ok().flatten();
        let owner = match &core {
            None => header::Owner::None,
            // The first element of each name that names someone: an empty
            // `dc:creator` before a real one hides nobody. The name is kept
            // as written, its spaces too; an edge holding a line break is
            // the XML's indentation, not the name's.
            Some(core) => {
                let named = |local: &'static str| {
                    core.children_named(local)
                        .map(Element::text)
                        .find(|t| !t.trim().is_empty())
                        .map(|t| without_layout(&t).to_string())
                };
                match (named("creator"), named("lastModifiedBy")) {
                    (Some(name), _) => header::Owner::Creator(name),
                    (None, Some(name)) => header::Owner::LastModifiedBy(name),
                    (None, None) => header::Owner::Unnamed,
                }
            }
        };
        // Anchors before resolution: a reject view drops a deleted range.
        let mut anchored = HashSet::new();
        agent::comment_anchors(original, &mut anchored);
        for (_, root) in &note_roots {
            agent::comment_anchors(root, &mut anchored);
        }
        // Word shows a comment only from a reference in a main-story
        // paragraph (probe 2026-10-10): one anchored in a header or footer
        // is no anchored comment.
        let style_name = |id: &str| {
            styles_root.as_ref().and_then(|root| {
                root.children_named("style")
                    .find(|s| s.attr("styleId") == Some(id))
                    .and_then(|s| s.child("name"))
                    .and_then(|n| n.attr("val"))
            })
        };
        let comment_facts: Vec<header::CommentFact> = comments_root
            .as_ref()
            .map(|root| {
                root.children_named("comment")
                    .filter_map(|c| {
                        let id = c.attr("id")?.to_string();
                        let style = {
                            let mut ps = Vec::new();
                            c.find_all("p", &mut ps);
                            let id = ps
                                .first()
                                .and_then(|p| p.path(&["pPr", "pStyle"]))
                                .and_then(|s| s.attr("val"))
                                .filter(|id| Some(*id) != default_style.as_deref());
                            match id {
                                None => default_style.as_deref().unwrap_or("Normal").to_string(),
                                // Word's own, under any id (`a6` in a
                                // localized document): by name.
                                Some(id)
                                    if style_name(id).map_or(
                                        id.eq_ignore_ascii_case("CommentText"),
                                        |name| name.eq_ignore_ascii_case("annotation text"),
                                    ) =>
                                {
                                    "CommentText".to_string()
                                }
                                Some(id) => id.to_string(),
                            }
                        };
                        Some(header::CommentFact {
                            parent: threads.reply_of.get(&id).cloned(),
                            author: c.attr("author").map(str::to_string),
                            anchored: anchored.contains(id.as_str()),
                            id,
                            style,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        // Without Word's cached breaks (a document jubarte or another
        // writer made) the layout pass counts the pages; its breaks alone
        // would call a long page-break-free document one page.
        let laid_out = || {
            let track_changes = match options.revisions {
                Revisions::Markup => super::TrackChanges::All,
                Revisions::Accept => super::TrackChanges::Accept,
                Revisions::Reject => super::TrackChanges::Reject,
            };
            super::page_texts(
                bytes,
                track_changes,
                crate::convert::RevisionStyle::default(),
            )
            .ok()
            .map(|pages| pages.len().max(1))
        };
        let (pages, pages_source) = match (&options.pages, rendered) {
            (Some(pages), _) => (pages.len().max(1), header::PagesSource::Layout),
            (None, n) if n > 0 => (1 + n, header::PagesSource::Cached),
            // With page markers wanted, layout already failed: not again.
            (None, _) => match (!options.page_markers).then(laid_out).flatten() {
                Some(pages) => (pages, header::PagesSource::Layout),
                None => (1 + hard, header::PagesSource::Estimated),
            },
        };
        let theme = package.xml("word/theme/theme1.xml").ok().flatten();
        let mut heading_styles: Vec<(usize, String)> = Vec::new();
        let mut other_styles: Vec<String> = Vec::new();
        {
            let ps = agent::paragraphs(body);
            let mut uses: std::collections::BTreeMap<(usize, String), usize> =
                std::collections::BTreeMap::new();
            for p in &ps {
                let Some(style) = p.path(&["pPr", "pStyle"]).and_then(|s| s.attr("val")) else {
                    continue;
                };
                if let Some(level) = writer.styles.heading_level(style) {
                    *uses.entry((level, style.to_string())).or_default() += 1;
                }
            }
            for level in 1..=6 {
                if let Some(((_, style), _)) = uses
                    .iter()
                    .filter(|((l, _), _)| *l == level)
                    .max_by_key(|(_, n)| **n)
                {
                    heading_styles.push((level, style.clone()));
                }
            }
            // Every other paragraph style the body uses, in the order it
            // first does (the stories' follow): the legend describes each,
            // so the docx the view writes defines it as the original did.
            for p in &ps {
                let Some(style) = p.path(&["pPr", "pStyle"]).and_then(|s| s.attr("val")) else {
                    continue;
                };
                if style != default_style.as_deref().unwrap_or("Normal")
                    && !heading_styles.iter().any(|(_, h)| h == style)
                    && !other_styles.iter().any(|o| o == style)
                {
                    other_styles.push(style.to_string());
                }
            }
        }
        // The comments' and notes' paragraph styles too, as the stories'
        // below.
        // A note's list names Word's own style under its English id only
        // in a mixed note; the source's definition of it is kept like any
        // other (a footnote can be in `EndnoteText`).
        let mut note_list: Vec<&String> = note_styles.values().flatten().collect();
        note_list.sort();
        // CommentText is the comments' own, not a note's.
        for style in comment_facts
            .iter()
            .map(|c| c.style.as_str())
            .filter(|s| *s != "CommentText")
            .chain(note_list.into_iter().map(String::as_str))
        {
            if style != default_style.as_deref().unwrap_or("Normal")
                && styles_root.as_ref().is_some_and(|root| {
                    root.children_named("style").any(|s| {
                        s.attr("type") == Some("paragraph") && s.attr("styleId") == Some(style)
                    })
                })
                && !heading_styles.iter().any(|(_, h)| h == style)
                && !other_styles.iter().any(|o| o == style)
            {
                other_styles.push(style.to_string());
            }
        }
        let (table_styles, _) = table_styles(body, custom_table_grid(styles_root.as_ref()));
        let even_odd = settings
            .as_ref()
            .is_some_and(|s| s.child("evenAndOddHeaders").is_some());
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
                // `w:titlePg w:val="0"` states the default: off. Word writes
                // the switch out when a change turns the first page off
                // (ids-tables, #528).
                let title_page = sect
                    .child("titlePg")
                    .is_some_and(|t| !matches!(t.attr("val"), Some("0" | "false" | "off")));
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
                    let Some(target) = reference
                        .attr("id")
                        .and_then(|id| rels.get(id))
                        .map(|r| r.target.clone())
                    else {
                        continue;
                    };
                    let path = if target.starts_with("word/") {
                        target.clone()
                    } else {
                        format!("word/{}", target.trim_start_matches('/'))
                    };
                    let stem = part_stem(&target);
                    let Ok(Some(root)) = package.xml(&path) else {
                        continue;
                    };
                    let paragraphs = header::story_paragraphs(&root);
                    let story_scopes = drawing::ScopeMap::of(&root);
                    let read: Vec<(usize, String, header::ParaFact)> = paragraphs
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let align = p
                                .path(&["pPr", "jc"])
                                .and_then(|j| j.attr("val"))
                                .and_then(|v| match v {
                                    "center" => Some("center"),
                                    "right" | "end" => Some("right"),
                                    // Stated, left and justified differ
                                    // from none: a style's own alignment
                                    // applies only where the paragraph
                                    // names none.
                                    "left" | "start" => Some("left"),
                                    "both" | "distribute" => Some("justify"),
                                    _ => None,
                                });
                            let view = match accept {
                                None if options.ids => {
                                    header::StoryView::Tagged(&handles, stem.as_str())
                                }
                                Some(false) => header::StoryView::Reject(&writer.hidden_styles),
                                _ => header::StoryView::Accept(&writer.hidden_styles),
                            };
                            let own = header::ParaFact {
                                align,
                                color: story_color(p),
                                border: Some(agent::own_borders(
                                    p.path(&["pPr", "pBdr"]),
                                    &crate::markdown::borders::PARAGRAPH_SIDES,
                                ))
                                .filter(|sides| !sides.is_empty())
                                .map(|sides| {
                                    crate::markdown::borders::describe(
                                        &sides,
                                        &crate::markdown::borders::PARAGRAPH_SIDES,
                                    )
                                }),
                            };
                            (
                                i,
                                header::story_text_with(p, view, &mut |element, wrap| {
                                    Some(drawing::Pending::push(
                                        &mut writer.pendings,
                                        element,
                                        &path,
                                        story_scopes.scope(element),
                                        wrap,
                                    ))
                                }),
                                own,
                            )
                        })
                        .collect();
                    // From the first paragraph with text to the part's last:
                    // an empty paragraph after the text is a line Word lays
                    // out, so the header grows by it. A part with no text
                    // starts at its first paragraph.
                    let last = read.len().checked_sub(1);
                    let first = read
                        .iter()
                        .position(|(_, text, _)| !text.is_empty())
                        .or(last.map(|_| 0));
                    let (index, text, paras, more) = match (first, last) {
                        (Some(first), Some(last)) => (
                            read[first].0,
                            read[first].1.clone(),
                            read[first..=last]
                                .iter()
                                .map(|(_, _, own)| own.clone())
                                .collect::<Vec<_>>(),
                            read[first + 1..=last]
                                .iter()
                                .map(|(_, text, _)| text.clone())
                                .collect(),
                        ),
                        _ => (0, String::new(), Vec::new(), Vec::new()),
                    };
                    let align = paras.first().and_then(|p| p.align);
                    // The styles of the paragraphs the writer makes again,
                    // the empty ones before the text too: none when all
                    // are Word's for the story (`header`, by name).
                    // And past them, through the last one the view's `rev`
                    // brings back for its tracked mark.
                    let marked = (accept.is_none() && options.ids)
                        .then(|| {
                            paragraphs.iter().rposition(|p| {
                                p.path(&["pPr", "rPr"])
                                    .is_some_and(|r| r.child("ins").or(r.child("del")).is_some())
                            })
                        })
                        .flatten();
                    let written = match last.max(marked) {
                        Some(end) => &paragraphs[..=end],
                        None => &paragraphs[..paragraphs.len().min(1)],
                    };
                    let styles: Vec<String> = written
                        .iter()
                        .map(
                            |p| match p.path(&["pPr", "pStyle"]).and_then(|s| s.attr("val")) {
                                Some(id) if Some(id) != default_style.as_deref() => {
                                    let own = writer
                                        .styles
                                        .by_id
                                        .get(id)
                                        .map_or(id.eq_ignore_ascii_case(kind), |s| s.name == kind);
                                    if own {
                                        header::story_style(kind).to_string()
                                    } else {
                                        id.to_string()
                                    }
                                }
                                // No style is the default's, named by its id.
                                _ => default_style.as_deref().unwrap_or("Normal").to_string(),
                            },
                        )
                        .collect();
                    // The legend describes the styles the stories use, as
                    // the body's, so the view writes them as the original
                    // did; a character style's `, named` carries its
                    // w:name for `pstyle` to resolve.
                    for style in &styles {
                        if style != default_style.as_deref().unwrap_or("Normal")
                            && !matches!(style.as_str(), "Header" | "Footer")
                            && styles_root.as_ref().is_some_and(|root| {
                                root.children_named("style")
                                    .any(|s| s.attr("styleId") == Some(style.as_str()))
                            })
                            && !heading_styles.iter().any(|(_, h)| h == style)
                            && !other_styles.contains(style)
                        {
                            other_styles.push(style.clone());
                        }
                    }
                    let styles = if styles.iter().all(|s| s == header::story_style(kind)) {
                        Vec::new()
                    } else if styles.iter().all(|s| *s == styles[0]) {
                        styles[..1].to_vec()
                    } else {
                        styles
                    };
                    let (rev, fmt) = if accept.is_none() && options.ids {
                        let shown = std::iter::once(&text)
                            .chain(&more)
                            .cloned()
                            .collect::<Vec<_>>();
                        let (mut marks, ids) = agent::story_paragraph_marks(&root, &stem, &handles);
                        let (rev, fmt) =
                            agent::story_unshown(&root, &stem, &shown.join("\n"), &ids, &handles);
                        // A deletion names its kind (`del footer1.r3 @BC`):
                        // a bare tag reads back as an insertion.
                        marks.extend(rev.into_iter().map(|(inserted, tag)| {
                            if inserted { tag } else { format!("del {tag}") }
                        }));
                        // A formatting change names the paragraph it is in
                        // (`footer1.r1657 @AB in footer1.p1`), or reads
                        // positionally when outside every paragraph.
                        let fmt = fmt
                            .into_iter()
                            .map(|(at, tag)| match at {
                                Some(index) => format!(
                                    "{tag} in {}",
                                    crate::ids::short_paragraph(&format!("{stem}:p:{index}"))
                                ),
                                None => tag,
                            })
                            .collect();
                        (marks, fmt)
                    } else {
                        (Vec::new(), Vec::new())
                    };
                    // The id is the part stem (`header2` for `header2.xml`), Word's
                    // own numbering; a paragraph other than the first adds `.pN`.
                    // A stem with no short spelling prints long (`Header_1:p:0`).
                    stories.push(header::StoryFact {
                        kind,
                        ty,
                        id: crate::ids::short_paragraph(&format!("{stem}:p:{index}")),
                        part: crate::ids::short_paragraph(&format!("{stem}:p:0")),
                        first: index,
                        text,
                        more,
                        align,
                        paras,
                        styles,
                        active: match ty {
                            "first" => title_page,
                            "even" => even_odd,
                            _ => true,
                        },
                        rev,
                        fmt,
                    });
                }
                // A type a section does not name is inherited from the one before.
                if let Some(previous) = sections.last() {
                    for inherited in &previous.stories {
                        if !stories
                            .iter()
                            .any(|s| s.kind == inherited.kind && s.ty == inherited.ty)
                        {
                            let mut s = inherited.clone();
                            s.active = match s.ty {
                                "first" => title_page,
                                "even" => even_odd,
                                _ => true,
                            };
                            stories.push(s);
                        }
                    }
                }
                sections.push(header::SectionFact {
                    first,
                    last,
                    sect_pr: Some(sect),
                    stories,
                    title_page,
                });
                first = last + 1;
            }
            sections
        };
        // The changes of notes no definition line shows (Word's separators,
        // a note nothing references): the header's `notes:` names them.
        let mut note_changes = (Vec::new(), Vec::new());
        if accept.is_none() {
            for (name, root) in &note_roots {
                let endnote = *name == "endnote";
                let Some(story) = note_stories.get(&endnote) else {
                    continue;
                };
                for note in root.children_named(name) {
                    let id = note.attr("id").unwrap_or_default().to_string();
                    if defined.contains(&(endnote, id)) {
                        continue;
                    }
                    let (rev, fmt) =
                        agent::story_unshown(note, story, "", &HashSet::new(), &handles);
                    note_changes
                        .0
                        .extend(rev.into_iter().map(|(inserted, tag)| {
                            if inserted { tag } else { format!("del {tag}") }
                        }));
                    note_changes.1.extend(fmt.into_iter().map(|(_, tag)| tag));
                }
            }
        }
        // The notes parts that hold no note, Word's separators only: the
        // header's `notes:` names them, so the writer makes them. A part
        // with a note the view does not show (one no reference reaches) is
        // not empty.
        let empty_notes = note_roots
            .iter()
            .filter(|(name, root)| {
                root.children_named(name).all(|note| {
                    matches!(
                        note.attr("type"),
                        Some("separator" | "continuationSeparator" | "continuationNotice")
                    )
                })
            })
            .map(|(name, _)| {
                if *name == "endnote" {
                    "endnotes"
                } else {
                    "footnotes"
                }
            })
            .collect();
        let mut facts = header::Facts {
            unplaced: Vec::new(),
            note_changes,
            empty_notes,
            source: options.source.as_deref().unwrap_or("(bytes)"),
            sha256: crate::inspect::source_sha256(bytes),
            resolved: accept,
            comments_inline: options.comments,
            tracking_on: settings
                .as_ref()
                .is_some_and(|s| s.child("trackRevisions").is_some()),
            tags,
            changes: crate::changes::list_changes_tolerant(bytes).map_err(ooxml::invalid)?,
            comments: comment_facts,
            handles: &handles,
            done: &threads.done,
            owner,
            title: core
                .as_ref()
                .and_then(|core| core.child("title"))
                .map(|title| title.text().trim().to_string())
                .filter(|title| !title.is_empty()),
            paragraphs: stamped.0,
            tables: stamped.1,
            pages,
            pages_source,
            range,
            stories_changed,
            styles: styles_root.as_ref(),
            theme: theme.as_ref(),
            default_style: default_style.as_deref(),
            heading_styles,
            other_styles,
            table_styles,
            tables_differ: writer.tables_differ,
            sections,
        };
        let mut head = header::render(&facts);
        // The safety net: a listed change the view prints nowhere else (in
        // Word's Fallback copy with no Choice twin, in a text box with no
        // text to tag) is named on the header's `unplaced:` line.
        if accept.is_none() && facts.range.is_none() {
            let printed =
                agent::printed_change_ids(&head, &format!("{markdown}\n{}", defs.join("\n")));
            // Only a change's own printed id shows it. A container's tag
            // does not: accepting an inserted row keeps the row and leaves
            // a change inside it tracked, so that change needs its own id.
            let unplaced: Vec<String> = facts
                .changes
                .iter()
                .filter(|c| !printed.contains(&c.id))
                .map(|c| agent::unplaced_entry(c, &handles))
                .collect();
            facts.unplaced = unplaced;
            if !facts.unplaced.is_empty() {
                head = header::render(&facts);
            }
        }
        markdown = format!("{head}\n{markdown}");
    }
    if !defs.is_empty() {
        markdown.push_str("\n\n");
        // The agent view names the paragraphs `edit -p` takes for each note.
        let lines: Vec<String> = defs
            .iter()
            .map(|def| {
                match agent::note_label(def)
                    .and_then(|label| note_ids.get(label))
                    .filter(|_| options.ids)
                {
                    Some(id) => {
                        let style = agent::note_label(def)
                            .and_then(|label| note_styles.get(label))
                            .map(|styles| {
                                let tokens: Vec<String> = styles
                                    .iter()
                                    .map(|s| crate::markdown::view_notes::style_token(s))
                                    .collect();
                                format!(" {}", tokens.join(", "))
                            })
                            .unwrap_or_default();
                        format!("<!-- {id}{style} -->\n{def}")
                    }
                    None => def.clone(),
                }
            })
            .collect();
        markdown.push_str(&lines.join("\n"));
    }
    if !markdown.is_empty() {
        markdown.push('\n');
    }

    let types = drawing::content_types(&mut package);
    let markdown = drawing::fill(&markdown, &mut package, &types, &writer.pendings);

    Ok(Converted {
        markdown,
        selected: kept,
        selected_stories,
        media: writer.media.into_files(),
    })
}

/// `--by`'s author as `@HH` when it names a known author: a full name as
/// stored, or a handle with or without its `@`. Anything else stays as given
/// and matches no block.
fn resolve_author(by: &str, handles: &agent::Handles) -> String {
    let bare = by.strip_prefix('@').unwrap_or(by);
    // The handle the view prints wins over an author whose name is spelled
    // like it.
    handles
        .by_author
        .values()
        .find(|handle| *handle == bare)
        .or_else(|| handles.by_author.get(by))
        .map_or_else(|| bare.to_string(), |handle| format!("@{handle}"))
}

/// A `.docx`'s first non-empty `dc:creator` (its spaces kept, not the XML's
/// indentation) and its `dcterms:created` when that is a timestamp; `None`
/// for each the package does not tell.
pub(crate) fn creator_created(docx: &[u8]) -> (Option<String>, Option<String>) {
    let Some(core) = Package::open(docx, "DOCX")
        .ok()
        .and_then(|mut package| package.xml("docProps/core.xml").ok().flatten())
    else {
        return (None, None);
    };
    let creator = core
        .children_named("creator")
        .map(Element::text)
        .find(|t| !t.trim().is_empty())
        .map(|t| without_layout(&t).to_string());
    let created = core
        .children_named("created")
        .map(Element::text)
        .find_map(|t| crate::util::datetime::parse_datetime(t.trim()).ok());
    (creator, created)
}

/// `text` without an edge of whitespace that holds a line break: the
/// indentation of pretty-printed XML (`Un-named\n  `). Spaces alone are
/// kept (`  Bob  `).
fn without_layout(text: &str) -> &str {
    let breaks = |edge: &str| edge.contains(['\n', '\r']);
    let end = text.trim_end();
    let text = if breaks(&text[end.len()..]) {
        end
    } else {
        text
    };
    let start = text.trim_start();
    if breaks(&text[..text.len() - start.len()]) {
        start
    } else {
        text
    }
}

/// A code block's text in a fence of backticks, longer than any run of
/// backticks that opens one of its lines.
fn fenced(code: &str) -> String {
    let longest = code
        .lines()
        .map(|line| line.trim_start().chars().take_while(|&c| c == '`').count())
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest.max(2) + 1);
    format!("{fence}\n{code}\n{fence}")
}

/// A paragraph's line breaks (`w:br`) as Markdown hard breaks, which a
/// bare newline is not: it reads as a space. A break inside an equation
/// (`$…$`, `$$…$$`) stays a newline: it is part of the math text, and a
/// backslash before it would be a character of the equation.
fn hard_breaks(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        let Some(start) = unescaped_dollar(rest) else {
            out.push_str(&rest.replace('\n', "\\\n"));
            break;
        };
        out.push_str(&rest[..start].replace('\n', "\\\n"));
        let fence = if rest[start..].starts_with("$$") {
            "$$"
        } else {
            "$"
        };
        let body = start + fence.len();
        let Some(len) = math_body(&rest[body..], fence) else {
            out.push_str(&rest[start..].replace('\n', "\\\n"));
            break;
        };
        let end = body + len + fence.len();
        out.push_str(&rest[start..end]);
        rest = &rest[end..];
    }
    out
}

/// Escape a table cell for a Markdown pipe table. `escaped` text (the agent
/// view's) already has its backslashes escaped: its escapes are copied as
/// they are, and only a bare `|` gains one. In other text an equation
/// (`$…$`, `$$…$$`) keeps its LaTeX backslashes: Markdown does not unescape
/// math, so doubling them would turn `\frac` into a line break. Either way
/// every `|` in an equation gains a backslash, `\|` (‖) too: a pipe table
/// reads `\|` as `|` even there.
fn table_cell(value: &str, escaped: bool) -> String {
    if !escaped {
        return cell_math_kept(value);
    }
    let mut out = String::with_capacity(value.len() + 8);
    let mut rest = value;
    while let Some(start) = unescaped_dollar(rest) {
        let fence = if rest[start..].starts_with("$$") {
            "$$"
        } else {
            "$"
        };
        let body = start + fence.len();
        let Some(len) = rest[body..].find(fence) else {
            break;
        };
        out.push_str(&escaped_cell_text(&rest[..start]));
        let end = body + len + fence.len();
        out.push_str(&rest[start..end].replace('|', "\\|"));
        rest = &rest[end..];
    }
    out.push_str(&escaped_cell_text(rest));
    one_line(&out)
}

/// The byte offset of the first `$` no backslash escapes.
fn unescaped_dollar(value: &str) -> Option<usize> {
    let mut chars = value.char_indices();
    while let Some((at, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '$' => return Some(at),
            _ => {}
        }
    }
    None
}

/// The byte length of an equation's body after its opening fence, when
/// `fence` (`$` or `$$`) closes it. A backslash before `$` is not a fence.
fn math_body(after_open: &str, fence: &str) -> Option<usize> {
    let mut rest = after_open;
    let mut offset = 0usize;
    while let Some(at) = unescaped_dollar(rest) {
        if rest[at..].starts_with(fence) {
            return Some(offset + at);
        }
        let next = at + 1;
        offset += next;
        rest = &rest[next..];
    }
    None
}

/// Escaped cell text outside equations: escapes copied, bare `|` escaped.
fn escaped_cell_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                out.push(c);
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '|' => out.push_str("\\|"),
            _ => out.push(c),
        }
    }
    out
}

/// `table_cell` for unescaped text: backslashes doubled outside equations,
/// kept inside them; `|` escaped everywhere, as a pipe table needs.
fn cell_math_kept(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 8);
    let mut rest = value;
    while let Some(start) = rest.find('$') {
        let fence = if rest[start..].starts_with("$$") {
            "$$"
        } else {
            "$"
        };
        let body = start + fence.len();
        let Some(len) = rest[body..].find(fence) else {
            break;
        };
        out.push_str(&rest[..start].replace('\\', "\\\\").replace('|', "\\|"));
        let end = body + len + fence.len();
        out.push_str(&rest[start..end].replace('|', "\\|"));
        rest = &rest[end..];
    }
    out.push_str(&rest.replace('\\', "\\\\").replace('|', "\\|"));
    one_line(&out)
}

/// A cell's text on one line: Markdown table rows cannot break.
fn one_line(value: &str) -> String {
    value
        .replace("\r\n", " ")
        .replace(['\n', '\r'], " ")
        .trim()
        .to_string()
}

/// The alignment most of a column's cell paragraphs have, as its delimiter
/// row writes it: none on a tie, for justified text, or when most have
/// none. Left and none count together; the column is `left` only when
/// explicit left outnumbers none among them.
fn majority(aligns: impl Iterator<Item = Option<&'static str>>) -> Option<&'static str> {
    let mut tally: Vec<(Option<&'static str>, usize)> = Vec::new();
    let (mut left, mut none) = (0, 0);
    for align in aligns {
        match align {
            Some("left") => left += 1,
            None => none += 1,
            _ => {}
        }
        let key = align.filter(|a| *a != "left");
        match tally.iter_mut().find(|(k, _)| *k == key) {
            Some((_, count)) => *count += 1,
            None => tally.push((key, 1)),
        }
    }
    let most = tally.iter().map(|(_, count)| *count).max()?;
    let mut winners = tally.iter().filter(|(_, count)| *count == most);
    let (align, _) = winners.next()?;
    if winners.next().is_some() {
        return None;
    }
    match align {
        None => (left > none).then_some("left"),
        Some("justify") => None,
        other => *other,
    }
}

/// Rows as a Markdown pipe table; the first row is the header, and the
/// delimiter row aligns each column as `aligns` says.
/// The agent view's cell `<br>`s: the table line names each paragraph's
/// mark, so a `<br>` prints in a change only when its mark is tracked and a
/// paragraph beside it (the next, else its own) is wholly a change of that
/// kind, which the break then joins. An untracked mark's `<br>` stays out
/// of every change.
fn fold_tracked_breaks(parts: &mut [critic::Part]) {
    for i in 0..parts.len() {
        let fold = parts[i].1.take().and_then(|(kind, _)| {
            let same = |change: &Change| change.0 == kind;
            let next = parts
                .get(i + 1)
                .and_then(|next| critic::whole_change(&next.0))
                .filter(same);
            next.or_else(|| critic::whole_change(&parts[i].0).filter(same))
        });
        parts[i].1 = fold;
    }
}

/// The tracked change on `p`'s own mark: `w:pPr/w:rPr/w:ins` or `w:del`.
fn paragraph_mark_kind(p: &Element) -> Option<Mark> {
    p.path(&["pPr", "rPr"])?
        .elements()
        .find_map(|e| match e.local() {
            "ins" => Some(Mark::Insertion),
            "del" => Some(Mark::Deletion),
            _ => None,
        })
}

/// Whether `note` goes with its reference's change (`changes`, innermost
/// last): it holds no tracked change of its own (resolving the reference
/// resolves the note), or every run of it that holds text sits in a mark
/// of that kind, Word's shape for a note inserted or deleted with its
/// reference.
fn text_follows_reference(note: &Element, changes: &[Change]) -> bool {
    fn tracked(e: &Element) -> bool {
        e.elements().any(|child| {
            matches!(child.local(), "ins" | "del" | "moveTo" | "moveFrom") || tracked(child)
        })
    }
    fn walk(e: &Element, kind: &[&str], inside: bool, all: &mut bool) {
        for child in e.elements() {
            match child.local() {
                "r" if !inside => {
                    if child
                        .elements()
                        .any(|c| matches!(c.local(), "t" | "delText") && !c.text().is_empty())
                    {
                        *all = false;
                    }
                }
                local => walk(child, kind, inside || kind.contains(&local), all),
            }
        }
    }
    let kind: &[&str] = match changes.iter().rev().find_map(|(mark, _)| match mark {
        Mark::Insertion => Some(true),
        Mark::Deletion => Some(false),
        Mark::Scope => None,
    }) {
        Some(true) => &["ins", "moveTo"],
        Some(false) => &["del", "moveFrom"],
        None => return true,
    };
    if !tracked(note) {
        return true;
    }
    let mut all = true;
    walk(note, kind, false, &mut all);
    all
}

/// The paragraphs of `body` the view writes as blocks, in order, that are
/// headings: not those in tables, which a cell writes as text.
fn body_headings<'e>(body: &'e Element, styles: &Styles) -> Vec<&'e Element> {
    fn walk<'e>(e: &'e Element, styles: &Styles, out: &mut Vec<&'e Element>) {
        for child in e.elements() {
            match child.local() {
                "p" if heading_level(styles, child, true).is_some() => out.push(child),
                "body" | "sdt" | "sdtContent" | "customXml" | "txbxContent" | "ins" | "moveTo"
                | "del" | "moveFrom" => walk(child, styles, out),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(body, styles, &mut out);
    out
}

/// The name the writer gives heading `p`'s bookmark (github-slugger's, a
/// repeat suffixed by `slugger`); `None` for a heading of no slug.
fn own_slug(slugger: &mut super::write::Slugger, p: &Element) -> Option<String> {
    let slug = super::write::github_slug(&shown_text(p));
    let name = slugger.name(&slug);
    (!slug.is_empty()).then_some(name)
}

/// Whether bookmark `name` is a heading's `own` slug: that name, or one
/// the writer gives it (`_` before a slug of no first letter, cut to 40,
/// suffixed `-1` when an anchor took that name).
fn is_own_slug(name: &str, own: Option<&str>) -> bool {
    own.is_some_and(|own| super::write::is_bookmark_of(name, own))
}

/// A paragraph's heading level: its own outline level, else its style's;
/// `None` for body text. The plain view counts `#` up to six however
/// deep the level goes; the agent view (`agent`) stops at the writer's
/// heading styles instead, where the id line's `outline` clause carries
/// levels 7-9, for no `#` count reaches them.
fn heading_level(styles: &Styles, p: &Element, agent: bool) -> Option<usize> {
    let ppr = p.child("pPr");
    let from_style = || {
        ppr.and_then(|pr| pr.child("pStyle"))
            .and_then(|s| s.attr("val"))
            .and_then(|s| styles.heading_level(s))
    };
    match ppr
        .and_then(|pr| pr.child("outlineLvl"))
        .and_then(|o| o.attr("val"))
        .and_then(|v| v.parse::<usize>().ok())
    {
        // `w:val="9"` clears the level: body text, whatever a style says.
        Some(9..) => None,
        Some(l) if agent && l >= 6 => from_style(),
        Some(l) => Some((l + 1).min(6)),
        None => from_style(),
    }
}

fn markdown_table(rows: &[Vec<String>], aligns: &[Option<&str>], escaped: bool) -> String {
    // A table of empty cells keeps one column, so it stays a table.
    let width = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (index, row) in rows.iter().enumerate() {
        out.push('|');
        for column in 0..width {
            out.push_str(&table_cell(
                row.get(column).map(String::as_str).unwrap_or(""),
                escaped,
            ));
            out.push('|');
        }
        out.push('\n');
        if index == 0 {
            out.push('|');
            for column in 0..width {
                out.push_str(match aligns.get(column).copied().flatten() {
                    Some("left") => ":-|",
                    Some("center") => ":-:|",
                    Some("right") => "-:|",
                    _ => "-|",
                });
            }
            out.push('\n');
        }
    }
    out
}

#[derive(Default, Clone)]
struct Style {
    name: String,
    /// The name as the document writes it: Word's own are lower case
    /// (`footnote text`), and a style that spells one differently is the
    /// document's own (#528).
    exact_name: String,
    based_on: Option<String>,
    outline: Option<usize>,
    num: Option<(String, usize)>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    /// `w:vanish`: its runs are hidden.
    vanish: Option<bool>,
    /// `w:jc`, as Word writes it.
    jc: Option<String>,
    /// The `w:rFonts` face its runs are set in.
    font: Option<String>,
}

#[derive(Default)]
struct Styles {
    by_id: HashMap<String, Style>,
    /// The face the document's defaults set (`w:docDefaults`).
    default_font: Option<String>,
    /// The paragraph style a paragraph without `w:pStyle` has.
    default_paragraph: Option<String>,
}

impl Styles {
    fn parse(root: &Element) -> Self {
        let mut by_id = HashMap::new();
        for style in root.children_named("style") {
            let Some(id) = style.attr("styleId") else {
                continue;
            };
            let ppr = style.child("pPr");
            let rpr = style.child("rPr");
            by_id.insert(
                id.to_string(),
                Style {
                    name: style
                        .path(&["name"])
                        .and_then(|n| n.attr("val"))
                        .unwrap_or(id)
                        .to_ascii_lowercase(),
                    exact_name: style
                        .path(&["name"])
                        .and_then(|n| n.attr("val"))
                        .unwrap_or(id)
                        .to_string(),
                    based_on: style
                        .path(&["basedOn"])
                        .and_then(|n| n.attr("val"))
                        .map(str::to_string),
                    outline: ppr
                        .and_then(|p| p.child("outlineLvl"))
                        .and_then(|o| o.attr("val"))
                        .and_then(|v| v.parse().ok()),
                    num: ppr.and_then(|p| p.child("numPr")).and_then(num_pr),
                    bold: rpr.and_then(|r| r.toggle("b")),
                    italic: rpr.and_then(|r| r.toggle("i")),
                    underline: rpr.and_then(|r| r.child("u")).map(is_underlined),
                    vanish: rpr.and_then(|r| r.toggle("vanish")),
                    jc: ppr
                        .and_then(|p| p.child("jc"))
                        .and_then(|j| j.attr("val"))
                        .map(str::to_string),
                    font: rpr.and_then(font_of),
                },
            );
        }
        let default_font = root
            .path(&["docDefaults", "rPrDefault", "rPr"])
            .and_then(font_of);
        let default_paragraph = root
            .children_named("style")
            .find(|s| s.attr("type") == Some("paragraph") && s.attr("default") == Some("1"))
            .and_then(|s| s.attr("styleId"))
            .map(str::to_string);
        Self {
            by_id,
            default_font,
            default_paragraph,
        }
    }

    /// The face a run of style `id` is set in, nearest style first.
    fn font(&self, id: &str) -> Option<&str> {
        self.chain(id)
            .into_iter()
            .find_map(|style| style.font.as_deref())
    }

    /// Whether a paragraph of style `id` (or the default paragraph style)
    /// is set in a monospace face: its monospace runs are then not code.
    fn monospace_paragraph(&self, id: Option<&str>) -> bool {
        id.or(self.default_paragraph.as_deref())
            .and_then(|id| self.font(id))
            .or(self.default_font.as_deref())
            .is_some_and(is_monospace)
    }

    /// The alignment a paragraph of style `id` has, nearest first.
    fn jc(&self, id: &str) -> Option<&str> {
        self.chain(id)
            .into_iter()
            .find_map(|style| style.jc.as_deref())
    }

    /// The style and its `basedOn` ancestors, nearest first.
    fn chain(&self, id: &str) -> Vec<&Style> {
        let mut out = Vec::new();
        let mut current = Some(id.to_string());
        while let Some(id) = current {
            if out.len() >= 16 {
                break;
            }
            match self.by_id.get(&id) {
                Some(style) => {
                    out.push(style);
                    current = style.based_on.clone();
                }
                None => break,
            }
        }
        out
    }

    fn heading_level(&self, id: &str) -> Option<usize> {
        for style in self.chain(id) {
            if style.name == "title" {
                return Some(1);
            }
            if let Some(n) = style
                .name
                .strip_prefix("heading ")
                .and_then(|n| n.trim().parse::<usize>().ok())
            {
                return Some(n.clamp(1, 6));
            }
            if let Some(level) = style.outline {
                return (level < 9).then_some((level + 1).min(6));
            }
        }
        // Unknown style ids that still follow the built-in naming.
        let lower = id.to_ascii_lowercase();
        if lower == "title" {
            return Some(1);
        }
        lower
            .strip_prefix("heading")
            .and_then(|n| n.parse::<usize>().ok())
            .map(|n| n.clamp(1, 6))
    }

    fn num(&self, id: &str) -> Option<(String, usize)> {
        self.chain(id).into_iter().find_map(|s| s.num.clone())
    }

    fn emphasis(&self, id: &str) -> (Option<bool>, Option<bool>) {
        let chain = self.chain(id);
        (
            chain.iter().find_map(|s| s.bold),
            chain.iter().find_map(|s| s.italic),
        )
    }

    /// The character styles whose runs are hidden: the nearest `w:vanish`
    /// in the style chain is on.
    fn hidden(&self) -> HashSet<String> {
        self.by_id
            .keys()
            .filter(|id| self.chain(id).iter().find_map(|s| s.vanish) == Some(true))
            .cloned()
            .collect()
    }

    /// Whether the style chain underlines, nearest style first.
    fn underline(&self, id: &str) -> Option<bool> {
        self.chain(id).iter().find_map(|s| s.underline)
    }
}

/// The face `w:rFonts` names for Latin text in `rpr`.
fn font_of(rpr: &Element) -> Option<String> {
    let fonts = rpr.child("rFonts")?;
    fonts
        .attr("ascii")
        .or_else(|| fonts.attr("hAnsi"))
        .map(str::to_string)
}

/// `w:highlight w:val="…"` as one of Word's 16 named highlight colours
/// (`ST_HighlightColor`), the static `&str` the view prints. `None` when
/// `val` is empty or `none`; a colour Word does not name is the default
/// yellow, so the highlight round-trips even when the name does not.
fn highlight_color(c: &str) -> Option<&'static str> {
    let v = c.trim();
    if v.is_empty() || v == "none" {
        return None;
    }
    Some(
        crate::edit::HIGHLIGHTS
            .iter()
            .copied()
            .find(|h| *h != "none" && *h == v)
            .unwrap_or("yellow"),
    )
}

/// Whether `font` is a monospace face: the common code and typewriter faces,
/// and any whose name says so.
fn is_monospace(font: &str) -> bool {
    const FACES: [&str; 12] = [
        "consolas",
        "lucida console",
        "lucida sans typewriter",
        "menlo",
        "monaco",
        "inconsolata",
        "andale mono",
        "ocr a extended",
        "letter gothic",
        "prestige elite",
        "fixedsys",
        "terminal",
    ];
    let font = font.trim().to_ascii_lowercase();
    // Whole words only: "Monotype Corsiva" and "Barcode 39" are not code.
    font.starts_with("courier")
        || font.split([' ', '-', '_']).any(|word| {
            matches!(
                word,
                "mono" | "monospace" | "monospaced" | "code" | "typewriter"
            )
        })
        || FACES.contains(&font.as_str())
}

/// `w:u` underlines unless its value is `none`.
fn is_underlined(u: &Element) -> bool {
    u.attr("val").is_none_or(|v| v != "none")
}

fn num_pr(numpr: &Element) -> Option<(String, usize)> {
    let id = numpr
        .child("numId")
        .and_then(|n| n.attr("val"))?
        .to_string();
    let level = numpr
        .child("ilvl")
        .and_then(|n| n.attr("val"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    Some((id, level))
}

#[derive(Clone)]
struct Level {
    format: String,
    start: u32,
    /// `w:lvlText`: `%1)`, `%1.%2`.
    text: Option<String>,
}

#[derive(Default)]
struct Numbering {
    /// numId → (abstractNumId, per-level start overrides)
    nums: HashMap<String, (String, HashMap<usize, u32>)>,
    abstracts: HashMap<String, HashMap<usize, Level>>,
}

impl Numbering {
    fn parse(root: &Element) -> Self {
        let mut numbering = Self::default();
        for abs in root.children_named("abstractNum") {
            let Some(id) = abs.attr("abstractNumId") else {
                continue;
            };
            let mut levels = HashMap::new();
            for lvl in abs.children_named("lvl") {
                let Some(ilvl) = lvl.attr("ilvl").and_then(|v| v.parse().ok()) else {
                    continue;
                };
                levels.insert(ilvl, level_of(lvl));
            }
            numbering.abstracts.insert(id.to_string(), levels);
        }
        for num in root.children_named("num") {
            let (Some(id), Some(abs)) = (
                num.attr("numId"),
                num.child("abstractNumId").and_then(|a| a.attr("val")),
            ) else {
                continue;
            };
            let mut overrides = HashMap::new();
            for o in num.children_named("lvlOverride") {
                let Some(ilvl) = o.attr("ilvl").and_then(|v| v.parse().ok()) else {
                    continue;
                };
                let start = o
                    .child("startOverride")
                    .and_then(|s| s.attr("val"))
                    .and_then(|v| v.parse().ok())
                    .or_else(|| o.child("lvl").map(|l| level_of(l).start));
                if let Some(start) = start {
                    overrides.insert(ilvl, start);
                }
            }
            numbering
                .nums
                .insert(id.to_string(), (abs.to_string(), overrides));
        }
        numbering
    }

    fn level(&self, num_id: &str, ilvl: usize) -> Option<(&str, Level)> {
        let (abs, _) = self.nums.get(num_id)?;
        let level = self
            .abstracts
            .get(abs)
            .and_then(|l| l.get(&ilvl))
            .cloned()
            .unwrap_or(Level {
                format: "bullet".into(),
                start: 1,
                text: None,
            });
        Some((abs.as_str(), level))
    }
}

fn level_of(lvl: &Element) -> Level {
    Level {
        format: lvl
            .child("numFmt")
            .and_then(|f| f.attr("val"))
            .unwrap_or("decimal")
            .to_string(),
        start: lvl
            .child("start")
            .and_then(|s| s.attr("val"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        text: lvl
            .child("lvlText")
            .and_then(|t| t.attr("val"))
            .map(str::to_string),
    }
}

/// What closes a field the agent view prints as `{CODE}result{/}`.
const FIELD_END: &str = "{/}";

/// An open complex field (`fldChar begin … separate … end`).
struct Field {
    instruction: String,
    in_result: bool,
    link: Option<String>,
    /// Its `end` lies in the same paragraph.
    closes: bool,
    /// Its `end` lies in a later paragraph (see [`spanning_fields`]).
    spans: bool,
    /// A field sits inside its code, which `{CODE}` cannot show.
    nested: bool,
    /// Its `{CODE}` was written, so its end writes `{/}`.
    printed: bool,
}

/// The `fldChar begin` of each field in `p` whose `separate` and `end` are
/// in `p` too, inside the same tracked changes as its begin: the writer
/// reads a field within one change, never one a change boundary cuts.
/// Text boxes are left out (their paragraphs are written on their own).
fn closing_fields(p: &Element) -> HashSet<*const Element> {
    /// A field open in the walk: its begin, the changes around the begin,
    /// and whether its separate (if any yet) sat inside the same ones.
    type Open<'e> = (&'e Element, Vec<*const Element>, bool);
    fn walk<'e>(
        e: &'e Element,
        changes: &mut Vec<*const Element>,
        open: &mut Vec<Open<'e>>,
        out: &mut HashSet<*const Element>,
    ) {
        for child in e.elements() {
            if child.is("txbxContent") || child.is("pPr") {
                continue;
            }
            if child.is("fldChar") {
                match child.attr("fldCharType") {
                    Some("begin") => open.push((child, changes.clone(), true)),
                    Some("separate") => {
                        if let Some((_, around, same)) = open.last_mut() {
                            *same &= around == changes;
                        }
                    }
                    Some("end") => {
                        if let Some((begin, around, same)) = open.pop()
                            && same
                            && around == *changes
                        {
                            out.insert(std::ptr::from_ref(begin));
                        }
                    }
                    _ => {}
                }
            } else if matches!(child.local(), "ins" | "del" | "moveTo" | "moveFrom") {
                changes.push(std::ptr::from_ref(child));
                walk(child, changes, open, out);
                changes.pop();
            } else {
                walk(child, changes, open, out);
            }
        }
    }
    let mut out = HashSet::new();
    walk(p, &mut Vec::new(), &mut Vec::new(), &mut out);
    out
}

/// The `fldChar begin` of each field in `container`'s paragraphs whose
/// `end` lies in a later paragraph, with only paragraphs between (a table
/// of contents), its begin, separate and end outside any tracked change:
/// such a field prints `{CODE}` at the start of its first paragraph and
/// `{/}` in its last. Text boxes are left out, as in [`closing_fields`].
fn spanning_fields(container: &Element) -> HashSet<*const Element> {
    /// A field open in the walk: its begin, the paragraph it began in, and
    /// whether no tracked change has cut it yet.
    type Open<'e> = (&'e Element, usize, bool);
    fn walk<'e>(
        e: &'e Element,
        paragraph: usize,
        changed: bool,
        open: &mut Vec<Open<'e>>,
        out: &mut HashSet<*const Element>,
    ) {
        for child in e.elements() {
            if child.is("txbxContent") || child.is("pPr") {
                continue;
            }
            if child.is("fldChar") {
                match child.attr("fldCharType") {
                    Some("begin") => open.push((child, paragraph, !changed)),
                    Some("separate") => {
                        if let Some((_, _, clean)) = open.last_mut() {
                            *clean &= !changed;
                        }
                    }
                    Some("end") => {
                        if let Some((begin, began, clean)) = open.pop()
                            && clean
                            && !changed
                            && began < paragraph
                        {
                            out.insert(std::ptr::from_ref(begin));
                        }
                    }
                    _ => {}
                }
            } else {
                let changes = matches!(child.local(), "ins" | "del" | "moveTo" | "moveFrom");
                walk(child, paragraph, changed || changes, open, out);
            }
        }
    }
    let mut out = HashSet::new();
    let mut open = Vec::new();
    for (i, child) in container.elements().enumerate() {
        if child.is("p") {
            walk(child, i, false, &mut open, &mut out);
        } else {
            open.clear();
        }
    }
    out
}

/// The `{CODE}` the agent view prints for a field with `instruction`, when
/// it is a field code (see [`crate::markdown::critic::is_field_code`]).
fn printed_code(instruction: &str) -> Option<String> {
    crate::markdown::critic::is_field_code(instruction)
        .then(|| format!("{{{}}}", instruction.trim()))
}

/// A revision element (`w:ins`, `w:del`, `w:moveTo`, `w:moveFrom`) as a tracked
/// change. Insertions and move destinations are insertions; deletions and move
/// sources are deletions (the representation pandiff uses too).
fn change_of(element: &Element, story: Option<&str>) -> Change {
    let mark = if matches!(element.local(), "ins" | "moveTo") {
        Mark::Insertion
    } else {
        Mark::Deletion
    };
    (mark, attribution(element, story))
}

/// Which change it is, and who made it when: a plain-view tag
/// ([`critic::PLAIN`]) of the change's `w:id` in `story` and its note,
/// `w:author` and `w:date` exactly as stored, in the same form as a
/// comment's. It prints `r5 Ana Lima (2026-09-29T14:05:00Z)`: the number
/// tells the writer the note names the change's author, not a comment.
fn attribution(element: &Element, story: Option<&str>) -> Option<String> {
    // A leading tag sentinel in a stored author would be read as a tag.
    let author = element
        .attr("author")
        .map(|a| a.trim_start_matches(['\u{E000}', '\u{E001}']));
    let note = comment_note(author, element.attr("date"), "");
    // A mark with no id, author or date (only a malformed file has one)
    // names nothing.
    if note.is_empty() && element.attr("id").is_none() {
        return None;
    }
    let story = story.map_or(String::new(), |s| format!("{s}#"));
    Some(format!(
        "{}{story}{}",
        critic::PLAIN,
        agent::plain_tag_of(element, &note)
    ))
}

/// Tracked changes recorded in a properties element (`trPr`, `tcPr`, a
/// paragraph mark's `rPr`), in document order.
fn revision_marks(properties: &Element, story: Option<&str>) -> Vec<Change> {
    properties
        .elements()
        .filter_map(|p| match p.local() {
            "ins" | "moveTo" | "cellIns" => Some((Mark::Insertion, attribution(p, story))),
            "del" | "moveFrom" | "cellDel" => Some((Mark::Deletion, attribution(p, story))),
            _ => None,
        })
        .collect()
}

/// The tracked change on the mark that ends a paragraph, that is on the break
/// between it and the next one. A mark inserted and then deleted counts as
/// deleted, the later of the two changes.
fn paragraph_mark(p: &Element, story: Option<&str>) -> Option<Change> {
    let marks = p
        .path(&["pPr", "rPr"])
        .map(|rpr| revision_marks(rpr, story))
        .unwrap_or_default();
    let deleted = marks.iter().position(|(m, _)| *m == Mark::Deletion);
    marks.into_iter().nth(deleted.unwrap_or(0))
}

/// A comment's `{>>…<<}` text: `Author (date): comment`. The date is Word's
/// `w:date` exactly as stored (for example `2024-04-08T10:32:00Z`, which Word
/// writes as the author's local time with a `Z`), so it is never converted.
fn comment_note(author: Option<&str>, date: Option<&str>, text: &str) -> String {
    let mut head = Vec::new();
    if let Some(author) = author.map(str::trim).filter(|a| !a.is_empty()) {
        head.push(critic::escape(author));
    }
    if let Some(date) = date.map(str::trim).filter(|d| !d.is_empty()) {
        head.push(format!("({})", critic::escape(date)));
    }
    let head = head.join(" ");
    match (head.is_empty(), text.is_empty()) {
        (true, _) => text.to_string(),
        (false, true) => head,
        (false, false) => format!("{head}: {text}"),
    }
}

struct Writer<'a> {
    rels: &'a Rels,
    media: Media,
    styles: Styles,
    numbering: Numbering,
    /// abstractNumId → running counter per level (None = not started).
    counters: HashMap<String, [Option<i64>; 9]>,
    started_nums: Vec<String>,
    /// (is_endnote, id) in first-reference order.
    note_refs: Vec<(bool, String)>,
    /// The tracked changes around each note's first reference.
    note_changes: Vec<Vec<Change>>,
    /// Markdown for the start of the next paragraph with text, inside its
    /// tracked changes: a note's `[^N]: ` label.
    lead: Option<String>,
    /// Inside a nested table, whose cells are flattened onto one line.
    flattening: usize,
    /// The bookmarks a link or a reference field names, and Word's
    /// cross-reference targets (`_Ref…`), which print as `<a id="name"></a>`
    /// where they start.
    bookmarks: HashSet<String>,
    /// The tables differ in style (two styles, or one and none): each
    /// table line names its own, `style Ledger`.
    tables_differ: bool,
    /// The source defines Table Grid with borders of its own
    /// ([`custom_table_grid`]), so the view names it.
    custom_grid: bool,
    /// In a heading: the name a Markdown link names it by, whose bookmark
    /// the writer makes again and so prints no `<a id>`.
    heading_slug: Option<String>,
    /// Character styles whose runs are hidden (`w:vanish`).
    hidden_styles: HashSet<String>,
    /// The names the writer gives the headings written so far.
    slugger: super::write::Slugger,
    /// The paragraph being written is set in a monospace face.
    monospace_paragraph: bool,
    /// The `fldChar begin` of each field whose `end` lies in the paragraph
    /// being written: only those print as `{CODE}result{/}`.
    closing_fields: HashSet<*const Element>,
    /// The `fldChar begin` of each field whose `end` lies in a later
    /// paragraph of its container (see [`spanning_fields`]).
    spanning_fields: HashSet<*const Element>,
    /// The fields a paragraph left open for a later one to close.
    open_fields: Vec<Field>,
    /// Tracked changes around the content being written, outermost first.
    revisions: Vec<Change>,
    /// Comment id → its `w:comment` element.
    comments: HashMap<String, Element>,
    /// Comment id → the inside of its `{>>…<<}` note, in each of the forms
    /// its contexts print, rendered at first use so that anything numbered
    /// inside it (a footnote) is numbered where the comment is.
    notes: HashMap<String, critic::Note>,
    /// Comments with a `commentReference`, where their note goes. Others get
    /// their note where their range ends.
    referenced: HashSet<String>,
    /// Comments whose note followed their range's end, inside a paragraph
    /// and outside any change: there Word puts the reference, and the
    /// writer takes the note back as the range's own comment. Their
    /// reference, wherever it sits, prints nothing more.
    noted_at_end: HashSet<String>,
    /// Comments whose note a reference printed.
    noted_at_reference: HashSet<String>,
    /// Comment ranges open at this point, in the order they started.
    open_comments: Vec<String>,
    /// Agent view, comments hidden: the ranges open at this point. They
    /// highlight nothing; the id lines name them (`in c5`).
    hidden_ranges: Vec<String>,
    /// Notes of unreferenced comments whose range ended between paragraphs;
    /// they open the next paragraph.
    pending_notes: Vec<critic::Note>,
    /// Writing a comment's own text, where a `{>>…<<}` cannot appear: tracked
    /// changes keep their marks but not their author, and comment anchors are
    /// ignored.
    in_comment: bool,
    /// Writing a drawn text box's content for its comment lines: a drawing
    /// inside it prints no payload token (the captured drawing already holds
    /// it) and a box nested inside it is walked through, not printed twice.
    in_box: bool,
    /// Agent view (`Options::ids`).
    agent: bool,
    /// Agent view: comments inline, or hidden and listed on id lines.
    comments_inline: bool,
    /// Agent view: accept-all or reject-all (revisions already resolved).
    resolved: bool,
    /// Agent view: inline timestamps on notes (`Options::dates`).
    dates: bool,
    handles: agent::Handles,
    /// Agent view: empty paragraphs not yet written as `<!-- pN empty -->`.
    pending_empty: Vec<usize>,
    /// Agent view: the indentation of the comment lines written before a
    /// nested list item, its own, so they stay inside the list.
    line_indent: usize,
    /// Agent view, cached-break fallback: `Some(total)` makes the writer
    /// emit `<!-- page N of total -->` itself; `None` leaves markers to
    /// `paginate`.
    cached_pages: Option<usize>,
    /// Agent view, cached-break fallback with no `w:lastRenderedPageBreak`
    /// in the document: the paragraphs whose section break starts a page.
    /// `Some` makes hard page and section breaks turn the pages.
    page_sections: Option<std::collections::HashSet<usize>>,
    /// A hard break ended the last block: the next one opens a page.
    break_due: bool,
    /// Pages turned inside the last block, still to be named.
    announce: bool,
    /// Agent view: the page the writer is on (0 before the first marker).
    page: usize,
    /// Agent view: comment ids met since the last id line.
    para_comments: Vec<String>,
    /// Agent view: comment threads (`commentsExtended.xml`).
    threads: agent::Threads,
    /// Agent view: the story being written (`footnotes`, `header1`), whose
    /// stem prefixes its change tags; `None` in the body.
    story: Option<String>,
    /// The part being written (`word/document.xml`, `word/footnotes.xml`):
    /// whose relationships a captured drawing names.
    part: String,
    /// `xmlns` scopes of the part being written, by drawing element.
    scopes: drawing::ScopeMap,
    /// Drawings queued for `drawing:` payloads, in print order.
    pendings: Vec<drawing::Pending>,
}

impl Writer<'_> {
    /// The notes waiting for the next paragraph, as CriticMarkup comments:
    /// their block is prose, whatever their anchor was.
    fn take_notes(&mut self) -> String {
        std::mem::take(&mut self.pending_notes)
            .iter()
            .map(|note| format!("{{>>{}<<}}", note.prose))
            .collect()
    }

    fn blocks(&mut self, container: &Element, blocks: &mut Blocks, list: &mut ListIndent) {
        // The set is the container's own while it is written: pointers
        // into a part already dropped could name another part's element.
        let outer = std::mem::replace(&mut self.spanning_fields, spanning_fields(container));
        self.blocks_in(container, blocks, list);
        self.spanning_fields = outer;
        self.open_fields.clear();
    }

    fn blocks_in(&mut self, container: &Element, blocks: &mut Blocks, list: &mut ListIndent) {
        for child in container.elements() {
            match child.local() {
                "p" => self.paragraph(child, blocks, list),
                "tbl" => {
                    list.reset();
                    // A note waiting for the next paragraph would open the
                    // first cell; it gets its own block before the table.
                    let notes = self.take_notes();
                    blocks.push_prefixed("", &notes, false);
                    let mut turned = 0;
                    let spans = self.threads_of(&self.open_after(child));
                    if self.agent
                        && let Some(line) = agent::table_line(
                            child,
                            self.tables_differ
                                .then(|| view_table_style(child, self.custom_grid))
                                .flatten(),
                            &[self.column_alignments(child).1, self.cell_styles(child)].concat(),
                            self.resolved,
                            (!self.comments_inline).then_some(&self.comments),
                            &spans,
                            &self.handles,
                        )
                    {
                        self.flush_empty(blocks);
                        let (first, more) =
                            agent::table_breaks(child, self.page_sections.is_none());
                        let opens = self.opens_page(first);
                        self.page_lines(blocks, opens);
                        blocks.push_line(&line);
                        turned = more;
                    }
                    let table = self.table(child);
                    blocks.push(&table, false);
                    self.turn_pages(turned);
                    // The cells' hidden comments went on the table line.
                    self.para_comments.clear();
                }
                "sdt" => {
                    if let Some(content) = child.child("sdtContent") {
                        self.blocks(content, blocks, list);
                    }
                }
                "ins" | "moveTo" | "del" | "moveFrom" => {
                    self.revised(self.change_of(child), |w| w.blocks(child, blocks, list));
                }
                "commentRangeStart" | "commentRangeEnd" => self.comment_range(child, None),
                "customXml" | "sdtContent" | "txbxContent" => self.blocks(child, blocks, list),
                _ => {}
            }
        }
    }

    /// Writes content inside a tracked change.
    fn revised(&mut self, change: Change, write: impl FnOnce(&mut Self)) {
        self.revisions.push(change);
        write(self);
        self.revisions.pop();
    }

    /// A comment range start or end. With comments shown, the commented text
    /// is highlighted; a range that spans paragraphs gets one highlight per
    /// paragraph, because CriticMarkup cannot cross a block. With them hidden
    /// (the agent view's `--comments none`), nothing is highlighted: the range
    /// is only tracked, so each line it covers can name it.
    /// `name` is the bookmark the writer gives the heading being read: its
    /// slug, or the slug and `-1`, `-2`, … for a repeated heading.
    fn slug_of_heading(&self, name: &str) -> bool {
        is_own_slug(name, self.heading_slug.as_deref())
    }

    /// A paragraph's heading level: its own outline level, else its
    /// style's; `None` for body text.
    fn heading_of(&self, p: &Element) -> Option<usize> {
        heading_level(&self.styles, p, self.agent)
    }

    fn comment_range(&mut self, range: &Element, out: Option<&mut Critic>) {
        if self.agent
            && !self.in_comment
            && let Some(id) = range
                .attr("id")
                .filter(|id| self.comments.contains_key(*id))
        {
            if range.is("commentRangeStart") && !self.para_comments.iter().any(|c| c == id) {
                self.para_comments.push(id.to_string());
            }
            if !self.comments_inline {
                // Hidden, the range still runs: every paragraph and table
                // it covers names it on its id line.
                let open = self.hidden_ranges.iter().position(|open| open == id);
                match (range.is("commentRangeStart"), open) {
                    (true, None) => self.hidden_ranges.push(id.to_string()),
                    (false, Some(at)) => {
                        self.hidden_ranges.remove(at);
                    }
                    _ => {}
                }
                return;
            }
        }
        let Some(id) = range
            .attr("id")
            .filter(|id| !self.in_comment && self.comments.contains_key(*id))
        else {
            return;
        };
        let open = self.open_comments.iter().position(|open| open == id);
        match (range.is("commentRangeStart"), open) {
            (true, None) => {
                self.open_comments.push(id.to_string());
                if let Some(out) = out {
                    out.open(Mark::Scope, None);
                }
            }
            (false, Some(at)) => {
                self.open_comments.remove(at);
                match out {
                    Some(out) => {
                        out.close(Mark::Scope);
                        // Inside a change the note waits for the reference:
                        // `{++…{>>note<<}…++}` is no range's comment.
                        let unprinted = if self.revisions.is_empty() {
                            !self.noted_at_reference.contains(id)
                        } else {
                            !self.referenced.contains(id)
                        };
                        if unprinted && let Some(note) = self.note(id) {
                            out.comment(&note);
                            self.noted_at_end.insert(id.to_string());
                        }
                    }
                    None => {
                        let note = (!self.referenced.contains(id))
                            .then(|| self.note(id))
                            .flatten();
                        self.pending_notes.extend(note);
                    }
                }
            }
            _ => {}
        }
    }

    fn paragraph(&mut self, p: &Element, blocks: &mut Blocks, list: &mut ListIndent) {
        let ppr = p.child("pPr");
        let style = ppr
            .and_then(|pr| pr.child("pStyle"))
            .and_then(|s| s.attr("val"))
            .map(str::to_string);
        let heading = self.heading_of(p);
        let num = ppr
            .and_then(|pr| pr.child("numPr"))
            .and_then(num_pr)
            .or_else(|| style.as_deref().and_then(|s| self.styles.num(s)));

        let open_before = self.open_ranges().to_vec();
        self.heading_slug = heading.and_then(|_| own_slug(&mut self.slugger, p));
        let (mut inline, extra) = self.paragraph_inline(p);
        self.heading_slug = None;
        let code_block = style.as_deref() == Some("SourceCode");
        if code_block {
            inline.set_code_block();
        }
        let written = !inline.is_blank();
        let inline = inline.into_inline();
        // Comment threads whose range runs past this paragraph, for its id
        // line: their highlight repeats here with the note elsewhere.
        let mut spans = self.threads_of(&open_before);
        for id in self.open_threads() {
            if !spans.contains(&id) {
                spans.push(id);
            }
        }
        // Computed once. The agent view counts empty paragraphs too, as Word
        // does (it shows an empty numbered paragraph's label and spends its
        // number). The plain conversion counts written paragraphs only: its
        // labels are text, so a tracked change that merges an empty numbered
        // paragraph away would leave the resolved markup a number off
        // (Word's own markup shows both, `4.3.`).
        // (Markdown marker, Word's label, level)
        let list_marker: Option<(String, String, usize)> =
            if (written || self.agent) && heading.is_none() {
                num.clone().and_then(|(id, ilvl)| {
                    self.list_marker(&id, ilvl.min(8))
                        .map(|(m, label)| (m, label, ilvl))
                })
            } else {
                None
            };
        // A heading's number is text, so it reads as Word prints it.
        let heading_marker: Option<(String, usize)> = if self.agent && heading.is_some() {
            num.clone().and_then(|(id, ilvl)| {
                self.list_marker(&id, ilvl.min(8))
                    .map(|(_, label)| (label, ilvl))
            })
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
            let comments = if self.comments_inline {
                Vec::new()
            } else {
                std::mem::take(&mut self.para_comments)
            };
            self.para_comments.clear();
            // A comment the line lists (`comments c9`) is not named again
            // as a range it runs in.
            spans.retain(|id| !comments.contains(id));
            let empty = !written && extra.is_empty() && !page_break && !agent::has_section_break(p);
            let numbered = heading_marker.is_some() || list_marker.is_some();
            // A paragraph whose text is only blanks writes none of it, so
            // the tags its notes would carry print nowhere either.
            let unshown = if self.resolved {
                Vec::new()
            } else if written {
                agent::unshown_tags(p, &inline.render(true), &self.handles)
            } else {
                agent::unshown_tags(p, "", &self.handles)
            };
            // The level the paragraph states itself (`w:outlineLvl`),
            // which without a `w:pStyle` is the only place it can come
            // from: `outline 2`, and past the heading styles (`outline 7`
            // to `outline 9`) the only place at all, for no `#` count
            // reaches them.
            let own_outline = style
                .is_none()
                .then(|| {
                    ppr.and_then(|pr| pr.child("outlineLvl"))
                        .and_then(|o| o.attr("val"))
                        .and_then(|v| v.parse::<usize>().ok())
                        .filter(|l| *l < 9)
                        .map(|l| l + 1)
                })
                .flatten();
            if empty
                && !numbered
                && style.is_none()
                && own_outline.is_none()
                && comments.is_empty()
                && unshown.is_empty()
                && !agent::holds_revision_facts(p, self.resolved, &self.handles)
            {
                if self.page_sections.is_none() && agent::has_rendered_page_break(p) {
                    self.flush_empty(blocks);
                    self.page_lines(blocks, true);
                    self.turn_pages(agent::rendered_page_breaks(p) - 1);
                }
                self.pending_empty.push(index);
                return;
            }
            // A nested item's lines go where the item goes, and the level
            // its indentation says is not repeated on its id line.
            let place = list_marker.as_ref().map(|(_, _, ilvl)| list.place(*ilvl));
            self.line_indent = place.map_or(0, |(indent, _)| indent);
            self.flush_empty(blocks);
            let opens = self.opens_page(agent::has_rendered_page_break(p));
            self.page_lines(blocks, opens);
            // The pages a paragraph turns past the one it opens or ends.
            self.turn_pages(if self.page_sections.is_none() {
                agent::rendered_page_breaks(p).saturating_sub(1)
            } else {
                agent::page_breaks(p).saturating_sub(1)
            });
            if self
                .page_sections
                .as_ref()
                .is_some_and(|sections| page_break || sections.contains(&index))
            {
                self.break_due = true;
            }
            let (marker, level) = heading_marker
                .as_ref()
                .map(|(label, ilvl)| (label.as_str(), *ilvl))
                .or(list_marker
                    .as_ref()
                    .map(|(_, label, ilvl)| (label.as_str(), *ilvl)))
                .map_or((None, 0), |(label, ilvl)| (Some(label), ilvl.min(8)));
            let level = match place {
                Some((_, said)) if said == level => 0,
                _ => level,
            };
            let facts = agent::LineFacts {
                index,
                style: style.as_deref(),
                heading,
                // The level the paragraph states itself (`w:outlineLvl`),
                // which without a `w:pStyle` is the only place a `#` can
                // come from: `outline 2`.
                outline: own_outline,
                marker,
                level,
                page_break,
                resolved: self.resolved,
                comments: &comments,
                spans: &spans,
                empty,
                unshown: &unshown,
            };
            blocks.push_line(&format!(
                "{}{}",
                " ".repeat(self.line_indent),
                agent::id_line(p, &facts, &self.handles)
            ));
            self.line_indent = 0;
        }
        if written {
            let edges = inline.edges();
            if let Some(level) = heading {
                list.reset();
                let mut text = inline.render(false).replace('\n', " ");
                if let Some((label, _)) = &heading_marker {
                    text = format!("{label} {text}");
                }
                text = agent::escape_closing_hashes(&text);
                blocks.push_paragraph(&format!("{} ", "#".repeat(level)), &text, false, edges);
            } else if let Some(marker) = list_marker {
                let (marker, _, ilvl) = marker;
                let text = self.block_text(&inline.render(true));
                let item = list.item(ilvl, &marker, &hard_breaks(&text));
                let prefix = item.len() - item.trim_start().len() + marker.len() + 1;
                blocks.push_paragraph(&item[..prefix], &item[prefix..], true, edges);
            } else if code_block {
                list.reset();
                blocks.push_paragraph("", &fenced(&inline.render_code()), false, edges);
            } else {
                list.reset();
                let text = self.block_text(&inline.render(true));
                blocks.push_paragraph("", &hard_breaks(&text), false, edges);
            }
        }
        // A text box between this paragraph and the next one takes the break.
        let boxed = !extra.is_empty();
        for block in extra {
            list.reset();
            blocks.push(&block, false);
        }
        let mark = if self.agent {
            None
        } else {
            self.paragraph_mark(p).filter(|_| written && !boxed)
        };
        // Without id lines, a page break is its own line after the
        // paragraph that holds it, in the view's words; not when the
        // paragraph's mark is tracked, whose change splices into the next
        // block's separator and would be broken by a line between.
        if !self.agent && mark.is_none() && agent::has_page_break(p) {
            blocks.push(crate::markdown::write::PAGE_BREAK_LINE, false);
        }
        blocks.set_separator(mark);
    }

    /// A paragraph's rendered text as its block holds it: a line that would
    /// open a Markdown block (`# `, `- `, `1. `) is escaped, so it reads
    /// back as text.
    fn block_text(&self, rendered: &str) -> String {
        agent::escape_line_starts(rendered)
    }

    /// Agent view: writes the pending `<!-- pN empty -->` lines, after the
    /// page-1 marker when they open the document.
    fn flush_empty(&mut self, blocks: &mut Blocks) {
        if self.pending_empty.is_empty() {
            return;
        }
        // Empty paragraphs after a hard break sit on the new page.
        let opens = self.opens_page(false);
        self.page_lines(blocks, opens);
        for line in agent::empty_lines(&std::mem::take(&mut self.pending_empty)) {
            blocks.push_line(&format!("{}{line}", " ".repeat(self.line_indent)));
        }
    }

    /// Agent view, cached-break fallback: the page markers due before a
    /// block, page 1 included. With layout pages, `paginate` writes them.
    fn page_lines(&mut self, blocks: &mut Blocks, opens: bool) {
        let Some(total) = self.cached_pages else {
            return;
        };
        if self.page == 0 {
            self.page = 1;
            self.announce = false;
            blocks.push_line(&format!(
                "{}{}",
                " ".repeat(self.line_indent),
                agent::page_marker(1, total)
            ));
        }
        if opens && self.page < total {
            self.page += 1;
            self.announce = true;
        }
        if std::mem::take(&mut self.announce) {
            blocks.push_line(&format!(
                "{}{}",
                " ".repeat(self.line_indent),
                agent::page_marker(self.page, total)
            ));
        }
    }

    /// Whether the next block opens a page: its own cached break, or,
    /// without cached breaks, the hard break that ended the block before.
    fn opens_page(&mut self, cached_break: bool) -> bool {
        if self.page_sections.is_some() {
            std::mem::take(&mut self.break_due)
        } else {
            cached_break
        }
    }

    /// Pages turned inside a block, named before the next one.
    fn turn_pages(&mut self, turned: usize) {
        let Some(total) = self.cached_pages else {
            return;
        };
        let page = self.page.saturating_add(turned).min(total);
        if page > self.page {
            self.page = page;
            self.announce = true;
        }
    }

    /// The thread roots among `ids`, for an id line's `in c5` (agent view).
    fn threads_of(&self, ids: &[String]) -> Vec<String> {
        if !self.agent {
            return Vec::new();
        }
        let mut roots: Vec<String> = Vec::new();
        for id in ids {
            if !roots.contains(id) && !self.threads.reply_of.contains_key(id) {
                roots.push(id.clone());
            }
        }
        roots
    }

    /// The comment ranges open here plus those `container` opens and leaves
    /// open: every range a table line's `in c5` names. A text box is a
    /// story of its own; its ranges stay inside it.
    fn open_after(&self, container: &Element) -> Vec<String> {
        fn walk(e: &Element, comments: &HashMap<String, Element>, open: &mut Vec<String>) {
            for c in e.elements() {
                match c.local() {
                    "txbxContent" => {}
                    "commentRangeStart" | "commentRangeEnd" => {
                        if let Some(id) = c.attr("id").filter(|id| comments.contains_key(*id)) {
                            let at = open.iter().position(|o| o == id);
                            match (c.is("commentRangeStart"), at) {
                                (true, None) => open.push(id.to_string()),
                                (false, Some(at)) => {
                                    open.remove(at);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => walk(c, comments, open),
                }
            }
        }
        let mut open = self.open_ranges().to_vec();
        let before = open.clone();
        walk(container, &self.comments, &mut open);
        let mut all = before;
        for id in open {
            if !all.contains(&id) {
                all.push(id);
            }
        }
        all
    }

    /// The comment threads whose range is open here.
    fn open_threads(&self) -> Vec<String> {
        self.threads_of(self.open_ranges())
    }

    /// The comment ranges open here: the highlighted ones, or with the agent
    /// view's comments hidden, the ones only id lines name.
    fn open_ranges(&self) -> &[String] {
        if self.agent && !self.comments_inline {
            &self.hidden_ranges
        } else {
            &self.open_comments
        }
    }

    /// The Markdown marker for a numbered paragraph (`1.`) and the label Word
    /// prints for it (`a)`, `1.2`), advancing Word's counters.
    fn list_marker(&mut self, num_id: &str, ilvl: usize) -> Option<(String, String)> {
        if num_id == "0" {
            return None;
        }
        let (abs, level) = self.numbering.level(num_id, ilvl)?;
        let abs = abs.to_string();
        match level.format.as_str() {
            "none" => return None,
            "bullet" => return Some(("-".into(), "-".into())),
            _ => {}
        }
        let first_use = !self.started_nums.iter().any(|n| n == num_id);
        let counters = self.counters.entry(abs).or_insert([None; 9]);
        if first_use {
            self.started_nums.push(num_id.to_string());
            // A num with startOverride restarts the shared abstract list.
            if let Some((_, overrides)) = self.numbering.nums.get(num_id) {
                for (&lvl, &start) in overrides {
                    if lvl < 9 {
                        counters[lvl] = Some(i64::from(start) - 1);
                    }
                }
            }
        }
        let value = counters[ilvl].map_or(i64::from(level.start), |c| c + 1);
        counters[ilvl] = Some(value);
        for deeper in counters.iter_mut().skip(ilvl + 1) {
            *deeper = None;
        }
        let counters = *counters;
        // `%k` is level k's counter in level k's format; a level not used
        // yet stands at its start. A `%k` deeper than the paragraph's own
        // level makes Word print no label and no list indent, though the
        // paragraph counted above (probe lvl_undef_1010): no list item.
        let text = level.text.unwrap_or_else(|| format!("%{}.", ilvl + 1));
        if crate::convert::names_deeper_level(&text, u32::try_from(ilvl).unwrap_or(u32::MAX)) {
            return None;
        }
        let mut label = String::new();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            let Some(k) = (c == '%')
                .then(|| chars.peek().and_then(|d| d.to_digit(10)))
                .flatten()
                .filter(|k| (1..=9).contains(k))
            else {
                label.push(c);
                continue;
            };
            chars.next();
            let k = k as usize - 1;
            let Some((_, at)) = self.numbering.level(num_id, k) else {
                continue;
            };
            let n = counters[k].unwrap_or(i64::from(at.start));
            label.push_str(&crate::convert::list_number(
                &at.format,
                u32::try_from(n.max(0)).unwrap_or(u32::MAX),
            ));
        }
        Some((format!("{value}."), label))
    }

    /// Inline content of a paragraph plus any text-box blocks found inside it.
    /// Tracked changes around the paragraph and comment ranges still open from
    /// earlier paragraphs are opened again at its start.
    fn paragraph_inline(&mut self, p: &Element) -> (Critic, Vec<String>) {
        let mut inline = Critic::default();
        if self.flattening > 0 {
            inline.set_inline_end();
        }
        if self.agent {
            inline.set_agent();
        }
        let mut extra = Vec::new();
        let mut fields = std::mem::take(&mut self.open_fields);
        let (bold, italic) = p
            .path(&["pPr", "pStyle"])
            .and_then(|s| s.attr("val"))
            .map(|s| self.styles.emphasis(s))
            .unwrap_or((None, None));
        let base = (bold.unwrap_or(false), italic.unwrap_or(false));
        self.monospace_paragraph = self
            .styles
            .monospace_paragraph(p.path(&["pPr", "pStyle"]).and_then(|s| s.attr("val")));
        if !self.in_comment {
            for note in std::mem::take(&mut self.pending_notes) {
                inline.comment(&note);
            }
        }
        for (mark, by) in &self.revisions {
            // A change the container carries (a row's or cell's own mark, or
            // one that goes on past this paragraph) is not the content's:
            // [`critic::CARRIED`] keeps a sole change under it from naming
            // it in a note, which the writer would wrap again.
            let carried = by.as_deref().map(|by| format!("{by}{}", critic::CARRIED));
            inline.open(*mark, carried.as_deref());
        }
        // A note's label goes inside its tracked changes, before any comment.
        let start = inline.len();
        for _ in &self.open_comments {
            inline.open(Mark::Scope, None);
        }
        let closing = std::mem::replace(&mut self.closing_fields, closing_fields(p));
        self.inline(p, None, base, &mut fields, &mut inline, &mut extra);
        self.closing_fields = closing;
        // A field that ends in a later paragraph stays open for it, with
        // every field open inside it.
        if fields.iter().any(|f| f.spans) {
            self.open_fields = fields;
        }
        if !self.in_comment
            && !inline.is_blank()
            && let Some(lead) = self.lead.take()
        {
            inline.insert_raw(start, &lead);
        }
        if self.in_comment {
            inline = inline.unattributed();
        }
        (inline, extra)
    }

    fn inline(
        &mut self,
        container: &Element,
        link: Option<&str>,
        base: (bool, bool),
        fields: &mut Vec<Field>,
        out: &mut Critic,
        extra: &mut Vec<String>,
    ) {
        for child in container.elements() {
            match child.local() {
                "r" => self.run(child, link, base, fields, out, extra),
                "hyperlink" => {
                    let url = child.rel_attr("id").and_then(|id| self.rels.link(id));
                    let target = link_target(url.as_deref(), child.attr("anchor"))
                        .map(|t| ooxml::titled(&t, child.attr("tooltip")));
                    self.inline(child, target.as_deref().or(link), base, fields, out, extra);
                }
                "bookmarkStart" => {
                    if let Some(name) = child
                        .attr("name")
                        .filter(|n| self.bookmarks.contains(*n) && !self.slug_of_heading(n))
                    {
                        out.raw(&format!("<a id=\"{}\"></a>", super::xml::escape(name)));
                    }
                }
                "fldSimple" => {
                    let instruction = child.attr("instr").unwrap_or_default();
                    let url = hyperlink_instruction(instruction);
                    let code = (self.agent && url.is_none() && fields.iter().all(|f| f.in_result))
                        .then(|| printed_code(instruction))
                        .flatten();
                    if let Some(code) = &code {
                        out.raw(code);
                    }
                    self.inline(child, url.as_deref().or(link), base, fields, out, extra);
                    if code.is_some() {
                        out.raw(FIELD_END);
                    }
                }
                "ins" | "moveTo" | "del" | "moveFrom" => {
                    let (mark, by) = self.change_of(child);
                    out.open(mark, by.as_deref());
                    self.revised((mark, by), |w| {
                        w.inline(child, link, base, fields, out, extra);
                    });
                    out.close(mark);
                }
                "commentRangeStart" | "commentRangeEnd" => self.comment_range(child, Some(out)),
                "smartTag" | "customXml" | "sdt" | "sdtContent" | "bdo" | "dir" => {
                    self.inline(child, link, base, fields, out, extra);
                }
                "AlternateContent" if drawing::capturable_alternate(child) => {
                    // Sits where a `w:r` may, holding one whole drawing.
                    self.drawing(child, false, out, extra);
                }
                "AlternateContent" => {
                    if let Some(choice) = child.child("Choice").or_else(|| child.child("Fallback"))
                    {
                        self.inline(choice, link, base, fields, out, extra);
                    }
                }
                "oMathPara" | "oMath" => {
                    let latex = if child.is("oMath") {
                        let latex = math(child);
                        if latex.is_empty() {
                            continue;
                        }
                        format!("${latex}$")
                    } else {
                        let equations: Vec<String> = child
                            .children_named("oMath")
                            .map(math)
                            .filter(|m| !m.is_empty())
                            .collect();
                        if equations.is_empty() {
                            continue;
                        }
                        // Word draws a paragraph's equations side by side.
                        format!("$${}$$", equations.join(" "))
                    };
                    // An equation inserted or deleted whole is one change
                    // around it, as Word shows it.
                    match whole_math_change(child) {
                        Some(change) => {
                            let (mark, by) = self.change_of(change);
                            out.open(mark, by.as_deref());
                            self.revised((mark, by), |_| out.raw(&strip_math_changes(&latex)));
                            out.close(mark);
                        }
                        None => out.raw(&latex),
                    }
                }
                _ => {}
            }
        }
    }

    fn run(
        &mut self,
        run: &Element,
        link: Option<&str>,
        base: (bool, bool),
        fields: &mut Vec<Field>,
        out: &mut Critic,
        extra: &mut Vec<String>,
    ) {
        let rpr = run.child("rPr");
        if agent::hidden_run(rpr, &self.hidden_styles) {
            return;
        }
        let (style_bold, style_italic) = rpr
            .and_then(|r| r.child("rStyle"))
            .and_then(|s| s.attr("val"))
            .map(|s| self.styles.emphasis(s))
            .unwrap_or((None, None));
        let bold = rpr
            .and_then(|r| r.toggle("b"))
            .or(style_bold)
            .unwrap_or(base.0);
        let italic = rpr
            .and_then(|r| r.toggle("i"))
            .or(style_italic)
            .unwrap_or(base.1);
        // Agent view only: the plain conversion has no underline. In a link,
        // to a bookmark too, the character style's underline is the link's
        // look (Word's Hyperlink style), not formatting: only a direct `w:u`
        // counts.
        let in_link = link.is_some() || fields.iter().any(|f| f.link.is_some());
        let underline = self.agent
            && rpr
                .and_then(|r| r.child("u"))
                .map(is_underlined)
                .or_else(|| {
                    rpr.filter(|_| !in_link)
                        .and_then(|r| r.child("rStyle"))
                        .and_then(|s| s.attr("val"))
                        .and_then(|s| self.styles.underline(s))
                })
                .unwrap_or(false);
        // Agent view only, as underline: a highlight of any colour and a
        // single strike (spec §3). The colour (`w:highlight w:val`) round-
        // trips through the view as `<mark class="hl-<colour>">`; the
        // default yellow is the bare `<mark>`, and a colour Word does not
        // name prints as it.
        let val = self
            .agent
            .then(|| {
                rpr.and_then(|r| r.child("highlight"))
                    .and_then(|h| h.attr("val"))
            })
            .flatten();
        let highlight = val
            .is_some_and(|val| val != "none")
            .then(|| val.and_then(highlight_color).unwrap_or("yellow"));
        let strike = self.agent && rpr.and_then(|r| r.toggle("strike")).unwrap_or(false);
        // Code: a `VerbatimChar` run, or a run in a monospace face inside
        // a paragraph that is not (spec §3).
        let run_style = rpr
            .and_then(|r| r.child("rStyle"))
            .and_then(|s| s.attr("val"));
        let font = rpr.and_then(font_of).or_else(|| {
            run_style
                .and_then(|s| self.styles.font(s))
                .map(str::to_string)
        });
        let code = run_style == Some("VerbatimChar")
            || (font.as_deref().is_some_and(is_monospace) && !self.monospace_paragraph);
        let marks = Marks {
            bold,
            italic,
            underline,
            highlight,
            strike,
            code,
        };
        // A raised or lowered run keeps its tags, which the Markdown
        // reader takes back to w:vertAlign; a note reference is already
        // `[^n]`.
        let note = run
            .elements()
            .any(|c| matches!(c.local(), "footnoteReference" | "endnoteReference"));
        let tags = match rpr
            .and_then(|r| r.path(&["vertAlign"]))
            .and_then(|v| v.attr("val"))
            .filter(|_| !note)
        {
            Some("superscript") => Some(("<sup>", "</sup>")),
            Some("subscript") => Some(("<sub>", "</sub>")),
            _ => None,
        };
        // A raised run right after another one goes on inside its tags:
        // `<sup>[\[2\]](#n)</sup>`, not one link cut in two.
        let continued = tags.is_some_and(|(_, close)| out.reopen(close));
        let start = out.len();
        if let Some((open, _)) = tags.filter(|_| !continued) {
            out.raw(open);
        }
        let opened = out.len();
        for child in run.elements() {
            self.run_child(child, link, marks, fields, out, extra);
        }
        if let Some((_, close)) = tags {
            if out.len() == opened && !continued {
                out.truncate(start);
            } else {
                out.raw(close);
            }
        }
    }

    fn run_child(
        &mut self,
        child: &Element,
        link: Option<&str>,
        style: Marks,
        fields: &mut Vec<Field>,
        out: &mut Critic,
        extra: &mut Vec<String>,
    ) {
        let hidden = fields.iter().any(|f| !f.in_result);
        let field_link = fields.iter().rev().find_map(|f| f.link.clone());
        let link = field_link.as_deref().or(link);
        match child.local() {
            "t" | "delText" if !hidden => out.push_styled(&child.text(), style, link),
            "tab" | "ptab" if !hidden => out.push_styled("\t", style, link),
            "br" | "cr" if !hidden => {
                if child.attr("type") != Some("page") {
                    out.push("\n", false, false, None);
                }
            }
            "noBreakHyphen" if !hidden => out.push_styled("-", style, link),
            "instrText" | "delInstrText" => {
                if let Some(field) = fields.last_mut()
                    && !field.in_result
                {
                    field.instruction.push_str(&child.text());
                }
            }
            "fldChar" => match child.attr("fldCharType") {
                Some("begin") => {
                    if let Some(outer) = fields.last_mut()
                        && !outer.in_result
                    {
                        outer.nested = true;
                    }
                    let begin = std::ptr::from_ref(child);
                    // A field across paragraphs prints its code only where
                    // it begins its paragraph, as the writer reads it.
                    let spans = self.spanning_fields.contains(&begin)
                        && out.len() == 0
                        && self.lead.is_none();
                    fields.push(Field {
                        instruction: String::new(),
                        in_result: false,
                        link: None,
                        closes: spans || self.closing_fields.contains(&begin),
                        spans,
                        nested: false,
                        printed: false,
                    });
                }
                Some("separate") => {
                    let shown = fields
                        .split_last()
                        .is_some_and(|(_, outer)| outer.iter().all(|f| f.in_result));
                    if let Some(field) = fields.last_mut() {
                        field.in_result = true;
                        field.link = hyperlink_instruction(&field.instruction);
                        let code = (self.agent
                            && shown
                            && field.closes
                            && !field.nested
                            && field.link.is_none())
                        .then(|| printed_code(&field.instruction))
                        .flatten();
                        if let Some(code) = code {
                            out.raw(&code);
                            field.printed = true;
                        }
                    }
                }
                Some("end") => {
                    let printed = fields.pop().is_some_and(|f| f.printed);
                    if printed {
                        out.raw(FIELD_END);
                    }
                }
                _ => {}
            },
            "footnoteReference" | "endnoteReference" if !hidden => {
                if let Some(id) = child.attr("id") {
                    let key = (child.is("endnoteReference"), id.to_string());
                    let number = match self.note_refs.iter().position(|k| *k == key) {
                        Some(i) => i + 1,
                        None => {
                            self.note_refs.push(key);
                            self.note_changes.push(self.revisions.clone());
                            self.note_refs.len()
                        }
                    };
                    out.raw(&format!("[^{number}]"));
                }
            }
            "commentReference" if !hidden && !self.in_comment => {
                let Some(id) = child.attr("id") else {
                    return;
                };
                if self.agent && self.comments.contains_key(id) {
                    if !self.para_comments.iter().any(|c| c == id) {
                        self.para_comments.push(id.to_string());
                    }
                    if !self.comments_inline {
                        return;
                    }
                }
                if self.noted_at_end.contains(id) {
                    return;
                }
                if let Some(note) = self.note(id) {
                    out.comment(&note);
                    self.noted_at_reference.insert(id.to_string());
                }
            }
            "drawing" | "pict" | "object" if !hidden => self.drawing(child, true, out, extra),
            "AlternateContent" if !hidden && drawing::capturable_alternate(child) => {
                self.drawing(child, true, out, extra);
            }
            "AlternateContent" => {
                if let Some(choice) = child.child("Choice").or_else(|| child.child("Fallback")) {
                    for inner in choice.elements() {
                        self.run_child(inner, link, style, fields, out, extra);
                    }
                }
            }
            _ => {}
        }
    }

    /// Drawings as Markdown: `![alt](name)` for a plain inline image whose
    /// raster extracts to a file, a `drawing:` payload (its element, parts
    /// and relationships verbatim) for everything else — anchors, shapes,
    /// text boxes, VML, AlternateContent and unextracted images. Inside a
    /// comment the writer cannot restore a payload, so the drawing prints
    /// its text instead.
    fn drawing(
        &mut self,
        element: &Element,
        wrap: bool,
        out: &mut Critic,
        extra: &mut Vec<String>,
    ) {
        if self.in_box {
            // Inside a drawn box's content: the capturing drawing already
            // holds this one; print only what the layer can say in words.
            self.drawing_text(element, out, extra);
            return;
        }
        if self.in_comment {
            self.drawing_text(element, out, extra);
            return;
        }
        self.box_comments(element, extra);
        let mut plain_image = false;
        if drawing::is_plain_inline(element)
            && let Some(doc_pr) = element.find("docPr")
        {
            let alt = doc_pr
                .attr("descr")
                .filter(|d| !d.trim().is_empty())
                .or_else(|| doc_pr.attr("title"))
                .unwrap_or("");
            let target = element
                .find("blip")
                .and_then(|b| b.rel_attr("embed").or_else(|| b.rel_attr("link")))
                .and_then(|id| self.rels.get(id))
                .map(|r| r.target.clone());
            if let Some(image) = self.media.extracted_markdown(alt, target.as_deref()) {
                out.raw(&image);
                plain_image = true;
            }
        }
        if !plain_image {
            let scope = self.scopes.scope(element);
            let part = self.part.clone();
            out.raw(&drawing::Pending::push(
                &mut self.pendings,
                element,
                &part,
                scope,
                wrap,
            ));
        }
    }

    /// The `txbxContent` Word renders of `element` — under an
    /// `mc:AlternateContent`, only the shown branch's, never the
    /// `mc:Fallback` copy — each rendered as a block of `<!-- -->` comment
    /// lines pushed to `extra`. The drawing's `dN` payload already carries
    /// the text, so printing it as a real paragraph would write it twice;
    /// the comment keeps the view readable without cost to the write.
    fn box_comments(&mut self, element: &Element, extra: &mut Vec<String>) {
        let mut boxes = Vec::new();
        let root = match element
            .child("Choice")
            .or_else(|| element.child("Fallback"))
        {
            Some(branch) => branch,
            None => element,
        };
        root.find_all("txbxContent", &mut boxes);
        if boxes.is_empty() {
            return;
        }
        let open_comments = std::mem::take(&mut self.open_comments);
        let hidden_ranges = std::mem::take(&mut self.hidden_ranges);
        let in_box = std::mem::replace(&mut self.in_box, true);
        for content in boxes {
            let mut blocks = Blocks::new();
            let mut list = ListIndent::default();
            self.blocks(content, &mut blocks, &mut list);
            let block = blocks.finish();
            let mut comment = String::new();
            for line in block.lines() {
                // A line ending `-->` would close the comment early.
                let line = line.replace("-->", "-- >");
                if line.is_empty() {
                    comment.push('\n');
                } else {
                    let _ = writeln!(comment, "<!-- {line} -->");
                }
            }
            extra.push(comment.trim_end_matches('\n').to_string());
        }
        self.in_box = in_box;
        self.open_comments = open_comments;
        self.hidden_ranges = hidden_ranges;
    }

    /// A drawing's printable content inside a comment or a drawn text box:
    /// its text boxes as extra blocks (a text box is a story of its own, so
    /// comment ranges around its anchor do not cover it) and its image's
    /// alt text.
    fn drawing_text(&mut self, drawing: &Element, out: &mut Critic, extra: &mut Vec<String>) {
        let mut boxes = Vec::new();
        // Inside a drawn box's content, `blocks` already walked a nested
        // box through: printing it again as a block would show it twice.
        if !self.in_box {
            drawing.find_all("txbxContent", &mut boxes);
        }
        let open_comments = std::mem::take(&mut self.open_comments);
        let hidden_ranges = std::mem::take(&mut self.hidden_ranges);
        for content in boxes {
            let mut blocks = Blocks::new();
            let mut list = ListIndent::default();
            self.blocks(content, &mut blocks, &mut list);
            extra.push(blocks.finish());
        }
        self.open_comments = open_comments;
        self.hidden_ranges = hidden_ranges;
        if let Some(doc_pr) = drawing.find("docPr") {
            let alt = doc_pr
                .attr("descr")
                .filter(|d| !d.trim().is_empty())
                .or_else(|| doc_pr.attr("title"))
                .unwrap_or("");
            let target = drawing
                .find("blip")
                .and_then(|b| b.rel_attr("embed").or_else(|| b.rel_attr("link")))
                .and_then(|id| self.rels.get(id))
                .map(|r| r.target.clone());
            if let Some(image) = self.media.markdown(alt, target.as_deref()) {
                out.raw(&image);
            }
        }
    }

    fn table(&mut self, table: &Element) -> String {
        let mut rows = Vec::new();
        for tr in table_rows(table) {
            let mut row = Vec::new();
            let row_marks = tr
                .child("trPr")
                .map(|pr| self.revision_marks(pr))
                .unwrap_or_default();
            let before = tr
                .path(&["trPr", "gridBefore"])
                .and_then(|g| g.attr("val"))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);
            row.extend(std::iter::repeat_n(String::new(), before.min(64)));
            for tc in row_cells(tr) {
                // Text marked on its own keeps its tags in the agent view,
                // and so does text that nests a change in a change in
                // either view; a cell's other text shows the row's mark.
                let marks = if (self.agent && agent::text_marked(tc)) || agent::nested_marked(tc) {
                    &[][..]
                } else {
                    &row_marks[..]
                };
                row.push(self.cell_text(marks, tc));
                let span = tc
                    .path(&["tcPr", "gridSpan"])
                    .and_then(|g| g.attr("val"))
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1);
                row.extend(std::iter::repeat_n(String::new(), span.clamp(1, 64) - 1));
            }
            rows.push(row);
        }
        // Drop trailing empty columns and fully empty rows; the agent view
        // keeps every row, which its id line counts (row1, row2...).
        if !self.agent {
            rows.retain(|r| r.iter().any(|c| !c.trim().is_empty()));
        }
        if rows.is_empty() {
            return String::new();
        }
        let width = (0..rows.iter().map(Vec::len).max().unwrap_or(0))
            .rev()
            .find(|&c| {
                rows.iter()
                    .any(|r| r.get(c).is_some_and(|v| !v.trim().is_empty()))
            })
            .map_or(0, |c| c + 1);
        for row in &mut rows {
            row.truncate(width);
        }
        let (mut aligns, _) = self.column_alignments(table);
        aligns.resize(width, None);
        // Both views escape their cell text (#519), so every cell holds
        // what `table_cell` reads as already escaped.
        markdown_table(&rows, &aligns, true)
    }

    /// Each grid column's alignment, as most of its cell paragraphs have
    /// it ([`majority`]), and the cell paragraphs aligned against their
    /// column, as the table line names them (`p5 center`). Left and no
    /// alignment look alike: neither is an exception to the other.
    /// The table line's `p5 ListParagraph`, `p6-p9 Caption`: the cell
    /// paragraphs in a style other than the default, consecutive ones in
    /// one style as a range.
    fn cell_styles(&self, table: &Element) -> Vec<String> {
        let mut runs: Vec<(usize, usize, &str)> = Vec::new();
        for tr in table_rows(table) {
            for tc in row_cells(tr) {
                // As the cells range numbers them: a nested table's too.
                let mut ps = Vec::new();
                tc.find_all("p", &mut ps);
                for p in ps {
                    let (Some(index), Some(style)) = (
                        p.attr(agent::INDEX).and_then(|i| i.parse::<usize>().ok()),
                        p.path(&["pPr", "pStyle"]).and_then(|s| s.attr("val")),
                    ) else {
                        continue;
                    };
                    // A cell that states a style names it, the default too
                    // (#528); one without `w:pStyle` names none.
                    match runs.last_mut() {
                        Some((_, last, said)) if *last + 1 == index && *said == style => {
                            *last = index;
                        }
                        _ => runs.push((index, index, style)),
                    }
                }
            }
        }
        runs.into_iter()
            .map(|(first, last, style)| {
                let style = crate::markdown::view_notes::style_token(style);
                if first == last {
                    format!("p{first} {style}")
                } else {
                    format!("p{first}-p{last} {style}")
                }
            })
            .collect()
    }

    fn column_alignments(&self, table: &Element) -> (Vec<Option<&'static str>>, Vec<String>) {
        let mut columns: Vec<Vec<(Option<&str>, Option<&'static str>)>> = Vec::new();
        for tr in table_rows(table) {
            let mut column = tr
                .path(&["trPr", "gridBefore"])
                .and_then(|g| g.attr("val"))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0)
                .min(64);
            for tc in row_cells(tr) {
                if columns.len() <= column {
                    columns.resize_with(column + 1, Vec::new);
                }
                for p in tc.children_named("p") {
                    columns[column].push((p.attr(agent::INDEX), self.alignment(p)));
                }
                column += tc
                    .path(&["tcPr", "gridSpan"])
                    .and_then(|g| g.attr("val"))
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1)
                    .clamp(1, 64);
            }
        }
        let leftish = |a: Option<&str>| a.is_none_or(|a| a == "left");
        let mut aligns = Vec::new();
        let mut exceptions = Vec::new();
        for cells in &columns {
            let align = majority(cells.iter().map(|(_, a)| *a));
            aligns.push(align);
            for (index, own) in cells {
                let differs = if leftish(align) {
                    !leftish(*own)
                } else {
                    *own != align
                };
                if differs && let Some(index) = index {
                    exceptions.push(format!("p{index} {}", own.unwrap_or("left")));
                }
            }
        }
        (aligns, exceptions)
    }

    /// A paragraph's alignment: its own `w:jc`, else its style's; `None`
    /// when neither sets one. `left`, `center`, `right` or `justify`.
    fn alignment(&self, p: &Element) -> Option<&'static str> {
        let ppr = p.child("pPr");
        let jc = ppr
            .and_then(|pr| pr.child("jc"))
            .and_then(|j| j.attr("val"))
            .or_else(|| {
                ppr.and_then(|pr| pr.child("pStyle"))
                    .and_then(|s| s.attr("val"))
                    .and_then(|id| self.styles.jc(id))
            })?;
        match jc {
            "left" | "start" => Some("left"),
            "center" => Some("center"),
            "right" | "end" => Some("right"),
            "both" | "distribute" => Some("justify"),
            _ => None,
        }
    }

    /// A cell's paragraphs joined with `<br>`, inside the tracked changes of its
    /// row (`trPr`) and of the cell itself (`tcPr`). Text marked on its own
    /// keeps its tags in the agent view (`agent::text_marked`), and so does
    /// text that nests a change in a change in either view: the container's
    /// mark cannot merge into the nest's outer change without losing it.
    fn cell_text(&mut self, row_marks: &[Change], cell: &Element) -> String {
        let depth = self.revisions.len();
        let marked = (self.agent && agent::text_marked(cell)) || agent::nested_marked(cell);
        if !marked {
            self.revisions.extend_from_slice(row_marks);
        }
        if let Some(properties) = cell.child("tcPr").filter(|_| !marked) {
            let marks = self.revision_marks(properties);
            self.revisions.extend(marks);
        }
        let mut parts = Vec::new();
        self.cell_parts(cell, &mut parts);
        self.revisions.truncate(depth);
        if self.agent {
            fold_tracked_breaks(&mut parts);
        }
        // The agent view tells a line break (`<br/>`) from a paragraph's
        // end (`<br>`), which its table line counts.
        let line_break = if self.agent { "<br/>" } else { "<br>" };
        critic::join_paragraphs(parts, "<br>", self.agent).replace('\n', line_break)
    }

    /// Paragraph texts in a cell, each with the tracked change on its mark.
    fn cell_parts(&mut self, container: &Element, parts: &mut Vec<critic::Part>) {
        for child in container.elements() {
            match child.local() {
                "p" => {
                    let (inline, extra) = self.paragraph_inline(child);
                    // The agent view keeps only the mark's kind here, for
                    // `cell_text` to fold the `<br>` with.
                    let mark = if self.agent {
                        paragraph_mark_kind(child).map(|kind| (kind, None))
                    } else {
                        self.paragraph_mark(child)
                    };
                    let mark = mark.filter(|_| extra.is_empty());
                    let inline = inline.into_inline();
                    parts.push((inline.render(true), mark, inline.edges()));
                    parts.extend(extra.into_iter().map(|block| (block, None, (false, false))));
                }
                "tbl" => {
                    // Nested tables are flattened to text, one row per line.
                    self.flattening += 1;
                    for tr in table_rows(child) {
                        let row_marks = tr
                            .child("trPr")
                            .map(|pr| self.revision_marks(pr))
                            .unwrap_or_default();
                        let cells: Vec<String> = row_cells(tr)
                            .map(|tc| self.cell_text(&row_marks, tc))
                            .filter(|t| !t.is_empty())
                            .collect();
                        parts.push((critic::join_changed(cells, "; "), None, (false, false)));
                    }
                    self.flattening -= 1;
                }
                "ins" | "moveTo" | "del" | "moveFrom" => {
                    self.revised(self.change_of(child), |w| w.cell_parts(child, parts));
                }
                "commentRangeStart" | "commentRangeEnd" => self.comment_range(child, None),
                "sdt" | "sdtContent" | "customXml" => self.cell_parts(child, parts),
                _ => {}
            }
        }
    }

    /// The tracked change on a paragraph's mark, without its author inside a
    /// comment's own text.
    /// Agent view: none; paragraph marks are printed on id and table lines.
    fn paragraph_mark(&self, p: &Element) -> Option<Change> {
        if self.agent {
            return None;
        }
        paragraph_mark(p, self.story.as_deref())
            .map(|(mark, by)| (mark, by.filter(|_| !self.in_comment)))
    }

    /// The tracked change on a note paragraph's mark. A note has no id
    /// lines, so the agent view prints it inline with its tag.
    fn note_mark(&self, p: &Element) -> Option<Change> {
        if !self.agent {
            return paragraph_mark(p, self.story.as_deref());
        }
        let marks = p
            .path(&["pPr", "rPr"])
            .map(|rpr| self.revision_marks(rpr))
            .unwrap_or_default();
        let deleted = marks.iter().position(|(m, _)| *m == Mark::Deletion);
        // A note printed in its reference's change shows a mark of that
        // kind no further: the view names it with the note's other marks.
        marks
            .into_iter()
            .nth(deleted.unwrap_or(0))
            .filter(|(mark, _)| !self.revisions.iter().any(|(open, _)| open == mark))
    }

    /// The agent tag of a revision element, with its timestamp when
    /// `--dates` asks for one and the author has several.
    fn agent_tag(&self, element: &Element) -> String {
        let story = self
            .story
            .as_deref()
            .map_or(String::new(), |s| format!("{s}#"));
        let mut tagged = format!(
            "{}{story}{}",
            critic::TAG,
            agent::tag_of(element, &self.handles)
        );
        if self.dates
            && self.handles.needs_date(element.attr("author"))
            && let Some(date) = element.attr("date")
        {
            tagged.push(' ');
            tagged.push_str(date);
        }
        tagged
    }

    /// A tracked change's mark with its attribution: the agent tag, or the
    /// `Author (date)` note of the plain conversion.
    fn change_of(&self, element: &Element) -> Change {
        let (mark, by) = change_of(element, self.story.as_deref());
        if self.agent {
            (mark, Some(self.agent_tag(element)))
        } else {
            (mark, by)
        }
    }

    /// [`revision_marks`] with agent tags when the agent view is on.
    fn revision_marks(&self, properties: &Element) -> Vec<Change> {
        if !self.agent {
            return revision_marks(properties, self.story.as_deref());
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

    /// A comment's note, rendered the first time it is needed. Its text is
    /// written apart from the document around its anchor: no tracked change
    /// or comment range there applies to it.
    fn note(&mut self, id: &str) -> Option<critic::Note> {
        if let Some(note) = self.notes.get(id) {
            return Some(note.clone());
        }
        let comment = self.comments.get(id)?.clone();
        let revisions = std::mem::take(&mut self.revisions);
        let open_comments = std::mem::take(&mut self.open_comments);
        self.in_comment = true;
        let paragraphs: Vec<_> = comment.children_named("p").collect();
        let prose = self.note_text(&paragraphs, false);
        let code = self.note_text(&paragraphs, true);
        self.in_comment = false;
        self.revisions = revisions;
        self.open_comments = open_comments;
        let note = if self.agent {
            let date = (self.dates && self.handles.needs_date(comment.attr("author")))
                .then(|| comment.attr("date"))
                .flatten();
            let head = |text: String| {
                let mut inner = agent::comment_head(
                    id,
                    comment.attr("author"),
                    date,
                    &self.handles,
                    &self.threads,
                );
                inner.push_str(&text);
                inner
            };
            critic::Note {
                prose: head(prose),
                code: head(code),
            }
        } else {
            critic::Note {
                prose: comment_note(comment.attr("author"), comment.attr("date"), &prose),
                code: comment_note(comment.attr("author"), comment.attr("date"), &code),
            }
        };
        self.notes.insert(id.to_string(), note.clone());
        Some(note)
    }

    /// The note's paragraphs as one line of text. `code` renders them as
    /// the code block the note is anchored in: a fence holds the text as
    /// written and unwinds only the escape that form uses, so the note
    /// round-trips there too (#518).
    fn note_text(&mut self, paragraphs: &[&Element], code: bool) -> String {
        let mut parts = Vec::new();
        for p in paragraphs {
            let (mut inline, extra) = self.paragraph_inline(p);
            if code {
                inline.set_code_block();
            }
            let mark = self.paragraph_mark(p).filter(|_| extra.is_empty());
            let inline = inline.into_inline();
            parts.push((inline.render(true), mark, inline.edges()));
            parts.extend(extra.into_iter().map(|block| (block, None, (false, false))));
        }
        critic::join_marked(parts, " ").replace('\n', " ")
    }
}

fn table_rows(table: &Element) -> Vec<&Element> {
    let mut rows = Vec::new();
    collect_named(
        table,
        "tr",
        &["sdt", "sdtContent", "customXml", "ins"],
        &mut rows,
    );
    rows
}

fn row_cells(row: &Element) -> impl Iterator<Item = &Element> {
    let mut cells = Vec::new();
    collect_named(
        row,
        "tc",
        &["sdt", "sdtContent", "customXml", "ins"],
        &mut cells,
    );
    cells.into_iter()
}

fn collect_named<'a>(
    container: &'a Element,
    name: &str,
    wrappers: &[&str],
    out: &mut Vec<&'a Element>,
) {
    for child in container.elements() {
        if child.is(name) {
            out.push(child);
        } else if wrappers.contains(&child.local()) {
            collect_named(child, name, wrappers, out);
        }
    }
}

/// Office Math (OMML) to LaTeX, covering the structures Word's equation
/// editor emits. Each structure prints in one form the writer
/// (`markdown/math.rs`) reads back as the same element: bare brackets are a
/// delimiter, so a bracket that is text prints as `\lparen`; a delimiter
/// Word draws with other characters prints with `\left`, `\middle` and
/// `\right`. A change inside the math prints as CriticMarkup around the
/// run's text (`{++2++}`, `{--π--}`); an accepted or rejected view has none
/// left to print.
fn math(element: &Element) -> String {
    math_space(&math_inner(element, 0, false))
}

/// The equation's LaTeX with its edges trimmed. A run of spaces stays:
/// collapsing it onto one moves the line. A line break reads as a space,
/// since a break in the view would end the line (`- y` after it starts a
/// list, and a heading or a table row ends there).
fn math_space(latex: &str) -> String {
    latex
        .trim()
        .chars()
        .map(|c| if matches!(c, '\n' | '\r') { ' ' } else { c })
        .collect()
}

/// The LaTeX of `element`; `name` while inside a function's name, whose
/// words print as their command (`\sin`, `\operatorname{argmax}`).
fn math_inner(element: &Element, depth: usize, name: bool) -> String {
    if depth > 64 {
        return String::new();
    }
    let inner = |e: &Element| math_inner(e, depth + 1, name);
    // In a function's name only the base is the name: `\lim_{n}`'s limit
    // is math.
    let part = |part: &str| {
        element
            .child(part)
            .map(|e| math_inner(e, depth + 1, name && part == "e"))
            .unwrap_or_default()
    };
    let prop = |pr: &str, name: &str| {
        element
            .path(&[pr, name])
            .and_then(|e| e.attr("val"))
            .map(str::to_string)
    };
    match element.local() {
        "r" => math_run(element, name),
        "f" => {
            let (num, den) = (part("num"), part("den"));
            if prop("fPr", "type").as_deref() == Some("noBar") {
                format!("\\genfrac{{}}{{}}{{0pt}}{{}}{{{num}}}{{{den}}}")
            } else {
                format!("\\frac{{{num}}}{{{den}}}")
            }
        }
        "sSup" => {
            let sup = part("sup");
            match super::math::prime_count(&sup) {
                Some(count) => format!("{}{}", base(&part("e")), "'".repeat(count)),
                None => format!("{}^{}", base(&part("e")), group(&sup)),
            }
        }
        "sSub" => format!("{}_{}", base(&part("e")), group(&part("sub"))),
        "sSubSup" => format!(
            "{}_{}^{}",
            base(&part("e")),
            group(&part("sub")),
            group(&part("sup"))
        ),
        "sPre" => format!(
            "\\prescript{{{}}}{{{}}}{{{}}}",
            part("sub"),
            part("sup"),
            part("e")
        ),
        "rad" => {
            let degree = part("deg");
            if degree.trim().is_empty() {
                format!("\\sqrt{{{}}}", part("e"))
            } else {
                format!("\\sqrt[{degree}]{{{}}}", part("e"))
            }
        }
        "d" => {
            let open = prop("dPr", "begChr").unwrap_or_else(|| "(".into());
            let close = prop("dPr", "endChr").unwrap_or_else(|| ")".into());
            let separator = prop("dPr", "sepChr").unwrap_or_else(|| "|".into());
            let items: Vec<&Element> = element.children_named("e").collect();
            // `\binom`: Word's binomial is a bar-less fraction in brackets.
            if (open.as_str(), close.as_str()) == ("(", ")")
                && let [item] = items.as_slice()
                && let [fraction] = structure(item).as_slice()
                && fraction.is("f")
                && fraction.path(&["fPr", "type"]).and_then(|e| e.attr("val")) == Some("noBar")
            {
                let half = |part: &str| fraction.child(part).map(inner).unwrap_or_default();
                return format!("\\binom{{{}}}{{{}}}", half("num"), half("den"));
            }
            let items: Vec<String> = items.into_iter().map(inner).collect();
            let bare = match (open.as_str(), close.as_str()) {
                ("(", ")") | ("[", "]") => Some((open.clone(), close.clone())),
                ("{", "}") => Some(("\\{".into(), "\\}".into())),
                _ => None,
            };
            match bare {
                Some((open, close)) if items.len() == 1 => {
                    format!("{open}{}{close}", items.concat())
                }
                _ => format!(
                    "\\left{}{}\\right{}",
                    fence(&open),
                    items.join(&format!("\\middle{}", fence(&separator))),
                    fence(&close)
                ),
            }
        }
        "func" => {
            let fname = element
                .child("fName")
                .map(|e| math_inner(e, depth + 1, true))
                .unwrap_or_default();
            format!("{}{}", fname.trim(), operand(element.child("e"), depth))
        }
        "nary" => {
            let chr = prop("naryPr", "chr");
            let symbol = match chr.as_deref().and_then(|c| c.chars().next()) {
                None => "\\int".to_string(),
                Some(c) => super::math::nary_command(c)
                    .map_or_else(|| c.to_string(), |name| format!("\\{name}")),
            };
            let (sub, sup) = (part("sub"), part("sup"));
            let mut out = symbol;
            if !sub.trim().is_empty() {
                out.push_str(&format!("_{}", group(&sub)));
            }
            if !sup.trim().is_empty() {
                out.push_str(&format!("^{}", group(&sup)));
            }
            format!("{out}{}", operand(element.child("e"), depth))
        }
        "acc" => {
            let chr = prop("accPr", "chr");
            let command = match chr.as_deref() {
                None => "hat",
                Some("~") => "tilde",
                Some("\u{304}" | "¯") => "bar",
                Some("˙") => "dot",
                Some("→") => "vec",
                Some(c) => c
                    .chars()
                    .next()
                    .and_then(super::math::accent_command)
                    .unwrap_or("hat"),
            };
            format!("\\{command}{{{}}}", part("e"))
        }
        "bar" => {
            let command = if prop("barPr", "pos").as_deref() == Some("top") {
                "overline"
            } else {
                "underline"
            };
            format!("\\{command}{{{}}}", part("e"))
        }
        "groupChr" => {
            let top = prop("groupChrPr", "pos").as_deref() == Some("top")
                || prop("groupChrPr", "chr").as_deref() == Some("⏞");
            let command = if top { "overbrace" } else { "underbrace" };
            format!("\\{command}{{{}}}", part("e"))
        }
        "limLow" | "limUpp" => {
            let upper = element.local() == "limUpp";
            let (e, lim) = (part("e"), part("lim"));
            // A function's name and a brace take their limit as a script;
            // anything else is stacked.
            let brace = if upper {
                "\\overbrace{"
            } else {
                "\\underbrace{"
            };
            if name || e.trim_start().starts_with(brace) {
                let e = if name { base(&e) } else { e.trim().to_string() };
                format!("{e}{}{}", if upper { '^' } else { '_' }, group(&lim))
            } else {
                let command = if upper { "overset" } else { "underset" };
                format!("\\{command}{{{lim}}}{{{e}}}")
            }
        }
        "m" => {
            let rows: Vec<String> = element
                .children_named("mr")
                .map(|row| {
                    row.children_named("e")
                        .map(inner)
                        .collect::<Vec<_>>()
                        .join(" & ")
                })
                .collect();
            format!("\\begin{{matrix}}{}\\end{{matrix}}", rows.join(" \\\\ "))
        }
        "eqArr" => {
            // Word aligns an array's lines at each `&` in their text and
            // draws none: `aligned`'s `&` is that alignment point. Only the
            // line's own runs hold one; a matrix inside keeps its `\&`.
            let own_runs = |e: &Element| {
                e.is("r") || (e.is("ins") || e.is("del")) && e.elements().all(|r| r.is("r"))
            };
            let rows: Vec<String> = element
                .children_named("e")
                .map(|row| {
                    row.elements()
                        .map(|e| {
                            let latex = inner(e);
                            if own_runs(e) {
                                latex.replace("\\&", "&")
                            } else {
                                latex
                            }
                        })
                        .collect()
                })
                .collect();
            format!("\\begin{{aligned}}{}\\end{{aligned}}", rows.join(" \\\\ "))
        }
        "box" => {
            let operator = prop("boxPr", "opEmu").is_some_and(|v| v != "0" && v != "off");
            let command = if operator { "mathop" } else { "mathord" };
            format!("\\{command}{{{}}}", part("e"))
        }
        name if name.ends_with("Pr") => String::new(),
        _ => element.elements().map(inner).collect(),
    }
}

/// The `w:ins` or `w:del` of math every run of which is inserted, or
/// every run deleted.
fn whole_math_change(math: &Element) -> Option<&Element> {
    fn runs<'e>(element: &'e Element, out: &mut Vec<&'e Element>) {
        for child in element.elements() {
            if child.is("r") {
                out.push(child);
            } else if !child.local().ends_with("Pr") {
                runs(child, out);
            }
        }
    }
    let mut all = Vec::new();
    runs(math, &mut all);
    let mut first: Option<&Element> = None;
    for run in all {
        let change = run.elements().find(|e| e.is("ins") || e.is("del"))?;
        match first {
            Some(seen) if seen.local() != change.local() => return None,
            Some(_) => {}
            None => first = Some(change),
        }
    }
    first
}

/// `latex` without the marks of the changes inside it.
fn strip_math_changes(latex: &str) -> String {
    use super::math::{DELETE_END, DELETE_START, INSERT_END, INSERT_START};
    latex
        .chars()
        .filter(|c| ![INSERT_START, INSERT_END, DELETE_START, DELETE_END].contains(c))
        .collect()
}

/// The structures inside `e`, without its properties.
fn structure(e: &Element) -> Vec<&Element> {
    e.elements()
        .filter(|c| !c.local().ends_with("Pr"))
        .collect()
}

/// A math run's text as LaTeX: in a function's name, the name's command;
/// else escaped and set in its style, inside CriticMarkup when the run is
/// inserted or deleted.
fn math_run(run: &Element, name: bool) -> String {
    let mut text = String::new();
    let mut change = None;
    for child in run.elements() {
        match child.local() {
            "t" => text.push_str(&child.text()),
            local @ ("ins" | "del") => {
                // Stand-ins the paragraph's CriticMarkup leaves whole.
                change = Some(if local == "ins" {
                    (super::math::INSERT_START, super::math::INSERT_END)
                } else {
                    (super::math::DELETE_START, super::math::DELETE_END)
                });
                child
                    .elements()
                    .filter(|e| e.is("t"))
                    .for_each(|t| text.push_str(&t.text()));
            }
            _ => {}
        }
    }
    let latex = math_text(run, &text.replace('\u{2061}', ""), name);
    match change {
        Some((open, close)) if !latex.is_empty() => format!("{open}{latex}{close}"),
        _ => latex,
    }
}

/// A math run's text, `text`, as LaTeX in the run's style.
fn math_text(run: &Element, text: &str, name: bool) -> String {
    if text.is_empty() {
        return String::new();
    }
    let properties = |name: &str| {
        run.path(&["rPr", name])
            .or_else(|| run.path(&["ins", "rPr", name]))
            .or_else(|| run.path(&["del", "rPr", name]))
    };
    if name && properties("nor").is_none() {
        let word = text.trim();
        return if super::math::FUNCTIONS.contains(&word) {
            format!("\\{word} ")
        } else if word.chars().all(char::is_alphabetic) {
            format!("\\operatorname{{{word}}}")
        } else {
            escape_math(text)
        };
    }
    if properties("nor").is_some() {
        return format!("\\text{{{}}}", escape_math_text(text));
    }
    let command = match (
        properties("scr").and_then(|e| e.attr("val")),
        properties("sty").and_then(|e| e.attr("val")),
    ) {
        (Some("double-struck"), _) => Some("mathbb"),
        (Some("script"), _) => Some("mathcal"),
        (Some("fraktur"), _) => Some("mathfrak"),
        (Some("sans-serif"), _) => Some("mathsf"),
        (Some("monospace"), _) => Some("mathtt"),
        // Word sets digits and operators upright already.
        (_, Some("p")) if text.chars().any(char::is_alphabetic) => Some("mathrm"),
        (_, Some("b")) => Some("mathbf"),
        (_, Some("bi")) => Some("boldsymbol"),
        _ => None,
    };
    let latex = escape_math(text);
    match command {
        Some(command) => format!("\\{command}{{{latex}}}"),
        None => latex,
    }
}

/// Math text with the characters LaTeX acts on escaped; brackets, which
/// would build a delimiter, as their commands.
fn escape_math(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\backslash "),
            '{' => out.push_str("\\lbrace "),
            '}' => out.push_str("\\rbrace "),
            '(' => out.push_str("\\lparen "),
            ')' => out.push_str("\\rparen "),
            '[' => out.push_str("\\lbrack "),
            ']' => out.push_str("\\rbrack "),
            '^' | '_' | '&' | '$' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

/// `\text{…}`'s text: braces, backslashes and `$` escaped.
fn escape_math_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\backslash "),
            '{' | '}' | '$' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

/// A delimiter's character after `\left`, `\middle` or `\right`: none is `.`.
fn fence(chr: &str) -> String {
    match chr {
        "" => ".".into(),
        "{" => "\\{".into(),
        "}" => "\\}".into(),
        "‖" => "\\|".into(),
        other => other.into(),
    }
}

/// What a function or an n-ary operator applies to: one bracketed
/// delimiter, one character after a space, anything else in braces.
fn operand(e: Option<&Element>, depth: usize) -> String {
    let Some(e) = e else {
        return "{}".into();
    };
    let latex = math_inner(e, depth + 1, false);
    let latex = latex.trim();
    let single = math_chars(latex) == 1 && latex != "{" && latex != "}";
    let delimiter = matches!(structure(e).as_slice(), [d] if d.is("d"))
        && (latex.starts_with('(') || latex.starts_with('[') || latex.starts_with("\\{"));
    if delimiter {
        latex.to_string()
    } else if single {
        format!(" {latex}")
    } else {
        format!("{{{latex}}}")
    }
}

/// The characters of math LaTeX, without the marks of changes inside it.
fn math_chars(latex: &str) -> usize {
    use super::math::{DELETE_END, DELETE_START, INSERT_END, INSERT_START};
    latex
        .chars()
        .filter(|c| ![INSERT_START, INSERT_END, DELETE_START, DELETE_END].contains(c))
        .count()
}

fn group(s: &str) -> String {
    let s = s.trim();
    if math_chars(s) == 1 {
        s.to_string()
    } else {
        format!("{{{s}}}")
    }
}

/// The base of a script or limit: a function's command bare, else a group.
fn base(s: &str) -> String {
    let s = s.trim();
    let command = s.strip_prefix('\\').is_some_and(|rest| {
        rest.chars().all(|c| c.is_ascii_alphabetic())
            || (rest.starts_with("operatorname{") && rest.matches('{').count() == 1)
    });
    if command { s.to_string() } else { group(s) }
}

/// The link of a `HYPERLINK` field instruction: its URL, `#bookmark` for
/// `\l "bookmark"`, both (`url#bookmark`), titled with `\o`'s tooltip.
fn hyperlink_instruction(instruction: &str) -> Option<String> {
    let words = field_words(instruction);
    let ((name, false), rest) = words.split_first()? else {
        return None;
    };
    if !name.eq_ignore_ascii_case("HYPERLINK") {
        return None;
    }
    let (mut url, mut anchor, mut title) = (None, None, None);
    let mut rest = rest.iter();
    while let Some((word, quoted)) = rest.next() {
        if *quoted || !word.starts_with('\\') {
            url.get_or_insert(word.as_str());
            continue;
        }
        // `\l`, `\o` and `\t` take an argument; `\h`, `\m`, `\n` none.
        let argument = matches!(word.to_ascii_lowercase().as_str(), "\\l" | "\\o" | "\\t")
            .then(|| rest.next())
            .flatten()
            .map(|(w, _)| w.as_str());
        match word.to_ascii_lowercase().as_str() {
            "\\l" => anchor = argument,
            "\\o" => title = argument,
            _ => {}
        }
    }
    link_target(url, anchor).map(|t| ooxml::titled(&t, title))
}

/// The table style the view names for `t`. Table Grid, the writer's
/// default (Word's Insert Table), is none unless the source defines one of
/// its own (`custom_grid`); no style is Word's Normal Table, which the view
/// names so the writer adds none. Borders the table carries itself print on
/// its line (`borders …`).
fn view_table_style(t: &Element, custom_grid: bool) -> Option<&str> {
    match t.path(&["tblPr", "tblStyle"]).and_then(|s| s.attr("val")) {
        Some("TableGrid") if !custom_grid => None,
        Some(style) => Some(style),
        None => Some("TableNormal"),
    }
}

/// Whether `styles` defines Table Grid with one single border all round
/// that is not Word's 0.5pt: a grid the legend can name (`TableGrid, all
/// borders 1.5pt`) and the writer would otherwise replace with the thin
/// one. Borders that vary, or none, keep Word's grid.
fn custom_table_grid(styles: Option<&Element>) -> bool {
    let Some(borders) = styles
        .and_then(|s| {
            s.children_named("style")
                .find(|s| s.attr("styleId") == Some("TableGrid"))
        })
        .and_then(|s| s.path(&["tblPr", "tblBorders"]))
    else {
        return false;
    };
    let sizes: Option<Vec<&str>> = ["top", "left", "bottom", "right", "insideH", "insideV"]
        .iter()
        .map(|side| {
            borders
                .child(side)
                .filter(|b| b.attr("val") == Some("single"))
                .and_then(|b| b.attr("sz"))
        })
        .collect();
    sizes.is_some_and(|sizes| sizes.iter().all(|&sz| sz == sizes[0]) && sizes[0] != "4")
}

/// The table styles of `body`'s tables the view names, in the order they
/// first come, and whether a table has none (the writer's Table Grid).
fn table_styles(body: &Element, custom_grid: bool) -> (Vec<String>, bool) {
    let mut styles: Vec<String> = Vec::new();
    let mut unstyled = false;
    for t in agent::tables(body) {
        match view_table_style(t, custom_grid) {
            Some(s) => {
                if !styles.iter().any(|x| x == s) {
                    styles.push(s.to_string());
                }
            }
            None => unstyled = true,
        }
    }
    (styles, unstyled)
}

/// The text of `p` the accepted document shows: its `w:t`, not deleted
/// or moved-away text.
fn shown_text(p: &Element) -> String {
    fn walk(e: &Element, out: &mut String) {
        for c in e.elements() {
            match c.local() {
                "del" | "moveFrom" | "pPr" | "rPr" => {}
                "t" => out.push_str(&c.text()),
                _ => walk(c, out),
            }
        }
    }
    let mut out = String::new();
    walk(p, &mut out);
    out
}

/// A link's destination from its URL and its bookmark: `url`, `#bookmark`,
/// or `url#bookmark`.
fn link_target(url: Option<&str>, anchor: Option<&str>) -> Option<String> {
    match (
        url.filter(|u| !u.is_empty()),
        anchor.filter(|a| !a.is_empty()),
    ) {
        (Some(url), Some(anchor)) => Some(format!("{url}#{anchor}")),
        (Some(url), None) => Some(url.to_string()),
        (None, Some(anchor)) => Some(format!("#{anchor}")),
        (None, None) => None,
    }
}

/// A field instruction's words, each with whether it was quoted: `"a b"`
/// is one word, and `\"` inside quotes is a quote.
fn field_words(instruction: &str) -> Vec<(String, bool)> {
    let mut words = Vec::new();
    let mut chars = instruction.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '"' {
            chars.next();
            let mut word = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '"' => break,
                    // `\"` is a quote, `\\` a backslash (a path's).
                    '\\' if matches!(chars.peek(), Some('"' | '\\')) => {
                        word.extend(chars.next());
                    }
                    _ => word.push(c),
                }
            }
            words.push((word, true));
        } else {
            let mut word = String::new();
            while let Some(&c) = chars.peek().filter(|c| !c.is_whitespace() && **c != '"') {
                word.push(c);
                chars.next();
            }
            words.push((word, false));
        }
    }
    words
}

/// The bookmarks under `roots` a link (`w:hyperlink w:anchor`, `HYPERLINK
/// \l`) or a reference field (`REF`, `PAGEREF`, `NOTEREF`) names. Unless
/// `fields_print` (the agent view, which prints a field as its code), every
/// `_Ref…` bookmark too: Word's cross-reference targets.
fn bookmark_targets<'e>(
    roots: impl Iterator<Item = &'e Element>,
    fields_print: bool,
) -> HashSet<String> {
    fn walk(e: &Element, refs: bool, codes: &mut Vec<String>, out: &mut HashSet<String>) {
        for c in e.elements() {
            match c.local() {
                "hyperlink" => {
                    if let Some(anchor) = c.attr("anchor").filter(|a| !a.is_empty()) {
                        out.insert(anchor.to_string());
                    }
                    walk(c, refs, codes, out);
                }
                "fldSimple" => {
                    codes.push(c.attr("instr").unwrap_or_default().to_string());
                    walk(c, refs, codes, out);
                }
                "fldChar" if c.attr("fldCharType") == Some("begin") => codes.push(String::new()),
                "instrText" | "delInstrText" => {
                    if let Some(code) = codes.last_mut() {
                        code.push_str(&c.text());
                    }
                }
                // Word's cross-reference targets print whether or not a
                // field still names them: the plain view prints a REF field
                // as its text, and the file it writes back keeps only the
                // bookmark.
                "bookmarkStart" if refs => {
                    if let Some(name) = c.attr("name").filter(|n| n.starts_with("_Ref")) {
                        out.insert(name.to_string());
                    }
                }
                _ => walk(c, refs, codes, out),
            }
        }
    }
    let mut codes = Vec::new();
    let mut out = HashSet::new();
    for root in roots {
        walk(root, !fields_print, &mut codes, &mut out);
    }
    for code in &codes {
        let words = field_words(code);
        match words
            .first()
            .map(|(w, _)| w.to_ascii_uppercase())
            .as_deref()
        {
            Some("HYPERLINK") => {
                if let Some(target) = hyperlink_instruction(code)
                    && let Some((_, anchor)) = target
                        .split(ooxml::TITLE)
                        .next()
                        .unwrap_or("")
                        .split_once('#')
                {
                    out.insert(anchor.to_string());
                }
            }
            Some("REF" | "PAGEREF" | "NOTEREF") => {
                if let Some((name, _)) = words.get(1) {
                    out.insert(name.clone());
                }
            }
            _ => {}
        }
    }
    out
}

/// The colour every text run of a header or footer paragraph `p` sets
/// itself (`w:rPr/w:color`), when they all set the same one.
fn story_color(p: &Element) -> Option<String> {
    let mut runs = Vec::new();
    p.find_all("r", &mut runs);
    let mut colors = runs
        .iter()
        .filter(|r| r.child("t").is_some())
        .map(|r| r.path(&["rPr", "color"]).and_then(|c| c.attr("val")));
    let first = colors.next()??;
    colors.all(|c| c == Some(first)).then(|| first.to_string())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::io::Write;

    const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#;

    #[test]
    fn a_break_inside_an_equation_stays_a_newline() {
        assert_eq!(hard_breaks("a\nb $x\ny$ c\nd"), "a\\\nb $x\ny$ c\\\nd");
        assert_eq!(hard_breaks("$$a\nb$$"), "$$a\nb$$");
        assert_eq!(hard_breaks("\\$ not\n $x$"), "\\$ not\\\n $x$");
        assert_eq!(hard_breaks("plain\n"), "plain\\\n");
    }

    #[test]
    fn math_keeps_a_run_of_spaces_and_reads_a_line_break_as_one() {
        let spaced = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"><m:r><m:t>logit</m:t></m:r><m:r><m:t xml:space="preserve"> =  </m:t></m:r><m:r><m:t>λ</m:t></m:r></m:oMath>"#;
        let element = super::ooxml::parse_xml(spaced.as_bytes()).expect("spaced math");
        assert_eq!(math(&element), "logit =  λ");
        let broken = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"><m:r><m:t xml:space="preserve">.&#xA;\label</m:t></m:r></m:oMath>"#;
        let element = super::ooxml::parse_xml(broken.as_bytes()).expect("broken math");
        assert_eq!(math(&element), ". \\backslash label");
        // A break in the view would end the line: `- y` would start a list,
        // and a heading or a table row would end there.
        for (text, latex) in [
            ("x&#xA;- y", "x - y"),
            ("x&#xA;# y", "x # y"),
            ("x&#xA;&#xA;y", "x  y"),
            ("x&#xD;&#xA;y", "x  y"),
        ] {
            let xml = format!(
                r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"><m:r><m:t xml:space="preserve">{text}</m:t></m:r></m:oMath>"#
            );
            let element = super::ooxml::parse_xml(xml.as_bytes()).expect("math");
            assert_eq!(math(&element), latex, "{text}");
        }
    }

    #[test]
    fn a_name_loses_the_xml_indentation_and_keeps_its_spaces() {
        assert_eq!(without_layout("Un-named\n  "), "Un-named");
        assert_eq!(without_layout("\n    Ada\r\n"), "Ada");
        assert_eq!(without_layout("  Bob  "), "  Bob  ");
        assert_eq!(without_layout("  Bob\n"), "  Bob");
        assert_eq!(without_layout("Ann Lee"), "Ann Lee");
    }

    pub(crate) fn zip(parts: &[(&str, &str)]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, body) in parts {
            writer.start_file(*name, options).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn docx(body: &str, extra: &[(&str, &str)]) -> Vec<u8> {
        let document = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {W}><w:body>{body}</w:body></w:document>"#
        );
        let mut parts: Vec<(&str, &str)> = vec![
            (
                "_rels/.rels",
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            ),
            ("word/document.xml", &document),
        ];
        parts.extend_from_slice(extra);
        zip(&parts)
    }

    fn md(bytes: &[u8]) -> String {
        convert(bytes, &Options::default()).unwrap().markdown
    }

    fn p(style: &str, runs: &str) -> String {
        let ppr = if style.is_empty() {
            String::new()
        } else {
            format!(r#"<w:pPr><w:pStyle w:val="{style}"/></w:pPr>"#)
        };
        format!("<w:p>{ppr}{runs}</w:p>")
    }

    fn r(text: &str) -> String {
        format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
    }

    #[test]
    fn headings_emphasis_links_and_title_style() {
        let body = [
            p("Title", &r("Report")),
            p("Heading2", r#"<w:r><w:rPr><w:b/></w:rPr><w:t>Scope</w:t></w:r>"#),
            p(
                "",
                &format!(
                    r#"{}<w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">bold </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>run</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t xml:space="preserve"> it</w:t></w:r><w:r><w:t>, see </w:t></w:r><w:hyperlink r:id="rId9"><w:r><w:t>docs</w:t></w:r></w:hyperlink><w:r><w:t>.</w:t></w:r>"#,
                    r("Plain ")
                ),
            ),
            r#"<w:p><w:pPr><w:outlineLvl w:val="2"/></w:pPr><w:r><w:t>Outline</w:t></w:r></w:p>"#.to_string(),
            r#"<w:p><w:r><w:t>a</w:t></w:r><w:r><w:br/><w:t>b</w:t><w:tab/><w:t>c</w:t></w:r></w:p>"#.to_string(),
            "<w:p/>".to_string(),
        ]
        .concat();
        let bytes = docx(
            &body,
            &[
                (
                    "word/_rels/document.xml.rels",
                    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/a b" TargetMode="External"/></Relationships>"#,
                ),
                (
                    "docProps/core.xml",
                    r#"<cp:coreProperties xmlns:cp="x" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Quarterly</dc:title><dc:creator>Ada</dc:creator></cp:coreProperties>"#,
                ),
            ],
        );
        assert_eq!(
            md(&bytes),
            "# Report\n\n## Scope\n\nPlain **bold run** _it_, see [docs](https://example.com/a%20b).\n\n### Outline\n\na\\\nb\tc\n"
        );
    }

    #[test]
    fn numbered_and_bulleted_lists() {
        let numbering = format!(
            r#"<w:numbering {W}>
<w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/></w:lvl><w:lvl w:ilvl="1"><w:numFmt w:val="bullet"/></w:lvl></w:abstractNum>
<w:abstractNum w:abstractNumId="2"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/></w:lvl><w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="lowerLetter"/></w:lvl></w:abstractNum>
<w:num w:numId="5"><w:abstractNumId w:val="1"/></w:num><w:num w:numId="6"><w:abstractNumId w:val="2"/></w:num>
</w:numbering>"#
        );
        let item = |num: u32, lvl: u32, text: &str| {
            format!(
                r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="{lvl}"/><w:numId w:val="{num}"/></w:numPr></w:pPr>{}</w:p>"#,
                r(text)
            )
        };
        let body = [
            item(5, 0, "apple"),
            item(5, 1, "green"),
            item(5, 0, "pear"),
            p("", &r("between")),
            item(6, 0, "one"),
            item(6, 1, "sub"),
            item(6, 1, "sub2"),
            item(6, 0, "two"),
        ]
        .concat();
        let bytes = docx(&body, &[("word/numbering.xml", &numbering)]);
        assert_eq!(
            md(&bytes),
            "- apple\n  - green\n- pear\n\nbetween\n\n1. one\n   1. sub\n   2. sub2\n2. two\n"
        );
    }

    #[test]
    fn tables_with_spans_and_nested_tables() {
        let tc = |content: &str| format!("<w:tc>{content}</w:tc>");
        let nested = format!(
            "<w:tbl><w:tr>{}{}</w:tr></w:tbl>",
            tc(&p("", &r("n1"))),
            tc(&p("", &r("n2")))
        );
        let body = format!(
            r#"<w:tbl><w:tr>{}{}{}</w:tr><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr>{}</w:tc>{}</w:tr><w:tr>{}{}{}</w:tr></w:tbl>"#,
            tc(&p("", &r("Name"))),
            tc(&p("", &r("Q1"))),
            tc(&p("", &r("Q2"))),
            p("", &r("wide|cell")),
            tc(&p("", &r("x"))),
            tc(&format!("{}{}", p("", &r("line1")), p("", &r("line2")))),
            tc(&nested),
            tc(""),
        );
        assert_eq!(
            md(&docx(&body, &[])),
            "|Name|Q1|Q2|\n|-|-|-|\n|wide\\|cell||x|\n|line1<br>line2|n1; n2||\n"
        );
    }

    #[test]
    fn subscript_and_superscript_runs_keep_their_tags_through_a_round_trip() {
        let vert = |val: &str, text: &str| {
            format!(r#"<w:r><w:rPr><w:vertAlign w:val="{val}"/></w:rPr><w:t>{text}</w:t></w:r>"#)
        };
        let runs = [
            r("Water is H"),
            vert("subscript", "2"),
            r("O and area x"),
            vert("superscript", "2"),
            r("."),
            vert("baseline", " Plain"),
        ]
        .concat();
        let expected = "Water is H<sub>2</sub>O and area x<sup>2</sup>. Plain\n";
        assert_eq!(md(&docx(&p("", &runs), &[])), expected);
        // The Markdown reader takes the tags back to w:vertAlign.
        let docx_again = crate::markdown::markdown_to_docx(expected, &Default::default())
            .unwrap()
            .docx;
        assert_eq!(md(&docx_again), expected);
        // A raised note reference is already `[^n]`.
        let note = r#"<w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteReference w:id="2"/></w:r>"#;
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="2"><w:p><w:r><w:t>Note.</w:t></w:r></w:p></w:footnote></w:footnotes>"#
        );
        assert_eq!(
            md(&docx(
                &p("", &format!("{runs}{note}")),
                &[("word/footnotes.xml", &footnotes)]
            )),
            "Water is H<sub>2</sub>O and area x<sup>2</sup>. Plain[^1]\n\n[^1]: Note.\n"
        );
    }

    #[test]
    fn a_table_cell_keeps_its_equations_latex() {
        assert_eq!(
            table_cell(r"C:\dir $x=\frac{a|b}{2}$ and $$\sqrt{y}$$ a|b", false),
            r"C:\\dir $x=\frac{a\|b}{2}$ and $$\sqrt{y}$$ a\|b"
        );
        // Two dollars are math, as Markdown reads them; a lone one is text.
        assert_eq!(table_cell(r"$5 \ $", false), r"$5 \ $");
        assert_eq!(table_cell(r"costs $5\", false), r"costs $5\\");
    }

    #[test]
    fn an_agent_view_cell_keeps_an_equations_norm_bars() {
        // A pipe table reads `\|` as `|`, so an equation's `\|` (‖) needs
        // its backslash doubled; outside an equation it stays an escape.
        assert_eq!(
            table_cell(r"$\left\|x\right\|$ a\\b \| c", true),
            r"$\left\\|x\right\\|$ a\\b \| c"
        );
        assert_eq!(table_cell(r"$$\|y\|$$ a|b", true), r"$$\\|y\\|$$ a\|b");
        // An escaped dollar opens no equation.
        assert_eq!(table_cell(r"\$5 \| \$6", true), r"\$5 \| \$6");
        assert_eq!(table_cell(r"$a|b$", true), r"$a\|b$");
    }

    #[test]
    fn monospace_faces_are_named_by_whole_words() {
        for face in [
            "Courier New",
            "Consolas",
            "DejaVu Sans Mono",
            "Source Code Pro",
            "Cascadia-Code",
            "Lucida Sans Typewriter",
        ] {
            assert!(is_monospace(face), "{face}");
        }
        for face in [
            "Monotype Corsiva",
            "Barcode 39",
            "Code2000",
            "Calibri",
            "Times New Roman",
        ] {
            assert!(!is_monospace(face), "{face}");
        }
    }

    #[test]
    fn office_math_structures_become_latex() {
        let mr = |t: &str| format!("<m:r><m:t>{t}</m:t></m:r>");
        let e = |inner: &str| format!("<m:e>{inner}</m:e>");
        let cases = [
            // Word's equation editor nests lim and sin² inside m:fName.
            (
                format!(
                    "<m:func><m:fName><m:limLow>{}<m:lim>{}</m:lim></m:limLow></m:fName>{}</m:func>",
                    e(&mr("lim")),
                    mr("n→∞"),
                    e(&mr("a"))
                ),
                r"\lim_{n→∞} a",
            ),
            (
                format!(
                    "<m:func><m:fName><m:sSup>{}<m:sup>{}</m:sup></m:sSup></m:fName>{}</m:func>",
                    e(&mr("sin")),
                    mr("2"),
                    e(&mr("x"))
                ),
                r"\sin^2 x",
            ),
            (
                format!(
                    "<m:func><m:fName>{}</m:fName>{}</m:func>",
                    mr("f"),
                    e(&format!("<m:d>{}</m:d>", e(&mr("x"))))
                ),
                r"\operatorname{f}(x)",
            ),
            // A bracket that is text is not a delimiter.
            (mr("f(x]"), r"f\lparen x\rbrack"),
            (
                format!(
                    r#"<m:d><m:dPr><m:begChr m:val="⟨"/><m:endChr m:val="⟩"/></m:dPr>{}{}</m:d>"#,
                    e(&mr("a")),
                    e(&mr("b"))
                ),
                r"\left⟨a\middle|b\right⟩",
            ),
            (
                format!(
                    r#"<m:d><m:dPr><m:begChr m:val="{{"/><m:endChr m:val=""/></m:dPr>{}</m:d>"#,
                    e(&mr("x"))
                ),
                r"\left\{x\right.",
            ),
            (
                format!(
                    r#"<m:d>{}</m:d>"#,
                    e(&format!(
                        r#"<m:f><m:fPr><m:type m:val="noBar"/></m:fPr><m:num>{}</m:num><m:den>{}</m:den></m:f>"#,
                        mr("n"),
                        mr("k")
                    ))
                ),
                r"\binom{n}{k}",
            ),
            (
                format!(
                    "<m:limLow>{}<m:lim>{}</m:lim></m:limLow>",
                    e(&format!("<m:groupChr>{}</m:groupChr>", e(&mr("x+y")))),
                    mr("n")
                ),
                r"\underbrace{x+y}_n",
            ),
            (
                format!("<m:limUpp>{}<m:lim>{}</m:lim></m:limUpp>", e(&mr("x")), mr("n")),
                r"\overset{n}{x}",
            ),
            (
                r#"<m:r><m:rPr><m:sty m:val="p"/></m:rPr><m:t>d</m:t></m:r><m:r><m:t>x</m:t></m:r>"#.to_string(),
                r"\mathrm{d}x",
            ),
            // A change inside the math: CriticMarkup around its runs.
            (
                r#"<m:r><m:t>A=</m:t></m:r><m:r><w:ins w:id="1" w:author="a"><m:t>2</m:t></w:ins></m:r><m:r><w:del w:id="2" w:author="a"><m:t>3</m:t></w:del></m:r>"#.to_string(),
                "A={++2++}{--3--}",
            ),
            (
                format!("<m:rad><m:deg/>{}</m:rad>", e(&mr("x"))),
                r"\sqrt{x}",
            ),
            (
                format!("<m:rad><m:deg>{}</m:deg>{}</m:rad>", mr("3"), e(&mr("x"))),
                r"\sqrt[3]{x}",
            ),
            (
                format!(
                    r#"<m:nary><m:naryPr><m:chr m:val="∑"/></m:naryPr><m:sub>{}</m:sub><m:sup>{}</m:sup>{}</m:nary>"#,
                    mr("i=1"),
                    mr("n"),
                    e(&mr("i"))
                ),
                r"\sum_{i=1}^n i",
            ),
            (
                format!(
                    "<m:nary><m:sub>{}</m:sub><m:sup>{}</m:sup>{}</m:nary>",
                    mr("0"),
                    mr("1"),
                    e(&mr("x"))
                ),
                r"\int_0^1 x",
            ),
            (
                format!(
                    r#"<m:nary><m:naryPr><m:chr m:val="∭"/></m:naryPr><m:sub/><m:sup/>{}</m:nary>"#,
                    e(&mr("f"))
                ),
                r"\iiint f",
            ),
            (
                format!(
                    r#"<m:acc><m:accPr><m:chr m:val="̃"/></m:accPr>{}</m:acc>"#,
                    e(&mr("x"))
                ),
                r"\tilde{x}",
            ),
            (format!("<m:acc>{}</m:acc>", e(&mr("x"))), r"\hat{x}"),
            (
                format!(
                    r#"<m:bar><m:barPr><m:pos m:val="top"/></m:barPr>{}</m:bar>"#,
                    e(&mr("x"))
                ),
                r"\overline{x}",
            ),
            (format!("<m:bar>{}</m:bar>", e(&mr("x"))), r"\underline{x}"),
            (
                format!(
                    "<m:m><m:mr>{}{}</m:mr><m:mr>{}{}</m:mr></m:m>",
                    e(&mr("a")),
                    e(&mr("b")),
                    e(&mr("c")),
                    e(&mr("d"))
                ),
                r"\begin{matrix}a & b \\ c & d\end{matrix}",
            ),
            (
                format!("<m:eqArr>{}{}</m:eqArr>", e(&mr("x=1")), e(&mr("y=2"))),
                r"\begin{aligned}x=1 \\ y=2\end{aligned}",
            ),
            // A prime superscript reads as the apostrophes that build it.
            (
                format!("<m:sSup>{}<m:sup>{}</m:sup></m:sSup>", e(&mr("f")), mr("′")),
                "f'",
            ),
            (
                format!("<m:sSup>{}<m:sup>{}</m:sup></m:sSup>", e(&mr("g")), mr("″")),
                "g''",
            ),
            (
                format!("<m:sSup>{}<m:sup>{}</m:sup></m:sSup>", e(&mr("h")), mr("′2")),
                "h^{′2}",
            ),
            // Word aligns an array's lines at each `&` and draws none.
            (
                format!("<m:eqArr>{}{}</m:eqArr>", e(&mr("a&amp;=1")), e(&mr("bc&amp;=2"))),
                r"\begin{aligned}a&=1 \\ bc&=2\end{aligned}",
            ),
            // A matrix inside keeps its literal `\&`.
            (
                format!(
                    "<m:eqArr>{}{}</m:eqArr>",
                    e(&format!(
                        "<m:m><m:mr>{}{}</m:mr></m:m>{}",
                        e(&mr("R&amp;D")),
                        e(&mr("x")),
                        mr("&amp;=1")
                    )),
                    e(&mr("q&amp;=2"))
                ),
                r"\begin{aligned}\begin{matrix}R\&D & x\end{matrix}&=1 \\ q&=2\end{aligned}",
            ),
            (
                format!(
                    "<m:sSubSup>{}<m:sub>{}</m:sub><m:sup>{}</m:sup></m:sSubSup>",
                    e(&mr("x")),
                    mr("i"),
                    mr("2")
                ),
                "x_i^2",
            ),
            (
                format!(
                    "<m:sPre><m:sub>{}</m:sub><m:sup>{}</m:sup>{}</m:sPre>",
                    mr("a"),
                    mr("b"),
                    e(&mr("X"))
                ),
                r"\prescript{a}{b}{X}",
            ),
        ];
        for (omml, latex) in cases {
            let body = format!(
                r#"<w:p><m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">{omml}</m:oMath></w:p>"#
            );
            assert_eq!(md(&docx(&body, &[])), format!("${latex}$\n"), "{omml}");
        }
    }

    #[test]
    fn footnotes_images_and_fields() {
        let body = [
            p(
                "",
                r#"<w:r><w:t>Claim</w:t></w:r><w:r><w:footnoteReference w:id="2"/></w:r>"#,
            ),
            p(
                "",
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> HYPERLINK "https://f.example" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>field link</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            p(
                "",
                r#"<w:r><w:drawing><wp:inline xmlns:wp="wp"><wp:docPr id="1" name="Picture 1" descr="A chart of sales"/><a:graphic xmlns:a="a"><a:graphicData><pic:pic xmlns:pic="p"><pic:blipFill><a:blip r:embed="rIdImg"/></pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#,
            ),
            p("", r#"<w:r><w:drawing><wp:inline xmlns:wp="wp"><wp:docPr id="2" name="Picture 2"/></wp:inline></w:drawing></w:r>"#),
        ]
        .concat();
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:id="2"><w:p><w:r><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> Source: survey.</w:t></w:r></w:p></w:footnote></w:footnotes>"#
        );
        let bytes = docx(
            &body,
            &[
                ("word/footnotes.xml", &footnotes),
                (
                    "word/_rels/document.xml.rels",
                    r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdImg" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/></Relationships>"#,
                ),
            ],
        );
        assert_eq!(
            md(&bytes),
            "Claim[^1]\n\n[field link](https://f.example)\n\n![A chart of sales](image1.png)\n\n[^1]: Source: survey.\n"
        );
    }

    #[test]
    fn a_link_to_a_bookmark_is_a_markdown_link_and_its_bookmark_an_anchor() {
        // `w:hyperlink w:anchor` and a `HYPERLINK \l` field both lead to a
        // bookmark: standard Markdown `[t](#name)`, the tooltip its title.
        // The bookmark a link or a reference field names prints where it
        // starts, `<a id="name"></a>`; one nothing names prints nothing.
        let body = format!(
            "{}{}{}{}{}",
            p(
                "",
                r#"<w:bookmarkStart w:id="0" w:name="exec_summary"/><w:r><w:t>Summary</w:t></w:r><w:bookmarkEnd w:id="0"/><w:bookmarkStart w:id="1" w:name="unused"/><w:bookmarkEnd w:id="1"/>"#
            ),
            p(
                "",
                r#"<w:r><w:t xml:space="preserve">Back to </w:t></w:r><w:hyperlink w:anchor="exec_summary" w:tooltip="Go up"><w:r><w:t>the summary</w:t></w:r></w:hyperlink><w:r><w:t>.</w:t></w:r>"#
            ),
            p(
                "",
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> HYPERLINK \l "exec_summary" \o "Say \"up\"" \h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Jump</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#
            ),
            p(
                "",
                r#"<w:bookmarkStart w:id="2" w:name="_Ref9"/><w:r><w:t>Target</w:t></w:r><w:bookmarkEnd w:id="2"/>"#
            ),
            p(
                "",
                r#"<w:r><w:t xml:space="preserve">See page </w:t></w:r><w:fldSimple w:instr=" PAGEREF _Ref9 \h "><w:r><w:t>2</w:t></w:r></w:fldSimple><w:r><w:t xml:space="preserve"> and </w:t></w:r><w:fldSimple w:instr=" HYPERLINK &quot;https://x.example/a&quot; \l &quot;part&quot; \t &quot;_blank&quot; "><w:r><w:t>there</w:t></w:r></w:fldSimple>"#
            ),
        );
        assert_eq!(
            md(&docx(&body, &[])),
            "<a id=\"exec_summary\"></a>Summary\n\nBack to [the summary](#exec_summary \"Go up\").\n\n\
             [Jump](#exec_summary \"Say \\\"up\\\"\")\n\n<a id=\"_Ref9\"></a>Target\n\n\
             See page 2 and [there](https://x.example/a#part)\n"
        );
        assert_eq!(
            hyperlink_instruction(r#" HYPERLINK \l "bookmark" "#),
            Some("#bookmark".to_string())
        );
        assert_eq!(hyperlink_instruction(" HYPERLINK "), None);
    }

    #[test]
    fn a_cross_reference_target_prints_its_anchor_with_nothing_naming_it() {
        // Word names its cross-reference targets `_Ref…`. The plain view
        // prints a REF field as its text, so the file it writes back has
        // the bookmark and no field: the anchor still prints, or a second
        // read would lose it. Other names nothing points at print nothing.
        let body = p(
            "",
            r#"<w:bookmarkStart w:id="2" w:name="_Ref100"/><w:r><w:t>Target</w:t></w:r><w:bookmarkEnd w:id="2"/><w:bookmarkStart w:id="3" w:name="_Hlk7"/><w:bookmarkEnd w:id="3"/>"#,
        );
        assert_eq!(md(&docx(&body, &[])), "<a id=\"_Ref100\"></a>Target\n");
    }

    #[test]
    fn image_names_keep_delimiter_characters_out_of_the_markup() {
        let image = r#"<w:r><w:drawing><wp:inline xmlns:wp="wp"><wp:docPr id="1" name="P" descr="x"/><a:graphic xmlns:a="a"><a:graphicData><pic:pic xmlns:pic="p"><pic:blipFill><a:blip r:embed="rIdImg"/></pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#;
        let bytes = docx(
            &p("", &format!("<w:del>{image}</w:del>")),
            &[(
                "word/_rels/document.xml.rels",
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdImg" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/a--}~&gt;{b}&lt;.png"/></Relationships>"#,
            )],
        );
        // `{`, `}`, `<` and `>` may not appear in a URI; encoded, they cannot form a
        // delimiter, so the destination reaches the Markdown unchanged.
        assert_eq!(md(&bytes), "{--![x](a--%7D~%3E%7Bb%7D%3C.png)--}\n");
    }

    #[test]
    fn tracked_changes_become_critic_markup_without_losing_formatting() {
        let body = [
            p(
                "",
                r#"<w:r><w:t xml:space="preserve">Keep </w:t></w:r><w:ins w:id="2"><w:r><w:rPr><w:b/></w:rPr><w:t>new</w:t></w:r></w:ins><w:del w:id="1"><w:r><w:rPr><w:i/></w:rPr><w:delText>old</w:delText></w:r></w:del><w:r><w:t>.</w:t></w:r>"#,
            ),
            p(
                "",
                r#"<w:moveTo w:id="3"><w:r><w:t>destination</w:t></w:r></w:moveTo><w:r><w:t xml:space="preserve"> / </w:t></w:r><w:moveFrom w:id="3"><w:r><w:delText>source</w:delText></w:r></w:moveFrom>"#,
            ),
        ]
        .concat();

        // An insertion then a deletion is a replacement, as Word Compare
        // writes it: CriticMarkup's substitution, as pandiff writes it.
        assert_eq!(
            md(&docx(&body, &[])),
            "Keep {~~_old_~>**new**~~}{>>r1+2<<}.\n\n{++destination++}{>>r3<<} / {--source--}{>>r3<<}\n"
        );
    }

    #[test]
    fn tracked_whole_paragraphs_keep_markdown_block_structure() {
        let body = format!(
            r#"<w:ins w:id="1">{}</w:ins><w:del w:id="2">{}</w:del>"#,
            p("Heading2", &r("Added heading")),
            p("", &r("Deleted paragraph")),
        );

        assert_eq!(
            md(&docx(&body, &[])),
            "## {++Added heading++}{>>r1<<}\n\n{--Deleted paragraph--}{>>r2<<}\n"
        );
    }

    #[test]
    fn tracked_table_rows_and_cells_keep_the_table_valid() {
        let body = format!(
            r#"<w:tbl><w:tr><w:tc>{}</w:tc><w:tc>{}</w:tc></w:tr><w:tr><w:trPr><w:del/></w:trPr><w:tc>{}</w:tc><w:tc><w:tcPr><w:cellIns/></w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
            p("", &r("Before")),
            p("", &r("After")),
            p("", &r("old")),
            p("", &r("new")),
        );

        // The inserted cell sits in a deleted row: accepting removes the row and
        // rejecting removes the cell, so the text is gone either way.
        assert_eq!(
            md(&docx(&body, &[])),
            "|Before|After|\n|-|-|\n|{--old--}|{++new++}|\n"
        );
    }

    // Tracked-change building blocks, written the way Word writes them.
    fn ins(content: &str) -> String {
        format!(
            r#"<w:ins w:id="90" w:author="Ana" w:date="2026-01-02T03:04:00Z">{content}</w:ins>"#
        )
    }

    fn del(content: &str) -> String {
        format!(
            r#"<w:del w:id="91" w:author="Ana" w:date="2026-01-02T03:04:00Z">{content}</w:del>"#
        )
    }

    fn dr(text: &str) -> String {
        format!(r#"<w:r><w:delText xml:space="preserve">{text}</w:delText></w:r>"#)
    }

    /// A paragraph whose mark (the break after it) carries `mark` (`ins`, `del`,
    /// `moveFrom`, `moveTo`, or several), with an optional style.
    fn pm(style: &str, marks: &[&str], runs: &str) -> String {
        let style = if style.is_empty() {
            String::new()
        } else {
            format!(r#"<w:pStyle w:val="{style}"/>"#)
        };
        let marks: String = marks
            .iter()
            .map(|m| format!(r#"<w:{m} w:id="92" w:author="Ana" w:date="2026-01-02T03:04:00Z"/>"#))
            .collect();
        format!("<w:p><w:pPr>{style}<w:rPr>{marks}</w:rPr></w:pPr>{runs}</w:p>")
    }

    fn text_box(inner: &str) -> String {
        format!(
            r#"<w:r><w:drawing><wp:anchor xmlns:wp="wp"><wp:docPr id="7" name="Text Box 7"/><a:graphic xmlns:a="a"><a:graphicData><wps:wsp xmlns:wps="wps"><wps:txbx><w:txbxContent>{inner}</w:txbxContent></wps:txbx></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r>"#
        )
    }

    #[test]
    fn review_deleted_row_with_deleted_runs_is_marked_once() {
        // Word deletes a row by marking the row, every run and every paragraph mark.
        let body = format!(
            r#"<w:tbl><w:tr><w:tc>{}</w:tc></w:tr><w:tr><w:trPr><w:del w:id="1" w:author="Ana" w:date="2026-01-02T03:04:00Z"/></w:trPr><w:tc>{}</w:tc></w:tr></w:tbl>"#,
            p("", &r("Head")),
            pm("", &["del"], &del(&dr("old"))),
        );
        assert_eq!(
            md(&docx(&body, &[])),
            "|Head|\n|-|\n|{--old--}{>>r1 Ana (2026-01-02T03:04:00Z)<<}|\n"
        );
    }

    #[test]
    fn review_text_box_inside_a_tracked_change_keeps_the_change() {
        let body = [
            p("", &del(&text_box(&p("", &r("Deleted box"))))),
            p(
                "",
                &format!("{}{}", r("Keep"), ins(&text_box(&p("", &r("Added box"))))),
            ),
        ]
        .concat();
        // The deleted box leaves no empty `{----}` paragraph behind.
        assert_eq!(
            md(&docx(&body, &[])),
            "{--Deleted box--}{>>r91 Ana (2026-01-02T03:04:00Z)<<}\n\nKeep\n\n{++Added box++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n"
        );
    }

    #[test]
    fn review_tracked_paragraph_marks_mark_the_break() {
        let deleted = [pm("", &["del"], &r("A")), p("", &r("B"))].concat();
        assert_eq!(
            md(&docx(&deleted, &[])),
            "A{--\n\n--}{>>r92 Ana (2026-01-02T03:04:00Z)<<}B\n"
        );
        let inserted = [pm("", &["ins"], &r("A")), p("", &r("B"))].concat();
        assert_eq!(
            md(&docx(&inserted, &[])),
            "A{++\n\n++}{>>r92 Ana (2026-01-02T03:04:00Z)<<}B\n"
        );
        // A whole deleted paragraph: its text and its break are one
        // deletion, whose note names both marks.
        let whole = [
            pm("", &["del"], &del(&dr("Deleted paragraph"))),
            p("", &r("Next")),
        ]
        .concat();
        assert_eq!(
            md(&docx(&whole, &[])),
            "{--Deleted paragraph\n\n--}{>>r91+92 Ana (2026-01-02T03:04:00Z)<<}Next\n"
        );
    }

    #[test]
    fn review_footnote_revisions_are_marked() {
        let body = p(
            "",
            r#"<w:r><w:t>Claim</w:t></w:r><w:r><w:footnoteReference w:id="2"/></w:r>"#,
        );
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="2">{}{}</w:footnote></w:footnotes>"#,
            pm(
                "",
                &["del"],
                &format!("{}{}", r("Source"), ins(&r(" updated")))
            ),
            p("", &r("page 4")),
        );
        assert_eq!(
            md(&docx(&body, &[("word/footnotes.xml", &footnotes)])),
            "Claim[^1]\n\n[^1]: Source{++ updated++}{>>footnotes.r90 Ana (2026-01-02T03:04:00Z)<<}{-- --}{>>footnotes.r92 Ana (2026-01-02T03:04:00Z)<<}page 4\n"
        );
    }

    /// `word/comments.xml` with one `w:comment` per entry: (id, author, date,
    /// paragraphs). An empty author or date leaves the attribute out.
    fn comments(entries: &[(&str, &str, &str, &str)]) -> String {
        let body: String = entries
            .iter()
            .map(|(id, author, date, paragraphs)| {
                let author = if author.is_empty() {
                    String::new()
                } else {
                    format!(r#" w:author="{author}""#)
                };
                let date = if date.is_empty() {
                    String::new()
                } else {
                    format!(r#" w:date="{date}""#)
                };
                format!(r#"<w:comment w:id="{id}"{author}{date} w:initials="X">{paragraphs}</w:comment>"#)
            })
            .collect();
        format!(r#"<w:comments {W}>{body}</w:comments>"#)
    }

    fn start(id: &str) -> String {
        format!(r#"<w:commentRangeStart w:id="{id}"/>"#)
    }

    fn end(id: &str) -> String {
        format!(r#"<w:commentRangeEnd w:id="{id}"/>"#)
    }

    /// The reference run Word writes right after a comment's range end.
    fn reference(id: &str) -> String {
        format!(
            r#"<w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="{id}"/></w:r>"#
        )
    }

    fn with_comments(body: &str, xml: &str) -> String {
        md(&docx(body, &[("word/comments.xml", xml)]))
    }

    const BILL: (&str, &str) = ("Bill Winter", "2024-04-08T10:32:00Z");

    #[test]
    fn comment_highlights_its_range_and_names_author_and_date_as_word_stores_them() {
        let body = p(
            "",
            &format!(
                "{}{}{}{}{}{}",
                r("Truth is "),
                start("0"),
                r("stranger than fiction"),
                end("0"),
                reference("0"),
                r(".")
            ),
        );
        let xml = comments(&[("0", BILL.0, BILL.1, &p("", &r("true")))]);
        assert_eq!(
            with_comments(&body, &xml),
            "Truth is {==stranger than fiction==}{>>Bill Winter (2024-04-08T10:32:00Z): true<<}.\n"
        );
    }

    #[test]
    fn comment_date_is_copied_verbatim_whatever_its_form() {
        // Fractional seconds and offsets are valid xsd:dateTime; they are kept.
        for date in [
            "2024-04-08T10:32:17.123Z",
            "2024-04-08T10:32:00+02:00",
            "2024-04-08T10:32:00",
        ] {
            let body = p("", &format!("{}{}", r("a"), reference("1")));
            let xml = comments(&[("1", "Ana", date, &p("", &r("n")))]);
            assert_eq!(
                with_comments(&body, &xml),
                format!("a{{>>Ana ({date}): n<<}}\n")
            );
        }
    }

    #[test]
    fn comment_note_leaves_out_what_is_missing() {
        assert_eq!(
            comment_note(Some("Ana"), Some("2026-01-02T03:04:00Z"), "hi"),
            "Ana (2026-01-02T03:04:00Z): hi"
        );
        assert_eq!(comment_note(Some("Ana"), None, "hi"), "Ana: hi");
        assert_eq!(
            comment_note(None, Some("2026-01-02T03:04:00Z"), "hi"),
            "(2026-01-02T03:04:00Z): hi"
        );
        assert_eq!(comment_note(Some("  "), Some(" "), "hi"), "hi");
        assert_eq!(
            comment_note(Some("Ana"), Some("2026-01-02T03:04:00Z"), ""),
            "Ana (2026-01-02T03:04:00Z)"
        );
        assert_eq!(comment_note(None, None, ""), "");
        assert_eq!(comment_note(Some("A <<} B"), None, "x"), "A <<\\} B: x");
    }

    #[test]
    fn comment_without_a_range_is_a_plain_comment_at_its_reference() {
        let body = p(
            "",
            &format!(
                "{}{}{}",
                r("Lorem ipsum dolor sit amet."),
                reference("3"),
                r(" Next")
            ),
        );
        let xml = comments(&[("3", "Ana", "", &p("", &r("This is a comment")))]);
        assert_eq!(
            with_comments(&body, &xml),
            "Lorem ipsum dolor sit amet.{>>Ana: This is a comment<<} Next\n"
        );
    }

    #[test]
    fn replies_and_overlapping_ranges_share_one_highlight() {
        // Word writes a reply as its own comment over the same range.
        let body = p(
            "",
            &[
                start("0"),
                r("a"),
                start("1"),
                start("2"),
                r("b"),
                end("0"),
                reference("0"),
                r("c"),
                end("1"),
                reference("1"),
                end("2"),
                reference("2"),
            ]
            .concat(),
        );
        let xml = comments(&[
            ("0", "Ana", "2026-01-01T09:00:00Z", &p("", &r("first"))),
            ("1", "Bo", "2026-01-01T10:00:00Z", &p("", &r("second"))),
            ("2", "Ana", "2026-01-01T11:00:00Z", &p("", &r("reply"))),
        ]);
        assert_eq!(
            with_comments(&body, &xml),
            "{==abc==}{>>Ana (2026-01-01T09:00:00Z): first<<}{>>Bo (2026-01-01T10:00:00Z): second<<}{>>Ana (2026-01-01T11:00:00Z): reply<<}\n"
        );
    }

    #[test]
    fn comment_range_across_paragraphs_highlights_each_paragraph() {
        let body = [
            p("", &format!("{}{}{}", r("one "), start("5"), r("two"))),
            p("Heading1", &r("three")),
            p(
                "",
                &format!("{}{}{}{}", r("four"), end("5"), reference("5"), r(" five")),
            ),
        ]
        .concat();
        let xml = comments(&[("5", "Ana", "", &p("", &r("long")))]);
        assert_eq!(
            with_comments(&body, &xml),
            "one {==two==}\n\n# {==three==}\n\n{==four==}{>>Ana: long<<} five\n"
        );
    }

    #[test]
    fn comment_ranges_at_block_level_and_in_tables() {
        let body = format!(
            "{}{}<w:tbl><w:tr><w:tc>{}{}</w:tc></w:tr></w:tbl>{}",
            start("1"),
            p("", &r("para")),
            p("", &r("cell")),
            end("1"),
            p("", &format!("{}{}", r("after"), reference("1"))),
        );
        let xml = comments(&[("1", "Ana", "", &p("", &r("n")))]);
        assert_eq!(
            with_comments(&body, &xml),
            "{==para==}\n\n|{==cell==}|\n|-|\n\nafter{>>Ana: n<<}\n"
        );
    }

    #[test]
    fn comment_without_a_reference_gets_its_note_at_the_range_end() {
        let body = p(
            "",
            &format!("{}{}{}{}", start("4"), r("x"), end("4"), r("y")),
        );
        let xml = comments(&[("4", "Ana", "", &p("", &r("n")))]);
        assert_eq!(with_comments(&body, &xml), "{==x==}{>>Ana: n<<}y\n");
    }

    #[test]
    fn unknown_or_malformed_comments_are_ignored() {
        let body = p(
            "",
            &format!(
                "{}{}{}{}{}{}{}",
                start("9"),
                r("a"),
                end("9"),
                reference("9"),
                end("8"),
                r("b"),
                r#"<w:commentRangeStart/><w:r><w:commentReference/></w:r>"#,
            ),
        );
        // Without a comments part every anchor is ignored.
        assert_eq!(md(&docx(&body, &[])), "ab\n");
        // A comment without an id cannot be referenced; one never closed stays a
        // highlight to the end of its paragraph only.
        let xml = format!(
            r#"<w:comments {W}><w:comment w:author="Ana">{}</w:comment><w:comment w:id="8" w:author="Ana">{}</w:comment></w:comments>"#,
            p("", &r("x")),
            p("", &r("y"))
        );
        let open = [
            p("", &format!("{}{}", start("8"), r("open"))),
            p("", &r("still")),
        ]
        .concat();
        assert_eq!(with_comments(&body, &xml), "ab\n");
        assert_eq!(with_comments(&open, &xml), "{==open==}\n\n{==still==}\n");
        // A range start seen twice opens once.
        let twice = p(
            "",
            &format!(
                "{}{}{}{}{}",
                start("8"),
                start("8"),
                r("x"),
                end("8"),
                reference("8")
            ),
        );
        assert_eq!(with_comments(&twice, &xml), "{==x==}{>>Ana: y<<}\n");
    }

    #[test]
    fn comment_text_keeps_formatting_joins_paragraphs_and_escapes_delimiters() {
        let body = p("", &format!("{}{}", r("a"), reference("1")));
        let text = [
            p("", r#"<w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r><w:r><w:br/><w:t>line &lt;&lt;} end</w:t></w:r>"#),
            p("", ""),
            pm("", &["del"], &r("gone")),
            p("", &format!("{}{}", r("kept"), ins(&r(" new")))),
        ]
        .concat();
        let xml = comments(&[("1", "Ana", "", &text)]);
        assert_eq!(
            with_comments(&body, &xml),
            "a{>>Ana: **Bold** line <<\\} end gone{-- --}kept{++ new++}<<}\n"
        );
    }

    #[test]
    fn comments_meet_tracked_changes_without_breaking_either() {
        // Word wraps the reference run of a comment added with tracking on in
        // w:ins; the note must not become an insertion.
        let tracked = p(
            "",
            &format!(
                "{}{}{}{}",
                start("1"),
                r("text"),
                end("1"),
                ins(&reference("1"))
            ),
        );
        // A comment on deleted text, and a range that ends inside an insertion.
        let deleted = p(
            "",
            &format!(
                "{}{}{}{}",
                start("1"),
                del(&dr("old")),
                end("1"),
                reference("1")
            ),
        );
        let crossing = p(
            "",
            &format!(
                "{}{}{}{}",
                start("1"),
                r("a"),
                ins(&format!("{}{}{}", r("b"), end("1"), r("c"))),
                reference("1")
            ),
        );
        let xml = comments(&[("1", "Ana", "", &p("", &r("n")))]);
        assert_eq!(with_comments(&tracked, &xml), "{==text==}{>>Ana: n<<}\n");
        assert_eq!(
            with_comments(&deleted, &xml),
            "{=={--old--}{>>r91 Ana (2026-01-02T03:04:00Z)<<}==}{>>Ana: n<<}\n"
        );
        assert_eq!(
            with_comments(&crossing, &xml),
            "{==a{++b++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}==}{++c++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}{>>Ana: n<<}\n"
        );
    }

    #[test]
    fn comment_range_around_a_text_box_does_not_cover_the_box() {
        let body = p(
            "",
            &format!(
                "{}{}{}{}{}",
                start("1"),
                r("anchor"),
                text_box(&p("", &r("inside"))),
                end("1"),
                reference("1")
            ),
        );
        let xml = comments(&[("1", "Ana", "", &p("", &r("n")))]);
        assert_eq!(
            with_comments(&body, &xml),
            "{==anchor==}{>>Ana: n<<}\n\ninside\n"
        );
    }

    #[test]
    fn comment_part_is_found_through_the_document_relationships() {
        let body = p("", &format!("{}{}", r("a"), reference("1")));
        let xml = comments(&[("1", "Ana", "", &p("", &r("n")))]);
        let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="notes/remarks.xml"/></Relationships>"#;
        let bytes = docx(
            &body,
            &[
                ("word/_rels/document.xml.rels", rels),
                ("word/notes/remarks.xml", &xml),
            ],
        );
        assert_eq!(md(&bytes), "a{>>Ana: n<<}\n");
    }

    #[test]
    fn comments_in_footnotes_and_ranges_left_open_before_them() {
        let body = p(
            "",
            &format!(
                "{}{}{}",
                start("1"),
                r("Claim"),
                r#"<w:r><w:footnoteReference w:id="2"/></w:r>"#
            ),
        );
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="2">{}</w:footnote></w:footnotes>"#,
            p(
                "",
                &format!(
                    "{}{}{}{}",
                    start("2"),
                    r("Source"),
                    end("2"),
                    reference("2")
                )
            ),
        );
        let xml = comments(&[
            ("1", "Ana", "", &p("", &r("never closed"))),
            ("2", "Bo", "", &p("", &r("check"))),
        ]);
        assert_eq!(
            md(&docx(
                &body,
                &[
                    ("word/comments.xml", &xml),
                    ("word/footnotes.xml", &footnotes)
                ]
            )),
            "{==Claim[^1]==}\n\n[^1]: {==Source==}{>>Bo: check<<}\n"
        );
    }

    #[test]
    fn substitutions_merges_and_empty_changes() {
        let body = [
            // Replacement written insertion first still reads old ~> new.
            p("", &format!("{}{}{}", r("I really love "), ins(&r("font-styles")), del(&dr("fonts")))),
            // Word splits one insertion over several w:ins; they join.
            p("", &format!("{}{}{}", ins(&r("one ")), ins(&r("insertion")), r("."))),
            // Empty change elements vanish; an inserted space stays.
            p("", &format!("{}{}{}{}", r("a"), ins(""), del(&r("")), ins(&r(" ")))),
            // Inserted by one author and deleted by another: the insertion
            // held nothing else, so the note names it after ` on `.
            p("", &format!("{}{}", r("x"), ins(&del(&dr("y"))))),
            // Moves are an insertion and a deletion; the inserted side then
            // the deleted one read as a substitution.
            p("", r#"<w:moveTo w:id="4"><w:r><w:t>there</w:t></w:r></w:moveTo><w:moveFrom w:id="3"><w:r><w:delText>here</w:delText></w:r></w:moveFrom>"#),
        ]
        .concat();
        assert_eq!(
            md(&docx(&body, &[])),
            "I really love {~~fonts~>font-styles~~}{>>r91+90 Ana (2026-01-02T03:04:00Z)<<}\n\n{++one insertion++}{>>r90+90 Ana (2026-01-02T03:04:00Z)<<}.\n\na{++ ++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n\nx{--y--}{>>r91 Ana (2026-01-02T03:04:00Z) on r90 Ana (2026-01-02T03:04:00Z)<<}\n\n{~~here~>there~~}{>>r3+4<<}\n"
        );
    }

    #[test]
    fn delimiters_in_document_text_do_not_open_spans() {
        // With a tracked change in the document, delimiter-like text is escaped.
        let body = p("", &r("x {++ y ++} ~> z")) + &p("", &ins(&r("new")));
        assert_eq!(
            md(&docx(&body, &[])),
            "x {\\++ y ++\\} \\~\\> z\n\n{++new++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n"
        );
        // Without one the text is escaped all the same (#519): a later
        // convert cannot read it as changes.
        let body = p("", &r("x {++ y ++} ~> z"));
        assert_eq!(md(&docx(&body, &[])), "x {\\++ y ++\\} \\~\\> z\n");
    }

    #[test]
    fn tracked_changes_with_links_fields_and_hidden_runs() {
        let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://x.test" TargetMode="External"/></Relationships>"#;
        let body = [
            p("", &format!(r#"<w:hyperlink r:id="rId9">{}{}</w:hyperlink>"#, r("keep "), ins(&r("new")))),
            p("", &ins(r#"<w:hyperlink r:id="rId9"><w:r><w:t>added link</w:t></w:r></w:hyperlink>"#)),
            // A deleted hyperlink field keeps its target through w:delInstrText.
            p("", &del(r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText> HYPERLINK "https://f.test" </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:delText>old link</w:delText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#)),
            // Hidden text is not shown in Word, tracked or not.
            p("", &format!("{}{}", r("seen"), ins(r#"<w:r><w:rPr><w:vanish/></w:rPr><w:t>hidden</w:t></w:r>"#))),
        ]
        .concat();
        assert_eq!(
            md(&docx(&body, &[("word/_rels/document.xml.rels", rels)])),
            "[keep](https://x.test) {++[new](https://x.test)++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n\n{++[added link](https://x.test)++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n\n{--[old link](https://f.test)--}{>>r91 Ana (2026-01-02T03:04:00Z)<<}\n\nseen\n"
        );
    }

    #[test]
    fn tracked_breaks_keep_headings_lists_and_blocks_valid() {
        let numbering = format!(
            r#"<w:numbering {W}><w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/></w:lvl></w:abstractNum><w:num w:numId="5"><w:abstractNumId w:val="1"/></w:num></w:numbering>"#
        );
        let item = |marks: &[&str], runs: &str| {
            let marks: String = marks.iter().map(|m| format!("<w:{m}/>")).collect();
            format!(
                r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="5"/></w:numPr><w:rPr>{marks}</w:rPr></w:pPr>{runs}</w:p>"#
            )
        };
        let body = [
            // Before a heading the break closes after `## `.
            pm("", &["ins"], &r("A")),
            p("Heading2", &r("B")),
            // Before a list item it closes after the marker; a deleted item keeps
            // its number, as Word shows it.
            item(&["moveFrom"], &r("one")),
            item(&[], &del(&dr("two"))),
            item(&["moveTo"], &r("three")),
            item(&[], &r("four")),
            // A blank paragraph keeps its own break; a table cannot join a paragraph.
            pm("", &["del"], &r("C")),
            p("", ""),
            pm("", &["ins", "del"], &r("D")),
            p("", &r("E")),
            pm("", &["del"], &r("F")),
            format!(
                "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
                p("", &r("G"))
            ),
            // A paragraph holding a text box gives its break to the box.
            pm(
                "",
                &["del"],
                &format!("{}{}", r("H"), text_box(&p("", &r("box")))),
            ),
            p("", &r("I")),
            // The last paragraph's mark has no break after it.
            pm("", &["del"], &r("J")),
        ]
        .concat();
        assert_eq!(
            md(&docx(&body, &[("word/numbering.xml", &numbering)])),
            "A{++\n\n## ++}{>>r92 Ana (2026-01-02T03:04:00Z)<<}B\n\n1. one{--\n2. two--}{>>r91 Ana (2026-01-02T03:04:00Z)<<}\n3. three{++\n4. ++}four\n\nC\n\nD{--\n\n--}{>>r92 Ana (2026-01-02T03:04:00Z)<<}E\n\nF\n\n|G|\n|-|\n\nH\n\nbox\n\nI\n\nJ\n"
        );
    }

    #[test]
    fn tracked_cells_rows_and_nested_tables() {
        let body = format!(
            r#"<w:tbl><w:tr><w:tc>{}{}{}</w:tc><w:tc>{}</w:tc></w:tr><w:tr><w:trPr><w:ins/></w:trPr><w:tc><w:tbl><w:tr><w:trPr><w:del/></w:trPr><w:tc>{}</w:tc><w:tc><w:tcPr><w:cellDel/></w:tcPr>{}</w:tc></w:tr></w:tbl></w:tc><w:tc>{}</w:tc></w:tr></w:tbl>"#,
            // Paragraph marks inside a cell.
            pm("", &["del"], &r("a")),
            pm("", &["ins"], &r("b")),
            pm("", &["del"], &r("c")),
            ins(&p("", &r("block ins"))),
            p("", &r("n1")),
            p("", &r("n2")),
            del(&p("", &r("wrapped"))),
        );
        assert_eq!(
            md(&docx(&body, &[])),
            "|a{--<br>--}{>>r92 Ana (2026-01-02T03:04:00Z)<<}b{++<br>++}{>>r92 Ana (2026-01-02T03:04:00Z)<<}c|{++block ins++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}|\n|-|-|\n|{--n1; n2--}|{--wrapped--}{>>r91 Ana (2026-01-02T03:04:00Z)<<}|\n"
        );
    }

    #[test]
    fn tracked_blocks_inside_content_controls() {
        let body = ins(&format!(
            "<w:sdt><w:sdtContent>{}</w:sdtContent></w:sdt><w:customXml>{}</w:customXml>",
            p("", &r("in control")),
            p("", &r("in custom xml")),
        ));
        assert_eq!(
            md(&docx(&body, &[])),
            "{++in control++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n\n{++in custom xml++}{>>r90 Ana (2026-01-02T03:04:00Z)<<}\n"
        );
    }

    #[test]
    fn tracked_changes_inside_inline_wrappers_and_styled_breaks() {
        let body = [
            // Smart tags and inline content controls wrap tracked runs too.
            p(
                "",
                &format!(
                    "<w:smartTag>{}</w:smartTag><w:sdt><w:sdtContent>{}</w:sdtContent></w:sdt>",
                    ins(&r("tagged")),
                    del(&dr(" boxed"))
                ),
            ),
            // A heading whose own mark is deleted joins the next paragraph.
            pm("Heading1", &["del"], &r("Title")),
            p("", &r("Body")),
            // A note reference without an id has nothing to point at.
            p(
                "",
                &format!("{}{}", r("x"), ins(r#"<w:r><w:footnoteReference/></w:r>"#)),
            ),
        ]
        .concat();
        assert_eq!(
            md(&docx(&body, &[])),
            "{~~ boxed~>tagged~~}{>>r91+90 Ana (2026-01-02T03:04:00Z)<<}\n\n# Title{--\n\n--}{>>r92 Ana (2026-01-02T03:04:00Z)<<}Body\n\nx\n"
        );
    }

    #[test]
    fn comment_without_an_author_shows_only_its_date() {
        let body = p("", &format!("{}{}", r("a"), reference("1")));
        let xml = comments(&[("1", "", "2026-01-02T03:04:00Z", &p("", &r("anonymous")))]);
        assert_eq!(
            with_comments(&body, &xml),
            "a{>>(2026-01-02T03:04:00Z): anonymous<<}\n"
        );
    }

    #[test]
    fn text_boxes_and_content_controls_inside_tracked_cells() {
        let body = format!(
            r#"<w:tbl><w:tr><w:trPr><w:del/></w:trPr><w:tc>{}</w:tc><w:tc><w:sdt><w:sdtContent>{}</w:sdtContent></w:sdt></w:tc></w:tr></w:tbl>"#,
            // The box's own paragraph mark cannot join the box text.
            pm(
                "",
                &["del"],
                &format!("{}{}", r("cell"), text_box(&p("", &r("box"))))
            ),
            p("", &r("control")),
        );
        assert_eq!(
            md(&docx(&body, &[])),
            "|{--cell<br>box--}|{--control--}|\n|-|-|\n"
        );
    }

    #[test]
    fn deleted_paragraph_mark_preserves_text_and_marks_only_the_separator() {
        let body = r#"<w:p><w:pPr><w:rPr><w:del w:id="1"/></w:rPr></w:pPr><w:r><w:t>First</w:t></w:r></w:p>
<w:p><w:r><w:t>Second</w:t></w:r></w:p>
<w:p><w:r><w:t>Third</w:t></w:r></w:p>"#;
        assert_eq!(
            md(&docx(body, &[])),
            "First{--\n\n--}{>>r1<<}Second\n\nThird\n"
        );
    }

    #[test]
    fn deleted_text_boxes_keep_block_structure_without_an_empty_paragraph() {
        let content = r#"<w:txbxContent>
<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Heading</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:numId w:val="1"/></w:numPr></w:pPr><w:del><w:r><w:delText>Item</w:delText></w:r></w:del></w:p>
<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
</w:txbxContent>"#;
        let numbering = format!(
            r#"<w:numbering {W}><w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="1"/></w:num></w:numbering>"#
        );
        let expected = "## {--Heading--}\n\n- {--Item--}\n\n|{--Cell--}|\n|-|\n";
        for drawing in ["drawing", "pict", "object"] {
            let run = format!("<w:r><w:{drawing}>{content}</w:{drawing}></w:r>");
            for body in [
                p("", &format!("<w:del>{run}</w:del>")),
                format!("<w:del>{}</w:del>", p("", &run)),
                p(
                    "",
                    &format!(
                        r#"<w:del><w:hyperlink w:anchor="bookmark"><w:sdt><w:sdtContent>{run}</w:sdtContent></w:sdt></w:hyperlink></w:del>"#
                    ),
                ),
            ] {
                assert_eq!(
                    md(&docx(&body, &[("word/numbering.xml", &numbering)])),
                    expected
                );
            }
        }
        let body = p(
            "",
            &format!(
                "{}<w:del><w:r><w:delText>Old</w:delText><w:drawing>{content}</w:drawing></w:r></w:del>",
                r("Keep ")
            ),
        );
        assert_eq!(
            md(&docx(&body, &[("word/numbering.xml", &numbering)])),
            format!("Keep {{--Old--}}\n\n{expected}")
        );
    }

    #[test]
    fn deleted_row_with_inline_deletion_has_no_nested_delimiters() {
        let body = r#"<w:tbl>
<w:tr><w:tc><w:p><w:r><w:t>Header</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:trPr><w:del w:id="1"/></w:trPr><w:tc><w:p>
<w:r><w:t xml:space="preserve">Before </w:t></w:r>
<w:del w:id="2"><w:r><w:rPr><w:b/></w:rPr><w:delText>old</w:delText></w:r></w:del>
<w:r><w:t xml:space="preserve"> after</w:t></w:r>
</w:p></w:tc></w:tr></w:tbl>"#;
        assert_eq!(
            md(&docx(body, &[])),
            "|Header|\n|-|\n|{--Before **old** after--}{>>r1<<}|\n"
        );
    }

    /// A revision element with an explicit author and date (either may be empty
    /// to leave the attribute out).
    fn by(tag: &str, author: &str, date: &str, content: &str) -> String {
        let author = if author.is_empty() {
            String::new()
        } else {
            format!(r#" w:author="{author}""#)
        };
        let date = if date.is_empty() {
            String::new()
        } else {
            format!(r#" w:date="{date}""#)
        };
        format!(r#"<w:{tag} w:id="7"{author}{date}>{content}</w:{tag}>"#)
    }

    #[test]
    fn every_tracked_change_names_its_author_and_date_as_word_stores_them() {
        let t1 = "2026-09-29T14:05:00Z";
        let t2 = "2026-09-29T15:10:00Z";
        let body = [
            // One person's replacement: one note after the substitution.
            p(
                "",
                &format!(
                    "{}{}",
                    by("ins", "Ana Lima", t1, &r("styles")),
                    by("del", "Ana Lima", t1, &dr("fonts"))
                ),
            ),
            // Two people's: both notes, old side first.
            p(
                "",
                &format!(
                    "{}{}",
                    by("ins", "Bo Chen", t2, &r("new")),
                    by("del", "Ana Lima", t1, &dr("old"))
                ),
            ),
            // The same author at two times stays two changes.
            p(
                "",
                &format!(
                    "{}{}",
                    by("ins", "Ana Lima", t1, &r("a")),
                    by("ins", "Ana Lima", t2, &r("b"))
                ),
            ),
            // Moves carry their own attribution.
            p("", &by("moveTo", "Bo Chen", t2, &r("moved here"))),
            // Missing date, missing author, neither, and a name that needs escaping.
            p(
                "",
                &format!(
                    "{}{}{}{}",
                    by("ins", "Ana Lima", "", &r("w")),
                    by("del", "", t1, &dr("x")),
                    by("ins", "", "", &r("y")),
                    by("ins", "A <<} B", t1, &r("z")),
                ),
            ),
        ]
        .concat();
        assert_eq!(
            md(&docx(&body, &[])),
            "{~~fonts~>styles~~}{>>r7+7 Ana Lima (2026-09-29T14:05:00Z)<<}\n\n\
             {~~old~>new~~}{>>r7 Ana Lima (2026-09-29T14:05:00Z)<<}{>>r7 Bo Chen (2026-09-29T15:10:00Z)<<}\n\n\
             {++a++}{>>r7 Ana Lima (2026-09-29T14:05:00Z)<<}{++b++}{>>r7 Ana Lima (2026-09-29T15:10:00Z)<<}\n\n\
             {++moved here++}{>>r7 Bo Chen (2026-09-29T15:10:00Z)<<}\n\n\
             {~~x~>w~~}{>>r7 (2026-09-29T14:05:00Z)<<}{>>r7 Ana Lima<<}{++y++}{>>r7<<}{++z++}{>>r7 A <<\\} B (2026-09-29T14:05:00Z)<<}\n"
        );
    }

    #[test]
    fn paragraph_breaks_rows_and_cells_carry_their_own_attribution() {
        let t1 = "2026-09-29T14:05:00Z";
        let t2 = "2026-09-29T15:10:00Z";
        let mark = |author: &str, date: &str| {
            format!(
                r#"<w:pPr><w:rPr><w:del w:id="8" w:author="{author}" w:date="{date}"/></w:rPr></w:pPr>"#
            )
        };
        let body = [
            // Text deleted by Ana, the break after it by Bo: two changes.
            format!("<w:p>{}{}</w:p>", mark("Bo Chen", t2), by("del", "Ana Lima", t1, &dr("A"))),
            p("", &r("Next")),
            // Text and break both deleted by Ana at t1: one change.
            format!("<w:p>{}{}</w:p>", mark("Ana Lima", t1), by("del", "Ana Lima", t1, &dr("B"))),
            p("", &by("del", "Ana Lima", t1, &dr("C"))),
            format!(
                r#"<w:tbl><w:tr><w:trPr><w:ins w:id="9" w:author="Bo Chen" w:date="{t2}"/></w:trPr><w:tc>{}</w:tc><w:tc><w:tcPr><w:cellDel w:id="10" w:author="Ana Lima" w:date="{t1}"/></w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
                p("", &r("row")),
                p("", &r("cell")),
            ),
        ]
        .concat();
        assert_eq!(
            md(&docx(&body, &[])),
            "{--A--}{>>r7 Ana Lima (2026-09-29T14:05:00Z)<<}{--\n\n--}{>>r8 Bo Chen (2026-09-29T15:10:00Z)<<}Next\n\n\
             {--B\n\nC--}{>>r7+8 Ana Lima (2026-09-29T14:05:00Z)<<}\n\n\
             |{++row++}{>>r9 Bo Chen (2026-09-29T15:10:00Z)<<}|{--cell--}{>>r10 Ana Lima (2026-09-29T14:05:00Z)<<}|\n|-|-|\n"
        );
    }

    #[test]
    fn footnotes_in_comment_text_are_numbered_where_the_comment_is() {
        let body = [
            p(
                "",
                r#"<w:r><w:t>B</w:t></w:r><w:r><w:footnoteReference w:id="2"/></w:r>"#,
            ),
            p("", &format!("{}{}", r("A"), reference("1"))),
        ]
        .concat();
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="2">{}</w:footnote><w:footnote w:id="3">{}</w:footnote></w:footnotes>"#,
            p("", &r("body note")),
            p("", &r("comment note")),
        );
        let xml = comments(&[(
            "1",
            "Ana",
            "",
            &p(
                "",
                r#"<w:r><w:t>see</w:t></w:r><w:r><w:footnoteReference w:id="3"/></w:r>"#,
            ),
        )]);
        assert_eq!(
            md(&docx(
                &body,
                &[
                    ("word/comments.xml", &xml),
                    ("word/footnotes.xml", &footnotes)
                ]
            )),
            "B[^1]\n\nA{>>Ana: see[^2]<<}\n\n[^1]: body note\n[^2]: comment note\n"
        );
    }

    #[test]
    fn a_range_ending_between_paragraphs_keeps_its_note() {
        let xml = comments(&[
            ("1", "Ana", "", &p("", &r("first"))),
            ("2", "Bo", "", &p("", &r("last"))),
        ]);
        let body = [
            p("", &format!("{}{}", start("1"), r("a"))),
            end("1"),
            p("", &r("b")),
            p("", &format!("{}{}", start("2"), r("c"))),
            end("2"),
        ]
        .concat();
        assert_eq!(
            with_comments(&body, &xml),
            "{==a==}\n\n{>>Ana: first<<}b\n\n{==c==}\n\n{>>Bo: last<<}\n"
        );
    }

    #[test]
    fn a_note_between_paragraph_and_table_stays_out_of_the_table() {
        let xml = comments(&[("1", "Ana", "", &p("", &r("first")))]);
        let table = |cell: &str| format!("<w:tbl><w:tr><w:tc>{cell}</w:tc></w:tr></w:tbl>");
        let body = [
            p("", &format!("{}{}", start("1"), r("a"))),
            end("1"),
            table(&p("", &r("cell"))),
        ]
        .concat();
        assert_eq!(
            with_comments(&body, &xml),
            "{==a==}\n\n{>>Ana: first<<}\n\n|cell|\n|-|\n"
        );
        // A deleted paragraph mark before the note keeps its markup.
        let body = [
            pm("", &["del"], &format!("{}{}", start("1"), r("a"))),
            end("1"),
            table(&p("", &r("cell"))),
        ]
        .concat();
        assert_eq!(
            with_comments(&body, &xml),
            "{==a==}{--\n\n--}{>>r92 Ana (2026-01-02T03:04:00Z)<<}{>>Ana: first<<}\n\n|cell|\n|-|\n"
        );
    }

    #[test]
    fn comment_text_keeps_text_boxes_without_nesting_comments() {
        let inner = [
            p(
                "",
                &by("ins", "Bo Chen", "2026-09-29T15:10:00Z", &r("boxed")),
            ),
            p("", &format!("{}{}", r("with a note"), reference("2"))),
        ]
        .concat();
        let text = p(
            "",
            &format!("{}{}{}", r("see "), text_box(&inner), reference("2")),
        );
        let xml = comments(&[
            ("1", "Ana", "", &text),
            ("2", "Bo", "", &p("", &r("inner"))),
        ]);
        let body = p(
            "",
            &format!("{}{}{}{}", start("1"), r("x"), end("1"), reference("1")),
        );
        assert_eq!(
            with_comments(&body, &xml),
            "{==x==}{>>Ana: see {++boxed++} with a note<<}\n"
        );
    }

    #[test]
    fn a_comment_used_twice_is_written_once_and_its_footnote_counted_once() {
        let body = [
            p("", &format!("{}{}", r("A"), reference("1"))),
            p("", &format!("{}{}", r("B"), reference("1"))),
        ]
        .concat();
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="3">{}</w:footnote></w:footnotes>"#,
            p("", &r("source")),
        );
        let xml = comments(&[(
            "1",
            "Ana",
            "",
            &p(
                "",
                r#"<w:r><w:t>see</w:t></w:r><w:r><w:footnoteReference w:id="3"/></w:r>"#,
            ),
        )]);
        assert_eq!(
            md(&docx(
                &body,
                &[
                    ("word/comments.xml", &xml),
                    ("word/footnotes.xml", &footnotes)
                ]
            )),
            "A{>>Ana: see[^1]<<}\n\nB{>>Ana: see[^1]<<}\n\n[^1]: source\n"
        );
    }

    fn resolved(bytes: &[u8], revisions: Revisions) -> String {
        let options = Options {
            revisions,
            ..Options::default()
        };
        convert(bytes, &options).unwrap().markdown
    }

    #[test]
    fn accept_and_reject_resolve_paragraphs_breaks_and_moves() {
        let body = [
            pm("", &["del"], &del(&r("Gone"))),
            p("", &r("Next")),
            p("", &format!("{}{}", ins(&r("New ")), r("text"))),
            p("", &by("moveFrom", "Bo", "", &r("Moved away"))),
            p("", &by("moveTo", "Bo", "", &r("Moved here"))),
            pm("", &["ins"], &r("One")),
            p("", &r("two")),
        ]
        .concat();
        let bytes = docx(&body, &[]);
        assert_eq!(
            resolved(&bytes, Revisions::Accept),
            "Next\n\nNew text\n\nMoved here\n\nOne\n\ntwo\n"
        );
        assert_eq!(
            resolved(&bytes, Revisions::Reject),
            "Gone\n\nNext\n\ntext\n\nMoved away\n\nOnetwo\n"
        );
    }

    #[test]
    fn a_break_that_goes_away_joins_across_empty_markers() {
        // Word writes bookmark and range ends between paragraphs; they hold no
        // content, so the paragraphs still join.
        for marker in [
            end("1"),
            r#"<w:bookmarkEnd w:id="0"/>"#.to_string(),
            r#"<w:moveToRangeEnd w:id="3"/>"#.to_string(),
        ] {
            let body = format!("{}{marker}{}", pm("", &["del"], &r("A")), p("", &r("B")));
            let bytes = docx(&body, &[]);
            assert_eq!(resolved(&bytes, Revisions::Accept), "AB\n", "{marker}");
            assert_eq!(resolved(&bytes, Revisions::Reject), "A\n\nB\n", "{marker}");
        }
        // A table between them is content: the break before it stays.
        let body = format!(
            "{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>{}",
            pm("", &["del"], &r("A")),
            p("", &r("cell")),
            p("", &r("B"))
        );
        assert_eq!(
            resolved(&docx(&body, &[]), Revisions::Accept),
            "A\n\n|cell|\n|-|\n\nB\n"
        );
    }

    #[test]
    fn a_joined_paragraph_takes_the_later_paragraphs_properties() {
        // Word keeps paragraph properties on the mark that ends the paragraph,
        // so deleting a break gives the joined text the second one's format.
        let heading =
            r#"<w:p><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#;
        let body = format!("{}{heading}", pm("", &["del"], &r("Intro ")));
        let bytes = docx(&body, &[]);
        assert_eq!(resolved(&bytes, Revisions::Accept), "# Intro Title\n");
        assert_eq!(resolved(&bytes, Revisions::Reject), "Intro\n\n# Title\n");
    }

    #[test]
    fn accept_and_reject_resolve_rows_cells_and_nested_tables() {
        let body = format!(
            r#"<w:tbl><w:tr><w:tc>{}{}{}</w:tc><w:tc>{}</w:tc></w:tr><w:tr><w:trPr><w:ins/></w:trPr><w:tc><w:tbl><w:tr><w:trPr><w:del/></w:trPr><w:tc>{}</w:tc><w:tc><w:tcPr><w:cellDel/></w:tcPr>{}</w:tc></w:tr></w:tbl></w:tc><w:tc>{}</w:tc></w:tr><w:tr><w:tc>{}</w:tc><w:tc><w:tcPr><w:cellDel/></w:tcPr>{}</w:tc><w:tc><w:tcPr><w:cellIns/></w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
            pm("", &["del"], &r("a")),
            pm("", &["ins"], &r("b")),
            pm("", &["del"], &r("c")),
            ins(&p("", &r("block ins"))),
            p("", &r("n1")),
            p("", &r("n2")),
            del(&p("", &r("wrapped"))),
            p("", &r("x")),
            p("", &r("gone")),
            p("", &r("added")),
        );
        let bytes = docx(&body, &[]);
        assert_eq!(
            resolved(&bytes, Revisions::Accept),
            "|ab<br>c|block ins|\n|-|-|\n|x|added|\n"
        );
        assert_eq!(
            resolved(&bytes, Revisions::Reject),
            "|a<br>bc||\n|-|-|\n|x|gone|\n"
        );
    }

    #[test]
    fn accept_and_reject_resolve_text_boxes_and_footnotes() {
        let body = [
            p("", &del(&text_box(&p("", &r("Deleted box"))))),
            p(
                "",
                &format!(
                    "{}{}{}",
                    r("Keep"),
                    ins(&text_box(&p("", &r("Added box")))),
                    r#"<w:r><w:footnoteReference w:id="2"/></w:r>"#
                ),
            ),
        ]
        .concat();
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="2">{}{}</w:footnote></w:footnotes>"#,
            pm(
                "",
                &["del"],
                &format!("{}{}", r("Source"), ins(&r(" updated")))
            ),
            p("", &r("page 4")),
        );
        let bytes = docx(&body, &[("word/footnotes.xml", &footnotes)]);
        assert_eq!(
            resolved(&bytes, Revisions::Accept),
            "Keep[^1]\n\nAdded box\n\n[^1]: Source updatedpage 4\n"
        );
        assert_eq!(
            resolved(&bytes, Revisions::Reject),
            "Deleted box\n\nKeep[^1]\n\n[^1]: Source page 4\n"
        );
    }

    #[test]
    fn reject_restores_formatting_that_a_tracked_change_replaced() {
        let run = r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="1" w:author="Ana"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>styled</w:t></w:r>"#;
        let bytes = docx(&p("", run), &[]);
        // A formatting change alone is not shown as markup.
        assert_eq!(md(&bytes), "**styled**\n");
        assert_eq!(resolved(&bytes, Revisions::Accept), "**styled**\n");
        assert_eq!(resolved(&bytes, Revisions::Reject), "_styled_\n");
        // LibreOffice records a change from plain text with no earlier `w:rPr`
        // inside: rejecting it leaves the text plain.
        let run = r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="1" w:author="Ana"></w:rPrChange></w:rPr><w:t>styled</w:t></w:r>"#;
        let bytes = docx(&p("", run), &[]);
        assert_eq!(resolved(&bytes, Revisions::Accept), "**styled**\n");
        assert_eq!(resolved(&bytes, Revisions::Reject), "styled\n");
    }

    #[test]
    fn accept_and_reject_leave_comments_out() {
        let body = p(
            "",
            &format!("{}{}{}{}", start("1"), r("noted"), end("1"), reference("1")),
        );
        let xml = comments(&[("1", "Ana", "", &p("", &r("why?")))]);
        let bytes = docx(&body, &[("word/comments.xml", &xml)]);
        assert!(md(&bytes).contains("{==noted==}"), "{}", md(&bytes));
        assert_eq!(resolved(&bytes, Revisions::Accept), "noted\n");
        assert_eq!(resolved(&bytes, Revisions::Reject), "noted\n");
    }

    #[test]
    fn a_note_whose_reference_is_changed_is_changed_too() {
        let reference = |id: &str| format!(r#"<w:r><w:footnoteReference w:id="{id}"/></w:r>"#);
        let body = [
            p("", &(r("Kept") + &reference("2"))),
            p("", &ins(&(r("Added") + &reference("3")))),
            p("", &del(&(dr("Gone") + &reference("4")))),
        ]
        .concat();
        let footnotes = format!(
            r#"<w:footnotes {W}><w:footnote w:id="2">{}</w:footnote><w:footnote w:id="3">{}</w:footnote><w:footnote w:id="4">{}</w:footnote></w:footnotes>"#,
            p("", &r("Plain note.")),
            p("", &r("Added note.")),
            p("", &format!("{}{}", r("Gone "), ins(&r("note")))),
        );
        let bytes = docx(&body, &[("word/footnotes.xml", &footnotes)]);
        let by = |id: &str| format!("{{>>{id} Ana (2026-01-02T03:04:00Z)<<}}");
        let (added, gone) = (by("r90"), by("r91"));
        assert_eq!(
            md(&bytes),
            format!(
                "Kept[^1]\n\n{{++Added[^2]++}}{added}\n\n{{--Gone[^3]--}}{gone}\n\n[^1]: Plain note.\n{{++[^2]: Added note.++}}{added}\n{{--[^3]: Gone --}}{gone}{{++note++}}{}\n",
                by("footnotes.r90")
            )
        );
        // Resolved first, the notes are numbered as they are left.
        assert_eq!(
            resolved(&bytes, Revisions::Reject),
            "Kept[^1]\n\nGone[^2]\n\n[^1]: Plain note.\n[^2]: Gone\n"
        );
    }

    #[test]
    fn malformed_input_is_invalid_not_panic() {
        assert!(matches!(
            convert(b"not a zip", &Options::default()),
            Err(ConvertError::Invalid(_))
        ));
        let no_doc = zip(&[("hello.txt", "hi")]);
        assert!(matches!(
            convert(&no_doc, &Options::default()),
            Err(ConvertError::Invalid(_))
        ));
        let broken = zip(&[("word/document.xml", "<w:document><w:body><w:p></w:body>")]);
        assert!(convert(&broken, &Options::default()).is_err());
        let truncated = docx(&p("", &r("ok")), &[]);
        assert!(convert(&truncated[..truncated.len() / 2], &Options::default()).is_err());
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod markdown_source_owner_boundary_tests {
    use super::*;

    fn docx(body: &str, extra: &[(&str, &str)]) -> Vec<u8> {
        let document = format!(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml"><w:body>{body}</w:body></w:document>"#
        );
        let mut parts = vec![("word/document.xml", document.as_str())];
        parts.extend_from_slice(extra);
        super::tests::zip(&parts)
    }

    #[test]
    fn style_title_outline_and_long_ancestry_follow_authored_nearest_properties() {
        let mut xml = String::from(
            "<w:styles xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'><w:style w:styleId='Report'><w:name w:val='Title'/></w:style><w:style w:styleId='Body'><w:name w:val='Body Text'/><w:pPr><w:outlineLvl w:val='9'/></w:pPr></w:style><w:style w:styleId='Deep'><w:name w:val='Custom Heading'/><w:pPr><w:outlineLvl w:val='8'/></w:pPr></w:style>",
        );
        for i in 0..17 {
            xml.push_str(&format!(
                "<w:style w:styleId='S{i}'><w:name w:val='Custom {i}'/>{}</w:style>",
                if i == 16 {
                    "<w:pPr><w:outlineLvl w:val='0'/></w:pPr><w:rPr><w:b/><w:i/></w:rPr>"
                        .to_string()
                } else {
                    format!("<w:basedOn w:val='S{}'/>", i + 1)
                }
            ));
        }
        xml.push_str("</w:styles>");
        let source = ooxml::parse_xml(xml.as_bytes()).unwrap();
        let styles = Styles::parse(&source);
        assert_eq!(styles.heading_level("Report"), Some(1));
        assert_eq!(styles.heading_level("Body"), None);
        assert_eq!(styles.heading_level("Deep"), Some(6));
        assert_eq!(styles.chain("S0").len(), 16);
        assert_eq!(styles.heading_level("S0"), None);
        assert_eq!(styles.emphasis("S0"), (None, None));
        assert_eq!(styles.heading_level("S1"), Some(1));
        assert_eq!(styles.emphasis("S1"), (Some(true), Some(true)));
        let body = "<w:p><w:pPr><w:pStyle w:val='Report'/></w:pPr><w:r><w:t>Report</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val='Report'/><w:outlineLvl w:val='9'/></w:pPr><w:r><w:t>Ordinary</w:t></w:r></w:p><w:p><w:pPr><w:outlineLvl w:val='8'/></w:pPr><w:r><w:t>Deep</w:t></w:r></w:p>";
        let bytes = docx(body, &[("word/styles.xml", &xml)]);
        let frozen = bytes.clone();
        assert_eq!(
            convert(&bytes, &Options::default()).unwrap().markdown,
            "# Report\n\nOrdinary\n\n###### Deep\n"
        );
        assert_eq!(bytes, frozen);
    }

    #[test]
    fn hidden_runs_cannot_leak_text_symbols_note_links_or_comment_markers() {
        let body = "<w:p><w:r><w:t>Visible</w:t><w:noBreakHyphen/></w:r><w:r><w:rPr><w:vanish/></w:rPr><w:t>hidden</w:t><w:tab/><w:br/><w:noBreakHyphen/><w:footnoteReference w:id='1'/><w:commentReference w:id='1'/><w:pict><v:rect style='width:20pt;height:20pt'/></w:pict></w:r></w:p>";
        let footnotes = "<w:footnotes xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'><w:footnote w:id='1'><w:p><w:r><w:t>hidden note</w:t></w:r></w:p></w:footnote></w:footnotes>";
        let comments = "<w:comments xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'><w:comment w:id='1' w:author='Ada'><w:p><w:r><w:t>hidden comment</w:t></w:r></w:p></w:comment></w:comments>";
        let bytes = docx(
            body,
            &[
                ("word/footnotes.xml", footnotes),
                ("word/comments.xml", comments),
            ],
        );
        let frozen = bytes.clone();
        for revisions in [Revisions::Markup, Revisions::Accept, Revisions::Reject] {
            let result = convert(
                &bytes,
                &Options {
                    revisions,
                    ..Options::default()
                },
            )
            .unwrap();
            assert_eq!(result.markdown, "Visible-\n");
            assert!(result.media.is_empty());
        }
        assert_eq!(bytes, frozen);
    }

    #[test]
    fn numbering_zero_and_out_of_range_overrides_do_not_change_source_order() {
        let numbering = "<w:numbering xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'><w:abstractNum w:abstractNumId='1'><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/></w:lvl></w:abstractNum><w:num w:numId='7'><w:abstractNumId w:val='1'/><w:lvlOverride w:ilvl='9'><w:startOverride w:val='99'/></w:lvlOverride></w:num></w:numbering>";
        let body = "<w:p><w:pPr><w:numPr><w:numId w:val='0'/></w:numPr></w:pPr><w:r><w:t>Unnumbered</w:t></w:r></w:p><w:p><w:pPr><w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr></w:pPr><w:r><w:t>First</w:t></w:r></w:p><w:p><w:pPr><w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr></w:pPr><w:r><w:t>Second</w:t></w:r></w:p>";
        let bytes = docx(body, &[("word/numbering.xml", numbering)]);
        let frozen = bytes.clone();
        assert_eq!(
            convert(&bytes, &Options::default()).unwrap().markdown,
            "Unnumbered\n\n1. First\n2. Second\n"
        );
        assert_eq!(bytes, frozen);
        assert_eq!(
            hyperlink_instruction(" HYPERLINK \\l bookmark "),
            Some("#bookmark".to_string())
        );
        assert_eq!(
            hyperlink_instruction(" HYPERLINK https://example.invalid "),
            Some("https://example.invalid".to_string())
        );
    }
}
