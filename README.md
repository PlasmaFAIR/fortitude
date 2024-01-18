![Tests](https://github.com/PlasmaFAIR/fortitude/actions/workflows/test.yml/badge.svg)
![Clippy](https://github.com/PlasmaFAIR/fortitude/actions/workflows/clippy.yml/badge.svg)

# Fortitude

A Fortran linter and formatter, written in Rust :crab:. Currently a work-in-progress.

## Installation

To install from source, you must first have a working Rust environment (see
[rustup](https://rustup.rs/)). Clone this repository using `--recurse-submodules`, and
install using `cargo`:

```bash
$ git clone https://github.com/PlasmaFAIR/fortitude --recurse-submodules
$ cd fortitude
$ cargo install
```

## Usage:

After installing, you can view available commands simply by calling:

```bash
$ fortitude
```

The `check` command is used to analyse your Fortran files:

```bash
$ fortitude check [FILES]
```

where `[FILES]` is a list of files and directories to search. If no files are provided,
`fortitude` will search for them from your current working directory. To see additional
options for the `check` tool:

```bash
$ fortitude check --help
```

## TODO

- Command line interface.
    - `check` mode
      - Use `.fortitude.toml` file to set rules project wide.
  - `explain` mode, print help text for a given rule
- Propagate rule errors
- Syntax error rules (scan the tree and report error nodes)
- A few code style rules (leave most until after initial release)
- After gathering violations, check per-file and per-line ignores and discard those we
  don't care about.
- Python package (see how ruff does it, use `maturin`).
- Publish to `crates.io` and PyPI.

## Wishlist

The following will require better analysis of scope:

- Report if a function can be marked pure.
- Report unused variables.
- Report undefined variables.

## Contributing

Please feel free to add or suggest new rules, or comment on the layout of the project
while it's still at this early stage of development. When contributing, please use
`cargo clippy` to lint your code, and `cargo fmt` to format it.
