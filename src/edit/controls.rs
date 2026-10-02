// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `fill_control`: write a value into one content control (`w:sdt`) of the
//! body. The control is found by id, tag or alias; its `w:sdtPr` survives
//! (only `w:showingPlcHdr` goes, as when a person types into it) and its
//! `w:sdtContent` is replaced by one run, or one paragraph for a block-level
//! control.

use serde::{Deserialize, Serialize};

use super::{
    EditError, EditOutcome, ExistingRevisions, Resolution, Resolved, Transaction, check_text, err,
    excerpt,
};
use crate::namespaces::{W, W14};
use crate::xmllinq::{Dom, NodeId, XNamespace};

/// Which content control to fill; every form must match exactly one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ControlSelector {
    /// `"body:sdt:3"`, the same as `{"id": "body:sdt:3"}`.
    Name(String),
    /// `{"id": "body:sdt:3"}`.
    Id {
        /// `body:sdt:N` from `inspect`'s `controls`.
        id: String,
    },
    /// `{"tag": "Name"}`: the control's `w:tag`.
    Tag {
        /// Exact tag.
        tag: String,
    },
    /// `{"alias": "Full name"}`: the control's `w:alias` (its title).
    Alias {
        /// Exact alias.
        alias: String,
    },
}

/// What a resolved fill writes.
#[derive(Clone, Debug)]
pub(super) enum FillValue {
    /// Plain text (a `text` value or a choice's display text).
    Text(String),
    /// A checkbox state and the glyph that shows it.
    Checked { on: bool, glyph: char, font: String },
    /// `w:fullDate` and the text in the control's date format.
    Date { full: String, text: String },
}

/// The value fields of a `fill_control` operation.
pub(super) struct FillRequest<'a> {
    pub(super) control: &'a ControlSelector,
    pub(super) text: Option<&'a str>,
    pub(super) choice: Option<&'a str>,
    pub(super) checked: Option<bool>,
    pub(super) date: Option<&'a str>,
}

/// Kinds a fill cannot write.
const UNFILLABLE: &[&str] = &["picture", "group", "repeating", "building_block"];

impl Transaction<'_> {
    pub(super) fn resolve_fill_control(
        &self,
        id: &str,
        request: &FillRequest<'_>,
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
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let control = match self.select_control(request.control) {
            Ok(n) => n,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(code, msg, outcome));
            }
        };
        let record = &self.control_records[control];
        outcome.matches = 1;
        outcome.paragraph = record.paragraph_ids.first().cloned();
        let dom = &self.opened.dom;
        let sdt = self.controls[control];
        if self.plan.existing_revisions == ExistingRevisions::Keep {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                "fill_control is not emitted as tracked changes under existing_revisions \"keep\" yet; accept or reject the existing revisions first".into(),
                outcome,
            ));
        }
        if record.locked {
            return Err(fail(
                "LOCKED_CONTROL",
                format!("{} is locked against editing (w:lock)", record.id),
                outcome,
            ));
        }
        if UNFILLABLE.contains(&record.kind.as_str()) {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!(
                    "{} is a {} control, which fill_control cannot write",
                    record.id, record.kind
                ),
                outcome,
            ));
        }
        let content = dom.element(sdt, &W::sdt_content());
        let block = dom.ancestors(sdt, Some(&W::p())).is_empty();
        if let Some(content) = content
            && dom.elements(content, None).into_iter().any(|c| {
                [W::name("tr"), W::name("tc"), W::tbl()]
                    .iter()
                    .any(|n| dom.name_is(c, n))
            })
        {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!("{} holds table rows, cells or tables", record.id),
                outcome,
            ));
        }
        if let Some(content) = content
            && ["footnoteReference", "endnoteReference", "commentReference"]
                .iter()
                .any(|r| !dom.descendants(content, Some(&W::name(r))).is_empty())
        {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!(
                    "{} holds a footnote, endnote or comment reference the fill would remove",
                    record.id
                ),
                outcome,
            ));
        }
        if block && content.is_some_and(|c| !dom.descendants(c, Some(&W::sect_pr())).is_empty()) {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!("{} holds a section break", record.id),
                outcome,
            ));
        }
        let value = match self.fill_value(record, sdt, request) {
            Ok(value) => value,
            Err(msg) => return Err(fail("INVALID_EDIT", msg, outcome)),
        };
        let new_text = match &value {
            FillValue::Text(text) | FillValue::Date { text, .. } => text.clone(),
            FillValue::Checked { glyph, .. } => glyph.to_string(),
        };
        outcome.context = Some(format!(
            "{{⌷ {} \"{}\"→\"{}\"}}",
            record.id,
            excerpt(&record.text, 40),
            excerpt(&new_text, 40)
        ));
        let paragraphs = self
            .paragraph_nodes
            .iter()
            .enumerate()
            .filter(|&(g, _)| {
                let (story, local) = self.paragraph_story[g];
                story == 0
                    && record
                        .paragraph_ids
                        .iter()
                        .any(|p| p == &format!("body:p:{local}"))
            })
            .map(|(g, _)| g)
            .collect();
        Ok((
            vec![Resolved::FillControl {
                control,
                paragraphs,
                block,
                value,
            }],
            outcome,
        ))
    }

    /// The control index, or `(code, message, matches)`.
    fn select_control(
        &self,
        selector: &ControlSelector,
    ) -> Result<usize, (&'static str, String, usize)> {
        let records = &self.control_records;
        let (what, hits): (String, Vec<usize>) = match selector {
            ControlSelector::Name(id) | ControlSelector::Id { id } => {
                let n = id
                    .strip_prefix("body:sdt:")
                    .and_then(|n| n.parse::<usize>().ok())
                    .filter(|&n| n < records.len());
                return n.ok_or_else(|| {
                    (
                        "ANCHOR_NOT_FOUND",
                        format!(
                            "no content control {id:?}; the body has {} (body:sdt:0 to body:sdt:N)",
                            records.len()
                        ),
                        0,
                    )
                });
            }
            ControlSelector::Tag { tag } => (
                format!("tag {tag:?}"),
                (0..records.len())
                    .filter(|&n| records[n].tag.as_deref() == Some(tag))
                    .collect(),
            ),
            ControlSelector::Alias { alias } => (
                format!("alias {alias:?}"),
                (0..records.len())
                    .filter(|&n| records[n].alias.as_deref() == Some(alias))
                    .collect(),
            ),
        };
        match hits.as_slice() {
            [n] => Ok(*n),
            [] => Err((
                "ANCHOR_NOT_FOUND",
                format!("no content control has {what}"),
                0,
            )),
            many => Err((
                "AMBIGUOUS_ANCHOR",
                format!(
                    "{} content controls have {what}: {}; select one by id",
                    many.len(),
                    many.iter()
                        .map(|&n| records[n].id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                many.len(),
            )),
        }
    }

    /// Check the value form against the control and compute what to write.
    fn fill_value(
        &self,
        record: &crate::inspect::ContentControl,
        sdt: NodeId,
        request: &FillRequest<'_>,
    ) -> Result<FillValue, String> {
        let dom = &self.opened.dom;
        let given = [
            request.text.is_some(),
            request.choice.is_some(),
            request.checked.is_some(),
            request.date.is_some(),
        ]
        .iter()
        .filter(|&&g| g)
        .count();
        if given != 1 {
            return Err(format!(
                "fill_control takes exactly one of text, choice, checked, date (got {given})"
            ));
        }
        let kind = record.kind.as_str();
        let form_for = |kind: &str| match kind {
            "drop_down" => "choice",
            "checkbox" => "checked",
            "date" => "date",
            _ => "text",
        };
        let wrong = |form: &str| {
            format!(
                "{} is a {kind} control; fill it with {}, not {form}",
                record.id,
                form_for(kind)
            )
        };
        let pr = dom.element(sdt, &W::sdt_pr());
        if let Some(text) = request.text {
            if matches!(kind, "drop_down" | "checkbox" | "date") {
                return Err(wrong("text"));
            }
            check_text(text)?;
            return Ok(FillValue::Text(text.to_string()));
        }
        if let Some(choice) = request.choice {
            if !matches!(kind, "drop_down" | "combo_box") {
                return Err(wrong("choice"));
            }
            let list = pr.and_then(|pr| {
                dom.element(pr, &W::name("dropDownList"))
                    .or_else(|| dom.element(pr, &W::name("comboBox")))
            });
            let items = list.map_or_else(Vec::new, |l| dom.elements(l, Some(&W::name("listItem"))));
            for item in items {
                let value = dom.attribute(item, &W::name("value"));
                let display = dom.attribute(item, &W::name("displayText"));
                if value == Some(choice) || display == Some(choice) {
                    let shown = display.or(value).unwrap_or(choice);
                    return Ok(FillValue::Text(shown.to_string()));
                }
            }
            return Err(format!(
                "choice {choice:?} is not one of {} for {}",
                record.choices.join(", "),
                record.id
            ));
        }
        if let Some(on) = request.checked {
            if kind != "checkbox" {
                return Err(wrong("checked"));
            }
            let checkbox = pr.and_then(|pr| dom.element(pr, &W14::name("checkbox")));
            let state = checkbox.and_then(|c| {
                dom.element(
                    c,
                    &W14::name(if on { "checkedState" } else { "uncheckedState" }),
                )
            });
            let glyph = state
                .and_then(|s| dom.attribute(s, &W14::name("val")))
                .and_then(|v| u32::from_str_radix(v, 16).ok())
                .and_then(char::from_u32)
                .filter(|c| !c.is_control())
                .unwrap_or(if on { '\u{2612}' } else { '\u{2610}' });
            let font = state
                .and_then(|s| dom.attribute(s, &W14::name("font")))
                .unwrap_or("MS Gothic")
                .to_string();
            return Ok(FillValue::Checked { on, glyph, font });
        }
        let date = request.date.unwrap_or_default();
        if kind != "date" {
            return Err(wrong("date"));
        }
        let (y, m, d) = parse_date(date)
            .ok_or_else(|| format!("date {date:?} is not a calendar date in YYYY-MM-DD form"))?;
        let pattern = pr
            .and_then(|pr| dom.element(pr, &W::name("date")))
            .and_then(|e| dom.element(e, &W::name("dateFormat")))
            .and_then(|f| dom.attribute(f, &W::val()))
            .unwrap_or("yyyy-MM-dd");
        Ok(FillValue::Date {
            full: format!("{date}T00:00:00Z"),
            text: format_date(pattern, y, m, d),
        })
    }

    /// Fills against each other and against paragraph operations.
    pub(super) fn check_control_conflicts(&self, deleted: &[usize]) -> Result<(), EditError> {
        let dom = &self.opened.dom;
        let fills: Vec<(usize, usize, &Vec<usize>, bool)> = self
            .resolved
            .iter()
            .filter_map(|(i, r)| match r {
                Resolved::FillControl {
                    control,
                    paragraphs,
                    block,
                    ..
                } => Some((*i, *control, paragraphs, *block)),
                _ => None,
            })
            .collect();
        for (k, &(i, control, _, _)) in fills.iter().enumerate() {
            for &(_, other, _, _) in &fills[..k] {
                if other == control {
                    return Err(self.conflict(i, "fills a control another operation fills"));
                }
                let (a, b) = (self.controls[control], self.controls[other]);
                if dom.ancestors(a, None).contains(&b) || dom.ancestors(b, None).contains(&a) {
                    return Err(
                        self.conflict(i, "fills a control nested in another filled control")
                    );
                }
            }
        }
        for &(i, _, paragraphs, block) in &fills {
            for (j, r) in &self.resolved {
                let touched: Vec<usize> = match r {
                    Resolved::Text { para, .. }
                    | Resolved::CommentRange { para, .. }
                    | Resolved::FormatParagraph { para, .. }
                        if block =>
                    {
                        vec![*para]
                    }
                    Resolved::InsertParagraph { anchor, like, .. } if block => {
                        vec![*anchor, *like]
                    }
                    Resolved::InsertTable { anchor, .. } if block => vec![*anchor],
                    Resolved::List { paras, .. } if block => paras.clone(),
                    Resolved::CommentSpan { para, last, .. } if block => (*para..=*last).collect(),
                    Resolved::MergeParagraphs { para, next, .. } => vec![*para, *next],
                    _ => Vec::new(),
                };
                if touched.iter().any(|p| paragraphs.contains(p)) {
                    return Err(self.conflict(
                        i.max(*j),
                        "fills a control whose paragraph another operation changes",
                    ));
                }
            }
            if paragraphs.iter().any(|p| deleted.contains(p)) {
                return Err(self.conflict(i, "fills a control in a deleted paragraph"));
            }
        }
        Ok(())
    }

    /// Apply step 1b: write every fill. Text edits (step 1) never touch
    /// content-control runs, so the controls' nodes are still in place.
    pub(super) fn apply_fills(&mut self) {
        let fills: Vec<(usize, bool, FillValue)> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::FillControl {
                    control,
                    block,
                    value,
                    ..
                } => Some((*control, *block, value.clone())),
                _ => None,
            })
            .collect();
        for (control, block, value) in fills {
            fill(&mut self.opened.dom, self.controls[control], block, &value);
        }
    }
}

/// Write `value` into the control `sdt`.
fn fill(dom: &mut Dom, sdt: NodeId, block: bool, value: &FillValue) {
    let pr = match dom.element(sdt, &W::sdt_pr()) {
        Some(pr) => pr,
        None => {
            let pr = dom.new_element(W::sdt_pr());
            dom.add_first(sdt, pr);
            pr
        }
    };
    if let Some(plc) = dom.element(pr, &W::name("showingPlcHdr")) {
        dom.remove(plc);
    }
    let content = match dom.element(sdt, &W::sdt_content()) {
        Some(content) => content,
        None => {
            let content = dom.new_element(W::sdt_content());
            dom.add(sdt, content);
            content
        }
    };
    let rpr = first_run_properties(dom, content);
    let text = match value {
        FillValue::Text(text) => text.clone(),
        FillValue::Date { full, text } => {
            if let Some(date) = dom.element(pr, &W::name("date")) {
                dom.set_attribute_value(date, &W::name("fullDate"), Some(full));
            }
            text.clone()
        }
        FillValue::Checked { on, glyph, font } => {
            if let Some(checkbox) = dom.element(pr, &W14::name("checkbox")) {
                let checked = match dom.element(checkbox, &W14::name("checked")) {
                    Some(checked) => checked,
                    None => {
                        let checked = dom.new_element(W14::name("checked"));
                        dom.add_first(checkbox, checked);
                        checked
                    }
                };
                dom.set_attribute_value(
                    checked,
                    &W14::name("val"),
                    Some(if *on { "1" } else { "0" }),
                );
            }
            let rpr = rpr.unwrap_or_else(|| dom.new_element(W::r_pr()));
            set_glyph_font(dom, rpr, font);
            return write_content(dom, content, block, Some(rpr), &glyph.to_string());
        }
    };
    write_content(dom, content, block, rpr, &text);
}

/// Range markers a fill carries over: starts go before the new run, ends
/// after it, so bookmarks, comment ranges and permissions keep both halves.
const RANGE_STARTS: &[&str] = &["bookmarkStart", "commentRangeStart", "permStart"];
const RANGE_ENDS: &[&str] = &["bookmarkEnd", "commentRangeEnd", "permEnd"];

/// The range markers under `content`, detached: `(starts, ends)` in
/// document order.
fn take_range_markers(dom: &mut Dom, content: NodeId) -> (Vec<NodeId>, Vec<NodeId>) {
    let (mut starts, mut ends) = (Vec::new(), Vec::new());
    for node in dom.descendants(content, None) {
        let Some(name) = dom.name(node) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        if RANGE_STARTS.contains(&name.local_name()) {
            starts.push(node);
        } else if RANGE_ENDS.contains(&name.local_name()) {
            ends.push(node);
        }
    }
    for &node in starts.iter().chain(&ends) {
        dom.remove(node);
    }
    (starts, ends)
}

/// Replace `content`'s children with one run (or one paragraph holding it,
/// keeping the first paragraph's properties, for a block-level control).
/// Range markers inside the old content surround the new run.
fn write_content(dom: &mut Dom, content: NodeId, block: bool, rpr: Option<NodeId>, text: &str) {
    let (starts, ends) = take_range_markers(dom, content);
    let r = dom.new_element(W::r());
    if let Some(rpr) = rpr
        && !dom.elements(rpr, None).is_empty()
    {
        dom.add(r, rpr);
    }
    let t = dom.new_element(W::t());
    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(t, text);
    dom.add(r, t);
    let ppr = dom
        .descendants(content, Some(&W::p()))
        .first()
        .and_then(|&first| dom.element(first, &W::p_pr()))
        .map(|ppr| dom.clone_subtree(ppr));
    dom.remove_nodes(content);
    let holder = if block {
        let p = dom.new_element(W::p());
        if let Some(ppr) = ppr {
            for child in dom.elements(ppr, None) {
                if dom.name_is(child, &W::p_pr_change()) || dom.name_is(child, &W::sect_pr()) {
                    dom.remove(child);
                }
            }
            dom.add(p, ppr);
        }
        dom.add(content, p);
        p
    } else {
        content
    };
    for node in starts.into_iter().chain([r]).chain(ends) {
        dom.add(holder, node);
    }
}

/// A copy of the first run's properties in `content`, without the
/// placeholder style and revision marks.
fn first_run_properties(dom: &mut Dom, content: NodeId) -> Option<NodeId> {
    let source = dom
        .descendants(content, Some(&W::r()))
        .into_iter()
        .find_map(|r| dom.element(r, &W::r_pr()))?;
    let rpr = dom.clone_subtree(source);
    for child in dom.elements(rpr, None) {
        let placeholder = dom.name_is(child, &W::name("rStyle"))
            && dom.attribute(child, &W::val()) == Some("PlaceholderText");
        if placeholder
            || dom.name_is(child, &W::r_pr_change())
            || dom.name_is(child, &W::ins())
            || dom.name_is(child, &W::del())
        {
            dom.remove(child);
        }
    }
    Some(rpr)
}

/// Point every script's font at `font`, as Word does for a checkbox glyph.
fn set_glyph_font(dom: &mut Dom, rpr: NodeId, font: &str) {
    let fonts = match dom.element(rpr, &W::name("rFonts")) {
        Some(fonts) => fonts,
        None => {
            let fonts = dom.new_element(W::name("rFonts"));
            super::insert_rpr_child(dom, rpr, fonts);
            fonts
        }
    };
    for slot in ["ascii", "hAnsi", "eastAsia", "cs"] {
        dom.set_attribute_value(fonts, &W::name(slot), Some(font));
    }
    dom.set_attribute_value(fonts, &W::name("hint"), Some("eastAsia"));
}

/// `YYYY-MM-DD` with a real calendar day.
fn parse_date(date: &str) -> Option<(i32, u32, u32)> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let digits = |s: &str| -> Option<u32> {
        s.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| s.parse().ok())?
    };
    let y = i32::try_from(digits(&date[..4])?).ok()?;
    let m = digits(&date[5..7])?;
    let d = digits(&date[8..])?;
    ((1..=12).contains(&m) && (1..=days_in_month(y, m)).contains(&d)).then_some((y, m, d))
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Day of the week, 0 = Sunday (Sakamoto's method).
fn weekday(y: i32, m: u32, d: u32) -> usize {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if m < 3 { y - 1 } else { y };
    let w = y + y / 4 - y / 100 + y / 400 + T[m as usize - 1] + d as i32;
    w.rem_euclid(7) as usize
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Format a date with a Word date pattern (`yyyy`, `yy`, `MMMM`, `MMM`,
/// `MM`, `M`, `dddd`, `ddd`, `dd`, `d`; `'quoted'` text is literal; other
/// characters are copied). Names are English; `w:lid` is not consulted.
fn format_date(pattern: &str, y: i32, m: u32, d: u32) -> String {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' {
            let close = chars[i + 1..].iter().position(|&q| q == '\'');
            let end = close.map_or(chars.len(), |p| i + 1 + p);
            out.extend(&chars[i + 1..end]);
            i = end + 1;
            continue;
        }
        let run = chars[i..].iter().take_while(|&&x| x == c).count();
        match c {
            'y' => {
                if run >= 3 {
                    out.push_str(&format!("{y:04}"));
                } else {
                    out.push_str(&format!("{:02}", y.rem_euclid(100)));
                }
            }
            'M' => match run {
                1 => out.push_str(&m.to_string()),
                2 => out.push_str(&format!("{m:02}")),
                3 => out.push_str(&MONTHS[m as usize - 1][..3]),
                _ => out.push_str(MONTHS[m as usize - 1]),
            },
            'd' => match run {
                1 => out.push_str(&d.to_string()),
                2 => out.push_str(&format!("{d:02}")),
                3 => out.push_str(&DAYS[weekday(y, m, d)][..3]),
                _ => out.push_str(DAYS[weekday(y, m, d)]),
            },
            _ => out.extend(std::iter::repeat_n(c, run)),
        }
        i += run;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_parse_only_real_days() {
        assert_eq!(parse_date("2024-02-29"), Some((2024, 2, 29)));
        assert_eq!(parse_date("2000-02-29"), Some((2000, 2, 29)));
        for bad in [
            "2023-02-29",
            "1900-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-1-01",
            "2026/10/02",
            "+026-10-02",
            "abcd-10-02",
        ] {
            assert_eq!(parse_date(bad), None, "{bad}");
        }
    }

    #[test]
    fn word_date_patterns_format_in_english() {
        assert_eq!(format_date("yyyy-MM-dd", 2026, 10, 2), "2026-10-02");
        assert_eq!(format_date("d MMMM yyyy", 2027, 1, 5), "5 January 2027");
        assert_eq!(format_date("M/d/yy", 2026, 3, 9), "3/9/26");
        assert_eq!(format_date("dddd, MMM dd", 2026, 10, 2), "Friday, Oct 02");
        assert_eq!(format_date("ddd 'the' d", 2026, 10, 4), "Sun the 4");
        assert_eq!(format_date("'open", 2026, 10, 4), "open");
        assert_eq!(format_date("yyyy.MM", 2026, 10, 4), "2026.10");
    }
}
