# deprecated-relational-operator (S051)
Fix is always available.

## What does it do?
Checks for deprecated relational operators

## Why is this bad?
Fortran 90 introduced the traditional symbols for relational operators: `>`,
`>=`, `<`, and so on. Prefer these over the deprecated forms `.gt.`, `.le.`, and
so on.