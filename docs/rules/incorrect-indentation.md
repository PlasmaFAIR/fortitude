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
- [`check.indentation.program-indent`][check.indentation.program-indent]
- [`check.indentation.module-indent`][check.indentation.module-indent]
- [`check.indentation.procedure-indent`][check.indentation.procedure-indent]
- [`check.indentation.derived-type-indent`][check.indentation.derived-type-indent]
- [`check.indentation.control-flow-indent`][check.indentation.control-flow-indent]
- [`check.indentation.interface-indent`][check.indentation.interface-indent]


[check.indent-width]: ../settings.md#check_indent-width
[check.indentation.program-indent]: ../settings.md#check_indentation_program-indent
[check.indentation.module-indent]: ../settings.md#check_indentation_module-indent
[check.indentation.procedure-indent]: ../settings.md#check_indentation_procedure-indent
[check.indentation.derived-type-indent]: ../settings.md#check_indentation_derived-type-indent
[check.indentation.control-flow-indent]: ../settings.md#check_indentation_control-flow-indent
[check.indentation.interface-indent]: ../settings.md#check_indentation_interface-indent

