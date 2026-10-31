// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What a refusal says when another author's revision holds the text
//! (#1108): the revision by id, author and text, then each way around it
//! with what that way costs, quoted in the syntax the caller wrote.

use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

/// The request syntax a refusal quotes, so a hint never names a form the
/// caller's parser would reject.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlanSyntax {
    /// A `schema_version: 1` JSON plan (`edit --plan`, the library).
    #[default]
    V1,
    /// The operation flags (`edit FILE -p p3 --anchor … --content …`).
    Flags,
    /// A `schema_version: 2` plan (`edit apply`).
    V2,
    /// The typed commands (`edit replace FILE --at p3 --find … --text …`),
    /// which take their decisions as `--accept-change` / `--reject-change`.
    Typed,
}

impl PlanSyntax {
    /// Every syntax, for the tests that check each one's fragments.
    pub const ALL: [Self; 4] = [Self::V1, Self::Flags, Self::V2, Self::Typed];

    /// A partial change selection, with the whole-change retry in the
    /// caller's flags. Plans retain their released wording (#1895).
    pub(super) fn incomplete_change(
        self,
        error: &crate::changes::ChangeError,
        verb: &str,
    ) -> String {
        if let (
            Self::Flags | Self::Typed,
            crate::changes::ChangeError::PartOfChange { named, whole, .. },
        ) = (self, error)
        {
            format!(
                "{named} is part of {whole}; name the whole change: {}",
                settle_revision(self, verb, whole, "")
            )
        } else {
            format!("resolve_revisions: {error}")
        }
    }

    /// Selection errors quote plan fields only for a plan caller.
    pub(super) fn selection_message(self, message: impl std::fmt::Display) -> String {
        match self {
            Self::V1 | Self::V2 => format!("resolve_revisions: {message}"),
            Self::Flags | Self::Typed => message.to_string(),
        }
    }

    /// Quote the occurrence selector in the caller's grammar. Plans retain
    /// #1013's released wording; one-shot flags also show every candidate.
    pub(super) fn ambiguous_anchor(
        self,
        text: &str,
        anchor: &str,
        hits: &[(usize, usize)],
    ) -> String {
        let shown = super::anchor::shown(anchor);
        let n = hits.len();
        let selector = match self {
            Self::Flags | Self::Typed => format!("set --occurrence 1..={n}"),
            Self::V1 | Self::V2 => format!("set \"occurrence\" to 1..={n}"),
        };
        let mut message = format!("{shown} occurs {n} times in the paragraph; {selector}");
        if self == Self::Flags {
            for (i, &(start, end)) in hits.iter().enumerate() {
                let before = if text[..start].chars().nth_back(20).is_some() {
                    "…"
                } else {
                    ""
                };
                let after = if text[end..].chars().nth(20).is_some() {
                    "…"
                } else {
                    ""
                };
                let window = super::context(text, start, end, &text[start..end]);
                message.push_str(&format!(
                    "\n  {}: {}",
                    i + 1,
                    super::anchor::shown(&format!("{before}{window}{after}"))
                ));
            }
            message.push_str(
                "\nlengthen --anchor to the words around the one you mean, or pass --occurrence",
            );
        }
        message
    }

    /// The same choice as a refusal's data records it.
    pub(super) const fn revision(self) -> super::refusal::RevisionSyntax {
        use super::refusal::RevisionSyntax;
        match self {
            Self::V1 => RevisionSyntax::V1Plan,
            Self::Flags => RevisionSyntax::LegacyFlags,
            Self::V2 => RevisionSyntax::V2Plan,
            Self::Typed => RevisionSyntax::TypedAction,
        }
    }
}

/// What another author's revision did to the text it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Held {
    Inserted,
    Deleted,
    MovedHere,
    MovedAway,
    /// A tracked formatting change (`w:pPrChange`, `w:rPrChange`).
    Formatted,
}

impl Held {
    /// The kind of `node`, a `w:ins`, `w:del`, `w:moveTo`, `w:moveFrom`,
    /// `w:pPrChange` or `w:rPrChange`.
    pub(super) fn of(dom: &Dom, node: NodeId) -> Option<Self> {
        [
            ("ins", Self::Inserted),
            ("del", Self::Deleted),
            ("moveTo", Self::MovedHere),
            ("moveFrom", Self::MovedAway),
            ("pPrChange", Self::Formatted),
            ("rPrChange", Self::Formatted),
        ]
        .into_iter()
        .find(|(local, _)| dom.name_is(node, &W::name(local)))
        .map(|(_, held)| held)
    }

    fn verb(self) -> &'static str {
        match self {
            Self::Inserted => "inserted",
            Self::Deleted => "deleted",
            Self::MovedHere => "moved here",
            Self::MovedAway => "moved away",
            Self::Formatted => "reformatted",
        }
    }

    /// The noun a cost names: `erases "Bob"'s deletion`.
    fn noun(self) -> &'static str {
        match self {
            Self::Inserted => "insertion",
            Self::Deleted => "deletion",
            Self::MovedHere | Self::MovedAway => "move",
            Self::Formatted => "formatting",
        }
    }

    /// The accept and reject routes, each with what it costs.
    fn settles(self) -> [(&'static str, &'static str); 2] {
        match self {
            Self::Deleted => [
                ("let it go", "their words leave the document for good"),
                (
                    "put it back silently",
                    "their deletion disappears, no trace",
                ),
            ],
            Self::Inserted => [
                ("keep it", "their words stay, unmarked"),
                (
                    "take it out silently",
                    "their insertion disappears, no trace",
                ),
            ],
            Self::MovedHere | Self::MovedAway => [
                ("keep the move", "the words stay where they moved, unmarked"),
                (
                    "undo it silently",
                    "the words go back where they were, no trace",
                ),
            ],
            Self::Formatted => [
                ("keep their formatting", "their formatting stays, unmarked"),
                (
                    "undo it silently",
                    "the old formatting comes back, no trace",
                ),
            ],
        }
    }
}

/// One revision a refused edit touched, as the hint names it.
pub(super) struct Holder<'a> {
    pub(super) held: Held,
    /// The printed id (`r12`, `header1.r3`).
    pub(super) id: &'a str,
    /// The revision's long id, which a v2 plan's target carries.
    pub(super) long_id: &'a str,
    pub(super) author: &'a str,
    /// The plan's author. Equal to `author` when the revision is your own.
    pub(super) actor: &'a str,
    /// [`super::Others::Keep`]: accept and reject are not offered for a
    /// revision that is not your own.
    pub(super) keep_others: bool,
    /// The revision's text; empty for a paragraph mark.
    pub(super) text: &'a str,
    /// The printed id of the paragraph that holds it (`p160`).
    pub(super) paragraph: &'a str,
    /// What the refused edit touched, up to the revision's id: `the text is
    /// inside`, `…cannot be merged: that mark is`.
    pub(super) lead: &'a str,
    /// Whether restore takes the revision, another author's deletion, as
    /// [`super::visible::restorable`] decides.
    pub(super) restorable: bool,
}

/// The `w:del` or `w:moveFrom` of `paragraph` whose text `find` reaches
/// into, reading the paragraph's visible and removed text in document
/// order, glyphs as the projection reads them (a tab as `\t`): an anchor
/// copied from another author's deletion misses the visible text, and the
/// hint should name the deletion, not the miss. The anchor matches as
/// written, as it reads without marks, and with a deletion's marks
/// unwrapped (`{--x--}` as `x`), each read past lookalike characters.
pub(super) fn removed_holder(dom: &Dom, paragraph: NodeId, find: &str) -> Option<NodeId> {
    if find.is_empty() {
        return None;
    }
    let mut text = String::new();
    let mut removed: Vec<(usize, usize, NodeId)> = Vec::new();
    for node in dom.descendants(paragraph, None) {
        // Text of a paragraph nested in a text box belongs to that one.
        if !dom.parent(node).is_some_and(|r| dom.name_is(r, &W::r()))
            || dom.ancestors(node, Some(&W::p())).first() != Some(&paragraph)
        {
            continue;
        }
        let Some(piece) = run_text(dom, node) else {
            continue;
        };
        let start = text.len();
        text.push_str(&piece);
        let holder = dom
            .ancestors(node, None)
            .into_iter()
            .find(|&a| matches!(Held::of(dom, a), Some(Held::Deleted | Held::MovedAway)));
        if let Some(holder) = holder {
            removed.push((start, text.len(), holder));
        }
    }
    let plain = crate::markdown::plain_anchor(find);
    let unwrapped = crate::markdown::plain_anchor_with_removed(find);
    [find, plain.as_str(), unwrapped.as_str()]
        .into_iter()
        .flat_map(|needle| super::anchor::reading_hits(&text, needle))
        .find_map(|(start, end)| {
            removed
                .iter()
                .find(|&&(s, e, _)| s < end && start < e)
                .map(|&(.., holder)| holder)
        })
}

/// What a run child reads as in the paragraph's text: its text, or the
/// character the projection gives a glyph; `None` for the rest.
fn run_text(dom: &Dom, node: NodeId) -> Option<String> {
    let local = dom.name(node).filter(|n| n.namespace_name() == W::URI)?;
    let glyph = match local.local_name() {
        "t" | "delText" => return Some(dom.value(node)),
        "tab" | "ptab" => '\t',
        "cr" => '\n',
        "br" if !matches!(dom.attribute(node, &W::type_()), Some("page" | "column")) => '\n',
        "noBreakHyphen" => '\u{2011}',
        "softHyphen" => '\u{00ad}',
        "sym" => '\u{fffc}',
        _ => return None,
    };
    Some(glyph.to_string())
}

/// The longest stretch of a revision's text the hint quotes.
const QUOTED: usize = 60;

/// The refusal for an edit on text `holder` holds: what holds it, then
/// accept, reject and a comment, each a fragment the caller can paste.
///
/// Another author's revision names what accepting or rejecting erases.
/// With the guard on, those two routes are left out: a deletion offers
/// restore, an insertion offers strike where the syntax has it, and a
/// move or a formatting change says plainly that settling would erase it.
pub(super) fn settle_hint(holder: &Holder<'_>, syntax: PlanSyntax) -> String {
    let mut lines = vec![format!("{}.", named(holder))];
    let foreign = holder.author != holder.actor;
    let guard = holder.keep_others && foreign;
    let noun = holder.held.noun();
    let [accept, reject] = holder.held.settles();
    let mut routes: Vec<(String, String, Option<&str>)> = Vec::new();
    if guard {
        if holder.held == Held::Inserted {
            if let Some(quote) = strike(syntax, holder) {
                routes.push((
                    "strike it".to_string(),
                    quote,
                    Some("their insertion stays, your deletion shows"),
                ));
            }
        } else if let Some(quote) = restore(syntax, holder) {
            routes.push((
                "put it back visibly".to_string(),
                quote,
                Some("their deletion stays, your insertion shows"),
            ));
        }
    } else {
        // Guard off, an insertion still offers the keeping move first,
        // where this syntax can spell a strike. Accept and reject stay,
        // and say what each one erases.
        if holder.held == Held::Inserted
            && foreign
            && let Some(quote) = strike(syntax, holder)
        {
            routes.push((
                "strike it".to_string(),
                quote,
                Some("their insertion stays, your deletion shows"),
            ));
        }
        if let Some(quote) = restore(syntax, holder) {
            routes.push((
                "put it back visibly".to_string(),
                quote,
                Some("their deletion stays, your insertion shows"),
            ));
        }
        routes.push((
            route_name(accept.0, holder, noun),
            settle(syntax, "accept", holder),
            Some(accept.1),
        ));
        routes.push((
            route_name(reject.0, holder, noun),
            settle(syntax, "reject", holder),
            Some(reject.1),
        ));
    }
    // No restore and no strike: say so, without a colon, so it is not a route.
    let bare = guard && routes.is_empty();
    routes.push((
        "or comment on it".to_string(),
        comment(syntax, holder.paragraph),
        None,
    ));
    let width = routes.iter().map(|r| r.0.len()).max().unwrap_or(0) + 1;
    let fragment = routes
        .iter()
        .filter(|r| r.2.is_some())
        .map(|r| r.1.chars().count())
        .max()
        .unwrap_or(0);
    if bare {
        lines.push("  accepting or rejecting would erase their change.".to_string());
    }
    for (name, quote, cost) in routes {
        let name = format!("{name}:");
        lines.push(match cost {
            Some(cost) => {
                let pad = " ".repeat(fragment - quote.chars().count());
                format!("  {name:<width$} {quote}{pad} ({cost})")
            }
            None => format!("  {name:<width$} {quote}"),
        });
    }
    lines.join("\n")
}

/// `base` as the route's name. Another author's revision says whose
/// markup accepting or rejecting erases; your own keeps `base`.
fn route_name(base: &str, holder: &Holder<'_>, noun: &str) -> String {
    if holder.author == holder.actor {
        base.to_string()
    } else {
        format!(r#"{base} (erases {:?}'s {noun})"#, holder.author)
    }
}

/// The refusal of an edit inside a deletion the same command restores:
/// the restore lands first, and the edit goes in a later command.
pub(super) fn restored_here_hint(holder: &Holder<'_>) -> String {
    format!(
        "{}, which this command restores: edit the restored words in a command of their own, after the restore.",
        named(holder)
    )
}

/// The lead with the revision named: its id, text, what it did, by whom.
fn named(holder: &Holder<'_>) -> String {
    let text = if holder.text.is_empty() {
        String::new()
    } else {
        format!("{} ", quoted(holder.text))
    };
    format!(
        "{} {}, {text}{} by {:?}",
        holder.lead,
        holder.id,
        holder.held.verb(),
        holder.author
    )
}

/// The target a v2 comment route names until [`super::v2_apply`] writes in
/// the refused operation's own: valid JSON, left to fill in like `find`.
pub(super) const OPERATION_TARGET: &str = r#"{"short_id":"…","long_id":"…"}"#;

/// `text` as a JSON string, quotes and backslashes escaped.
fn json(text: &str) -> String {
    serde_json::Value::from(text).to_string()
}

/// The strike of an insertion, as `syntax` spells it. A v2 plan and the
/// typed commands have no strike yet.
fn strike(syntax: PlanSyntax, holder: &Holder<'_>) -> Option<String> {
    if holder.held != Held::Inserted {
        return None;
    }
    match syntax {
        PlanSyntax::V1 => Some(format!(
            r#"{{"kind":"strike","revision":{}}}"#,
            json(holder.id)
        )),
        PlanSyntax::Flags => Some(format!("-p {} --strike", shell_word(holder.id))),
        PlanSyntax::V2 | PlanSyntax::Typed => None,
    }
}

/// The restore of the revision, as `syntax` spells it: offered for a
/// deletion with words restore takes, in a syntax that has it.
fn restore(syntax: PlanSyntax, holder: &Holder<'_>) -> Option<String> {
    if holder.held != Held::Deleted || !holder.restorable || holder.text.is_empty() {
        return None;
    }
    match syntax {
        PlanSyntax::V1 => Some(format!(
            r#"{{"kind":"restore","revision":{}}}"#,
            json(holder.id)
        )),
        PlanSyntax::Flags => Some(format!("-p {} --restore", shell_word(holder.id))),
        // A v2 plan and the typed commands have no restore yet.
        PlanSyntax::V2 | PlanSyntax::Typed => None,
    }
}

/// `id` as one shell word. A short id prints bare; a colon id carries the
/// document's `w:id` verbatim (`body:rev:1;x`), so it is single-quoted when
/// a shell would read anything in it.
fn shell_word(id: &str) -> std::borrow::Cow<'_, str> {
    let bare = !id.is_empty()
        && !id.starts_with(['#', '=', '~', '-'])
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._%:+-".contains(&b));
    if bare {
        id.into()
    } else {
        format!("'{}'", id.replace('\'', r"'\''")).into()
    }
}

/// `verb` (`accept` or `reject`) of the revision, as `syntax` spells it.
fn settle(syntax: PlanSyntax, verb: &str, holder: &Holder<'_>) -> String {
    settle_revision(syntax, verb, holder.id, holder.long_id)
}

fn settle_revision(syntax: PlanSyntax, verb: &str, id: &str, long_id: &str) -> String {
    match syntax {
        PlanSyntax::V1 => format!(
            r#""resolve_revisions":{{"{verb}":{{"ids":[{}]}}}}"#,
            json(id)
        ),
        PlanSyntax::Flags => format!("-p {} --{verb}", shell_word(id)),
        // Added to the refused command itself, which settles before it edits.
        PlanSyntax::Typed => format!("--{verb}-change {}", shell_word(id)),
        // A v2 plan's resolve_revisions names both lists.
        PlanSyntax::V2 => {
            let target = format!(
                r#"[{{"short_id":{},"long_id":{}}}]"#,
                json(id),
                json(long_id)
            );
            let (accept, reject) = if verb == "accept" {
                (target.as_str(), "[]")
            } else {
                ("[]", target.as_str())
            };
            format!(r#""resolve_revisions":{{"accept":{accept},"reject":{reject}}}"#)
        }
    }
}

/// A comment on the paragraph, as `syntax` spells it; a paragraph the
/// hint could not name is left to fill in.
fn comment(syntax: PlanSyntax, paragraph: &str) -> String {
    let paragraph = if paragraph.is_empty() {
        "…"
    } else {
        paragraph
    };
    match syntax {
        PlanSyntax::V1 => format!(
            r#"{{"kind":"comment","paragraph":{},"find":"…","text":"…"}}"#,
            json(paragraph)
        ),
        // The typed commands have no comment of their own; `add` is the
        // command line's.
        PlanSyntax::Flags | PlanSyntax::Typed => {
            format!(r#"add -p {paragraph} --anchor "…" --content "…""#)
        }
        PlanSyntax::V2 => {
            format!(r#"{{"kind":"comment","target":{OPERATION_TARGET},"find":"…","text":"…"}}"#)
        }
    }
}

/// `text` in quotes, cut to [`QUOTED`] characters with the cut said.
fn quoted(text: &str) -> String {
    if text.chars().count() <= QUOTED {
        return format!("{text:?}");
    }
    let head: String = text.chars().take(QUOTED).collect();
    format!("{head:?}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holder(held: Held) -> Holder<'static> {
        Holder {
            held,
            id: "r12",
            long_id: "document.w:del.w:id.12",
            author: "Bob Lee",
            actor: "Bob Lee",
            keep_others: false,
            text: "the bankers",
            paragraph: "p3",
            lead: "the text is inside",
            restorable: held == Held::Deleted,
        }
    }

    #[test]
    fn a_deletion_offers_restore_first_then_both_settles_then_comment() {
        assert_eq!(
            settle_hint(&holder(Held::Deleted), PlanSyntax::V1),
            [
                r#"the text is inside r12, "the bankers" deleted by "Bob Lee"."#,
                r#"  put it back visibly:  {"kind":"restore","revision":"r12"}            (their deletion stays, your insertion shows)"#,
                r#"  let it go:            "resolve_revisions":{"accept":{"ids":["r12"]}} (their words leave the document for good)"#,
                r#"  put it back silently: "resolve_revisions":{"reject":{"ids":["r12"]}} (their deletion disappears, no trace)"#,
                r#"  or comment on it:     {"kind":"comment","paragraph":"p3","find":"…","text":"…"}"#,
            ]
            .join("\n")
        );
    }

    /// A colon id carries the document's `w:id` verbatim (`body:rev:1;x`), so
    /// the command fragments quote it for the shell (review gid=54).
    #[test]
    fn a_colon_id_with_shell_syntax_is_quoted_in_command_fragments() {
        let mut odd = holder(Held::Deleted);
        odd.id = "body:rev:1;touch x";
        let flags = settle_hint(&odd, PlanSyntax::Flags);
        assert!(
            flags.contains("-p 'body:rev:1;touch x' --restore"),
            "{flags}"
        );
        assert!(
            flags.contains("-p 'body:rev:1;touch x' --accept"),
            "{flags}"
        );
        assert!(
            flags.contains("-p 'body:rev:1;touch x' --reject"),
            "{flags}"
        );
        let typed = settle_hint(&odd, PlanSyntax::Typed);
        assert!(
            typed.contains("--accept-change 'body:rev:1;touch x'"),
            "{typed}"
        );
        odd.id = "body:rev:it's";
        let flags = settle_hint(&odd, PlanSyntax::Flags);
        assert!(
            flags.contains(r#"-p 'body:rev:it'\''s' --accept"#),
            "{flags}"
        );
        // A shell-safe id prints bare, as before.
        assert!(settle_hint(&holder(Held::Deleted), PlanSyntax::Flags).contains("-p r12 --accept"));
    }

    #[test]
    fn each_syntax_quotes_its_own_fragments() {
        for syntax in PlanSyntax::ALL {
            let hint = settle_hint(&holder(Held::Deleted), syntax);
            let (wanted, foreign): (&[&str], &[&str]) = match syntax {
                PlanSyntax::V1 => (
                    &[
                        r#"{"ids":["r12"]}"#,
                        r#""paragraph":"p3""#,
                        r#"{"kind":"restore","revision":"r12"}"#,
                    ],
                    &["--accept", "short_id"],
                ),
                PlanSyntax::Flags => (
                    &[
                        "-p r12 --restore",
                        "-p r12 --accept",
                        "-p r12 --reject",
                        "add -p p3 --anchor",
                    ],
                    &["resolve_revisions"],
                ),
                PlanSyntax::V2 => (
                    &[
                        r#""accept":[{"short_id":"r12","long_id":"document.w:del.w:id.12"}]"#,
                        r#""target":"#,
                    ],
                    // A v2 plan has no restore yet.
                    &[r#""ids""#, "--accept", r#""paragraph""#, "restore"],
                ),
                PlanSyntax::Typed => (
                    &[
                        "--accept-change r12",
                        "--reject-change r12",
                        "add -p p3 --anchor",
                    ],
                    &["resolve_revisions", "-p r12"],
                ),
            };
            for want in wanted {
                assert!(hint.contains(want), "{syntax:?} lacks {want}: {hint}");
            }
            for not in foreign {
                assert!(!hint.contains(not), "{syntax:?} quotes {not}: {hint}");
            }
        }
    }

    #[test]
    fn every_kind_names_what_the_revision_did() {
        for (held, says, first) in [
            (Held::Inserted, "inserted by", "keep it:"),
            (Held::Deleted, "deleted by", "put it back visibly:"),
            (Held::MovedHere, "moved here by", "keep the move:"),
            (Held::MovedAway, "moved away by", "keep the move:"),
            (Held::Formatted, "reformatted by", "keep their formatting:"),
        ] {
            let hint = settle_hint(&holder(held), PlanSyntax::V1);
            assert!(hint.lines().next().unwrap().contains(says), "{hint}");
            assert!(
                hint.lines().nth(1).unwrap().trim_start().starts_with(first),
                "{hint}"
            );
            assert!(!hint.contains("settle it first"), "{hint}");
        }
    }

    #[test]
    fn a_deletion_restore_cannot_take_offers_no_visible_route() {
        let hint = settle_hint(
            &Holder {
                restorable: false,
                ..holder(Held::Deleted)
            },
            PlanSyntax::V1,
        );
        assert_eq!(hint.lines().count(), 4, "{hint}");
        assert!(!hint.contains("restore"), "{hint}");
    }

    #[test]
    fn the_restore_route_parses_whatever_the_id_holds() {
        let odd = Holder {
            id: r#"r"1\2"#,
            ..holder(Held::Deleted)
        };
        let hint = settle_hint(&odd, PlanSyntax::V1);
        let line = hint.lines().nth(1).unwrap();
        let route: serde_json::Value = serde_json::from_str(fragment(line))
            .unwrap_or_else(|e| panic!("not JSON ({e}): {hint}"));
        assert_eq!(
            route,
            serde_json::json!({"kind":"restore","revision": odd.id})
        );
    }

    #[test]
    fn a_mark_without_text_is_named_without_quotes() {
        let hint = settle_hint(
            &Holder {
                text: "",
                lead: "that mark is",
                ..holder(Held::Inserted)
            },
            PlanSyntax::V1,
        );
        assert_eq!(
            hint.lines().next().unwrap(),
            r#"that mark is r12, inserted by "Bob Lee"."#
        );
    }

    /// A route line's fragment: what follows its label, without the cost.
    fn fragment(line: &str) -> &str {
        let rest = line.split_once(':').map_or(line, |(_, r)| r).trim_start();
        rest.rsplit_once(" (").map_or(rest, |(f, _)| f.trim_end())
    }

    /// The route lines of `hint` as JSON: accept, reject, comment (the
    /// last three, after any visible route).
    fn routes(hint: &str) -> [serde_json::Value; 3] {
        let all: Vec<&str> = hint.lines().collect();
        let lines = &all[all.len() - 4..];
        let parse = |s: String| {
            serde_json::from_str(&s).unwrap_or_else(|e| panic!("{s} is not JSON ({e}): {hint}"))
        };
        [
            parse(format!("{{{}}}", fragment(lines[1]))),
            parse(format!("{{{}}}", fragment(lines[2]))),
            parse(fragment(lines[3]).to_string()),
        ]
    }

    #[test]
    fn every_json_route_parses_whatever_the_ids_hold() {
        let odd = Holder {
            id: r#"r"1\2"#,
            long_id: r#"document.w:del.w:id."1\2"#,
            paragraph: r#"p"3\"#,
            ..holder(Held::Deleted)
        };
        let [accept, reject, comment] = routes(&settle_hint(&odd, PlanSyntax::V1));
        assert_eq!(accept["resolve_revisions"]["accept"]["ids"][0], odd.id);
        assert_eq!(reject["resolve_revisions"]["reject"]["ids"][0], odd.id);
        assert_eq!(comment["paragraph"], odd.paragraph);
        let [accept, reject, comment] = routes(&settle_hint(&odd, PlanSyntax::V2));
        let target = serde_json::json!({"short_id": odd.id, "long_id": odd.long_id});
        assert_eq!(accept["resolve_revisions"]["accept"][0], target);
        assert_eq!(reject["resolve_revisions"]["reject"][0], target);
        assert_eq!(comment["kind"], "comment");
    }

    #[test]
    fn v2_routes_parse_as_the_plan_v2_types() {
        let hint = settle_hint(&holder(Held::Deleted), PlanSyntax::V2);
        let [accept, reject, comment] = routes(&hint);
        for route in [accept, reject] {
            serde_json::from_value::<super::super::v2::RevisionDecisions>(
                route["resolve_revisions"].clone(),
            )
            .unwrap_or_else(|e| panic!("{route} is no resolve_revisions ({e}): {hint}"));
        }
        serde_json::from_value::<super::super::v2::Operation>(comment.clone())
            .unwrap_or_else(|e| panic!("{comment} is no operation ({e}): {hint}"));
    }

    #[test]
    fn a_paragraph_the_hint_cannot_name_is_left_to_fill_in() {
        let lost = Holder {
            paragraph: "",
            ..holder(Held::Deleted)
        };
        let v1 = settle_hint(&lost, PlanSyntax::V1);
        let [.., comment] = routes(&v1);
        assert_eq!(comment["paragraph"], "…", "{v1}");
        let flags = settle_hint(&lost, PlanSyntax::Flags);
        assert!(
            flags
                .lines()
                .last()
                .unwrap()
                .ends_with(r#"add -p … --anchor "…" --content "…""#),
            "{flags}"
        );
    }

    #[test]
    fn a_long_revision_is_quoted_with_the_cut_said() {
        let text = "x".repeat(QUOTED + 5);
        let hint = settle_hint(
            &Holder {
                text: &text,
                ..holder(Held::Deleted)
            },
            PlanSyntax::V1,
        );
        assert!(
            hint.contains(&format!("\"{}\"…", "x".repeat(QUOTED))),
            "{hint}"
        );
    }

    /// Another author's revision: the plan author is not Bob Lee.
    fn foreign(held: Held) -> Holder<'static> {
        Holder {
            author: "Trainer 1 - AgentCo",
            actor: "Jubarte",
            keep_others: false,
            ..holder(held)
        }
    }

    #[test]
    fn hint_names_the_cost_of_erasing_another_authors_deletion_in_every_syntax() {
        for syntax in PlanSyntax::ALL {
            let hint = settle_hint(&foreign(Held::Deleted), syntax);
            assert!(
                hint.contains(r#"let it go (erases "Trainer 1 - AgentCo"'s deletion):"#),
                "{syntax:?}\n{hint}"
            );
            assert!(
                hint.contains(r#"put it back silently (erases "Trainer 1 - AgentCo"'s deletion):"#),
                "{syntax:?}\n{hint}"
            );
        }
    }

    #[test]
    fn hint_names_the_cost_of_erasing_another_authors_insertion_move_and_formatting() {
        for (held, first, noun) in [
            (Held::Inserted, "keep it", "insertion"),
            (Held::MovedHere, "keep the move", "move"),
            (Held::MovedAway, "undo it silently", "move"),
            (Held::Formatted, "keep their formatting", "formatting"),
        ] {
            let hint = settle_hint(&foreign(held), PlanSyntax::V1);
            let label = format!(r#"{first} (erases "Trainer 1 - AgentCo"'s {noun}):"#);
            assert!(hint.contains(&label), "{hint}");
        }
    }

    #[test]
    fn hint_guard_on_drops_accept_and_reject_for_another_author_in_every_syntax() {
        for syntax in PlanSyntax::ALL {
            let guarded = Holder {
                keep_others: true,
                ..foreign(Held::Deleted)
            };
            let hint = settle_hint(&guarded, syntax);
            assert!(
                !hint.contains("--accept")
                    && !hint.contains("\"accept\"")
                    && !hint.contains("--reject")
                    && !hint.contains("\"reject\"")
                    && !hint.contains("--accept-change")
                    && !hint.contains("--reject-change"),
                "{syntax:?}\n{hint}"
            );
            match syntax {
                PlanSyntax::V1 | PlanSyntax::Flags => {
                    assert!(hint.contains("restore"), "{syntax:?}\n{hint}");
                }
                PlanSyntax::V2 | PlanSyntax::Typed => {
                    assert!(
                        hint.contains("accepting or rejecting would erase their change"),
                        "{syntax:?}\n{hint}"
                    );
                    assert!(!hint.contains("restore"), "{syntax:?}\n{hint}");
                }
            }
        }
    }

    #[test]
    fn hint_guard_off_offers_strike_first_for_another_authors_insertion() {
        for syntax in [PlanSyntax::V1, PlanSyntax::Flags] {
            let hint = settle_hint(&foreign(Held::Inserted), syntax);
            let routes: Vec<&str> = hint.lines().skip(1).collect();
            assert!(
                routes[0].contains("strike it")
                    && routes[0].contains("their insertion stays, your deletion shows"),
                "{syntax:?}\n{hint}"
            );
            assert!(
                hint.contains(r#"keep it (erases "Trainer 1 - AgentCo"'s insertion):"#),
                "{syntax:?}\n{hint}"
            );
            assert!(
                hint.contains(
                    r#"take it out silently (erases "Trainer 1 - AgentCo"'s insertion):"#
                ),
                "{syntax:?}\n{hint}"
            );
        }
        for syntax in [PlanSyntax::V2, PlanSyntax::Typed] {
            let hint = settle_hint(&foreign(Held::Inserted), syntax);
            assert!(
                !hint.lines().nth(1).unwrap().contains("strike"),
                "{syntax:?}\n{hint}"
            );
            assert!(
                hint.contains(r#"keep it (erases "Trainer 1 - AgentCo"'s insertion):"#),
                "{syntax:?}\n{hint}"
            );
        }
        let own = settle_hint(&holder(Held::Inserted), PlanSyntax::V1);
        assert!(!own.contains("strike"), "{own}");
    }

    #[test]
    fn hint_guard_on_strikes_an_insertion_where_the_syntax_has_strike() {
        let guarded = Holder {
            keep_others: true,
            ..foreign(Held::Inserted)
        };
        let v1 = settle_hint(&guarded, PlanSyntax::V1);
        assert!(v1.contains(r#"{"kind":"strike","revision":"r12"}"#), "{v1}");
        assert!(!v1.contains("resolve_revisions"), "{v1}");
        let flags = settle_hint(&guarded, PlanSyntax::Flags);
        assert!(flags.contains("-p r12 --strike"), "{flags}");
        assert!(
            !flags.contains("--accept") && !flags.contains("--reject"),
            "{flags}"
        );
        for syntax in [PlanSyntax::V2, PlanSyntax::Typed] {
            let hint = settle_hint(&guarded, syntax);
            assert!(
                hint.contains("accepting or rejecting would erase their change"),
                "{syntax:?}\n{hint}"
            );
            assert!(!hint.contains("strike"), "{syntax:?}\n{hint}");
            assert!(!hint.contains("resolve_revisions"), "{syntax:?}\n{hint}");
        }
    }

    #[test]
    fn hint_guard_on_offers_only_a_comment_for_another_authors_move() {
        let guarded = Holder {
            keep_others: true,
            ..foreign(Held::MovedHere)
        };
        for syntax in PlanSyntax::ALL {
            let hint = settle_hint(&guarded, syntax);
            assert!(
                hint.contains("accepting or rejecting would erase their change"),
                "{syntax:?}\n{hint}"
            );
            assert!(
                !hint.contains("resolve_revisions")
                    && !hint.contains("--accept")
                    && !hint.contains("--reject-change"),
                "{syntax:?}\n{hint}"
            );
        }
    }

    #[test]
    fn hint_guard_on_keeps_the_four_routes_for_your_own_deletion() {
        let own = Holder {
            keep_others: true,
            ..holder(Held::Deleted)
        };
        let hint = settle_hint(&own, PlanSyntax::V1);
        assert!(hint.contains("  let it go:"), "{hint}");
        assert!(hint.contains("  put it back silently:"), "{hint}");
        assert!(!hint.contains("erases"), "{hint}");
    }
}
