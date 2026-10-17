# Enums

199 items (non-test). Part of the [symbol manifest](README.md).

| Name | Defined at | Vis | Does | Detail |
|---|---|---|---|---|
| `AdmissionErrorKind` | src/admission.rs:136 | pub | Why a package was refused. | variants: InputLimit,DuplicatePart,UnsupportedPackage,InvalidPackage,InvalidXml,LegacyDocument |
| `SectionBreak` | src/append.rs:38 | pub | What separates A's content from B's. | variants: NextPage,Continuous,None |
| `AppendComments` | src/append.rs:54 | pub | What happens to B's comments. | variants: Drop,Carry |
| `AppendError` | src/append.rs:88 | pub | Why two documents could not be appended. | variants: Admission,Package,Invalid |
| `AuditError` | src/audit.rs:85 | pub | Why an audit could not run. | variants: Inspect,Layout,UnknownRule |
| `Produced` | src/bin/jubarte.rs:263 | private | A Word package, or text already in the output's format (CriticMarkup, convert's Markdown). | variants: Docx,Text |
| `Mode` | src/bin/jubarte.rs:889 | private | Whether `edit` and `add` track their changes. | variants: Suggesting,Editing |
| `ChangeKind` | src/changes.rs:32 | pub | What a change does. | variants: Insertion,Deletion,Move,Formatting |
| `MoveSide` | src/changes.rs:46 | pub | Which side of a move a change is. | variants: From,To |
| `ChangeError` | src/changes.rs:192 | pub | Why changes could not be listed or resolved. | variants: Package,UnknownChange,InvalidId,Ambiguous |
| `Group` | src/changes.rs:415 | private | The revisions that resolve together: a move's sides and range markers, or one element on its own. | variants: Move,Own |
| `CommentsArg` | src/cli.rs:147 | pub | `--comments` of `read`. | variants: Inline,None |
| `EditTarget` | src/cli.rs:201 | pub | What `edit` and `add` write: one document, or the bundle directory. | variants: Single,Bundle |
| `ExistingArg` | src/cli.rs:236 | pub | `--existing-revisions` of `edit` and `add`. | variants: Auto,Keep,Accept,Reject,Refuse |
| `Command` | src/cli.rs:644 | pub | The subcommands; none given is the two-file compare. | variants: Compare,Changes,Accept,Reject,Convert,Diff,Read,Edit,Add,Capabilities,SelfUpdate,Debug,VisualDiff,Comments,Append,Validate,Scrub |
| `DebugCommand` | src/cli.rs:1313 | pub | Package-level diagnostic comparison tasks. | variants: Diff |
| `DebugCheck` | src/cli.rs:1356 | pub | `jubarte debug --check`. | variants: Orphans,Fields,Bookmarks,Package,Structure,Ids,Styles,Chains,Elements,Textbox,Text,Xml,Runs,Changes,Styledefs,Numbering,Render |
| `PatchFormat` | src/cli.rs:1441 | pub | `jubarte diff --format`: what goes to stdout. | variants: Patch,Critic,Github,Word,Normal,Context,SideBySide |
| `CompareMode` | src/cli.rs:1470 | pub | `jubarte --mode` (compare). | variants: Word,Powertools |
| `Format` | src/cli.rs:1482 | pub | A document format for `-f/--from` and `-t/--to`. | variants: Docx,Md,Pdf,Png,Json |
| `Task` | src/cli.rs:1556 | pub | A task that writes a file through `-o`. | variants: Compare,Convert,Read,Diff,Accept,Reject,Append,Scrub |
| `TrackChanges` | src/cli.rs:2076 | pub | `--track-changes`, pandoc's values. | variants: All,Accept,Reject |
| `Tracking` | src/cli.rs:2256 | pub | `--tracking`. | variants: On,Off |
| `Page` | src/cli.rs:2266 | pub | `--page`. | variants: Letter,A4 |
| `Revisions` | src/cli.rs:2285 | pub | `jubarte convert --revisions`. | variants: Conventional,Word,Custom |
| `KindArg` | src/cli.rs:2312 | pub | `--kind` values. | variants: Insertion,Deletion,Move,Formatting |
| `SectionBreakArg` | src/cli.rs:2338 | pub | `jubarte append --section-break`. | variants: NextPage,Continuous,None |
| `CommentError` | src/comments.rs:83 | pub | Why comments could not be listed. | variants: Admission,Package |
| `ComparisonUnit` | src/comparer/atoms.rs:345 | pub | A comparison unit — a word or a group (atoms live inside words). | variants: Word,Group |
| `Kind` | src/comparer/comments.rs:227 | private | _no doc comment_ | variants: Start,End,Point |
| `Kind` | src/comparer/cross_para.rs:44 | private | _no doc comment_ | variants: Word,Sep |
| `Item` | src/comparer/cross_para.rs:65 | private | _no doc comment_ | variants: Eq,Ins,Del |
| `Block` | src/comparer/cross_para.rs:88 | private | The paragraph-level alignment, in the order the marks close. | variants: Pair,Del,Ins,Barrier |
| `Mark` | src/comparer/cross_para.rs:1311 | private | ─── the segmenter ────────────────────────────────────────────────────────── _(from a code comment)_ | variants: Equal,Inserted,Deleted |
| `MergedItem` | src/comparer/cross_para.rs:1491 | private | _no doc comment_ | variants: Paired,Ins,Del,BoundaryEqual,BoundaryIns,BoundaryDel |
| `Anchor` | src/comparer/cross_para.rs:1738 | private | Anchor chain: matched tokens and paired boundaries, in stream order. _(from a code comment)_ | variants: Tok,Bound |
| `Op` | src/comparer/cross_para.rs:1960 | private | ─── the per-cell token diff (IrTokenDiffer) ──────────────────────────────── _(from a code comment)_ | variants: Equal,Delete,Insert |
| `FlattenSide` | src/comparer/finalize.rs:10960 | pub | Which input [`flatten_tracked_deletions`] is running against — the two sides carry DIFFERENT contracts: - `Original` (`body1`): `accept(redline) ≡ B`. | variants: Original,Revised |
| `RectifyError` | src/comparer/footnotes.rs:581 | pub | Errors raised by [`rectify_footnote_endnote_ids`]. | variants: MissingNoteDef,MissingTargetPart,MissingSourcePart,UnsupportedReferenceStatus |
| `WordRowOp` | src/comparer/lcs_table.rs:134 | pub(crate) | One row operation from Word's shared-word alignment. | variants: Pair,Delete,Insert |
| `WordTableMeshContext` | src/comparer/lcs_table.rs:373 | pub(crate) | _no doc comment_ | variants: M42,M333,M337,M348,M350 |
| `CorrelationStatus` | src/comparer/mod.rs:1260 | pub | Port of `CorrelationStatus`. | variants: Nil,Normal,Unknown,Inserted,Deleted,Equal,Group,MovedSource,MovedDestination,FormatChanged |
| `ComparisonUnitGroupType` | src/comparer/mod.rs:1291 | pub | Which structural container a `super::atoms::ComparisonUnitGroup` came from. | variants: Paragraph,Table,Row,Cell,Textbox |
| `WmlComparerRevisionType` | src/comparer/mod.rs:1308 | pub | Revisions surfaced by `super::revisions`. | variants: Inserted,Deleted,Moved,FormatChanged |
| `CompareMode` | src/comparer/mod.rs:1332 | pub | The two supported comparer configurations. | variants: Word,PowerTools |
| `RunFrag` | src/comparer/preprocess.rs:725 | private | Fragment after r-expansion: text (mergeable) or an empty leaf element. | variants: Text,Leaf |
| `WordClass` | src/comparer/units.rs:90 | private | The character classes Word Compare builds its words from: a run of one class is one word, and a word ends where the class changes. | variants: Letter,Sign,Script |
| `ComparisonLogCode` | src/comparison_log.rs:16 | pub | Severity / category tag of a [`ComparisonLogEntry`]. | variants: Info,Warning,Error |
| `FontStep` | src/convert/font.rs:276 | pub | How the conversion resolved the physical face for a run (plan Step 2f). | variants: Embedded,Explicit,AltName,Theme,WordSubstitution,OpenFallback,Generic,Unknown |
| `FaceId` | src/convert/font.rs:479 | pub(crate) | A bundled face used by the DOCX→PDF writer. | variants: CarlitoRegular,CarlitoBold,CarlitoItalic,CarlitoBoldItalic,SansRegular,SansBold,SansItalic,SansBoldItalic,SerifRegular,SerifBold,SerifItalic,SerifBoldItalic,MonoRegular,MonoBold,MonoItalic,MonoBoldItalic,AptosRegular,AptosBold,AptosItalic,AptosBoldItalic,AptosDisplayRegular,AptosDisplayBol |
| `FaceRef` | src/convert/font.rs:542 | pub(crate) | Catalogue slot or a per-document embedded face (plan xml 3.1 ckpt 4). | variants: Catalogue,Embedded |
| `FontFamilyClass` | src/convert/font_table.rs:17 | pub(crate) | ECMA-376 `w:family` on a font table entry. | variants: Roman,Swiss,Modern,Script,Decorative,Auto |
| `Stretch` | src/convert/math_font.rs:66 | pub(crate) | A stretchy glyph, either one variant or a stacked assembly. | variants: Single,Assembly |
| `GdiObj` | src/convert/metafile.rs:378 | private | _no doc comment_ | variants: Empty,Brush,NullBrush,Font,Pen,Other |
| `ConvertError` | src/convert/mod.rs:73 | pub | Opening a DOCX, emitting a PDF, or selecting pages failed. | variants: OpenPackage,MissingDocument,Emit,Raster,PageOutOfRange,PageNotKept |
| `MarkLines` | src/convert/mod.rs:130 | pub | How many strike or underline lines a revision mark draws under changed text. | variants: None,Single,Double |
| `RevisionStyle` | src/convert/mod.rs:252 | pub | How tracked changes (`w:ins`, `w:del`, `w:moveFrom`, `w:moveTo`) are painted. | variants: Conventional,Word,Custom |
| `CommentPlacement` | src/convert/mod.rs:395 | pub | Where a converted PDF puts the document's comments. | variants: Margin,End |
| `Align` | src/convert/mod.rs:980 | private | _no doc comment_ | variants: Left,Center,Right,Justify |
| `FontHint` | src/convert/mod.rs:989 | private | _no doc comment_ | variants: Default,EastAsia,Complex |
| `VertAlign` | src/convert/mod.rs:997 | private | _no doc comment_ | variants: Baseline,Super,Sub,StackNum,StackDen |
| `TabAlign` | src/convert/mod.rs:1287 | private | _no doc comment_ | variants: Left,Right,Center,Decimal,Bar |
| `TabLeader` | src/convert/mod.rs:1296 | private | _no doc comment_ | variants: None,Dot,Hyphen,Underscore,Heavy |
| `BorderLine` | src/convert/mod.rs:1367 | private | The `w:val` line family of a page border edge. | variants: Single,Double,Dotted,Dashed,MediumGap |
| `BorderDisplay` | src/convert/mod.rs:1387 | private | `w:pgBorders/@w:display` (ST_PageBorderDisplay). | variants: AllPages,FirstPage,NotFirstPage |
| `PageNumFmt` | src/convert/mod.rs:1408 | private | _no doc comment_ | variants: Decimal,LowerRoman,UpperRoman,LowerLetter,UpperLetter,DecimalZero,CardinalText,Ordinal,OrdinalText,Hex,Chicago,IdeographDigital,IdeographTraditional,IdeographZodiac,IdeographZodiacTraditional,IdeographLegalTraditional,IdeographEnclosedCircle,JapaneseCounting,Aiueo,Iroha,DecimalFullWidth,Ga |
| `FieldKind` | src/convert/mod.rs:1815 | private | _no doc comment_ | variants: None,Page,NumPages,NumWords |
| `Block` | src/convert/mod.rs:1976 | private | _no doc comment_ | variants: Paragraph,Table,PageBreak,ColumnBreak,SectionCols |
| `TblWidth` | src/convert/mod.rs:2109 | private | Preferred table width from `tblW`. | variants: Grid,Dxa,Pct |
| `PrefWidth` | src/convert/mod.rs:2117 | private | First-row `w:tcW`. | variants: Dxa,Pct,Auto |
| `CharacterSpacing` | src/convert/mod.rs:2163 | private | ECMA-376 17.15.1.18 / ST_CharacterSpacing. | variants: DoNotCompress,CompressPunctuation,CompressPunctuationAndKana |
| `RowSrc` | src/convert/mod.rs:2303 | private | A table row as laid out: the parsed row, or one part of it after a split. | variants: Orig,Owned,Head |
| `VMerge` | src/convert/mod.rs:2319 | private | _no doc comment_ | variants: None,Restart,Continue |
| `TextAnchor` | src/convert/mod.rs:2560 | private | _no doc comment_ | variants: Top,Center,Bottom |
| `ChartKind` | src/convert/mod.rs:2580 | private | _no doc comment_ | variants: Bar,Pie,Line,Area,Scatter,Radar |
| `ImageSlot` | src/convert/mod.rs:2612 | private | Inline / wrapTopAndBottom images consume flow; wrapSquare anchors overlay. | variants: Flow,Float |
| `RelFrame` | src/convert/mod.rs:2687 | private | An anchor's `relativeFrom` frame (ST_RelFromH / ST_RelFromV), one byte so `ImageSlot` stays small inside `Block`. | variants: Page,Margin,Column,Character,LeftMargin,RightMargin,InsideMargin,OutsideMargin,Paragraph,Line,TopMargin,BottomMargin |
| `WrapMode` | src/convert/mod.rs:2778 | private | _no doc comment_ | variants: None,Square,TopBottom |
| `ShapeGeom` | src/convert/mod.rs:2933 | private | _no doc comment_ | variants: Box,RightArrow,BentConnector,CurvedConnector,Line,RoundRect,Ellipse,Triangle,Diamond,Hexagon,Parallelogram,Trapezoid,Chevron,Plus,HomePlate,Pentagon,Octagon,Star4,Star5,RtTriangle,UpDownArrow,Heart,Donut,Frame,FlowChartTerminator,Heptagon,Star6,Cube,FoldedCorner,Can,Cloud,Pie,LeftRightArro |
| `ImageKind` | src/convert/mod.rs:3034 | private | _no doc comment_ | variants: Jpeg,Rgb,Reserve,Broken,VmlLine,VmlRect,TextBox,Metafile |
| `NumFmt` | src/convert/mod.rs:5399 | private | _no doc comment_ | variants: Decimal,DecimalZero,CardinalText,Ordinal,OrdinalText,Hex,Chicago,IdeographDigital,IdeographTraditional,IdeographZodiac,IdeographZodiacTraditional,IdeographLegalTraditional,IdeographEnclosedCircle,ChineseCounting,ChineseCountingThousand,TaiwaneseCountingThousand,JapaneseCounting,Aiueo,Iroha |
| `RevMark` | src/convert/mod.rs:15079 | private | _no doc comment_ | variants: None,Ins,Del,MoveTo,MoveFrom |
| `MathFace` | src/convert/mod.rs:15772 | private | Which mathematical alphanumerics this conversion's Cambria Math face draws. | variants: All,Only |
| `BreakText` | src/convert/mod.rs:16076 | private | What a run's page or column break becomes in its collected text. | variants: Drop,Mark,Line |
| `SizeFrom` | src/convert/mod.rs:19082 | private | The frame a `wp14:sizeRelH/V` percentage is taken of. | variants: Page,Margin,LeftMargin,RightMargin,TopMargin,BottomMargin |
| `B` | src/convert/mod.rs:32702 | private | _no doc comment_ | variants: R,L,En,An,N |
| `Op` | src/convert/pdf.rs:20 | pub(crate) | One drawing command on a page (PDF user space, origin bottom-left). | variants: Text,Line,FillRect,StrokeRect,FillPoly,StrokePoly,FillPath,StrokePath,Cubic,Jpeg,Rgb,Pin,Watermark |
| `Lit` | src/convert/pdf.rs:1373 | private | A text-showing operand: glyph ids as a hex string under Identity-H, or WinAnsi bytes as a literal string. | variants: Glyphs,Bytes |
| `Fill` | src/convert/preset_geom.rs:22 | pub(crate) | `a:path/@fill`: how the path's fill is shaded from the shape fill. | variants: None,Norm,Lighten,LightenLess,Darken,DarkenLess |
| `Cmd` | src/convert/preset_geom.rs:49 | pub(crate) | One path command; operands are guide names or numeric literals. | variants: M,L,A,C,Q,Z |
| `CmdRef` | src/convert/preset_geom.rs:77 | private | A path command borrowed from a preset or from a document's `a:custGeom`, so one evaluator serves both. | variants: M,L,A,C,Q,Z |
| `OwnedCmd` | src/convert/preset_geom.rs:100 | pub(crate) | One `a:custGeom` path command; operands are guide names or numbers. | variants: M,L,A,C,Q,Z |
| `Check` | src/debug.rs:58 | pub | Which reports to print. | variants: Orphans,Fields,Bookmarks,Package,Structure,Ids,Styles,Chains,Elements,Textbox,Text,Xml,Runs,Changes,StyleDefs,Numbering,Render |
| `Tok` | src/debug.rs:1640 | private | _no doc comment_ | variants: Open,Close,Text |
| `ExistingRevisions` | src/edit.rs:150 | pub | Policy for a source that already contains tracked changes. | variants: Refuse,Accept,Reject,Keep |
| `OperationKind` | src/edit.rs:177 | pub | The supported operation kinds (wire tag `kind`). | variants: Replace,Insert,Delete,Comment,InsertParagraph,DeleteParagraph,FormatParagraph,MergeParagraphs,Rewrite,ReplyComment,ResolveComment,EditComment,DeleteComment,InsertTable,List,Watermark,FillControl,FormatRun,InsertFootnote,InsertImage,PageSetup,InsertToc,Redact,Settings |
| `ListKind` | src/edit.rs:631 | pub | The numbering of a [`OperationKind::List`]. | variants: Bullet,Decimal,LowerLetter |
| `Alignment` | src/edit.rs:717 | pub | Paragraph alignment for [`OperationKind::FormatParagraph`]. | variants: Left,Center,Right,Justify |
| `Edge` | src/edit.rs:800 | pub | Paragraph edge for an insertion without a text anchor. | variants: Start,End |
| `Side` | src/edit.rs:812 | pub | Side of the anchor paragraph for [`OperationKind::InsertParagraph`]. | variants: Before,After |
| `Selector` | src/edit.rs:834 | pub | Paragraph selector; every form must match exactly one paragraph. | variants: Name,Id,Index,StartsWith,Contains |
| `Resolved` | src/edit.rs:1774 | private | A resolved operation, in source coordinates (bytes of the projection). | variants: Text,CommentRange,DeleteParagraph,FormatParagraph,MergeParagraphs,InsertParagraph,InsertTable,List,CommentSpan,Thread,FillControl,FormatRun,InsertFootnote,InsertImage,PageSetup |
| `ThreadOp` | src/edit.rs:1897 | private | What a thread operation does to an existing comment. | variants: Reply,Resolve,Edit,Delete |
| `ControlSelector` | src/edit/controls.rs:27 | pub | Which content control to fill; every form must match exactly one. | variants: Name,Id,Tag,Alias |
| `FillValue` | src/edit/controls.rs:50 | pub(super) | What a resolved fill writes. | variants: Text,Checked,Date |
| `Verb` | src/edit/flags.rs:52 | pub | Which command the flags came with: `edit` changes what is there, `add` makes something new. | variants: Edit,Add |
| `Fate` | src/edit/origin.rs:221 | private | Where a source paragraph went when the plan's revisions were settled. | variants: Kept,Merged,Gone |
| `RecordError` | src/edit/record.rs:61 | pub | Why a recording failed. | variants: Usage,Plan |
| `SectionScope` | src/edit/sections.rs:29 | pub | Which sections a [`super::OperationKind::PageSetup`] changes. | variants: Last,All |
| `PageSize` | src/edit/sections.rs:40 | pub | A paper size, by name or in twentieths of a point. | variants: Named,Custom |
| `Paper` | src/edit/sections.rs:50 | pub | Named paper sizes. | variants: Letter,A4 |
| `Orientation` | src/edit/sections.rs:72 | pub | Page orientation. | variants: Portrait,Landscape |
| `Track` | src/edit/v2.rs:97 | pub | Tracking policy for newly requested edits. | variants: On,Off |
| `TextView` | src/edit/v2.rs:107 | pub | The initial profile supports the final logical working projection only. | variants: Final |
| `Scope` | src/edit/v2.rs:141 | pub | Direct body/review-operation authority, frozen on the original source. | variants: All,Selected |
| `Operation` | src/edit/v2.rs:167 | pub | The eleven typed requests in the initial PlanV2 profile. | variants: Replace,Insert,Delete,Rewrite,InsertParagraph,DeleteParagraph,Comment,ReplyComment,ResolveComment,EditComment,DeleteComment |
| `ContextKind` | src/edit/v2.rs:347 | pub | Kind of immutable original-source observation. | variants: Paragraph,Comment,Opaque |
| `Kind` | src/edit/whole.rs:218 | private | _no doc comment_ | variants: Kept,Inserted,Deleted,Marker,Other |
| `FieldError` | src/fields.rs:75 | pub | Why [`update_fields`] failed. | variants: Inspect,Layout,Package |
| `NumberFormat` | src/fields.rs:533 | private | _no doc comment_ | variants: Arabic,Roman,Alpha |
| `At` | src/fields.rs:1235 | private | Where each entry's runs go: after the separate mark, appended to a new paragraph, or (the last entry of a TOC that already ends in its own paragraph) before the end mark. _(from a code comment)_ | variants: After,Append,Before |
| `Location` | src/ids.rs:196 | pub | What a `-p` value names. | variants: Paragraph,Comment,CommentText,Changes |
| `RawCommentId` | src/ids.rs:311 | pub(crate) | The same wire shape feeds deserialization and legacy field discovery. _(from a code comment)_ | variants: Number,Text |
| `Story` | src/ids/short.rs:47 | pub | The story an id lives in: the body, or a part named by its file stem (`header1`, `footer2`, `footnotes`, `endnotes`, `comments`, `styles`). | variants: Body,Part |
| `Step` | src/ids/short.rs:56 | pub | One step of a block's path inside its story. | variants: Paragraph,Table,Row,Cell,TextBox,ContentControl |
| `At` | src/ids/short.rs:227 | private | _no doc comment_ | variants: Story,Paragraph,Table,Row,Cell,TextBox,Leaf |
| `ShortId` | src/ids/short.rs:258 | pub | A short id (see the module doc). | variants: Block,Range,Change,Comment,CommentText,Section,Toc |
| `InspectError` | src/inspect.rs:244 | pub | An invalid package or XML part. | variants: Package,MissingDocument,Invalid,Admission |
| `Piece` | src/inspect.rs:985 | pub(crate) | What one visible piece of paragraph text maps back to. | variants: Text,Glyph |
| `Block` | src/legacy_doc.rs:95 | pub | One block of the main story, in document order. | variants: Paragraph,Table |
| `Fkp` | src/legacy_doc.rs:1284 | private | _no doc comment_ | variants: Paragraph,Character |
| `Keep` | src/markdown/anchor.rs:73 | private | _no doc comment_ | variants: Inner,Nothing,NewSide |
| `Token` | src/markdown/critic.rs:23 | pub(crate) | A CriticMarkup delimiter. | variants: InsertStart,InsertEnd,DeleteStart,DeleteEnd,SubstituteStart,SubstituteSeparator,SubstituteEnd,HighlightStart,HighlightEnd,CommentStart,CommentEnd,CommentBreak,MarkStart,MarkEnd,FieldStart,FieldSeparator,FieldEnd |
| `Piece` | src/markdown/critic.rs:1696 | pub(crate) | Text or a delimiter, in source order. | variants: Text,Token |
| `In` | src/markdown/critic.rs:1764 | private | _no doc comment_ | variants: Text,Inserted,Deleted,Old,New,Comment |
| `Side` | src/markdown/diff.rs:448 | private | _no doc comment_ | variants: Delete,Insert |
| `Entry` | src/markdown/diff.rs:463 | private | One line of a changed hunk: paired, or only in one version. | variants: Pair,Only |
| `Mark` | src/markdown/from_docx/critic.rs:35 | pub(crate) | A CriticMarkup span kind. | variants: Insertion,Deletion,Scope |
| `Leaf` | src/markdown/from_docx/critic.rs:221 | private | _no doc comment_ | variants: Text,Raw,Comment |
| `Token` | src/markdown/from_docx/critic.rs:257 | private | _no doc comment_ | variants: Leaf,Open,Close |
| `Node` | src/markdown/from_docx/critic.rs:264 | private | _no doc comment_ | variants: Leaf,Span,Split |
| `Owner` | src/markdown/from_docx/header.rs:13 | pub(crate) | Where the owner's name came from. | variants: Creator,LastModifiedBy,Unnamed,None |
| `PagesSource` | src/markdown/from_docx/header.rs:24 | pub(crate) | Where the page count and markers came from. | variants: Layout,Cached,Estimated |
| `StoryView` | src/markdown/from_docx/header.rs:1528 | pub(crate) | How [`story_text`] reads a header or footer's tracked changes. | variants: Tagged,Accept,Reject |
| `Revisions` | src/markdown/from_docx/mod.rs:32 | pub(crate) | How tracked changes and comments come out. | variants: Markup,Accept,Reject |
| `ConvertError` | src/markdown/from_docx/mod.rs:65 | pub(crate) | _no doc comment_ | variants: Invalid |
| `Node` | src/markdown/from_docx/ooxml.rs:267 | pub(crate) | _no doc comment_ | variants: Element,Text |
| `Style` | src/markdown/math.rs:541 | private | How a run of math text is set (`m:rPr`). | variants: Math,Upright,Bold,BoldItalic,Text,Script |
| `Node` | src/markdown/math.rs:557 | private | One piece of an equation. | variants: Text,Group,Fraction,Radical,Scripts,Pre,Delimiter,Function,Nary,Accent,Bar,Limit,Brace,Matrix,Array,Boxed |
| `Token` | src/markdown/math.rs:627 | private | A LaTeX token. | variants: Char,Command,Open,Close,Sup,Sub,Amp,Change |
| `Frame` | src/markdown/math.rs:699 | private | _no doc comment_ | variants: Bracket,Wall |
| `TrackChanges` | src/markdown/mod.rs:74 | pub | What happens to the tracked changes a Markdown / `.docx` document describes. | variants: All,Accept,Reject |
| `PageSize` | src/markdown/mod.rs:106 | pub | The page a Markdown document is written on when no reference document gives one. | variants: Letter,A4 |
| `Select` | src/markdown/mod.rs:359 | pub | Which blocks of the agent view to print. | variants: Head,Tail,Picks,Changed |
| `Pick` | src/markdown/mod.rs:378 | pub | One item of a `-p` selection: a contiguous paragraph range (`pN`, `pN-pM`, `pN-`, `-pM`) or a whole table (`tN`). | variants: Paragraphs,Table |
| `MarkdownError` | src/markdown/mod.rs:980 | pub | Why a Markdown / `.docx` document could not be read or written. | variants: Reference,Package,Docx |
| `Kind` | src/markdown/patch.rs:63 | private | The kind of a block or paragraph, for pairing and for the properties a new paragraph copies. | variants: Heading,List,Cell,Plain |
| `Edit` | src/markdown/patch.rs:274 | private | One step of the alignment of paragraphs (by position in `paragraphs`) and blocks (by position in `blocks`). | variants: Pair,Insert,Delete |
| `Source` | src/markdown/redline.rs:31 | pub | A document side of a comparison: either a `.docx` package or a Markdown text. | variants: Docx,Markdown |
| `Locator` | src/markdown/unified.rs:69 | pub | Where a hunk's paragraph is. | variants: Line,Paragraph |
| `Side` | src/markdown/unified.rs:577 | private | _no doc comment_ | variants: Old,New |
| `Segment` | src/markdown/unified.rs:610 | private | _no doc comment_ | variants: Same,Edit,HighlightStart,HighlightEnd,Note |
| `Owner` | src/markdown/view_notes.rs:217 | pub(crate) | Who owns the document, as `authors: document_owner:` and its note say. | variants: Creator,LastModifiedBy,Unnamed,NoPart |
| `CellsAt` | src/markdown/view_notes.rs:553 | pub(super) | The cells a `cell-borders` group names: every cell, a row's, or one. | variants: All,Row,Cell |
| `CellMark` | src/markdown/view_notes.rs:636 | pub(crate) | A cell's own change: `w:cellIns`, `w:cellDel` or `w:cellMerge`. | variants: Insert,Delete,Merge |
| `Value` | src/markdown/view_stories.rs:232 | private | A value of a YAML flow map as the view prints it. | variants: Flag,Text,List,Map |
| `Change` | src/markdown/view_stories.rs:690 | private | One tracked change at the start of a story's text, and what follows it. | variants: Insert,Delete,Substitute |
| `Field` | src/markdown/view_stories.rs:726 | private | A field at the start of a story's text, and what follows it. | variants: Coded,Bare |
| `CellShape` | src/markdown/write.rs:731 | private | What one Markdown cell of a planned row is. | variants: Cell,Spanned |
| `RowCells` | src/markdown/write.rs:1485 | private | The cells of a table row so far: all empty, all one change whole of one kind, or anything else. | variants: Empty,Whole,Mixed |
| `Kind` | src/markdown/xml.rs:13 | pub(super) | An inserted or a deleted tracked change. | variants: Insert,Delete |
| `Content` | src/markdown/xml.rs:42 | pub(super) | What a run holds. | variants: Text,Break,PageBreak,Tab,NoteReference,Picture,Drawing,Math,FieldBegin,FieldEnd |
| `Link` | src/markdown/xml.rs:182 | pub(super) | A hyperlink's target. | variants: External,Anchor |
| `Item` | src/markdown/xml.rs:189 | pub(super) | One element of a story, in document order. | variants: Paragraph,ParagraphEnd,Run,RangeStart,RangeEnd,CommentReference,LinkStart,Bookmark,LinkEnd,Table,Row,Cell,CellEnd,RowEnd,TableEnd |
| `Label` | src/markdown/xml.rs:418 | pub(super) | A list label an id line names, written as it reads. | variants: Bullet,Text |
| `ListFormat` | src/markdown/xml.rs:2098 | pub(crate) | The format of every level of a list definition. | variants: Bullet,Decimal,LowerLetter |
| `ZipEntry` | src/opc/mod.rs:360 | private | One zip entry `PartFs::to_zip` writes after the two package-level ones. | variants: Rels,Part |
| `Stage` | src/perf.rs:17 | pub | Coarse pipeline stages recorded by [`record_stage_ns`]. | variants: PackageOpen,Preprocess,Atomize,Unitize,Lcs,Produce,Serialize,Zip |
| `Status` | src/result.rs:47 | pub | The terminal status of a v2 result. | variants: Ok,Refused,Error |
| `Phase` | src/result.rs:62 | pub | The stage a result reports. | variants: Inspected,Generated,Validated,Committed |
| `Effect` | src/result.rs:80 | pub | The primary document effect a result reports. | variants: Changed,Unchanged,NotApplied |
| `PredictedEffect` | src/result.rs:93 | pub | The predicted primary effect of a dry run. | variants: Changed,Unchanged |
| `CommitState` | src/result.rs:134 | pub | Publication state known when an error was reported. | variants: Committed,NotCommitted,Unknown |
| `MediaRole` | src/result.rs:146 | pub | The role of an artifact: which contract slot it fills. | variants: Document,Plan,Patch,Report,Render,Other |
| `ResultDataError` | src/result.rs:606 | pub | Invalid combinations of (phase, effect, predicted_effect). | variants: MissingPhase,FailureRequired,NotAppliedRequired,EffectRequired,PredictedWithoutValidation |
| `StreamError` | src/result.rs:999 | pub | Why a JSONL stream is malformed. | variants: MissingHeader,HeaderParse,InvalidLine,MissingTerminal,MultipleTerminals,TerminalParse |
| `TagType` | src/revision_processor.rs:745 | pub | `TagTypeEnum` (RevisionProcessor.cs :2389): how an element appears in the doc-order tag stream. | variants: Element,EmptyElement,EndElement |
| `TableMerge` | src/revision_processor.rs:2838 | private | Which runs of adjacent tables A.8 merges. | variants: Revised,WordSave |
| `Resolution` | src/revision_processor/sections.rs:23 | pub(crate) | Accept All or Reject All. | variants: Accept,Reject |
| `ScrubError` | src/scrub.rs:81 | pub | Why [`scrub`] refused its input. | variants: Package,Invalid |
| `ProtectionEdit` | src/settings.rs:147 | pub | What `w:documentProtection` allows (`ST_DocProtect`). | variants: None,ReadOnly,Comments,TrackedChanges,Forms |
| `UriSite` | src/strict_translation.rs:520 | private | Where in a part a Strict URI may be rewritten. | variants: NamespaceDeclaration,RelationshipType |
| `TextFormat` | src/text_diff.rs:43 | pub | Presentation of a document's line changes. | variants: Github,Word,Normal,Context,SideBySide |
| `Row` | src/text_diff.rs:235 | private | _no doc comment_ | variants: Pair,Delete,Insert |
| `Piece` | src/text_diff.rs:464 | private | One stretch of a word-level change line. | variants: Same,Delete,Insert,Replace |
| `Plan` | src/update.rs:67 | pub | The decision [`plan`] returns, once the target release is known. | variants: UpToDate,Install |
| `Kind` | src/util/words.rs:38 | private | What the token being built is made of. | variants: Word,Space,Orphan,Other |
| `ValidateError` | src/validate.rs:210 | pub | Why a package could not be validated at all (a finding is never an error). | variants: Refused,Package,Changes,Inspect,Usage,Layout |
| `NodeKind` | src/xmllinq/mod.rs:328 | private | Node kind discriminant + kind-specific data. | variants: Element,Text,Comment,Pi,Document |
