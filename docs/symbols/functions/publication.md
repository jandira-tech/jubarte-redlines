# Functions and methods in `src/publication.rs`

22 non-test, 22 test. Part of the [symbol manifest](../README.md).

#### `Backend::stage`  (L24-24, trait decl, no body)
`fn stage(&mut self, path: &Path) -> io::Result<Self::Staged>`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `Backend::step`  (L25-25, trait decl, no body)
`fn step(&mut self, staged: &mut Self::Staged, phase: Phase, bytes: &[u8]) -> io::Result<()>`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `Backend::cleanup`  (L26-26, trait decl, no body)
`fn cleanup(&mut self, staged: &mut Self::Staged) -> io::Result<()>`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<PublishError as std::fmt::Display>::fmt`  (L50-62, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `<PublishError as std::error::Error>::source`  (L65-67, impl std::error::Error)
`fn source(&self) -> Option<&(dyn std::error::Error + 'static)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `run_text_diff` (bin/jubarte.rs:3327) (?); `run_diff` (bin/jubarte.rs:3364) (?)

#### `finish`  (L70-81)
`fn finish( result: Result<(), PublishError>, warnings: Vec<String>, ) -> Result<PublishReport, PublishError>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Called by:** `transaction` (publication.rs:83); `StagedFile::commit` (publication.rs:317)

#### `transaction`  (L83-144)
`fn transaction<B: Backend>( backend: &mut B, files: &[(PathBuf, &[u8])], ) -> Result<PublishReport, PublishError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::new` (result.rs:360) L91,104; `failure` (ambiguous: `failure` (bin/jubarte/edit_template.rs:19); `failure` (bin/jubarte/edit_typed.rs:17)) L103,112,120,126,130; `finish` (publication.rs:70) L143
- **Calls (std/external/unresolved):** `.any`, `.cleanup`, `.clone`, `.collect`, `.display`, `.err`, `.expect`, `.filter_map`, `.iter_mut`, `.iter`, `.kind`, `.last_mut`, `.len`, `.map_err`, `.map`, `.push`, `.stage`, `.step`, `.to_vec`, `Err`, `Ok`, `Vec::new`, `Vec::with_capacity`
- **Macros:** `format!`, `vec!`
- **Called by:** `OutputRoot::publish` (publication.rs:175); `publish_paths` (publication.rs:211); `tests::cleanup_failure_is_reported_after_install_and_alongside_publish_failure` (publication.rs:582); `tests::every_publish_path_removes_partial_directories` (publication.rs:654); `tests::fs_failure` (publication.rs:853); `tests::audit_d13_all_precommit_failures_preserve_every_prior_output` (publication.rs:913); `tests::audit_d13_rename_failure_never_deletes_old_and_reports_prior_commits` (publication.rs:926); `tests::audit_d13_postcommit_sync_failure_reports_visible_output_and_uncertain_durability` (publication.rs:940); `tests::audit_d13_validates_the_entire_set_before_the_first_commit` (publication.rs:949); `tests::staging_creation_failure_and_duplicate_names_never_commit` (publication.rs:1030)

#### `OutputRoot::open`  (L155-159, pub)
`pub fn open(root: &Path) -> io::Result<Self>`

- **Does:** Grant access to an existing trusted directory once, before serving callers.
- **Calls (std/external/unresolved):** `Ok`, `cap_std::ambient_authority`, `cap_std::fs::Dir::open_ambient_dir`
- **Called by:** `image_loader` (bin/jubarte.rs:3196); `tests::cleanup_failure_is_reported_after_install_and_alongside_publish_failure` (publication.rs:582); `tests::fs_failure` (publication.rs:853)

#### `OutputRoot::read`  (L162-166, pub)
`pub fn read(&self, relative: &Path) -> io::Result<Vec<u8>>`

- **Does:** Read a confined file without reopening the root by pathname.
- **Calls (std/external/unresolved):** `.read`
- **Called by:** none found in src

#### `OutputRoot::publish`  (L175-200, pub)
`pub fn publish( &self, files: &[(PathBuf, &[u8])], overwrite: bool, ) -> Result<PublishReport, PublishError>`

- **Does:** Stage and verify every file before committing any.
- **Calls (crate):** `validate_relative` (publication.rs:228) L182; `transaction` (publication.rs:83) L189
- **Calls (std/external/unresolved):** `.map_err`, `Some`
- **Macros:** `vec!`
- **Called by:** none found in src

#### `publish_paths`  (L211-226, pub)
`pub fn publish_paths( files: &[(PathBuf, &[u8])], overwrite: bool, ) -> Result<PublishReport, PublishError>`

- **Does:** Publish explicitly authorized local paths.
- **Calls (crate):** `transaction` (publication.rs:83) L215
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_paths` (bin/jubarte.rs:262); `write_plan` (edit/record.rs:161); `tests::every_publish_path_removes_partial_directories` (publication.rs:654)

#### `validate_relative`  (L228-274)
`fn validate_relative(path: &Path) -> io::Result<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::new` (result.rs:360) L230
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chars`, `.display`, `.ends_with`, `.is_control`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.next`, `.ok_or_else`, `.split`, `.strip_prefix`, `.to_ascii_uppercase`, `.to_str`, `.unwrap_or`, `Err`, `Ok`, `bad`
- **Macros:** `format!`, `matches!`
- **Called by:** `OutputRoot::publish` (publication.rs:175); `tests::audit_d35_portable_names_cannot_smuggle_extra_path_authority` (publication.rs:966)

#### `StagedFile::prepare`  (L299-313, pub)
`pub fn prepare(path: &Path, bytes: &[u8], overwrite: bool) -> io::Result<Self>`

- **Does:** Write, flush, sync and verify bytes without changing the destination.
- **Calls (crate):** `<FsBackend as Backend>::stage` (publication.rs:421) L308; `<FsBackend as Backend>::step` (publication.rs:519) L310
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_files` (bin/jubarte.rs:1854); `tests::prepared_commit_reports_cleanup_failure_after_install` (publication.rs:641); `tests::cleanup_refuses_a_swapped_scratch_symlink` (publication.rs:708)

#### `StagedFile::commit`  (L317-351, pub)
`pub fn commit(mut self) -> Result<PublishReport, PublishError>`

- **Does:** Install the verified bytes, then acknowledge parent-directory durability.
- **Calls (crate):** `<FsBackend as Backend>::step` (publication.rs:519) L327,335; `StagedFile::cleanup` (publication.rs:355) L344; `finish` (publication.rs:70) L350
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.display`, `.err`, `.into_iter`, `.map_err`, `.map`
- **Macros:** `format!`, `vec!`
- **Called by:** `publish_files` (bin/jubarte.rs:1854) (?); `tests::prepared_commit_reports_cleanup_failure_after_install` (publication.rs:641) (?); `tests::cleanup_refuses_a_swapped_scratch_symlink` (publication.rs:708) (?)

#### `StagedFile::cleanup`  (L355-384)
`fn cleanup(&mut self) -> io::Result<()>`

- **Does:** Remove staging after closing all of its handles.
- **Calls (crate):** `Error::new` (result.rs:360) L369
- **Calls (std/external/unresolved):** `.as_ref`, `.is_symlink`, `.kind`, `.remove_dir_all`, `.symlink_metadata`, `.take`, `Err`, `Ok`, `drop`, `io::Error::other`
- **Called by:** `StagedFile::commit` (publication.rs:317); `<StagedFile as Drop>::drop` (publication.rs:415); `<FsBackend as Backend>::cleanup` (publication.rs:516)

#### `sync_directory_for_platform`  (L390-395)
`fn sync_directory_for_platform( windows: bool, sync: impl FnOnce() -> io::Result<()>, ) -> io::Result<()>`

- **Does:** Windows directory handles are read-only, but FlushFileBuffers needs GENERIC_WRITE.
- **Calls (std/external/unresolved):** `Ok`, `sync`
- **Called by:** `sync_directory_handle` (publication.rs:400); `tests::windows_directory_sync_is_a_noop` (publication.rs:742); `tests::unix_directory_sync_calls_the_operation_and_preserves_its_result` (publication.rs:759)

#### `sync_directory_handle`  (L400-406)
`fn sync_directory_handle(dir: &cap_std::fs::Dir) -> io::Result<()>`

- **Does:** cap-std opens directories `O_PATH` on Linux, and `fsync` on an `O_PATH` descriptor is EBADF (#1351).
- **Calls (crate):** `sync_directory_for_platform` (publication.rs:390) L401
- **Calls (std/external/unresolved):** `.into_std`, `.open_with`, `.read`, `.sync_all`, `cap_std::fs::OpenOptions::new`
- **Macros:** `cfg!`
- **Called by:** `sync_directory` (publication.rs:409); `<FsBackend as Backend>::stage` (publication.rs:421); `<FsBackend as Backend>::step` (publication.rs:519); `tests::a_cap_std_directory_handle_can_be_synced` (publication.rs:781)

#### `sync_directory`  (L409-412, pub)
`pub fn sync_directory(path: &Path) -> io::Result<()>`

- **Does:** Acknowledge directory durability using a held directory handle (no-op on Windows).
- **Calls (crate):** `sync_directory_handle` (publication.rs:400) L411
- **Calls (std/external/unresolved):** `cap_std::ambient_authority`, `cap_std::fs::Dir::open_ambient_dir`
- **Called by:** `sync_bundle_parent` (bin/jubarte.rs:1999); `tests::a_cap_std_directory_handle_can_be_synced` (publication.rs:781)

#### `<StagedFile as Drop>::drop`  (L415-417, impl Drop)
`fn drop(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `StagedFile::cleanup` (publication.rs:355) L416
- **Called by:** none found in src

#### `<FsBackend as Backend>::stage`  (L421-515, impl Backend)
`fn stage(&mut self, path: &Path) -> io::Result<StagedFile>`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::new` (result.rs:360) L429,451,460; `sync_directory_handle` (publication.rs:400) L436
- **Calls (std/external/unresolved):** `.as_os_str`, `.as_ref`, `.components`, `.create_dir_with`, `.create_dir`, `.create_new`, `.display`, `.expect`, `.file_name`, `.filter`, `.inspect_err`, `.is_dir`, `.is_empty`, `.is_file`, `.is_some`, `.is_symlink`, `.kind`, `.mode`, `.ok_or_else`, `.open_dir`, `.open_with`, `.parent`, `.permissions`, `.read`, `.remove_dir`, `.set_permissions`, `.symlink_metadata`, `.to_os_string`, `.to_path_buf`, `.try_clone`, `.unwrap_or`, `.write`, `Dir::open_ambient_dir`, `DirBuilder::new`, `Err`, `Ok`, `OpenOptions::new`, `Path::new`, `Some`, `cap_std::ambient_authority`, `uuid::Uuid::new_v4`
- **Macros:** `format!`
- **Called by:** `StagedFile::prepare` (publication.rs:299)

#### `<FsBackend as Backend>::cleanup`  (L516-518, impl Backend)
`fn cleanup(&mut self, staged: &mut StagedFile) -> io::Result<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `StagedFile::cleanup` (publication.rs:355) L517
- **Called by:** none found in src

#### `<FsBackend as Backend>::step`  (L519-572, impl Backend)
`fn step(&mut self, staged: &mut StagedFile, phase: Phase, bytes: &[u8]) -> io::Result<()>`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::metadata` (markdown/write.rs:4453) (by name) L540; `Error::new` (result.rs:360) L541,550; `sync_directory_handle` (publication.rs:400) L570
- **Calls (std/external/unresolved):** `.as_mut`, `.as_ref`, `.chunks`, `.expect`, `.flush`, `.hard_link`, `.len`, `.read_exact`, `.remove_file`, `.rename`, `.seek`, `.sync_all`, `.write_all`, `Err`, `Ok`, `SeekFrom::Start`, `Some`, `io::Error::other`
- **Macros:** `format!`
- **Called by:** `StagedFile::prepare` (publication.rs:299); `StagedFile::commit` (publication.rs:317)

## Test functions (22)

- `tests::cleanup_failure_is_reported_after_install_and_alongside_publish_failure` (L582): Filesystem integration probes: exercise actual handle lifetime and hard links. _(from a code comment)_
- `tests::prepared_commit_reports_cleanup_failure_after_install` (L641): _no doc comment_
- `tests::every_publish_path_removes_partial_directories` (L654): _no doc comment_
- `tests::assert_no_partials` (L694): _no doc comment_
- `tests::cleanup_refuses_a_swapped_scratch_symlink` (L708): _no doc comment_
- `tests::windows_directory_sync_is_a_noop` (L742): _no doc comment_
- `tests::unix_directory_sync_calls_the_operation_and_preserves_its_result` (L759): _no doc comment_
- `tests::a_cap_std_directory_handle_can_be_synced` (L781): cap-std opens directories `O_PATH` on Linux, and `fsync` on an `O_PATH` descriptor is EBADF (#1351): every publication failed after its commit there, with "durability uncertain".
- `Memory::new` (L801): _no doc comment_
- `<Memory as Backend>::stage` (L817): _no doc comment_
- `<Memory as Backend>::cleanup` (L825): _no doc comment_
- `<Memory as Backend>::step` (L828): _no doc comment_
- `tests::files` (L849): _no doc comment_
- `tests::fs_failure` (L853): _no doc comment_
- `tests::audit_d13_fs_rename_failure_preserves_prior_output_and_cleans_staging` (L903): _no doc comment_
- `tests::audit_d13_fs_parent_sync_failure_reports_installed_output_and_cleans_staging` (L908): _no doc comment_
- `tests::audit_d13_all_precommit_failures_preserve_every_prior_output` (L913): _no doc comment_
- `tests::audit_d13_rename_failure_never_deletes_old_and_reports_prior_commits` (L926): _no doc comment_
- `tests::audit_d13_postcommit_sync_failure_reports_visible_output_and_uncertain_durability` (L940): _no doc comment_
- `tests::audit_d13_validates_the_entire_set_before_the_first_commit` (L949): _no doc comment_
- `tests::audit_d35_portable_names_cannot_smuggle_extra_path_authority` (L966): _no doc comment_
- `tests::staging_creation_failure_and_duplicate_names_never_commit` (L1030): _no doc comment_
