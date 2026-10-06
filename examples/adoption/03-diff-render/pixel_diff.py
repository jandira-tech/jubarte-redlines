# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Pixel-diff two PNG files with the Python standard library only.

Stands in for `magick compare A B null:` where ImageMagick is absent:
exit 0 when the images are pixel-identical, 1 when any pixel differs
(or the sizes differ), 2 on an unreadable or unsupported file.
Prints one line per comparison: changed pixel count, ratio and bbox.
Only 8-bit non-interlaced grayscale/RGB/RGBA PNGs are supported, which
is what pdftoppm and jubarte write.
"""
import struct
import sys
import zlib


def read_png(path):
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG file")
    width = height = bit_depth = color_type = None
    idat = b""
    i = 8
    while i < len(data):
        length = struct.unpack(">I", data[i : i + 4])[0]
        chunk = data[i + 4 : i + 8]
        payload = data[i + 8 : i + 8 + length]
        i += 12 + length
        if chunk == b"IHDR":
            width, height, bit_depth, color_type = struct.unpack(">IIBB", payload[:10])
        elif chunk == b"IDAT":
            idat += payload
        elif chunk == b"IEND":
            break
    if bit_depth != 8 or color_type not in (0, 2, 6):
        raise ValueError(f"unsupported PNG: depth={bit_depth} color={color_type}")
    channels = {0: 1, 2: 3, 6: 4}[color_type]
    stride = width * channels
    raw = zlib.decompress(idat)
    if len(raw) < height * (stride + 1):
        raise ValueError("truncated PNG data")
    rows = []
    prev = bytearray(stride)
    pos = 0
    for _ in range(height):
        ftype = raw[pos]
        line = bytearray(raw[pos + 1 : pos + 1 + stride])
        pos += 1 + stride
        if ftype == 1:
            for x in range(channels, stride):
                line[x] = (line[x] + line[x - channels]) & 0xFF
        elif ftype == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 0xFF
        elif ftype == 3:
            for x in range(stride):
                left = line[x - channels] if x >= channels else 0
                line[x] = (line[x] + ((left + prev[x]) >> 1)) & 0xFF
        elif ftype == 4:
            for x in range(stride):
                left = line[x - channels] if x >= channels else 0
                up = prev[x]
                ul = prev[x - channels] if x >= channels else 0
                pa, pb, pc = abs(up - ul), abs(left - ul), abs(left + up - 2 * ul)
                pick = left if pa <= pb and pa <= pc else (up if pb <= pc else ul)
                line[x] = (line[x] + pick) & 0xFF
        rows.append(bytes(line))
        prev = line
    return width, height, channels, rows


def main(argv):
    if len(argv) != 3:
        print(f"usage: {argv[0]} A.png B.png", file=sys.stderr)
        return 2
    try:
        aw, ah, ac, arows = read_png(argv[1])
        bw, bh, bc, brows = read_png(argv[2])
    except (OSError, ValueError, zlib.error) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    if (aw, ah) != (bw, bh):
        print(f"{argv[1]} vs {argv[2]}: sizes differ ({aw}x{ah} vs {bw}x{bh})")
        return 1
    changed = 0
    bbox = None
    for y in range(ah):
        ra, rb = arows[y], brows[y]
        if ra == rb:
            continue
        for x in range(aw):
            ca = ra[x * ac : x * ac + min(ac, 3)]
            cb = rb[x * bc : x * bc + min(bc, 3)]
            if ca != cb:
                changed += 1
                px, py = x, y
                if bbox is None:
                    bbox = [px, py, px, py]
                else:
                    bbox[0] = min(bbox[0], px)
                    bbox[1] = min(bbox[1], py)
                    bbox[2] = max(bbox[2], px)
                    bbox[3] = max(bbox[3], py)
    if changed == 0:
        print(f"{argv[1]} vs {argv[2]}: identical ({aw}x{ah})")
        return 0
    ratio = changed / (aw * ah)
    print(
        f"{argv[1]} vs {argv[2]}: {changed} px differ "
        f"({100 * ratio:.2f}%), bbox {bbox}"
    )
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
