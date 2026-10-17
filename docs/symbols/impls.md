# Impl blocks

334 items (non-test). Part of the [symbol manifest](README.md).

| Name | Defined at | Vis | Does | Detail |
|---|---|---|---|---|
| `<InputLimits as Default>` | src/admission.rs:55 | private | _no doc comment_ | impl Default for InputLimits |
| `InputLimits` | src/admission.rs:68 | private | _no doc comment_ | impl InputLimits |
| `InputLimitOverrides` | src/admission.rs:106 | private | _no doc comment_ | impl InputLimitOverrides |
| `AdmissionErrorKind` | src/admission.rs:153 | private | _no doc comment_ | impl AdmissionErrorKind |
| `AdmissionError` | src/admission.rs:254 | private | _no doc comment_ | impl AdmissionError |
| `<AdmissionError as fmt::Display>` | src/admission.rs:269 | private | _no doc comment_ | impl fmt::Display for AdmissionError |
| `<AdmissionError as std::error::Error>` | src/admission.rs:275 | private | _no doc comment_ | impl std::error::Error for AdmissionError |
| `AppendError` | src/append.rs:112 | private | _no doc comment_ | impl AppendError |
| `<AppendError as fmt::Display>` | src/append.rs:124 | private | _no doc comment_ | impl fmt::Display for AppendError |
| `<AppendError as std::error::Error>` | src/append.rs:135 | private | _no doc comment_ | impl std::error::Error for AppendError |
| `ListCopy` | src/append.rs:1108 | private | _no doc comment_ | impl ListCopy<'_> |
| `<AuditError as fmt::Display>` | src/audit.rs:94 | private | _no doc comment_ | impl fmt::Display for AuditError |
| `<AuditError as std::error::Error>` | src/audit.rs:112 | private | _no doc comment_ | impl std::error::Error for AuditError |
| `<AuditError as From>` | src/audit.rs:114 | private | _no doc comment_ | impl From<InspectError> for AuditError |
| `Story` | src/audit.rs:314 | private | _no doc comment_ | impl Story |
| `Package` | src/audit.rs:353 | private | _no doc comment_ | impl Package |
| `Field` | src/audit.rs:835 | private | _no doc comment_ | impl Field |
| `<ScrubSelection as ScrubOptions>` | src/bin/jubarte.rs:66 | private | _no doc comment_ | impl ScrubOptions for ScrubSelection |
| `<Selection as SelectionFilter>` | src/bin/jubarte.rs:84 | private | _no doc comment_ | impl SelectionFilter for Selection |
| `Output` | src/bin/jubarte.rs:222 | private | _no doc comment_ | impl<'a> Output<'a> |
| `<ConvertFailure as From>` | src/bin/jubarte.rs:482 | private | _no doc comment_ | impl From<String> for ConvertFailure |
| `<ConvertFailure as From>` | src/bin/jubarte.rs:488 | private | _no doc comment_ | impl From<&str> for ConvertFailure |
| `Input` | src/bin/jubarte.rs:2212 | private | _no doc comment_ | impl Input |
| `Operations` | src/capabilities.rs:197 | private | _no doc comment_ | impl Operations |
| `<InputBudget as From>` | src/capabilities.rs:308 | private | _no doc comment_ | impl From<crate::admission::InputLimits> for InputBudget |
| `ChangeFilter` | src/changes.rs:114 | private | _no doc comment_ | impl ChangeFilter |
| `<ChangeError as fmt::Display>` | src/changes.rs:209 | private | _no doc comment_ | impl fmt::Display for ChangeError |
| `<ChangeError as std::error::Error>` | src/changes.rs:244 | private | _no doc comment_ | impl std::error::Error for ChangeError |
| `EditOptions` | src/cli.rs:208 | private | _no doc comment_ | impl EditOptions |
| `ExistingArg` | src/cli.rs:250 | private | _no doc comment_ | impl ExistingArg |
| `<Cli as std::ops::Deref>` | src/cli.rs:634 | private | _no doc comment_ | impl std::ops::Deref for Cli |
| `<crate::debug::Check as From>` | src/cli.rs:1413 | private | _no doc comment_ | impl From<DebugCheck> for crate::debug::Check |
| `PatchFormat` | src/cli.rs:1460 | private | _no doc comment_ | impl PatchFormat |
| `Format` | src/cli.rs:1511 | private | _no doc comment_ | impl Format |
| `Task` | src/cli.rs:1575 | private | _no doc comment_ | impl Task |
| `<crate::markdown::TrackChanges as From>` | src/cli.rs:2085 | private | _no doc comment_ | impl From<TrackChanges> for crate::markdown::TrackChanges |
| `<RenderArgs as Default>` | src/cli.rs:2139 | private | _no doc comment_ | impl Default for RenderArgs |
| `RenderArgs` | src/cli.rs:2153 | private | _no doc comment_ | impl RenderArgs |
| `<crate::markdown::PageSize as From>` | src/cli.rs:2273 | private | _no doc comment_ | impl From<Page> for crate::markdown::PageSize |
| `<crate::append::SectionBreak as From>` | src/cli.rs:2348 | private | _no doc comment_ | impl From<SectionBreakArg> for crate::append::SectionBreak |
| `Cli` | src/cli.rs:2911 | private | _no doc comment_ | impl Cli |
| `CommentError` | src/comments.rs:90 | private | _no doc comment_ | impl CommentError |
| `<CommentError as fmt::Display>` | src/comments.rs:101 | private | _no doc comment_ | impl fmt::Display for CommentError |
| `<CommentError as std::error::Error>` | src/comments.rs:110 | private | _no doc comment_ | impl std::error::Error for CommentError |
| `<CommentError as From>` | src/comments.rs:112 | private | _no doc comment_ | impl From<crate::inspect::InspectError> for CommentError |
| `Anchors` | src/comments.rs:237 | private | _no doc comment_ | impl Anchors |
| `CommentFamily` | src/comments.rs:427 | private | _no doc comment_ | impl CommentFamily |
| `AtomHash` | src/comparer/atoms.rs:45 | private | _no doc comment_ | impl AtomHash |
| `<AtomHash as std::fmt::Debug>` | src/comparer/atoms.rs:80 | private | _no doc comment_ | impl std::fmt::Debug for AtomHash |
| `<AtomHash as From>` | src/comparer/atoms.rs:87 | private | _no doc comment_ | impl From<&str> for AtomHash |
| `<AtomHash as From>` | src/comparer/atoms.rs:96 | private | _no doc comment_ | impl From<String> for AtomHash |
| `<AtomHash as From>` | src/comparer/atoms.rs:101 | private | _no doc comment_ | impl From<&String> for AtomHash |
| `ComparisonUnitAtom` | src/comparer/atoms.rs:194 | private | _no doc comment_ | impl ComparisonUnitAtom |
| `Sha1Keyed` | src/comparer/atoms.rs:235 | private | _no doc comment_ | impl Sha1Keyed |
| `ComparisonUnitWord` | src/comparer/atoms.rs:293 | private | _no doc comment_ | impl ComparisonUnitWord |
| `ComparisonUnit` | src/comparer/atoms.rs:353 | private | _no doc comment_ | impl ComparisonUnit |
| `CorrelatedSequence` | src/comparer/atoms.rs:502 | private | _no doc comment_ | impl CorrelatedSequence |
| `Seg` | src/comparer/comments.rs:447 | private | _no doc comment_ | impl Seg |
| `Tok` | src/comparer/cross_para.rs:58 | private | _no doc comment_ | impl Tok |
| `<RectifyError as std::fmt::Display>` | src/comparer/footnotes.rs:610 | private | _no doc comment_ | impl std::fmt::Display for RectifyError |
| `<RectifyError as std::error::Error>` | src/comparer/footnotes.rs:641 | private | _no doc comment_ | impl std::error::Error for RectifyError |
| `<U64IdentityHasher as std::hash::Hasher>` | src/comparer/lcs.rs:38 | private | _no doc comment_ | impl std::hash::Hasher for U64IdentityHasher |
| `WmlComparerSettings` | src/comparer/mod.rs:1421 | private | _no doc comment_ | impl WmlComparerSettings |
| `<WmlComparerSettings as Default>` | src/comparer/mod.rs:1474 | private | _no doc comment_ | impl Default for WmlComparerSettings |
| `CompareContext` | src/comparer/mod.rs:1630 | private | _no doc comment_ | impl CompareContext |
| `ComparisonLog` | src/comparison_log.rs:41 | private | _no doc comment_ | impl ComparisonLog |
| `Fmt` | src/convert/altchunk.rs:201 | private | _no doc comment_ | impl Fmt |
| `Builder` | src/convert/altchunk.rs:294 | private | _no doc comment_ | impl Builder |
| `<DiffOptions as Default>` | src/convert/diff_render.rs:42 | private | _no doc comment_ | impl Default for DiffOptions |
| `PageDiff` | src/convert/diff_render.rs:69 | private | _no doc comment_ | impl PageDiff |
| `RenderDiff` | src/convert/diff_render.rs:98 | private | _no doc comment_ | impl RenderDiff |
| `<RestoreOnDrop as Drop>` | src/convert/font.rs:217 | private | _no doc comment_ | impl<T> Drop for RestoreOnDrop<'_, T> |
| `FontStep` | src/convert/font.rs:296 | private | _no doc comment_ | impl FontStep |
| `<FontStep as fmt::Display>` | src/convert/font.rs:315 | private | _no doc comment_ | impl fmt::Display for FontStep |
| `FontReportEntry` | src/convert/font.rs:342 | private | _no doc comment_ | impl FontReportEntry |
| `<FaceRef as From>` | src/convert/font.rs:547 | private | _no doc comment_ | impl From<FaceId> for FaceRef |
| `<FaceRef as PartialEq>` | src/convert/font.rs:553 | private | _no doc comment_ | impl PartialEq<FaceId> for FaceRef |
| `FaceId` | src/convert/font.rs:559 | private | _no doc comment_ | impl FaceId |
| `FaceId` | src/convert/font.rs:675 | private | _no doc comment_ | impl FaceId |
| `Face` | src/convert/font.rs:941 | private | _no doc comment_ | impl<'a> Face<'a> |
| `Catalogue` | src/convert/font.rs:1431 | private | _no doc comment_ | impl Catalogue |
| `Fonts` | src/convert/font.rs:1461 | private | _no doc comment_ | impl<'a> Fonts<'a> |
| `Fonts` | src/convert/font.rs:1979 | private | _no doc comment_ | impl Fonts<'_> |
| `FaceSource` | src/convert/font.rs:3068 | private | _no doc comment_ | impl FaceSource |
| `IndexEntry` | src/convert/font.rs:3154 | private | _no doc comment_ | impl IndexEntry |
| `FontIndex` | src/convert/font.rs:3182 | private | _no doc comment_ | impl FontIndex |
| `FontTable` | src/convert/font_table.rs:45 | private | _no doc comment_ | impl FontTable |
| `Glyph` | src/convert/math_font.rs:45 | private | _no doc comment_ | impl Glyph |
| `MathFont` | src/convert/math_font.rs:86 | private | _no doc comment_ | impl MathFont |
| `MathFontGuard` | src/convert/math_font.rs:189 | private | _no doc comment_ | impl MathFontGuard |
| `<MathFontGuard as Drop>` | src/convert/math_font.rs:195 | private | _no doc comment_ | impl Drop for MathFontGuard |
| `Canvas` | src/convert/metafile.rs:72 | private | _no doc comment_ | impl Canvas |
| `Map` | src/convert/metafile.rs:305 | private | _no doc comment_ | impl Map |
| `WmfFont` | src/convert/metafile.rs:402 | private | _no doc comment_ | impl WmfFont |
| `Xform` | src/convert/metafile.rs:730 | private | _no doc comment_ | impl Xform |
| `<ConvertError as fmt::Display>` | src/convert/mod.rs:100 | private | _no doc comment_ | impl fmt::Display for ConvertError |
| `<ConvertError as std::error::Error>` | src/convert/mod.rs:125 | private | _no doc comment_ | impl std::error::Error for ConvertError |
| `RevisionPalette` | src/convert/mod.rs:166 | private | _no doc comment_ | impl RevisionPalette |
| `RevisionPalette` | src/convert/mod.rs:196 | private | _no doc comment_ | impl RevisionPalette |
| `RevisionStyle` | src/convert/mod.rs:265 | private | _no doc comment_ | impl RevisionStyle |
| `<Restore as Drop>` | src/convert/mod.rs:310 | private | _no doc comment_ | impl Drop for Restore |
| `RenderReport` | src/convert/mod.rs:495 | private | _no doc comment_ | impl RenderReport |
| `RunStyle` | src/convert/mod.rs:1128 | private | _no doc comment_ | impl RunStyle |
| `ThemeFonts` | src/convert/mod.rs:1509 | private | _no doc comment_ | impl ThemeFonts |
| `Defaults` | src/convert/mod.rs:1684 | private | _no doc comment_ | impl Defaults |
| `TextRun` | src/convert/mod.rs:1928 | private | _no doc comment_ | impl TextRun |
| `TableCell` | src/convert/mod.rs:2247 | private | _no doc comment_ | impl TableCell |
| `RowSrc` | src/convert/mod.rs:2309 | private | _no doc comment_ | impl RowSrc<'_> |
| `RelFrame` | src/convert/mod.rs:2702 | private | _no doc comment_ | impl RelFrame |
| `Numbering` | src/convert/mod.rs:5507 | private | _no doc comment_ | impl Numbering |
| `EndnoteBag` | src/convert/mod.rs:8514 | private | _no doc comment_ | impl EndnoteBag |
| `AuthorColors` | src/convert/mod.rs:15053 | private | _no doc comment_ | impl AuthorColors |
| `MathFace` | src/convert/mod.rs:15778 | private | _no doc comment_ | impl MathFace |
| `MathAlphanumerics` | src/convert/mod.rs:15806 | private | _no doc comment_ | impl MathAlphanumerics |
| `<MathAlphanumerics as Drop>` | src/convert/mod.rs:15818 | private | _no doc comment_ | impl Drop for MathAlphanumerics |
| `Layout` | src/convert/mod.rs:23062 | private | _no doc comment_ | impl<'a> Layout<'a> |
| `Squeeze` | src/convert/mod.rs:32972 | private | _no doc comment_ | impl Squeeze |
| `Page` | src/convert/pdf.rs:201 | private | _no doc comment_ | impl Page |
| `Op` | src/convert/pdf.rs:307 | private | _no doc comment_ | impl Op |
| `FaceObjIds` | src/convert/pdf.rs:1126 | private | _no doc comment_ | impl FaceObjIds |
| `Show` | src/convert/pdf.rs:1378 | private | _no doc comment_ | impl<'a> Show<'a> |
| `<Show as std::fmt::Display>` | src/convert/pdf.rs:1404 | private | _no doc comment_ | impl std::fmt::Display for Show<'_> |
| `Fill` | src/convert/preset_geom.rs:31 | private | _no doc comment_ | impl Fill |
| `Cmd` | src/convert/preset_geom.rs:86 | private | _no doc comment_ | impl Cmd |
| `OwnedCmd` | src/convert/preset_geom.rs:109 | private | _no doc comment_ | impl OwnedCmd |
| `<OutlineSink as ttf_parser::OutlineBuilder>` | src/convert/raster.rs:514 | private | _no doc comment_ | impl ttf_parser::OutlineBuilder for OutlineSink |
| `<Options as Default>` | src/debug.rs:138 | private | _no doc comment_ | impl Default for Options |
| `Package` | src/debug.rs:170 | private | _no doc comment_ | impl Package |
| `Findings` | src/debug.rs:268 | private | _no doc comment_ | impl Findings |
| `<DiffOptions as Default>` | src/debug/diff.rs:31 | private | _no doc comment_ | impl Default for DiffOptions |
| `Filter` | src/debug/diff.rs:138 | private | _no doc comment_ | impl Filter |
| `N` | src/debug/diff.rs:164 | private | _no doc comment_ | impl N |
| `Build` | src/debug/diff.rs:490 | private | _no doc comment_ | impl Build<'_> |
| `Seg` | src/debug/diff.rs:1196 | private | _no doc comment_ | impl Seg |
| `Walker` | src/debug/diff.rs:1250 | private | _no doc comment_ | impl Walker<'_> |
| `StyleBook` | src/debug/render.rs:155 | private | _no doc comment_ | impl StyleBook |
| `Tally` | src/debug/render.rs:479 | private | _no doc comment_ | impl Tally |
| `ResolvedRevisions` | src/edit.rs:140 | private | _no doc comment_ | impl ResolvedRevisions |
| `ListKind` | src/edit.rs:641 | private | _no doc comment_ | impl ListKind |
| `<LineSpacing as Serialize>` | src/edit.rs:665 | private | _no doc comment_ | impl Serialize for LineSpacing |
| `<LineSpacing as Deserialize>` | src/edit.rs:671 | private | _no doc comment_ | impl<'de> Deserialize<'de> for LineSpacing |
| `<Points as Serialize>` | src/edit.rs:688 | private | _no doc comment_ | impl Serialize for Points |
| `<Points as Deserialize>` | src/edit.rs:694 | private | _no doc comment_ | impl<'de> Deserialize<'de> for Points |
| `Alignment` | src/edit.rs:728 | private | _no doc comment_ | impl Alignment |
| `<HalfPoints as Serialize>` | src/edit.rs:779 | private | _no doc comment_ | impl Serialize for HalfPoints |
| `<HalfPoints as Deserialize>` | src/edit.rs:785 | private | _no doc comment_ | impl<'de> Deserialize<'de> for HalfPoints |
| `RunSpec` | src/edit.rs:889 | private | _no doc comment_ | impl RunSpec |
| `RevisionCounts` | src/edit.rs:957 | private | _no doc comment_ | impl RevisionCounts |
| `EditReport` | src/edit.rs:1026 | private | _no doc comment_ | impl EditReport |
| `<EditError as fmt::Display>` | src/edit.rs:1137 | private | _no doc comment_ | impl fmt::Display for EditError |
| `<EditError as std::error::Error>` | src/edit.rs:1146 | private | _no doc comment_ | impl std::error::Error for EditError |
| `EditPlan` | src/edit.rs:1410 | private | _no doc comment_ | impl EditPlan |
| `ThreadOp` | src/edit.rs:1904 | private | _no doc comment_ | impl ThreadOp |
| `Transaction` | src/edit.rs:2003 | private | _no doc comment_ | impl<'p> Transaction<'p> |
| `Transaction` | src/edit/controls.rs:71 | private | _no doc comment_ | impl Transaction<'_> |
| `Transaction` | src/edit/images.rs:139 | private | _no doc comment_ | impl Transaction<'_> |
| `Transaction` | src/edit/notes.rs:21 | private | _no doc comment_ | impl Transaction<'_> |
| `Ids` | src/edit/origin.rs:39 | private | _no doc comment_ | impl Ids<'_> |
| `Origin` | src/edit/origin.rs:245 | private | _no doc comment_ | impl Origin |
| `Recorded` | src/edit/record.rs:82 | private | _no doc comment_ | impl Recorded |
| `Line` | src/edit/resolution.rs:122 | private | _no doc comment_ | impl Line |
| `Transaction` | src/edit/runs.rs:14 | private | _no doc comment_ | impl Transaction<'_> |
| `Geometry` | src/edit/sections.rs:134 | private | _no doc comment_ | impl Geometry |
| `Transaction` | src/edit/sections.rs:349 | private | _no doc comment_ | impl Transaction<'_> |
| `Stamp` | src/edit/tracked.rs:39 | private | _no doc comment_ | impl Stamp<'_> |
| `Target` | src/edit/v2.rs:466 | private | _no doc comment_ | impl Target |
| `PlanV2` | src/edit/v2.rs:498 | private | _no doc comment_ | impl PlanV2 |
| `Operation` | src/edit/v2.rs:571 | private | _no doc comment_ | impl Operation |
| `Transaction` | src/edit/watermark.rs:171 | private | _no doc comment_ | impl Transaction<'_> |
| `Stamp` | src/edit/whole.rs:206 | private | _no doc comment_ | impl Stamp<'_> |
| `<FieldError as fmt::Display>` | src/fields.rs:84 | private | _no doc comment_ | impl fmt::Display for FieldError |
| `<FieldError as std::error::Error>` | src/fields.rs:94 | private | _no doc comment_ | impl std::error::Error for FieldError |
| `<FieldError as From>` | src/fields.rs:96 | private | _no doc comment_ | impl From<InspectError> for FieldError |
| `NumberFormat` | src/fields.rs:541 | private | _no doc comment_ | impl NumberFormat |
| `BookmarkNamer` | src/fields.rs:1084 | private | _no doc comment_ | impl BookmarkNamer |
| `IdError` | src/ids.rs:99 | private | _no doc comment_ | impl IdError |
| `<IdError as fmt::Display>` | src/ids.rs:111 | private | _no doc comment_ | impl fmt::Display for IdError |
| `<IdError as std::error::Error>` | src/ids.rs:121 | private | _no doc comment_ | impl std::error::Error for IdError |
| `ChangeId` | src/ids.rs:139 | private | _no doc comment_ | impl ChangeId |
| `Step` | src/ids/short.rs:72 | private | _no doc comment_ | impl Step |
| `Step` | src/ids/short.rs:94 | private | _no doc comment_ | impl Step |
| `<Step as fmt::Display>` | src/ids/short.rs:184 | private | _no doc comment_ | impl fmt::Display for Step |
| `ShortId` | src/ids/short.rs:294 | private | _no doc comment_ | impl ShortId |
| `<ShortId as FromStr>` | src/ids/short.rs:384 | private | _no doc comment_ | impl FromStr for ShortId |
| `<Story as fmt::Display>` | src/ids/short.rs:395 | private | _no doc comment_ | impl fmt::Display for Story |
| `<ShortId as fmt::Display>` | src/ids/short.rs:404 | private | _no doc comment_ | impl fmt::Display for ShortId |
| `Snapshot` | src/inspect.rs:149 | private | _no doc comment_ | impl Snapshot |
| `<InspectError as fmt::Display>` | src/inspect.rs:256 | private | _no doc comment_ | impl fmt::Display for InspectError |
| `<InspectError as std::error::Error>` | src/inspect.rs:267 | private | _no doc comment_ | impl std::error::Error for InspectError |
| `Opened` | src/inspect.rs:524 | private | _no doc comment_ | impl Opened |
| `LegacyDocError` | src/legacy_doc.rs:75 | private | _no doc comment_ | impl LegacyDocError |
| `<LegacyDocError as fmt::Display>` | src/legacy_doc.rs:83 | private | _no doc comment_ | impl fmt::Display for LegacyDocError |
| `<LegacyDocError as std::error::Error>` | src/legacy_doc.rs:89 | private | _no doc comment_ | impl std::error::Error for LegacyDocError |
| `Paragraph` | src/legacy_doc.rs:127 | private | _no doc comment_ | impl Paragraph |
| `CompoundFile` | src/legacy_doc.rs:657 | private | _no doc comment_ | impl<'a> CompoundFile<'a> |
| `DirEntry` | src/legacy_doc.rs:844 | private | _no doc comment_ | impl DirEntry |
| `Fib` | src/legacy_doc.rs:894 | private | _no doc comment_ | impl Fib |
| `CharModifier` | src/legacy_doc.rs:980 | private | _no doc comment_ | impl CharModifier |
| `FkpIndex` | src/legacy_doc.rs:1295 | private | _no doc comment_ | impl FkpIndex |
| `Lists` | src/legacy_doc.rs:1488 | private | _no doc comment_ | impl Lists |
| `Story` | src/legacy_doc.rs:1587 | private | _no doc comment_ | impl Story<'_> |
| `Border` | src/markdown/borders.rs:49 | private | _no doc comment_ | impl Border |
| `Closers` | src/markdown/critic.rs:451 | private | _no doc comment_ | impl<'a> Closers<'a> |
| `Literal` | src/markdown/critic.rs:727 | private | _no doc comment_ | impl Literal |
| `Pieces` | src/markdown/critic.rs:1708 | private | _no doc comment_ | impl Pieces |
| `Output` | src/markdown/diff.rs:379 | private | _no doc comment_ | impl Output |
| `Side` | src/markdown/diff.rs:453 | private | _no doc comment_ | impl Side |
| `Handles` | src/markdown/from_docx/agent.rs:45 | private | _no doc comment_ | impl Handles |
| `Mark` | src/markdown/from_docx/critic.rs:42 | private | _no doc comment_ | impl Mark |
| `Note` | src/markdown/from_docx/critic.rs:244 | private | _no doc comment_ | impl Note |
| `Node` | src/markdown/from_docx/critic.rs:275 | private | _no doc comment_ | impl Node |
| `Critic` | src/markdown/from_docx/critic.rs:302 | private | _no doc comment_ | impl Critic |
| `Escaping` | src/markdown/from_docx/critic.rs:820 | private | _no doc comment_ | impl Escaping |
| `ScopeMap` | src/markdown/from_docx/drawing.rs:158 | private | _no doc comment_ | impl ScopeMap |
| `Pending` | src/markdown/from_docx/drawing.rs:208 | private | _no doc comment_ | impl Pending |
| `Deps` | src/markdown/from_docx/drawing.rs:435 | private | _no doc comment_ | impl Deps<'_> |
| `Drawing` | src/markdown/from_docx/drawing.rs:1304 | private | Write `drawing` into `package` as content of `part`: the XML with fresh relationship ids, and every dependent part (its own relationships and parts too). | impl Drawing |
| `State` | src/markdown/from_docx/header.rs:1816 | private | _no doc comment_ | impl State<'_> |
| `Shown` | src/markdown/from_docx/header.rs:1883 | private | _no doc comment_ | impl Shown |
| `Extracted` | src/markdown/from_docx/media.rs:24 | private | _no doc comment_ | impl Extracted |
| `<ConvertError as std::fmt::Display>` | src/markdown/from_docx/mod.rs:70 | private | _no doc comment_ | impl std::fmt::Display for ConvertError |
| `Styles` | src/markdown/from_docx/mod.rs:1725 | private | _no doc comment_ | impl Styles |
| `Numbering` | src/markdown/from_docx/mod.rs:1969 | private | _no doc comment_ | impl Numbering |
| `Writer` | src/markdown/from_docx/mod.rs:2382 | private | _no doc comment_ | impl Writer<'_> |
| `Package` | src/markdown/from_docx/ooxml.rs:37 | private | _no doc comment_ | impl<'a> Package<'a> |
| `Rels` | src/markdown/from_docx/ooxml.rs:205 | private | _no doc comment_ | impl Rels |
| `Element` | src/markdown/from_docx/ooxml.rs:276 | private | _no doc comment_ | impl Element |
| `Inline` | src/markdown/from_docx/ooxml.rs:540 | private | _no doc comment_ | impl Inline |
| `Media` | src/markdown/from_docx/ooxml.rs:1051 | private | _no doc comment_ | impl Media |
| `Blocks` | src/markdown/from_docx/ooxml.rs:1124 | private | _no doc comment_ | impl Blocks |
| `ListIndent` | src/markdown/from_docx/ooxml.rs:1220 | private | _no doc comment_ | impl ListIndent |
| `Parser` | src/markdown/math.rs:791 | private | _no doc comment_ | impl Parser |
| `Writer` | src/markdown/math.rs:1672 | private | _no doc comment_ | impl Writer<'_> |
| `TrackChanges` | src/markdown/mod.rs:84 | private | _no doc comment_ | impl TrackChanges |
| `PageSize` | src/markdown/mod.rs:114 | private | _no doc comment_ | impl PageSize |
| `<DocxOptions as Default>` | src/markdown/mod.rs:216 | private | _no doc comment_ | impl Default for DocxOptions<'_> |
| `<DocxOptions as std::fmt::Debug>` | src/markdown/mod.rs:286 | private | _no doc comment_ | impl std::fmt::Debug for DocxOptions<'_> |
| `<MarkdownOptions as Default>` | src/markdown/mod.rs:339 | private | _no doc comment_ | impl Default for MarkdownOptions |
| `Select` | src/markdown/mod.rs:406 | private | _no doc comment_ | impl Select |
| `<ReadOptions as Default>` | src/markdown/mod.rs:694 | private | _no doc comment_ | impl Default for ReadOptions |
| `<MarkdownError as std::fmt::Display>` | src/markdown/mod.rs:989 | private | _no doc comment_ | impl std::fmt::Display for MarkdownError |
| `<MarkdownError as std::error::Error>` | src/markdown/mod.rs:999 | private | _no doc comment_ | impl std::error::Error for MarkdownError |
| `<PartRelate as Relate>` | src/markdown/package.rs:1488 | private | _no doc comment_ | impl Relate for PartRelate<'_, '_, '_> |
| `Block` | src/markdown/patch.rs:77 | private | _no doc comment_ | impl Block |
| `<RedlineOptions as std::fmt::Debug>` | src/markdown/redline.rs:65 | private | _no doc comment_ | impl std::fmt::Debug for RedlineOptions<'_> |
| `<Locator as fmt::Display>` | src/markdown/unified.rs:83 | private | _no doc comment_ | impl fmt::Display for Locator |
| `Patch` | src/markdown/unified.rs:161 | private | _no doc comment_ | impl Patch |
| `<Patch as fmt::Display>` | src/markdown/unified.rs:188 | private | _no doc comment_ | impl fmt::Display for Patch |
| `Search` | src/markdown/unified.rs:414 | private | _no doc comment_ | impl<'a> Search<'a> |
| `Edit` | src/markdown/unified.rs:596 | private | _no doc comment_ | impl Edit |
| `Block` | src/markdown/unified.rs:822 | private | _no doc comment_ | impl Block |
| `CellsAt` | src/markdown/view_notes.rs:559 | private | _no doc comment_ | impl CellsAt |
| `IdLine` | src/markdown/view_notes.rs:1348 | private | _no doc comment_ | impl IdLine |
| `PageSetup` | src/markdown/view_notes.rs:1989 | private | _no doc comment_ | impl PageSetup |
| `ParaFormat` | src/markdown/view_stories.rs:78 | private | _no doc comment_ | impl ParaFormat |
| `Marks` | src/markdown/view_stories.rs:141 | private | _no doc comment_ | impl Marks<'_> |
| `Stories` | src/markdown/view_stories.rs:486 | private | _no doc comment_ | impl Stories |
| `SectionStories` | src/markdown/view_stories.rs:950 | private | _no doc comment_ | impl SectionStories |
| `Given` | src/markdown/view_styles.rs:568 | private | _no doc comment_ | impl Given |
| `Anchor` | src/markdown/write.rs:1504 | private | _no doc comment_ | impl Anchor |
| `Writer` | src/markdown/write.rs:1704 | private | _no doc comment_ | impl<'o, 'a> Writer<'o, 'a> |
| `Slugger` | src/markdown/write.rs:5031 | private | _no doc comment_ | impl Slugger |
| `Writer` | src/markdown/write.rs:5115 | private | _no doc comment_ | impl Writer<'_, '_> |
| `Document` | src/markdown/xml.rs:383 | private | _no doc comment_ | impl Document |
| `Context` | src/markdown/xml.rs:506 | private | _no doc comment_ | impl Context<'_> |
| `W` | src/namespaces.rs:194 | private | _no doc comment_ | impl W |
| `PT` | src/namespaces.rs:261 | private | ── PowerTools correlation attribute names ──────────────────────────────────── _(from a code comment)_ | impl PT |
| `Relationships` | src/opc/mod.rs:39 | private | _no doc comment_ | impl Relationships |
| `<Relationships as Deref>` | src/opc/mod.rs:55 | private | _no doc comment_ | impl Deref for Relationships |
| `<Relationships as DerefMut>` | src/opc/mod.rs:63 | private | _no doc comment_ | impl DerefMut for Relationships |
| `PartFs` | src/opc/mod.rs:441 | private | _no doc comment_ | impl PartFs |
| `Snapshot` | src/perf.rs:168 | private | _no doc comment_ | impl Snapshot |
| `Artifact` | src/result.rs:183 | private | _no doc comment_ | impl Artifact |
| `Context` | src/result.rs:237 | private | _no doc comment_ | impl Context |
| `NextAction` | src/result.rs:276 | private | _no doc comment_ | impl NextAction |
| `Error` | src/result.rs:335 | private | _no doc comment_ | impl Error |
| `Source` | src/result.rs:449 | private | _no doc comment_ | impl Source |
| `Runtime` | src/result.rs:487 | private | _no doc comment_ | impl Runtime |
| `<Verification as Default>` | src/result.rs:511 | private | _no doc comment_ | impl Default for Verification |
| `Counts` | src/result.rs:573 | private | _no doc comment_ | impl Counts |
| `<ResultDataError as std::fmt::Display>` | src/result.rs:634 | private | _no doc comment_ | impl std::fmt::Display for ResultDataError |
| `<ResultDataError as std::error::Error>` | src/result.rs:655 | private | _no doc comment_ | impl std::error::Error for ResultDataError |
| `ResultData` | src/result.rs:657 | private | _no doc comment_ | impl ResultData |
| `Envelope` | src/result.rs:770 | private | _no doc comment_ | impl Envelope |
| `<StreamHeader as From>` | src/result.rs:919 | private | _no doc comment_ | impl From<&Envelope> for StreamHeader |
| `ProgressEvent` | src/result.rs:942 | private | _no doc comment_ | impl ProgressEvent |
| `<ProgressEvent as From>` | src/result.rs:954 | private | _no doc comment_ | impl From<serde_json::Value> for ProgressEvent |
| `<serde_json::Value as From>` | src/result.rs:960 | private | _no doc comment_ | impl From<ProgressEvent> for serde_json::Value |
| `<StreamError as std::fmt::Display>` | src/result.rs:1029 | private | _no doc comment_ | impl std::fmt::Display for StreamError |
| `<StreamError as std::error::Error>` | src/result.rs:1048 | private | _no doc comment_ | impl std::error::Error for StreamError |
| `ExitStatus` | src/result.rs:1134 | private | _no doc comment_ | impl ExitStatus |
| `<u8 as From>` | src/result.rs:1154 | private | _no doc comment_ | impl From<ExitStatus> for u8 |
| `Parsed` | src/revision_processor/comments.rs:37 | private | _no doc comment_ | impl Parsed |
| `Slot` | src/revision_processor/style_records.rs:109 | private | _no doc comment_ | impl Slot |
| `Levels` | src/revision_processor/style_records.rs:280 | private | _no doc comment_ | impl Levels |
| `<ScrubOptions as Default>` | src/scrub.rs:68 | private | _no doc comment_ | impl Default for ScrubOptions |
| `<ScrubError as std::fmt::Display>` | src/scrub.rs:89 | private | _no doc comment_ | impl std::fmt::Display for ScrubError |
| `<ScrubError as std::error::Error>` | src/scrub.rs:98 | private | _no doc comment_ | impl std::error::Error for ScrubError |
| `Part` | src/scrub.rs:188 | private | _no doc comment_ | impl Part |
| `ProtectionEdit` | src/settings.rs:160 | private | _no doc comment_ | impl ProtectionEdit |
| `SettingsRequest` | src/settings.rs:211 | private | _no doc comment_ | impl SettingsRequest |
| `UriSite` | src/strict_translation.rs:529 | private | _no doc comment_ | impl UriSite |
| `<UnifiedOptions as Default>` | src/text_diff.rs:31 | private | _no doc comment_ | impl Default for UnifiedOptions |
| `<TextOptions as Default>` | src/text_diff.rs:82 | private | _no doc comment_ | impl Default for TextOptions |
| `Piece` | src/text_diff.rs:471 | private | _no doc comment_ | impl Piece |
| `Finding` | src/validate.rs:115 | private | _no doc comment_ | impl Finding |
| `<ValidateError as fmt::Display>` | src/validate.rs:226 | private | _no doc comment_ | impl fmt::Display for ValidateError |
| `<ValidateError as std::error::Error>` | src/validate.rs:239 | private | _no doc comment_ | impl std::error::Error for ValidateError |
| `<ValidateError as From>` | src/validate.rs:241 | private | _no doc comment_ | impl From<AdmissionError> for ValidateError |
| `<ValidateError as From>` | src/validate.rs:247 | private | _no doc comment_ | impl From<ChangeError> for ValidateError |
| `<ValidateError as From>` | src/validate.rs:253 | private | _no doc comment_ | impl From<InspectError> for ValidateError |
| `<ValidateError as From>` | src/validate.rs:259 | private | _no doc comment_ | impl From<crate::opc::OpcError> for ValidateError |
| `<Finding as From>` | src/validate.rs:388 | private | _no doc comment_ | impl From<crate::audit::AuditFinding> for Finding |
| `WmlDocument` | src/wml_document.rs:35 | private | _no doc comment_ | impl WmlDocument |
| `XNamespace` | src/xmllinq/mod.rs:104 | private | _no doc comment_ | impl XNamespace |
| `<XNamespace as PartialEq>` | src/xmllinq/mod.rs:145 | private | _no doc comment_ | impl PartialEq for XNamespace |
| `<XNamespace as Eq>` | src/xmllinq/mod.rs:154 | private | _no doc comment_ | impl Eq for XNamespace |
| `<XNamespace as std::hash::Hash>` | src/xmllinq/mod.rs:155 | private | _no doc comment_ | impl std::hash::Hash for XNamespace |
| `<XNamespace as std::fmt::Debug>` | src/xmllinq/mod.rs:160 | private | _no doc comment_ | impl std::fmt::Debug for XNamespace |
| `XName` | src/xmllinq/mod.rs:175 | private | _no doc comment_ | impl XName |
| `<XName as PartialEq>` | src/xmllinq/mod.rs:235 | private | _no doc comment_ | impl PartialEq for XName |
| `<XName as Eq>` | src/xmllinq/mod.rs:243 | private | _no doc comment_ | impl Eq for XName |
| `<XName as std::hash::Hash>` | src/xmllinq/mod.rs:244 | private | _no doc comment_ | impl std::hash::Hash for XName |
| `<XName as std::fmt::Debug>` | src/xmllinq/mod.rs:250 | private | _no doc comment_ | impl std::fmt::Debug for XName |
| `NodeData` | src/xmllinq/mod.rs:349 | private | _no doc comment_ | impl NodeData |
| `Dom` | src/xmllinq/mod.rs:383 | private | _no doc comment_ | impl Dom |
| `NsScope` | src/xmllinq/parse.rs:153 | private | _no doc comment_ | impl NsScope |
| `TagPolicy` | src/xmllinq/parse.rs:208 | private | _no doc comment_ | impl TagPolicy |
| `Parser` | src/xmllinq/parse.rs:370 | private | _no doc comment_ | impl<'a> Parser<'a> |
| `Scope` | src/xmllinq/serialize.rs:210 | private | _no doc comment_ | impl Scope<'static> |
| `Scope` | src/xmllinq/serialize.rs:224 | private | _no doc comment_ | impl<'a> Scope<'a> |
| `<String as XmlBuf>` | src/xmllinq/serialize.rs:446 | private | _no doc comment_ | impl XmlBuf for String |
| `HashXmlBuf` | src/xmllinq/serialize.rs:471 | private | _no doc comment_ | impl HashXmlBuf |
| `<HashXmlBuf as XmlBuf>` | src/xmllinq/serialize.rs:501 | private | _no doc comment_ | impl XmlBuf for HashXmlBuf |
