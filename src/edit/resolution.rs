// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What `edit -p rN --accept/--reject` prints: one line per resolved change.
//!
//! `accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"` names the
//! action, the short id as the view prints it, the short paragraph id, the
//! change as CriticMarkup, and the text that replaces it. Strings are
//! JSON-quoted.
//!
//! The CriticMarkup uses the agent view's delimiters and escapes, built
//! from each change's plain text: the view's bold, italic and link markup
//! is not printed. A deletion beside an insertion in one paragraph (either
//! order, nothing unchanged between them) prints as one substitution.

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
        || crate::ids::change_id(&change.id).map_or_else(|| "-".to_string(), |id| id.story),
        crate::ids::short_paragraph,
    )
}

/// The combined ref for a substitution pair in one story, or `None` when
/// either half has no short spelling (then the halves print separately).
fn pair_ref(first: &Change, second: &Change) -> Option<String> {
    let a = crate::ids::change_id(&first.id)?;
    let b = crate::ids::change_id(&second.id)?;
    if a.story != b.story {
        return None;
    }
    let combined = if a.story == "body" {
        format!("r{}+{}", a.wid, b.wid)
    } else {
        format!("{}.r{}+{}", a.story, a.wid, b.wid)
    };
    matches!(
        combined.parse::<crate::ids::ShortId>(),
        Ok(crate::ids::ShortId::Change { .. })
    )
    .then_some(combined)
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
        crate::ids::change_id(&first.id),
        crate::ids::change_id(&second.id),
    ) else {
        return false;
    };
    a.story == b.story
}

/// Adjacent revision elements, including across the inline containers that
/// the view treats as transparent. Consecutive entries in `list_changes`
/// alone are insufficient: unchanged runs can sit between them.
fn adjacent_changes(source: &[u8]) -> Result<HashSet<(String, String)>, ChangeError> {
    use crate::namespaces::W;
    use crate::xmllinq::{Dom, NodeId};

    fn flatten(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>) {
        for child in dom.elements(node, None) {
            let name = dom.name(child);
            match name.as_ref().map(|n| n.local_name()) {
                Some("hyperlink" | "smartTag" | "customXml" | "fldSimple" | "bdo" | "dir") => {
                    flatten(dom, child, out);
                }
                Some("sdt") => {
                    if let Some(content) = dom.element(child, &W::name("sdtContent")) {
                        flatten(dom, content, out);
                    }
                }
                Some("pPr" | "proofErr") => {}
                _ => out.push(child),
            }
        }
    }

    let normalized = crate::strict_translation::strict_to_transitional_docx(source);
    let package =
        crate::opc::PartFs::open(&normalized).map_err(|e| ChangeError::Package(e.to_string()))?;
    let main = package
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".into());
    let mut adjacent = HashSet::new();
    for part in package.parts().iter().filter(|part| part.ends_with(".xml")) {
        let Some(xml) = package.part_string(part) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else { continue };
        let story = if *part == main {
            "body".into()
        } else {
            crate::ids::part_story(part)
        };
        for paragraph in dom.descendants(root, Some(&W::p())) {
            let mut children = Vec::new();
            flatten(&dom, paragraph, &mut children);
            for pair in children.windows(2) {
                if ((dom.name_is(pair[0], &W::ins()) && dom.name_is(pair[1], &W::del()))
                    || (dom.name_is(pair[0], &W::del()) && dom.name_is(pair[1], &W::ins())))
                    && let (Some(a), Some(b)) = (
                        dom.attribute(pair[0], &W::id()),
                        dom.attribute(pair[1], &W::id()),
                    )
                {
                    adjacent.insert((
                        crate::ids::ChangeId::new(&story, a).short(),
                        crate::ids::ChangeId::new(&story, b).short(),
                    ));
                }
            }
        }
    }
    Ok(adjacent)
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
    let adjacent = adjacent_changes(source)?;
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
                && adjacent.contains(&(first.id.clone(), second.id.clone()))
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
