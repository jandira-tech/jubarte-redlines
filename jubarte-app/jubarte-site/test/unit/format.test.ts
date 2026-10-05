import { describe, expect, it } from "vitest";
import { nameParts, size } from "../../site/static/js/format.js";

describe("size", () => {
  it("prints small files in kilobytes, so neighbours read alike", () => {
    expect(size(1009)).toBe("1.0 KB");
    expect(size(1100)).toBe("1.1 KB");
    expect(size(0)).toBe("0.0 KB");
  });
  it("rounds from 10 KB and switches to megabytes at 1 MB", () => {
    expect(size(10_240)).toBe("10 KB");
    expect(size(250_000)).toBe("244 KB");
    expect(size(1_048_576)).toBe("1.0 MB");
    expect(size(5_400_000)).toBe("5.1 MB");
  });
});

describe("nameParts", () => {
  it("cuts a name after each _ - and ., keeping every character", () => {
    const name = "NDA_v2_counterparty-final.docx";
    expect(nameParts(name)).toEqual(["NDA_", "v2_", "counterparty-", "final.", "docx"]);
    expect(nameParts(name).join("")).toBe(name);
  });
  it("leaves a name with no separators whole", () => {
    expect(nameParts("Agreement")).toEqual(["Agreement"]);
  });
});
