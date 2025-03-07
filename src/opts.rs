//! Options passed to `up` commands.
mod paths;
pub(crate) mod start_time;

use crate::opts::paths::TempDir;
use crate::opts::start_time::StartTime;
use camino::Utf8PathBuf;
use clap::Parser;
use clap::ValueEnum;
use clap::ValueHint;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Styles;
use clap_complete::Shell;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::ffi::OsString;

/// The default fallback path inside a fallback repo to look for the up.yaml file in.
pub(crate) const FALLBACK_CONFIG_PATH: &str = "dotfiles/.config/up/up.yaml";
/// URL to use to find the latest version of up.
pub(crate) const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/gibfahn/up/releases/latest";
#[cfg(target_os = "linux")]
/// URL to use to download the latest release of up for Linux.
pub(crate) const SELF_UPDATE_URL: &str =
    "https://github.com/gibfahn/up/releases/latest/download/up-linux";
#[cfg(target_os = "macos")]
/// URL to use to download the latest release of up for macOS.
pub(crate) const SELF_UPDATE_URL: &str =
    "https://github.com/gibfahn/up/releases/latest/download/up-darwin";

/// `up --help` terminal styling.
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().bold())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Blue.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

/// Builds the Args struct from CLI input and from environment variable input.
#[must_use]
pub fn parse() -> Opts {
    Opts::parse()
}

// Don't complain about bare links in my clap document output.
#[allow(clippy::doc_markdown, rustdoc::bare_urls)]
/**
Up is a tool to help you manage your developer machine. `up run` runs the tasks defined in its
config directory. It handles linking configuration files into the right locations, and running
scripts to make sure the tools you need are installed and up to date. It is designed to complete
common bootstrapping tasks without dependencies, so you can bootstrap a new machine by:

❯ curl --create-dirs -Lo ~/bin/up https://github.com/gibfahn/up/releases/latest/download/up-$(uname) && chmod +x ~/bin/up

❯ ~/bin/up run --bootstrap --fallback-url https://github.com/gibfahn/dot

Running `up` without a subcommand runs `up run` with no parameters, which is useful for
post-bootstrapping, when you want to just run all your setup steps again, to make sure
everything is installed and up-to-date. For this reason it's important to make your up tasks
idempotent, so they skip if nothing is needed.

There are also a number of libraries built into up, that can be accessed directly as well as via
up task configs, e.g. `up link` to link dotfiles.

For debugging, run with `RUST_LIB_BACKTRACE=1` to show error/panic traces.
Logs from the latest run are available at `$TMPDIR/up/logs/up_<timestamp>.log` by default.
Parallel tasks are run with rayon, so you can control the number of threads used via `RAYON_NUM_THREADS`, e.g. `RAYON_NUM_THREADS=1 up` to run everything sequentially.
*/
#[derive(Debug, Clone, Parser)]
#[clap(version, styles = STYLES)]
pub struct Opts {
    /// Set the logging level explicitly (options: off, error, warn, info,
    /// debug, trace).
    #[clap(
        long,
        short = 'l',
        default_value = "up=info",
        env = "RUST_LOG",
        alias = "log-level"
    )]
    pub log: String,

    /**
    Temporary directory to use for logs, fifos, and other intermediate artifacts.
    */
    #[clap(long, env = "UP_TEMP_DIR", default_value_t, value_hint = ValueHint::DirPath, alias = "up-dir")]
    pub temp_dir: TempDir,

    /// Set the file logging level explicitly (options: off, error, warn, info,
    /// debug, trace).
    #[clap(long, default_value = "trace", env = "FILE_RUST_LOG")]
    pub file_log_level: String,

    /// Whether to color terminal output.
    #[clap(long, default_value = "auto", ignore_case = true, value_enum)]
    pub color: Color,

    /// Path to the up.yaml file for up.
    #[clap(long, short = 'c', default_value = "$XDG_CONFIG_HOME/up/up.yaml", value_hint = ValueHint::FilePath)]
    pub(crate) config: String,

    /**
    The timestamp where we started this action.

    Hidden as users shouldn't normally be setting this.
    */
    #[clap(long, hide(true), default_value_t)]
    pub start_time: StartTime,

    /// Clap subcommand to run.
    #[clap(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

/// Settings for colouring output.
#[derive(Debug, ValueEnum, Clone)]
pub enum Color {
    /// Auto: Colour on if stderr isatty, else off.
    Auto,
    /// Always: Always enable colours.
    Always,
    /// Never: Never enable colours.
    Never,
}

/// Optional subcommand (e.g. the "link" in "up link").
#[derive(Debug, Clone, Parser)]
pub(crate) enum SubCommand {
    /**
    Run the update tasks.

    If you don't provide a subcommand this is the default action.
    If you want to pass Run args you will need to specify the subcommand.
    */
    Run(RunOptions),
    /// Symlink your dotfiles from a git repo to your home directory.
    Link(LinkOptions),
    /// Clone or update a repo at a path.
    Git(GitOptions),
    /// Set macOS defaults in plist files.
    Defaults(DefaultsOptions),
    /// Generate up config from current system state.
    Generate(GenerateOptions),
    /// Update the up CLI itself.
    Self_(UpdateSelfOptions),
    /// Generate various docs or completions for up.
    Doc(DocOptions),

    /// List available tasks.
    List(RunOptions),
    /**
    Runs a command in a fake tty.
    */
    Faketty(FakettyOptions),
}

/// Options passed to `up run`.
#[derive(Debug, Clone, Parser, Default)]
pub(crate) struct RunOptions {
    /// Run the bootstrap list of tasks in series first, then run the rest in
    /// parallel. Designed for first-time setup.
    #[clap(short, long)]
    pub(crate) bootstrap: bool,
    /// Keep going even if a bootstrap task fails.
    #[clap(short, long)]
    pub(crate) keep_going: bool,
    /// Fallback git repo URL to download to get the config.
    #[clap(short = 'f', long, value_hint = ValueHint::Url)]
    pub(crate) fallback_url: Option<String>,
    /// Fallback path inside the git repo to get the config.
    /// The default path assumes your `fallback_url` points to a dotfiles repo
    /// that is linked into ~.
    #[clap(
        short = 'p',
        long,
        default_value = FALLBACK_CONFIG_PATH,
        value_hint = ValueHint::FilePath
    )]
    pub(crate) fallback_path: Utf8PathBuf,
    /**
    Optionally pass one or more tasks to run. The default is to run all
    tasks. This option can be provided multiple times, or use a comma-separated list of values.

    EXAMPLES:

    ❯ up run --tasks=rust,apt --tasks=otherslowtask
    */
    #[clap(short = 't', long, value_delimiter = ',')]
    pub(crate) tasks: Option<Vec<String>>,

    /**
    Tasks stdout/stderr inherit from up's stdout/stderr.

    By default this is true if only one task is executed, and false otherwise.
    Piping multiple commands to the stdout/stderr of the process will cause task output to be interleaved, which is very confusing when many tasks are run.
    */
    #[clap(long)]
    pub(crate) console: Option<bool>,

    /**
    Optionally pass one or more tasks to exclude. The default is to exclude no
    tasks. Excluded tasks are not run even if specified in `--tasks` (excluding takes
    priority). This option can be provided multiple times. Tasks specified do not have to exist.

    EXAMPLES:

    ❯ up run --exclude-tasks=brew,slowtask --exclude-tasks=otherslowtask
    */
    #[clap(long, value_delimiter = ',')]
    pub(crate) exclude_tasks: Option<Vec<String>>,
}

/// Options passed to `up link`.
#[derive(Debug, Clone, Parser, Default, Serialize, Deserialize)]
pub(crate) struct LinkOptions {
    /// Path where your dotfiles are kept (hopefully in source control).
    #[clap(short = 'f', long = "from", default_value = "~/code/dotfiles", value_hint = ValueHint::DirPath)]
    pub(crate) from_dir: String,
    /// Path to link them to.
    #[clap(short = 't', long = "to", default_value = "~", value_hint = ValueHint::DirPath)]
    pub(crate) to_dir: String,
}

/// Options passed to `up git`.
#[derive(Debug, Clone, Default, Parser)]
pub struct GitOptions {
    /// URL of git repo to download.
    #[clap(long, value_hint = ValueHint::Url)]
    pub git_url: String,
    /// Path to download git repo to.
    #[clap(long, value_hint = ValueHint::DirPath)]
    pub git_path: Utf8PathBuf,
    /// Remote to set/update.
    #[clap(long, default_value = crate::tasks::git::DEFAULT_REMOTE_NAME)]
    pub remote: String,
    /// Branch to checkout when cloning/updating. Defaults to default branch for
    /// cloning, and current branch for updating.
    #[clap(long)]
    pub branch: Option<String>,
    /// Prune merged PR branches. Deletes local branches where the push branch
    /// has been merged into the upstream branch, and the push branch has now
    /// been deleted.
    #[clap(long)]
    pub prune: bool,
}

/// Options passed to `up generate`.
#[derive(Debug, Clone, Parser)]
pub(crate) struct GenerateOptions {
    /// Lib to generate.
    #[clap(subcommand)]
    pub(crate) lib: Option<GenerateLib>,
}

/// Options passed to `up schema`.
#[derive(Debug, Clone, Parser)]
pub(crate) struct SchemaOptions {
    /// Lib to generate. Defaults to writing to stdout.
    pub(crate) path: Option<Utf8PathBuf>,
}

/// Arguments for the `up generate manpages` subcommand.
#[derive(Debug, Clone, Parser)]
pub struct ManpagesOptions {
    /// Directory into which to write the generated manpages.
    #[clap(long, value_hint = ValueHint::DirPath)]
    pub(crate) output_dir: Utf8PathBuf,
}

/// Options passed to `up self`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub(crate) struct UpdateSelfOptions {
    /// URL to download update from.
    #[clap(long, default_value = SELF_UPDATE_URL, value_hint = ValueHint::Url)]
    pub(crate) url: String,
    /// Set to update self even if it seems to be a development install.
    /// Assumes a dev install when the realpath of the current binary is in a
    /// subdirectory of the cargo root path that the binary was originally built in.
    #[clap(long)]
    pub(crate) always_update: bool,
}

/// Options passed to `up doc`.
#[derive(Debug, Clone, Parser)]
pub(crate) struct DocOptions {
    /// Type of documentation to generate.
    #[clap(subcommand)]
    pub(crate) subcmd: DocSubcommand,
}

/// Subcommands supported by `up doc`.
#[derive(Debug, Clone, Parser)]
pub(crate) enum DocSubcommand {
    /**
    Generate shell completions to stdout.

    Completions are printed to stdout. They are designed to be written to a file.

    EXAMPLES:

    ❯ up doc completions zsh | sudo tee >/dev/null /usr/local/share/zsh/site-functions/_up
    */
    Completions(CompletionsOptions),
    /**
    Write the up task yaml schema.

    EXAMPLES:

    ❯ up doc schema --path /path/to/up-task-schema.json
    */
    Schema(SchemaOptions),
    /**
    Generate man pages for up and its subcommands.

    Manpages are generated into the output directory specified by `--output-dir`.

    EXAMPLES:

    ❯ up generate manpages --output-dir /usr/local/share/man/man1/
    */
    #[clap(visible_alias = "man")]
    Manpages(ManpagesOptions),
    /**
    Print a markdown file with documentation for up and its subcommands.

    Markdown file contents are written to the stdout.
    */
    Markdown,
}

/// Options passed to `up completions`.
#[derive(Debug, Clone, Parser)]
pub(crate) struct CompletionsOptions {
    /// Shell for which to generate completions.
    #[clap(value_enum)]
    pub(crate) shell: Shell,
}

impl Default for UpdateSelfOptions {
    fn default() -> Self {
        Self {
            url: SELF_UPDATE_URL.to_owned(),
            always_update: false,
        }
    }
}

/// Subcommands supported by `up generate`.
#[derive(Debug, Clone, Parser)]
pub(crate) enum GenerateLib {
    /// Generate a git repo.
    Git(GenerateGitConfig),
    /// Generate macOS defaults commands (not yet implemented).
    Defaults(GenerateDefaultsConfig),
}

/// Options passed to `up generate git`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct GenerateGitConfig {
    /// Path to yaml file to update.
    #[clap(long, value_hint = ValueHint::FilePath)]
    pub(crate) path: Utf8PathBuf,
    /// Paths to search within.
    #[clap(long, default_value = "~", value_hint = ValueHint::DirPath)]
    pub(crate) search_paths: Vec<Utf8PathBuf>,
    /// Exclude paths containing this value. e.g. '/tmp/' to exclude anything in
    /// a tmp dir.
    #[clap(long)]
    pub(crate) excludes: Option<Vec<String>>,
    /// Prune all repos for branches that have already been merged and deleted
    /// upstream.
    #[clap(long)]
    pub(crate) prune: bool,
    /// Order to save remotes, other remotes will be included after those listed here.
    #[clap(long)]
    pub(crate) remote_order: Vec<String>,
}

/// Options passed to `up generate defaults`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct GenerateDefaultsConfig {
    /// Path to yaml file to update.
    #[clap(long, value_hint = ValueHint::FilePath)]
    pub(crate) path: Utf8PathBuf,
}

/// Options passed to `up defaults`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct DefaultsOptions {
    /// Read from the current host, same as `defaults -currentHost`.
    #[clap(long = "currentHost")]
    pub(crate) current_host: bool,
    /// Defaults action to take.
    #[clap(subcommand)]
    pub(crate) subcommand: DefaultsSubcommand,
}

/// Subcommands supported by `up defaults`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub enum DefaultsSubcommand {
    /// Read a defaults option and print it to the stdout as yaml.
    Read(DefaultsReadOptions),
    /**
    Write a yaml-encoded value to a defaults plist file.
    A domain, key, and value must be provided (you can optionally use `-g` to specify the global domain).
    */
    Write(DefaultsWriteOptions),
}

/// Options passed to `up defaults read`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct DefaultsReadOptions {
    /// Read from the global domain. If you set this, do not also pass a domain argument.
    #[clap(short = 'g', long = "globalDomain")]
    pub(crate) global_domain: bool,
    /**
    Defaults domain to print. Use `-` to read from stdin.
    */
    pub(crate) domain: Option<String>,
    /// Defaults key to print.
    pub(crate) key: Option<String>,
}

/// Options passed to `up defaults write`.
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct DefaultsWriteOptions {
    /// Read from the global domain. If you set this, do not also pass a domain argument.
    #[clap(short = 'g', long = "globalDomain")]
    pub(crate) global_domain: bool,
    /// Defaults domain to write to.
    pub(crate) domain: String,
    /// Defaults key to write to.
    pub(crate) key: String,
    /**
    Value to write (as a yaml string).

    If you want to append to an existing array or dictionary, use `...` as an array value, or `...:...` as a dictionary entry, to represent the existing items in the array.
    If there are duplicates, the first entry will be preserved.

    So if the array contained `["a", "foo", "b", "bar", "c"]`, and you write `["foo", "...", "bar", "baz"]`, you would end up with `["foo", "a", "b", "bar", "c", "baz"]`

    Similarly if the dict contained `{"a": 1, "foo": 2, "b": 3, "bar": 4, "c": 5}`, and you write `{"foo": 6 "...":"...", "bar": 7, "baz": 8}`, you would end up with `{"a": 1, "foo": 6, "b": 3, "bar": 4, "c": 5, "baz": 8}`
    */
    pub(crate) value: Option<String>,
}

/// Options supported by the `up faketty` subcommand.
#[derive(Debug, Parser, Default, Clone)]
pub struct FakettyOptions {
    /// The program to run.
    #[clap(
        num_args(1..),
        value_parser(clap::builder::OsStringValueParser::new()),
        trailing_var_arg(true),
    )]
    pub(crate) program: Vec<OsString>,
}
