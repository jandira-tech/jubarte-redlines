// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The Markdown side of the Word redline task.
//!
//! A redline takes two [`Source`] documents (`.docx` bytes or Markdown
//! strings), turns each into a Word package, and feeds the comparator
//! the bodies, returning a tracked-changes `.docx`. Markdown against a
//! Word document applies the Markdown's edits to that document
//! ([`apply_markdown`]) so direct formatting, fields, section breaks
//! and empty paragraphs the Markdown cannot hold stay as they were.
//! Markdown against Markdown is written to Word first
//! ([`markdown_to_docx`]).
//!
//! Backs `jubarte compare` / `jubarte redline`, the Markdown input of
//! [`crate::text_diff`], and the Python / WASM
//! [`crate::markdown::redline`] binding.

use super::{
    DocxOptions, ImageLoader, MarkdownError, PageSize, TrackChanges, apply_markdown,
    markdown_to_docx,
};
use crate::comparer::WmlComparerSettings;
use crate::document_comparer::compare_documents_with_settings;

/// A document side of a comparison: either a `.docx` package or a
/// Markdown text. Backed onto [`crate::markdown::redline`] from the
/// redline command line and from the Python / WASM bindings.
#[derive(Clone, Copy, Debug)]
pub enum Source<'a> {
    /// A `.docx` package bytes.
    Docx(&'a [u8]),
    /// Markdown text.
    Markdown(&'a str),
}

/// How [`redline`] reads and compares two [`Source`] documents.
///
/// Defaults: [`WmlComparerSettings::default`] (author
/// `"Open-Xml-PowerTools"`; the CLI and bindings pass `Redline`,
/// `DATE "1970-01-01T00:00:00Z"` for reproducible output, Word's own
/// detail threshold); no reference document, no CriticMarkup recognition,
/// no image loader.
#[derive(Clone, Default)]
pub struct RedlineOptions<'a> {
    /// The comparer's settings: author, date, detail threshold, mode,
    /// move detection, format-change detection, the input budget for
    /// [`crate::admission`]. See [`crate::comparer::WmlComparerSettings`].
    pub settings: WmlComparerSettings,
    /// Styles, page setup, headers and footers when both sides are
    /// Markdown; built-in styles when `None`. Against a Word document the
    /// Markdown is applied to that document instead, so this only affects
    /// the Markdown / Markdown case.
    pub reference: Option<&'a [u8]>,
    /// Read CriticMarkup in a Markdown side as tracked changes and
    /// comments. Off by default: a Markdown document being compared is
    /// treated as plain text.
    pub critic: bool,
    /// The bytes of each image a Markdown side names; `None` writes every
    /// image as its alt text.
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

/// Compares `original` against `modified` and returns a Word
/// tracked-changes document: the original with every difference as a
/// revision, as
/// [`crate::document_comparer::compare_documents_with_settings`] writes
/// it. Backs `jubarte compare`, `jubarte redline`, and the Python / WASM
/// redline bindings.
///
/// Either side may be Markdown. Against a Word document, the Markdown's
/// edits are applied to that Word document ([`apply_markdown`]), so what
/// Markdown cannot represent (empty paragraphs, fields, section breaks,
/// direct formatting) stays as it was. Two Markdown documents are first
/// written to Word ([`markdown_to_docx`]).
///
/// # Errors
///
/// [`MarkdownError::Package`] when a Word side paired with Markdown cannot
/// be admitted or read by [`apply_markdown`], or when the comparer's
/// `compare_documents_with_settings` call fails; what [`markdown_to_docx`]
/// returns (a bad reference document is [`MarkdownError::Reference`]) when
/// both sides are Markdown.
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
                        author_fixed: false,
                        date_fixed: false,
                        images: options.images,
                        page: PageSize::Letter,
                        tracking: None,
                        scholar: false,
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
