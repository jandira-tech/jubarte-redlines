// The sample pair the App page's walkthrough redlines with the real engine:
// two small, Word-valid contract versions written from scratch (no corpus
// material), deterministic byte for byte.

import { writeZip } from "./static/js/zip.js";

export const DEMO_ORIGINAL: string = "MSA — Acme, v3.docx";
export const DEMO_MODIFIED: string = "MSA — Acme, v4 (K. Nguyen).docx";

type Para = { style?: "Title" | "Heading1"; text: string };

const V3: Para[] = [
  { style: "Title", text: "Master Services Agreement" },
  {
    text: "This Agreement is entered into as of January 1, 2026 by and between Acme Corporation (“Client”) and Northwind Legal Services LLC (“Provider”).",
  },
  { style: "Heading1", text: "1. Term." },
  {
    text: "This Agreement shall continue for an initial term of twelve (12) months, and shall automatically renew for successive one-year terms.",
  },
  { style: "Heading1", text: "2. Fees and Payment." },
  { text: "Client shall pay all undisputed invoices within forty-five (45) days of receipt." },
  { text: "Late payments shall accrue interest at 1.0% per month on the unpaid balance." },
  { style: "Heading1", text: "3. Confidentiality." },
  {
    text: "Each party shall protect the other’s Confidential Information using reasonable care and shall not disclose it to any third party.",
  },
  { style: "Heading1", text: "4. Governing Law." },
  { text: "This Agreement is governed by the laws of the State of New York." },
];

const V4: Para[] = [
  { style: "Title", text: "Master Services Agreement" },
  {
    text: "This Agreement is entered into as of March 14, 2026 by and between Acme Corporation (“Client”) and Northwind Legal Services LLC (“Provider”).",
  },
  { style: "Heading1", text: "1. Term." },
  {
    text: "This Agreement shall continue for an initial term of twenty-four (24) months, and shall automatically renew for successive one-year terms unless either party provides written notice of non-renewal at least sixty (60) days prior to the end of the then-current term.",
  },
  { style: "Heading1", text: "2. Fees and Payment." },
  { text: "Client shall pay all undisputed invoices within thirty (30) days of receipt." },
  { style: "Heading1", text: "3. Confidentiality." },
  {
    text: "Each party shall protect the other’s Confidential Information using the same degree of care it uses to protect its own, but no less than reasonable care, and shall not disclose it to any third party.",
  },
  { style: "Heading1", text: "4. Governing Law." },
  { text: "This Agreement is governed by the laws of the State of New York." },
  { text: "Late payments shall accrue interest at 1.0% per month on the unpaid balance." },
];

const W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const PR = "http://schemas.openxmlformats.org/package/2006/relationships";
const xmlHead = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n';

const xmlEsc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function documentXml(paras: Para[]): string {
  const body = paras
    .map((p) => {
      const ppr = p.style ? `<w:pPr><w:pStyle w:val="${p.style}"/></w:pPr>` : "";
      return `<w:p>${ppr}<w:r><w:t xml:space="preserve">${xmlEsc(p.text)}</w:t></w:r></w:p>`;
    })
    .join("");
  return `${xmlHead}<w:document xmlns:w="${W}" xmlns:r="${R}"><w:body>${body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>`;
}

const STYLES = `${xmlHead}<w:styles xmlns:w="${W}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="240"/></w:pPr><w:rPr><w:b/><w:caps/><w:sz w:val="32"/><w:szCs w:val="32"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style></w:styles>`;

function core(creator: string, modified: string): string {
  return `${xmlHead}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>Master Services Agreement</dc:title><dc:creator>${xmlEsc(creator)}</dc:creator><cp:lastModifiedBy>${xmlEsc(creator)}</cp:lastModifiedBy><dcterms:created xsi:type="dcterms:W3CDTF">${modified}</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">${modified}</dcterms:modified></cp:coreProperties>`;
}

function docx(paras: Para[], creator: string, modified: string): Uint8Array {
  return writeZip([
    [
      "[Content_Types].xml",
      `${xmlHead}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/></Types>`,
    ],
    [
      "_rels/.rels",
      `${xmlHead}<Relationships xmlns="${PR}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>`,
    ],
    ["word/document.xml", documentXml(paras)],
    [
      "word/_rels/document.xml.rels",
      `${xmlHead}<Relationships xmlns="${PR}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>`,
    ],
    ["word/styles.xml", STYLES],
    ["docProps/core.xml", core(creator, modified)],
    [
      "docProps/app.xml",
      `${xmlHead}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Jubarte demo</Application></Properties>`,
    ],
  ]);
}

/** The two documents, keyed by the file names the App page shows. */
export function demoDocs(): Record<string, Uint8Array> {
  return {
    [DEMO_ORIGINAL]: docx(V3, "Acme Legal", "2026-09-22T09:00:00Z"),
    [DEMO_MODIFIED]: docx(V4, "K. Nguyen", "2026-09-30T16:30:00Z"),
  };
}
