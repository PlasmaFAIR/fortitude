# Configuration

Fortitude will look for either a `fortitude.toml` or `fpm.toml` file
in the current directory, or one of its parents. If using
`fortitude.toml` or `.fortitude.toml`, settings should be under the command name, while for
`fpm.toml` files, this has to be additionally nested under the
`extra.fortitude` table:


=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    select = ["S", "T"]
    ignore = ["S001", "S051"]
    line-length = 132
    ```
=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    select = ["S", "T"]
    ignore = ["S001", "S051"]
    line-length = 132
    ```

For complete documentation of the available configuration options, see
[_Settings_](settings.md).

## Discovering files

Without any other arguments, `fortitude check` searches the current directory recursively
for Fortran files. You can also pass an explicit list of files or directories to search,
or control which files or directories should be excluded from this search. You can also
configure what extensions Fortitude searches for in directories with
[`check.file-extensions`](settings.md#file-extensions) or `--file-extensions`:

```console
$ fortitude check --file-extensions=f90,fpp
```

Files in your `.gitignore` will be excluded from the file search automatically, though
this behaviour can be deactivated by passing `--no-respect-gitignore`.  Files in certain
directories (`build/`, `.git/`, `.venv/`, etc.) will also be excluded by default. An
additional comma-separated list of excluded files and directories can be set using the
[`check.exclude`](settings.md#exclude) option. For example, to exclude all files in the
directories `benchmarks/` and `tests/`:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    exclude = ["benchmarks", "tests"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    exclude = ["benchmarks", "tests"]
    ```

You can also use pattern matching with a glob (`*`) symbol:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    exclude = ["test_*"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    exclude = ["test_*"]
    ```

Note that Fortitude will still check excluded files if you pass their paths
directly, so the following will still check the `benchmarks/` directory:

```console
$ fortitude check --exclude=benchmarks benchmarks
```

Setting [`check.force-excludes`](settings.md#force-exclude) to `true` will enforce
exclusions even in this scenario.

### Default inclusions

By default, Fortitude will discover Fortran files with many common extensions,
including `.f90`, `.F90`, `.f95`. You can change the default selection using the
[`include`](settings.md#include) setting:

=== "`fpm.toml`"

    ```toml
    [extra.fortitude]
    include = ["*.f90", "*.fpp"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    include = ["*.f90", "*.fpp"]
    ```

!!! info "Introduced in Fortitude 0.7.6"

## Command-line interface

Some settings can be given or overidden explicitly on the command line, particularly those
related to rule selection:

```console
$ fortitude check --select S001 --select implicit-typing --quiet
```

Settings on the command line always take precedence over those given in the configuration
file.

### Full command-line interface

See `fortitude help` for the full list of Fortitude's top-level commands:

<!-- Begin auto-generated command help. -->

```text
A Fortran linter, written in Rust and installable with Python

Usage: fortitude [OPTIONS] <COMMAND>

Commands:
  check    Perform static analysis on files and report issues
  explain  Get descriptions, rationales, and solutions for each rule
  version  Display Fortitude's version
  help     Print this message or the help of the given subcommand(s)

Options:
      --config-file <CONFIG_FILE>  Path to a TOML configuration file
  -h, --help                       Print help
  -V, --version                    Print version

Log levels:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Print diagnostics, but nothing else
  -s, --silent   Disable all logging (but still exit with status code "1" upon detecting diagnostics)

For help with a specific command, see: `fortitude help <command>`.
```

<!-- End auto-generated command help. -->

Or `fortitude help check` for more on the linting command:

<!-- Begin auto-generated check help. -->

```text
Perform static analysis on files and report issues

Usage: fortitude check [OPTIONS] [FILES]...

Arguments:
  [FILES]...  List of files or directories to check [default: .]

Options:
      --fix
          Apply fixes to resolve lint violations. Use `--no-fix` to disable or `--unsafe-fixes` to include unsafe fixes
      --unsafe-fixes
          Include fixes that may not retain the original intent of the code. Use `--no-unsafe-fixes` to disable
      --show-fixes
          Show an enumeration of all fixed lint violations. Use `--no-show-fixes` to disable
      --fix-only
          Apply fixes to resolve lint violations, but don't report on, or exit non-zero for, leftover violations. Implies `--fix`. Use `--no-fix-only` to disable or `--unsafe-fixes` to include unsafe fixes
      --ignore-allow-comments
          Ignore any `allow` comments
      --output-format <OUTPUT_FORMAT>
          Output serialization format for violations. The default serialization format is "full" [env: FORTITUDE_OUTPUT_FORMAT=] [possible values: concise, full, json, json-lines, junit, grouped, github, gitlab, pylint, rdjson, azure, sarif]
  -o, --output-file <OUTPUT_FILE>
          Specify file to write the linter output to (default: stdout) [env: FORTITUDE_OUTPUT_FILE=]
      --preview
          Enable preview mode; checks will include unstable rules and fixes. Use `--no-preview` to disable
      --progress-bar <PROGRESS_BAR>
          Progress bar settings. Options are "off" (default), "ascii", and "fancy" [possible values: off, fancy, ascii]
      --show-settings
          See the settings fortitude will use to check a given Fortran file
      --show-files
          See the files fortitude will be run against with the current settings
      --statistics
          Show counts for every rule with at least one violation
  -h, --help
          Print help

Rule selection:
      --ignore <RULE_CODE>
          Comma-separated list of rules to ignore
      --select <RULE_CODE>
          Comma-separated list of rule codes to enable (or ALL, to enable all rules)
      --extend-select <RULE_CODE>
          Like --select, but adds additional rule codes on top of those already specified
      --per-file-ignores <FILE_PATTERN:RULE_CODE>
          List of mappings from file pattern to code to exclude
      --extend-per-file-ignores <FILE_PATTERN:RULE_CODE>
          Like `--per-file-ignores`, but adds additional ignores on top of those already specified

File selection:
      --file-extensions <EXTENSION>
          File extensions to check
      --exclude <FILE_PATTERN>
          List of paths, used to omit files and/or directories from analysis
      --extend-exclude <FILE_PATTERN>
          Like --exclude, but adds additional files and directories on top of those already excluded
      --force-exclude
          Enforce exclusions, even for paths passed to Fortitude directly on the command-line. Use `--no-force_exclude` to disable
      --respect-gitignore
          Respect `.gitignore`` files when determining which files to check. Use `--no-respect-gitignore` to disable

Per-Rule Options:
      --line-length <LINE_LENGTH>  Set the maximum allowable line length

Miscellaneous:
      --stdin-filename <STDIN_FILENAME>
          The name of the file when passing it through stdin
  -e, --exit-zero
          Exit with status code "0", even upon detecting lint violations
      --exit-non-zero-on-fix
          Exit with a non-zero status code if any files were modified via fix, even if no lint violations remain

Log levels:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Print diagnostics, but nothing else
  -s, --silent   Disable all logging (but still exit with status code "1" upon detecting diagnostics)
```

<!-- End auto-generated check help. -->

## Shell autocompletion

Fortitude supports autocompletion for most shells. A shell-specific completion script can be generated
by `fortitude generate-shell-completion <SHELL>`, where `<SHELL>` is one of `bash`, `elvish`, `fig`, `fish`,
`powershell`, or `zsh`.

The exact steps required to enable autocompletion will vary by shell. For example instructions,
see the [Poetry](https://python-poetry.org/docs/#enable-tab-completion-for-bash-fish-or-zsh) or
[ripgrep](https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#complete) documentation.

As an example: to enable autocompletion for Bash, run `fortitude
generate-shell-completion bash > ~/.bash_completion`, then reload your
shell.

## Editor integration

We aim to add full Language Server Protocol (LSP) support in a future release
which will enable Fortitude to run from within editors such as (Neo)Vim, Emacs
and VSCode. In the meantime, it is possible to configure these editors to
automate some tasks.

### (NeoVim)

Adding the following to your `~/.vimrc` or `~/.config/nvim/init.vim` file
will set the commands `:FortitudeFix` and `:FortitudeFixUnsafe` which will
apply fixes to the current file in your buffer:

```vim
command! FortitudeFix execute ':%! fortitude check --fix-only --quiet --stdin-filename=%'
command! FortitudeFixUnsafe execute ':%! fortitude check --fix-only --unsafe-fixes --quiet --stdin-filename=%'
```
