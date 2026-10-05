// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//
// The Finder Sync extension behind "Compare with Jubarte" and "Convert to
// PDF with Jubarte" at the top of Finder's right-click menu, offered by what
// is selected (FinderSyncMenu.swift). It badges nothing and watches no
// folder's contents: the whole disk is its scope so the menu shows anywhere.

import Cocoa
import FinderSync

@objc(FinderSync)
final class FinderSync: FIFinderSync {
    override init() {
        super.init()
        FIFinderSyncController.default().directoryURLs = [URL(fileURLWithPath: "/")]
    }

    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        guard menuKind == .contextualMenuForItems,
              let action = finderAction(for: FIFinderSyncController.default().selectedItemURLs() ?? [])
        else { return nil }
        let menu = NSMenu(title: "")
        let item = menu.addItem(withTitle: action.title, action: #selector(run(_:)), keyEquivalent: "")
        item.image = NSImage(named: "AppIcon")
        return menu
    }

    @objc func run(_ sender: AnyObject?) {
        let urls = FIFinderSyncController.default().selectedItemURLs() ?? []
        guard let action = finderAction(for: urls) else { return }
        let pasteboard = NSPasteboard(name: NSPasteboard.Name("com.jandira.jubarte.finder"))
        pasteboard.clearContents()
        pasteboard.writeObjects(urls as [NSURL])
        NSPerformService(action.service, pasteboard)
    }
}
