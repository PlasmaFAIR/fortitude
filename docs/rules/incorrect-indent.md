# incorrect-indent (S105)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks that the correct indentation has been used

The complexity of handling semicolons requires that this
rule removes any semicolons used midway through a line

## Why is this bad?
Inconsistent indentation makes Fortran less readable and difficult to
understand the scoping of logic.

## Options
- [`check.indent-width`][check.indent-width]


[check.indent-width]: ../settings.md#check_indent-width

