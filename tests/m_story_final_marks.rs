// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word pairs the two documents' final paragraph marks. The revised list ends
//! at "Ωω Omega"; the original goes on with "Meeting Agenda" and a table.
//! Word joins "Ωω Omega" to "Meeting Agenda" under the original's paragraph
//! properties with a deleted mark, and the story-final paragraph after the
//! deleted table takes the revised properties. Full LCS gave the joined
//! paragraph the revised style and a live mark and left the final paragraph
//! plain, one line higher than Word.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[test]
fn revised_final_mark_pairs_with_original_final_mark() {
    let src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(src.join("file_205.docx")).unwrap(),
        &std::fs::read(src.join("file_206.docx")).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let joined = xml
        .split("</w:p>")
        .find(|p| p.contains("Meeting Agenda"))
        .expect("joined paragraph");
    assert!(
        joined.contains("Omega") && !joined.contains("PreformattedText"),
        "joined paragraph should keep the original's properties: {joined}"
    );
    let tail = &xml[xml.rfind("</w:tbl>").expect("deleted table")..];
    assert!(
        tail.contains("PreformattedText"),
        "story-final paragraph should carry the revised properties: {tail}"
    );
}
