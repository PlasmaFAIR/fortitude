# deprecated-relational-operator (MOD021)
Fix is always available.

This rule is turned on by default.

## What does it do?
Checks for deprecated relational operators

## Why is this bad?
Fortran 90 introduced the traditional symbols for relational operators: `>`,
`>=`, `<`, and so on. Prefer these over the deprecated forms `.gt.`, `.le.`, and
so on.
