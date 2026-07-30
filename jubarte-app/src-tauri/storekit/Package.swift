// swift-tools-version:5.9
//
// Jubarte StoreKit 2 bridge — a *static* Swift library linked into the Tauri
// macOS binary by `swift-rs` (see ../build.rs). It exposes four @_cdecl entry
// points that the Rust side calls through the `swift!` macro.
//
// Kept at swift-tools-version 5.9 on purpose: that leaves the package in the
// Swift 5 language mode, so the DispatchSemaphore + shared-box bridge used to
// turn StoreKit 2's async APIs into synchronous @_cdecl calls does not trip
// Swift 6 strict-concurrency errors. The bridge is only ever entered from a
// Rust background thread (Tauri `spawn_blocking`), never the UI thread.
import PackageDescription

let package = Package(
    name: "jubarte-storekit",
    platforms: [
        // StoreKit 2 (Product, Transaction, VerificationResult, AppStore.sync)
        // is macOS 12+. Must match SwiftLinker::new("12") in build.rs and
        // `minimumSystemVersion` in tauri.conf.json.
        .macOS(.v12),
    ],
    products: [
        .library(
            name: "jubarte-storekit",
            type: .static,
            targets: ["jubarte-storekit"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/Brendonovich/swift-rs", from: "1.0.7"),
    ],
    targets: [
        .target(
            name: "jubarte-storekit",
            dependencies: [
                .product(name: "SwiftRs", package: "swift-rs"),
            ],
            path: "Sources/jubarte-storekit"
        ),
    ]
)
