# program-with-module (S212)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for programs and modules in one file

## Why is this bad?
Separating top-level constructs into their own files improves
maintainability by making each easier to locate for developers,
and also making dependency generation in build systems easier.
