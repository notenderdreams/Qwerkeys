#!/usr/bin/env swift
import AppKit
import Foundation

let root = URL(fileURLWithPath: #file)
    .deletingLastPathComponent()
    .deletingLastPathComponent()

let srcURL = root.appendingPathComponent("assets/app-icon.svg")
let outIcnsURL = root.appendingPathComponent("assets/AppIcon.icns")
let outPngURL = root.appendingPathComponent("assets/app-icon.png")
let iconsetURL = root.appendingPathComponent("packaging/AppIcon.iconset")

let fm = FileManager.default

guard fm.fileExists(atPath: srcURL.path) else {
    fputs("error: \(srcURL.path) not found\n", stderr)
    exit(1)
}

try? fm.removeItem(at: iconsetURL)
try fm.createDirectory(at: iconsetURL, withIntermediateDirectories: true)
defer { try? fm.removeItem(at: iconsetURL) }

guard let data = try? Data(contentsOf: srcURL),
      let svg = NSImage(data: data) else {
    fputs("error: failed to load svg\n", stderr)
    exit(1)
}

func renderPNG(size: Int) -> Data {
    let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: size,
        pixelsHigh: size,
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: size * 4,
        bitsPerPixel: 32
    )!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    svg.draw(
        in: NSRect(x: 0, y: 0, width: size, height: size),
        from: .zero,
        operation: .copy,
        fraction: 1.0
    )
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])!
}

let specs: [(String, Int)] = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]

for (name, size) in specs {
    let pngData = renderPNG(size: size)
    try pngData.write(to: iconsetURL.appendingPathComponent(name))
    if size == 512 && name == "icon_512x512.png" {
        try pngData.write(to: outPngURL)
    }
}

let process = Process()
process.executableURL = URL(fileURLWithPath: "/usr/bin/iconutil")
process.arguments = ["-c", "icns", iconsetURL.path, "-o", outIcnsURL.path]
try process.run()
process.waitUntilExit()

guard process.terminationStatus == 0 else {
    fputs("error: iconutil failed with code \(process.terminationStatus)\n", stderr)
    exit(process.terminationStatus)
}

print("✓ Generated \(outIcnsURL.lastPathComponent) and \(outPngURL.lastPathComponent) from \(srcURL.lastPathComponent)")
