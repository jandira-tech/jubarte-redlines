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
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

mod archive;
pub(crate) use archive::open_archive;

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
fn read_package(bytes: &[u8]) -> Result<PartFs, OpcError> {
    let mut archive = open_archive(Cursor::new(bytes))?;
    let mut parts = HashMap::new();
    let mut source_order = HashMap::new();
    let mut identities = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.is_dir() {
            continue;
        }
        // zip 8's name decode (bit 11/CP437, 0x7075 override): the same one
        // `admit` applies, so both readers see one name per entry (#524).
        let name = crate::admission::zip_entry_name(
            entry.name_raw(),
            entry.flags().as_u16(),
            crate::admission::unicode_path_fields(entry.extra_data_fields().cloned()),
        )?;
        source_order.insert(name.clone(), index);
        // OPC part names are case-insensitive: two entries whose decoded
        // names collide (a legacy CP437 alias or an ASCII case pair) are the
        // same part, and the second insert used to overwrite the first
        // silently. Upstream's reader refuses `DuplicatePartName`; so does
        // the compatibility loader, so no part is lost.
        if !identities.insert(name.to_ascii_lowercase()) {
            return Err(OpcError::DuplicatePartName(name));
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        parts.insert(name, data);
    }
    let mut package = PartFs {
        pkg: OpcPackage::new(),
        package_rels: Relationships::new(),
        part_rels: HashMap::new(),
        source_order,
        repaired_duplicate_ids: Vec::new(),
    };
    package.pkg.content_types = read_content_types(
        &parts
            .remove("[Content_Types].xml")
            .ok_or_else(|| OpcError::PartNotFound("[Content_Types].xml".to_string()))?,
    )?;
    if let Some(xml) = parts.remove("_rels/.rels") {
        package.package_rels = read_relationships(&xml)?;
    }
    for (name, data) in parts {
        if name.ends_with(".rels") {
            let owner = rels_path_to_part_name(&name)
                .unwrap_or_else(|| name.strip_suffix(".rels").unwrap_or(&name).to_string());
            package
                .part_rels
                .insert(norm(&owner), read_relationships(&data)?);
        } else {
            package.pkg.parts.insert(norm(&name), data);
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
fn rels_path_to_part_name(name: &str) -> Option<String> {
    let name = name.strip_prefix('/').unwrap_or(name);
    let file = name.strip_suffix(".rels")?;
    let (dir, base) = file.rsplit_once('/')?;
    let owner_dir = if dir == "_rels" {
        ""
    } else {
        dir.strip_suffix("/_rels")?
    };
    Some(if owner_dir.is_empty() {
        base.to_string()
    } else {
        format!("{owner_dir}/{base}")
    })
}

/// One zip entry `PartFs::to_zip` writes after the two package-level ones.
enum ZipEntry<'a> {
    Rels(&'a Relationships),
    Part(&'a [u8]),
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

fn check_package_metadata(bytes: &[u8], limit: u64) -> Result<(), OpcError> {
    let mut archive = open_archive(Cursor::new(bytes))?;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name()?;
        if name != "[Content_Types].xml" && !name.ends_with(".rels") {
            continue;
        }
        let mut xml = Vec::new();
        entry.take(limit.saturating_add(1)).read_to_end(&mut xml)?;
        if u64::try_from(xml.len()).unwrap_or(u64::MAX) > limit {
            return Err(refused(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "OPC metadata exceeds part byte budget",
            )));
        }
        check_opc_attributes(&xml)?;
    }
    Ok(())
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
}

/// Canonical target resolution shared by package APIs and text story labels.
pub(crate) fn resolve_rel_target(source_part: &str, rel_target: &str) -> String {
    let joined = OpcPackage::resolve_rel_target(&norm(source_part), rel_target);
    let mut segments: Vec<&str> = Vec::new();
    for seg in joined.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    segments.join("/")
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
    /// # Errors
    ///
    /// [`OpcError`] for an unreadable zip, an OPC metadata element
    /// exceeding 256 attributes, two relationships under the same `Id` that
    /// differ in any attribute, or two entries whose decoded part names
    /// collide case-insensitively ([`OpcError::DuplicatePartName`]).
    pub fn open(bytes: &[u8]) -> Result<Self, OpcError> {
        check_package_metadata(
            bytes,
            crate::admission::InputLimits::compare().max_part_bytes,
        )?;
        let mut fs = read_package(bytes)?;
        let mut repaired_duplicate_ids: Vec<(String, String)> =
            drop_repeated_relationships(&mut fs.package_rels)?
                .into_iter()
                .map(|id| (String::new(), id))
                .collect();
        let mut owners: Vec<&String> = fs.part_rels.keys().collect();
        owners.sort();
        let owners: Vec<String> = owners.into_iter().cloned().collect();
        for owner in owners {
            if let Some(rels) = fs.part_rels.get_mut(&owner) {
                let part = owner.trim_start_matches('/').to_string();
                repaired_duplicate_ids.extend(
                    drop_repeated_relationships(rels)?
                        .into_iter()
                        .map(|id| (part.clone(), id)),
                );
            }
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

    /// `PartFS.partBytes(name)` — raw bytes of a part.
    pub fn part_bytes(&self, name: &str) -> Option<&[u8]> {
        self.pkg.get_part(&norm(name))
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
    pub fn set_part(&mut self, name: &str, data: Vec<u8>) {
        if let Some(owner) = rels_path_to_part_name(name)
            && check_opc_attributes(&data).is_ok()
            && let Ok(rels) = read_relationships(&data)
        {
            if owner.is_empty() {
                self.package_rels = rels;
            } else {
                self.part_rels.insert(norm(&owner), rels);
            }
            return;
        }
        self.pkg.set_part(&norm(name), data);
    }

    /// Remove a part (no-op when absent). Content-type overrides and rels
    /// pointing at it are left to the caller.
    pub fn remove_part(&mut self, name: &str) {
        self.pkg.parts.remove(&norm(name));
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
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(1))
            .last_modified_time(zip::DateTime::default());

        let ct_xml = self.pkg.content_types.to_xml()?;
        zip.start_file("[Content_Types].xml", options)?;
        zip.write_all(&ct_xml)?;

        let pkg_rels_xml = self.package_rels.to_xml()?;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(&pkg_rels_xml)?;

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
            // Our own Default under another case (`png` for `image3.PNG`)
            // already types the part: a second Default for the same extension
            // makes the package unopenable.
            if by_ext(self).is_some() {
                continue;
            }
            let key = norm(&part);
            for src in sources {
                if let Some(ct) = src.pkg.content_types.overrides.get(&key).cloned() {
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

    /// Remove an Override entry from `[Content_Types].xml` (no-op when absent).
    pub fn remove_content_type_override(&mut self, part_name: &str) {
        self.pkg.content_types.overrides.remove(&norm(part_name));
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
        let Some(target) = self.part_rels.get(&key).and_then(|rels| {
            rels.items
                .iter()
                .find(|r| r.id == rel_id && r.target_mode.as_deref() != Some("External"))
                .map(|r| r.target.clone())
        }) else {
            return;
        };
        let part = self.resolve_rel_target(source_part, &target);
        if let Some(rels) = self.part_rels.get_mut(&key) {
            rels.items.retain(|r| r.id != rel_id);
        }
        self.remove_untargeted_part(&part);
    }

    /// Remove `part`, its relationships and content-type override when no
    /// internal relationship of another part targets it, then each part it
    /// targeted that is left untargeted. Its own relationships never keep
    /// it, and a removed part ends a loop.
    fn remove_untargeted_part(&mut self, part: &str) {
        let key = norm(part);
        if !self.pkg.parts.contains_key(&key) {
            return;
        }
        let still_targeted = self
            .part_rels
            .iter()
            .filter(|(source, _)| **source != key)
            .map(|(source, rels)| (source.as_str(), rels))
            .chain(std::iter::once(("/", &self.package_rels)))
            .any(|(source, rels)| {
                rels.items.iter().any(|r| {
                    r.target_mode.as_deref() != Some("External")
                        && self.resolve_rel_target(source, &r.target) == part
                })
            });
        if still_targeted {
            return;
        }
        self.pkg.parts.remove(&key);
        let targets: Vec<String> = self
            .part_rels
            .remove(&key)
            .map(|rels| {
                rels.items
                    .iter()
                    .filter(|r| r.target_mode.as_deref() != Some("External"))
                    .map(|r| self.resolve_rel_target(part, &r.target))
                    .collect()
            })
            .unwrap_or_default();
        self.remove_content_type_override(&format!("/{part}"));
        for target in targets {
            self.remove_untargeted_part(&target);
        }
    }

    /// `readRelsFor(part)` — the relationships of a part, if any.
    pub fn read_rels_for(&self, part_name: &str) -> Option<&Relationships> {
        self.part_rels.get(self.part_rels_key(part_name)?)
    }

    // Preserve upstream's case-insensitive, normalized lookup and deterministic
    // spelling choice when more than one stored owner has the same identity.
    fn part_rels_key(&self, part_name: &str) -> Option<&String> {
        let key = resolve_rel_target("", part_name);
        self.part_rels
            .keys()
            .filter(|candidate| resolve_rel_target("", candidate).eq_ignore_ascii_case(&key))
            .min()
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
        // Not `or_default()`: the derived `Default` is the rId0 counter.
        match self.part_rels.entry(norm(source_part)) {
            Entry::Occupied(rels) => rels.into_mut(),
            Entry::Vacant(slot) => slot.insert(Relationships::new()),
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
        assert!(check_package_metadata(&bytes, 32).is_err());
        assert!(check_package_metadata(b"not a zip", 32).is_err());

        // Content types are also parsed by upstream, before relationships.
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        write!(writer, "<Types{}/>", attrs(257)).unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let err = check_package_metadata(&bytes, 4096).unwrap_err();
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
        // `DuplicatePartName` on the ASCII-lowercased name
        // (oxml-opc-0.14.2/src/package.rs 159-162); the compatibility loader
        // refuses the same way, so nothing is lost (#524).
        let base = |extra: &[zip_names::RawEntry<'_>]| zip_names::docx_package("", extra);
        // (a) two raw names, bit 11 set, whose lossy UTF-8 decode is the same
        // `x\u{FFFD}.bin`: two parts, one name.
        let aliased = base(&[
            zip_names::RawEntry::raw(b"x\x82.bin", b"FIRST").utf8(),
            zip_names::RawEntry::raw(b"x\x83.bin", b"SECOND").utf8(),
        ]);
        assert!(matches!(
            PartFs::open(&aliased),
            Err(OpcError::DuplicatePartName(name)) if name == "x\u{FFFD}.bin"
        ));
        // (b) two names that differ only in ASCII case: one part, OPC names
        // are case-insensitive.
        let cased = base(&[
            zip_names::RawEntry::raw(b"word/A.bin", b"FIRST"),
            zip_names::RawEntry::raw(b"word/a.bin", b"SECOND"),
        ]);
        assert!(matches!(
            PartFs::open(&cased),
            Err(OpcError::DuplicatePartName(name)) if name == "word/a.bin"
        ));
        // (c) the zip 8 shape: both names flag-clear, CP437 decodes them
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
}
