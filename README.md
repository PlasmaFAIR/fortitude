![Tests](https://github.com/PlasmaFAIR/fortitude/actions/workflows/test.yml/badge.svg)
![Clippy](https://github.com/PlasmaFAIR/fortitude/actions/workflows/clippy.yml/badge.svg)

# Fortitude

A Fortran linter, written in Rust :crab: and installable with Python :snake:.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

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

## Configuration

Fortitude will look for either a `fortitude.toml` or `fpm.toml` file in the
current directory, or one of its parents. If using `fortitude.toml`, settings
should be under the command name:

```toml
[check]
ignore = ["S001", "S051"]
line-length = 132
```

For `fpm.toml` files, this has to be additionally nested under the
`extra.fortitude` table:

```toml
[extra.fortitude.check]
ignore = ["S001", "S051"]
line-length = 132
```

## Contributing

Please feel free to add or suggest new rules or comment on the layout of the project
while it's still at this early stage of development. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for a guide on contributing to the project, and
[`README.dev.md`](README.dev.md) for details on building the project from source,
running tests, and linting/formatting the code. Please consult our [code of
conduct](CODE_OF_CONDUCT.md) before contributing.

## License

This work is distributed under the MIT License. See [`LICENSE`](LICENSE) for more
information.

Fortitude is inspired by, and uses parts from
[ruff](https://github.com/astral-sh/ruff), used under the MIT licence. See
[`LICENSE`](LICENSE) for more information.
