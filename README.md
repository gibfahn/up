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

Or if you use homebrew you can install it via:

```shell
brew install gibfahn/tap/up
```

## Usage

See [CommandLineHelp-macOS.md](https://github.com/gibfahn/up/tree/main/docs/CommandLineHelp-macOS.md) or [CommandLineHelp-Linux.md](https://github.com/gibfahn/up/tree/main/docs/CommandLineHelp-Linux.md) (or run `up --help` or `man up` locally) for full usage.

## Contributing and Developing

See [CONTRIBUTING.md](/docs/CONTRIBUTING.md).

## Related Projects

- [`topgrade`](https://github.com/r-darwish/topgrade)
