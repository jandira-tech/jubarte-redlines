// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `WmlDocument` / `PtMainDocumentPart` (`WmlDocument.ts`) — M1.7.
//!
//! Models the C# `WmlDocument` surface the comparer uses: it carries
//! `DocumentByteArray` and exposes the main document part (parsed into the arena
//! DOM on demand). Backed by the M1.5 `PartFs` OPC layer.

use crate::admission::{InputLimits, admit};
use crate::opc::{OpcError, PartFs, refused};
use crate::xmllinq::{Dom, NodeId};

/// A WordprocessingML document held as its package bytes plus the main
/// document part parsed on demand. Mirrors the C# `WmlDocument`
/// (`DocumentByteArray` + lazy `MainDocumentPart`) used by the
/// [`crate::comparer`] pipeline.
///
/// Constructed with [`Self::from_bytes`] (which admits under
/// [`InputLimits::compare`](crate::admission::InputLimits::compare));
/// reads and edits go through [`Self::dom`] / [`Self::dom_mut`] and
/// [`Self::part_fs`] / [`Self::part_fs_mut`].
pub struct WmlDocument {
    /// `DocumentByteArray`: the bytes the document was opened from.
    document_byte_array: Vec<u8>,
    /// `FileName` — the inherited `WmlDocument` property, kept as a free
    /// string for callers that gather metrics or build paths from it.
    pub file_name: String,
    part_fs: PartFs,
    dom: Dom,
    main_doc: Option<NodeId>,
}

impl WmlDocument {
    /// Opens `bytes` as a `.docx`, admits it under
    /// [`InputLimits::compare`](crate::admission::InputLimits::compare),
    /// and lazily parses the main document part on the first
    /// [`Self::main_document`] call.
    ///
    /// # Errors
    ///
    /// - `OpcError::Io` wrapping
    ///   [`crate::admission::AdmissionError`] when admission refuses the
    ///   bytes (a budget, a duplicate or unsafe part name, an encrypted
    ///   package, an `.doc` or RTF that is not a `.docx`, or an XML part
    ///   too deep or malformed).
    /// - Any error [`PartFs::open`] reports (the standard OPC errors:
    ///   [`OpcError::Zip`], [`OpcError::PartNotFound`], etc.).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OpcError> {
        admit(bytes, InputLimits::compare()).map_err(refused)?;
        Ok(WmlDocument {
            document_byte_array: bytes.to_vec(),
            file_name: String::new(),
            part_fs: PartFs::open(bytes)?,
            dom: Dom::new(),
            main_doc: None,
        })
    }

    /// The bytes the document was opened from. Edits through
    /// [`Self::part_fs_mut`] or [`Self::dom_mut`] do not show here;
    /// repack through [`crate::opc::PartFs::to_zip`] to get the new bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.document_byte_array
    }

    /// The main document part name (`word/document.xml` unless OPC
    /// relationships name otherwise). The string always returned even
    /// when the part is absent.
    pub fn main_document_part_name(&self) -> String {
        self.part_fs
            .main_document_part()
            .unwrap_or_else(|| "word/document.xml".to_string())
    }

    /// Parses the main document part (once) and returns its document
    /// node. Subsequent calls return the cached node.
    ///
    /// # Errors
    ///
    /// [`OpcError::PartNotFound`] when the package has no part at
    /// [`Self::main_document_part_name`] (the body the comparer would
    /// diff into is missing).
    pub fn main_document(&mut self) -> Result<NodeId, OpcError> {
        if let Some(id) = self.main_doc {
            return Ok(id);
        }
        let name = self.main_document_part_name();
        let xml = self
            .part_fs
            .part_string(&name)
            .ok_or_else(|| OpcError::PartNotFound(name.clone()))?;
        let doc = self.dom.parse_xdocument(&xml);
        self.main_doc = Some(doc);
        Ok(doc)
    }

    /// The root element (`<w:document>`) of the main document part.
    ///
    /// # Errors
    ///
    /// [`OpcError::PartNotFound`] when the part is absent, or the parsed
    /// XML has no root element.
    pub fn main_document_root(&mut self) -> Result<NodeId, OpcError> {
        let doc = self.main_document()?;
        let name = self.main_document_part_name();
        self.dom
            .root(doc)
            .ok_or_else(|| OpcError::PartNotFound(format!("{name}: no root element")))
    }

    /// The arena DOM (read-only). Empty until a successful
    /// [`Self::main_document`] parses the part into it.
    pub fn dom(&self) -> &Dom {
        &self.dom
    }
    /// The arena DOM (mutable). The comparer mutates this DOM while it
    /// produces the body diff.
    pub fn dom_mut(&mut self) -> &mut Dom {
        &mut self.dom
    }
    /// The opened OPC package (read-only), by which every other part and
    /// relationship is read.
    pub fn part_fs(&self) -> &PartFs {
        &self.part_fs
    }
    /// The opened OPC package (mutable).
    pub fn part_fs_mut(&mut self) -> &mut PartFs {
        &mut self.part_fs
    }
}
