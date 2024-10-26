//! Generates up CLI completions.
use crate::opts::CompletionsOptions;
use crate::opts::Opts;
use clap::CommandFactory;

/// Run the `up completions` command.
pub(crate) fn run(cmd_opts: CompletionsOptions) {
    let CompletionsOptions { shell } = cmd_opts;
    clap_complete::generate(shell, &mut Opts::command(), "up", &mut std::io::stdout());
}
