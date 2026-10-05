---
name: Jubarte
description: Word-faithful redlines and rendering, without Word. A redline desk in graphite around paper-white documents.
colors:
  graphite-paper: "oklch(97% 0.003 106.4)"
  graphite-paper-night: "oklch(20% 0.023 249)"
  graphite-ink: "oklch(18.2% 0 0)"
  graphite-ink-night: "oklch(96.2% 0.008 241.7)"
  graphite-lead: "oklch(21.8% 0 0)"
  night-ink-blue: "oklch(79.1% 0.094 237.6)"
  graphite-lead-pressed: "oklch(0% 0 0)"
  night-ink-blue-pressed: "oklch(86.2% 0.065 236)"
  deep: "oklch(20.5% 0 0)"
  deep-night: "oklch(94.5% 0.013 244.3)"
  on-deep: "oklch(100% 0 0)"
  on-deep-night: "oklch(22.7% 0.038 244.5)"
  hairline: "oklch(87.5% 0.005 106.5)"
  hairline-night: "oklch(33.4% 0.039 250.5)"
  hairline-strong: "oklch(73.3% 0.007 106.6)"
  hairline-strong-night: "oklch(43.6% 0.043 246.4)"
  surface: "oklch(94.2% 0.004 106.5)"
  surface-night: "oklch(23.9% 0.026 249)"
  surface-sunk: "oklch(91.2% 0.005 106.5)"
  surface-sunk-night: "oklch(17.5% 0.02 252.3)"
  hover: "oklch(92.4% 0.005 106.5)"
  hover-night: "oklch(27.1% 0.029 246.9)"
  highlighter-yellow: "oklch(91.6% 0.128 94.1)"
  highlighter-night: "oklch(40.5% 0.075 243)"
  highlighter-soft: "oklch(96.2% 0.062 95.4)"
  highlighter-soft-night: "oklch(28.4% 0.046 245.3)"
  disabled: "oklch(48.5% 0.005 106.6)"
  disabled-night: "oklch(67.8% 0.028 244.6)"
  insertion-blue: "oklch(45.3% 0.209 262)"
  insertion-blue-night: "oklch(83.2% 0.087 249.4)"
  insertion-wash: "oklch(92.8% 0.034 267.6)"
  insertion-wash-night: "oklch(30.6% 0.071 253.5)"
  deletion-red: "oklch(52.7% 0.199 27.5)"
  deletion-red-night: "oklch(79.4% 0.119 30)"
  deletion-wash: "oklch(93.6% 0.032 17.7)"
  deletion-wash-night: "oklch(29.3% 0.073 28.4)"
  move-green: "oklch(51% 0.13 150.7)"
  move-green-night: "oklch(81.9% 0.108 163.1)"
  move-wash: "oklch(94.4% 0.031 154.2)"
  move-wash-night: "oklch(31.8% 0.055 162.9)"
  page-white: "oklch(100% 0 0)"
typography:
  display:
    fontFamily: "Manrope, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "clamp(40px, 6vw, 72px)"
    fontWeight: 700
    lineHeight: 1.08
    letterSpacing: "-0.034em"
  headline:
    fontFamily: "Manrope, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "clamp(30px, 4.4vw, 48px)"
    fontWeight: 700
    lineHeight: 1.05
    letterSpacing: "-0.03em"
  title:
    fontFamily: "Manrope, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "clamp(28px, 3.4vw, 38px)"
    fontWeight: 600
    lineHeight: 1.1
    letterSpacing: "-0.025em"
  section:
    fontFamily: "Manrope, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "19px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  lead:
    fontFamily: "Manrope, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: 1.55
  body:
    fontFamily: "Manrope, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "JetBrains Mono, ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0.14em"
  code:
    fontFamily: "JetBrains Mono, ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.6
  document:
    fontFamily: "Source Serif 4, Georgia, serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.6
rounded:
  check: "3px"
  page: "4px"
  control: "6px"
  dot: "50%"
spacing:
  xs: "12px"
  sm: "18px"
  md: "24px"
  lg: "44px"
  xl: "56px"
  2xl: "72px"
components:
  button-primary:
    backgroundColor: "{colors.graphite-lead}"
    textColor: "{colors.on-deep}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "12px 20px"
  button-primary-hover:
    backgroundColor: "{colors.graphite-lead-pressed}"
  button-outline:
    backgroundColor: "{colors.graphite-paper}"
    textColor: "{colors.graphite-lead}"
    rounded: "{rounded.control}"
    padding: "12px 20px"
  button-ghost:
    backgroundColor: "{colors.graphite-paper}"
    textColor: "{colors.graphite-ink}"
    rounded: "{rounded.control}"
    padding: "12px 20px"
  button-ghost-hover:
    backgroundColor: "{colors.hover}"
  button-disabled:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.disabled}"
    rounded: "{rounded.control}"
    padding: "12px 20px"
  button-sm:
    padding: "9px 16px"
  tag:
    backgroundColor: "{colors.graphite-paper}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "5px 9px"
  pill:
    backgroundColor: "{colors.highlighter-soft}"
    textColor: "{colors.graphite-lead-pressed}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  field:
    backgroundColor: "{colors.graphite-paper}"
    textColor: "{colors.graphite-ink}"
    rounded: "{rounded.control}"
    padding: "0 14px"
  code-block:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.code}"
    rounded: "{rounded.control}"
    padding: "10px 12px"
  cell:
    backgroundColor: "{colors.graphite-paper}"
    textColor: "{colors.graphite-ink}"
    padding: "26px 24px 28px"
  document-page:
    backgroundColor: "{colors.page-white}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.document}"
    rounded: "{rounded.page}"
---

# Design System: Jubarte

## Overview

**Creative North Star: "The Redline Desk"**

Jubarte is a working desk for one job: reading what changed between two
documents. Graphite chrome sits around paper-white pages, so a redline reads
like a document and not like a diff. The interface is an instrument. It is
drawn in hairlines, labelled in small mono capitals, and kept out of the way.
The colour on the page belongs to the work: Word's revision marks, and one
yellow highlighter that marks the claim.

The system is precise and quiet. Surfaces are flat. Structure comes from 1px
hairlines and from grids of cells that share them. Controls have slightly
rounded corners, and only the document and the scores pull the eye. Two
themes share every token: **Graphite** (light, the default) and **Night**
(dark, used when the browser or macOS asks for it). On jubarte.pro a footer
switch can pin either one. A document page stays white paper in both.

The same tokens drive jubarte.pro (`jubarte-site/site/static/css/site.css`)
and the Mac app (`src/styles.css`). Every colour is an OKLCH `light-dark()`
pair. The frontmatter lists each Graphite value with its Night sibling
(`*-night`).

**Key Characteristics:**
- Graphite chrome, paper-white documents, maximum text contrast.
- 1px hairlines and shared-border cell grids instead of shadows.
- Manrope for words, JetBrains Mono for labels, code and figures, and Source
  Serif 4 for the document itself.
- One highlighter (yellow in Graphite, ink blue in Night) for the claim.
- Word's revision colours (blue insertions, red deletions, green moves) keep
  that meaning everywhere.

## Colors

A near-neutral graphite with one warm highlighter. Night trades both for
deep-water blues.

### Primary
- **Graphite Lead**, with **Night Ink Blue** in Night: the brand colour.
  It is the primary call to action (`--cta` is `--primary`), links, the
  active nav underline, focus rings and the "ours" bars. In Graphite it is
  nearly black. Night swaps it for a light, slightly saturated blue.
- **Deep**: the app's title bar and the "Open in Word" button inside the
  Mac app. It is near-black in Graphite and near-white in Night; text on it
  is **On-Deep**. On the site it is never a button.

### Secondary
- **Highlighter Yellow**, and the deep blue highlighter in Night: the marker
  stroke behind the claim ("Without Word.", "Perfect redlines"), text
  selection, a slot with a file dragged over it, and the "ours" rows. Its
  soft tint (`highlighter-soft`) fills featured cells and pills.

### Tertiary: revision marks
- **Insertion Blue**, **Deletion Red** and **Move Green**, each with a pale
  wash. These are Word's semantics, and they appear only where revisions
  appear: chips, previews and the redlines themselves. A chip whose count is
  zero is a plain tag: no revision, no revision colour.
- **The marks.** An insertion or a deletion is marked once: blue and
  underlined, red and struck. A move is marked twice, in green: double-struck
  where it left, double-underlined where it landed. Marking a move twice is
  what tells it from an insertion without relying on colour. The struck
  source shows where the text came from, so no "moved from" label is added.
- **The chips are the legend.** Each count wears its revision's mark (the
  deleted count is struck, not negative), so no separate legend repeats them.
  Formatting changes get a chip only when the redline has some.

### Neutral
- **Graphite Paper**: the page background.
- **Graphite Ink**: all text.
- **Surface** and **Surface Sunk**: raised and recessed panels, and code.
- **Hover**: the hover fill.
- **Hairline**: dividers and the cell grid.
- **Hairline Strong**: emphasised borders and the disabled outline.
- **Disabled**: unavailable text.
- **Page White**: the document page, which never changes.

### Named Rules
**The Max Contrast Rule.** Secondary text is ink, not gray (`--muted` is
`--ink`). Hierarchy comes from size, weight and the mono face, never from
fading the text.

**The One Highlighter Rule.** The highlighter means "this is the claim". Use
it only on the claim, the selection and the "ours" rows, and never as
decoration.

**The Word Colours Rule.** Blue, red and green mean inserted, deleted and
moved. Never use them for anything else.

**The Paper Stays Paper Rule.** Every document page, strip, thumbnail and
preview sets `color-scheme: light` and **Page White**. The theme never
re-colours the user's document.

## Typography

**Display Font:** Manrope (with system-ui)
**Body Font:** Manrope (with system-ui)
**Label/Mono Font:** JetBrains Mono (with ui-monospace)
**Document Font:** Source Serif 4 (with Georgia), only inside document
previews

**Character:** a warm geometric sans for statements and prose, and a mono
for everything that is a label, a command, a file name or a figure. The
serif appears only on the paper, so the document looks like a document.

### Hierarchy
- **Display** (700, 40–72px clamp, line-height 1.08, -0.034em): the home
  hero only, at most 14ch wide.
- **Headline** (700, 30–48px clamp, line-height 1.05, -0.03em): page titles.
- **Title** (600, 28–38px clamp, line-height 1.1, -0.025em): major sections.
  Headings balance their lines.
- **Section** (600, 19px, line-height 1.3): section-head titles and the
  titles of grouped blocks.
- **Lead** (17px, line-height 1.55, at most 62ch) and **Body** (16px,
  line-height 1.6): prose, which wraps "pretty".
- **Label** (JetBrains Mono 11px, 0.06–0.14em tracking, uppercase): page-head
  kickers, tags, slot labels, table heads and the footer. The nav uses the
  same uppercase voice in Manrope.
- **Code** (JetBrains Mono 12px, line-height 1.6): command blocks. Figures
  are mono, so scores line up in columns.

### Named Rules
**The 11px Floor Rule.** No functional text is set below 11px.

**The Reader's Size Rule.** The sizes above are given at the browser's
default of 16px. The stylesheet sets them in rem (px ÷ 16, so 11px is
0.6875rem) and leaves the root size alone, so a reader who raises the
default font size gets all of the type larger.

**The Page-Head Label Rule.** An uppercase mono label may sit above a page's
title. Section heads are plain sentence-case headings with a mono meta line
beside them, with no eyebrow and no section numbers.

## Layout

- **Content:** a 1180px max container. Pages are bands that alternate between
  paper and surface.
- **Cell grids:** a grid with a 1px gap over a hairline background, so the
  cells share borders. Columns are `auto-fit` (minimums of 200 or 240px).
  Cell padding is 26px 24px 28px.
- **Rhythm:** an observed step scale of 12, 18, 24, 28, 36, 40, 44, 56, 64
  and 72px. The tokens hold the common steps. There is more space above a
  section than inside it.
- **Breakpoints:**
  - at 1000px, the Contact grid and the release columns stack;
  - at 760px, the nav collapses into a menu, the gutters narrow to 18px, and
    two-column panels (`m-stack`) go single-column.
- **Touch:** under `(pointer: coarse)`, standalone links get a 44px target.
- The nav is sticky, with a hairline bottom border. The scrollbar keeps its
  gutter, so the nav never shifts between short and long pages.

## Elevation & Depth

Flat by default. Depth comes from tone (Paper, Surface and Surface Sunk) and
from hairlines, not from shadows. Exactly two shadows exist, both on mock
windows that stand in for a real application: the browser frame on Benchmark,
and the app window on App.

### Shadow Vocabulary
- **Mock window, browser** (`box-shadow: 0 24px 60px -36px var(--shadow)`):
  the case viewer's browser frame.
- **Mock window, app** (`box-shadow: 0 24px 60px -30px var(--shadow)`): the
  simulated Mac app window.

### Named Rules
**The Hairline Rule.** Elevation is a 1px border or a tonal step, never
both, and never a soft shadow under a card.

## Shapes

- **Controls** (6px): buttons, fields, segmented tabs, toasts, code blocks
  and the theme switch.
- **Documents** (4px): the corners of document pages, so they read as paper.
- **Small marks** (3px): checkboxes and the "soon" badge.
- **Dots** (50%): status and pulse dots.
- **Cell grids:** square corners on the cells inside, with the radius on the
  outer frame only.
- **Lines:** borders are 1px everywhere, except the 2px underline on the
  active nav item.

## Components

### Buttons
- **Shape:** gently rounded (6px). 14px Manrope 600, padding 12px 20px.
  `btn-sm` is 9px 16px and `btn-lg` is 13px 22px.
- **Variants:**
  - **Primary:** a solid fill of the brand colour (Graphite Lead, or Night
    Ink Blue in Night) with On-Deep text. `btn-cta` is the same rule.
  - **Outline:** a brand-colour border and text on paper.
  - **Ghost:** a hairline border with ink text.
  - **Text:** no border at all.
- **Hover / Focus:** fills step to their pressed shade or to Hover over
  0.15s. Focus is a 2px brand-colour outline offset 2px.
- **Disabled:** a dashed Hairline Strong outline on Surface with Disabled
  text, in every variant. Never a faded fill, which reads as enabled.

### Chips
- **Tag:** an uppercase mono label in a hairline box on paper.
- **Pill:** an uppercase mono label on Highlighter Soft.

### Cards / Containers
- **Cells** share 1px hairlines inside a grid frame and have no shadow.
- A featured cell is tinted with Highlighter Soft. It never gets a side
  stripe.

### Inputs / Fields
- A hairline box on paper with a 6px radius. Its mono label sits inside on
  the left.
- **Focus:** the border turns the brand colour.
- **Drop slots** (Demo and the app) are dashed Hairline Strong boxes that
  fill with the Highlighter while a file is over them.

### Navigation
- **Links:** uppercase Manrope (12px, 500, 0.06em tracking), ink. The active
  item gets a 2px brand-colour underline.
- **Download** is an outlined brand-colour button that fills when it is the
  current page.
- **Below 760px:** a "Menu" disclosure replaces the links.
- **Footer:** small mono links, with the light/dark switch, whose label names
  the mode a click turns on.

### Settings sheet (Mac app)
- A sheet over a scrim, opened from the title bar's gear or ⌘,. The groups
  run in order of use: tracked-change marks, then "Revisions by", then
  appearance. Each group has a plain heading and no eyebrow label.
- The agent fingerprint is a plain labelled field under "Revisions by", in the
  mono face because it is machine text, with its note tied to it through
  `aria-describedby`.
- Changes apply as they are made; Done closes the sheet. Line styles use
  Word's own Track Changes names (Underline, Double strikethrough, No line).
- Custom marks override the mark custom properties on `:root` (`--ins`,
  `--ins-mark`, `--movfrom`, …), so the preview, the chips and the sheet's
  sample all follow them. The paper stays light; on a dark window a custom
  colour is lightened so the chips stay readable.

### Installer
- One code block with a tab per surface (CLI, Rust, Python, Node · browser)
  instead of four similar cards.
- **Keyboard:** roving tabindex, and the arrow keys move between tabs.

### Document strip
- Three Page White pages side by side: Word's, jubarte's and LibreOffice's.
  Each has a 4px radius, with the scores under it.
- A page swaps in only after it has decoded. Until then it says "Loading
  page…" and never paints blank white.

## Do's and Don'ts

### Do:
- **Do** write colours as OKLCH `light-dark()` pairs. A test bans hex, rgb
  and hsl in site.css.
- **Do** keep secondary text at ink contrast and build hierarchy with size,
  weight and the mono face.
- **Do** set `color-scheme: light` on anything that shows a document.
- **Do** use the 6px radius on controls and 4px on pages.
- **Do** separate content with 1px hairlines and shared-border cell grids.
- **Do** keep commands and figures in JetBrains Mono, and wrap commands
  rather than cutting them off.

### Don't:
- **Don't** put an eyebrow or a number (01 / 02) above section headings.
  Labels belong on page heads only.
- **Don't** use side-stripe accents, gradient text or soft drop shadows on
  cards.
- **Don't** use the highlighter or the revision colours as decoration.
- **Don't** fade a disabled button's fill. Disabled is a dashed outline.
- **Don't** add a second accent colour; the orange was removed on purpose.
- **Don't** set functional text below 11px.
