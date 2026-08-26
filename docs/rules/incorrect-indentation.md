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
- [`check.incorrect-indentation.program-indents`][check.incorrect-indentation.program-indents]
- [`check.incorrect-indentation.module-indents`][check.incorrect-indentation.module-indents]
- [`check.incorrect-indentation.procedure-indents`][check.incorrect-indentation.procedure-indents]
- [`check.incorrect-indentation.derived-type-indents`][check.incorrect-indentation.derived-type-indents]
- [`check.incorrect-indentation.control-flow-indents`][check.incorrect-indentation.control-flow-indents]
- [`check.incorrect-indentation.interface-indents`][check.incorrect-indentation.interface-indents]
- [`check.incorrect-indentation.line-continuation-indents`][check.incorrect-indentation.line-continuation-indents]


[check.indent-width]: ../settings.md#check_indent-width
[check.incorrect-indentation.ignore-semicolons]: ../settings.md#check_incorrect-indentation_ignore-semicolons
[check.incorrect-indentation.program-indents]: ../settings.md#check_incorrect-indentation_program-indents
[check.incorrect-indentation.module-indents]: ../settings.md#check_incorrect-indentation_module-indents
[check.incorrect-indentation.procedure-indents]: ../settings.md#check_incorrect-indentation_procedure-indents
[check.incorrect-indentation.derived-type-indents]: ../settings.md#check_incorrect-indentation_derived-type-indents
[check.incorrect-indentation.control-flow-indents]: ../settings.md#check_incorrect-indentation_control-flow-indents
[check.incorrect-indentation.interface-indents]: ../settings.md#check_incorrect-indentation_interface-indents
[check.incorrect-indentation.line-continuation-indents]: ../settings.md#check_incorrect-indentation_line-continuation-indents

