// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The `.docx` package around the written items: built-in parts, or a
//! reference document's styles, numbering, page setup, headers and footers.

use std::collections::{BTreeSet, HashSet};
use std::io::{Cursor, Write as _};

use super::xml::{self, Context, Document, Relate, escape};
use super::{DocxOptions, MarkdownError, PageSize};
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
    if let Some(styles) = package.part_string(&styles_part) {
        package.set_part(&styles_part, with_styles(&styles, &wanted).into_bytes());
    }

    let num_ids = numbering(&mut package, &main, document);
    let text_width = text_width(&section);
    let next_drawing = max_drawing_id(&package) + 1;
    let mut context = Context {
        document,
        author: escape(&options.author).into_owned(),
        date: escape(&options.date).into_owned(),
        next_revision: 1,
        num_ids,
        note_base: 0,
        next_drawing,
        text_width,
    };
    let mut media = Media::default();

    if !document.notes.is_empty() {
        footnotes(&mut package, &main, &mut context, &mut media);
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
fn sibling(main: &str, name: &str) -> String {
    match main.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{name}"),
        None => name.to_string(),
    }
}

/// The part `main` relates to with the relationship type ending in `kind`.
fn related(package: &PartFs, main: &str, kind: &str) -> Option<String> {
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

/// `styles` with a definition added for each wanted style it lacks.
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

/// The largest value of attribute `name` on elements `element` in `xml`.
fn max_attribute(xml: &str, element: &str, name: &str) -> Option<i64> {
    let mut best = None;
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
            best = Some(best.map_or(value, |b: i64| b.max(value)));
        }
        rest = &rest[tag_end..];
    }
    best
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
    related(package, main, "/footnotes").unwrap_or_else(|| {
        let part = sibling(main, "footnotes.xml");
        package.add_document_relationship(
            main,
            &format!("{RELS}/footnotes"),
            &relative_rel_target(main, &part),
        );
        package.add_content_type_override(
            &format!("/{part}"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        );
        package.set_part(
            &part,
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                 <w:footnotes xmlns:w=\"{W_NS}\" xmlns:r=\"{R_NS}\" xmlns:wp=\"{WP_NS}\">\
                 <w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:pPr>\
                 <w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
                 <w:r><w:separator/></w:r></w:p></w:footnote>\
                 <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:pPr>\
                 <w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
                 <w:r><w:continuationSeparator/></w:r></w:p></w:footnote></w:footnotes>"
            )
            .into_bytes(),
        );
        part
    })
}

/// Writes the footnotes into the notes part, making one if needed.
fn footnotes(package: &mut PartFs, main: &str, context: &mut Context<'_>, media: &mut Media) {
    let part = ensure_footnotes_part(package, main);
    let Some(existing) = package.part_string(&part) else {
        return;
    };
    let highest = max_attribute(&existing, "w:footnote", "w:id")
        .unwrap_or(0)
        .max(0);
    context.note_base = u32::try_from(highest).unwrap_or(0);
    let document = context.document;
    let mut notes = String::new();
    {
        let mut relate = PartRelate {
            package,
            part: part.clone(),
            media,
            document,
        };
        for (index, note) in document.notes.iter().enumerate() {
            let id = context.note_base + u32::try_from(index + 1).unwrap_or(0);
            let body = xml::story(context, &note.items, &mut relate, Some(note.change));
            notes.push_str(&format!("<w:footnote w:id=\"{id}\">{body}</w:footnote>"));
        }
    }
    let mut root = existing;
    for (prefix, namespace) in [("r", R_NS), ("wp", WP_NS)] {
        if let Some(at) = root.find("<w:footnotes")
            && let Some(end) = root[at..].find('>').map(|i| at + i)
            && !root[at..end].contains(&format!("xmlns:{prefix}="))
        {
            root.insert_str(end, &format!(" xmlns:{prefix}=\"{namespace}\""));
        }
    }
    if let Some(at) = root.rfind("</w:footnotes>") {
        root.insert_str(at, &notes);
        package.set_part(&part, root.into_bytes());
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

/// `dc:title` and `dc:creator` from the front matter, when the package has
/// core properties.
fn core_properties(package: &mut PartFs, document: &Document) {
    if document.title.is_none() && document.author.is_none() {
        return;
    }
    let Some(part) = package
        .parts()
        .into_iter()
        .find(|name| name.ends_with("core.xml") && name.starts_with("docProps/"))
    else {
        return;
    };
    let Some(mut xml) = package.part_string(&part) else {
        return;
    };
    for (element, value) in [
        ("dc:title", &document.title),
        ("dc:creator", &document.author),
    ] {
        let Some(value) = value else {
            continue;
        };
        let replacement = format!("<{element}>{}</{element}>", escape(value));
        let open = format!("<{element}>");
        let close = format!("</{element}>");
        let empty = format!("<{element}/>");
        if let (Some(start), Some(end)) = (xml.find(&open), xml.find(&close)) {
            xml.replace_range(start..end + close.len(), &replacement);
        } else if let Some(start) = xml.find(&empty) {
            xml.replace_range(start..start + empty.len(), &replacement);
        } else if let Some(at) = xml.rfind("</cp:coreProperties>") {
            xml.insert_str(at, &replacement);
        }
    }
    package.set_part(&part, xml.into_bytes());
}

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
        self.package.add_document_relationship(
            &self.part,
            &format!("{RELS}/image"),
            &relative_rel_target(&self.part, &target),
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
mod tests {
    use super::*;

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
