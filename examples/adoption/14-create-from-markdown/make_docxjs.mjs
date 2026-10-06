// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//
// Builds create_docxjs.docx with the docx npm package (docx-js), the way
// Anthropic's docx skill creates documents: a Node script assembling the
// document object by object. The page size is deliberately NOT set, which
// is the skill's documented footgun: docx-js then defaults to A4, not US
// Letter (see page_size_docxjs.txt for what that produced).
//
// Run from a directory that has node_modules/docx installed (run.sh copies
// this script into a temp dir next to the package). Output path: argv[2].
import {
  Document,
  Packer,
  Paragraph,
  TextRun,
  HeadingLevel,
  AlignmentType,
  Table,
  TableRow,
  TableCell,
  WidthType,
  ShadingType,
} from "docx";
import { writeFileSync } from "node:fs";

const out = process.argv[2];

const cell = (text, { header = false } = {}) =>
  new TableCell({
    width: { size: 3060, type: WidthType.DXA },
    shading: header ? { type: ShadingType.CLEAR, fill: "D9E2F3" } : undefined,
    children: [
      new Paragraph({
        children: [new TextRun({ text, bold: header })],
      }),
    ],
  });

const doc = new Document({
  // NOTE: no `page: { size: ... }` here on purpose — the A4 footgun.
  sections: [
    {
      children: [
        new Paragraph({
          alignment: AlignmentType.CENTER,
          heading: HeadingLevel.HEADING_1,
          children: [new TextRun("Consulting Agreement — Summary")],
        }),
        new Paragraph({
          children: [
            new TextRun("Prepared for Northwind Traders Ltd. by Jandira Technologies, LLC."),
          ],
        }),
        new Paragraph({
          children: [new TextRun("This letter summarizes the engagement agreed on 12 January 2026.")],
        }),
        new Paragraph({ heading: HeadingLevel.HEADING_2, children: [new TextRun("Scope of work")] }),
        new Paragraph({
          children: [
            new TextRun(
              "Jandira will redesign the invoicing workflow and deliver documentation and training for the accounts team.",
            ),
          ],
        }),
        new Paragraph({ children: [new TextRun({ text: "Audit the current invoice flow and tooling", bullet: { level: 0 } })] }),
        new Paragraph({ children: [new TextRun({ text: "Implement the new template set in the accounting system", bullet: { level: 0 } })] }),
        new Paragraph({ children: [new TextRun({ text: "Train the accounts team and hand over runbooks", bullet: { level: 0 } })] }),
        new Paragraph({ heading: HeadingLevel.HEADING_2, children: [new TextRun("Milestones")] }),
        new Table({
          width: { size: 9180, type: WidthType.DXA },
          rows: [
            new TableRow({
              tableHeader: true,
              children: [cell("Milestone", { header: true }), cell("Week", { header: true }), cell("Owner", { header: true })],
            }),
            new TableRow({ children: [cell("Kickoff and audit plan"), cell("1"), cell("Jandira")] }),
            new TableRow({ children: [cell("Template implementation"), cell("3"), cell("Jandira")] }),
            new TableRow({ children: [cell("Training and handover"), cell("6"), cell("Northwind")] }),
          ],
        }),
        new Paragraph({ heading: HeadingLevel.HEADING_2, children: [new TextRun("Terms")] }),
        new Paragraph({
          children: [
            new TextRun(
              "Invoices are payable within fifteen days of receipt. Either party may end the engagement with two weeks' written notice.",
            ),
          ],
        }),
        new Paragraph({
          numbering: { reference: "terms-numbering", level: 0 },
          children: [new TextRun("Work is billed weekly in arrears")],
        }),
        new Paragraph({
          numbering: { reference: "terms-numbering", level: 0 },
          children: [new TextRun("Expenses are pre-approved in writing")],
        }),
      ],
    },
  ],
  numbering: {
    config: [
      {
        reference: "terms-numbering",
        levels: [
          {
            level: 0,
            format: "decimal",
            text: "%1.",
            alignment: AlignmentType.START,
          },
        ],
      },
    ],
  },
});

const buffer = await Packer.toBuffer(doc);
writeFileSync(out, buffer);
