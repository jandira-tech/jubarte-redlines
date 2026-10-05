//! The agent fingerprint: Settings' text, written into a redline as the custom
//! document property `AgentFingerprint`. It is invisible in the text; Word
//! shows it under File › Properties › Custom, and an agent reading the package
//! finds it in `docProps/custom.xml`.

use std::borrow::Cow;

use jubarte::opc::PartFs;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesText, Event};
use quick_xml::name::PrefixDeclaration;

/// The property's name.
pub const NAME: &str = "AgentFingerprint";
/// Word's Custom properties tab holds at most 255 characters of text.
pub const MAX_CHARS: usize = 255;

const REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";
const CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.custom-properties+xml";
const NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";
const VT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";
/// The format id of every user-defined property (MS-OSHARED 2.3.3.1.1).
const FMTID: &str = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}";
/// Where Word puts the part when a package has none.
const PART: &str = "docProps/custom.xml";

/// The text as the property holds it: one line, trimmed, at most
/// `MAX_CHARS` characters, and only characters XML 1.0 allows.
pub fn clean(value: &str) -> String {
    let line: String = value
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .filter(|&c| !c.is_control() && c != '\u{FFFE}' && c != '\u{FFFF}')
        .collect();
    line.trim().chars().take(MAX_CHARS).collect()
}

/// `docx` with its `AgentFingerprint` set to `value`. A property of the same
/// name, in any case, is replaced: Word matches names without regard to case
/// and refuses a package that holds two. Other properties are kept. A blank
/// value leaves the package as it was.
pub fn stamp(docx: &[u8], value: &str) -> Result<Vec<u8>, String> {
    let value = clean(value);
    if value.is_empty() {
        return Ok(docx.to_vec());
    }
    let reopen = |e| format!("Could not add the agent fingerprint ({e})");
    let mut pkg = PartFs::open(docx).map_err(reopen)?;
    let related = pkg
        .package_relationships()
        .items
        .iter()
        .find(|r| r.rel_type == REL)
        .map(|r| r.target.trim_start_matches('/').to_owned());
    let part = related.clone().unwrap_or_else(|| PART.to_owned());
    let xml = match pkg.part_string(&part) {
        Some(existing) => with_property(&existing, &value)
            .ok_or("Could not add the agent fingerprint (unreadable custom properties)")?,
        None => format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\r\n\
             <Properties xmlns=\"{NS}\" xmlns:vt=\"{VT}\">{}</Properties>",
            property(2, "", "vt", false, &value)
        ),
    };
    pkg.set_part(&part, xml.into_bytes());
    pkg.add_content_type_override(&format!("/{part}"), CONTENT_TYPE);
    if related.is_none() {
        pkg.add_package_relationship(REL, &part);
    }
    pkg.to_zip().map_err(reopen)
}

/// One `<property>`: `prefix` is the custom-properties prefix in scope (empty
/// for the default namespace), `vt` the variant-types one, declared on the
/// value when `declare_vt`.
fn property(pid: u32, prefix: &str, vt: &str, declare_vt: bool, value: &str) -> String {
    let p = if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix}:")
    };
    let decl = if declare_vt {
        format!(" xmlns:{vt}=\"{VT}\"")
    } else {
        String::new()
    };
    format!(
        "<{p}property fmtid=\"{FMTID}\" pid=\"{pid}\" name=\"{NAME}\">\
         <{vt}:lpwstr{decl}>{}</{vt}:lpwstr></{p}property>",
        quick_xml::escape::escape(value)
    )
}

/// `xml` (a custom-properties part) without any property named `NAME` and
/// with a new one at the end, numbered after the highest `pid`.
fn with_property(xml: &str, value: &str) -> Option<String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut writer = quick_xml::Writer::new(Vec::new());
    let mut root: Option<(String, String, bool)> = None;
    let mut pids = std::collections::BTreeSet::new();
    // Depth inside a dropped property, 0 when none is open.
    let mut dropping = 0usize;
    let named = |e: &quick_xml::events::BytesStart<'_>| {
        e.local_name().as_ref() == "property"
            && e.try_get_attribute("name")
                .ok()
                .flatten()
                .and_then(|a| a.normalized_value(XmlVersion::Implicit1_0).ok())
                .is_some_and(|n| n.to_lowercase() == NAME.to_lowercase())
    };
    loop {
        let event = reader.read_event().ok()?;
        if dropping > 0 {
            match event {
                Event::Start(_) => dropping += 1,
                Event::End(_) => dropping -= 1,
                Event::Eof => return None,
                _ => {}
            }
            continue;
        }
        match &event {
            Event::Start(e) | Event::Empty(e) if root.is_none() => {
                if e.local_name().as_ref() != "Properties" {
                    return None;
                }
                root = Some(scope(e));
                if matches!(event, Event::Empty(_)) {
                    // `<Properties/>`: open it, add the property, close it.
                    let (prefix, vt, declared) = root.as_ref()?;
                    let name = e.name().as_ref().to_owned();
                    writer.write_event(Event::Start(e.clone())).ok()?;
                    let new = property(2, prefix, vt, !declared, value);
                    writer
                        .write_event(Event::Text(BytesText::from_escaped(new)))
                        .ok()?;
                    writer
                        .write_event(Event::End(quick_xml::events::BytesEnd::new(name)))
                        .ok()?;
                    continue;
                }
            }
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "property" => {
                // A dropped property's pid counts too: the new one never
                // reuses a number a reader may remember.
                let pid = e
                    .try_get_attribute("pid")
                    .ok()
                    .flatten()
                    .and_then(|a| a.normalized_value(XmlVersion::Implicit1_0).ok())
                    .and_then(|v: Cow<'_, str>| v.trim().parse::<u32>().ok());
                pids.extend(pid);
                if named(e) {
                    if matches!(event, Event::Start(_)) {
                        dropping = 1;
                    }
                    continue;
                }
            }
            Event::End(e) if e.local_name().as_ref() == "Properties" => {
                let (prefix, vt, declared) = root.as_ref()?;
                let new = property(next_pid(&pids), prefix, vt, !declared, value);
                writer
                    .write_event(Event::Text(BytesText::from_escaped(new)))
                    .ok()?;
            }
            Event::Eof => break,
            _ => {}
        }
        writer.write_event(event).ok()?;
    }
    root?;
    String::from_utf8(writer.into_inner()).ok()
}

/// The number after the highest `pid` (2, the first a user property takes,
/// when there is none). `CT_Property/@pid` is an `xsd:int`: when the highest
/// leaves no room, the lowest free number from 2 is taken.
fn next_pid(pids: &std::collections::BTreeSet<u32>) -> u32 {
    const TOP: u32 = i32::MAX as u32;
    match pids.last() {
        None => 2,
        Some(&high) if high < TOP => high.max(1) + 1,
        Some(_) => (2..=TOP).find(|p| !pids.contains(p)).unwrap_or(2),
    }
}

/// The root's prefix, the prefix bound to the variant types (`vt` when none
/// is), and whether the root binds one.
fn scope(root: &quick_xml::events::BytesStart<'_>) -> (String, String, bool) {
    let prefix = root
        .name()
        .prefix()
        .map(|p| p.as_ref().to_owned())
        .unwrap_or_default();
    let vt = root.attributes().flatten().find_map(|a| {
        let Some(PrefixDeclaration::Named(name)) = a.key.as_namespace_binding() else {
            return None;
        };
        let uri = a.normalized_value(XmlVersion::Implicit1_0).ok()?;
        (uri == VT).then(|| name.to_owned())
    });
    let declared = vt.is_some();
    (prefix, vt.unwrap_or_else(|| "vt".to_owned()), declared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const CT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;

    /// A package; `custom` adds `docProps/custom.xml` and its relationship.
    fn docx(custom: Option<&str>) -> Vec<u8> {
        let mut ct = CT.to_owned();
        let mut rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>"#,
        );
        let mut parts = vec![("word/document.xml", DOC.to_owned())];
        if let Some(xml) = custom {
            ct = ct.replace(
                "</Types>",
                &format!(r#"<Override PartName="/docProps/custom.xml" ContentType="{CONTENT_TYPE}"/></Types>"#),
            );
            rels.push_str(&format!(
                r#"<Relationship Id="rId9" Type="{REL}" Target="docProps/custom.xml"/>"#
            ));
            parts.push(("docProps/custom.xml", xml.to_owned()));
        }
        rels.push_str("</Relationships>");
        parts.push(("[Content_Types].xml", ct));
        parts.push(("_rels/.rels", rels));
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, xml) in parts {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    fn custom_rels(pkg: &PartFs) -> usize {
        pkg.package_relationships()
            .items
            .iter()
            .filter(|r| r.rel_type == REL)
            .count()
    }

    #[test]
    fn a_package_without_custom_properties_gains_the_part_its_type_and_its_relationship() {
        let out = PartFs::open(&stamp(&docx(None), "agent: claude · run 7").unwrap()).unwrap();
        let xml = out.part_string(PART).unwrap();
        assert!(xml.contains(&format!(
            r#"<property fmtid="{FMTID}" pid="2" name="AgentFingerprint"><vt:lpwstr>agent: claude · run 7</vt:lpwstr></property>"#
        )));
        assert!(xml.contains(&format!(r#"<Properties xmlns="{NS}" xmlns:vt="{VT}">"#)));
        assert_eq!(
            out.content_type_for(&format!("/{PART}")).as_deref(),
            Some(CONTENT_TYPE)
        );
        assert_eq!(custom_rels(&out), 1);
        assert_eq!(out.part_string("word/document.xml").as_deref(), Some(DOC));
    }

    #[test]
    fn existing_properties_are_kept_and_a_same_named_one_in_any_case_is_replaced() {
        let existing = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="{NS}" xmlns:vt="{VT}"><property fmtid="{FMTID}" pid="2" name="Client"><vt:lpwstr>Acme &amp; Co</vt:lpwstr></property><property fmtid="{FMTID}" pid="5" name="agentfingerprint"><vt:lpwstr>old</vt:lpwstr></property></Properties>"#
        );
        let out = PartFs::open(&stamp(&docx(Some(&existing)), "new").unwrap()).unwrap();
        let xml = out.part_string(PART).unwrap();
        assert!(xml.contains("<vt:lpwstr>Acme &amp; Co</vt:lpwstr>"));
        assert!(!xml.contains("old"), "{xml}");
        assert_eq!(
            xml.to_lowercase()
                .matches("name=\"agentfingerprint\"")
                .count(),
            1
        );
        // Numbered after the highest pid, the dropped one's included.
        assert!(xml.contains(r#"pid="6" name="AgentFingerprint"><vt:lpwstr>new</vt:lpwstr></property></Properties>"#), "{xml}");
        assert_eq!(custom_rels(&out), 1);
    }

    #[test]
    fn a_pid_at_the_top_of_its_range_never_overflows() {
        // CT_Property/@pid is an xsd:int: past i32::MAX there is no next
        // number, so the lowest free one is taken.
        for top in ["2147483647", "4294967295"] {
            let existing = format!(
                r#"<Properties xmlns="{NS}" xmlns:vt="{VT}"><property fmtid="{FMTID}" pid="2" name="A"><vt:lpwstr>a</vt:lpwstr></property><property fmtid="{FMTID}" pid="{top}" name="B"><vt:lpwstr>b</vt:lpwstr></property></Properties>"#
            );
            let xml = PartFs::open(&stamp(&docx(Some(&existing)), "x").unwrap())
                .unwrap()
                .part_string(PART)
                .unwrap();
            assert!(xml.contains(r#"pid="3" name="AgentFingerprint""#), "{xml}");
        }
    }

    #[test]
    fn prefixes_in_scope_are_used_and_a_missing_vt_binding_is_declared() {
        let prefixed = format!(r#"<op:Properties xmlns:op="{NS}" xmlns:t="{VT}"></op:Properties>"#);
        let xml = PartFs::open(&stamp(&docx(Some(&prefixed)), "x").unwrap())
            .unwrap()
            .part_string(PART)
            .unwrap();
        assert!(xml.contains(r#"<op:property fmtid="#), "{xml}");
        assert!(
            xml.contains("<t:lpwstr>x</t:lpwstr></op:property></op:Properties>"),
            "{xml}"
        );

        let bare = format!(r#"<Properties xmlns="{NS}"/>"#);
        let xml = PartFs::open(&stamp(&docx(Some(&bare)), "y").unwrap())
            .unwrap()
            .part_string(PART)
            .unwrap();
        assert!(
            xml.contains(&format!(r#"pid="2" name="AgentFingerprint"><vt:lpwstr xmlns:vt="{VT}">y</vt:lpwstr></property></Properties>"#)),
            "{xml}"
        );
    }

    #[test]
    fn the_value_is_one_escaped_line_of_at_most_255_characters() {
        assert_eq!(clean("  a\tb\nc\u{0}d\u{FFFF}  "), "a b cd");
        assert_eq!(clean(&"é".repeat(300)).chars().count(), MAX_CHARS);
        let xml = PartFs::open(&stamp(&docx(None), r#"<a href="x">&</a>"#).unwrap())
            .unwrap()
            .part_string(PART)
            .unwrap();
        assert!(
            xml.contains("<vt:lpwstr>&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;</vt:lpwstr>"),
            "{xml}"
        );
    }

    #[test]
    fn a_blank_fingerprint_leaves_the_package_alone_and_garbage_is_refused() {
        let plain = docx(None);
        assert_eq!(stamp(&plain, " \n ").unwrap(), plain);
        assert!(stamp(b"not a zip", "x").is_err());
        assert!(stamp(&docx(Some("<Other/>")), "x").is_err());
    }
}
