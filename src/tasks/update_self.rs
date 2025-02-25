//! The `up self` library, for updating the CLI itself.
use self::UpdateSelfError as E;
use crate::cmd;
use crate::opts::UpdateSelfOptions;
use crate::tasks::ResolveEnv;
use crate::tasks::task::TaskStatus;
use camino::Utf8PathBuf;
use chrono::Utc;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use displaydoc::Display;
use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use thiserror::Error;
use tracing::debug;
use tracing::info;
use tracing::trace;

/// GitHub latest release API endpoint JSON response.
/// <https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28#get-the-latest-release>
#[derive(Debug, Deserialize)]
struct GitHubReleaseJsonResponse {
    /// Name of the git tag the release is for.
    tag_name: String,
}

/// Name user agent after the app, e.g. up/1.2.3.
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
/// Current version of up we're building.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl ResolveEnv for UpdateSelfOptions {}

/// Downloads the latest version of the binary from the specified URL and
/// replaces the current executable path with it.
pub(crate) fn run(opts: &UpdateSelfOptions) -> Result<TaskStatus> {
    let up_path = Utf8PathBuf::try_from(env::current_exe()?)?.canonicalize_utf8()?;

    // If the current binary's location is where it was originally compiled, assume it is a dev
    // build, and thus skip the update.
    if !opts.always_update && up_path.starts_with(env!("CARGO_MANIFEST_DIR")) {
        debug!("Skipping up update, current version '{up_path}' is a dev build.",);
        return Ok(TaskStatus::Skipped);
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    trace!("Self update opts: {opts:?}");
    if opts.url == crate::opts::SELF_UPDATE_URL {
        let latest_github_release = client
            .get(crate::opts::LATEST_RELEASE_URL)
            .send()?
            .error_for_status()?
            .json::<GitHubReleaseJsonResponse>()?;
        trace!("latest_github_release: {latest_github_release:?}");
        let latest_github_release = latest_github_release.tag_name;
        if semver::Version::parse(&latest_github_release)?
            <= semver::Version::parse(CURRENT_VERSION)?
        {
            debug!(
                "Skipping up update, current version '{CURRENT_VERSION}' is not older than latest \
                 GitHub version '{latest_github_release}'",
            );
            return Ok(TaskStatus::Skipped);
        }
        trace!("Updating up from '{CURRENT_VERSION}' to '{latest_github_release}'",);
    }

    let temp_dir = Utf8PathBuf::try_from(env::temp_dir())?;
    let temp_path = &temp_dir.join(format!("up-{}", Utc::now().to_rfc3339()));

    trace!("Downloading url {url} to path {up_path}", url = &opts.url,);

    trace!("Using temporary path: {temp_path}");
    let mut response = reqwest::blocking::get(&opts.url)?.error_for_status()?;

    fs::create_dir_all(&temp_dir).wrap_err_with(|| E::CreateDir { path: temp_dir })?;
    let mut dest = File::create(temp_path).wrap_err_with(|| E::CreateFile {
        path: temp_path.clone(),
    })?;
    io::copy(&mut response, &mut dest).wrap_err(E::Copy {})?;

    let permissions = Permissions::from_mode(0o755);
    fs::set_permissions(temp_path, permissions).wrap_err_with(|| E::SetPermissions {
        path: temp_path.clone(),
    })?;

    let new_version = cmd!(temp_path.as_str(), "--version").read()?;
    let new_version = new_version.trim_start_matches(concat!(env!("CARGO_PKG_NAME"), " "));
    if semver::Version::parse(new_version)? > semver::Version::parse(CURRENT_VERSION)? {
        info!("Updating up from '{CURRENT_VERSION}' to '{new_version}'",);
        fs::rename(temp_path, &up_path).wrap_err_with(|| E::Rename {
            from: temp_path.clone(),
            to: up_path.clone(),
        })?;
        Ok(TaskStatus::Passed)
    } else {
        debug!(
            "Skipping up update, current version '{CURRENT_VERSION}' and new version \
             '{new_version}'",
        );
        Ok(TaskStatus::Skipped)
    }
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum UpdateSelfError {
    /// Failed to create directory `{path}`
    CreateDir {
        /// Dir path we failed to create.
        path: Utf8PathBuf,
    },
    /// Failed to create file `{path}`
    CreateFile {
        /// File path we failed to create.
        path: Utf8PathBuf,
    },
    /// Failed to copy to destination file.
    Copy,
    /// Failed to set permissions for `{path}`.
    SetPermissions {
        /// Path we failed to set permissions for.
        path: Utf8PathBuf,
    },
    /// Failed to rename `{from}` to `{to}`.
    Rename {
        /// Old name (path).
        from: Utf8PathBuf,
        /// Attempted new name (path).
        to: Utf8PathBuf,
    },
}
