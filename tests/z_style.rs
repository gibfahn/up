//! This module is called "`z_style`" rather than "style" so that it runs last
//! (for people who aren't aware of the `--no-fail-fast` flag for `cargo test`
//! or would rather not type it).

use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::Result;
use color_eyre::eyre::Context;
use color_eyre::eyre::ensure;
use color_eyre::eyre::eyre;
use std::env;
use std::fs;
use std::process::Command;
use std::process::Output;
/// Fail if rustdoc (cargo doc) hasn't been run on the public items in this crate.
#[test]
fn test_rustdoc_public() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    let check_output = cargo_cmd(&current_dir, CargoCmdType::RustdocCheckPublic)?;

    ensure!(
        check_output.status.success(),
        "Private documentation building failed. Please run the above command and fix any issues."
    );
    Ok(())
}

/// Fail if rustdoc (cargo doc) hasn't been run on the private items in this crate.
#[test]
fn test_rustdoc_private() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    let check_output = cargo_cmd(&current_dir, CargoCmdType::RustdocCheckPrivate)?;

    ensure!(
        check_output.status.success(),
        "Private documentation building failed. Please run the above command and fix any issues."
    );
    Ok(())
}

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
fn test_rustfmt() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    let check_output = if use_stable() {
        cargo_cmd(&current_dir, CargoCmdType::RustfmtStableCheck)?
    } else {
        let check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtCheck)?;

        if !check_output.status.success() {
            // Fix the formatting.
            cargo_cmd(&current_dir, CargoCmdType::RustfmtFix)?;
        }
        check_output
    };

    ensure!(
        check_output.status.success(),
        "Rustfmt needs to be run, we ran 'cargo fmt' to fix, please commit the changes."
    );
    Ok(())
}

/// Fail if rustfmt (cargo fmt) hasn't been run on testutils.
#[test]
fn test_testutils_rustfmt() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?.join("tests/testutils");
    let check_output = if use_stable() {
        cargo_cmd(&current_dir, CargoCmdType::RustfmtStableCheck)?
    } else {
        let check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtCheck)?;

        if !check_output.status.success() {
            // Fix the formatting.
            cargo_cmd(&current_dir, CargoCmdType::RustfmtFix)?;
        }
        check_output
    };

    ensure!(
        check_output.status.success(),
        "Rustfmt needs to be run. We ran 'cargo fmt' to fix, please commit the changes."
    );
    Ok(())
}

/// Fail if cargo clippy hasn't been run.
#[test]
fn test_clippy() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    let clippy_output = if use_stable() {
        cargo_cmd(&current_dir, CargoCmdType::ClippyStableCheck)?
    } else {
        let clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck)?;

        if !clippy_output.status.success() {
            // Fix the clippy errors if possible.
            cargo_cmd(&current_dir, CargoCmdType::ClippyFix)?;
        }
        clippy_output
    };

    ensure!(
        clippy_output.status.success(),
        "Clippy needs to be run. Please run the above command and fix any issues."
    );
    Ok(())
}

/// Fail if cargo clippy hasn't been run on testutils.
#[test]
fn test_testutils_clippy() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?.join("tests/testutils");
    let clippy_output = if use_stable() {
        cargo_cmd(&current_dir, CargoCmdType::ClippyStableCheck)?
    } else {
        let clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck)?;

        if !clippy_output.status.success() {
            // Fix the clippy errors if possible.
            cargo_cmd(&current_dir, CargoCmdType::ClippyFix)?;
        }
        clippy_output
    };

    ensure!(
        clippy_output.status.success(),
        "Clippy needs to be run. Please run the above command and fix any issues."
    );
    Ok(())
}

#[ignore = "unhelpful when running tests in a loop while developing"]
#[test]
fn test_no_todo() -> Result<()> {
    const DISALLOWED_STRINGS: [&str; 4] = ["XXX(", "XXX:", "todo!", "dbg!"];
    let mut files_with_todos = Vec::new();
    for file in ignore::WalkBuilder::new("./")
        // Check hidden files too.
        .hidden(false)
        .build()
    {
        let file = file?;

        // Only scan files, not dirs or symlinks.
        if file
            .file_type()
            .is_none_or(|file_type| !file_type.is_file())
            || file.path().ends_with(file!())
        {
            continue;
        }
        // Find anything containing a todo.
        let path = Utf8PathBuf::try_from(file.path().to_path_buf())?;
        let text = fs::read_to_string(&path)
            .wrap_err_with(|| eyre!("Failed to read the contents of file {path}"))?;

        for disallowed_string in DISALLOWED_STRINGS {
            if text.contains(disallowed_string) {
                eprintln!("ERROR: {path} contains disallowed string '{disallowed_string}'");
                files_with_todos.push(path.clone());
            }
        }
    }

    ensure!(
        files_with_todos.is_empty(),
        "\nFiles with blocking todos should not be committed to the main branch, use TODO: \
         instead\n{files_with_todos:#?}\n",
    );
    Ok(())
}

/// Check whether we can use nightly rust or whether we need to use stable rust.
fn use_stable() -> bool {
    // We assume in CI and in Linux you're not actually developing, just running a test, and
    // thus you probably don't have nightly Rust installed.
    std::env::var("CI").is_ok() || cfg!(target_os = "linux")
}

/// Whether to check for the formatter having been run, or to actually fix any
/// formatting issues.
#[derive(Debug, PartialEq, Eq)]
enum CargoCmdType {
    /// Run rustdoc on stable.
    RustdocCheckPublic,
    /// Run rustdoc on stable.
    RustdocCheckPrivate,
    /// Check the format in CI.
    RustfmtStableCheck,
    /// Check the format.
    RustfmtCheck,
    /// Fix any formatting issues.
    RustfmtFix,
    /// Run clippy on stable.
    ClippyStableCheck,
    /// Run clippy on nightly.
    ClippyCheck,
    /// Fix clippy errors if possible.
    ClippyFix,
}

fn cargo_cmd(current_dir: &Utf8Path, fmt: CargoCmdType) -> Result<Output> {
    let mut cmd = Command::new("cargo");

    match fmt {
        // Use stable cargo.
        CargoCmdType::RustdocCheckPublic
        | CargoCmdType::RustdocCheckPrivate
        | CargoCmdType::RustfmtStableCheck
        | CargoCmdType::ClippyStableCheck => {}
        // Use nightly cargo.
        CargoCmdType::RustfmtCheck
        | CargoCmdType::RustfmtFix
        | CargoCmdType::ClippyCheck
        | CargoCmdType::ClippyFix => {
            cmd.arg("+nightly");
            // Nightly cargo shouldn't be run with stable cargo.
            cmd.env_remove("CARGO");
        }
    }
    cmd.args(match fmt {
        CargoCmdType::RustdocCheckPublic => {
            ["doc", "--no-deps", "--keep-going", "--color=always"].iter()
        }
        CargoCmdType::RustdocCheckPrivate => [
            "doc",
            "--no-deps",
            "--keep-going",
            "--color=always",
            "--document-private-items",
        ]
        .iter(),
        CargoCmdType::RustfmtStableCheck => ["fmt", "--", "--check", "--color=always"].iter(),
        CargoCmdType::RustfmtCheck => ["fmt", "--", "--check", "--color=always"].iter(),
        CargoCmdType::RustfmtFix => ["fmt", "--", "--color=always"].iter(),
        CargoCmdType::ClippyStableCheck => [
            "clippy",
            #[cfg(not(debug_assertions))]
            "--release",
            "--color=always",
            "--",
            "--deny=warnings",
            "--allow=unknown_lints",
        ]
        .iter(),
        CargoCmdType::ClippyCheck => [
            "clippy",
            #[cfg(not(debug_assertions))]
            "--release",
            "--color=always",
            "--",
            "--deny=warnings",
        ]
        .iter(),
        CargoCmdType::ClippyFix => [
            "clippy",
            #[cfg(not(debug_assertions))]
            "--release",
            "--color=always",
            "--fix",
            "--allow-staged",
        ]
        .iter(),
    });

    // Only used by `cargo doc`, but should be fine to have set everywhere.
    cmd.env("RUSTDOCFLAGS", "--deny=warnings");
    cmd.current_dir(current_dir);
    eprintln!("Running '{cmd:?}'");
    let cmd_output = cmd.output()?;
    eprintln!("  status: {}", cmd_output.status);
    if !cmd_output.stdout.is_empty() {
        eprintln!("  stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    }
    if !cmd_output.stderr.is_empty() {
        eprintln!(
            "  stderr:\n<<<\n{}\n>>>",
            String::from_utf8_lossy(&cmd_output.stderr)
        );
    }
    Ok(cmd_output)
}
