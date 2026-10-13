// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What `edit -p rN --accept/--reject` prints: one line per resolved change.
//!
//! `accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"` names the
//! action, the short id as the view prints it, the short paragraph id, the
//! change as CriticMarkup exactly as the agent view prints it, and the text
//! that replaces it. Strings are JSON-quoted.

use std::collections::HashSet;

use crate::changes::{Change, ChangeError, ChangeKind};
use crate::edit::ResolvedRevisions;

/// Escape CriticMarkup delimiters as the agent view does.
///
/// Mirrors `crate::markdown::from_docx::critic::escape` (11 delimiters);
/// kept here so `edit` does not reach into the view's private module. If
/// that function gains a delimiter, this one must too.
fn escape_critic(text: &str) -> String {
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

/// JSON-quote as the other JSON lines do (`serde_json`).
fn quoted(text: &str) -> String {
    serde_json::to_string(text).expect("a string serializes")
}

/// The short paragraph (or story) a line names.
fn short_place(change: &Change) -> String {
    change.paragraph.as_deref().map_or_else(
        || {
            crate::ids::ChangeId::from_long(&change.id)
                .map_or_else(|| "-".to_string(), |id| id.story)
        },
        crate::ids::short_paragraph,
    )
}

/// Whether `wid` has a short spelling (`rN`): ASCII digits, no leading
/// zeros (`0` itself is fine).
fn is_short_wid(wid: &str) -> bool {
    wid == "0"
        || (!wid.is_empty() && wid.bytes().all(|b| b.is_ascii_digit()) && !wid.starts_with('0'))
}

/// The combined ref for a substitution pair in one story, or `None` when
/// either half has no short spelling (then the halves print separately).
fn pair_ref(first: &Change, second: &Change) -> Option<String> {
    let a = crate::ids::ChangeId::from_long(&first.id)?;
    let b = crate::ids::ChangeId::from_long(&second.id)?;
    if a.story != b.story || !is_short_wid(&a.wid) || !is_short_wid(&b.wid) {
        return None;
    }
    if !crate::ids::is_story(&a.story) && a.story != "body" {
        return None;
    }
    if a.story == "body" {
        Some(format!("r{}+{}", a.wid, b.wid))
    } else {
        Some(format!("{}.r{}+{}", a.story, a.wid, b.wid))
    }
}

/// The CriticMarkup and the replacement text for one text change.
fn text_from_to(kind: ChangeKind, move_from: bool, text: &str, accepted: bool) -> (String, String) {
    let escaped = escape_critic(text);
    // A move's source reads as a deletion, its destination as an insertion.
    let is_insertion = match kind {
        ChangeKind::Insertion => true,
        ChangeKind::Deletion => false,
        ChangeKind::Move => !move_from,
        ChangeKind::Formatting => {
            return (text.to_string(), text.to_string());
        }
    };
    if is_insertion {
        let from = format!("{{++{escaped}++}}");
        let to = if accepted {
            text.to_string()
        } else {
            String::new()
        };
        (from, to)
    } else {
        let from = format!("{{--{escaped}--}}");
        let to = if accepted {
            String::new()
        } else {
            text.to_string()
        };
        (from, to)
    }
}

/// One line's parts before quoting.
struct Line {
    action: &'static str,
    r#ref: String,
    place: String,
    from: String,
    to: String,
}

impl Line {
    fn render(&self) -> String {
        format!(
            "{} {} in {}: {} > {}",
            self.action,
            self.r#ref,
            self.place,
            quoted(&self.from),
            quoted(&self.to)
        )
    }
}

/// Whether two text changes in one paragraph read as one substitution in
/// the view: a deletion beside an insertion (either order), the same
/// story and paragraph, both resolved the same way.
fn is_substitution_pair(first: &Change, second: &Change) -> bool {
    if first.paragraph != second.paragraph || first.paragraph.is_none() {
        return false;
    }
    if first.target != "text" || second.target != "text" {
        return false;
    }
    let kinds = (first.kind, second.kind);
    if !matches!(
        kinds,
        (ChangeKind::Deletion, ChangeKind::Insertion)
            | (ChangeKind::Insertion, ChangeKind::Deletion)
    ) {
        return false;
    }
    // Moves never pair as substitutions; each side prints on its own.
    if first.move_name.is_some() || second.move_name.is_some() {
        return false;
    }
    let (Some(a), Some(b)) = (
        crate::ids::ChangeId::from_long(&first.id),
        crate::ids::ChangeId::from_long(&second.id),
    ) else {
        return false;
    };
    a.story == b.story
}

/// Build the stdout lines for a pure accept/reject: `source` is the bytes
/// before resolution, `resolved` what `apply_plan` reported. In document
/// order, a substitution pair whose halves resolved the same way prints
/// once as `rA+B`.
pub fn resolution_lines(
    source: &[u8],
    resolved: &ResolvedRevisions,
) -> Result<Vec<String>, ChangeError> {
    let listed = crate::changes::list_changes(source)?;
    let accepted: HashSet<&str> = resolved.accepted.iter().map(String::as_str).collect();
    let rejected: HashSet<&str> = resolved.rejected.iter().map(String::as_str).collect();
    // Document order, with the action each change resolved with.
    let mut ordered: Vec<(&Change, bool)> = Vec::new();
    for change in &listed {
        if accepted.contains(change.id.as_str()) {
            ordered.push((change, true));
        } else if rejected.contains(change.id.as_str()) {
            ordered.push((change, false));
        }
    }
    let mut lines = Vec::new();
    let mut i = 0;
    while i < ordered.len() {
        let (first, accepted_first) = ordered[i];
        // A substitution pair prints once when its halves are consecutive
        // in the source and resolved the same way.
        if i + 1 < ordered.len() {
            let (second, accepted_second) = ordered[i + 1];
            if accepted_first == accepted_second
                && is_substitution_pair(first, second)
                && let Some(combined) = pair_ref(first, second)
            {
                // Consecutive in the source, not only among the resolved:
                // no other change sits between the halves.
                let pos = |id: &str| listed.iter().position(|c| c.id == id);
                let consecutive = pos(&first.id)
                    .zip(pos(&second.id))
                    .is_some_and(|(a, b)| a + 1 == b || b + 1 == a);
                if consecutive {
                    let (deletion, insertion) = if first.kind == ChangeKind::Deletion {
                        (first, second)
                    } else {
                        (second, first)
                    };
                    let from = format!(
                        "{{~~{}~>{}~~}}",
                        escape_critic(&deletion.text),
                        escape_critic(&insertion.text)
                    );
                    let to = if accepted_first {
                        insertion.text.clone()
                    } else {
                        deletion.text.clone()
                    };
                    lines.push(
                        Line {
                            action: if accepted_first {
                                "accepted"
                            } else {
                                "rejected"
                            },
                            r#ref: combined,
                            place: short_place(first),
                            from,
                            to,
                        }
                        .render(),
                    );
                    i += 2;
                    continue;
                }
            }
        }
        lines.push(single_line(first, accepted_first));
        i += 1;
    }
    Ok(lines)
}

/// One line for a single (unpaired) change.
fn single_line(change: &Change, accepted: bool) -> String {
    let action = if accepted { "accepted" } else { "rejected" };
    let place = short_place(change);
    let (from, to) = match change.target {
        "paragraph_mark" => {
            // The CriticMarkup spec's marked break (`{++\n\n++}`); the
            // replacement keeps the break or joins the paragraphs.
            let is_insertion = match change.kind {
                ChangeKind::Insertion => true,
                ChangeKind::Deletion => false,
                // A moved mark prints with its side, like text.
                ChangeKind::Move => change.move_side != Some(crate::changes::MoveSide::From),
                ChangeKind::Formatting => false,
            };
            if change.kind == ChangeKind::Formatting {
                (change.text.clone(), change.text.clone())
            } else if is_insertion {
                let to = if accepted {
                    "\n\n".to_string()
                } else {
                    String::new()
                };
                ("{++\n\n++}".to_string(), to)
            } else {
                let to = if accepted {
                    String::new()
                } else {
                    "\n\n".to_string()
                };
                ("{--\n\n--}".to_string(), to)
            }
        }
        "properties" => {
            // Formatting changes no CriticMarkup: the affected text, which
            // the view shows with `fmt` on the id line, stays the same.
            (change.text.clone(), change.text.clone())
        }
        _ => {
            let move_from = change.move_side == Some(crate::changes::MoveSide::From);
            text_from_to(change.kind, move_from, &change.text, accepted)
        }
    };
    Line {
        action,
        r#ref: change.r#ref.clone(),
        place,
        from,
        to,
    }
    .render()
}

/// The lines as text: one per line, each with its newline.
pub fn resolution_text(source: &[u8], resolved: &ResolvedRevisions) -> Result<String, ChangeError> {
    let mut out = String::new();
    for line in resolution_lines(source, resolved)? {
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// The lines for a report JSON (what the hosts hold): the report's
/// `resolved_revisions` read from `report_json`.
pub fn resolution_text_from_report_json(
    source: &[u8],
    report_json: &str,
) -> Result<String, String> {
    let report: crate::edit::EditReport =
        serde_json::from_str(report_json).map_err(|e| format!("invalid report: {e}"))?;
    resolution_text(source, &report.resolved_revisions).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_escapes_quotes_and_backslashes() {
        assert_eq!(quoted("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }

    #[test]
    fn critic_escaping_mirrors_the_view() {
        assert_eq!(escape_critic("{++x++}"), "{\\++x++\\}");
        assert_eq!(escape_critic("a~>b"), "a~\\>b");
        assert_eq!(escape_critic("plain"), "plain");
    }
}
