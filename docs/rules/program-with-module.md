# program-with-module (S212)
## What it does
Checks for programs and modules in one file

## Why is this bad?
Separating top-level constructs into their own files improves
maintainability by making each easier to locate for developers,
and also making dependency generation in build systems easier.
