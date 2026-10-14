// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! What the Markdown round trip lost (tests/md_fixed_point.rs, run of
//! 2026-10-14), one shape each: the view written to Word and read back
//! prints the same line.

use std::path::Path;
use std::process::Command;

fn jubarte(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

const HEADER: &str = "---\n\
view: tracked\n\
authors:\n  SW: S Wheeler                    # 2026-09-25T20:04:00Z\n\
---\n\n";

/// `body` under a view header, written to Word and read back: the body
/// lines of the second view, and the docx.
fn round_trip(body: &str) -> (String, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.md"), format!("{HEADER}{body}")).unwrap();
    jubarte(
        dir.path(),
        &[
            "convert",
            "in.md",
            "-o",
            "out.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    let view = jubarte(dir.path(), &["read", "out.docx", "--no-page-markers"]);
    let body = view
        .splitn(3, "---\n")
        .nth(2)
        .unwrap_or_default()
        .to_string();
    (body, std::fs::read(dir.path().join("out.docx")).unwrap())
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut text = String::new();
    std::io::Read::read_to_string(&mut zip.by_name(name).unwrap(), &mut text).unwrap();
    text
}

/// A link inside one tracked change is one change: Word's own shape, a
/// HYPERLINK field inside the one `w:del`, not a `w:hyperlink` that parts
/// the mark in two (`r213` read back as `r213+361`).
#[test]
fn a_link_inside_a_change_stays_one_change() {
    let line =
        "{--Title: [Ida Greaves](https://eur.on.worldcat.org/oclc/1381732921)--}{>>r213 @SW<<}";
    let (body, docx) = round_trip(&format!("<!-- p0 -->\n{line}\n"));
    assert!(body.contains(&format!("{line}\n")), "{body}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    assert!(
        document.contains("HYPERLINK &quot;https://eur.on.worldcat.org/oclc/1381732921&quot;")
            || document.contains("HYPERLINK \"https://eur.on.worldcat.org/oclc/1381732921\""),
        "{document}"
    );
    assert!(document.contains("<w:delInstrText"), "{document}");
    // An inserted link the same; a plain link stays a hyperlink element.
    let inserted = "{++See [the site](https://example.com) now++}{>>r4 @SW<<}";
    let (body, docx) = round_trip(&format!(
        "<!-- p0 -->\n{inserted}\n\n<!-- p1 -->\n[plain](https://example.org)\n"
    ));
    assert!(body.contains(&format!("{inserted}\n")), "{body}");
    assert!(body.contains("[plain](https://example.org)\n"), "{body}");
    let document = part(&docx, "word/document.xml");
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert!(document.contains("<w:instrText"), "{document}");
    assert_eq!(document.matches("<w:hyperlink ").count(), 1, "{document}");
}

/// A heading in a bulleted list prints its bullet before its text
/// (`## - Economics`); the bullet is the list's, not text to write.
#[test]
fn a_bulleted_heading_keeps_one_bullet() {
    let body = "<!-- p0 bullet -->\n## - {--Economics & business--}{>>r211 @SW<<}\n";
    let (got, docx) = round_trip(body);
    assert!(
        got.contains("## - {--Economics & business--}{>>r211 @SW<<}\n"),
        "{got}"
    );
    assert!(!got.contains("## - - "), "{got}");
    assert!(!part(&docx, "word/document.xml").contains(">- <"));
}

/// The definition of the table style `id` in the styles part.
fn table_style(docx: &[u8], id: &str) -> String {
    let styles = part(docx, "word/styles.xml");
    let open = format!("w:styleId=\"{id}\"");
    let at = styles
        .find(&open)
        .unwrap_or_else(|| panic!("no {id}: {styles}"));
    styles[at..].split("</w:style>").next().unwrap().to_string()
}

/// A table under one of Word's built-in table styles (`Grid Table 4 Accent
/// 1`) keeps its look: the view names it by id, and the writer writes
/// Word's own definition, its colours from the document's theme (Word's
/// default theme here), not a bare style with the id for a name.
#[test]
fn a_built_in_table_style_keeps_its_definition() {
    let body = "<!-- t0 2x2, cells p0-p3 by row, style GridTable4-Accent1 -->\n|a|b|\n|-|-|\n|c|d|\n\n\
                <!-- t1 2x2, cells p4-p7 by row, style GridTable6Colorful -->\n|a|b|\n|-|-|\n|c|d|\n";
    let (got, docx) = round_trip(body);
    assert!(got.contains("style GridTable4-Accent1 -->"), "{got}");
    let style = table_style(&docx, "GridTable4-Accent1");
    for want in [
        "<w:name w:val=\"Grid Table 4 Accent 1\"/>",
        "<w:tblStylePr w:type=\"firstRow\">",
        // The header row: accent 1 itself; the bands its 20% tint.
        "w:fill=\"156082\" w:themeFill=\"accent1\"/>",
        "w:fill=\"C1E4F5\" w:themeFill=\"accent1\" w:themeFillTint=\"33\"/>",
        "w:color=\"45B0E1\" w:themeColor=\"accent1\" w:themeTint=\"99\"/>",
    ] {
        assert!(style.contains(want), "{want}: {style}");
    }
    assert!(
        !style.contains("\"@\"") && !style.contains("{C}"),
        "{style}"
    );
    let plain = table_style(&docx, "GridTable6Colorful");
    assert!(
        plain.contains("<w:name w:val=\"Grid Table 6 Colorful\"/>"),
        "{plain}"
    );
    assert!(
        plain.contains("w:color=\"666666\" w:themeColor=\"text1\" w:themeTint=\"99\"/>"),
        "{plain}"
    );
    // A name Word has no definition for stays the bare style it was.
    let (_, docx) = round_trip("<!-- t0 1x1, cells p0-p0 by row, style Ledger -->\n|a|\n|-|\n");
    assert!(table_style(&docx, "Ledger").contains("<w:name w:val=\"Ledger\"/>"));
}

/// A built-in table style's borders are Word's own: the writer no longer
/// warns that it writes the style without them.
#[test]
fn a_built_in_table_style_writes_its_borders_unwarned() {
    let dir = tempfile::tempdir().unwrap();
    let header = "---\nview: tracked\nstyles:\n  Normal: Calibri 11pt  # default\n  table: GridTable4-Accent5, borders vary\n---\n\n";
    let body = "<!-- t0 1x1, cells p0-p0 by row -->\n|a|\n|-|\n";
    for (style, warned) in [("GridTable4-Accent5", false), ("Ledger", true)] {
        let md = format!("{header}{body}").replace("GridTable4-Accent5", style);
        std::fs::write(dir.path().join("t.md"), md).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(["convert", "t.md", "-o", "t.docx", "--force"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(stderr.contains("borders vary"), warned, "{style}: {stderr}");
    }
}
