// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The cargo-deny build-script list is a closed inventory.
//!
//! An omitted `allow-build-scripts` key allows every new build script.
//! The list names the crates whose scripts may run. It is not a sandbox.

use std::path::Path;

#[test]
fn build_scripts_are_a_closed_inventory() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("deny.toml");
    let text = std::fs::read_to_string(&path).expect("read deny.toml");
    let lines: Vec<&str> = text.lines().collect();
    let header = lines
        .iter()
        .position(|line| line.trim() == "[bans.build]")
        .unwrap_or_else(|| panic!("deny.toml has no [bans.build] inventory"));
    let start = header.saturating_sub(8);
    let end = (header + 24).min(lines.len());
    let window = lines[start..end].join("\n");
    let lower = window.to_ascii_lowercase();
    assert!(
        lower.contains("inventory") && lower.contains("not a sandbox"),
        "the build-script section must say this inventory is not a sandbox:\n{window}"
    );
    let section_end = lines
        .iter()
        .enumerate()
        .skip(header + 1)
        .find(|(_, line)| line.starts_with('['))
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    let section = lines[header..section_end].join("\n");
    let names = quoted_allow_names(&section);
    assert!(
        !names.is_empty(),
        "allow-build-scripts is missing or empty:\n{section}"
    );
    assert!(
        names.contains(&"libmimalloc-sys"),
        "the native allocator exception libmimalloc-sys is not in the inventory: {names:?}"
    );
    assert!(
        names.contains(&"constant_time_eq"),
        "constant_time_eq's rustc probe is not in the inventory: {names:?}"
    );
}

/// Quoted crate names in the `allow-build-scripts` array. Comments are skipped.
fn quoted_allow_names(section: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut in_list = false;
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("allow-build-scripts") {
            in_list = true;
        }
        if !in_list {
            continue;
        }
        if let Some(name) = quoted(trimmed) {
            names.push(name);
        }
        if trimmed == "]" {
            break;
        }
    }
    names
}

fn quoted(line: &str) -> Option<&str> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}
