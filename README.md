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

It can also be installed as a pure Rust project:

```bash
git clone https://github.com/PlasmaFAIR/fortitude
cargo install --path fortitude
```

## Usage

Fortitude can lint your whole project under the working directory
using the `check` command:

```bash
fortitude check
```

You can also call `check` on individual files, globs, and
directories. You can configure what extensions `fortitude` searches
for in directories with `--file-extensions`:

```bash
fortitude check src --file-extensions=f90,fpp
```

You can select or ignore individual rules or whole groups with
`--select` and `--ignore`:

```bash
# Just check for missing `implicit none`
fortitude check --select=T001
# Also check for missing `implicit none` in interfaces
fortitude check --select=T001,T002
# Ignore all styling rules
fortitude check --ignore=S
# Only check for typing rules, but ignore superfluous implicit none
fortitude check --select=T --ignore=T003
# Rules and categories can also be referred to by name
fortitude check --select=typing --ignore=superfluous-implicit-none
```

Use `--output-format=concise` to get shorter output:

```bash
$ fortitude check --concise
test.f90:2:1: M001 function not contained within (sub)module or program
test.f90:5:1: S061 end statement should read 'end function double'
test.f90:7:1: M001 subroutine not contained within (sub)module or program
test.f90:8:3: P021 real has implicit kind
```

The `explain` command can be used to get extra information about any rules:

```bash
# Print extra information for all rules
fortitude explain
# Only get information for selected rules
fortitude explain T001,T011
# Print information on all style rules
fortitude explain S
# Rules and categories can also be referred to by name
fortitude explain style,superfluous-implicit-none
```

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
select = ["S", "T"]
ignore = ["S001", "S051"]
line-length = 132
```

For `fpm.toml` files, this has to be additionally nested under the
`extra.fortitude` table:

```toml
[extra.fortitude.check]
select = ["S", "T"]
ignore = ["S001", "S051"]
line-length = 132
```

You can use `--extend-select` from the command line to select additional
rules on top of those in the configuration file.

```bash
# Selects S, T, and M categories
fortitude check --extend-select=M
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
