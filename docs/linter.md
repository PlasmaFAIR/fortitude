# The Fortitude Linter

The Fortitude linter is a _very_ fast Fortran linter heavily inspired by and built on the
[Ruff linter for Python](https://docs.astral.sh/ruff). If you've ever used Ruff, Fortitude
should feel very familiar.

## `fortitude check`

`fortitude check` is the main feature of Fortitude, taking a list of files and/or
directories, and checking all discovered Fortran files for common (and less common!)
errors and style violations, optionally fixing any that are fixable. Fortitude searches
directories recursively for any Fortran files:

```console
$ fortitude check                   # Lint Fortran files in current directory
$ fortitude check --fix             # Lint and fix files in current directory
$ fortitude check path/to/source/   # Lint Fortran files in `path/to/source/`
```

You can see the list of all available options with [`fortitude check
--help`](configuration.md#full-command-line-interface).

## Rule selection

The set of enabled rules is controlled via the [`check.select`][check_select],
[`check.extend-select`][check_extend_select], and [`check.ignore`][check_ignore] settings.

Like Ruff and Flake8, Fortitude gives each rule a short code (for example, `C003`)
consisting of a one-to-three letter prefix (for the category) followed by three
digits. Fortitude also gives each rule a more human-readable English name, such as
`implicit-external-procedures`, which can always be used instead of the short code in
places like [`check.select`][check_select] and [`check.ignore`][check_ignore].

Rule selectors like [`check.select`][check_select] and [`check.ignore`][check_ignore]
accept any combination of the following:

- category prefixes like `C`,
- category names like `correctness`,
- rule short codes like `C003`,
- rule names like `implicit-external-procedures`.

For example, in the following configuration file:

=== "fpm.toml"

    ```toml
    [extra.fortitude.check]
    select = ["C", "style"]
    ignore = ["implict-external-procedures", "S001"]
    ```

=== "fortitude.toml"

    ```toml
    [check]
    select = ["C", "style"]
    ignore = ["implict-external-procedures", "S001"]
    ```

Fortitude would enable all rules in the `correctness` (`C`) and `style` (`S`) categories,
except for `implicit-external-procedures` (`C003`) and `line-too-long` (`S001`).

Like Ruff, Fortitude also has the special `ALL` code, which enables all rules. This should
be used with discretion as it will implicitly enable any new rules whenever you upgrade
Fortitude.

As a guideline, prefer using [`check.select`][check_select] over
[`check.extend-select`][check_extend_select] to make your rule set explicit in your
configuration file.

Setting [`--select`][check_select] on the command line will override
[`check.select`][check_select] in the configuration file. For example, given the
configuration file above, running `fortitude check --select S001` will select *only*
`S001` (`line-too-long`).

If instead you want to select additional rules from the command line, use
[`--extend-select`][check_extend_select]. Running `fortitude check --extend-select
obsolescent` in combination with the settings file above will result in Fortitude
enforcing all rules in the `correctness`, `style`, and `obsolescent` categories, except
for `C003` and `S001`.

### Preview rules

New rules and other features may be in 'preview' mode while they undergo further review
and testing. To activate them, use the [`--preview`](settings.md#preview) flag:

```bash
fortitude check --preview
```

For more details on how preview works, see [_Preview_](preview.md).

## Fixes

For some lint errors, Fortitude supports automatic fixes, such as rewriting some
deprecated syntax, remove/add whitespace, add missing construct names, and so on.

To apply these fixes, pass `--fix` to `fortitude check`:

```console
$ fortitude check --fix
```

To see which rules Fortitude can automatically fix, see [_Rules_](rules.md).

### Fix safety

Another concept Fortitude borrows from Ruff is that of _fix safety_: fixes are labelled as
either "safe" or "unsafe". Safe fixes will not change the meaning or intent of your code,
while unsafe fixes may change the meaning.

For example, [`implicit-typing`](rules/implicit-typing.md) (`C001`) checks for missing
`implicit none` statements, common in older code, and a frequent source of bugs. However,
for codes that are relying on implicit typing, this is not a sufficient fix (as variables
will need to be explicitly declared as well), so it is not always safe to apply.

Fortitude only enables safe fixes by default. Unsafe fixes can be enabled by settings
[`unsafe-fixes`](settings.md#unsafe-fixes) in your configuration file or passing the
`--unsafe-fixes` flag to `fortitude check`:

```console
# Show unsafe fixes
fortitude check --unsafe-fixes

# Apply unsafe fixes
fortitude check --fix --unsafe-fixes
```

By default, Fortitude will display a hint when unsafe fixes are available but not
enabled. The suggestion can be silenced by setting the
[`unsafe-fixes`](settings.md#unsafe-fixes) setting to `false` or using the
`--no-unsafe-fixes` flag.


## Error suppression

Fortitude has several ways of suppressing lint errors, whether they're false positives or
permissible in context.

To globally ignore a rule, add it to the [`check.ignore`][check_ignore] list, either on
the command line for a one-off check, or in your `fpm.toml` or `fortitude.toml` file for a
more permanent suppression for a project.

For slightly more fine-grained control, rules can be ignored for individual files through
the [`check.per-file-ignores`](settings.md#check_per-file-ignores) setting in your
`fpm.toml` or `fortitude.toml` file.

For the most fine-grained control, rules can be suppressed in the source code too, which
is useful for allowing individual exceptions for whatever reason. Fortitude deviates from
how Ruff works here, and instead uses something like [Rust's
approach](https://doc.rust-lang.org/rustc/lints/levels.html#via-an-attribute), by using an
`allow` or "suppression" comment _before_ the statement. This `allow` comment then applies
to the whole of the next statement, which could be an entire module, for example:

```f90
! allow(superfluous-implicit-none)
module numbers
  ! allow(use-all)
  use, intrinsic :: iso_fortran_env
  implicit none

contains
  subroutine greater_than_five(x)
    use some_other_module
    implicit none
  ...
```

Here, the `superfluous-implicit-none` check is disabled for all procedures in the whole
`numbers` module, including in `greater_than_five`, while `use-all` is disabled
only for the `iso_fortran_env` import and will still apply to `some_other_module`.

Multiple rules can be in an `allow` comment, separated by commas, and you can use the
typical [rule selection](linter.md#rule-selection) naming:

```f90
! allow(style, M, FORT002, implicit-real-kind)
```

will allow all `style` and `modernisation` rules, as well as `FORT002`
(`unused-allow-comment`) and `implicit-real-kind` (`C022`).

### Unused `allow` comments

By default, Fortitude will detect "unused" `allow` comments via
[`unused-allow-comment`](rules/unused-allow-comment.md): that is, if a suppression comment
says it allows a given warning, then the statement(s) it applies to _should_ generate
those warnings (and then be allowed by the comment).

Fortitude can automatically remove unused `allow` comments.

You can also temporarily ignore these suppression comments with `--ignore-allow-comments`
on the command line.

## Learning more about a rule

The `explain` command can be used to get extra information about any rules. Without any
arguments, it returns information on all rules; otherwise you can pass it any number of
rules or categories, following the usual [rule selection](linter.md#rule-selection)
naming:

```console
# Print extra information for all rules
$ fortitude explain
# Only get information for selected rules, by code or by name
$ fortitude explain C001 trailing-whitespace
# Print information on all style rules
$ fortitude explain style
```

## Exit codes

By default, `fortitude check` exits with the following status codes:

- `0` if no violations were found, or if all present violations were fixed automatically.
- `1` if violations were found.
- `2` if Fortitude terminates abnormally due to invalid configuration, invalid CLI options, or an
    internal error.

This convention mirrors that of tools like ESLint, Prettier, and RuboCop.

`fortitude check` supports two command-line flags that alter its exit code behavior:

- `--exit-zero` will cause Fortitude to exit with a status code of `0` even if violations were found.
    Note that Fortitude will still exit with a status code of `2` if it terminates abnormally.
- `--exit-non-zero-on-fix` will cause Fortitude to exit with a status code of `1` if violations were
    found, _even if_ all such violations were fixed automatically. Note that the use of
    `--exit-non-zero-on-fix` can result in a non-zero exit code even if no violations remain after
    fixing.

[check_select]: settings.md#check_select
[check_extend_select]: settings.md#check_extend-select
[check_ignore]: settings.md#check_ignore
