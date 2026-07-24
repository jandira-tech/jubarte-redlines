// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word opens a PACKAGE. A clean document.xml with settings/notes out of
//! sync still triggers "unreadable content".

use jubarte::document_comparer::compare_documents;
use std::collections::HashSet;
use std::io::Read;
use zip::ZipArchive;

fn part_ids(
    zip: &mut ZipArchive<std::io::Cursor<Vec<u8>>>,
    part: &str,
    local: &str,
) -> HashSet<String> {
    let Ok(mut f) = zip.by_name(part) else {
        return HashSet::new();
    };
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // crude but sufficient: w:footnote w:id="…" / w:endnote w:id="…"
    let needle = format!("<{local}");
    let alt = format!(":{local}"); // namespaced
    let mut ids = HashSet::new();
    let mut rest = xml.as_str();
    while let Some(i) = rest.find("id=\"") {
        // only count ids that appear on footnote/endnote-ish tags nearby
        let start = rest[..i].rfind('<').unwrap_or(0);
        let tag_region = &rest[start..i];
        if tag_region.contains(local) {
            let after = &rest[i + 4..];
            if let Some(end) = after.find('"') {
                ids.insert(after[..end].to_string());
            }
        }
        rest = &rest[i + 4..];
        let _ = (needle.as_str(), alt.as_str());
    }
    ids
}

fn settings_special_ids(
    zip: &mut ZipArchive<std::io::Cursor<Vec<u8>>>,
    child: &str,
) -> HashSet<String> {
    let Ok(mut f) = zip.by_name("word/settings.xml") else {
        return HashSet::new();
    };
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // inside footnotePr/endnotePr the children are bare <w:footnote w:id="N"/>
    let mut ids = HashSet::new();
    // Find footnotePr or endnotePr blocks
    let pr = if child == "footnote" {
        "footnotePr"
    } else {
        "endnotePr"
    };
    let Some(pr_at) = xml.find(pr) else {
        return ids;
    };
    // take a window after the Pr open until next major close — simple: whole file scan
    // of <w:CHILD w:id=".."/> patterns that are NOT the notes part (settings only has empty ones)
    let mut rest = &xml[pr_at..];
    // stop at next Pr of the other kind or end
    if let Some(stop) = rest[pr.len()..].find("Pr>") {
        // rough: use first 2k of the Pr section
        rest = &rest[..(pr.len() + stop).min(rest.len())];
    }
    let mut s = rest;
    while let Some(i) = s.find("id=\"") {
        let after = &s[i + 4..];
        if let Some(end) = after.find('"') {
            let id = &after[..end];
            // only negative or small specials typically
            ids.insert(id.to_string());
        }
        s = &s[i + 4..];
    }
    ids
}

#[test]
fn package_notes_and_settings_coherent_on_treasury_x_5lb() {
    let a = std::env::var_os("JUBARTE_FIXTURE_A").and_then(|p| std::fs::read(p).ok());
    let b = std::env::var_os("JUBARTE_FIXTURE_B").and_then(|p| std::fs::read(p).ok());
    let (Ok(a), Ok(b)) = (a, b) else {
        // fixtures not present in CI — skip
        eprintln!("fixtures missing; skip package coherence test");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let mut zip = ZipArchive::new(std::io::Cursor::new(out)).unwrap();

    // Package must carry the usual parts Word loads
    for part in [
        "word/document.xml",
        "word/settings.xml",
        "word/styles.xml",
        "word/footnotes.xml",
        "word/endnotes.xml",
        "word/_rels/document.xml.rels",
        "[Content_Types].xml",
    ] {
        assert!(
            zip.by_name(part).is_ok(),
            "package missing {part} — document.xml alone is not a Word doc"
        );
    }

    let fn_ids = part_ids(&mut zip, "word/footnotes.xml", "footnote");
    let en_ids = part_ids(&mut zip, "word/endnotes.xml", "endnote");
    assert!(
        fn_ids.contains("-1") && fn_ids.contains("0"),
        "footnotes need separators: {fn_ids:?}"
    );
    assert!(
        en_ids.contains("-1") && en_ids.contains("0"),
        "endnotes need separators: {en_ids:?}"
    );

    // settings special list ⊆ notes part ids (use helper — CR #3642397974)
    let settings_fn = settings_special_ids(&mut zip, "footnote");
    for id in &settings_fn {
        assert!(
            fn_ids.contains(id),
            "settings footnotePr id={id} missing from footnotes part {fn_ids:?}"
        );
    }
    let settings_en = settings_special_ids(&mut zip, "endnote");
    for id in &settings_en {
        assert!(
            en_ids.contains(id),
            "settings endnotePr id={id} missing from endnotes part {en_ids:?}"
        );
    }

    // no powertools Unid left on notes (package parts Word loads)
    for part in [
        "word/footnotes.xml",
        "word/endnotes.xml",
        "word/document.xml",
    ] {
        let mut s = String::new();
        zip.by_name(part).unwrap().read_to_string(&mut s).unwrap();
        assert!(!s.contains("Unid="), "{part} still carries pt Unid scratch");
    }
}
