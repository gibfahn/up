//! Handle `liv generate` subcommand.

pub mod completions;
mod manpages;
mod markdown;
pub(crate) mod schema;
use crate::opts::DocOptions;
use crate::opts::DocSubcommand;
use color_eyre::Result;

/// Delegate to the correct `up doc` subcommand's run function.
pub(crate) fn run(cmd_opts: DocOptions) -> Result<()> {
    let DocOptions { subcmd } = cmd_opts;

    match subcmd {
        DocSubcommand::Completions(subcmd_opts) => completions::run(subcmd_opts),
        DocSubcommand::Schema(subcmd_opts) => schema::run(subcmd_opts)?,
        DocSubcommand::Manpages(subcmd_opts) => manpages::run(subcmd_opts)?,
        DocSubcommand::Markdown => markdown::run(),
    }
    Ok(())
}
