// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word redlines between Word and Markdown documents.

use super::{
    DocxOptions, ImageLoader, MarkdownError, TrackChanges, apply_markdown, markdown_to_docx,
};
use crate::comparer::WmlComparerSettings;
use crate::document_comparer::compare_documents_with_settings;

/// A document to compare.
#[derive(Clone, Copy, Debug)]
pub enum Source<'a> {
    /// A `.docx` package.
    Docx(&'a [u8]),
    /// Markdown text.
    Markdown(&'a str),
}

/// How [`redline`] reads and compares two documents.
#[derive(Clone, Default)]
pub struct RedlineOptions<'a> {
    /// The comparer's settings: author, date, detail threshold, mode.
    pub settings: WmlComparerSettings,
    /// Styles, page setup, headers and footers when both sides are
    /// Markdown; built-in styles when `None`. (Against a Word document the
    /// Markdown is applied to that document instead.)
    pub reference: Option<&'a [u8]>,
    /// Read CriticMarkup in a Markdown side as tracked changes. Off by
    /// default: a document being compared is text.
    pub critic: bool,
    /// The bytes of each image a Markdown side names.
    pub images: Option<ImageLoader<'a>>,
}

impl std::fmt::Debug for RedlineOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedlineOptions")
            .field("settings", &self.settings)
            .field("reference", &self.reference.map(<[u8]>::len))
            .field("critic", &self.critic)
            .field("images", &self.images.is_some())
            .finish()
    }
}

/// `original` against `modified` as a Word tracked-changes document: the
/// original with every difference as a revision, as
/// [`compare_documents_with_settings`] writes it. Either side may be
/// Markdown. Against a Word document, the Markdown's edits are applied to
/// that document ([`apply_markdown`]), so what Markdown cannot hold stays;
/// two Markdown documents are written to Word first ([`markdown_to_docx`]).
///
/// ```
/// use jubarte::markdown::{RedlineOptions, Source, redline};
///
/// let docx = redline(
///     Source::Markdown("Payment in 30 days.\n"),
///     Source::Markdown("Payment in 45 days.\n"),
///     &RedlineOptions::default(),
/// )
/// .unwrap();
/// assert!(docx.starts_with(b"PK"));
/// ```
pub fn redline(
    original: Source<'_>,
    modified: Source<'_>,
    options: &RedlineOptions<'_>,
) -> Result<Vec<u8>, MarkdownError> {
    let reference = options.reference;
    let word = |source: Source<'_>| -> Result<Vec<u8>, MarkdownError> {
        match source {
            Source::Docx(docx) => Ok(docx.to_vec()),
            Source::Markdown(markdown) => {
                let written = markdown_to_docx(
                    markdown,
                    &DocxOptions {
                        reference,
                        critic: options.critic,
                        track_changes: TrackChanges::All,
                        author: options.settings.author_for_revisions.clone(),
                        date: options.settings.date_time_for_revisions.clone(),
                        images: options.images,
                    },
                )?;
                Ok(written.docx)
            }
        }
    };
    // Markdown against Word: the Markdown's edits are applied to the Word
    // document, so all it cannot say (empty paragraphs, fields, section
    // breaks, direct formatting) stays as it was.
    let (original, modified) = match (original, modified) {
        (Source::Docx(docx), Source::Markdown(markdown)) => {
            let patched = apply_markdown(docx, markdown)?;
            (docx.to_vec(), patched.docx)
        }
        (Source::Markdown(markdown), Source::Docx(docx)) => {
            let patched = apply_markdown(docx, markdown)?;
            (patched.docx, docx.to_vec())
        }
        (original, modified) => (word(original)?, word(modified)?),
    };
    compare_documents_with_settings(&original, &modified, &options.settings)
        .map_err(|e| MarkdownError::Package(format!("comparing: {e}")))
}
