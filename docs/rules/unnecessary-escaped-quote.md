# unnecessary-escaped-quote (S243)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for strings that include unnecessarily escaped quotes.

## Why is this bad?
If a string contains an escaped quote that doesn't match the quote
character used for the string, it's unnecessary and can be removed.

## Example
```f90
foo = "bar''s"
```

Use instead:
```f90
foo = "bar's"
```
