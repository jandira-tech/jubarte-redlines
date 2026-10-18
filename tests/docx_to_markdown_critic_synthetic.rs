// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2024-2026 SylphxAI
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Copied from anymd's `anymd-formats` tests with the converter.
//!
//! Twenty documents heavy on tracked changes and comments, each checked
//! against LibreOffice Writer's own Accept All and Reject All.
//!
//! `fixtures/from-docx/critic/synthesize.py` writes ten seeded `.fodt` files in
//! `fixtures/from-docx/critic/synthetic`, and `challenging.py` ten more by hand
//! (`hard-*`), each around one hard case (see their headers for the commands). Writer saves
//! each as Word (`NAME.docx`), and `libreoffice-oracle.py --docx` saves Writer's
//! Accept All and Reject All results as `NAME.accepted.docx` and
//! `NAME.rejected.docx`, with their plain text in `.txt`.
//!
//! The converter's output for `NAME.docx`, as anymd wrote it and jubarte
//! keeps it, is kept as the source of truth for
//! regressions: `NAME.critic.md` (the default, with CriticMarkup),
//! `NAME.accept.md` and `NAME.reject.md` (`TrackChanges::Accept` and `reject`,
//! without it). After an intended change, rewrite them with
//! `JUBARTE_BLESS=1 cargo test --test docx_to_markdown_critic_synthetic` and
//! review the diff. Where Writer's result is wrong, the check against Writer
//! uses the checked file instead, and `WRITER_WRONG` says why.
//!
//! For every document:
//! - `TrackChanges::Accept` gives the same Markdown as Writer's accepted file,
//!   and `TrackChanges::Reject` as its rejected file;
//! - accepting (or rejecting) the CriticMarkup of the default output gives
//!   that same Markdown too, so the markup says exactly what changed.

mod common;

use std::path::{Path, PathBuf};

use jubarte::markdown::{MarkdownOptions, TrackChanges, docx_to_markdown, unescape_markdown};

const DOCUMENTS: [&str; 20] = [
    "all",
    "comments",
    "deletions",
    "additions",
    "comments-deletions",
    "comments-additions",
    "substitutions",
    "breaks",
    "tables",
    "structure",
    // Hand-built around one hard case each (challenging.py).
    "hard-delimiters",
    "hard-unicode",
    "hard-links",
    "hard-notes",
    "hard-lists",
    "hard-comments",
    "hard-adjacent",
    "hard-headings",
    "hard-formatting",
    "hard-tables",
];

/// Results where Writer's Accept All or Reject All disagrees with its own
/// document, and why. The checked output (`NAME.accept.md` or
/// `NAME.reject.md`) stands in for it.
const WRITER_WRONG: [(&str, &str, &str); 1] = [(
    "hard-links",
    "rejected",
    "rejecting an insertion that is a whole link leaves the full stop after it \
     linked, `[.](https://example.com/b)`, though the stop is outside the link \
     in both the .fodt and the .docx",
)];

/// The Markdown `revisions` should give for `result` (`accepted` or
/// `rejected`): Writer's own result, unless Writer is known to be wrong.
fn expected(name: &str, result: &str, revisions: TrackChanges) -> String {
    if WRITER_WRONG
        .iter()
        .any(|(n, r, _why)| *n == name && *r == result)
    {
        let golden = if result == "accepted" {
            "accept"
        } else {
            "reject"
        };
        return std::fs::read_to_string(path(&format!("{name}.{golden}.md"))).unwrap();
    }
    markdown(&format!("{name}.{result}.docx"), revisions)
}

fn path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/from-docx/critic/synthetic")
        .join(name)
}

fn markdown(name: &str, revisions: TrackChanges) -> String {
    let bytes = std::fs::read(path(name)).unwrap();
    let options = MarkdownOptions {
        track_changes: revisions,
        ..MarkdownOptions::default()
    };
    docx_to_markdown(&bytes, &options).unwrap().markdown
}

/// Replaces each `open…close` span with `keep(inner)`, matching the closer
/// lazily, as the CriticMarkup toolkit's regular expressions do.
fn spans(text: &str, open: &str, close: &str, keep: impl Fn(&str) -> String) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        let inner = start + open.len();
        let Some(length) = rest[inner..].find(close) else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push_str(&keep(&rest[inner..inner + length]));
        rest = &rest[inner + length + close.len()..];
    }
    out.push_str(rest);
    out
}

/// The Markdown after accepting (or rejecting) every change in its
/// CriticMarkup: comments and highlights drop their markup either way.
fn resolve(markdown: &str, accept: bool) -> String {
    let pick = |kept: bool, text: &str| {
        if kept {
            text.to_string()
        } else {
            String::new()
        }
    };
    let text = spans(markdown, "{>>", "<<}", |_| String::new());
    // A pilcrow attributes the following paragraph boundary. Move only
    // that separator and an ATX heading prefix into an insertion so the
    // independent span resolver can accept or reject it with the mark.
    let text = regex::Regex::new(
        r"¶\{(?:[a-z0-9_.]+\.)?(?:r[0-9]+(?:\+[0-9]+)*|noid)\b[^{}]*\}(\n\n)(#{1,6}[ \t]+)?",
    )
    .unwrap()
    .replace_all(&text, "{++$1$2++}")
    .into_owned();
    let text =
        regex::Regex::new(r"(?:¶)?\{(?:[a-z0-9_.]+\.)?(?:r[0-9]+(?:\+[0-9]+)*|noid)\b[^{}]*\}")
            .unwrap()
            .replace_all(&text, "")
            .into_owned();
    let text = spans(&text, "{==", "==}", str::to_string);
    let text = spans(&text, "{~~", "~~}", |inner| {
        let (old, new) = inner.split_once("~>").unwrap();
        if accept { new } else { old }.to_string()
    });
    let text = spans(&text, "{++", "++}", |inner| pick(accept, inner));
    let text = spans(&text, "{--", "--}", |inner| pick(!accept, inner));
    rendered(&text)
}

/// The text as Markdown renders it: the backslash escapes the converter
/// writes (CriticMarkup's delimiters, Markdown's marks) dropped, as a
/// reader takes them.
fn rendered(markdown: &str) -> String {
    unescape_markdown(markdown).replace("\\^", "^")
}

/// Paragraphs as Writer's plain text has them: one per line, no Markdown
/// syntax, no blank lines. A drawing payload's reference definition
/// (`[d1]: drawing:…`) is not the document's words, so it is no line
/// either. Space that Markdown does not show (padding in a table cell, a
/// run of spaces) is normalised; a missing space is not.
fn lines(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| {
            !line
                .split_once(": ")
                .is_some_and(|(name, rest)| name.starts_with('[') && rest.starts_with("drawing:"))
        })
        .map(|line| {
            let line = line
                .trim_start_matches('#')
                .replace("**", "")
                .replace('_', "");
            let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
            line.split('|').map(str::trim).collect::<Vec<_>>().join("|")
        })
        .filter(|line| !line.is_empty())
        .collect()
}

/// Footnote labels renumbered in order of first appearance, as a Markdown
/// renderer numbers them: the markup keeps the numbers of the marked-up
/// document, and a resolved note leaves a gap.
fn renumber(markdown: &str) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut out = String::new();
    let mut rest = markdown;
    while let Some(at) = rest.find("[^") {
        let Some(len) = rest[at + 2..].find(']') else {
            break;
        };
        let label = rest[at + 2..at + 2 + len].to_string();
        let number = match seen.iter().position(|l| *l == label) {
            Some(i) => i + 1,
            None => {
                seen.push(label);
                seen.len()
            }
        };
        out.push_str(&rest[..at]);
        out.push_str(&format!("[^{number}]"));
        rest = &rest[at + 3 + len..];
    }
    out.push_str(rest);
    out
}

/// A line without note citations: `[^N]` in Markdown, `N` or a roman numeral
/// (endnotes) right after a word in Writer's plain text.
fn without_citations(line: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut at = 0;
    while at < chars.len() {
        if chars[at] == '['
            && chars.get(at + 1) == Some(&'^')
            && let Some(len) = chars[at..].iter().position(|c| *c == ']')
        {
            at += len + 1;
            continue;
        }
        let citation = |c: &char| c.is_ascii_digit() || "ivx".contains(*c);
        if at > 0 && !chars[at - 1].is_whitespace() && citation(&chars[at]) {
            let end = (at..chars.len())
                .find(|&i| !citation(&chars[i]))
                .unwrap_or(chars.len());
            let alone = chars
                .get(end)
                .is_none_or(|c| c.is_whitespace() || c.is_ascii_punctuation());
            // After a word, or after a word and its punctuation (`control.1`).
            let word = |c: &char| c.is_alphabetic() && !citation(c);
            let after_word = word(&chars[at - 1])
                || (chars[at - 1].is_ascii_punctuation() && at > 1 && word(&chars[at - 2]));
            if alone
                && after_word
                && chars[at].is_ascii_digit() == chars[at..end].iter().all(char::is_ascii_digit)
            {
                at = end;
                continue;
            }
        }
        out.push(chars[at]);
        at += 1;
    }
    out
}

/// `[text](url)` as `text`, as Writer's plain text has it.
fn link_text(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let label = &rest[open + 1..];
        let Some((text, after)) = label.split_once("](") else {
            break;
        };
        let Some(close) = after.find(')') else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str(text);
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

#[test]
fn every_document_is_heavy_on_what_its_name_says() {
    // A deletion, its note, then an insertion.
    let replacement = regex::Regex::new(
        r"--\}(?:\{(?:[a-z0-9_.]+\.)?(?:r[0-9]+(?:\+[0-9]+)*|noid)\b[^{}]*\})?\{\+\+",
    )
    .unwrap();
    for &name in &DOCUMENTS[..10] {
        let markup = markdown(&format!("{name}.docx"), TrackChanges::All);
        let count = |delimiter: &str| markup.matches(delimiter).count();
        let (inserted, deleted, substituted, commented) =
            (count("{++"), count("{--"), count("{~~"), count("<<}"));
        let changes = inserted + deleted + substituted + count("¶{");
        assert!(changes + commented >= 10, "{name}: {markup}");
        match name {
            "comments" => assert!(commented >= 10 && changes == 0, "{name}: {markup}"),
            "deletions" => assert!(deleted >= 10 && inserted == 0, "{name}: {markup}"),
            "additions" => assert!(inserted >= 10 && deleted == 0, "{name}: {markup}"),
            // Replacements pair only when the deleted and live inserted
            // sides have the same author; other authors stay separate.
            "substitutions" => {
                let replaced = replacement.find_iter(&markup).count();
                assert!(substituted + replaced >= 5, "{name}: {markup}");
            }
            _ => {}
        }
    }
}

#[test]
fn every_output_matches_its_source_of_truth() {
    let bless = std::env::var_os("JUBARTE_BLESS").is_some();
    for name in DOCUMENTS {
        for (revisions, kind) in [
            (TrackChanges::All, "critic"),
            (TrackChanges::Accept, "accept"),
            (TrackChanges::Reject, "reject"),
        ] {
            let ours = markdown(&format!("{name}.docx"), revisions);
            let golden = path(&format!("{name}.{kind}.md"));
            if bless {
                std::fs::write(&golden, &ours).unwrap();
                continue;
            }
            let golden = std::fs::read_to_string(&golden).unwrap_or_else(|_| {
                panic!("{name}.{kind}.md is missing; run with JUBARTE_BLESS=1")
            });
            assert_eq!(ours, golden, "{name}.{kind}.md");
        }
    }
    // Without CriticMarkup means without it: a delimiter appears only as often
    // as in Writer's own text (hard-delimiters has some as plain text). The
    // converter escapes literal ones, so the count is of the rendered text.
    for name in DOCUMENTS {
        for (kind, result) in [("accept", "accepted"), ("reject", "rejected")] {
            let ours = lines(&rendered(
                &std::fs::read_to_string(path(&format!("{name}.{kind}.md"))).unwrap(),
            ))
            .join("\n");
            let writer =
                lines(&std::fs::read_to_string(path(&format!("{name}.{result}.txt"))).unwrap())
                    .join("\n");
            for delimiter in ["{++", "{--", "{~~", "{==", "{>>", "<<}"] {
                assert_eq!(
                    ours.matches(delimiter).count(),
                    writer.matches(delimiter).count(),
                    "{name}.{kind}.md: {delimiter}"
                );
            }
        }
    }
}

/// Word coalesces identical text runs when saving a settled box. Only
/// normalize those empty-property slots, keeping the rest of the XML.
fn coalesced_box_slots(markdown: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    let boxes = regex::Regex::new(r"(?s)<w:txbxContent>.*?</w:txbxContent>").unwrap();
    let slots =
        regex::Regex::new(r#"<w:r><w:rPr/><w:(t|delText)(?: xml:space="preserve")?/></w:r>"#)
            .unwrap();
    let all_slots = regex::Regex::new(r"<w:(?:t|delText)(?:\s[^>]*)?/>").unwrap();
    let mut normalized = markdown
        .lines()
        .map(|line| {
            let Some((name, payload)) = line.split_once(": drawing:") else {
                return line.to_string();
            };
            let bytes = URL_SAFE_NO_PAD.decode(payload).unwrap();
            let mut drawing: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            if drawing.get("boxes").is_none() {
                return line.to_string();
            }
            let xml = drawing["xml"].as_str().unwrap().to_string();
            let mut index = 0;
            let normalized_xml = boxes
                .replace_all(&xml, |boxed: &regex::Captures<'_>| {
                    let source = boxed.get(0).unwrap().as_str();
                    let frame = &mut drawing["boxes"][index];
                    index += 1;
                    let lengths = frame["lengths"].as_array().unwrap();
                    let mut summed: Vec<u64> = Vec::new();
                    let mut out = String::new();
                    let mut from = 0;
                    let mut slot = 0;
                    let mut previous_kind = String::new();
                    for run in slots.captures_iter(source) {
                        let range = run.get(0).unwrap();
                        let between = &source[from..range.start()];
                        for _ in all_slots.find_iter(between) {
                            summed.push(lengths[slot].as_u64().unwrap());
                            slot += 1;
                        }
                        let kind = &run[1];
                        let length = lengths[slot].as_u64().unwrap();
                        slot += 1;
                        if between.is_empty() && previous_kind == kind {
                            *summed.last_mut().unwrap() += length;
                        } else {
                            out.push_str(between);
                            out.push_str(&format!("<w:r><w:rPr/><w:{kind}/></w:r>"));
                            summed.push(length);
                        }
                        previous_kind = kind.to_string();
                        from = range.end();
                    }
                    out.push_str(&source[from..]);
                    for _ in all_slots.find_iter(&source[from..]) {
                        summed.push(lengths[slot].as_u64().unwrap());
                        slot += 1;
                    }
                    assert_eq!(slot, lengths.len(), "slot/length mismatch");
                    frame["lengths"] = serde_json::json!(summed);
                    out
                })
                .into_owned();
            drawing["xml"] = normalized_xml.into();
            format!(
                "{name}: drawing:{}",
                URL_SAFE_NO_PAD.encode(serde_json::to_vec(&drawing).unwrap())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if markdown.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

#[test]
fn revisions_accept_and_reject_match_writers_accept_all_and_reject_all() {
    for name in DOCUMENTS {
        for (revisions, result) in [
            (TrackChanges::Accept, "accepted"),
            (TrackChanges::Reject, "rejected"),
        ] {
            assert_eq!(
                coalesced_box_slots(&markdown(&format!("{name}.docx"), revisions)),
                coalesced_box_slots(&expected(name, result, revisions)),
                "{name} {result}"
            );
        }
    }
}

#[test]
fn resolving_the_markup_matches_writers_accept_all_and_reject_all() {
    for name in DOCUMENTS {
        let markup = markdown(&format!("{name}.docx"), TrackChanges::All);
        for (accept, result, revisions) in [
            (true, "accepted", TrackChanges::Accept),
            (false, "rejected", TrackChanges::Reject),
        ] {
            let ours = lines(&renumber(&resolve(&markup, accept)));
            let writer = lines(&renumber(&rendered(&expected(name, result, revisions))));
            let differ = (0..ours.len().max(writer.len())).find(|&i| ours.get(i) != writer.get(i));
            assert!(
                differ.is_none(),
                "{name} {result}, line {differ:?}:\n  markup resolved: {:?}\n  Writer:          {:?}",
                differ.and_then(|i| ours.get(i)),
                differ.and_then(|i| writer.get(i)),
            );
        }
    }
}

#[test]
fn writers_plain_text_is_in_the_accepted_and_rejected_markdown() {
    // Writer's plain text leaves tables out, so each of its lines is looked
    // for in order in our text rather than compared line for line.
    for name in DOCUMENTS {
        for (result, revisions) in [
            ("accepted", TrackChanges::Accept),
            ("rejected", TrackChanges::Reject),
        ] {
            let writer = std::fs::read_to_string(path(&format!("{name}.{result}.txt"))).unwrap();
            let ours = rendered(&without_citations(&link_text(
                &lines(&markdown(&format!("{name}.docx"), revisions)).join("\n"),
            )));
            let mut from = 0;
            for line in lines(&writer).iter().map(|line| without_citations(line)) {
                let found = ours[from..].find(line.as_str());
                assert!(
                    found.is_some(),
                    "{name} {result}: {line:?} not in order in\n{ours}"
                );
                from += found.unwrap() + line.len();
            }
        }
    }
}

/// jubarte's own CriticMarkup reader, which `markdown_to_docx` uses, reads
/// the converter's markup the same way: resolving it gives what the
/// converter writes with `TrackChanges::Accept` and `Reject`.
#[test]
fn jubartes_reader_resolves_the_markup_as_the_converter_resolves_the_document() {
    use jubarte::markdown::resolve_critic;
    for name in DOCUMENTS {
        let markup = markdown(&format!("{name}.docx"), TrackChanges::All);
        for choice in [TrackChanges::Accept, TrackChanges::Reject] {
            // Literal delimiters stay escaped in the resolved markup; the
            // converter writes them bare where there is no markup.
            let ours = lines(&renumber(&rendered(&resolve_critic(&markup, choice))));
            let direct = lines(&renumber(&rendered(&markdown(
                &format!("{name}.docx"),
                choice,
            ))));
            let differ = (0..ours.len().max(direct.len())).find(|&i| ours.get(i) != direct.get(i));
            assert!(
                differ.is_none(),
                "{name} {choice:?}, line {differ:?}:\n  resolve_critic: {:?}\n  converter:      {:?}",
                differ.and_then(|i| ours.get(i)),
                differ.and_then(|i| direct.get(i)),
            );
        }
    }
}

fn slot_probe(xml: &str, lengths: &[usize]) -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    let drawing = serde_json::json!({"xml": format!("<w:txbxContent>{xml}</w:txbxContent>"), "boxes": [{"lengths": lengths, "hash": "same", "text_hash": "same"}]});
    format!(
        "[d1]: drawing:{}\n",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&drawing).unwrap())
    )
}

#[test]
fn coalescing_box_slots_detects_dropped_words() {
    let slot = "<w:r><w:rPr/><w:t/></w:r>";
    assert_ne!(
        coalesced_box_slots(&slot_probe(&slot.repeat(2), &[4, 5])),
        coalesced_box_slots(&slot_probe(slot, &[4]))
    );
}

#[test]
fn coalescing_box_slots_preserves_slot_kind() {
    assert_ne!(
        coalesced_box_slots(&slot_probe("<w:r><w:rPr/><w:t/></w:r>", &[4])),
        coalesced_box_slots(&slot_probe("<w:r><w:rPr/><w:delText/></w:r>", &[4]))
    );
}

#[test]
fn coalescing_box_slots_detects_corrupted_lengths() {
    let xml = "<w:r><w:rPr/><w:t/></w:r>".repeat(2);
    assert_ne!(
        coalesced_box_slots(&slot_probe(&xml, &[4, 5])),
        coalesced_box_slots(&slot_probe(&xml, &[4, 0]))
    );
    assert_eq!(
        coalesced_box_slots(&slot_probe(&xml, &[4, 5])),
        coalesced_box_slots(&slot_probe("<w:r><w:rPr/><w:t/></w:r>", &[9]))
    );
}

fn box_payloads(markdown: &str) -> Vec<serde_json::Value> {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    markdown
        .lines()
        .filter_map(|line| line.split_once(": drawing:"))
        .map(|(_, payload)| {
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap()
        })
        .collect()
}

#[test]
fn every_synthetic_box_payload_is_text_free_in_every_view() {
    let words = regex::Regex::new(r"<w:(?:t|delText)(?:\s[^>]*)?>[^<]+").unwrap();
    for name in DOCUMENTS {
        for suffix in ["", ".accepted", ".rejected"] {
            for revisions in [
                TrackChanges::All,
                TrackChanges::Accept,
                TrackChanges::Reject,
            ] {
                for ids in [false, true] {
                    let bytes = std::fs::read(path(&format!("{name}{suffix}.docx"))).unwrap();
                    let view = docx_to_markdown(
                        &bytes,
                        &MarkdownOptions {
                            ids,
                            track_changes: revisions,
                            ..MarkdownOptions::default()
                        },
                    )
                    .unwrap()
                    .markdown;
                    for drawing in box_payloads(&view) {
                        if drawing.get("boxes").is_none() {
                            continue;
                        }
                        let xml = drawing["xml"].as_str().unwrap();
                        assert!(
                            !words.is_match(xml),
                            "{name}{suffix} ids={ids} {revisions:?}: {xml}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn settled_box_templates_have_no_bare_deleted_text_slots() {
    for revisions in [TrackChanges::Accept, TrackChanges::Reject] {
        let view = markdown("hard-tables.docx", revisions);
        for drawing in box_payloads(&view) {
            let xml = drawing["xml"].as_str().unwrap();
            assert!(!xml.contains("w:delText"), "{revisions:?}: {xml}");
        }
        let out =
            jubarte::markdown::markdown_to_docx(&view, &jubarte::markdown::DocxOptions::default())
                .unwrap();
        let xml = common::docx::part_string(&out.docx, "word/document.xml").unwrap();
        assert!(!xml.contains("w:delText"), "{revisions:?}: {xml}");
    }
}
