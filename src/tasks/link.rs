//! The link library task.
use crate::opts::LinkOptions;
use crate::tasks::ResolveEnv;
use crate::tasks::TaskError;
use crate::tasks::task::TaskStatus;
use crate::utils::files;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use chrono::DateTime;
use chrono::Utc;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use color_eyre::eyre::ensure;
use color_eyre::eyre::eyre;
use displaydoc::Display;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::os::unix;
use thiserror::Error;
use tracing::debug;
use tracing::info;
use tracing::trace;
use tracing::warn;
use walkdir::DirEntry;
use walkdir::WalkDir;

impl ResolveEnv for LinkOptions {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<(), TaskError>
    where
        F: Fn(&str) -> Result<String, TaskError>,
    {
        self.from_dir = env_fn(&self.from_dir)?;
        self.to_dir = env_fn(&self.to_dir)?;
        Ok(())
    }
}

/// Symlink everything from `to_dir` (default: ~/code/dotfiles/) into `from_dir`
/// (default: ~). Anything that would be overwritten is copied into `backup_dir`
/// (default: `up_dir/backup/link/`).
///
/// Basically you put your dotfiles in ~/code/dotfiles/, in the same structure
/// they were in relative to ~. Then if you want to edit your .bashrc (for
/// example) you just edit ~/.bashrc, and as it's a symlink it'll actually edit
/// ~/code/dotfiles/.bashrc. Then you can add and commit that change in ~/code/
/// dotfiles.
pub(crate) fn run(config: LinkOptions, up_dir: &Utf8Path) -> Result<TaskStatus> {
    let now: DateTime<Utc> = Utc::now();
    debug!("UTC time is: {now}");

    let from_dir = Utf8PathBuf::from(config.from_dir);
    let to_dir = Utf8PathBuf::from(config.to_dir);
    let backup_dir = up_dir.join("backup/link");

    let from_dir = resolve_directory(from_dir, "From")?;
    let to_dir = resolve_directory(to_dir, "To")?;

    // Create the backup dir if it doesn't exist.
    if !backup_dir.exists() {
        debug!("Backup dir '{backup_dir}' doesn't exist, creating it.",);
        fs::create_dir_all(&backup_dir).map_err(|e| LinkError::CreateDirError {
            path: backup_dir.clone(),
            source: e,
        })?;
    }
    let backup_dir = resolve_directory(backup_dir, "Backup")?;

    debug!("Linking from {from_dir} to {to_dir} (backup dir {backup_dir}).",);
    debug!(
        "to_dir contents: {:?}",
        fs::read_dir(&to_dir)?
            .filter_map(|d| d
                .ok()
                .map(|x| Ok(x.path().strip_prefix(&to_dir)?.to_path_buf())))
            .collect::<Result<Vec<_>>>()
    );

    let mut work_done = false;
    // For each non-directory file in from_dir.
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|f| !f.file_type().is_dir())
    {
        let rel_path = Utf8Path::from_path(from_path.path())
            .ok_or_else(|| eyre!("Invalid path {from_path:?}"))?
            .strip_prefix(&from_dir)?;
        create_parent_dir(&to_dir, rel_path, &backup_dir)?;
        if link_path(&from_path, &to_dir, rel_path, &backup_dir)? {
            work_done = true;
        }
    }

    // Remove backup dir if not empty.
    match fs::remove_dir(&backup_dir) {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            trace!("Looks like another link process already cleaned the backup directory.");
        }

        Err(e) => warn!("Backup dir {backup_dir} non-empty, check contents: {e:?}"),
        Ok(()) => (),
    }

    debug!(
        "to_dir final contents: {:#?}",
        fs::read_dir(&to_dir)?
            .filter_map(|e| e.ok().map(|d| d.path()))
            .collect::<Vec<_>>()
    );

    if backup_dir.exists() {
        debug!(
            "backup_dir final contents: {:#?}",
            fs::read_dir(&backup_dir)?
                .filter_map(|e| e.ok().map(|d| d.path()))
                .collect::<Vec<_>>()
        );
    }

    if work_done {
        Ok(TaskStatus::Passed)
    } else {
        Ok(TaskStatus::Skipped)
    }
}

/// Ensure dir exists, and resolve symlinks to find it's canonical path.
fn resolve_directory(dir_path: Utf8PathBuf, name: &str) -> Result<Utf8PathBuf> {
    ensure!(
        &dir_path.is_dir(),
        LinkError::MissingDir {
            name: name.to_owned(),
            path: dir_path
        }
    );

    dir_path.canonicalize_utf8().map_err(|e| {
        LinkError::CanonicalizeError {
            path: dir_path,
            source: e,
        }
        .into()
    })
}

/// Create the parent directory to create the symlink in.
fn create_parent_dir(to_dir: &Utf8Path, rel_path: &Utf8Path, backup_dir: &Utf8Path) -> Result<()> {
    let to_path = to_dir.join(rel_path);
    let to_path_parent = get_parent_path(&to_path)?;
    fs::create_dir_all(to_path_parent).or_else(|_err| {
        info!(
            "Failed to create parent dir, walking up the tree to see if there's a file that needs \
             to become a directory."
        );
        for path in rel_path
            .ancestors()
            .skip(1)
            .filter(|p| p != &Utf8Path::new(""))
        {
            debug!("Checking path {path}");
            let abs_path = to_dir.join(path);
            // The path is a file/dir/symlink, or a broken symlink.
            if abs_path.exists() || abs_path.symlink_metadata().is_ok() {
                ensure!(
                    !abs_path.is_dir(),
                    "Failed to create the parent directory for the symlink. We assumed it was \
                     because one of the parent directories was a file or symlink, but that \
                     doesn't seem to be the case, as the first file we've come across that exists \
                     is a directory.\n  Path: {abs_path}",
                );
                warn!(
                    "File will be overwritten by parent directory of link.\n  File: {abs_path}\n  \
                     Link: {to_path}",
                );
                if abs_path.is_file() {
                    if let Some(parent_path) = &path.parent() {
                        info!("Path: {path}, parent: {parent_path}");
                        if parent_path != &Utf8Path::new("") {
                            let path = backup_dir.join(parent_path);
                            fs::create_dir_all(&path)
                                .map_err(|e| LinkError::CreateDirError { path, source: e })?;
                        }
                        let backup_path = backup_dir.join(path);
                        info!("Moving file to backup: {abs_path} -> {backup_path}",);
                        fs::rename(&abs_path, backup_path)?;
                    }
                } else {
                    info!("Removing symlink: {abs_path}");
                    fs::remove_file(abs_path)?;
                }
            }
        }
        // We should be able to create the directory now (if not bail with a Failure error).
        let to_parent_path = get_parent_path(&to_path)?;
        fs::create_dir_all(to_parent_path)
            .wrap_err_with(|| format!("Failed to create parent dir {:?}.", to_path.parent()))
    })
}

/// Get the parent directory of a path.
fn get_parent_path(path: &Utf8Path) -> Result<&Utf8Path> {
    Ok(path.parent().ok_or_else(|| LinkError::MissingParentDir {
        path: path.to_path_buf(),
    })?)
}

/// Create a symlink from `from_path` -> `to_path`.
/// `rel_path` is the relative path within `from_dir`.
/// Moves any existing files that would be overwritten into `backup_dir`.
/// Returns a boolean indicating whether any symlinks were created.
#[allow(clippy::filetype_is_file)]
fn link_path(
    from_path_direntry: &DirEntry,
    to_dir: &Utf8Path,
    rel_path: &Utf8Path,
    backup_dir: &Utf8Path,
) -> Result<bool> {
    let to_path = to_dir.join(rel_path);
    let from_path = Utf8Path::from_path(from_path_direntry.path())
        .ok_or_else(|| eyre!("Invalid UTF-8 in path {from_path_direntry:?}"))?;
    if to_path.exists() {
        let to_path_file_type = to_path.symlink_metadata()?.file_type();
        if to_path_file_type.is_symlink() {
            match to_path.read_link_utf8() {
                Ok(existing_link) => {
                    if existing_link == from_path {
                        debug!("Link at {to_path} already points to {existing_link}, skipping.",);
                        return Ok(false);
                    }
                    warn!("Link at {to_path} points to {existing_link}, changing to {from_path}.");
                    fs::remove_file(&to_path).map_err(|e| LinkError::DeleteError {
                        path: to_path.clone(),
                        source: e,
                    })?;
                }
                Err(e) => {
                    bail!("read_link returned error {e:?} for {to_path}");
                }
            }
        } else if to_path_file_type.is_dir() {
            warn!("Expected file or link at {to_path}, found directory, moving to {backup_dir}",);
            let backup_path = backup_dir.join(rel_path);
            fs::create_dir_all(&backup_path).map_err(|e| LinkError::CreateDirError {
                path: backup_path.clone(),
                source: e,
            })?;
            fs::rename(&to_path, &backup_path).map_err(|e| LinkError::RenameError {
                from_path: to_path.clone(),
                to_path: backup_path,
                source: e,
            })?;
        } else if to_path_file_type.is_file() {
            warn!("Existing file at {to_path}, moving to {backup_dir}");
            let backup_path = backup_dir.join(rel_path);
            let backup_parent_path = get_parent_path(&backup_path)?;
            fs::create_dir_all(backup_parent_path).map_err(|e| LinkError::CreateDirError {
                path: backup_parent_path.to_path_buf(),
                source: e,
            })?;
            fs::rename(&to_path, &backup_path).map_err(|e| LinkError::RenameError {
                from_path: to_path.clone(),
                to_path: backup_path,
                source: e,
            })?;
        } else {
            bail!("This should be unreachable.")
        }
    } else if to_path.symlink_metadata().is_ok() {
        files::remove_broken_symlink(&to_path)?;
    } else {
        trace!("File '{to_path}' doesn't exist.");
    }
    info!("Linking:\n  From: {from_path}\n  To: {to_path}");
    unix::fs::symlink(from_path, &to_path)
        // If we got here, we did work, so return true.
        .map(|()| true)
        .map_err(|e| {
            LinkError::SymlinkError {
                from_path: from_path.to_owned(),
                to_path: to_path.clone(),
                source: e,
            }
            .into()
        })
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum LinkError {
    /// {name} directory `{path}` should exist and be a directory.
    MissingDir {
        /// Directory name.
        name: String,
        /// Directory path.
        path: Utf8PathBuf,
    },
    /// Error canonicalizing `{path}`.
    CanonicalizeError {
        /// Path we failed  to canonicalize.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Failed to create directory `{path}`
    CreateDirError {
        /// Directory path we failed to create.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Failed to delete `{path}`.
    DeleteError {
        /// Path we failed to delete.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Failure for path `{path}`.
    IoError {
        /// Path we got an IO error for.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Failed to rename from `{from_path}` to `{to_path}`.
    RenameError {
        /// Existing name.
        from_path: Utf8PathBuf,
        /// New name.
        to_path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Failed to symlink from `{from_path}` to `{to_path}`.
    SymlinkError {
        /// Real file we were trying to symlink from.
        from_path: Utf8PathBuf,
        /// Symbolic link we were trying to create.
        to_path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Path `{path}` should have a parent directory.
    MissingParentDir {
        /// Path that doesn't have a parent dir.
        path: Utf8PathBuf,
    },
}
