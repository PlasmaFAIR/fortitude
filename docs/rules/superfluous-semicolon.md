# superfluous-semicolon (S081)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Catches a semicolon at the end of a line of code.

## Why is this bad?
Many languages use semicolons to denote the end of a statement, but in Fortran each
line of code is considered its own statement (unless it ends with a line
continuation character, `'&'`). Semicolons may be used to separate multiple
statements written on the same line, but a semicolon at the end of a line has no
effect.

A semicolon at the beginning of a statement similarly has no effect, nor do
multiple semicolons in sequence.