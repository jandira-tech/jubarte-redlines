fn main() {
    // Compile and link the Swift StoreKit 2 helper (storekit/) into the binary.
    // macOS-only: StoreKit does not exist on other platforms, and the Rust side
    // gates the FFI behind cfg(target_os = "macos").
    #[cfg(target_os = "macos")]
    {
        use swift_rs::SwiftLinker;

        // SwiftLinker does not auto-invalidate on Swift source changes
        // (CodeRabbit #3583634303) — force rebuild when the bridge moves.
        println!("cargo:rerun-if-changed=storekit");

        // Minimum macOS must match Package.swift (.macOS(.v12)) and
        // tauri.conf.json `minimumSystemVersion`. StoreKit 2 requires 12+.
        SwiftLinker::new("12")
            .with_package("jubarte-storekit", "storekit")
            .link();
        globalize_swift_rs_shim();

        // swift-rs links the Swift runtime but not app-specific system
        // frameworks. The Swift helper imports StoreKit, so link it here.
        println!("cargo:rustc-link-lib=framework=StoreKit");

        // The Swift concurrency runtime (libswift_Concurrency.dylib, pulled in
        // by our async StoreKit code) has an @rpath install name. swift-rs adds
        // build-dir rpaths but not the OS Swift runtime dir, so without this the
        // binary aborts at load: "Library not loaded: @rpath/
        // libswift_Concurrency.dylib". macOS 12+ ships it in /usr/lib/swift
        // (served from the dyld shared cache); this rpath is what an Xcode Swift
        // app with a 12+ deployment target gets automatically.
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }

    engine_version();
    tauri_build::build();
}

/// The version of the engine this app links, for the About window: the
/// `[package]` version of the enclosing jubarte-redlines checkout (the path
/// dependency in Cargo.toml).
fn engine_version() {
    let manifest = std::path::Path::new("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(manifest).expect("the engine's Cargo.toml");
    let version = text
        .split("\n[")
        .find(|table| table.starts_with("package]") || table.starts_with("[package]"))
        .and_then(|table| {
            table.lines().find_map(|line| {
                let value = line.trim().strip_prefix("version")?.trim_start();
                Some(value.strip_prefix('=')?.trim().trim_matches('"').to_owned())
            })
        })
        .expect("a [package] version in the engine's Cargo.toml");
    println!("cargo:rustc-env=JUBARTE_ENGINE_VERSION={version}");
}

/// The C entry points of swift-rs's own runtime shim (`SwiftRs.o`), which the
/// Rust half of swift-rs calls.
#[cfg(target_os = "macos")]
const SWIFT_RS_SHIM: [&str; 4] = [
    "_retain_object",
    "_release_object",
    "_data_from_bytes",
    "_string_from_bytes",
];

/// Xcode 27's SwiftPM (the Swift Build backend) internalizes `@_cdecl`
/// functions in release static products: `nm` lists them as local `t`. swift-rs
/// 1.0.8 re-globalizes only the consuming package's own member, never
/// `SwiftRs.o` (Brendonovich/swift-rs#81). So a release build fails to link
/// with "Undefined symbols: _retain_object, _string_from_bytes", while debug
/// links. Promote the shim's four symbols back to global, as swift-rs does
/// for ours. A no-op once they are global (debug, or a fixed swift-rs).
#[cfg(target_os = "macos")]
fn globalize_swift_rs_shim() {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn archives(dir: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                archives(&path, found);
            } else if path
                .file_name()
                .is_some_and(|n| n == "libjubarte-storekit.a")
            {
                found.push(path);
            }
        }
    }

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let mut found = Vec::new();
    archives(&out.join("swift-rs").join("jubarte-storekit"), &mut found);
    for archive in found {
        let nm = Command::new("nm").arg(&archive).output().expect("nm");
        let local: Vec<&str> = String::from_utf8_lossy(&nm.stdout)
            .lines()
            .filter_map(
                |line| match line.split_whitespace().collect::<Vec<_>>()[..] {
                    [_, "t", name] => SWIFT_RS_SHIM.iter().find(|s| **s == name).copied(),
                    _ => None,
                },
            )
            .collect();
        if local.is_empty() {
            continue;
        }
        // Cargo's own compiler, so the sysroot is the toolchain building us.
        let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
        let sysroot = Command::new(rustc)
            .args(["--print", "sysroot"])
            .output()
            .expect("rustc --print sysroot");
        let objcopy = Path::new(String::from_utf8_lossy(&sysroot.stdout).trim())
            .join("lib/rustlib")
            .join(format!("{}-apple-darwin", std::env::consts::ARCH))
            .join("bin/llvm-objcopy");
        assert!(
            objcopy.exists(),
            "swift-rs runtime symbols {local:?} are internalized in {} and llvm-objcopy is \
             missing: run `rustup component add llvm-tools`",
            archive.display()
        );
        let mut cmd = Command::new(objcopy);
        for name in &local {
            cmd.arg(format!("--globalize-symbol={name}"));
        }
        let status = cmd.arg(&archive).status().expect("llvm-objcopy");
        assert!(
            status.success(),
            "llvm-objcopy failed on {}",
            archive.display()
        );
    }
}
