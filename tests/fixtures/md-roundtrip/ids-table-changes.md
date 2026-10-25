---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 435                     # 422 marks shown as 421 changes: 3 insertions, 174 deletions, 1 substitution, 217 paragraph marks, 26 table rows; 5 formatting; 2 in header1; 2 in footer1; 2 in endnotes; 2 in footnotes
comments: 4 threads open, style Normal  # 6 comments: c16 (+ reply c17), c76, c207 (+ reply c208), c267
authors:
  document_owner: Un-named         # dc:creator
  AS: Arthur Souza Rodrigues       # 435 revisions, 4 comments, 2026-07-04..2026-09-26
  U: User                          # 2 comments, 2026-03-05T19:41:00Z
body: p0-p218, 12 tables, 7 pages  # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in; fmt r440 @AS
styles:
  Normal: Inter 11pt, left         # default; unannotated paragraphs use it
  ListParagraph: Inter 11pt, named List Paragraph
  table: TableNormal
notes: {fmt: footnotes.r0 @AS; footnotes.r1 @AS; endnotes.r2 @AS; endnotes.r3 @AS, empty: [footnotes, endnotes]}  # changes in notes the body does not show (separators)
headers:
  default: {id: header1, text: "{--Docx Editor: Project Charter & Contributor Agreement       --}{>>header1.r437 @AS<<}", style: Normal, fmt: header1.r436 @AS in header1}
footers:
  default: {id: footer1, text: "{--Page       {PAGE \\* MERGEFORMAT}1       {/}of       {NUMPAGES \\* MERGEFORMAT}1       {/}·       {HYPERLINK \\h}eigenpal.com         {/}--}{>>footer1.r439 @AS<<}", style: ListParagraph, fmt: footer1.r438 @AS in footer1}
---

<!-- p0 fmt r4 @AS; r5 @AS; r6 @AS; r9 @AS -->
<mark>file\_</mark>{~~**176**~><mark>177</mark>~~}{>>r8+7 @AS<<}<mark>.docx</mark>

<!-- p1 break-ins r10 @AS -->
{++<mark>Yellow Highlight Demo</mark>++}{>>r11 @AS<<}

<!-- p2 break-ins r12 @AS -->
{++<mark>This document demonstrates yellow text highlighting.</mark>++}{>>r13 @AS<<}

<!-- p3 break-del r14 @AS, in c16 -->
{++<mark>Highlighted text stands out for easy reference and review.</mark>++}{>>r15 @AS<<}{=={--**eigenpal/docx-editor**         --}{>>r18 @AS<<}==}

<!-- p4 break-del r19 @AS, in c16 -->
{=={--Project Charter & Contributor Agreement         --}{>>r20 @AS<<}==}

<!-- p5 empty, break-del r21 @AS, in c16 -->

<!-- t0 1x2, cells p6-p7 by row, borders single 0.5pt, break-del r23 @AS in p6; r25 @AS in p7, rev-del r22 @AS in t0.row0, in c16 -->
|{=={--npm               [<u>`@eigenpal/docx-js-editor`</u>](https://www.npmjs.com/package/@eigenpal/docx-js-editor)                 --}{>>r24 @AS<<}==}|{=={--github               [<u>`eigenpal/docx-editor`</u>](https://github.com/eigenpal/docx-editor)                 --}{>>r26 @AS<<}==}|
|-|-|

<!-- p8 break-del r27 @AS, in c16 -->
{=={--**1. What this is**         --}{>>r28 @AS<<}==}

<!-- p9 break-del r29 @AS, in c16 -->
{=={--An open-source WYSIWYG         `.docx`         editor for React. Drop in         `<DocxEditor documentBuffer={file} />`         and your users can open, edit, and save Word documents entirely in the browser —         no backend required           , no document data leaving the client.         --}{>>r30 @AS<<}==}

<!-- p10 empty, break-del r31 @AS, in c16 -->

<!-- p11 break-del r32 @AS, in c16 -->
{=={--Most editors serialise to HTML and back:         `DOCX → HTML → edits → HTML → DOCX`         . Every hop loses formatting, tracked changes, and styles.         This library skips it           — OOXML is the canonical format throughout.         --}{>>r33 @AS<<}==}

<!-- p12 empty, break-del r34 @AS, in c16 -->

<!-- t1 1x1, cells t1.row0 p13-p23, borders single 0.5pt, break-del r36 @AS in p13; r38 @AS in p14; r40 @AS in p15; r41 @AS in p16; r43 @AS in p17; r45 @AS in p18; r47 @AS in p19; r49 @AS in p20; r51 @AS in p21; r53 @AS in p22; r55 @AS in p23, rev-del r35 @AS in t1.row0, in c16 -->
|{=={--`import { DocxEditor } from '@eigenpal/docx-js-editor'`               --}{>>r37 @AS<<}==}<br>{=={--`import '@eigenpal/docx-js-editor/styles.css'`               --}{>>r39 @AS<<}==}<br><br>{=={--`function App({ file }: { file: ArrayBuffer }) {`               --}{>>r42 @AS<<}==}<br>{=={--`return (`               --}{>>r44 @AS<<}==}<br>{=={--`<DocxEditor`               --}{>>r46 @AS<<}==}<br>{=={--`documentBuffer={file}`               --}{>>r48 @AS<<}==}<br>{=={--`onSave={(buffer) => uploadToServer(buffer)}`               --}{>>r50 @AS<<}==}<br>{=={--`/>`               --}{>>r52 @AS<<}==}<br>{=={--`)`               --}{>>r54 @AS<<}==}<br>{=={--`}`               --}{>>r56 @AS<<}==}|
|-|

<!-- p24 break-del r57 @AS, rev-del r58 @AS -->
{>>c16 @AS: Horrible.<<}{>>c17 @AS re c16: Terrible.<<}

<!-- p25 break-del r59 @AS -->
{--**2. Collaboration — Tracked Changes & Comments**         --}{>>r60 @AS<<}

<!-- p26 break-del r61 @AS -->
{--Both features round-trip through OOXML so documents open correctly in Word, LibreOffice, and Google Docs.         --}{>>r62 @AS<<}

<!-- p27 empty, break-del r63 @AS -->

<!-- p28 break-del r64 @AS -->
{--**Suggestion mode**         --}{>>r65 @AS<<}

<!-- p29 break-del r66 @AS -->
{--Toggle between         `Editing`         and         `Suggesting`         via the toolbar dropdown. In Suggesting mode, every edit becomes a tracked insertion (         green underline         ) or deletion (         red strikethrough         ). Consecutive keystrokes group into a single change. Accept or reject individually or all at once.         --}{>>r67 @AS<<}

<!-- p30 empty, break-del r68 @AS -->

<!-- p31 break-del r69 @AS -->
{--Serialises to         `w:ins`         /         `w:del`         — the same OOXML markup Word uses. Tracked changes survive a save/reload cycle and are reviewable in Word with full accept/reject UI.         --}{>>r70 @AS<<}

<!-- p32 empty, break-del r71 @AS -->

<!-- p33 break-del r72 @AS -->
{--**Comments**         --}{>>r73 @AS<<}

<!-- p34 break-del r74 @AS -->
{--Select text and click the floating button on the page edge to         {==add a comment anchored to that selection         ==}{>>c76 @U: Just like that!<<}. A Google Docs-style sidebar shows threaded cards positioned at their anchor point in the document. Reply, resolve, reopen, or delete inline.         --}{>>r75 @AS<<}

<!-- p35 empty, break-del r77 @AS -->

<!-- p36 break-del r78 @AS -->
{--Serialises to         `comments.xml`         with         `w:commentRangeStart`         /         `w:commentRangeEnd`         markers. Comments and reply threads are preserved on round-trip.         --}{>>r79 @AS<<}

<!-- p37 empty, break-del r80 @AS -->

<!-- p38 break-del r81 @AS -->
{--**Agent-friendly**         --}{>>r82 @AS<<}

<!-- p39 break-del r83 @AS -->
{--Because tracked changes and comments are native OOXML, an AI agent can write         `w:ins`         /         `w:del`         /         `w:comment`         markup directly into a document and hand it to         `<DocxEditor />`         for human review — no custom diff format, no proprietary API.         --}{>>r84 @AS<<}

<!-- p40 empty, break-del r85 @AS -->

<!-- p41 break-del r86 @AS -->
{--**3. Props**         --}{>>r87 @AS<<}

<!-- p42 break-del r88 @AS -->
{--Pass         `author="Your App"`         to set the author name for comments and tracked changes. Toggle suggestion mode via the toolbar dropdown.         --}{>>r89 @AS<<}

<!-- p43 empty, break-del r90 @AS -->

<!-- t2 8x4, cells p44-p75 by row, borders single 0.5pt, break-del r92 @AS in p44; r94 @AS in p45; r96 @AS in p46; r98 @AS in p47; r101 @AS in p48; r103 @AS in p49; r105 @AS in p50; r107 @AS in p51; r110 @AS in p52; r112 @AS in p53; r114 @AS in p54; r116 @AS in p55; r119 @AS in p56; r121 @AS in p57; r123 @AS in p58; r125 @AS in p59; r128 @AS in p60; r130 @AS in p61; r132 @AS in p62; r134 @AS in p63; r137 @AS in p64; r139 @AS in p65; r141 @AS in p66; r143 @AS in p67; r146 @AS in p68; r148 @AS in p69; r150 @AS in p70; r152 @AS in p71; r155 @AS in p72; r157 @AS in p73; r159 @AS in p74; r161 @AS in p75, rev-del r91 @AS in t2.row0; r100 @AS in t2.row1; r109 @AS in t2.row2; r118 @AS in t2.row3; r127 @AS in t2.row4; r136 @AS in t2.row5; r145 @AS in t2.row6; r154 @AS in t2.row7 -->
|{--**Prop**               --}{>>r93 @AS<<}|{--**Type**               --}{>>r95 @AS<<}|{--**Default**               --}{>>r97 @AS<<}|{--**Description**               --}{>>r99 @AS<<}|
|-|-|-|-|
|{--`documentBuffer`               --}{>>r102 @AS<<}|{--`ArrayBuffer`               --}{>>r104 @AS<<}|{--`—`               --}{>>r106 @AS<<}|{--.docx file contents to load               --}{>>r108 @AS<<}|
|{--`readOnly`               --}{>>r111 @AS<<}|{--`boolean`               --}{>>r113 @AS<<}|{--`false`               --}{>>r115 @AS<<}|{--Preview mode — no editing UI               --}{>>r117 @AS<<}|
|{--`showToolbar`               --}{>>r120 @AS<<}|{--`boolean`               --}{>>r122 @AS<<}|{--`true`               --}{>>r124 @AS<<}|{--Show formatting toolbar               --}{>>r126 @AS<<}|
|{--`author`               --}{>>r129 @AS<<}|{--`string`               --}{>>r131 @AS<<}|{--`'User'`               --}{>>r133 @AS<<}|{--Author name for comments and tracked changes               --}{>>r135 @AS<<}|
|{--`onChange`               --}{>>r138 @AS<<}|{--`function`               --}{>>r140 @AS<<}|{--`—`               --}{>>r142 @AS<<}|{--Called on every document change               --}{>>r144 @AS<<}|
|{--`onSave`               --}{>>r147 @AS<<}|{--`function`               --}{>>r149 @AS<<}|{--`—`               --}{>>r151 @AS<<}|{--Called on save, receives ArrayBuffer               --}{>>r153 @AS<<}|
|{--`onError`               --}{>>r156 @AS<<}|{--`function`               --}{>>r158 @AS<<}|{--`—`               --}{>>r160 @AS<<}|{--Called on error               --}{>>r162 @AS<<}|

<!-- p76 empty, break-del r163 @AS -->

<!-- p77 page-break-del r165 @AS, break-del r164 @AS -->

<!-- p78 break-del r166 @AS -->
{--**Ref methods**         --}{>>r167 @AS<<}

<!-- t3 1x1, cells t3.row0 p79-p85, borders single 0.5pt, break-del r169 @AS in p79; r171 @AS in p80; r172 @AS in p81; r174 @AS in p82; r176 @AS in p83; r178 @AS in p84; r180 @AS in p85, rev-del r168 @AS in t3.row0 -->
|{--`const ref = useRef<DocxEditorRef>(null)`               --}{>>r170 @AS<<}<br>{--<br>`await ref.current.save() // ArrayBuffer of the .docx`               --}{>>r173 @AS<<}{--<br>`ref.current.getDocument() // current document object`               --}{>>r175 @AS<<}{--<br>`ref.current.setZoom(1.5) // set zoom to 150%`               --}{>>r177 @AS<<}{--<br>`ref.current.scrollToPage(3) // scroll to page 3`               --}{>>r179 @AS<<}{--<br>`ref.current.print() // print`               --}{>>r181 @AS<<}|
|-|

<!-- p86 empty, break-del r182 @AS -->

<!-- p87 break-del r183 @AS -->
{--**4. Contributing**         --}{>>r184 @AS<<}

<!-- p88 break-del r185 @AS -->
{--Read         [<u>`docs/PLUGINS.md`</u>](https://github.com/eigenpal/docx-editor/blob/main/docs/PLUGINS.md)           before opening a pull request that touches the plugin API.         --}{>>r186 @AS<<}

<!-- p89 empty, break-del r187 @AS -->

<!-- p90 break-del r188 @AS -->
{--**Ground rules**         --}{>>r189 @AS<<}

<!-- p91 ListParagraph bullet, break-del r190 @AS -->
- {--Every PR must pass the round-trip test: load a .docx, apply an edit, export, reload — content and formatting must be stable on unchanged content.         --}{>>r191 @AS<<}

<!-- p92 ListParagraph bullet, break-del r192 @AS -->
- {--Run         `bun test`         and         `bun run test:e2e`         locally before pushing.         --}{>>r193 @AS<<}

<!-- p93 ListParagraph bullet, break-del r194 @AS -->
- {--Avoid intermediate HTML serialisation where possible.           --}{>>r195 @AS<<}

<!-- p94 ListParagraph bullet, break-del r196 @AS -->
- {--Tracked changes and comments must survive a full OOXML round-trip — save, open in Word, verify changes are reviewable, reload in the editor.         --}{>>r197 @AS<<}

<!-- p95 empty, break-del r198 @AS -->

<!-- p96 break-del r199 @AS -->
{--**Dev setup**         --}{>>r200 @AS<<}

<!-- t4 1x1, cells t4.row0 p97-p98, borders single 0.5pt, break-del r202 @AS in p97; r204 @AS in p98, rev-del r201 @AS in t4.row0 -->
|{--`bun install`               --}{>>r203 @AS<<}{--<br>`bun run dev // Vite demo on localhost:5173`               --}{>>r205 @AS<<}|
|-|

<!-- p99 break-del r206 @AS, in c207 -->
{=={--**eigenpal/docx-editor**         --}{>>r209 @AS<<}==}

<!-- p100 break-del r210 @AS, in c207 -->
{=={--Project Charter & Contributor Agreement         --}{>>r211 @AS<<}==}

<!-- p101 empty, break-del r212 @AS, in c207 -->

<!-- t5 1x2, cells p102-p103 by row, borders single 0.5pt, break-del r214 @AS in p102; r216 @AS in p103, rev-del r213 @AS in t5.row0, in c207 -->
|{=={--npm               [<u>`@eigenpal/docx-js-editor`</u>](https://www.npmjs.com/package/@eigenpal/docx-js-editor)                 --}{>>r215 @AS<<}==}|{=={--github               [<u>`eigenpal/docx-editor`</u>](https://github.com/eigenpal/docx-editor)                 --}{>>r217 @AS<<}==}|
|-|-|

<!-- p104 break-del r218 @AS, in c207 -->
{=={--**1. What this is**         --}{>>r219 @AS<<}==}

<!-- p105 break-del r220 @AS, in c207 -->
{=={--An open-source WYSIWYG         `.docx`         editor for React. Drop in         `<DocxEditor documentBuffer={file} />`         and your users can open, edit, and save Word documents entirely in the browser —         no backend required           , no document data leaving the client.         --}{>>r221 @AS<<}==}

<!-- p106 empty, break-del r222 @AS, in c207 -->

<!-- p107 break-del r223 @AS, in c207 -->
{=={--Most editors serialise to HTML and back:         `DOCX → HTML → edits → HTML → DOCX`         . Every hop loses formatting, tracked changes, and styles.         This library skips it           — OOXML is the canonical format throughout.         --}{>>r224 @AS<<}==}

<!-- p108 empty, break-del r225 @AS, in c207 -->

<!-- t6 1x1, cells t6.row0 p109-p119, borders single 0.5pt, break-del r227 @AS in p109; r229 @AS in p110; r231 @AS in p111; r232 @AS in p112; r234 @AS in p113; r236 @AS in p114; r238 @AS in p115; r240 @AS in p116; r242 @AS in p117; r244 @AS in p118; r246 @AS in p119, rev-del r226 @AS in t6.row0, in c207 -->
|{=={--`import { DocxEditor } from '@eigenpal/docx-js-editor'`               --}{>>r228 @AS<<}==}<br>{=={--`import '@eigenpal/docx-js-editor/styles.css'`               --}{>>r230 @AS<<}==}<br><br>{=={--`function App({ file }: { file: ArrayBuffer }) {`               --}{>>r233 @AS<<}==}<br>{=={--`return (`               --}{>>r235 @AS<<}==}<br>{=={--`<DocxEditor`               --}{>>r237 @AS<<}==}<br>{=={--`documentBuffer={file}`               --}{>>r239 @AS<<}==}<br>{=={--`onSave={(buffer) => uploadToServer(buffer)}`               --}{>>r241 @AS<<}==}<br>{=={--`/>`               --}{>>r243 @AS<<}==}<br>{=={--`)`               --}{>>r245 @AS<<}==}<br>{=={--`}`               --}{>>r247 @AS<<}==}|
|-|

<!-- p120 break-del r248 @AS, rev-del r249 @AS -->
{>>c207 @AS: Horrible.<<}{>>c208 @AS re c207: Terrible.<<}

<!-- p121 break-del r250 @AS -->
{--**2. Collaboration — Tracked Changes & Comments**         --}{>>r251 @AS<<}

<!-- p122 break-del r252 @AS -->
{--Both features round-trip through OOXML so documents open correctly in Word, LibreOffice, and Google Docs.         --}{>>r253 @AS<<}

<!-- p123 empty, break-del r254 @AS -->

<!-- p124 break-del r255 @AS -->
{--**Suggestion mode**         --}{>>r256 @AS<<}

<!-- p125 break-del r257 @AS -->
{--Toggle between         `Editing`         and         `Suggesting`         via the toolbar dropdown. In Suggesting mode, every edit becomes a tracked insertion (         green underline         ) or deletion (         red strikethrough         ). Consecutive keystrokes group into a single change. Accept or reject individually or all at once.         --}{>>r258 @AS<<}

<!-- p126 empty, break-del r259 @AS -->

<!-- p127 break-del r260 @AS -->
{--Serialises to         `w:ins`         /         `w:del`         — the same OOXML markup Word uses. Tracked changes survive a save/reload cycle and are reviewable in Word with full accept/reject UI.         --}{>>r261 @AS<<}

<!-- p128 empty, break-del r262 @AS -->

<!-- p129 break-del r263 @AS -->
{--**Comments**         --}{>>r264 @AS<<}

<!-- p130 break-del r265 @AS -->
{--Select text and click the floating button on the page edge to         {==add a comment anchored to that selection         ==}{>>c267 @U: Just like that!<<}. A Google Docs-style sidebar shows threaded cards positioned at their anchor point in the document. Reply, resolve, reopen, or delete inline.         --}{>>r266 @AS<<}

<!-- p131 empty, break-del r268 @AS -->

<!-- p132 break-del r269 @AS -->
{--Serialises to         `comments.xml`         with         `w:commentRangeStart`         /         `w:commentRangeEnd`         markers. Comments and reply threads are preserved on round-trip.         --}{>>r270 @AS<<}

<!-- p133 empty, break-del r271 @AS -->

<!-- p134 break-del r272 @AS -->
{--**Agent-friendly**         --}{>>r273 @AS<<}

<!-- p135 break-del r274 @AS -->
{--Because tracked changes and comments are native OOXML, an AI agent can write         `w:ins`         /         `w:del`         /         `w:comment`         markup directly into a document and hand it to         `<DocxEditor />`         for human review — no custom diff format, no proprietary API.         --}{>>r275 @AS<<}

<!-- p136 empty, break-del r276 @AS -->

<!-- p137 break-del r277 @AS -->
{--**3. Props**         --}{>>r278 @AS<<}

<!-- p138 break-del r279 @AS -->
{--Pass         `author="Your App"`         to set the author name for comments and tracked changes. Toggle suggestion mode via the toolbar dropdown.         --}{>>r280 @AS<<}

<!-- p139 empty, break-del r281 @AS -->

<!-- t7 8x4, cells p140-p171 by row, borders single 0.5pt, break-del r283 @AS in p140; r285 @AS in p141; r287 @AS in p142; r289 @AS in p143; r292 @AS in p144; r294 @AS in p145; r296 @AS in p146; r298 @AS in p147; r301 @AS in p148; r303 @AS in p149; r305 @AS in p150; r307 @AS in p151; r310 @AS in p152; r312 @AS in p153; r314 @AS in p154; r316 @AS in p155; r319 @AS in p156; r321 @AS in p157; r323 @AS in p158; r325 @AS in p159; r328 @AS in p160; r330 @AS in p161; r332 @AS in p162; r334 @AS in p163; r337 @AS in p164; r339 @AS in p165; r341 @AS in p166; r343 @AS in p167; r346 @AS in p168; r348 @AS in p169; r350 @AS in p170; r352 @AS in p171, rev-del r282 @AS in t7.row0; r291 @AS in t7.row1; r300 @AS in t7.row2; r309 @AS in t7.row3; r318 @AS in t7.row4; r327 @AS in t7.row5; r336 @AS in t7.row6; r345 @AS in t7.row7 -->
|{--**Prop**               --}{>>r284 @AS<<}|{--**Type**               --}{>>r286 @AS<<}|{--**Default**               --}{>>r288 @AS<<}|{--**Description**               --}{>>r290 @AS<<}|
|-|-|-|-|
|{--`documentBuffer`               --}{>>r293 @AS<<}|{--`ArrayBuffer`               --}{>>r295 @AS<<}|{--`—`               --}{>>r297 @AS<<}|{--.docx file contents to load               --}{>>r299 @AS<<}|
|{--`readOnly`               --}{>>r302 @AS<<}|{--`boolean`               --}{>>r304 @AS<<}|{--`false`               --}{>>r306 @AS<<}|{--Preview mode — no editing UI               --}{>>r308 @AS<<}|
|{--`showToolbar`               --}{>>r311 @AS<<}|{--`boolean`               --}{>>r313 @AS<<}|{--`true`               --}{>>r315 @AS<<}|{--Show formatting toolbar               --}{>>r317 @AS<<}|
|{--`author`               --}{>>r320 @AS<<}|{--`string`               --}{>>r322 @AS<<}|{--`'User'`               --}{>>r324 @AS<<}|{--Author name for comments and tracked changes               --}{>>r326 @AS<<}|
|{--`onChange`               --}{>>r329 @AS<<}|{--`function`               --}{>>r331 @AS<<}|{--`—`               --}{>>r333 @AS<<}|{--Called on every document change               --}{>>r335 @AS<<}|
|{--`onSave`               --}{>>r338 @AS<<}|{--`function`               --}{>>r340 @AS<<}|{--`—`               --}{>>r342 @AS<<}|{--Called on save, receives ArrayBuffer               --}{>>r344 @AS<<}|
|{--`onError`               --}{>>r347 @AS<<}|{--`function`               --}{>>r349 @AS<<}|{--`—`               --}{>>r351 @AS<<}|{--Called on error               --}{>>r353 @AS<<}|

<!-- p172 empty, break-del r354 @AS -->

<!-- p173 page-break-del r356 @AS, break-del r355 @AS -->

<!-- p174 break-del r357 @AS -->
{--**Ref methods**         --}{>>r358 @AS<<}

<!-- t8 1x1, cells t8.row0 p175-p181, borders single 0.5pt, break-del r360 @AS in p175; r362 @AS in p176; r363 @AS in p177; r365 @AS in p178; r367 @AS in p179; r369 @AS in p180; r371 @AS in p181, rev-del r359 @AS in t8.row0 -->
|{--`const ref = useRef<DocxEditorRef>(null)`               --}{>>r361 @AS<<}<br>{--<br>`await ref.current.save() // ArrayBuffer of the .docx`               --}{>>r364 @AS<<}{--<br>`ref.current.getDocument() // current document object`               --}{>>r366 @AS<<}{--<br>`ref.current.setZoom(1.5) // set zoom to 150%`               --}{>>r368 @AS<<}{--<br>`ref.current.scrollToPage(3) // scroll to page 3`               --}{>>r370 @AS<<}{--<br>`ref.current.print() // print`               --}{>>r372 @AS<<}|
|-|

<!-- p182 empty, break-del r373 @AS -->

<!-- p183 break-del r374 @AS -->
{--**4. Contributing**         --}{>>r375 @AS<<}

<!-- p184 break-del r376 @AS -->
{--Read         [<u>`docs/PLUGINS.md`</u>](https://github.com/eigenpal/docx-editor/blob/main/docs/PLUGINS.md)           before opening a pull request that touches the plugin API.         --}{>>r377 @AS<<}

<!-- p185 empty, break-del r378 @AS -->

<!-- p186 break-del r379 @AS -->
{--**Ground rules**         --}{>>r380 @AS<<}

<!-- p187 ListParagraph bullet, break-del r381 @AS -->
- {--Every PR must pass the round-trip test: load a .docx, apply an edit, export, reload — content and formatting must be stable on unchanged content.         --}{>>r382 @AS<<}

<!-- p188 ListParagraph bullet, break-del r383 @AS -->
- {--Run         `bun test`         and         `bun run test:e2e`         locally before pushing.         --}{>>r384 @AS<<}

<!-- p189 ListParagraph bullet, break-del r385 @AS -->
- {--Avoid intermediate HTML serialisation where possible.           --}{>>r386 @AS<<}

<!-- p190 ListParagraph bullet, break-del r387 @AS -->
- {--Tracked changes and comments must survive a full OOXML round-trip — save, open in Word, verify changes are reviewable, reload in the editor.         --}{>>r388 @AS<<}

<!-- p191 empty, break-del r389 @AS -->

<!-- p192 break-del r390 @AS -->
{--**Dev setup**         --}{>>r391 @AS<<}

<!-- t9 1x1, cells t9.row0 p193-p194, borders single 0.5pt, break-del r393 @AS in p193; r395 @AS in p194, rev-del r392 @AS in t9.row0 -->
|{--`bun install`               --}{>>r394 @AS<<}{--<br>`bun run dev // Vite demo on localhost:5173`               --}{>>r396 @AS<<}|
|-|

<!-- p195 empty, break-del r397 @AS -->

<!-- p196 break-del r398 @AS -->
{--**Sign-off**         --}{>>r399 @AS<<}

<!-- p197 empty, break-del r400 @AS -->

<!-- t10 1x2, cells t10.row0 p198-p205, break-del r402 @AS in p198; r404 @AS in p199; r406 @AS in p200; r407 @AS in p201; r409 @AS in p202; r411 @AS in p203; r413 @AS in p204; r414 @AS in p205, rev-del r401 @AS in t10.row0 -->
|{--**EigenPal**               --}{>>r403 @AS<<}{--<br>Maintainer · eigenpal.com               --}{>>r405 @AS<<}<br>{--<br>signature · date               --}{>>r408 @AS<<}|{--**Contributor**               --}{>>r410 @AS<<}{--<br>on behalf of the community               --}{>>r412 @AS<<}<br>{--<br>signature · date               --}{>>r415 @AS<<}|
|-|-|

<!-- p206 empty, break-del r416 @AS -->

<!-- p207 empty, break-del r417 @AS -->

<!-- p208 break-del r418 @AS -->
{--**Sign-off**         --}{>>r419 @AS<<}

<!-- p209 empty, break-del r420 @AS -->

<!-- t11 1x2, cells t11.row0 p210-p217, break-del r422 @AS in p210; r424 @AS in p211; r426 @AS in p212; r427 @AS in p213; r429 @AS in p214; r431 @AS in p215; r433 @AS in p216; r434 @AS in p217, rev-del r421 @AS in t11.row0 -->
|{--**EigenPal**               --}{>>r423 @AS<<}{--<br>Maintainer · eigenpal.com               --}{>>r425 @AS<<}<br>{--<br>signature · date               --}{>>r428 @AS<<}|{--**Contributor**               --}{>>r430 @AS<<}{--<br>on behalf of the community               --}{>>r432 @AS<<}<br>{--<br>signature · date               --}{>>r435 @AS<<}|
|-|-|

<!-- p218 empty -->
