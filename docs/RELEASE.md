# Release Guide

The Release process is still somewhat manual, and only works when run on macOS for now.

## Process

1. Ensure all changes are pushed, check that CI on the latest commit was green.
   You can also check this badge: ![Master CI Status](https://github.com/gibfahn/up/workflows/Rust/badge.svg)
2. Run the [meta/release][] script.
3. Go to the [GitHub Releases][] page and check everything is working properly.
4. Update the [homebrew formula][].

[CHANGELOG.md]: /CHANGELOG.md
[GitHub Releases]: https://github.com/gibfahn/up/releases
[homebrew formula]: https://github.com/gibfahn/homebrew-tap/tree/main/Formula/up.rb
[meta/release]: ../meta/release
