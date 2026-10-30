// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Stage complete bytes before replacing outputs. A set has one commit per
//! file; errors name any files already committed. No rollback or delete-old
//! fallback is attempted. Native host filesystems only.

use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Write,
    Flush,
    Sync,
    Validate,
    Commit,
    SyncParent,
}

trait Backend {
    type Staged;
    fn stage(&mut self, path: &Path) -> io::Result<Self::Staged>;
    fn step(&mut self, staged: &mut Self::Staged, phase: Phase, bytes: &[u8]) -> io::Result<()>;
    fn cleanup(&mut self, staged: &mut Self::Staged) -> io::Result<()>;
}

/// A successful publication, including any staging cleanup warnings.
#[derive(Debug, Default)]
pub struct PublishReport {
    /// Cleanup failures do not undo installed output.
    pub warnings: Vec<String>,
}

/// A publication failure, including output that has already become visible.
#[derive(Debug)]
pub struct PublishError {
    /// Files committed before the error. An empty list means nothing published.
    pub committed: Vec<PathBuf>,
    /// True when a committed file's parent could not acknowledge durability.
    pub durability_uncertain: bool,
    /// The operation's I/O error.
    pub source: io::Error,
    /// Staging cleanup failures, separate from the original publication error.
    pub warnings: Vec<String>,
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; committed outputs: {:?}{}",
            self.source,
            self.committed,
            if self.durability_uncertain {
                "; durability uncertain"
            } else {
                ""
            }
        )
    }
}
impl std::error::Error for PublishError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn finish(
    result: Result<(), PublishError>,
    warnings: Vec<String>,
) -> Result<PublishReport, PublishError> {
    match result {
        Ok(()) => Ok(PublishReport { warnings }),
        Err(mut error) => {
            error.warnings = warnings;
            Err(error)
        }
    }
}

fn transaction<B: Backend>(
    backend: &mut B,
    files: &[(PathBuf, &[u8])],
) -> Result<PublishReport, PublishError> {
    let mut staged = Vec::with_capacity(files.len());
    let mut committed = Vec::new();
    let failure = |source: io::Error, path: &Path, committed: &[PathBuf], durability_uncertain| {
        PublishError {
            source: io::Error::new(
                source.kind(),
                format!("writing {}: {source}", path.display()),
            ),
            committed: committed.to_vec(),
            durability_uncertain,
            warnings: vec![],
        }
    };
    let result = (|| {
        for (path, bytes) in files {
            if staged.iter().any(|(previous, _)| previous == path) {
                return Err(failure(
                    io::Error::new(io::ErrorKind::InvalidInput, "duplicate output path"),
                    path,
                    &committed,
                    false,
                ));
            }
            let held = backend
                .stage(path)
                .map_err(|e| failure(e, path, &committed, false))?;
            // Retain every staged file, including the one whose next step fails,
            // until the explicit cleanup below has reported its outcome.
            staged.push((path.clone(), held));
            let (_, held) = staged.last_mut().expect("just staged");
            for phase in [Phase::Write, Phase::Flush, Phase::Sync, Phase::Validate] {
                backend
                    .step(held, phase, bytes)
                    .map_err(|e| failure(e, path, &committed, false))?;
            }
        }
        for (path, held) in &mut staged {
            backend
                .step(held, Phase::Commit, &[])
                .map_err(|e| failure(e, path, &committed, false))?;
            committed.push(path.clone());
            backend
                .step(held, Phase::SyncParent, &[])
                .map_err(|e| failure(e, path, &committed, true))?;
        }
        Ok(())
    })();
    let warnings = staged
        .iter_mut()
        .filter_map(|(path, held)| {
            backend
                .cleanup(held)
                .err()
                .map(|error| format!("cleaning staging for {}: {error}", path.display()))
        })
        .collect();
    finish(result, warnings)
}

/// A caller-granted directory held open across all operations. Relative path
/// resolution cannot follow a symlink outside this capability, even if a
/// pathname checked earlier is concurrently replaced.
pub struct OutputRoot {
    dir: cap_std::fs::Dir,
}

impl OutputRoot {
    /// Grant access to an existing trusted directory once, before serving callers.
    pub fn open(root: &Path) -> io::Result<Self> {
        Ok(Self {
            dir: cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())?,
        })
    }

    /// Read a confined file without reopening the root by pathname.
    pub fn read(&self, relative: &Path) -> io::Result<Vec<u8>> {
        // Input names are literal local names. The directory capability itself
        // confines resolution; export portability rules apply only to writes.
        self.dir.read(relative)
    }

    /// Stage and verify every file before committing any. Replacement is atomic
    /// per file, not across the set. Existing outputs survive all staging errors.
    ///
    /// # Errors
    /// Rejects nonportable paths, escaping symlinks, output symlinks and conflicts.
    /// A commit or durability error reports the files already committed.
    /// Cleanup failures are returned as warnings on either outcome.
    pub fn publish(
        &self,
        files: &[(PathBuf, &[u8])],
        overwrite: bool,
    ) -> Result<PublishReport, PublishError> {
        // Validate all names before even creating output directories.
        for (path, _) in files {
            validate_relative(path).map_err(|source| PublishError {
                committed: vec![],
                durability_uncertain: false,
                warnings: vec![],
                source,
            })?;
        }
        transaction(
            &mut FsBackend {
                root: Some(&self.dir),
                overwrite,
                #[cfg(test)]
                fail: None,
                #[cfg(test)]
                fail_cleanup: false,
            },
            files,
        )
    }
}

/// Publish explicitly authorized local paths. Each file's parent is opened
/// once and held through staging, replacement and directory sync. Unlike
/// [`OutputRoot`], this API grants ambient local access (the CLI policy).
/// Replacing a symlink replaces the link itself, never its referent.
///
/// # Errors
/// See [`PublishError`]; no fallback removes an old destination.
/// Cleanup failures are returned as warnings, including after successful installation.
pub fn publish_paths(
    files: &[(PathBuf, &[u8])],
    overwrite: bool,
) -> Result<PublishReport, PublishError> {
    transaction(
        &mut FsBackend {
            root: None,
            overwrite,
            #[cfg(test)]
            fail: None,
            #[cfg(test)]
            fail_cleanup: false,
        },
        files,
    )
}

fn validate_relative(path: &Path) -> io::Result<()> {
    let bad = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsafe export path: {}", path.display()),
        )
    };
    let text = path.to_str().ok_or_else(bad)?;
    if text.is_empty() {
        return Err(bad());
    }
    for part in text.split('/') {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.len() > 255
            || part.ends_with(['.', ' '])
            || part.chars().any(|c| {
                c.is_control()
                    || matches!(
                        c,
                        '\\' | ':' | '%' | '<' | '>' | '"' | '|' | '?' | '*'
                            | '\u{061c}' | '\u{200e}' | '\u{200f}'
                            | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
                            | '\u{feff}'
                    )
            })
        {
            return Err(bad());
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|n| {
                matches!(
                    n,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        }) {
            return Err(bad());
        }
    }
    Ok(())
}

struct FsBackend<'a> {
    root: Option<&'a cap_std::fs::Dir>,
    overwrite: bool,
    #[cfg(test)]
    fail: Option<Phase>,
    #[cfg(test)]
    fail_cleanup: bool,
}
/// Verified bytes held in private staging beside an authorized local output.
/// Dropping an uncommitted file removes its staging directory best effort.
pub struct StagedFile {
    path: PathBuf,
    parent: cap_std::fs::Dir,
    scratch: Option<cap_std::fs::Dir>,
    scratch_name: Option<String>,
    file: Option<cap_std::fs::File>,
    target: std::ffi::OsString,
    overwrite: bool,
    #[cfg(test)]
    fail_cleanup: bool,
}
impl StagedFile {
    /// Write, flush, sync and verify bytes without changing the destination.
    pub fn prepare(path: &Path, bytes: &[u8], overwrite: bool) -> io::Result<Self> {
        let mut backend = FsBackend {
            root: None,
            overwrite,
            #[cfg(test)]
            fail: None,
            #[cfg(test)]
            fail_cleanup: false,
        };
        let mut staged = backend.stage(path)?;
        for phase in [Phase::Write, Phase::Flush, Phase::Sync, Phase::Validate] {
            backend.step(&mut staged, phase, bytes)?;
        }
        Ok(staged)
    }

    /// Install the verified bytes, then acknowledge parent-directory durability.
    /// A durability error names the output that has already committed.
    pub fn commit(mut self) -> Result<PublishReport, PublishError> {
        let mut backend = FsBackend {
            root: None,
            overwrite: self.overwrite,
            #[cfg(test)]
            fail: None,
            #[cfg(test)]
            fail_cleanup: false,
        };
        let result = (|| {
            backend
                .step(&mut self, Phase::Commit, &[])
                .map_err(|source| PublishError {
                    committed: vec![],
                    durability_uncertain: false,
                    source,
                    warnings: vec![],
                })?;
            backend
                .step(&mut self, Phase::SyncParent, &[])
                .map_err(|source| PublishError {
                    committed: vec![self.path.clone()],
                    durability_uncertain: true,
                    source,
                    warnings: vec![],
                })
        })();
        let warnings = self
            .cleanup()
            .err()
            .map(|error| format!("cleaning staging for {}: {error}", self.path.display()))
            .into_iter()
            .collect();
        finish(result, warnings)
    }

    /// Remove staging after closing all of its handles. Safe to call again.
    /// A swapped symlink is refused; its referent must never be traversed.
    fn cleanup(&mut self) -> io::Result<()> {
        drop(self.file.take());
        // cap-std directory handles deny FILE_SHARE_DELETE on Windows. Even
        // a clone passed to remove_open_dir_all leaves the original blocking it.
        drop(self.scratch.take());
        let Some(name) = self.scratch_name.as_ref() else {
            return Ok(());
        };
        #[cfg(test)]
        if self.fail_cleanup {
            return Err(io::Error::other("injected cleanup"));
        }
        match self.parent.symlink_metadata(name) {
            Ok(metadata) if metadata.is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "refusing staging cleanup through a symlink",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.scratch_name = None;
                return Ok(());
            }
            Err(error) => return Err(error),
            Ok(_) => (),
        }
        self.parent.remove_dir_all(name)?;
        self.scratch_name = None;
        Ok(())
    }
}

/// Windows directory handles are read-only, but `FlushFileBuffers` needs
/// `GENERIC_WRITE`. NTFS journals metadata; directory sync is a no-op there.
/// Reopening the parent for writing by path would break root confinement (D35).
fn sync_directory_for_platform(
    windows: bool,
    sync: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    if windows { Ok(()) } else { sync() }
}

/// cap-std opens directories `O_PATH` on Linux, and `fsync` on an `O_PATH`
/// descriptor is EBADF (#1351). Opening `.` through the handle gives a
/// readable descriptor without walking a path, so confinement holds (D35).
fn sync_directory_handle(dir: &cap_std::fs::Dir) -> io::Result<()> {
    sync_directory_for_platform(cfg!(windows), || {
        dir.open_with(".", cap_std::fs::OpenOptions::new().read(true))?
            .into_std()
            .sync_all()
    })
}

/// Acknowledge directory durability using a held directory handle (no-op on Windows).
pub fn sync_directory(path: &Path) -> io::Result<()> {
    let dir = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority())?;
    sync_directory_handle(&dir)
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
impl Backend for FsBackend<'_> {
    type Staged = StagedFile;
    fn stage(&mut self, path: &Path) -> io::Result<StagedFile> {
        use cap_std::fs::{Dir, DirBuilder, OpenOptions};
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let target = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output has no filename"))?
            .to_os_string();
        let parent = if let Some(root) = self.root {
            let mut dir = root.try_clone()?;
            for component in parent.components() {
                if let std::path::Component::Normal(name) = component {
                    match dir.create_dir(name) {
                        Ok(()) => sync_directory_handle(&dir)?,
                        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
                        Err(e) => return Err(e),
                    }
                    dir = dir.open_dir(name)?;
                }
            }
            dir
        } else {
            Dir::open_ambient_dir(parent, cap_std::ambient_authority())?
        };
        let mut permissions = None;
        match parent.symlink_metadata(&target) {
            Ok(metadata) => {
                if !self.overwrite {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "{} exists; pass overwrite=true or --force to replace it",
                            path.display()
                        ),
                    ));
                }
                if metadata.is_dir() || (self.root.is_some() && metadata.is_symlink()) {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("output is a directory or symlink: {}", path.display()),
                    ));
                }
                if metadata.is_file() {
                    permissions = Some(metadata.permissions());
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        }
        let builder = DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use cap_std::fs::DirBuilderExt;
            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        let name = format!(".jubarte-{}.partial", uuid::Uuid::new_v4());
        // Exclusive mkdir: a collision is an error, never an existing staging area.
        parent.create_dir_with(&name, &builder)?;
        let scratch = parent.open_dir(&name).inspect_err(|_| {
            let _ = parent.remove_dir(&name);
        })?;
        let mut staged = StagedFile {
            path: path.to_path_buf(),
            parent,
            scratch: Some(scratch),
            scratch_name: Some(name),
            file: None,
            target,
            overwrite: self.overwrite,
            #[cfg(test)]
            fail_cleanup: self.fail_cleanup,
        };
        staged.file = Some(
            staged
                .scratch
                .as_ref()
                .expect("open staging directory")
                .open_with(
                    "bytes",
                    OpenOptions::new().read(true).write(true).create_new(true),
                )?,
        );
        if let Some(permissions) = permissions {
            staged
                .file
                .as_ref()
                .expect("open staging file")
                .set_permissions(permissions)?;
        }
        Ok(staged)
    }
    fn cleanup(&mut self, staged: &mut StagedFile) -> io::Result<()> {
        staged.cleanup()
    }
    fn step(&mut self, staged: &mut StagedFile, phase: Phase, bytes: &[u8]) -> io::Result<()> {
        use std::io::{Read, Seek, SeekFrom, Write};
        #[cfg(test)]
        if self.fail == Some(phase) {
            if phase == Phase::Commit {
                staged
                    .scratch
                    .as_ref()
                    .expect("open staging directory")
                    .remove_file("bytes")?;
            } else {
                return Err(io::Error::other(format!("injected {phase:?}")));
            }
        }
        let file = staged.file.as_mut().expect("open staging file");
        match phase {
            Phase::Write => file.write_all(bytes),
            Phase::Flush => file.flush(),
            Phase::Sync => file.sync_all(),
            Phase::Validate => {
                file.seek(SeekFrom::Start(0))?;
                if file.metadata()?.len() != bytes.len() as u64 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "staged output length mismatch",
                    ));
                }
                let mut buffer = [0; 8192];
                for chunk in bytes.chunks(buffer.len()) {
                    file.read_exact(&mut buffer[..chunk.len()])?;
                    if buffer[..chunk.len()] != *chunk {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "staged output bytes mismatch",
                        ));
                    }
                }
                Ok(())
            }
            Phase::Commit if staged.overwrite => staged
                .scratch
                .as_ref()
                .expect("open staging directory")
                .rename("bytes", &staged.parent, &staged.target),
            // hard_link is an atomic no-clobber installation, including when a
            // hostile writer creates the target after the initial check.
            Phase::Commit => staged
                .scratch
                .as_ref()
                .expect("open staging directory")
                .hard_link("bytes", &staged.parent, &staged.target),
            Phase::SyncParent => sync_directory_handle(&staged.parent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    // Filesystem integration probes: exercise actual handle lifetime and hard links.
    #[test]
    fn cleanup_failure_is_reported_after_install_and_alongside_publish_failure() {
        for phase in [
            None,
            Some(Phase::Write),
            Some(Phase::Commit),
            Some(Phase::SyncParent),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = OutputRoot::open(temp.path()).unwrap();
            let mut backend = FsBackend {
                root: Some(&root.dir),
                overwrite: false,
                fail: phase,
                fail_cleanup: true,
            };
            let result = transaction(&mut backend, &[("output".into(), b"Alice")]);
            let installed = phase.is_none() || phase == Some(Phase::SyncParent);
            assert_eq!(temp.path().join("output").exists(), installed);
            if installed {
                assert_eq!(root.read(Path::new("output")).unwrap(), b"Alice");
            }
            let warnings = match result {
                Ok(report) => {
                    assert!(phase.is_none());
                    report.warnings
                }
                Err(error) => {
                    assert!(phase.is_some());
                    assert_eq!(
                        error.committed,
                        if installed {
                            vec![PathBuf::from("output")]
                        } else {
                            vec![]
                        }
                    );
                    assert_eq!(error.durability_uncertain, phase == Some(Phase::SyncParent));
                    if phase == Some(Phase::Commit) {
                        assert_eq!(error.source.kind(), io::ErrorKind::NotFound);
                    } else {
                        assert!(
                            error
                                .source
                                .to_string()
                                .contains(&format!("injected {:?}", phase.unwrap()))
                        );
                    }
                    error.warnings
                }
            };
            assert_eq!(warnings.len(), 1);
            assert!(
                warnings[0].contains("injected cleanup"),
                "cleanup failure was swallowed: {warnings:?}"
            );
        }
    }

    #[test]
    fn prepared_commit_reports_cleanup_failure_after_install() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("output");
        let mut staged = StagedFile::prepare(&path, b"Bob", false).unwrap();
        staged.fail_cleanup = true;
        let result = staged.commit();
        assert_eq!(std::fs::read(path).unwrap(), b"Bob");
        let report = result.unwrap();
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("injected cleanup"));
    }

    #[test]
    fn every_publish_path_removes_partial_directories() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("output");
        // Fresh replacement, forced replacement, then no-overwrite hard-link commit.
        for overwrite in [true, true, false] {
            if !overwrite {
                std::fs::remove_file(&output).unwrap();
            }
            let report = publish_paths(&[(output.clone(), b"Carol Diaz")], overwrite).unwrap();
            assert!(report.warnings.is_empty(), "{report:?}");
            assert_no_partials(temp.path());
            assert_eq!(std::fs::read(&output).unwrap(), b"Carol Diaz");
        }
        for phase in [
            Phase::Write,
            Phase::Flush,
            Phase::Sync,
            Phase::Validate,
            Phase::Commit,
            Phase::SyncParent,
        ] {
            let mut backend = FsBackend {
                root: None,
                overwrite: true,
                fail: Some(phase),
                fail_cleanup: false,
            };
            let error = transaction(
                &mut backend,
                &[
                    (output.clone(), b"Alice"),
                    (temp.path().join("second"), b"Bob"),
                ],
            )
            .unwrap_err();
            assert!(error.warnings.is_empty(), "{error:?}");
            assert_no_partials(temp.path());
        }
    }

    fn assert_no_partials(parent: &Path) {
        let partials: Vec<_> = std::fs::read_dir(parent)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .filter(|name| {
                let name = name.to_string_lossy();
                name.starts_with(".jubarte-") && name.ends_with(".partial")
            })
            .collect();
        assert!(partials.is_empty(), "staging leftovers: {partials:?}");
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_refuses_a_swapped_scratch_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("keep"), b"Carol Diaz").unwrap();
        let path = temp.path().join("output");
        let staged = StagedFile::prepare(&path, b"Alice", false).unwrap();
        let name = std::fs::read_dir(temp.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name();
        std::fs::rename(temp.path().join(&name), temp.path().join("held")).unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join(&name)).unwrap();
        let result = staged.commit();
        assert_eq!(
            std::fs::read(outside.path().join("keep")).unwrap(),
            b"Carol Diaz"
        );
        assert!(
            temp.path()
                .join(&name)
                .symlink_metadata()
                .unwrap()
                .is_symlink()
        );
        let report = format!("{result:?}");
        assert!(
            report.contains("symlink"),
            "swapped symlink was not reported: {report}"
        );
    }

    #[test]
    fn windows_directory_sync_is_a_noop() {
        let mut called = false;
        let result = sync_directory_for_platform(true, || {
            called = true;
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "read-only directory",
            ))
        });
        assert!(
            result.is_ok(),
            "Windows directory sync must succeed: {result:?}"
        );
        assert!(!called, "Windows must not attempt the directory sync");
    }

    #[test]
    fn unix_directory_sync_calls_the_operation_and_preserves_its_result() {
        for kind in [None, Some(io::ErrorKind::PermissionDenied)] {
            let mut called = false;
            let result = sync_directory_for_platform(false, || {
                called = true;
                match kind {
                    None => Ok(()),
                    Some(kind) => Err(io::Error::new(kind, "read-only directory")),
                }
            });
            assert!(called, "Unix must attempt the directory sync");
            assert_eq!(result.as_ref().err().map(io::Error::kind), kind);
            if let Err(error) = result {
                assert_eq!(error.to_string(), "read-only directory");
            }
        }
    }

    /// cap-std opens directories `O_PATH` on Linux, and `fsync` on an
    /// `O_PATH` descriptor is EBADF (#1351): every publication failed after
    /// its commit there, with "durability uncertain".
    #[test]
    fn a_cap_std_directory_handle_can_be_synced() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("sub")).unwrap();
        let dir =
            cap_std::fs::Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).unwrap();
        sync_directory_handle(&dir).expect("an ambient directory handle syncs");
        sync_directory_handle(&dir.open_dir("sub").unwrap())
            .expect("a directory opened through a handle syncs");
        sync_directory(temp.path()).expect("sync_directory syncs");
    }

    /// Deterministic filesystem fake: all state and injected failures stay in memory.
    struct Memory {
        visible: BTreeMap<PathBuf, Vec<u8>>,
        fail: Option<(usize, Phase)>,
        count: usize,
        order: Vec<(usize, Phase)>,
        fail_stage: Option<usize>,
    }
    impl Memory {
        fn new(fail: Option<(usize, Phase)>) -> Self {
            Self {
                visible: [
                    (PathBuf::from("a"), b"old a".to_vec()),
                    (PathBuf::from("b"), b"old b".to_vec()),
                ]
                .into(),
                fail,
                count: 0,
                order: vec![],
                fail_stage: None,
            }
        }
    }
    impl Backend for Memory {
        type Staged = (usize, PathBuf, Vec<u8>);
        fn stage(&mut self, path: &Path) -> io::Result<Self::Staged> {
            let id = self.count;
            self.count += 1;
            if self.fail_stage == Some(id) {
                return Err(io::Error::other("injected staging creation failure"));
            }
            Ok((id, path.into(), vec![]))
        }
        fn cleanup(&mut self, _staged: &mut Self::Staged) -> io::Result<()> {
            Ok(())
        }
        fn step(
            &mut self,
            staged: &mut Self::Staged,
            phase: Phase,
            bytes: &[u8],
        ) -> io::Result<()> {
            let (id, path, held) = staged;
            self.order.push((*id, phase));
            if self.fail == Some((*id, phase)) {
                return Err(io::Error::other(format!("injected {phase:?}")));
            }
            match phase {
                Phase::Write => *held = bytes.to_vec(),
                Phase::Commit => {
                    self.visible.insert(path.clone(), held.clone());
                }
                _ => (),
            }
            Ok(())
        }
    }
    fn files() -> [(PathBuf, &'static [u8]); 2] {
        [("a".into(), b"new a"), ("b".into(), b"new b")]
    }

    fn fs_failure(phase: Phase) {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("output"), b"old output").unwrap();
        let root = OutputRoot::open(temp.path()).unwrap();
        let mut backend = FsBackend {
            root: Some(&root.dir),
            overwrite: true,
            fail: Some(phase),
            fail_cleanup: false,
        };
        let error = transaction(&mut backend, &[("output".into(), b"new output")])
            .expect_err("configured filesystem fault must fail publication");
        let installed = phase == Phase::SyncParent;
        assert_eq!(
            root.read(Path::new("output")).unwrap(),
            if installed {
                b"new output"
            } else {
                b"old output"
            }
        );
        assert_eq!(
            error.committed,
            if installed {
                vec![PathBuf::from("output")]
            } else {
                vec![]
            }
        );
        assert_eq!(error.durability_uncertain, installed);
        if installed {
            assert!(
                error.to_string().contains("durability uncertain"),
                "{error}"
            );
            assert!(
                error.source.to_string().contains("injected SyncParent"),
                "{error}"
            );
        } else {
            assert_eq!(error.source.kind(), io::ErrorKind::NotFound);
        }
        let remaining: Vec<_> = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(remaining, [std::ffi::OsString::from("output")]);
    }

    #[test]
    fn audit_d13_fs_rename_failure_preserves_prior_output_and_cleans_staging() {
        fs_failure(Phase::Commit);
    }

    #[test]
    fn audit_d13_fs_parent_sync_failure_reports_installed_output_and_cleans_staging() {
        fs_failure(Phase::SyncParent);
    }

    #[test]
    fn audit_d13_all_precommit_failures_preserve_every_prior_output() {
        for phase in [Phase::Write, Phase::Flush, Phase::Sync, Phase::Validate] {
            for index in 0..2 {
                let mut fs = Memory::new(Some((index, phase)));
                let error = transaction(&mut fs, &files()).unwrap_err();
                assert!(error.source.to_string().contains("injected"), "{error}");
                assert!(error.committed.is_empty());
                assert!(!error.durability_uncertain);
                assert_eq!(fs.visible, Memory::new(None).visible);
            }
        }
    }
    #[test]
    fn audit_d13_rename_failure_never_deletes_old_and_reports_prior_commits() {
        for index in 0..2 {
            let mut fs = Memory::new(Some((index, Phase::Commit)));
            let error = transaction(&mut fs, &files()).unwrap_err();
            assert!(error.source.to_string().contains("injected"), "{error}");
            assert_eq!(error.committed.len(), index);
            assert_eq!(fs.visible[Path::new("b")], b"old b");
            assert_eq!(
                fs.visible[Path::new("a")],
                if index == 0 { b"old a" } else { b"new a" }
            );
        }
    }
    #[test]
    fn audit_d13_postcommit_sync_failure_reports_visible_output_and_uncertain_durability() {
        let mut fs = Memory::new(Some((0, Phase::SyncParent)));
        let error = transaction(&mut fs, &files()).unwrap_err();
        assert_eq!(error.committed, [PathBuf::from("a")]);
        assert!(error.durability_uncertain);
        assert_eq!(fs.visible[Path::new("a")], b"new a");
        assert_eq!(fs.visible[Path::new("b")], b"old b");
    }
    #[test]
    fn audit_d13_validates_the_entire_set_before_the_first_commit() {
        let mut fs = Memory::new(None);
        transaction(&mut fs, &files()).unwrap();
        assert_eq!(fs.visible[Path::new("b")], b"new b");
        let first_commit = fs
            .order
            .iter()
            .position(|(_, p)| *p == Phase::Commit)
            .unwrap();
        let last_validate = fs
            .order
            .iter()
            .rposition(|(_, p)| *p == Phase::Validate)
            .unwrap();
        assert!(last_validate < first_commit);
    }
    #[test]
    fn audit_d35_portable_names_cannot_smuggle_extra_path_authority() {
        for path in [
            "",
            "PRN",
            "AUX",
            "CONIN$",
            "CONOUT$",
            "COM¹.txt",
            "LPT².docx",
            "a<b",
            "a>b",
            "a\"b",
            "a|b",
            "a?b",
            "a*b",
            "../escape",
            "/absolute",
            "ok/../../escape",
            "C:\\escape",
            "a\\b",
            "NUL.txt",
            "con",
            "COM1.docx",
            "LPT9",
            "file:stream",
            "trailing.",
            "trailing ",
            "a/../b",
            "a//b",
            "./a",
            "a/%2e%2e/b",
            "a/%2Ftmp",
            "a/line\n",
            "a\u{061c}b",
            "a\u{200e}b",
            "a\u{200f}b",
            "a\u{202a}b",
            "a\u{202b}b",
            "a\u{202c}b",
            "a\u{202d}b",
            "a\u{202e}b",
            "a\u{2066}b",
            "a\u{2067}b",
            "a\u{2068}b",
            "a\u{2069}b",
            "a\u{feff}b",
        ] {
            assert!(
                validate_relative(Path::new(path)).is_err(),
                "accepted {path:?}"
            );
        }
        assert!(validate_relative(Path::new(&"x".repeat(256))).is_err());
        for path in [
            "page-01.png",
            "media/0123456789abcdef.png",
            "new folder/redline.docx",
            "نام\u{200c}نامه.docx",
            "नाम\u{200d}पत्र.docx",
        ] {
            assert!(validate_relative(Path::new(path)).is_ok(), "refused {path}");
        }
    }
    #[test]
    fn staging_creation_failure_and_duplicate_names_never_commit() {
        let mut fs = Memory::new(None);
        fs.fail_stage = Some(1);
        let error = transaction(&mut fs, &files()).unwrap_err();
        assert!(error.committed.is_empty());
        assert_eq!(fs.visible, Memory::new(None).visible);
        let mut fs = Memory::new(None);
        let error = transaction(
            &mut fs,
            &[("a".into(), b"first".as_slice()), ("a".into(), b"second")],
        )
        .unwrap_err();
        assert_eq!(error.source.kind(), io::ErrorKind::InvalidInput);
        assert!(error.committed.is_empty());
        assert_eq!(fs.visible, Memory::new(None).visible);
        assert!(transaction(&mut fs, &[]).is_ok());
    }
}
