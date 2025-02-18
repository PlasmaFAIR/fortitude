# missing-default-case (B001)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks that `select case` statements have a `case default`.

## Why is this bad?
Select statements without a default case can lead to incomplete handling of
the possible options. If a value isn't handled by any of the cases, the
program will continue execution, which may lead to surprising results.
Unfortunately, because Fortran doesn't have proper enums, it's not possible
for the compiler to issue warnings for non-exhaustive cases. Having a default
case allows for the program to gracefully handle errors.