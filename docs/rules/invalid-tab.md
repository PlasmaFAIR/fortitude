# invalid-tab (PORT031)
Fix is sometimes available.

This rule is turned on by default.

## What it does
Checks for the use of tab characters as whitespace

## Why is this bad?
Tabs are not part of the Fortran standard, and compilers may
reject the source if using a strict conformance mode (for example,
`gfortran -std=f2023 -Werror`).

## Options
If the more fine grained option (`check.invalid-tab.indent-width`) is provided this will take precedent.
- [`check.indent-width`][check.indent-width]
- [`check.invalid-tab.indent-width`][check.invalid-tab.indent-width]


[check.indent-width]: ../settings.md#check_indent-width
[check.invalid-tab.indent-width]: ../settings.md#check_invalid-tab_indent-width

