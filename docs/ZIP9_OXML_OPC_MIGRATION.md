<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# zip 9 and oxml-opc migration (#524)

Source baseline: `4329bd2c490f006239c187b0254deb5480e960cc`.
Read issue [#524](https://github.com/jandira-tech/jubarte-redlines/issues/524),
especially the 2026-10-17 update, and
[#772](https://github.com/jandira-tech/jubarte-redlines/issues/772).
This map was checked against the published sources of `rdocx-opc 0.2.0`,
`oxml-opc 0.14.2`, `zip 8.6.0`, and `zip 9.0.4`, not an older migration plan.
Run artifacts, source inspection, commands, input hashes and before/after
measurements live under `/tmp/zip9/`. The root, Python, WASM, in-process and
fuzz manifests are separate workspaces.

## Dependency and feature selection

| Old item | New item | Action |
| --- | --- | --- |
| `rdocx-opc = "0.2"` / `rdocx_opc` | `oxml-opc = "0.14.2"` / `oxml_opc` | Disable defaults; enable no optional features. Use upstream public types. |
| `zip 8.6.0`, defaults off, `deflate` | `zip 9.0.4`, defaults off, `deflate` | Keep the same pure-Rust zlib-rs and Zopfli backend features, avoiding a compressor change. |
| `self_update 1.3.0` | `self_update >=1.3.1` | Update both target tables and the targeted lock entry; its Windows ZIP graph now requires zip 9. |
| root path dependency in bindings/fuzz | same path dependency | No direct ZIP/OPC pin exists in these manifests. Refresh all four tracked locks; never commit fuzz's lock. |
| `ooxmlsdk` dev oracle | locked 0.13.2 | The `parts` feature still requires zip 8.6.0. Keep and report this advisory edge; do not change deny policy. |

`oxml-opc 0.14.2` has **`default = []`**. Unlike the older upstream snapshots
mentioned in issue comments, its default graph enables none of its optional
crypto dependencies:

| Feature | Optional dependencies enabled |
| --- | --- |
| `agile-encryption` | aes, base64, cbc (alloc/block-padding), cfb, getrandom, hmac, sha1, sha2 |
| `digital-signatures` | base64, ring (std, defaults off), sha2, x509-cert (std, defaults off) |
| none (selected) | none of the above; mandatory quick-xml 0.41, thiserror 2, zip ^9.0.3 with `deflate-flate2-zlib-rs` |

The direct `deflate` feature adds `deflate-zopfli` and
`deflate-flate2-zlib-rs`, just as zip 8 did. No C compressor, ZIP encryption,
legacy codecs, or ZIP `time` feature is needed by the engine. Native CLI TLS
can still bring ring through self_update/rustls independently. This must not
be confused with oxml-opc's disabled signature feature. Verify the WASM target
with the binding manifest, rather than assuming native compilation proves it.

## OPC healing changes

All direct old OPC imports are in `src/opc/mod.rs`. Public re-exports remain
`jubarte::opc::{OpcError, Relationship, Relationships}` with the new upstream
source. Relationship fields and existing method signatures remain available.
`OpcError` gains `DuplicatePartName`, `PackageLimitExceeded` and
`InvalidXmlCharacter`; exhaustive downstream enum matches may need updating.
The XML/attribute error payloads still use quick-xml 0.41. Existing
`refused()` continues wrapping typed admission errors in `OpcError::Io`.

| Call site / old item | New item and required behavior |
| --- | --- |
| imports and `PartFs.pkg: OpcPackage` | Import upstream oxml_opc types; preserve leading-slash map keys. |
| `OpcPackage::from_reader` in `PartFs::open` | The new whole-package reader rejects duplicate relationship IDs before our repair, normalizes part identities and preserves source XML. Load ZIP entries into an upstream `OpcPackage::new()` through the compatibility adapter, parsing metadata through upstream types, so existing repair/refusal policies remain reachable. |
| `reparse_explicitly_closed`, `fold_explicit_closes` | Retain explicit-close folding in the compatibility metadata reader; upstream also understands closes, but the adapter's record selection must retain the baseline behavior. |
| `Relationships::from_xml` / `unescape_relationships` | Upstream decodes entities and normalizes attributes. Do not decode again: a literal `&amp;` in a target would be decoded twice. Parse selected legacy records individually through upstream, retain record order and duplicates until `drop_repeated_relationships`, and seed allocation with `add_with_id`. |
| `drop_repeated_relationships`, repaired IDs | Keep identical-ID repair and conflicting-ID refusal, before upstream serialization validates unique IDs. |
| `ContentTypes::from_xml` / `unescape_content_types` | Upstream decodes attributes, checks namespaces and rejects repeated case-insensitive identities. Parse the adapter-selected Default/Override records individually and merge their maps, preserving baseline duplicate overwrite behavior; remove the second unescape. |
| package/part relationships `to_xml` in `PartFs::to_zip` | Upstream returns original XML if unchanged. Serialize a fresh upstream Relationships holding the same items, producing baseline canonical indentation, attribute order and declaration. Do not use upstream whole-package `write_to`, whose graph validation/output policy differs. |
| `content_types.to_xml` | Same sorted canonical writer; new identity validation may refuse case-colliding mappings. Report such behavior; never silently discard a mapping. |
| `part_name_to_rels_path` | Still a private fn in upstream's private package module. Keep the local copy. |
| `get_part`, `set_part`, map access, `get_part_rels`, `get_or_create_part_rels` | Same items available. Upstream lookups now match OPC names case-insensitively; existing map edits preserve the stored spelling. |
| `content_type_for`, `add_default`, `add_override` | Same methods; upstream matches identities case-insensitively and normalizes part paths. Standard-conforming lookup improvement; document edge-case drift rather than hiding it. |
| `resolve_rel_target`, `main_document_part` | Same static/method surface; upstream normalizes dot segments and accepts Strict relationships too. Keep the adapter's canonical no-leading-slash result. |
| `Relationships::new`, `Default`, `add`, `items` | New still allocates rId1, derived Default starts rId0. Preserve explicit new for new part scopes. Upstream avoids collisions and wraps exhausted numeric IDs instead of the old overflow panic. |
| all indirect OPC consumers in src/tests/fuzz | Continue using jubarte's re-exports. Fuzz relationships target tests the new upstream parser; no local shadow types or crypto features. |

Attribute whitespace normalization (literal CR/LF/TAB to spaces, encoded
references retained according to XML rules), rejection of duplicate content-type
identities at serialization, case-insensitive upstream lookups and collision-safe
relationship allocation are upstream semantic changes. They are distinct from
ZIP byte determinism and must be reported if they affect measurements. No golden
or existing assertion may be changed to conceal them.

## ZIP reader healing changes

| Old API | zip 9 API | Required action |
| --- | --- | --- |
| `ZipFile::name() -> &str` | `ZipResult<Cow<'_, str>>` | Decode once; propagate/refuse or retain the caller's established fallback; own before mutable reads or dropping the entry. |
| `file_names() -> Iterator<&str>` | `Iterator<ZipResult<Cow<str>>>` | Collect fallibly or explicitly expect valid fixture names. Never silently flatten errors in package inventory. |
| `name_for_index(i) -> Option<&str>` | `Option<ZipResult<Cow<str>>>` | Distinguish missing index from decode failure; transpose before collecting source order. |
| `ZipArchive::new`, `by_index`, `by_index_raw`, `by_name`, `index_for_name`, `len`, `is_dir`, `size`, `compressed_size`, `encrypted`, `compression`, `Read` | same APIs used here | Keep existing bounds, duplicate checks, compression refusal and read policies. |
| `ZipError` into io::Error | still implemented | The Markdown writer's `?` conversions remain valid. Error text on corrupt streams can differ; preserve typed refusal. |

In zip 9.0.4 `ZipFileData::name` tries UTF-8, then CP437. CP437 maps every
byte, so invalid UTF-8 (even with bit 11 set) does **not** establish an error
case. Do not invent an impossible undecodable-name regression test. Handle the
fallible contract for forward compatibility. zip 8 decoded invalid UTF-8 lossily;
zip 9 preserves it as CP437. This is an intentional standards-compatible name
interpretation change on malformed/legacy inputs, not byte identity for those
inputs; ordinary UTF-8 names must retain their bytes.

| Production site | Error/fallback and ownership |
| --- | --- |
| `src/admission.rs`: both entry loops | Map decode failures to InvalidPackage before path/duplicate/metadata checks; own the second-loop name before bounded inflation. |
| `src/opc/mod.rs`: metadata guard | Propagate ZIP decode errors before selecting XML; no entry bypasses the guard. |
| `src/opc/mod.rs`: compatibility loader, source-order map | Propagate errors; own names; retain exact source order. Missing indices and errors are distinct. |
| `src/strict_translation.rs`: rebuilding loop | A decode failure returns the complete original bytes, never a partial archive. |
| `src/scrub.rs::leaks` | Keep its best-effort skip policy when decoding or reading an entry fails. |
| `src/debug.rs::Package::open` | Own decoded name first, return String error with entry index, retain name in read-error context. |
| `src/markdown/from_docx/ooxml.rs::Package::open` | Collect all decoded names as owned Strings; map failure to ConvertError::Invalid. |

Fixture helpers and embedded/integration tests must explicitly expect successful
name decoding and use `into_owned` when retaining names. Keep the best-effort
`by_index` skip in `tests/m66_final_sect_footer_adopt.rs`; do not alter assertions.
String match arms use a decoded Cow's `as_ref()`. Appendix A inventories every
direct reference (including unchanged writers); Appendix B inventories changed
name expressions, including those that have no explicit `zip::` on the line.
DOM/XML `name` calls and the unrelated CLI directory helper are not ZIP APIs.

## Writer invariants

`ZipWriter`, `SimpleFileOptions`, `start_file`, `write_all`, `finish`,
`CompressionMethod::{Stored,Deflated}` and `DateTime::default` retain the surface
used here. Keep all writer call sites unchanged except where the OPC source
preservation above requires canonical metadata serialization.

- PartFs: `[Content_Types].xml`, `_rels/.rels`, then source-ranked entries,
  then new names sorted; Deflate `Some(1)` and explicit 1980-01-01 00:00:00.
- Strict translation and Markdown package construction: preserve their existing
  entry ordering and default Deflate level, with explicit 1980 timestamps.
- Fixture writers: retain their existing compression, level and time choices.
  No global policy change to test builders is part of this migration.
- Keep flags, local/central extra fields, data-descriptor behavior and Zip64
  defaults. Seekable in-memory writers continue patching local headers. No
  forced descriptor, streaming, raw-copy or large-file mode is introduced.
- zip 8 and 9 use the same selected Deflate backend. Full output hashes and
  per-entry metadata/payload hashes must verify this, rather than assuming it.
- When a feature graph enables ZIP time (notably Windows updater features),
  explicit timestamps in engine writers still ensure determinism.

## Appendix A: every direct crate reference at the baseline

Line numbers refer to the baseline commit, so this inventory remains useful
after healing edits shift lines. Unchanged constructors, writer options, ZIP
errors, metadata and test builders are covered by the API rows above.

```text
tests/m434_image_doc_last_mix_keeps_live_numpr.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m311_textless_next_layout_pure_id.rs:40:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m397_blue_underline_x_ooxml_bold.rs:68:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m397_blue_underline_x_ooxml_bold.rs:113:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m4h_parts.rs:120:        let mut z = zip::ZipWriter::new(Cursor::new(&mut src_buf));
tests/m4h_parts.rs:121:        let opt = zip::write::SimpleFileOptions::default();
tests/m379_tiff_b_notes_separators.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/debug/diff.rs:1627:            let mut z = zip::ZipWriter::new(&mut buf);
src/debug/diff.rs:1628:            let opt = zip::write::SimpleFileOptions::default();
src/debug/diff.rs:1973:            let mut z = zip::ZipWriter::new(&mut buf);
src/debug/diff.rs:1974:            let opt = zip::write::SimpleFileOptions::default();
tests/m115_both_sides_short_residual.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_table_margin_default_style.rs:16:use zip::ZipWriter;
tests/m_table_margin_default_style.rs:17:use zip::write::SimpleFileOptions;
tests/m_table_margin_default_style.rs:64:    zip::ZipArchive::new(Cursor::new(out))
tests/m474_real_field_ins_keeps_heading_ppr.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m348_eigenpal_free_mesh.rs:64:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m21_header_footer_diff.rs:22:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m21_header_footer_diff.rs:23:        let opt = zip::write::SimpleFileOptions::default();
tests/m21_header_footer_diff.rs:68:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m460_free_mesh_bookended_ins_del.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m410_ooxml_x_alpha_list_pure_id.rs:73:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/compare_is_reproducible.rs:29:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/compare_is_reproducible.rs:30:        let opt = zip::write::SimpleFileOptions::default();
tests/compare_is_reproducible.rs:87:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/compare_is_reproducible.rs:94:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m371_mixed_simple_heading_d_no_fold.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/cli_default_stamp.rs:59:    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
tests/cli_default_stamp.rs:60:    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
tests/cli_default_stamp.rs:61:    let options = zip::write::SimpleFileOptions::default()
tests/cli_default_stamp.rs:62:        .compression_method(zip::CompressionMethod::Deflated)
tests/cli_default_stamp.rs:63:        .last_modified_time(zip::DateTime::default());
tests/cli_default_stamp.rs:82:    let mut zip = zip::ZipArchive::new(std::fs::File::open(docx).unwrap()).unwrap();
tests/perf_atom_stack_footnote_path.rs:125:            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
tests/perf_atom_stack_footnote_path.rs:126:            let opt = zip::write::FileOptions::<()>::default()
tests/perf_atom_stack_footnote_path.rs:127:                .compression_method(zip::CompressionMethod::Stored);
tests/perf_atom_stack_footnote_path.rs:214:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out[..])).unwrap();
tests/m62_footer_part_path_reconcile.rs:33:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m62_footer_part_path_reconcile.rs:74:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m62_footer_part_path_reconcile.rs:89:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m83_trailing_empty_ins_and_last_del_spacing.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/markdown_to_docx.rs:596:            let zip = zip::ZipArchive::new(Cursor::new(&written.docx)).unwrap();
tests/markdown_to_docx.rs:597:            zip.file_names().any(|n| n == "word/media/image1.png")
tests/m107_connector_residual_pair.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m8_strict.rs:37:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m8_strict.rs:38:        let opt = zip::write::SimpleFileOptions::default();
tests/m8_strict.rs:139:    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("valid zip/docx");
tests/m473_restamp_del_mark_onto_del_only_para.rs:106:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m135_off_diag_body_over_ordered_prefix.rs:28:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m315_single_para_wrap_x_employment.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m315_single_para_wrap_x_employment.rs:75:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m396_comprehensive_x_strikethrough_demo.rs:68:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m475_propertyless_ppr.rs:18:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).expect("zip");
tests/m_identical_missing_main_part.rs:14:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m_identical_missing_main_part.rs:15:        let opt = zip::write::SimpleFileOptions::default();
tests/m25_sanitize_sdt_pr.rs:25:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m25_sanitize_sdt_pr.rs:26:        let opt = zip::write::SimpleFileOptions::default();
tests/m25_sanitize_sdt_pr.rs:56:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
src/debug/render.rs:1270:            let mut zip = zip::ZipWriter::new(&mut out);
src/debug/render.rs:1273:                zip::write::SimpleFileOptions::default(),
src/debug/render.rs:1278:                zip.start_file(*name, zip::write::SimpleFileOptions::default())
tests/m_count_gate_junction.rs:27:    zip::ZipArchive::new(Cursor::new(out))
tests/append_comments.rs:128:    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/append_comments.rs:129:    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
tests/append_comments.rs:130:    let opts = zip::write::SimpleFileOptions::default();
tests/m346_ooxml_title_peel_before_free_mesh.rs:83:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_story_final_mark.rs:19:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m_story_final_mark.rs:20:        let opt = zip::write::SimpleFileOptions::default();
tests/m_story_final_mark.rs:75:    zip::ZipArchive::new(Cursor::new(out))
tests/m66_final_sect_footer_adopt.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m66_final_sect_footer_adopt.rs:32:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m66_final_sect_footer_adopt.rs:57:        let mut zip = zip::ZipArchive::new(Cursor::new(out.to_vec())).unwrap();
tests/m327_title_last_sig_free_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m67_pure_del_heading_residual_spacing.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
src/convert/altchunk.rs:1077:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/altchunk.rs:1080:            zip::write::SimpleFileOptions::default(),
src/convert/altchunk.rs:1085:            zip.start_file(*name, zip::write::SimpleFileOptions::default())
tests/m105_subtitle_document_peel.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m338_short_cell_table_x_report_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m427_tab_x_diff_after7_pure_id.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m64_normal_ind_and_pure_del_before.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/append_documents.rs:548:    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/append_documents.rs:549:    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
tests/append_documents.rs:550:    let opts = zip::write::SimpleFileOptions::default();
tests/m46_strip_default_sect_type.rs:11:use zip::ZipWriter;
tests/m46_strip_default_sect_type.rs:12:use zip::write::SimpleFileOptions;
tests/m46_strip_default_sect_type.rs:60:    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
tests/m51_accept_pre_tc_package.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/m147_comments_union_carryover.rs:24:use zip::ZipWriter;
tests/m147_comments_union_carryover.rs:25:use zip::write::SimpleFileOptions;
tests/m147_comments_union_carryover.rs:235:        use zip::ZipWriter;
tests/m147_comments_union_carryover.rs:236:        use zip::write::SimpleFileOptions;
tests/m381_file54_short_b_demo_title_mix.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m369_ordered_sublist_short_label_zip.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m124_short_pure_ins_no_multi_del_fold.rs:15:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m482_numbering_collision_remap.rs:18:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes.to_vec())).unwrap();
tests/m364_orphan_comment_no_fold_into_prose.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m373_toc_short_a_no_heading_mix.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m10_deleted_drawing.rs:39:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).expect("valid docx");
tests/m9_roundtrip.rs:44:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).expect("valid docx");
tests/m76_stamp_confetti_before_word_run.rs:29:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m462_no_a_styles_factory_docdefaults.rs:60:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m351_ooxml_x_short_prose_free_mesh.rs:67:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m352_pure_ins_keeps_line276_spacing.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m131_short_into_long_content_nest.rs:14:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m467_normal_rpr_delta_only.rs:61:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m377_tiff_title_shared_token_eq.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m368_sdts_shape_drawing_before_mix.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m451_strip_empty_pprchange_mix_jc.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m161_unit_count.rs:37:        let mut z = zip::ZipArchive::new(std::io::Cursor::new(data_a)).unwrap();
tests/m161_unit_count.rs:44:        let mut z = zip::ZipArchive::new(std::io::Cursor::new(data_b)).unwrap();
tests/md_reader_id_lines.rs:1531:    zip::ZipArchive::new(std::io::Cursor::new(docx))
tests/md_reader_id_lines.rs:2022:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).expect("a docx zip");
tests/m337_both_table_multi_tbl_full_lcs.rs:66:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m337_both_table_multi_tbl_full_lcs.rs:111:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m146_wholedoc_replacement_no_fold.rs:15:use zip::ZipWriter;
tests/m146_wholedoc_replacement_no_fold.rs:16:use zip::write::SimpleFileOptions;
tests/m146_wholedoc_replacement_no_fold.rs:56:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m386b_heading4_helvetica_body_mix.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m466_period_attach_skips_rprchange.rs:34:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m84_lcr_prefer_content_over_space.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
src/fields.rs:1467:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/fields.rs:1468:        let opt = zip::write::SimpleFileOptions::default();
tests/m428_list_def_mix_x_numbering_pure_id.rs:40:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m438_spaces_trailing_empty_after_pure_d.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m468_no_fold_across_trailing_empties.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m463_math_internal_revision_marks.rs:47:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m325_list_item_x_demo_title_no_mix.rs:35:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m459_free_mesh_wholesale_body.rs:44:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m350_ooxml_x_rtl_table_free_mesh.rs:67:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/admission.rs:37:use zip::{CompressionMethod, ZipArchive};
src/admission.rs:684:    use zip::write::SimpleFileOptions;
src/admission.rs:693:        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/admission.rs:792:        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/admission.rs:888:        let mut out = zip::ZipWriter::new_append(Cursor::new(docx_with(&[]))).unwrap();
tests/m92_trailing_empty_spacing_pprchange.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m92_trailing_empty_spacing_pprchange.rs:118:        let mut z = zip::ZipWriter::new(&mut buf);
tests/m92_trailing_empty_spacing_pprchange.rs:119:        let opt = zip::write::SimpleFileOptions::default();
tests/m471_rotate_ins_mark_del_only_para.rs:61:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_textbox_text_change.rs:23:use zip::ZipWriter;
tests/m_textbox_text_change.rs:24:use zip::write::SimpleFileOptions;
tests/s1_style_chain_resolution.rs:43:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/s1_style_chain_resolution.rs:194:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/s1_style_chain_resolution.rs:195:        let opt = zip::write::SimpleFileOptions::default();
tests/m22_alternate_content_resolve.rs:28:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m22_alternate_content_resolve.rs:29:        let opt = zip::write::SimpleFileOptions::default();
tests/m22_alternate_content_resolve.rs:67:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_styles_matched_by_name.rs:18:use zip::ZipWriter;
tests/m_styles_matched_by_name.rs:19:use zip::write::SimpleFileOptions;
tests/m_styles_matched_by_name.rs:64:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_styles_matched_by_name.rs:298:        let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
tests/m_list_definition_change.rs:17:use zip::ZipWriter;
tests/m_list_definition_change.rs:18:use zip::write::SimpleFileOptions;
tests/m_list_definition_change.rs:86:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m460_heading_line_stamp_gate.rs:47:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m102_jc_addition_pprchange.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m59_pure_del_spacing_and_normal_clear.rs:26:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).unwrap();
tests/m43_strip_dangling_pstyle.rs:19:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m43_strip_dangling_pstyle.rs:30:        let mut zip = zip::ZipWriter::new(&mut buf);
tests/m43_strip_dangling_pstyle.rs:31:        let opts = zip::write::SimpleFileOptions::default();
tests/m42c_eigenpal_pkg.rs:9:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/md_review_probes.rs:79:    let mut zip = zip::ZipArchive::new(file).unwrap();
tests/m469_split_head_title_mix.rs:40:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/strict_translation.rs:43:use zip::write::SimpleFileOptions;
src/strict_translation.rs:44:use zip::{CompressionMethod, ZipArchive, ZipWriter};
src/strict_translation.rs:820:        .last_modified_time(zip::DateTime::default());
src/strict_translation.rs:1208:                Some(zip::DateTime::default()),
tests/m449_body_mix_live_jc.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m392_file36_empty_spacers_before_title_del.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m392_file36_empty_spacers_before_title_del.rs:145:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m77_no_fold_unrelated_mid_del.rs:30:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m417_sd2517_x_borderbox_pure_id.rs:76:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m42b_pair02_pkg.rs:10:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m362_stamp_tiff_verdana_no_title_mix.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m462_wholesale_coverage_free_mesh.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m114_ordered_prefix_body_pair.rs:27:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m383_a_only_hf_marked_pure_d.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m476_ladder_word_markup.rs:20:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).expect("zip");
tests/m_accept_own_redlines.rs:1569:    let mut src = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m_accept_own_redlines.rs:1570:    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
tests/m_accept_own_redlines.rs:1571:    let opts = zip::write::SimpleFileOptions::default();
tests/m453_empty_pprchange_heading_spacing.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m436_list_def_mix_keeps_listparagraph.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/convert/mod.rs:49437:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/mod.rs:49440:            zip::write::SimpleFileOptions::default(),
src/convert/mod.rs:56538:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/mod.rs:56541:            zip::write::SimpleFileOptions::default(),
src/convert/mod.rs:59417:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/mod.rs:59433:            zip.start_file(name, zip::write::SimpleFileOptions::default())
src/convert/mod.rs:63291:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/mod.rs:63303:            zip.start_file(name, zip::write::SimpleFileOptions::default())
src/convert/mod.rs:64011:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/mod.rs:64014:            zip::write::SimpleFileOptions::default(),
src/convert/mod.rs:64267:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/convert/mod.rs:64270:            zip::write::SimpleFileOptions::default(),
tests/edit_insert_image.rs:118:        zip::ZipArchive::new(Cursor::new(&result.clean))
tests/m16_relationship_ids.rs:27:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m312_short_table_next_pure_id.rs:99:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m312_short_table_next_pure_id.rs:162:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m312_short_table_next_pure_id.rs:210:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m312_short_table_next_pure_id.rs:254:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m312_short_table_next_pure_id.rs:293:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m444_title_mix_keeps_live_spacing.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m323_hyperlink_table_title_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m339_short_demo_first_title_token_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m339_short_demo_first_title_token_mesh.rs:98:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m339_short_demo_first_title_token_mesh.rs:128:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m14_multibody.rs:23:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m96_last_sig_demo_title_pair.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m365_bookmark_short_list_no_fold_into_prose.rs:55:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m390_fields_test_unwraps_sdts.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m390_fields_test_unwraps_sdts.rs:80:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m37_normal_style_merge.rs:30:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_missing_body_sectpr.rs:15:use zip::ZipWriter;
tests/m_missing_body_sectpr.rs:16:use zip::write::SimpleFileOptions;
tests/m_missing_body_sectpr.rs:54:    zip::ZipArchive::new(Cursor::new(out))
tests/m345_whitespace_fold_keeps_list_ppr.rs:22:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
tests/m111_cascade_normal_style_change.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m111_cascade_normal_style_change.rs:107:    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
tests/m_tblgridchange_no_author_date.rs:30:use zip::ZipWriter;
tests/m_tblgridchange_no_author_date.rs:31:use zip::write::SimpleFileOptions;
tests/m_tblgridchange_no_author_date.rs:90:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m326_ooxml_property_demo_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m20_rel_collision.rs:26:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m20_rel_collision.rs:27:        let opt = zip::write::SimpleFileOptions::default();
tests/m20_rel_collision.rs:62:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m56_no_docdefaults_into_normal.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m148_tolerated_inputs.rs:18:use zip::ZipWriter;
tests/m148_tolerated_inputs.rs:19:use zip::write::SimpleFileOptions;
tests/m99_drawing_residual_stamp_confetti.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m307_demo_x_catalog_mix_carrier.rs:21:use zip::ZipWriter;
tests/m307_demo_x_catalog_mix_carrier.rs:22:use zip::write::SimpleFileOptions;
tests/m307_demo_x_catalog_mix_carrier.rs:68:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m344_heading_over_list_mix_style.rs:35:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m40_footer_style_rpr_merge.rs:33:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m347_both_table_peel_nontbl_titles.rs:68:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_compare_orphan_header_footer.rs:32:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m_compare_orphan_header_footer.rs:87:    let mut zip = zip::ZipArchive::new(Cursor::new(&out[..])).unwrap();
tests/m435_diff16_mix_keeps_heading1.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m308_list_wholesale_pure_id.rs:18:use zip::ZipWriter;
tests/m308_list_wholesale_pure_id.rs:19:use zip::write::SimpleFileOptions;
tests/m308_list_wholesale_pure_id.rs:111:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_table_alignment_3d4318d7e9.rs:62:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).unwrap();
src/bin/jubarte.rs:3420:            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
src/bin/jubarte.rs:3421:            let opt = zip::write::SimpleFileOptions::default();
src/bin/jubarte.rs:3716:    fn file_names(dir: &Path) -> Vec<String> {
src/bin/jubarte.rs:3841:            file_names(dir.path()),
src/bin/jubarte.rs:3870:        assert_eq!(file_names(dir.path()), ["in.docx"], "nothing written");
src/bin/jubarte.rs:3899:            file_names(&out),
src/bin/jubarte.rs:3948:        assert_eq!(file_names(&out), ["diff.json"]);
src/bin/jubarte.rs:3966:            file_names(&out),
tests/m320_support_tickets_first_table_mesh.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m320_support_tickets_first_table_mesh.rs:72:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m375_ordered_shared_label_eq.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/s2_bonly_style_docdefaults_bake.rs:48:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m402_complex2_fields_html_mesh.rs:86:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m58_stamp_confetti_then_replace.rs:106:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m58_stamp_confetti_then_replace.rs:141:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m464_s2_bakes_ppr_spacing.rs:60:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m78_conjoin_prefers_inserted_ppr.rs:31:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m464_peel_for_word_across_mix.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/markdown/from_docx/ooxml.rs:33:    archive: zip::ZipArchive<Cursor<&'a [u8]>>,
src/markdown/from_docx/ooxml.rs:39:        let archive = zip::ZipArchive::new(Cursor::new(bytes))
src/markdown/from_docx/ooxml.rs:42:        let names = archive.file_names().map(str::to_string).collect();
src/markdown/from_docx/ooxml.rs:149:    archive: &zip::ZipArchive<R>,
src/markdown/from_docx/ooxml.rs:1365:        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/markdown/from_docx/ooxml.rs:1367:            zip.start_file(*name, zip::write::SimpleFileOptions::default())
tests/m_bookmark_sdt_placement.rs:23:use zip::ZipWriter;
tests/m_bookmark_sdt_placement.rs:24:use zip::write::SimpleFileOptions;
tests/m367_strip_pure_i_normal_pstyle_bidi.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m413_broken_probe.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m452_title_mix_park_jc.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_same_slot_diagonal.rs:22:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m149_formatting_rpr_change.rs:16:use zip::ZipWriter;
tests/m149_formatting_rpr_change.rs:17:use zip::write::SimpleFileOptions;
tests/m149_formatting_rpr_change.rs:59:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m86_fold_ws_pure_ins_into_pure_del.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
src/convert/font_table.rs:341:            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
src/convert/font_table.rs:342:            let opt = zip::write::SimpleFileOptions::default();
tests/m474_l0_text_roundtrip.rs:47:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).expect("zip");
src/markdown/from_docx/mod.rs:4660:        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
src/markdown/from_docx/mod.rs:4661:        let options = zip::write::SimpleFileOptions::default()
src/markdown/from_docx/mod.rs:4662:            .compression_method(zip::CompressionMethod::Deflated);
tests/m132_short_into_long_stream_peel.rs:15:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m426_short_base_x_table_next_pure_id.rs:41:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m4f6_fixupids.rs:102:    use zip::ZipArchive;
tests/m_story_final_marks.rs:34:    zip::ZipArchive::new(Cursor::new(out))
src/convert/font.rs:4858:            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
src/convert/font.rs:4859:            let opt = zip::write::SimpleFileOptions::default();
src/convert/font.rs:4909:            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
src/convert/font.rs:4910:            let opt = zip::write::SimpleFileOptions::default();
tests/m145b_heading_first_del_folds.rs:28:    zip::ZipArchive::new(Cursor::new(out))
src/opc/mod.rs:16://! `rdocx_opc::OpcPackage` already provides everything `opc-partfs.ts` needs, at
src/opc/mod.rs:38:use rdocx_opc::{ContentTypes, OpcPackage};
src/opc/mod.rs:39:pub use rdocx_opc::{OpcError, Relationship, Relationships};
src/opc/mod.rs:54:use zip::write::SimpleFileOptions;
src/opc/mod.rs:55:use zip::{CompressionMethod, ZipArchive, ZipWriter};
src/opc/mod.rs:216:/// private `rdocx_opc::package::part_name_to_rels_path`.
src/opc/mod.rs:296:/// Thin adapter over `rdocx_opc::OpcPackage`. Port-equivalent of `PartFS`.
src/opc/mod.rs:369:                    .filter_map(|i| zip.name_for_index(i).map(|n| (n.to_string(), i)))
src/opc/mod.rs:450:            .last_modified_time(zip::DateTime::default());
src/opc/mod.rs:867:            .file_names()
src/opc/mod.rs:896:                Some(zip::DateTime::default()),
tests/m429_simple_ordered_x_sublist_peel.rs:30:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m101_digit_only_sole_ins_no_fold.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m128_short_short_related_lcs.rs:16:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m354_tiff_title_mark_rpr_adopt.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m57_normal_single_line_when_a_has_dd.rs:27:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m374_ordered_mix_spacing_empty_spacer.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m483_theme_color_recache.rs:34:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_opc_explicit_close.rs:18:use zip::write::SimpleFileOptions;
tests/m_opc_explicit_close.rs:19:use zip::{ZipArchive, ZipWriter};
src/markdown/package.rs:1611:    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
src/markdown/package.rs:1612:    let options = zip::write::SimpleFileOptions::default()
src/markdown/package.rs:1613:        .compression_method(zip::CompressionMethod::Deflated)
src/markdown/package.rs:1614:        .last_modified_time(zip::DateTime::default());
tests/m95_short_title_residual_pair.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m450_last_mix_heading_spacing.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m100_endzip_short_titles_only.rs:27:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_notes_style_remap_survives_writeback.rs:57:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m_notes_style_remap_survives_writeback.rs:58:        let opt = zip::write::SimpleFileOptions::default();
tests/m_notes_style_remap_survives_writeback.rs:100:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m356_ooxml_high_residual_flat_free_mesh.rs:98:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m356_ooxml_high_residual_flat_free_mesh.rs:156:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m38_table_alignment.rs:35:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m134_colon_list_stream_lcs.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_numpr_child_order.rs:29:use zip::ZipWriter;
tests/m_numpr_child_order.rs:30:use zip::write::SimpleFileOptions;
tests/m_numpr_child_order.rs:74:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m35_comments.rs:365:        let mut z = zip::ZipWriter::new(&mut buf);
tests/m35_comments.rs:367:            z.start_file(name, zip::write::SimpleFileOptions::default())
tests/m393_broken_list_cluster_interleave.rs:42:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m65_bare_normal_skip_rpr_promote.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/common/mod.rs:82:        zip::ZipArchive::new(Cursor::new(bytes)).expect("input is not a valid zip/docx");
tests/m446_subtitle_mix_keeps_live_spacing.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_edge_word_anchor.rs:27:    zip::ZipArchive::new(Cursor::new(out))
tests/m_edge_word_anchor.rs:59:    zip::ZipArchive::new(Cursor::new(out))
tests/common/docx.rs:11:use zip::ZipWriter;
tests/common/docx.rs:12:use zip::write::SimpleFileOptions;
tests/common/docx.rs:139:    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/common/docx.rs:156:    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).ok()?;
tests/m71_normal_rpr_promote_without_hf.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m404_basic_list_x_sd1707_pure_id.rs:85:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/input_admission.rs:19:use zip::write::SimpleFileOptions;
tests/input_admission.rs:20:use zip::{CompressionMethod, ZipWriter};
tests/md_comment_styles.rs:93:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/md_roundtrip_losses.rs:60:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/md_roundtrip_losses.rs:671:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
tests/md_roundtrip_losses.rs:752:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
tests/md_roundtrip_losses.rs:955:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&docx)).unwrap();
tests/m109_reverse_short_into_long.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m439_list_pure_i_snug_spacing.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m318_related_legal_prose_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m411_lease_x_memo_pure_id.rs:76:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m123_path_a_shared_word.rs:27:    zip::ZipArchive::new(Cursor::new(out))
tests/m24_strict_namespaces.rs:27:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m24_strict_namespaces.rs:28:        let opt = zip::write::SimpleFileOptions::default();
tests/m24_strict_namespaces.rs:50:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m88_mixed_fold_keep_del_numpr.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_cli_no_panic.rs:15:use zip::ZipWriter;
tests/m_cli_no_panic.rs:16:use zip::write::SimpleFileOptions;
tests/m50_detect_moves_word_mode.rs:126:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).expect("zip");
tests/m454_empty_pprchange_eq_live_jc.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m360_toc_field_no_heading_ppr_adopt.rs:41:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_textbox_field_deletion.rs:20:use zip::ZipWriter;
tests/m_textbox_field_deletion.rs:21:use zip::write::SimpleFileOptions;
tests/m440_list_spacer_empty_del_fold.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m98_empty_following_sole_del_fold.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_demo_spacing_source_default.rs:19:use zip::ZipWriter;
tests/m_demo_spacing_source_default.rs:20:use zip::write::SimpleFileOptions;
tests/m_demo_spacing_source_default.rs:69:    zip::ZipArchive::new(Cursor::new(out))
tests/m91_last_pure_del_jc_pprchange.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m341_missing_sectpr_empty_fold_order.rs:96:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m19_section_breaks.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_final_empties_no_seam.rs:27:    zip::ZipArchive::new(Cursor::new(out))
tests/m412_text_color_x_threaded_pure_i_head.rs:76:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m378_tiff_hf_adopted_as_pure_i.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m461_normal_rpr_keeps_b_kern_ligatures.rs:54:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m332_ooxml_x_short_table_mesh.rs:66:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m333_both_tables_unrelated_mesh.rs:65:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m104_short_into_long_confetti.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m89_sole_ins_multi_del_fold.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m386_calibri_center_body_mix.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m87_last_pure_del_numpr_pprchange.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/scrub_and_redact.rs:21:    let mut z = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/scrub_and_redact.rs:34:    let z = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/scrub_and_redact.rs:35:    z.file_names().map(str::to_string).collect()
tests/m424_sd2517_x_vmerge_title_peel.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m81_body_ppr_format_change.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m81_body_ppr_format_change.rs:135:            let mut zip = zip::ZipWriter::new(&mut buf);
tests/m81_body_ppr_format_change.rs:136:            let opts = zip::write::SimpleFileOptions::default();
tests/m72_normal_live_ppr_spacing_only.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m363_lists_sub_no_list_ppr_on_prose_mix.rs:40:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m385_nda_listnumber_empty_heading_mix.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m97_stamp_mark_rpr_change.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m370_orphan_pure_i_strips_line276_title.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m116_multi_del_fold_relatedness.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m113_m109_peel_only.rs:28:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m130_spacing_addition_pprchange.rs:14:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m70_normal_ind_follows_b.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m27_namespace_prefix_preserve.rs:27:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m27_namespace_prefix_preserve.rs:28:        let opt = zip::write::SimpleFileOptions::default();
tests/m27_namespace_prefix_preserve.rs:58:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_ppr_layout_change.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
tests/m80_style_fonts_follow_normal.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_cross_paragraph_stream.rs:37:    zip::ZipArchive::new(Cursor::new(out))
tests/m_cross_paragraph_stream.rs:61:    zip::ZipArchive::new(Cursor::new(out))
tests/md_note_styles.rs:131:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/s2_bake_respects_basedon_chain.rs:57:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m309_basic_list_wholesale_pure_id.rs:15:use zip::ZipWriter;
tests/m309_basic_list_wholesale_pure_id.rs:16:use zip::write::SimpleFileOptions;
tests/m309_basic_list_wholesale_pure_id.rs:106:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/edit_plan.rs:750:    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(source.clone())).unwrap();
tests/edit_plan.rs:751:    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
tests/edit_plan.rs:758:            zip::write::SimpleFileOptions::default(),
tests/edit_plan.rs:765:        zip::write::SimpleFileOptions::default(),
tests/edit_plan.rs:784:    let mut out = zip::ZipWriter::new_append(std::io::Cursor::new(docx(&para("x")))).unwrap();
tests/edit_plan.rs:787:        zip::write::SimpleFileOptions::default(),
tests/m133_last_sig_over_ordered_prefix.rs:30:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m380_file197_images_heading_mix.rs:35:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m380_file197_images_heading_mix.rs:96:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m26_paragraph_mark_order.rs:28:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m26_paragraph_mark_order.rs:29:        let opt = zip::write::SimpleFileOptions::default();
tests/m26_paragraph_mark_order.rs:44:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m18_ppr_first.rs:50:    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
tests/m425_diff_doc2_x_numwords_pure_i_num.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/word_package_notes_settings_coherence.rs:11:use zip::ZipArchive;
tests/m430_title_page_trailing_empty_pure_i.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m106_normal_same_dd_pprchange.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m106_normal_same_dd_pprchange.rs:90:        let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
tests/m419_math_delimiter_eqarr_mix.rs:45:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m389_file82_empty_spacer_before_demo_del.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m480_bothsides_style_dd_neutralizers.rs:58:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m139_unrelated_demo_title_no_fold.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m442_list_spacer_pure_d_live_numpr.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m442_list_spacer_pure_d_live_numpr.rs:98:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m112_no_merge_clean_adjacent_tables.rs:69:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m384_file45_short_list_mix_adopts_numpr.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m448_strip_trailing_empty_pure_i_dominant.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m93_last_pure_del_pstyle_pprchange.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m461_pure_i_this_text_free_mesh.rs:39:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/edit_single_output.rs:36:    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
tests/m125_file18_no_false_title_nest.rs:14:    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/m414_alpha_list_x_long_base_pure_id.rs:76:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_validity_ring1.rs:15:use zip::ZipWriter;
tests/m_validity_ring1.rs:16:use zip::write::SimpleFileOptions;
tests/m431_diff_doc2_br_drawing_pure_id.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m60_bare_normal_no_dd_promote.rs:27:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m85_empty_ins_before_trailing_pure_dels.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m470_deleted_hyperlink_field_form.rs:35:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m137_demo_subtitle_then_this_body.rs:25:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_redefined_style_metrics.rs:18:use zip::ZipWriter;
tests/m_redefined_style_metrics.rs:19:use zip::write::SimpleFileOptions;
tests/m_redefined_style_metrics.rs:66:    zip::ZipArchive::new(Cursor::new(out))
tests/edit_watermark.rs:338:    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
tests/edit_watermark.rs:339:    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
tests/edit_watermark.rs:353:        out.start_file(name, zip::write::SimpleFileOptions::default())
tests/package_relationship_carry.rs:10:use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
tests/package_relationship_carry.rs:39:        .map(|i| zip.name_for_index(i).unwrap().to_string())
tests/md_story_styles.rs:104:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/md_story_styles.rs:106:        .file_names()
tests/md_story_styles.rs:242:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
tests/m465_demo_title_fold_anchor_gate.rs:69:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m465_demo_title_fold_anchor_gate.rs:167:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(package)).unwrap();
tests/m358_localized_keeps_line276.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m358_localized_keeps_line276.rs:69:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m403_features_fields_html_mesh.rs:85:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m_part_rels_dangling.rs:35:        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
tests/m_part_rels_dangling.rs:36:        let opt = zip::write::SimpleFileOptions::default();
tests/m_part_rels_dangling.rs:102:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_part_rels_dangling.rs:149:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_part_rels_dangling.rs:236:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m349_ooxml_same_title_residual_pure_id.rs:84:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m359_list_shape_no_boundary_mix.rs:40:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m74_opaque_vml_pict.rs:28:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m74_opaque_vml_pict.rs:39:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m_interior_replace_stays_separate.rs:23:use zip::ZipWriter;
tests/m_interior_replace_stays_separate.rs:24:use zip::write::SimpleFileOptions;
tests/m_bookmark_carryover.rs:21:use zip::ZipWriter;
tests/m_bookmark_carryover.rs:22:use zip::write::SimpleFileOptions;
tests/m330_demo_cousin_no_last_sig_freemesh.rs:64:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m331_short_demo_list_x_prose_zip.rs:64:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m394_legal_mid_splice.rs:87:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m394_legal_mid_splice.rs:137:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m416_heading_x_hummingbird_pure_id.rs:76:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m382_file45_long_prose_short_list_mix.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m94_last_mixed_spacing_pprchange.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m336_related_demo_cousin_mesh.rs:64:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m336_related_demo_cousin_mesh.rs:98:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m376_bookmark_mid_pure_d_no_list_layout.rs:38:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m90_two_ins_multi_del_boundary_fold.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m310_parallel_sectioned_demo_mix.rs:14:use zip::ZipWriter;
tests/m310_parallel_sectioned_demo_mix.rs:15:use zip::write::SimpleFileOptions;
tests/m310_parallel_sectioned_demo_mix.rs:53:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m324_rstyle_parallel_mesh.rs:77:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/scrub.rs:401:    let Ok(mut zip) = zip::ZipArchive::new(std::io::Cursor::new(docx)) else {
tests/m82_stamp_endzip_no_shared_sig.rs:26:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m69_trailing_empty_pure_del.rs:24:    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
tests/m458_strip_leading_del_echo.rs:36:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m387_broken_list_list_item_mix.rs:37:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
tests/m413_spaces_x_spacing_pure_id.rs:76:    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
src/debug.rs:183:            zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a zip: {e}"))?;
src/debug.rs:2659:            let mut z = zip::ZipWriter::new(&mut buf);
src/debug.rs:2660:            let opt = zip::write::SimpleFileOptions::default();
src/debug.rs:3269:            let mut z = zip::ZipArchive::new(Cursor::new(pkg)).unwrap();
src/debug.rs:3849:            let mut zip = zip::ZipWriter::new(&mut bytes);
src/debug.rs:3851:                zip.start_file(*name, zip::write::SimpleFileOptions::default())
tests/convert_docx_to_pdf.rs:126:use zip::ZipArchive;
tests/convert_docx_to_pdf.rs:127:use zip::ZipWriter;
tests/convert_docx_to_pdf.rs:128:use zip::write::SimpleFileOptions;
tests/convert_docx_to_pdf.rs:281:    let mut archive = zip::ZipArchive::new(Cursor::new(base)).unwrap();
```

## Appendix B: entry-name expressions at the baseline

```text
src/cli.rs:1600:            .find(|(task, _)| *task == self.name())
src/cli.rs:1638:            _ => format!("{} does not write .{ext}", self.name()),
src/strict_translation.rs:776:        let name = f.name().to_string();
src/strict_translation.rs:1210:                entry.name()
src/scrub.rs:409:        let name = entry.name().to_string();
src/markdown/from_docx/ooxml.rs:42:        let names = archive.file_names().map(str::to_string).collect();
src/admission.rs:350:        let name = entry.name();
src/admission.rs:397:        let name = entry.name().to_string();
src/debug.rs:189:                .map_err(|e| format!("{}: {e}", f.name()))?;
src/debug.rs:191:                name: f.name().to_string(),
src/debug.rs:3275:                parts.push((f.name().to_string(), s.replace("1F3864", fill)));
tests/perf_atom_stack_footnote_path.rs:218:        let name = f.name().to_string();
tests/markdown_to_docx.rs:597:            zip.file_names().any(|n| n == "word/media/image1.png")
src/opc/mod.rs:167:        let name = file.name().to_string();
src/opc/mod.rs:280:        if entry.name() != "[Content_Types].xml" && !entry.name().ends_with(".rels") {
src/opc/mod.rs:369:                    .filter_map(|i| zip.name_for_index(i).map(|n| (n.to_string(), i)))
src/opc/mod.rs:867:            .file_names()
src/opc/mod.rs:898:                entry.name()
tests/cli_default_stamp.rs:67:        let name = entry.name().to_string();
tests/m66_final_sect_footer_adopt.rs:27:        .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string()))
tests/m8_strict.rs:146:        let name = f.name().to_string();
tests/append_comments.rs:133:        let name = file.name().to_string();
tests/append_documents.rs:553:        let entry = file.name().to_string();
tests/m62_footer_part_path_reconcile.rs:35:        .map(|i| zip.by_index(i).unwrap().name().to_string())
tests/m_accept_own_redlines.rs:1574:        let name = f.name().to_string();
tests/compare_is_reproducible.rs:89:        .map(|i| zip.by_index(i).unwrap().name().to_string())
tests/common/mod.rs:89:        let name = f.name().to_string();
tests/common/docx.rs:144:        let entry = file.name().to_string();
tests/md_roundtrip_losses.rs:957:        .map(|i| zip.by_index(i).unwrap().name().to_string())
tests/scrub_and_redact.rs:35:    z.file_names().map(str::to_string).collect()
tests/m_compare_orphan_header_footer.rs:89:        .map(|i| zip.by_index(i).unwrap().name().to_string())
tests/edit_plan.rs:757:            f.name().to_string(),
tests/m_opc_explicit_close.rs:33:        let name = file.name().to_string();
tests/m74_opaque_vml_pict.rs:30:        let name = zip.by_index(i).unwrap().name().to_string();
tests/m_part_rels_dangling.rs:108:            parts.insert(f.name().to_string(), s);
tests/m_part_rels_dangling.rs:238:        .map(|i| zip.by_index(i).unwrap().name().to_string())
tests/edit_watermark.rs:344:        let name = file.name().replacen("word/", "doc/", 1);
tests/convert_docx_to_pdf.rs:286:        let name = file.name().to_string();
tests/convert_docx_to_pdf.rs:35843:        let name = file.name().to_string();
tests/convert_docx_to_pdf.rs:43437:        let name = match f.name() {
tests/convert_docx_to_pdf.rs:43578:        let name = f.name().to_string();
tests/md_story_styles.rs:106:        .file_names()
tests/package_relationship_carry.rs:39:        .map(|i| zip.name_for_index(i).unwrap().to_string())
```
