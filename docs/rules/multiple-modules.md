# multiple-modules (M041)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for multiple modules in one file

## Why is this bad?
Placing each module into its own file improves maintainability
by making each module easier to locate for developers, and also
making dependency generation in build systems easier.