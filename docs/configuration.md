# Configuration

Fortitude will look for either a `fortitude.toml` or `fpm.toml` file
in the current directory, or one of its parents. If using
`fortitude.toml`, settings should be under the command name, while for
`fpm.toml` files, this has to be additionally nested under the
`extra.fortitude` table:


=== "fortitude.toml"

    ```toml
    [check]
    select = ["S", "T"]
    ignore = ["S001", "S051"]
    line-length = 132
    ```
=== "fpm.toml"

    ```toml
    [extra.fortitude.check]
    select = ["S", "T"]
    ignore = ["S001", "S051"]
    line-length = 132
    ```

For complete documentation of the available configuration options, see
[_Settings_](settings.md).

## Full command-line interface

See `fortitude help` for the full list of Fortitude's top-level commands:

<!-- Begin auto-generated command help. -->

```text
A Fortran linter, written in Rust and installable with Python

Usage: fortitude [OPTIONS] <COMMAND>

Commands:
  check    Perform static analysis on files and report issues
  explain  Get descriptions, rationales, and solutions for each rule
  help     Print this message or the help of the given subcommand(s)

Options:
      --config-file <CONFIG_FILE>  Path to a TOML configuration file
  -h, --help                       Print help
  -V, --version                    Print version

Log levels:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Print diagnostics, but nothing else
  -s, --silent   Disable all logging (but still exit with status code "1" upon detecting diagnostics)
```

<!-- End auto-generated command help. -->

Or `fortitude help check` for more on the linting command:

<!-- Begin auto-generated check help. -->

```text
Perform static analysis on files and report issues

Usage: fortitude check [OPTIONS] [FILES]...

Arguments:
  [FILES]...  List of files or directories to check. Directories are searched recursively for Fortran files. The `--file-extensions` option can be used to control which files are included in the search [default: .]

Options:
      --fix
          Apply fixes to resolve lint violations. Use `--no-fix` to disable or `--unsafe-fixes` to include unsafe fixes
      --unsafe-fixes
          Include fixes that may not retain the original intent of the code. Use `--no-unsafe-fixes` to disable
      --show-fixes
          Show an enumeration of all fixed lint violations. Use `--no-show-fixes` to disable
      --fix-only
          Apply fixes to resolve lint violations, but don't report on, or exit non-zero for, leftover violations. Implies `--fix`. Use `--no-fix-only` to disable or `--unsafe-fixes` to include unsafe fixes
      --output-format <OUTPUT_FORMAT>
          Output serialization format for violations. The default serialization format is "full" [env: FORTITUDE_OUTPUT_FORMAT=] [possible values: concise, full, json, json-lines, junit, grouped, github, gitlab, pylint, rdjson, azure, sarif]
  -o, --output-file <OUTPUT_FILE>
          Specify file to write the linter output to (default: stdout) [env: FORTITUDE_OUTPUT_FILE=]
      --preview
          Enable preview mode; checks will include unstable rules and fixes. Use `--no-preview` to disable
      --progress-bar <PROGRESS_BAR>
          Progress bar settings. Options are "off" (default), "ascii", and "fancy" [possible values: off, fancy, ascii]
      --stdin-filename <STDIN_FILENAME>
          The name of the file when passing it through stdin
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

Log levels:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Print diagnostics, but nothing else
  -s, --silent   Disable all logging (but still exit with status code "1" upon detecting diagnostics)
```

<!-- End auto-generated check help. -->
