# non-portable-io-unit (PORT001)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for certain literals as units in `read`/`write` statements.

## Why is this bad?
The Fortran standard does not specify numeric values for `stdin`, `stdout`, or
`stderr`, and although many compilers do "pre-connect" units `5`, `6`, and `0`,
respectively, some use other numbers. Instead, use the named constants `input_unit`,
`output_unit`, or `error_unit` from the `iso_fortran_env` module.
