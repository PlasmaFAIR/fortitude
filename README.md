![Tests](https://github.com/PlasmaFAIR/fortitude/actions/workflows/test.yml/badge.svg)
![Clippy](https://github.com/PlasmaFAIR/fortitude/actions/workflows/clippy.yml/badge.svg)

# Fortitude

A Fortran linter, written in Rust :crab: and installable with Python :snake:.

## Installation

Fortitude can be installed directly into your Python environment:

```bash
pip install fortitude-lint
```

It is also available as a pure Rust project:

```bash
cargo install fortitude
```

## Usage

Fortitude can lint your project using the `check` command:

```bash
fortitude check my_code.f90
```

You can also call `check` on directories, and if no files are provided, `fortitude` will
search for them from your current working directory.

The `explain` command can be used to get extra information about any rules:

```bash
fortitude explain B023
```

If no rules are provided, this will print all rule descriptions to the terminal.

To see further commands and optional arguments, try using `--help`:

```bash
fortitude --help
fortitude check --help
```

## Contributing

Please feel free to add or suggest new rules or comment on the layout of the project
while it's still at this early stage of development. See
[`README.dev.md`](README.dev.md) for a guide on building the project from source,
running tests, and linting/formatting the code.

## License

This work is distributed under the MIT License. See [`LICENSE`](LICENSE) for more
information.
