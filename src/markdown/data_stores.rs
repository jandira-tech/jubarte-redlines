// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The custom XML data stores of the content controls the writer writes
//! bound to one (#1427).
//!
//! Word fills a bound control (`w:sdtPr/w:dataBinding`) from its store when
//! it opens the file. A drawing payload (a cover page's text box) keeps its
//! controls' bindings, but the store is a package part the drawing never
//! reaches: without it Word shows the control empty. So for every store a
//! written binding names and the main part does not reach, the writer
//! builds one from the bindings themselves: each simple path
//! (`/ns0:CoverPageProperties[1]/ns0:Abstract[1]`) a node holding what its
//! control shows. Only a plain text, date or list control's value is
//! its text; any other binding (rich text, a picture, a check box, a path
//! that is not simple, content plain text cannot hold) loses its
//! `w:dataBinding` instead, so Word keeps what the control shows. (A box
//! whose words the view edits is written with plain runs, no control, so
//! the edit shows as written.)

use std::fmt::Write as _;
use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::from_docx::{Element, element_from, parse_xml};
use super::xml::escape;
use crate::opc::{PartFs, relative_rel_target};

const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const DS_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/customXml";
const PROPS_CT: &str = "application/vnd.openxmlformats-officedocument.customXmlProperties+xml";

/// Word's own stores, which `docProps` holds: the core and the extended
/// document properties.
const BUILT_IN: [&str; 2] = [
    "{6C3C8BC8-F283-45AE-878A-BAB7291924A1}",
    "{6668398D-A668-4E3E-A5EB-62B293D839F1}",
];

/// One control's binding: its store, the path's steps (namespace URI and
/// local name; `None` when the path is not a simple one), and the value
/// its store holds (`None` when no value of the control's own is one).
struct Binding {
    store: String,
    xpath: String,
    steps: Option<Vec<(String, String)>>,
    value: Option<String>,
}

/// A node of a store being built.
struct Node {
    namespace: String,
    local: String,
    value: Option<String>,
    children: Vec<Node>,
}

/// Write the stores the written parts' bindings name and the main part
/// does not reach; drop the bindings no store can be built for.
pub(crate) fn write_missing(package: &mut PartFs, main: &str, warnings: &mut Vec<String>) {
    let held = stores(package, main);
    let mut parts: Vec<(String, Vec<Binding>)> = Vec::new();
    for part in package.parts() {
        if !part.ends_with(".xml") || part.starts_with("customXml/") {
            continue;
        }
        let Some(bytes) = package.part_bytes(&part) else {
            continue;
        };
        if !bytes.windows(11).any(|w| w == b"dataBinding") {
            continue;
        }
        let Some(root) = std::str::from_utf8(bytes)
            .ok()
            .and_then(|xml| parse_xml(xml.as_bytes()).ok())
        else {
            warnings.push(format!(
                "{part} could not be read for its bound controls; their stores are not written"
            ));
            continue;
        };
        let mut found = Vec::new();
        bindings(&root, &mut found);
        found.retain(|b| !built_in(&b.store) && !held.contains(&b.store.to_ascii_uppercase()));
        if !found.is_empty() {
            parts.push((part, found));
        }
    }
    // Each store's nodes, in the order the parts name them.
    let mut built: Vec<(String, Node)> = Vec::new();
    let mut unbuilt: Vec<(String, String, String)> = Vec::new();
    for (part, found) in &parts {
        for b in found {
            let placed = match (&b.steps, &b.value) {
                (Some(steps), Some(value)) => {
                    let at = match built
                        .iter()
                        .position(|(id, _)| id.eq_ignore_ascii_case(&b.store))
                    {
                        Some(at) => at,
                        None => {
                            let (namespace, local) = steps[0].clone();
                            built.push((
                                b.store.clone(),
                                Node {
                                    namespace,
                                    local,
                                    value: None,
                                    children: Vec::new(),
                                },
                            ));
                            built.len() - 1
                        }
                    };
                    match insert(&mut built[at].1, steps, value) {
                        Placed::Conflict(first) => {
                            warnings.push(format!(
                                "{part}: two controls bind {} with different text; the store holds the first, {first:?}",
                                b.xpath
                            ));
                            true
                        }
                        Placed::Yes => true,
                        Placed::Elsewhere => false,
                    }
                }
                _ => false,
            };
            if !placed {
                unbuilt.push((part.clone(), b.store.clone(), b.xpath.clone()));
            }
        }
    }
    for (store, root) in &built {
        write_store(package, main, store, root);
    }
    for (part, store, xpath) in unbuilt {
        let Some(xml) = package
            .part_bytes(&part)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
        else {
            continue;
        };
        let xml = unbound(xml, &store, &xpath);
        package.set_part(&part, xml.into_bytes());
    }
}

/// The stores the main part reaches, by item ID (upper case): each
/// `customXml` relationship's item, through its `customXmlProps`
/// relationship to the properties naming it. Word loads those.
pub(crate) fn stores(package: &PartFs, main: &str) -> Vec<String> {
    let related = |source: &str, kind: &str| -> Vec<String> {
        package
            .read_rels_for(source)
            .into_iter()
            .flat_map(|rels| &rels.items)
            // Transitional's type or Strict's: the same last segment.
            .filter(|r| {
                r.rel_type.rsplit('/').next() == Some(kind)
                    && r.target_mode.as_deref() != Some("External")
            })
            .map(|r| {
                package
                    .resolve_rel_target(source, &r.target)
                    .trim_start_matches('/')
                    .to_string()
            })
            .collect()
    };
    related(main, "customXml")
        .into_iter()
        .filter(|item| package.part_bytes(item).is_some())
        .flat_map(|item| related(&item, "customXmlProps"))
        .filter_map(|props| parse_xml(package.part_bytes(&props)?).ok())
        .filter(|root| root.is("datastoreItem"))
        .filter_map(|root| root.attr("itemID").map(str::to_ascii_uppercase))
        .collect()
}

/// Whether a binding names one of Word's own stores, which `docProps`
/// holds.
pub(crate) fn built_in(store: &str) -> bool {
    BUILT_IN.iter().any(|id| id.eq_ignore_ascii_case(store))
}

/// Every bound control under `e`, nested ones too.
fn bindings(e: &Element, out: &mut Vec<Binding>) {
    for c in e.elements() {
        if c.is("sdt")
            && let Some(binding) = c.path(&["sdtPr", "dataBinding"])
            && let Some(store) = binding.attr("storeItemID")
        {
            let xpath = binding.attr("xpath").unwrap_or_default().to_string();
            let prefixes = prefix_mappings(binding.attr("prefixMappings").unwrap_or_default());
            out.push(Binding {
                store: store.to_string(),
                steps: steps(&xpath, &prefixes),
                xpath,
                value: shown(c),
            });
        }
        bindings(c, out);
    }
}

/// `xmlns:ns0='uri' xmlns:ns1="uri"` as (prefix, URI) pairs.
fn prefix_mappings(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("xmlns:") {
        rest = &rest[at + "xmlns:".len()..];
        let Some((prefix, after)) = rest.split_once('=') else {
            break;
        };
        let Some(quote) = after.chars().next().filter(|c| *c == '\'' || *c == '"') else {
            break;
        };
        let Some((uri, tail)) = after[1..].split_once(quote) else {
            break;
        };
        out.push((prefix.trim().to_string(), uri.to_string()));
        rest = tail;
    }
    out
}

/// A simple path's steps: `/p:a[1]/p:b[1]`, each a mapped prefix's name,
/// the first of its name (`[1]` or no index). `None` for anything else,
/// an unprefixed name too (XPath 1.0 puts it in no namespace).
fn steps(xpath: &str, prefixes: &[(String, String)]) -> Option<Vec<(String, String)>> {
    let rest = xpath.strip_prefix('/')?;
    let steps = rest
        .split('/')
        .map(|step| {
            let name = step.strip_suffix("[1]").unwrap_or(step);
            let (prefix, local) = name.split_once(':')?;
            // An NCName: a letter or `_`, then letters, digits, `_-.`.
            let simple = |s: &str| {
                s.chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                    && s.chars()
                        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
            };
            if !simple(prefix) || !simple(local) {
                return None;
            }
            let namespace = prefixes.iter().find(|(p, _)| p == prefix)?.1.clone();
            Some((namespace, local.to_string()))
        })
        .collect::<Option<Vec<_>>>()?;
    (!steps.is_empty()).then_some(steps)
}

/// What a control shows, as its store holds it: nothing
/// while it shows its placeholder; a date control's date as its
/// `w:storeMappedDataAs` asks; a list's chosen item's value; a plain text
/// control's text. `None` for the rest, whose store value is not their
/// text (rich text, a picture, a check box), and for content plain text
/// cannot hold (a field, a symbol, a second paragraph): those lose their
/// binding instead.
fn shown(sdt: &Element) -> Option<String> {
    let properties = sdt.child("sdtPr")?;
    let kind = ["text", "date", "dropDownList", "comboBox"]
        .into_iter()
        .find(|kind| properties.child(kind).is_some())?;
    if properties.child("showingPlcHdr").is_some() {
        return Some(String::new());
    }
    let mut text = String::new();
    let mut paragraphs = 0;
    if let Some(content) = sdt.child("sdtContent") {
        plain(content, &mut text, &mut paragraphs)?;
    }
    let control = properties.child(kind)?;
    match kind {
        "date" => {
            let full = control.attr("fullDate");
            match control
                .child("storeMappedDataAs")
                .and_then(|s| s.attr("val"))
                .unwrap_or("dateTime")
            {
                "text" => Some(text),
                "date" => full.and_then(|d| d.split('T').next()).map(str::to_string),
                _ => full.map(str::to_string),
            }
        }
        "dropDownList" | "comboBox" => {
            let chosen = control.children_named("listItem").find(|item| {
                item.attr("displayText").or_else(|| item.attr("value")) == Some(text.as_str())
            });
            match chosen {
                Some(item) => item.attr("value").map(str::to_string),
                None if kind == "comboBox" => Some(text),
                None => None,
            }
        }
        _ => Some(text),
    }
}

/// The text a plain text control's runs show: `w:t`, a tab, a break.
/// `None` at anything plain text does not hold.
fn plain(e: &Element, out: &mut String, paragraphs: &mut usize) -> Option<()> {
    for c in e.elements() {
        match c.local() {
            "t" => out.push_str(&c.text()),
            "tab" => out.push('\t'),
            "br" | "cr" => out.push('\n'),
            "p" => {
                *paragraphs += 1;
                if *paragraphs > 1 {
                    return None;
                }
                plain(c, out, paragraphs)?;
            }
            "r" | "ins" | "moveTo" | "hyperlink" | "smartTag" | "customXml" | "sdt"
            | "sdtContent" | "bdo" | "dir" => plain(c, out, paragraphs)?,
            // Properties, and text the control no longer shows.
            "pPr" | "rPr" | "sdtPr" | "sdtEndPr" | "del" | "moveFrom" => {}
            // A marker (a bookmark, a comment's range) shows nothing.
            _ if c.children.is_empty() && !SHOWS.contains(&c.local()) => {}
            _ => return None,
        }
    }
    Some(())
}

/// Empty run content that shows something plain text cannot hold.
const SHOWS: [&str; 16] = [
    "fldChar",
    "sym",
    "ptab",
    "noBreakHyphen",
    "softHyphen",
    "footnoteReference",
    "endnoteReference",
    "pgNum",
    "dayShort",
    "dayLong",
    "monthShort",
    "monthLong",
    "yearShort",
    "yearLong",
    "annotationRef",
    "separator",
];

/// Where [`insert`] put a value.
enum Placed {
    Yes,
    /// The node already held another value, kept.
    Conflict(String),
    /// The path starts at another root.
    Elsewhere,
}

/// Put `value` at `steps` under `root` (whose own step is `steps[0]`).
/// The first value a node gets is kept: a drawing's fallback repeats its
/// controls, with the same text.
fn insert(root: &mut Node, steps: &[(String, String)], value: &str) -> Placed {
    let (namespace, local) = &steps[0];
    if *namespace != root.namespace || *local != root.local {
        return Placed::Elsewhere;
    }
    let mut node = root;
    for (namespace, local) in &steps[1..] {
        let at = match node
            .children
            .iter()
            .position(|c| c.namespace == *namespace && c.local == *local)
        {
            Some(at) => at,
            None => {
                node.children.push(Node {
                    namespace: namespace.clone(),
                    local: local.clone(),
                    value: None,
                    children: Vec::new(),
                });
                node.children.len() - 1
            }
        };
        node = &mut node.children[at];
    }
    match &node.value {
        None => {
            node.value = Some(value.to_string());
            Placed::Yes
        }
        Some(first) if first == value => Placed::Yes,
        Some(first) => Placed::Conflict(first.clone()),
    }
}

/// The namespaces a store's nodes use, in the order they use them.
fn namespaces(root: &Node) -> Vec<&str> {
    fn walk<'a>(node: &'a Node, out: &mut Vec<&'a str>) {
        if !out.contains(&node.namespace.as_str()) {
            out.push(&node.namespace);
        }
        for c in &node.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

/// The store as XML: one namespace as the root's default, as Word writes
/// a cover page's; several declared once on the root, `ns0`, `ns1`, … in
/// the order the nodes use them.
fn store_xml(root: &Node) -> String {
    fn element(node: &Node, uris: &[&str], declarations: &str, out: &mut String) {
        let name = match uris {
            [_] => node.local.clone(),
            _ => {
                let prefix = uris.iter().position(|u| *u == node.namespace).unwrap_or(0);
                format!("ns{prefix}:{}", node.local)
            }
        };
        let _ = write!(out, "<{name}{declarations}");
        // A node bound whole and in parts (a repeating section's item)
        // holds its parts.
        match (&node.value, node.children.is_empty()) {
            (_, false) => {
                out.push('>');
                for c in &node.children {
                    element(c, uris, "", out);
                }
                let _ = write!(out, "</{name}>");
            }
            (Some(value), true) if !value.is_empty() => {
                let _ = write!(out, ">{}</{name}>", escape(value));
            }
            (_, true) => out.push_str("/>"),
        }
    }
    let uris = namespaces(root);
    let declarations: String = match uris.as_slice() {
        [uri] => format!(" xmlns=\"{}\"", escape(uri)),
        _ => uris
            .iter()
            .enumerate()
            .map(|(i, uri)| format!(" xmlns:ns{i}=\"{}\"", escape(uri)))
            .collect(),
    };
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>");
    element(root, &uris, &declarations, &mut out);
    out
}

/// Write `customXml/itemN.xml` with its `itemPropsN.xml`, related from the
/// main document part.
fn write_store(package: &mut PartFs, main: &str, store: &str, root: &Node) {
    let parts = package.parts();
    let n = (1..)
        .find(|n| {
            let taken = |name: String| parts.iter().any(|p| p.eq_ignore_ascii_case(&name));
            // Nor a stale relationship part of the item's.
            !taken(format!("customXml/item{n}.xml"))
                && !taken(format!("customXml/itemProps{n}.xml"))
                && package
                    .read_rels_for(&format!("customXml/item{n}.xml"))
                    .is_none()
        })
        .unwrap_or(1);
    let item = format!("customXml/item{n}.xml");
    let props = format!("customXml/itemProps{n}.xml");
    package.set_part(&item, store_xml(root).into_bytes());
    let schemas: String = namespaces(root)
        .iter()
        .map(|uri| format!("<ds:schemaRef ds:uri=\"{}\"/>", escape(uri)))
        .collect();
    package.set_part(
        &props,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\
             <ds:datastoreItem ds:itemID=\"{}\" xmlns:ds=\"{DS_NS}\">\
             <ds:schemaRefs>{schemas}</ds:schemaRefs></ds:datastoreItem>",
            escape(store)
        )
        .into_bytes(),
    );
    if package.content_type_for(&format!("/{item}")).is_none() {
        package.add_content_type_override(&format!("/{item}"), "application/xml");
    }
    package.add_content_type_override(&format!("/{props}"), PROPS_CT);
    package.add_document_relationship(
        &item,
        &format!("{RELS}/customXmlProps"),
        &format!("itemProps{n}.xml"),
    );
    package.add_document_relationship(
        main,
        &format!("{RELS}/customXml"),
        &relative_rel_target(main, &item),
    );
}

/// `xml` without the `dataBinding` elements that bind `store` at `xpath`,
/// read as the parser reads them (either quote, entities, an element
/// closed by its own end tag).
fn unbound(xml: &str, store: &str, xpath: &str) -> String {
    let binds = |start: &BytesStart<'_>| {
        element_from(start, &Arc::default()).is_ok_and(|e| {
            e.is("dataBinding")
                && e.attr("storeItemID")
                    .is_some_and(|s| s.eq_ignore_ascii_case(store))
                && e.attr("xpath") == Some(xpath)
        })
    };
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<(usize, String)> = None;
    loop {
        let before = reader.buffer_position() as usize;
        let Ok(event) = reader.read_event() else {
            break;
        };
        let after = reader.buffer_position() as usize;
        match (&event, &open) {
            (Event::Eof, _) => break,
            (Event::Empty(start), None) if binds(start) => cuts.push((before, after)),
            (Event::Start(start), None) if binds(start) => {
                open = Some((before, start.name().as_ref().to_string()));
            }
            (Event::End(end), Some((from, name))) if end.name().as_ref() == name => {
                cuts.push((*from, after));
                open = None;
            }
            _ => {}
        }
    }
    let mut out = String::with_capacity(xml.len());
    let mut at = 0;
    for (from, to) in cuts {
        out.push_str(&xml[at..from]);
        at = to;
    }
    out.push_str(&xml[at..]);
    out
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    const COVER: &str = "http://schemas.microsoft.com/office/2006/coverPageProps";

    #[test]
    fn a_simple_path_reads_as_its_steps() {
        let prefixes = prefix_mappings(&format!("xmlns:ns0='{COVER}' xmlns:ns1=\"urn:b\""));
        assert_eq!(
            prefixes,
            vec![
                ("ns0".to_string(), COVER.to_string()),
                ("ns1".to_string(), "urn:b".to_string())
            ]
        );
        assert_eq!(
            steps("/ns0:CoverPageProperties[1]/ns1:Abstract", &prefixes),
            Some(vec![
                (COVER.to_string(), "CoverPageProperties".to_string()),
                ("urn:b".to_string(), "Abstract".to_string())
            ])
        );
        for odd in [
            "ns0:Root[1]",
            "/ns0:Root[2]",
            "/ns0:Root[1]/@ns0:attr",
            "/nsX:Root[1]",
            "/ns0:Root[1]//ns0:Any",
            "/Root",
            "",
        ] {
            assert_eq!(steps(odd, &prefixes), None, "{odd}");
        }
    }

    #[test]
    fn a_store_holds_each_path_once_under_one_root() {
        let cover = |local: &str| (COVER.to_string(), local.to_string());
        let mut root = Node {
            namespace: COVER.to_string(),
            local: "CoverPageProperties".to_string(),
            value: None,
            children: Vec::new(),
        };
        let abstract_ = [cover("CoverPageProperties"), cover("Abstract")];
        assert!(matches!(
            insert(&mut root, &abstract_, "A & b"),
            Placed::Yes
        ));
        // A fallback's copy agrees; another control's other text does not.
        assert!(matches!(
            insert(&mut root, &abstract_, "A & b"),
            Placed::Yes
        ));
        assert!(matches!(
            insert(&mut root, &abstract_, "later"),
            Placed::Conflict(first) if first == "A & b"
        ));
        assert!(matches!(
            insert(
                &mut root,
                &[cover("CoverPageProperties"), cover("PublishDate")],
                ""
            ),
            Placed::Yes
        ));
        assert!(matches!(
            insert(&mut root, &[cover("Other"), cover("Abstract")], "x"),
            Placed::Elsewhere
        ));
        // One namespace is the root's default, as Word writes it.
        assert_eq!(
            store_xml(&root),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <CoverPageProperties xmlns=\"{COVER}\"><Abstract>A &amp; b</Abstract>\
                 <PublishDate/></CoverPageProperties>"
            )
        );
        insert(
            &mut root,
            &[
                cover("CoverPageProperties"),
                ("urn:b".to_string(), "Extra".to_string()),
            ],
            "e",
        );
        assert_eq!(namespaces(&root), [COVER, "urn:b"]);
        assert_eq!(
            store_xml(&root),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <ns0:CoverPageProperties xmlns:ns0=\"{COVER}\" xmlns:ns1=\"urn:b\">\
                 <ns0:Abstract>A &amp; b</ns0:Abstract><ns0:PublishDate/><ns1:Extra>e</ns1:Extra>\
                 </ns0:CoverPageProperties>"
            )
        );
    }

    #[test]
    fn a_name_that_is_no_ncname_is_no_step() {
        let prefixes = prefix_mappings("xmlns:ns0='urn:x'");
        for odd in ["/ns0:-bad[1]", "/ns0:.bad", "/ns0:1bad", "/ns0:a b"] {
            assert_eq!(steps(odd, &prefixes), None, "{odd}");
        }
        assert!(steps("/ns0:_ok-1.2[1]", &prefixes).is_some());
    }

    #[test]
    fn a_control_s_store_value_is_its_text_only_where_word_keeps_it_so() {
        let sdt = |pr: &str, content: &str| {
            parse_xml(
                format!(
                    "<w:sdt xmlns:w=\"urn:w\" xmlns:w14=\"urn:w14\"><w:sdtPr>{pr}</w:sdtPr>\
                     <w:sdtContent>{content}</w:sdtContent></w:sdt>"
                )
                .as_bytes(),
            )
            .unwrap()
        };
        let runs = "<w:p><w:pPr><w:tabs><w:tab w:val=\"left\" w:pos=\"720\"/></w:tabs></w:pPr>\
                    <w:bookmarkStart w:id=\"0\" w:name=\"b\"/><w:r><w:rPr><w:b/></w:rPr><w:t>One</w:t>\
                    <w:tab/><w:t>two</w:t></w:r><w:del><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>";
        let shown_as = |pr: &str, content: &str| shown(&sdt(pr, content));
        assert_eq!(shown_as("<w:text/>", runs).as_deref(), Some("One\ttwo"));
        assert_eq!(
            shown_as("<w:showingPlcHdr/><w:text/>", runs).as_deref(),
            Some("")
        );
        // A date as `w:storeMappedDataAs` asks (dateTime by default).
        let date =
            |mapped: &str| format!("<w:date w:fullDate=\"2015-11-18T00:00:00Z\">{mapped}</w:date>");
        assert_eq!(
            shown_as(&date(""), runs).as_deref(),
            Some("2015-11-18T00:00:00Z")
        );
        assert_eq!(
            shown_as(&date("<w:storeMappedDataAs w:val=\"date\"/>"), runs).as_deref(),
            Some("2015-11-18")
        );
        assert_eq!(
            shown_as(&date("<w:storeMappedDataAs w:val=\"text\"/>"), runs).as_deref(),
            Some("One\ttwo")
        );
        assert_eq!(shown_as("<w:date/>", runs), None);
        // A list holds its chosen item's value.
        let list = |kind: &str| {
            format!(
                "<w:{kind}><w:listItem w:displayText=\"Chosen\" w:value=\"1\"/>\
                 <w:listItem w:value=\"Other\"/></w:{kind}>"
            )
        };
        let chosen = "<w:r><w:t>Chosen</w:t></w:r>";
        assert_eq!(
            shown_as(&list("dropDownList"), chosen).as_deref(),
            Some("1")
        );
        let typed = "<w:r><w:t>typed</w:t></w:r>";
        assert_eq!(shown_as(&list("dropDownList"), typed), None);
        assert_eq!(shown_as(&list("comboBox"), typed).as_deref(), Some("typed"));
        assert_eq!(
            shown_as(&list("comboBox"), "<w:r><w:t>Other</w:t></w:r>").as_deref(),
            Some("Other")
        );
        // Controls whose store value is not their text, and content plain
        // text cannot hold.
        for (pr, content) in [
            ("", runs),
            ("<w:richText/>", runs),
            ("<w:picture/>", runs),
            (
                "<w14:checkbox><w14:checked w14:val=\"1\"/></w14:checkbox>",
                runs,
            ),
            ("<w:text/>", "<w:p><w:r><w:t>a</w:t></w:r></w:p><w:p/>"),
            (
                "<w:text/>",
                "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>",
            ),
            (
                "<w:text/>",
                "<w:r><w:sym w:font=\"Symbol\" w:char=\"F0B7\"/></w:r>",
            ),
            ("<w:text/>", "<w:r><w:drawing><w:x/></w:drawing></w:r>"),
        ] {
            assert_eq!(shown_as(pr, content), None, "{pr} {content}");
        }
        // A multi-line control's break is a line feed.
        assert_eq!(
            shown_as(
                "<w:text w:multiLine=\"1\"/>",
                "<w:r><w:t>a</w:t><w:br/><w:t>b</w:t></w:r>"
            )
            .as_deref(),
            Some("a\nb")
        );
    }

    #[test]
    fn an_unbuilt_binding_is_dropped_and_no_other() {
        let xml = "<w:sdtPr><w:dataBinding w:prefixMappings=\"xmlns:ns0=&apos;urn:a&apos;\" w:xpath=\"/ns0:a[1]/@ns0:b\" w:storeItemID=\"{AB}\"/><w:text/></w:sdtPr>\
                   <w:sdtPr><w:dataBinding w:xpath=\"/ns0:a[1]\" w:storeItemID=\"{AB}\"/></w:sdtPr>";
        assert_eq!(
            unbound(xml, "{ab}", "/ns0:a[1]/@ns0:b"),
            "<w:sdtPr><w:text/></w:sdtPr>\
             <w:sdtPr><w:dataBinding w:xpath=\"/ns0:a[1]\" w:storeItemID=\"{AB}\"/></w:sdtPr>"
        );
    }

    #[test]
    fn a_binding_is_dropped_however_its_tag_is_written() {
        // Single quotes, an entity in the path, an element closed by its
        // own end tag, and the word in text: only the two bindings go.
        let xml = "<w:p><w:t>dataBinding</w:t></w:p>\
                   <w:sdtPr><w:dataBinding w:xpath='/ns0:a[1]/ns0:b[2]' w:storeItemID='{AB}'/><w:text/></w:sdtPr>\
                   <w:sdtPr><w:dataBinding w:xpath=\"/ns0:a[1]/ns0:b[2]\" w:storeItemID=\"{AB}\"></w:dataBinding><w:text/></w:sdtPr>\
                   <w:sdtPr><w:dataBinding w:xpath=\"/ns0:a[1]/ns0:b[&#50;]\" w:storeItemID=\"{CD}\"/></w:sdtPr>";
        assert_eq!(
            unbound(xml, "{ab}", "/ns0:a[1]/ns0:b[2]"),
            "<w:p><w:t>dataBinding</w:t></w:p>\
             <w:sdtPr><w:text/></w:sdtPr>\
             <w:sdtPr><w:text/></w:sdtPr>\
             <w:sdtPr><w:dataBinding w:xpath=\"/ns0:a[1]/ns0:b[&#50;]\" w:storeItemID=\"{CD}\"/></w:sdtPr>"
        );
        assert_eq!(
            unbound(xml, "{CD}", "/ns0:a[1]/ns0:b[2]"),
            "<w:p><w:t>dataBinding</w:t></w:p>\
             <w:sdtPr><w:dataBinding w:xpath='/ns0:a[1]/ns0:b[2]' w:storeItemID='{AB}'/><w:text/></w:sdtPr>\
             <w:sdtPr><w:dataBinding w:xpath=\"/ns0:a[1]/ns0:b[2]\" w:storeItemID=\"{AB}\"></w:dataBinding><w:text/></w:sdtPr>\
             <w:sdtPr></w:sdtPr>"
        );
    }

    #[test]
    fn a_node_bound_whole_and_in_parts_keeps_its_parts() {
        // A repeating section binds its item, then the item's fields.
        let node = |local: &str| ("urn:r".to_string(), local.to_string());
        let whole = [node("root"), node("item")];
        let part = [node("root"), node("item"), node("name")];
        let expected = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <root xmlns=\"urn:r\"><item><name>Ann</name></item></root>";
        // Either order holds the parts.
        for order in [[&whole[..], &part[..]], [&part[..], &whole[..]]] {
            let mut root = Node {
                namespace: "urn:r".to_string(),
                local: "root".to_string(),
                value: None,
                children: Vec::new(),
            };
            for steps in order {
                let value = if steps.len() == 2 { "Ann 3" } else { "Ann" };
                assert!(matches!(insert(&mut root, steps, value), Placed::Yes));
            }
            assert_eq!(store_xml(&root), expected);
        }
    }
}
