# Modules

155 items (non-test). Part of the [symbol manifest](README.md).

| Name | Defined at | Vis | Does | Detail |
|---|---|---|---|---|
| `check` | src/bin/jubarte.rs:34 | private | _no doc comment_ |  |
| `edit_template` | src/bin/jubarte.rs:36 | private | _no doc comment_ |  |
| `edit_typed` | src/bin/jubarte.rs:38 | private | _no doc comment_ |  |
| `discovery` | src/capabilities.rs:19 | private | _no doc comment_ |  |
| `logical` | src/changes.rs:31 | private | _no doc comment_ |  |
| `edit_actions` | src/cli.rs:9 | pub | Typed edit action grammar, independent of legacy flag grouping. |  |
| `edit_template` | src/cli.rs:11 | pub | Desired-text template generation grammar. |  |
| `atomize` | src/comparer/mod.rs:30 | pub | _no doc comment_ |  |
| `atoms` | src/comparer/mod.rs:31 | pub | _no doc comment_ |  |
| `bookmarks` | src/comparer/mod.rs:32 | pub | _no doc comment_ |  |
| `comments` | src/comparer/mod.rs:33 | pub | _no doc comment_ |  |
| `cross_para` | src/comparer/mod.rs:34 | pub | _no doc comment_ |  |
| `finalize` | src/comparer/mod.rs:38 | pub | Word-parity passes, public for the tests that run one at a time; outside the version promise (docs/VERSIONING.md). _(from a code comment)_ |  |
| `fixups` | src/comparer/mod.rs:39 | pub | _no doc comment_ |  |
| `footnotes` | src/comparer/mod.rs:40 | pub | _no doc comment_ |  |
| `formatchg` | src/comparer/mod.rs:41 | pub | _no doc comment_ |  |
| `lcs` | src/comparer/mod.rs:42 | pub | _no doc comment_ |  |
| `lcs_table` | src/comparer/mod.rs:43 | pub | _no doc comment_ |  |
| `moves` | src/comparer/mod.rs:44 | pub | _no doc comment_ |  |
| `order_tables` | src/comparer/mod.rs:46 | pub | Hand order tables for Ring 1½ schema oracle (`tests/schema_consistency.rs`). |  |
| `parts` | src/comparer/mod.rs:47 | pub | _no doc comment_ |  |
| `preprocess` | src/comparer/mod.rs:48 | pub | _no doc comment_ |  |
| `produce` | src/comparer/mod.rs:49 | pub | _no doc comment_ |  |
| `revisions` | src/comparer/mod.rs:50 | pub | _no doc comment_ |  |
| `tables` | src/comparer/mod.rs:54 | pub | Constant tables ported from the C# `WmlComparer`; outside the version promise (docs/VERSIONING.md). _(from a code comment)_ |  |
| `textbox` | src/comparer/mod.rs:55 | pub | _no doc comment_ |  |
| `units` | src/comparer/mod.rs:56 | pub | _no doc comment_ |  |
| `altchunk` | src/convert/mod.rs:31 | pub(crate) | _no doc comment_ |  |
| `diff_render` | src/convert/mod.rs:32 | private | _no doc comment_ |  |
| `font` | src/convert/mod.rs:33 | private | _no doc comment_ |  |
| `font_table` | src/convert/mod.rs:34 | private | _no doc comment_ |  |
| `math_font` | src/convert/mod.rs:35 | private | _no doc comment_ |  |
| `math_layout` | src/convert/mod.rs:36 | private | _no doc comment_ |  |
| `metafile` | src/convert/mod.rs:37 | private | _no doc comment_ |  |
| `pdf` | src/convert/mod.rs:38 | private | _no doc comment_ |  |
| `preset_geom` | src/convert/mod.rs:39 | private | _no doc comment_ |  |
| `preset_geom_data` | src/convert/mod.rs:40 | private | _no doc comment_ |  |
| `preset_text_rect_data` | src/convert/mod.rs:41 | private | _no doc comment_ |  |
| `raster` | src/convert/mod.rs:42 | private | _no doc comment_ |  |
| `word_subst` | src/convert/mod.rs:43 | private | _no doc comment_ |  |
| `diff` | src/debug.rs:54 | pub | _no doc comment_ |  |
| `render` | src/debug.rs:55 | private | _no doc comment_ |  |
| `anchor` | src/edit.rs:39 | private | _no doc comment_ |  |
| `anchor_error` | src/edit.rs:40 | private | _no doc comment_ |  |
| `controls` | src/edit.rs:41 | private | _no doc comment_ |  |
| `flags` | src/edit.rs:42 | pub | _no doc comment_ |  |
| `hint` | src/edit.rs:43 | private | _no doc comment_ |  |
| `images` | src/edit.rs:44 | private | _no doc comment_ |  |
| `notes` | src/edit.rs:45 | private | _no doc comment_ |  |
| `origin` | src/edit.rs:46 | private | _no doc comment_ |  |
| `prepared` | src/edit.rs:49 | pub | One preparation path for dry run and apply: the chosen primary bytes and the checks the serialized candidate passes (V2-08). |  |
| `ranges` | src/edit.rs:50 | private | _no doc comment_ |  |
| `record` | src/edit.rs:51 | pub | _no doc comment_ |  |
| `refusal` | src/edit.rs:53 | pub | Caller-specific, neutral review choices for revision-blocked edits. |  |
| `replacement_order` | src/edit.rs:54 | private | _no doc comment_ |  |
| `resolution` | src/edit.rs:55 | pub | _no doc comment_ |  |
| `rewrite` | src/edit.rs:56 | private | _no doc comment_ |  |
| `runs` | src/edit.rs:57 | private | _no doc comment_ |  |
| `sections` | src/edit.rs:58 | private | _no doc comment_ |  |
| `skip` | src/edit.rs:59 | private | _no doc comment_ |  |
| `structural` | src/edit.rs:60 | private | _no doc comment_ |  |
| `tracked` | src/edit.rs:61 | private | _no doc comment_ |  |
| `visible` | src/edit.rs:62 | private | _no doc comment_ |  |
| `watermark` | src/edit.rs:63 | private | _no doc comment_ |  |
| `whole` | src/edit.rs:64 | private | _no doc comment_ |  |
| `comment_settlement` | src/edit.rs:66 | private | _no doc comment_ |  |
| `inline` | src/edit.rs:69 | pub | Compile typed inline CLI actions into the same source-bound PlanV2 contract. |  |
| `v2` | src/edit.rs:71 | pub | Source-bound PlanV2 text-and-review requests and exact logical text codec. |  |
| `v2_apply` | src/edit.rs:74 | pub | _no doc comment_ |  |
| `for_edit` | src/edit.rs:77 | pub | `read --for-edit`: the working document, hashes and readiness (V2-10). |  |
| `native_keys` | src/edit.rs:81 | pub(crate) | The one native-key policy (`w14:paraId`, revision `w:id`) shared by generation, application and the expectations checker for paired long IDs. |  |
| `source_refs` | src/edit.rs:82 | private | _no doc comment_ |  |
| `template` | src/edit.rs:84 | pub | Source-bound desired-text template generation over the for-edit view. |  |
| `short` | src/ids.rs:39 | private | _no doc comment_ |  |
| `tables` | src/inspect.rs:26 | private | _no doc comment_ |  |
| `admission` | src/lib.rs:96 | pub | _no doc comment_ |  |
| `append` | src/lib.rs:97 | pub | _no doc comment_ |  |
| `audit` | src/lib.rs:98 | pub | _no doc comment_ |  |
| `builtin_styles` | src/lib.rs:100 | private | Word's built-in style names. |  |
| `capabilities` | src/lib.rs:102 | pub | Machine-readable manifest of what this build can do. |  |
| `changes` | src/lib.rs:104 | pub | Tracked changes one at a time: list, accept or reject a selection. |  |
| `cli` | src/lib.rs:106 | pub | _no doc comment_ |  |
| `comments` | src/lib.rs:107 | pub | _no doc comment_ |  |
| `comparer` | src/lib.rs:109 | pub | Core WmlComparer engine (atomize → LCS → produce → finalize). |  |
| `comparison_log` | src/lib.rs:111 | pub | Structured comparison log (info / warning / error entries). |  |
| `convert` | src/lib.rs:113 | pub | Independent DOCX → PDF conversion (not LibreOffice). |  |
| `debug` | src/lib.rs:115 | pub | `jubarte debug`: short Word-validity triage of a package, or two compared. |  |
| `document_comparer` | src/lib.rs:117 | pub | Byte-level package API: compare, list, accept, and reject revisions. |  |
| `edit` | src/lib.rs:118 | pub | _no doc comment_ |  |
| `expect` | src/lib.rs:120 | pub | V2-13 (issue #845): the checker core for `--expect` evaluations. |  |
| `fields` | src/lib.rs:121 | pub | _no doc comment_ |  |
| `ids` | src/lib.rs:123 | pub | The one grammar of change, comment and paragraph ids (`r12`, `c5`, `p12`). |  |
| `inspect` | src/lib.rs:125 | pub | Read-only paragraph/package views and the Markdown projection for agents. |  |
| `legacy_doc` | src/lib.rs:126 | pub | _no doc comment_ |  |
| `markdown` | src/lib.rs:127 | pub | _no doc comment_ |  |
| `markup_simplifier` | src/lib.rs:129 | pub | Markup simplification (PowerTools `MarkupSimplifier` port). |  |
| `namespaces` | src/lib.rs:131 | pub | WordprocessingML and related namespace / `XName` constants. |  |
| `opc` | src/lib.rs:133 | pub | Open Packaging Conventions adapter (`PartFs`). |  |
| `perf` | src/lib.rs:135 | pub | P0-LAB-01 stage counters/timers — no-ops unless `perf-profile` is enabled. |  |
| `result` | src/lib.rs:138 | pub | Machine result envelope for the v2 routes (`edit apply`, `edit template`, `review accept`, and friends). |  |
| `revision_processor` | src/lib.rs:140 | pub | Accept / reject tracked revisions across a package. |  |
| `scrub` | src/lib.rs:141 | pub | _no doc comment_ |  |
| `settings` | src/lib.rs:142 | pub | _no doc comment_ |  |
| `strict_translation` | src/lib.rs:144 | pub | ISO Strict → Transitional package normalization. |  |
| `text_diff` | src/lib.rs:145 | pub | _no doc comment_ |  |
| `unid` | src/lib.rs:146 | pub | _no doc comment_ |  |
| `update` | src/lib.rs:149 | pub | `jubarte self-update`: install a GitHub release, only when asked. |  |
| `util` | src/lib.rs:151 | pub | Shared small utilities. |  |
| `validate` | src/lib.rs:152 | pub | _no doc comment_ |  |
| `wml_document` | src/lib.rs:154 | pub | `WmlDocument` — document bytes + lazily parsed main part. |  |
| `word_default_theme` | src/lib.rs:156 | private | Word's default theme part, for originals without one. |  |
| `xmllinq` | src/lib.rs:158 | pub | Arena DOM (`xmllinq`) used by the comparer. |  |
| `publication` | src/lib.rs:165 | pub | Capability-confined, staged filesystem publication for host applications. |  |
| `agent` | src/markdown/from_docx/mod.rs:16 | private | _no doc comment_ |  |
| `critic` | src/markdown/from_docx/mod.rs:17 | private | _no doc comment_ |  |
| `drawing` | src/markdown/from_docx/mod.rs:18 | pub(crate) | _no doc comment_ |  |
| `header` | src/markdown/from_docx/mod.rs:19 | private | _no doc comment_ |  |
| `media` | src/markdown/from_docx/mod.rs:20 | private | _no doc comment_ |  |
| `mermaid` | src/markdown/from_docx/mod.rs:21 | private | _no doc comment_ |  |
| `ooxml` | src/markdown/from_docx/mod.rs:22 | pub(crate) | _no doc comment_ |  |
| `revise` | src/markdown/from_docx/mod.rs:23 | private | _no doc comment_ |  |
| `anchor` | src/markdown/mod.rs:38 | private | _no doc comment_ |  |
| `borders` | src/markdown/mod.rs:39 | pub(crate) | _no doc comment_ |  |
| `critic` | src/markdown/mod.rs:40 | private | _no doc comment_ |  |
| `data_stores` | src/markdown/mod.rs:41 | pub(crate) | _no doc comment_ |  |
| `diff` | src/markdown/mod.rs:42 | private | _no doc comment_ |  |
| `frames` | src/markdown/mod.rs:43 | pub(crate) | _no doc comment_ |  |
| `from_docx` | src/markdown/mod.rs:44 | private | _no doc comment_ |  |
| `math` | src/markdown/mod.rs:45 | private | _no doc comment_ |  |
| `package` | src/markdown/mod.rs:46 | pub(crate) | _no doc comment_ |  |
| `pages` | src/markdown/mod.rs:47 | private | _no doc comment_ |  |
| `patch` | src/markdown/mod.rs:48 | private | _no doc comment_ |  |
| `redline` | src/markdown/mod.rs:49 | private | _no doc comment_ |  |
| `unified` | src/markdown/mod.rs:50 | private | _no doc comment_ |  |
| `view_notes` | src/markdown/mod.rs:51 | private | _no doc comment_ |  |
| `view_stories` | src/markdown/mod.rs:52 | private | _no doc comment_ |  |
| `view_styles` | src/markdown/mod.rs:53 | private | _no doc comment_ |  |
| `write` | src/markdown/mod.rs:54 | private | _no doc comment_ |  |
| `xml` | src/markdown/mod.rs:55 | pub(crate) | _no doc comment_ |  |
| `archive` | src/opc/mod.rs:103 | private | _no doc comment_ |  |
| `zip32` | src/opc/mod.rs:104 | private | _no doc comment_ |  |
| `active` | src/perf.rs:218 | private | _no doc comment_ |  |
| `codes` | src/result.rs:102 | pub | Stable error codes the proposal names explicitly. |  |
| `bookmarks` | src/revision_processor.rs:20 | private | _no doc comment_ |  |
| `comments` | src/revision_processor.rs:21 | pub(crate) | _no doc comment_ |  |
| `notes` | src/revision_processor.rs:22 | private | _no doc comment_ |  |
| `sections` | src/revision_processor.rs:23 | private | _no doc comment_ |  |
| `style_records` | src/revision_processor.rs:24 | pub(crate) | _no doc comment_ |  |
| `word_save` | src/revision_processor.rs:25 | private | _no doc comment_ |  |
| `datetime` | src/util/mod.rs:23 | pub(crate) | _no doc comment_ |  |
| `group_adjacent` | src/util/mod.rs:24 | pub | _no doc comment_ |  |
| `sha1` | src/util/mod.rs:25 | pub | _no doc comment_ |  |
| `words` | src/util/mod.rs:26 | pub | _no doc comment_ |  |
| `parse` | src/xmllinq/mod.rs:36 | pub | _no doc comment_ |  |
| `serialize` | src/xmllinq/mod.rs:37 | pub | _no doc comment_ |  |
