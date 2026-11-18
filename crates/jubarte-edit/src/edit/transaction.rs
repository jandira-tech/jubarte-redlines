// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The plan transaction's methods: resolving each operation against the
//! untouched source, locating anchors, checks across operations, applying
//! them to the copy, and the comments they author or carry.

use super::*;

impl<'p> Transaction<'p> {
    pub(super) fn start(source: &[u8], plan: &'p EditPlan) -> Result<Self, EditError> {
        Self::start_quoting(source, plan, PlanSyntax::V1)
    }

    #[doc(hidden)]
    pub fn start_quoting(
        source: &[u8],
        plan: &'p EditPlan,
        syntax: PlanSyntax,
    ) -> Result<Self, EditError> {
        if plan.schema_version != SCHEMA_VERSION {
            return Err(err(
                "UNSUPPORTED_SCHEMA",
                None,
                format!("schema_version {} is not supported", plan.schema_version),
            ));
        }
        let source_hash = source_sha256(source);
        if let Some(expected) = &plan.source_sha256
            && expected != &source_hash
        {
            return Err(err(
                "STALE_SOURCE",
                None,
                "source bytes do not match the plan's source_sha256",
            ));
        }
        if plan.author.trim().is_empty() {
            return Err(err("INVALID_PLAN", None, "author must be nonempty"));
        }
        visible::check_keep(plan)?;
        let probe = Opened::open(source).map_err(open_error)?;
        if probe
            .pkg
            .parts()
            .iter()
            .any(|name| name.starts_with("_xmlsignatures/") || name.ends_with("vbaProject.bin"))
        {
            return Err(err(
                "UNSUPPORTED_PACKAGE",
                None,
                "signed or macro-bearing package",
            ));
        }
        let source_bytes = source;
        let source_comment_max = existing_comment_ids(&probe);
        let mut used_revision_ids = tracked::used_ids(&probe, &[]);
        let Settled {
            mut base,
            mut opened,
            resolved_revisions,
            comment_changes,
            changed,
        } = settle(source, plan, probe, syntax)?;
        let mut renumbered = false;
        if let Some(normalized) = ranges::normalize_permission_ids(&base)? {
            base = normalized;
            opened = Opened::open(&base).map_err(open_error)?;
            renumbered = true;
        }
        if changed || renumbered {
            used_revision_ids.extend(tracked::used_ids(&opened, &[]));
        }
        let (removed_comments, kept_comments) = comment_changes.records(source)?;
        let base_sha256 = source_sha256(&base);
        let Loaded {
            stories,
            paragraph_nodes,
            paragraph_story,
        } = load_stories(&mut opened)?;
        // Settling merged or removed paragraphs: positional ids keep
        // naming the source's paragraphs through a map.
        let origin = if changed {
            let ids: Vec<String> = stories.iter().map(|s| s.id.clone()).collect();
            Some(origin::Origin::build(
                source_bytes,
                plan,
                &ids,
                &paragraph_story,
            )?)
        } else {
            None
        };
        let projections = paragraph_nodes
            .iter()
            .map(|&p| project_paragraph(&opened.dom, p))
            .collect();
        let date = plan
            .date
            .clone()
            .unwrap_or_else(crate::util::datetime::utc_now_iso8601);
        let initials = plan.initials.clone().unwrap_or_else(|| {
            plan.author
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect::<String>()
                .to_uppercase()
        });
        // Above the source's ids too: an id the settling removed never
        // names a new comment in the same report (#1264).
        let next_comment_id = existing_comment_ids(&opened)
            .max(source_comment_max)
            .map_or(0, |max| u64::from(max) + 1);
        let controls = crate::inspect::body_control_nodes(&opened.dom, opened.body);
        let control_records = crate::inspect::collect_controls(&opened.dom, opened.body);
        let family = crate::comments::CommentFamily::load(&opened.pkg, &opened.main)
            .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
        let revision_ids = RevisionCounter::new(used_revision_ids.clone());
        Ok(Self {
            plan,
            revision_refusal: std::cell::RefCell::new(None),
            source_sha256: source_hash,
            base,
            base_sha256,
            resolved_revisions,
            removed_comments,
            kept_comments,
            date,
            initials,
            opened,
            stories,
            paragraph_nodes,
            paragraph_story,
            projections,
            anchor_candidate_cache: std::cell::RefCell::new(HashMap::new()),
            origin,
            outcomes: Vec::new(),
            resolved: Vec::new(),
            claims: BTreeMap::new(),
            anchor_window: std::cell::Cell::new(None),
            syntax,
            comments: Vec::new(),
            deletion_comments: Vec::new(),
            preset_comment_ids: Vec::new(),
            next_comment_id,
            comments_added: 0,
            whole_marks: Vec::new(),
            new_numbering: (Vec::new(), Vec::new()),
            needed_styles: std::collections::BTreeSet::new(),
            family: Some(family),
            reply_parents: BTreeMap::new(),
            rewritten: Vec::new(),
            watermark: None,
            controls,
            control_records,
            notes_story: None,
            settings: None,
            written_stories: std::collections::BTreeSet::new(),
            revision_ids,
            used_revision_ids,
        })
    }

    /// Whether the redline is emitted directly, as under keep: the comparer
    /// moves a comment's range off text an edit replaces whole, so a plan
    /// that does is emitted, unless it holds an operation the emitter
    /// cannot track (a footnote, an image). `update_fields` refreshes only
    /// the clean copy, which the emitted redline would then not accept to:
    /// refused beside such an edit, as under keep.
    ///
    /// The comparer also places a bookmark, comment or permission marker by
    /// the words around it, so an edit right beside one (or over one) can
    /// carry it into the new insertion, or drop it from the rejected side:
    /// such a plan is emitted too, and its new words join no range (#1156).
    /// Under `update_fields` it is compared, as before: a heading the TOC
    /// lists holds its `_Toc` bookmark around its text, and renaming it is
    /// what a plan refreshing the TOC does.
    ///
    /// A plan that only writes comment threads
    /// ([`Self::edits_only_comments`]) is emitted as well: it edits no text,
    /// and the comparer's own normalizations would come back as changes.
    pub(super) fn emits_directly(&self) -> Result<bool, EditError> {
        if !tracked::emits_all(self) {
            return Ok(false);
        }
        // The comparer resolves compatibility branches before comparing.
        // Emit edits beside raw AC tables directly so an untouched Fallback
        // survives, as it does under keep.
        let has_alternate_table = self
            .opened
            .dom
            .descendants(self.opened.body, Some(&W::tbl()))
            .into_iter()
            .any(|table| {
                !crate::inspect::in_text_box(&self.opened.dom, table)
                    && !self
                        .opened
                        .dom
                        .ancestors(
                            table,
                            Some(&crate::namespaces::MC::name("AlternateContent")),
                        )
                        .is_empty()
            });
        if self.edits_a_comment_range() || has_alternate_table {
            if self.plan.update_fields {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    "update_fields cannot be combined with replacing the text a comment covers; refresh the fields with `jubarte convert FILE --update-fields -o OUT.docx` afterwards",
                ));
            }
            return Ok(true);
        }
        Ok(!self.plan.update_fields
            && (self.edits_beside_a_range_marker() || self.edits_only_comments()))
    }

    /// Whether every operation of the plan writes a comment thread and
    /// nothing else: a comment, a reply, a resolution, a reworded or a
    /// deleted comment. Through the comparer, such a plan on a clean file
    /// listed the comparer's normalizations as changes: a spacing's
    /// `lineRule` written out, the stylesheet merged, a paragraph written
    /// again (#1861).
    pub(super) fn edits_only_comments(&self) -> bool {
        !self.plan.operations.is_empty()
            && self.plan.operations.iter().all(|op| {
                matches!(
                    op.kind,
                    OperationKind::Comment { .. }
                        | OperationKind::ReplyComment { .. }
                        | OperationKind::ResolveComment { .. }
                        | OperationKind::EditComment { .. }
                        | OperationKind::DeleteComment { .. }
                )
            })
    }

    /// Whether a text edit writes new words beside a range marker: one in
    /// a word the edit touches (the comparer rewrites whole words), or one
    /// at body level next to the paragraph's first or last character. A
    /// comment's marker inside the text a replacement removes does not
    /// count: a replacement of the text a comment covers is
    /// [`Self::edits_a_comment_range`], and the comparer takes a comment
    /// whose words an edit removes into the deletion, as Word does. One at
    /// the replaced text's edge, or further into its word, counts.
    pub(super) fn edits_beside_a_range_marker(&self) -> bool {
        let separators = crate::comparer::WmlComparerSettings::default().word_separators;
        self.resolved.iter().any(|(_, r)| match r {
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                ..
            } if !replacement.is_empty() => {
                let text = &self.projections[*para].text;
                let (from, to) = word_window(text, *start, *end, &separators);
                let marks = self.range_marks(*para);
                marks.iter().any(|(name, _, at)| {
                    (from..=to).contains(at)
                        && !(name.starts_with("comment") && (*start + 1..*end).contains(at))
                }) || (from == 0 && self.body_level_marker_beside(*para, true))
                    || (to == text.len() && self.body_level_marker_beside(*para, false))
            }
            _ => false,
        })
    }

    /// Whether a range marker sits at body level right before (or after)
    /// paragraph `para`.
    fn body_level_marker_beside(&self, para: usize, before: bool) -> bool {
        let dom = &self.opened.dom;
        let node = self.paragraph_nodes[para];
        let beside = if before {
            dom.nodes_before_self(node)
                .into_iter()
                .rev()
                .find(|&n| dom.is_element(n))
        } else {
            dom.next_element(node)
        };
        beside.is_some_and(|n| range_marker(dom, n).is_some())
    }

    /// Whether the plan names a comment's text (`cN.text`), or one of its
    /// text edits replaces exactly the text a comment covers with new text.
    fn edits_a_comment_range(&self) -> bool {
        let names_comment_text = self
            .plan
            .operations
            .iter()
            .any(|op| selector_of(&op.kind).and_then(comment_text_of).is_some());
        if names_comment_text {
            return true;
        }
        self.resolved.iter().any(|(_, r)| match r {
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                ..
            } if !replacement.is_empty() => self.comment_ranges(*para).contains(&(*start, *end)),
            _ => false,
        })
    }

    /// The `(start, end)` projection offsets of each comment whose range
    /// opens and closes in paragraph `para`.
    fn comment_ranges(&self, para: usize) -> Vec<(usize, usize)> {
        let mut open: Vec<(String, usize)> = Vec::new();
        let mut ranges = Vec::new();
        for (id, starts, offset) in self.comment_marks(para) {
            if starts {
                open.push((id, offset));
            } else if let Some(at) = open.iter().position(|(o, _)| *o == id) {
                ranges.push((open.remove(at).1, offset));
            }
        }
        ranges
    }

    /// Each comment range marker of paragraph `para`, in document order:
    /// its `w:id`, whether it opens the range, and its projection offset.
    fn comment_marks(&self, para: usize) -> Vec<(String, bool, usize)> {
        #[cfg(test)]
        COMMENT_MARK_SCANS.with(|scans| scans.set(scans.get() + 1));
        self.range_marks(para)
            .into_iter()
            .filter_map(|(name, id, offset)| {
                let starts = match name {
                    "commentRangeStart" => true,
                    "commentRangeEnd" => false,
                    _ => return None,
                };
                Some((id?, starts, offset))
            })
            .collect()
    }

    /// Each range marker of paragraph `para` (see [`range_marker`]), in
    /// document order: its local name, its `w:id`, and its projection
    /// offset.
    fn range_marks(&self, para: usize) -> Vec<(&'static str, Option<String>, usize)> {
        let dom = &self.opened.dom;
        let mut ends = BTreeMap::new();
        for segment in &self.projections[para].segments {
            let node = match segment.piece {
                Piece::Text { t, .. } => t,
                Piece::Glyph { element, .. } => element,
            };
            ends.entry(node.0).or_insert(segment.end);
        }
        let mut offset = 0;
        let mut marks = Vec::new();
        for node in dom.descendants(self.paragraph_nodes[para], None) {
            if let Some(&end) = ends.get(&node.0) {
                offset = end;
                continue;
            }
            if let Some(name) = range_marker(dom, node) {
                let id = dom.attribute(node, &W::id()).map(str::to_string);
                marks.push((name, id, offset));
            }
        }
        marks
    }

    /// Every complete range in one document pass. Like `comment_text`,
    /// retain the first start and first following end for each literal id.
    pub(super) fn comment_anchors(&self) -> BTreeMap<String, Covered> {
        let mut open = BTreeMap::new();
        let mut covered = BTreeMap::new();
        for para in 0..self.paragraph_nodes.len() {
            for (id, starts, offset) in self.comment_marks(para) {
                if covered.contains_key(&id) {
                    continue;
                }
                if starts {
                    open.entry(id).or_insert((para, offset));
                } else if let Some((first, start)) = open.remove(&id) {
                    covered.insert(
                        id,
                        Covered {
                            first,
                            start,
                            last: para,
                            end: offset,
                        },
                    );
                }
            }
        }
        covered
    }

    /// The text comment `n` covers (`cN.text`), from its range start to
    /// its range end.
    pub(super) fn comment_text(&self, n: u32) -> Result<Covered, origin::Miss> {
        let id = n.to_string();
        let mut opened: Option<(usize, usize)> = None;
        for para in 0..self.paragraph_nodes.len() {
            for (mark, starts, offset) in self.comment_marks(para) {
                if mark != id {
                    continue;
                }
                match opened {
                    None if starts => opened = Some((para, offset)),
                    Some((first, start)) if !starts => {
                        return Ok(Covered {
                            first,
                            start,
                            last: para,
                            end: offset,
                        });
                    }
                    _ => {}
                }
            }
        }
        Err((
            "ANCHOR_NOT_FOUND".into(),
            format!("no comment c{n} covers text in this document"),
            0,
        ))
    }

    /// `cN.text` rewritten as `new_text` (empty: deleted): one tracked
    /// substitution inside the comment. A range over two paragraphs that
    /// follow each other also deletes the mark between them, as Word does.
    fn resolve_comment_text(
        &self,
        n: u32,
        new_text: &str,
        outcome: &mut EditOutcome,
    ) -> Result<Vec<Resolved>, (String, String)> {
        let covered = self.comment_text(n).map_err(|(code, message, matches)| {
            outcome.matches = matches;
            (code, message)
        })?;
        let Covered {
            first,
            start,
            last,
            end,
        } = covered;
        outcome.paragraph = Some(self.paragraph_id(first));
        if first == last && start == end {
            return Err((
                "INVALID_EDIT".into(),
                format!(
                    "c{n} covers no text; edit its paragraph ({}) instead",
                    crate::ids::short_paragraph(&self.paragraph_id(first))
                ),
            ));
        }
        check_run_text(new_text).map_err(|m| ("INVALID_EDIT".to_string(), m))?;
        let structure = |m| ("UNSUPPORTED_STRUCTURE".to_string(), m);
        outcome.matches = 1;
        let text = &self.projections[first].text;
        if first == last {
            self.check_range(&self.projections[first], start, end)
                .map_err(structure)?;
            outcome.context = Some(context(
                text,
                start,
                end,
                &format!("{{{}→{}}}", &text[start..end], new_text),
            ));
            return Ok(vec![Resolved::Text {
                para: first,
                start,
                end,
                replacement: new_text.to_string(),
                comment: None,
                attach_before: false,
                format: None,
            }]);
        }
        let dom = &self.opened.dom;
        let adjacent = last == first + 1
            && self.paragraph_story[first].0 == self.paragraph_story[last].0
            && dom.next_element(self.paragraph_nodes[first]) == Some(self.paragraph_nodes[last]);
        if !adjacent {
            return Err(structure(format!(
                "c{n} covers text from {} to {}; cN.text edits one paragraph, or two that follow each other",
                crate::ids::short_paragraph(&self.paragraph_id(first)),
                crate::ids::short_paragraph(&self.paragraph_id(last))
            )));
        }
        let first_end = text.len();
        self.check_range(&self.projections[first], start, first_end)
            .map_err(structure)?;
        self.check_range(&self.projections[last], 0, end)
            .map_err(structure)?;
        outcome.context = Some(context(
            text,
            start,
            first_end,
            &format!(
                "{{{}¶{}→{}}}",
                &text[start..],
                &self.projections[last].text[..end],
                new_text
            ),
        ));
        let mut resolved = vec![Resolved::Text {
            para: first,
            start,
            end: first_end,
            replacement: new_text.to_string(),
            comment: None,
            attach_before: start == first_end,
            format: None,
        }];
        if end > 0 {
            resolved.push(Resolved::Text {
                para: last,
                start: 0,
                end,
                replacement: String::new(),
                comment: None,
                attach_before: false,
                format: None,
            });
        }
        resolved.push(Resolved::MergeParagraphs {
            para: first,
            next: last,
            separator: String::new(),
            strike: false,
        });
        Ok(resolved)
    }

    pub(super) fn report(&self, ok: bool) -> EditReport {
        EditReport {
            schema_version: SCHEMA_VERSION,
            ok,
            source_sha256: self.source_sha256.clone(),
            base_sha256: self.base_sha256.clone(),
            guarded: self.plan.source_sha256.is_some(),
            author: self.plan.author.clone(),
            date: self.date.clone(),
            existing_revisions: self.plan.existing_revisions,
            resolved_revisions: self.resolved_revisions.clone(),
            removed_comments: self.removed_comments.clone(),
            clean_removed_comments: Vec::new(),
            kept_comments: self
                .kept_comments
                .iter()
                .cloned()
                .map(|mut c| {
                    let mut family: Vec<u32> = std::iter::once(c.id.as_str())
                        .chain(c.replies.iter().map(String::as_str))
                        .filter_map(|s| s.strip_prefix('c')?.parse().ok())
                        .collect();
                    for (&id, &parent) in &self.reply_parents {
                        if family.contains(&parent) && !family.contains(&id) {
                            family.push(id);
                            c.replies.push(format!("c{id}"));
                        }
                    }
                    c
                })
                .collect(),
            paragraph_ids: self
                .origin
                .as_ref()
                .map(|o| o.moves(|p| self.paragraph_id(p)))
                .unwrap_or_default(),
            split_revisions: Vec::new(),
            paragraphs: ParagraphDelta {
                from: self.body_paragraph_count(),
                to: self.body_paragraph_count(),
            },
            operations: self.outcomes.clone(),
            comments_added: self.comments_added,
            revisions: RevisionCounts::default(),
            #[cfg(feature = "render")]
            fields: Vec::new(),
            skipped: 0,
            notes: self.rewrite_notes(),
            output_sha256: None,
        }
    }

    /// New comments take ids above every comment `source` defines, as
    /// [`Self::start`] does when it settles the source itself.
    pub(super) fn skip_comment_ids_of(&mut self, source: &[u8]) -> Result<(), EditError> {
        let opened = Opened::open(source).map_err(open_error)?;
        if let Some(max) = existing_comment_ids(&opened) {
            self.next_comment_id = self.next_comment_id.max(u64::from(max) + 1);
        }
        Ok(())
    }

    /// One note per comment of another author a thread operation changed:
    /// rewording keeps their name on our words; deleting, resolving or
    /// reopening speaks for them.
    fn rewrite_notes(&self) -> Vec<String> {
        self.rewritten
            .iter()
            .filter_map(|(i, others)| {
                let outcome = self.outcomes.get(*i).filter(|o| o.status == "ok")?;
                Some(format!(
                    "{} ({}): {}",
                    outcome.id,
                    outcome.kind,
                    others.note()
                ))
            })
            .collect()
    }

    /// Record each thread operation among `resolved` (the operation whose
    /// outcome will be `outcomes[outcome]`) that changes a comment not the
    /// plan author's, for [`Self::rewrite_notes`]: v1 and v2 alike.
    pub(super) fn note_rewrites(&mut self, outcome: usize, resolved: &[Resolved]) {
        let Some(family) = &self.family else {
            return;
        };
        let changed: Vec<OthersComment> = resolved
            .iter()
            .filter_map(|r| match r {
                Resolved::Thread { op, .. } => Some(self.others_comments(family, op)),
                _ => None,
            })
            .flatten()
            .collect();
        self.rewritten
            .extend(changed.into_iter().map(|others| (outcome, others)));
    }

    /// The author of comment `id` when it is not the plan author's, `""`
    /// for one that names no author: only a comment that names the plan's
    /// author is its own (#1264).
    fn not_own_comment(&self, family: &crate::comments::CommentFamily, id: u32) -> Option<String> {
        let author = family.author(id).unwrap_or_default();
        (author != self.plan.author).then(|| author.to_string())
    }

    /// The comments of other authors `op` changes: the one it rewords,
    /// deletes, resolves or reopens, or the replies by others that deleting
    /// the plan author's own comment takes with it (#1846). A reply
    /// changes no one's comment.
    fn others_comments(
        &self,
        family: &crate::comments::CommentFamily,
        op: &ThreadOp,
    ) -> Vec<OthersComment> {
        let change = match op {
            ThreadOp::Reply { .. } => return Vec::new(),
            ThreadOp::Edit { .. } => OthersChange::Reword,
            ThreadOp::Delete { .. } => OthersChange::Delete,
            ThreadOp::Resolve { done: true, .. } => OthersChange::Resolve,
            ThreadOp::Resolve { done: false, .. } => OthersChange::Reopen,
        };
        let target = op.target();
        if let Some(author) = self.not_own_comment(family, target) {
            return vec![OthersComment {
                comment: target,
                author,
                change,
            }];
        }
        if !matches!(op, ThreadOp::Delete { .. }) {
            return Vec::new();
        }
        family
            .with_replies(target)
            .into_iter()
            .filter(|&reply| reply != target)
            .filter_map(|reply| {
                self.not_own_comment(family, reply)
                    .map(|author| OthersComment {
                        comment: reply,
                        author,
                        change: OthersChange::DeletedWith(target),
                    })
            })
            .collect()
    }

    /// Resolve every operation against the untouched source; the first
    /// failure is returned after every operation has been tried.
    pub(super) fn resolve(&mut self) -> Result<(), EditError> {
        let mut first_failure: Option<EditError> = None;
        for (i, op) in self.plan.operations.iter().enumerate() {
            self.revision_refusal.take();
            let id = op.id.clone().unwrap_or_else(|| format!("op-{}", i + 1));
            let kind = kind_name(&op.kind).to_string();
            let window = selector_of(&op.kind)
                .and_then(comment_text_of)
                .and_then(|n| {
                    self.comment_text(n)
                        .ok()
                        .filter(|c| c.first == c.last)
                        .map(|c| (n, c.start, c.end))
                });
            self.anchor_window.set(window);
            let mut claim = None;
            let result = match &op.kind {
                OperationKind::Rewrite {
                    paragraph,
                    text,
                    comment,
                } => self.resolve_rewrite(&id, paragraph, text, comment.as_deref()),
                // `-p cN.text --delete`: the covered text goes.
                OperationKind::DeleteParagraph {
                    paragraph,
                    comment: None,
                } if comment_text_of(paragraph).is_some() => {
                    self.resolve_rewrite(&id, paragraph, "", None)
                }
                OperationKind::List {
                    paragraphs,
                    kind_of_list,
                    level,
                    restart,
                } => self
                    .resolve_list(&id, paragraphs, *kind_of_list, *level, *restart)
                    .map(|(resolved, outcome)| (vec![resolved], outcome)),
                OperationKind::PageSetup {
                    section,
                    page,
                    orientation,
                    margins_dxa,
                } => {
                    let mut outcome = EditOutcome {
                        id: id.clone(),
                        kind: String::new(),
                        status: String::new(),
                        paragraph: None,
                        matches: 0,
                        context: None,
                        comment_id: None,
                        code: None,
                        message: None,
                        anchor_given: None,
                        anchor_read_as: None,
                        near: None,
                        candidates: Vec::new(),
                    };
                    match self.resolve_page_setup(
                        *section,
                        *page,
                        *orientation,
                        *margins_dxa,
                        &mut outcome,
                    ) {
                        Ok(targets) => {
                            outcome.context = Some(format!("{{§ {} section(s)}}", targets.len()));
                            Ok((vec![Resolved::PageSetup { targets }], outcome))
                        }
                        Err((code, message)) => {
                            Err(Box::new((err(&code, Some(&id), message), outcome)))
                        }
                    }
                }
                OperationKind::ReplyComment { .. }
                | OperationKind::ResolveComment { .. }
                | OperationKind::EditComment { .. }
                | OperationKind::DeleteComment { .. } => self
                    .resolve_thread(&id, &op.kind)
                    .map(|(resolved, outcome)| (vec![resolved], outcome)),
                OperationKind::Watermark {
                    text,
                    color,
                    diagonal,
                    font,
                } => self
                    .resolve_watermark(&id, (text, color.as_deref(), *diagonal, font.as_deref()))
                    .map(|(spec, outcome)| {
                        self.watermark = Some(spec);
                        (Vec::new(), outcome)
                    }),
                OperationKind::FillControl {
                    control,
                    text,
                    choice,
                    checked,
                    date,
                } => self.resolve_fill_control(
                    &id,
                    &controls::FillRequest {
                        control,
                        text: text.as_deref(),
                        choice: choice.as_deref(),
                        checked: *checked,
                        date: date.as_deref(),
                    },
                ),
                OperationKind::Settings {
                    track_revisions,
                    update_fields,
                    protection,
                } => {
                    let request = crate::settings::SettingsRequest {
                        track_revisions: *track_revisions,
                        update_fields: *update_fields,
                        protection: protection.clone(),
                    };
                    self.resolve_settings(&id, &request).map(|outcome| {
                        self.settings = Some(request);
                        (Vec::new(), outcome)
                    })
                }
                OperationKind::Strike { revision, comment }
                | OperationKind::Restore { revision, comment } => {
                    let visible = match &op.kind {
                        OperationKind::Strike { .. } => visible::Visible::Strike,
                        _ => visible::Visible::Restore,
                    };
                    self.resolve_visible(&id, visible, revision, comment.as_deref())
                }
                other => self.resolve_one(&id, other).map(|(resolved, mut outcome)| {
                    if let OperationKind::Replace {
                        whole: false,
                        format: None,
                        ..
                    } = other
                    {
                        if let Resolved::Text {
                            para, start, end, ..
                        } = resolved
                        {
                            claim = Some((para, start, end));
                        }
                        let pieces = self.replace_pieces(resolved, &mut outcome);
                        return (pieces, outcome);
                    }
                    // A deletion's comment sits on the words it strikes, as a
                    // strike's does: in the redline only.
                    let comment = match (other, &resolved) {
                        (
                            OperationKind::Delete {
                                comment: Some(_), ..
                            },
                            Resolved::Text {
                                para, start, end, ..
                            },
                        ) => Some(Resolved::StrikeComment {
                            para: *para,
                            start: *start,
                            end: *end,
                        }),
                        _ => None,
                    };
                    (comment.into_iter().chain([resolved]).collect(), outcome)
                }),
            };
            self.anchor_window.set(None);
            match result {
                Ok((resolved, mut outcome)) => {
                    outcome.id = id;
                    outcome.kind = kind;
                    outcome.status = "ok".into();
                    if let Some(claim) = claim {
                        self.claims.insert(i, claim);
                    }
                    self.note_rewrites(self.outcomes.len(), &resolved);
                    self.outcomes.push(outcome);
                    self.resolved
                        .extend(resolved.into_iter().map(|resolved| (i, resolved)));
                }
                Err(boxed) => {
                    let (e, outcome) = *boxed;
                    let mut e = self.attach_revision_refusal(e);
                    e.operation = Some(id.clone());
                    e.near = outcome.near.clone().map(Box::new);
                    self.outcomes.push(EditOutcome {
                        id,
                        kind,
                        status: "failed".into(),
                        code: Some(e.code.clone()),
                        message: Some(e.message.clone()),
                        ..outcome
                    });
                    if let Some(first) = &mut first_failure {
                        first.revision_refusals = std::mem::take(&mut first.revision_refusals)
                            .into_vec()
                            .into_iter()
                            .chain(e.revision_refusals.into_vec())
                            .collect();
                    } else {
                        first_failure = Some(e);
                    }
                }
            }
        }
        if let Some(mut e) = first_failure {
            e.outcomes = self.outcomes.clone();
            return Err(e);
        }
        self.check_conflicts()?;
        Ok(())
    }

    pub(super) fn resolve_one(
        &self,
        id: &str,
        kind: &OperationKind,
    ) -> Result<(Resolved, EditOutcome), Box<(EditError, EditOutcome)>> {
        match kind {
            OperationKind::Redact { find, .. } => {
                let result = self.resolve_one_unreported(id, kind);
                // A refusal can quote a selector, field, control or revision
                // author as well as the anchor. None is safe locating help for
                // a redaction, including the deferred revision diagnostics.
                self.revision_refusal.take();
                let scrub = |outcome: EditOutcome| EditOutcome {
                    id: outcome.id,
                    kind: outcome.kind,
                    status: outcome.status,
                    paragraph: outcome.paragraph,
                    matches: outcome.matches,
                    context: outcome.context.map(|_| format!("{{{}}}", redaction(find))),
                    comment_id: None,
                    anchor_given: None,
                    anchor_read_as: None,
                    near: None,
                    // #1889's candidates name paragraphs by the text they
                    // hold: for a redaction that says where to find it.
                    candidates: Vec::new(),
                    code: None,
                    message: None,
                };
                // Rebuild from safe fields on both paths. Adding an outcome
                // field requires an explicit privacy decision here, rather
                // than silently forwarding another copy of document text.
                match result {
                    Ok((resolved, outcome)) => Ok((resolved, scrub(outcome))),
                    Err(boxed) => {
                        let (error, outcome) = *boxed;
                        let at = outcome.paragraph.as_ref().map_or_else(
                            || "in the selected paragraphs".to_string(),
                            |p| format!("in {}", crate::ids::short_paragraph(p)),
                        );
                        let message = match error.code.as_str() {
                            "ANCHOR_NOT_FOUND" => format!(
                                "the text to redact does not occur {at}{}",
                                if outcome.near.is_some() {
                                    "; a passage differs from it only in whitespace"
                                } else {
                                    ""
                                }
                            ),
                            "AMBIGUOUS_ANCHOR" => format!(
                                "the text to redact has {} matches {at}; select one paragraph and set a 1-based occurrence",
                                outcome.matches
                            ),
                            "INVALID_EDIT" => format!(
                                "the text to redact needs a nonempty find; occurrence is 1-based {at}"
                            ),
                            _ => format!("the text to redact cannot be edited {at}"),
                        };
                        Err(Box::new((
                            err(&error.code, Some(id), message),
                            scrub(outcome),
                        )))
                    }
                }
            }
            _ => self.resolve_one_unreported(id, kind),
        }
    }

    fn resolve_one_unreported(
        &self,
        id: &str,
        kind: &OperationKind,
    ) -> Result<(Resolved, EditOutcome), Box<(EditError, EditOutcome)>> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
            candidates: Vec::new(),
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let selector = match kind {
            OperationKind::Replace { paragraph, .. }
            | OperationKind::Insert { paragraph, .. }
            | OperationKind::Delete { paragraph, .. }
            | OperationKind::Comment { paragraph, .. }
            | OperationKind::InsertParagraph { paragraph, .. }
            | OperationKind::DeleteParagraph { paragraph, .. }
            | OperationKind::FormatParagraph { paragraph, .. }
            | OperationKind::MergeParagraphs { paragraph, .. }
            | OperationKind::Rewrite { paragraph, .. }
            | OperationKind::InsertTable { paragraph, .. }
            | OperationKind::InsertToc { paragraph, .. }
            | OperationKind::FormatRun { paragraph, .. }
            | OperationKind::InsertFootnote { paragraph, .. }
            | OperationKind::InsertImage { paragraph, .. }
            | OperationKind::Redact { paragraph, .. } => paragraph,
            OperationKind::List { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "list resolves through resolve_list".into(),
                    outcome,
                ));
            }
            OperationKind::PageSetup { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "page_setup resolves through resolve_page_setup".into(),
                    outcome,
                ));
            }
            OperationKind::ReplyComment { .. }
            | OperationKind::ResolveComment { .. }
            | OperationKind::EditComment { .. }
            | OperationKind::DeleteComment { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "thread operations resolve through resolve_thread".into(),
                    outcome,
                ));
            }
            OperationKind::Watermark { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "watermark resolves through resolve_watermark".into(),
                    outcome,
                ));
            }
            OperationKind::FillControl { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "fill_control resolves through resolve_fill_control".into(),
                    outcome,
                ));
            }
            OperationKind::Settings { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "settings resolve through resolve_settings".into(),
                    outcome,
                ));
            }
            OperationKind::Strike { .. } | OperationKind::Restore { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "strike and restore resolve through resolve_visible".into(),
                    outcome,
                ));
            }
        };
        let para = match self.select(selector, finds_text(kind)) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                // #1889: a positional miss (the paragraph does not exist)
                // names the paragraphs the operation's anchor does occur
                // in. A text selector's miss is about the selector's own
                // text, so it gains nothing.
                let note = match (code.as_str(), selector, anchor_of(kind)) {
                    (
                        "ANCHOR_NOT_FOUND",
                        Selector::Name(_) | Selector::Id { .. } | Selector::Index { .. },
                        Some(anchor),
                    ) if comment_text_of(selector).is_none() => {
                        self.candidates_note(anchor, &mut outcome)
                    }
                    _ => String::new(),
                };
                return Err(fail(&code, format!("{msg}{note}"), outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        if let Some(n) = comment_text_of(selector) {
            let paragraph_level = matches!(
                kind,
                OperationKind::InsertParagraph { .. }
                    | OperationKind::FormatParagraph { .. }
                    | OperationKind::MergeParagraphs { .. }
                    | OperationKind::InsertTable { .. }
                    | OperationKind::InsertToc { .. }
                    | OperationKind::InsertImage { .. }
            );
            let message = if matches!(kind, OperationKind::DeleteParagraph { .. }) {
                Some(format!("deleting c{n}.text takes no comment"))
            } else if paragraph_level {
                Some(format!(
                    "c{n}.text names text, not a paragraph; use the paragraph's id ({})",
                    crate::ids::short_paragraph(&self.paragraph_id(para))
                ))
            } else if self.anchor_window.get().is_none() {
                Some(format!(
                    "c{n} covers text in more than one paragraph; give the anchor with a paragraph id, or rewrite c{n}.text whole"
                ))
            } else {
                None
            };
            if let Some(message) = message {
                return Err(fail("INVALID_EDIT", message, outcome));
            }
        }
        // `occurrence` picks a hit of an anchor; with no anchor it would be
        // silently ignored.
        let anchorless = matches!(
            kind,
            OperationKind::Insert {
                after: None,
                before: None,
                occurrence: Some(_),
                ..
            } | OperationKind::Comment {
                find: None,
                occurrence: Some(_),
                ..
            }
        );
        if anchorless {
            return Err(fail(
                "INVALID_EDIT",
                "occurrence needs an anchor: give find, after or before".into(),
                outcome,
            ));
        }
        let comments = match kind {
            OperationKind::Replace { comment, .. }
            | OperationKind::Insert { comment, .. }
            | OperationKind::Delete { comment, .. }
            | OperationKind::InsertParagraph { comment, .. } => comment.is_some(),
            OperationKind::Comment { .. } => true,
            _ => false,
        };
        if comments && self.paragraph_story[para].0 != 0 {
            return Err(fail(
                "COMMENT_NOT_IN_BODY",
                "comments are supported in the body only: Word cannot anchor one in a header, footer or note".into(),
                outcome,
            ));
        }
        let projection = &self.projections[para];
        let text = &projection.text;
        match kind {
            OperationKind::Replace {
                find,
                replacement,
                format,
                comment,
                occurrence,
                ..
            } => {
                check_run_text(replacement)
                    .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                if let Some(format) = format {
                    check_format(format, replacement)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{{}→{}}}", &text[start..end], replacement),
                ));
                Ok((
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement: replacement.clone(),
                        comment: comment.clone(),
                        attach_before: true,
                        format: format.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::Redact {
                find, occurrence, ..
            } => {
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                let blocks = redaction(find);
                outcome.context = Some(format!("{{{blocks}}}"));
                Ok((
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement: blocks,
                        comment: None,
                        attach_before: true,
                        format: None,
                    },
                    outcome,
                ))
            }
            OperationKind::Delete {
                find,
                occurrence,
                comment,
                ..
            } => {
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{-{}}}", &text[start..end]),
                ));
                Ok((
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement: String::new(),
                        comment: None,
                        attach_before: true,
                        format: None,
                    },
                    outcome,
                ))
            }
            OperationKind::Insert {
                after,
                before,
                position,
                text: new,
                format,
                comment,
                occurrence,
                ..
            } => {
                check_run_text(new).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                if let Some(format) = format {
                    check_format(format, new)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if new.is_empty() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "text must be nonempty".into(),
                        outcome,
                    ));
                }
                let (pos, attach_before) = match (after, before, position) {
                    (Some(after), None, None) => {
                        let (_, end) = self
                            .find_range(projection, after, *occurrence, &mut outcome)
                            .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                        (end, true)
                    }
                    (None, Some(before), None) => {
                        let (start, _) = self
                            .find_range(projection, before, *occurrence, &mut outcome)
                            .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                        (start, false)
                    }
                    (None, None, Some(Edge::Start)) => {
                        outcome.matches = 1;
                        (
                            self.anchor_window.get().map_or(0, |(_, from, _)| from),
                            false,
                        )
                    }
                    (None, None, Some(Edge::End)) => {
                        outcome.matches = 1;
                        (
                            self.anchor_window.get().map_or(text.len(), |(_, _, to)| to),
                            true,
                        )
                    }
                    _ => {
                        return Err(fail(
                            "INVALID_EDIT",
                            "insert needs exactly one of after, before, position".into(),
                            outcome,
                        ));
                    }
                };
                self.check_insert_position(projection, pos, attach_before)
                    .map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
                outcome.context = Some(context(text, pos, pos, &format!("{{+{new}}}")));
                Ok((
                    Resolved::Text {
                        para,
                        start: pos,
                        end: pos,
                        replacement: new.clone(),
                        comment: comment.clone(),
                        attach_before,
                        format: format.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::Comment {
                find,
                text: note,
                through: Some(through),
                ..
            } => {
                check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                if find.is_some() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "find and through cannot be combined: through comments on whole paragraphs"
                            .into(),
                        outcome,
                    ));
                }
                let last = match self.select(through, false) {
                    Ok(p) => p,
                    Err((code, msg, matches)) => {
                        outcome.matches = matches;
                        return Err(fail(&code, format!("through: {msg}"), outcome));
                    }
                };
                if self.paragraph_story[last].0 != self.paragraph_story[para].0 || last < para {
                    return Err(fail(
                        "INVALID_EDIT",
                        format!(
                            "through ({}) must be in the same story as paragraph and not before it",
                            self.paragraph_id(last)
                        ),
                        outcome,
                    ));
                }
                outcome.matches = 1;
                outcome.context = Some(format!(
                    "{{#{} ... {}}}",
                    self.paragraph_id(para),
                    self.paragraph_id(last)
                ));
                Ok((
                    Resolved::CommentSpan {
                        para,
                        last,
                        text: note.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::Comment {
                find,
                text: note,
                occurrence,
                ..
            } => {
                check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                // A comment edits nothing: its range may cross a tab, a
                // symbol, a hyperlink, a field or a revision.
                let (start, end) = match find {
                    Some(find) => {
                        let found = locate(
                            projection,
                            find,
                            *occurrence,
                            &mut outcome,
                            self.anchor_window.get(),
                            self.syntax,
                        );
                        match found {
                            // #1889: the anchor names text the paragraph
                            // does not hold; say where it does. Under a
                            // comment's window the paragraph is right and
                            // the window is wrong.
                            Err((code, message))
                                if code == "ANCHOR_NOT_FOUND"
                                    && self.anchor_window.get().is_none() =>
                            {
                                let note = self.candidates_note(find, &mut outcome);
                                return Err(fail(&code, format!("{message}{note}"), outcome));
                            }
                            other => other.map_err(|(c, m)| fail(&c, m, outcome.clone()))?,
                        }
                    }
                    None => {
                        outcome.matches = 1;
                        self.anchor_window
                            .get()
                            .map_or((0, text.len()), |(_, from, to)| (from, to))
                    }
                };
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{#{}}}", &text[start..end]),
                ));
                Ok((
                    Resolved::CommentRange {
                        para,
                        start,
                        end,
                        text: note.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::DeleteParagraph { comment, .. } => {
                outcome.matches = 1;
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let node = self.paragraph_nodes[para];
                let dom = &self.opened.dom;
                if !dom.descendants(node, Some(&W::sect_pr())).is_empty() {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "paragraph carries section properties".into(),
                        outcome,
                    ));
                }
                // A container's final paragraph is emptied, not removed, when
                // nothing else would stay in it (`emptied_paragraphs`).
                outcome.context = Some(format!("{{-¶ {}}}", excerpt(text, 60)));
                Ok((Resolved::DeleteParagraph { para }, outcome))
            }
            OperationKind::FormatRun {
                find,
                format,
                occurrence,
                ..
            } => {
                let (start, end) = self
                    .resolve_format_run(projection, find, *occurrence, format, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{~{}}}", &text[start..end]),
                ));
                Ok((
                    Resolved::FormatRun {
                        para,
                        start,
                        end,
                        format: format.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::InsertFootnote {
                after,
                text: note,
                occurrence,
                ..
            } => {
                let at = self
                    .resolve_footnote(para, projection, after, *occurrence, note, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    at,
                    at,
                    &format!("{{^{}}}", excerpt(note, 40)),
                ));
                Ok((
                    Resolved::InsertFootnote {
                        para,
                        at,
                        text: note.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::InsertImage {
                position,
                image_base64,
                content_type,
                width_emu,
                alt,
                ..
            } => {
                outcome.matches = 1;
                if self.paragraph_story[para].0 != 0 {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "pictures can be inserted in the body only".into(),
                        outcome,
                    ));
                }
                let picture = images::picture(
                    image_base64,
                    content_type.as_deref(),
                    *width_emu,
                    alt.as_deref().unwrap_or_default(),
                )
                .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(format!(
                    "{{+¶ picture {} {}x{} EMU}} {}",
                    picture.content_type,
                    picture.width,
                    picture.height,
                    excerpt(text, 40)
                ));
                Ok((
                    Resolved::InsertImage {
                        anchor: para,
                        side: *position,
                        picture,
                    },
                    outcome,
                ))
            }
            OperationKind::Rewrite { .. }
            | OperationKind::List { .. }
            | OperationKind::ReplyComment { .. }
            | OperationKind::ResolveComment { .. }
            | OperationKind::EditComment { .. }
            | OperationKind::DeleteComment { .. }
            | OperationKind::PageSetup { .. } => Err(fail(
                "INVALID_PLAN",
                "rewrite, list, page_setup and thread operations resolve on their own paths".into(),
                outcome,
            )),
            OperationKind::Watermark { .. } => Err(fail(
                "INVALID_PLAN",
                "watermark resolves through resolve_watermark".into(),
                outcome,
            )),
            OperationKind::FillControl { .. } => Err(fail(
                "INVALID_PLAN",
                "fill_control resolves through resolve_fill_control".into(),
                outcome,
            )),
            OperationKind::Settings { .. } => Err(fail(
                "INVALID_PLAN",
                "settings resolve through resolve_settings".into(),
                outcome,
            )),
            OperationKind::Strike { .. } | OperationKind::Restore { .. } => Err(fail(
                "INVALID_PLAN",
                "strike and restore resolve through resolve_visible".into(),
                outcome,
            )),
            OperationKind::InsertParagraph {
                position,
                runs,
                like,
                style,
                comment,
                ..
            } => {
                outcome.matches = 1;
                let like = match like {
                    Some(selector) => self.select(selector, true).map_err(|(code, msg, _)| {
                        fail(&code, format!("like: {msg}"), outcome.clone())
                    })?,
                    None => para,
                };
                if runs.is_empty() || runs.iter().all(|r| r.text.is_empty()) {
                    return Err(fail("INVALID_EDIT", "runs must carry text".into(), outcome));
                }
                for r in runs {
                    check_run_text(&r.text)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                    check_format(&r.format(), &r.text)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let style = match style {
                    Some(style) => Some(
                        self.resolve_style(style)
                            .map_err(|m| fail("UNKNOWN_STYLE", m, outcome.clone()))?,
                    ),
                    None => None,
                };
                let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
                outcome.context = Some(format!("{{+¶ {}}}", excerpt(&joined, 60)));
                // Accepting a deleted mark joins its paragraph to the one
                // after it. The new paragraph is that one, inserted after
                // the mark's paragraph or before the paragraph right after
                // it; leave the mark.
                let marked = match position {
                    Side::After => Some(para),
                    Side::Before => self.paragraph_right_before(para),
                };
                if let Some(note) = marked.and_then(|m| self.pending_deleted_mark_note(m)) {
                    outcome.message = Some(note);
                }
                Ok((
                    Resolved::InsertParagraph {
                        anchor: para,
                        side: *position,
                        runs: runs.clone(),
                        like,
                        style,
                        comment: comment.clone(),
                        toc: None,
                        keep_mark: false,
                    },
                    outcome,
                ))
            }
            OperationKind::FormatParagraph {
                style,
                alignment,
                line_spacing,
                space_before,
                space_after,
                ..
            } => {
                outcome.matches = 1;
                let spacing = Spacing {
                    line: *line_spacing,
                    before: *space_before,
                    after: *space_after,
                };
                if style.is_none() && alignment.is_none() && spacing == Spacing::default() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "format_paragraph needs style, alignment, line_spacing, space_before or space_after".into(),
                        outcome,
                    ));
                }
                let style = match style {
                    Some(style) => Some(
                        self.resolve_style(style)
                            .map_err(|m| fail("UNKNOWN_STYLE", m, outcome.clone()))?,
                    ),
                    None => None,
                };
                let mut changes = Vec::new();
                if let Some(style) = &style {
                    changes.push(format!("style={style}"));
                }
                if let Some(alignment) = alignment {
                    changes.push(format!("alignment={}", alignment.jc()));
                }
                if let Some(LineSpacing(line)) = spacing.line {
                    changes.push(format!("line_spacing={}", f64::from(line) / 240.0));
                }
                if let Some(Points(before)) = spacing.before {
                    changes.push(format!("space_before={}pt", f64::from(before) / 20.0));
                }
                if let Some(Points(after)) = spacing.after {
                    changes.push(format!("space_after={}pt", f64::from(after) / 20.0));
                }
                outcome.context =
                    Some(format!("{{¶ {}}} {}", changes.join(" "), excerpt(text, 40)));
                Ok((
                    Resolved::FormatParagraph {
                        para,
                        style,
                        alignment: *alignment,
                        spacing,
                    },
                    outcome,
                ))
            }
            OperationKind::MergeParagraphs { separator, .. } => {
                outcome.matches = 1;
                let separator = separator.clone().unwrap_or_default();
                check_text(&separator).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                let next = self
                    .merge_partner(para)
                    .map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
                let tail: String = text
                    .chars()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                outcome.context = Some(format!(
                    "{tail}{{¶→{separator}}}{}",
                    excerpt(&self.projections[next].text, 20)
                ));
                Ok((
                    Resolved::MergeParagraphs {
                        para,
                        next,
                        separator,
                        strike: false,
                    },
                    outcome,
                ))
            }
            OperationKind::InsertTable {
                position,
                rows,
                header_row,
                widths_dxa,
                style,
                ..
            } => {
                outcome.matches = 1;
                if self.paragraph_story[para].0 != 0 {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "insert_table is supported in the body only".into(),
                        outcome,
                    ));
                }
                let dom = &self.opened.dom;
                if !dom
                    .ancestors(self.paragraph_nodes[para], Some(&W::tc()))
                    .is_empty()
                {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "the anchor paragraph is in a table cell; nested tables are not supported"
                            .into(),
                        outcome,
                    ));
                }
                let columns = structural::check_rows(rows, widths_dxa.as_deref())
                    .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                let widths = match widths_dxa {
                    Some(widths) => widths.clone(),
                    None => {
                        let each = structural::text_width(dom, self.opened.body)
                            / u32::try_from(columns).unwrap_or(1);
                        vec![each; columns]
                    }
                };
                let (style, add_style) = structural::resolve_table_style(
                    &structural::table_styles(&self.opened),
                    style.as_deref().unwrap_or("TableGrid"),
                )
                .map_err(|m| fail("UNKNOWN_STYLE", m, outcome.clone()))?;
                outcome.context = Some(format!(
                    "{{+table {}x{columns}}} {}",
                    rows.len(),
                    excerpt(&rows[0].join(" | "), 60)
                ));
                Ok((
                    Resolved::InsertTable {
                        anchor: para,
                        side: *position,
                        rows: rows.clone(),
                        header_row: *header_row,
                        widths,
                        style,
                        add_style,
                    },
                    outcome,
                ))
            }
            OperationKind::InsertToc {
                position,
                levels,
                title,
                ..
            } => {
                outcome.matches = 1;
                if self.paragraph_story[para].0 != 0 {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "a table of contents goes in the body".into(),
                        outcome,
                    ));
                }
                if !(1..=9).contains(levels) {
                    return Err(fail(
                        "INVALID_EDIT",
                        format!("levels {levels} is outside 1..=9"),
                        outcome,
                    ));
                }
                if let Some(title) = title {
                    if title.is_empty() {
                        return Err(fail(
                            "INVALID_EDIT",
                            "title must carry text".into(),
                            outcome,
                        ));
                    }
                    check_text(title).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                outcome.context = Some(format!("{{+TOC 1-{levels}}}"));
                if !self.plan.update_fields {
                    outcome.message = Some(
                        "the TOC has no entries until its fields are updated: set \"update_fields\": true, or update fields in Word".into(),
                    );
                }
                Ok((
                    Resolved::InsertParagraph {
                        anchor: para,
                        side: *position,
                        runs: Vec::new(),
                        like: para,
                        style: None,
                        comment: None,
                        toc: Some(TocSpec {
                            levels: *levels,
                            title: title.clone(),
                        }),
                        keep_mark: false,
                    },
                    outcome,
                ))
            }
        }
    }

    /// The paragraph `para` merges with: the next sibling paragraph, with only
    /// range markup (bookmarks, comment ranges, ...) between them. `para`'s
    /// properties are discarded, so it may not carry a section break.
    /// `reply_comment`, `resolve_comment`, `edit_comment`, `delete_comment`:
    /// the comment must exist (`UNKNOWN_COMMENT`); a reply needs the
    /// comment's reference to anchor on. With [`Others::Keep`],
    /// `edit_comment`, `delete_comment` and `resolve_comment` on another
    /// author's comment, or a delete whose thread holds another author's
    /// reply, are refused (`COMMENT_NOT_YOURS`). The default allows them
    /// and notes each one.
    pub(super) fn resolve_thread(&self, id: &str, kind: &OperationKind) -> Resolution<Resolved> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
            candidates: Vec::new(),
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let op = match kind {
            OperationKind::ReplyComment { comment_id, text } => ThreadOp::Reply {
                parent: *comment_id,
                text: text.clone(),
            },
            OperationKind::ResolveComment { comment_id, done } => ThreadOp::Resolve {
                id: *comment_id,
                done: *done,
            },
            OperationKind::EditComment { comment_id, text } => ThreadOp::Edit {
                id: *comment_id,
                text: text.clone(),
            },
            OperationKind::DeleteComment { comment_id } => ThreadOp::Delete { id: *comment_id },
            _ => {
                return Err(fail(
                    "INVALID_PLAN",
                    "not a thread operation".into(),
                    outcome,
                ));
            }
        };
        let target = op.target();
        let family = self.family.as_ref().expect("loaded in start");
        if !family.contains(target) {
            let message = if self.origin.as_ref().is_some_and(|o| o.had_comment(target)) {
                let revisions = self
                    .resolved_revisions
                    .accepted
                    .iter()
                    .chain(&self.resolved_revisions.rejected)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                // A blanket settlement names no revision: no parenthesis.
                let revisions = if revisions.is_empty() {
                    String::new()
                } else {
                    format!(" ({revisions})")
                };
                format!(
                    "c{target} is gone once the plan's revisions are settled{revisions}: its anchor and replies are removed, as Word does; a reply, edit or resolve on that thread would be lost; settle fewer revisions, drop the thread operation, or set \"removed_anchor_comments\": \"keep\"",
                )
            } else {
                format!("no comment c{target} in the document (jubarte comments lists them)")
            };
            return Err(fail("UNKNOWN_COMMENT", message, outcome));
        }
        if let ThreadOp::Reply { text, .. } | ThreadOp::Edit { text, .. } = &op {
            check_comment(text).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        }
        // With the guard on, another author answers a comment, and only its
        // author rewords, deletes, resolves or reopens it. Deleting one's
        // own comment deletes the replies in its thread too. The default
        // allows the operation and the report notes it.
        if self.plan.others.keeps()
            && let Some(others) = self.others_comments(family, &op).first()
        {
            let (reply, resolve) = match self.syntax {
                PlanSyntax::Flags | PlanSyntax::Typed => (
                    format!("-p c{target} --reply TEXT"),
                    format!("-p c{target} --resolve"),
                ),
                PlanSyntax::V1 | PlanSyntax::V2 => {
                    ("reply_comment".to_string(), "resolve_comment".to_string())
                }
            };
            let message = others.refusal(&reply, &resolve);
            return Err(fail("COMMENT_NOT_YOURS", message, outcome));
        }
        let ids = match &op {
            ThreadOp::Delete { id } => family.with_replies(*id),
            _ => vec![target],
        };
        let (stories, reference) = self.comment_markers(&ids, target);
        let anchor = reference.and_then(|r| {
            self.opened
                .dom
                .ancestors(r, Some(&W::p()))
                .into_iter()
                .find_map(|p| self.paragraph_nodes.iter().position(|&n| n == p))
        });
        if matches!(op, ThreadOp::Reply { .. }) && reference.is_none() {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!("comment {target} has no reference in the document to anchor a reply on"),
                outcome,
            ));
        }
        outcome.matches = 1;
        outcome.paragraph = anchor.map(|p| self.paragraph_id(p));
        if !matches!(op, ThreadOp::Reply { .. }) {
            outcome.comment_id = Some(target);
        }
        outcome.context = Some(format!("comment {target}"));
        let stories = match op {
            ThreadOp::Resolve { .. } | ThreadOp::Edit { .. } => Vec::new(),
            _ => stories,
        };
        Ok((
            Resolved::Thread {
                op,
                stories,
                anchor,
            },
            outcome,
        ))
    }

    /// Stories holding markers of the comments `ids`, and the reference
    /// element of comment `target`.
    pub(super) fn comment_markers(&self, ids: &[u32], target: u32) -> (Vec<usize>, Option<NodeId>) {
        let dom = &self.opened.dom;
        let wanted: Vec<String> = ids.iter().map(u32::to_string).collect();
        let target = target.to_string();
        let mut stories = Vec::new();
        let mut reference = None;
        for (index, story) in self.stories.iter().enumerate() {
            for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
                for marker in dom.descendants(story.root, Some(&W::name(local))) {
                    let Some(value) = dom.attribute(marker, &W::id()) else {
                        continue;
                    };
                    if !wanted.iter().any(|w| w == value) {
                        continue;
                    }
                    if !stories.contains(&index) {
                        stories.push(index);
                    }
                    if local == "commentReference" && value == target && reference.is_none() {
                        reference = Some(marker);
                    }
                }
            }
        }
        (stories, reference)
    }

    /// A `replace` as the word-level edits that make its matched text read
    /// as the replacement (plan 12): kept words stay in their runs, with
    /// their formatting and the document's characters (a curly quote the
    /// anchor typed straight), and only the words that change are written,
    /// a new one formatted like the text before it. A comment covers the
    /// whole matched text. When a piece has no place to go, the replacement
    /// stays one edit that re-types the range, as `whole` does.
    fn replace_pieces(&self, resolved: Resolved, outcome: &mut EditOutcome) -> Vec<Resolved> {
        let Resolved::Text {
            para,
            start,
            end,
            replacement,
            comment,
            ..
        } = &resolved
        else {
            return vec![resolved];
        };
        let (para, start, end) = (*para, *start, *end);
        let ranges = rewrite::replace_ranges(&self.projections[para].text[start..end], replacement)
            .into_iter()
            .map(|(s, e, piece)| (start + s, start + e, piece));
        let Some(mut pieces) = self.checked_pieces(para, ranges) else {
            return vec![resolved];
        };
        if pieces.is_empty() {
            outcome.message = Some(
                "nothing changed: the text already reads as the replacement (a look-alike such as a curly quote stays; whole: true writes the replacement as typed)"
                    .into(),
            );
        }
        if let Some(text) = comment.clone() {
            pieces.push(Resolved::CommentRange {
                para,
                start,
                end,
                text,
            });
        }
        pieces
    }

    /// The word-level `pieces` of one replacement (paragraph offsets) as
    /// the edits that write them, each checked where it lands: a new word
    /// takes the side of the text before it, or the other side when that one
    /// is refused (a hyperlink, a move, a symbol), and a changed range must be
    /// editable. `None` when a piece has no place to go: the caller keeps the
    /// whole edit, already checked, so the pieces' refusals are dropped.
    pub(super) fn checked_pieces(
        &self,
        para: usize,
        pieces: impl IntoIterator<Item = (usize, usize, String)>,
    ) -> Option<Vec<Resolved>> {
        let projection = &self.projections[para];
        let text = &projection.text;
        let refusal = self.revision_refusal.take();
        let checked = pieces
            .into_iter()
            .map(|(start, end, replacement)| {
                let mut attach_before = text[..start]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c != '\u{FFFC}');
                let checked = if start == end {
                    self.check_insert_either(projection, start, &mut attach_before)
                } else {
                    self.check_range(projection, start, end)
                };
                checked.ok().map(|()| Resolved::Text {
                    para,
                    start,
                    end,
                    replacement,
                    comment: None,
                    attach_before,
                    format: None,
                })
            })
            .collect();
        self.revision_refusal.replace(refusal);
        checked
    }

    /// `rewrite`: the smallest word-level edits that make the paragraph read
    /// as `new_text`, each checked as a `replace` or an `insert` is.
    fn resolve_rewrite(
        &self,
        id: &str,
        paragraph: &Selector,
        new_text: &str,
        comment: Option<&str>,
    ) -> Resolution<Vec<Resolved>> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
            candidates: Vec::new(),
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        if let Some(note) = comment {
            check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        }
        if let Some(n) = comment_text_of(paragraph) {
            if comment.is_some() {
                return Err(fail(
                    "INVALID_EDIT",
                    format!(
                        "a rewrite of the text comment c{n} covers takes no comment; add one with a comment operation"
                    ),
                    outcome,
                ));
            }
            return match self.resolve_comment_text(n, new_text, &mut outcome) {
                Ok(resolved) => Ok((resolved, outcome)),
                Err((code, message)) => Err(fail(&code, message, outcome)),
            };
        }
        let para = match self.select(paragraph, false) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        if comment.is_some() && self.paragraph_story[para].0 != 0 {
            return Err(fail(
                "COMMENT_NOT_IN_BODY",
                "comments are supported in the body only: Word cannot anchor one in a header, footer or note".into(),
                outcome,
            ));
        }
        let projection = &self.projections[para];
        // The paragraph's symbols may be copied into the text; they stay.
        check_run_text(&new_text.replace('\u{FFFC}', ""))
            .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        let text = &projection.text;
        let symbols = |s: &str| s.matches('\u{FFFC}').count();
        if symbols(new_text) > symbols(text) {
            return Err(fail(
                "INVALID_EDIT",
                format!(
                    "text holds {} U+FFFC symbol placeholders but the paragraph has {} symbols; a symbol cannot be written as text",
                    symbols(new_text),
                    symbols(text)
                ),
                outcome,
            ));
        }
        let mut edits = rewrite::rewrite_ranges(text, new_text);
        // Keep emits the comparer's hunks when this paragraph has no tracked
        // change of its own, so a rewrite marks the same phrase the clean
        // path marks. A change already in the paragraph stays on the word
        // diff: comparing the document again would re-mark it (#1923).
        if !edits.is_empty()
            && self.plan.existing_revisions == ExistingRevisions::Keep
            && !Self::paragraph_has_own_revision(&self.opened.dom, self.paragraph_nodes[para])
        {
            let target = rewrite::apply_ranges(text, &edits);
            if let Some(compared) = rewrite::comparer_ranges(text, &target) {
                edits = compared;
            }
        }
        if edits.is_empty() {
            outcome.message = Some(
                "nothing changed: the paragraph already reads as the text (a tab or a break written as a space stays)"
                    .into(),
            );
        }
        let mut resolved = Vec::with_capacity(edits.len());
        for (start, end, replacement) in edits {
            // New text joins the run before it (a tab or a break takes it
            // after itself), else the run after it when a symbol, a link or
            // a field is before it.
            let mut attach_before = text[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c != '\u{FFFC}');
            let checked = if start == end {
                self.check_insert_either(projection, start, &mut attach_before)
            } else {
                self.check_range(projection, start, end)
            };
            checked.map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
            resolved.push(Resolved::Text {
                para,
                start,
                end,
                replacement,
                comment: None,
                attach_before,
                format: None,
            });
        }
        // The comment covers the rewritten paragraph, words kept and words
        // inserted at either edge included, as PlanV2's rewrite anchors it.
        if let Some(note) = comment {
            resolved.push(Resolved::CommentSpan {
                para,
                last: para,
                text: note.to_string(),
            });
        }
        outcome.matches = 1;
        outcome.context = Some(format!("{{≡ {}}}", excerpt(new_text, 60)));
        Ok((resolved, outcome))
    }

    /// `paragraph` already holds an insertion, a deletion or a move. A
    /// keep-path rewrite of it must not ask the comparer to mark the
    /// paragraph again.
    fn paragraph_has_own_revision(dom: &Dom, paragraph: NodeId) -> bool {
        dom.descendants(paragraph, None).into_iter().any(|n| {
            dom.name_is(n, &W::ins())
                || dom.name_is(n, &W::del())
                || dom.name_is(n, &W::move_from())
                || dom.name_is(n, &W::move_to())
        })
    }

    /// `list`: every selector matches one body paragraph, once; `restart:
    /// false` finds the list to continue.
    fn resolve_list(
        &self,
        id: &str,
        selectors: &[Selector],
        kind: ListKind,
        level: u32,
        restart: bool,
    ) -> Resolution<Resolved> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
            candidates: Vec::new(),
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        if selectors.is_empty() {
            return Err(fail(
                "INVALID_EDIT",
                "paragraphs must select at least one paragraph".into(),
                outcome,
            ));
        }
        if level > 8 {
            return Err(fail(
                "INVALID_EDIT",
                format!("level {level} is outside 0..=8"),
                outcome,
            ));
        }
        let mut paras: Vec<usize> = Vec::with_capacity(selectors.len());
        for (i, selector) in selectors.iter().enumerate() {
            let para = match self.select(selector, false) {
                Ok(p) => p,
                Err((code, msg, matches)) => {
                    outcome.matches = matches;
                    return Err(fail(&code, format!("paragraphs[{i}]: {msg}"), outcome));
                }
            };
            if paras.contains(&para) {
                return Err(fail(
                    "INVALID_EDIT",
                    format!("paragraphs[{i}] selects {} again", self.paragraph_id(para)),
                    outcome,
                ));
            }
            if self.paragraph_story[para].0 != 0 {
                return Err(fail(
                    "UNSUPPORTED_STRUCTURE",
                    format!(
                        "paragraphs[{i}]: list is supported in the body only, not {}",
                        self.paragraph_id(para)
                    ),
                    outcome,
                ));
            }
            paras.push(para);
        }
        outcome.matches = paras.len();
        outcome.paragraph = Some(
            paras
                .iter()
                .map(|&p| self.paragraph_id(p))
                .collect::<Vec<_>>()
                .join(", "),
        );
        let join = if restart {
            None
        } else {
            let first = paras.iter().copied().min().unwrap_or(0);
            let found = (0..first)
                .rev()
                .find_map(|p| structural::direct_num_id(&self.opened.dom, self.paragraph_nodes[p]));
            match found {
                Some(num_id) => Some(num_id),
                None => {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "restart: false continues the list of a numbered paragraph before the first one, and there is none".into(),
                        outcome,
                    ));
                }
            }
        };
        // The document's own "List Paragraph" (localized documents give it
        // another id), else the engine's, added when used.
        let styles = self.paragraph_styles();
        let (style, add_style) = styles
            .iter()
            .find(|(id, _)| id == "ListParagraph")
            .or_else(|| {
                styles
                    .iter()
                    .find(|(_, name)| name.eq_ignore_ascii_case("List Paragraph"))
            })
            .map_or(("ListParagraph".to_string(), true), |(id, _)| {
                (id.clone(), false)
            });
        let how = match &join {
            Some(num_id) => format!("continues list {num_id}"),
            None => format!("list {}", kind.name()),
        };
        outcome.context = Some(format!(
            "{{¶ {how} level {level}}} {} paragraph(s): {}",
            paras.len(),
            excerpt(&self.projections[paras[0]].text, 40)
        ));
        Ok((
            Resolved::List {
                paras,
                kind,
                level,
                join,
                style,
                add_style,
            },
            outcome,
        ))
    }

    pub(super) fn merge_partner(&self, para: usize) -> Result<usize, String> {
        let dom = &self.opened.dom;
        let node = self.paragraph_nodes[para];
        let mut next_node = dom.next_element(node);
        while let Some(n) = next_node.filter(|&n| is_range_markup(dom, n)) {
            next_node = dom.next_element(n);
        }
        let next_node = next_node
            .filter(|&n| dom.name_is(n, &W::p()))
            .ok_or_else(|| {
                "the next element is not a paragraph in the same container".to_string()
            })?;
        let next = self
            .paragraph_nodes
            .iter()
            .position(|&n| n == next_node)
            .ok_or_else(|| "the next paragraph is not addressable".to_string())?;
        let ppr = dom.element(node, &W::p_pr());
        if ppr.is_some_and(|ppr| dom.element(ppr, &W::sect_pr()).is_some()) {
            return Err("the paragraph carries section properties a merge would drop".into());
        }
        Ok(next)
    }

    /// A paragraph style id from its id or its name (case-insensitive).
    fn resolve_style(&self, requested: &str) -> Result<String, String> {
        let styles = self.paragraph_styles();
        if styles.iter().any(|(id, _)| id == requested) {
            return Ok(requested.to_string());
        }
        if let Some((id, _)) = styles
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(requested))
        {
            return Ok(id.clone());
        }
        let known: Vec<&str> = styles.iter().map(|(id, _)| id.as_str()).take(12).collect();
        Err(format!(
            "no paragraph style has id or name {requested:?}; defined: {}",
            known.join(", ")
        ))
    }

    /// `(styleId, name)` of every paragraph style in the styles part.
    pub(super) fn paragraph_styles(&self) -> Vec<(String, String)> {
        let Some(name) = self.opened.related("styles").into_iter().next() else {
            return Vec::new();
        };
        let Some(xml) = self.opened.pkg.part_string(&name) else {
            return Vec::new();
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            return Vec::new();
        };
        dom.elements(root, Some(&W::name("style")))
            .into_iter()
            .filter(|&s| dom.attribute(s, &W::type_()).unwrap_or("paragraph") == "paragraph")
            .filter_map(|s| {
                let id = dom.attribute(s, &W::name("styleId"))?.to_string();
                let name = dom
                    .element(s, &W::name("name"))
                    .and_then(|n| dom.attribute(n, &W::val()))
                    .unwrap_or("")
                    .to_string();
                Some((id, name))
            })
            .collect()
    }

    /// What this plan's own ids resolve against.
    pub(super) fn ids(&self) -> origin::Ids<'_> {
        origin::Ids {
            dom: &self.opened.dom,
            body: self.opened.body,
            story_ids: self.stories.iter().map(|s| s.id.as_str()).collect(),
            paragraph_nodes: &self.paragraph_nodes,
            paragraph_story: &self.paragraph_story,
        }
    }

    /// A plan's selector that reaches paragraph `global` by its text: its
    /// first words, or all of it, as `{"contains": …}` (with its story
    /// outside the body), when no other paragraph of its story holds them.
    /// Only a JSON plan of the first version has one.
    fn text_selector(&self, global: usize) -> Option<String> {
        if self.syntax != PlanSyntax::V1 {
            return None;
        }
        let text = &self.projections[global].text;
        let story = self.paragraph_story[global].0;
        let unique = |wanted: &str| {
            !wanted.trim().is_empty()
                && (0..self.paragraph_nodes.len())
                    .filter(|&g| self.paragraph_story[g].0 == story)
                    .filter(|&g| self.projections[g].text.contains(wanted))
                    .count()
                    == 1
        };
        let first: String = text.chars().take(SELECTOR_CHARS).collect();
        let first = match first.rfind(char::is_whitespace) {
            Some(at) if first.len() < text.len() => first[..at].trim_end(),
            _ => first.as_str(),
        };
        let contains = [first, text.as_str()]
            .into_iter()
            .find(|wanted| unique(wanted))?
            .to_string();
        let selector = Selector::Contains {
            contains,
            story: (story != 0).then(|| self.stories[story].id.clone()),
        };
        serde_json::to_string(&selector).ok()
    }

    /// The paragraph `selector` names. Positional ids name the source's
    /// paragraphs, as its author read them, even when settling revisions
    /// moved them; a paragraph merged with others is only followed when
    /// `merged_ok` (an edit that finds its text there). Text selectors read
    /// the settled text.
    fn select(&self, selector: &Selector, merged_ok: bool) -> Result<usize, origin::Miss> {
        if let Some(n) = comment_text_of(selector) {
            return self.comment_text(n).map(|covered| covered.first);
        }
        let story_index = match &self.origin {
            Some(origin) => match origin.ids().position(selector)? {
                Ok(source) => {
                    return origin.follow(
                        source,
                        merged_ok,
                        |p| self.paragraph_id(p),
                        |p| self.text_selector(p),
                    );
                }
                // The text selector's story, by name in the settled document.
                Err(_) => self.ids().position(selector)?,
            },
            None => self.ids().position(selector)?,
        };
        let story_index = match story_index {
            Ok(global) => return Ok(global),
            Err(story_index) => story_index,
        };
        let story = &self.stories[story_index].id;
        let members: Vec<usize> = (0..self.paragraph_nodes.len())
            .filter(|&g| self.paragraph_story[g].0 == story_index)
            .collect();
        let (wanted, prefix) = match selector {
            Selector::StartsWith { starts_with, .. } => (starts_with, true),
            Selector::Contains { contains, .. } => (contains, false),
            _ => unreachable!("ids and indexes are positional"),
        };
        if wanted.is_empty() {
            return Err((
                "INVALID_EDIT".into(),
                "paragraph selector text must be nonempty".into(),
                0,
            ));
        }
        let hits: Vec<usize> = members
            .into_iter()
            .filter(|&g| {
                let text = &self.projections[g].text;
                if prefix {
                    text.starts_with(wanted.as_str())
                } else {
                    text.contains(wanted.as_str())
                }
            })
            .collect();
        match hits.as_slice() {
            [one] => Ok(*one),
            [] => Err((
                "ANCHOR_NOT_FOUND".into(),
                format!("no paragraph of {story} matches {wanted:?}"),
                0,
            )),
            many => Err((
                "AMBIGUOUS_ANCHOR".into(),
                format!(
                    "{} paragraphs match {wanted:?}: {}",
                    many.len(),
                    many.iter()
                        .map(|&g| self.paragraph_id(g))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                many.len(),
            )),
        }
    }

    fn body_paragraph_count(&self) -> usize {
        self.paragraph_story.iter().filter(|(s, _)| *s == 0).count()
    }

    /// Stories an operation of the plan edits. Page setup edits the body's
    /// section properties, so that story is one of them.
    fn touched_stories(&self) -> std::collections::BTreeSet<usize> {
        self.resolved
            .iter()
            .flat_map(|(_, r)| match r {
                Resolved::PageSetup { .. } => vec![0],
                Resolved::Text { para, .. }
                | Resolved::CommentRange { para, .. }
                | Resolved::CommentSpan { para, .. }
                | Resolved::DeleteParagraph { para }
                | Resolved::FormatParagraph { para, .. }
                | Resolved::MergeParagraphs { para, .. }
                | Resolved::FormatRun { para, .. }
                | Resolved::InsertFootnote { para, .. }
                | Resolved::StrikeComment { para, .. }
                | Resolved::Restore { para, .. } => vec![self.paragraph_story[*para].0],
                Resolved::InsertParagraph { anchor, .. }
                | Resolved::InsertTable { anchor, .. }
                | Resolved::InsertImage { anchor, .. } => {
                    vec![self.paragraph_story[*anchor].0]
                }
                Resolved::List { paras, .. } => vec![self.paragraph_story[paras[0]].0],
                Resolved::Thread { stories, .. } => stories.clone(),
                Resolved::FillControl { .. } => vec![0],
            })
            .collect()
    }

    /// Ids of the paragraphs the plan deletes whole.
    pub(super) fn deleted_paragraph_ids(&self) -> HashSet<String> {
        self.resolved
            .iter()
            .filter_map(|(_, resolved)| match resolved {
                Resolved::DeleteParagraph { para } => Some(self.paragraph_id(*para)),
                _ => None,
            })
            .collect()
    }

    /// The paragraph whose mark sits right before `para`: its previous
    /// sibling, when that is a paragraph of the same story.
    fn paragraph_right_before(&self, para: usize) -> Option<usize> {
        let before = para.checked_sub(1)?;
        (self.opened.dom.next_element(self.paragraph_nodes[before])
            == Some(self.paragraph_nodes[para])
            && self.paragraph_story[before].0 == self.paragraph_story[para].0)
            .then_some(before)
    }

    /// The report when `para`'s mark is a pending `w:del`: accepting it
    /// merges `para` with a paragraph inserted after it. `None` when the
    /// mark is not a deletion.
    fn pending_deleted_mark_note(&self, para: usize) -> Option<String> {
        let node = self.paragraph_nodes[para];
        let dom = &self.opened.dom;
        let rpr = dom
            .element(node, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::r_pr()))?;
        let del = dom.elements(rpr, None).into_iter().find(|&child| {
            dom.name(child)
                .is_some_and(|name| name.namespace_name() == W::URI && name.local_name() == "del")
        })?;
        let wid = dom.attribute(del, &W::id())?;
        let (story, _) = self.paragraph_story[para];
        let rev = crate::ids::ChangeId::new(&self.stories[story].id, wid).short();
        let author = dom
            .attribute(del, &W::author())
            .unwrap_or("an unnamed author");
        let pid = self.paragraph_id(para);
        Some(format!(
            "{pid}'s paragraph mark is a pending deletion by {author} ({rev}); if it is accepted, {pid} and the new paragraph merge; restore {rev} to keep them apart"
        ))
    }

    /// The id of a paragraph ([`crate::ids::paragraph_id`]).
    #[doc(hidden)]
    pub fn paragraph_id(&self, para: usize) -> String {
        let (story, local) = self.paragraph_story[para];
        crate::ids::paragraph_id(&self.stories[story].id, local)
    }

    /// Every paragraph of the transaction's current DOM, in the fixed
    /// projection order the expectations checker reopens bytes in: the
    /// body's paragraphs in body order, then each story's own, story by
    /// story (the same collectors `load_stories` numbers paragraphs
    /// with). Structural edits shift what this walk returns; the
    /// expectations correspondence reads it off the DOM that produced the
    /// chosen primary bytes, so it never re-matches text.
    pub(super) fn paragraph_walk(&self) -> Vec<NodeId> {
        let dom = &self.opened.dom;
        let mut nodes = Vec::new();
        for (index, story) in self.stories.iter().enumerate() {
            if index == 0 {
                nodes.extend(crate::inspect::body_paragraph_nodes(dom, story.root));
            } else {
                nodes.extend(crate::inspect::story_paragraph_nodes(dom, story.root));
            }
        }
        nodes
    }

    /// The body, header, footer or note a paragraph belongs to, for the
    /// "keeps one paragraph" checks.
    fn story_container(&self, node: NodeId) -> (NodeId, &'static str) {
        let dom = &self.opened.dom;
        for (name, what) in [
            ("footnote", "a footnote"),
            ("endnote", "an endnote"),
            ("hdr", "a header"),
            ("ftr", "a footer"),
        ] {
            if let Some(&found) = dom.ancestors(node, Some(&W::name(name))).first() {
                return (found, what);
            }
        }
        (self.opened.body, "the body")
    }

    /// Paragraphs whose text `anchor` would match ([`anchor_readings`]'s
    /// ladder, the one `locate` uses), as [`AnchorCandidate`]s in document
    /// order: what an `ANCHOR_NOT_FOUND` names so its reader sees where
    /// the anchor lives (#1889). At most [`NAMED_ANCHOR_CANDIDATES`] come
    /// back; the second return counts the rest.
    fn anchor_candidates(&self, anchor: &str) -> (Vec<AnchorCandidate>, usize) {
        if let Some(cached) = self.anchor_candidate_cache.borrow().get(anchor) {
            return cached.clone();
        }
        #[cfg(test)]
        CANDIDATE_SCANS.with(|calls| calls.set(calls.get() + 1));
        // Selectors are read against the source, not the settled projection.
        // Invert the same fates the views and `select` follow, choosing one
        // source per working paragraph. Anchored edits can follow merges.
        let sources = self.origin.as_ref().map(|origin| {
            let ids = origin.ids();
            let mut sources = vec![None; self.paragraph_nodes.len()];
            for source in 0..ids.paragraph_nodes.len() {
                if let origin::Fate::Kept(post) | origin::Fate::Merged(post) = origin.fate(source) {
                    sources[post].get_or_insert_with(|| ids.id(source));
                }
            }
            sources
        });
        let mut found: Vec<AnchorCandidate> = Vec::new();
        let mut more = 0;
        for g in 0..self.paragraph_nodes.len() {
            // A paragraph without a surviving source cannot be retried by id.
            let source_id = match &sources {
                Some(sources) => match &sources[g] {
                    Some(id) => Some(id),
                    None => continue,
                },
                None => None,
            };
            let text = &self.projections[g].text;
            let Some(span) = anchor_readings(text, anchor).find_map(|hits| hits.first().copied())
            else {
                continue;
            };
            if found.len() == NAMED_ANCHOR_CANDIDATES {
                more += 1;
            } else {
                found.push(AnchorCandidate {
                    at: source_id.cloned().unwrap_or_else(|| self.paragraph_id(g)),
                    ctx: anchor_candidate_excerpt(text, span),
                });
            }
        }
        self.anchor_candidate_cache
            .borrow_mut()
            .insert(anchor.to_string(), (found.clone(), more));
        (found, more)
    }

    /// The `; it occurs in …` half of an anchor refusal (#1889): empty
    /// when the anchor occurs nowhere, so #1013's released message stands
    /// alone. Fills `outcome.candidates` for the JSON `op` event. A
    /// suggestion only — the edit stays refused, nothing is retargeted
    /// (#957).
    fn candidates_note(&self, anchor: &str, outcome: &mut EditOutcome) -> String {
        let (candidates, more) = self.anchor_candidates(anchor);
        if candidates.is_empty() {
            return String::new();
        }
        let named: Vec<String> = candidates
            .iter()
            .map(|c| format!("{} (\"{}\")", c.at, c.ctx))
            .collect();
        let mut note = format!("; it occurs in {}", named.join(", "));
        if more > 0 {
            note.push_str(&format!(", and {more} more"));
        }
        outcome.candidates = candidates;
        note
    }

    /// The unique occurrence of `find`, or its `occurrence`-th hit (1-based)
    /// when given (overlapping occurrences count), checked to lie within
    /// editable direct text.
    pub(super) fn find_range(
        &self,
        projection: &Projection,
        find: &str,
        occurrence: Option<usize>,
        outcome: &mut EditOutcome,
    ) -> Result<(usize, usize), (String, String)> {
        let (start, end) = match locate(
            projection,
            find,
            occurrence,
            outcome,
            self.anchor_window.get(),
            self.syntax,
        ) {
            Err((code, message)) if code == "ANCHOR_NOT_FOUND" => {
                return Err(match self.removed_anchor(projection, find) {
                    Some(hint) => ("UNSUPPORTED_STRUCTURE".into(), hint),
                    // #1889: under a comment's window the paragraph is the
                    // right one and the window is wrong, so there is no
                    // elsewhere to name.
                    None if self.anchor_window.get().is_some() => (code, message),
                    None => {
                        let note = self.candidates_note(find, outcome);
                        (code, format!("{message}{note}"))
                    }
                });
            }
            found => found?,
        };
        self.check_range(projection, start, end)
            .map_err(|m| ("UNSUPPORTED_STRUCTURE".to_string(), m))?;
        Ok((start, end))
    }

    /// Every byte of `[start, end)` must come from a direct run or an
    /// editable insertion: text, a tab, break or non-breaking hyphen,
    /// which an edit over it removes. A symbol is never edited: its run's font would draw the text.
    pub(super) fn check_range(
        &self,
        projection: &Projection,
        start: usize,
        end: usize,
    ) -> Result<(), String> {
        let mut covered = start;
        for seg in &projection.segments {
            if seg.end <= covered || seg.start >= end {
                continue;
            }
            if !seg.direct && editable_insertion(&self.opened.dom, &seg.piece).is_none() {
                return Err(self.refuse_opaque(run_of(&seg.piece), "the text"));
            }
            if is_symbol(projection, seg) {
                return Err("the text crosses a symbol".into());
            }
            covered = covered.max(seg.end);
        }
        if covered < end {
            return Err("the text is not addressable".into());
        }
        if projection
            .field_marks
            .iter()
            .any(|&mark| start < mark && mark < end)
        {
            return Err(
                "the text crosses a field; edit the text on either side of it, or comment on it"
                    .into(),
            );
        }
        Ok(())
    }

    /// Why `subject` (the text, the insertion point) in `run` cannot be
    /// edited, and the way around it: another author's move or deletion by
    /// id, author and text, with each route around it (#1108); else the
    /// innermost hyperlink, field or content control. An insertion is
    /// editable (#1136), so what refuses inside one is the structure
    /// around or inside it.
    fn refuse_opaque(&self, run: NodeId, subject: &str) -> String {
        let dom = &self.opened.dom;
        let revision = dom
            .ancestors(run, None)
            .into_iter()
            .take_while(|&n| !dom.name_is(n, &W::p()))
            .find(|&n| hint::Held::of(dom, n).is_some_and(|held| held != hint::Held::Inserted));
        match revision {
            Some(revision) => self.revision_hint(revision, &format!("{subject} is inside")),
            None => {
                let holders = deleted_field_revisions(&self.opened.dom, run);
                if holders.is_empty() {
                    format!("{subject} sits inside {}", self.opaque_structure(run))
                } else {
                    // The result is visible, and the refusal names the
                    // deletions of the field's characters (#1285): one hint
                    // each, then one structured refusal for all of them.
                    // The message opens with that refusal's lead and first
                    // id, as `revision_next_actions` reads it.
                    let lead = format!(
                        "{subject} sits inside the result of a field whose characters are in"
                    );
                    let lines: Vec<String> = holders
                        .iter()
                        .map(|&holder| self.revision_hint(holder, &lead))
                        .collect();
                    if self.revision_refusal.borrow().is_some() {
                        self.revision_refusal
                            .replace(Some(refusal::RevisionRefusal::new(self, &lead, &holders)));
                    }
                    lines.join("\n")
                }
            }
        }
    }

    /// The innermost structure that holds `run`, and the way around it:
    /// a hyperlink, field, content control, smart tag, custom XML markup
    /// or direction override. Anything else is reported as a field result.
    fn opaque_structure(&self, run: NodeId) -> String {
        let dom = &self.opened.dom;
        let ancestors = dom.ancestors(run, None);
        let mut inner: Option<String> = None;
        for &node in &ancestors {
            if dom.name_is(node, &W::p()) {
                break;
            }
            if inner.is_some() {
                continue;
            }
            let attr = |local: &str| dom.attribute(node, &W::name(local)).unwrap_or_default();
            if dom.name_is(node, &W::hyperlink()) {
                inner = Some(
                    "a hyperlink; edit the text on either side of it, or comment on it".into(),
                );
            } else if dom.name_is(node, &W::fld_simple()) {
                inner = Some(format!(
                    "the result of the field{}; edit the text on either side of it, or comment on it",
                    match attr("instr").trim() {
                        "" => String::new(),
                        instr => format!(" {instr:?}"),
                    }
                ));
            } else if dom.name_is(node, &W::sdt()) {
                let property = |local: &str| {
                    dom.element(node, &W::sdt_pr())
                        .and_then(|pr| dom.element(pr, &W::name(local)))
                        .and_then(|e| dom.attribute(e, &W::val()))
                        .map(|v| format!(" ({local} {v:?})"))
                };
                inner = Some(format!(
                    "a content control{}; change its text with fill_control, or comment on it",
                    property("alias")
                        .or_else(|| property("tag"))
                        .unwrap_or_default()
                ));
            } else if dom.name_is(node, &W::smart_tag()) {
                inner = Some(
                    "a smart tag (`w:smartTag`); edit the text on either side of it, or comment on it"
                        .into(),
                );
            } else if dom.name_is(node, &W::custom_xml()) {
                inner = Some(
                    "custom XML markup (`w:customXml`); edit the text on either side of it, or comment on it"
                        .into(),
                );
            } else if dom.name_is(node, &W::dir()) || dom.name_is(node, &W::bdo()) {
                inner = Some(
                    "a direction override (`w:dir`/`w:bdo`); edit the text on either side of it, or comment on it"
                        .into(),
                );
            }
        }
        inner.unwrap_or_else(|| {
            "the result of a field; edit the text on either side of it, or comment on it".into()
        })
    }

    /// The refusal for an edit `revision` holds (a `w:ins`, `w:del`,
    /// `w:moveTo`, `w:moveFrom`, `w:pPrChange` or `w:rPrChange`): `lead`
    /// says what was refused up to the revision's id, then the revision and
    /// its routes in the plan's syntax.
    pub(super) fn revision_hint(&self, revision: NodeId, lead: &str) -> String {
        let dom = &self.opened.dom;
        let ancestors = dom.ancestors(revision, None);
        let story = self.stories.iter().find(|s| ancestors.contains(&s.root));
        // The same revision as data (`revision_refusals`, #1147), beside
        // the routes the message spells out.
        if story.is_some() {
            self.revision_refusal
                .replace(Some(refusal::RevisionRefusal::new(self, lead, &[revision])));
        }
        let wid = dom.attribute(revision, &W::id()).unwrap_or_default();
        let story_id = story.map_or("body", |s| s.id.as_str());
        let id = crate::ids::ChangeId::new(story_id, wid).short();
        let local = dom.name(revision).map(|n| n.local_name().to_string());
        let long_id = v2_apply::revision_long_id(
            story.map_or("word/document.xml", |s| s.part.as_str()),
            local.as_deref().unwrap_or("ins"),
            story_id,
            wid,
        );
        let held = hint::Held::of(dom, revision).unwrap_or(hint::Held::Inserted);
        // A formatting change holds the text of the run or paragraph it
        // formats; a mark revision holds none.
        let owner = if held == hint::Held::Formatted {
            ancestors
                .iter()
                .copied()
                .find(|&n| dom.name_is(n, &W::r()) || dom.name_is(n, &W::p()))
                .unwrap_or(revision)
        } else {
            revision
        };
        let text: String = dom
            .descendants(owner, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
            .map(|n| dom.value(n))
            .collect();
        let paragraph = ancestors
            .iter()
            .find(|&&n| dom.name_is(n, &W::p()))
            .and_then(|p| self.paragraph_nodes.iter().position(|n| n == p))
            .map_or_else(String::new, |para| self.paragraph_id(para));
        let author = dom.attribute(revision, &W::author()).unwrap_or_default();
        // Restore takes another author's deletion in one stretch.
        let restorable = held == hint::Held::Deleted
            && author != self.plan.author
            && story.is_some_and(|s| {
                let stretches = dom
                    .descendants(s.root, Some(&W::del()))
                    .into_iter()
                    .filter(|&n| {
                        dom.attribute(n, &W::id()) == Some(wid)
                            && dom.parent(n).is_some_and(|p| !dom.name_is(p, &W::r_pr()))
                    })
                    .count();
                visible::restorable(dom, revision, stretches)
            });
        let holder = hint::Holder {
            held,
            id: &id,
            long_id: &long_id,
            author,
            actor: self.plan.author.as_str(),
            keep_others: self.plan.others.keeps(),
            text: &text,
            paragraph: &paragraph,
            lead,
            restorable,
        };
        // A restore of the same deletion in this command lands first; the
        // edit of its words waits for the next command.
        let restored_here = held == hint::Held::Deleted
            && self.plan.operations.iter().any(|op| {
                matches!(&op.kind, OperationKind::Restore { revision, .. }
                    if crate::ids::parse_change_ids(revision.trim())
                        .is_ok_and(|ids| ids.iter().any(|c| c.short() == id)))
            });
        if restored_here {
            return hint::restored_here_hint(&holder);
        }
        hint::settle_hint(&holder, self.syntax)
    }

    /// When `find` misses `projection`'s visible text but reaches into text
    /// another author deleted or moved away, the refusal that names that
    /// revision; the anchor is otherwise just not there.
    fn removed_anchor(&self, projection: &Projection, find: &str) -> Option<String> {
        // The projections are the transaction's own, one per paragraph.
        let para = self
            .projections
            .iter()
            .position(|p| std::ptr::eq(p, projection))?;
        self.removed_anchor_in(para, find)
    }

    /// The short id (`r4`) of the deletion or move [`Self::removed_anchor_in`]
    /// would name. Only the typed inline actions ask (`edit::inline`, a
    /// `cli` module), so a library built without `cli` leaves it out.
    #[doc(hidden)]
    pub fn removed_anchor_holder(&self, para: usize, find: &str) -> Option<String> {
        let dom = &self.opened.dom;
        let revision = hint::removed_holder(dom, self.paragraph_nodes[para], find)?;
        let story = self
            .stories
            .iter()
            .find(|story| dom.ancestors(revision, None).contains(&story.root));
        let wid = dom.attribute(revision, &W::id()).unwrap_or_default();
        let story_id = story.map_or("body", |story| story.id.as_str());
        Some(crate::ids::ChangeId::new(story_id, wid).short())
    }

    /// [`Self::removed_anchor`] for paragraph `para`.
    #[doc(hidden)]
    pub fn removed_anchor_in(&self, para: usize, find: &str) -> Option<String> {
        let dom = &self.opened.dom;
        hint::removed_holder(dom, self.paragraph_nodes[para], find)
            .map(|revision| self.revision_hint(revision, "the text is inside"))
    }

    /// [`Self::check_insert_position`] on the side `attach_before` names,
    /// else on the other, which `attach_before` then names. A first side
    /// that failed leaves no refusal behind when the other holds: its
    /// revision did not block the edit (#1308 review).
    pub(super) fn check_insert_either(
        &self,
        projection: &Projection,
        pos: usize,
        attach_before: &mut bool,
    ) -> Result<(), String> {
        self.check_insert_position(projection, pos, *attach_before)
            .or_else(|first| {
                self.check_insert_position(projection, pos, !*attach_before)
                    .map(|()| {
                        *attach_before = !*attach_before;
                        self.revision_refusal.take();
                    })
                    .map_err(|_| first)
            })
    }

    pub(super) fn check_insert_position(
        &self,
        projection: &Projection,
        pos: usize,
        attach_before: bool,
    ) -> Result<(), String> {
        if projection.text.is_empty() {
            // Typed as Word types it: a run formatted as the mark.
            return Ok(());
        }
        let seg = attach_segment(projection, pos, attach_before)
            .ok_or_else(|| "no run holds the insertion point".to_string())?;
        if !seg.direct && editable_insertion(&self.opened.dom, &seg.piece).is_none() {
            return Err(self.refuse_opaque(run_of(&seg.piece), "the insertion point"));
        }
        if is_symbol(projection, seg) {
            return Err("the insertion point touches a symbol".into());
        }
        Ok(())
    }

    pub(super) fn check_conflicts(&self) -> Result<(), EditError> {
        self.check_conflicts_with_comments(false)
    }

    pub(super) fn check_conflicts_v2(&self) -> Result<(), EditError> {
        self.check_conflicts_with_comments(true)
    }

    fn check_conflicts_with_comments(&self, v2: bool) -> Result<(), EditError> {
        let comment_conflict = |i, message| {
            let mut error = self.conflict(i, message);
            if v2 {
                error.code = COMMENT_ANCHOR_CONFLICT.into();
                error.outcomes[i].code = Some(error.code.clone());
            }
            error
        };
        let mut deleted: Vec<usize> = Vec::new();
        for (i, r) in &self.resolved {
            if let Resolved::DeleteParagraph { para } = r {
                if deleted.contains(para) {
                    return Err(self.conflict(*i, "deletes a paragraph another operation deletes"));
                }
                deleted.push(*para);
            }
        }
        self.check_paragraph_ops(&deleted)?;
        self.check_control_conflicts(&deleted)?;
        self.check_comment_ids_fit()?;
        self.check_thread_ops()?;
        let mut ranges: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
        for (&i, &(para, start, end)) in &self.claims {
            if deleted.contains(&para) {
                return Err(self.conflict(i, "edits text of a deleted paragraph"));
            }
            ranges.entry(para).or_default().push((start, end, i));
        }
        for (i, r) in &self.resolved {
            let (para, start, end) = match r {
                // A replace's pieces lie inside the range it claims.
                Resolved::Text { .. } if self.claims.contains_key(i) => continue,
                Resolved::Text {
                    para, start, end, ..
                } => (*para, *start, *end),
                // A restore inserts its copy where the deletion is.
                Resolved::Restore { para, at, .. } => (*para, *at, *at),
                Resolved::CommentRange { para, .. } => {
                    if deleted.contains(para) {
                        return Err(comment_conflict(*i, "comments on a deleted paragraph"));
                    }
                    continue;
                }
                // A new paragraph beside a deleted one takes its place.
                Resolved::InsertParagraph { .. } => continue,
                Resolved::InsertImage { anchor, .. } => {
                    if deleted.contains(anchor) {
                        return Err(self.conflict(*i, "anchors a new image on a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::InsertTable { anchor, .. } => {
                    if deleted.contains(anchor) {
                        return Err(self.conflict(*i, "anchors a new table on a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::FormatRun { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "formats text of a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::InsertFootnote { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "adds a footnote to a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::CommentSpan { para, last, .. } => {
                    // V2 has tested preservation across interior deletions;
                    // the adapter separately checks content and endpoints.
                    let removes_anchor = if v2 {
                        deleted.contains(para) || deleted.contains(last)
                    } else {
                        (*para..=*last).any(|p| deleted.contains(&p))
                    };
                    if removes_anchor {
                        return Err(comment_conflict(*i, "comments on a deleted paragraph"));
                    }
                    continue;
                }
                // A reply to a comment whose paragraph the plan deletes:
                // the tracked deletion keeps the reference in struck text,
                // where Word shows the thread (#1159). Editing mode removes
                // the anchor, which v2's anchor check refuses.
                Resolved::DeleteParagraph { .. }
                | Resolved::FormatParagraph { .. }
                | Resolved::MergeParagraphs { .. }
                | Resolved::List { .. }
                | Resolved::PageSetup { .. }
                | Resolved::Thread { .. }
                | Resolved::FillControl { .. }
                | Resolved::StrikeComment { .. } => continue,
            };
            if deleted.contains(&para) {
                return Err(self.conflict(*i, "edits text of a deleted paragraph"));
            }
            ranges.entry(para).or_default().push((start, end, *i));
        }
        for list in ranges.values_mut() {
            list.sort_by_key(|&(s, e, i)| (s, e, i));
            for &[(s0, e0, _), (s1, e1, i1)] in list.array_windows() {
                let strictly_inside = s1 > s0 && s1 < e0;
                let same_start_nonempty = s1 == s0 && e0 > s0 && e1 > s1;
                if strictly_inside || same_start_nonempty {
                    return Err(self.conflict(i1, "overlaps an earlier edit's text range"));
                }
            }
        }
        // A separate comment wholly on words a deletion removes has nowhere
        // to sit on those words. The deletion's own comment keeps the note
        // there. A comment around more than the deletion keeps the words
        // that stay; one that crosses its edge is refused below. v2 refuses
        // this shape after the conflict checks, as COMMENT_ANCHOR_REMOVED.
        for (i, r) in self.resolved.iter().filter(|_| !v2) {
            let Resolved::CommentRange {
                para, start, end, ..
            } = r
            else {
                continue;
            };
            let Some(deleter) = self.resolved.iter().find_map(|(j, text)| match text {
                Resolved::Text {
                    para: p,
                    start: s,
                    end: e,
                    replacement,
                    ..
                } if p == para
                    && replacement.is_empty()
                    && *e > *s
                    && *s <= *start
                    && *end <= *e =>
                {
                    Some(*j)
                }
                _ => None,
            }) else {
                continue;
            };
            let message = {
                let delete_id = &self.outcomes[deleter].id;
                let comment_id = &self.outcomes[*i].id;
                format!(
                    "{comment_id} comments on words {delete_id} deletes; give {delete_id} a \"comment\" instead"
                )
            };
            return Err(self.conflict(*i, &message));
        }
        // A comment boundary strictly inside a changed range has no position
        // in the edited text. Containing or touching an edit is fine.
        for (i, r) in &self.resolved {
            let Resolved::CommentRange {
                para, start, end, ..
            } = r
            else {
                continue;
            };
            let cuts =
                ranges.get(para).into_iter().flatten().any(|&(s, e, _)| {
                    e > s && ((*start > s && *start < e) || (*end > s && *end < e))
                });
            if cuts {
                return Err(comment_conflict(
                    *i,
                    "comment range cuts through an edited range",
                ));
            }
        }
        // Formatting applies to source text: no text edit may change it.
        for (i, r) in &self.resolved {
            let Resolved::FormatRun {
                para, start, end, ..
            } = r
            else {
                continue;
            };
            let overlaps = ranges
                .get(para)
                .into_iter()
                .flatten()
                .any(|&(s, e, _)| runs::format_overlaps_edit((*start, *end), (s, e)));
            if overlaps {
                return Err(self.conflict(*i, "formats text another operation changes"));
            }
        }
        // One page setup per section.
        let setups: Vec<(usize, &sections::Targets)> = self
            .resolved
            .iter()
            .filter_map(|(i, r)| match r {
                Resolved::PageSetup { targets } => Some((*i, targets)),
                _ => None,
            })
            .collect();
        for (n, (i, targets)) in setups.iter().enumerate() {
            if setups[..n]
                .iter()
                .any(|(_, earlier)| sections::targets_overlap(earlier, targets))
            {
                return Err(self.conflict(*i, "sets up a section another operation sets up"));
            }
        }
        // A reference mark needs its anchor's end to survive the text edits.
        for (i, r) in &self.resolved {
            let Resolved::InsertFootnote { para, at, .. } = r else {
                continue;
            };
            let inside = ranges
                .get(para)
                .into_iter()
                .flatten()
                .any(|&(s, e, _)| e > s && runs::format_overlaps_edit((*at, *at), (s, e)));
            if inside {
                return Err(
                    self.conflict(*i, "puts a footnote inside text another operation changes")
                );
            }
        }
        Ok(())
    }

    /// Formatting and merging, checked against each other, against deletions
    /// and against paragraph insertions.
    fn check_paragraph_ops(&self, deleted: &[usize]) -> Result<(), EditError> {
        let mut formatted: Vec<usize> = Vec::new();
        let mut merge_heads: Vec<usize> = Vec::new();
        let mut merge_tails: Vec<usize> = Vec::new();
        for (i, r) in &self.resolved {
            match r {
                Resolved::FormatParagraph { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "formats a deleted paragraph"));
                    }
                    if formatted.contains(para) {
                        return Err(
                            self.conflict(*i, "formats a paragraph another operation formats")
                        );
                    }
                    formatted.push(*para);
                }
                Resolved::MergeParagraphs { para, next, .. } => {
                    if deleted.contains(para) || deleted.contains(next) {
                        return Err(self.conflict(*i, "merges a deleted paragraph"));
                    }
                    if merge_heads.contains(para) {
                        return Err(
                            self.conflict(*i, "merges a paragraph another operation merges")
                        );
                    }
                    merge_heads.push(*para);
                    merge_tails.push(*next);
                }
                _ => {}
            }
        }
        for (i, r) in &self.resolved {
            match r {
                Resolved::FormatParagraph { para, .. } if merge_heads.contains(para) => {
                    return Err(
                        self.conflict(*i, "formats a paragraph whose properties a merge discards")
                    );
                }
                Resolved::InsertParagraph { anchor, side, .. }
                | Resolved::InsertImage { anchor, side, .. }
                    if (*side == Side::After && merge_heads.contains(anchor))
                        || (*side == Side::Before && merge_tails.contains(anchor)) =>
                {
                    return Err(self.conflict(*i, "inserts a paragraph between two a merge joins"));
                }
                Resolved::InsertTable { anchor, side, .. }
                    if (*side == Side::After && merge_heads.contains(anchor))
                        || (*side == Side::Before && merge_tails.contains(anchor)) =>
                {
                    return Err(self.conflict(*i, "inserts a table between two a merge joins"));
                }
                _ => {}
            }
        }
        let mut listed: Vec<usize> = Vec::new();
        for (i, r) in &self.resolved {
            let Resolved::List { paras, .. } = r else {
                continue;
            };
            for para in paras {
                let message = if deleted.contains(para) {
                    "numbers a deleted paragraph"
                } else if listed.contains(para) {
                    "numbers a paragraph another list operation numbers"
                } else if formatted.contains(para) {
                    "numbers a paragraph another operation formats"
                } else if merge_heads.contains(para) {
                    "numbers a paragraph whose properties a merge discards"
                } else {
                    listed.push(*para);
                    continue;
                };
                return Err(self.conflict(*i, message));
            }
        }
        Ok(())
    }

    /// The deleted paragraphs a plan empties instead of removing. Word
    /// never deletes the mark of a container's final paragraph (a table
    /// cell's, the body's, a header's, a footer's or a note's): when no new
    /// paragraph takes that mark and no paragraph before it can carry it
    /// away (`final_carrier`), the final paragraph stays, its content gone
    /// (#1668).
    pub(super) fn emptied_paragraphs(&self) -> BTreeSet<usize> {
        self.resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::DeleteParagraph { para } => Some(*para),
                _ => None,
            })
            .filter(|&para| {
                self.final_replacement(para).is_none() && self.final_carrier(para).is_none() && {
                    let node = self.paragraph_nodes[para];
                    self.final_paragraph(node) == Some(node)
                }
            })
            .collect()
    }

    /// The paragraph that carries a deleted container-final paragraph's
    /// mark away: its own mark deleted, so the final one's content joins
    /// it, as Word writes the deletion of a last paragraph. That is the
    /// nearest paragraph before it among its siblings (past range markers
    /// and the paragraphs the plan deletes) that the plan keeps, when its
    /// mark is not already deleted: carrying the mark past another
    /// author's deleted mark would join two paragraphs the plan keeps
    /// (MiniMax on #1670, round 8). `None` there, when a table, a content
    /// control or the start of its parent comes first, and when the plan
    /// places a block in between, which the carried mark would join (glm-5.3
    /// on #1670, round 8): nothing can carry the mark.
    pub(super) fn final_carrier(&self, para: usize) -> Option<NodeId> {
        let dom = &self.opened.dom;
        let gone: Vec<NodeId> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::DeleteParagraph { para } => Some(self.paragraph_nodes[*para]),
                _ => None,
            })
            .collect();
        // Deleted, or moved away (crush on #1670, round 8).
        let deleted_mark = |p: NodeId| {
            dom.element(p, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                .is_some_and(|rpr| {
                    dom.element(rpr, &W::del()).is_some()
                        || dom.element(rpr, &W::move_from()).is_some()
                })
        };
        let placed = |anchor: usize, wanted: Side| {
            self.resolved.iter().any(|(_, r)| {
                matches!(r,
                    Resolved::InsertParagraph { anchor: a, side, .. }
                    | Resolved::InsertTable { anchor: a, side, .. }
                        if *a == anchor && *side == wanted)
            })
        };
        if placed(para, Side::Before) {
            return None;
        }
        let mut cursor = self.paragraph_nodes[para];
        loop {
            let previous = previous_block(dom, cursor)?;
            if !dom.name_is(previous, &W::p()) {
                return None;
            }
            let index = self.paragraph_nodes.iter().position(|&n| n == previous);
            if index.is_some_and(|i| placed(i, Side::After)) {
                return None;
            }
            if !gone.contains(&previous) {
                return (!deleted_mark(previous)).then_some(previous);
            }
            if index.is_some_and(|i| placed(i, Side::Before)) {
                return None;
            }
            cursor = previous;
        }
    }

    /// The final paragraph of `node`'s container (a table cell or a story):
    /// its last paragraph, through the content controls, custom XML
    /// elements and the alternate-content branch Word shows that may hold
    /// it, whatever table follows it in a cell. Its mark is the one Word
    /// never deletes.
    pub(super) fn final_paragraph(&self, node: NodeId) -> Option<NodeId> {
        let dom = &self.opened.dom;
        crate::validate::container_blocks(dom, self.paragraph_container(node))
            .into_iter()
            .rev()
            .find(|&b| dom.name_is(b, &W::p()))
    }

    /// The table cell `node` sits in, or else its story (body, header,
    /// footer or note).
    fn paragraph_container(&self, node: NodeId) -> NodeId {
        match self.opened.dom.ancestors(node, Some(&W::tc())).first() {
            Some(&cell) => cell,
            None => self.story_container(node).0,
        }
    }

    /// The operation whose new paragraph takes a deleted container-final
    /// paragraph's mark: the last new paragraph placed after it, else the
    /// last placed before it (MiniMax on #1670, round 8); with nothing placed beside it, the last block
    /// placed after the paragraph before it, when that block is a paragraph
    /// (codex on #1670), back through the paragraphs the plan deletes and
    /// what is placed before them (codex on #1670, round 8). `None` for a
    /// paragraph that is not its container's final one, or that no new
    /// paragraph replaces.
    pub(super) fn final_replacement(&self, para: usize) -> Option<usize> {
        let dom = &self.opened.dom;
        let node = self.paragraph_nodes[para];
        if self.final_paragraph(node) != Some(node) {
            return None;
        }
        // The blocks the plan places beside `anchor` on `wanted`, in plan
        // order (the placement order on either side), and whether each is
        // a paragraph.
        let placed = |anchor: usize, wanted: Side| -> Vec<(usize, bool)> {
            self.resolved
                .iter()
                .filter_map(|(i, r)| match r {
                    Resolved::InsertParagraph {
                        anchor: a, side, ..
                    } if *a == anchor && *side == wanted => Some((*i, true)),
                    Resolved::InsertTable {
                        anchor: a, side, ..
                    } if *a == anchor && *side == wanted => Some((*i, false)),
                    _ => None,
                })
                .collect()
        };
        let last_paragraph =
            |blocks: &[(usize, bool)]| blocks.iter().rev().find(|(_, p)| *p).map(|(i, _)| *i);
        let before = placed(para, Side::Before);
        if let Some(op) = last_paragraph(&placed(para, Side::After)).or(last_paragraph(&before)) {
            return Some(op);
        }
        if !before.is_empty() {
            return None;
        }
        let gone: BTreeSet<usize> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::DeleteParagraph { para } => Some(*para),
                _ => None,
            })
            .collect();
        let mut cursor = node;
        loop {
            let previous = previous_block(dom, cursor)
                .filter(|&n| dom.name_is(n, &W::p()))
                .and_then(|p| self.paragraph_nodes.iter().position(|&n| n == p))?;
            // The last block placed after it ends the container; with none,
            // a kept paragraph does, and a deleted one gives way to what is
            // placed before it.
            if let Some(&(op, paragraph)) = placed(previous, Side::After).last() {
                return paragraph.then_some(op);
            }
            if !gone.contains(&previous) {
                return None;
            }
            if let Some(&(op, paragraph)) = placed(previous, Side::Before).last() {
                return paragraph.then_some(op);
            }
            cursor = self.paragraph_nodes[previous];
        }
    }

    /// Thread operations against each other: nothing may act on a comment
    /// another operation deletes, and a comment is edited once.
    fn check_thread_ops(&self) -> Result<(), EditError> {
        let family = self.family.as_ref().expect("loaded in start");
        let mut gone: Vec<(usize, Vec<u32>)> = Vec::new();
        let mut edited: Vec<u32> = Vec::new();
        for (i, r) in &self.resolved {
            let Resolved::Thread { op, .. } = r else {
                continue;
            };
            match op {
                ThreadOp::Delete { id } => gone.push((*i, family.with_replies(*id))),
                ThreadOp::Edit { id, .. } => {
                    if edited.contains(id) {
                        return Err(self.conflict(*i, "edits a comment another operation edits"));
                    }
                    edited.push(*id);
                }
                _ => {}
            }
        }
        for (i, r) in &self.resolved {
            let Resolved::Thread { op, .. } = r else {
                continue;
            };
            let target = op.target();
            if gone.iter().any(|(j, ids)| j != i && ids.contains(&target)) {
                return Err(self.conflict(*i, "acts on a comment another operation deletes"));
            }
        }
        Ok(())
    }

    /// New comments take ids after the source's highest; refuse a plan whose
    /// comments would not fit in `w:id`'s 32 bits.
    fn check_comment_ids_fit(&self) -> Result<(), EditError> {
        let needed = self.comments_planned() as u64;
        if needed > 0 && self.next_comment_id + needed - 1 > u64::from(u32::MAX) {
            return Err(err(
                "INVALID_DOCUMENT",
                None,
                "the source's comment ids leave no room for new comments",
            ));
        }
        Ok(())
    }

    /// How many comments the resolved operations write: a comment on a
    /// text edit, an inserted or deleted paragraph, a comment operation and
    /// a reply each write one.
    fn comments_planned(&self) -> usize {
        self.resolved
            .iter()
            .filter(|(i, r)| match r {
                Resolved::Text { comment, .. } | Resolved::InsertParagraph { comment, .. } => {
                    comment.is_some()
                }
                Resolved::CommentRange { .. } | Resolved::CommentSpan { .. } => true,
                Resolved::Thread { op, .. } => matches!(op, ThreadOp::Reply { .. }),
                Resolved::DeleteParagraph { .. } | Resolved::StrikeComment { .. } => {
                    self.deletion_comment(*i).is_some()
                }
                Resolved::Restore { comment, .. } => comment.is_some(),
                Resolved::FormatParagraph { .. }
                | Resolved::MergeParagraphs { .. }
                | Resolved::InsertTable { .. }
                | Resolved::List { .. }
                | Resolved::FillControl { .. }
                | Resolved::FormatRun { .. }
                | Resolved::InsertFootnote { .. }
                | Resolved::InsertImage { .. }
                | Resolved::PageSetup { .. } => false,
            })
            .count()
    }

    /// Check a `settings` operation: something to write, no password, and
    /// one per plan.
    fn resolve_settings(
        &self,
        id: &str,
        request: &crate::settings::SettingsRequest,
    ) -> Result<EditOutcome, Box<(EditError, EditOutcome)>> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
            candidates: Vec::new(),
        };
        let fail = |code: &str, msg: &str, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        if *request == crate::settings::SettingsRequest::default() {
            return Err(fail(
                "INVALID_EDIT",
                "settings needs track_revisions, update_fields or protection",
                outcome,
            ));
        }
        if request
            .protection
            .as_ref()
            .is_some_and(|p| p.password.is_some())
        {
            return Err(fail(
                "UNSUPPORTED",
                "a protection password is not written: Word's legacy hash needs w:cryptProviderType, w:cryptAlgorithmSid, a spin count and a salt; leave the password out to enforce the restriction without one",
                outcome,
            ));
        }
        if self.settings.is_some() {
            return Err(fail(
                "OVERLAPPING_EDITS",
                "one settings operation per plan; put every setting in the first",
                outcome,
            ));
        }
        outcome.matches = 1;
        outcome.context = Some(request.describe());
        Ok(outcome)
    }

    /// Refuse the plan when the text of a redaction still occurs in one of
    /// `outputs` (a comment on it, another paragraph, a header, the
    /// document properties), or when a part is in an encoding the scan
    /// does not certify. The message names the parts, never the text.
    pub(super) fn check_redactions(&self, outputs: &[&[u8]]) -> Result<(), EditError> {
        for (op, operation) in self.plan.operations.iter().enumerate() {
            let OperationKind::Redact { find, .. } = &operation.kind else {
                continue;
            };
            let mut parts = std::collections::BTreeSet::new();
            for doc in outputs {
                match crate::scrub::leak_scan(doc, find) {
                    crate::scrub::LeakScan::Incomplete { part, reason } => {
                        let message =
                            format!("{part} ({reason}); the redaction scan is incomplete");
                        let mut error = self.conflict(op, &message);
                        error.code = "REDACTION_INCOMPLETE".into();
                        error.outcomes[op].code = Some(error.code.clone());
                        return Err(error);
                    }
                    crate::scrub::LeakScan::Complete(found) => parts.extend(found),
                }
            }
            if parts.is_empty() {
                continue;
            }
            let message = format!(
                "the redacted text still occurs in {}; redact every copy in the same plan, or remove the comment or part that holds it",
                parts.into_iter().collect::<Vec<_>>().join(", ")
            );
            let mut error = self.conflict(op, &message);
            error.code = "REDACTION_LEAK".into();
            error.outcomes[op].code = Some(error.code.clone());
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn conflict(&self, op_index: usize, message: &str) -> EditError {
        let id = self.outcomes[op_index].id.clone();
        let mut e = err("OVERLAPPING_EDITS", Some(&id), message);
        let mut outcomes = self.outcomes.clone();
        outcomes[op_index].status = "failed".into();
        outcomes[op_index].code = Some(e.code.clone());
        outcomes[op_index].message = Some(message.to_string());
        e.outcomes = outcomes;
        e
    }

    /// Mutate the copy. Resolution succeeded, so every step here is total.
    fn apply_comments(&mut self) -> BTreeMap<usize, u32> {
        // 0. Comment ids follow plan order, whatever paragraph they land in.
        let mut ids: BTreeMap<usize, u32> = BTreeMap::new();
        let comments: Vec<(usize, String)> =
            self.resolved
                .iter()
                .filter_map(|(i, r)| {
                    let text = match r {
                        Resolved::Text { comment, .. }
                        | Resolved::InsertParagraph { comment, .. } => comment.clone(),
                        Resolved::CommentRange { text, .. }
                        | Resolved::CommentSpan { text, .. } => Some(text.clone()),
                        Resolved::Thread {
                            op: ThreadOp::Reply { text, .. },
                            ..
                        } => Some(text.clone()),
                        Resolved::DeleteParagraph { .. } | Resolved::StrikeComment { .. } => {
                            self.deletion_comment(*i).map(str::to_string)
                        }
                        Resolved::Restore { comment, .. } => comment.clone(),
                        Resolved::FormatParagraph { .. }
                        | Resolved::MergeParagraphs { .. }
                        | Resolved::InsertTable { .. }
                        | Resolved::List { .. }
                        | Resolved::FormatRun { .. }
                        | Resolved::InsertFootnote { .. }
                        | Resolved::InsertImage { .. }
                        | Resolved::PageSetup { .. }
                        | Resolved::Thread { .. }
                        | Resolved::FillControl { .. } => None,
                    };
                    text.map(|t| (*i, t))
                })
                .collect();
        for (i, text) in comments {
            let id = if self.deletion_comment(i).is_some() || self.reply_in_deleted(i) {
                // Written into the source copy the comparer reads, not here.
                let id = self.reserve_comment_id();
                self.deletion_comments.push((id, i));
                id
            } else {
                self.new_comment(text)
            };
            ids.insert(i, id);
            self.outcomes[i].comment_id = Some(id);
        }
        for (i, r) in &self.resolved {
            if let Resolved::Thread {
                op: ThreadOp::Reply { parent, .. },
                ..
            } = r
            {
                self.reply_parents.insert(ids[i], *parent);
            }
        }
        ids
    }

    fn apply_text_edits(&mut self, ids: &BTreeMap<usize, u32>) -> Result<(), EditError> {
        // 1. Text edits and their comments, paragraph by paragraph.
        let mut by_para: BTreeMap<usize, Vec<ScheduledEdit>> = BTreeMap::new();
        let mut comment_ranges: BTreeMap<usize, Vec<(usize, usize, usize, String)>> =
            BTreeMap::new();
        let mut format_runs: BTreeMap<usize, Vec<(usize, usize, RunFormat)>> = BTreeMap::new();
        let mut references: BTreeMap<usize, Vec<(usize, u32)>> = BTreeMap::new();
        let reference_style = self
            .style_defined("FootnoteReference")
            .then_some("FootnoteReference");
        let notes: Vec<(usize, usize, String)> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::InsertFootnote { para, at, text } => Some((*para, *at, text.clone())),
                _ => None,
            })
            .collect();
        if !notes.is_empty() {
            let story = self.footnotes_story()?;
            self.notes_story = Some(story);
            let root = self.stories[story].root;
            let text_style = self.style_defined("FootnoteText").then_some("FootnoteText");
            let mut id = notes::next_footnote_id(&self.opened.dom, root);
            for (para, at, text) in notes {
                notes::append_footnote(
                    &mut self.opened.dom,
                    root,
                    id,
                    &text,
                    (text_style, reference_style),
                );
                references.entry(para).or_default().push((at, id));
                id = id.saturating_add(1);
            }
        }
        for (i, r) in &self.resolved {
            match r {
                Resolved::Text {
                    para,
                    start,
                    end,
                    replacement,
                    attach_before,
                    comment,
                    format,
                } => by_para.entry(*para).or_default().push(ScheduledEdit {
                    start: *start,
                    end: *end,
                    op: *i,
                    replacement: replacement.clone(),
                    attach_before: *attach_before,
                    comment: comment.clone(),
                    format: format.clone(),
                }),
                Resolved::CommentRange {
                    para,
                    start,
                    end,
                    text,
                } => {
                    comment_ranges
                        .entry(*para)
                        .or_default()
                        .push((*start, *end, *i, text.clone()));
                }
                Resolved::FormatRun {
                    para,
                    start,
                    end,
                    format,
                } => {
                    format_runs
                        .entry(*para)
                        .or_default()
                        .push((*start, *end, format.clone()));
                }
                _ => {}
            }
        }
        let touched: Vec<usize> = by_para
            .keys()
            .chain(comment_ranges.keys())
            .chain(format_runs.keys())
            .chain(references.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        for para in touched {
            let node = self.paragraph_nodes[para];
            let mut edits = by_para.remove(&para).unwrap_or_default();
            let mut empty = project_paragraph(&self.opened.dom, node)
                .text
                .is_empty()
                .then(EmptyPlacement::default);
            sort_edits(&mut edits, empty.is_some());
            // Apply in reverse so earlier source offsets stay valid.
            for edit in edits.iter().rev() {
                let projection = project_paragraph(&self.opened.dom, node);
                // The insertions the edit reaches into: one it empties goes,
                // as in the redline.
                let mut carriers: Vec<NodeId> = projection
                    .segments
                    .iter()
                    .filter(|s| s.start < edit.end && edit.start < s.end)
                    .filter_map(|s| {
                        self.opened
                            .dom
                            .parent(run_of(&s.piece))
                            .filter(|&n| self.opened.dom.name_is(n, &W::ins()))
                    })
                    .collect();
                carriers.dedup();
                if let Some(placement) = empty.as_mut() {
                    if !edit.replacement.is_empty() {
                        let run = mark_run(&mut self.opened.dom, node);
                        placement.place(&mut self.opened.dom, node, run, edit.attach_before);
                        write_mark_run_text(&mut self.opened.dom, run, &edit.replacement);
                    }
                } else {
                    // The runs the edit takes words from.
                    let reached: Vec<NodeId> = projection
                        .segments
                        .iter()
                        .filter(|s| s.start < edit.end && edit.start < s.end)
                        .map(|s| run_of(&s.piece))
                        .collect();
                    apply_text_edit(
                        &mut self.opened.dom,
                        &projection,
                        edit.start,
                        edit.end,
                        &edit.replacement,
                        edit.attach_before,
                    );
                    // Nothing before the edit has moved yet: its new words
                    // stand at its own start.
                    clean_new_words(
                        &mut self.opened.dom,
                        node,
                        edit.start,
                        edit.start + edit.replacement.len(),
                        &mut self.revision_ids,
                    );
                    drop_emptied_records(&mut self.opened.dom, node, &reached, &self.revision_ids);
                }
                for ins in carriers {
                    tracked::remove_empty_insertion(&mut self.opened.dom, ins);
                }
            }
            // Formatting of new text, in new coordinates.
            for edit in &edits {
                if let Some(format) = &edit.format {
                    let s = new_position(&edits, edit.start, true, Some(edit.op));
                    format_range(
                        &mut self.opened.dom,
                        node,
                        s,
                        s + edit.replacement.len(),
                        format,
                        &mut self.revision_ids,
                    );
                }
            }
            // `format_run` ranges, in new coordinates and plan order.
            for (start, end, format) in format_runs.remove(&para).unwrap_or_default() {
                let s = new_position(&edits, start, true, None);
                let e = new_position(&edits, end, false, None);
                format_range(
                    &mut self.opened.dom,
                    node,
                    s,
                    e,
                    &format,
                    &mut self.revision_ids,
                );
            }
            // Footnote reference marks; reversed so marks sharing a point
            // end up in plan order.
            for (at, id) in references
                .remove(&para)
                .unwrap_or_default()
                .into_iter()
                .rev()
            {
                let at = new_position(&edits, at, true, None);
                notes::insert_reference(
                    &mut self.opened.dom,
                    node,
                    at,
                    id,
                    reference_style,
                    &mut self.revision_ids,
                );
            }
            // Helper bookmarks around `whole` replacements. Comment ranges
            // placed below land inside them, on the inserted text.
            let plan = self.plan;
            let substitutions = boundary_substitutions(self, &edits, para)?;
            for edit in &substitutions {
                let paired = edit.start != edit.end
                    && matches!(plan.operations[edit.op].kind, OperationKind::Insert { .. });
                if !paired
                    && !matches!(
                        plan.operations[edit.op].kind,
                        OperationKind::Replace { whole: true, .. }
                    )
                {
                    continue;
                }
                if edit.replacement.is_empty() {
                    continue;
                }
                // The text the anchor matched, as the document has it: an
                // anchor typed with look-alikes or Markdown marks reads as it.
                let find = self.projections[para].text[edit.start..edit.end].to_string();
                let s = new_position(&substitutions, edit.start, true, Some(edit.op));
                let end = s + edit.replacement.len();
                let story = self.paragraph_story[para].0;
                let part = (story != 0).then(|| self.stories[story].part.clone());
                if let Some(mark) = whole::mark(
                    &mut self.opened.dom,
                    node,
                    (s, end),
                    (edit.op, part),
                    (&find, &edit.replacement),
                    &mut self.revision_ids,
                ) {
                    self.whole_marks.push(mark);
                }
            }
            // Comment ranges, in new coordinates.
            let mut pending: Vec<(usize, usize, usize, String)> = Vec::new();
            for edit in &edits {
                if let Some(text) = &edit.comment {
                    let s = new_position(&edits, edit.start, true, Some(edit.op));
                    pending.push((s, s + edit.replacement.len(), edit.op, text.clone()));
                }
            }
            for (start, end, i, text) in comment_ranges.remove(&para).unwrap_or_default() {
                let (s, e) = comment_bounds(&edits, start, end, i);
                pending.push((s, e, i, text));
            }
            pending.sort_by_key(|&(s, _, i, _)| (s, i));
            for (start, end, i, _) in pending {
                anchor_comment(
                    &mut self.opened.dom,
                    node,
                    start,
                    end,
                    ids[&i],
                    &mut self.revision_ids,
                );
            }
        }
        // 1a. Restores: plain copies of the deletion's runs after it, which
        // stays; found by `w:id`, so the text edits before do not move them.
        // An operation's comment runs from its first copy to its last.
        let mut spans: BTreeMap<usize, (NodeId, NodeId)> = BTreeMap::new();
        for (i, r) in &self.resolved {
            let Resolved::Restore { para, wid, .. } = r else {
                continue;
            };
            let node = self.paragraph_nodes[*para];
            if let Some((first, last)) = visible::restore_plain(&mut self.opened.dom, node, wid) {
                spans.entry(*i).or_insert((first, last)).1 = last;
            }
        }
        for (i, (first, last)) in spans {
            if let Some(&id) = ids.get(&i) {
                visible::comment_around(&mut self.opened.dom, first, last, id);
            }
        }
        Ok(())
    }

    fn apply_comment_spans(&mut self, ids: &BTreeMap<usize, u32>) {
        // 1b. Comments over several paragraphs, in their edited text.
        for (i, r) in &self.resolved {
            if let Resolved::CommentSpan { para, last, .. } = r {
                anchor_span(
                    &mut self.opened.dom,
                    self.paragraph_nodes[*para],
                    self.paragraph_nodes[*last],
                    ids[i],
                    &mut self.revision_ids,
                );
            }
        }
    }

    fn apply_insertions(&mut self, ids: &BTreeMap<usize, u32>) -> Vec<NodeId> {
        // 2. Paragraph and table insertions (anchors are source paragraphs,
        // untouched by 1).
        let inserts: Vec<(usize, Resolved)> = self
            .resolved
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r,
                    Resolved::InsertParagraph { .. }
                        | Resolved::InsertTable { .. }
                        | Resolved::InsertImage { .. }
                )
            })
            .cloned()
            .collect();
        let mut tables: Vec<NodeId> = Vec::new();
        let mut media_used = std::collections::HashSet::new();
        let mut drawing_id = crate::markdown::max_drawing_id(&self.opened.pkg);
        // Several paragraphs after one anchor follow it in plan order: each
        // goes after the one inserted there before it.
        let mut last_after: BTreeMap<usize, NodeId> = BTreeMap::new();
        for (i, r) in inserts {
            let (anchor, side, news, commented) = match r {
                Resolved::InsertParagraph {
                    anchor,
                    side,
                    runs,
                    like,
                    style,
                    comment,
                    toc,
                    keep_mark,
                } => {
                    let like_node = self.paragraph_nodes[like];
                    let dom = &mut self.opened.dom;
                    let news = match &toc {
                        Some(toc) => toc_paragraphs(dom, toc),
                        None => vec![build_paragraph(dom, like_node, &runs, style.as_deref())],
                    };
                    if keep_mark {
                        copy_mark_formatting(dom, like_node, news[0]);
                    }
                    (anchor, side, news, comment.is_some())
                }
                Resolved::InsertTable {
                    anchor,
                    side,
                    rows,
                    header_row,
                    widths,
                    style,
                    ..
                } => {
                    let new = structural::build_table(
                        &mut self.opened.dom,
                        self.paragraph_nodes[anchor],
                        &rows,
                        header_row,
                        &widths,
                        &style,
                    );
                    tables.push(new);
                    (anchor, side, vec![new], false)
                }
                Resolved::InsertImage {
                    anchor,
                    side,
                    picture,
                } => {
                    drawing_id = drawing_id.saturating_add(1);
                    let new = self.image_paragraph(&picture, &mut media_used, drawing_id);
                    (anchor, side, vec![new], false)
                }
                _ => continue,
            };
            let anchor_node = self.paragraph_nodes[anchor];
            for &new in &news {
                match side {
                    Side::After => {
                        let prev = last_after.get(&anchor).copied().unwrap_or(anchor_node);
                        self.opened.dom.add_after_self(prev, new);
                        last_after.insert(anchor, new);
                    }
                    Side::Before => self.opened.dom.add_before_self(anchor_node, new),
                }
            }
            let new = news[news.len() - 1];
            if commented {
                let projection = project_paragraph(&self.opened.dom, new);
                anchor_comment(
                    &mut self.opened.dom,
                    new,
                    0,
                    projection.text.len(),
                    ids[&i],
                    &mut self.revision_ids,
                );
            }
        }
        tables
    }

    fn apply_paragraph_formatting(&mut self) {
        // 3. Paragraph formatting.
        for (_, r) in &self.resolved {
            if let Resolved::FormatParagraph {
                para,
                style,
                alignment,
                spacing,
            } = r
            {
                format_paragraph(
                    &mut self.opened.dom,
                    self.paragraph_nodes[*para],
                    style.as_deref(),
                    *alignment,
                    *spacing,
                );
            }
        }
    }

    fn apply_lists(&mut self) {
        // 3b. Lists: new numbering instances take ids after the source's.
        let (mut next_abstract, mut next_num) = structural::next_numbering_ids(&self.opened);
        for (_, r) in &self.resolved {
            let Resolved::List {
                paras,
                kind,
                level,
                join,
                style,
                add_style,
            } = r
            else {
                continue;
            };
            let num_id = match join {
                Some(num_id) => num_id.clone(),
                None => {
                    let (abstracts, nums) = &mut self.new_numbering;
                    abstracts.push(crate::markdown::xml::abstract_num(
                        next_abstract,
                        kind.format(),
                    ));
                    nums.push(crate::markdown::xml::num(next_num, next_abstract, None));
                    next_abstract += 1;
                    next_num += 1;
                    (next_num - 1).to_string()
                }
            };
            for &para in paras {
                let styled = structural::number_paragraph(
                    &mut self.opened.dom,
                    self.paragraph_nodes[para],
                    *level,
                    &num_id,
                    style,
                );
                if styled && *add_style {
                    self.needed_styles.insert(style.clone());
                }
            }
        }
    }

    fn apply_merges(&mut self) {
        // 4. Merges, first first, so a chain folds into its last paragraph.
        let mut merges: Vec<(usize, usize, String)> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::MergeParagraphs {
                    para,
                    next,
                    separator,
                    ..
                } => Some((*para, *next, separator.clone())),
                _ => None,
            })
            .collect();
        merges.sort_by_key(|&(para, ..)| para);
        for (para, next, separator) in merges {
            merge_into(
                &mut self.opened.dom,
                self.paragraph_nodes[para],
                self.paragraph_nodes[next],
                &separator,
            );
        }
    }

    fn apply_paragraph_deletions(&mut self, emptied: &BTreeSet<usize>) {
        // 5. Paragraph deletions; a container's final paragraph that nothing
        // replaces keeps its mark, as Word keeps it. An empty bookmark has no
        // characters for Accept All to drop, so it stays on the paragraph the
        // deleted mark joins, or in the emptied paragraph (#1676).
        let mut deletions: Vec<usize> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::DeleteParagraph { para } => Some(*para),
                _ => None,
            })
            .collect();
        deletions.sort_unstable();
        for para in deletions {
            let node = self.paragraph_nodes[para];
            if emptied.contains(&para) {
                Self::retain_empty_bookmarks(&mut self.opened.dom, node);
            } else {
                Self::rehome_empty_bookmarks(&mut self.opened.dom, node);
                self.opened.dom.remove(node);
            }
        }
    }

    /// Bookmark endpoints with no characters between them. Accept All keeps
    /// those, including when the paragraph around them is deleted.
    fn empty_bookmark_markers(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
        let children = dom.elements(paragraph, None);
        let mut open: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut kept = Vec::new();
        for (index, &child) in children.iter().enumerate() {
            if dom.name_is(child, &W::bookmark_start())
                && let Some(id) = dom.attribute(child, &W::id())
            {
                open.entry(id).or_default().push(index);
            } else if dom.name_is(child, &W::bookmark_end())
                && let Some(id) = dom.attribute(child, &W::id())
                && let Some(start_at) = open.get_mut(id).and_then(Vec::pop)
                && children[start_at + 1..index]
                    .iter()
                    .all(|&node| Self::is_range_marker(dom, node))
            {
                kept.push(start_at);
                kept.push(index);
            }
        }
        kept.sort_unstable();
        kept.into_iter().map(|index| children[index]).collect()
    }

    fn is_range_marker(dom: &Dom, node: NodeId) -> bool {
        dom.name(node).is_some_and(|name| {
            name.namespace_name() == W::URI
                && matches!(
                    name.local_name(),
                    "bookmarkStart"
                        | "bookmarkEnd"
                        | "commentRangeStart"
                        | "commentRangeEnd"
                        | "permStart"
                        | "permEnd"
                        | "proofErr"
                )
        })
    }

    /// Leave an emptied paragraph's empty bookmarks where they are. Its text
    /// goes; the markers have no deleted characters to follow.
    fn retain_empty_bookmarks(dom: &mut Dom, paragraph: NodeId) {
        let keep = Self::empty_bookmark_markers(dom, paragraph);
        for child in dom.nodes(paragraph) {
            if !dom.name_is(child, &W::p_pr()) && !keep.contains(&child) {
                dom.remove(child);
            }
        }
    }

    /// Move an empty bookmark onto the following paragraph, ahead of its
    /// content: Accept All joins the deleted mark forward and the bookmark
    /// has nothing left to sit in. A last paragraph (of the body or of a
    /// cell that keeps another) has no following paragraph; Accept All then
    /// leaves the bookmark at the end of the preceding one.
    fn rehome_empty_bookmarks(dom: &mut Dom, paragraph: NodeId) {
        let p = W::p();
        let next = dom
            .nodes_after_self(paragraph)
            .into_iter()
            .find(|&node| dom.name_is(node, &p));
        let Some(next) = next else {
            let Some(previous) = dom
                .nodes_before_self(paragraph)
                .into_iter()
                .rev()
                .find(|&node| dom.name_is(node, &p))
            else {
                return;
            };
            for marker in Self::empty_bookmark_markers(dom, paragraph) {
                dom.remove(marker);
                dom.add(previous, marker);
            }
            return;
        };
        let markers = Self::empty_bookmark_markers(dom, paragraph);
        for &marker in &markers {
            dom.remove(marker);
        }
        let first_content = dom
            .elements(next, None)
            .into_iter()
            .find(|&child| !dom.name_is(child, &W::p_pr()));
        if let Some(first_content) = first_content {
            // Each insert lands immediately before the content, so walking
            // forward leaves the markers in document order.
            for marker in markers {
                dom.add_before_self(first_content, marker);
            }
        } else {
            for marker in markers {
                dom.add(next, marker);
            }
        }
    }

    fn apply_reply_markers(&mut self, ids: &BTreeMap<usize, u32>) {
        // 6. Reply markers beside their comment's; deleted comments' markers.
        let roots: Vec<NodeId> = self.stories.iter().map(|s| s.root).collect();
        for (i, r) in &self.resolved {
            let Resolved::Thread { op, .. } = r else {
                continue;
            };
            match op {
                ThreadOp::Reply { parent, .. } => {
                    place_reply_markers(&mut self.opened.dom, &roots, *parent, ids[i]);
                }
                ThreadOp::Delete { id } => {
                    let family = self.family.as_ref().expect("loaded in start");
                    let gone: Vec<String> = family
                        .with_replies(*id)
                        .iter()
                        .map(u32::to_string)
                        .collect();
                    remove_comment_markers(&mut self.opened.dom, &roots, &gone);
                }
                ThreadOp::Resolve { .. } | ThreadOp::Edit { .. } => {}
            }
        }
    }

    fn apply_table_separations(&mut self, tables: Vec<NodeId>) {
        // 7. A paragraph after each new table, and between it and a table
        // before it, once the deletions have settled its neighbours.
        for table in tables {
            structural::separate(&mut self.opened.dom, table);
        }
    }

    fn apply_page_setups(&mut self) {
        // 8. Page setup.
        let setups: Vec<sections::Targets> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::PageSetup { targets } => Some(targets.clone()),
                _ => None,
            })
            .collect();
        for targets in &setups {
            self.apply_page_setup(targets);
        }
    }

    /// Mutate the copy. Resolution succeeded, so every step here is total.
    pub(super) fn apply(&mut self) -> Result<(), EditError> {
        // Read off the source's structure, before any step changes it.
        let emptied = self.emptied_paragraphs();
        let ids = self.apply_comments();
        self.apply_text_edits(&ids)?;
        self.apply_comment_spans(&ids);
        // 1c. Content control fills (text edits never reach control content).
        self.apply_fills();
        let tables = self.apply_insertions(&ids);
        self.apply_paragraph_formatting();
        self.apply_lists();
        self.apply_merges();
        self.apply_paragraph_deletions(&emptied);
        self.apply_reply_markers(&ids);
        self.apply_table_separations(tables);
        self.apply_page_setups();
        Ok(())
    }
    fn new_comment(&mut self, text: String) -> u32 {
        let id = self.reserve_comment_id();
        self.comments.push((id, text));
        id
    }

    fn reserve_comment_id(&mut self) -> u32 {
        self.comments_added += 1;
        if let Some(id) = self.preset_comment_ids.pop() {
            return id;
        }
        // check_conflicts proved every new id fits in u32.
        let id = u32::try_from(self.next_comment_id).unwrap_or(u32::MAX);
        self.next_comment_id += 1;
        id
    }

    /// The comment of operation `op` when it deletes text or a paragraph,
    /// or strikes inserted text: on deleted text, so in the redline only.
    /// A struck paragraph mark's comment sits on a word that stays.
    fn deletion_comment(&self, op: usize) -> Option<&str> {
        match &self.plan.operations[op].kind {
            OperationKind::Delete { comment, .. }
            | OperationKind::DeleteParagraph { comment, .. } => comment.as_deref(),
            OperationKind::Strike { comment, .. } => comment.as_deref().filter(|_| {
                !self
                    .resolved
                    .iter()
                    .any(|(i, r)| *i == op && matches!(r, Resolved::CommentRange { .. }))
            }),
            _ => None,
        }
    }

    /// Whether operation `op` replies to a comment whose reference sits in
    /// a paragraph this plan deletes: the copy loses the reference, so the
    /// reply goes where the parent's stays, on the deleted text (#1159).
    fn reply_in_deleted(&self, op: usize) -> bool {
        self.reply_on_deleted(op).is_some()
    }

    /// The parent of reply `op` and the operation that deletes the
    /// paragraph the parent is on, when the plan deletes it.
    pub(super) fn reply_on_deleted(&self, op: usize) -> Option<(u32, usize)> {
        self.resolved.iter().find_map(|(i, r)| match r {
            Resolved::Thread {
                op: ThreadOp::Reply { parent, .. },
                anchor: Some(anchor),
                ..
            } if *i == op => self.deletes_paragraph(*anchor).map(|by| (*parent, by)),
            _ => None,
        })
    }

    /// The operation that deletes paragraph `para`, if one does.
    fn deletes_paragraph(&self, para: usize) -> Option<usize> {
        self.resolved.iter().find_map(|(i, r)| {
            matches!(r, Resolved::DeleteParagraph { para: p } if *p == para).then_some(*i)
        })
    }

    /// The text of [`Self::deletion_comment`], or of a reply
    /// [`Self::reply_in_deleted`] names: what goes on the deleted text.
    pub(super) fn deleted_text_comment(&self, op: usize) -> Option<&str> {
        self.deletion_comment(op)
            .or_else(|| match &self.plan.operations[op].kind {
                OperationKind::ReplyComment { text, .. } if self.reply_in_deleted(op) => {
                    Some(text.as_str())
                }
                _ => None,
            })
    }

    /// The copy the comparer reads as the original: the base, with each
    /// deleted paragraph's comment anchored on its whole text, and each
    /// reply to a comment in a deleted paragraph beside its parent. The comparer
    /// carries the comment onto the deleted text of the redline. A plan's
    /// watermark is written into this copy as well, so it stays untracked.
    pub(super) fn commented_base(&self) -> Result<std::borrow::Cow<'_, [u8]>, EditError> {
        // Edited and deleted comments are edited and deleted in the original
        // too: the comparer carries the copy's comment parts only when they
        // define every comment of the original unchanged.
        let thread_ops: Vec<Operation> = self
            .plan
            .operations
            .iter()
            .filter(|op| {
                matches!(
                    op.kind,
                    OperationKind::EditComment { .. } | OperationKind::DeleteComment { .. }
                )
            })
            .cloned()
            .collect();
        // A redaction is no change either: the base loses the text too.
        let redactions: Vec<Operation> = self
            .plan
            .operations
            .iter()
            .filter(|op| matches!(op.kind, OperationKind::Redact { .. }))
            .cloned()
            .collect();
        if self.deletion_comments.is_empty()
            && thread_ops.is_empty()
            && self.watermark.is_none()
            && redactions.is_empty()
        {
            return Ok(std::borrow::Cow::Borrowed(&self.base));
        }
        // A strike's comment is emitted with it, under keep: never compared.
        let mut operations: Vec<Operation> = self
            .deletion_comments
            .iter()
            .filter_map(|&(_, op)| {
                let operation = &self.plan.operations[op];
                // A reply sits beside its parent in the base, as in Word.
                if matches!(operation.kind, OperationKind::ReplyComment { .. }) {
                    return Some(operation.clone());
                }
                // A deleted text's comment takes the deletion's own anchor,
                // which finds the same words in the base.
                let (paragraph, find, occurrence, text) = match &operation.kind {
                    OperationKind::DeleteParagraph {
                        paragraph,
                        comment: Some(text),
                    } => (paragraph, None, None, text),
                    OperationKind::Delete {
                        paragraph,
                        find,
                        occurrence,
                        comment: Some(text),
                    } => (paragraph, Some(find.clone()), *occurrence, text),
                    _ => return None,
                };
                // A positional id names the paragraph as the plan's author
                // read it, and the base is settled: the copy names the
                // deletion's own paragraph by its id there. A comment's
                // text (`c5.text`) reads the same in both.
                let paragraph = match comment_text_of(paragraph) {
                    Some(_) => paragraph.clone(),
                    None => self
                        .resolved
                        .iter()
                        .find_map(|(i, resolved)| match resolved {
                            Resolved::Text { para, .. } | Resolved::DeleteParagraph { para }
                                if *i == op =>
                            {
                                Some(Selector::Name(self.paragraph_id(*para)))
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| paragraph.clone()),
                };
                Some(Operation {
                    id: operation.id.clone(),
                    group: operation.group.clone(),
                    kind: OperationKind::Comment {
                        paragraph,
                        find,
                        text: text.clone(),
                        through: None,
                        occurrence,
                    },
                })
            })
            .collect();
        operations.extend(thread_ops);
        // A watermark is header content, not a change: the base carries it
        // too, so the comparer leaves it untracked.
        operations.extend(
            self.plan
                .operations
                .iter()
                .filter(|op| matches!(op.kind, OperationKind::Watermark { .. }))
                .cloned(),
        );
        operations.extend(redactions);
        let plan = EditPlan {
            schema_version: SCHEMA_VERSION,
            source_sha256: None,
            author: self.plan.author.clone(),
            date: Some(self.date.clone()),
            initials: Some(self.initials.clone()),
            resolve_revisions: None,
            removed_anchor_comments: RemovedAnchorComments::default(),
            existing_revisions: ExistingRevisions::default(),
            operations,
            update_fields: false,
            rewrite_others_comments: self.plan.rewrite_others_comments,
            others: self.plan.others,
        };
        let mut tx = Transaction::start(&self.base, &plan)?;
        tx.preset_comment_ids = self
            .deletion_comments
            .iter()
            .rev()
            .map(|&(id, _)| id)
            .collect();
        tx.resolve()?;
        tx.apply()?;
        let (commented, _) = tx.finish()?;
        Ok(std::borrow::Cow::Owned(commented))
    }

    /// The clean copy, and the copy the comparer reads when `whole`
    /// replacements carry helper bookmarks (the clean copy never does).
    pub(super) fn finish(&mut self) -> Result<(Vec<u8>, Option<Vec<u8>>), EditError> {
        let threads = self
            .resolved
            .iter()
            .any(|(_, r)| matches!(r, Resolved::Thread { .. }));
        if !self.comments.is_empty() || threads {
            self.write_comments_part()?;
        }
        let mut styles: std::collections::BTreeSet<String> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::InsertTable {
                    style,
                    add_style: true,
                    ..
                } => Some(style.clone()),
                _ => None,
            })
            .collect();
        styles.extend(self.needed_styles.iter().cloned());
        let main = self.opened.main.clone();
        if !styles.is_empty() {
            structural::add_styles(&mut self.opened.pkg, &main, &styles);
        }
        let (abstracts, nums) = std::mem::take(&mut self.new_numbering);
        if !nums.is_empty() {
            structural::splice_numbering(&mut self.opened.pkg, &main, &abstracts, &nums);
        }
        // A story is written only when an operation edits it, so an
        // untouched main document keeps its exact bytes. A watermark that
        // creates a header records that reference in the body and names
        // the body among the stories it returns.
        let mut written = self.touched_stories();
        written.extend(self.apply_watermark()?);
        written.extend(self.notes_story);
        if let Some(request) = &self.settings {
            crate::settings::apply_settings(&mut self.opened.pkg, &main, request)
                .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
        }
        let lost_reference = self.removes_comment_reference(&written);
        let marked = if self.whole_marks.is_empty() {
            None
        } else {
            self.write_stories(&written);
            let bytes = self
                .opened
                .pkg
                .to_zip()
                .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
            for &story in &written {
                whole::strip(&mut self.opened.dom, self.stories[story].root);
            }
            Some(bytes)
        };
        self.write_stories(&written);
        self.written_stories = written;
        if lost_reference {
            // A comment whose reference went with deleted text goes too, as
            // Accept All takes it (Word rule R1): no definition is left
            // without an anchor.
            let stories: Vec<String> =
                crate::revision_processor::revision_bearing_parts(&self.opened.pkg)
                    .into_iter()
                    .filter(|(_, is_styles)| !is_styles)
                    .map(|(part, _)| part)
                    .collect();
            let comments = crate::revision_processor::comments_part(&self.opened.pkg);
            crate::revision_processor::prune_orphan_comments(
                &mut self.opened.pkg,
                &stories,
                comments.as_deref(),
            );
        }
        let clean = self
            .opened
            .pkg
            .to_zip()
            .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
        #[cfg(test)]
        let clean = prepared::tests::serialized_candidate(clean);
        Ok((clean, marked))
    }

    /// Whether the plan took a comment reference out of a story it writes
    /// (a deleted paragraph's, or deleted text's), read against the story as
    /// the package still holds it.
    fn removes_comment_reference(&self, stories: &std::collections::BTreeSet<usize>) -> bool {
        if self.opened.pkg.part_bytes("word/comments.xml").is_none() {
            return false;
        }
        let name = W::comment_reference();
        let ids = |dom: &Dom, root: NodeId| -> std::collections::HashSet<String> {
            dom.descendants(root, Some(&name))
                .into_iter()
                .filter_map(|r| dom.attribute(r, &W::id()).map(str::to_string))
                .collect()
        };
        stories.iter().any(|&story| {
            let StoryPart { part, root, .. } = &self.stories[story];
            let Some(xml) = self.opened.pkg.part_string(part) else {
                return false;
            };
            let mut source = Dom::new();
            let document = source.parse_xdocument(&xml);
            source.root(document).is_some_and(|source_root| {
                !ids(&source, source_root).is_subset(&ids(&self.opened.dom, *root))
            })
        })
    }

    fn write_stories(&mut self, stories: &std::collections::BTreeSet<usize>) {
        for &story in stories {
            let StoryPart { part, document, .. } = &self.stories[story];
            let xml = self.opened.dom.serialize_document(*document);
            self.opened.pkg.set_part(part, xml.into_bytes());
        }
    }

    /// Write the comment part family: the plan's new comments and replies,
    /// then its edits, resolutions and deletions; every part consistent
    /// (see [`crate::comments`]).
    fn write_comments_part(&mut self) -> Result<(), EditError> {
        let main = self.opened.main.clone();
        let mut family = match self.family.take() {
            Some(family) => family,
            None => crate::comments::CommentFamily::load(&self.opened.pkg, &main)
                .map_err(|m| err("INVALID_DOCUMENT", None, m))?,
        };
        for (id, text) in &self.comments {
            family.add(&crate::comments::NewComment {
                id: *id,
                author: &self.plan.author,
                date: &self.date,
                initials: &self.initials,
                text,
                parent: self.reply_parents.get(id).copied(),
            });
        }
        let ops: Vec<&ThreadOp> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::Thread { op, .. } => Some(op),
                _ => None,
            })
            .collect();
        for op in &ops {
            if let ThreadOp::Edit { id, text } = op {
                family.set_text(*id, text);
            }
        }
        for op in &ops {
            if let ThreadOp::Resolve { id, done } = op {
                family.set_done(*id, *done);
            }
        }
        for op in &ops {
            if let ThreadOp::Delete { id } = op {
                // Rescue at settlement is provisional: both plan schemas'
                // reports must describe the family we actually publish.
                let removed: Vec<_> = family
                    .remove(*id)
                    .into_iter()
                    .map(|id| format!("c{id}"))
                    .collect();
                self.kept_comments.retain(|c| !removed.contains(&c.id));
                for kept in &mut self.kept_comments {
                    kept.replies.retain(|id| !removed.contains(id));
                }
            }
        }
        family.store(&mut self.opened.pkg, &main);
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod resolve_branch_tests {
    use super::super::deeper_boundary_fixture::{Part, docx, docx_with_sect, para, part_string};
    use super::super::{EditResult, apply_plan};
    use super::*;

    fn plan(author: &str, extra: serde_json::Value) -> EditPlan {
        let mut value = serde_json::json!({
            "schema_version": 1,
            "author": author,
            "date": "2026-02-02T00:00:00Z",
            "operations": [],
        });
        let serde_json::Value::Object(extra) = extra else {
            panic!("plan fields are an object");
        };
        for (k, v) in extra {
            value[k.as_str()] = v;
        }
        EditPlan::from_json(&value.to_string()).unwrap()
    }

    fn ok(source: &[u8], extra: serde_json::Value) -> EditResult {
        apply_plan(source, &plan("Ed", extra)).unwrap()
    }

    fn refused(source: &[u8], extra: serde_json::Value) -> EditError {
        match apply_plan(source, &plan("Ed", extra)) {
            Ok(_) => panic!("expected a refusal"),
            Err(e) => e,
        }
    }

    fn three() -> Vec<u8> {
        docx(&format!(
            "{}{}{}",
            para("Alpha one."),
            para("Beta two."),
            para("Gamma three.")
        ))
    }

    fn with_header(header_text: &str) -> Vec<u8> {
        let header = format!(
            r#"<w:hdr xmlns:w="{}"><w:p><w:r><w:t>{header_text}</w:t></w:r></w:p></w:hdr>"#,
            W::URI
        );
        docx_with_sect(
            &para("Body text."),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        )
    }

    #[test]
    fn a_comment_span_must_run_forward_in_one_story() {
        let e = refused(
            &three(),
            serde_json::json!({"operations": [
                {"kind":"comment","paragraph":"p2","through":"p0","text":"backwards"}
            ]}),
        );
        assert_eq!(e.code, "INVALID_EDIT");
        assert!(
            e.message.contains("must be in the same story"),
            "{}",
            e.message
        );
        let e = refused(
            &with_header("Head."),
            serde_json::json!({"operations": [
                {"kind":"comment","paragraph":"p0","through":"header1.p0","text":"across"}
            ]}),
        );
        assert_eq!(e.code, "INVALID_EDIT");
        assert_eq!(
            e.message,
            "through (header1) must be in the same story as paragraph and not before it"
        );
        let result = ok(
            &three(),
            serde_json::json!({"operations": [
                {"kind":"comment","paragraph":"p0","through":"p2","text":"all three"}
            ]}),
        );
        let comments = crate::comments::list_comments(&result.redline).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "all three");
    }

    #[test]
    fn a_missing_comment_anchor_names_where_the_text_is() {
        let e = refused(
            &three(),
            serde_json::json!({"operations": [
                {"kind":"comment","paragraph":"p0","find":"Gamma","text":"wrong paragraph"}
            ]}),
        );
        assert_eq!(e.code, "ANCHOR_NOT_FOUND");
        assert!(e.message.contains("p2"), "{}", e.message);
    }

    #[test]
    fn header_rewrites_take_no_comment() {
        let source = with_header("Head line.");
        let e = refused(
            &source,
            serde_json::json!({"operations": [
                {"kind":"rewrite","paragraph":"header1.p0","text":"Head, new.","comment":"why"}
            ]}),
        );
        assert_eq!(e.code, "COMMENT_NOT_IN_BODY");
        let result = ok(
            &source,
            serde_json::json!({"operations": [
                {"kind":"rewrite","paragraph":"header1.p0","text":"Head line, new."}
            ]}),
        );
        let header = part_string(&result.clean, "word/header1.xml").unwrap();
        assert!(header.contains("Head line, new."), "{header}");
    }

    #[test]
    fn comment_text_rewrites_cover_at_most_two_following_paragraphs() {
        let source = docx(&format!(
            "{}{}{}",
            r#"<w:p><w:r><w:t xml:space="preserve">Alpha </w:t></w:r><w:commentRangeStart w:id="0"/><w:r><w:t>one.</w:t></w:r></w:p>"#,
            r#"<w:p><w:r><w:t>Beta</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r><w:r><w:t xml:space="preserve"> two.</w:t></w:r></w:p>"#,
            r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>Gamma</w:t></w:r></w:p><w:p><w:r><w:t>Delta</w:t></w:r></w:p><w:p><w:r><w:t>Eps</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>"#,
        ));
        // Two paragraphs that follow each other: rewritten as one.
        let result = ok(
            &source,
            serde_json::json!({"existing_revisions":"keep","operations": [
                {"kind":"rewrite","paragraph":"c0.text","text":"joined"}
            ]}),
        );
        let xml = part_string(&result.clean, "word/document.xml").unwrap();
        assert!(xml.contains("joined"), "{xml}");
        assert!(!xml.contains("one."), "{xml}");
        // Three paragraphs apart: refused.
        let e = refused(
            &source,
            serde_json::json!({"existing_revisions":"keep","operations": [
                {"kind":"rewrite","paragraph":"c1.text","text":"x"}
            ]}),
        );
        assert!(e.message.contains("covers text from"), "{}", e.message);
        // A comment on its rewrite is refused.
        let e = refused(
            &source,
            serde_json::json!({"existing_revisions":"keep","operations": [
                {"kind":"rewrite","paragraph":"c0.text","text":"x","comment":"no"}
            ]}),
        );
        assert!(e.message.contains("takes no comment"), "{}", e.message);
    }

    #[test]
    fn a_kept_rewrite_marks_what_the_comparer_marks() {
        let source = docx(&para("The quick brown fox jumps."));
        let result = ok(
            &source,
            serde_json::json!({"existing_revisions":"keep","operations": [
                {"kind":"rewrite","paragraph":"p0","text":"The quick red fox leaps."}
            ]}),
        );
        let changes = crate::changes::list_changes(&result.redline).unwrap();
        let deleted: Vec<&str> = changes
            .iter()
            .filter(|c| c.kind == crate::changes::ChangeKind::Deletion)
            .map(|c| c.text.as_str())
            .collect();
        assert!(deleted.iter().any(|t| t.contains("brown")), "{deleted:?}");
        assert!(deleted.iter().any(|t| t.contains("jumps")), "{deleted:?}");
    }

    #[test]
    fn editing_inside_another_authors_deletion_points_at_the_answer() {
        let source = docx(
            r#"<w:p><w:r><w:t xml:space="preserve">Keep </w:t></w:r><w:del w:id="3" w:author="Bo" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>old words</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> end.</w:t></w:r></w:p>"#,
        );
        // The same plan restores it first: the edit waits.
        let e = refused(
            &source,
            serde_json::json!({"existing_revisions":"keep","operations": [
                {"kind":"restore","revision":"r3"},
                {"kind":"replace","paragraph":"p0","find":"old words","replacement":"new"}
            ]}),
        );
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
        assert!(
            e.message.ends_with("which this command restores: edit the restored words in a command of their own, after the restore."),
            "{}",
            e.message
        );
        // Alone, the hint offers restore.
        let e = refused(
            &source,
            serde_json::json!({"existing_revisions":"keep","operations": [
                {"kind":"replace","paragraph":"p0","find":"old words","replacement":"new"}
            ]}),
        );
        assert!(e.message.contains(r#""kind":"restore""#), "{}", e.message);
        // The plan's own deletion is no other author's.
        let own = apply_plan(
            &source,
            &plan(
                "Bo",
                serde_json::json!({"existing_revisions":"keep","operations": [
                    {"kind":"replace","paragraph":"p0","find":"old words","replacement":"new"}
                ]}),
            ),
        );
        match own {
            Ok(_) => panic!("the plan's own deletion is still a deletion"),
            Err(e) => {
                assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
                assert!(e.message.contains("let it go:"), "{}", e.message);
                assert!(!e.message.contains("\"restore\""), "{}", e.message);
            }
        }
    }
}
