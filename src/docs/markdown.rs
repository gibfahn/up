//! Handle the `up doc markdown` subcommand.

use crate::opts::Opts;

/// Run the `up doc markdown` subcommand.
/// Prints the generated markdown to the stdout.
pub(crate) fn run() {
    clap_markdown::print_help_markdown::<Opts>();
}
