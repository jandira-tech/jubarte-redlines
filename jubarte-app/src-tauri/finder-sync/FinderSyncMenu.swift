// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//
// What Finder's right-click menu offers for a selection: two Word documents
// compare, one (or three and more) convert to PDF, anything else nothing.
// Each action runs the app's own Finder service (Info.plist NSServices,
// src-tauri/src/finder_service.rs), the one way a sandboxed extension's
// selection reaches the sandboxed app with read access (probed 2026-10-03:
// opening the files in the app from the extension fails with permErr -54).

import Foundation

enum FinderAction: Equatable {
    case compare
    case convert

    var title: String {
        switch self {
        case .compare: return "Compare with Jubarte"
        case .convert: return "Convert to PDF with Jubarte"
        }
    }

    /// The `NSMenuItem` title of the app's service that receives the files.
    var service: String {
        switch self {
        case .compare: return "Redline with Jubarte"
        case .convert: return "Convert to PDF with Jubarte"
        }
    }
}

/// The action for a Finder selection; nil when it holds anything but .docx
/// files (Word's `~$` owner files included).
func finderAction(for urls: [URL]) -> FinderAction? {
    let docx = urls.allSatisfy { url in
        url.pathExtension.lowercased() == "docx" && !url.lastPathComponent.hasPrefix("~$")
    }
    guard !urls.isEmpty, docx else { return nil }
    return urls.count == 2 ? .compare : .convert
}
