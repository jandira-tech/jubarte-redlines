// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! OPC (Open Packaging Conventions) layer — M1.5.
//!
//! SPIKE FINDINGS (rdocx-opc 0.1, verified 2026-06-27):
//! `rdocx_opc::OpcPackage` already provides everything `opc-partfs.ts` needs, at
//! the BYTE level — so the plan's Step-5 fallback (porting opc-partfs over `zip`)
//! is NOT required:
//!   - `from_reader` / `write_to`   → open from bytes / write the zip back
//!   - `get_part` / `set_part`      → byte-level read / replace / add a part
//!   - `parts: HashMap`             → enumerate parts
//!   - `get_part_rels` / `get_or_create_part_rels` + `Relationships::add`
//!     → parse / resolve / add relationships
//!   - `resolve_rel_target`         → relative target resolution
//!   - `content_types.content_type_for` / `add_default` / `add_override`
//!     → read / mutate [Content_Types].xml
//!   - `main_document_part`         → the package → main-document rel
//!
//! rdocx-opc keys parts/rels with a LEADING SLASH (`/word/document.xml`). This
//! adapter accepts the docxodus / opc-partfs style (no leading slash,
//! `word/document.xml`) and normalizes internally, so the rest of the crate is
//! oblivious to the difference.

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Cursor, Write};

use rdocx_opc::OpcPackage;
pub use rdocx_opc::{OpcError, Relationship, Relationships};

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
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Exactly one leading slash: callers build `/{part}` from names that may
/// already be absolute, and a `//word/…` part name is invalid OPC.
fn norm(name: &str) -> String {
    format!("/{}", name.trim_start_matches('/'))
}

/// The relationship `Target` that reaches `target_part` from `source_part`:
/// relative to the source's folder when the target sits under it, absolute
/// (`/media/image.bin`) otherwise. Stripping `word/` blindly pointed a
/// root-level `media/image.bin` at the non-existent `word/media/image.bin`
/// and Word refused the package.
pub fn relative_rel_target(source_part: &str, target_part: &str) -> String {
    let source = source_part.trim_start_matches('/');
    let target = target_part.trim_start_matches('/');
    let dir = source.rsplit_once('/').map_or("", |(d, _)| d);
    match target.strip_prefix(dir).and_then(|t| t.strip_prefix('/')) {
        Some(rel) if !dir.is_empty() => rel.to_string(),
        _ => format!("/{target}"),
    }
}

/// rdocx-opc 0.1.0 parses relationship attributes without decoding entities
/// but escapes them when writing, so each open/write cycle added one `&amp;`
/// to a target like `image11.jpg&ehk=…` until it named no part. Decode once on
/// the way in; the writer's escape is then the only one.
fn unescape_relationships(rels: &mut Relationships) {
    let decode = |v: &mut String| {
        if v.contains('&') {
            *v = crate::xmllinq::parse::unescape_xml_text(v);
        }
    };
    for r in &mut rels.items {
        decode(&mut r.id);
        decode(&mut r.rel_type);
        decode(&mut r.target);
        if let Some(m) = r.target_mode.as_mut() {
            decode(m);
        }
    }
}

/// The same decode for `[Content_Types].xml`, which rdocx-opc parses the same
/// way: a `Default Extension="jpg&amp;ehk=…"` otherwise grew one `&amp;` per
/// cycle and stopped typing its part.
fn unescape_content_types(map: &mut std::collections::HashMap<String, String>) {
    if map.iter().any(|(k, v)| k.contains('&') || v.contains('&')) {
        *map = map
            .drain()
            .map(|(k, v)| {
                (
                    crate::xmllinq::parse::unescape_xml_text(&k),
                    crate::xmllinq::parse::unescape_xml_text(&v),
                )
            })
            .collect();
    }
}

fn denorm(name: &str) -> String {
    name.trim_start_matches('/').to_string()
}

/// Convert a part name (leading-slash form, e.g. `/word/document.xml`) to its
/// `.rels` file path (e.g. `word/_rels/document.xml.rels`). Mirrors the
/// private `rdocx_opc::package::part_name_to_rels_path`.
fn part_name_to_rels_path(part_name: &str) -> String {
    let name = part_name.strip_prefix('/').unwrap_or(part_name);
    if let Some(pos) = name.rfind('/') {
        let dir = &name[..pos];
        let file = &name[pos + 1..];
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

/// Thin adapter over `rdocx_opc::OpcPackage`. Port-equivalent of `PartFS`.
pub struct PartFs {
    pkg: OpcPackage,
    /// Each source zip entry's position, so `to_zip` writes the package back
    /// in the source's order rather than the hash maps' run-to-run order.
    source_order: HashMap<String, usize>,
}

impl PartFs {
    /// Open a `.docx`/OPC package from raw bytes.
    pub fn open(bytes: &[u8]) -> Result<Self, OpcError> {
        let mut pkg = OpcPackage::from_reader(Cursor::new(bytes.to_vec()))?;
        unescape_relationships(&mut pkg.package_rels);
        pkg.part_rels.values_mut().for_each(unescape_relationships);
        unescape_content_types(&mut pkg.content_types.defaults);
        unescape_content_types(&mut pkg.content_types.overrides);
        let source_order = ZipArchive::new(Cursor::new(bytes))
            .map(|zip| {
                (0..zip.len())
                    .filter_map(|i| zip.name_for_index(i).map(|n| (n.to_string(), i)))
                    .collect()
            })
            .unwrap_or_default();
        Ok(PartFs { pkg, source_order })
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
            && let Ok(mut rels) = Relationships::from_xml(&data)
        {
            unescape_relationships(&mut rels);
            if owner.is_empty() {
                self.pkg.package_rels = rels;
            } else {
                self.pkg.part_rels.insert(norm(&owner), rels);
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
    /// opens any deflate level. ZIP-LEVEL-01 (WASM_PERF_PLAN.md).
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

        let pkg_rels_xml = self.pkg.package_rels.to_xml()?;
        zip.start_file("_rels/.rels", options)?;
        zip.write_all(&pkg_rels_xml)?;

        // Source entries keep their places; added ones follow, by name.
        let mut entries: Vec<(Cow<str>, ZipEntry)> = self
            .pkg
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
        // The relationships reader keeps the Target attribute as written, so
        // "image1.jpg&amp;ehk=…" names the part "image1.jpg&ehk=…".
        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
        let joined = OpcPackage::resolve_rel_target(&norm(source_part), &target);
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

    /// `contentTypeFor(part)`.
    pub fn content_type_for(&self, name: &str) -> Option<String> {
        self.pkg
            .content_types
            .content_type_for(&norm(name))
            .map(|s| s.to_string())
    }

    /// Add an Override entry to [Content_Types].xml.
    pub fn add_content_type_override(&mut self, part_name: &str, content_type: &str) {
        self.pkg
            .content_types
            .add_override(&norm(part_name), content_type);
    }

    /// Add a Default extension mapping to [Content_Types].xml.
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

    /// Remove an Override entry from [Content_Types].xml (no-op when absent).
    pub fn remove_content_type_override(&mut self, part_name: &str) {
        self.pkg.content_types.overrides.remove(&norm(part_name));
    }

    /// Remove every relationship of `source_part` with the given type.
    /// No-op when the source part has no relationships part yet — must not
    /// invent an empty `.rels` entry just to remove from it (PR #81 review).
    pub fn remove_relationships_by_type(&mut self, source_part: &str, rel_type: &str) {
        let key = norm(source_part);
        if self.pkg.get_part_rels(&key).is_none() {
            return;
        }
        let rels = self.pkg.get_or_create_part_rels(&key);
        rels.items.retain(|r| r.rel_type != rel_type);
    }

    /// Remove the relationship `rel_id` of `source_part` and, unless another
    /// internal relationship still targets it, the part it names with that
    /// part's own relationships and content-type override. No-op for an
    /// unknown or external relationship.
    pub fn remove_related_part(&mut self, source_part: &str, rel_id: &str) {
        let key = norm(source_part);
        let Some(target) = self.pkg.get_part_rels(&key).and_then(|rels| {
            rels.items
                .iter()
                .find(|r| r.id == rel_id && r.target_mode.as_deref() != Some("External"))
                .map(|r| r.target.clone())
        }) else {
            return;
        };
        let part = self.resolve_rel_target(source_part, &target);
        self.pkg
            .get_or_create_part_rels(&key)
            .items
            .retain(|r| r.id != rel_id);
        let still_targeted = self
            .pkg
            .part_rels
            .iter()
            .map(|(source, rels)| (source.as_str(), rels))
            .chain(std::iter::once(("/", &self.pkg.package_rels)))
            .any(|(source, rels)| {
                rels.items.iter().any(|r| {
                    r.target_mode.as_deref() != Some("External")
                        && self.resolve_rel_target(source, &r.target) == part
                })
            });
        if still_targeted {
            return;
        }
        self.pkg.parts.remove(&norm(&part));
        self.pkg.part_rels.remove(&norm(&part));
        self.remove_content_type_override(&format!("/{part}"));
    }

    /// `readRelsFor(part)` — the relationships of a part, if any.
    pub fn read_rels_for(&self, part_name: &str) -> Option<&Relationships> {
        self.pkg.get_part_rels(&norm(part_name))
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
        &self.pkg.package_rels
    }

    /// Add a package-level relationship, returning its id.
    pub fn add_package_relationship(&mut self, rel_type: &str, target: &str) -> String {
        self.pkg.package_rels.add(rel_type, target)
    }

    /// A part's relationships, created empty when missing. Created through
    /// `Relationships::new`, which numbers from `rId1` as Word does; the
    /// dependency's `get_or_create_part_rels` defaults to `rId0`.
    fn part_rels_mut(&mut self, source_part: &str) -> &mut Relationships {
        use std::collections::hash_map::Entry;
        // Not `or_default()`: the derived `Default` is the rId0 counter.
        match self.pkg.part_rels.entry(norm(source_part)) {
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
        self.pkg.main_document_part().map(|s| denorm(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
    const HYPERLINK: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";

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
            .map(str::to_string)
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
                entry.name()
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
}
