---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 226                     # 213 marks shown as 211 changes: 2 insertions, 85 deletions, 2 substitutions, 109 paragraph marks, 13 table rows; 5 formatting; 2 in header1; 2 in footer1; 2 in endnotes; 2 in footnotes
comments: 1 thread open, style Normal  # 1 comment: c73
authors:
  document_owner: Un-named         # dc:creator
  ASR: Arthur Souza Rodrigues      # 226 revisions, 2026-09-26T14:54:00Z
  U: User                          # 1 comment, 2026-03-05T19:41:00Z
body: p0-p110, 6 tables, 3 pages   # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in; fmt r226 @ASR
styles:
  Normal: Inter 11pt, left         # default; unannotated paragraphs use it
  ListParagraph: Inter 11pt, named List Paragraph
  table: TableNormal
notes: {fmt: footnotes.r0 @ASR; footnotes.r1 @ASR; endnotes.r2 @ASR; endnotes.r3 @ASR, empty: [footnotes, endnotes]}  # changes in notes the body does not show (separators)
headers:
  default: {id: header1, text: "{--Docx Editor: Project Charter & Contributor Agreement       --}{>>header1.r223 @ASR<<}", style: Normal, fmt: header1.r222 @ASR in header1}
footers:
  default: {id: footer1, text: "{--Page       {PAGE \\* MERGEFORMAT}1       {/}of       {NUMPAGES \\* MERGEFORMAT}1       {/}·       {HYPERLINK \\h}eigenpal.com         {/}--}{>>footer1.r225 @ASR<<}", style: ListParagraph, fmt: footer1.r224 @ASR in footer1}
---

<!-- p0 fmt r4 @ASR; r5 @ASR; r6 @ASR; r9 @ASR -->
_file\__{~~**14**~>_15_~~}{>>r8+7 @ASR<<}_.docx_

<!-- p1 break-ins r10 @ASR -->
{++_Verdana Italic Centered Demo_++}{>>r11 @ASR<<}

<!-- p2 break-ins r12 @ASR -->
{++_This document shows Verdana font with italic and center alignment._++}{>>r13 @ASR<<}

<!-- p3 break-del r14 @ASR -->
{~~**eigenpal/docx-editor**         ~>_Verdana italic centered creates a clean and modern look._~~}{>>r16+15 @ASR<<}

<!-- p4 break-del r17 @ASR -->
{--Project Charter & Contributor Agreement         --}{>>r18 @ASR<<}

<!-- p5 empty, break-del r19 @ASR -->

<!-- t0 1x2, cells p6-p7 by row, borders single 0.5pt, break-del r21 @ASR in p6; r23 @ASR in p7, rev-del r20 @ASR in t0.row0 -->
|{--npm               [<u>`@eigenpal/docx-js-editor`</u>](https://www.npmjs.com/package/@eigenpal/docx-js-editor)                 --}{>>r22 @ASR<<}|{--github               [<u>`eigenpal/docx-editor`</u>](https://github.com/eigenpal/docx-editor)                 --}{>>r24 @ASR<<}|
|-|-|

<!-- p8 break-del r25 @ASR -->
{--**1. What this is**         --}{>>r26 @ASR<<}

<!-- p9 break-del r27 @ASR -->
{--An open-source WYSIWYG         `.docx`         editor for React. Drop in         `<DocxEditor documentBuffer={file} />`         and your users can open, edit, and save Word documents entirely in the browser —         no backend required           , no document data leaving the client.         --}{>>r28 @ASR<<}

<!-- p10 empty, break-del r29 @ASR -->

<!-- p11 break-del r30 @ASR -->
{--Most editors serialise to HTML and back:         `DOCX → HTML → edits → HTML → DOCX`         . Every hop loses formatting, tracked changes, and styles.         This library skips it           — OOXML is the canonical format throughout.         --}{>>r31 @ASR<<}

<!-- p12 empty, break-del r32 @ASR -->

<!-- t1 1x1, cells t1.row0 p13-p23, borders single 0.5pt, break-del r34 @ASR in p13; r36 @ASR in p14; r38 @ASR in p15; r39 @ASR in p16; r41 @ASR in p17; r43 @ASR in p18; r45 @ASR in p19; r47 @ASR in p20; r49 @ASR in p21; r51 @ASR in p22; r53 @ASR in p23, rev-del r33 @ASR in t1.row0 -->
|{--`import { DocxEditor } from '@eigenpal/docx-js-editor'`               --}{>>r35 @ASR<<}{--<br>`import '@eigenpal/docx-js-editor/styles.css'`               --}{>>r37 @ASR<<}<br>{--<br>`function App({ file }: { file: ArrayBuffer }) {`               --}{>>r40 @ASR<<}{--<br>`return (`               --}{>>r42 @ASR<<}{--<br>`<DocxEditor`               --}{>>r44 @ASR<<}{--<br>`documentBuffer={file}`               --}{>>r46 @ASR<<}{--<br>`onSave={(buffer) => uploadToServer(buffer)}`               --}{>>r48 @ASR<<}{--<br>`/>`               --}{>>r50 @ASR<<}{--<br>`)`               --}{>>r52 @ASR<<}{--<br>`}`               --}{>>r54 @ASR<<}|
|-|

<!-- p24 empty, break-del r55 @ASR -->

<!-- p25 break-del r56 @ASR -->
{--**2. Collaboration — Tracked Changes & Comments**         --}{>>r57 @ASR<<}

<!-- p26 break-del r58 @ASR -->
{--Both features round-trip through OOXML so documents open correctly in Word, LibreOffice, and Google Docs.         --}{>>r59 @ASR<<}

<!-- p27 empty, break-del r60 @ASR -->

<!-- p28 break-del r61 @ASR -->
{--**Suggestion mode**         --}{>>r62 @ASR<<}

<!-- p29 break-del r63 @ASR -->
{--Toggle between         `Editing`         and         `Suggesting`         via the toolbar dropdown. In Suggesting mode, every edit becomes a tracked insertion (         green underline         ) or deletion (         red strikethrough         ). Consecutive keystrokes group into a single change. Accept or reject individually or all at once.         --}{>>r64 @ASR<<}

<!-- p30 empty, break-del r65 @ASR -->

<!-- p31 break-del r66 @ASR -->
{--Serialises to         `w:ins`         /         `w:del`         — the same OOXML markup Word uses. Tracked changes survive a save/reload cycle and are reviewable in Word with full accept/reject UI.         --}{>>r67 @ASR<<}

<!-- p32 empty, break-del r68 @ASR -->

<!-- p33 break-del r69 @ASR -->
{--**Comments**         --}{>>r70 @ASR<<}

<!-- p34 break-del r71 @ASR -->
{--Select text and click the floating button on the page edge to         {==add a comment anchored to that selection         ==}{>>c73 @U: Just like that!<<}. A Google Docs-style sidebar shows threaded cards positioned at their anchor point in the document. Reply, resolve, reopen, or delete inline.         --}{>>r72 @ASR<<}

<!-- p35 empty, break-del r74 @ASR -->

<!-- p36 break-del r75 @ASR -->
{--Serialises to         `comments.xml`         with         `w:commentRangeStart`         /         `w:commentRangeEnd`         markers. Comments and reply threads are preserved on round-trip.         --}{>>r76 @ASR<<}

<!-- p37 empty, break-del r77 @ASR -->

<!-- p38 break-del r78 @ASR -->
{--**Agent-friendly**         --}{>>r79 @ASR<<}

<!-- p39 break-del r80 @ASR -->
{--Because tracked changes and comments are native OOXML, an AI agent can write         `w:ins`         /         `w:del`         /         `w:comment`         markup directly into a document and hand it to         `<DocxEditor />`         for human review — no custom diff format, no proprietary API.         --}{>>r81 @ASR<<}

<!-- p40 empty, break-del r82 @ASR -->

<!-- p41 break-del r83 @ASR -->
{--**3. Props**         --}{>>r84 @ASR<<}

<!-- p42 break-del r85 @ASR -->
{--Pass         `author="Your App"`         to set the author name for comments and tracked changes. Toggle suggestion mode via the toolbar dropdown.         --}{>>r86 @ASR<<}

<!-- p43 empty, break-del r87 @ASR -->

<!-- t2 8x4, cells p44-p75 by row, borders single 0.5pt, break-del r89 @ASR in p44; r91 @ASR in p45; r93 @ASR in p46; r95 @ASR in p47; r98 @ASR in p48; r100 @ASR in p49; r102 @ASR in p50; r104 @ASR in p51; r107 @ASR in p52; r109 @ASR in p53; r111 @ASR in p54; r113 @ASR in p55; r116 @ASR in p56; r118 @ASR in p57; r120 @ASR in p58; r122 @ASR in p59; r125 @ASR in p60; r127 @ASR in p61; r129 @ASR in p62; r131 @ASR in p63; r134 @ASR in p64; r136 @ASR in p65; r138 @ASR in p66; r140 @ASR in p67; r143 @ASR in p68; r145 @ASR in p69; r147 @ASR in p70; r149 @ASR in p71; r152 @ASR in p72; r154 @ASR in p73; r156 @ASR in p74; r158 @ASR in p75, rev-del r88 @ASR in t2.row0; r97 @ASR in t2.row1; r106 @ASR in t2.row2; r115 @ASR in t2.row3; r124 @ASR in t2.row4; r133 @ASR in t2.row5; r142 @ASR in t2.row6; r151 @ASR in t2.row7 -->
|{--**Prop**               --}{>>r90 @ASR<<}|{--**Type**               --}{>>r92 @ASR<<}|{--**Default**               --}{>>r94 @ASR<<}|{--**Description**               --}{>>r96 @ASR<<}|
|-|-|-|-|
|{--`documentBuffer`               --}{>>r99 @ASR<<}|{--`ArrayBuffer`               --}{>>r101 @ASR<<}|{--`—`               --}{>>r103 @ASR<<}|{--.docx file contents to load               --}{>>r105 @ASR<<}|
|{--`readOnly`               --}{>>r108 @ASR<<}|{--`boolean`               --}{>>r110 @ASR<<}|{--`false`               --}{>>r112 @ASR<<}|{--Preview mode — no editing UI               --}{>>r114 @ASR<<}|
|{--`showToolbar`               --}{>>r117 @ASR<<}|{--`boolean`               --}{>>r119 @ASR<<}|{--`true`               --}{>>r121 @ASR<<}|{--Show formatting toolbar               --}{>>r123 @ASR<<}|
|{--`author`               --}{>>r126 @ASR<<}|{--`string`               --}{>>r128 @ASR<<}|{--`'User'`               --}{>>r130 @ASR<<}|{--Author name for comments and tracked changes               --}{>>r132 @ASR<<}|
|{--`onChange`               --}{>>r135 @ASR<<}|{--`function`               --}{>>r137 @ASR<<}|{--`—`               --}{>>r139 @ASR<<}|{--Called on every document change               --}{>>r141 @ASR<<}|
|{--`onSave`               --}{>>r144 @ASR<<}|{--`function`               --}{>>r146 @ASR<<}|{--`—`               --}{>>r148 @ASR<<}|{--Called on save, receives ArrayBuffer               --}{>>r150 @ASR<<}|
|{--`onError`               --}{>>r153 @ASR<<}|{--`function`               --}{>>r155 @ASR<<}|{--`—`               --}{>>r157 @ASR<<}|{--Called on error               --}{>>r159 @ASR<<}|

<!-- p76 empty, break-del r160 @ASR -->

<!-- p77 page-break-del r162 @ASR, break-del r161 @ASR -->

<!-- p78 break-del r163 @ASR -->
{--**Ref methods**         --}{>>r164 @ASR<<}

<!-- t3 1x1, cells t3.row0 p79-p85, borders single 0.5pt, break-del r166 @ASR in p79; r168 @ASR in p80; r169 @ASR in p81; r171 @ASR in p82; r173 @ASR in p83; r175 @ASR in p84; r177 @ASR in p85, rev-del r165 @ASR in t3.row0 -->
|{--`const ref = useRef<DocxEditorRef>(null)`               --}{>>r167 @ASR<<}<br>{--<br>`await ref.current.save() // ArrayBuffer of the .docx`               --}{>>r170 @ASR<<}{--<br>`ref.current.getDocument() // current document object`               --}{>>r172 @ASR<<}{--<br>`ref.current.setZoom(1.5) // set zoom to 150%`               --}{>>r174 @ASR<<}{--<br>`ref.current.scrollToPage(3) // scroll to page 3`               --}{>>r176 @ASR<<}{--<br>`ref.current.print() // print`               --}{>>r178 @ASR<<}|
|-|

<!-- p86 empty, break-del r179 @ASR -->

<!-- p87 break-del r180 @ASR -->
{--**4. Contributing**         --}{>>r181 @ASR<<}

<!-- p88 break-del r182 @ASR -->
{--Read         [<u>`docs/PLUGINS.md`</u>](https://github.com/eigenpal/docx-editor/blob/main/docs/PLUGINS.md)           before opening a pull request that touches the plugin API.         --}{>>r183 @ASR<<}

<!-- p89 empty, break-del r184 @ASR -->

<!-- p90 break-del r185 @ASR -->
{--**Ground rules**         --}{>>r186 @ASR<<}

<!-- p91 ListParagraph bullet, break-del r187 @ASR -->
- {--Every PR must pass the round-trip test: load a .docx, apply an edit, export, reload — content and formatting must be stable on unchanged content.         --}{>>r188 @ASR<<}

<!-- p92 ListParagraph bullet, break-del r189 @ASR -->
- {--Run         `bun test`         and         `bun run test:e2e`         locally before pushing.         --}{>>r190 @ASR<<}

<!-- p93 ListParagraph bullet, break-del r191 @ASR -->
- {--Avoid intermediate HTML serialisation where possible.           --}{>>r192 @ASR<<}

<!-- p94 ListParagraph bullet, break-del r193 @ASR -->
- {--Tracked changes and comments must survive a full OOXML round-trip — save, open in Word, verify changes are reviewable, reload in the editor.         --}{>>r194 @ASR<<}

<!-- p95 empty, break-del r195 @ASR -->

<!-- p96 break-del r196 @ASR -->
{--**Dev setup**         --}{>>r197 @ASR<<}

<!-- t4 1x1, cells t4.row0 p97-p98, borders single 0.5pt, break-del r199 @ASR in p97; r201 @ASR in p98, rev-del r198 @ASR in t4.row0 -->
|{--`bun install`               --}{>>r200 @ASR<<}{--<br>`bun run dev // Vite demo on localhost:5173`               --}{>>r202 @ASR<<}|
|-|

<!-- p99 empty, break-del r203 @ASR -->

<!-- p100 break-del r204 @ASR -->
{--**Sign-off**         --}{>>r205 @ASR<<}

<!-- p101 empty, break-del r206 @ASR -->

<!-- t5 1x2, cells t5.row0 p102-p109, break-del r208 @ASR in p102; r210 @ASR in p103; r212 @ASR in p104; r213 @ASR in p105; r215 @ASR in p106; r217 @ASR in p107; r219 @ASR in p108; r220 @ASR in p109, rev-del r207 @ASR in t5.row0 -->
|{--**EigenPal**               --}{>>r209 @ASR<<}{--<br>Maintainer · eigenpal.com               --}{>>r211 @ASR<<}<br>{--<br>signature · date               --}{>>r214 @ASR<<}|{--**Contributor**               --}{>>r216 @ASR<<}{--<br>on behalf of the community               --}{>>r218 @ASR<<}<br>{--<br>signature · date               --}{>>r221 @ASR<<}|
|-|-|

<!-- p110 empty -->
