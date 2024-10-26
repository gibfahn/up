# up

[![Latest Version (crates.io)](https://img.shields.io/crates/v/up.svg)](https://crates.io/crates/up)
[![Latest Version (lib.rs)](https://img.shields.io/crates/v/up.svg)](https://lib.rs/crates/up)
[![Documentation (docs.rs)](https://docs.rs/up/badge.svg)](https://docs.rs/up)
![Master CI Status](https://github.com/gibfahn/up-rs/workflows/Rust/badge.svg)

Wrapper tool to keep a dev machine up to date. It has a few different features that help with this.

See `up --help` for more details.

## Install

The binary is self-contained, you can simply download it and mark the binary as executable:

```shell
curl --create-dirs -Lo ~/bin/up https://github.com/gibfahn/up/releases/latest/download/up-$(uname)
chmod +x ~/bin/up
```

Or if you have Cargo on your system you can also build it from source:

```shell
cargo install up
```

## Subcommands

### Link

```console
$ up link --from ~/code/dotfiles --to ~
```

symlinks the files in `dotfiles` into the matching directory in `~` (so `~/.config/git/config` becomes a link to
`~/code/dotfiles/.config/git/config`).

### Update

Coming soon.

Updates all the software on your machine.

## Contributing and Developing

See [CONTRIBUTING.md](/docs/CONTRIBUTING.md).

## Related Projects

- [`topgrade`](https://github.com/r-darwish/topgrade)
