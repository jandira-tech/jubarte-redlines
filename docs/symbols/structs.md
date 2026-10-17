# Structs

497 items (non-test). Part of the [symbol manifest](README.md).

| Name | Defined at | Vis | Does | Detail |
|---|---|---|---|---|
| `InputLimits` | src/admission.rs:42 | pub | Resource budgets. | fields: max_compressed_bytes,max_entries,max_part_bytes,max_uncompressed_bytes,max_xml_depth |
| `InputLimitOverrides` | src/admission.rs:93 | pub | Field-by-field overrides of an [`InputLimits`] budget: the shape the Python and WASM bindings take as a JSON object. | fields: max_compressed_bytes,max_entries,max_part_bytes,max_uncompressed_bytes,max_xml_depth |
| `AdmissionError` | src/admission.rs:247 | pub | A refused package: the kind and what exactly was refused. | fields: kind,message |
| `AdmittedPackage` | src/admission.rs:279 | pub | Facts about an admitted package. | fields: entries,inflated_bytes,main_part |
| `AppendOptions` | src/append.rs:67 | pub | How [`append_documents`] joins two documents. | fields: section_break,keep_sections,comments |
| `Appended` | src/append.rs:79 | pub | The appended package and what could not be carried. | fields: docx,warnings |
| `StagedComments` | src/append.rs:542 | private | B's comments that carried content anchors, under their new ids. | fields: container,src_part,b_root,state |
| `NoteKind` | src/append.rs:633 | private | One kind of note: `(element, part file, relationship suffix, content type)`. | fields: element,file,rel,content_type |
| `CarriedNotes` | src/append.rs:656 | private | B's notes of one kind, staged for A's notes part. | fields: dest_part,dest_doc,dest_root,src_part,staged |
| `ListCopy` | src/append.rs:1097 | private | B's lists to copy, and what their copies need rewritten. | fields: b_abstracts,b_nums,style_map,abstracts,nums,used_nsids |
| `AuditFinding` | src/audit.rs:58 | pub | One audit finding. | fields: code,rule_set,severity,location,message |
| `AuditReport` | src/audit.rs:73 | pub | Findings plus what ran to produce them. | fields: findings,rules,layout |
| `Story` | src/audit.rs:304 | private | One parsed part with its addressable paragraphs. | fields: id,part,dom,root,paragraphs |
| `StyleFacts` | src/audit.rs:331 | private | A style's facts the rules read. | fields: name,based_on,numbering,font,size,lang |
| `Package` | src/audit.rs:341 | private | The package, read once. | fields: main,stories,styles_part,styles,default_paragraph_style,defaults_font,defaults_size,defaults_lang,theme_font_lang |
| `Field` | src/audit.rs:824 | private | One field's instruction and cached result. | fields: story,node,instruction,separated,result,keyword,location |
| `Output` | src/bin/jubarte.rs:206 | private | One task's `-o`: where, as what, and how a Word result becomes it. | fields: kind,path,force,quiet,input,view,source,render |
| `ConvertJob` | src/bin/jubarte.rs:433 | private | Which artifacts `convert` writes and where. | fields: file,bytes,output,force,font_report,options,to,dpi,report,pages,fail_on_substitution,quiet |
| `ConvertFailure` | src/bin/jubarte.rs:477 | private | Why `convert` failed, and the exit status that says so. | fields: code,message |
| `DiffRenderJob` | src/bin/jubarte.rs:669 | private | What `visual-diff` compares and where it writes. | fields: a,b,out_dir,dpi,json,overlay,force |
| `Summary` | src/bin/jubarte.rs:695 | private | `diff.json`. | fields: a,b,dpi,a_pages,b_pages,changed,pages |
| `EditJob` | src/bin/jubarte.rs:897 | private | Options for `edit` and `add`. | fields: file,plan,explicit_output,target,mode,dry_run,force,pdf,png,dpi,revisions,quiet |
| `EditFlags` | src/bin/jubarte.rs:1786 | private | One `edit` or `add` invocation, parsed. | fields: verb,file,plan,plan_default,consume,operations,options,dry_run,pdf,png,dpi,revisions,quiet |
| `Job` | src/bin/jubarte.rs:1993 | private | A fully-resolved comparison job (positional/named merged). | fields: original,modified,output,read,view_given,render,reference,critic,resource_path,author,date,force,quiet,detail_threshold,powertools_faithful,no_paragraph_merge |
| `Input` | src/bin/jubarte.rs:2207 | private | A document read for comparison: Word bytes or Markdown text. | fields: bytes,markdown |
| `DiffJob` | src/bin/jubarte.rs:2336 | private | Options for `diff`: text only (D2). | fields: old,new,output,from,force,settings,patch,quiet |
| `ValidateJob` | src/bin/jubarte.rs:2777 | private | One `jubarte validate` run. | fields: file,json,rules,strict,repair,original,author,force,quiet |
| `Capabilities` | src/capabilities.rs:30 | pub | Machine-readable capability manifest reported by `jubarte capabilities --json` and `capabilities::capabilities_json`. | fields: schema_version,engine_version,runtime,operations,edit_plan_versions,edit_operations,limits,audit_rules,validate_rules,formats,ids |
| `Ids` | src/capabilities.rs:70 | pub | The id spellings the CLI, plans and selectors take, by kind ([`crate::ids`]). | fields: change,comment,comment_text,paragraph,table,control,note |
| `Operations` | src/capabilities.rs:101 | pub | Per-operation availability in this build. | fields: compare,accept_revisions,reject_revisions,selective_revisions,revision_records,pdf,png,inspect_body,markdown,edit,comments,markdown_to_docx,markdown_diff,patch,diff_render,page_ranges,comment_threads,edit_keeps_revisions,append,content_controls,validate,repair,fields,scrub,audit |
| `Limits` | src/capabilities.rs:260 | pub | Documented scope limits an agent must plan around. | fields: stories,plain_text_runs,refuses_opaque_ranges,reads_legacy_doc,input |
| `InputBudget` | src/capabilities.rs:295 | pub | Package budgets as the manifest reports them, in the same units [`crate::admission::InputLimits`] uses. | fields: max_compressed_bytes,max_entries,max_part_bytes,max_uncompressed_bytes,max_xml_depth |
| `Change` | src/changes.rs:55 | pub | One tracked change. | fields: id,r#ref,kind,target,author,date,text,paragraph,move_name,move_side,inside |
| `ChangeFilter` | src/changes.rs:101 | pub | Which changes to resolve. | fields: ids,authors,kinds |
| `Carrier` | src/changes.rs:421 | private | A revision element: a listed change, or a move range marker. | fields: node,group,change |
| `Cli` | src/cli.rs:28 | pub | Read, edit, compare and render Word documents. | fields: command,quiet,compare |
| `ReadArgs` | src/cli.rs:56 | pub | Options of `read`. | fields: track_changes,comments,dates,no_page_markers,paragraphs,head,tail,changed,by,json,tables |
| `EditOptions` | src/cli.rs:157 | pub | Options `edit` and `add` share. | fields: author,datetime,suggesting_mode,editing_mode,existing_revisions,output,out_dir,force |
| `CompareArgs` | src/cli.rs:525 | pub | Inputs and settings shared by explicit and shorthand comparisons. | fields: original_pos,modified_pos,original,modified,output,author,datetime,force,detail_threshold,mode,powertools_faithful,no_paragraph_merge,reference_doc,critic,resource_path,render,read,view_given |
| `ScrubSelection` | src/cli.rs:1294 | pub | What `jubarte scrub` removes; everything when no flag is given. | fields: author_alias,rsids,docprops,comments |
| `Retired` | src/cli.rs:1747 | pub | A spelling 0.13 still accepts, with a note, and 0.14 removes. | fields: old,now,on |
| `Spelled` | src/cli.rs:1877 | private | One option as `argv` spells it: `-p`, `--location` (a long's `=value` split off). | fields: name |
| `RenderArgs` | src/cli.rs:2099 | pub | How `-o FILE.pdf` and `-o FILE.png` are rendered (`convert`, `diff`); refused with any other output. | fields: revisions,revision_palette,dpi,move_comments,comment_notes,changed_pages,pages |
| `MarkdownArgs` | src/cli.rs:2171 | pub | `convert`'s format and Markdown flags. | fields: from,to,track_changes,no_critic,reference_doc,resource_path,author,datetime,page,tracking,scholar,no_page_markers,commonmark |
| `Selection` | src/cli.rs:2296 | pub | Filters selecting individual tracked changes. | fields: ids,authors,kinds |
| `CommentRecord` | src/comments.rs:44 | pub | One comment with its thread position and where it sits. | fields: id,r#ref,author,initials,date,text,parent,done,paragraph,last_paragraph,anchor_text,before,after |
| `Anchors` | src/comments.rs:227 | private | Where each comment's markers sit, story by story. | fields: stories,starts,ends,references |
| `NewComment` | src/comments.rs:386 | pub(crate) | A new comment for [`CommentFamily::add`]. | fields: id,author,date,initials,text,parent |
| `Part` | src/comments.rs:398 | private | One parsed part of the family. | fields: name,doc,root |
| `Meta` | src/comments.rs:405 | private | What the extended parts say about one comment. | fields: node,parent,done,ext_row,id_row,cex_row |
| `CommentFamily` | src/comments.rs:416 | pub(crate) | The comment part family of one package, loaded for reading and editing (module docs). | fields: dom,parts,meta,removed_authors,added_authors |
| `AtomHash` | src/comparer/atoms.rs:43 | pub | Inline, `Copy` atom content hash — the raw 20-byte SHA-1 digest. |  |
| `FormatChangeInfo` | src/comparer/atoms.rs:114 | pub | Old/new run or paragraph properties (as DOM nodes) and the friendly names of properties that changed. | fields: old_run_properties,new_run_properties,old_para_properties,changed_properties |
| `AtomBlock` | src/comparer/atoms.rs:129 | pub | A maximal run of same-status atoms (by index into the flattened atom list). | fields: atoms,start_index |
| `ComparisonUnitAtom` | src/comparer/atoms.rs:147 | pub | One single-character run / content leaf, with its ancestor chain (outermost → leaf, excluding body) and content hash. | fields: correlation_status,sha1_hash,content_element,ancestor_elements,correlated_sha1_hash,content_element_before,comparison_unit_atom_before,ancestor_unids,rev_track_element,move_group_id,move_name,format_change |
| `Sha1Keyed` | src/comparer/atoms.rs:229 | pub | A unit's content hash with its two cached LCS fingerprints. | fields: hash,key,key128 |
| `ComparisonUnitWord` | src/comparer/atoms.rs:282 | pub | Port of `ComparisonUnitWord` — a word is a run of atoms; its hash is the SHA-1 of the concatenation of its atoms' hashes (hex digests). | fields: correlation_status,contents,sha1 |
| `ComparisonUnitGroup` | src/comparer/atoms.rs:319 | pub | Port of `ComparisonUnitGroup` — paragraph/table/row/cell/textbox. | fields: correlation_status,group_type,contents,level,sha1,correlated_sha1_hash,structure_sha1_hash,atom_count_memo |
| `CorrelatedSequence` | src/comparer/atoms.rs:493 | pub | Port of `CorrelatedSequence` — a run of comparison units that share a correlation status, as emitted by [`super::lcs_table::mark_rows_as_deleted_or_inserted`] and consumed by [`super::produce::produce_new_wml_markup_from_correlated_sequence`]. | fields: correlation_status,com_units_1,com_units_2 |
| `WmlComparerRevision` | src/comparer/atoms.rs:547 | pub | Port of `WmlComparerRevision` (full shape — D.2). | fields: revision_type,text,author,date,content_element,revision_element,part_name,move_group_id,is_move_source,format_change |
| `Marker` | src/comparer/bookmarks.rs:45 | private | One source bookmark: its name and `[start, end)` character interval in the source's text projection. | fields: name,start,end,leads,start_trails,end_leads,cols,in_sdt |
| `Event` | src/comparer/comments.rs:235 | private | _no doc comment_ | fields: offset,kind,id,seq |
| `Range` | src/comparer/comments.rs:244 | private | A comment's [start, end) character interval in its source projection. | fields: id,start,end,seq |
| `Seg` | src/comparer/comments.rs:440 | pub(super) | A counted text leaf in the merged body: the `w:t`/`w:delText` element, its run, and its [start, start+len) character interval in the projection. | fields: leaf,run,start,len |
| `Tok` | src/comparer/cross_para.rs:51 | private | _no doc comment_ | fields: kind,key,chars,unit |
| `Entry` | src/comparer/cross_para.rs:73 | private | One stream entry: a unit of sequence `seq`, or a barrier (`item` empty) standing for that whole non-flattenable sequence. | fields: item,seq,plain |
| `Para` | src/comparer/cross_para.rs:80 | private | One side's paragraph: entry indices of its tokens and of its mark. | fields: entries,mark,plain |
| `Stream` | src/comparer/cross_para.rs:102 | private | The flattened sequence list, its paragraphs and their alignment. | fields: dom,settings,entries,lparas,rparas,blocks,para_of_l,para_of_r |
| `Region` | src/comparer/cross_para.rs:115 | private | One region the segmenter may re-stream: member paragraphs per side and the word-matched pairs among them (member ordinals). | fields: left,right,pairs,run_ends_story |
| `Bag` | src/comparer/cross_para.rs:836 | private | A paragraph's token multisets (`MatchKeyBag`). | fields: counts,total,word_count,pairing,pairing_count,trimmed,content |
| `Cell` | src/comparer/cross_para.rs:1320 | private | One output paragraph: a slice of at most one left and one right member paragraph (member, start, len) and the mark that closes it. | fields: left,right,mark,mark_left,mark_right |
| `Unit` | src/comparer/cross_para.rs:1331 | private | A content-anchor unit: a word, or a compound (words joined by single non-whitespace separators, "right-aligned"). | fields: start,len,key,chars,first,last |
| `RevStamp` | src/comparer/finalize.rs:2764 | private | Who and when a rebuilt revision run is attributed to, and the run properties its text copies. | fields: author,date,sample_rpr |
| `LadderSources` | src/comparer/finalize.rs:14864 | pub | Source main parts from before accept-before-diff. | fields: original_xml,revised_xml,settings |
| `SuffixHit` | src/comparer/finalize.rs:14979 | private | _no doc comment_ | fields: full,cut,author,date |
| `RevisedSpacing` | src/comparer/finalize.rs:15359 | private | _no doc comment_ | fields: signature,node |
| `NotesSet` | src/comparer/footnotes.rs:648 | pub | The three roots the rectifier needs for one note kind (footnote or endnote): the original `before`, the comparison `after`, and the freshly-built `with_revisions` (where the rectifier keeps separators and re-adds the renumbered referenced definitions). | fields: before,after,with_revisions |
| `RectifyPlan` | src/comparer/footnotes.rs:740 | private | _no doc comment_ | fields: refs,defs |
| `NoteDefs` | src/comparer/footnotes.rs:1042 | private | The definitions one note reference needs, found in the notes parts. | fields: before,after |
| `U64IdentityHasher` | src/comparer/lcs.rs:37 | pub(crate) | Passthrough hasher for `u64`-keyed maps whose keys are already well-mixed FNV-1a fingerprints of SHA-1 hex digests (`ComparisonUnit::sha1_key`). |  |
| `TaggedAtom` | src/comparer/lcs.rs:59 | pub | One atom tagged with its correlation status, ready for the produce step. | fields: atom,status |
| `CorrelatedHashRun` | src/comparer/lcs.rs:8858 | private | M4.C.11 — `ProcessCorrelatedHashes` (WmlComparer.ts :7184): pre-correlate runs of groups (Paragraph/Table/Row) by `pt:CorrelatedSHA1Hash`, emitting one `Unknown` per matched group, with before/after `Deleted`/`Inserted`/`Unknown`. | fields: left_start,right_start,len |
| `NotesContext` | src/comparer/mod.rs:144 | pub | Notes-part roots the reference-driven note processing reads definitions from and writes redlined definitions into. | fields: fn_before,fn_after,en_before,en_after,fn_with_revisions,en_with_revisions |
| `WmlComparerSettings` | src/comparer/mod.rs:1350 | pub | Comparer configuration (port of `WmlComparerSettings`, defaults verified against WmlComparer.ts:415-457). | fields: word_separators,author_for_revisions,date_time_for_revisions,detail_threshold,case_insensitive,conflate_breaking_and_nonbreaking_spaces,starting_id_for_footnotes_endnotes,detect_moves,simplify_move_markup,move_similarity_threshold,move_minimum_word_count,detect_format_changes,merge_replaced_ |
| `CompareContext` | src/comparer/mod.rs:1622 | pub | Optional log holder used by the comparison pipeline. | fields: settings,log |
| `BlockText` | src/comparer/moves.rs:480 | private | Per-block precomputed text features, shared by every pair comparison in `detect_moves_memoized` (computed once per block instead of O(del × ins)). | fields: text,collapsed,words,tokens,chars |
| `RelationshipRow` | src/comparer/parts.rs:28 | pub | One `<Relationship>` row from an OPC `.rels` part. | fields: id,rel_type,target,external |
| `RecursionInfo` | src/comparer/tables.rs:169 | pub | One `RecursionElements` (WmlComparer.ts :9074) entry: an element the atomizer recurses into, plus the property children that `super::atomize::coalesce_recurse` re-attaches structurally after atomization. | fields: element_name,child_property_names |
| `ComparisonLogEntry` | src/comparison_log.rs:27 | pub | One entry in a [`ComparisonLog`]: a severity tag and the message. | fields: code,message |
| `ComparisonLog` | src/comparison_log.rs:36 | pub | Accumulates log entries during a comparison run, in insertion order. | fields: entries |
| `Fmt` | src/convert/altchunk.rs:192 | private | _no doc comment_ | fields: bold,italic,underline,size,font |
| `Para` | src/convert/altchunk.rs:254 | private | _no doc comment_ | fields: ppr,runs |
| `Cell` | src/convert/altchunk.rs:259 | private | _no doc comment_ | fields: span,wml |
| `Table` | src/convert/altchunk.rs:264 | private | _no doc comment_ | fields: border,rows,depth |
| `Builder` | src/convert/altchunk.rs:271 | private | _no doc comment_ | fields: css,sinks,para,fmts,tables,cell_spans |
| `DiffOptions` | src/convert/diff_render.rs:31 | pub | How [`diff_render`] renders and marks the two documents. | fields: dpi,pdf,overlay |
| `PageDiff` | src/convert/diff_render.rs:55 | pub | How one page differs between the two documents. | fields: index,changed_ratio,bbox,only_in |
| `RenderDiff` | src/convert/diff_render.rs:80 | pub | Output of [`diff_render`]. | fields: pages,a,b,overlays,a_report,b_report |
| `RestoreOnDrop` | src/convert/font.rs:212 | private | Puts the previous thread-local value back when dropped, so a scope that unwinds (a panicking conversion caught by a test harness or a long-lived host) cannot leak its state into the next conversion. | fields: slot,prev |
| `FontReportEntry` | src/convert/font.rs:325 | pub | One distinct requested family + style from a conversion (plan Step 2f). | fields: requested,step,physical,bold,italic,synthetic |
| `FaceKey` | src/convert/font.rs:532 | pub(crate) | Physical family + style. | fields: family,bold,italic |
| `Face` | src/convert/font.rs:883 | pub(crate) | Parsed metrics + cmap for one bundled, system-overlaid or embedded face. | fields: bytes,buzz,pdf_name,upem,descent,line_height,line_descent,paint_ascent,strike_pos,strike_size,east_asian,serif,avg_char_width,bbox,widths,cmap,symbol_slot,plans,shaped |
| `Catalogue` | src/convert/font.rs:1417 | private | Process-lifetime catalogue (one slot per `FaceId::all()` member). | fields: faces |
| `Fonts` | src/convert/font.rs:1452 | pub(crate) | Bundled catalogue plus per-document embedded faces (`.odttf`). | fields: extra,extra_index,catalogue_slots |
| `FaceSource` | src/convert/font.rs:3063 | private | Where an installed face came from: its file and, inside a collection, its index. | fields: path,index |
| `IndexedFace` | src/convert/font.rs:3140 | private | _no doc comment_ | fields: style,source,stamp |
| `IndexEntry` | src/convert/font.rs:3149 | private | One looked-up family: the folders its search listed, stamped, and the faces it found (none is an answer too). | fields: dirs,faces |
| `FontIndex` | src/convert/font.rs:3177 | private | _no doc comment_ | fields: path,entries |
| `FontEntry` | src/convert/font_table.rs:28 | pub(crate) | One `w:font` row, including embed rels (`w:embedRegular` etc.). | fields: name,alt_name,family,usb,charset,embedded |
| `FontTable` | src/convert/font_table.rs:41 | pub(crate) | Keyed by the exact `w:name` (no normalisation). | fields: map |
| `MathConstants` | src/convert/math_font.rs:15 | pub(crate) | The MATH constants this step reads. | fields: script_percent,script_script_percent,axis_height,fraction_rule_thickness,fraction_num_shift_up,fraction_num_display_shift_up,fraction_denom_shift_down,fraction_denom_display_shift_down,fraction_num_gap_min,fraction_num_display_gap_min,fraction_denom_gap_min,fraction_denom_display_gap_min,rad |
| `Glyph` | src/convert/math_font.rs:39 | pub(crate) | One glyph's design-space box. | fields: advance,y_min,y_max |
| `AssemblyPart` | src/convert/math_font.rs:58 | pub(crate) | One piece of a vertical assembly. | fields: gid,full_advance,extender |
| `MathFont` | src/convert/math_font.rs:78 | pub(crate) | Font bytes plus the constants laid out against them. | fields: bytes,index,upem,has_math,c |
| `MathFontGuard` | src/convert/math_font.rs:187 | pub(crate) | Installs the conversion's math font, restoring the previous one on drop. |  |
| `MathGlyph` | src/convert/math_layout.rs:21 | pub(crate) | One positioned glyph. | fields: text,gid,x,y,size |
| `MathRule` | src/convert/math_layout.rs:31 | pub(crate) | A horizontal rule. | fields: x,y_top,w,thickness |
| `MathBox` | src/convert/math_layout.rs:40 | pub(crate) | A laid-out equation: width, ascent, descent, glyphs, and rules. | fields: width,ascent,descent,glyphs,rules |
| `MetaText` | src/convert/metafile.rs:39 | pub(crate) | A string a metafile writes (`META_EXTTEXTOUT` / `META_TEXTOUT`), placed as fractions of the picture: `x` from its left, `y` from its top, and `size` (the font's `lfHeight`) of its height. | fields: text,x,y,size,cell,family,bold,italic,color,align,aspect |
| `Canvas` | src/convert/metafile.rs:66 | private | _no doc comment_ | fields: w,h,px |
| `Map` | src/convert/metafile.rs:296 | private | _no doc comment_ | fields: org_x,org_y,ext_x,ext_y,w,h |
| `WmfFont` | src/convert/metafile.rs:395 | private | `META_CREATEFONTINDIRECT`'s `LogFont`. | fields: height,weight,italic,face |
| `WmfState` | src/convert/metafile.rs:421 | private | The device context `META_SAVEDC` keeps. | fields: brush,pen,font,text_color,text_align |
| `Xform` | src/convert/metafile.rs:722 | private | EMF logical → device coordinates (SETMAPMODE / window / viewport). | fields: mode,win_org,win_ext,vp_org,vp_ext |
| `RevisionMark` | src/convert/mod.rs:143 | pub | How one kind of tracked change is painted: the ink and the strike / underline it draws. | fields: color,strike,underline |
| `RevisionPalette` | src/convert/mod.rs:155 | pub | The marks for each kind of tracked change (`w:del`, `w:ins`, `w:moveFrom`, `w:moveTo`). | fields: deleted,inserted,moved_from,moved_to |
| `Restore` | src/convert/mod.rs:309 | private | _no doc comment_ |  |
| `PdfOptions` | src/convert/mod.rs:363 | pub | The flags a DOCX → PDF conversion runs with. | fields: compress,revisions,comments,comment_notes,changed_only |
| `ConvertedPdf` | src/convert/mod.rs:408 | pub | Rendered PDF plus the distinct font resolutions for this document. | fields: pdf,font_report |
| `PageText` | src/convert/mod.rs:465 | pub | Text painted on one page, in paint order (header first when present). | fields: index,number,text |
| `RenderReport` | src/convert/mod.rs:479 | pub | Page facts from one layout pass. | fields: page_count,document_pages,pages,fonts |
| `RenderRequest` | src/convert/mod.rs:540 | pub | What [`render`] should produce from its single layout pass. | fields: pdf,png_dpi,pages |
| `Rendered` | src/convert/mod.rs:559 | pub | Output of [`render`]. | fields: pdf,pngs,report |
| `LayoutFacts` | src/convert/mod.rs:742 | pub(crate) | What one layout pass learned about pages, for writing field results back into the package (`crate::fields`). | fields: page_count,bookmark_pages |
| `RunStyle` | src/convert/mod.rs:1009 | private | _no doc comment_ | fields: family,family_ea,family_cs,lang_ea,ea_theme_slot,hint,rtl,family_hansi,size,size_cs,bold,italic,underline,underline_double,underline_wave,strike,strike_double,color,color_auto,highlight,highlight_marker,tint,tint_id,track,scale,caps,small_caps,bold_set,italic_set,hidden,auto_space_de_off,aut |
| `ParaStyle` | src/convert/mod.rs:1184 | private | _no doc comment_ | fields: fmt_rev,mark_run,hrule,align,after,before,before_auto,after_auto,before_spec,after_spec,bidi,widow_control,snap_to_grid,cell_grid,line_mult,line_exact,line_at_least,indent_left,indent_right,indent_first,contextual,sum_spacing,auto_space_de_off,auto_space_dn_off,list_num,style_id,style_name,b |
| `TabStop` | src/convert/mod.rs:1271 | private | _no doc comment_ | fields: pos,align,leader,numbering |
| `PageSetup` | src/convert/mod.rs:1305 | private | _no doc comment_ | fields: width,height,margin_l,margin_r,margin_t,margin_b,top_exact,bottom_exact,header,footer,valign_center,vertical,page_num_start,page_num_fmt,chap_style,chap_sep,balloon_gutter,gutter_at_top,default_tab,borders,col_count,col_space,col_custom,col_w,col_gap,ln_count_by,ln_start,ln_distance,ln_resta |
| `PageBorder` | src/convert/mod.rs:1378 | private | _no doc comment_ | fields: color,width,space,line |
| `PageBorders` | src/convert/mod.rs:1395 | private | _no doc comment_ | fields: top,left,bottom,right,from_page,display,back |
| `NamedStyle` | src/convert/mod.rs:1454 | private | _no doc comment_ | fields: para,run,num_id,ilvl,sets_size,sets_family,sets_ind,sets_jc,sets_spacing,not_para,frame,hidden |
| `ThemeFonts` | src/convert/mod.rs:1488 | private | _no doc comment_ | fields: major,minor,major_ea,minor_ea,major_cs,minor_cs,script_fonts,colors,fill_styles,ln_widths,ln_colors |
| `StyleSheet` | src/convert/mod.rs:1550 | private | _no doc comment_ | fields: defaults,by_id,tables,theme,latent,default_table |
| `TblStyle` | src/convert/mod.rs:1567 | private | _no doc comment_ | fields: para,sets_line,sets_space,sets_ind_jc,run_size,own_size,run_family,cell_mar_lr,cell_mar_tb,first_row_fill,band1_fill,band2_fill,first_row_bold,first_row_italic,first_col_bold,first_col_italic,first_col_fill,last_row_fill,last_col_fill,first_row_color,first_row_borders,first_col_borders,borde |
| `TblBorders` | src/convert/mod.rs:1621 | private | _no doc comment_ | fields: top,bottom,left,right,inside_h,inside_v,color,width,outer_width |
| `CellBorders` | src/convert/mod.rs:1640 | private | Per-cell `w:tcBorders`. | fields: top,bottom,left,right |
| `TblLook` | src/convert/mod.rs:1648 | pub(crate) | _no doc comment_ | fields: first_row,first_col,no_h_band |
| `RawStyle` | src/convert/mod.rs:1654 | private | _no doc comment_ | fields: based,ppr,rpr |
| `Defaults` | src/convert/mod.rs:1660 | private | _no doc comment_ | fields: run,para,page,legacy_compat,punct_squeeze,wp_justify,normal_spacing,normal_run,normal_ind_jc |
| `CommentNote` | src/convert/mod.rs:1823 | private | _no doc comment_ | fields: id,author,initials,label,color,after,resolved,text |
| `TextRun` | src/convert/mod.rs:1842 | private | _no doc comment_ | fields: text,style,field,rev,comments,comment_marker,pageref,ref_name,styleref,char_style,ref_copy_text,rule,footnote_id,note_ref,endnote_id,note_rule,para_gap,hf_para,ends_line,hf_pic_h,list_marker,frame_right,frame_center,hf_cont,hf_tab_wrap,hf_default_tab,checkbox,strut,math_ascii,math |
| `FootnotePara` | src/convert/mod.rs:2018 | private | One paragraph of a `w:footnote` (plan Step 7). | fields: runs,style |
| `FootnoteCatalog` | src/convert/mod.rs:2024 | private | _no doc comment_ | fields: notes,display,separator |
| `TableGeom` | src/convert/mod.rs:2045 | private | _no doc comment_ | fields: row_min,bottom_above,row_exact,row_cant_split,pad_v,width,unstyled,header_rows,table_grid,tbl_ind,mar_l,pref,rules,grid_padded,fixed,float,keep_at_margin,rtl,pct_margins,pct_rules,content_autofit,cell_spacing |
| `SectionChrome` | src/convert/mod.rs:2124 | private | _no doc comment_ | fields: page,header,footer,header_align,footer_align,header_bottom,footer_top,watermark,header_explicit,header_rest,footer_rest,even_and_odd,header_even,header_odd,footer_even,footer_odd,header_images,footer_images,header_boxes,footer_boxes,header_tables,footer_tables,mirror_margins,punct_squeeze,ul |
| `Watermark` | src/convert/mod.rs:2171 | private | _no doc comment_ | fields: text,size,color,rotate_deg |
| `CellPara` | src/convert/mod.rs:2179 | private | _no doc comment_ | fields: runs,images,boxes,style,bookmarks,blank_bookmarks,continued,vertical,squeeze,hang_spaces |
| `TableCell` | src/convert/mod.rs:2207 | private | _no doc comment_ | fields: paras,nested,nested_at,col,colspan,rowspan,fill,fill_explicit,valign_center,valign_bottom,align,pad_l,pad_r,pad_t,pad_b,nowrap,vertical,hide_mark,grid_skip,borders,style_fill |
| `RawCell` | src/convert/mod.rs:2325 | private | _no doc comment_ | fields: paras,nested,nested_at,pref,colspan,vmerge,fill,fill_explicit,valign_center,valign_bottom,align,pad_l,pad_r,pad_t,pad_b,nowrap,vertical,hide_mark,grid_skip,borders |
| `LaidImage` | src/convert/mod.rs:2350 | private | _no doc comment_ | fields: w,h,kind,slot,behind,z,crop,rotate_deg,oval,chrome_align,chrome_lead,chrome_flow,chrome_under_table,chrome_under_text,inset,chrome_leading,chrome_tab_line,chrome_text_under,chrome_above,chrome_drop,chrome_drop_tab,chrome_para,chrome_after,tail_anchor,outline,gap_before,lead_chars,after_text, |
| `LastLine` | src/convert/mod.rs:2441 | private | The last painted body line: its ops start at `ops_start` on page `page` (a count of finished pages), its baseline sits `drop` below its top, and `room` is the measure its text left free (trailing space included, as a following picture sits after it). | fields: page,ops_start,drop,room,trail,align |
| `FirstLine` | src/convert/mod.rs:2453 | private | A paragraph's first painted line, so inline pictures ahead of its text can open it: `line` as for `LastLine`, its ops ending at `ops_end`. | fields: line,ops_end,baseline,tab_led |
| `LaidTextBox` | src/convert/mod.rs:2460 | private | _no doc comment_ | fields: w,h,runs,slot,chart,stroke,fill,line,line_width,geom,reserve_only,behind,z,flip_h,flip_v,tail_end,diag_shapes,text_dx,text_dy,text_anchor,adj,prst,paras,tables,insets,custom,group,chrome_para_top,fit_width,fit_height,raise,frame,vml |
| `GroupChild` | src/convert/mod.rs:2551 | private | One shape of a group: its box as fractions of the group's box (x, y from the top-left, w, h) and its own geometry. | fields: frac,shape |
| `DiagShape` | src/convert/mod.rs:2566 | private | _no doc comment_ | fields: x,y,w,h,fill,stroke,label,round |
| `ChartData` | src/convert/mod.rs:2589 | private | _no doc comment_ | fields: title,cats,series,names,colors,legend,kind,indexed,x_indexed,n_cats,x_scaling,y_scaling |
| `PlaceCtx` | src/convert/mod.rs:2764 | private | Page/emit context for `resolve_anchor` (xml 3.4). | fields: page_w,page_h,margin_l,margin_r,margin_t,margin_b,column_x,para_top,line_top,cursor_x |
| `Placement` | src/convert/mod.rs:2789 | private | _no doc comment_ | fields: x,y,w,h,wrap |
| `AnchorSpec` | src/convert/mod.rs:2798 | private | Word `wp:anchor` / VML `mso-position-*` inputs for `resolve_anchor`. | fields: w,h,h_from,h_align,h_off,v_from,v_align,v_off,wrap |
| `FmtScheme` | src/convert/mod.rs:3383 | private | _no doc comment_ | fields: fills,widths,lines |
| `NumLevel` | src/convert/mod.rs:5450 | private | _no doc comment_ | fields: fmt,text,start,left,hanging,family,suff_nothing,jc_right,jc,tab_stops,size,underline,bold,italic |
| `Numbering` | src/convert/mod.rs:5477 | private | _no doc comment_ | fields: instances,levels,counters,starts,restarts,is_lgl,lgl_override,pic_bytes,pic_extent,lvl_pic,lvl_pstyle,orig,old_label |
| `WalkCtx` | src/convert/mod.rs:7756 | private | _no doc comment_ | fields: pkg,main,sheet,sects,authors,comments,open_comments |
| `CommentRec` | src/convert/mod.rs:7769 | private | _no doc comment_ | fields: author,initials,text,resolved,label |
| `EndnoteBag` | src/convert/mod.rs:8505 | private | Endnotes collected during the body walk so `endnotePr/@w:pos=sectEnd` can flush at the section boundary (plan.md Step 10 F). | fields: ndom,by_id,separator,pending,seen |
| `ParityHf` | src/convert/mod.rs:8868 | private | A section's picked header/footer plus the even/odd split both `section_chrome` and `first_section_hf` derive from them: the odd part only exists when an even part does. | fields: header,footer,even_and_odd,header_odd,footer_odd |
| `TableParaSpacing` | src/convert/mod.rs:11218 | private | A cell's table-style paragraph properties, and whether the table style itself sets its spacing / line (else they stand in for Normal's). | fields: para,sets,sets_ind_jc |
| `HRule` | src/convert/mod.rs:11435 | private | A VML horizontal line: Word paints it on its paragraph's baseline. | fields: frac,width,h,color,align |
| `StyleRefOp` | src/convert/mod.rs:12056 | private | A painted `STYLEREF` result: its page and op, its line's ops (`line`) and the share of a width change the line's start takes. | fields: page,op,target,line,align |
| `StyleRef` | src/convert/mod.rs:12067 | private | A `STYLEREF` field's target: the style it names (by name, any case, or by id; a bare digit is that heading level) and its `\l` switch. | fields: id,last |
| `CivilDateTime` | src/convert/mod.rs:12262 | private | _no doc comment_ | fields: y,m,d,h,min,s |
| `CoreDates` | src/convert/mod.rs:12272 | private | _no doc comment_ | fields: created,printed,modified |
| `RunCollect` | src/convert/mod.rs:14669 | private | _no doc comment_ | fields: dom,base,theme,styles,authors,in_table,toc,comments,open,pending,bound,pageref,ref_name,styleref,field_instr,field_result,field_emitted,dropdown,in_dropdown,math_vert,in_math,math_upright,math_display,keep_xml_space |
| `RunBag` | src/convert/mod.rs:14706 | private | _no doc comment_ | fields: authors,comments,in_table,toc,open |
| `AuthorColors` | src/convert/mod.rs:15049 | private | _no doc comment_ | fields: names |
| `MathAlphanumerics` | src/convert/mod.rs:15804 | private | Sets [`MathFace`] for the conversion's lifetime, restoring the previous one on drop. |  |
| `GroupText` | src/convert/mod.rs:16771 | private | What a group's text boxes need to lay their text out. | fields: theme,base,sheet |
| `GroupFrame` | src/convert/mod.rs:16796 | private | A group's place in the outer group's box and the fill its `grpFill` children inherit from the groups around it. | fields: frac,fill,canvas |
| `HfChrome` | src/convert/mod.rs:20316 | private | Header/footer chrome from the first section's `sectPr` refs only. | fields: header,footer,header_align,footer_align,header_bottom,footer_top,watermark,header_rest,footer_rest,even_and_odd,header_even,header_odd,footer_even,footer_odd,header_images,footer_images,header_boxes,footer_boxes,header_tables,footer_tables,mirror_margins,rev_bars_facing,punct_squeeze,wp_just |
| `ChromePart` | src/convert/mod.rs:20404 | private | _no doc comment_ | fields: runs,border,align,watermark,images,tables,boxes |
| `PickedHf` | src/convert/mod.rs:20431 | private | _no doc comment_ | fields: start,rest,even,odd |
| `ChromeTable` | src/convert/mod.rs:20472 | private | _no doc comment_ | fields: block,before_text,lead,lead_lines,w,h,color,width,top,bottom,left,right |
| `FieldScan` | src/convert/mod.rs:22243 | private | _no doc comment_ | fields: kind,result,emitted,instr,styleref |
| `SideFloat` | src/convert/mod.rs:22497 | private | _no doc comment_ | fields: align,inset,top,bottom |
| `FloatWrap` | src/convert/mod.rs:22510 | private | The next paragraph's first line as `emit_runs` will place it: How wrapSquare floats narrow a paragraph: `left`/`right` insets, the band's bottom `until` and its top `from`, both below the first line's top. | fields: left,right,until,from |
| `LineProbe` | src/convert/mod.rs:22522 | private | `top` after its effective space-before, `h` its real line box, and the full `before` a jump below a float would otherwise apply twice. | fields: top,h,before,single |
| `Layout` | src/convert/mod.rs:22529 | private | _no doc comment_ | fields: fonts,page,pages,y,header,footer,header_align,footer_align,header_bottom,footer_top,watermark,body_top,body_floor,page_has_body,chrome_end,behind_end,at_page_top,tail_bar,last_table_left,join_table,join_run_width,keep_chain_until,keep_next_follow,rev_bar,suppress_space_before,top_credit,top_ |
| `StyleHit` | src/convert/mod.rs:32298 | private | Body text in one style, for header `STYLEREF` (`Layout::note_style_hit`). | fields: id,serial,pages,text,open |
| `Squeeze` | src/convert/mod.rs:32959 | private | How far a line's spaces may narrow to keep its last word. | fields: share,word_cap,face_bound,line_share |
| `LineFit` | src/convert/mod.rs:33061 | private | How a line makes room for its last word. | fields: squeeze,char_break,hang_spaces,narrow |
| `WrapTabs` | src/convert/mod.rs:33079 | private | Where a paragraph's lines start relative to the tab origin (the flow left edge), so a tab can take its real jump while wrapping. | fields: stops,default_tab,first_start,start,squeeze |
| `ConnectorCubic` | src/convert/mod.rs:34200 | private | _no doc comment_ | fields: start,segments |
| `PdfComment` | src/convert/pdf.rs:142 | pub(crate) | Sticky-note PDF annotation (not painted into the content stream). | fields: id,x,y,w,h,top,bottom,contents,author,initials,label,color,resolved |
| `CommentTint` | src/convert/pdf.rs:165 | pub(crate) | One tinted box of a commented range (laid-out units, bottom-left), in paint order: the first is the range's start, the last its end. | fields: id,x,y,w,h |
| `Page` | src/convert/pdf.rs:174 | pub(crate) | One finished page, with the section `pgSz` it was laid out against. | fields: ops,width,height,comments,tints,markup_pane,margin_r,vertical,changed,number,chrome_changed,section |
| `MarkupChrome` | src/convert/pdf.rs:234 | pub(crate) | Word Save-as-PDF All Markup: the page is scaled by `k` from x = 0.96 and a 0.949 gray pane 257.3pt wide (page units) overlaps its right margin from 9.15pt past the text edge. | fields: gx,gy,gw,gh,k,tx,ty |
| `FaceObjIds` | src/convert/pdf.rs:1122 | private | Face → (font object id, PDF resource name). | fields: items |
| `Show` | src/convert/pdf.rs:1366 | private | A run's text-showing operation, `{lit} Tj`, written in place. | fields: lit,actual |
| `PathDef` | src/convert/preset_geom.rs:59 | pub(crate) | _no doc comment_ | fields: w,h,fill,stroke,cmds |
| `Preset` | src/convert/preset_geom.rs:68 | pub(crate) | _no doc comment_ | fields: av,gd,paths |
| `CustomPath` | src/convert/preset_geom.rs:123 | pub(crate) | One `a:custGeom/a:pathLst/a:path`. | fields: w,h,fill,stroke,cmds |
| `CustomGeom` | src/convert/preset_geom.rs:132 | pub(crate) | A document's `a:custGeom`: guides and paths like a preset's. | fields: av,gd,paths |
| `PathView` | src/convert/preset_geom.rs:138 | private | _no doc comment_ | fields: w,h,fill,stroke,cmds |
| `Subpath` | src/convert/preset_geom.rs:148 | pub(crate) | A flattened subpath in shape space (pt, y down). | fields: pts,closed |
| `EvalPath` | src/convert/preset_geom.rs:155 | pub(crate) | One evaluated `a:path`. | fields: fill,stroke,subpaths |
| `TextRect` | src/convert/preset_geom.rs:162 | pub(crate) | A preset's `a:rect`: the guides its text rectangle reads. | fields: av,gd,rect |
| `Piece` | src/convert/raster.rs:152 | private | One painted glyph string on a baseline. | fields: x,advance,size,text |
| `TextRun` | src/convert/raster.rs:439 | private | One glyph string to paint. | fields: face,size,x,y,glyphs,color,hscale,rotate_deg |
| `OutlineSink` | src/convert/raster.rs:510 | private | _no doc comment_ | fields: pb |
| `Placement` | src/convert/raster.rs:534 | private | Where a picture goes: its user-space box plus the writer's crop, rotation and oval-clip flags. | fields: rect,crop,rotate_deg,oval |
| `SubstRow` | src/convert/word_subst.rs:17 | pub(crate) | _no doc comment_ | fields: key,physical,stems |
| `Options` | src/debug.rs:122 | pub | Report options. | fields: checks,part,grep,limit,context |
| `Entry` | src/debug.rs:156 | private | One zip entry. | fields: name,size,packed,data |
| `Package` | src/debug.rs:164 | private | A package's zip entries, in archive order. | fields: entries,raw |
| `Findings` | src/debug.rs:264 | private | Findings by kind, each with its examples in document order. | fields: by_kind |
| `OpenField` | src/debug.rs:439 | private | _no doc comment_ | fields: begin,separate,instr,code,at |
| `IdUses` | src/debug.rs:965 | private | Ids one package should hold once: revision ids and drawing `wp:docPr` ids. | fields: revisions,drawings |
| `Analysis` | src/debug.rs:1050 | private | Everything one package yields for the selected checks. | fields: findings,elements,chains,textboxes |
| `Walk` | src/debug.rs:1560 | private | Walk state: the lines, the paragraph count, the `runs` view flag. | fields: out,paras,props,critic |
| `DiffOptions` | src/debug/diff.rs:16 | pub | `jubarte debug diff` options. | fields: part,style,para_text,raw,full,limit |
| `Filter` | src/debug/diff.rs:132 | private | What `--style` and `--para-text` keep. | fields: style,para |
| `N` | src/debug/diff.rs:145 | private | One element of a part, normalized. | fields: kind,key,label,lines,kids,block,hit,sub_hit |
| `Build` | src/debug/diff.rs:477 | private | Builds one part's tree. | fields: dom,raw,filter,style_names,targets,paras,tables |
| `Seg` | src/debug/diff.rs:1188 | private | A run of paragraph content: its revision mark, formatting and text. | fields: rev,fmt,text,text_run |
| `Walker` | src/debug/diff.rs:1244 | private | Walks aligned trees and writes hunks. | fields: labels,filtered,full |
| `Rel` | src/debug/render.rs:68 | private | A relationship of the main document part: its type's last segment (`styles`, `theme`, `header`, …), id and target part. | fields: kind,id,part |
| `StyleInfo` | src/debug/render.rs:119 | private | What a style contributes to the page, its `basedOn` parent and name. | fields: name,based,para_shd,color,highlight,run_shd,vanish,frame,fonts,table_shd |
| `StyleBook` | src/debug/render.rs:145 | private | _no doc comment_ | fields: by_id,defaults,default_fonts,default_para |
| `Tally` | src/debug/render.rs:470 | private | Each fact's stretches and the first one's text. | fields: facts,para,before,now |
| `DocDefaultPprDelta` | src/document_comparer.rs:549 | private | What [`doc_default_ppr_delta`] writes into the live Normal pPr. | fields: elements,spacing |
| `EditPlan` | src/edit.rs:70 | pub | A versioned, portable set of operations against one document snapshot. | fields: schema_version,source_sha256,author,date,initials,resolve_revisions,existing_revisions,operations,update_fields |
| `ResolveRevisions` | src/edit.rs:110 | pub | The tracked changes a plan accepts and rejects before it edits, as Word's Accept / Reject This Change (see [`crate::changes`]). | fields: accept,reject |
| `ParagraphMove` | src/edit.rs:121 | pub | Where a source paragraph id went when the plan's revisions were settled. | fields: from,to |
| `ResolvedRevisions` | src/edit.rs:131 | pub | The changes `resolve_revisions` resolved, by id. | fields: accepted,rejected |
| `Operation` | src/edit.rs:165 | pub | One operation. | fields: id,kind |
| `LineSpacing` | src/edit.rs:663 | pub | Line spacing, stored as `w:line` (240 = single); JSON is the multiple. |  |
| `Points` | src/edit.rs:686 | pub | A length stored in twips (1/20 pt); JSON is points. |  |
| `Spacing` | src/edit.rs:708 | private | Paragraph spacing changes for [`OperationKind::FormatParagraph`]. | fields: line,before,after |
| `RunFormat` | src/edit.rs:744 | pub | Run formatting for inserted or replacement text. | fields: bold,italic,underline,highlight,font,size_pt,color,strike,caps |
| `HalfPoints` | src/edit.rs:777 | pub | A font size stored in half points (`w:sz`); JSON is points. |  |
| `RunSpec` | src/edit.rs:872 | pub | A run of an inserted paragraph. | fields: text,bold,italic,underline,highlight |
| `EditOutcome` | src/edit.rs:904 | pub | Outcome of one operation. | fields: id,kind,status,paragraph,matches,context,comment_id,anchor_given,anchor_read_as,code,message |
| `RevisionCounts` | src/edit.rs:943 | pub | The redline's changes by kind, as `jubarte changes` lists them. | fields: inserted,deleted,moved,format_changed,total |
| `ParagraphDelta` | src/edit.rs:977 | pub | Paragraph count before and after. | fields: from,to |
| `EditReport` | src/edit.rs:986 | pub | What happened, per operation and overall. | fields: schema_version,ok,source_sha256,base_sha256,guarded,author,date,existing_revisions,resolved_revisions,paragraph_ids,paragraphs,operations,comments_added,revisions,fields |
| `EditResult` | src/edit.rs:1112 | pub | Clean copy, redline and report. | fields: clean,redline,report |
| `EditError` | src/edit.rs:1124 | pub | Why a plan was refused. | fields: code,operation,message,outcomes |
| `Settled` | src/edit.rs:1248 | private | The document a plan edits once `resolve_revisions` and `existing_revisions` have settled the tracked changes they select. | fields: base,opened,resolved_revisions,changed |
| `Loaded` | src/edit.rs:1300 | private | A document's stories and its addressable paragraphs. | fields: stories,paragraph_nodes,paragraph_story |
| `ScheduledEdit` | src/edit.rs:1761 | private | A text edit scheduled on one paragraph, in source projection bytes. | fields: start,end,op,replacement,attach_before,comment,format |
| `TocSpec` | src/edit.rs:1918 | private | A resolved `insert_toc`. | fields: levels,title |
| `Covered` | src/edit.rs:1930 | private | The text a comment covers: from `start` in paragraph `first` to `end` in paragraph `last` (projection offsets; global paragraph indexes). | fields: first,start,last,end |
| `Transaction` | src/edit.rs:1937 | private | _no doc comment_ | fields: plan,source_sha256,base,base_sha256,resolved_revisions,date,initials,opened,stories,paragraph_nodes,paragraph_story,projections,origin,outcomes,resolved,anchor_window,comments,deletion_comments,preset_comment_ids,next_comment_id,comments_added,whole_marks,new_numbering,needed_styles,family,r |
| `StoryPart` | src/edit.rs:1994 | private | The body, or a header, footer or notes part, parsed into the plan's DOM. | fields: id,part,document,root |
| `FillRequest` | src/edit/controls.rs:60 | pub(super) | The value fields of a `fill_control` operation. | fields: control,text,choice,checked,date |
| `FlagOp` | src/edit/flags.rs:23 | pub | One operation as the flags describe it, grouped by its `-p`. | fields: at,anchor,content,delete,resolve,accept,reject,reply,before,comment,styles |
| `FlagPlan` | src/edit/flags.rs:61 | pub | A plan built from flags, and the notes to print before the view. | fields: plan,notes |
| `Ids` | src/edit/origin.rs:26 | pub(super) | What positional ids resolve against: a document's body and its addressable paragraphs, in the order `load_stories` lists them. | fields: dom,body,story_ids,paragraph_nodes,paragraph_story |
| `Origin` | src/edit/origin.rs:232 | pub(super) | The source as the plan's author read it, and where each of its paragraphs went once the plan's revisions were settled. | fields: opened,story_ids,paragraph_nodes,paragraph_story,fates,control_ids,control_fates |
| `Recorded` | src/edit/record.rs:70 | pub | A recording, ready to be checked against the source and written. | fields: plan,notes,added,added_revisions |
| `RecordOptions` | src/edit/record.rs:227 | pub | What [`record_json`] takes besides the operations: the command line's options, as the Python and npm CLIs parse them. | fields: author,date,existing_revisions,file,plan,default_plan,dry_run |
| `Line` | src/edit/resolution.rs:114 | private | One line's parts before quoting. | fields: action,r#ref,place,from,to |
| `Unit` | src/edit/rewrite.rs:19 | private | A comparison unit of the old text: its key and its bytes. | fields: key,start,end |
| `CustomPage` | src/edit/sections.rs:60 | pub | A custom page size in twentieths of a point (1440 per inch). | fields: width_dxa,height_dxa |
| `Margins` | src/edit/sections.rs:86 | pub | Page margins in twentieths of a point; fields not given keep the section's. | fields: top,right,bottom,left,header,footer |
| `Geometry` | src/edit/sections.rs:119 | pub(super) | A section's page geometry, all in twentieths of a point. | fields: width,height,landscape,keep_code,top,right,bottom,left,header,footer,gutter |
| `Stamp` | src/edit/tracked.rs:33 | private | Revision attribution for the containers the emitter creates. | fields: author,date,next_id |
| `PlanV2` | src/edit/v2.rs:37 | pub | An explicit request against one immutable original DOCX snapshot. | fields: schema_version,source,policy,resolve_revisions,author,date,scope,operations,source_context,coverage |
| `Source` | src/edit/v2.rs:74 | pub | SHA-256 binding to the exact original package bytes, independent of paths. | fields: sha256 |
| `Policy` | src/edit/v2.rs:82 | pub | Tracking concerns new changes only; selected decisions run first. | fields: track,existing_revisions,text_view |
| `Target` | src/edit/v2.rs:120 | pub | Exactly the established short/long pair, with no third representation. | fields: short_id,long_id |
| `RevisionDecisions` | src/edit/v2.rs:131 | pub | Original-source logical revision selections. | fields: accept,reject |
| `AttachedComment` | src/edit/v2.rs:155 | pub | A new comment on the verified nonempty result of its owning operation. | fields: text |
| `SourceContext` | src/edit/v2.rs:360 | pub | Removing an observation requests no edit. | fields: kind,target,text,reason_codes |
| `Coverage` | src/edit/v2.rs:380 | pub | Generator observations, never a substitute for recomputed constraints. | fields: traversal_complete,text_complete,eligible_paragraphs,conditional_paragraphs,context_paragraphs,opaque_regions |
| `Spec` | src/edit/watermark.rs:33 | pub(super) | A validated watermark. | fields: text,color,diagonal,font |
| `Mark` | src/edit/whole.rs:42 | pub(super) | One helper bookmark and the change it marks. | fields: op,part,name,find,replacement |
| `Stamp` | src/edit/whole.rs:200 | private | Revision attribution for the containers a rewrite creates. | fields: author,date,next_id |
| `Item` | src/edit/whole.rs:229 | private | A paragraph child, or a child of a paragraph-level `w:ins`/`w:del`. | fields: node,kind,container |
| `FieldUpdate` | src/fields.rs:48 | pub | One field whose cached result [`update_fields`] rewrote. | fields: kind,code,paragraph,old,new |
| `Updated` | src/fields.rs:64 | pub | Output of [`update_fields`]. | fields: docx,fields,page_count |
| `Story` | src/fields.rs:237 | private | _no doc comment_ | fields: id,part,dom,document,root,body,changed |
| `Context` | src/fields.rs:256 | private | What every result is computed from. | fields: facts,defined |
| `Field` | src/fields.rs:267 | private | One field in document order. | fields: kind,code,begin,separate,end,simple,in_result_of |
| `Code` | src/fields.rs:486 | private | A parsed field code: positional arguments and switches with their argument (`\o "1-3"` gives `("o", Some("1-3"))`). | fields: args,switches |
| `Tocs` | src/fields.rs:945 | private | ── TOC ─────────────────────────────────────────────────────────────────── _(from a code comment)_ | fields: rebuilt,max_level |
| `Heading` | src/fields.rs:952 | private | _no doc comment_ | fields: paragraph,level,text |
| `BookmarkNamer` | src/fields.rs:1078 | private | Gives each heading a `_Toc` bookmark inside its paragraph, reusing one it already has. | fields: taken,next_id,next_name |
| `IdError` | src/ids.rs:86 | pub | A refused id: what was given and what was expected. | fields: given,expected,hint |
| `ChangeId` | src/ids.rs:128 | pub | One tracked change, addressed by story and the change's `w:id` value. | fields: story,wid |
| `Paragraph` | src/inspect.rs:49 | pub | One paragraph of the body or a story. | fields: index,id,text,style,numbered,in_table,page_break,runs,limitations |
| `Span` | src/inspect.rs:77 | pub | Direct formatting of a run of `text` (`[start, end)` in chars). | fields: start,end,bold,italic,underline,highlight |
| `Summary` | src/inspect.rs:94 | pub | Package facts. | fields: paragraphs,tables,fields,sections,comments,revisions,footnotes,endnotes,headers,footers,images,list_numbering,track_changes |
| `Snapshot` | src/inspect.rs:128 | pub | Snapshot of a package as it stood at inspection time, serialized by [`inspect_json`] and consumed by [`crate::edit::apply_plan_json`]. | fields: schema_version,source_sha256,summary,paragraphs,stories,tables,controls,changes,comments |
| `Story` | src/inspect.rs:196 | pub | A header, footer or notes part an edit plan can address. | fields: id,kind,part,paragraphs |
| `ContentControl` | src/inspect.rs:210 | pub | A content control (`w:sdt`) in the body, in document order. | fields: id,tag,alias,kind,text,paragraph_ids,locked,choices,checked,placeholder |
| `Opened` | src/inspect.rs:516 | pub(crate) | A parsed main document part plus its package. | fields: pkg,main,dom,document,body |
| `Segment` | src/inspect.rs:994 | pub(crate) | One projected segment: `text[start..end]` (bytes) comes from `piece`. | fields: start,end,piece,direct |
| `Projection` | src/inspect.rs:1004 | pub(crate) | Visible projection of one paragraph, with the segment map edits need. | fields: text,segments,spans,page_break,limitations,field_marks |
| `Format` | src/inspect.rs:1016 | private | _no doc comment_ | fields: bold,italic,underline,highlight |
| `Table` | src/inspect/tables.rs:18 | pub | A body table, nested tables included (each is its own entry). | fields: index,rows,header_rows,widths_dxa |
| `TableCell` | src/inspect/tables.rs:34 | pub | A table cell. | fields: paragraph_ids,text |
| `LegacyDocError` | src/legacy_doc.rs:71 | pub | Why a `.doc` could not be read, formatted as `LEGACY_DOC: …` on the wire. | fields: message |
| `Table` | src/legacy_doc.rs:104 | pub | A table of the main story. | fields: rows,header_rows |
| `Paragraph` | src/legacy_doc.rs:116 | pub | A paragraph of the main story. | fields: heading,list,spans |
| `Span` | src/legacy_doc.rs:138 | pub | A run of text with one bold and italic setting. | fields: text,bold,italic |
| `ListItem` | src/legacy_doc.rs:149 | pub | A paragraph's place in a list. | fields: ordered,level |
| `LegacyDocument` | src/legacy_doc.rs:158 | pub | The main story of a `.doc`, the only place this module reads from. | fields: blocks |
| `DirEntry` | src/legacy_doc.rs:636 | private | A directory entry: name, type, first sector and size. | fields: name,kind,left,right,child,start,size |
| `CompoundFile` | src/legacy_doc.rs:646 | private | _no doc comment_ | fields: bytes,sector_size,mini_sector_size,mini_cutoff,fat,mini_fat,entries,mini_stream |
| `Fib` | src/legacy_doc.rs:876 | private | --------------------------------------------------------------------------- File Information Block (MS-DOC 2.5.1) --------------------------------------------------------------------------- _(from a code comment)_ | fields: table_one,ccp_text,fc_stshf,lcb_stshf,fc_plcf_bte_papx,lcb_plcf_bte_papx,fc_plcf_bte_chpx,lcb_plcf_bte_chpx,fc_clx,lcb_clx,plf_lst,plf_lfo,plcf_sed |
| `Piece` | src/legacy_doc.rs:962 | private | --------------------------------------------------------------------------- Piece table (MS-DOC 2.9.38 Clx, 2.9.177 Pcd) --------------------------------------------------------------------------- _(from a code comment)_ | fields: cp_start,cp_end,fc,compressed,modifier |
| `CharModifier` | src/legacy_doc.rs:975 | private | Bold and italic a piece's property modifier sets (`None`: left as the CHPX has it). | fields: bold,italic |
| `StoryChar` | src/legacy_doc.rs:1134 | private | A character of the main story and the stream offset it was read from. | fields: ch,cp,fc,modifier |
| `Props` | src/legacy_doc.rs:1270 | private | The properties read from a PAPX or a CHPX; each kind fills its own. | fields: istd,in_table,row_end,ilfo,table_header,ilvl,bold,italic |
| `FkpIndex` | src/legacy_doc.rs:1290 | private | Formatted disk pages of one kind, read once. | fields: runs |
| `Lists` | src/legacy_doc.rs:1477 | private | Number format of each list level, by list id, and the list id of each list override. | fields: overrides,formats |
| `Story` | src/legacy_doc.rs:1577 | private | What the main story's characters are read against. | fields: papx,chpx,styles,lists,section_marks |
| `Border` | src/markdown/borders.rs:32 | pub(crate) | One side's border. | fields: val,size,color,space,theme,tint,shade |
| `ChangeRegion` | src/markdown/critic.rs:198 | pub(crate) | A top-level change span of a source: an insertion, a deletion or a substitution that no other span holds. | fields: open,body,split,close,inserted |
| `Closers` | src/markdown/critic.rs:422 | private | Where every unescaped closer and separator sits in the source, found once so that an opener without its closer does not scan the rest of the text, and the parts Markdown reads literally. | fields: patterns,positions,comments,highlights,cover,runs,literal,marks,blanks,fields,unwind |
| `Literal` | src/markdown/critic.rs:718 | private | The parts of a Markdown source that Markdown reads literally: code spans, link destinations and autolinks (`verbatim`), where nothing opens, and fenced code blocks and the YAML front matter (`fences`), where CriticMarkup still marks tracked changes (the reader prints them there) but `==` and `{CODE}` open nothing: a h... | fields: verbatim,fences,front_matter |
| `Check` | src/markdown/critic.rs:1011 | private | What a whole-`encode` pass found amiss, for a caller that warns: positions, line numbers resolved at the end so the scan stays cheap. | fields: unclosed,consumed |
| `Pieces` | src/markdown/critic.rs:1704 | pub(crate) | Splits parsed text into [`Piece`]s. | fields: escape_pending |
| `Output` | src/markdown/diff.rs:369 | private | The marked-up document being written, line by line. | fields: text,started,carry,last |
| `OpenSpan` | src/markdown/diff.rs:966 | private | A formatting span [`atoms`] is reading: where it started, the range of its text so far, and whether a change anywhere in it covers all of it (an image or an autolink, whose text is as verbatim as its syntax). | fields: start,inner,whole |
| `Atom` | src/markdown/diff.rs:974 | private | A span that is read whole: a change that touches its `triggers` covers all of `range`. | fields: range,triggers |
| `Handles` | src/markdown/from_docx/agent.rs:26 | pub(crate) | Author handles, order and timestamps for tags, notes and the header. | fields: by_author,order,dates,shown_ranges,shown_bookmarks,hidden_styles |
| `RevTag` | src/markdown/from_docx/agent.rs:80 | pub(crate) | One logical revision as the id line and the header count it. | fields: kind,tag,author,date |
| `Counter` | src/markdown/from_docx/agent.rs:501 | private | _no doc comment_ | fields: p,t |
| `LineFacts` | src/markdown/from_docx/agent.rs:796 | pub(crate) | What the id line of a paragraph says beyond its index. | fields: index,style,heading,marker,level,outline,page_break,resolved,comments,spans,empty,unshown |
| `Threads` | src/markdown/from_docx/agent.rs:2113 | pub(crate) | Comment threads from `commentsExtended.xml`. | fields: reply_of,done |
| `Block` | src/markdown/from_docx/agent.rs:2239 | private | One block of the rendered body: its lines, the paragraph span it covers and, for a table, its number. | fields: text,span,table,page |
| `Note` | src/markdown/from_docx/critic.rs:239 | pub(crate) | A comment's note, as each of its contexts prints it: the prose around its anchor, and the code block it is anchored in, whose fence unwinds only the escape that form's text uses (#518). | fields: prose,code |
| `Critic` | src/markdown/from_docx/critic.rs:289 | pub(crate) | Inline content of one paragraph with CriticMarkup spans. | fields: tokens,inline_end,agent,code_block |
| `Piece` | src/markdown/from_docx/critic.rs:802 | private | One piece of rendered inline text. | fields: text,marks,link,marker |
| `Escaping` | src/markdown/from_docx/critic.rs:813 | private | How [`render`] escapes document text. | fields: code_block |
| `Drawing` | src/markdown/from_docx/drawing.rs:73 | pub(crate) | What `![label](drawing:PAYLOAD)` decodes to: the element's XML with its relationship ids symbolized (`r1`, `d3.2` — part 3's second relationship), and every part they resolve to. | fields: xml,wrap,rels,deps,slot,mhash,mermaid |
| `PayRel` | src/markdown/from_docx/drawing.rs:107 | private | One relationship of the element's part or of a dependent part. | fields: sym,kind,u,part,raw |
| `Dep` | src/markdown/from_docx/drawing.rs:127 | private | One package part a drawing's relationships reach. | fields: name,kind,b64,xml,rels |
| `ScopeMap` | src/markdown/from_docx/drawing.rs:153 | pub(crate) | `xmlns` scopes for the drawings of one part's element tree, by element address. | fields: root,by_ptr |
| `Pending` | src/markdown/from_docx/drawing.rs:199 | pub(crate) | A drawing queued while a story was read: its element cloned, where it sits, and the scope frozen where it was found. | fields: element,part,scope,wrap |
| `Deps` | src/markdown/from_docx/drawing.rs:428 | private | Scratch for the capture in flight: the parts collected and why it stopped. | fields: types,parts,index,failed |
| `StoryFact` | src/markdown/from_docx/header.rs:35 | pub(crate) | A header or footer of one section. | fields: kind,ty,id,part,first,text,more,align,paras,styles,active,rev,fmt |
| `ParaFact` | src/markdown/from_docx/header.rs:70 | pub(crate) | One paragraph of a header or footer: its own alignment, text colour and borders. | fields: align,color,border |
| `SectionFact` | src/markdown/from_docx/header.rs:82 | pub(crate) | A section of the body (Task 11 fills these). | fields: first,last,sect_pr,stories,title_page |
| `CommentFact` | src/markdown/from_docx/header.rs:91 | pub(crate) | _no doc comment_ | fields: id,author,parent,anchored,style |
| `Facts` | src/markdown/from_docx/header.rs:103 | pub(crate) | _no doc comment_ | fields: source,sha256,resolved,comments_inline,tracking_on,tags,changes,comments,handles,done,owner,title,paragraphs,tables,pages,pages_source,range,stories_changed,styles,theme,default_style,heading_styles,other_styles,table_styles,tables_differ,sections,note_changes,empty_notes,unplaced |
| `Resolved` | src/markdown/from_docx/header.rs:694 | private | Resolved paragraph/run properties of a style through its `basedOn` chain and the document defaults. | fields: character,font,size_half_points,bold,italic,before,after,line,keep_next,keep_lines,contextual_spacing,left,first_line,hanging,align |
| `State` | src/markdown/from_docx/header.rs:1564 | private | _no doc comment_ | fields: out,fields,view,deleting,last,in_result |
| `Side` | src/markdown/from_docx/header.rs:1578 | private | One side of a change the tagged view printed: its text, its marks' tags, and their author. | fields: text,tags,author |
| `Shown` | src/markdown/from_docx/header.rs:1586 | private | A change the tagged view printed from `start` to `end`: an insertion, a deletion, or the insertion then the deletion Word's Compare writes for a substitution. | fields: start,end,new,old |
| `Open` | src/markdown/from_docx/header.rs:1594 | private | A complex field being read. | fields: instr,result,printed,nested,at |
| `Extracted` | src/markdown/from_docx/media.rs:17 | pub(crate) | Where pictures go and what has been collected. | fields: dir,files |
| `Mermaid` | src/markdown/from_docx/mermaid.rs:150 | private | A Mermaid fence's text, parsed: its chart's series in order — `pie` slices as one series, `xychart-beta` `bar`/`line` lines as theirs — with the categories and title it names. | fields: title,cats,sers |
| `Options` | src/markdown/from_docx/mod.rs:43 | pub(crate) | _no doc comment_ | fields: revisions,media_dir,ids,comments,source,pages,page_markers,dates,select |
| `Converted` | src/markdown/from_docx/mod.rs:79 | pub(crate) | _no doc comment_ | fields: markdown,selected,selected_stories,media |
| `Style` | src/markdown/from_docx/mod.rs:1696 | private | _no doc comment_ | fields: name,exact_name,based_on,outline,num,bold,italic,underline,vanish,jc,font |
| `Styles` | src/markdown/from_docx/mod.rs:1717 | private | _no doc comment_ | fields: by_id,default_font,default_paragraph |
| `Level` | src/markdown/from_docx/mod.rs:1955 | private | _no doc comment_ | fields: format,start,text |
| `Numbering` | src/markdown/from_docx/mod.rs:1963 | private | _no doc comment_ | fields: nums,abstracts |
| `Field` | src/markdown/from_docx/mod.rs:2052 | private | An open complex field (`fldChar begin … separate … end`). | fields: instruction,in_result,link,closes,spans,nested,printed |
| `Writer` | src/markdown/from_docx/mod.rs:2260 | private | _no doc comment_ | fields: rels,media,styles,numbering,counters,started_nums,note_refs,note_changes,lead,flattening,bookmarks,tables_differ,custom_grid,heading_slug,hidden_styles,slugger,monospace_paragraph,closing_fields,spanning_fields,open_fields,revisions,comments,notes,referenced,noted_at_end,noted_at_reference,o |
| `Package` | src/markdown/from_docx/ooxml.rs:32 | pub(crate) | A zip container with entry-count and inflated-size limits enforced. | fields: archive,names |
| `Rel` | src/markdown/from_docx/ooxml.rs:193 | pub(crate) | _no doc comment_ | fields: kind,target,external |
| `Rels` | src/markdown/from_docx/ooxml.rs:200 | pub(crate) | _no doc comment_ | fields: by_id,order |
| `Element` | src/markdown/from_docx/ooxml.rs:260 | pub(crate) | A parsed XML element. | fields: name,attrs,children |
| `Marks` | src/markdown/from_docx/ooxml.rs:505 | pub(crate) | How a stretch of inline text is set: the marks printed around it. | fields: bold,italic,underline,highlight,strike,code |
| `Span` | src/markdown/from_docx/ooxml.rs:524 | pub(crate) | One formatted stretch of inline text. | fields: text,marks,link |
| `Inline` | src/markdown/from_docx/ooxml.rs:532 | pub(crate) | Accumulates spans for one paragraph. | fields: spans,keep_spaces,verbatim |
| `Media` | src/markdown/from_docx/ooxml.rs:1046 | pub(crate) | The image parts a part's relationships list, read up front so rendering needs no package access. | fields: extracted,files |
| `Blocks` | src/markdown/from_docx/ooxml.rs:1112 | pub(crate) | Join Markdown blocks: list items stay tight, everything else gets a blank line. | fields: out,last_was_list,separator,trailing_space,tight |
| `ListIndent` | src/markdown/from_docx/ooxml.rs:1213 | pub(crate) | Tracks list marker widths so nested items indent under their parent's content. | fields: widths |
| `Stops` | src/markdown/math.rs:749 | private | Where a list of nodes ends. | fields: group,at,right,rows,cells |
| `Parser` | src/markdown/math.rs:762 | private | _no doc comment_ | fields: tokens,pairs,at,change,depth |
| `Writer` | src/markdown/math.rs:1665 | private | Writes nodes as OMML, marking each run and control with a tracked change when `mark` gives one. | fields: xml,change,mark |
| `DocxOptions` | src/markdown/mod.rs:172 | pub | How [`markdown_to_docx`] writes a Markdown document to a `.docx`. | fields: reference,critic,track_changes,author,date,author_fixed,date_fixed,images,page,tracking,scholar |
| `DocxOptionsJson` | src/markdown/mod.rs:238 | private | [`DocxOptions`] as the Python and WebAssembly bindings pass them (JSON, every field optional). | fields: page,author,date,critic,track_changes,tracking,scholar |
| `MarkdownOptions` | src/markdown/mod.rs:306 | pub | How [`docx_to_markdown`] reads a `.docx`. | fields: track_changes,extract_media,ids,comments,source,pages,page_markers,dates,select |
| `ReadDocx` | src/markdown/mod.rs:604 | pub | A document read as Markdown. | fields: markdown,media,selected,selected_stories |
| `ReadOptions` | src/markdown/mod.rs:678 | pub | How [`read`] (and `jubarte read`) prints the agent view. | fields: track_changes,comments,dates,page_markers,select,source |
| `ReadView` | src/markdown/mod.rs:710 | pub | A document's agent view and what [`read`] could not show (always populated, even empty). | fields: markdown,warnings,selected,selected_stories |
| `WrittenDocx` | src/markdown/mod.rs:968 | pub | A written document and what could not be written as asked. | fields: docx,warnings |
| `CoreEdit` | src/markdown/package.rs:1330 | private | One core property the view sets (`value`), or clears (`clear`). | fields: namespace,local,value,clear |
| `Media` | src/markdown/package.rs:1475 | private | Picture parts added so far, by picture index. | fields: parts,used |
| `PartRelate` | src/markdown/package.rs:1481 | private | Relationships from one story part. | fields: package,part,media,document |
| `Patched` | src/markdown/patch.rs:49 | pub | A Word document the Markdown edits landed on. | fields: docx,warnings |
| `Block` | src/markdown/patch.rs:72 | private | A Markdown block: its kind, its plain text and its runs. | fields: kind,runs |
| `Paragraph` | src/markdown/patch.rs:85 | private | A body paragraph: its index in the body, text and kind. | fields: index,text,kind |
| `RedlineOptions` | src/markdown/redline.rs:46 | pub | How [`redline`] reads and compares two [`Source`] documents. | fields: settings,reference,critic,images |
| `Attribution` | src/markdown/unified.rs:46 | pub | The author and date of a change or comment, in the form Word writes and jubarte's Markdown edit path encodes (`YYYY-MM-DDTHH:MM:SSZ`). | fields: author,date |
| `PatchOptions` | src/markdown/unified.rs:58 | pub | The names written on the patch's `---` and `+++` header lines, and the document owner (the author and date of every change and comment that names none). | fields: old_name,new_name,owner |
| `Comment` | src/markdown/unified.rs:96 | pub | A comment, on a change, on highlighted text, or at a point. | fields: author,date,text,on |
| `Change` | src/markdown/unified.rs:114 | pub | One tracked change (or self-contained substitution), with its author and date. | fields: old,new,author,date,comment |
| `Hunk` | src/markdown/unified.rs:133 | pub | One changed paragraph: its [`Locator`] (where it is in the old or new document), whether it is removed (a paragraph only in the old version), its rendered text with markup intact, its changes in order, and any comments that were not written right after a change. | fields: at,removed,text,changes,comments |
| `Patch` | src/markdown/unified.rs:150 | pub | The changed paragraphs of a document, as a `git diff --word-diff` patch. | fields: old_name,new_name,owner,hunks |
| `WordParagraph` | src/markdown/unified.rs:399 | private | A Word paragraph as [`patch_word`] looks for it. | fields: at,letters |
| `Search` | src/markdown/unified.rs:405 | private | The paragraphs of one version, looked for in order. | fields: letters,paragraphs,next,last |
| `Paragraphs` | src/markdown/unified.rs:509 | private | A block's text in the old and new versions, and its hunk if it has one. | fields: hunk,old,new |
| `Edit` | src/markdown/unified.rs:585 | private | Deleted and inserted text: `plain` without the markup nested in it, `marked` with it. | fields: old,new,old_marked,new_marked,by,new_by |
| `Block` | src/markdown/unified.rs:813 | private | A paragraph (or table row) of segments, with the line breaks before it. | fields: separator,old_before_visible,new_before_visible,segments |
| `Author` | src/markdown/view_notes.rs:18 | pub(super) | An author of the `authors:` table: the full name, and the timestamp every one of their changes and comments shares, when they share one. | fields: name,date,range |
| `FileDefaults` | src/markdown/view_notes.rs:136 | pub(super) | What a Markdown file's front matter gives every comment and change that names none of its own: the top-level `author:`, and the file's date (`last_modified:`, else `modified:`, `date:`, `created:`). | fields: author,date,unread |
| `Tag` | src/markdown/view_notes.rs:353 | pub(super) | One tag of a revision note: who and, when the note says, when. | fields: handle,date,ids |
| `PlainRevision` | src/markdown/view_notes.rs:400 | pub(super) | A plain view's revision note: the change's ids, then who made it and when, as stored (`r5 Ana Lima (2026-…)`, `r1+2 (2026-…)`, `r7 Di Eng`, `r7`). | fields: ids,author,date |
| `CommentHead` | src/markdown/view_notes.rs:541 | pub(super) | A comment head the view writes: `c5 @AC: `, `c5 @AC 2026-…: `, `c6 @AS re c5: `, `c5 @AC resolved: `, `c5: `. | fields: number,handle,date,reply_to,resolved |
| `TableLine` | src/markdown/view_notes.rs:574 | pub(super) | What a table's id line says about its rows (`<!-- t0 4x7, cells t0.row0 p1-p13 t0.row1 p14-p50, header row repeats -->`): how many paragraphs each row holds, when it counts them row by row, and whether its first row repeats on each page. | fields: row_paragraphs,one_per_cell,header_repeats,marks,columns,align,merges,aligns,styles,style,borders,cell_borders,row_borders,in_comment,fmt,grid_fmt,row_fmt,cell_fmt,para_fmt,row_marks,cell_marks,gaps |
| `IdLine` | src/markdown/view_notes.rs:1275 | pub(super) | What a paragraph's id line says about it (`<!-- p5 TitlePage center, first-line 0.5in -->`): its style, alignment (`w:jc`) and indents in twips. | fields: number,style,outline,align,first_line,hanging,left,right,page_break,page_break_change,section_break,label,level,break_ins,break_del,fmt,rev,rev_del,in_comments |
| `PageSetup` | src/markdown/view_notes.rs:1964 | pub(super) | A section's page as the view's YAML header prints it (`page: A4 landscape, margins 0.5in, header/footer 0.49in`), in twips. | fields: width,height,landscape,margins,header,footer,gutter,columns,column_gap,column_line,grid,char_space,starts |
| `Story` | src/markdown/view_stories.rs:35 | pub(super) | One header or footer of a section. | fields: kind,ty,part,paragraph,text,more,align,formats,styles,active,rev,fmt |
| `ParaFormat` | src/markdown/view_stories.rs:68 | pub(super) | One header or footer paragraph's own formatting. | fields: align,color,border |
| `People` | src/markdown/view_stories.rs:117 | pub(super) | Who made a story's changes: each handle's name and date, and the document's for a change no tag names. | fields: by_handle,author,date,range_ends,started,drawing_defs,drawing_backings |
| `Marks` | src/markdown/view_stories.rs:136 | private | The marks of one story part being written: their people and the next `w:id` for a mark no tag numbers. | fields: people,next_id |
| `Stories` | src/markdown/view_stories.rs:479 | pub(super) | What a section's headers and footers say: its stories, and whether its first page has its own (`first: none` says so with no story). | fields: stories,title_page,dropped |
| `SectionStories` | src/markdown/view_stories.rs:943 | pub(super) | What each section's `w:sectPr` takes from the view's headers and footers: its references and whether its first page has its own. | fields: by_section,named |
| `OpenTag` | src/markdown/view_styles.rs:148 | private | Where an element's start tag lies. | fields: start,end,empty |
| `StyleSpec` | src/markdown/view_styles.rs:178 | pub(super) | One style the legend describes. | fields: id,name,character,heading,default,font,bold,italic,size,before,after,line,keep_next,keep_lines,left,first_line,hanging,contextual_spacing,align |
| `Given` | src/markdown/view_styles.rs:550 | private | What a style already inherits, so a property equal to it is not written. | fields: font,size,bold,italic,before,after,line_twips,line_rule,align,keep_next,keep_lines,contextual,left,first_line,hanging |
| `TablePlan` | src/markdown/write.rs:678 | private | What the view's table line says about one table, read before the table: whether its first row repeats, its grid's columns, and how each row's Markdown cells and `<br>`s map onto its cells and paragraphs. | fields: header_repeats,columns,align,rows,marks,aligns,style,borders,cell_borders,row_borders,styles,in_comment,fmt,grid_fmt,row_fmt,cell_fmt,para_fmt,row_marks,cell_marks |
| `RowPlan` | src/markdown/write.rs:715 | private | One row of a [`TablePlan`]. | fields: breaks,first,cells,pad,gaps |
| `HtmlTag` | src/markdown/write.rs:1300 | private | One inline HTML tag, as CommonMark names it. | fields: name,closing,empty |
| `OpenList` | src/markdown/write.rs:1461 | private | A list being written: its numbering instance, its level, and how many paragraphs its current item has. | fields: instance,level,paragraphs |
| `SpanStart` | src/markdown/write.rs:1470 | private | Where a change span opened: in which paragraph and whether before any of its text, so a paragraph that is one change whole can take its mark in. | fields: serial,kind,paragraph,at_start,runs_before,at_break |
| `Anchor` | src/markdown/write.rs:1494 | private | A span that just closed; a comment right after it anchors to it. | fields: slot,group,serials,attributed |
| `Writer` | src/markdown/write.rs:1514 | private | _no doc comment_ | fields: options,soft_break,held,body,definitions,shared_tags,endnotes,endnote_next,references,target,note_clauses,paragraph,paragraph_serial,paragraph_runs,heading,label_prefix,view,resolved,html_block,tables,table_plan,table_count,row_number,row_breaks,note_breaks,item_levels,item_count,label_defin |
| `Slugger` | src/markdown/write.rs:5027 | pub(super) | The names github-slugger gives a document's headings, one at a time. | fields: seen |
| `RunFormat` | src/markdown/xml.rs:24 | pub(super) | Direct run formatting. | fields: bold,italic,strike,underline,superscript,subscript,mark,mark_color,code,link |
| `Run` | src/markdown/xml.rs:70 | pub(super) | A run: content, formatting, the change it belongs to and the comment scope (`{==…==}`) group it is in. | fields: content,format,change,scope |
| `Paragraph` | src/markdown/xml.rs:79 | pub(super) | Paragraph properties. | fields: style,numbering,indent,ind,align,outline,rule,mark,mark_by,whole,label,mark_fixed,section_break,heading,bookmark,formats,unshown,number,text_box |
| `TextBox` | src/markdown/xml.rs:130 | pub(super) | A minimal anchored text box: the change its anchor run carries, as the paragraph's `rev` clause names it, and the box's content, its runs carrying the changes the header's `unplaced:` names for the paragraph. | fields: inserted,mark,items |
| `FormatMark` | src/markdown/xml.rs:144 | pub(super) | A formatting change, or a mark whose text the view does not show, as the view names it: its `w:id`, who made it and when. | fields: id,author,date |
| `TableFormats` | src/markdown/xml.rs:153 | pub(super) | The formatting changes a table line names: the table's own, a row's (`in t0.row1`) and a cell's (`in t0.row1.cell0`). | fields: table,grid,row_marks,cell_marks,rows,cells,borders,cell_borders,row_borders |
| `Indent` | src/markdown/xml.rs:173 | pub(super) | A paragraph's `w:ind`, side by side, in twips. | fields: first_line,hanging,left,right |
| `List` | src/markdown/xml.rs:233 | pub(super) | A numbering instance: a bullet list, or an ordered list and where it starts. | fields: ordered,level,start |
| `Picture` | src/markdown/xml.rs:241 | pub(crate) | An embedded picture. | fields: bytes,extension,content_type,width,height,alt |
| `Note` | src/markdown/xml.rs:253 | pub(super) | A footnote or an endnote: its items and the change of its reference, which the note shares when its own text has none. | fields: items,change,endnote |
| `Comment` | src/markdown/xml.rs:262 | pub(super) | A comment's paragraphs. | fields: id,paragraphs,by,initials,number,parent,done,style |
| `Document` | src/markdown/xml.rs:283 | pub(super) | A Markdown document read into Word items. | fields: body,notes,empty_notes,note_formats,style_changes,comment_style,comment_styles,comments,slots,shared,commented,note_changes,pictures,drawings,drawing_defs,drawing_backings,lists,styles,named_styles,title,author,owner,table_style,table_legend,table_styles,tracking,warnings,attributions,revisi |
| `LabelUse` | src/markdown/xml.rs:410 | pub(super) | A paragraph numbered with a label: which one, at which list level, and the number its list counts it (`2.` in `2. | fields: label,level,start |
| `Context` | src/markdown/xml.rs:466 | pub(super) | Numbers shared by every story of one document. | fields: document,author,date,next_revision,cursors,used_revisions,pending_marks,num_ids,label_nums,note_ids,in_endnote,next_drawing,next_bookmark,text_width,sections_ended,default_section,section_stories |
| `Relationships` | src/opc/mod.rs:37 | pub | Relationships with this crate's per-record refusal contract: oxml-opc skips elements outside the relationships namespace, while this crate also reads legacy unnamespaced records and refuses a record missing a required attribute. |  |
| `PartFs` | src/opc/mod.rs:412 | pub | Thin adapter over `oxml_opc::OpcPackage`. | fields: pkg,package_rels,part_rels,source_order,repaired_duplicate_ids |
| `Snapshot` | src/perf.rs:148 | pub | Snapshot of counters and stage timers. | fields: stage_ns,lcs_calls,lcs_window_area,corr_run_scans,corr_run_hits,unit_clones,atoms,units |
| `Artifact` | src/result.rs:168 | pub | A committed or uncommitted artifact recorded in the result. | fields: path,media_role,media_type,sha256,byte_size,committed |
| `Context` | src/result.rs:225 | pub | Bounded context around a refused match. | fields: before,matched,after,truncated |
| `NextAction` | src/result.rs:264 | pub | A structured next action: an `argv` array (never a shell string) with prerequisites and purpose. | fields: argv,prerequisites,purpose |
| `Error` | src/result.rs:303 | pub | An error block for a refused or failed result. | fields: code,operation,source_digest,target,matches,context,message,next_actions,retryable,commit_state |
| `Source` | src/result.rs:438 | pub | A reference to the bound source. | fields: sha256,path,byte_size |
| `Runtime` | src/result.rs:479 | pub | The runtime that produced the result. | fields: name,build |
| `Verification` | src/result.rs:500 | pub | Verification block for an `ok` envelope. | fields: operation_postconditions,integrity,expectations,task_semantics |
| `Counts` | src/result.rs:529 | pub | Counts of operations, revisions and comments. | fields: requested_operations,unchanged_operations,body_operations_applied,review_operations_applied,revisions_accepted,revisions_rejected,logical_revisions,revision_xml_entries,comment_threads,comment_replies,comments_added |
| `ResultData` | src/result.rs:590 | pub | Result data for success or failure: phase (null before any completed stage), effect, optional predicted effect, counts and verification. | fields: phase,effect,predicted_effect,counts,verification |
| `Envelope` | src/result.rs:735 | pub | The terminal record of a v2 command. | fields: schema_version,command,engine_version,runtime,status,source,data,artifacts,warnings,error,effective_options,continuation |
| `StreamHeader` | src/result.rs:908 | pub | First JSONL record. | fields: schema_version,command,engine_version,source |
| `ProgressEvent` | src/result.rs:937 | pub | Progress event for JSONL streams: a free-form JSON value so callers can attach any `ev`/`load`/`op`/`save` line they want. | fields: value |
| `ParsedStream` | src/result.rs:988 | pub | A parsed JSONL stream: the progress events and the terminal record. | fields: header,events,terminal |
| `ExitStatus` | src/result.rs:1132 | pub | Exit status values per the proposal §12 table. |  |
| `Tag` | src/revision_processor.rs:757 | pub | `Tag` (RevisionProcessor.cs :2393): one open/empty/close event. | fields: element,tag_type |
| `BlockContentInfo` | src/revision_processor.rs:834 | pub | `BlockContentInfo` (RevisionProcessor.cs :52): prev / this / next links for block-level content. | fields: previous_block_content_element,this_block_content_element,next_block_content_element |
| `Parsed` | src/revision_processor/comments.rs:30 | pub(super) | One parsed part, written back by [`Parsed::store`]. | fields: name,dom,doc,root |
| `Slot` | src/revision_processor/style_records.rs:103 | private | One property slot: the element's expanded name and, for the attribute-wise properties, the first attribute of its group (empty for a whole element). | fields: ns,local,group |
| `Levels` | src/revision_processor/style_records.rs:278 | pub(super) | The pPr slots of each numbering level, by numId and ilvl: the abstract's levels (through a `w:numStyleLink`, the linked abstract's, as rendering reads them), a full `lvlOverride` level taking over the abstract one. |  |
| `ScrubOptions` | src/scrub.rs:53 | pub | What [`scrub`] removes. | fields: author_alias,rsids,docprops,comments |
| `Part` | src/scrub.rs:181 | private | One parsed XML part, written back only when changed. | fields: name,dom,doc,root |
| `Protection` | src/settings.rs:185 | pub | An editing restriction. | fields: edit,enforcement,password |
| `SettingsRequest` | src/settings.rs:202 | pub | The settings one plan operation writes. | fields: track_revisions,update_fields,protection |
| `UnifiedOptions` | src/text_diff.rs:22 | pub | Names written on the patch headers and the number of unchanged context lines around each change. | fields: old_name,new_name,context |
| `TextOptions` | src/text_diff.rs:65 | pub | Review presentation options for [`diff_text_view`] and [`diff_documents_view`]; the raw unified patch APIs above ([`diff_text`], [`diff_documents`]) stay unclipped. | fields: unified,format,accept_changes,window |
| `Options` | src/update.rs:53 | pub | The flags `jubarte self-update` accepts, resolved from the CLI before [`run`] sees them. | fields: check,yes,version |
| `Finding` | src/validate.rs:87 | pub | One thing wrong with a package, from any rule set: Word validity (`word`), tracked-edit discipline (`edits`), or the [`crate::audit`] sets (`a11y`, `style`, `structure`). | fields: code,set,severity,part,location,r#ref,message,word_fatal,repairable |
| `Repaired` | src/validate.rs:268 | pub | The package with every repairable finding fixed, the findings it fixed and the ones it could not. | fields: docx,repaired,remaining |
| `ExtendedGraph` | src/validate.rs:1363 | private | _no doc comment_ | fields: keys,parents |
| `WmlDocument` | src/wml_document.rs:24 | pub | A WordprocessingML document held as its package bytes plus the main document part parsed on demand. | fields: document_byte_array,file_name,part_fs,dom,main_doc |
| `XNamespace` | src/xmllinq/mod.rs:100 | pub | An XML namespace, identified by its URI string, owned through a thread-local-interned `Arc<str>`. | fields: name |
| `XName` | src/xmllinq/mod.rs:170 | pub | An expanded XML name: a local name plus its namespace. | fields: local,namespace |
| `NodeId` | src/xmllinq/mod.rs:290 | pub | Handle into a [`Dom`] arena: the slot index of a node, plus a `Debug` / `Copy` so it can travel cheaply through hot loops (compare, atomize). |  |
| `XDeclaration` | src/xmllinq/mod.rs:297 | pub | An XML declaration (`<?xml version encoding standalone?>`). | fields: version,encoding,standalone |
| `Attr` | src/xmllinq/mod.rs:309 | private | An attribute (name + value). | fields: name,value |
| `PiData` | src/xmllinq/mod.rs:317 | private | Processing-instruction payload. | fields: target,data |
| `NodeData` | src/xmllinq/mod.rs:340 | private | _no doc comment_ | fields: kind,parent,content,attrs |
| `Dom` | src/xmllinq/mod.rs:375 | pub | The arena holding every XML node the engine constructs while comparing or editing a part. | fields: nodes,annotations |
| `Parser` | src/xmllinq/parse.rs:127 | private | _no doc comment_ | fields: dom,c,pos |
| `NsScope` | src/xmllinq/parse.rs:141 | private | In-scope namespace bindings as an overlay stack (N02): one `HashMap` holds the live prefix → URI bindings, and each element's declarations push overrides that `pop` restores — where the old port cloned the entire parent map on every element that declared an `xmlns`, this pays only the declared bytes, and nested rebind... | fields: bindings,undo,live_bytes,live_bindings |
| `TagPolicy` | src/xmllinq/parse.rs:197 | pub(crate) | The checked gate's per-document tag policy (N01/N02): tag width, attribute counts and sizes, namespace budgets, and expanded-name duplicate detection. | fields: scope,marks,declared_bytes,html_attributes |
| `State` | src/xmllinq/serialize.rs:198 | private | Namespace prefix generator state. | fields: counter |
| `Scope` | src/xmllinq/serialize.rs:204 | private | A scope in the namespace-prefix stack. | fields: parent,local_uri_to_prefix,local_prefix_to_uri |
| `HashXmlBuf` | src/xmllinq/serialize.rs:464 | private | Stream serialized XML into SHA-1, stripping the first WML default xmlns from the root open tag (same as `replacen` on the full string). | fields: hasher,open,past_root_open |
