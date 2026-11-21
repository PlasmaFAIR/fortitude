# sqrt-as-power (S331)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for square roots written as a power of one half, such as `x ** 0.5`.

## Why is this bad?
`sqrt(x)` states the intent directly, and is typically evaluated using a
dedicated (and often faster) square root instruction, while `x ** 0.5` may
be evaluated by a general-purpose power function.

## Automatic Fix
The automatic fix is unsafe, as `sqrt(x)` and `x ** 0.5` may not give
bit-identical results, and may differ in how they handle special values
such as negative zero.

## Example
```f90
y = x ** 0.5
```

Use instead:
```f90
y = sqrt(x)
```
