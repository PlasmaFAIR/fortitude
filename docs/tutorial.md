# Tutorial

This tutorial will walk you through the basics of setting up Fortitude in your
project. See [_Configuring Fortitude_](configuration.md) for more details.

## Getting Started

Let's make a new Fortran file, `numbers.f90`:

```f90
module numbers
  implicit none (external, type)
  private

  public :: greater_than_five

contains
  subroutine greater_than_five(x)
    real, intent(in) :: x
    if (x .gt. 5.0) then
      print*, "Greater than five"
    else
      print*, "Not greater than five"
    end if
  end subroutine greater_than_five
end module numbers
```

To get started, see [_Installing Fortitude_](installation.md) for how to install
Fortitude, and run:

```console
$ fortitude check
```

!!! note

    If you have [uv](https://docs.astral.sh/uv/) installed, you can try Fortitude in one line:

    ```console
    $ uvx --from fortitude-lint fortitude check
    ```

You should see some output like:

```console
$ fortitude check
numbers.f90:8:11: MOD021 [*] deprecated relational operator '.gt.', prefer '>' instead
   |
 6 |   subroutine greater_than_five(x)
 7 |     real, intent(in) :: x
 8 |     if (x .gt. 5) then
   |           ^^^^ MOD021
 9 |       print*, "Greater than five"
10 |     else
   |
   = help: Use '>'

fortitude: 1 files scanned.
Number of errors: 1

For more information about specific rules, run:

    fortitude explain X001,Y002,...

[*] 1 fixable with the `--fix` option.
```

Fortitude has found a use of an old style greater-than operator, and suggests a fix. In
fact, we can even see that Fortitude says this is "fixable", and we can automatically
apply the fix with `fortitude check --fix`:

```console
$ fortitude check --fix
fortitude: 1 files scanned.
All checks passed!
```

It's that easy!

We can get more information about Fortitude's rules using `fortitude
explain`:

```console
$ fortitude explain MOD021
# MOD021: deprecated-relational-operator

Fix is always available.

## What does it do?
Checks for deprecated relational operators

## Why is this bad?
Fortran 90 introduced the traditional symbols for relational operators: `>`,
`>=`, `<`, and so on. Prefer these over the deprecated forms `.gt.`, `.le.`, and
so on.
```

## Configuration

We can control Fortitude's behaviour with a configuration file, one of `fpm.toml`,
`fortitude.toml`, or `.fortitude.toml`. Let's add a new file:


=== "`fpm.toml`"

    ```toml
    [extra.fortitude.check]
    # Add check for implicit kinds on real variables
    extend-select = ["implicit-real-kind"]
    ```

=== "`fortitude.toml` or `.fortitude.toml`"

    ```toml
    [check]
    # Add check for implicit kinds on real variables
    extend-select = ["implicit-real-kind"]
    ```

If we run Fortitude again, we can see we get a new error:

```console
$ fortitude check
numbers.f90:7:5: C022 real has implicit kind
  |
5 | contains
6 |   subroutine greater_than_five(x)
7 |     real, intent(in) :: x
  |     ^^^^ C022
8 |     if (x > 5.0) then
9 |       print*, "Greater than five"
  |

fortitude: 1 files scanned.
Number of errors: 1

For more information about specific rules, run:

    fortitude explain X001,Y002,...
```

For a complete description of supported settings, see [_Settings_](settings.md).

### Rule Selection

Fortitude has [many lint rules](rules.md), split into several categories: `error` (syntax
errors and problems reading files), `correctness` (bugprone constructions), `obsolescent`
(deprecated features), `modernisation` (discouraged out-dated features), `style`
(opinionated advice), `portability` (non-standard features), and `fortitude` (specific to
Fortitude). Not all rules are suitable for all projects: some are more opinionated than
others, or are more suitable for libraries rather than applications, and so on.

The default set of rules is limited to those that we think are the most useful to the
majority of projects, and are a mixture of the different categories.

#### Understanding and Discovering Rules and Categories

As well as the [online documentation](rules.md), you can also get an explanation
of a particular rule on the command line with the `explain` subcommand:

```console
$ fortitude explain implicit-typing
# C001: implicit-typing

Fix is sometimes available.

## What does it do?
Checks for missing `implicit none`.
...
```

`fortitude explain` takes the same arguments as [`select`](settings.md#select):
a rule name or short code, a category name or short code, or `ALL`.

You can get a quick list of all the available rules, along with their short
codes and a brief summary by running `fortitude explain --summary`:

```console
$ fortitude explain --summary
C001    implicit-typing: {entity} missing 'implicit none'. Rule is stable. Fix is sometimes available.
C002    interface-implicit-typing: interface '{name}' missing 'implicit none'. Rule is stable. Fix is sometimes available.
C003    implicit-external-procedures: 'implicit none' missing 'external'. Rule is stable. Fix is not available.
C011    missing-default-case: Missing default case may not handle all values. Rule is in preview. Fix is not available.
...
```

For a quick summary of all the rules in a given category:
```console
$ fortitude explain --summary style
S001    line-too-long: line length of {actual_length}, exceeds maximum {max_length}. Rule is stable. Fix is not available.
S061    unnamed-end-statement: end statement should be named.. Rule is stable. Fix is always available.
S071    missing-double-colon: variable declaration missing '::'. Rule is stable. Fix is always available.
...
```

And, lastly, to see all the categories use `--list-categories`:

```console
$ fortitude explain --list-categories
E       error: Failure to parse a file.
C       correctness: Detect code that is bug-prone or likely to be incorrect.
OB      obsolescent: Obsolescent language features, as determined by the Fortran standard.
MOD     modernisation: Update to modern Fortran features. Used for less severe issues than `Obsolescent`, and goes beyond recommendations in the Fortran standard.
S       style: Violation of style conventions.
PORT    portability: Avoid platform/compiler-specific features.
FORT    fortitude: Fortitude specific rules.
```

### Ignore Errors

Any rule can be ignored by adding a `! allow(<rule-name>)` comment before the statement in
question. For example, let's ignore the `implicit-real-kind` rule:

```f90
  subroutine greater_than_five(x)
    ! allow(implicit-real-kind)
    real, intent(in) :: x
    if (x > 5.0) then
      print*, "Greater than five"
    else
      print*, "Not greater than five"
    end if
  end subroutine greater_than_five
```

If we run `fortitude check` again, we'll see no errors reported:

```console
$ fortitude check
fortitude: 1 files scanned.
All checks passed!
```

See [_Error suppression_](linter.md#error-suppression) for more details on how these
comments work.
