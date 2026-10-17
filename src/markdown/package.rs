// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The `.docx` package around the written items: built-in parts, or a
//! reference document's styles, numbering, page setup, headers and footers.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{Cursor, Write as _};

use super::xml::{self, Context, Document, Relate, escape};
use super::{DocxOptions, MarkdownError, PageSize};
use crate::comments::CommentFamily;
use crate::namespaces::W;
use crate::opc::{PartFs, relative_rel_target};
use crate::xmllinq::{Dom, serialize_element};

const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const MAIN_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const WP_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";

/// Page setup when there is no reference document: US Letter, one-inch
/// margins, as Word's default template.
const LETTER_SECTION: &str = "<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
    <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
    w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/><w:cols w:space=\"720\"/></w:sectPr>";

/// Page setup for [`PageSize::A4`]: A4 with the same one-inch margins as
/// Letter. Word's own A4 template uses 2 cm margins; keeping Letter's means a
/// document changes only the page size the user asked for.
const A4_SECTION: &str = "<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/>\
    <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
    w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/><w:cols w:space=\"720\"/></w:sectPr>";

/// The section a document gets when nothing else gives one.
fn default_section(page: PageSize) -> &'static str {
    match page {
        PageSize::Letter => LETTER_SECTION,
        PageSize::A4 => A4_SECTION,
    }
}

/// The styles every document gets, so a reference without them still
/// resolves the defaults the others are based on.
const BASE_STYLES: [&str; 4] = ["Normal", "DefaultParagraphFont", "TableNormal", "NoList"];

/// Relationship types of the main part a reference document keeps; the rest
/// belong to its body, which is replaced.
const KEPT: [&str; 14] = [
    "/styles",
    "/stylesWithEffects",
    "/numbering",
    "/settings",
    "/webSettings",
    "/fontTable",
    "/theme",
    "/header",
    "/footer",
    "/footnotes",
    "/endnotes",
    "/customXml",
    "/glossaryDocument",
    "/people",
];

/// Replies (`c6 @AS re c5`) and resolved comments (`c5 @AC resolved`):
/// Word keys both by the `w14:paraId` of a comment's last paragraph, in
/// `commentsExtended.xml`. A resolved thread's replies are resolved too, as
/// Word resolves a thread whole; the view marks only its first comment.
fn threads(
    package: &mut PartFs,
    main: &str,
    comments: &[xml::Comment],
) -> Result<(), MarkdownError> {
    if !comments.iter().any(|c| c.parent.is_some() || c.done) {
        return Ok(());
    }
    let mut family = CommentFamily::load(package, main).map_err(err)?;
    for comment in comments {
        family.set_thread(comment.id, comment.parent, comment.done);
    }
    for comment in comments.iter().filter(|c| c.done && c.parent.is_none()) {
        family.set_done(comment.id, true);
    }
    family.store(package, main);
    Ok(())
}

fn err(message: impl std::fmt::Display) -> MarkdownError {
    MarkdownError::Package(message.to_string())
}

/// The finished package.
pub(super) fn assemble(
    document: &Document,
    options: &DocxOptions<'_>,
) -> Result<Vec<u8>, MarkdownError> {
    let template = match options.reference {
        Some(bytes) => bytes.to_vec(),
        None => default_template(options.page).map_err(err)?,
    };
    let mut package = PartFs::open(&template)
        .map_err(|e| MarkdownError::Reference(format!("not a readable .docx ({e})")))?;
    let main = package
        .main_document_part()
        .ok_or_else(|| MarkdownError::Reference("it has no main document part".into()))?;
    let source = package
        .part_string(&main)
        .ok_or_else(|| MarkdownError::Reference(format!("'{main}' cannot be read")))?;
    // A reference's page setup wins; `page` only fills in for its absence
    // when there is no reference at all.
    let page = match options.reference {
        Some(_) => PageSize::Letter,
        None => options.page,
    };
    let (root, section) = frame(&source, page);
    prune(&mut package, &main);

    let styles_part = styles_part(&mut package, &main);
    let mut wanted: BTreeSet<&str> = BASE_STYLES.into_iter().collect();
    wanted.extend(document.styles.iter().copied());
    // The headers and footers the view names take Word's styles for them.
    for story in document.stories.values().flat_map(|s| &s.stories) {
        wanted.insert(if story.kind == "header" {
            "Header"
        } else {
            "Footer"
        });
        // A header in the Footer style, or the reverse, takes that one too.
        for style in &story.styles {
            if let Some(own) = ["Header", "Footer"].into_iter().find(|own| style == own) {
                wanted.insert(own);
            }
        }
    }
    // And the styles they name (`style: ListParagraph`), as id lines do.
    let mut named = document.named_styles.clone();
    named.extend(
        document
            .stories
            .values()
            .flat_map(|s| &s.stories)
            .flat_map(|story| &story.styles)
            .filter(|id| !matches!(id.as_str(), "Normal" | "Header" | "Footer"))
            .cloned(),
    );
    // And the comments' (`comments: …, style X`), CommentText aside.
    named.extend(
        document
            .comment_style
            .iter()
            .chain(document.comment_styles.values())
            .filter(|id| !matches!(id.as_str(), "Normal" | "CommentText"))
            .cloned(),
    );
    if let Some(styles) = package.part_string(&styles_part) {
        let styles = with_styles(&styles, &wanted);
        let styles = with_named_styles(&styles, &named);
        let styles = super::view_styles::apply(&styles, &document.style_specs);
        let styles = super::view_styles::with_style_changes(&styles, &document.style_changes);
        let theme = related(&package, &main, "/theme")
            .and_then(|part| package.part_string(&part))
            .unwrap_or_else(|| crate::word_default_theme::WORD_DEFAULT_THEME.to_string());
        let styles = with_table_styles(&styles, &document.table_styles, &theme);
        package.set_part(&styles_part, styles.into_bytes());
    }

    let num_ids = numbering(&mut package, &main, document);
    let label_nums = label_numbering(&mut package, &main, document);
    // A view's YAML header names its sections' pages, headers and footers.
    let sections = document.section_breaks() + 1;
    let next_revision = document
        .revision_ids
        .values()
        .flatten()
        .chain(&document.reserved_ids)
        .chain(document.note_formats.iter().flat_map(|(_, tag)| &tag.ids))
        .max()
        .map_or(1, |max| max.saturating_add(1));
    let people = super::view_stories::People {
        by_handle: document.people.clone(),
        author: options.author.clone(),
        date: options.date.clone(),
        range_ends: document.story_range_ends.clone(),
        drawing_defs: document.drawing_defs.clone(),
        drawing_backings: document.drawing_backings.clone(),
        ..super::view_stories::People::default()
    };
    // A header's fresh ids sit far above the body's.
    let section_stories = super::view_stories::write(
        &mut package,
        &main,
        &document.stories,
        sections,
        &people,
        next_revision
            .saturating_add(1_000_000)
            .min(super::view_notes::MOST_ID / 2),
        document
            .style_specs
            .iter()
            .find(|s| s.default)
            .map_or("Normal", |s| s.id.as_str()),
    );
    let default_section = section.clone();
    let section = section_stories.apply(sections, &document.section(sections, &section));
    let text_width = text_width(&section);
    let next_drawing = max_drawing_id(&package) + 1;
    let mut context = Context {
        document,
        author: escape(&options.author).into_owned(),
        date: escape(&options.date).into_owned(),
        next_revision,
        cursors: HashMap::new(),
        used_revisions: HashSet::new(),
        pending_marks: Vec::new(),
        num_ids,
        label_nums,
        note_ids: Vec::new(),
        in_endnote: false,
        next_drawing,
        next_bookmark: 0,
        text_width,
        sections_ended: 0,
        default_section,
        section_stories,
    };
    let mut media = Media::default();

    if !document.notes.is_empty()
        || !document.empty_notes.is_empty()
        || !document.note_formats.is_empty()
    {
        // The separators' formatting changes; a tag that numbers no id
        // takes one far above the body's and the headers'.
        let separator_changes: HashMap<&str, Vec<String>> = ["footnote", "endnote"]
            .into_iter()
            .map(|kind| {
                let tags: Vec<_> = document
                    .note_formats
                    .iter()
                    .filter(|(k, _)| *k == kind)
                    .map(|(_, tag)| tag)
                    .collect();
                let changes = super::view_stories::format_changes(
                    &people,
                    &tags,
                    next_revision
                        .saturating_add(2_000_000)
                        .min(super::view_notes::MOST_ID / 2 + 1_000_000),
                );
                (kind, changes)
            })
            .collect();
        footnotes(
            &mut package,
            &main,
            &mut context,
            &mut media,
            &separator_changes,
        );
    }
    let body = {
        let mut relate = PartRelate {
            package: &mut package,
            part: main.clone(),
            media: &mut media,
            document,
        };
        xml::story(&mut context, &document.body, &mut relate, None)
    };
    if !document.comments.is_empty() {
        let part = sibling(&main, "comments.xml");
        package.set_part(
            &part,
            xml::comments_part(&context, &document.comments).into_bytes(),
        );
        package.add_content_type_override(
            &format!("/{part}"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        );
        package.add_document_relationship(
            &main,
            &format!("{RELS}/comments"),
            &relative_rel_target(&main, &part),
        );
        threads(&mut package, &main, &document.comments)?;
    }
    core_properties(&mut package, document);
    package.set_part(
        &main,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             {root}<w:body>{body}{section}</w:body></w:document>"
        )
        .into_bytes(),
    );
    if package.content_type_for(&format!("/{main}")).is_none() {
        package.add_content_type_override(&format!("/{main}"), MAIN_CT);
    }
    package.to_zip().map_err(err)
}

/// The styles part `main` relates to, made empty when there is none.
pub(crate) fn styles_part(package: &mut PartFs, main: &str) -> String {
    related(package, main, "/styles").unwrap_or_else(|| {
        let part = sibling(main, "styles.xml");
        package.add_document_relationship(
            main,
            &format!("{RELS}/styles"),
            &relative_rel_target(main, &part),
        );
        package.add_content_type_override(
            &format!("/{part}"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        );
        package.set_part(
            &part,
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                 <w:styles xmlns:w=\"{W_NS}\"></w:styles>"
            )
            .into_bytes(),
        );
        part
    })
}

/// The part next to `main` named `name` (`word/` + `name`).
/// Whether a reference document's body names headers or footers of its
/// own, which the written document keeps.
pub(super) fn has_stories(reference: &[u8]) -> bool {
    let Ok(package) = PartFs::open(reference) else {
        return false;
    };
    package
        .main_document_part()
        .and_then(|main| package.part_string(&main))
        .is_some_and(|xml| xml.contains("<w:headerReference") || xml.contains("<w:footerReference"))
}

pub(super) fn sibling(main: &str, name: &str) -> String {
    match main.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{name}"),
        None => name.to_string(),
    }
}

/// The part `main` relates to with the relationship type ending in `kind`.
pub(super) fn related(package: &PartFs, main: &str, kind: &str) -> Option<String> {
    let rels = package.read_rels_for(main)?;
    let rel = rels
        .items
        .iter()
        .find(|r| r.rel_type.ends_with(kind) && r.target_mode.as_deref() != Some("External"))?;
    let part = package.resolve_rel_target(main, &rel.target);
    package.part_bytes(&part).is_some().then_some(part)
}

/// The document element's start tag, declaring the prefixes the body uses,
/// and the body's final section properties.
fn frame(source: &str, page: PageSize) -> (String, String) {
    let fallback = default_section(page);
    let default_root =
        format!("<w:document xmlns:w=\"{W_NS}\" xmlns:r=\"{R_NS}\" xmlns:wp=\"{WP_NS}\">");
    let Some(start) = source.find("<w:document") else {
        return (default_root, fallback.to_string());
    };
    let Some(length) = source[start..].find('>') else {
        return (default_root, fallback.to_string());
    };
    let mut root = source[start..start + length].to_string();
    if root.ends_with('/') {
        return (default_root, fallback.to_string());
    }
    for (prefix, namespace) in [("w", W_NS), ("r", R_NS), ("wp", WP_NS)] {
        if !root.contains(&format!("xmlns:{prefix}=")) {
            root.push_str(&format!(" xmlns:{prefix}=\"{namespace}\""));
        }
    }
    root.push('>');
    (
        root,
        final_section(source).unwrap_or_else(|| fallback.to_string()),
    )
}

/// The `w:sectPr` that is the body's last child, without a tracked change
/// of its own (`w:sectPrChange`): the reference's revisions are not the
/// written document's.
fn final_section(source: &str) -> Option<String> {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(source);
    let root = dom.root(document)?;
    let body = dom.element(root, &W::body())?;
    let last = *dom.elements(body, None).last()?;
    if !dom.name_is(last, &W::sect_pr()) {
        return None;
    }
    for change in dom.elements(last, Some(&W::name("sectPrChange"))) {
        dom.remove(change);
    }
    Some(serialize_element(&dom, last))
}

/// The page width less the side margins, in twentieths of a point.
fn text_width(section: &str) -> u32 {
    let attribute = |element: &str, name: &str| -> Option<u32> {
        let at = section.find(&format!("<w:{element} "))?;
        let tag = &section[at..at + section[at..].find('>')?];
        let key = format!("w:{name}=\"");
        let value = &tag[tag.find(&key)? + key.len()..];
        value[..value.find('"')?].parse().ok()
    };
    let width = attribute("pgSz", "w").unwrap_or(12240);
    let left = attribute("pgMar", "left").unwrap_or(1440);
    let right = attribute("pgMar", "right").unwrap_or(1440);
    width.saturating_sub(left + right).max(1440)
}

/// Drops the reference body's relationships (pictures, links, comments...)
/// and their parts, and the notes the body referenced.
fn prune(package: &mut PartFs, main: &str) {
    let dropped: Vec<String> = package
        .read_rels_for(main)
        .map(|rels| {
            rels.items
                .iter()
                .filter(|r| !KEPT.iter().any(|kind| r.rel_type.ends_with(kind)))
                .map(|r| r.id.clone())
                .collect()
        })
        .unwrap_or_default();
    for id in dropped {
        let external = package
            .read_rels_for(main)
            .and_then(|rels| rels.items.iter().find(|r| r.id == id))
            .is_some_and(|r| r.target_mode.as_deref() == Some("External"));
        if external {
            let rel_type = package
                .read_rels_for(main)
                .and_then(|rels| rels.items.iter().find(|r| r.id == id))
                .map(|r| r.rel_type.clone())
                .unwrap_or_default();
            // External targets have no part: drop every one of this type.
            package.remove_relationships_by_type(main, &rel_type);
        } else {
            package.remove_related_part(main, &id);
        }
    }
    for kind in ["/footnotes", "/endnotes"] {
        if let Some(part) = related(package, main, kind)
            && let Some(xml) = package.part_string(&part)
        {
            package.set_part(&part, separators_only(&xml).into_bytes());
        }
    }
}

/// A notes part with only its separator notes.
fn separators_only(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    loop {
        let next = ["<w:footnote ", "<w:footnote>", "<w:endnote ", "<w:endnote>"]
            .iter()
            .filter_map(|tag| rest.find(tag).map(|at| (at, *tag)))
            .min_by_key(|(at, _)| *at);
        let Some((at, tag)) = next else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..at]);
        let name = if tag.starts_with("<w:footnote") {
            "w:footnote"
        } else {
            "w:endnote"
        };
        let close = format!("</{name}>");
        let open_end = rest[at..].find('>').map_or(rest.len(), |i| at + i + 1);
        let self_closing = rest[..open_end].ends_with("/>");
        let end = if self_closing {
            open_end
        } else {
            rest[at..]
                .find(&close)
                .map_or(rest.len(), |i| at + i + close.len())
        };
        let note = &rest[at..end];
        let start_tag = &rest[at..open_end];
        if start_tag.contains("w:type=\"separator\"")
            || start_tag.contains("w:type=\"continuationSeparator\"")
            || start_tag.contains("w:type=\"continuationNotice\"")
        {
            out.push_str(note);
        }
        rest = &rest[end..];
    }
}

/// The table style `xml` with its `w:name` replaced by `name`: a
/// built-in template keeps Word's definition, the name is the
/// document's own.
fn renamed(xml: &str, name: &str) -> String {
    const OPEN: &str = "<w:name w:val=\"";
    let Some(at) = xml.find(OPEN) else {
        return xml.to_string();
    };
    let value = at + OPEN.len();
    let Some(end) = xml[value..].find('"').map(|e| value + e) else {
        return xml.to_string();
    };
    format!(
        "{}{}{}",
        &xml[..value],
        escape(&word_name(name)),
        &xml[end..]
    )
}

/// The `w:name` value for `name`: no comma, which `w:name` forbids
/// (`[^,]*`; Word reads one as the start of an alias list), so every
/// docx the writer makes is Word-valid. The comma becomes a space, as
/// the paragraph styles' names do.
fn word_name(name: &str) -> String {
    name.replace(',', " ")
}

/// `styles` with a definition for each table style the view names that it
/// lacks: one of Word's built-in Grid and List Tables as Word defines it,
/// its colours from `theme`; else the name is the view's (`named …`),
/// Word's own for an id Word names, else the id, on Word's Normal Table,
/// with the legend's border all round when it gives one.
fn with_table_styles(
    styles: &str,
    wanted: &BTreeMap<String, (Option<u32>, Option<String>)>,
    theme: &str,
) -> String {
    let missing: String = wanted
        .iter()
        .filter(|(id, _)| !styles.contains(&format!("w:styleId=\"{}\"", escape(id))))
        .map(|(id, (border, name))| {
            if let Some(style) = built_in_table_style(id, theme) {
                // The legend's `named` renames even Word's own template:
                // the document renamed the built-in, and the name is its
                // (#549).
                if let Some(name) = name {
                    return renamed(&style, name);
                }
                return style;
            }
            let borders = (*border).map_or_else(String::new, |sz| {
                let sides: String = ["top", "left", "bottom", "right", "insideH", "insideV"]
                    .iter()
                    .map(|side| {
                        format!("<w:{side} w:val=\"single\" w:sz=\"{sz}\" w:space=\"0\" w:color=\"auto\"/>")
                    })
                    .collect();
                format!("<w:tblPr><w:tblBorders>{sides}</w:tblBorders></w:tblPr>")
            });
            let name = name
                .clone()
                .or_else(|| word_table_style_name(id))
                .unwrap_or_else(|| id.clone());
            format!(
                "<w:style w:type=\"table\" w:styleId=\"{id}\"><w:name w:val=\"{name}\"/>\
                 <w:basedOn w:val=\"TableNormal\"/>{borders}</w:style>",
                id = escape(id),
                name = escape(&word_name(&name))
            )
        })
        .collect();
    match styles.rfind("</w:styles>") {
        Some(at) if !missing.is_empty() => {
            format!("{}{missing}{}", &styles[..at], &styles[at..])
        }
        _ => styles.to_string(),
    }
}

/// Word's built-in Grid Table and List Table styles (1 to 7, plain and
/// Accent 1 to 6), one template per family as Word writes them, `{C}` for
/// the theme colour and `@` for each colour resolved from it (written by
/// scripts/table_styles_asset.py).
const WORD_TABLE_STYLES: &str = include_str!("word_table_styles.tsv");

/// The definition of `id` when it is one of Word's built-in Grid or List
/// Tables (`GridTable4-Accent1`, `ListTable6Colorful`), named as Word names
/// it (`Grid Table 4 Accent 1`), its colours from `theme`.
fn built_in_table_style(id: &str, theme: &str) -> Option<String> {
    let (name, body) = built_in(id)?;
    Some(format!(
        "<w:style w:type=\"table\" w:styleId=\"{}\"><w:name w:val=\"{}\"/>{}</w:style>",
        escape(id),
        escape(&name),
        with_theme_colours(&body, theme)
    ))
}

/// `id` as Word names it and its unstyled body, when it is one of the
/// built-in Grid or List Tables.
fn built_in(id: &str) -> Option<(String, String)> {
    let (family, accent) = match id.split_once("-Accent") {
        Some((family, n)) if matches!(n, "1" | "2" | "3" | "4" | "5" | "6") => (family, Some(n)),
        Some(_) => return None,
        None => (id, None),
    };
    let template = |variant: &str| {
        WORD_TABLE_STYLES.lines().find_map(|line| {
            let mut fields = line.splitn(3, '\t');
            (fields.next()? == family && fields.next()? == variant)
                .then(|| fields.next())
                .flatten()
        })
    };
    let body = match accent {
        Some(n) => template("accent")?.replace("{C}", &format!("accent{n}")),
        None => match template("plain") {
            Some(plain) => plain.to_string(),
            None => template("accent")?.replace("{C}", "text1"),
        },
    };
    // `GridTable1Light` is `Grid Table 1 Light`.
    let mut name = String::new();
    let mut previous = ' ';
    for c in family.chars() {
        if !name.is_empty()
            && (c.is_ascii_uppercase() || (c.is_ascii_digit() && !previous.is_ascii_digit()))
        {
            name.push(' ');
        }
        name.push(c);
        previous = c;
    }
    if let Some(n) = accent {
        name.push_str(" Accent ");
        name.push_str(n);
    }
    Some((name, body))
}

/// Whether `id` is one of Word's built-in Grid or List Tables, which the
/// writer defines as Word does.
pub(super) fn is_built_in_table_style(id: &str) -> bool {
    built_in_table_style(id, "").is_some()
}

/// The table-style families Word named before its Grid and List Tables
/// (`LightShading-Accent1`, `MediumList2`), each with the accents 1 to 6.
const OLD_TABLE_FAMILIES: [&str; 15] = [
    "TableGrid",
    "LightShading",
    "LightList",
    "LightGrid",
    "MediumShading1",
    "MediumShading2",
    "MediumList1",
    "MediumList2",
    "MediumGrid1",
    "MediumGrid2",
    "MediumGrid3",
    "DarkList",
    "ColorfulShading",
    "ColorfulList",
    "ColorfulGrid",
];

/// Word's name for a built-in table style `id`: the Grid and List Tables
/// from their definitions, the older families and the two defaults by
/// the id itself (`TableGrid` → `Table Grid`, `LightShading-Accent1` →
/// `Light Shading Accent 1`). `None` for an id Word does not name, whose
/// name is the id until the view says another (#528).
pub(super) fn word_table_style_name(id: &str) -> Option<String> {
    if let Some(style) = built_in_table_style(id, "") {
        let open = "<w:name w:val=\"";
        let at = style.find(open)? + open.len();
        let end = style[at..].find('"')?;
        return Some(style[at..at + end].to_string());
    }
    let (family, accent) = match id.split_once("-Accent") {
        Some((family, n)) if matches!(n, "1" | "2" | "3" | "4" | "5" | "6") => {
            (family, format!(" Accent {n}"))
        }
        _ => (id, String::new()),
    };
    if family == "TableNormal" {
        return Some("Normal Table".to_string());
    }
    OLD_TABLE_FAMILIES.contains(&family).then(|| {
        let mut name = String::new();
        for c in family.chars() {
            if c.is_uppercase() && !name.is_empty() {
                name.push(' ');
            }
            if c.is_ascii_digit() && name.chars().last().is_some_and(|last| last.is_alphabetic()) {
                name.push(' ');
            }
            name.push(c);
        }
        format!("{name}{accent}")
    })
}

/// `xml` with each `@` colour resolved from `theme` as Word resolves a
/// theme colour: the element's `w:themeColor` (or `w:themeFill`), lightened
/// by its tint and darkened by its shade; `auto` for a colour the theme
/// does not hold.
fn with_theme_colours(xml: &str, theme: &str) -> String {
    xml.split_inclusive('>')
        .map(|tag| {
            if !tag.contains("\"@\"") {
                return tag.to_string();
            }
            let attribute = |name: &str| {
                tag.split_once(&format!(" w:{name}=\""))
                    .and_then(|(_, rest)| rest.split_once('"'))
                    .map(|(value, _)| value)
            };
            let mut out = tag.to_string();
            for (key, colour, tint, shade) in [
                ("color", "themeColor", "themeTint", "themeShade"),
                ("val", "themeColor", "themeTint", "themeShade"),
                ("fill", "themeFill", "themeFillTint", "themeFillShade"),
            ] {
                let unresolved = format!(" w:{key}=\"@\"");
                if out.contains(&unresolved) {
                    let hex = attribute(colour)
                        .and_then(|name| {
                            theme_colour(theme, name, attribute(tint), attribute(shade))
                        })
                        .unwrap_or_else(|| "auto".to_string());
                    out = out.replace(&unresolved, &format!(" w:{key}=\"{hex}\""));
                }
            }
            out
        })
        .collect()
}

/// The theme colour `name` (`accent1`, `text1`), tinted and shaded as Word
/// does: by lightness, in HSL, each channel truncated.
/// The prefix (`a:`, or none for a default namespace) a theme part binds
/// to DrawingML: any name is valid, `a` is only Word's.
fn drawing_prefix(theme: &str) -> String {
    const DRAWINGML: &str = "=\"http://schemas.openxmlformats.org/drawingml/2006/main\"";
    theme
        .match_indices(DRAWINGML)
        .find_map(|(at, _)| {
            let before = &theme[..at];
            let name = &before[before.rfind(char::is_whitespace)? + 1..];
            match name.strip_prefix("xmlns") {
                Some("") => Some(String::new()),
                Some(prefix) => prefix.strip_prefix(':').map(|p| format!("{p}:")),
                None => None,
            }
        })
        .unwrap_or_else(|| "a:".to_string())
}

fn theme_colour(
    theme: &str,
    name: &str,
    tint: Option<&str>,
    shade: Option<&str>,
) -> Option<String> {
    let slot = match name {
        "text1" | "dark1" => "dk1",
        "background1" | "light1" => "lt1",
        "text2" | "dark2" => "dk2",
        "background2" | "light2" => "lt2",
        "hyperlink" => "hlink",
        "followedHyperlink" => "folHlink",
        other => other,
    };
    let a = drawing_prefix(theme);
    let open = format!("<{a}{slot}>");
    let at = theme.find(&open)? + open.len();
    let scheme = &theme[at..at + theme[at..].find(&format!("</{a}{slot}>"))?];
    let hex = ["lastClr=\"", "val=\""].iter().find_map(|key| {
        let (_, rest) = scheme.split_once(key)?;
        rest.get(..6)
            .filter(|h| h.bytes().all(|b| b.is_ascii_hexdigit()))
    })?;
    // Neither tint nor shade: the theme's colour as it is.
    if tint.is_none() && shade.is_none() {
        return Some(hex.to_ascii_uppercase());
    }
    let channel =
        |at: usize| f64::from(u8::from_str_radix(&hex[at..at + 2], 16).unwrap_or(0)) / 255.0;
    let (h, mut l, s) = rgb_to_hls(channel(0), channel(2), channel(4));
    let fraction = |value: Option<&str>| {
        value
            .and_then(|v| u8::from_str_radix(v, 16).ok())
            .map(|v| f64::from(v) / 255.0)
    };
    if let Some(t) = fraction(tint) {
        l = l * t + (1.0 - t);
    }
    if let Some(t) = fraction(shade) {
        l *= t;
    }
    let (r, g, b) = hls_to_rgb(h, l, s);
    // Truncated, as Word writes it: each value is within 0..=255.
    let byte = |v: f64| (v * 255.0).clamp(0.0, 255.0).floor();
    Some(format!(
        "{:02X}{:02X}{:02X}",
        byte(r) as u8,
        byte(g) as u8,
        byte(b) as u8
    ))
}

fn rgb_to_hls(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, l, 0.0);
    }
    let d = max - min;
    let s = if l <= 0.5 {
        d / (max + min)
    } else {
        d / (2.0 - max - min)
    };
    let (rc, gc, bc) = ((max - r) / d, (max - g) / d, (max - b) / d);
    let h = if (r - max).abs() < f64::EPSILON {
        bc - gc
    } else if (g - max).abs() < f64::EPSILON {
        2.0 + rc - bc
    } else {
        4.0 + gc - rc
    };
    ((h / 6.0).rem_euclid(1.0), l, s)
}

fn hls_to_rgb(h: f64, l: f64, s: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let m2 = if l <= 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let m1 = 2.0 * l - m2;
    let value = |hue: f64| {
        let hue = hue.rem_euclid(1.0);
        if hue < 1.0 / 6.0 {
            m1 + (m2 - m1) * hue * 6.0
        } else if hue < 0.5 {
            m2
        } else if hue < 2.0 / 3.0 {
            m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
        } else {
            m1
        }
    };
    (value(h + 1.0 / 3.0), value(h), value(h - 1.0 / 3.0))
}

fn with_styles(styles: &str, wanted: &BTreeSet<&str>) -> String {
    let missing: String = wanted
        .iter()
        .filter(|id| !styles.contains(&format!("w:styleId=\"{id}\"")))
        .filter_map(|id| xml::style_definition(id))
        .collect();
    match styles.rfind("</w:styles>") {
        Some(at) if !missing.is_empty() => {
            format!("{}{missing}{}", &styles[..at], &styles[at..])
        }
        _ => styles.to_string(),
    }
}

/// `styles` with a definition for each style an id line names that it
/// lacks: the reader's own definition when it has one (`Quote`), else a
/// paragraph style on Normal.
fn with_named_styles(styles: &str, named: &BTreeSet<String>) -> String {
    let missing: String = named
        .iter()
        .filter(|id| !styles.contains(&format!("w:styleId=\"{}\"", escape(id))))
        .map(|id| xml::style_definition(id).unwrap_or_else(|| xml::named_style_definition(id)))
        .collect();
    match styles.rfind("</w:styles>") {
        Some(at) if !missing.is_empty() => {
            format!("{}{missing}{}", &styles[..at], &styles[at..])
        }
        _ => styles.to_string(),
    }
}

/// The largest value of attribute `name` on elements `element` in `xml`.
fn max_attribute(xml: &str, element: &str, name: &str) -> Option<i64> {
    attribute_values(xml, element, name).into_iter().max()
}

/// Every numeric `name` attribute of `element` in `xml`.
fn attribute_values(xml: &str, element: &str, name: &str) -> Vec<i64> {
    let mut values = Vec::new();
    let mut rest = xml;
    let open = format!("<{element} ");
    let key = format!("{name}=\"");
    while let Some(at) = rest.find(&open) {
        let tag_end = rest[at..].find('>').map_or(rest.len(), |i| at + i);
        let tag = &rest[at..tag_end];
        if let Some(value) = tag
            .find(&key)
            .map(|i| &tag[i + key.len()..])
            .and_then(|v| v[..v.find('"')?].parse::<i64>().ok())
        {
            values.push(value);
        }
        rest = &rest[tag_end..];
    }
    values
}

/// The numbering part `main` relates to, made empty when there is none.
pub(crate) fn numbering_part(package: &mut PartFs, main: &str) -> String {
    related(package, main, "/numbering").unwrap_or_else(|| {
        let part = sibling(main, "numbering.xml");
        package.add_document_relationship(
            main,
            &format!("{RELS}/numbering"),
            &relative_rel_target(main, &part),
        );
        package.add_content_type_override(
            &format!("/{part}"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        );
        package.set_part(
            &part,
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                 <w:numbering xmlns:w=\"{W_NS}\"></w:numbering>"
            )
            .into_bytes(),
        );
        part
    })
}

/// Adds the document's lists to the numbering part, making one if needed;
/// returns each list's `w:numId`.
fn numbering(package: &mut PartFs, main: &str, document: &Document) -> Vec<u32> {
    if document.lists.is_empty() {
        return Vec::new();
    }
    let Some((_, first_num)) = append_numbering(package, main, |bullets, first_num| {
        xml::numbering(&document.lists, bullets, bullets + 1, first_num)
    }) else {
        return Vec::new();
    };
    (0..document.lists.len())
        .map(|i| first_num + u32::try_from(i).unwrap_or(0))
        .collect()
}

/// Adds a list for each label an id line names; returns each label's
/// `w:numId`.
fn label_numbering(package: &mut PartFs, main: &str, document: &Document) -> Vec<u32> {
    if document.label_uses.is_empty() {
        return Vec::new();
    }
    let Some((_, first_num)) = append_numbering(package, main, |first_abstract, first_num| {
        xml::label_numbering(
            &document.labels,
            &document.label_uses,
            first_abstract,
            first_num,
        )
    }) else {
        return Vec::new();
    };
    (first_num..).take(document.label_uses.len()).collect()
}

/// Appends `w:abstractNum`s and `w:num`s to `main`'s numbering part, making
/// one if needed. `build` gets the first free abstract and num ids and
/// returns the `(abstracts, nums)` markup; returns those first ids.
pub(crate) fn append_numbering(
    package: &mut PartFs,
    main: &str,
    build: impl FnOnce(u32, u32) -> (String, String),
) -> Option<(u32, u32)> {
    let part = numbering_part(package, main);
    let existing = package.part_string(&part)?;
    let next = |value: Option<i64>| u32::try_from(value.unwrap_or(0).max(0) + 1).unwrap_or(1);
    let first_abstract = next(max_attribute(&existing, "w:abstractNum", "w:abstractNumId"));
    let first_num = next(max_attribute(&existing, "w:num", "w:numId"));
    let (abstracts, nums) = build(first_abstract, first_num);
    // Every w:abstractNum precedes every w:num.
    let abstract_at = [
        "<w:num ",
        "<w:num>",
        "<w:numIdMacAtCleanup",
        "</w:numbering>",
    ]
    .iter()
    .filter_map(|tag| existing.find(tag))
    .min();
    let num_at = ["<w:numIdMacAtCleanup", "</w:numbering>"]
        .iter()
        .filter_map(|tag| existing.find(tag))
        .min();
    if let (Some(a), Some(n)) = (abstract_at, num_at) {
        let updated = format!(
            "{}{abstracts}{}{nums}{}",
            &existing[..a],
            &existing[a..n],
            &existing[n..]
        );
        package.set_part(&part, updated.into_bytes());
    }
    Some((first_abstract, first_num))
}

/// The footnotes part `main` relates to; created with Word's separator and
/// continuation separator notes, its relationship and its content type when
/// absent.
pub(crate) fn ensure_footnotes_part(package: &mut PartFs, main: &str) -> String {
    ensure_notes_part(package, main, "footnote", None)
}

/// The notes part of `kind` (`footnote`, `endnote`) `main` relates to,
/// made as [`ensure_footnotes_part`] makes one. `changes` are the
/// `w:pPrChange` start tags of its separators, the separator's first and
/// the continuation separator's second.
fn ensure_notes_part(
    package: &mut PartFs,
    main: &str,
    kind: &str,
    changes: Option<&[String]>,
) -> String {
    let separators = || {
        let changes = changes.unwrap_or_default();
        let separator = |id: i32, name: &str, change: Option<&String>| {
            let spacing = "<w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/>";
            let change = change.map_or_else(String::new, |change| {
                format!("<{change}><w:pPr>{spacing}</w:pPr></w:pPrChange>")
            });
            format!(
                "<w:{kind} w:type=\"{name}\" w:id=\"{id}\"><w:p><w:pPr>{spacing}{change}</w:pPr>\
                 <w:r><w:{name}/></w:r></w:p></w:{kind}>"
            )
        };
        format!(
            "{}{}",
            separator(-1, "separator", changes.first()),
            separator(0, "continuationSeparator", changes.get(1))
        )
    };
    if let Some(part) = related(package, main, &format!("/{kind}s")) {
        if let Some(xml) = package.part_string(&part)
            && !xml.contains(&format!("<w:{kind} "))
            && !xml.contains(&format!("<w:{kind}>"))
        {
            let close = format!("</w:{kind}s>");
            let filled = match xml.rfind(&close) {
                Some(at) => format!("{}{}{}", &xml[..at], separators(), &xml[at..]),
                // A self-closed root (`<w:footnotes …/>`).
                None => match xml.rfind("/>") {
                    Some(at) if xml[..at].contains(&format!("<w:{kind}s")) => {
                        format!("{}>{}{close}", &xml[..at], separators())
                    }
                    _ => xml,
                },
            };
            package.set_part(&part, filled.into_bytes());
        }
        if let Some(changes) = changes
            && let Some(xml) = package.part_string(&part)
        {
            package.set_part(&part, separator_formats(&xml, kind, changes).into_bytes());
        }
        return part;
    }
    {
        let part = sibling(main, &format!("{kind}s.xml"));
        package.add_document_relationship(
            main,
            &format!("{RELS}/{kind}s"),
            &relative_rel_target(main, &part),
        );
        package.add_content_type_override(
            &format!("/{part}"),
            &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}s+xml"),
        );
        package.set_part(
            &part,
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                 <w:{kind}s xmlns:w=\"{W_NS}\" xmlns:r=\"{R_NS}\" xmlns:wp=\"{WP_NS}\">{}</w:{kind}s>",
                separators(),
            )
            .into_bytes(),
        );
        part
    }
}

/// Reusing a reference keeps its separator layout, while its revision
/// metadata follows the current view (including removal of a fmt clause).
fn separator_formats(xml: &str, kind: &str, changes: &[String]) -> String {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let Some(root) = dom.root(doc) else {
        return xml.to_string();
    };
    for (at, ty) in ["separator", "continuationSeparator"]
        .into_iter()
        .enumerate()
    {
        let note = dom
            .elements(root, Some(&W::name(kind)))
            .into_iter()
            .find(|&note| dom.attribute(note, &W::name("type")) == Some(ty));
        let Some(note) = note else { continue };
        let Some(paragraph) = dom.element(note, &W::p()) else {
            continue;
        };
        let properties = dom.element(paragraph, &W::p_pr()).unwrap_or_else(|| {
            let properties = dom.new_element(W::p_pr());
            dom.add_first(paragraph, properties);
            properties
        });
        for old in dom.elements(properties, Some(&W::name("pPrChange"))) {
            dom.remove(old);
        }
        if let Some(change) = changes.get(at) {
            let doc = dom.parse_xdocument(&format!("<{change} xmlns:w=\"{W_NS}\"/>"));
            if let Some(change) = dom.root(doc) {
                let previous = dom.clone_subtree(properties);
                dom.add(change, previous);
                dom.add(properties, change);
            }
        }
    }
    serialize_element(&dom, root)
}

/// Writes the footnotes and the endnotes into their notes parts, making
/// each part a kind of note needs.
fn footnotes(
    package: &mut PartFs,
    main: &str,
    context: &mut Context<'_>,
    media: &mut Media,
    separator_changes: &HashMap<&str, Vec<String>>,
) {
    let document = context.document;
    context.note_ids = vec![0; document.notes.len()];
    for (kind, endnote) in [("footnote", false), ("endnote", true)] {
        let changes = separator_changes.get(kind).map_or(&[][..], Vec::as_slice);
        if !document.notes.iter().any(|note| note.endnote == endnote) {
            // A part the view names empty holds Word's separators only.
            if document.empty_notes.contains(&kind) || !changes.is_empty() {
                ensure_notes_part(package, main, kind, Some(changes));
            }
            continue;
        }
        let part = ensure_notes_part(package, main, kind, Some(changes));
        let Some(existing) = package.part_string(&part) else {
            continue;
        };
        let highest = max_attribute(&existing, &format!("w:{kind}"), "w:id")
            .unwrap_or(0)
            .max(0);
        let mut next = u32::try_from(highest).unwrap_or(0);
        for (index, note) in document.notes.iter().enumerate() {
            if note.endnote == endnote {
                next += 1;
                context.note_ids[index] = next;
            }
        }
        let mut notes = String::new();
        // Revision ids are unique within a part, not across them: a note's
        // mark keeps the id its reference's has in the body.
        let in_part: HashSet<u32> = [
            "w:ins",
            "w:del",
            "w:moveFrom",
            "w:moveTo",
            "w:pPrChange",
            "w:rPrChange",
        ]
        .iter()
        .flat_map(|element| attribute_values(&existing, element, "w:id"))
        .filter_map(|id| u32::try_from(id).ok())
        .collect();
        let outside = std::mem::replace(&mut context.used_revisions, in_part);
        {
            let mut relate = PartRelate {
                package,
                part: part.clone(),
                media,
                document,
            };
            context.in_endnote = endnote;
            for (index, note) in document.notes.iter().enumerate() {
                if note.endnote != endnote {
                    continue;
                }
                let id = context.note_ids[index];
                let body = xml::story(context, &note.items, &mut relate, Some(note.change));
                notes.push_str(&format!("<w:{kind} w:id=\"{id}\">{body}</w:{kind}>"));
            }
            context.in_endnote = false;
        }
        context.used_revisions = outside;
        let mut root = existing;
        let open = format!("<w:{kind}s");
        for (prefix, namespace) in [("r", R_NS), ("wp", WP_NS)] {
            if let Some(at) = root.find(&open)
                && let Some(end) = root[at..].find('>').map(|i| at + i)
                && !root[at..end].contains(&format!("xmlns:{prefix}="))
            {
                root.insert_str(end, &format!(" xmlns:{prefix}=\"{namespace}\""));
            }
        }
        if let Some(at) = root.rfind(&format!("</w:{kind}s>")) {
            root.insert_str(at, &notes);
            package.set_part(&part, root.into_bytes());
        }
    }
}

/// The largest picture id (`wp:docPr`) the kept parts use.
pub(crate) fn max_drawing_id(package: &PartFs) -> u32 {
    package
        .parts()
        .iter()
        .filter(|name| name.ends_with(".xml"))
        .filter_map(|name| package.part_string(name))
        .filter_map(|xml| max_attribute(&xml, "wp:docPr", "id"))
        .max()
        .and_then(|id| u32::try_from(id.max(0)).ok())
        .unwrap_or(0)
}

/// `dc:title`, and the owner (`dc:creator`, or `cp:lastModifiedBy` alone)
/// from the front matter. A view whose document had no core properties
/// (`document_owner: none # no docProps/core.xml`) and no title writes
/// none; a reference without them gets them when the view names someone.
fn core_properties(package: &mut PartFs, document: &Document) {
    use super::view_notes::Owner;
    let part = package
        .parts()
        .into_iter()
        .find(|name| name.ends_with("core.xml") && name.starts_with("docProps/"));
    let owner = document.owner.as_ref();
    if owner == Some(&Owner::NoPart) && document.title.is_none() && document.author.is_none() {
        if let Some(part) = part {
            package.remove_part(&part);
            package.remove_content_type_override(&part);
            package.remove_package_relationships_by_type(CORE_PROPERTIES);
        }
        return;
    }
    // The owner line says whom the file names, and so whom the template's
    // names give way to; a top-level `author:` names the creator only when
    // there is no owner line.
    let (creator, editor, named) = match owner {
        Some(Owner::Creator(name)) => (Some(name), None, true),
        Some(Owner::LastModifiedBy(name)) => (None, Some(name), true),
        Some(Owner::Unnamed | Owner::NoPart) => (None, None, true),
        None => (document.author.as_ref(), None, false),
    };
    let edits = [
        CoreEdit {
            namespace: DC_NS,
            local: "title",
            value: document.title.as_deref(),
            clear: false,
        },
        CoreEdit {
            namespace: DC_NS,
            local: "creator",
            value: creator.map(String::as_str),
            clear: named,
        },
        CoreEdit {
            namespace: CP_NS,
            local: "lastModifiedBy",
            value: editor.map(String::as_str),
            clear: named,
        },
    ];
    if edits.iter().all(|edit| edit.value.is_none()) && (part.is_none() || !named) {
        return;
    }
    let (part, xml) = match part {
        Some(part) => {
            let Some(xml) = package.part_string(&part) else {
                return;
            };
            (part, xml)
        }
        None => {
            let part = "docProps/core.xml".to_string();
            package.add_content_type_override(
                &part,
                "application/vnd.openxmlformats-package.core-properties+xml",
            );
            package.add_package_relationship(CORE_PROPERTIES, &part);
            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\r\n\
                 <cp:coreProperties xmlns:cp=\"{CP_NS}\" xmlns:dc=\"{DC_NS}\" \
                 xmlns:dcterms=\"http://purl.org/dc/terms/\" \
                 xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" \
                 xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"></cp:coreProperties>"
            );
            (part, xml)
        }
    };
    if let Some(xml) = edit_core(&xml, &edits) {
        package.set_part(&part, xml.into_bytes());
    }
}

/// The Dublin Core namespace of `dc:title` and `dc:creator`.
const DC_NS: &str = "http://purl.org/dc/elements/1.1/";
/// The core properties namespace of `cp:lastModifiedBy`.
const CP_NS: &str = "http://schemas.openxmlformats.org/package/2006/metadata/core-properties";

/// One core property the view sets (`value`), or clears (`clear`).
struct CoreEdit<'a> {
    namespace: &'static str,
    local: &'static str,
    value: Option<&'a str>,
    clear: bool,
}

/// `xml` with each edited property's first element given its value in
/// place, keeping its own tag, its later copies dropped, and a property
/// the part lacks written last in the root. Elements are matched by
/// namespace, not by spelling, and comments and CDATA pass through
/// untouched. `None` for XML the reader refuses: the part is then left as
/// it is.
fn edit_core(xml: &str, edits: &[CoreEdit<'_>]) -> Option<String> {
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::name::ResolveResult;
    /// `start` holding `value`.
    fn element(
        writer: &mut quick_xml::Writer<Vec<u8>>,
        start: BytesStart<'_>,
        value: &str,
    ) -> Option<()> {
        let end = BytesEnd::new(start.name().as_ref().to_owned());
        writer.write_event(Event::Start(start)).ok()?;
        writer
            .write_event(Event::Text(BytesText::new(value)))
            .ok()?;
        writer.write_event(Event::End(end)).ok()
    }
    /// The edits the root lacks, written last in it.
    fn missing(
        writer: &mut quick_xml::Writer<Vec<u8>>,
        edits: &[CoreEdit<'_>],
        written: &[bool],
        prefixes: &HashMap<String, String>,
    ) -> Option<()> {
        for (edit, _) in edits.iter().zip(written).filter(|(_, done)| !**done) {
            let Some(value) = edit.value else {
                continue;
            };
            let (name, declare) = match prefixes.get(edit.namespace) {
                Some(prefix) => (format!("{prefix}:{}", edit.local), None),
                None => {
                    let prefix = if edit.namespace == DC_NS { "dc" } else { "cp" };
                    (format!("{prefix}:{}", edit.local), Some(prefix))
                }
            };
            let mut start = BytesStart::new(name.as_str());
            if let Some(prefix) = declare {
                start.push_attribute((format!("xmlns:{prefix}").as_str(), edit.namespace));
            }
            element(writer, start, value)?;
        }
        Some(())
    }
    let mut reader = quick_xml::NsReader::from_str(xml);
    let mut writer = quick_xml::Writer::new(Vec::new());
    let mut depth = 0usize;
    // The prefix the root binds to each namespace.
    let mut prefixes: HashMap<String, String> = HashMap::new();
    // The edits whose element is written.
    let mut written = vec![false; edits.len()];
    let edited = |resolved: &ResolveResult<'_>, local: &str| {
        let ResolveResult::Bound(namespace) = resolved else {
            return None;
        };
        edits.iter().position(|edit| {
            (edit.value.is_some() || edit.clear)
                && namespace.as_ref() == edit.namespace
                && local == edit.local
        })
    };
    loop {
        let (resolved, event) = reader.read_resolved_event().ok()?;
        match event {
            Event::Start(start) => {
                if depth == 0 {
                    for attribute in start.attributes().flatten() {
                        if let Some(prefix) = attribute.key.as_ref().strip_prefix("xmlns:") {
                            prefixes.insert(attribute.value.to_string(), prefix.to_string());
                        }
                    }
                }
                if depth == 1
                    && let Some(at) = edited(&resolved, start.local_name().as_ref())
                {
                    reader.read_to_end(start.name()).ok()?;
                    if let Some(value) = edits[at].value.filter(|_| !written[at]) {
                        element(&mut writer, start.into_owned(), value)?;
                    }
                    written[at] = true;
                    continue;
                }
                depth += 1;
                writer.write_event(Event::Start(start)).ok()?;
            }
            Event::Empty(empty) if depth == 0 => {
                // `<cp:coreProperties/>`: a root with no properties yet
                // opens to take the edits.
                for attribute in empty.attributes().flatten() {
                    if let Some(prefix) = attribute.key.as_ref().strip_prefix("xmlns:") {
                        prefixes.insert(attribute.value.to_string(), prefix.to_string());
                    }
                }
                let end = BytesEnd::new(empty.name().as_ref().to_owned());
                writer.write_event(Event::Start(empty)).ok()?;
                missing(&mut writer, edits, &written, &prefixes)?;
                writer.write_event(Event::End(end)).ok()?;
            }
            Event::Empty(empty) => {
                if depth == 1
                    && let Some(at) = edited(&resolved, empty.local_name().as_ref())
                {
                    if let Some(value) = edits[at].value.filter(|_| !written[at]) {
                        element(&mut writer, empty.into_owned(), value)?;
                    }
                    written[at] = true;
                    continue;
                }
                writer.write_event(Event::Empty(empty)).ok()?;
            }
            Event::End(end) => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    missing(&mut writer, edits, &written, &prefixes)?;
                }
                writer.write_event(Event::End(end)).ok()?;
            }
            // A root the part never closes is XML cut short: no edit.
            Event::Eof if depth > 0 => return None,
            Event::Eof => break,
            other => {
                writer.write_event(other).ok()?;
            }
        }
    }
    String::from_utf8(writer.into_inner()).ok()
}

/// The package relationship to the core properties part.
const CORE_PROPERTIES: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties";

/// Picture parts added so far, by picture index.
#[derive(Default)]
struct Media {
    parts: Vec<Option<String>>,
    used: HashSet<String>,
}

/// Relationships from one story part.
struct PartRelate<'p, 'm, 'd> {
    package: &'p mut PartFs,
    part: String,
    media: &'m mut Media,
    document: &'d Document,
}

impl Relate for PartRelate<'_, '_, '_> {
    fn hyperlink(&mut self, url: &str) -> String {
        self.package.add_document_relationship_external(
            &self.part,
            &format!("{RELS}/hyperlink"),
            url,
        )
    }

    fn picture(&mut self, index: usize) -> String {
        if self.media.parts.len() <= index {
            self.media.parts.resize(index + 1, None);
        }
        let target = match &self.media.parts[index] {
            Some(part) => part.clone(),
            None => {
                let picture = &self.document.pictures[index];
                let existing: HashSet<String> = self.package.parts().into_iter().collect();
                let mut number = index + 1;
                let part = loop {
                    let candidate = sibling(
                        &self.part,
                        &format!("media/image{number}.{}", picture.extension),
                    );
                    if !existing.contains(&candidate) && !self.media.used.contains(&candidate) {
                        break candidate;
                    }
                    number += 1;
                };
                self.package.set_part(&part, picture.bytes.clone());
                if self.package.content_type_for(&format!("/{part}")).is_none() {
                    self.package
                        .add_content_type_default(picture.extension, picture.content_type);
                }
                self.media.used.insert(part.clone());
                self.media.parts[index] = Some(part.clone());
                part
            }
        };
        // A picture used twice is one part and one relationship to it.
        let rel_target = relative_rel_target(&self.part, &target);
        if let Some(rels) = self.package.read_rels_for(&self.part)
            && let Some(rel) = rels
                .items
                .iter()
                .find(|r| r.rel_type == format!("{RELS}/image") && r.target == rel_target)
        {
            return rel.id.clone();
        }
        self.package
            .add_document_relationship(&self.part, &format!("{RELS}/image"), &rel_target)
    }

    fn drawing(&mut self, index: usize, next_doc_pr: &mut u32) -> Option<String> {
        let drawing = self.document.drawings.get(index)?;
        let existing: HashSet<String> = self.package.parts().into_iter().collect();
        super::from_docx::drawing::materialize(
            drawing,
            self.package,
            &self.part,
            &existing,
            &mut self.media.used,
            next_doc_pr,
        )
    }
}

/// The package a document is written into when there is no reference.
fn default_template(page: PageSize) -> Result<Vec<u8>, std::io::Error> {
    let section = default_section(page);
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
        <Override PartName=\"/word/settings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml\"/>\
        <Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>\
        <Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/>\
        </Types>";
    let package_rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"{RELS}/officeDocument\" Target=\"word/document.xml\"/>\
         <Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/>\
         <Relationship Id=\"rId3\" Type=\"{RELS}/extended-properties\" Target=\"docProps/app.xml\"/>\
         </Relationships>"
    );
    let document_rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"{RELS}/styles\" Target=\"styles.xml\"/>\
         <Relationship Id=\"rId2\" Type=\"{RELS}/settings\" Target=\"settings.xml\"/>\
         </Relationships>"
    );
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <w:document xmlns:w=\"{W_NS}\" xmlns:r=\"{R_NS}\" xmlns:wp=\"{WP_NS}\"><w:body>{section}</w:body></w:document>"
    );
    let styles = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <w:styles xmlns:w=\"{W_NS}\"><w:docDefaults><w:rPrDefault><w:rPr>\
         <w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\" w:eastAsia=\"Calibri\" w:cs=\"Calibri\"/>\
         <w:sz w:val=\"22\"/><w:szCs w:val=\"22\"/>\
         <w:lang w:val=\"en-US\" w:eastAsia=\"en-US\" w:bidi=\"ar-SA\"/></w:rPr></w:rPrDefault>\
         <w:pPrDefault><w:pPr><w:spacing w:after=\"160\" w:line=\"259\" w:lineRule=\"auto\"/></w:pPr>\
         </w:pPrDefault></w:docDefaults></w:styles>"
    );
    let settings = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <w:settings xmlns:w=\"{W_NS}\"><w:defaultTabStop w:val=\"720\"/>\
         <w:characterSpacingControl w:val=\"doNotCompress\"/><w:compat>\
         <w:compatSetting w:name=\"compatibilityMode\" w:uri=\"http://schemas.microsoft.com/office/word\" w:val=\"15\"/>\
         </w:compat></w:settings>"
    );
    let core = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
        xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" \
        xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\
        </cp:coreProperties>";
    let app = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
        <Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\">\
        <Application>jubarte</Application></Properties>";
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, body) in [
        ("[Content_Types].xml", content_types.to_string()),
        ("_rels/.rels", package_rels),
        ("word/document.xml", document),
        ("word/_rels/document.xml.rels", document_rels),
        ("word/styles.xml", styles),
        ("word/settings.xml", settings),
        ("docProps/core.xml", core.to_string()),
        ("docProps/app.xml", app.to_string()),
    ] {
        zip.start_file(name, options)?;
        zip.write_all(body.as_bytes())?;
    }
    Ok(zip.finish()?.into_inner())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn a_theme_colour_is_found_under_any_drawingml_prefix() {
        for (root, a) in [("xmlns:a", "a:"), ("xmlns:d", "d:"), ("xmlns", "")] {
            let theme = format!(
                "<{a}theme {root}=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
                 <{a}accent1><{a}srgbClr val=\"112233\"/></{a}accent1></{a}theme>"
            );
            assert_eq!(drawing_prefix(&theme), a, "{root}");
            assert_eq!(
                theme_colour(&theme, "accent1", None, None).as_deref(),
                Some("112233"),
                "{root}"
            );
        }
        // No DrawingML binding: Word's own prefix.
        assert_eq!(drawing_prefix("<theme/>"), "a:");
    }

    #[test]
    fn the_final_section_is_found_only_as_the_bodys_last_child() {
        let body = |inner: &str| {
            format!("<w:document xmlns:w=\"{W_NS}\"><w:body>{inner}</w:body></w:document>")
        };
        let section = final_section(&body(
            "<w:p/><w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr>",
        ))
        .unwrap();
        assert!(
            section.starts_with("<w:sectPr") && section.contains("w:w=\"11906\""),
            "{section}"
        );
        assert_eq!(
            final_section(&body("<w:p><w:pPr><w:sectPr/></w:pPr></w:p><w:p/>")),
            None
        );
        // A tracked section change nests a w:sectPr: the outer one is the
        // section, and its change stays in the reference.
        let section = final_section(&body(
            "<w:p/><w:sectPr><w:pgSz w:w=\"12240\"/><w:sectPrChange w:id=\"1\" w:author=\"a\">\
             <w:sectPr><w:pgSz w:w=\"11906\"/></w:sectPr></w:sectPrChange></w:sectPr>",
        ))
        .unwrap();
        assert!(section.contains("w:w=\"12240\""), "{section}");
        assert!(
            !section.contains("sectPrChange") && !section.contains("11906"),
            "{section}"
        );
    }

    #[test]
    fn a_main_part_without_a_usable_root_or_section_falls_back_to_the_page() {
        // No document element, an unterminated one, and an empty one: the
        // default root and the page's section.
        for source in ["<x/>", "<w:document", "<w:document/>"] {
            for page in [PageSize::Letter, PageSize::A4] {
                let (root, section) = frame(source, page);
                assert!(root.starts_with("<w:document xmlns:w="), "{source}: {root}");
                assert_eq!(section, default_section(page), "{source} {page:?}");
            }
        }
        // A document element whose body has no final section.
        let (_, section) = frame(
            "<w:document><w:body><w:p/></w:body></w:document>",
            PageSize::A4,
        );
        assert_eq!(section, A4_SECTION);
    }

    #[test]
    fn the_text_width_is_the_page_less_its_margins() {
        assert_eq!(text_width(LETTER_SECTION), 9360);
        assert_eq!(text_width(A4_SECTION), 9026);
        assert_eq!(
            text_width(
                "<w:sectPr><w:pgSz w:w=\"11906\"/><w:pgMar w:left=\"1134\" w:right=\"1134\"/></w:sectPr>"
            ),
            9638
        );
    }

    #[test]
    fn separators_survive_and_notes_do_not() {
        let xml = "<w:footnotes><w:footnote w:type=\"separator\" w:id=\"-1\"><w:p/></w:footnote>\
                   <w:footnote w:id=\"1\"><w:p><w:r><w:t>old</w:t></w:r></w:p></w:footnote>\
                   <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"/></w:footnotes>";
        assert_eq!(
            separators_only(xml),
            "<w:footnotes><w:footnote w:type=\"separator\" w:id=\"-1\"><w:p/></w:footnote>\
             <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"/></w:footnotes>"
        );
    }

    #[test]
    fn missing_styles_are_added_and_present_ones_kept() {
        let styles = "<w:styles><w:style w:type=\"paragraph\" w:styleId=\"Heading1\"><w:name w:val=\"x\"/></w:style></w:styles>";
        let wanted: BTreeSet<&str> = ["Heading1", "Quote"].into_iter().collect();
        let out = with_styles(styles, &wanted);
        assert_eq!(out.matches("w:styleId=\"Heading1\"").count(), 1);
        assert!(out.contains("w:styleId=\"Quote\""));
    }

    #[test]
    fn max_attribute_reads_only_the_named_element() {
        let xml = "<w:abstractNum w:abstractNumId=\"3\"/><w:num w:numId=\"7\"><w:abstractNumId w:val=\"3\"/></w:num>";
        assert_eq!(
            max_attribute(xml, "w:abstractNum", "w:abstractNumId"),
            Some(3)
        );
        assert_eq!(max_attribute(xml, "w:num", "w:numId"), Some(7));
        assert_eq!(max_attribute(xml, "w:lvl", "w:ilvl"), None);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod literal_package_metadata_contract_tests {
    use super::*;

    #[test]
    fn front_matter_preserves_exact_core_property_owners_for_every_authored_shape() {
        let root = r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">"#;
        for (body, changed) in [
            (
                "<dc:title>old title</dc:title><dc:creator>old author</dc:creator><dc:description>untouched</dc:description>",
                "<dc:title>Ω &amp; &lt;Title&gt;</dc:title><dc:creator>Ada &amp; Bo</dc:creator><dc:description>untouched</dc:description>",
            ),
            (
                "<dc:title/><dc:creator/><dc:description>untouched</dc:description>",
                "<dc:title>Ω &amp; &lt;Title&gt;</dc:title><dc:creator>Ada &amp; Bo</dc:creator><dc:description>untouched</dc:description>",
            ),
            (
                "<dc:description>untouched</dc:description>",
                "<dc:description>untouched</dc:description><dc:title>Ω &amp; &lt;Title&gt;</dc:title><dc:creator>Ada &amp; Bo</dc:creator>",
            ),
        ] {
            let mut package =
                PartFs::open(include_bytes!("../../tests/fixtures/redline/original.docx")).unwrap();
            package.set_part(
                "docProps/core.xml",
                format!("{root}{body}</cp:coreProperties>").into_bytes(),
            );
            let main = package.part_bytes("word/document.xml").unwrap().to_vec();
            let document = Document {
                title: Some("Ω & <Title>".into()),
                author: Some("Ada & Bo".into()),
                ..Document::default()
            };
            core_properties(&mut package, &document);
            assert_eq!(
                package.part_string("docProps/core.xml").unwrap(),
                format!("{root}{changed}</cp:coreProperties>")
            );
            assert_eq!(package.part_bytes("word/document.xml").unwrap(), main);
            let again = package.part_string("docProps/core.xml").unwrap();
            core_properties(&mut package, &document);
            assert_eq!(package.part_string("docProps/core.xml").unwrap(), again);
        }
    }

    #[test]
    fn a_self_closing_core_root_opens_to_take_the_owner() {
        let source = r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"/>"#;
        let edits = [CoreEdit {
            namespace: DC_NS,
            local: "creator",
            value: Some("Ada"),
            clear: false,
        }];
        assert_eq!(
            edit_core(source, &edits).unwrap(),
            format!(
                "{}><dc:creator>Ada</dc:creator></cp:coreProperties>",
                source.trim_end_matches("/>")
            )
        );
    }

    #[test]
    fn absent_front_matter_fields_preserve_every_existing_core_property() {
        let source = r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>owned title</dc:title><dc:creator>owned author</dc:creator></cp:coreProperties>"#;
        for (title, author, expected) in [
            (None, None, source.to_string()),
            (
                Some("new title"),
                None,
                source.replace("owned title", "new title"),
            ),
            (
                None,
                Some("new author"),
                source.replace("owned author", "new author"),
            ),
        ] {
            let mut package =
                PartFs::open(include_bytes!("../../tests/fixtures/redline/original.docx")).unwrap();
            package.set_part("docProps/core.xml", source.as_bytes().to_vec());
            core_properties(
                &mut package,
                &Document {
                    title: title.map(str::to_string),
                    author: author.map(str::to_string),
                    ..Document::default()
                },
            );
            assert_eq!(package.part_string("docProps/core.xml").unwrap(), expected);
        }
    }

    #[test]
    fn separator_notes_keep_complete_authored_markup_without_ordinary_notes() {
        for kind in ["footnote", "endnote"] {
            for separator in ["separator", "continuationSeparator", "continuationNotice"] {
                for short in [false, true] {
                    let retained = if short {
                        format!("<w:{kind} w:type=\"{separator}\" w:id=\"-1\"/>")
                    } else {
                        format!(
                            "<w:{kind} w:type=\"{separator}\" w:id=\"-1\"><w:p><w:r><w:t>owned separator</w:t></w:r></w:p></w:{kind}>"
                        )
                    };
                    let root = format!("<w:{kind}s xmlns:w=\"{W_NS}\">");
                    let end = format!("</w:{kind}s>");
                    let source = format!(
                        "{root}{retained}<w:{kind} w:id=\"1\"><w:p><w:r><w:t>ordinary source note</w:t></w:r></w:p></w:{kind}>{end}"
                    );
                    assert_eq!(separators_only(&source), format!("{root}{retained}{end}"));
                }
            }
        }
        assert_eq!(
            max_attribute(
                r#"<w:num w:other="1"/><w:num w:numId="invalid"/><w:num w:numId="3"/><w:num w:numId="-5"/>"#,
                "w:num",
                "w:numId"
            ),
            Some(3)
        );
        assert_eq!(
            max_attribute(r#"<w:num w:other="1"/>"#, "w:num", "w:numId"),
            None
        );
    }
}
