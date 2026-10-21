---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 435                     # 422 marks shown as 246 changes: 2 insertions, 173 deletions, 2 substitutions, 43 paragraph marks, 26 table rows; 5 formatting; 2 in header1; 2 in footer1; 2 in endnotes; 2 in footnotes
comments: 4 threads open, style Normal  # 6 comments: c16 (+ reply c17), c76, c207 (+ reply c208), c267
authors:
  document_owner: Un-named         # dc:creator
  AS: Arthur Souza Rodrigues       # 435 revisions, 4 comments, latest 2026-09-26T14:55:00-04:00
  U: User                          # 2 comments, 2026-03-05T19:41:00Z
body: p0-p218, 12 tables, 7 pages  # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in; fmt r440 @AS
styles:
  Normal: Inter 11pt, left         # default; unannotated paragraphs use it
  ListParagraph: Inter 11pt, named List Paragraph
  table: TableNormal
notes: {fmt: footnotes.r0 @AS; footnotes.r1 @AS; endnotes.r2 @AS; endnotes.r3 @AS, empty: [footnotes, endnotes]}  # changes in notes the body does not show (separators)
headers:
  default: {id: header1, text: "{--Docx Editor: Project Charter & Contributor Agreement       --}{header1.r437 @AS}", style: Normal, fmt: header1.r436 @AS in header1}
footers:
  default: {id: footer1, text: "{--Page       {PAGE \\* MERGEFORMAT}1       {/}of       {NUMPAGES \\* MERGEFORMAT}1       {/}·       {HYPERLINK \\h}eigenpal.com         {/}--}{footer1.r439 @AS}", style: ListParagraph, fmt: footer1.r438 @AS in footer1}
---

<!-- p0 fmt r4 @AS; r5 @AS; r6 @AS; r9 @AS -->
<mark>file\_</mark>{++<mark>177</mark>++}{r7+8 @AS}{--**176**--}{r7+8 @AS}<mark>.docx</mark>

<!-- p1 -->
{++<mark>Yellow Highlight Demo</mark>++}{r11+10 @AS}¶{r11+10 @AS}

<!-- p2 -->
{++<mark>This document demonstrates yellow text highlighting.</mark>++}{r13+12 @AS}¶{r13+12 @AS}

<!-- p3 after 1pt, break-del r15+18+14 @AS, in c16 -->
{++<mark>Highlighted text stands out for easy reference and review.</mark>++}{r15+18+14 @AS}{=={--**eigenpal/docx-editor**         --}{r15+18+14 @AS}==}

<!-- p4 after 3pt, size 12pt, break-del r20+19 @AS, in c16 -->
{=={--Project Charter & Contributor Agreement         --}{r20+19 @AS}==}

<!-- p5 empty, after 3pt, break-del r21 @AS, in c16 -->

<!-- t0 1x2, cells p6-p7 by row, width 6.5in, indent -0.003in, borders single 0.5pt, break-del r24+23 @AS in p6; r26+25 @AS in p7, rev-del r22 @AS in t0.row0, in c16 -->
|{=={--npm               [<u>` @eigenpal/docx-js-editor                  `</u>](https://www.npmjs.com/package/@eigenpal/docx-js-editor)--}{r24+23 @AS}==}|{=={--github               [<u>` eigenpal/docx-editor                  `</u>](https://github.com/eigenpal/docx-editor)--}{r26+25 @AS}==}|
|-|-|

<!-- p8 before 15pt, after 4pt, size 16pt, break-del r28+27 @AS, in c16 -->
{=={--**1. What this is**         --}{r28+27 @AS}==}

<!-- p9 before 3pt, after 3pt, break-del r30+29 @AS, in c16 -->
{=={--An open-source WYSIWYG         ` .docx          `editor for React. Drop in         ` <DocxEditor documentBuffer={file} />          `and your users can open, edit, and save Word documents entirely in the browser —         no backend required           , no document data leaving the client.         --}{r30+29 @AS}==}

<!-- p10 empty, before 3pt, break-del r31 @AS, in c16 -->

<!-- p11 before 3pt, after 3pt, break-del r33+32 @AS, in c16 -->
{=={--Most editors serialise to HTML and back:         ` DOCX → HTML → edits → HTML → DOCX          `. Every hop loses formatting, tracked changes, and styles.         This library skips it           — OOXML is the canonical format throughout.         --}{r33+32 @AS}==}

<!-- p12 empty, before 4pt, break-del r34 @AS, in c16 -->

<!-- t1 1x1, cells t1.row0 p13-p23, p13-p14 size 9.5pt, p16-p23 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r37+36 @AS in p13; r39+38 @AS in p14; r40 @AS in p15; r42+41 @AS in p16; r44+43 @AS in p17; r46+45 @AS in p18; r48+47 @AS in p19; r50+49 @AS in p20; r52+51 @AS in p21; r54+53 @AS in p22; r56+55 @AS in p23, rev-del r35 @AS in t1.row0, in c16 -->
|{=={--` import { DocxEditor } from '@eigenpal/docx-js-editor'                `--}{r37+36 @AS}==}<br>{=={--` import '@eigenpal/docx-js-editor/styles.css'                `--}{r39+38 @AS}==}<br><br>{=={--` function App({ file }: { file: ArrayBuffer }) {                `--}{r42+41 @AS}==}<br>{=={--` return (                `--}{r44+43 @AS}==}<br>{=={--` <DocxEditor                `--}{r46+45 @AS}==}<br>{=={--` documentBuffer={file}                `--}{r48+47 @AS}==}<br>{=={--` onSave={(buffer) => uploadToServer(buffer)}                `--}{r50+49 @AS}==}<br>{=={--` />                `--}{r52+51 @AS}==}<br>{=={--` )                `--}{r54+53 @AS}==}<br>{=={--` }                `--}{r56+55 @AS}==}|
|-|

<!-- p24 before 8pt, break-del r57 @AS, rev-del r58 @AS -->
{>>c16 @AS: Horrible.<<}{>>c17 @AS re c16: Terrible.<<}

<!-- p25 before 15pt, after 4pt, size 16pt, break-del r60+59 @AS -->
{--**2. Collaboration — Tracked Changes & Comments**         --}{r60+59 @AS}

<!-- p26 before 3pt, after 3pt, break-del r62+61 @AS -->
{--Both features round-trip through OOXML so documents open correctly in Word, LibreOffice, and Google Docs.         --}{r62+61 @AS}

<!-- p27 empty, before 4pt, break-del r63 @AS -->

<!-- p28 before 9pt, after 2pt, break-del r65+64 @AS -->
{--**Suggestion mode**         --}{r65+64 @AS}

<!-- p29 before 3pt, after 3pt, break-del r67+66 @AS -->
{--Toggle between         ` Editing          `and         ` Suggesting          `via the toolbar dropdown. In Suggesting mode, every edit becomes a tracked insertion (         green underline         ) or deletion (         red strikethrough         ). Consecutive keystrokes group into a single change. Accept or reject individually or all at once.         --}{r67+66 @AS}

<!-- p30 empty, before 3pt, break-del r68 @AS -->

<!-- p31 before 3pt, after 3pt, break-del r70+69 @AS -->
{--Serialises to         ` w:ins          `/         ` w:del          `— the same OOXML markup Word uses. Tracked changes survive a save/reload cycle and are reviewable in Word with full accept/reject UI.         --}{r70+69 @AS}

<!-- p32 empty, before 4pt, break-del r71 @AS -->

<!-- p33 before 9pt, after 2pt, break-del r73+72 @AS -->
{--**Comments**         --}{r73+72 @AS}

<!-- p34 before 3pt, after 3pt, break-del r75+74 @AS -->
{--Select text and click the floating button on the page edge to         {==add a comment anchored to that selection         ==}{>>c76 @U: Just like that!<<}. A Google Docs-style sidebar shows threaded cards positioned at their anchor point in the document. Reply, resolve, reopen, or delete inline.         --}{r75+74 @AS}

<!-- p35 empty, before 3pt, break-del r77 @AS -->

<!-- p36 before 3pt, after 3pt, break-del r79+78 @AS -->
{--Serialises to         ` comments.xml          `with         ` w:commentRangeStart          `/         ` w:commentRangeEnd          `markers. Comments and reply threads are preserved on round-trip.         --}{r79+78 @AS}

<!-- p37 empty, before 4pt, break-del r80 @AS -->

<!-- p38 before 9pt, after 2pt, break-del r82+81 @AS -->
{--**Agent-friendly**         --}{r82+81 @AS}

<!-- p39 before 3pt, after 3pt, break-del r84+83 @AS -->
{--Because tracked changes and comments are native OOXML, an AI agent can write         ` w:ins          `/         ` w:del          `/         ` w:comment          `markup directly into a document and hand it to         ` <DocxEditor />          `for human review — no custom diff format, no proprietary API.         --}{r84+83 @AS}

<!-- p40 empty, before 8pt, break-del r85 @AS -->

<!-- p41 before 15pt, after 4pt, size 16pt, break-del r87+86 @AS -->
{--**3. Props**         --}{r87+86 @AS}

<!-- p42 before 3pt, after 3pt, break-del r89+88 @AS -->
{--Pass         ` author="Your App"          `to set the author name for comments and tracked changes. Toggle suggestion mode via the toolbar dropdown.         --}{r89+88 @AS}

<!-- p43 empty, before 3pt, break-del r90 @AS -->

<!-- t2 8x4, cells p44-p75 by row, p44-p50 size 9.5pt, p51 size 10pt, p52-p54 size 9.5pt, p55 size 10pt, p56-p58 size 9.5pt, p59 size 10pt, p60-p62 size 9.5pt, p63 size 10pt, p64-p66 size 9.5pt, p67 size 10pt, p68-p70 size 9.5pt, p71 size 10pt, p72-p74 size 9.5pt, p75 size 10pt, widths 1.309in 1.191in 0.67in 3.33in, width 6.5in, indent -0.003in, borders single 0.5pt, break-del r93+92 @AS in p44; r95+94 @AS in p45; r97+96 @AS in p46; r99+98 @AS in p47; r102+101 @AS in p48; r104+103 @AS in p49; r106+105 @AS in p50; r108+107 @AS in p51; r111+110 @AS in p52; r113+112 @AS in p53; r115+114 @AS in p54; r117+116 @AS in p55; r120+119 @AS in p56; r122+121 @AS in p57; r124+123 @AS in p58; r126+125 @AS in p59; r129+128 @AS in p60; r131+130 @AS in p61; r133+132 @AS in p62; r135+134 @AS in p63; r138+137 @AS in p64; r140+139 @AS in p65; r142+141 @AS in p66; r144+143 @AS in p67; r147+146 @AS in p68; r149+148 @AS in p69; r151+150 @AS in p70; r153+152 @AS in p71; r156+155 @AS in p72; r158+157 @AS in p73; r160+159 @AS in p74; r162+161 @AS in p75, rev-del r91 @AS in t2.row0; r100 @AS in t2.row1; r109 @AS in t2.row2; r118 @AS in t2.row3; r127 @AS in t2.row4; r136 @AS in t2.row5; r145 @AS in t2.row6; r154 @AS in t2.row7 -->
|{--**Prop**               --}{r93+92 @AS}|{--**Type**               --}{r95+94 @AS}|{--**Default**               --}{r97+96 @AS}|{--**Description**               --}{r99+98 @AS}|
|-|-|-|-|
|{--` documentBuffer                `--}{r102+101 @AS}|{--` ArrayBuffer                `--}{r104+103 @AS}|{--` —                `--}{r106+105 @AS}|{--.docx file contents to load               --}{r108+107 @AS}|
|{--` readOnly                `--}{r111+110 @AS}|{--` boolean                `--}{r113+112 @AS}|{--` false                `--}{r115+114 @AS}|{--Preview mode — no editing UI               --}{r117+116 @AS}|
|{--` showToolbar                `--}{r120+119 @AS}|{--` boolean                `--}{r122+121 @AS}|{--` true                `--}{r124+123 @AS}|{--Show formatting toolbar               --}{r126+125 @AS}|
|{--` author                `--}{r129+128 @AS}|{--` string                `--}{r131+130 @AS}|{--` 'User'                `--}{r133+132 @AS}|{--Author name for comments and tracked changes               --}{r135+134 @AS}|
|{--` onChange                `--}{r138+137 @AS}|{--` function                `--}{r140+139 @AS}|{--` —                `--}{r142+141 @AS}|{--Called on every document change               --}{r144+143 @AS}|
|{--` onSave                `--}{r147+146 @AS}|{--` function                `--}{r149+148 @AS}|{--` —                `--}{r151+150 @AS}|{--Called on save, receives ArrayBuffer               --}{r153+152 @AS}|
|{--` onError                `--}{r156+155 @AS}|{--` function                `--}{r158+157 @AS}|{--` —                `--}{r160+159 @AS}|{--Called on error               --}{r162+161 @AS}|

<!-- p76 empty, before 4pt, break-del r163 @AS -->

<!-- p77 page-break-del r165+164 @AS, break-del r165+164 @AS -->

<!-- p78 before 9pt, after 2pt, break-del r167+166 @AS -->
{--**Ref methods**         --}{r167+166 @AS}

<!-- t3 1x1, cells t3.row0 p79-p85, p79 size 9.5pt, p81-p85 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r170+169 @AS in p79; r171 @AS in p80; r173+172 @AS in p81; r175+174 @AS in p82; r177+176 @AS in p83; r179+178 @AS in p84; r181+180 @AS in p85, rev-del r168 @AS in t3.row0 -->
|{--` const ref = useRef<DocxEditorRef>(null)                `--}{r170+169 @AS}<br>{--<br>` await ref.current.save() // ArrayBuffer of the .docx                `--}{r173+172 @AS}{--<br>` ref.current.getDocument() // current document object                `--}{r175+174 @AS}{--<br>` ref.current.setZoom(1.5) // set zoom to 150%                `--}{r177+176 @AS}{--<br>` ref.current.scrollToPage(3) // scroll to page 3                `--}{r179+178 @AS}{--<br>` ref.current.print() // print                `--}{r181+180 @AS}|
|-|

<!-- p86 empty, before 8pt, break-del r182 @AS -->

<!-- p87 before 15pt, after 4pt, size 16pt, break-del r184+183 @AS -->
{--**4. Contributing**         --}{r184+183 @AS}

<!-- p88 before 3pt, after 3pt, break-del r186+185 @AS -->
{--Read         [<u>` docs/PLUGINS.md            `</u>](https://github.com/eigenpal/docx-editor/blob/main/docs/PLUGINS.md)before opening a pull request that touches the plugin API.         --}{r186+185 @AS}

<!-- p89 empty, before 3pt, break-del r187 @AS -->

<!-- p90 before 9pt, after 2pt, break-del r189+188 @AS -->
{--**Ground rules**         --}{r189+188 @AS}

<!-- p91 ListParagraph before 2pt, after 2pt, bullet, break-del r191+190 @AS -->
- {--Every PR must pass the round-trip test: load a .docx, apply an edit, export, reload — content and formatting must be stable on unchanged content.         --}{r191+190 @AS}

<!-- p92 ListParagraph before 2pt, after 2pt, bullet, break-del r193+192 @AS -->
- {--Run         ` bun test          `and         ` bun run test:e2e          `locally before pushing.         --}{r193+192 @AS}

<!-- p93 ListParagraph before 2pt, after 2pt, bullet, break-del r195+194 @AS -->
- {--Avoid intermediate HTML serialisation where possible.           --}{r195+194 @AS}

<!-- p94 ListParagraph before 2pt, after 2pt, bullet, break-del r197+196 @AS -->
- {--Tracked changes and comments must survive a full OOXML round-trip — save, open in Word, verify changes are reviewable, reload in the editor.         --}{r197+196 @AS}

<!-- p95 empty, before 4pt, break-del r198 @AS -->

<!-- p96 before 9pt, after 2pt, break-del r200+199 @AS -->
{--**Dev setup**         --}{r200+199 @AS}

<!-- t4 1x1, cells t4.row0 p97-p98, p97-p98 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r203+202 @AS in p97; r205+204 @AS in p98, rev-del r201 @AS in t4.row0 -->
|{--` bun install                `--}{r203+202 @AS}{--<br>` bun run dev // Vite demo on localhost:5173                `--}{r205+204 @AS}|
|-|

<!-- p99 after 1pt, size 32pt, break-del r209+206 @AS, in c207 -->
{=={--**eigenpal/docx-editor**         --}{r209+206 @AS}==}

<!-- p100 after 3pt, size 12pt, break-del r211+210 @AS, in c207 -->
{=={--Project Charter & Contributor Agreement         --}{r211+210 @AS}==}

<!-- p101 empty, after 3pt, break-del r212 @AS, in c207 -->

<!-- t5 1x2, cells p102-p103 by row, width 6.5in, indent -0.003in, borders single 0.5pt, break-del r215+214 @AS in p102; r217+216 @AS in p103, rev-del r213 @AS in t5.row0, in c207 -->
|{=={--npm               [<u>` @eigenpal/docx-js-editor                  `</u>](https://www.npmjs.com/package/@eigenpal/docx-js-editor)--}{r215+214 @AS}==}|{=={--github               [<u>` eigenpal/docx-editor                  `</u>](https://github.com/eigenpal/docx-editor)--}{r217+216 @AS}==}|
|-|-|

<!-- p104 before 15pt, after 4pt, size 16pt, break-del r219+218 @AS, in c207 -->
{=={--**1. What this is**         --}{r219+218 @AS}==}

<!-- p105 before 3pt, after 3pt, break-del r221+220 @AS, in c207 -->
{=={--An open-source WYSIWYG         ` .docx          `editor for React. Drop in         ` <DocxEditor documentBuffer={file} />          `and your users can open, edit, and save Word documents entirely in the browser —         no backend required           , no document data leaving the client.         --}{r221+220 @AS}==}

<!-- p106 empty, before 3pt, break-del r222 @AS, in c207 -->

<!-- p107 before 3pt, after 3pt, break-del r224+223 @AS, in c207 -->
{=={--Most editors serialise to HTML and back:         ` DOCX → HTML → edits → HTML → DOCX          `. Every hop loses formatting, tracked changes, and styles.         This library skips it           — OOXML is the canonical format throughout.         --}{r224+223 @AS}==}

<!-- p108 empty, before 4pt, break-del r225 @AS, in c207 -->

<!-- t6 1x1, cells t6.row0 p109-p119, p109-p110 size 9.5pt, p112-p119 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r228+227 @AS in p109; r230+229 @AS in p110; r231 @AS in p111; r233+232 @AS in p112; r235+234 @AS in p113; r237+236 @AS in p114; r239+238 @AS in p115; r241+240 @AS in p116; r243+242 @AS in p117; r245+244 @AS in p118; r247+246 @AS in p119, rev-del r226 @AS in t6.row0, in c207 -->
|{=={--` import { DocxEditor } from '@eigenpal/docx-js-editor'                `--}{r228+227 @AS}==}<br>{=={--` import '@eigenpal/docx-js-editor/styles.css'                `--}{r230+229 @AS}==}<br><br>{=={--` function App({ file }: { file: ArrayBuffer }) {                `--}{r233+232 @AS}==}<br>{=={--` return (                `--}{r235+234 @AS}==}<br>{=={--` <DocxEditor                `--}{r237+236 @AS}==}<br>{=={--` documentBuffer={file}                `--}{r239+238 @AS}==}<br>{=={--` onSave={(buffer) => uploadToServer(buffer)}                `--}{r241+240 @AS}==}<br>{=={--` />                `--}{r243+242 @AS}==}<br>{=={--` )                `--}{r245+244 @AS}==}<br>{=={--` }                `--}{r247+246 @AS}==}|
|-|

<!-- p120 before 8pt, break-del r248 @AS, rev-del r249 @AS -->
{>>c207 @AS: Horrible.<<}{>>c208 @AS re c207: Terrible.<<}

<!-- p121 before 15pt, after 4pt, size 16pt, break-del r251+250 @AS -->
{--**2. Collaboration — Tracked Changes & Comments**         --}{r251+250 @AS}

<!-- p122 before 3pt, after 3pt, break-del r253+252 @AS -->
{--Both features round-trip through OOXML so documents open correctly in Word, LibreOffice, and Google Docs.         --}{r253+252 @AS}

<!-- p123 empty, before 4pt, break-del r254 @AS -->

<!-- p124 before 9pt, after 2pt, break-del r256+255 @AS -->
{--**Suggestion mode**         --}{r256+255 @AS}

<!-- p125 before 3pt, after 3pt, break-del r258+257 @AS -->
{--Toggle between         ` Editing          `and         ` Suggesting          `via the toolbar dropdown. In Suggesting mode, every edit becomes a tracked insertion (         green underline         ) or deletion (         red strikethrough         ). Consecutive keystrokes group into a single change. Accept or reject individually or all at once.         --}{r258+257 @AS}

<!-- p126 empty, before 3pt, break-del r259 @AS -->

<!-- p127 before 3pt, after 3pt, break-del r261+260 @AS -->
{--Serialises to         ` w:ins          `/         ` w:del          `— the same OOXML markup Word uses. Tracked changes survive a save/reload cycle and are reviewable in Word with full accept/reject UI.         --}{r261+260 @AS}

<!-- p128 empty, before 4pt, break-del r262 @AS -->

<!-- p129 before 9pt, after 2pt, break-del r264+263 @AS -->
{--**Comments**         --}{r264+263 @AS}

<!-- p130 before 3pt, after 3pt, break-del r266+265 @AS -->
{--Select text and click the floating button on the page edge to         {==add a comment anchored to that selection         ==}{>>c267 @U: Just like that!<<}. A Google Docs-style sidebar shows threaded cards positioned at their anchor point in the document. Reply, resolve, reopen, or delete inline.         --}{r266+265 @AS}

<!-- p131 empty, before 3pt, break-del r268 @AS -->

<!-- p132 before 3pt, after 3pt, break-del r270+269 @AS -->
{--Serialises to         ` comments.xml          `with         ` w:commentRangeStart          `/         ` w:commentRangeEnd          `markers. Comments and reply threads are preserved on round-trip.         --}{r270+269 @AS}

<!-- p133 empty, before 4pt, break-del r271 @AS -->

<!-- p134 before 9pt, after 2pt, break-del r273+272 @AS -->
{--**Agent-friendly**         --}{r273+272 @AS}

<!-- p135 before 3pt, after 3pt, break-del r275+274 @AS -->
{--Because tracked changes and comments are native OOXML, an AI agent can write         ` w:ins          `/         ` w:del          `/         ` w:comment          `markup directly into a document and hand it to         ` <DocxEditor />          `for human review — no custom diff format, no proprietary API.         --}{r275+274 @AS}

<!-- p136 empty, before 8pt, break-del r276 @AS -->

<!-- p137 before 15pt, after 4pt, size 16pt, break-del r278+277 @AS -->
{--**3. Props**         --}{r278+277 @AS}

<!-- p138 before 3pt, after 3pt, break-del r280+279 @AS -->
{--Pass         ` author="Your App"          `to set the author name for comments and tracked changes. Toggle suggestion mode via the toolbar dropdown.         --}{r280+279 @AS}

<!-- p139 empty, before 3pt, break-del r281 @AS -->

<!-- t7 8x4, cells p140-p171 by row, p140-p146 size 9.5pt, p147 size 10pt, p148-p150 size 9.5pt, p151 size 10pt, p152-p154 size 9.5pt, p155 size 10pt, p156-p158 size 9.5pt, p159 size 10pt, p160-p162 size 9.5pt, p163 size 10pt, p164-p166 size 9.5pt, p167 size 10pt, p168-p170 size 9.5pt, p171 size 10pt, widths 1.309in 1.191in 0.67in 3.33in, width 6.5in, indent -0.003in, borders single 0.5pt, break-del r284+283 @AS in p140; r286+285 @AS in p141; r288+287 @AS in p142; r290+289 @AS in p143; r293+292 @AS in p144; r295+294 @AS in p145; r297+296 @AS in p146; r299+298 @AS in p147; r302+301 @AS in p148; r304+303 @AS in p149; r306+305 @AS in p150; r308+307 @AS in p151; r311+310 @AS in p152; r313+312 @AS in p153; r315+314 @AS in p154; r317+316 @AS in p155; r320+319 @AS in p156; r322+321 @AS in p157; r324+323 @AS in p158; r326+325 @AS in p159; r329+328 @AS in p160; r331+330 @AS in p161; r333+332 @AS in p162; r335+334 @AS in p163; r338+337 @AS in p164; r340+339 @AS in p165; r342+341 @AS in p166; r344+343 @AS in p167; r347+346 @AS in p168; r349+348 @AS in p169; r351+350 @AS in p170; r353+352 @AS in p171, rev-del r282 @AS in t7.row0; r291 @AS in t7.row1; r300 @AS in t7.row2; r309 @AS in t7.row3; r318 @AS in t7.row4; r327 @AS in t7.row5; r336 @AS in t7.row6; r345 @AS in t7.row7 -->
|{--**Prop**               --}{r284+283 @AS}|{--**Type**               --}{r286+285 @AS}|{--**Default**               --}{r288+287 @AS}|{--**Description**               --}{r290+289 @AS}|
|-|-|-|-|
|{--` documentBuffer                `--}{r293+292 @AS}|{--` ArrayBuffer                `--}{r295+294 @AS}|{--` —                `--}{r297+296 @AS}|{--.docx file contents to load               --}{r299+298 @AS}|
|{--` readOnly                `--}{r302+301 @AS}|{--` boolean                `--}{r304+303 @AS}|{--` false                `--}{r306+305 @AS}|{--Preview mode — no editing UI               --}{r308+307 @AS}|
|{--` showToolbar                `--}{r311+310 @AS}|{--` boolean                `--}{r313+312 @AS}|{--` true                `--}{r315+314 @AS}|{--Show formatting toolbar               --}{r317+316 @AS}|
|{--` author                `--}{r320+319 @AS}|{--` string                `--}{r322+321 @AS}|{--` 'User'                `--}{r324+323 @AS}|{--Author name for comments and tracked changes               --}{r326+325 @AS}|
|{--` onChange                `--}{r329+328 @AS}|{--` function                `--}{r331+330 @AS}|{--` —                `--}{r333+332 @AS}|{--Called on every document change               --}{r335+334 @AS}|
|{--` onSave                `--}{r338+337 @AS}|{--` function                `--}{r340+339 @AS}|{--` —                `--}{r342+341 @AS}|{--Called on save, receives ArrayBuffer               --}{r344+343 @AS}|
|{--` onError                `--}{r347+346 @AS}|{--` function                `--}{r349+348 @AS}|{--` —                `--}{r351+350 @AS}|{--Called on error               --}{r353+352 @AS}|

<!-- p172 empty, before 4pt, break-del r354 @AS -->

<!-- p173 page-break-del r356+355 @AS, break-del r356+355 @AS -->

<!-- p174 before 9pt, after 2pt, break-del r358+357 @AS -->
{--**Ref methods**         --}{r358+357 @AS}

<!-- t8 1x1, cells t8.row0 p175-p181, p175 size 9.5pt, p177-p181 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r361+360 @AS in p175; r362 @AS in p176; r364+363 @AS in p177; r366+365 @AS in p178; r368+367 @AS in p179; r370+369 @AS in p180; r372+371 @AS in p181, rev-del r359 @AS in t8.row0 -->
|{--` const ref = useRef<DocxEditorRef>(null)                `--}{r361+360 @AS}<br>{--<br>` await ref.current.save() // ArrayBuffer of the .docx                `--}{r364+363 @AS}{--<br>` ref.current.getDocument() // current document object                `--}{r366+365 @AS}{--<br>` ref.current.setZoom(1.5) // set zoom to 150%                `--}{r368+367 @AS}{--<br>` ref.current.scrollToPage(3) // scroll to page 3                `--}{r370+369 @AS}{--<br>` ref.current.print() // print                `--}{r372+371 @AS}|
|-|

<!-- p182 empty, before 8pt, break-del r373 @AS -->

<!-- p183 before 15pt, after 4pt, size 16pt, break-del r375+374 @AS -->
{--**4. Contributing**         --}{r375+374 @AS}

<!-- p184 before 3pt, after 3pt, break-del r377+376 @AS -->
{--Read         [<u>` docs/PLUGINS.md            `</u>](https://github.com/eigenpal/docx-editor/blob/main/docs/PLUGINS.md)before opening a pull request that touches the plugin API.         --}{r377+376 @AS}

<!-- p185 empty, before 3pt, break-del r378 @AS -->

<!-- p186 before 9pt, after 2pt, break-del r380+379 @AS -->
{--**Ground rules**         --}{r380+379 @AS}

<!-- p187 ListParagraph before 2pt, after 2pt, bullet, break-del r382+381 @AS -->
- {--Every PR must pass the round-trip test: load a .docx, apply an edit, export, reload — content and formatting must be stable on unchanged content.         --}{r382+381 @AS}

<!-- p188 ListParagraph before 2pt, after 2pt, bullet, break-del r384+383 @AS -->
- {--Run         ` bun test          `and         ` bun run test:e2e          `locally before pushing.         --}{r384+383 @AS}

<!-- p189 ListParagraph before 2pt, after 2pt, bullet, break-del r386+385 @AS -->
- {--Avoid intermediate HTML serialisation where possible.           --}{r386+385 @AS}

<!-- p190 ListParagraph before 2pt, after 2pt, bullet, break-del r388+387 @AS -->
- {--Tracked changes and comments must survive a full OOXML round-trip — save, open in Word, verify changes are reviewable, reload in the editor.         --}{r388+387 @AS}

<!-- p191 empty, before 4pt, break-del r389 @AS -->

<!-- p192 before 9pt, after 2pt, break-del r391+390 @AS -->
{--**Dev setup**         --}{r391+390 @AS}

<!-- t9 1x1, cells t9.row0 p193-p194, p193-p194 size 9.5pt, width 6.5in, indent 0.0035in, borders single 0.5pt, break-del r394+393 @AS in p193; r396+395 @AS in p194, rev-del r392 @AS in t9.row0 -->
|{--` bun install                `--}{r394+393 @AS}{--<br>` bun run dev // Vite demo on localhost:5173                `--}{r396+395 @AS}|
|-|

<!-- p195 empty, before 10pt, break-del r397 @AS -->

<!-- p196 before 15pt, after 4pt, size 16pt, break-del r399+398 @AS -->
{--**Sign-off**         --}{r399+398 @AS}

<!-- p197 empty, before 3pt, break-del r400 @AS -->

<!-- t10 1x2, cells t10.row0 p198-p205, p198 after 0.4pt size 12pt, p199 after 16pt size 10pt, p200 after 3pt, p201 size 10pt, p202 after 0.4pt size 12pt, p203 after 16pt size 10pt, p204 after 3pt, p205 size 10pt, width 6.5in, break-del r403+402 @AS in p198; r405+404 @AS in p199; r406 @AS in p200; r408+407 @AS in p201; r410+409 @AS in p202; r412+411 @AS in p203; r413 @AS in p204; r415+414 @AS in p205, rev-del r401 @AS in t10.row0 -->
|{--**EigenPal**               --}{r403+402 @AS}{--<br>Maintainer · eigenpal.com               --}{r405+404 @AS}<br>{--<br>signature · date               --}{r408+407 @AS}|{--**Contributor**               --}{r410+409 @AS}{--<br>on behalf of the community               --}{r412+411 @AS}<br>{--<br>signature · date               --}{r415+414 @AS}|
|-|-|

<!-- p206 empty, break-del r416 @AS -->

<!-- p207 empty, before 10pt, break-del r417 @AS -->

<!-- p208 before 15pt, after 4pt, size 16pt, break-del r419+418 @AS -->
{--**Sign-off**         --}{r419+418 @AS}

<!-- p209 empty, before 3pt, break-del r420 @AS -->

<!-- t11 1x2, cells t11.row0 p210-p217, p210 after 0.4pt size 12pt, p211 after 16pt size 10pt, p212 after 3pt, p213 size 10pt, p214 after 0.4pt size 12pt, p215 after 16pt size 10pt, p216 after 3pt, p217 size 10pt, width 6.5in, break-del r423+422 @AS in p210; r425+424 @AS in p211; r426 @AS in p212; r428+427 @AS in p213; r430+429 @AS in p214; r432+431 @AS in p215; r433 @AS in p216; r435+434 @AS in p217, rev-del r421 @AS in t11.row0 -->
|{--**EigenPal**               --}{r423+422 @AS}{--<br>Maintainer · eigenpal.com               --}{r425+424 @AS}<br>{--<br>signature · date               --}{r428+427 @AS}|{--**Contributor**               --}{r430+429 @AS}{--<br>on behalf of the community               --}{r432+431 @AS}<br>{--<br>signature · date               --}{r435+434 @AS}|
|-|-|

<!-- p218 empty -->
