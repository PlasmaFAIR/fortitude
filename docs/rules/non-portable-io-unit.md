# non-portable-io-unit (PORT001)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for the literals `5` or `6` as units in `read`/`write` statements.

## Why is this bad?
The Fortran standard does not specify numeric values for `stdin` or
`stdout`. Instead, use the named constants `input_unit` and `output_unit`
from the `iso_fortran_env` module.