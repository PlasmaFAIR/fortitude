# Fortitude: A Fortran Linter

A Fortran linter, inspired by (and built upon) [Ruff](https://github.com/astral-sh/ruff).
Written in Rust :crab: and installable with Python :snake:.

- :zap: Blazingly fast, up to hundreds of times faster than other open-source Fortran
  linters.
- :wrench: Automatically fixes linter warnings.
- :chart_with_upwards_trend: 50+ rules, with many more planned.
- :page_with_curl: Multiple output formats, including SARIF and GitHub/GitLab CI.
- :handshake: Follows [community best
  practices](https://fortran-lang.org/learn/best_practices/).
- :muscle: Built on a robust [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
  parser.

## Quickstart

Download Fortitude by your prefered method as [described here](installation.md)

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
fortitude explain C001 trailing-whitespace
# Print information on all style rules
fortitude explain style
```

New rules and other features may be in 'preview' mode while they undergo further review
and testing. To activate them, use the [`--preview`](settings.md#preview) flag:

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
[`--select`](settings.md#select) and [`--ignore`](settings.md#ignore):

```bash
# Just check for missing `implicit none`
fortitude check --select=C001
# Also check for missing `implicit none` in interfaces
fortitude check --select=C001,C002
# Ignore all styling rules
fortitude check --ignore=S
# Only check for style rules, but ignore superfluous implicit none
fortitude check --select=S --ignore=S201
# Rules and categories can also be referred to by name
fortitude check --select=style --ignore=superfluous-implicit-none
```

It is also possible to switch off individual rules or rule categories for specific
files using [`--per-file-ignores`](settings.md#per-file-ignores):

```bash
fortitude check --per-file-ignores=**/*.f95:non-standard-file-extension
```

as well as for individual statements through "allow" comments of the
form:

```f90
! allow(rule)
```

Multiple rules can be given as a comma-separated list. Allow comments
apply to the next statement and its contents. So in the example below,
we allow `line-too-long` and `superfluous-implicit-none` throughout
the whole module, and `use-all` on the `iso_fortran_env` `use`
statement specifically, while `some_other_module` will still generate
a warning.

```f90
! allow(line-too-long, superfluous-implicit-none)
module example
    ! allow(use-all)
    use, intrinsic :: iso_fortran_env
    use some_other_module
    implicit none (type, external)
...
```

### Filtering Files

You can configure what extensions Fortitude searches for in directories with
[`--file-extensions`](settings.md#file-extensions):

```bash
fortitude check --file-extensions=f90,fpp
```

Files in your `.gitignore` will be excluded from the file search automatically,
though this behaviour can be deactivated by passing `--no-respect-gitignore`.
Files in certain directories (`build/`, `.git/`, `.venv/`, etc.) will also be
excluded by default. An additional comma-separated list of excluded files and
directories can be set using the [`--exclude`](settings.md#exclude) option. For
example, to exclude all files in the directories `benchmarks/` and `tests/`:

```bash
fortitude check --exclude=benchmarks,tests
```

You can also use pattern matching with a glob (`*`) symbol:

```bash
fortitude check --exclude=test_*
```

Note that Fortitude will still check excluded files if you pass their paths
directly, so the following will still check the `benchmarks/` directory:

```bash
fortitude check --exclude=benchmarks benchmarks
```

Passing [`--force-excludes`](settings.md#force-exclude) will enforce exclusions
even in this scenario.

### Configuration

Fortitude will look for either a `fortitude.toml` or `fpm.toml` file in the
current directory or one of its parents. If using `fortitude.toml`, settings
should be under the command name:

```toml
[check]
select = ["C", "E", "S"]
ignore = ["S001", "S082"]
line-length = 132
```

For `fpm.toml` files, this has to be additionally nested under the
`extra.fortitude` table:

```toml
[extra.fortitude.check]
select = ["C", "E", "S"]
ignore = ["S001", "S082"]
line-length = 132
```

Arguments on the command line take precedence over those in the configuration
file, so using `--select` will override the choices shown above. You should
instead use [`--extend-select`](settings.md#extend-select) from the command line
to select additional rules on top of those in the configuration file:

```bash
fortitude check --extend-select=OB
```

Similar options include [`--extend-exclude`](settings.md#extend-exclude),
`--extend-ignore` (command line only), and `--extend-per-file-ignores` (command
line only).

For a full list of options, please see the [settings page](settings.md).
