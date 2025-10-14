# invalid-tab (PORT031)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for the use of tab characters as whitespace

## Why is this bad?
Tabs are not part of the Fortran standard, and compilers may
reject the source if using a strict conformance mode (for example,
`gfortran -std=f2023 -Werror`).

## Options
- [`check.invalid-tab.indent-width`][check.invalid-tab.indent-width]


[check.invalid-tab.indent-width]: ../settings.md#check_invalid-tab_indent-width

