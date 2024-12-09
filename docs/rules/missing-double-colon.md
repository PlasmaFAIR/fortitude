# missing-double-colon (S071)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for missing double-colon separator in variable declarations.

## Why is this bad?
The double-colon separator is required when declaring variables with
attributes, so for consistency, all variable declarations should use it.