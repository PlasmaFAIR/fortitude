# incorrect-indentation (S105)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks that the correct indentation has been used.

## Why is this bad?
Inconsistent indentation makes Fortran less readable and difficult to
understand the scoping of logic.

The expected indentation is inferred on a per-file basis, or it can be set
using the `check.indent-width` option. Continued lines are ignored, as
they are often indented differently for readability.

## Options
- [`check.indent-width`][check.indent-width]
- [`check.incorrect-indentation.indent-programs`][check.incorrect-indentation.indent-programs]
- [`check.incorrect-indentation.indent-modules`][check.incorrect-indentation.indent-modules]
- [`check.incorrect-indentation.indent-procedures`][check.incorrect-indentation.indent-procedures]
- [`check.incorrect-indentation.indent-derived-types`][check.incorrect-indentation.indent-derived-types]
- [`check.incorrect-indentation.indent-control-flow`][check.incorrect-indentation.indent-control-flow]
- [`check.incorrect-indentation.indent-interfaces`][check.incorrect-indentation.indent-interfaces]


[check.indent-width]: ../settings.md#check_indent-width
[check.incorrect-indentation.indent-programs]: ../settings.md#check_incorrect-indentation_indent-programs
[check.incorrect-indentation.indent-modules]: ../settings.md#check_incorrect-indentation_indent-modules
[check.incorrect-indentation.indent-procedures]: ../settings.md#check_incorrect-indentation_indent-procedures
[check.incorrect-indentation.indent-derived-types]: ../settings.md#check_incorrect-indentation_indent-derived-types
[check.incorrect-indentation.indent-control-flow]: ../settings.md#check_incorrect-indentation_indent-control-flow
[check.incorrect-indentation.indent-interfaces]: ../settings.md#check_incorrect-indentation_indent-interfaces

