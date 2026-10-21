# Functions and methods in `src/strict_translation.rs`

11 non-test, 11 test. Part of the [symbol manifest](../README.md).

#### `UriSite::carries`  (L534-543)
`fn carries(self, attr: &str, value: &str, table: &[(&str, &str)]) -> bool`

- **Does:** Whether an attribute named `attr` carrying `value` is a URI of this site.
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.starts_with`
- **Called by:** `rewrite_tag` (strict_translation.rs:613)

#### `translate`  (L552-573)
`fn translate(text: &str, table: &[(&str, &str)], site: UriSite) -> String`

- **Does:** Apply a (strict → transitional) table to the URI-bearing attribute values of `text` — namespace declarations, a `.rels` part's `Type`, and attributes whose whole value is a Strict namespace URI — leaving every other byte: element and attribute names, text content, comments, as the file spells them.
- **Calls (crate):** `split_opaque_markup` (strict_translation.rs:577) L558; `split_tag` (strict_translation.rs:595) L562; `rewrite_tag` (strict_translation.rs:613) L563
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `.split_at`, `String::with_capacity`
- **Called by:** `strict_to_transitional_docx_within` (strict_translation.rs:769); `tests::text_quoting_a_strict_uri_keeps_it` (strict_translation.rs:873); `tests::relationship_targets_keep_their_strict_uri` (strict_translation.rs:901); `tests::translates_strict_document_xml_namespaces` (strict_translation.rs:989); `tests::graphic_data_uri_attributes_are_translated` (strict_translation.rs:1019); `tests::translates_strict_rels_including_irregular` (strict_translation.rs:1059)

#### `split_opaque_markup`  (L577-591)
`fn split_opaque_markup(s: &str) -> Option<(&str, &str)>`

- **Does:** Comments (`<!--…-->`), CDATA (`<![CDATA[…]]>`), processing instructions (`<?…?>`) and the DOCTYPE (`<!…>`): returned with the text after them.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.map`, `.saturating_add`, `.split_at`, `.starts_with`, `.strip_prefix`, `Some`
- **Called by:** `translate` (strict_translation.rs:552)

#### `split_tag`  (L595-609)
`fn split_tag(s: &str) -> Option<(&str, &str)>`

- **Does:** A start or end tag: from its `<` to the first `>` outside quoted attribute values (a `>` inside a value is legal XML).
- **Calls (std/external/unresolved):** `.char_indices`, `.saturating_add`, `.skip`, `.split_at`, `Some`
- **Called by:** `translate` (strict_translation.rs:552)

#### `rewrite_tag`  (L613-638)
`fn rewrite_tag(tag: &str, table: &[(&str, &str)], site: UriSite, out: &mut String)`

- **Does:** Copy `tag` to `out`, rewriting only the values of the attributes that carry URIs of `site`.
- **Calls (crate):** `UriSite::carries` (strict_translation.rs:534) L629; `attr_name` (strict_translation.rs:642) L629; `replace_all` (strict_translation.rs:652) L630
- **Calls (std/external/unresolved):** `.chars`, `.expect`, `.find`, `.next`, `.push_str`, `.push`, `.split_at`, `.strip_prefix`, `.unwrap_or`
- **Called by:** `translate` (strict_translation.rs:552)

#### `attr_name`  (L642-649)
`fn attr_name(before: &str) -> &str`

- **Does:** The attribute name ending `before` — the token after the last whitespace, `=` stripped: `… <w:p xmlns:w=` → `xmlns:w`.
- **Calls (std/external/unresolved):** `.rsplit_once`, `.strip_prefix`, `.strip_suffix`, `.trim_end`, `.unwrap_or`
- **Called by:** `rewrite_tag` (strict_translation.rs:613)

#### `replace_all`  (L652-660)
`fn replace_all(text: &str, table: &[(&str, &str)]) -> String`

- **Does:** The old whole-string substitution, now scoped to one attribute value.
- **Calls (std/external/unresolved):** `.contains`, `.replace`, `.to_string`
- **Called by:** `rewrite_tag` (strict_translation.rs:613)

#### `main_part_is_strict`  (L670-717, pub)
`pub fn main_part_is_strict(bytes: &[u8]) -> bool`

- **Does:** Whether the package's main document part (`word/document.xml`) is ISO Strict — decided by the namespace its root element declares, the same fact Word reads the dialect from.
- **Calls (crate):** `root_open` (strict_translation.rs:721) L693,706
- **Calls (std/external/unresolved):** `.by_name`, `.clear`, `.contains`, `.copied`, `.drain`, `.extend`, `.find`, `.iter`, `.read`, `.rposition`, `.split_at`, `.take`, `Cursor::new`, `String::from_utf8_lossy`, `Vec::new`, `crate::opc::open_archive`
- **Macros:** `vec!`
- **Called by:** `tests::a_long_preamble_does_not_hide_a_strict_root` (strict_translation.rs:1132)

#### `root_open`  (L721-728)
`fn root_open(head: &str) -> Option<(usize, char)>`

- **Does:** Where `head` opens its root element: the first `<` an element name follows (a comment, PI or DOCTYPE `<!`/`<?` opens none).
- **Calls (std/external/unresolved):** `.char_indices`, `.find`, `.get`, `.is_some_and`, `.saturating_add`, `.starts_with`
- **Called by:** `main_part_is_strict` (strict_translation.rs:670)

#### `strict_to_transitional_docx`  (L749-751, pub)
`pub fn strict_to_transitional_docx(bytes: &[u8]) -> Result<Vec<u8>, crate::opc::Zip32Limit>`

- **Does:** Normalize an ISO/IEC 29500 **Strict** `.docx` to **Transitional**.
- **Calls (crate):** `strict_to_transitional_docx_within` (strict_translation.rs:769) L750; `InputLimits::compare` (admission.rs:87) L750
- **Called by:** `open` (append.rs:316); `tolerant` (changes.rs:620); `open` (changes.rs:1031); `with_layout` (convert/mod.rs:769); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45341); `comments_spacing_tests::list_bullet_style_is_contextual` (convert/mod.rs:45393); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45491); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45550); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45735); `comments_spacing_tests::strict01_section_break_census` (convert/mod.rs:45768); `comments_spacing_tests::listbullet_marked` (convert/mod.rs:45863); +7 more

#### `strict_to_transitional_docx_within`  (L769-860, pub)
`pub fn strict_to_transitional_docx_within( bytes: &[u8], limits: InputLimits, ) -> Result<Vec<u8>, crate::opc::Zip32Limit>`

- **Does:** [`strict_to_transitional_docx`] with an explicit inflation budget.
- **Calls (crate):** `translate` (strict_translation.rs:552) L818
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.by_index`, `.collect`, `.compression_method`, `.contains`, `.ends_with`, `.finish`, `.into_bytes`, `.into_inner`, `.into_owned`, `.is_dir`, `.is_err`, `.iter`, `.last_modified_time`, `.len`, `.map`, `.min`, `.name`, `.push`, `.read_to_end`, `.saturating_add`, `.saturating_sub`, `.start_file`, `.take`, `.to_vec`, `.write_all`, `Cursor::new`, `Ok`, `SimpleFileOptions::default`, `String::from_utf8_lossy`, `Vec::new`, `Vec::with_capacity`, `ZipWriter::new`, `crate::opc::open_archive`, `crate::opc::zip32_plan`, `zip::DateTime::default`
- **Called by:** `Package::open_within` (debug.rs:191); `compare_documents_impl` (document_comparer.rs:6520); `Opened::open_within` (inspect.rs:739); `strict_to_transitional_docx` (strict_translation.rs:749); `tests::oversized_part_is_left_untouched_under_a_small_budget` (strict_translation.rs:1089)

## Test functions (11)

- `tests::text_quoting_a_strict_uri_keeps_it` (L873): A Strict document whose *text* quotes a Strict URI: the namespace declaration is rewritten, the `w:t` keeps the bytes the file's own text carries (#531 review defect 2 — a whole-part `str::replace` rewrote the document's prose as well).
- `tests::relationship_targets_keep_their_strict_uri` (L901): A Strict `.rels` whose hyperlink `Target` is itself a Strict URI: the relationship `Type` is translated, the target the author linked to stays verbatim.
- `tests::transitional_package_quoting_a_strict_uri_stays_untouched` (L928): A Transitional package whose text merely mentions a Strict URI is handed back byte-for-byte: the marker in prose no longer counts as a Strict package (#531 review defect 2).
- `tests::strict_package_keeps_field_text_quoting_a_strict_uri` (L955): A Strict package whose `w:instrText` quotes a Strict URI keeps the instruction's bytes while its namespaces are translated end to end.
- `tests::translates_strict_document_xml_namespaces` (L989): A Strict `document.xml` snippet: the `w`, `r`, and `m` namespaces flip to their Transitional URIs, and no `purl.oclc.org` survives.
- `tests::graphic_data_uri_attributes_are_translated` (L1019): A Strict `a:graphicData/@uri` — an attribute whose whole value is a namespace URI, not a declaration — is translated with the namespaces, while `mc:Ignorable`, which names prefixes rather than URIs, stays as the file spells it (#561 review defect 1).
- `tests::translates_strict_rels_including_irregular` (L1059): A Strict `.rels` snippet: the IRREGULAR `extendedProperties` rel becomes `extended-properties` (longest-key-first beats the `customProperty`/ `extendedProperty` prefixes), and a regular `styles` rel becomes the Transitional styles rel.
- `tests::oversized_part_is_left_untouched_under_a_small_budget` (L1089): A Strict package whose media part inflates past the part budget is handed back untouched instead of being inflated without bound; the same package passes under the comparer's default budget.
- `tests::a_long_preamble_does_not_hide_a_strict_root` (L1132): A preamble longer than the old 64 KiB window no longer hides the Strict root: the dialect is read from the root's own opening tag, wherever in the part it lies (#561 review defect 2).
- `tests::transitional_package_returned_byte_identical` (L1155): A Transitional package is returned byte-for-byte unchanged (zero-churn).
- `tests::strict_package_is_rewritten_to_transitional` (L1184): A Strict package is actually rewritten (the zero-churn fast path does NOT fire): every text part ends up Transitional.
