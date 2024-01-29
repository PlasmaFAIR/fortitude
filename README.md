![Tests](https://github.com/PlasmaFAIR/fortitude/actions/workflows/test.yml/badge.svg)
![Clippy](https://github.com/PlasmaFAIR/fortitude/actions/workflows/clippy.yml/badge.svg)

# Fortitude

A Fortran linter, written in Rust :crab:.

This project is a work-in-progress, and needs a few more feature additions before its
initial release.

## Installation

To install from source, you must first have a working Rust environment (see
[rustup](https://rustup.rs/)). Clone this repository using `--recurse-submodules`, and
install using `cargo`:

```bash
$ git clone https://github.com/PlasmaFAIR/fortitude --recurse-submodules
$ cd fortitude
$ cargo install --path .
```

## Usage

After installing, you can view available commands by calling:

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

To see a list of available rules and their descriptions, you can use:

```bash
$ fortitude explain [RULES]
```

where `[RULES]` is a list of rule codes to explain. If no rules are provided, this
will print all rule descriptions to the terminal.

## Testing

Unit tests can be run by calling:

```bash
$ cargo test
```

Testing is also being performed manually using the file `test.f90`:

```bash
$ fortitude check test.f90
```

The test suite is still in need of work, and we hope to include proper integration
tests soon.

## Contributing

Please feel free to add or suggest new rules, or comment on the layout of the project
while it's still at this early stage of development. When contributing, please use
`cargo clippy` to lint your code, and `cargo fmt` to format it.

## License

This work is distributed under the MIT License. See `LICENSE` for more information.
