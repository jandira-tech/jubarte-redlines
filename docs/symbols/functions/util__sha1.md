# Functions and methods in `src/util/sha1.rs`

13 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `hex_string_from_bytes`  (L22-31, pub(crate))
`pub(crate) fn hex_string_from_bytes(bytes: &[u8]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.enumerate`, `.expect`, `.iter`, `.len`, `String::from_utf8`
- **Macros:** `vec!`
- **Called by:** `sha1_hex_of_digest_hexes` (util/sha1.rs:97); `sha1_hex_bytes` (util/sha1.rs:110); `sha1_hex_parts` (util/sha1.rs:120); `HashXmlBuf::finish_hex` (xmllinq/serialize.rs:481)

#### `sha1_hex`  (L35-37, pub)
`pub fn sha1_hex(s: &str) -> String`

- **Does:** Lowercase-hex SHA-1 of `s`'s UTF-8 bytes.
- **Calls (crate):** `sha1_hex_bytes` (util/sha1.rs:110) L36
- **Calls (std/external/unresolved):** `.as_bytes`
- **Called by:** `rehash_words_by_text_content` (comparer/lcs.rs:2099); `try_stream_hash_simple_paragraph` (comparer/preprocess.rs:690); `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928)

#### `sha1_digest`  (L43-47, pub)
`pub fn sha1_digest(bytes: &[u8]) -> [u8; 20]`

- **Does:** Raw 20-byte SHA-1 digest of `bytes`.
- **Calls (std/external/unresolved):** `.finalize`, `.into`, `.update`, `Sha1::new`
- **Called by:** `AtomHash::of_bytes` (comparer/atoms.rs:49)

#### `hex_encode_20`  (L53-60, pub)
`pub fn hex_encode_20(digest: &[u8; 20]) -> [u8; 40]`

- **Does:** Lowercase-hex-encode 20 digest bytes into a fixed 40-byte ASCII buffer (no heap allocation).
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`
- **Called by:** `AtomHash::to_hex` (comparer/atoms.rs:66); `sha1_hex_of_digest_hexes` (util/sha1.rs:97)

#### `hex_val`  (L63-70)
`fn hex_val(c: u8) -> Option<u8>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `hex_decode_20` (util/sha1.rs:77)

#### `hex_decode_20`  (L77-89, pub)
`pub fn hex_decode_20(s: &str) -> Option<[u8; 20]>`

- **Does:** Decode exactly 40 hex characters into the 20-byte digest they encode; `None` for any other length or any non-hex character.
- **Calls (crate):** `hex_val` (util/sha1.rs:63) L84,85
- **Calls (std/external/unresolved):** `.as_bytes`, `.enumerate`, `.iter_mut`, `.len`, `Some`
- **Called by:** `AtomHash::from_hex` (comparer/atoms.rs:59)

#### `sha1_hex_of_digest_hexes`  (L97-106, pub)
`pub fn sha1_hex_of_digest_hexes<'a, I>(digests: I) -> String where I: IntoIterator<Item = &'a [u8; 20]>,`

- **Does:** SHA-1 of the concatenation of the 40-char lowercase hex encodings of each digest.
- **Calls (crate):** `hex_encode_20` (util/sha1.rs:53) L103; `hex_string_from_bytes` (util/sha1.rs:22) L105
- **Calls (std/external/unresolved):** `.finalize`, `.update`, `Sha1::new`
- **Called by:** `ComparisonUnitWord::new` (comparer/atoms.rs:298)

#### `sha1_hex_bytes`  (L110-114, pub)
`pub fn sha1_hex_bytes(bytes: &[u8]) -> String`

- **Does:** Lowercase-hex SHA-1 of `bytes`.
- **Calls (crate):** `hex_string_from_bytes` (util/sha1.rs:22) L113
- **Calls (std/external/unresolved):** `.finalize`, `.update`, `Sha1::new`
- **Called by:** `sha1_hex` (util/sha1.rs:35)

#### `sha1_hex_parts`  (L120-129, pub)
`pub fn sha1_hex_parts<'a, I>(parts: I) -> String where I: IntoIterator<Item = &'a str>,`

- **Does:** SHA-1 of the concatenation of `parts`, without allocating the concatenated string.
- **Calls (crate):** `hex_string_from_bytes` (util/sha1.rs:22) L128
- **Calls (std/external/unresolved):** `.as_bytes`, `.finalize`, `.update`, `Sha1::new`
- **Called by:** none found in src

#### `fnv1a_64`  (L143-152, pub)
`pub fn fnv1a_64(s: &str) -> u64`

- **Does:** 64-bit FNV-1a of the string's bytes (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`).
- **Calls (std/external/unresolved):** `.as_bytes`, `.wrapping_mul`
- **Called by:** `Sha1Keyed::new` (comparer/atoms.rs:237); `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (comparer/atoms.rs:580); `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588); `sha1_keyed_tests::a_colliding_key_changes_only_the_u64_key` (comparer/atoms.rs:597); `sha1_fingerprint` (util/sha1.rs:181)

#### `fnv1a_128`  (L168-177, pub)
`pub fn fnv1a_128(s: &str) -> u128`

- **Does:** 128-bit FNV-1a of the string's bytes (offset basis `0x6c62272e07bb014262b821756295c58d`, prime `0x1000000000000000000013b`).
- **Calls (std/external/unresolved):** `.as_bytes`, `.wrapping_mul`
- **Called by:** `Sha1Keyed::new` (comparer/atoms.rs:237); `sha1_keyed_tests::new_derives_both_keys_from_the_hash` (comparer/atoms.rs:580); `sha1_keyed_tests::set_hash_leaves_no_key_behind` (comparer/atoms.rs:588); `sha1_keyed_tests::a_colliding_key_changes_only_the_u64_key` (comparer/atoms.rs:597); `sha1_fingerprint128` (util/sha1.rs:190)

#### `sha1_fingerprint`  (L181-183, pub)
`pub fn sha1_fingerprint(s: &str) -> u64`

- **Does:** Former name of [`fnv1a_64`].
- **Calls (crate):** `fnv1a_64` (util/sha1.rs:143) L182
- **Called by:** none found in src

#### `sha1_fingerprint128`  (L190-192, pub)
`pub fn sha1_fingerprint128(s: &str) -> u128`

- **Does:** Former name of [`fnv1a_128`].
- **Calls (crate):** `fnv1a_128` (util/sha1.rs:168) L191
- **Called by:** none found in src
