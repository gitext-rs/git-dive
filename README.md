# git-dive

> **Dive into a file's history to find root cause**

[![codecov](https://codecov.io/gh/gitext-rs/git-dive/branch/master/graph/badge.svg)](https://codecov.io/gh/gitext-rs/git-dive)
[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/git-dive.svg)
[![Crates Status](https://img.shields.io/crates/v/git-dive.svg)](https://crates.io/crates/git-dive)

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE)

## Documentation

- [About](#about)
- [Install](#install)
- [Getting Started](#getting-started)
- [FAQ](#faq)
- [Comparison](docs/comparison.md)
- [Contribute](CONTRIBUTING.md)
- [CHANGELOG](CHANGELOG.md)

## About

`git-dive` is for better understanding why a change was made.  Frequently, we
work on code bases we didn't start which have too little documentation.  Even
worse if the original authors are not around.  `git-blame` is an invaluable
tool for this but it requires a lot of ceremony to get the information you
need.

Features
- Git-native experience
- Syntax highlighting
- Focuses on relative references (e.g. `HEAD~10`)
  - More room for code by merging the SHA and Time columns into a rev column
  - Easier to compare timestamps via the rev column (e.g. `HEAD~10`)
  - Easier to remember, avoiding the need for copy/pasting SHAs
- Easy to find relevant config with `git dive --dump-config -`

Planned Features
- [Interactive pager that let's you browse through time](https://github.com/epage/git-dive/issues?q=is%3Aopen+is%3Aissue+milestone%3A%220.2+-+Interactive+Pager%22)

`git-dive` was inspired by [perforce time lapse
view](https://www.perforce.com/video-tutorials/vcs/using-time-lapse-view).

## Install

[Download](https://github.com/gitext-rs/git-dive/releases) a pre-built binary
(installable via [gh-install](https://github.com/crate-ci/gh-install)).

Or use rust to install:
```console
$ cargo install git-dive
```

### Uninstall

See the uninstall method for your installer.

## Getting Started

## FAQ

[Crates.io]: https://crates.io/crates/git-dive
[Documentation]: https://docs.rs/git-dive
