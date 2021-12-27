//! Common functions that are used by other tests.

use std::{
    env, fs,
    io::ErrorKind,
    os::unix,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use color_eyre::eyre::Result;
use walkdir::WalkDir;

pub mod assert;

/// Returns the path to target/debug or target/release.
fn test_crate_binary_dir(binary_name: &str) -> PathBuf {
    let mut target_dir_path = env::current_exe()
        .unwrap()
        .parent()
        .expect("test binary directory")
        .to_path_buf();
    if !&target_dir_path.join(binary_name).is_file() {
        // Sometimes it is ./target/debug/deps/test_* not just ./target/debug/test_*.
        assert!(target_dir_path.pop());
    }
    target_dir_path.canonicalize().unwrap();
    target_dir_path
}

/// Returns the path to the binary being run.
pub fn test_binary_path(binary_name: &str) -> PathBuf {
    test_crate_binary_dir(binary_name).join(binary_name)
}

/// Returns the path to the root of the project (the {crate}/ folder).
fn test_project_dir() -> PathBuf {
    // Directory of the testutils Cargo.toml i.e. {crate}/tests/testutils/
    let mut project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Pop up to tests/ dir.
    assert!(project_dir.pop());
    // Pop up to crate dir.
    assert!(project_dir.pop());
    project_dir
}

/// Returns a new command starting with /path/to/{binary} (add args as needed).
#[must_use]
pub fn test_binary_cmd(binary_name: &str, temp_dir: &Path) -> Command {
    let mut cmd = Command::new(test_binary_path(binary_name));
    // Set temp dir to be inside our test's temp dir.
    cmd.env("TMPDIR", temp_dir.join(format!("{}_temp_dir", binary_name)));
    // Always print colours, even when output is not a tty.
    cmd.env("RUST_LOG_STYLE", "always");
    // Show backtrace on exit, nightly only for now.
    // https://github.com/rust-lang/rust/issues/53487
    cmd.env("RUST_BACKTRACE", "1");
    cmd.args(
        [
            "--log-level=trace",
            "--up-dir",
            temp_dir.join("up-rs").to_str().unwrap(),
            "--color=always",
        ]
        .iter(),
    );
    cmd
}

/// Runs a command and prints out the stdout/stderr nicely.
/// Returns the command output.
#[must_use]
pub fn run_cmd(cmd: &mut Command) -> Output {
    println!("Running command '{:?}'.", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("  status: {}", cmd_output.status);
    if !cmd_output.stdout.is_empty() {
        println!("  stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    }
    if !cmd_output.stderr.is_empty() {
        println!(
            "  stderr:\n\n{}",
            String::from_utf8_lossy(&cmd_output.stderr)
        );
    }
    cmd_output
}

/// Returns the path to the tests/fixtures directory (relative to the crate
/// root).
#[must_use]
pub fn fixture_dir(function_path: &str) -> PathBuf {
    test_project_dir()
        .join("tests/fixtures")
        .join(function_path.replace("::", "/"))
}

/// Returns the path to a temporary directory for your test (OS tempdir + test
/// function path). Cleans the directory if it already exists.
///
/// ```rust
/// let temp_dir = temp_dir(testutils::function_path!()).unwrap();
/// ```
///
/// # Errors
///
/// Fails if any of the underlying file system operations fail.
pub fn temp_dir(binary_name: &str, function_path: &str) -> Result<PathBuf> {
    let os_temp_dir = env::temp_dir().canonicalize()?;
    let mut temp_dir = os_temp_dir.clone();
    temp_dir.push(format!("{}_test_tempdirs", binary_name));
    temp_dir.push(function_path.replace("::", "/"));
    assert!(temp_dir.starts_with(os_temp_dir));
    let remove_dir_result = fs::remove_dir_all(&temp_dir);
    if matches!(&remove_dir_result, Err(e) if e.kind() != ErrorKind::NotFound) {
        remove_dir_result?;
    }
    assert!(!temp_dir.exists());
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

/// Expands to the current function path.
#[macro_export]
macro_rules! function_path {
    () => {{
        // Okay, this is ugly, I get it. However, this is the best we can get on a stable rust.
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // `3` is the length of the `::f`.
        &name[..name.len() - 3]
    }};
}

/// Copy everything in `from_dir` into `to_dir` (including broken links).
///
/// # Errors
///
/// Fails if any of the underlying file system operations fail.
pub fn copy_all(from_dir: &Path, to_dir: &Path) -> Result<()> {
    println!("Copying everything in {:?} to {:?}", from_dir, to_dir);
    assert!(
        from_dir.exists(),
        "Cannot copy from non-existent directory {:?}.",
        from_dir
    );
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let from_path_metadata = from_path.metadata()?;
        let from_path = from_path.path();

        let rel_path = from_path.strip_prefix(&from_dir)?;
        println!("Copying: {:?}", &rel_path);
        let to_path = to_dir.join(rel_path);

        let file_type = from_path_metadata.file_type();
        fs::create_dir_all(to_path.parent().unwrap())?;
        if file_type.is_dir() {
            fs::create_dir(to_path)?;
        } else if file_type.is_symlink() {
            unix::fs::symlink(fs::read_link(&from_path)?, to_path)?;
        } else if file_type.is_file() {
            fs::copy(from_path, to_path)?;
        }
    }
    Ok(())
}

/// Run defaults command with args provided, check it passed, and return the stdout.
pub fn run_defaults(args: &[&str]) -> String {
    let mut cmd = Command::new("defaults");
    cmd.args(args);
    let output = run_cmd(&mut cmd);
    assert!(output.status.success(), "Running {:?} failed.", cmd);
    String::from_utf8_lossy(&output.stdout).to_string()
}