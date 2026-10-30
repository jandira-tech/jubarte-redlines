// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// Untrusted bytes reach this module: an out-of-range index or an integer overflow
// is an abort in the Python and WASM consumers, so both are refused here
// (test fixtures are exempt).
#![cfg_attr(
    not(test),
    deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)
)]

//! OPC (Open Packaging Conventions) layer — M1.5.
//!
//! Uses `oxml_opc::OpcPackage` for byte-level parts, content types and target
//! resolution, with local relationship storage and metadata readers to retain
//! the adapter's refusal, repair and canonical serialization contracts.
//!
//! oxml-opc keys parts/rels with a LEADING SLASH (`/word/document.xml`). This
//! adapter accepts the docxodus / opc-partfs style (no leading slash,
//! `word/document.xml`) and normalizes internally, so the rest of the crate is
//! oblivious to the difference.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use oxml_opc::{ContentTypes, OpcPackage};
pub use oxml_opc::{OpcError, Relationship};

/// Relationships with this crate's per-record refusal contract: oxml-opc skips
/// elements outside the relationships namespace, while this crate also reads
/// legacy unnamespaced records and refuses a record missing a required
/// attribute. Direct parsing uses the same reader as [`PartFs::open`].
#[derive(Clone, Debug, Default)]
pub struct Relationships(oxml_opc::Relationships);

impl Relationships {
    /// An empty collection whose generated IDs start at `rId1`.
    pub fn new() -> Self {
        Self(oxml_opc::Relationships::new())
    }

    /// Read a relationships part with the package reader's refusal contract.
    ///
    /// # Errors
    ///
    /// Returns [`OpcError`] for malformed XML or an invalid relationship record.
    pub fn from_xml(xml: &[u8]) -> Result<Self, OpcError> {
        read_relationships(xml)
    }
}

impl Deref for Relationships {
    type Target = oxml_opc::Relationships;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Relationships {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Input the engine refuses before or while reading it, as an `Err` the
/// caller can handle rather than a panic, which would abort a WASM instance
/// or the Python interpreter. `InvalidData` is std's kind for input that is
/// well-formed but unacceptable; the typed `err` travels as the
/// [`std::io::Error`] source, so `Display` carries its message and
/// `io::Error::get_ref` lets a caller downcast it (for example to
/// [`crate::admission::AdmissionError`] and read its stable code).
pub(crate) fn refused<E>(err: E) -> OpcError
where
    E: std::error::Error + Send + Sync + 'static,
{
    OpcError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

/// The message a text surface prints for an [`OpcError`].
/// `DuplicatePartName` carries only the second entry's spelling, so its
/// line is the admission refusal for the same package — `DUPLICATE_PART:`
/// with both spellings — and any other error keeps its own text (#600).
#[must_use]
pub fn refusal_message(docx: &[u8], err: &OpcError) -> String {
    if let OpcError::DuplicatePartName(_) = err
        && let Ok(archive) = open_archive_unchecked(Cursor::new(docx))
    {
        return duplicate_parts(&mut Cursor::new(docx), archive.central_directory_start())
            .into_iter()
            .next()
            .map(|(_, defect)| defect.refusal().to_string())
            .unwrap_or_else(|| err.to_string());
    }
    err.to_string()
}
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

mod archive;
mod zip32;
pub(crate) use archive::{
    ZipDefect, ZipDefectKind, duplicate_parts, open_archive, open_archive_unchecked,
    open_archive_within, zip_defects,
};
pub use zip32::Zip32Limit;
pub(crate) use zip32::zip32_plan;

/// Exactly one leading slash: callers build `/{part}` from names that may
/// already be absolute, and a `//word/…` part name is invalid OPC.
/// Keeps one of each relationship `rels` repeats identically (same Id,
/// type, target and mode), returning the repaired Ids; refuses two
/// different relationships under one Id. Word reads neither as valid, and
/// only the first is a repair that cannot guess wrong.
fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>, OpcError> {
    let mut seen: HashMap<&str, &Relationship> = HashMap::new();
    let mut repaired: Vec<String> = Vec::new();
    let mut keep = Vec::with_capacity(rels.items.len());
    for item in &rels.items {
        match seen.get(item.id.as_str()) {
            None => {
                seen.insert(&item.id, item);
                keep.push(true);
            }
            Some(first) => {
                if first.rel_type != item.rel_type
                    || first.target != item.target
                    || first.target_mode != item.target_mode
                {
                    return Err(OpcError::InvalidRelationship);
                }
                if !repaired.contains(&item.id) {
                    repaired.push(item.id.clone());
                }
                keep.push(false);
            }
        }
    }
    let mut keep = keep.into_iter();
    rels.items.retain(|_| keep.next().unwrap_or(true));
    Ok(repaired)
}

fn norm(name: &str) -> String {
    format!("/{}", name.trim_start_matches('/'))
}

/// The OPC identity of a part or relationship-owner name: resolved like a
/// relationship target (leading slashes, `.`/`..` segments) and compared
/// under ASCII case (ECMA-376 Part 2 §6.2.2.3 M1.12 — `word/Document.xml`
/// IS `word/document.xml`, which is how Word opens probes p3–p5 of
/// ZIP-15/#600). The `part_index`/`rels_index` maps keep one stored
/// spelling per identity, so lookups stay O(1) without a scan.
fn identity(name: &str) -> String {
    // `resolve_rel_target` changes a name only to strip its leading '/'
    // or to resolve `.`, `..` and empty segments: a name with none of
    // those IS the resolved name, so the common lookup allocates once
    // for the lowercasing instead of twice (#600 lookup cost).
    let needs_resolve = name.starts_with('/')
        || name
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..");
    if needs_resolve {
        let mut resolved = resolve_rel_target("", name);
        resolved.make_ascii_lowercase();
        resolved
    } else {
        name.to_ascii_lowercase()
    }
}

/// The relationship `Target` value that reaches `target_part` from
/// `source_part`: relative to the source's folder when the target sits
/// under it, absolute (`/media/image.bin`) otherwise. Stripping `word/`
/// blindly pointed a root-level `media/image.bin` at the non-existent
/// `word/media/image.bin` and Word refused the package.
pub fn relative_rel_target(source_part: &str, target_part: &str) -> String {
    let source = source_part.trim_start_matches('/');
    let target = target_part.trim_start_matches('/');
    let dir = source.rsplit_once('/').map_or("", |(d, _)| d);
    match target.strip_prefix(dir).and_then(|t| t.strip_prefix('/')) {
        Some(rel) if !dir.is_empty() => rel.to_string(),
        _ => format!("/{target}"),
    }
}

/// Select the same unprefixed metadata records as the legacy adapter, folding
/// explicit closes, then let upstream decode and validate each record. Parsing
/// records separately preserves identical-duplicate repair and last-wins content
/// types before the newer whole-document parsers reject those producer shapes.
/// A record's own default-namespace declaration is dropped: it would unbind
/// the synthetic root's OPC namespace (`<Relationship xmlns=""/>`) and upstream
/// would silently skip the record instead of validating it, where the legacy
/// parser matched records by local name whatever namespaces declared.
fn metadata_records(
    xml: &[u8],
    root: &str,
    namespace: &str,
    tags: &[&str],
) -> Result<Vec<Vec<u8>>, OpcError> {
    // quick-xml 0.41 is the version `OpcError::Xml`/`XmlAttr` carry (oxml-opc
    // 0.14.2), so the scanner's errors convert into those variants directly;
    // 0.42's error types do not, and wrapping them made every syntax error an
    // I/O refusal.
    use quick_xml_041::events::{BytesEnd, BytesStart, Event};
    let fold = |name: &str, tag: quick_xml_041::events::BytesStart<'_>| {
        let mut record = BytesStart::new(name);
        for attr in tag.attributes().with_checks(false) {
            let attr = attr?;
            if attr.key.as_ref() != &b"xmlns"[..] {
                record.push_attribute(attr);
            }
        }
        let mut writer = quick_xml_041::Writer::new(Vec::new());
        let mut start = BytesStart::new(root);
        start.push_attribute(("xmlns", namespace));
        writer.write_event(Event::Start(start))?;
        writer.write_event(Event::Empty(record))?;
        writer.write_event(Event::End(BytesEnd::new(root)))?;
        Ok::<_, OpcError>(writer.into_inner())
    };
    let mut reader = quick_xml_041::Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut records = Vec::new();
    // An explicit close (`<Relationship …></Relationship>`) is folded between
    // events: a `Start` immediately followed by its matching `End`, with no
    // other event between, is one `Empty` record. The reader still checks end
    // names, so a mismatched close stays an XML error, and a `>` inside an
    // attribute value (`Target="a></Relationship>b"`, legal unescaped) is
    // never rewritten — a textual `></tag>` replacement did both.
    let mut pending: Option<(&str, BytesStart<'static>)> = None;
    loop {
        let event = reader.read_event_into(&mut buf).map_err(OpcError::from)?;
        if let Some((name, tag)) = pending.take()
            && matches!(&event, Event::End(end) if end.name() == tag.name())
        {
            records.push(fold(name, tag)?);
            continue;
        }
        let matched = |tag: &BytesStart<'_>| {
            tags.iter()
                .copied()
                .find(|name| tag.name().as_ref() == name.as_bytes())
        };
        match event {
            Event::Start(ref tag) => {
                if let Some(name) = matched(tag) {
                    pending = Some((name, tag.to_owned()));
                }
            }
            Event::Empty(ref tag) => {
                if let Some(name) = matched(tag) {
                    records.push(fold(name, tag.to_owned())?);
                }
            }
            Event::Eof => return Ok(records),
            _ => {}
        }
    }
}

fn read_relationships(xml: &[u8]) -> Result<Relationships, OpcError> {
    let mut rels = Relationships::new();
    let mut items = Vec::new();
    for record in metadata_records(
        xml,
        "Relationships",
        "http://schemas.openxmlformats.org/package/2006/relationships",
        &["Relationship"],
    )? {
        for item in oxml_opc::Relationships::from_xml(&record)?.items {
            items.push(item);
        }
    }
    // Seed the private counter once: add_with_id removes existing IDs, so
    // calling it for every record would turn a linear read into a quadratic one.
    // A fresh collection also has no retained source XML: to_xml stays canonical.
    if let Some((_, item)) = items
        .iter()
        .filter_map(|item| {
            item.id
                .strip_prefix("rId")
                .and_then(|id| id.parse::<u32>().ok())
                .map(|id| (id, item))
        })
        .max_by_key(|(id, _)| *id)
    {
        rels.add_with_id(&item.id, &item.rel_type, &item.target);
    }
    rels.items = items;
    Ok(rels)
}

/// Whether `name` ends with `suffix`, ASCII case-blind (OPC compares part
/// names so).
pub(crate) fn ends_with_ignore_case(name: &str, suffix: &str) -> bool {
    name.len()
        .checked_sub(suffix.len())
        .and_then(|at| name.get(at..))
        .is_some_and(|end| end.eq_ignore_ascii_case(suffix))
}

/// Whether a content type names XML: `…+xml`, `application/xml` or
/// `text/xml`, ASCII case-blind.
pub(crate) fn is_xml_content_type(content_type: &str) -> bool {
    ends_with_ignore_case(content_type, "+xml")
        || content_type.eq_ignore_ascii_case("application/xml")
        || content_type.eq_ignore_ascii_case("text/xml")
}

/// Transitional's and Strict's `customXml` relationship types.
const CUSTOM_XML_RELATIONSHIPS: [&str; 2] = [
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/customXml",
];

/// The custom XML data parts `rels` reach, each `(source part, its
/// relationships)`, the package's own under `""`: the parts only
/// `customXml` relationships reach, by their exact type. Such a part holds
/// the application's own XML. A part another relationship reaches too (the
/// main document, a header) is that part, whatever else points at it.
pub(crate) fn custom_xml_data<'a>(
    rels: impl IntoIterator<Item = (&'a str, &'a Relationships)>,
) -> Vec<String> {
    // By ASCII-lowercase name: OPC compares part names so.
    let mut only_custom: HashMap<String, bool> = HashMap::new();
    for (source, rels) in rels {
        for r in rels
            .items
            .iter()
            .filter(|r| r.target_mode.as_deref() != Some("External"))
        {
            let target = crate::xmllinq::parse::unescape_xml_text(&r.target);
            let part = resolve_rel_target(source, &target).to_ascii_lowercase();
            *only_custom.entry(part).or_insert(true) &=
                CUSTOM_XML_RELATIONSHIPS.contains(&r.rel_type.as_str());
        }
    }
    only_custom
        .into_iter()
        .filter_map(|(part, custom)| custom.then_some(part))
        .collect()
}

/// For a reader that has not opened the package: whether the raw
/// `[Content_Types].xml` `content_types` types part `name` as XML
/// ([`is_xml_content_type`]). Unreadable content types type nothing.
pub(crate) fn typed_as_xml(content_types: Option<&[u8]>) -> impl Fn(&str) -> bool + use<> {
    let types = content_types.and_then(|xml| read_content_types(xml).ok());
    move |name| {
        types
            .as_ref()
            .and_then(|types| types.content_type_for(&norm(name)))
            .is_some_and(is_xml_content_type)
    }
}

fn read_content_types(xml: &[u8]) -> Result<ContentTypes, OpcError> {
    let mut types = ContentTypes {
        defaults: HashMap::new(),
        overrides: HashMap::new(),
    };
    for record in metadata_records(
        xml,
        "Types",
        "http://schemas.openxmlformats.org/package/2006/content-types",
        &["Default", "Override"],
    )? {
        let parsed = ContentTypes::from_xml(&record)?;
        types.defaults.extend(parsed.defaults);
        types.overrides.extend(parsed.overrides);
    }
    Ok(types)
}

/// Populate parts and local relationships without invoking upstream's whole-package
/// normalization and duplicate-refusal policies ahead of the adapter's repairs.
///
/// Bounded like [`crate::admission::admit`] (#645): each entry inflates
/// through a reader capped at the smaller of `limits.max_part_bytes` and the
/// package budget left, with a checked cumulative total, because the sizes
/// the central directory declares are not trusted and no caller can be
/// relied on to have admitted the bytes first. An entry that declares or
/// inflates past its cap is refused with the admission `INPUT_LIMIT`
/// wording.
fn read_package(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<PartFs, OpcError> {
    crate::admission::preindex_budget(bytes, limits).map_err(refused)?;
    // Duplicate part identities are refused below as
    // `DuplicatePartName`, the variant callers match on (#600); every
    // other structural defect refuses in the shared open. The raw walk
    // of the central directory must catch them: zip 9's index collapses
    // a byte-equal duplicate before the decode loop can see both
    // records, and its decoded names agree with the per-entry decode on
    // every shape (bit 11 / CP437 / 0x7075).
    let mut archive = open_archive_within(Cursor::new(bytes), false)?;
    if let Some((second, _)) =
        duplicate_parts(&mut Cursor::new(bytes), archive.central_directory_start())
            .into_iter()
            .next()
    {
        return Err(OpcError::DuplicatePartName(second));
    }
    let mut parts = HashMap::new();
    let mut source_order = HashMap::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        // zip 8's name decode (bit 11/CP437, 0x7075 override): the same one
        // `admit` applies, so both readers see one name per entry (#524).
        let name = crate::admission::zip_entry_name(
            entry.name_raw(),
            entry.flags().as_u16(),
            crate::admission::unicode_path_fields(entry.extra_data_fields().cloned()),
        )?;
        let directory = entry.is_dir();
        if !directory {
            source_order.insert(name.clone(), index);
        }
        if entry.size() > limits.max_part_bytes {
            return Err(refused(crate::admission::part_declared_over(
                &name,
                entry.size(),
                &limits,
            )));
        }
        let cap = limits
            .max_part_bytes
            .min(limits.max_uncompressed_bytes.saturating_sub(total));
        let mut data = Vec::new();
        // A directory entry's payload is drained too: the CRC stream is the
        // only check on bytes nobody else reads, and the inflated count
        // still charges the same budgets (#644). A corrupt directory
        // payload is INVALID_PACKAGE, as it is in `admit`.
        let read = if directory {
            std::io::copy(
                &mut (&mut entry).take(cap.saturating_add(1)),
                &mut std::io::sink(),
            )
            .map_err(|e| {
                refused(crate::admission::AdmissionError {
                    kind: crate::admission::AdmissionErrorKind::InvalidPackage,
                    message: format!("{}: {e}", crate::admission::shown(&name)),
                })
            })?
        } else {
            u64::try_from(
                (&mut entry)
                    .take(cap.saturating_add(1))
                    .read_to_end(&mut data)?,
            )
            .unwrap_or(u64::MAX)
        };
        crate::admission::tally_inflated(&mut total, read, &name, cap, &limits).map_err(refused)?;
        if directory {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if lower == "[content_types].xml" || lower.ends_with(".rels") {
            check_opc_attributes(&data)?;
        }
        parts.insert(name, data);
    }
    let mut package = PartFs {
        pkg: OpcPackage::new(),
        package_rels: Relationships::new(),
        part_rels: HashMap::new(),
        source_order,
        repaired_duplicate_ids: Vec::new(),
        part_index: HashMap::new(),
        rels_index: HashMap::new(),
    };
    // The reserved names match under ASCII case (M1.12): a lone
    // `[content_types].xml` or `_RELS/.RELS` is the part (Word opens it,
    // #600 probes p4/p5); `to_zip` writes the canonical spelling back.
    let content_types = parts
        .keys()
        .find(|name| name.eq_ignore_ascii_case("[Content_Types].xml"))
        .cloned()
        .and_then(|name| parts.remove(&name))
        .ok_or_else(|| OpcError::PartNotFound("[Content_Types].xml".to_string()))?;
    package.pkg.content_types = read_content_types(&content_types)?;
    if let Some(name) = parts
        .keys()
        .find(|name| name.eq_ignore_ascii_case("_rels/.rels"))
        .cloned()
    {
        package.package_rels = read_relationships(&parts.remove(&name).unwrap_or_default())?;
    }
    for (name, data) in parts {
        if name.to_ascii_lowercase().ends_with(".rels") {
            let owner = rels_path_to_part_name(&name).unwrap_or_else(|| {
                name.get(..name.len().saturating_sub(5))
                    .unwrap_or(name.as_str())
                    .to_string()
            });
            let key = norm(&owner);
            package.rels_index.insert(identity(&key), key.clone());
            package.part_rels.insert(key, read_relationships(&data)?);
        } else {
            let key = norm(&name);
            package.part_index.insert(identity(&key), key.clone());
            package.pkg.parts.insert(key, Arc::new(data));
        }
    }
    Ok(package)
}

fn denorm(name: &str) -> String {
    name.trim_start_matches('/').to_string()
}

/// `.rels` file path (e.g. `word/_rels/document.xml.rels`). Mirrors the
/// private `oxml_opc::package::part_name_to_rels_path`.
fn part_name_to_rels_path(part_name: &str) -> String {
    let name = part_name.strip_prefix('/').unwrap_or(part_name);
    if let Some((dir, file)) = name.rsplit_once('/') {
        format!("{dir}/_rels/{file}.rels")
    } else {
        format!("_rels/{name}.rels")
    }
}

/// The part a `.rels` path belongs to (`word/_rels/header1.xml.rels` →
/// `word/header1.xml`; `_rels/.rels` → the package, as `""`), or None when
/// `name` is not a relationships path. Inverse of `part_name_to_rels_path`.
/// The `_rels` segment and the `.rels` suffix match under ASCII case —
/// `_RELS/.RELS` is the package's own relationships (Word opens it, ZIP-15
/// probe p5) — while the owner's spelling keeps the entry's case.
pub(crate) fn rels_path_to_part_name(name: &str) -> Option<String> {
    let name = name.strip_prefix('/').unwrap_or(name);
    let lower = name.to_ascii_lowercase();
    let file = lower.strip_suffix(".rels")?;
    let (dir, base) = file.rsplit_once('/')?;
    let tail = name.len().checked_sub(5)?;
    let base = name.get(tail.checked_sub(base.len())?..tail)?;
    let owner_dir = if dir == "_rels" {
        ""
    } else {
        dir.strip_suffix("/_rels")?
    };
    Some(if owner_dir.is_empty() {
        base.to_string()
    } else {
        format!("{}/{base}", name.get(..owner_dir.len())?)
    })
}

/// One zip entry `PartFs::to_zip` writes after the two package-level ones.
enum ZipEntry<'a> {
    Rels(&'a Relationships),
    Part(&'a [u8]),
}

impl ZipEntry<'_> {
    /// The uncompressed byte length of the entry's content.
    fn len(&self) -> u64 {
        match self {
            // Serialized at plan time; a rels part never nears the boundary.
            ZipEntry::Rels(rels) => rels.to_xml().map_or(0, |x| x.len() as u64),
            ZipEntry::Part(data) => data.len() as u64,
        }
    }
}

// Retain the OPC metadata budget independently of the parser upgrade that
// fixed RUSTSEC-2026-0194/0195. Direct Relationships::from_xml callers use
// patched quick-xml through oxml-opc 0.14.2; package admission also bounds the
// attribute count. 256 matches quick-xml's default namespace declaration limit.
fn check_opc_attributes(xml: &[u8]) -> Result<(), OpcError> {
    let mut reader = quick_xml::Reader::from_reader(xml);
    loop {
        match reader.read_event().map_err(refused)? {
            quick_xml::events::Event::Start(tag) | quick_xml::events::Event::Empty(tag) => {
                for (index, attr) in tag.attributes().with_checks(false).enumerate() {
                    if index >= 256 {
                        return Err(refused(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "OPC metadata element exceeds 256 attributes",
                        )));
                    }
                    attr.map_err(refused)?;
                }
            }
            quick_xml::events::Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

/// Thin adapter over `oxml_opc::OpcPackage`. Port-equivalent of `PartFS`.
pub struct PartFs {
    // Only parts and content types live upstream; its relationship maps stay empty.
    pkg: OpcPackage,
    package_rels: Relationships,
    part_rels: HashMap<String, Relationships>,
    /// Each source zip entry's position, so `to_zip` writes the package back
    /// in the source's order rather than the hash maps' run-to-run order.
    source_order: HashMap<String, usize>,
    /// `(part, Id)` of each relationship the source repeated identically and
    /// [`PartFs::open`] kept once (`""` is the package's own relationships).
    repaired_duplicate_ids: Vec<(String, String)>,
    /// `pkg.parts` by OPC identity (ASCII-lowercased resolved name): one
    /// stored key per identity, so an alias resolves in O(1) (ZIP-15/#600).
    part_index: HashMap<String, String>,
    /// `part_rels` owners by identity, same contract as `part_index`.
    rels_index: HashMap<String, String>,
}

/// Canonical target resolution shared by package APIs and text story labels.
pub(crate) fn resolve_rel_target(source_part: &str, rel_target: &str) -> String {
    // The upstream resolver already returns the normalized absolute
    // name (`.`/`..`/empty segments collapsed under a leading '/');
    // PartFs stores the no-leading-slash form.
    OpcPackage::resolve_rel_target(&norm(source_part), rel_target)
        .trim_start_matches('/')
        .to_string()
}

impl PartFs {
    /// Open a `.docx`/OPC package from raw bytes: read its parts,
    /// relationships and `[Content_Types].xml`, normalize their part
    /// names to the no-leading-slash form, fold any explicit `<…></…>`
    /// closes of empty elements (`<Relationship …></Relationship>` and
    /// its `[Content_Types]` siblings) into self-closing ones, decode
    /// XML entities that the underlying parser left raw, and keep one
    /// copy of any identical repeated relationship.
    ///
    /// The package is read under [`crate::admission::InputLimits::compare`]
    /// (#645): an input past the compressed-size or entry-count budget is
    /// refused before the archive is indexed, and a part past the per-part
    /// or package budget while it inflates, with the admission `INPUT_LIMIT`
    /// wording. Callers that admit under their own budget open through
    /// `open_within` so their limits keep governing.
    ///
    /// # Errors
    ///
    /// [`OpcError`] for an unreadable zip, a package past the input budget
    /// (an [`crate::admission::AdmissionError`] with code `INPUT_LIMIT` as
    /// the [`std::io::Error`] source), an OPC metadata element exceeding
    /// 256 attributes, two relationships under the same `Id` that differ in
    /// any attribute, or two entries whose decoded part names are equal
    /// byte-for-byte or under ASCII case
    /// ([`OpcError::DuplicatePartName`], naming the second entry's
    /// spelling; text surfaces print the admission `DUPLICATE_PART`
    /// detail through [`refusal_message`]).
    pub fn open(bytes: &[u8]) -> Result<Self, OpcError> {
        Self::open_within(bytes, crate::admission::InputLimits::compare())
    }

    /// [`PartFs::open`] under an explicit budget: the entry points that
    /// admit an input under their own limits (the comparer's settings)
    /// open it under the same budget here, so a looser caller budget still
    /// governs and a tighter one has already refused the package.
    pub(crate) fn open_within(
        bytes: &[u8],
        limits: crate::admission::InputLimits,
    ) -> Result<Self, OpcError> {
        let mut fs = read_package(bytes, limits)?;
        let mut repaired_duplicate_ids: Vec<(String, String)> =
            drop_repeated_relationships(&mut fs.package_rels)?
                .into_iter()
                .map(|id| (String::new(), id))
                .collect();
        let mut owners: Vec<(&String, &mut Relationships)> = fs.part_rels.iter_mut().collect();
        owners.sort_by(|a, b| a.0.cmp(b.0));
        for (owner, rels) in owners {
            let part = owner.trim_start_matches('/').to_string();
            repaired_duplicate_ids.extend(
                drop_repeated_relationships(rels)?
                    .into_iter()
                    .map(|id| (part.clone(), id)),
            );
        }
        fs.repaired_duplicate_ids = repaired_duplicate_ids;
        Ok(fs)
    }

    /// `(part, Id)` of each relationship the source repeated identically,
    /// kept once on open (`""`: the package's own relationships). Word
    /// refuses such a package; `validate` reports these as `DUPLICATE_RID`.
    #[must_use]
    pub fn repaired_duplicate_ids(&self) -> &[(String, String)] {
        &self.repaired_duplicate_ids
    }

    /// `PartFS.partBytes(name)` — raw bytes of a part. The lookup resolves
    /// under OPC identity: `word/document.xml` finds a part stored as
    /// `word/Document.xml` (#600).
    pub fn part_bytes(&self, name: &str) -> Option<&[u8]> {
        self.pkg
            .parts
            .get(self.part_index.get(&identity(name))?)
            .map(|data| data.as_slice())
    }

    /// Read a part as a UTF-8 string.
    pub fn part_string(&self, name: &str) -> Option<String> {
        self.part_bytes(name)
            .map(|b| String::from_utf8_lossy(b).into_owned())
    }

    /// `PartFS.setPart(name, data)` — replace or add a part.
    ///
    /// A `…/_rels/<part>.rels` name replaces that part's relationships: the
    /// package keeps relationships parsed, and `to_zip` writes them next to the
    /// raw parts, so a raw `.rels` copy would be a second zip entry of the same
    /// name (the whole package fails) and invisible to `read_rels_for`.
    /// Unparseable relationship XML is kept raw, as before.
    /// `PartFS.setPart(name, data)` — replace or add a part. A name that
    /// is a case alias of a stored part replaces it and keeps the stored
    /// spelling (there is one part per identity, #600).
    ///
    /// A `…/_rels/<part>.rels` name (matched under ASCII case) replaces
    /// that part's relationships: the package keeps relationships parsed,
    /// and `to_zip` writes them next to the raw parts, so a raw `.rels`
    /// copy would be a second zip entry of the same name (the whole
    /// package fails) and invisible to `read_rels_for`. Unparseable
    /// relationship XML is kept raw, as before; `to_zip` then refuses the
    /// collision rather than write a duplicate name. A
    /// `[Content_Types].xml` name likewise replaces the parsed content
    /// types when it parses.
    pub fn set_part(&mut self, name: &str, data: Vec<u8>) {
        if let Some(owner) = rels_path_to_part_name(name)
            && check_opc_attributes(&data).is_ok()
            && let Ok(rels) = read_relationships(&data)
        {
            if owner.is_empty() {
                self.package_rels = rels;
            } else {
                let key = self
                    .rels_index
                    .get(&identity(&owner))
                    .cloned()
                    .unwrap_or_else(|| norm(&owner));
                self.rels_index.insert(identity(&key), key.clone());
                self.part_rels.insert(key, rels);
            }
            // A raw part of the same .rels identity is superseded: `to_zip`
            // writes the parsed copy, and two entries of one name fail.
            if let Some(key) = self.part_index.remove(&identity(name)) {
                self.pkg.parts.remove(&key);
            }
            return;
        }
        if name
            .trim_start_matches('/')
            .eq_ignore_ascii_case("[Content_Types].xml")
            && check_opc_attributes(&data).is_ok()
            && let Ok(types) = read_content_types(&data)
        {
            self.pkg.content_types = types;
            if let Some(key) = self.part_index.remove(&identity(name)) {
                self.pkg.parts.remove(&key);
            }
            return;
        }
        let key = self
            .part_index
            .get(&identity(name))
            .cloned()
            .unwrap_or_else(|| norm(name));
        self.part_index.insert(identity(&key), key.clone());
        self.pkg.parts.insert(key, Arc::new(data));
    }

    /// Remove a part (no-op when absent), resolved under OPC identity:
    /// an alias removes the stored part, a `.rels` name removes the
    /// owner's relationships, and the `[Content_Types].xml` identity
    /// clears the content types. Content-type overrides and rels pointing
    /// at the part are left to the caller.
    pub fn remove_part(&mut self, name: &str) {
        if let Some(owner) = rels_path_to_part_name(name) {
            if owner.is_empty() {
                self.package_rels = Relationships::new();
            } else if let Some(key) = self.rels_index.remove(&identity(&owner)) {
                self.part_rels.remove(&key);
            }
        }
        if name
            .trim_start_matches('/')
            .eq_ignore_ascii_case("[Content_Types].xml")
        {
            self.pkg.content_types = ContentTypes {
                defaults: HashMap::new(),
                overrides: HashMap::new(),
            };
        }
        if let Some(key) = self.part_index.remove(&identity(name)) {
            self.pkg.parts.remove(&key);
        }
    }

    /// Enumerate all part names (docxodus style, no leading slash), sorted.
    pub fn parts(&self) -> Vec<String> {
        let mut v: Vec<String> = self.pkg.parts.keys().map(|k| denorm(k)).collect();
        v.sort();
        v
    }

    /// Serialize the package back to zip bytes.
    ///
    /// Uses deflate compression level 1 (`deflate_quick`) instead of the
    /// `zip` crate default level 6. Level 1 skips `longest_match` (the
    /// largest WASM self-time frame, 26% of the deflate cluster per the W5
    /// profile) while producing content-identical decompressed bytes — Word
    /// opens any deflate level. ZIP-LEVEL-01 (docs/WASM_PERF_PLAN.md).
    ///
    /// Every entry is dated 1980-01-01 00:00, as Office dates its own, so the
    /// same input writes the same bytes.
    pub fn to_zip(&self) -> Result<Vec<u8>, OpcError> {
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(1))
            .last_modified_time(zip::DateTime::default());

        let ct_xml = self.pkg.content_types.to_xml()?;
        let pkg_rels_xml = self.package_rels.to_xml()?;

        // Source entries keep their places; added ones follow, by name.
        let mut entries: Vec<(Cow<str>, ZipEntry)> = self
            .part_rels
            .iter()
            .map(|(part, rels)| {
                (
                    Cow::Owned(part_name_to_rels_path(part)),
                    ZipEntry::Rels(rels),
                )
            })
            .chain(self.pkg.parts.iter().map(|(name, data)| {
                let zip_name = name.strip_prefix('/').unwrap_or(name);
                (Cow::Borrowed(zip_name), ZipEntry::Part(data))
            }))
            .collect();
        entries.sort_by(|(a, _), (b, _)| {
            let rank = |name: &str| self.source_order.get(name).copied().unwrap_or(usize::MAX);
            (rank(a), a).cmp(&(rank(b), b))
        });
        // jubarte writes ZIP32 only (ZIP-10/#604): a package that would
        // need ZIP64 is refused before `ZipWriter::new`, not mid-write.
        let plan: Vec<(&str, u64)> = [
            ("[Content_Types].xml", ct_xml.len() as u64),
            ("_rels/.rels", pkg_rels_xml.len() as u64),
        ]
        .into_iter()
        .chain(
            entries
                .iter()
                .map(|(name, entry)| (name.as_ref(), entry.len())),
        )
        .collect();
        // One entry per OPC identity (ZIP-15/#600): a raw part that shares
        // a name with a parsed rels part or a reserved name is
        // `DUPLICATE_PART`, not an archive Word refuses.
        let mut emitted = HashSet::new();
        if let Some(name) = plan
            .iter()
            .map(|(name, _)| name)
            .find(|name| !emitted.insert(name.to_ascii_lowercase()))
        {
            return Err(refused(crate::admission::AdmissionError::new(
                crate::admission::AdmissionErrorKind::DuplicatePart,
                format!("'{name}' would be written twice (part names ignore case)"),
            )));
        }
        zip32_plan(&plan).map_err(refused)?;

        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(&ct_xml)?;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(&pkg_rels_xml)?;
        for (name, entry) in &entries {
            zip.start_file(name.as_ref(), options)?;
            match entry {
                ZipEntry::Rels(rels) => zip.write_all(&rels.to_xml()?)?,
                ZipEntry::Part(data) => zip.write_all(data)?,
            }
        }

        Ok(zip.finish()?.into_inner())
    }

    // ── gap helpers (the few bits opc-partfs adds on top of raw zip) ───────────

    /// `resolveRelTarget(sourcePart, relTarget)` — the part a rel target names,
    /// relative to its source part, as a canonical part name: no leading slash
    /// (the form `parts()` returns) and `.`/`..` segments resolved. An echoed
    /// absolute target ("/word/footer1.xml") was later re-prefixed into a
    /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
    pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
        // PartFs receives raw relationship attributes; DOM consumers already
        // decoded XML entities and call the pure resolver directly.
        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
        resolve_rel_target(source_part, &target)
    }

    /// `contentTypeFor(part)`.
    pub fn content_type_for(&self, name: &str) -> Option<String> {
        self.pkg
            .content_types
            .content_type_for(&norm(name))
            .map(|s| s.to_string())
    }

    /// Add an Override entry to `[Content_Types].xml`.
    pub fn add_content_type_override(&mut self, part_name: &str, content_type: &str) {
        self.pkg
            .content_types
            .add_override(&norm(part_name), content_type);
    }

    /// Add a Default extension mapping to `[Content_Types].xml`.
    pub fn add_content_type_default(&mut self, ext: &str, content_type: &str) {
        self.pkg.content_types.add_default(ext, content_type);
    }

    /// Give every part that has no content type the one its source declares:
    /// the source's entry for the same part name, else the source's Default for
    /// the extension (case-insensitive, as OPC matches extensions). A part no
    /// source types is left alone rather than guessed.
    pub fn adopt_missing_content_types(&mut self, sources: &[&PartFs]) {
        for part in self.parts() {
            // `content_type_for` already matches a Default under another
            // case (`png` for `image3.PNG`), so a part typed by our own
            // Default stops here: a second Default for the same extension
            // would make the package unopenable.
            if self.content_type_for(&part).is_some() {
                continue;
            }
            let Some((_, ext)) = part.rsplit_once('.').filter(|(_, e)| !e.contains('/')) else {
                continue;
            };
            let by_ext = |fs: &PartFs| {
                fs.pkg
                    .content_types
                    .defaults
                    .iter()
                    .find(|(e, _)| e.eq_ignore_ascii_case(ext))
                    .map(|(_, ct)| ct.clone())
            };
            let key = norm(&part);
            for src in sources {
                if let Some(ct) = src.pkg.content_types.override_for(&key).map(str::to_string) {
                    self.add_content_type_override(&part, &ct);
                    break;
                }
                if let Some(ct) = by_ext(src) {
                    self.add_content_type_default(ext, &ct);
                    break;
                }
            }
        }
    }

    /// Remove an Override entry from `[Content_Types].xml` (no-op when
    /// absent). The name resolves under OPC identity, so an alias removes
    /// the stored spelling (#600).
    pub fn remove_content_type_override(&mut self, part_name: &str) {
        self.pkg.content_types.remove_override(&norm(part_name));
    }

    /// Remove every relationship of `source_part` with the given type.
    /// No-op when the source part has no relationships part yet — must not
    /// invent an empty `.rels` entry just to remove from it (PR #81 review).
    pub fn remove_relationships_by_type(&mut self, source_part: &str, rel_type: &str) {
        if let Some(key) = self.part_rels_key(source_part).cloned()
            && let Some(rels) = self.part_rels.get_mut(&key)
        {
            rels.items.retain(|r| r.rel_type != rel_type);
        }
    }

    /// Remove every package-level relationship (`_rels/.rels`) of the given
    /// type. The parts they target are left to the caller.
    pub fn remove_package_relationships_by_type(&mut self, rel_type: &str) {
        self.package_rels.items.retain(|r| r.rel_type != rel_type);
    }

    /// Remove the relationship `rel_id` of `source_part` and, unless another
    /// internal relationship still targets it, the part it names with that
    /// part's own relationships and content-type override, then the parts
    /// (a header's images) nothing else targets any more. No-op for an
    /// unknown or external relationship.
    pub fn remove_related_part(&mut self, source_part: &str, rel_id: &str) {
        let Some(key) = self.part_rels_key(source_part).cloned() else {
            return;
        };
        let Some(rels) = self.part_rels.get_mut(&key) else {
            return;
        };
        let Some(target) = rels
            .items
            .iter()
            .find(|r| r.id == rel_id && r.target_mode.as_deref() != Some("External"))
            .map(|r| r.target.clone())
        else {
            return;
        };
        rels.items.retain(|r| r.id != rel_id);
        let part = self.resolve_rel_target(source_part, &target);
        self.remove_untargeted_part(&part);
    }

    /// Remove `part`, its relationships and content-type override when no
    /// internal relationship of another part targets it, then each part it
    /// targeted that is left untargeted. Its own relationships never keep
    /// it, and a removed part ends a loop.
    fn remove_untargeted_part(&mut self, part: &str) {
        let part_id = identity(part);
        let Some(key) = self.part_index.get(&part_id).cloned() else {
            return;
        };
        let still_targeted = self
            .part_rels
            .iter()
            .filter(|(source, _)| identity(source) != part_id)
            .map(|(source, rels)| (source.as_str(), rels))
            .chain(std::iter::once(("/", &self.package_rels)))
            .any(|(source, rels)| {
                rels.items.iter().any(|r| {
                    r.target_mode.as_deref() != Some("External")
                        && self
                            .resolve_rel_target(source, &r.target)
                            .eq_ignore_ascii_case(part)
                })
            });
        if still_targeted {
            return;
        }
        self.part_index.remove(&part_id);
        self.pkg.parts.remove(&key);
        let rels_key = self.rels_index.get(&part_id).cloned().unwrap_or(key);
        let targets: Vec<String> = self
            .part_rels
            .remove(&rels_key)
            .map(|rels| {
                rels.items
                    .iter()
                    .filter(|r| r.target_mode.as_deref() != Some("External"))
                    .map(|r| self.resolve_rel_target(part, &r.target))
                    .collect()
            })
            .unwrap_or_default();
        self.rels_index.remove(&part_id);
        self.remove_content_type_override(&format!("/{part}"));
        for target in targets {
            self.remove_untargeted_part(&target);
        }
    }

    /// `readRelsFor(part)` — the relationships of a part, if any.
    pub fn read_rels_for(&self, part_name: &str) -> Option<&Relationships> {
        self.part_rels.get(self.part_rels_key(part_name)?)
    }

    // The stored owner key for `part_name` under OPC identity — upstream's
    // case-insensitive, normalized lookup over the rels index.
    fn part_rels_key(&self, part_name: &str) -> Option<&String> {
        self.rels_index.get(&identity(part_name))
    }

    /// `addDocumentRelationship(...)` — add a relationship to a part, returning
    /// the new relationship id.
    pub fn add_document_relationship(
        &mut self,
        source_part: &str,
        rel_type: &str,
        target: &str,
    ) -> String {
        self.part_rels_mut(source_part).add(rel_type, target)
    }

    /// The package's own relationships (`_rels/.rels`).
    pub fn package_relationships(&self) -> &Relationships {
        &self.package_rels
    }

    /// Add a package-level relationship, returning its id.
    pub fn add_package_relationship(&mut self, rel_type: &str, target: &str) -> String {
        self.package_rels.add(rel_type, target)
    }

    /// A part's relationships, created empty when missing. Created through
    /// `Relationships::new`, which numbers from `rId1` as Word does; the
    /// dependency's `get_or_create_part_rels` defaults to `rId0`.
    fn part_rels_mut(&mut self, source_part: &str) -> &mut Relationships {
        use std::collections::hash_map::Entry;
        // An alias of a stored owner resolves to it rather than creating a
        // second rels entry of one identity (#600). Not `or_default()`:
        // the derived `Default` is the rId0 counter.
        let key = self
            .rels_index
            .get(&identity(source_part))
            .cloned()
            .unwrap_or_else(|| norm(source_part));
        match self.part_rels.entry(key.clone()) {
            Entry::Occupied(rels) => rels.into_mut(),
            Entry::Vacant(slot) => {
                self.rels_index.insert(identity(&key), key);
                slot.insert(Relationships::new())
            }
        }
    }

    /// Add a relationship with `TargetMode="External"` (absolute-URI targets
    /// are ILLEGAL for the default Internal mode — strict packaging layers
    /// like Word's reject the package without this).
    pub fn add_document_relationship_external(
        &mut self,
        source_part: &str,
        rel_type: &str,
        target: &str,
    ) -> String {
        let rels = self.part_rels_mut(source_part);
        let id = rels.add(rel_type, target);
        if let Some(r) = rels.items.iter_mut().find(|r| r.id == id) {
            r.target_mode = Some("External".to_string());
        }
        id
    }

    /// Mark an existing relationship of `source_part` as External (test aid).
    pub fn set_rel_target_mode_external(&mut self, source_part: &str, rel_id: &str) {
        let rels = self.part_rels_mut(source_part);
        if let Some(r) = rels.items.iter_mut().find(|r| r.id == rel_id) {
            r.target_mode = Some("External".to_string());
        }
    }

    /// The main document part name (docxodus style).
    pub fn main_document_part(&self) -> Option<String> {
        self.package_rels
            .get_by_type(oxml_opc::relationship::rel_types::DOCUMENT)
            .map(|rel| denorm(&rel.target))
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use crate::admission::zip_names;
    use zip::ZipArchive;

    #[test]
    fn opc_attribute_budget_bounds_upstream_duplicate_checks() {
        let attrs = |count| {
            (0..count)
                .map(|i| format!(" a{i}=\"x\""))
                .collect::<String>()
        };
        for close in ["/>", "></Relationship>"] {
            let at_limit = format!("<Relationship{}{close}", attrs(256));
            assert!(check_opc_attributes(at_limit.as_bytes()).is_ok());
            let over_limit = format!("<Relationship{}{close}", attrs(257));
            assert!(check_opc_attributes(over_limit.as_bytes()).is_err());
            let bytes = package_with_document_rels(&over_limit);
            assert!(PartFs::open(&bytes).is_err());
        }
        assert!(check_opc_attributes(b"<Relationship broken/>").is_err());
        assert!(check_opc_attributes(b"<Relationship").is_err());
        let bytes = package_with_document_rels(
            "<Relationship Id=\"rId1\" Type=\"test\" Target=\"a.xml\"/>",
        );
        assert!(PartFs::open(&bytes).is_ok());
        assert!(
            PartFs::open_within(
                &bytes,
                crate::admission::InputLimits {
                    max_part_bytes: 32,
                    ..crate::admission::InputLimits::compare()
                }
            )
            .is_err()
        );
        assert!(PartFs::open(b"not a zip").is_err());

        // Content types are also parsed by upstream, before relationships.
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        write!(writer, "<Types{}/>", attrs(257)).unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let err = PartFs::open(&bytes)
            .err()
            .expect("metadata attribute budget");
        assert!(err.to_string().contains("256 attributes"));
    }

    const RELS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    const HYPERLINK: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";

    #[test]
    fn public_and_package_relationship_readers_agree() {
        for namespace in [format!(r#" xmlns="{RELS_NS}""#), String::new()] {
            let document_type = oxml_opc::relationship::rel_types::DOCUMENT;
            let document_target = if namespace.is_empty() {
                "/word/document.xml"
            } else {
                "word/document.xml"
            };
            let xml = format!(
                r#"<Relationships{namespace}><Relationship Id="rId7" Type="{HYPERLINK}" Target="https://example.com/?a=1&amp;b=2" TargetMode="External"></Relationship><Relationship Id="rId2" Type="{document_type}" Target="{document_target}"/></Relationships>"#
            );
            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
            for (name, data) in [
                ("[Content_Types].xml", "<Types/>"),
                ("_rels/.rels", xml.as_str()),
                ("word/document.xml", "<document/>"),
                ("word/_rels/document.xml.rels", xml.as_str()),
            ] {
                writer
                    .start_file(name, SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(data.as_bytes()).unwrap();
            }
            let bytes = writer.finish().unwrap().into_inner();
            let fs = PartFs::open(&bytes).unwrap();
            let mut direct = Relationships::from_xml(xml.as_bytes()).unwrap();
            assert_eq!(direct.items.len(), 2);
            assert_eq!(direct.items, fs.package_relationships().items);
            for owner in ["word/document.xml", "/WORD/folder/../document.xml"] {
                assert_eq!(direct.items, fs.read_rels_for(owner).unwrap().items);
            }
            assert_eq!(
                fs.main_document_part().as_deref(),
                Some("word/document.xml")
            );
            assert_eq!(
                direct.to_xml().unwrap(),
                fs.read_rels_for("word/document.xml")
                    .unwrap()
                    .to_xml()
                    .unwrap()
            );
            assert_eq!(direct.add("image", "media/b.png"), "rId8");
        }
    }

    fn package_with_header_rels() -> PartFs {
        let mut buf = Vec::new();
        {
            let mut z = ZipWriter::new(Cursor::new(&mut buf));
            let opt = SimpleFileOptions::default();
            let parts: [(&str, String); 5] = [
                ("[Content_Types].xml", r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/></Types>"#.to_string()),
                ("_rels/.rels", format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#)),
                ("word/document.xml", "<w:document/>".to_string()),
                ("word/header1.xml", "<w:hdr/>".to_string()),
                ("word/_rels/header1.xml.rels", format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="{HYPERLINK}" Target="https://a.example/" TargetMode="External"/></Relationships>"#)),
            ];
            for (name, xml) in parts {
                z.start_file(name, opt).unwrap();
                z.write_all(xml.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        PartFs::open(&buf).unwrap()
    }

    /// A package whose `word/_rels/document.xml.rels` holds `rels`.
    fn package_with_document_rels(rels: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut z = ZipWriter::new(Cursor::new(&mut buf));
            let opt = SimpleFileOptions::default();
            let parts: [(&str, String); 4] = [
                ("[Content_Types].xml", r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/></Types>"#.to_string()),
                ("_rels/.rels", format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#)),
                ("word/document.xml", "<w:document/>".to_string()),
                ("word/_rels/document.xml.rels", format!(r#"<Relationships xmlns="{RELS_NS}">{rels}</Relationships>"#)),
            ];
            for (name, xml) in parts {
                z.start_file(name, opt).unwrap();
                z.write_all(xml.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf
    }

    /// Word refuses a package that repeats a relationship Id in one part
    /// (docxide case8's fontTable rels, opened in Word 16: unreadable).
    /// The same relationship twice is repaired by keeping one, and the
    /// repair is recorded for `validate`; two relationships under one Id
    /// are ambiguous and refused.
    #[test]
    fn a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise() {
        let font = |id: &str, target: &str| {
            format!(
                r#"<Relationship Id="{id}" Type="{HYPERLINK}" Target="{target}" TargetMode="External"/>"#
            )
        };
        let same = package_with_document_rels(&format!(
            "{}{}{}",
            font("rId1", "https://a.example/"),
            font("rId2", "https://b.example/"),
            font("rId1", "https://a.example/")
        ));
        let fs = PartFs::open(&same).unwrap();
        let rels = fs.read_rels_for("word/document.xml").unwrap();
        let ids: Vec<&str> = rels.items.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["rId1", "rId2"]);
        assert_eq!(
            fs.repaired_duplicate_ids(),
            [("word/document.xml".to_string(), "rId1".to_string())]
        );
        let conflict = package_with_document_rels(&format!(
            "{}{}",
            font("rId1", "https://a.example/"),
            font("rId1", "https://other.example/")
        ));
        assert!(matches!(
            PartFs::open(&conflict),
            Err(OpcError::InvalidRelationship)
        ));
        let clean = package_with_document_rels(&font("rId1", "https://a.example/"));
        assert!(
            PartFs::open(&clean)
                .unwrap()
                .repaired_duplicate_ids()
                .is_empty()
        );
    }

    #[test]
    fn the_package_is_written_in_the_source_order_then_new_parts_by_name() {
        // Parts and relationships live in hash maps, so the zip entries came
        // out in a different order on every run: same content, other bytes.
        let mut fs = package_with_header_rels();
        fs.set_part("word/b.xml", b"<b/>".to_vec());
        fs.set_part("word/a.xml", b"<a/>".to_vec());
        let zip = fs.to_zip().unwrap();
        let names: Vec<String> = ZipArchive::new(Cursor::new(&zip))
            .unwrap()
            .file_names()
            .map(|name| name.expect("decode ZIP fixture name"))
            .map(|name| name.into_owned())
            .collect();
        assert_eq!(
            names,
            [
                "[Content_Types].xml",
                "_rels/.rels",
                "word/document.xml",
                "word/header1.xml",
                "word/_rels/header1.xml.rels",
                "word/a.xml",
                "word/b.xml",
            ]
        );
        assert_eq!(fs.to_zip().unwrap(), zip);
    }

    #[test]
    fn entries_carry_no_wall_clock_time() {
        // Another dependency turns on zip's `time` feature, and with it the
        // default options stamp each entry with the current time: two writes
        // a second apart gave other bytes. Office writes 1980-01-01 00:00.
        let zip = package_with_header_rels().to_zip().unwrap();
        let mut archive = ZipArchive::new(Cursor::new(&zip)).unwrap();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            assert_eq!(
                entry.last_modified(),
                Some(zip::DateTime::default()),
                "{}",
                entry.name().expect("decode ZIP fixture name")
            );
        }
    }

    #[test]
    fn part_names_carry_exactly_one_leading_slash() {
        // `format!("/{part}")` on a part that already starts with '/' (an
        // absolute rel target) wrote a `//word/footer1.xml` content-type override;
        // OPC readers then refuse the whole package.
        let mut fs = package_with_header_rels();
        fs.add_content_type_override("//word/footer1.xml", "application/xml");
        fs.set_part("//word/footer1.xml", b"<w:ftr/>".to_vec());
        assert_eq!(
            fs.content_type_for("word/footer1.xml").as_deref(),
            Some("application/xml")
        );
        assert!(fs.parts().contains(&"word/footer1.xml".to_string()));
        let zip = fs.to_zip().unwrap();
        let ct = String::from_utf8(
            PartFs::open(&zip)
                .unwrap()
                .pkg
                .content_types
                .to_xml()
                .unwrap(),
        )
        .unwrap();
        assert!(!ct.contains("//word"), "{ct}");
    }

    #[test]
    fn parts_without_a_content_type_adopt_the_sources_default() {
        // B's header pulls in `media/hdphoto1.wdp` (HD Photo); the copy kept the
        // bytes but not B's `wdp` Default, and a part with no content type makes
        // Word refuse the package.
        let mut src = package_with_header_rels();
        src.add_content_type_default("wdp", "image/vnd.ms-photo");
        src.add_content_type_default("PNG", "image/png");
        let mut out = package_with_header_rels();
        out.set_part("word/media/hdphoto1.wdp", vec![1, 2, 3]);
        out.set_part("word/media/pic.png", vec![4]);
        out.set_part("word/media/unknown.zzz", vec![5]);
        out.adopt_missing_content_types(&[&src]);
        assert_eq!(
            out.content_type_for("word/media/hdphoto1.wdp").as_deref(),
            Some("image/vnd.ms-photo")
        );
        // Extension matching is case-insensitive in OPC.
        assert_eq!(
            out.content_type_for("word/media/pic.png").as_deref(),
            Some("image/png")
        );
        // Nothing to adopt: left alone rather than guessed.
        assert_eq!(out.content_type_for("word/media/unknown.zzz"), None);
        // A part with no extension has no Default to ask for.
        out.set_part("word/media/LICENSE", vec![6]);
        out.adopt_missing_content_types(&[&src]);
        assert_eq!(out.content_type_for("word/media/LICENSE"), None);
        // A part the source names in an Override adopts that Override.
        let mut typed = package_with_header_rels();
        typed.add_content_type_override("word/media/logo.bin", "image/x-logo");
        let mut out = package_with_header_rels();
        out.set_part("word/media/logo.bin", vec![7]);
        out.adopt_missing_content_types(&[&typed]);
        assert_eq!(
            out.content_type_for("word/media/logo.bin").as_deref(),
            Some("image/x-logo")
        );
    }

    #[test]
    fn an_upper_case_extension_is_already_typed_by_the_lower_case_default() {
        // `image3.PNG` under our own `png` Default is typed (OPC matches
        // extensions case-insensitively). Adding a second `PNG` Default made the
        // package unopenable (647bbcfb, 2026-09-26 English redlines).
        let mut out = package_with_header_rels();
        out.add_content_type_default("png", "image/png");
        out.set_part("word/media/image3.PNG", vec![4]);
        out.adopt_missing_content_types(&[]);
        let pngs = out
            .pkg
            .content_types
            .defaults
            .iter()
            .filter(|(e, _)| e.eq_ignore_ascii_case("png"))
            .count();
        assert_eq!(pngs, 1);
    }

    #[test]
    fn migration_keeps_canonical_metadata_and_decodes_literal_entities_once() {
        let bytes = package_with_document_rels(
            r#"<Relationship Id="rId7" Type="image" Target="media/a&amp;amp;b.png"/>"#,
        );
        let mut fs = PartFs::open(&bytes).unwrap();
        for _ in 0..3 {
            assert_eq!(
                fs.read_rels_for("word/document.xml").unwrap().items[0].target,
                "media/a&amp;b.png"
            );
            let bytes = fs.to_zip().unwrap();
            let mut archive = ZipArchive::new(Cursor::new(&bytes)).unwrap();
            let mut xml = String::new();
            archive
                .by_name("word/_rels/document.xml.rels")
                .unwrap()
                .read_to_string(&mut xml)
                .unwrap();
            assert_eq!(
                xml,
                concat!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
                    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n",
                    "  <Relationship Id=\"rId7\" Type=\"image\" Target=\"media/a&amp;amp;b.png\"/>\n",
                    "</Relationships>"
                )
            );
            fs = PartFs::open(&bytes).unwrap();
        }
        assert_eq!(
            fs.add_document_relationship("word/document.xml", "image", "next.png"),
            "rId8"
        );
    }

    #[test]
    fn rel_attributes_survive_round_trips_unescaped() {
        // rdocx-opc 0.1.0 keeps `Target="a&amp;b"` escaped when parsing and
        // escapes again when writing: every open/write added one `&amp;`, and
        // `image11.jpg&ehk=…` ended up pointing at `image11.jpg&amp;amp;amp;…`,
        // a part that does not exist (a2412654, 2026-09-26 English redlines).
        let mut fs = package_with_header_rels();
        fs.set_part(
            "word/_rels/document.xml.rels",
            format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/p.jpg&amp;ehk=Q"/></Relationships>"#).into_bytes(),
        );
        let target = |fs: &PartFs| {
            fs.read_rels_for("word/document.xml").unwrap().items[0]
                .target
                .clone()
        };
        assert_eq!(target(&fs), "media/p.jpg&ehk=Q");
        let once = PartFs::open(&fs.to_zip().unwrap()).unwrap();
        let twice = PartFs::open(&once.to_zip().unwrap()).unwrap();
        assert_eq!(target(&twice), "media/p.jpg&ehk=Q");
    }

    #[test]
    fn content_types_survive_round_trips_unescaped() {
        // Same rdocx-opc escaping gap as the relationships: a2412654's
        // `Default Extension="jpg&amp;ehk=…"` grew one `&amp;` per cycle and
        // stopped typing its picture.
        let mut fs = package_with_header_rels();
        fs.add_content_type_default("jpg&ehk=Q", "image/jpeg");
        fs.set_part("word/media/p.jpg&ehk=Q", vec![1]);
        let once = PartFs::open(&fs.to_zip().unwrap()).unwrap();
        let twice = PartFs::open(&once.to_zip().unwrap()).unwrap();
        assert_eq!(
            twice.content_type_for("word/media/p.jpg&ehk=Q").as_deref(),
            Some("image/jpeg")
        );
    }

    #[test]
    fn rel_targets_resolve_to_canonical_part_names() {
        let fs = package_with_header_rels();
        // An absolute target once came back as "/word/footer1.xml" and was then
        // re-prefixed into a "//word/footer1.xml" relationship (package refused).
        assert_eq!(
            fs.resolve_rel_target("word/document.xml", "/word/footer1.xml"),
            "word/footer1.xml"
        );
        assert_eq!(
            fs.resolve_rel_target("word/document.xml", "media/a.png"),
            "word/media/a.png"
        );
        assert_eq!(
            fs.resolve_rel_target("word/document.xml", "../customXml/item1.xml"),
            "customXml/item1.xml"
        );
        assert_eq!(
            fs.resolve_rel_target("word/header1.xml", "./media/a.png"),
            "word/media/a.png"
        );
        assert_eq!(
            fs.resolve_rel_target("/word/document.xml", "styles.xml"),
            "word/styles.xml"
        );
    }

    #[test]
    fn rel_targets_are_relative_to_the_source_folder() {
        assert_eq!(
            relative_rel_target("word/header1.xml", "word/media/P1.png"),
            "media/P1.png"
        );
        assert_eq!(
            relative_rel_target("/word/document.xml", "/word/comments.xml"),
            "comments.xml"
        );
        assert_eq!(
            relative_rel_target("word/header1.xml", "customXml/item1.xml"),
            "/customXml/item1.xml"
        );
        assert_eq!(
            relative_rel_target("word/document.xml", "media/image.bin"),
            "/media/image.bin"
        );
        // `word/document.xml` is not a folder prefix of `wordy/x.xml`.
        assert_eq!(
            relative_rel_target("word/document.xml", "wordy/x.xml"),
            "/wordy/x.xml"
        );

        // Edge cases
        assert_eq!(
            relative_rel_target("a.xml", "b.xml"),
            "/b.xml",
            "Root level source and target"
        );
        assert_eq!(
            relative_rel_target("document.xml", "media/image.bin"),
            "/media/image.bin",
            "Root level source"
        );
        assert_eq!(
            relative_rel_target("word/theme/theme1.xml", "word/theme/themeManager.xml"),
            "themeManager.xml",
            "Deeply nested matching paths"
        );
        assert_eq!(
            relative_rel_target("//word/document.xml", "//word/comments.xml"),
            "comments.xml",
            "Multiple leading slashes"
        );
        assert_eq!(
            relative_rel_target("a/b.xml", "a/c/d.xml"),
            "c/d.xml",
            "Target as a subdirectory of the source directory"
        );
        assert_eq!(
            relative_rel_target("a/b/c.xml", "a/b/d.xml"),
            "d.xml",
            "Target and source in the same deeply nested directory"
        );
        assert_eq!(
            relative_rel_target("a/b.xml", "b/c.xml"),
            "/b/c.xml",
            "Target in completely different directory"
        );
    }

    #[test]
    fn rels_paths_name_their_owning_part() {
        assert_eq!(
            rels_path_to_part_name("word/_rels/header1.xml.rels").as_deref(),
            Some("word/header1.xml")
        );
        assert_eq!(
            rels_path_to_part_name("/word/_rels/document.xml.rels").as_deref(),
            Some("word/document.xml")
        );
        assert_eq!(rels_path_to_part_name("_rels/.rels").as_deref(), Some(""));
        assert_eq!(rels_path_to_part_name("word/header1.xml"), None);
        assert_eq!(rels_path_to_part_name("word/media/odd.rels"), None);
    }

    /// A part's first relationship is `rId1`, as Word numbers them: the
    /// dependency's `Default` relationships start from `rId0`.
    #[test]
    fn a_new_rels_part_starts_at_rid1() {
        let mut fs = package_with_header_rels();
        let id = fs.add_document_relationship(
            "word/numbering.xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
            "media/image1.gif",
        );
        assert_eq!(id, "rId1");
        let external = fs.add_document_relationship_external(
            "word/footer9.xml",
            HYPERLINK,
            "https://c.example/",
        );
        assert_eq!(external, "rId1");
    }

    #[test]
    fn writing_a_rels_part_replaces_the_parts_relationships() {
        let mut fs = package_with_header_rels();
        fs.set_part(
            "word/_rels/header1.xml.rels",
            format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId7" Type="{HYPERLINK}" Target="https://b.example/" TargetMode="External"/></Relationships>"#).into_bytes(),
        );
        let ids: Vec<String> = fs
            .read_rels_for("word/header1.xml")
            .unwrap()
            .items
            .iter()
            .map(|r| r.id.clone())
            .collect();
        assert_eq!(ids, ["rId7"]);
        // One .rels entry per part: a raw copy next to the parsed one is a
        // duplicate zip name and fails the whole package.
        let reopened = PartFs::open(&fs.to_zip().expect("no duplicate .rels")).unwrap();
        assert_eq!(
            reopened
                .read_rels_for("word/header1.xml")
                .unwrap()
                .items
                .len(),
            1
        );
    }

    /// Dropping a header takes the media only it used with it; media another
    /// part still uses stays, and so does a part reached in a loop.
    #[test]
    fn removing_a_related_part_drops_the_parts_only_it_targeted() {
        const HEADER: &str =
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
        const IMAGE: &str =
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
        let rel = |id: &str, ty: &str, target: &str| {
            format!(r#"<Relationship Id="{id}" Type="{ty}" Target="{target}"/>"#)
        };
        let mut fs = PartFs::open(&package_with_document_rels(&format!(
            "{}{}",
            rel("rId1", HEADER, "header1.xml"),
            rel("rId2", HEADER, "header2.xml")
        )))
        .unwrap();
        for part in ["header1.xml", "header2.xml"] {
            fs.set_part(&format!("word/{part}"), b"<w:hdr/>".to_vec());
        }
        for media in ["logo.png", "shared.png"] {
            fs.set_part(&format!("word/media/{media}"), vec![0x89]);
        }
        fs.set_part(
            "word/_rels/header1.xml.rels",
            format!(
                r#"<Relationships xmlns="{RELS_NS}">{}{}{}</Relationships>"#,
                rel("rId1", IMAGE, "media/logo.png"),
                rel("rId2", IMAGE, "media/shared.png"),
                rel("rId3", HEADER, "header1.xml")
            )
            .into_bytes(),
        );
        fs.set_part(
            "word/_rels/header2.xml.rels",
            format!(
                r#"<Relationships xmlns="{RELS_NS}">{}</Relationships>"#,
                rel("rId1", IMAGE, "media/shared.png")
            )
            .into_bytes(),
        );

        fs.remove_related_part("word/document.xml", "rId1");

        let parts = fs.parts();
        let has = |name: &str| parts.iter().any(|p| p == name);
        assert!(!has("word/header1.xml"), "{parts:?}");
        assert!(
            !has("word/media/logo.png"),
            "only header1 used it: {parts:?}"
        );
        assert!(has("word/media/shared.png"), "header2 still uses it");
        assert!(has("word/header2.xml"));
        assert!(fs.read_rels_for("word/header1.xml").is_none());
        PartFs::open(&fs.to_zip().unwrap()).unwrap();
    }

    #[test]
    fn migration_skips_zip_directories_and_keeps_a_missing_package_relationship_scope_empty() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        writer.add_directory("word/", options).unwrap();
        writer.start_file("[Content_Types].xml", options).unwrap();
        writer.write_all(br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#).unwrap();
        writer.start_file("word/document.xml", options).unwrap();
        writer.write_all(b"<document/>").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let package = PartFs::open(&bytes).unwrap();
        assert_eq!(package.parts(), ["word/document.xml"]);
        assert!(package.package_relationships().items.is_empty());
        assert_eq!(package.main_document_part(), None);
        let reopened = PartFs::open(&package.to_zip().unwrap()).unwrap();
        assert_eq!(reopened.parts(), package.parts());
        assert!(reopened.package_relationships().items.is_empty());
        assert_eq!(
            reopened.part_bytes("word/document.xml"),
            Some(b"<document/>".as_slice())
        );
    }

    #[test]
    fn migration_refuses_missing_required_metadata_attributes_in_package_loading() {
        let bytes = package_with_document_rels("<Relationship Id='rId1'/>");
        assert!(matches!(
            PartFs::open(&bytes),
            Err(OpcError::InvalidRelationship)
        ));
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml"/></Types>"#).unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert!(matches!(
            PartFs::open(&bytes),
            Err(OpcError::InvalidContentTypes)
        ));
    }

    #[test]
    fn a_record_local_default_namespace_cannot_bypass_metadata_validation() {
        // `<Default xmlns=""/>` unbinds the OPC namespace the reader wraps
        // records in, so upstream skipped the record and the package lost its
        // missing-attribute refusal; the same shape reaches `.rels` parts.
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        writer
            .write_all(
                br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default xmlns="" Extension="xml"/></Types>"#,
            )
            .unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert!(matches!(
            PartFs::open(&bytes),
            Err(OpcError::InvalidContentTypes)
        ));
        let bytes = package_with_document_rels("<Relationship xmlns='' Id='rId1'/>");
        assert!(matches!(
            PartFs::open(&bytes),
            Err(OpcError::InvalidRelationship)
        ));
    }

    /// A minimal package whose one media entry carries the raw name bytes and
    /// flag `media` names, with the document relationship pointing at the
    /// CP437 spelling of `\xc3\xa9` (`├⌐`), as a legacy writer left it.
    fn legacy_media_package(media: zip_names::RawEntry<'_>) -> Vec<u8> {
        let rels = format!(
            "<Relationship Id=\"rId1\" Type=\"{HYPERLINK}\" Target=\"media/\u{251C}\u{2310}.png\"/>"
        );
        zip_names::docx_package(&rels, &[media])
    }

    #[test]
    fn legacy_cp437_entry_names_keep_resolving_their_relationships() {
        // zip 8 decoded a flag-clear name as CP437 (zip-8.6.0/src/read.rs
        // 523-529: bit 11 set means UTF-8, clear means CP437); zip 9's
        // `name()` tries UTF-8 first whatever the flag says
        // (zip-9.0.4/src/types.rs:122-130), so `word/media/\xc3\xa9.png`
        // became `word/media/é.png` while the relationship still named
        // `media/├⌐.png`, and the media lookup (the one convert uses) came
        // back empty (#524: no read/write regressions).
        let bytes = legacy_media_package(zip_names::RawEntry::raw(
            b"word/media/\xc3\xa9.png",
            b"PNGPAYLOAD",
        ));
        let fs = PartFs::open(&bytes).unwrap();
        let rel = &fs.read_rels_for("word/document.xml").unwrap().items[0];
        assert_eq!(rel.target, "media/\u{251C}\u{2310}.png");
        let part = fs.resolve_rel_target("word/document.xml", &rel.target);
        assert_eq!(part, "word/media/\u{251C}\u{2310}.png");
        assert_eq!(fs.part_bytes(&part), Some(b"PNGPAYLOAD".as_slice()));
        // The UTF-8 spelling of the same bytes is not a part of this package.
        assert_eq!(fs.part_bytes("word/media/\u{e9}.png"), None);
    }

    #[test]
    fn decoded_part_name_aliases_are_refused_not_overwritten() {
        // Two entries whose decoded names collide are one OPC part: the second
        // `parts.insert` used to overwrite the first silently, `parts()` lost
        // a member and `to_zip()` dropped it. Upstream's reader refuses
        // `DuplicatePartName` on the second entry's name
        // (oxml-opc-0.14.2/src/package.rs 159-162); so does the constructor —
        // the raw central-directory pass reports the defect for `admit`,
        // `debug` and the archive readers while the constructor keeps the
        // typed variant Rust callers match on (#600).
        let base = |extra: &[zip_names::RawEntry<'_>]| zip_names::docx_package("", extra);
        // (a) two entries of the same name, byte for byte (the p1 shape):
        // one part, the refusal names the second entry's spelling.
        let exact = base(&[
            zip_names::RawEntry::raw(b"x.bin", b"FIRST"),
            zip_names::RawEntry::raw(b"x.bin", b"SECOND"),
        ]);
        assert!(
            matches!(PartFs::open(&exact), Err(OpcError::DuplicatePartName(ref n)) if n == "x.bin"),
            "{:?}",
            PartFs::open(&exact).err()
        );
        assert!(
            matches!(
                PartFs::open_within(&exact, crate::admission::InputLimits::compare()),
                Err(OpcError::DuplicatePartName(ref n)) if n == "x.bin"
            ),
            "open_within refuses the same"
        );
        // (b) two raw names, bit 11 set, whose lossy UTF-8 decode is the same
        // `x\u{FFFD}.bin`: two parts, one name.
        let aliased = base(&[
            zip_names::RawEntry::raw(b"x\x82.bin", b"FIRST").utf8(),
            zip_names::RawEntry::raw(b"x\x83.bin", b"SECOND").utf8(),
        ]);
        assert!(
            matches!(PartFs::open(&aliased), Err(OpcError::DuplicatePartName(ref n)) if n == "x\u{FFFD}.bin"),
            "{:?}",
            PartFs::open(&aliased).err()
        );
        // (c) two names that differ only in ASCII case: one part, OPC names
        // are case-insensitive.
        let cased = base(&[
            zip_names::RawEntry::raw(b"word/A.bin", b"FIRST"),
            zip_names::RawEntry::raw(b"word/a.bin", b"SECOND"),
        ]);
        assert!(
            matches!(PartFs::open(&cased), Err(OpcError::DuplicatePartName(ref n)) if n == "word/a.bin"),
            "{:?}",
            PartFs::open(&cased).err()
        );
        // (d) the zip 8 shape: both names flag-clear, CP437 decodes them
        // apart (`x\x82.bin` → `xé.bin`, `x\xc3\xa9.bin` → `x├⌐.bin`), and
        // both payloads survive instead of aliasing as zip 9's UTF-8-first
        // `name()` had them.
        let apart = base(&[
            zip_names::RawEntry::raw(b"x\x82.bin", b"FIRST"),
            zip_names::RawEntry::raw(b"x\xc3\xa9.bin", b"SECOND"),
        ]);
        let fs = PartFs::open(&apart).unwrap();
        assert_eq!(fs.part_bytes("x\u{e9}.bin"), Some(b"FIRST".as_slice()));
        assert_eq!(
            fs.part_bytes("x\u{251C}\u{2310}.bin"),
            Some(b"SECOND".as_slice())
        );
    }
    #[test]
    fn unicode_path_extra_fields_override_entry_names_as_zip8_did() {
        // zip 8 replaced the header name with the 0x7075 field's CRC-checked
        // UTF-8 content (zip-8.6.0/src/read.rs:705-709) and refused the
        // archive when the content was not UTF-8; zip 9 applies the valid
        // case itself and ignores the invalid one, so the readers refuse it.
        let bytes = legacy_media_package(
            zip_names::RawEntry::raw(b"word/media/plain.bin", b"BINPAYLOAD")
                .unicode_path("word/media/\u{2714}.bin".as_bytes()),
        );
        let fs = PartFs::open(&bytes).unwrap();
        assert!(fs.parts().contains(&"word/media/\u{2714}.bin".to_string()));
        assert_eq!(
            fs.part_bytes("word/media/\u{2714}.bin"),
            Some(b"BINPAYLOAD".as_slice())
        );

        let bytes = legacy_media_package(
            zip_names::RawEntry::raw(b"word/media/plain.bin", b"BINPAYLOAD")
                .unicode_path(b"\xff\xfe"),
        );
        assert!(matches!(PartFs::open(&bytes), Err(OpcError::Zip(_))));
    }

    /// A package with caller-chosen entry names — case-variant spellings
    /// of the reserved parts and the main part (#600 probes p3–p5).
    fn variant_package(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
        zip_names::zip(
            &entries
                .iter()
                .map(|(name, data)| zip_names::RawEntry::raw(name, data))
                .collect::<Vec<_>>(),
        )
    }

    const TYPES: &[u8] = br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    const PKG_RELS: &[u8] = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

    /// ZIP-15 (#600): OPC part names ignore ASCII case, so a single
    /// case-variant name is the part — Word opens these packages (probes
    /// p3, p4, p5) and every lookup must resolve the variant's content.
    #[test]
    fn a_case_variant_name_is_the_part_on_every_lookup() {
        let fs = PartFs::open(&variant_package(&[
            (b"[Content_Types].xml", TYPES),
            (b"_rels/.rels", PKG_RELS),
            (b"word/Document.xml", b"<w:document/>"),
        ]))
        .expect("the case-variant main part is the part the rels target names (p3)");
        assert_eq!(
            fs.main_document_part().as_deref(),
            Some("word/document.xml")
        );
        assert_eq!(
            fs.part_bytes("word/document.xml"),
            Some(b"<w:document/>".as_slice())
        );
        assert_eq!(
            fs.content_type_for("word/document.xml").as_deref(),
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
            ),
            "the Override names /word/document.xml and must match by identity"
        );

        let fs = PartFs::open(&variant_package(&[
            (b"[content_types].xml", TYPES),
            (b"_rels/.rels", PKG_RELS),
            (b"word/document.xml", b"<w:document/>"),
        ]))
        .expect("the lower-case [Content_Types].xml is the content types (p4)");
        assert_eq!(
            fs.content_type_for("word/document.xml").as_deref(),
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
            )
        );

        let fs = PartFs::open(&variant_package(&[
            (b"[Content_Types].xml", TYPES),
            (b"_RELS/.RELS", PKG_RELS),
            (b"word/document.xml", b"<w:document/>"),
        ]))
        .expect("the upper-case _RELS/.RELS is the package relationships (p5)");
        assert_eq!(fs.package_relationships().items.len(), 1);
        assert_eq!(
            fs.main_document_part().as_deref(),
            Some("word/document.xml")
        );
    }

    /// `to_zip` of a package opened from case-variant spellings writes
    /// exactly one entry per identity: the reserved parts come out with
    /// their canonical names, an ordinary part keeps the spelling it was
    /// read with (#600).
    #[test]
    fn to_zip_writes_one_entry_per_identity() {
        let fs = PartFs::open(&variant_package(&[
            (b"[content_types].xml", TYPES),
            (b"_RELS/.RELS", PKG_RELS),
            (b"word/Document.xml", b"<w:document/>"),
        ]))
        .expect("case variants are the parts");
        let bytes = fs.to_zip().expect("one entry per identity serializes");
        let mut names: Vec<String> = ZipArchive::new(Cursor::new(&bytes))
            .unwrap()
            .file_names()
            .map(|name| name.expect("decode ZIP fixture name").into_owned())
            .collect();
        names.sort();
        assert_eq!(
            names,
            ["[Content_Types].xml", "_rels/.rels", "word/Document.xml"]
        );
        // … which the same open reads back under one identity.
        assert_eq!(
            PartFs::open(&bytes)
                .unwrap()
                .part_bytes("word/document.xml"),
            Some(b"<w:document/>".as_slice())
        );
    }

    /// Mutation keeps one part per identity: `set_part` with a case alias
    /// replaces the stored part and keeps its spelling; `remove_part`
    /// with an alias removes it; a `.rels` alias routes to the owner's
    /// parsed relationships (#600).
    #[test]
    fn mutations_resolve_under_the_one_identity() {
        let mut fs = package_with_header_rels();
        fs.set_part("WORD/DOCUMENT.XML", b"replaced".to_vec());
        assert_eq!(fs.parts(), ["word/document.xml", "word/header1.xml"]);
        assert_eq!(
            fs.part_bytes("word/document.xml"),
            Some(b"replaced".as_slice()),
            "the alias replaced the stored part and kept its spelling"
        );
        // `_RELS/.RELS` is the package's own relationships.
        fs.set_part(
            "_RELS/.RELS",
            format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId9" Type="{HYPERLINK}" Target="https://c.example/" TargetMode="External"/></Relationships>"#).into_bytes(),
        );
        assert_eq!(fs.package_relationships().items.len(), 1);
        assert_eq!(fs.package_relationships().items[0].id, "rId9");
        // A rels alias replaces the owner's parsed relationships.
        fs.set_part(
            "word/_RELS/HEADER1.xml.RELS",
            format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId3" Type="{HYPERLINK}" Target="https://d.example/" TargetMode="External"/></Relationships>"#).into_bytes(),
        );
        let rels = fs
            .read_rels_for("WORD/header1.XML")
            .expect("owner resolves by identity");
        assert_eq!(rels.items.len(), 1);
        assert_eq!(rels.items[0].id, "rId3");
        // A `.rels` name for the package itself (any case) removes the
        // package's own relationships.
        fs.remove_part("_RELS/.rels");
        assert!(fs.package_relationships().items.is_empty());
        fs.remove_part("WORD/HEADER1.XML");
        assert_eq!(fs.parts(), ["word/document.xml"]);
        fs.remove_part("word/document.XML");
        assert!(fs.parts().is_empty());
    }

    /// `set_part` with a `.rels` name whose XML does not parse keeps the
    /// raw part, and a later parseable `.rels` supersedes it — `to_zip`
    /// writes the parsed copy and no second entry of the one identity
    /// (#600).
    #[test]
    fn a_raw_rels_part_is_superseded_by_parsed_relationships() {
        let mut fs = package_with_header_rels();
        fs.set_part("word/_rels/extra.xml.rels", b"<broken".to_vec());
        assert_eq!(
            fs.part_bytes("word/_rels/extra.xml.rels"),
            Some(b"<broken".as_slice()),
            "unparseable relationship XML is kept raw"
        );
        fs.set_part(
            "word/_rels/extra.xml.rels",
            format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId5" Type="{HYPERLINK}" Target="https://e.example/" TargetMode="External"/></Relationships>"#).into_bytes(),
        );
        assert_eq!(
            fs.part_bytes("word/_rels/extra.xml.rels"),
            None,
            "the parsed copy supersedes the raw part"
        );
        assert_eq!(
            fs.read_rels_for("word/extra.xml").unwrap().items[0].id,
            "rId5"
        );
        fs.to_zip()
            .expect("one entry of the rels identity serializes");
    }

    /// The identity a part or owner name resolves to: the part a
    /// relationship target names (leading slashes and `.`/`..` segments
    /// resolved), compared under ASCII case (#600).
    #[test]
    fn identity_resolves_like_a_relationship_target() {
        assert_eq!(identity("word/document.xml"), "word/document.xml");
        assert_eq!(identity("/word/Document.xml"), "word/document.xml");
        assert_eq!(identity("WORD/document.xml"), "word/document.xml");
        assert_eq!(identity("word/./document.xml"), "word/document.xml");
        assert_eq!(identity("word/../word/document.xml"), "word/document.xml");
        assert_eq!(identity(""), "");
    }

    /// `to_zip` must not write two entries of one identity: a raw part
    /// whose name aliases a rels path (or the reserved names) is
    /// `DUPLICATE_PART`, not a broken archive (#600).
    #[test]
    fn to_zip_refuses_an_identity_collision() {
        let mut fs = package_with_header_rels();
        fs.pkg.parts.insert(
            "word/_rels/header1.xml.rels".to_string(),
            Arc::new(b"raw".to_vec()),
        );
        let err = fs
            .to_zip()
            .expect_err("the raw copy collides with the parsed one");
        let text = err.to_string();
        assert!(text.contains("DUPLICATE_PART"), "{text}");
        let mut fs = package_with_header_rels();
        fs.pkg
            .parts
            .insert("[CONTENT_TYPES].XML".to_string(), Arc::new(b"raw".to_vec()));
        let err = fs
            .to_zip()
            .expect_err("the raw copy collides with the reserved part");
        assert!(err.to_string().contains("DUPLICATE_PART"), "{err}");
    }

    /// A `.rels` entry that is not a `…/_rels/….rels` path still parses:
    /// its owner is the part the name's stem names, and `to_zip` writes it
    /// back beside that root part (#600 identity coverage).
    #[test]
    fn a_root_rels_part_belongs_to_its_stem() {
        let rels = format!(
            r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="{HYPERLINK}" Target="https://f.example/" TargetMode="External"/></Relationships>"#
        );
        let bytes = variant_package(&[
            (b"[Content_Types].xml", TYPES),
            (b"_rels/.rels", PKG_RELS),
            (b"word/document.xml", b"<w:document/>"),
            (b"x.rels", rels.as_bytes()),
        ]);
        let fs = PartFs::open(&bytes).expect("a root .rels part opens");
        // It is relationships storage, not a part of its own.
        assert!(!fs.parts().iter().any(|p| p == "x.rels"));
        assert_eq!(fs.read_rels_for("x").unwrap().items[0].id, "rId1");
    }

    /// A package without `[Content_Types].xml` names the reserved part it
    /// is missing, under any spelling the entry could have carried.
    #[test]
    fn a_package_without_content_types_is_part_not_found() {
        let bytes = variant_package(&[
            (b"_rels/.rels", PKG_RELS),
            (b"word/document.xml", b"<w:document/>"),
        ]);
        assert!(matches!(
            PartFs::open(&bytes),
            Err(OpcError::PartNotFound(name)) if name == "[Content_Types].xml"
        ));
    }

    /// Mutations resolve under the one OPC identity everywhere: a `.rels`
    /// name drops its owner's parsed relationships, the
    /// `[Content_Types].xml` identity clears the content types, a raw
    /// part of a reserved identity is superseded by a parsed one, the
    /// relationship helpers no-op on absent owners and unknown ids, and
    /// a root part's relationships serialize as `_rels/<part>.rels`.
    #[test]
    fn helper_mutations_resolve_under_the_same_index() {
        let mut fs = package_with_header_rels();
        fs.remove_part("word/_rels/header1.xml.rels");
        assert!(fs.read_rels_for("word/header1.xml").is_none());
        fs.remove_part("[CONTENT_TYPES].XML");
        assert_eq!(fs.content_type_for("word/document.xml"), None);
        // Absent owners and unknown or external ids are no-ops.
        assert!(fs.read_rels_for("word/styles.xml").is_none());
        fs.remove_relationships_by_type("word/styles.xml", HYPERLINK);
        fs.remove_related_part("word/styles.xml", "rId1");
        fs.remove_related_part("word/document.xml", "rId404");
        fs.set_rel_target_mode_external("word/document.xml", "rId9");
        // Removing by type drops matching items of a part's rels or the
        // package's own.
        assert_eq!(
            fs.add_package_relationship(HYPERLINK, "https://g.example/"),
            "rId2"
        );
        fs.add_document_relationship_external("word/document.xml", HYPERLINK, "https://h.example/");
        fs.set_rel_target_mode_external("word/document.xml", "rId1");
        fs.set_rel_target_mode_external("word/document.xml", "rId77");
        fs.remove_package_relationships_by_type(HYPERLINK);
        assert!(
            fs.package_relationships()
                .items
                .iter()
                .all(|r| r.rel_type != HYPERLINK)
        );
        fs.remove_relationships_by_type("word/document.xml", HYPERLINK);
        assert!(
            fs.read_rels_for("word/document.xml")
                .unwrap()
                .items
                .iter()
                .all(|r| r.rel_type != HYPERLINK)
        );
        // An absent id inside present rels is a no-op; an internal
        // relationship's removal drops the part it alone targeted.
        fs.remove_related_part("word/document.xml", "rId404");
        fs.set_part("word/media/bye.png", vec![9]);
        let drop_id = fs.add_document_relationship("word/document.xml", "image", "media/bye.png");
        fs.remove_related_part("word/document.xml", &drop_id);
        assert_eq!(fs.part_bytes("word/media/bye.png"), None);
        // A `.rels` path whose owner has no rels is a no-op, as is a
        // parseable [Content_Types].xml over none raw part.
        fs.remove_part("word/_rels/styles.xml.rels");
        fs.set_part("[Content_Types].xml", TYPES.to_vec());
        // A raw [Content_Types].xml part is superseded by a parseable one.
        fs.set_part("[Content_Types].xml", b"<broken".to_vec());
        assert!(fs.part_bytes("[content_types].xml").is_some());
        fs.set_part("[content_types].xml", TYPES.to_vec());
        assert_eq!(
            fs.part_bytes("[Content_Types].xml"),
            None,
            "the parsed content types supersede the raw part"
        );
        // Root-part relationships serialize as `_rels/<part>.rels`.
        fs.set_part("x.xml", b"<x/>".to_vec());
        fs.add_document_relationship("x.xml", HYPERLINK, "https://i.example/");
        let mut names: Vec<String> = ZipArchive::new(Cursor::new(&fs.to_zip().unwrap()))
            .unwrap()
            .file_names()
            .map(|name| name.expect("decode ZIP fixture name").into_owned())
            .collect();
        names.sort();
        assert!(names.contains(&"_rels/x.xml.rels".to_string()), "{names:?}");
    }

    /// A relationship Id repeated three or more times still repairs to
    /// one entry and reports the Id once, in the package's own rels as in
    /// a part's (#600 coverage of the package-level repair and the
    /// recorded-ids dedup).
    #[test]
    fn a_relationship_repeated_more_than_twice_is_repaired_once() {
        let rels = format!(
            r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="{HYPERLINK}" Target="https://j.example/" TargetMode="External"/><Relationship Id="rId2" Type="{HYPERLINK}" Target="https://j.example/" TargetMode="External"/><Relationship Id="rId2" Type="{HYPERLINK}" Target="https://j.example/" TargetMode="External"/></Relationships>"#
        );
        let bytes = variant_package(&[
            (b"[Content_Types].xml", TYPES),
            (b"_rels/.rels", rels.as_bytes()),
            (b"word/document.xml", b"<w:document/>"),
        ]);
        let fs = PartFs::open(&bytes).expect("identical repeats repair");
        assert_eq!(fs.package_relationships().items.len(), 2);
        assert_eq!(
            fs.repaired_duplicate_ids(),
            [
                (String::new(), "rId1".to_string()),
                (String::new(), "rId2".to_string())
            ]
        );
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod bounded_input_tests {
    use super::*;
    use crate::admission::{InputLimits, limit_test_support as fixture};

    fn refuses(limits: InputLimits, detail: &str) {
        let bytes = fixture::package();
        let text = PartFs::open_within(&bytes, limits)
            .err()
            .expect("over-budget package must be refused")
            .to_string();
        assert!(text.contains("INPUT_LIMIT"), "{text}");
        assert!(text.contains(detail), "{text}");
    }
    #[test]
    fn compressed_limit() {
        refuses(fixture::compressed(), "input is");
    }
    #[test]
    fn count_limit() {
        refuses(fixture::count(), "3 ZIP entries; the limit is 2");
    }
    #[test]
    fn part_limit() {
        refuses(fixture::part(), "a.bin");
    }
    #[test]
    fn aggregate_limit() {
        refuses(fixture::total(), "the package limit is 160");
    }
    #[test]
    fn exact_budget_control() {
        let bytes = fixture::package();
        assert!(PartFs::open_within(&bytes, fixture::limits()).is_ok());
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod zip32_tests {
    use super::*;
    use zip::ZipArchive;

    /// `(name, local version-needed, central version-needed, local extra
    /// length, central extra length)` for every entry, read from the raw
    /// headers — the local field at +4 (APPNOTE 4.3.7), the central one at
    /// +6 (APPNOTE 4.3.12).
    fn header_versions(bytes: &[u8]) -> Vec<(String, u16, u16, u16, u16)> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        (0..archive.len())
            .map(|i| {
                let entry = archive.by_index(i).unwrap();
                let local_at = usize::try_from(entry.header_start()).unwrap();
                let central_at = usize::try_from(entry.central_header_start()).unwrap();
                let u16_at = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
                (
                    entry.name().unwrap().into_owned(),
                    u16_at(local_at + 4),
                    u16_at(central_at + 6),
                    u16_at(local_at + 28),
                    u16_at(central_at + 30),
                )
            })
            .collect()
    }

    /// ZIP-10 (#604): `to_zip` writes Deflated entries, and a Deflated
    /// entry's version-needed is 20 (zip's `get_version_needed`), in both
    /// headers — never a hardcoded value, never the 45 a ZIP64 record
    /// carries. No extra fields means no 0x0001 ZIP64 field, and no ZIP64
    /// end record or locator follows.
    #[test]
    fn to_zip_entries_carry_version_20_and_no_zip64() {
        let mut fs = PartFs::open(&crate::admission::zip_names::docx_package("", &[])).unwrap();
        fs.set_part("word/media/logo.png", vec![7u8; 300]);
        let bytes = fs.to_zip().unwrap();
        let versions = header_versions(&bytes);
        assert!(versions.len() >= 4, "{versions:?}");
        for (name, local, central, local_extra, central_extra) in &versions {
            assert_eq!(
                (*local, *central),
                (20, 20),
                "{name}: a Deflated entry needs version 20 in both headers"
            );
            assert_eq!(
                (*local_extra, *central_extra),
                (0, 0),
                "{name}: no extra fields means no 0x0001 ZIP64 field"
            );
        }
        assert!(
            !bytes
                .windows(4)
                .any(|w| w == b"PK\x06\x06" || w == b"PK\x06\x07"),
            "no ZIP64 end record or locator"
        );
    }

    /// The version-needed values differ by what the entry is, which is why
    /// no assertion may hardcode 20 (ZIP-10/#604): a Stored file needs 10
    /// (zip's `MIN_VERSION`), a Stored directory needs 20
    /// (`get_version_needed` raises a directory to 20), and a forced
    /// `large_file(true)` entry needs 45 and carries the 0x0001 ZIP64
    /// field — the record shape `zip32_plan` refuses to emit.
    #[test]
    fn version_needed_distinguishes_stored_deflate_and_zip64() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer.start_file("stored.bin", stored).unwrap();
        writer.write_all(b"raw").unwrap();
        writer.add_directory("dir/", stored).unwrap();
        writer
            .start_file(
                "wide.bin",
                SimpleFileOptions::default()
                    .compression_method(CompressionMethod::Stored)
                    .large_file(true),
            )
            .unwrap();
        writer.write_all(b"wide").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let versions = header_versions(&bytes);
        let find = |name: &str| {
            versions
                .iter()
                .find(|(n, ..)| n == name)
                .unwrap_or_else(|| panic!("{name} in {versions:?}"))
        };
        let (_, local, central, ..) = find("stored.bin");
        assert_eq!((*local, *central), (10, 10), "a Stored file needs 10");
        let (_, local, central, ..) = find("dir/");
        assert_eq!(
            (*local, *central),
            (20, 20),
            "a Stored directory needs 20, not 10"
        );
        let (_, local, central, local_extra, central_extra) = find("wide.bin");
        assert_eq!((*local, *central), (45, 45), "a ZIP64 entry needs 45");
        assert!(
            *local_extra > 0 && *central_extra > 0,
            "a forced ZIP64 entry carries the 0x0001 field"
        );
    }
}
