// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The pure option types of a DOCX -> PDF conversion.
//!
//! `PdfOptions`, `RenderRequest` and the mark/palette types describe what a
//! conversion should do, without a line of layout in them, so every layer
//! above (fields, audit, the debug views, the CLI and the bindings) can
//! name them without linking the renderer. The renderer re-exports them at
//! `jubarte::convert::…`, the path they have always had.

/// How many strike or underline lines a revision mark draws under
/// changed text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkLines {
    /// No line.
    #[default]
    None,
    /// One line.
    Single,
    /// Two lines.
    Double,
}

/// How one kind of tracked change is painted: the ink and the strike /
/// underline it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RevisionMark {
    /// Ink of the changed text and its lines, as an RGB triple.
    pub color: [u8; 3],
    /// Lines struck through the text.
    pub strike: MarkLines,
    /// Lines under the text.
    pub underline: MarkLines,
}

/// The marks for each kind of tracked change (`w:del`, `w:ins`,
/// `w:moveFrom`, `w:moveTo`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RevisionPalette {
    /// `w:del`: text deleted by the redline.
    pub deleted: RevisionMark,
    /// `w:ins`: text inserted by the redline.
    pub inserted: RevisionMark,
    /// `w:moveFrom`: text moved from here.
    pub moved_from: RevisionMark,
    /// `w:moveTo`: text moved to here.
    pub moved_to: RevisionMark,
}

impl RevisionPalette {
    /// The legal-redline convention, as Litera Compare sets Word's track
    /// changes: an insertion or deletion is marked once (blue underline,
    /// red strike) and a move twice, in green (double strike where it
    /// left, double underline where it landed), so a move never reads as
    /// an insertion by its colour alone.
    pub const CONVENTIONAL: Self = Self {
        deleted: RevisionMark {
            color: [0xFF, 0x00, 0x00],
            strike: MarkLines::Single,
            underline: MarkLines::None,
        },
        inserted: RevisionMark {
            color: [0x00, 0x00, 0xFF],
            strike: MarkLines::None,
            underline: MarkLines::Single,
        },
        moved_from: RevisionMark {
            color: [0x00, 0x80, 0x00],
            strike: MarkLines::Double,
            underline: MarkLines::None,
        },
        moved_to: RevisionMark {
            color: [0x00, 0x80, 0x00],
            strike: MarkLines::None,
            underline: MarkLines::Double,
        },
    };
}

impl RevisionPalette {
    /// Parse `kind=#RRGGBB[:line...],...` over the conventional palette.
    /// Kinds: `deleted`, `inserted`, `moved-from`, `moved-to`. Lines:
    /// `strike`, `double-strike`, `underline`, `double-underline`, `plain`
    /// (none). A kind's colour alone keeps its conventional lines.
    ///
    /// ```
    /// use jubarte_opc::render_options::{MarkLines, RevisionPalette};
    /// let p = RevisionPalette::parse("inserted=#00AA00:underline").unwrap();
    /// assert_eq!(p.inserted.color, [0x00, 0xAA, 0x00]);
    /// assert_eq!(p.inserted.underline, MarkLines::Single);
    /// assert_eq!(p.deleted, RevisionPalette::CONVENTIONAL.deleted);
    /// ```
    pub fn parse(spec: &str) -> Result<Self, String> {
        let mut palette = Self::CONVENTIONAL;
        for entry in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let (kind, value) = entry
                .split_once('=')
                .ok_or_else(|| format!("'{entry}': expected kind=#RRGGBB[:lines]"))?;
            let mark = match kind.trim() {
                "deleted" | "del" => &mut palette.deleted,
                "inserted" | "ins" => &mut palette.inserted,
                "moved-from" | "move-from" => &mut palette.moved_from,
                "moved-to" | "move-to" => &mut palette.moved_to,
                other => return Err(format!("unknown revision kind '{other}'")),
            };
            let mut parts = value.split(':').map(str::trim);
            let hex = parts.next().unwrap_or("").trim_start_matches('#');
            if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("'{value}': colour must be #RRGGBB"));
            }
            let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            mark.color = [byte(0), byte(2), byte(4)];
            let lines: Vec<&str> = parts.collect();
            if !lines.is_empty() {
                mark.strike = MarkLines::None;
                mark.underline = MarkLines::None;
            }
            for line in lines {
                match line {
                    "strike" => mark.strike = MarkLines::Single,
                    "double-strike" => mark.strike = MarkLines::Double,
                    "underline" => mark.underline = MarkLines::Single,
                    "double-underline" => mark.underline = MarkLines::Double,
                    "plain" => {}
                    other => return Err(format!("unknown line style '{other}'")),
                }
            }
        }
        Ok(palette)
    }
}

/// How tracked changes (`w:ins`, `w:del`, `w:moveFrom`, `w:moveTo`) are
/// painted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RevisionStyle {
    /// [`RevisionPalette::CONVENTIONAL`] (red / blue / green redline).
    #[default]
    Conventional,
    /// What Microsoft Word's Save as PDF paints (per-author ink, single
    /// strike and underline); the fidelity benchmarks measure against this
    /// one, so a jubarte PDF compared against a Word one must be rendered
    /// with this mode.
    Word,
    /// The caller's own marks.
    Custom(RevisionPalette),
}

impl RevisionStyle {
    /// Parse `"conventional"`, `"word"` or `"custom"` (which needs a
    /// [`RevisionPalette::parse`] spec), as the CLI flag and the bindings
    /// name them.
    ///
    /// # Errors
    ///
    /// A message when `name` is unknown, when `name = "custom"` but
    /// `palette` is `None`, when `palette` is set with any name other than
    /// `"custom"`, or when `palette` fails [`RevisionPalette::parse`].
    ///
    /// ```
    /// use jubarte_opc::render_options::RevisionStyle;
    /// assert_eq!(RevisionStyle::from_choice("word", None), Ok(RevisionStyle::Word));
    /// assert!(RevisionStyle::from_choice("custom", None).is_err());
    /// ```
    pub fn from_choice(name: &str, palette: Option<&str>) -> Result<Self, String> {
        match (name, palette) {
            ("custom", Some(spec)) => RevisionPalette::parse(spec).map(Self::Custom),
            ("custom", None) => Err("revisions \"custom\" needs a revision palette".into()),
            (_, Some(_)) => Err("a revision palette needs revisions \"custom\"".into()),
            ("conventional", None) => Ok(Self::Conventional),
            ("word", None) => Ok(Self::Word),
            (other, None) => Err(format!(
                "unknown revisions \"{other}\" (conventional, word, custom)"
            )),
        }
    }
}

/// The flags a DOCX → PDF conversion runs with.
///
/// Defaults match what `docx_to_pdf(docx)` passes
/// ([`PdfOptions::default`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PdfOptions {
    /// Deflate content streams (`/Filter /FlateDecode`). Image samples and
    /// font programs are binary and always deflate.
    ///
    /// Off by default, because an uncompressed stream is plain text: it is
    /// what the conversion suite asserts on and what makes a generated page
    /// greppable when diffing against Word. Turning it on costs a fraction of
    /// a second and takes a text-heavy document to roughly a seventh of its
    /// size (a 217-page redline: 48.8 MB → 6.7 MB).
    pub compress: bool,
    /// How tracked changes are painted (default: the conventional
    /// red/blue/green redline marks).
    pub revisions: RevisionStyle,
    /// Where comments go: in balloons beside the text (the default, as
    /// Word prints them) or listed after the last page.
    pub comments: CommentPlacement,
    /// Also attach each comment as a PDF note (a text annotation) at its
    /// anchor. Off by default for every revision style: a viewer paints
    /// the note as an icon over the text, and the balloon already shows
    /// the comment. Word's Save as PDF writes none.
    pub comment_notes: bool,
    /// Lay the whole document out, then keep only the pages a tracked
    /// change touches (a revision in the body, a table, a footnote or an
    /// endnote), the
    /// pages a comment is anchored on, and the first page of each section
    /// on which a revised header or footer paints. Page numbers stay the
    /// document's. A document with none of these keeps its first page.
    pub changed_only: bool,
}

/// Where a converted PDF puts the document's comments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CommentPlacement {
    /// Balloons in a pane beside the page, joined to the commented text, as
    /// Word's Save as PDF prints them.
    #[default]
    Margin,
    /// The page keeps its own width: the commented text keeps its tint
    /// and a `[JR1]`-style marker, and the comments are listed after the
    /// last page.
    End,
}

/// What `convert::render` should produce from its single layout pass. All fields
/// default to "off"; `pdf = true` writes the PDF, `png_dpi = Some(d)`
/// rasterizes, `pages = Some([…])` narrows any of the above to a chosen
/// set of document pages.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderRequest {
    /// Write the PDF (only when at least one page is kept).
    pub pdf: bool,
    /// Rasterize every page written to PNG at this DPI. Must be in
    /// `1.0..=MAX_PNG_DPI` when set.
    pub png_dpi: Option<f32>,
    /// Zero-based document pages to write (any order, repeats ignored):
    /// the PDF, the PNGs and the report hold these pages only; `None`
    /// writes every page. They count the document's pages whatever
    /// [`PdfOptions::changed_only`] keeps, then the listing pages of
    /// [`CommentPlacement::End`]. A page it dropped is
    /// `ConvertError::PageNotKept`; one past the last is
    /// `ConvertError::PageOutOfRange`; an empty list writes no page and
    /// no PDF. Layout always covers the whole document.
    pub pages: Option<Vec<usize>>,
}
