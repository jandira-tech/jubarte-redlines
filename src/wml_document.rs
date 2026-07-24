// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `WmlDocument` / `PtMainDocumentPart` (`WmlDocument.ts`) — M1.7.
//!
//! Models the C# `WmlDocument` surface the comparer uses: it carries
//! `DocumentByteArray` and exposes the main document part (parsed into the arena
//! DOM on demand). Backed by the M1.5 `PartFs` OPC layer.

use crate::opc::{OpcError, PartFs};
use crate::xmllinq::{Dom, NodeId};

/// A WordprocessingML document (bytes + lazily-parsed main document part).
pub struct WmlDocument {
    /// `DocumentByteArray` — the backing bytes.
    pub document_byte_array: Vec<u8>,
    /// `FileName` — mirrors the inherited property (consumers read it for metrics).
    pub file_name: String,
    part_fs: PartFs,
    dom: Dom,
    main_doc: Option<NodeId>,
}

impl WmlDocument {
    /// `new WmlDocument(bytes)`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OpcError> {
        Ok(WmlDocument {
            document_byte_array: bytes.to_vec(),
            file_name: String::new(),
            part_fs: PartFs::open(bytes)?,
            dom: Dom::new(),
            main_doc: None,
        })
    }

    /// The main document part name (e.g. `word/document.xml`).
    pub fn main_document_part_name(&self) -> String {
        self.part_fs
            .main_document_part()
            .unwrap_or_else(|| "word/document.xml".to_string())
    }

    /// `MainDocumentPart` — parse (once) the main document and return its
    /// Document node. Subsequent calls return the cached node.
    ///
    /// Returns [`OpcError::PartNotFound`] when the package has no main document
    /// part (fallible input surface — never panics on missing user content).
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
    pub fn main_document_root(&mut self) -> Result<NodeId, OpcError> {
        let doc = self.main_document()?;
        let name = self.main_document_part_name();
        self.dom
            .root(doc)
            .ok_or_else(|| OpcError::PartNotFound(format!("{name}: no root element")))
    }

    /// Borrow the arena DOM (read).
    pub fn dom(&self) -> &Dom {
        &self.dom
    }
    /// Borrow the arena DOM (mutate).
    pub fn dom_mut(&mut self) -> &mut Dom {
        &mut self.dom
    }
    /// Borrow the OPC package (read).
    pub fn part_fs(&self) -> &PartFs {
        &self.part_fs
    }
    /// Borrow the OPC package (mutate).
    pub fn part_fs_mut(&mut self) -> &mut PartFs {
        &mut self.part_fs
    }
}
