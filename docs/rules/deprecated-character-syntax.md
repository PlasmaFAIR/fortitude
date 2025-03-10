# deprecated-character-syntax (OB062)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Checks for outdated declarations of `character*N`

## Why is this bad?
The syntax `character*N` has been replaced by `character(len=N)` in modern
Fortran. Prefer the second form.