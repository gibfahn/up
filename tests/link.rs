use assert_cmd::assert::Assert;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::Result;
use std::fs;
use std::fs::File;
use std::os::unix;
use testutils::ensure_utils;

/// Set up a basic `home_dir`, run the link function against it, and make sure we
/// get the expected changes.
#[test]
fn test_new_link() -> Result<()> {
    let (home_dir, dotfile_dir, backup_dir, temp_dir) =
        get_home_dotfile_dirs(testutils::function_path!())?;
    // Create empty dir (can't check in as git doesn't store dirs without contents.
    fs::create_dir(home_dir.join("existing_dir")).unwrap();
    run_link_cmd(&dotfile_dir, &home_dir, &temp_dir, LinkResult::Success)?;

    // Existing files shouldn't be touched.
    ensure_utils::file(&home_dir.join("existing_file"), "existing file 1\n")?;
    // Existing dirs shouldn't be touched.
    ensure_utils::dir(&home_dir.join("existing_dir"))?;
    // Files should be linked.
    ensure_utils::link(&home_dir.join("file"), &dotfile_dir.join("file"))?;
    // Links should be linked.
    ensure_utils::link(&home_dir.join("good_link"), &dotfile_dir.join("good_link"))?;
    // Empty backup dir should be removed.
    ensure_utils::nothing_at(&backup_dir)?;

    Ok(())
}

/// Set up a basic `home_dir`, run the link function against it, and make sure we
/// get the expected changes.
#[test]
fn test_backup_files() -> Result<()> {
    let (home_dir, dotfile_dir, backup_dir, temp_dir) =
        get_home_dotfile_dirs(testutils::function_path!())?;
    run_link_cmd(&dotfile_dir, &home_dir, &temp_dir, LinkResult::Success)?;

    // Backup dir should stay.
    ensure_utils::dir(&backup_dir)?;
    // Files in backup should be overwritten with the new backups.
    ensure_utils::file(&backup_dir.join("already_in_backup"), "new backup\n")?;
    // Symlinks in home should be overwritten.
    ensure_utils::link(
        &home_dir.join("existing_symlink"),
        &dotfile_dir.join("existing_symlink"),
    )?;
    // Files in home should become symlinks.
    ensure_utils::link(
        &home_dir.join("already_in_backup"),
        &dotfile_dir.join("already_in_backup"),
    )?;
    // Symlinks in home should not be moved to backup.
    ensure_utils::nothing_at(&backup_dir.join("existing_symlink"))?;

    // Existing subdir backup files should not be overwritten.
    ensure_utils::file(
        &backup_dir.join("subdir/prev_backup_subdir_file"),
        "previous backup subdir file\n",
    )?;
    // Existing subdir files should not be overwritten.
    ensure_utils::file(
        &home_dir.join("subdir/existing_subdir_file"),
        "existing subdir file\n",
    )?;
    // Subdirectory files should be moved to backup.
    ensure_utils::file(
        &backup_dir.join("subdir/new_subdir_file"),
        "previous subdir file\n",
    )?;
    // Subdirectory files should be added into existing directories.
    ensure_utils::link(
        &home_dir.join("subdir/new_subdir_file"),
        &dotfile_dir.join("subdir/new_subdir_file"),
    )?;

    // Nested subdirectory files should be moved to backup.
    ensure_utils::file(
        &backup_dir.join("subdir/subdir2/subdir2_file"),
        "old subdir2 file\n",
    )?;
    // Nested subdirectory files should be added into existing directories.
    ensure_utils::link(
        &home_dir.join("subdir/subdir2/subdir2_file"),
        &dotfile_dir.join("subdir/subdir2/subdir2_file"),
    )?;

    Ok(())
}

#[test]
fn test_hidden_and_nested() -> Result<()> {
    let (home_dir, dotfile_dir, backup_dir, temp_dir) =
        get_home_dotfile_dirs(testutils::function_path!())?;
    // If this symlink is correct, it shouldn't make a difference.
    unix::fs::symlink(
        dotfile_dir.join("existing_link"),
        home_dir.join("existing_link"),
    )
    .unwrap();
    run_link_cmd(&dotfile_dir, &home_dir, &temp_dir, LinkResult::Success)?;

    // Backup dir should stay.
    ensure_utils::dir(&backup_dir)?;
    // Hidden files/dirs should still be moved to backup.
    ensure_utils::file(&backup_dir.join(".config/.file"), "old file\n")?;
    // Hidden files/dirs should still be linked to.
    ensure_utils::link(
        &home_dir.join(".config/.file"),
        &dotfile_dir.join(".config/.file"),
    )?;

    // Bad links should be updated (even to other bad links).
    ensure_utils::link(&home_dir.join("bad_link"), &dotfile_dir.join("bad_link"))?;
    // Arbitrarily nested directories should still be linked.
    ensure_utils::link(
        &home_dir.join(".config/a/b/c/d/e/f/g/.other_file"),
        &dotfile_dir.join(".config/a/b/c/d/e/f/g/.other_file"),
    )?;
    // Existing links shouldn't be changed.
    ensure_utils::link(
        &home_dir.join("existing_link"),
        &dotfile_dir.join("existing_link"),
    )?;

    // Directories should be overwritten with file links.
    ensure_utils::link(
        &home_dir.join("dir_to_file"),
        &dotfile_dir.join("dir_to_file"),
    )?;
    // Files inside directories that are converted to file links should be moved to
    // backup.
    ensure_utils::file(
        &backup_dir.join("dir_to_file/file"),
        "dir_to_file dir file\n",
    )?;
    // Files should be overwritten with directories containing file links.
    ensure_utils::dir(&home_dir.join("file_to_dir"))?;
    // Links should be inserted inside directories that overwrite files.
    ensure_utils::link(
        &home_dir.join("file_to_dir/file2"),
        &dotfile_dir.join("file_to_dir/file2"),
    )?;
    // Files that are converted to directories should be moved to backup.
    ensure_utils::file(
        &backup_dir.join("file_to_dir"),
        "file_to_dir original file\n",
    )?;

    // Directories should overwrite links.
    ensure_utils::dir(&home_dir.join("link_to_dir"))?;
    // Links should be inserted inside directories that override links.
    ensure_utils::link(
        &home_dir.join("link_to_dir/file3"),
        &dotfile_dir.join("link_to_dir/file3"),
    )?;
    // Links that are converted to directories should not be moved to backup.
    ensure_utils::nothing_at(&backup_dir.join("link_to_dir"))?;

    // Directories should overwrite bad links.
    ensure_utils::dir(&home_dir.join("badlink_to_dir"))?;
    // Links should be inserted inside directories that override links.
    ensure_utils::link(
        &home_dir.join("badlink_to_dir/file4"),
        &dotfile_dir.join("badlink_to_dir/file4"),
    )?;
    // Links that are converted to directories should not be moved to backup.
    ensure_utils::nothing_at(&backup_dir.join("badlink_to_dir"))?;

    Ok(())
}

/// Pass a `from_dir` that doesn't exist and make sure we fail.
#[test]
fn test_missing_from_dir() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!())?;
    let cmd_assert = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        &temp_dir,
        LinkResult::Failure,
    )?;
    ensure_utils::contains_all(
        &String::from_utf8_lossy(&cmd_assert.get_output().stderr),
        &[
            "From directory",
            "should exist and be a directory.",
            "missing_from_dir/dotfile_dir",
        ],
    )?;

    Ok(())
}

/// Pass a `to_dir` that doesn't exist and make sure we fail.
#[test]
fn test_missing_to_dir() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!())?;
    fs::create_dir(temp_dir.join("dotfile_dir")).unwrap();
    let assert = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        &temp_dir,
        LinkResult::Failure,
    )?;
    ensure_utils::contains_all(
        &String::from_utf8_lossy(&assert.get_output().stderr),
        &[
            "To directory",
            "should exist and be a directory.",
            "missing_to_dir/home_dir",
        ],
    )?;

    Ok(())
}

/// Make sure we fail if the backup dir can't be created (e.g. because it's
/// already a file).
#[test]
fn test_uncreateable_backup_dir() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    fs::create_dir(temp_dir.join("dotfile_dir")).unwrap();
    fs::create_dir(temp_dir.join("home_dir")).unwrap();
    fs::create_dir_all(temp_dir.join("up/backup")).unwrap();
    File::create(temp_dir.join("up/backup/link")).unwrap();
    let assert = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        &temp_dir,
        LinkResult::Failure,
    )?;
    ensure_utils::contains_all(
        &String::from_utf8_lossy(&assert.get_output().stderr),
        &[
            "Backup directory",
            "should exist and be a directory",
            "uncreateable_backup_dir/up/backup/link",
        ],
    )?;

    Ok(())
}

/// Helper function to copy the test fixtures for a given test into the OS
/// tempdir (and return the created `home_dir` and `dotfile_dir` paths.
#[cfg(test)]
fn get_home_dotfile_dirs(
    test_fn: &str,
) -> Result<(Utf8PathBuf, Utf8PathBuf, Utf8PathBuf, Utf8PathBuf)> {
    let temp_dir = testutils::temp_dir("up", test_fn).unwrap();

    testutils::copy_all(&testutils::fixtures_subdir(test_fn)?, &temp_dir)?;

    Ok((
        temp_dir.join("home_dir").canonicalize_utf8().unwrap(),
        temp_dir.join("dotfile_dir").canonicalize_utf8().unwrap(),
        temp_dir.join("up/backup/link"),
        temp_dir,
    ))
}

/// Enum to capture whether we expected the link command to return success or
/// failure?
#[derive(Debug, PartialEq)]
enum LinkResult {
    Success,
    Failure,
}

impl LinkResult {
    /// Convert [`LinkResult`] to a bool ([`LinkResult::Success`] -> true,
    /// [`LinkResult::Failure`] -> false).
    fn to_bool(&self) -> bool {
        match &self {
            LinkResult::Success => true,
            LinkResult::Failure => false,
        }
    }
}

/// Helper function to run ./up link <`home_dir`> <`dotfile_dir`> <`home_dir>/backup`.
#[cfg(test)]
fn run_link_cmd(
    dotfile_dir: &Utf8Path,
    home_dir: &Utf8Path,
    temp_dir: &Utf8Path,
    result: LinkResult,
) -> Result<Assert> {
    use testutils::AssertCmdExt;

    let mut cmd = testutils::crate_binary_cmd("up", temp_dir)?;
    // Always show coloured logs.
    cmd.args(
        [
            "link",
            "--from",
            dotfile_dir.as_str(),
            "--to",
            home_dir.as_str(),
        ]
        .iter(),
    );

    if result.to_bool() {
        Ok(cmd.assert().eprint_stdout_stderr().try_success()?)
    } else {
        Ok(cmd.assert().eprint_stdout_stderr().try_failure()?)
    }
}
