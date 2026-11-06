// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The agent examples name paragraphs with the settled spelling (`p3`), and
//! each plan that is bound to a source applies.

use std::fs;
use std::path::Path;
use std::process::Command;

use jubarte::edit::{EditPlan, apply_plan};

fn agents() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/agents")
}

fn letter() -> Vec<u8> {
    let dir = std::env::temp_dir().join(format!("jubarte-agent-letter-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("a temp directory");
    let out = dir.join("letter.docx");
    let status = Command::new("python3")
        .arg(agents().join("acme-letter/make_letter.py"))
        .arg(&out)
        .status()
        .expect("python3 runs");
    assert!(status.success(), "the letter is written, got {status}");
    fs::read(&out).expect("the letter is readable")
}

fn plan(path: &Path) -> EditPlan {
    EditPlan::from_json(&fs::read_to_string(path).expect("the plan is readable"))
        .expect("the plan parses")
}

/// `examples/agents` plans that name a real source apply, and none of those
/// examples still spell a paragraph `body:p:N`.
#[test]
fn agent_example_plans_apply() {
    let root = agents();
    let reviewed = apply_plan(
        &letter(),
        &plan(&root.join("comment-thread/plan-1-review.json")),
    );
    assert!(reviewed.is_ok(), "{}", reviewed.unwrap_err());
    let replied = apply_plan(
        &reviewed.unwrap().clean,
        &plan(&root.join("comment-thread/plan-2-reply.json")),
    );
    assert!(replied.is_ok(), "{}", replied.unwrap_err());

    let mut retired = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("the examples are readable") {
            let entry = entry.expect("an example entry");
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let bytes = fs::read(&path).expect("an example file");
            if bytes.windows(7).any(|window| window == b"body:p:") {
                retired.push(path);
            }
        }
    }
    assert!(
        retired.is_empty(),
        "retired paragraph spelling in {retired:?}"
    );
}
