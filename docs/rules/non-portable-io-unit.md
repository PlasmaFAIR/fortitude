# non-portable-io-unit (PORT001)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for certain literals as units in `read`/`write` statements.

## Why is this bad?
The Fortran standard does not specify numeric values for `stdin`, `stdout`, or
`stderr`, and although many compilers do "pre-connect" units `5`, `6`, and `0`,
respectively, some use other numbers. Instead, use the named constants `input_unit`,
`output_unit`, or `error_unit` from the `iso_fortran_env` module.

!!! note
    An `open` statement with one of these units is completely portable, it is just
    the use to mean `stdin`/`stdout`/`stderr` without an explicit `open` that is
    non-portable -- but see also [`magic-io-unit`](magic-io-unit.md) for why it's
    best to avoid literal integers as IO units altogether.

## Options
- [`check.portability.allow-cray-file-units`][check.portability.allow-cray-file-units]


[check.portability.allow-cray-file-units]: ../settings.md#check_portability_allow-cray-file-units

