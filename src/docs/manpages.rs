//! Handle `up doc manpages` subcommand.

use crate::opts::ManpagesOptions;
use crate::opts::Opts;
use crate::utils::files;
use camino::Utf8Path;
use clap::CommandFactory;
use clap_mangen::Man;
use color_eyre::Result;
use tracing::info;

/// Write man pages for the command and each subcommand to a directory.
pub(crate) fn run(manpages_opts: ManpagesOptions) -> Result<()> {
    let ManpagesOptions { output_dir } = manpages_opts;

    let cmd = Opts::command();
    let name = cmd.get_name();

    files::create_dir_all(&output_dir)?;

    write_man_page(name.to_owned(), &output_dir, &cmd)?;

    for subcommand in cmd.get_subcommands() {
        let subcommand_name = subcommand.get_name();
        let subcommand_name = format!("{name}-{subcommand_name}");
        write_man_page(subcommand_name.clone(), &output_dir, subcommand)?;
        for subsubcommand in subcommand.get_subcommands() {
            let subsubcommand_name = subsubcommand.get_name();
            let subsubcommand_name = format!("{subcommand_name}-{subsubcommand_name}");
            write_man_page(subsubcommand_name, &output_dir, subsubcommand)?;
        }
    }

    Ok(())
}

/// Write a specific man page to a directory.
fn write_man_page(name: String, output_dir: &Utf8Path, cmd: &clap::Command) -> Result<()> {
    let output_file = output_dir.join(format!("{name}.1"));
    info!("Writing man page for {name} to {output_file}");
    let man = Man::new(cmd.clone().name(name));
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;
    files::write(&output_file, buffer)?;
    Ok(())
}
