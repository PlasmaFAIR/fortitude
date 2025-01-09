# Fortitude: A Fortran Linter

A Fortran linter, inspired by (and built upon) [Ruff](https://github.com/astral-sh/ruff).
Written in Rust :crab: and installable with Python :snake:.

- :zap: Blazingly fast, up to hundreds of times faster than other open-source Fortran
  linters.
- :wrench: Automatically fixes linter warnings.
- :chart_with_upwards_trend: 30+ rules, with many more planned.
- :page_with_curl: Multiple output formats, including SARIF and GitHub/GitLab CI.
- :handshake: Follows [community best
  practices](https://fortran-lang.org/en/learn/best_practices/).
- :muscle: Built on a robust [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
  parser -- even syntax errors won't stop it!

## Quickstart

Fortitude can be installed directly into your Python environment:

```bash
pip install fortitude-lint
```

You can then lint your whole project under the current working directory
using the `check` command:

```bash
fortitude check
```

You can also call `check` on individual files, directories, and globs:

```bash
fortitude check main.f90 src/ extra/*.f90
```

Some rule violations can even be fixed automatically:

```bash
fortitude check --fix
```

The `explain` command can be used to get extra information about any rules:

```bash
# Print extra information for all rules
fortitude explain
# Only get information for selected rules, by code or by name
fortitude explain T001 trailing-whitespace
# Print information on all style rules
fortitude explain style
```

New rules and other features may be in 'preview' mode while they undergo further review
and testing. To activate them, use the `--preview` flag:

```bash
fortitude check --preview
```

To see further commands and optional arguments, try using `--help`:

```bash
fortitude --help
fortitude check --help
```

### Rule Selection

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

It is also possible to switch off individual rules or rule categories for specific
files using `--per-file-ignores`:

```bash
fortitude check --per-file-ignores=**/*.f95:non-standard-file-extension
```

as well as for individual statements through "allow" comments of the
form:

```f90
! allow(rule-or-category)
```

Multiple rules or categories can be given as a comma-separated
list. Allow comments apply to the next statement and its contents. So
in the example below, we allow all of the `style` rules and
`superfluous-implicit-none` throughout the whole module, and `use-all`
on the `iso_fortran_env` `use` statement specifically, while
`some_other_module` will still generate a warning.

```f90
! allow(style, superfluous-implicit-none)
module example
    ! allow(use-all)
    use, intrinsic :: iso_fortran_env
    use some_other_module
    implicit none (type, external)
...
```

### Filtering Files

Fortitude will automatically ignore files in some directories (`build/`, `.git/`,
`.venv/`, etc.), and this behaviour can be extended using the `--exclude` option. For
example, to ignore all files in the directory `benchmarks/`:

```bash
fortitude check --exclude=benchmarks
```

You can also configure what extensions Fortitude searches for in directories with
`--file-extensions`:

```bash
fortitude check --file-extensions=f90,fpp
```

### Configuration

Fortitude will look for either a `fortitude.toml` or `fpm.toml` file in the
current directory or one of its parents. If using `fortitude.toml`, settings
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

Arguments on the command line take precedence over those in the configuration file,
so using `--select` will override the choices shown above. You should instead use
`--extend-select` from the command line to select additional rules on top of those in
the configuration file:

```bash
# Selects S, T, and M categories
fortitude check --extend-select=M
```

Similar options include `--extend-exclude`, `--extend-ignore`, and
`--extend-per-file-ignores`.
