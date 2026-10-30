// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Build identity at the native and shared CLI boundaries (integration tests).
#![cfg(feature = "cli")]

use std::process::Command;

mod build_script {
    include!("../build.rs");

    #[test]
    fn checkout_build_script_runs() {
        main();
    }

    #[test]
    fn exported_source_ignores_an_enclosing_git_checkout() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::create_dir_all(root.join(".probe")).unwrap();
        let export = tempfile::tempdir_in(root.join(".probe")).unwrap();
        assert_eq!(build_version(export.path(), "0.15.0"), "0.15.0");
    }

    #[test]
    fn build_identity_tracks_clean_modified_staged_and_untracked_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::create_dir_all(root.join(".probe")).unwrap();
        let repo = tempfile::tempdir_in(root.join(".probe")).unwrap();
        let run = |args: &[&str]| {
            let result = Command::new("git")
                .current_dir(repo.path())
                .args([
                    "-c",
                    "core.hooksPath=missing-hooks",
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    "user.name=Version Test",
                    "-c",
                    "user.email=version@example.invalid",
                ])
                .args(args)
                .output()
                .unwrap();
            assert!(result.status.success(), "{result:?}");
            String::from_utf8(result.stdout).unwrap().trim().to_string()
        };
        run(&["init", "--quiet"]);
        // An unborn HEAD has no commit to embed.
        assert_eq!(build_version(repo.path(), "0.15.0"), "0.15.0");
        std::fs::write(repo.path().join("tracked"), "original").unwrap();
        run(&["add", "tracked"]);
        run(&["commit", "--quiet", "-m", "initial"]);
        let clean = format!("0.15.0+{}", run(&["rev-parse", "--short", "HEAD"]));
        assert_eq!(build_version(repo.path(), "0.15.0"), clean);
        std::fs::write(repo.path().join("tracked"), "modified").unwrap();
        assert_eq!(
            build_version(repo.path(), "0.15.0"),
            format!("{clean}.dirty")
        );
        run(&["add", "tracked"]);
        assert_eq!(
            build_version(repo.path(), "0.15.0"),
            format!("{clean}.dirty")
        );
        run(&["commit", "--quiet", "-m", "modified"]);
        let clean = format!("0.15.0+{}", run(&["rev-parse", "--short", "HEAD"]));
        assert_eq!(build_version(repo.path(), "0.15.0"), clean);
        std::fs::write(repo.path().join("untracked"), "new").unwrap();
        assert_eq!(
            build_version(repo.path(), "0.15.0"),
            format!("{clean}.dirty")
        );
    }
}

fn stdout(argument: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg(argument)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn version_identifies_the_cargo_version_and_git_head() {
    let version = stdout("--version");
    assert!(version.starts_with(concat!("jubarte ", env!("CARGO_PKG_VERSION"))));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    if root.join(".git").exists() {
        let head = Command::new("git")
            .current_dir(root)
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap();
        assert!(head.status.success());
        let sha = String::from_utf8(head.stdout).unwrap();
        assert!(
            version.contains(&format!("+{}", sha.trim())),
            "version must contain +HEAD short sha: {version:?}"
        );
    } else {
        assert_eq!(
            version.trim(),
            concat!("jubarte ", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn help_header_has_the_same_build_identity_as_version() {
    let version = stdout("--version");
    for option in ["--help", "-h"] {
        assert_eq!(stdout(option).lines().next().unwrap(), version.trim());
    }
}

#[test]
fn shared_parser_has_the_same_build_identity_as_native_cli() {
    let parse = |option: &str| -> serde_json::Value {
        serde_json::from_str(&jubarte::cli::parse_json(
            &[option.to_string()],
            "jubarte-redlines",
            &[],
        ))
        .unwrap()
    };
    let version = stdout("--version").replace("jubarte ", "jubarte-redlines ");
    assert_eq!(parse("--version")["text"], version);
    assert_eq!(
        parse("--help")["text"]
            .as_str()
            .unwrap()
            .lines()
            .next()
            .unwrap(),
        version.trim()
    );
}
