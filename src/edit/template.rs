// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PlanV2 generation, using the exact same S selection and W rows as for-edit.
//! This module never applies a plan or chooses revision decisions for the caller.

use super::{
    EditError,
    for_edit::{self, Correspondence, ForEditRequest, TargetPair},
    v2,
};
use serde::Serialize;
use std::collections::BTreeSet;

/// Explicit inputs for reproducible, source-specific generation. The caller
/// freezes attribution once; this pure API does not consult a clock or paths.
#[derive(Clone, Debug)]
pub struct TemplateRequest {
    /// Selection and working basis, shared with read --for-edit.
    pub view: ForEditRequest,
    /// Attribution for later edits, not a generation effect.
    pub author: String,
    /// Frozen timezone-bearing RFC 3339 timestamp.
    pub date: String,
}

/// A complete saved request and separately recomputed, bounded observations.
#[derive(Clone, Debug)]
pub struct GeneratedTemplate {
    /// The same PlanV2 edit apply consumes; no generator dialect.
    pub plan: v2::PlanV2,
    /// Coverage/readiness observations, never stored as authorization.
    pub summary: GenerationSummary,
}

/// Compact generation facts; no claim about which edits the user intends.
#[derive(Clone, Debug, Serialize)]
pub struct GenerationSummary {
    /// Complete traversal and representation facts for the selected scope.
    pub coverage: v2::Coverage,
    /// Paragraphs selected on original S, before settlement.
    pub selected_paragraphs: usize,
    /// Rewrite rows actually emitted, including conditional rows.
    pub rows_generated: usize,
    /// Selected paragraphs without a faithful rewrite representation.
    pub read_only_paragraphs: usize,
    /// Bounded working readiness examples; full evidence is available from read.
    pub working_readiness: Vec<WorkingReadiness>,
    /// Whether all selected readiness and blocker evidence is present here.
    pub readiness_complete: bool,
    /// How to obtain complete working evidence with the same inputs.
    pub inspect: String,
}

/// Readiness without repeating the document's text in a receipt.
#[derive(Clone, Debug, Serialize)]
pub struct WorkingReadiness {
    /// Real original paired identity.
    pub target: TargetPair,
    /// Original-to-working correspondence.
    pub correspondence: Correspondence,
    /// Per-operation support, recomputed from W.
    pub readiness: Vec<for_edit::Readiness>,
    /// Bounded examples of exact known working blockers.
    pub blockers: Vec<for_edit::Blocker>,
    /// Total blockers (readiness indices refer to the complete for-edit list).
    pub blocker_count: usize,
}

fn pair(target: &TargetPair) -> v2::Target {
    v2::Target {
        short_id: target.short_id.clone(),
        long_id: target.long_id.clone(),
    }
}

/// Generate unchanged desired-text rows and an honest inventory. Presentation
/// limits never truncate the plan. A caller may choose a smaller source scope.
///
/// # Errors
/// Returns source/selection/settlement errors, `EMPTY_SELECTION`, or model errors
/// for invalid attribution. No partial result can be published on failure.
pub fn generate(source: &[u8], request: &TemplateRequest) -> Result<GeneratedTemplate, EditError> {
    if request.view.cursor.is_some() {
        return Err(super::err(
            "USAGE",
            None,
            "template generation cannot continue a read cursor",
        ));
    }
    let (accept, reject) =
        super::source_refs::decisions(source, &request.view.accept, &request.view.reject)?;
    let mut query = request.view.clone();
    query.all = true;
    query.accept = accept.iter().map(|t| t.short_id.clone()).collect();
    query.reject = reject.iter().map(|t| t.short_id.clone()).collect();
    let view = for_edit::for_edit(source, &query)?;
    if view.paragraphs.is_empty() {
        return Err(super::err(
            "EMPTY_SELECTION",
            None,
            "0 source paragraphs matched; no plan generated",
        ));
    }
    if !view.coverage.selection_complete || view.coverage.truncated {
        return Err(super::err(
            "INCOMPLETE_TRAVERSAL",
            None,
            "selected traversal did not finish; no partial plan generated",
        ));
    }
    let mut operations = Vec::new();
    let mut context = Vec::new();
    let mut context_ids = BTreeSet::new();
    // Already-inventoried opaque targets, so the opaque scan stays linear
    // on opaque-heavy documents instead of rescanning the context list.
    let mut opaque_ids = BTreeSet::new();
    let mut conditional = 0;
    let mut opaque = 0;
    let mut read_only = 0;
    for row in &view.paragraphs {
        let rewrite = row.readiness.iter().find(|r| r.operation == "rewrite");
        let working = row.working.as_ref().filter(|w| w.value.complete);
        let eligible = row.correspondence == Correspondence::OneToOne
            && row.text_codec == "supported"
            && rewrite.is_some_and(|r| matches!(r.status.as_str(), "available" | "conditional"));
        if eligible
            && let Some((working, hash)) =
                working.and_then(|w| w.before_sha256.as_ref().map(|h| (w, h)))
        {
            let text = working.value.text.as_ref().ok_or_else(|| {
                super::err("INCOMPLETE_TRAVERSAL", None, "working text is incomplete")
            })?;
            operations.push(v2::Operation::Rewrite {
                target: pair(&row.target),
                before_sha256: hash.clone(),
                text: text.clone(),
                attached_comment: None,
            });
            conditional += usize::from(rewrite.is_some_and(|r| r.status == "conditional"));
        } else {
            read_only += 1;
            let reason = match row.correspondence {
                Correspondence::Merged => "merged",
                Correspondence::Removed => "removed",
                Correspondence::OneToOne => "unsupported_text_codec",
            };
            // A display projection of an opaque paragraph is not exact editable
            // text, and cannot be supplied as a verifiable paragraph observation.
            let representable = row.original_sha256.is_some();
            context.push(v2::SourceContext {
                kind: if representable {
                    v2::ContextKind::Paragraph
                } else {
                    opaque_ids.insert(row.target.short_id.clone());
                    v2::ContextKind::Opaque
                },
                target: pair(&row.target),
                text: if representable {
                    row.original.text.clone()
                } else {
                    None
                },
                reason_codes: vec![reason.into()],
            });
            context_ids.insert(row.target.short_id.clone());
        }
        // Preserve inventory of unrepresentable content even when a working
        // settlement removed it. Inventory uses real source ancestors.
        let has_opaque = row.original_sha256.is_none() || row.text_codec == "unsupported";
        if has_opaque {
            opaque += 1;
            if opaque_ids.insert(row.target.short_id.clone()) {
                context.push(v2::SourceContext {
                    kind: v2::ContextKind::Opaque,
                    target: pair(&row.target),
                    text: None,
                    reason_codes: vec!["unsupported_text_codec".into()],
                });
            }
        }
    }
    for row in &view.source_context {
        if context_ids.insert(row.target.short_id.clone()) {
            // Context is immutable original-S text, never the settled W value.
            // The reader's observation is complete under query.all.
            let representable = row.kind == "paragraph";
            if !representable {
                opaque += 1;
            }
            context.push(v2::SourceContext {
                kind: if representable {
                    v2::ContextKind::Paragraph
                } else {
                    v2::ContextKind::Opaque
                },
                target: pair(&row.target),
                text: if representable {
                    row.value.text.clone()
                } else {
                    None
                },
                reason_codes: row.reason_codes.clone(),
            });
        }
    }
    // The for-edit view already inventoried the source's comments; reuse
    // that instead of parsing the package a sixth time.
    let all_scope = query.select.is_none()
        && query.story.as_deref().is_none_or(|s| s == "all")
        && query.find.is_none()
        && query.regex.is_none()
        && !query.changed
        && query.head.is_none()
        && query.tail.is_none();
    for comment in &view.comments {
        let id = format!("c{}", comment.id);
        if all_scope
            || view
                .paragraphs
                .iter()
                .any(|r| r.comments.iter().any(|c| c.short_id == id))
        {
            context.push(v2::SourceContext {
                kind: v2::ContextKind::Comment,
                target: v2::Target {
                    short_id: id,
                    long_id: format!("comments.w:comment.w:id.{}", comment.id),
                },
                text: Some(comment.text.clone()),
                reason_codes: vec!["existing_comment".into()],
            });
        }
    }
    let coverage = v2::Coverage {
        traversal_complete: true,
        text_complete: opaque == 0,
        eligible_paragraphs: operations.len() as u64,
        conditional_paragraphs: Some(conditional as u64),
        context_paragraphs: context_ids.len() as u64,
        opaque_regions: opaque,
    };
    let plan = v2::PlanV2 {
        schema_version: 2,
        source: v2::Source {
            sha256: view.source_sha256,
        },
        policy: v2::Policy {
            track: if query.track {
                v2::Track::On
            } else {
                v2::Track::Off
            },
            existing_revisions: query.existing_revisions,
            text_view: v2::TextView::Final,
        },
        resolve_revisions: v2::RevisionDecisions { accept, reject },
        removed_anchor_comments: Default::default(),
        rewrite_others_comments: None,
        others: None,
        author: request.author.clone(),
        date: request.date.clone(),
        scope: if all_scope {
            v2::Scope::All {}
        } else {
            v2::Scope::Selected {
                targets: view.paragraphs.iter().map(|r| pair(&r.target)).collect(),
            }
        },
        operations,
        source_context: (!context.is_empty()).then_some(context),
        coverage: Some(coverage.clone()),
    };
    plan.validate()?;
    let summary = GenerationSummary {
        coverage, selected_paragraphs: view.paragraphs.len(), rows_generated: plan.operations.len(), read_only_paragraphs: read_only,
        working_readiness: view.paragraphs.iter().take(16).map(|row| WorkingReadiness {
            target: row.target.clone(), correspondence: row.correspondence, readiness: row.readiness.iter().cloned().map(|mut r| { r.blockers.truncate(8); r }).collect(),
            blockers: row.blockers.iter().take(8).cloned().collect(), blocker_count: row.blockers.len(),
        }).collect(),
        readiness_complete: view.paragraphs.len() <= 16 && view.paragraphs.iter().all(|row| row.blockers.len() <= 8),
        inspect: "Use jubarte read FILE --for-edit with the same selection and working-basis options for complete readiness.".into(),
    };
    Ok(GeneratedTemplate { plan, summary })
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &[u8] = include_bytes!("../../tests/fixtures/edit-v2/example-source.docx");

    #[test]
    fn read_cursor_cannot_generate_a_partial_template() {
        let request = TemplateRequest {
            view: ForEditRequest {
                cursor: Some("continuation".into()),
                ..Default::default()
            },
            author: "Jubarte".into(),
            date: "2026-10-17T07:00:00Z".into(),
        };
        assert_eq!(generate(SOURCE, &request).unwrap_err().code, "USAGE");
    }

    #[test]
    fn invalid_attribution_never_produces_a_saved_request() {
        for (author, date) in [(" ", "2026-10-17T07:00:00Z"), ("Jubarte", "not-a-date")] {
            let request = TemplateRequest {
                view: ForEditRequest::default(),
                author: author.into(),
                date: date.into(),
            };
            assert_eq!(generate(SOURCE, &request).unwrap_err().code, "INVALID_PLAN");
        }
    }
}
