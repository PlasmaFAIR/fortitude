# incorrect-indentation (S105)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks that the correct indentation has been used

The complexity of handling semicolons requires that this
rule either removes any semicolons used midway through a line
or ignores any lines containing a semicolon. This logic can be
toggled using the `ignore-semicolons` option.

## Why is this bad?
Inconsistent indentation makes Fortran less readable and difficult to
understand the scoping of logic.

## Options
- [`check.indent-width`][check.indent-width]
- [`check.incorrect-indentation.ignore-semicolons`][check.incorrect-indentation.ignore-semicolons]
- [`check.incorrect-indentation.program-indent`][check.incorrect-indentation.program-indent]
- [`check.incorrect-indentation.module-indent`][check.incorrect-indentation.module-indent]
- [`check.incorrect-indentation.procedure-indent`][check.incorrect-indentation.procedure-indent]
- [`check.incorrect-indentation.derived-type-indent`][check.incorrect-indentation.derived-type-indent]
- [`check.incorrect-indentation.control-flow-indent`][check.incorrect-indentation.control-flow-indent]
- [`check.incorrect-indentation.interface-indent`][check.incorrect-indentation.interface-indent]
- [`check.incorrect-indentation.line-continuation-indent`][check.incorrect-indentation.line-continuation-indent]


[check.indent-width]: ../settings.md#check_indent-width
[check.incorrect-indentation.ignore-semicolons]: ../settings.md#check_incorrect-indentation_ignore-semicolons
[check.incorrect-indentation.program-indent]: ../settings.md#check_incorrect-indentation_program-indent
[check.incorrect-indentation.module-indent]: ../settings.md#check_incorrect-indentation_module-indent
[check.incorrect-indentation.procedure-indent]: ../settings.md#check_incorrect-indentation_procedure-indent
[check.incorrect-indentation.derived-type-indent]: ../settings.md#check_incorrect-indentation_derived-type-indent
[check.incorrect-indentation.control-flow-indent]: ../settings.md#check_incorrect-indentation_control-flow-indent
[check.incorrect-indentation.interface-indent]: ../settings.md#check_incorrect-indentation_interface-indent
[check.incorrect-indentation.line-continuation-indent]: ../settings.md#check_incorrect-indentation_line-continuation-indent

