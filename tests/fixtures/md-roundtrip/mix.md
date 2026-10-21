---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 226                     # 213 marks shown as 123 changes: 2 insertions, 85 deletions, 2 substitutions, 21 paragraph marks, 13 table rows; 5 formatting; 2 in header1; 2 in footer1; 2 in endnotes; 2 in footnotes
comments: 1 thread open, style Normal  # 1 comment: c73
authors:
  document_owner: Un-named         # dc:creator
  ASR: Arthur Souza Rodrigues      # 226 revisions, 2026-09-26T14:54:00-04:00
  U: User                          # 1 comment, 2026-03-05T19:41:00Z
body: p0-p110, 6 tables, 3 pages   # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in; fmt r226 @ASR
styles:
  Normal: Inter 11pt, left         # default; unannotated paragraphs use it
  ListParagraph: Inter 11pt, named List Paragraph
  table: TableNormal
notes: {fmt: footnotes.r0 @ASR; footnotes.r1 @ASR; endnotes.r2 @ASR; endnotes.r3 @ASR, empty: [footnotes, endnotes]}  # changes in notes the body does not show (separators)
headers:
  default: {id: header1, text: "{--Docx Editor: Project Charter & Contributor Agreement       --}{header1.r223 @ASR}", style: Normal, fmt: header1.r222 @ASR in header1}
footers:
  default: {id: footer1, text: "{--Page       {PAGE \\* MERGEFORMAT}1       {/}of       {NUMPAGES \\* MERGEFORMAT}1       {/}·       {HYPERLINK \\h}eigenpal.com         {/}--}{footer1.r225 @ASR}", style: ListParagraph, fmt: footer1.r224 @ASR in footer1}
---

<!-- p0 fmt r4 @ASR; r5 @ASR; r6 @ASR; r9 @ASR -->
_file\__{++_15_++}{r7+8 @ASR}{--**14**--}{r7+8 @ASR}_.docx_

<!-- p1 -->
{++_Verdana Italic Centered Demo_++}{r11+10 @ASR}¶{r11+10 @ASR}

<!-- p2 -->
{++_This document shows Verdana font with italic and center alignment._++}{r13+12 @ASR}¶{r13+12 @ASR}

<!-- p3 after 1pt, break-del r15+16+14 @ASR -->
{++_Verdana italic centered creates a clean and modern look._++}{r15+16+14 @ASR}{--**eigenpal/docx-editor**         --}{r15+16+14 @ASR}

<!-- p4 after 3pt, size 12pt, break-del r18+17 @ASR -->
{--Project Charter & Contributor Agreement         --}{r18+17 @ASR}

<!-- p5 empty, after 3pt, break-del r19 @ASR -->

<!-- t0 1x2, cells p6-p7 by row, width 6.5in, indent -0.003in, borders single 0.5pt, break-del r22+21 @ASR in p6; r24+23 @ASR in p7, rev-del r20 @ASR in t0.row0 -->
|{--npm               [<u>` @eigenpal/docx-js-editor                  `</u>](https://www.npmjs.com/package/@eigenpal/docx-js-editor)--}{r22+21 @ASR}|{--github               [<u>` eigenpal/docx-editor                  `</u>](https://github.com/eigenpal/docx-editor)--}{r24+23 @ASR}|
|-|-|

<!-- p8 before 15pt, after 4pt, size 16pt, break-del r26+25 @ASR -->
{--**1. What this is**         --}{r26+25 @ASR}

<!-- p9 before 3pt, after 3pt, break-del r28+27 @ASR -->
{--An open-source WYSIWYG         ` .docx          `editor for React. Drop in         ` <DocxEditor documentBuffer={file} />          `and your users can open, edit, and save Word documents entirely in the browser —         no backend required           , no document data leaving the client.         --}{r28+27 @ASR}

<!-- p10 empty, before 3pt, break-del r29 @ASR -->

<!-- p11 before 3pt, after 3pt, break-del r31+30 @ASR -->
{--Most editors serialise to HTML and back:         ` DOCX → HTML → edits → HTML → DOCX          `. Every hop loses formatting, tracked changes, and styles.         This library skips it           — OOXML is the canonical format throughout.         --}{r31+30 @ASR}

<!-- p12 empty, before 4pt, break-del r32 @ASR -->

<!-- t1 1x1, cells t1.row0 p13-p23, p13-p14 size 9.5pt, p16-p23 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r35+34 @ASR in p13; r37+36 @ASR in p14; r38 @ASR in p15; r40+39 @ASR in p16; r42+41 @ASR in p17; r44+43 @ASR in p18; r46+45 @ASR in p19; r48+47 @ASR in p20; r50+49 @ASR in p21; r52+51 @ASR in p22; r54+53 @ASR in p23, rev-del r33 @ASR in t1.row0 -->
|{--` import { DocxEditor } from '@eigenpal/docx-js-editor'                `--}{r35+34 @ASR}{--<br>` import '@eigenpal/docx-js-editor/styles.css'                `--}{r37+36 @ASR}<br>{--<br>` function App({ file }: { file: ArrayBuffer }) {                `--}{r40+39 @ASR}{--<br>` return (                `--}{r42+41 @ASR}{--<br>` <DocxEditor                `--}{r44+43 @ASR}{--<br>` documentBuffer={file}                `--}{r46+45 @ASR}{--<br>` onSave={(buffer) => uploadToServer(buffer)}                `--}{r48+47 @ASR}{--<br>` />                `--}{r50+49 @ASR}{--<br>` )                `--}{r52+51 @ASR}{--<br>` }                `--}{r54+53 @ASR}|
|-|

<!-- p24 empty, before 8pt, break-del r55 @ASR -->

<!-- p25 before 15pt, after 4pt, size 16pt, break-del r57+56 @ASR -->
{--**2. Collaboration — Tracked Changes & Comments**         --}{r57+56 @ASR}

<!-- p26 before 3pt, after 3pt, break-del r59+58 @ASR -->
{--Both features round-trip through OOXML so documents open correctly in Word, LibreOffice, and Google Docs.         --}{r59+58 @ASR}

<!-- p27 empty, before 4pt, break-del r60 @ASR -->

<!-- p28 before 9pt, after 2pt, break-del r62+61 @ASR -->
{--**Suggestion mode**         --}{r62+61 @ASR}

<!-- p29 before 3pt, after 3pt, break-del r64+63 @ASR -->
{--Toggle between         ` Editing          `and         ` Suggesting          `via the toolbar dropdown. In Suggesting mode, every edit becomes a tracked insertion (         green underline         ) or deletion (         red strikethrough         ). Consecutive keystrokes group into a single change. Accept or reject individually or all at once.         --}{r64+63 @ASR}

<!-- p30 empty, before 3pt, break-del r65 @ASR -->

<!-- p31 before 3pt, after 3pt, break-del r67+66 @ASR -->
{--Serialises to         ` w:ins          `/         ` w:del          `— the same OOXML markup Word uses. Tracked changes survive a save/reload cycle and are reviewable in Word with full accept/reject UI.         --}{r67+66 @ASR}

<!-- p32 empty, before 4pt, break-del r68 @ASR -->

<!-- p33 before 9pt, after 2pt, break-del r70+69 @ASR -->
{--**Comments**         --}{r70+69 @ASR}

<!-- p34 before 3pt, after 3pt, break-del r72+71 @ASR -->
{--Select text and click the floating button on the page edge to         {==add a comment anchored to that selection         ==}{>>c73 @U: Just like that!<<}. A Google Docs-style sidebar shows threaded cards positioned at their anchor point in the document. Reply, resolve, reopen, or delete inline.         --}{r72+71 @ASR}

<!-- p35 empty, before 3pt, break-del r74 @ASR -->

<!-- p36 before 3pt, after 3pt, break-del r76+75 @ASR -->
{--Serialises to         ` comments.xml          `with         ` w:commentRangeStart          `/         ` w:commentRangeEnd          `markers. Comments and reply threads are preserved on round-trip.         --}{r76+75 @ASR}

<!-- p37 empty, before 4pt, break-del r77 @ASR -->

<!-- p38 before 9pt, after 2pt, break-del r79+78 @ASR -->
{--**Agent-friendly**         --}{r79+78 @ASR}

<!-- p39 before 3pt, after 3pt, break-del r81+80 @ASR -->
{--Because tracked changes and comments are native OOXML, an AI agent can write         ` w:ins          `/         ` w:del          `/         ` w:comment          `markup directly into a document and hand it to         ` <DocxEditor />          `for human review — no custom diff format, no proprietary API.         --}{r81+80 @ASR}

<!-- p40 empty, before 8pt, break-del r82 @ASR -->

<!-- p41 before 15pt, after 4pt, size 16pt, break-del r84+83 @ASR -->
{--**3. Props**         --}{r84+83 @ASR}

<!-- p42 before 3pt, after 3pt, break-del r86+85 @ASR -->
{--Pass         ` author="Your App"          `to set the author name for comments and tracked changes. Toggle suggestion mode via the toolbar dropdown.         --}{r86+85 @ASR}

<!-- p43 empty, before 3pt, break-del r87 @ASR -->

<!-- t2 8x4, cells p44-p75 by row, p44-p50 size 9.5pt, p51 size 10pt, p52-p54 size 9.5pt, p55 size 10pt, p56-p58 size 9.5pt, p59 size 10pt, p60-p62 size 9.5pt, p63 size 10pt, p64-p66 size 9.5pt, p67 size 10pt, p68-p70 size 9.5pt, p71 size 10pt, p72-p74 size 9.5pt, p75 size 10pt, widths 1.309in 1.191in 0.67in 3.33in, width 6.5in, indent -0.003in, borders single 0.5pt, break-del r90+89 @ASR in p44; r92+91 @ASR in p45; r94+93 @ASR in p46; r96+95 @ASR in p47; r99+98 @ASR in p48; r101+100 @ASR in p49; r103+102 @ASR in p50; r105+104 @ASR in p51; r108+107 @ASR in p52; r110+109 @ASR in p53; r112+111 @ASR in p54; r114+113 @ASR in p55; r117+116 @ASR in p56; r119+118 @ASR in p57; r121+120 @ASR in p58; r123+122 @ASR in p59; r126+125 @ASR in p60; r128+127 @ASR in p61; r130+129 @ASR in p62; r132+131 @ASR in p63; r135+134 @ASR in p64; r137+136 @ASR in p65; r139+138 @ASR in p66; r141+140 @ASR in p67; r144+143 @ASR in p68; r146+145 @ASR in p69; r148+147 @ASR in p70; r150+149 @ASR in p71; r153+152 @ASR in p72; r155+154 @ASR in p73; r157+156 @ASR in p74; r159+158 @ASR in p75, rev-del r88 @ASR in t2.row0; r97 @ASR in t2.row1; r106 @ASR in t2.row2; r115 @ASR in t2.row3; r124 @ASR in t2.row4; r133 @ASR in t2.row5; r142 @ASR in t2.row6; r151 @ASR in t2.row7 -->
|{--**Prop**               --}{r90+89 @ASR}|{--**Type**               --}{r92+91 @ASR}|{--**Default**               --}{r94+93 @ASR}|{--**Description**               --}{r96+95 @ASR}|
|-|-|-|-|
|{--` documentBuffer                `--}{r99+98 @ASR}|{--` ArrayBuffer                `--}{r101+100 @ASR}|{--` —                `--}{r103+102 @ASR}|{--.docx file contents to load               --}{r105+104 @ASR}|
|{--` readOnly                `--}{r108+107 @ASR}|{--` boolean                `--}{r110+109 @ASR}|{--` false                `--}{r112+111 @ASR}|{--Preview mode — no editing UI               --}{r114+113 @ASR}|
|{--` showToolbar                `--}{r117+116 @ASR}|{--` boolean                `--}{r119+118 @ASR}|{--` true                `--}{r121+120 @ASR}|{--Show formatting toolbar               --}{r123+122 @ASR}|
|{--` author                `--}{r126+125 @ASR}|{--` string                `--}{r128+127 @ASR}|{--` 'User'                `--}{r130+129 @ASR}|{--Author name for comments and tracked changes               --}{r132+131 @ASR}|
|{--` onChange                `--}{r135+134 @ASR}|{--` function                `--}{r137+136 @ASR}|{--` —                `--}{r139+138 @ASR}|{--Called on every document change               --}{r141+140 @ASR}|
|{--` onSave                `--}{r144+143 @ASR}|{--` function                `--}{r146+145 @ASR}|{--` —                `--}{r148+147 @ASR}|{--Called on save, receives ArrayBuffer               --}{r150+149 @ASR}|
|{--` onError                `--}{r153+152 @ASR}|{--` function                `--}{r155+154 @ASR}|{--` —                `--}{r157+156 @ASR}|{--Called on error               --}{r159+158 @ASR}|

<!-- p76 empty, before 4pt, break-del r160 @ASR -->

<!-- p77 page-break-del r162+161 @ASR, break-del r162+161 @ASR -->

<!-- p78 before 9pt, after 2pt, break-del r164+163 @ASR -->
{--**Ref methods**         --}{r164+163 @ASR}

<!-- t3 1x1, cells t3.row0 p79-p85, p79 size 9.5pt, p81-p85 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r167+166 @ASR in p79; r168 @ASR in p80; r170+169 @ASR in p81; r172+171 @ASR in p82; r174+173 @ASR in p83; r176+175 @ASR in p84; r178+177 @ASR in p85, rev-del r165 @ASR in t3.row0 -->
|{--` const ref = useRef<DocxEditorRef>(null)                `--}{r167+166 @ASR}<br>{--<br>` await ref.current.save() // ArrayBuffer of the .docx                `--}{r170+169 @ASR}{--<br>` ref.current.getDocument() // current document object                `--}{r172+171 @ASR}{--<br>` ref.current.setZoom(1.5) // set zoom to 150%                `--}{r174+173 @ASR}{--<br>` ref.current.scrollToPage(3) // scroll to page 3                `--}{r176+175 @ASR}{--<br>` ref.current.print() // print                `--}{r178+177 @ASR}|
|-|

<!-- p86 empty, before 8pt, break-del r179 @ASR -->

<!-- p87 before 15pt, after 4pt, size 16pt, break-del r181+180 @ASR -->
{--**4. Contributing**         --}{r181+180 @ASR}

<!-- p88 before 3pt, after 3pt, break-del r183+182 @ASR -->
{--Read         [<u>` docs/PLUGINS.md            `</u>](https://github.com/eigenpal/docx-editor/blob/main/docs/PLUGINS.md)before opening a pull request that touches the plugin API.         --}{r183+182 @ASR}

<!-- p89 empty, before 3pt, break-del r184 @ASR -->

<!-- p90 before 9pt, after 2pt, break-del r186+185 @ASR -->
{--**Ground rules**         --}{r186+185 @ASR}

<!-- p91 ListParagraph before 2pt, after 2pt, bullet, break-del r188+187 @ASR -->
- {--Every PR must pass the round-trip test: load a .docx, apply an edit, export, reload — content and formatting must be stable on unchanged content.         --}{r188+187 @ASR}

<!-- p92 ListParagraph before 2pt, after 2pt, bullet, break-del r190+189 @ASR -->
- {--Run         ` bun test          `and         ` bun run test:e2e          `locally before pushing.         --}{r190+189 @ASR}

<!-- p93 ListParagraph before 2pt, after 2pt, bullet, break-del r192+191 @ASR -->
- {--Avoid intermediate HTML serialisation where possible.           --}{r192+191 @ASR}

<!-- p94 ListParagraph before 2pt, after 2pt, bullet, break-del r194+193 @ASR -->
- {--Tracked changes and comments must survive a full OOXML round-trip — save, open in Word, verify changes are reviewable, reload in the editor.         --}{r194+193 @ASR}

<!-- p95 empty, before 4pt, break-del r195 @ASR -->

<!-- p96 before 9pt, after 2pt, break-del r197+196 @ASR -->
{--**Dev setup**         --}{r197+196 @ASR}

<!-- t4 1x1, cells t4.row0 p97-p98, p97-p98 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r200+199 @ASR in p97; r202+201 @ASR in p98, rev-del r198 @ASR in t4.row0 -->
|{--` bun install                `--}{r200+199 @ASR}{--<br>` bun run dev // Vite demo on localhost:5173                `--}{r202+201 @ASR}|
|-|

<!-- p99 empty, before 10pt, break-del r203 @ASR -->

<!-- p100 before 15pt, after 4pt, size 16pt, break-del r205+204 @ASR -->
{--**Sign-off**         --}{r205+204 @ASR}

<!-- p101 empty, before 3pt, break-del r206 @ASR -->

<!-- t5 1x2, cells t5.row0 p102-p109, p102 after 0.4pt size 12pt, p103 after 16pt size 10pt, p104 after 3pt, p105 size 10pt, p106 after 0.4pt size 12pt, p107 after 16pt size 10pt, p108 after 3pt, p109 size 10pt, width 6.5in, break-del r209+208 @ASR in p102; r211+210 @ASR in p103; r212 @ASR in p104; r214+213 @ASR in p105; r216+215 @ASR in p106; r218+217 @ASR in p107; r219 @ASR in p108; r221+220 @ASR in p109, rev-del r207 @ASR in t5.row0 -->
|{--**EigenPal**               --}{r209+208 @ASR}{--<br>Maintainer · eigenpal.com               --}{r211+210 @ASR}<br>{--<br>signature · date               --}{r214+213 @ASR}|{--**Contributor**               --}{r216+215 @ASR}{--<br>on behalf of the community               --}{r218+217 @ASR}<br>{--<br>signature · date               --}{r221+220 @ASR}|
|-|-|

<!-- p110 empty -->
