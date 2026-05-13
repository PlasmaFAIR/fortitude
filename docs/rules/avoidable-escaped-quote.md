# avoidable-escaped-quote (S242)
Fix is always available.

## What it does
Checks for strings that include escaped quotes that can be removed if the
quote style is changed.

## Why is this bad?
It's preferable to avoid escaped quotes in strings. By changing the
outer quote style, you can avoid escaping inner quotes.

## Example
```f90
foo = 'bar''s'
```

Use instead:
```f90
foo = "bar's"
```
