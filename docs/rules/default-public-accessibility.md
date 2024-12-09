# default-public-accessibility (M022)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks if the default accessibility in modules is set to `public`

## Why is this bad?
The `public` statement makes all entities (variables, types, procedures)
public by default. This decreases encapsulation and makes it more likely to
accidentally expose more than necessary. Public accessibility also makes
it harder to detect unused entities, which can often be indicative of
errors within the code.