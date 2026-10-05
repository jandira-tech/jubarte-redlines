import { describe, expect, it } from "vitest";
import { tabKey } from "../../site/static/js/tabs.js";

describe("tabKey: the APG tabs keys, as the index of the tab to select", () => {
  it("moves right and left one tab, wrapping at either end", () => {
    expect(tabKey("ArrowRight", 0, 4)).toBe(1);
    expect(tabKey("ArrowRight", 3, 4)).toBe(0);
    expect(tabKey("ArrowLeft", 0, 4)).toBe(3);
    expect(tabKey("ArrowLeft", 2, 4)).toBe(1);
  });

  it("jumps to the first tab on Home and the last on End, from anywhere", () => {
    for (const i of [0, 1, 3]) {
      expect(tabKey("Home", i, 4)).toBe(0);
      expect(tabKey("End", i, 4)).toBe(3);
    }
  });

  it("toggles a two-tab set with either arrow, as Demo's tabs do", () => {
    expect(tabKey("ArrowRight", 0, 2)).toBe(1);
    expect(tabKey("ArrowLeft", 0, 2)).toBe(1);
    expect(tabKey("ArrowRight", 1, 2)).toBe(0);
  });

  it("leaves every other key to the browser", () => {
    for (const key of ["Tab", "Enter", " ", "ArrowUp", "ArrowDown", "a"]) {
      expect(tabKey(key, 1, 4)).toBeNull();
    }
  });
});
