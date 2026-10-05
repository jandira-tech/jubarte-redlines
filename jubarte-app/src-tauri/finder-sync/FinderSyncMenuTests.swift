// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//
// Built and run by scripts/build-finder-sync.sh before the extension:
//   swiftc -parse-as-library FinderSyncMenu.swift FinderSyncMenuTests.swift -o menu-tests && ./menu-tests

import Foundation

@main
struct FinderSyncMenuTests {
    static var failures = 0
    static func expect(_ got: FinderAction?, _ want: FinderAction?, _ what: String) {
        if got != want {
            failures += 1
            print("FAIL \(what): got \(String(describing: got)), want \(String(describing: want))")
        }
    }
    static func files(_ names: String...) -> [URL] { names.map { URL(fileURLWithPath: "/Users/a/\($0)") } }

    static func main() {
        expect(finderAction(for: files("draft.docx", "turn.docx")), .compare, "two documents compare")
        expect(finderAction(for: files("draft.docx")), .convert, "one document converts")
        expect(finderAction(for: files("a.docx", "b.docx", "c.docx")), .convert, "three documents convert")
        expect(finderAction(for: files("Draft.DOCX", "Turn.Docx")), .compare, "the extension's case does not matter")
        expect(finderAction(for: files("draft.docx", "notes.pdf")), nil, "a non-Word file offers nothing")
        expect(finderAction(for: files("draft.doc")), nil, "Word 97-2003 .doc offers nothing")
        expect(finderAction(for: files("~$draft.docx")), nil, "Word's owner file offers nothing")
        expect(finderAction(for: []), nil, "no selection offers nothing")
        // The services are the app's Info.plist NSMenuItem titles, word for word.
        for (action, service) in [(FinderAction.compare, "Redline with Jubarte"), (.convert, "Convert to PDF with Jubarte")]
        where action.service != service {
            failures += 1
            print("FAIL \(action) runs \(action.service), want \(service)")
        }

        if failures > 0 { exit(1) }
        print("finder menu: all cases pass")
    }
}
