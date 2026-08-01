# jlic

[![CI](https://github.com/jtprogru/jlic/actions/workflows/ci.yml/badge.svg)](https://github.com/jtprogru/jlic/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/jlic.svg)](https://crates.io/crates/jlic)
[![docs.rs](https://img.shields.io/docsrs/jlic)](https://docs.rs/jlic)
[![downloads](https://img.shields.io/crates/d/jlic.svg)](https://crates.io/crates/jlic)
[![MSRV](https://img.shields.io/badge/rust-1.97%2B-orange.svg)](https://www.rust-lang.org)
[![license](https://img.shields.io/crates/l/jlic.svg)](LICENSE)

A LICENSE file generator for open source projects. Seven popular license texts are embedded in the binary; the copyright holder and year are filled in automatically from your git config.

Successor to the `srekit license` command, split out into a standalone tool.

```console
$ cd ~/projects/new-thing
$ jlic
MIT → LICENSE
```

## Installation

```bash
# Homebrew
brew install jtprogru/tap/jlic

# Cargo
cargo install jlic

# From source
git clone https://github.com/jtprogru/jlic && cd jlic && make install
```

## Quick start

```bash
jlic                                  # MIT into ./LICENSE, identity from git config
jlic apache-2.0                       # a different license
jlic mit -o docs/LICENSE.txt          # a different path
jlic wtfpl --stdout                   # print instead of writing
jlic mit --name "Acme Inc." --year 2020-2026 --no-email
jlic list                             # what is supported
jlic show gpl-3.0-or-later            # the raw template
jlic notice apache-2.0                # header block for source files
```

An existing file is never overwritten without `--force`, so an accidental `jlic` in someone else's repository cannot clobber the license already chosen there.

## Supported licenses

| SPDX | Aliases | Substitution |
|------|---------|--------------|
| `MIT` (default) | `mit` | year, holder |
| `Apache-2.0` | `apache`, `apache2`, `asl2` | year, holder |
| `GPL-3.0-or-later` | `gpl`, `gpl3`, `gplv3`, `gpl-3.0` | — |
| `BSD-3-Clause` | `bsd`, `bsd3` | year, holder |
| `MPL-2.0` | `mpl`, `mpl2` | — |
| `ISC` | `isc` | year, holder |
| `WTFPL` | `wtfpl`, `wtf` | year, holder |

The GPL-3.0 and MPL-2.0 texts are immutable: no copyright is written into them, it belongs in source file headers instead. That is what `notice` is for:

```console
$ jlic notice gpl-3.0-or-later
Copyright (C) 2026 Mikhail Savin <jtprogru@gmail.com>

This program is free software: you can redistribute it and/or modify
...
SPDX-License-Identifier: GPL-3.0-or-later
```

Passing `--name` to a license that does not accept one prints a warning to stderr and still generates the file.

## Where the name, email and year come from

Each value is looked up along a chain, first non-empty wins:

| Value | Priority |
|-------|----------|
| Name | `--name` → `JLIC_NAME` → `git config user.name` |
| Email | `--email` → `JLIC_EMAIL` → `git config user.email` |
| Year | `--year` → the system's current year |

The email is rendered into the copyright line as `Name <email>` and can be suppressed with `--no-email`. The year accepts both a single value (`2026`) and a range (`2020-2026`).

If the name cannot be determined and the license requires one, jlic exits with an error and writes no file.

## Commands

| Command | What it does |
|---------|--------------|
| `jlic [LICENSE]` | same as `jlic new` |
| `jlic new [LICENSE]` | create a license file |
| `jlic list [--json]` | list licenses, for humans or for scripts |
| `jlic show LICENSE` | the raw template with placeholders |
| `jlic notice [LICENSE]` | copyright block for a source file header |
| `jlic completions SHELL` | completions for bash, zsh, fish, elvish, powershell |
| `jlic man` | man page in roff format |

## Using it as a library

```rust
use jlic::{Context, License, render_license};

let ctx = Context {
    year: "2026".into(),
    name: Some("Mikhail Savin".into()),
    email: None,
};
let text = render_license(License::Mit, &ctx)?;
```

You do not have to build the context by hand either: `Context::resolve(None, None, None)` pulls the values from the environment and git config.

## Development

```bash
make help              # every target
make check             # fmt + clippy + tests, same as CI
make run ARGS="mit --stdout"
make dist              # local release archive with man pages and completions
make update-templates  # refetch license texts from upstream
```

Templates come from [choosealicense.com](https://choosealicense.com), the same source GitHub's license picker uses. Reference texts from SPDX live in `assets/spdx/` and are used only by the test suite: `templates_match_spdx_reference_text` compares every template against its reference, normalizing line wrapping and typography. An edit that changes the meaning of a license therefore fails CI.

After `make update-templates`, always run `make test` and review `git diff assets/`.

## Releases

Tagging `vX.Y.Z` triggers `.github/workflows/release.yml`, which builds four targets (`aarch64`/`x86_64` × macOS/Linux), publishes a GitHub Release with `.tar.gz` archives and their `.sha256` sums, updates `Formula/jlic.rb` in [jtprogru/homebrew-tap](https://github.com/jtprogru/homebrew-tap), and publishes the crate to crates.io. Tags containing a hyphen (`v1.0.0-rc1`) are marked as pre-releases and skipped for the tap.

Two repository secrets are required: `HOMEBREW_TAP_TOKEN` (a PAT with write access to the tap) and `CARGO_REGISTRY_TOKEN`.

## License

MIT — see [LICENSE](LICENSE). The file was generated by jlic itself.
