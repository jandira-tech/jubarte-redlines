import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { privacy } from "../../site/pages/legal.ts";

const html = render(privacy);
const section = (heading: string) => {
  const at = html.indexOf(`<h2>${heading}</h2>`);
  expect(at, heading).toBeGreaterThan(-1);
  const end = html.indexOf("<h2>", at + 4);
  return html.slice(at, end < 0 ? undefined : end);
};
const named = (part: string) =>
  [
    ...part.matchAll(
      /<li><strong>([^<]+)<\/strong>.*?<a href="([^"]+)">Privacy policy<\/a> · <a href="([^"]+)">Terms<\/a><\/li>/g,
    ),
  ].map((m) => m.slice(1));

describe("the privacy policy's providers", () => {
  it("links the privacy policy and terms of each provider that processes data for us", () => {
    expect(named(section("Service providers"))).toEqual([
      [
        "Apple Inc.",
        "https://www.apple.com/legal/privacy/",
        "https://www.apple.com/legal/internet-services/itunes/",
      ],
      [
        "Cloudflare, Inc.",
        "https://www.cloudflare.com/privacypolicy/",
        "https://www.cloudflare.com/website-terms/",
      ],
    ]);
  });

  it("names every service the site sends downloads to, with its policies", () => {
    const reached = named(section("Services you reach through us")).map((r) => r[0]);
    expect(reached).toEqual([
      "GitHub, Inc.",
      "Hugging Face, Inc.",
      "The Rust Foundation (crates.io)",
      "The Python Software Foundation (PyPI)",
      "npm, Inc. (npm)",
    ]);
    for (const [name, privacyUrl, terms] of named(section("Services you reach through us"))) {
      expect(privacyUrl, name).toMatch(/^https:\/\//);
      expect(terms, name).toMatch(/^https:\/\//);
    }
  });

  it("incorporates their terms by reference only where they apply", () => {
    const clause = section("Their terms, incorporated by reference");
    expect(clause).toContain("incorporated into this policy by reference");
    expect(clause).toContain("to the extent they apply to that use");
    // The providers' documents never stand in for our own promises.
    expect(clause).toContain("this policy governs\nwhat we do with it");
  });
});
