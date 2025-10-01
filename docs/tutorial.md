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

## Configuration

We can control Fortitude's behaviour with a configuration file, one of `fpm.toml`,
`fortitude.toml`, or `.fortitude.toml`. Let's add a new file:


=== "fpm.toml"

    ```toml
    [extra.fortitude.check]
    # Add check for implicit kinds on real variables
    extend-select = ["implicit-real-kind"]
    ```

=== "fortitude.toml"

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
