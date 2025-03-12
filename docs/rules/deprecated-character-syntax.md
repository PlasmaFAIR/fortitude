# deprecated-character-syntax (OB061)
Fix is always available.

This rule is turned on by default.

## What does it do?
Checks for outdated declarations of `character*N`, 'character*(*)',
`character*(:)`, and 'character*(integer-expression)'.

## Why is this bad?
The syntax `character*N` has been replaced by `character(len=N)` in modern
Fortran. Prefer the second form.