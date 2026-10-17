# Functions and methods in `src/update.rs`

9 non-test, 12 test. Part of the [symbol manifest](../README.md).

#### `asset_target`  (L87-89, pub)
`pub fn asset_target() -> Option<&'static str>`

- **Does:** The release archive suffix this binary will look up (`macos-aarch64`, `linux-x86_64`, …), matching the names `release.yml` publishes.
- **Calls (crate):** `asset_target_for` (update.rs:91) L88
- **Called by:** `run` (update.rs:186); `tests::asset_targets_match_the_release_workflow_archives` (update.rs:304)

#### `asset_target_for`  (L91-100)
`fn asset_target_for(os: &str, arch: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `asset_target` (update.rs:87); `tests::asset_targets_match_the_release_workflow_archives` (update.rs:304)

#### `bin_path_in_archive`  (L104-109)
`fn bin_path_in_archive(target: &str) -> String`

- **Does:** Path of the binary inside the release archive, as `self_update` expects it (`{{ version }}` is filled in by the crate).
- **Macros:** `format!`
- **Called by:** `run` (update.rs:186); `tests::archive_binary_path_names_the_staged_folder` (update.rs:320)

#### `preflight`  (L120-126, pub)
`pub fn preflight(opts: &Options, interactive: bool) -> Result<(), String>`

- **Does:** Refuse to start when the command would have to prompt and the stdin channel is not a terminal.
- **Calls (std/external/unresolved):** `.into`, `Err`, `Ok`
- **Called by:** `run` (update.rs:186); `tests::without_a_terminal_only_check_or_yes_proceed` (update.rs:333)

#### `bare`  (L129-131)
`fn bare(version: &str) -> &str`

- **Does:** Strip one leading `v` so `v0.10.0` and `0.10.0` name the same release.
- **Calls (std/external/unresolved):** `.strip_prefix`, `.unwrap_or`
- **Called by:** `plan` (update.rs:144); `run` (update.rs:186); `signature_required` (update.rs:273)

#### `plan`  (L144-162, pub)
`pub fn plan(current: &str, target: &str, requested: bool) -> Result<Plan, String>`

- **Does:** Decide between staying and installing for a given installed version and the version `self_update` resolved on GitHub.
- **Calls (crate):** `bare` (update.rs:129) L145
- **Calls (std/external/unresolved):** `.is_gt`, `.is_ne`, `.map_err`, `.to_string`, `Ok`, `cmp_versions`
- **Macros:** `format!`
- **Called by:** `run` (update.rs:186); `tests::latest_installs_only_when_newer` (update.rs:356); `tests::a_requested_version_installs_even_when_older` (update.rs:380); `tests::a_non_semver_tag_is_an_error` (update.rs:397)

#### `run`  (L186-259, pub)
`pub fn run(opts: &Options) -> Result<(), String>`

- **Does:** `jubarte self-update`.
- **Calls (crate):** `preflight` (update.rs:120) L188; `asset_target` (update.rs:87) L189; `.target` (ambiguous: `EditOptions::target` (cli.rs:212); `Operation::target` (edit/v2.rs:573); `ThreadOp::target` (edit.rs:1906)) L199; `bin_path_in_archive` (update.rs:104) L205; `bare` (update.rs:129) L213,227; `plan` (update.rs:144) L222; `confirm` (update.rs:287) L239; `signature_required` (update.rs:273) L244; `.build` (ambiguous: `ListCopy::build` (append.rs:1111); `Origin::build` (edit/origin.rs:256)) L250
- **Calls (std/external/unresolved):** `.bin_name`, `.bin_path_in_archive`, `.build`, `.checksum_from_asset`, `.current_version`, `.get_latest_release`, `.get_release_version`, `.into_iter`, `.into_vec`, `.is_empty`, `.is_some`, `.is_terminal`, `.map_err`, `.map`, `.next`, `.no_confirm`, `.ok_or_else`, `.ok_or`, `.release_tag`, `.repo_name`, `.repo_owner`, `.show_download_progress`, `.show_output`, `.to_string`, `.update`, `.verifying_keys`, `.version`, `Ok`, `github::Update::configure`, `std::io::stdin`
- **Macros:** `env!`, `eprintln!`, `format!`, `println!`
- **Called by:** `run_self_update` (bin/jubarte.rs:3379)

#### `signature_required`  (L273-285, pub)
`pub fn signature_required( target_version: &str, first_signed: Option<&str>, keys_present: bool, ) -> Result<bool, String>`

- **Does:** Whether installing `target_version` must pass a zipsign signature check.
- **Calls (crate):** `bare` (update.rs:129) L281
- **Calls (std/external/unresolved):** `.filter`, `.is_ge`, `.map_err`, `Ok`, `cmp_versions`
- **Macros:** `format!`
- **Called by:** `run` (update.rs:186); `tests::without_keys_no_signature_is_required` (update.rs:402); `tests::keys_without_a_first_signed_release_require_nothing` (update.rs:418); `tests::the_first_signed_release_and_later_ones_require_a_signature` (update.rs:424); `tests::releases_before_the_first_signed_one_are_checksum_only` (update.rs:438); `tests::a_non_semver_version_is_an_error_when_signing_is_on` (update.rs:447); `tests::the_shipped_key_set_and_threshold_agree` (update.rs:453)

#### `confirm`  (L287-296)
`fn confirm(from: &str, to: &str) -> Result<bool, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flush`, `.lock`, `.map_err`, `.read_line`, `.to_string`, `.trim`, `Ok`, `String::new`, `std::io::stderr`, `std::io::stdin`
- **Macros:** `eprint!`, `matches!`
- **Called by:** `run` (update.rs:186)

## Test functions (12)

- `tests::asset_targets_match_the_release_workflow_archives` (L304): _no doc comment_
- `tests::archive_binary_path_names_the_staged_folder` (L320): _no doc comment_
- `tests::without_a_terminal_only_check_or_yes_proceed` (L333): _no doc comment_
- `tests::latest_installs_only_when_newer` (L356): _no doc comment_
- `tests::a_requested_version_installs_even_when_older` (L380): _no doc comment_
- `tests::a_non_semver_tag_is_an_error` (L397): _no doc comment_
- `tests::without_keys_no_signature_is_required` (L402): _no doc comment_
- `tests::keys_without_a_first_signed_release_require_nothing` (L418): _no doc comment_
- `tests::the_first_signed_release_and_later_ones_require_a_signature` (L424): _no doc comment_
- `tests::releases_before_the_first_signed_one_are_checksum_only` (L438): _no doc comment_
- `tests::a_non_semver_version_is_an_error_when_signing_is_on` (L447): _no doc comment_
- `tests::the_shipped_key_set_and_threshold_agree` (L453): _no doc comment_
