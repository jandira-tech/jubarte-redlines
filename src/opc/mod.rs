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

use std::io::Cursor;

use rdocx_opc::OpcPackage;
pub use rdocx_opc::{OpcError, Relationship, Relationships};

fn norm(name: &str) -> String {
    if name.starts_with('/') {
        name.to_string()
    } else {
        format!("/{name}")
    }
}

fn denorm(name: &str) -> String {
    name.trim_start_matches('/').to_string()
}

/// Thin adapter over `rdocx_opc::OpcPackage`. Port-equivalent of `PartFS`.
pub struct PartFs {
    pkg: OpcPackage,
}

impl PartFs {
    /// Open a `.docx`/OPC package from raw bytes.
    pub fn open(bytes: &[u8]) -> Result<Self, OpcError> {
        Ok(PartFs {
            pkg: OpcPackage::from_reader(Cursor::new(bytes.to_vec()))?,
        })
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
    pub fn set_part(&mut self, name: &str, data: Vec<u8>) {
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
    pub fn to_zip(&self) -> Result<Vec<u8>, OpcError> {
        let mut buf = Cursor::new(Vec::new());
        self.pkg.write_to(&mut buf)?;
        Ok(buf.into_inner())
    }

    // ── gap helpers (the few bits opc-partfs adds on top of raw zip) ───────────

    /// `resolveRelTarget(sourcePart, relTarget)` — resolve a rel target relative
    /// to its source part. Style-preserving (input style is echoed back).
    pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
        OpcPackage::resolve_rel_target(source_part, rel_target)
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
        self.pkg
            .get_or_create_part_rels(&norm(source_part))
            .add(rel_type, target)
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
        let rels = self.pkg.get_or_create_part_rels(&norm(source_part));
        let id = rels.add(rel_type, target);
        if let Some(r) = rels.items.iter_mut().find(|r| r.id == id) {
            r.target_mode = Some("External".to_string());
        }
        id
    }

    /// Mark an existing relationship of `source_part` as External (test aid).
    pub fn set_rel_target_mode_external(&mut self, source_part: &str, rel_id: &str) {
        let rels = self.pkg.get_or_create_part_rels(&norm(source_part));
        if let Some(r) = rels.items.iter_mut().find(|r| r.id == rel_id) {
            r.target_mode = Some("External".to_string());
        }
    }

    /// The main document part name (docxodus style).
    pub fn main_document_part(&self) -> Option<String> {
        self.pkg.main_document_part().map(|s| denorm(&s))
    }
}
