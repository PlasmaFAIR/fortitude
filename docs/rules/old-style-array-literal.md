# old-style-array-literal (MOD011)
Fix is always available.

This rule is turned on by default.

## What does it do?
Checks for old style array literals

## Why is this bad?
Fortran 2003 introduced a shorter syntax for array literals: `[...]`. While the
older style, `(/.../)`, is still valid, the F2003 style is shorter and easier to
match.