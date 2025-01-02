stable: Pulling from clux/muslrust
4be1db8bbbeb: Pulling fs layer
cfaa55eda8f7: Pulling fs layer
442a5f1d1d46: Pulling fs layer
4c6c7026fc5b: Pulling fs layer
b5c939e59651: Pulling fs layer
0a07509a0746: Pulling fs layer
b6e1db31a9c4: Pulling fs layer
dedd2c7d8471: Pulling fs layer
1647917aac9e: Pulling fs layer
032acbb171e9: Pulling fs layer
d4d376ebf551: Pulling fs layer
383dd67a1d80: Pulling fs layer
aca21b397f39: Pulling fs layer
4c6c7026fc5b: Waiting
b5c939e59651: Waiting
0a07509a0746: Waiting
b6e1db31a9c4: Waiting
dedd2c7d8471: Waiting
1647917aac9e: Waiting
032acbb171e9: Waiting
aca21b397f39: Waiting
d4d376ebf551: Waiting
383dd67a1d80: Waiting
4be1db8bbbeb: Verifying Checksum
4be1db8bbbeb: Download complete
4be1db8bbbeb: Pull complete
4c6c7026fc5b: Verifying Checksum
4c6c7026fc5b: Download complete
b5c939e59651: Verifying Checksum
b5c939e59651: Download complete
0a07509a0746: Verifying Checksum
0a07509a0746: Download complete
cfaa55eda8f7: Download complete
b6e1db31a9c4: Download complete
dedd2c7d8471: Verifying Checksum
dedd2c7d8471: Download complete
1647917aac9e: Download complete
032acbb171e9: Download complete
383dd67a1d80: Verifying Checksum
383dd67a1d80: Download complete
cfaa55eda8f7: Pull complete
aca21b397f39: Verifying Checksum
aca21b397f39: Download complete
d4d376ebf551: Verifying Checksum
d4d376ebf551: Download complete
442a5f1d1d46: Verifying Checksum
442a5f1d1d46: Download complete
442a5f1d1d46: Pull complete
4c6c7026fc5b: Pull complete
b5c939e59651: Pull complete
0a07509a0746: Pull complete
b6e1db31a9c4: Pull complete
dedd2c7d8471: Pull complete
1647917aac9e: Pull complete
032acbb171e9: Pull complete
d4d376ebf551: Pull complete
383dd67a1d80: Pull complete
aca21b397f39: Pull complete
Digest: sha256:ad8f5707da474b774dc41b9073e0380918707d7408ad00b2261f3bd397d076d7
Status: Downloaded newer image for clux/muslrust:stable
docker.io/clux/muslrust:stable
# Command-Line Help for `up`

This document contains the help content for the `up` command-line program.

**Command Overview:**

* [`up`↴](#up)
* [`up run`↴](#up-run)
* [`up link`↴](#up-link)
* [`up git`↴](#up-git)
* [`up defaults`↴](#up-defaults)
* [`up defaults read`↴](#up-defaults-read)
* [`up defaults write`↴](#up-defaults-write)
* [`up generate`↴](#up-generate)
* [`up generate git`↴](#up-generate-git)
* [`up generate defaults`↴](#up-generate-defaults)
* [`up self`↴](#up-self)
* [`up doc`↴](#up-doc)
* [`up doc completions`↴](#up-doc-completions)
* [`up doc schema`↴](#up-doc-schema)
* [`up doc manpages`↴](#up-doc-manpages)
* [`up doc markdown`↴](#up-doc-markdown)
* [`up list`↴](#up-list)

## `up`

Up is a tool to help you manage your developer machine. `up run` runs the tasks defined in its config directory. It handles linking configuration files into the right locations, and running scripts to make sure the tools you need are installed and up to date. It is designed to complete common bootstrapping tasks without dependencies, so you can bootstrap a new machine by:

❯ curl --create-dirs -Lo ~/bin/up https://github.com/gibfahn/up/releases/latest/download/up-$(uname) && chmod +x ~/bin/up

❯ ~/bin/up run --bootstrap --fallback-url https://github.com/gibfahn/dot

Running `up` without a subcommand runs `up run` with no parameters, which is useful for post-bootstrapping, when you want to just run all your setup steps again, to make sure everything is installed and up-to-date. For this reason it's important to make your up tasks idempotent, so they skip if nothing is needed.

There are also a number of libraries built into up, that can be accessed directly as well as via up task configs, e.g. `up link` to link dotfiles.

For debugging, run with `RUST_LIB_BACKTRACE=1` to show error/panic traces. Logs from the latest run are available at `$TMPDIR/up/logs/up_<timestamp>.log` by default. Parallel tasks are run with rayon, so you can control the number of threads used via `RAYON_NUM_THREADS`, e.g. `RAYON_NUM_THREADS=1 up` to run everything sequentially.

**Usage:** `up [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `run` — Run the update tasks
* `link` — Symlink your dotfiles from a git repo to your home directory
* `git` — Clone or update a repo at a path
* `defaults` — Set macOS defaults in plist files
* `generate` — Generate up config from current system state
* `self` — Update the up CLI itself
* `doc` — Generate various docs or completions for up
* `list` — List available tasks

###### **Options:**

* `-l`, `--log <LOG>` — Set the logging level explicitly (options: off, error, warn, info, debug, trace)

  Default value: `up=info`
* `--temp-dir <TEMP_DIR>` — Temporary directory to use for logs, fifos, and other intermediate artifacts.

  Default value: `/tmp/up`
* `--file-log-level <FILE_LOG_LEVEL>` — Set the file logging level explicitly (options: off, error, warn, info, debug, trace)

  Default value: `trace`
* `--color <COLOR>` — Whether to color terminal output

  Default value: `auto`

  Possible values:
  - `auto`:
    Auto: Colour on if stderr isatty, else off
  - `always`:
    Always: Always enable colours
  - `never`:
    Never: Never enable colours

* `-c`, `--config <CONFIG>` — Path to the up.yaml file for up

  Default value: `$XDG_CONFIG_HOME/up/up.yaml`



## `up run`

Run the update tasks.

If you don't provide a subcommand this is the default action. If you want to pass Run args you will need to specify the subcommand.

**Usage:** `up run [OPTIONS]`

###### **Options:**

* `-b`, `--bootstrap` — Run the bootstrap list of tasks in series first, then run the rest in parallel. Designed for first-time setup
* `-k`, `--keep-going` — Keep going even if a bootstrap task fails
* `-f`, `--fallback-url <FALLBACK_URL>` — Fallback git repo URL to download to get the config
* `-p`, `--fallback-path <FALLBACK_PATH>` — Fallback path inside the git repo to get the config. The default path assumes your `fallback_url` points to a dotfiles repo that is linked into ~

  Default value: `dotfiles/.config/up/up.yaml`
* `-t`, `--tasks <TASKS>` — Optionally pass one or more tasks to run. The default is to run all tasks. This option can be provided multiple times, or use a comma-separated list of values.

   EXAMPLES:

   ❯ up run --tasks=rust,apt --tasks=otherslowtask
* `--console <CONSOLE>` — Tasks stdout/stderr inherit from up's stdout/stderr.

   By default this is true if only one task is executed, and false otherwise. Piping multiple commands to the stdout/stderr of the process will cause task output to be interleaved, which is very confusing when many tasks are run.

  Possible values: `true`, `false`

* `--exclude-tasks <EXCLUDE_TASKS>` — Optionally pass one or more tasks to exclude. The default is to exclude no tasks. Excluded tasks are not run even if specified in `--tasks` (excluding takes priority). This option can be provided multiple times. Tasks specified do not have to exist.

   EXAMPLES:

   ❯ up run --exclude-tasks=brew,slowtask --exclude-tasks=otherslowtask



## `up link`

Symlink your dotfiles from a git repo to your home directory

**Usage:** `up link [OPTIONS]`

###### **Options:**

* `-f`, `--from <FROM_DIR>` — Path where your dotfiles are kept (hopefully in source control)

  Default value: `~/code/dotfiles`
* `-t`, `--to <TO_DIR>` — Path to link them to

  Default value: `~`



## `up git`

Clone or update a repo at a path

**Usage:** `up git [OPTIONS] --git-url <GIT_URL> --git-path <GIT_PATH>`

###### **Options:**

* `--git-url <GIT_URL>` — URL of git repo to download
* `--git-path <GIT_PATH>` — Path to download git repo to
* `--remote <REMOTE>` — Remote to set/update

  Default value: `origin`
* `--branch <BRANCH>` — Branch to checkout when cloning/updating. Defaults to default branch for cloning, and current branch for updating
* `--prune` — Prune merged PR branches. Deletes local branches where the push branch has been merged into the upstream branch, and the push branch has now been deleted



## `up defaults`

Set macOS defaults in plist files

**Usage:** `up defaults [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `read` — Read a defaults option and print it to the stdout as yaml
* `write` — Write a yaml-encoded value to a defaults plist file. A domain, key, and value must be provided (you can optionally use `-g` to specify the global domain)

###### **Options:**

* `--currentHost` — Read from the current host, same as `defaults -currentHost`



## `up defaults read`

Read a defaults option and print it to the stdout as yaml

**Usage:** `up defaults read [OPTIONS] [DOMAIN] [KEY]`

###### **Arguments:**

* `<DOMAIN>` — Defaults domain to print. Use `-` to read from stdin.
* `<KEY>` — Defaults key to print

###### **Options:**

* `-g`, `--globalDomain` — Read from the global domain. If you set this, do not also pass a domain argument



## `up defaults write`

Write a yaml-encoded value to a defaults plist file. A domain, key, and value must be provided (you can optionally use `-g` to specify the global domain).

**Usage:** `up defaults write [OPTIONS] <DOMAIN> <KEY> [VALUE]`

###### **Arguments:**

* `<DOMAIN>` — Defaults domain to write to
* `<KEY>` — Defaults key to write to
* `<VALUE>` — Value to write (as a yaml string).

   If you want to append to an existing array or dictionary, use `...` as an array value, or `...:...` as a dictionary entry, to represent the existing items in the array. If there are duplicates, the first entry will be preserved.

   So if the array contained `["a", "foo", "b", "bar", "c"]`, and you write `["foo", "...", "bar", "baz"]`, you would end up with `["foo", "a", "b", "bar", "c", "baz"]`

   Similarly if the dict contained `{"a": 1, "foo": 2, "b": 3, "bar": 4, "c": 5}`, and you write `{"foo": 6 "...":"...", "bar": 7, "baz": 8}`, you would end up with `{"a": 1, "foo": 6, "b": 3, "bar": 4, "c": 5, "baz": 8}`

###### **Options:**

* `-g`, `--globalDomain` — Read from the global domain. If you set this, do not also pass a domain argument



## `up generate`

Generate up config from current system state

**Usage:** `up generate [COMMAND]`

###### **Subcommands:**

* `git` — Generate a git repo
* `defaults` — Generate macOS defaults commands (not yet implemented)



## `up generate git`

Generate a git repo

**Usage:** `up generate git [OPTIONS] --path <PATH>`

###### **Options:**

* `--path <PATH>` — Path to yaml file to update
* `--search-paths <SEARCH_PATHS>` — Paths to search within

  Default value: `~`
* `--excludes <EXCLUDES>` — Exclude paths containing this value. e.g. '/tmp/' to exclude anything in a tmp dir
* `--prune` — Prune all repos for branches that have already been merged and deleted upstream
* `--remote-order <REMOTE_ORDER>` — Order to save remotes, other remotes will be included after those listed here



## `up generate defaults`

Generate macOS defaults commands (not yet implemented)

**Usage:** `up generate defaults --path <PATH>`

###### **Options:**

* `--path <PATH>` — Path to yaml file to update



## `up self`

Update the up CLI itself

**Usage:** `up self [OPTIONS]`

###### **Options:**

* `--url <URL>` — URL to download update from

  Default value: `https://github.com/gibfahn/up/releases/latest/download/up-linux`
* `--always-update` — Set to update self even if it seems to be a development install. Assumes a dev install when the realpath of the current binary is in a subdirectory of the cargo root path that the binary was originally built in



## `up doc`

Generate various docs or completions for up

**Usage:** `up doc <COMMAND>`

###### **Subcommands:**

* `completions` — Generate shell completions to stdout
* `schema` — Write the up task yaml schema
* `manpages` — Generate man pages for liv and its subcommands
* `markdown` — Print a markdown file with documentation for up and its subcommands



## `up doc completions`

Generate shell completions to stdout.

Completions are printed to stdout. They are designed to be written to a file.

EXAMPLES:

❯ up doc completions zsh | sudo tee >/dev/null /usr/local/share/zsh/site-functions/_up

**Usage:** `up doc completions <SHELL>`

###### **Arguments:**

* `<SHELL>` — Shell for which to generate completions

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`




## `up doc schema`

Write the up task yaml schema.

EXAMPLES:

❯ up doc schema --path /path/to/up-task-schema.json

**Usage:** `up doc schema [PATH]`

###### **Arguments:**

* `<PATH>` — Lib to generate. Defaults to writing to stdout



## `up doc manpages`

Generate man pages for liv and its subcommands.

Manpages are generated into the output directory specified by `--output-dir`.

EXAMPLES:

❯ liv generate manpages --output-dir /usr/local/share/man/man1/

**Usage:** `up doc manpages --output-dir <OUTPUT_DIR>`

###### **Options:**

* `--output-dir <OUTPUT_DIR>` — Directory into which to write the generated manpages



## `up doc markdown`

Print a markdown file with documentation for up and its subcommands.

Markdown file contents are written to the stdout.

**Usage:** `up doc markdown`



## `up list`

List available tasks

**Usage:** `up list [OPTIONS]`

###### **Options:**

* `-b`, `--bootstrap` — Run the bootstrap list of tasks in series first, then run the rest in parallel. Designed for first-time setup
* `-k`, `--keep-going` — Keep going even if a bootstrap task fails
* `-f`, `--fallback-url <FALLBACK_URL>` — Fallback git repo URL to download to get the config
* `-p`, `--fallback-path <FALLBACK_PATH>` — Fallback path inside the git repo to get the config. The default path assumes your `fallback_url` points to a dotfiles repo that is linked into ~

  Default value: `dotfiles/.config/up/up.yaml`
* `-t`, `--tasks <TASKS>` — Optionally pass one or more tasks to run. The default is to run all tasks. This option can be provided multiple times, or use a comma-separated list of values.

   EXAMPLES:

   ❯ up run --tasks=rust,apt --tasks=otherslowtask
* `--console <CONSOLE>` — Tasks stdout/stderr inherit from up's stdout/stderr.

   By default this is true if only one task is executed, and false otherwise. Piping multiple commands to the stdout/stderr of the process will cause task output to be interleaved, which is very confusing when many tasks are run.

  Possible values: `true`, `false`

* `--exclude-tasks <EXCLUDE_TASKS>` — Optionally pass one or more tasks to exclude. The default is to exclude no tasks. Excluded tasks are not run even if specified in `--tasks` (excluding takes priority). This option can be provided multiple times. Tasks specified do not have to exist.

   EXAMPLES:

   ❯ up run --exclude-tasks=brew,slowtask --exclude-tasks=otherslowtask



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

