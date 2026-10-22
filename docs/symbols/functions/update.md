# Functions and methods in `src/update.rs`

11 non-test, 14 test. Part of the [symbol manifest](../README.md).

#### `asset_target`  (L87-89, pub)
`pub fn asset_target() -> Option<&'static str>`

- **Does:** The release archive suffix this binary will look up (`macos-aarch64`, `linux-x86_64`, …), matching the names `release.yml` publishes.
- **Calls (crate):** `asset_target_for` (update.rs:91) L88
- **Called by:** `run` (update.rs:225); `tests::asset_targets_match_the_release_workflow_archives` (update.rs:360)

#### `asset_target_for`  (L91-100)
`fn asset_target_for(os: &str, arch: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `asset_target` (update.rs:87); `tests::asset_targets_match_the_release_workflow_archives` (update.rs:360)

#### `bin_path_in_archive`  (L104-109)
`fn bin_path_in_archive(target: &str) -> String`

- **Does:** Path of the binary inside the release archive, as `self_update` expects it (`{{ version }}` is filled in by the crate).
- **Macros:** `format!`
- **Called by:** `run` (update.rs:225); `tests::archive_binary_path_names_the_staged_folder` (update.rs:376)

#### `preflight`  (L120-126, pub)
`pub fn preflight(opts: &Options, interactive: bool) -> Result<(), String>`

- **Does:** Refuse to start when the command would have to prompt and the stdin channel is not a terminal.
- **Calls (std/external/unresolved):** `.into`, `Err`, `Ok`
- **Called by:** `run` (update.rs:225); `tests::without_a_terminal_only_check_or_yes_proceed` (update.rs:389)

#### `bare`  (L129-131)
`fn bare(version: &str) -> &str`

- **Does:** Strip one leading `v` so `v0.10.0` and `0.10.0` name the same release.
- **Calls (std/external/unresolved):** `.strip_prefix`, `.unwrap_or`
- **Called by:** `plan` (update.rs:144); `run` (update.rs:225); `signature_required` (update.rs:304)

#### `plan`  (L144-162, pub)
`pub fn plan(current: &str, target: &str, requested: bool) -> Result<Plan, String>`

- **Does:** Decide between staying and installing for a given installed version and the version `self_update` resolved on GitHub.
- **Calls (crate):** `bare` (update.rs:129) L145
- **Calls (std/external/unresolved):** `.is_gt`, `.is_ne`, `.map_err`, `.to_string`, `Ok`, `cmp_versions`
- **Macros:** `format!`
- **Called by:** `run` (update.rs:225); `tests::latest_installs_only_when_newer` (update.rs:468); `tests::a_requested_version_installs_even_when_older` (update.rs:492); `tests::a_non_semver_tag_is_an_error` (update.rs:509)

#### `step`  (L186-201, pub)
`pub fn step(opts: &Options, decision: &Plan, resolved: &str) -> Step`

- **Does:** `decision` is [`plan`]'s answer; `resolved` is the bare release version the lookup returned (the up-to-date message quotes it).
- **Calls (std/external/unresolved):** `.clone`
- **Macros:** `format!`
- **Called by:** `run` (update.rs:225); `tests::check_reports_and_never_installs` (update.rs:412)

#### `run`  (L225-290, pub)
`pub fn run(opts: &Options) -> Result<(), String>`

- **Does:** `jubarte self-update`.
- **Calls (crate):** `preflight` (update.rs:120) L227; `asset_target` (update.rs:87) L228; `.target` (ambiguous: `EditOptions::target` (cli.rs:358); `Operation::target` (edit/v2.rs:585); `ThreadOp::target` (edit.rs:2567)) L238; `bin_path_in_archive` (update.rs:104) L244; `bare` (update.rs:129) L253,263; `plan` (update.rs:144) L262; `step` (update.rs:186) L263; `confirm` (update.rs:318) L270; `signature_required` (update.rs:304) L275; `.build` (ambiguous: `ListCopy::build` (append.rs:1117); `Selection::build` (edit/for_edit.rs:547); `Origin::build` (edit/origin.rs:260)) L281
- **Calls (std/external/unresolved):** `.bin_name`, `.bin_path_in_archive`, `.build`, `.checksum_from_asset`, `.current_version`, `.get_latest_release`, `.get_release_version`, `.into_iter`, `.into_vec`, `.is_empty`, `.is_some`, `.is_terminal`, `.map_err`, `.map`, `.next`, `.no_confirm`, `.ok_or_else`, `.ok_or`, `.release_tag`, `.repo_name`, `.repo_owner`, `.show_download_progress`, `.show_output`, `.to_string`, `.update`, `.verify_binary`, `.verifying_keys`, `.version`, `Ok`, `github::Update::configure`, `std::io::stdin`
- **Macros:** `env!`, `eprintln!`, `format!`, `println!`
- **Called by:** `run_self_update` (bin/jubarte.rs:4455)

#### `signature_required`  (L304-316, pub)
`pub fn signature_required( target_version: &str, first_signed: Option<&str>, keys_present: bool, ) -> Result<bool, String>`

- **Does:** Whether installing `target_version` must pass a zipsign signature check.
- **Calls (crate):** `bare` (update.rs:129) L312
- **Calls (std/external/unresolved):** `.filter`, `.is_ge`, `.map_err`, `Ok`, `cmp_versions`
- **Macros:** `format!`
- **Called by:** `run` (update.rs:225); `tests::without_keys_no_signature_is_required` (update.rs:514); `tests::keys_without_a_first_signed_release_require_nothing` (update.rs:530); `tests::the_first_signed_release_and_later_ones_require_a_signature` (update.rs:536); `tests::releases_before_the_first_signed_one_are_checksum_only` (update.rs:550); `tests::a_non_semver_version_is_an_error_when_signing_is_on` (update.rs:559); `tests::the_shipped_key_set_and_threshold_agree` (update.rs:565)

#### `confirm`  (L318-327)
`fn confirm(from: &str, to: &str) -> Result<bool, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.flush`, `.lock`, `.map_err`, `.read_line`, `.to_string`, `.trim`, `Ok`, `String::new`, `std::io::stderr`, `std::io::stdin`
- **Macros:** `eprint!`, `matches!`
- **Called by:** `run` (update.rs:225)

#### `extracted_binary_is_regular_file`  (L337-352)
`fn extracted_binary_is_regular_file(path: &std::path::Path) -> self_update::Result<()>`

- **Does:** The `verify_binary` hook `run` registers (ZIP-16/#639): refuse the extracted binary when it is anything but a regular file.
- **Calls (std/external/unresolved):** `.display`, `.is_file`, `.map_err`, `Err`, `Ok`, `self_update::Error::verification_rejected`, `std::fs::symlink_metadata`
- **Macros:** `format!`
- **Called by:** `tests::a_special_extracted_binary_is_refused_before_the_swap` (update.rs:591)

## Test functions (14)

- `tests::asset_targets_match_the_release_workflow_archives` (L360): _no doc comment_
- `tests::archive_binary_path_names_the_staged_folder` (L376): _no doc comment_
- `tests::without_a_terminal_only_check_or_yes_proceed` (L389): _no doc comment_
- `tests::check_reports_and_never_installs` (L412): _no doc comment_
- `tests::latest_installs_only_when_newer` (L468): _no doc comment_
- `tests::a_requested_version_installs_even_when_older` (L492): _no doc comment_
- `tests::a_non_semver_tag_is_an_error` (L509): _no doc comment_
- `tests::without_keys_no_signature_is_required` (L514): _no doc comment_
- `tests::keys_without_a_first_signed_release_require_nothing` (L530): _no doc comment_
- `tests::the_first_signed_release_and_later_ones_require_a_signature` (L536): _no doc comment_
- `tests::releases_before_the_first_signed_one_are_checksum_only` (L550): _no doc comment_
- `tests::a_non_semver_version_is_an_error_when_signing_is_on` (L559): _no doc comment_
- `tests::the_shipped_key_set_and_threshold_agree` (L565): _no doc comment_
- `tests::a_special_extracted_binary_is_refused_before_the_swap` (L591): ZIP-16 (#639): the binary swap must never install a link or a special file.
