# deprecated-character-syntax (OB061)
This rule is turned on by default.

## What does it do?
Checks for outdated declarations of `character*N`,
'character*(*)' and 'character*(expression)'.

## Why is this bad?
The syntax `character*N` has been replaced by `character(len=N)` in modern
Fortran. Prefer the second form.