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

    tauri_build::build();
}
