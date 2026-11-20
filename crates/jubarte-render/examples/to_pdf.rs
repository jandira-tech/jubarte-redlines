// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Render a redline to PDF in each revision style, and to PNG pages.
//! Run: `cargo run -p jubarte-render --example to_pdf [OUT_DIR]`

// ANCHOR: example
use jubarte_render::convert::{
    CommentPlacement, PdfOptions, RevisionPalette, RevisionStyle, docx_to_pdf_with, docx_to_png,
    pdf_page_count,
};

const REDLINE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/1686-word-compare.docx"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docx = std::fs::read(REDLINE)?;
    let out = std::env::args().nth(1);

    let styles = [
        ("conventional", RevisionStyle::Conventional),
        // What Word's own Save as PDF paints: use it whenever the PDF is
        // compared with one Word made.
        ("word", RevisionStyle::Word),
        (
            "custom",
            RevisionStyle::Custom(RevisionPalette::parse("inserted=#00AA00:double-underline")?),
        ),
    ];
    for (name, revisions) in styles {
        let options = PdfOptions {
            revisions,
            compress: true,
            comments: CommentPlacement::Margin,
            ..PdfOptions::default()
        };
        let pdf = docx_to_pdf_with(&docx, options)?;
        println!(
            "{name}: {} page(s), {} bytes",
            pdf_page_count(&pdf),
            pdf.len()
        );
        if let Some(dir) = &out {
            std::fs::write(format!("{dir}/redline-{name}.pdf"), &pdf)?;
        }
    }

    // One PNG per page at 96 DPI (screen resolution).
    let pages = docx_to_png(&docx, PdfOptions::default(), 96.0)?;
    println!("png: {} page(s)", pages.len());
    if let Some(dir) = &out {
        for (n, png) in pages.iter().enumerate() {
            std::fs::write(format!("{dir}/redline-page-{:02}.png", n + 1), png)?;
        }
    }
    Ok(())
}
// ANCHOR_END: example
