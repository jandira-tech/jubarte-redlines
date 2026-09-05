// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Meta tests for convert packaging: bundled Carlito/Liberation font assets,
//! their OFL license files, `REUSE.toml` annotations, `Cargo.toml` `include`
//! globs, and cross-file version/changelog consistency.
//!
//! These do not re-test `convert::docx_to_pdf` itself (unchanged in this
//! diff) — only the new/edited files: fonts, licenses, `REUSE.toml`,
//! `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and `jubarte-app`'s
//! `package.json` / `CHANGELOG.md`.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"))
}

fn read_bytes(rel: &str) -> Vec<u8> {
    let path = repo_root().join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"))
}

// ---------------------------------------------------------------------------
// Bundled font assets (assets/fonts/*.ttf)
// ---------------------------------------------------------------------------

const CARLITO_FONTS: &[&str] = &[
    "Carlito-Regular.ttf",
    "Carlito-Bold.ttf",
    "Carlito-Italic.ttf",
    "Carlito-BoldItalic.ttf",
];

const LIBERATION_SANS_FONTS: &[&str] = &[
    "LiberationSans-Regular.ttf",
    "LiberationSans-Bold.ttf",
    "LiberationSans-Italic.ttf",
    "LiberationSans-BoldItalic.ttf",
];

const LIBERATION_SERIF_FONTS: &[&str] = &[
    "LiberationSerif-Regular.ttf",
    "LiberationSerif-Bold.ttf",
    "LiberationSerif-Italic.ttf",
    "LiberationSerif-BoldItalic.ttf",
];

const LIBERATION_MONO_FONTS: &[&str] = &[
    "LiberationMono-Regular.ttf",
    "LiberationMono-Bold.ttf",
    "LiberationMono-Italic.ttf",
    "LiberationMono-BoldItalic.ttf",
];

fn all_bundled_fonts() -> Vec<&'static str> {
    CARLITO_FONTS
        .iter()
        .chain(LIBERATION_SANS_FONTS.iter())
        .chain(LIBERATION_SERIF_FONTS.iter())
        .chain(LIBERATION_MONO_FONTS.iter())
        .copied()
        .collect()
}

fn font_path(name: &str) -> String {
    format!("assets/fonts/{name}")
}

#[test]
fn every_bundled_font_file_exists_and_is_a_plausible_size() {
    for name in all_bundled_fonts() {
        let bytes = read_bytes(&font_path(name));
        assert!(
            bytes.len() > 10_000,
            "{name} is suspiciously small ({} bytes) — looks truncated/placeholder",
            bytes.len()
        );
        assert!(
            bytes.len() < 5_000_000,
            "{name} is suspiciously large ({} bytes)",
            bytes.len()
        );
    }
}

#[test]
fn every_bundled_font_has_a_recognized_sfnt_signature() {
    for name in all_bundled_fonts() {
        let bytes = read_bytes(&font_path(name));
        assert!(bytes.len() >= 4, "{name} too short to have a header");
        let ok = bytes.starts_with(&[0x00, 0x01, 0x00, 0x00]) // TrueType
            || bytes.starts_with(b"OTTO") // OpenType/CFF
            || bytes.starts_with(b"true") // legacy Apple TrueType
            || bytes.starts_with(b"ttcf"); // TrueType collection
        assert!(
            ok,
            "{name} has an unrecognized sfnt signature: {:?}",
            &bytes[..4]
        );
    }
}

#[test]
fn every_bundled_font_parses_with_ttf_parser_and_has_glyphs_and_cmap() {
    for name in all_bundled_fonts() {
        let bytes = read_bytes(&font_path(name));
        let face = ttf_parser::Face::parse(&bytes, 0)
            .unwrap_or_else(|e| panic!("{name} failed to parse with ttf-parser: {e:?}"));
        assert!(
            face.number_of_glyphs() > 0,
            "{name} parsed but reports zero glyphs"
        );
        assert!(
            face.tables().cmap.is_some(),
            "{name} is missing a cmap table (required for text shaping/embedding)"
        );
        assert!(face.units_per_em() > 0, "{name} reports zero units-per-em");
    }
}

#[test]
fn bold_and_italic_variants_are_distinct_from_regular() {
    // Guards against accidentally committing a duplicated/placeholder font
    // file under the wrong style name.
    let families: [&[&str]; 4] = [
        CARLITO_FONTS,
        LIBERATION_SANS_FONTS,
        LIBERATION_SERIF_FONTS,
        LIBERATION_MONO_FONTS,
    ];
    for family in families {
        let &[regular, bold, italic, bold_italic] = family else {
            panic!("expected exactly 4 styles per family");
        };
        let regular_bytes = read_bytes(&font_path(regular));
        for other in [bold, italic, bold_italic] {
            let other_bytes = read_bytes(&font_path(other));
            assert_ne!(
                regular_bytes, other_bytes,
                "{regular} and {other} are byte-for-byte identical"
            );
        }
    }
}

#[test]
fn rustybuzz_can_load_every_bundled_font() {
    // rustybuzz is the shaping engine `convert::docx_to_pdf` relies on; a
    // font that ttf-parser accepts but rustybuzz rejects would silently
    // break shaping at runtime.
    for name in all_bundled_fonts() {
        let bytes = read_bytes(&font_path(name));
        assert!(
            rustybuzz::Face::from_slice(&bytes, 0).is_some(),
            "{name} failed to load via rustybuzz::Face::from_slice"
        );
    }
}

// ---------------------------------------------------------------------------
// Font license files
// ---------------------------------------------------------------------------

#[test]
fn carlito_license_file_is_ofl_and_credits_typoland() {
    let text = read("assets/fonts/LICENSE-Carlito");
    assert!(
        text.contains("SIL OPEN FONT LICENSE"),
        "LICENSE-Carlito missing OFL header"
    );
    assert!(
        text.contains("Version 1.1"),
        "LICENSE-Carlito missing OFL version"
    );
    assert!(
        text.contains("tyPoland"),
        "LICENSE-Carlito missing tyPoland Lukasz Dziedzic attribution"
    );
    assert!(
        text.contains("Carlito"),
        "LICENSE-Carlito does not mention the reserved font name"
    );
}

#[test]
fn liberation_license_file_is_ofl_and_credits_red_hat() {
    let text = read("assets/fonts/LICENSE-Liberation");
    assert!(
        text.contains("SIL OPEN FONT LICENSE"),
        "LICENSE-Liberation missing OFL header"
    );
    assert!(
        text.contains("Version 1.1") || text.contains("1.1"),
        "LICENSE-Liberation missing OFL version"
    );
    assert!(
        text.contains("Red Hat"),
        "LICENSE-Liberation missing Red Hat, Inc. attribution"
    );
    assert!(
        text.contains("Liberation"),
        "LICENSE-Liberation does not mention the reserved font name"
    );
}

// ---------------------------------------------------------------------------
// REUSE.toml — SPDX annotations for the new font assets
// ---------------------------------------------------------------------------

#[test]
fn reuse_toml_covers_carlito_fonts_with_ofl() {
    let text = read("REUSE.toml");
    assert!(
        text.contains("assets/fonts/Carlito-*.ttf"),
        "REUSE.toml missing a glob annotation for Carlito-*.ttf"
    );
    // The Carlito glob annotation and its license text should both declare OFL-1.1.
    let carlito_block_idx = text
        .find("assets/fonts/Carlito-*.ttf")
        .expect("Carlito annotation present");
    let tail = &text[carlito_block_idx..];
    assert!(
        tail[..tail.len().min(400)].contains("OFL-1.1"),
        "Carlito annotation block does not declare SPDX-License-Identifier = \"OFL-1.1\""
    );
}

#[test]
fn reuse_toml_covers_liberation_fonts_with_ofl() {
    let text = read("REUSE.toml");
    assert!(
        text.contains("assets/fonts/Liberation*.ttf"),
        "REUSE.toml missing a glob annotation for Liberation*.ttf"
    );
    let idx = text
        .find("assets/fonts/Liberation*.ttf")
        .expect("Liberation annotation present");
    let tail = &text[idx..];
    assert!(
        tail[..tail.len().min(400)].contains("OFL-1.1"),
        "Liberation annotation block does not declare SPDX-License-Identifier = \"OFL-1.1\""
    );
}

#[test]
fn reuse_toml_covers_both_font_license_files() {
    let text = read("REUSE.toml");
    for path in [
        "assets/fonts/LICENSE-Carlito",
        "assets/fonts/LICENSE-Liberation",
    ] {
        assert!(
            text.contains(path),
            "REUSE.toml is missing an annotation for {path}"
        );
    }
}

// ---------------------------------------------------------------------------
// Cargo.toml packaging (`include`) + new dependencies
// ---------------------------------------------------------------------------

/// Pull out the contents of the `include = [ ... ]` array in `Cargo.toml` as
/// a list of trimmed, quote-stripped entries. Intentionally simple (no toml
/// crate dependency) since we only need the literal strings between the
/// brackets.
fn cargo_toml_include_entries() -> Vec<String> {
    let text = read("Cargo.toml");
    let start = text.find("include = [").expect("include array present");
    let after = &text[start + "include = [".len()..];
    let end = after.find(']').expect("include array closed");
    let body = &after[..end];
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.trim_end_matches(',').trim_matches('"').to_string())
        .collect()
}

#[test]
fn cargo_toml_include_entries_are_all_anchored() {
    // Regression guard for the bug fixed in this PR: cargo `include` globs
    // use gitignore semantics, so an unanchored "README.md" would also match
    // every README.md nested under e.g. jubarte-app/**/node_modules. Every
    // entry must start with "/".
    let entries = cargo_toml_include_entries();
    assert!(!entries.is_empty(), "expected at least one include entry");
    for entry in &entries {
        assert!(
            entry.starts_with('/'),
            "unanchored include pattern {entry:?} can match nested paths outside the crate root"
        );
    }
}

#[test]
fn cargo_toml_include_ships_the_bundled_fonts() {
    let entries = cargo_toml_include_entries();
    assert!(
        entries.iter().any(|e| e == "/assets/fonts/**"),
        "Cargo.toml `include` must ship assets/fonts/** so published crates embed the fonts"
    );
}

#[test]
fn cargo_toml_declares_the_font_and_image_dependencies() {
    let text = read("Cargo.toml");
    for dep in ["ttf-parser", "rustybuzz", "image"] {
        assert!(
            text.contains(dep),
            "Cargo.toml [dependencies] is missing `{dep}`, required by convert::docx_to_pdf"
        );
    }
}

#[test]
fn cargo_lock_is_consistent_with_the_new_dependencies() {
    let lock = read("Cargo.lock").replace("\r\n", "\n");
    for pkg in [
        "name = \"ttf-parser\"",
        "name = \"rustybuzz\"",
        "name = \"image\"",
    ] {
        assert!(
            lock.contains(pkg),
            "Cargo.lock does not contain a resolved entry for {pkg}"
        );
    }
    // The workspace member's own lock entry must track the manifest version.
    let v = env!("CARGO_PKG_VERSION");
    assert!(
        lock.contains(&format!("name = \"jubarte-redlines\"\nversion = \"{v}\"")),
        "Cargo.lock's jubarte-redlines entry is not pinned to {v}"
    );
}

// ---------------------------------------------------------------------------
// Version / changelog consistency across the crate and the desktop app
// ---------------------------------------------------------------------------

/// Desktop app version. The engine crate may move first (0.9.0); the app
/// package is still on its last published line.
const EXPECTED_VERSION: &str = "0.7.1";

#[test]
fn crate_version_matches_expected_release() {
    let v = env!("CARGO_PKG_VERSION");
    let toml = read("Cargo.toml");
    assert!(
        toml.contains(&format!("version = \"{v}\"")),
        "Cargo.toml [package] version must match CARGO_PKG_VERSION ({v})"
    );
}

#[test]
fn root_changelog_has_a_dated_entry_and_release_link_for_the_version() {
    let changelog = read("CHANGELOG.md");
    let heading = format!("## [{EXPECTED_VERSION}]");
    assert!(
        changelog.contains(&heading),
        "CHANGELOG.md is missing a `{heading}` section"
    );
    // The test is named "dated entry", so require the date rather than
    // accepting a bare `## [0.7.1]`. Matched as a shape, not a pinned
    // constant, so cutting a release does not mean editing two places.
    let dated = changelog
        .lines()
        .filter(|line| line.starts_with(&heading))
        .any(|line| {
            let rest = line[heading.len()..].trim_start();
            let Some(date) = rest.strip_prefix("- ") else {
                return false;
            };
            let date = date.trim();
            date.len() == 10
                && date.as_bytes()[4] == b'-'
                && date.as_bytes()[7] == b'-'
                && date.char_indices().all(|(i, c)| {
                    if i == 4 || i == 7 {
                        c == '-'
                    } else {
                        c.is_ascii_digit()
                    }
                })
        });
    assert!(
        dated,
        "CHANGELOG.md `{heading}` heading must carry a `- YYYY-MM-DD` release date"
    );
    let link = format!(
        "[{EXPECTED_VERSION}]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v{EXPECTED_VERSION}"
    );
    assert!(
        changelog.contains(&link),
        "CHANGELOG.md is missing the release-link footer for {EXPECTED_VERSION}"
    );
    // The heading must appear before its own release-link footnote.
    let heading_idx = changelog.find(&heading).unwrap();
    let link_idx = changelog.find(&link).unwrap();
    assert!(
        heading_idx < link_idx,
        "release link footer for {EXPECTED_VERSION} appears before its section heading"
    );
}

#[test]
fn root_changelog_orders_the_new_version_above_the_previous_one() {
    let changelog = read("CHANGELOG.md");
    let idx_071 = changelog.find("## [0.7.1]").expect("0.7.1 section present");
    let idx_070 = changelog.find("## [0.7.0]").expect("0.7.0 section present");
    assert!(
        idx_071 < idx_070,
        "0.7.1 changelog entry must be listed above the older 0.7.0 entry"
    );
}

#[test]
fn jubarte_app_package_json_version_matches_crate_version() {
    let package_json = read("jubarte-app/package.json");
    let needle = format!("\"version\": \"{EXPECTED_VERSION}\"");
    assert!(
        package_json.contains(&needle),
        "jubarte-app/package.json version is not {EXPECTED_VERSION}"
    );
}

#[test]
fn jubarte_app_changelog_documents_the_engine_bump() {
    let changelog = read("jubarte-app/CHANGELOG.md");
    assert!(
        changelog.contains(&format!("[{EXPECTED_VERSION}]")),
        "jubarte-app/CHANGELOG.md is missing a [{EXPECTED_VERSION}] section"
    );
    assert!(
        changelog.contains(&format!("jubarte-redlines {EXPECTED_VERSION}")),
        "jubarte-app/CHANGELOG.md does not mention the jubarte-redlines {EXPECTED_VERSION} engine bump"
    );
}

// ---------------------------------------------------------------------------
// README.md — new PDF-conversion / multi-language-binding documentation
// ---------------------------------------------------------------------------

#[test]
fn readme_documents_the_convert_cli_and_library_entry_point() {
    let readme = read("README.md");
    assert!(
        readme.contains("jubarte convert"),
        "README.md CLI section no longer documents `jubarte convert`"
    );
    assert!(
        readme.contains("convert::docx_to_pdf"),
        "README.md library table no longer documents `convert::docx_to_pdf`"
    );
}

#[test]
fn readme_advertises_python_and_npm_packages() {
    let readme = read("README.md");
    assert!(
        readme.contains("pypi.org/project/jubarte-redlines"),
        "README.md is missing the PyPI package link"
    );
    assert!(
        readme.contains("npmjs.com/package/jubarte-wasm"),
        "README.md is missing the npm package link"
    );
    assert!(
        readme.contains("jubarte-python/"),
        "README.md repository layout no longer lists jubarte-python/"
    );
}
