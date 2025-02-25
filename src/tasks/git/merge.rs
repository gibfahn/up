//! Merge updates into a branch.
use super::status::ensure_repo_clean;
use crate::tasks::git::checkout::set_and_checkout_head;
use crate::tasks::git::errors::GitError as E;
use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use git2::Reference;
use git2::Repository;
use std::str;
use tracing::debug;

/// Fast-forward merge, returns "Skipped" if nothing was updated.
/// Returns whether we did any work (`false` means skipped).
pub(super) fn do_ff_merge<'a>(
    repo: &'a Repository,
    branch_name: &str,
    fetch_commit: &git2::AnnotatedCommit<'a>,
) -> Result<bool> {
    // Do merge analysis
    let analysis = repo.merge_analysis(&[fetch_commit])?;

    debug!("Merge analysis: {analysis:?}");

    // Do the merge
    if analysis.0.is_fast_forward() {
        debug!("Doing a fast forward");
        // do a fast forward
        if let Ok(mut r) = repo.find_reference(branch_name) {
            fast_forward(repo, &mut r, fetch_commit)?;
        } else {
            // The branch doesn't exist so just set the reference to the
            // commit directly. Usually this is because you are pulling
            // into an empty repository.
            repo.reference(
                branch_name,
                fetch_commit.id(),
                true,
                &format!("Setting {branch_name} to {}", fetch_commit.id()),
            )?;
            set_and_checkout_head(repo, branch_name, false)?;
        }
        Ok(true)
    } else if analysis.0.is_up_to_date() {
        debug!("Skipping fast-forward merge as already up-to-date.");
        Ok(false)
    } else {
        bail!(E::CannotFastForwardMerge {
            analysis: analysis.0,
            preference: analysis.1
        });
    }
}

/// Do a git fast-forward merge.
fn fast_forward(repo: &Repository, lb: &mut Reference, rc: &git2::AnnotatedCommit) -> Result<()> {
    let name = lb.name().map_or_else(
        || String::from_utf8_lossy(lb.name_bytes()).to_string(),
        std::string::ToString::to_string,
    );
    let msg = format!("Fast-Forward: Setting {name} to id: {}", rc.id());
    debug!("{msg}");
    ensure_repo_clean(repo)?;
    lb.set_target(rc.id(), &msg)?;
    // Force checkout as we already changed what the HEAD branch points to, and we
    // just ensured the repo was clean above that.
    set_and_checkout_head(repo, &name, true)?;
    Ok(())
}
