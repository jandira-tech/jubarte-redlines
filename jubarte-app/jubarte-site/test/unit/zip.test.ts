import { describe, expect, it } from "vitest";
import { crc32, entries, readEntry, readText, writeZip } from "../../site/static/js/zip.js";

describe("zip", () => {
  it("computes the standard CRC-32", () => {
    expect(crc32(new TextEncoder().encode("123456789"))).toBe(0xcbf43926);
  });

  it("reads back what it writes, names and bytes", async () => {
    const bin = new Uint8Array([0, 1, 2, 250, 255]);
    const zip = writeZip([
      ["[Content_Types].xml", "<Types/>"],
      ["word/document.xml", "<w:document>Ação</w:document>"],
      ["media/x.bin", bin],
    ]);
    expect([...entries(zip).keys()]).toEqual([
      "[Content_Types].xml",
      "word/document.xml",
      "media/x.bin",
    ]);
    expect(await readText(zip, "word/document.xml")).toBe("<w:document>Ação</w:document>");
    expect(await readEntry(zip, "media/x.bin")).toEqual(bin);
    expect(await readEntry(zip, "missing.xml")).toBeNull();
  });

  it("writes the same bytes for the same input", () => {
    const parts: [string, string][] = [["a.txt", "a"]];
    expect(writeZip(parts)).toEqual(writeZip(parts));
  });

  it("rejects bytes that are not a zip", () => {
    expect(() => entries(new Uint8Array(64))).toThrow("not a zip file");
  });
});
