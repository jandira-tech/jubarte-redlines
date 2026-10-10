// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const wasm = createRequire(import.meta.url)("../../npm/node/jubarte_wasm.js");
const received = readFileSync(new URL("../../../tests/fixtures/agent-view/received.docx", import.meta.url));

function read(docx, options = {}) {
  const result = JSON.parse(wasm.readView(docx, JSON.stringify({ pageMarkers: false, ...options })));
  assert.ok(Array.isArray(result.warnings));
  return result.markdown;
}

function body(markdown) {
  assert.ok(markdown.startsWith("---\n"), markdown);
  const end = markdown.indexOf("\n---\n", 4);
  assert.notEqual(end, -1, markdown);
  return markdown.slice(end + 5).trim();
}

function blockIds(markdown) {
  return [...body(markdown).matchAll(/<!-- ([pt]\d+)(?=[ ,])/g)].map((match) => match[1]);
}

function plan(operations) {
  return JSON.stringify({
    schema_version: 1,
    author: "Ann Counsel",
    date: "2026-10-01T09:00:00Z",
    operations,
  });
}

test("readView changed selects changed paragraphs and whole tables in document order", () => {
  const view = read(received, { changed: true });
  assert.deepEqual(blockIds(view), ["p3", "p5", "p7", "t0", "p18"]);
  assert.match(view, /\nrange: changed \(p3, p5, p7, t0, p18\) of p0-p20\n/);
  assert.match(body(view), /\|Deliverable\|Due\|Owner\|/);
  assert.match(body(view), /\|Review call\|Monthly\|Both\|/);
  assert.doesNotMatch(body(view), /Consulting Agreement|<!-- page /);
});

test("readView by resolves names, bare handles and prefixed handles for comment replies", () => {
  const expected = read(received, { changed: true, by: "Arthur Souza Rodrigues" });
  assert.deepEqual(blockIds(expected), ["p5"]);
  assert.match(expected, /range: changed by @AS \(p5\)/);
  assert.match(body(expected), /#c6 @AS re #c5: Disagree/);
  // Filtering keeps the complete block, including other authors' marks.
  assert.match(body(expected), /#3\+4 @AC/);
  for (const by of ["AS", "@AS"]) {
    assert.equal(read(received, { changed: true, by }), expected);
  }
});

test("readView unknown authors never match known handles or document text", () => {
  for (const by of ["Bo", "Cy Young", "@missing", "@", "", "Ann", "AC2", "ann counsel"]) {
    const view = read(received, { changed: true, by });
    assert.equal(body(view), "", by);
    assert.match(view, /range: changed by .* \(none\) of p0-p20/);
  }
});

for (const trackChanges of ["accept", "reject"]) {
  test(`readView changed retains author metadata in the ${trackChanges} view`, () => {
    const view = read(received, { changed: true, by: "AC", trackChanges });
    assert.deepEqual(blockIds(view), ["p3", "p5", "p7", "t0", "p18"]);
    assert.match(body(view), /<!-- p3 rev #0 @AC; #1\+2 @AC -->/);
    assert.doesNotMatch(body(view), /\{(?:\+\+|--|~~)/);
    assert.ok(body(view).includes(trackChanges === "accept" ? "Day 15" : "Day 10"));
  });
}

test("readView changed includes hidden comment-only blocks without their text", () => {
  const view = read(received, { changed: true, comments: "none" });
  assert.deepEqual(blockIds(view), ["p3", "p5", "p7", "t0", "p18"]);
  assert.match(body(view), /<!-- p18 .*comments #c11 -->/);
  assert.doesNotMatch(body(view), /three-year survival|Disagree\./);
});

test("readView changed on an unmarked document is empty and false keeps the full view", () => {
  const docx = wasm.markdownToDocx("Quiet\n\nAlso quiet\n");
  assert.equal(body(read(docx, { changed: true })), "");
  assert.equal(read(docx, { changed: false }), read(docx));
  assert.deepEqual(blockIds(read(docx)), ["p0", "p1"]);
});

test("readView enforces changed selection conflicts even for empty and zero values", () => {
  for (const conflict of [{ paragraphs: "p0" }, { paragraphs: "" }, { head: 1 }, { head: 0 }, { tail: 1 }, { tail: 0 }]) {
    assert.throws(() => read(received, { changed: true, ...conflict }), /changed excludes paragraphs, head and tail/);
  }
  for (const by of ["AC", ""]) {
    assert.throws(() => read(received, { by }), /by needs changed/);
    assert.throws(() => read(received, { changed: false, by }), /by needs changed/);
  }
  for (const options of [{ changed: "true" }, { changed: 1 }, { changed: null }, { changed: true, by: 1 }, { changed: true, by: false }]) {
    assert.throws(() => read(received, options), /readView options/);
  }
});

test("shared CLI parsing applies changed conflicts to read and text aliases", () => {
  for (const command of ["read", "text"]) {
    for (const flags of [["--by", "AC"], ["--changed", "-p", "p0"], ["--changed", "--head", "0"], ["--changed", "--tail", "0"]]) {
      const result = JSON.parse(wasm.parseCli(JSON.stringify([command, "sample.docx", ...flags])));
      assert.equal(result.exit_code, 2, `${command} ${flags}`);
      assert.equal(result.stream, "stderr");
    }
    const result = JSON.parse(wasm.parseCli(JSON.stringify([command, "sample.docx", "--changed", "--by", "@AC"])));
    assert.equal(result.exit_code, 0);
    assert.equal(result.args.changed, true);
    assert.equal(result.args.by, "@AC");
  }
});

test("short selectors edit the intended body and table cells and report canonical ids", () => {
  const original = wasm.markdownToDocx("Intro\n\n| Item | Due |\n|---|---|\n| Report | Friday |\n\nTail\n");
  const result = wasm.applyEditPlan(original, plan([
    { kind: "replace", paragraph: "p0", find: "Intro", replacement: "Opening" },
    { kind: "replace", paragraph: { id: "t0.r1.c1.p0" }, find: "Friday", replacement: "Monday" },
    { kind: "replace", paragraph: "p5", find: "Tail", replacement: "Closing" },
  ]));
  try {
    assert.equal(result.ok, true, result.json);
    assert.deepEqual(JSON.parse(result.json).operations.map((op) => op.paragraph), ["body:p:0", "body:p:4", "body:p:5"]);
    const clean = body(read(result.clean));
    assert.match(clean, /Opening/);
    assert.match(clean, /\|Report\|Monday\|/);
    assert.match(clean, /Closing/);
    assert.doesNotMatch(clean, /Intro|Friday|Tail/);
    assert.match(body(read(original)), /Intro/);
    const tracked = body(read(result.redline, { changed: true, by: "AC" }));
    assert.match(tracked, /Opening/);
    assert.match(tracked, /Monday/);
    assert.match(tracked, /Closing/);
    assert.match(tracked, /@AC/);
  } finally {
    result.free();
  }
});

test("invalid short selectors return useful refusals without output documents", () => {
  const docx = wasm.markdownToDocx("Intro\n\n| Item | Due |\n|---|---|\n| Report | Friday |\n");
  for (const [id, message] of [
    ["p6", "paragraph index 6 does not exist in body (6 paragraphs)"],
    ["t1.r0.c0", "t1 is not a table of this document (1 table)"],
    ["t0.r2.c0", "t0 has 2 rows, no row 2"],
    ["t0.r0.c2", "t0.r0 has 2 cells, no cell 2"],
    ["t0.r1.c1.p1", "t0.r1.c1 has 1 paragraph, no p1"],
    ["t0.r0", "t0: a table id needs a row and a cell"],
    ["p+0", "unknown paragraph id p+0"],
    ["header1", "headers and footers: none"],
  ]) {
    const result = wasm.applyEditPlan(docx, plan([{ kind: "replace", paragraph: id, find: "Intro", replacement: "Changed" }]));
    try {
      assert.equal(result.ok, false, id);
      const error = JSON.parse(result.json);
      assert.equal(error.code, "ANCHOR_NOT_FOUND", id);
      assert.ok(error.message.includes(message), `${id}: ${error.message}`);
      assert.equal(result.clean, undefined);
      assert.equal(result.redline, undefined);
    } finally {
      result.free();
    }
  }
});
