# Impl blocks

389 items (non-test). Part of the [symbol manifest](README.md).

| Name | Defined at | Vis | Does | Detail |
|---|---|---|---|---|
| `<InputLimits as Default>` | src/admission.rs:63 | private | _no doc comment_ | impl Default for InputLimits |
| `InputLimits` | src/admission.rs:80 | private | _no doc comment_ | impl InputLimits |
| `InputLimitOverrides` | src/admission.rs:130 | private | _no doc comment_ | impl InputLimitOverrides |
| `AdmissionErrorKind` | src/admission.rs:194 | private | _no doc comment_ | impl AdmissionErrorKind |
| `AdmissionError` | src/admission.rs:297 | private | _no doc comment_ | impl AdmissionError |
| `<AdmissionError as fmt::Display>` | src/admission.rs:312 | private | _no doc comment_ | impl fmt::Display for AdmissionError |
| `<AdmissionError as std::error::Error>` | src/admission.rs:318 | private | _no doc comment_ | impl std::error::Error for AdmissionError |
| `AppendError` | src/append.rs:112 | private | _no doc comment_ | impl AppendError |
| `<AppendError as fmt::Display>` | src/append.rs:124 | private | _no doc comment_ | impl fmt::Display for AppendError |
| `<AppendError as std::error::Error>` | src/append.rs:135 | private | _no doc comment_ | impl std::error::Error for AppendError |
| `ListCopy` | src/append.rs:1114 | private | _no doc comment_ | impl ListCopy<'_> |
| `<AuditError as fmt::Display>` | src/audit.rs:94 | private | _no doc comment_ | impl fmt::Display for AuditError |
| `<AuditError as std::error::Error>` | src/audit.rs:112 | private | _no doc comment_ | impl std::error::Error for AuditError |
| `<AuditError as From>` | src/audit.rs:114 | private | _no doc comment_ | impl From<InspectError> for AuditError |
| `Story` | src/audit.rs:314 | private | _no doc comment_ | impl Story |
| `Package` | src/audit.rs:353 | private | _no doc comment_ | impl Package |
| `Field` | src/audit.rs:835 | private | _no doc comment_ | impl Field |
| `<ScrubSelection as ScrubOptions>` | src/bin/jubarte.rs:73 | private | _no doc comment_ | impl ScrubOptions for ScrubSelection |
| `<Selection as SelectionFilter>` | src/bin/jubarte.rs:91 | private | _no doc comment_ | impl SelectionFilter for Selection |
| `Output` | src/bin/jubarte.rs:296 | private | _no doc comment_ | impl<'a> Output<'a> |
| `<ConvertFailure as From>` | src/bin/jubarte.rs:615 | private | _no doc comment_ | impl From<String> for ConvertFailure |
| `<ConvertFailure as From>` | src/bin/jubarte.rs:621 | private | _no doc comment_ | impl From<(u8, String)> for ConvertFailure |
| `<ConvertFailure as From>` | src/bin/jubarte.rs:627 | private | _no doc comment_ | impl From<&str> for ConvertFailure |
| `Input` | src/bin/jubarte.rs:3009 | private | _no doc comment_ | impl Input |
| `Request` | src/bin/jubarte/edit_typed.rs:77 | private | _no doc comment_ | impl Request |
| `Operations` | src/capabilities.rs:207 | private | _no doc comment_ | impl Operations |
| `<InputBudget as From>` | src/capabilities.rs:320 | private | _no doc comment_ | impl From<crate::admission::InputLimits> for InputBudget |
| `Change` | src/changes.rs:115 | private | _no doc comment_ | impl Change |
| `ChangeFilter` | src/changes.rs:327 | private | _no doc comment_ | impl ChangeFilter |
| `<ChangeError as fmt::Display>` | src/changes.rs:450 | private | _no doc comment_ | impl fmt::Display for ChangeError |
| `<ChangeError as std::error::Error>` | src/changes.rs:508 | private | _no doc comment_ | impl std::error::Error for ChangeError |
| `Phase` | src/changes/logical.rs:63 | private | _no doc comment_ | impl Phase |
| `EditOptions` | src/cli.rs:354 | private | _no doc comment_ | impl EditOptions |
| `ExistingArg` | src/cli.rs:396 | private | _no doc comment_ | impl ExistingArg |
| `<Cli as std::ops::Deref>` | src/cli.rs:871 | private | _no doc comment_ | impl std::ops::Deref for Cli |
| `<crate::debug::Check as From>` | src/cli.rs:1752 | private | _no doc comment_ | impl From<DebugCheck> for crate::debug::Check |
| `PatchFormat` | src/cli.rs:1799 | private | _no doc comment_ | impl PatchFormat |
| `Format` | src/cli.rs:1850 | private | _no doc comment_ | impl Format |
| `Task` | src/cli.rs:1914 | private | _no doc comment_ | impl Task |
| `<crate::markdown::TrackChanges as From>` | src/cli.rs:2501 | private | _no doc comment_ | impl From<TrackChanges> for crate::markdown::TrackChanges |
| `<RenderArgs as Default>` | src/cli.rs:2555 | private | _no doc comment_ | impl Default for RenderArgs |
| `RenderArgs` | src/cli.rs:2569 | private | _no doc comment_ | impl RenderArgs |
| `<crate::markdown::PageSize as From>` | src/cli.rs:2691 | private | _no doc comment_ | impl From<Page> for crate::markdown::PageSize |
| `<crate::append::SectionBreak as From>` | src/cli.rs:2773 | private | _no doc comment_ | impl From<SectionBreakArg> for crate::append::SectionBreak |
| `Cli` | src/cli.rs:3489 | private | _no doc comment_ | impl Cli |
| `EditAction` | src/cli/edit_actions.rs:329 | private | _no doc comment_ | impl EditAction |
| `CommentError` | src/comments.rs:92 | private | _no doc comment_ | impl CommentError |
| `<CommentError as fmt::Display>` | src/comments.rs:103 | private | _no doc comment_ | impl fmt::Display for CommentError |
| `<CommentError as std::error::Error>` | src/comments.rs:112 | private | _no doc comment_ | impl std::error::Error for CommentError |
| `<CommentError as From>` | src/comments.rs:114 | private | _no doc comment_ | impl From<crate::inspect::InspectError> for CommentError |
| `Anchors` | src/comments.rs:247 | private | _no doc comment_ | impl Anchors |
| `CommentFamily` | src/comments.rs:455 | private | _no doc comment_ | impl CommentFamily |
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
| `ParagraphPropertySources` | src/comparer/finalize.rs:13102 | private | _no doc comment_ | impl ParagraphPropertySources |
| `<RectifyError as std::fmt::Display>` | src/comparer/footnotes.rs:610 | private | _no doc comment_ | impl std::fmt::Display for RectifyError |
| `<RectifyError as std::error::Error>` | src/comparer/footnotes.rs:641 | private | _no doc comment_ | impl std::error::Error for RectifyError |
| `<U64IdentityHasher as std::hash::Hasher>` | src/comparer/lcs.rs:38 | private | _no doc comment_ | impl std::hash::Hasher for U64IdentityHasher |
| `WmlComparerSettings` | src/comparer/mod.rs:1439 | private | _no doc comment_ | impl WmlComparerSettings |
| `<WmlComparerSettings as Default>` | src/comparer/mod.rs:1492 | private | _no doc comment_ | impl Default for WmlComparerSettings |
| `CompareContext` | src/comparer/mod.rs:1648 | private | _no doc comment_ | impl CompareContext |
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
| `RunStyle` | src/convert/mod.rs:1129 | private | _no doc comment_ | impl RunStyle |
| `ThemeFonts` | src/convert/mod.rs:1510 | private | _no doc comment_ | impl ThemeFonts |
| `Defaults` | src/convert/mod.rs:1685 | private | _no doc comment_ | impl Defaults |
| `TextRun` | src/convert/mod.rs:1929 | private | _no doc comment_ | impl TextRun |
| `TableCell` | src/convert/mod.rs:2248 | private | _no doc comment_ | impl TableCell |
| `RowSrc` | src/convert/mod.rs:2310 | private | _no doc comment_ | impl RowSrc<'_> |
| `RelFrame` | src/convert/mod.rs:2703 | private | _no doc comment_ | impl RelFrame |
| `Numbering` | src/convert/mod.rs:5508 | private | _no doc comment_ | impl Numbering |
| `EndnoteBag` | src/convert/mod.rs:8515 | private | _no doc comment_ | impl EndnoteBag |
| `AuthorColors` | src/convert/mod.rs:15089 | private | _no doc comment_ | impl AuthorColors |
| `MathFace` | src/convert/mod.rs:15814 | private | _no doc comment_ | impl MathFace |
| `MathAlphanumerics` | src/convert/mod.rs:15842 | private | _no doc comment_ | impl MathAlphanumerics |
| `<MathAlphanumerics as Drop>` | src/convert/mod.rs:15854 | private | _no doc comment_ | impl Drop for MathAlphanumerics |
| `Layout` | src/convert/mod.rs:23098 | private | _no doc comment_ | impl<'a> Layout<'a> |
| `Squeeze` | src/convert/mod.rs:33008 | private | _no doc comment_ | impl Squeeze |
| `Page` | src/convert/pdf.rs:201 | private | _no doc comment_ | impl Page |
| `Op` | src/convert/pdf.rs:307 | private | _no doc comment_ | impl Op |
| `FaceObjIds` | src/convert/pdf.rs:1126 | private | _no doc comment_ | impl FaceObjIds |
| `Show` | src/convert/pdf.rs:1378 | private | _no doc comment_ | impl<'a> Show<'a> |
| `<Show as std::fmt::Display>` | src/convert/pdf.rs:1404 | private | _no doc comment_ | impl std::fmt::Display for Show<'_> |
| `Fill` | src/convert/preset_geom.rs:31 | private | _no doc comment_ | impl Fill |
| `Cmd` | src/convert/preset_geom.rs:86 | private | _no doc comment_ | impl Cmd |
| `OwnedCmd` | src/convert/preset_geom.rs:109 | private | _no doc comment_ | impl OwnedCmd |
| `<OutlineSink as ttf_parser::OutlineBuilder>` | src/convert/raster.rs:514 | private | _no doc comment_ | impl ttf_parser::OutlineBuilder for OutlineSink |
| `<Options as Default>` | src/debug.rs:142 | private | _no doc comment_ | impl Default for Options |
| `Package` | src/debug.rs:179 | private | _no doc comment_ | impl Package |
| `Findings` | src/debug.rs:362 | private | _no doc comment_ | impl Findings |
| `<DiffOptions as Default>` | src/debug/diff.rs:31 | private | _no doc comment_ | impl Default for DiffOptions |
| `Filter` | src/debug/diff.rs:138 | private | _no doc comment_ | impl Filter |
| `N` | src/debug/diff.rs:164 | private | _no doc comment_ | impl N |
| `Build` | src/debug/diff.rs:490 | private | _no doc comment_ | impl Build<'_> |
| `Seg` | src/debug/diff.rs:1196 | private | _no doc comment_ | impl Seg |
| `Walker` | src/debug/diff.rs:1250 | private | _no doc comment_ | impl Walker<'_> |
| `StyleBook` | src/debug/render.rs:155 | private | _no doc comment_ | impl StyleBook |
| `Tally` | src/debug/render.rs:478 | private | _no doc comment_ | impl Tally |
| `RemovedAnchorComments` | src/edit.rs:161 | private | _no doc comment_ | impl RemovedAnchorComments |
| `<ResolveRevisions as From>` | src/edit.rs:209 | private | _no doc comment_ | impl From<ResolveRevisionsWire> for ResolveRevisions |
| `<ResolveRevisionsWire as From>` | src/edit.rs:224 | private | _no doc comment_ | impl From<ResolveRevisions> for ResolveRevisionsWire |
| `ResolvedRevisions` | src/edit.rs:269 | private | _no doc comment_ | impl ResolvedRevisions |
| `ListKind` | src/edit.rs:821 | private | _no doc comment_ | impl ListKind |
| `<LineSpacing as Serialize>` | src/edit.rs:845 | private | _no doc comment_ | impl Serialize for LineSpacing |
| `<LineSpacing as Deserialize>` | src/edit.rs:851 | private | _no doc comment_ | impl<'de> Deserialize<'de> for LineSpacing |
| `<Points as Serialize>` | src/edit.rs:868 | private | _no doc comment_ | impl Serialize for Points |
| `<Points as Deserialize>` | src/edit.rs:874 | private | _no doc comment_ | impl<'de> Deserialize<'de> for Points |
| `Alignment` | src/edit.rs:908 | private | _no doc comment_ | impl Alignment |
| `<HalfPoints as Serialize>` | src/edit.rs:959 | private | _no doc comment_ | impl Serialize for HalfPoints |
| `<HalfPoints as Deserialize>` | src/edit.rs:965 | private | _no doc comment_ | impl<'de> Deserialize<'de> for HalfPoints |
| `RunSpec` | src/edit.rs:1071 | private | _no doc comment_ | impl RunSpec |
| `RevisionCounts` | src/edit.rs:1147 | private | _no doc comment_ | impl RevisionCounts |
| `EditReport` | src/edit.rs:1243 | private | _no doc comment_ | impl EditReport |
| `<EditError as fmt::Display>` | src/edit.rs:1403 | private | _no doc comment_ | impl fmt::Display for EditError |
| `<EditError as std::error::Error>` | src/edit.rs:1412 | private | _no doc comment_ | impl std::error::Error for EditError |
| `EditPlan` | src/edit.rs:1781 | private | _no doc comment_ | impl EditPlan |
| `ThreadOp` | src/edit.rs:2565 | private | _no doc comment_ | impl ThreadOp |
| `Transaction` | src/edit.rs:2679 | private | _no doc comment_ | impl<'p> Transaction<'p> |
| `EmptyPlacement` | src/edit.rs:7713 | private | _no doc comment_ | impl EmptyPlacement |
| `NearMatches` | src/edit/anchor.rs:136 | private | _no doc comment_ | impl NearMatches |
| `AnchorRefusal` | src/edit/anchor_error.rs:80 | private | _no doc comment_ | impl<'a> AnchorRefusal<'a> |
| `AnchorChanges` | src/edit/comment_settlement.rs:55 | private | _no doc comment_ | impl AnchorChanges |
| `Transaction` | src/edit/controls.rs:72 | private | _no doc comment_ | impl Transaction<'_> |
| `<ForEditRequest as Default>` | src/edit/for_edit.rs:83 | private | _no doc comment_ | impl Default for ForEditRequest |
| `TextValue` | src/edit/for_edit.rs:135 | private | _no doc comment_ | impl TextValue |
| `Selection` | src/edit/for_edit.rs:546 | private | _no doc comment_ | impl Selection |
| `NativeKeys` | src/edit/for_edit.rs:850 | private | _no doc comment_ | impl NativeKeys |
| `PlanSyntax` | src/edit/hint.rs:28 | private | _no doc comment_ | impl PlanSyntax |
| `Held` | src/edit/hint.rs:55 | private | _no doc comment_ | impl Held |
| `Transaction` | src/edit/images.rs:139 | private | _no doc comment_ | impl Transaction<'_> |
| `Transaction` | src/edit/notes.rs:21 | private | _no doc comment_ | impl Transaction<'_> |
| `Ids` | src/edit/origin.rs:39 | private | _no doc comment_ | impl Ids<'_> |
| `Origin` | src/edit/origin.rs:249 | private | _no doc comment_ | impl Origin |
| `Paragraphs` | src/edit/origin.rs:584 | private | _no doc comment_ | impl Paragraphs |
| `CandidateChecks` | src/edit/prepared.rs:92 | private | _no doc comment_ | impl CandidateChecks |
| `<PrepareOptions as Default>` | src/edit/prepared.rs:157 | private | _no doc comment_ | impl Default for PrepareOptions |
| `Recorded` | src/edit/record.rs:82 | private | _no doc comment_ | impl Recorded |
| `RevisionRefusal` | src/edit/refusal.rs:50 | private | _no doc comment_ | impl RevisionRefusal |
| `EditError` | src/edit/refusal.rs:144 | private | _no doc comment_ | impl EditError |
| `Transaction` | src/edit/refusal.rs:194 | private | _no doc comment_ | impl Transaction<'_> |
| `Line` | src/edit/resolution.rs:123 | private | _no doc comment_ | impl Line |
| `Transaction` | src/edit/runs.rs:14 | private | _no doc comment_ | impl Transaction<'_> |
| `Geometry` | src/edit/sections.rs:134 | private | _no doc comment_ | impl Geometry |
| `Transaction` | src/edit/sections.rs:349 | private | _no doc comment_ | impl Transaction<'_> |
| `Stamp` | src/edit/tracked.rs:49 | private | _no doc comment_ | impl Stamp<'_> |
| `Target` | src/edit/v2.rs:477 | private | _no doc comment_ | impl Target |
| `PlanV2` | src/edit/v2.rs:509 | private | _no doc comment_ | impl PlanV2 |
| `Operation` | src/edit/v2.rs:583 | private | _no doc comment_ | impl Operation |
| `SourceIndex` | src/edit/v2_apply.rs:712 | private | _no doc comment_ | impl SourceIndex |
| `SourceModel` | src/edit/v2_apply.rs:813 | private | _no doc comment_ | impl SourceModel |
| `Visible` | src/edit/visible.rs:31 | private | _no doc comment_ | impl Visible |
| `Transaction` | src/edit/visible.rs:344 | private | _no doc comment_ | impl Transaction<'_> |
| `Transaction` | src/edit/watermark.rs:171 | private | _no doc comment_ | impl Transaction<'_> |
| `Stamp` | src/edit/whole.rs:201 | private | _no doc comment_ | impl Stamp<'_> |
| `<RefusalReason as std::fmt::Display>` | src/expect.rs:197 | private | Human-readable refusal reasons, matching the serde names. | impl std::fmt::Display for RefusalReason |
| `ExpectationReport` | src/expect.rs:272 | private | _no doc comment_ | impl ExpectationReport |
| `CorrespondenceEntry` | src/expect.rs:290 | private | _no doc comment_ | impl CorrespondenceEntry |
| `SourceToOutputCorrespondence` | src/expect.rs:334 | private | _no doc comment_ | impl SourceToOutputCorrespondence |
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
| `<ShortId as FromStr>` | src/ids/short.rs:386 | private | _no doc comment_ | impl FromStr for ShortId |
| `<Story as fmt::Display>` | src/ids/short.rs:397 | private | _no doc comment_ | impl fmt::Display for Story |
| `<ShortId as fmt::Display>` | src/ids/short.rs:406 | private | _no doc comment_ | impl fmt::Display for ShortId |
| `Snapshot` | src/inspect.rs:177 | private | _no doc comment_ | impl Snapshot |
| `<InspectError as fmt::Display>` | src/inspect.rs:284 | private | _no doc comment_ | impl fmt::Display for InspectError |
| `<InspectError as std::error::Error>` | src/inspect.rs:295 | private | _no doc comment_ | impl std::error::Error for InspectError |
| `Opened` | src/inspect.rs:734 | private | _no doc comment_ | impl Opened |
| `LegacyDocError` | src/legacy_doc.rs:75 | private | _no doc comment_ | impl LegacyDocError |
| `<LegacyDocError as fmt::Display>` | src/legacy_doc.rs:83 | private | _no doc comment_ | impl fmt::Display for LegacyDocError |
| `<LegacyDocError as std::error::Error>` | src/legacy_doc.rs:89 | private | _no doc comment_ | impl std::error::Error for LegacyDocError |
| `Paragraph` | src/legacy_doc.rs:127 | private | _no doc comment_ | impl Paragraph |
| `CompoundFile` | src/legacy_doc.rs:657 | private | _no doc comment_ | impl<'a> CompoundFile<'a> |
| `DirEntry` | src/legacy_doc.rs:882 | private | _no doc comment_ | impl DirEntry |
| `Fib` | src/legacy_doc.rs:932 | private | _no doc comment_ | impl Fib |
| `CharModifier` | src/legacy_doc.rs:1018 | private | _no doc comment_ | impl CharModifier |
| `FkpIndex` | src/legacy_doc.rs:1333 | private | _no doc comment_ | impl FkpIndex |
| `Lists` | src/legacy_doc.rs:1526 | private | _no doc comment_ | impl Lists |
| `Story` | src/legacy_doc.rs:1625 | private | _no doc comment_ | impl Story<'_> |
| `Border` | src/markdown/borders.rs:49 | private | _no doc comment_ | impl Border |
| `Closers` | src/markdown/critic.rs:521 | private | _no doc comment_ | impl<'a> Closers<'a> |
| `Literal` | src/markdown/critic.rs:797 | private | _no doc comment_ | impl Literal |
| `Pieces` | src/markdown/critic.rs:1910 | private | _no doc comment_ | impl Pieces |
| `SeamSpan` | src/markdown/critic.rs:2214 | private | _no doc comment_ | impl SeamSpan |
| `Output` | src/markdown/diff.rs:388 | private | _no doc comment_ | impl Output |
| `Side` | src/markdown/diff.rs:462 | private | _no doc comment_ | impl Side |
| `Handles` | src/markdown/from_docx/agent.rs:50 | private | _no doc comment_ | impl Handles |
| `<Restore as Drop>` | src/markdown/from_docx/agent.rs:398 | private | _no doc comment_ | impl Drop for Restore |
| `<Restore as Drop>` | src/markdown/from_docx/agent.rs:484 | private | _no doc comment_ | impl Drop for Restore |
| `Mark` | src/markdown/from_docx/critic.rs:43 | private | _no doc comment_ | impl Mark |
| `Note` | src/markdown/from_docx/critic.rs:285 | private | _no doc comment_ | impl Note |
| `Node` | src/markdown/from_docx/critic.rs:316 | private | _no doc comment_ | impl Node |
| `Critic` | src/markdown/from_docx/critic.rs:345 | private | _no doc comment_ | impl Critic |
| `Escaping` | src/markdown/from_docx/critic.rs:886 | private | _no doc comment_ | impl Escaping<'_> |
| `ScopeMap` | src/markdown/from_docx/drawing.rs:171 | private | _no doc comment_ | impl ScopeMap |
| `Pending` | src/markdown/from_docx/drawing.rs:224 | private | _no doc comment_ | impl Pending |
| `Deps` | src/markdown/from_docx/drawing.rs:589 | private | _no doc comment_ | impl Deps<'_> |
| `Drawing` | src/markdown/from_docx/drawing.rs:1549 | private | _no doc comment_ | impl Drawing |
| `Drawing` | src/markdown/from_docx/drawing.rs:1814 | private | Write `drawing` into `package` as content of `part`: the XML with fresh relationship ids, and every dependent part (its own relationships and parts too). | impl Drawing |
| `State` | src/markdown/from_docx/header.rs:2003 | private | _no doc comment_ | impl State<'_> |
| `Extracted` | src/markdown/from_docx/media.rs:24 | private | _no doc comment_ | impl Extracted |
| `<ConvertError as std::fmt::Display>` | src/markdown/from_docx/mod.rs:70 | private | _no doc comment_ | impl std::fmt::Display for ConvertError |
| `Styles` | src/markdown/from_docx/mod.rs:1808 | private | _no doc comment_ | impl Styles |
| `Numbering` | src/markdown/from_docx/mod.rs:2052 | private | _no doc comment_ | impl Numbering |
| `Writer` | src/markdown/from_docx/mod.rs:2534 | private | _no doc comment_ | impl Writer<'_> |
| `Package` | src/markdown/from_docx/ooxml.rs:42 | private | _no doc comment_ | impl<'a> Package<'a> |
| `Rels` | src/markdown/from_docx/ooxml.rs:244 | private | _no doc comment_ | impl Rels |
| `Element` | src/markdown/from_docx/ooxml.rs:317 | private | _no doc comment_ | impl Element |
| `Inline` | src/markdown/from_docx/ooxml.rs:641 | private | _no doc comment_ | impl Inline |
| `Media` | src/markdown/from_docx/ooxml.rs:1184 | private | _no doc comment_ | impl Media |
| `Blocks` | src/markdown/from_docx/ooxml.rs:1257 | private | _no doc comment_ | impl Blocks |
| `ListIndent` | src/markdown/from_docx/ooxml.rs:1365 | private | _no doc comment_ | impl ListIndent |
| `Parser` | src/markdown/math.rs:791 | private | _no doc comment_ | impl Parser |
| `Writer` | src/markdown/math.rs:1672 | private | _no doc comment_ | impl Writer<'_> |
| `TrackChanges` | src/markdown/mod.rs:87 | private | _no doc comment_ | impl TrackChanges |
| `PageSize` | src/markdown/mod.rs:118 | private | _no doc comment_ | impl PageSize |
| `<DocxOptions as Default>` | src/markdown/mod.rs:220 | private | _no doc comment_ | impl Default for DocxOptions<'_> |
| `<DocxOptions as std::fmt::Debug>` | src/markdown/mod.rs:290 | private | _no doc comment_ | impl std::fmt::Debug for DocxOptions<'_> |
| `<MarkdownOptions as Default>` | src/markdown/mod.rs:346 | private | _no doc comment_ | impl Default for MarkdownOptions |
| `Select` | src/markdown/mod.rs:414 | private | _no doc comment_ | impl Select |
| `<ReadOptions as Default>` | src/markdown/mod.rs:712 | private | _no doc comment_ | impl Default for ReadOptions |
| `<TablesError as std::fmt::Display>` | src/markdown/mod.rs:886 | private | _no doc comment_ | impl std::fmt::Display for TablesError |
| `<TablesError as std::error::Error>` | src/markdown/mod.rs:895 | private | _no doc comment_ | impl std::error::Error for TablesError |
| `<MarkdownError as std::fmt::Display>` | src/markdown/mod.rs:1139 | private | _no doc comment_ | impl std::fmt::Display for MarkdownError |
| `<MarkdownError as std::error::Error>` | src/markdown/mod.rs:1149 | private | _no doc comment_ | impl std::error::Error for MarkdownError |
| `<PartRelate as Relate>` | src/markdown/package.rs:1694 | private | _no doc comment_ | impl Relate for PartRelate<'_, '_, '_> |
| `Block` | src/markdown/patch.rs:77 | private | _no doc comment_ | impl Block |
| `<RedlineOptions as std::fmt::Debug>` | src/markdown/redline.rs:66 | private | _no doc comment_ | impl std::fmt::Debug for RedlineOptions<'_> |
| `<Locator as fmt::Display>` | src/markdown/unified.rs:83 | private | _no doc comment_ | impl fmt::Display for Locator |
| `Patch` | src/markdown/unified.rs:161 | private | _no doc comment_ | impl Patch |
| `<Patch as fmt::Display>` | src/markdown/unified.rs:188 | private | _no doc comment_ | impl fmt::Display for Patch |
| `Search` | src/markdown/unified.rs:422 | private | _no doc comment_ | impl<'a> Search<'a> |
| `Edit` | src/markdown/unified.rs:606 | private | _no doc comment_ | impl Edit |
| `Block` | src/markdown/unified.rs:1030 | private | _no doc comment_ | impl Block |
| `CellsAt` | src/markdown/view_notes.rs:580 | private | _no doc comment_ | impl CellsAt |
| `IdLine` | src/markdown/view_notes.rs:1414 | private | _no doc comment_ | impl IdLine |
| `PageSetup` | src/markdown/view_notes.rs:2163 | private | _no doc comment_ | impl PageSetup |
| `ParaFormat` | src/markdown/view_stories.rs:90 | private | _no doc comment_ | impl ParaFormat |
| `Pool` | src/markdown/view_stories.rs:226 | private | _no doc comment_ | impl Pool |
| `Marks` | src/markdown/view_stories.rs:429 | private | _no doc comment_ | impl Marks<'_> |
| `Stories` | src/markdown/view_stories.rs:815 | private | _no doc comment_ | impl Stories |
| `SectionStories` | src/markdown/view_stories.rs:1536 | private | _no doc comment_ | impl SectionStories |
| `Given` | src/markdown/view_styles.rs:768 | private | _no doc comment_ | impl Given |
| `Anchor` | src/markdown/write.rs:1705 | private | _no doc comment_ | impl Anchor |
| `Writer` | src/markdown/write.rs:1932 | private | _no doc comment_ | impl<'o, 'a> Writer<'o, 'a> |
| `Slugger` | src/markdown/write.rs:5678 | private | _no doc comment_ | impl Slugger |
| `Writer` | src/markdown/write.rs:5762 | private | _no doc comment_ | impl Writer<'_, '_> |
| `TableGeometry` | src/markdown/xml.rs:243 | private | _no doc comment_ | impl TableGeometry |
| `Spacing` | src/markdown/xml.rs:283 | private | _no doc comment_ | impl Spacing |
| `Document` | src/markdown/xml.rs:546 | private | _no doc comment_ | impl Document |
| `Context` | src/markdown/xml.rs:692 | private | _no doc comment_ | impl Context<'_> |
| `W` | src/namespaces.rs:194 | private | _no doc comment_ | impl W |
| `PT` | src/namespaces.rs:261 | private | ── PowerTools correlation attribute names ──────────────────────────────────── _(from a code comment)_ | impl PT |
| `ZipDefect` | src/opc/archive.rs:364 | private | _no doc comment_ | impl ZipDefect |
| `Relationships` | src/opc/mod.rs:39 | private | _no doc comment_ | impl Relationships |
| `<Relationships as Deref>` | src/opc/mod.rs:55 | private | _no doc comment_ | impl Deref for Relationships |
| `<Relationships as DerefMut>` | src/opc/mod.rs:63 | private | _no doc comment_ | impl DerefMut for Relationships |
| `ZipEntry` | src/opc/mod.rs:493 | private | _no doc comment_ | impl ZipEntry<'_> |
| `PartFs` | src/opc/mod.rs:558 | private | _no doc comment_ | impl PartFs |
| `<Zip32Limit as fmt::Display>` | src/opc/zip32.rs:46 | private | _no doc comment_ | impl fmt::Display for Zip32Limit |
| `<Zip32Limit as std::error::Error>` | src/opc/zip32.rs:52 | private | _no doc comment_ | impl std::error::Error for Zip32Limit |
| `Snapshot` | src/perf.rs:168 | private | _no doc comment_ | impl Snapshot |
| `<PublishError as std::fmt::Display>` | src/publication.rs:49 | private | _no doc comment_ | impl std::fmt::Display for PublishError |
| `<PublishError as std::error::Error>` | src/publication.rs:64 | private | _no doc comment_ | impl std::error::Error for PublishError |
| `OutputRoot` | src/publication.rs:153 | private | _no doc comment_ | impl OutputRoot |
| `StagedFile` | src/publication.rs:297 | private | _no doc comment_ | impl StagedFile |
| `<StagedFile as Drop>` | src/publication.rs:414 | private | _no doc comment_ | impl Drop for StagedFile |
| `<FsBackend as Backend>` | src/publication.rs:419 | private | _no doc comment_ | impl Backend for FsBackend<'_> |
| `Artifact` | src/result.rs:183 | private | _no doc comment_ | impl Artifact |
| `Context` | src/result.rs:237 | private | _no doc comment_ | impl Context |
| `NextAction` | src/result.rs:276 | private | _no doc comment_ | impl NextAction |
| `Error` | src/result.rs:344 | private | _no doc comment_ | impl Error |
| `Source` | src/result.rs:460 | private | _no doc comment_ | impl Source |
| `Runtime` | src/result.rs:498 | private | _no doc comment_ | impl Runtime |
| `<Verification as Default>` | src/result.rs:531 | private | _no doc comment_ | impl Default for Verification |
| `Counts` | src/result.rs:596 | private | _no doc comment_ | impl Counts |
| `<ResultDataError as std::fmt::Display>` | src/result.rs:672 | private | _no doc comment_ | impl std::fmt::Display for ResultDataError |
| `<ResultDataError as std::error::Error>` | src/result.rs:693 | private | _no doc comment_ | impl std::error::Error for ResultDataError |
| `ResultData` | src/result.rs:695 | private | _no doc comment_ | impl ResultData |
| `Envelope` | src/result.rs:816 | private | _no doc comment_ | impl Envelope |
| `<StreamHeader as From>` | src/result.rs:965 | private | _no doc comment_ | impl From<&Envelope> for StreamHeader |
| `ProgressEvent` | src/result.rs:988 | private | _no doc comment_ | impl ProgressEvent |
| `<ProgressEvent as From>` | src/result.rs:1000 | private | _no doc comment_ | impl From<serde_json::Value> for ProgressEvent |
| `<serde_json::Value as From>` | src/result.rs:1006 | private | _no doc comment_ | impl From<ProgressEvent> for serde_json::Value |
| `<StreamError as std::fmt::Display>` | src/result.rs:1075 | private | _no doc comment_ | impl std::fmt::Display for StreamError |
| `<StreamError as std::error::Error>` | src/result.rs:1094 | private | _no doc comment_ | impl std::error::Error for StreamError |
| `ExitStatus` | src/result.rs:1180 | private | _no doc comment_ | impl ExitStatus |
| `<u8 as From>` | src/result.rs:1200 | private | _no doc comment_ | impl From<ExitStatus> for u8 |
| `RevisionIds` | src/revision_processor.rs:3805 | private | _no doc comment_ | impl RevisionIds |
| `Parsed` | src/revision_processor/comments.rs:40 | private | _no doc comment_ | impl Parsed |
| `Slot` | src/revision_processor/style_records.rs:109 | private | _no doc comment_ | impl Slot |
| `Levels` | src/revision_processor/style_records.rs:279 | private | _no doc comment_ | impl Levels |
| `<ScrubOptions as Default>` | src/scrub.rs:68 | private | _no doc comment_ | impl Default for ScrubOptions |
| `<ScrubError as std::fmt::Display>` | src/scrub.rs:89 | private | _no doc comment_ | impl std::fmt::Display for ScrubError |
| `<ScrubError as std::error::Error>` | src/scrub.rs:98 | private | _no doc comment_ | impl std::error::Error for ScrubError |
| `Part` | src/scrub.rs:189 | private | _no doc comment_ | impl Part |
| `ProtectionEdit` | src/settings.rs:160 | private | _no doc comment_ | impl ProtectionEdit |
| `SettingsRequest` | src/settings.rs:211 | private | _no doc comment_ | impl SettingsRequest |
| `UriSite` | src/strict_translation.rs:529 | private | _no doc comment_ | impl UriSite |
| `<UnifiedOptions as Default>` | src/text_diff.rs:31 | private | _no doc comment_ | impl Default for UnifiedOptions |
| `<TextOptions as Default>` | src/text_diff.rs:82 | private | _no doc comment_ | impl Default for TextOptions |
| `Piece` | src/text_diff.rs:471 | private | _no doc comment_ | impl Piece |
| `Finding` | src/validate.rs:129 | private | _no doc comment_ | impl Finding |
| `<ValidateError as fmt::Display>` | src/validate.rs:252 | private | _no doc comment_ | impl fmt::Display for ValidateError |
| `<ValidateError as std::error::Error>` | src/validate.rs:265 | private | _no doc comment_ | impl std::error::Error for ValidateError |
| `<ValidateError as From>` | src/validate.rs:267 | private | _no doc comment_ | impl From<AdmissionError> for ValidateError |
| `<ValidateError as From>` | src/validate.rs:273 | private | _no doc comment_ | impl From<ChangeError> for ValidateError |
| `<ValidateError as From>` | src/validate.rs:279 | private | _no doc comment_ | impl From<InspectError> for ValidateError |
| `<ValidateError as From>` | src/validate.rs:285 | private | _no doc comment_ | impl From<crate::opc::OpcError> for ValidateError |
| `Finding` | src/validate.rs:502 | private | _no doc comment_ | impl Finding |
| `Anchors` | src/validate.rs:557 | private | _no doc comment_ | impl<'a> Anchors<'a> |
| `Part` | src/validate.rs:632 | private | _no doc comment_ | impl Part |
| `<Finding as From>` | src/validate.rs:1108 | private | _no doc comment_ | impl From<crate::audit::AuditFinding> for Finding |
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
| `NsScope` | src/xmllinq/parse.rs:188 | private | _no doc comment_ | impl NsScope |
| `TagPolicy` | src/xmllinq/parse.rs:244 | private | _no doc comment_ | impl TagPolicy |
| `Parser` | src/xmllinq/parse.rs:444 | private | _no doc comment_ | impl<'a> Parser<'a> |
| `Scope` | src/xmllinq/serialize.rs:210 | private | _no doc comment_ | impl Scope<'static> |
| `Scope` | src/xmllinq/serialize.rs:224 | private | _no doc comment_ | impl<'a> Scope<'a> |
| `<String as XmlBuf>` | src/xmllinq/serialize.rs:446 | private | _no doc comment_ | impl XmlBuf for String |
| `HashXmlBuf` | src/xmllinq/serialize.rs:471 | private | _no doc comment_ | impl HashXmlBuf |
| `<HashXmlBuf as XmlBuf>` | src/xmllinq/serialize.rs:501 | private | _no doc comment_ | impl XmlBuf for HashXmlBuf |
