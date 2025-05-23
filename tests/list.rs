use assert_cmd::cargo::cargo_bin;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::Result;
use itertools::Itertools;
use std::collections::HashMap;
use testutils::AssertCmdExt;
use testutils::ensure_eq;
use testutils::ensure_utils;

/// Run a full up with a bunch of configuration and check things work.
#[test]
fn test_up_list_passing() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    testutils::copy_all(
        &testutils::fixtures_subdir(testutils::function_path!())?,
        &temp_dir,
    )
    .unwrap();

    let mut envs = HashMap::new();
    // Used in link task.
    envs.insert("link_from_dir", temp_dir.join("link_dir/dotfile_dir"));
    envs.insert("link_to_dir", temp_dir.join("link_dir/home_dir"));
    envs.insert(
        "up_binary_path",
        Utf8PathBuf::try_from(cargo_bin("up")).unwrap(),
    );

    ensure_eq!(
        vec!["link", "run_self_cmd", "skip_self_cmd"],
        check_list(&[], &envs, &temp_dir)?
            .split_whitespace()
            .sorted()
            .collect_vec(),
    );

    ensure_eq!(
        vec!["link", "skip_self_cmd"],
        check_list(
            &["--tasks", "link", "--tasks", "skip_self_cmd"],
            &envs,
            &temp_dir,
        )?
        .split_whitespace()
        .sorted()
        .collect_vec(),
    );

    Ok(())
}

fn check_list(
    args: &[&str],
    envs: &HashMap<&str, Utf8PathBuf>,
    temp_dir: &Utf8Path,
) -> Result<String> {
    let mut cmd = testutils::crate_binary_cmd("up", temp_dir)?;
    cmd.envs(envs);
    cmd.args([
        "--config",
        temp_dir.join("up_config_dir/up.yaml").as_str(),
        "list",
    ]);
    cmd.args(args);

    let cmd_assert = cmd.assert().eprint_stdout_stderr().try_success()?;

    ensure_utils::nothing_at(&temp_dir.join("link_dir/home_dir/file_to_link"))?;

    Ok(String::from_utf8_lossy(&cmd_assert.get_output().stdout).to_string())
}
