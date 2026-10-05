// Invoke Finder's "Redline with Jubarte" or "Convert to PDF with Jubarte"
// service exactly as Finder does: the files go on a pasteboard as file URLs
// and AppKit routes them to the app that declares the service (launching it if
// needed).
//
//   swift scripts/finder-service-smoke.swift ORIGINAL.docx MODIFIED.docx
//   swift scripts/finder-service-smoke.swift --convert A.docx [B.docx …]
//
// Exit 0 when AppKit delivered the request; the app should then come forward
// with both slots filled and the redline running, or converting each file.

import AppKit

var paths = Array(CommandLine.arguments.dropFirst())
let convert = paths.first == "--convert"
if convert { paths.removeFirst() }
let service = convert ? "Convert to PDF with Jubarte" : "Redline with Jubarte"
guard !paths.isEmpty else {
    FileHandle.standardError.write("usage: swift finder-service-smoke.swift [--convert] A.docx [B.docx]\n".data(using: .utf8)!)
    exit(2)
}

let board = NSPasteboard.withUniqueName()
board.clearContents()
let urls = paths.map { URL(fileURLWithPath: $0).standardizedFileURL as NSURL }
guard board.writeObjects(urls) else {
    FileHandle.standardError.write("could not write the file URLs to a pasteboard\n".data(using: .utf8)!)
    exit(1)
}

let delivered = NSPerformService(service, board)
print(delivered ? "delivered \(urls.count) file(s) to \(service)" : "service not found or refused")
exit(delivered ? 0 : 1)
