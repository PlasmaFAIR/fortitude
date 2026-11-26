# compare-reals (C184)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for equality or inequality comparisons involving real values.

## Why is this bad?
Real numbers are stored with finite precision, so a value produced by
arithmetic may not compare equal to its mathematical result. Comparing real
values with `==`, `/=`, `.eq.` or `.ne.` can therefore give surprising
results.

## Example
```f90
if (x == y) error stop
```

Use an explicit tolerance instead:

```f90
if (abs(x - y) < tolerance) error stop
```

## References
- [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)
- Clerman, N. Spector, W., 2012, _Modern Fortran: Style and Usage_, Cambridge
  University Press, Rule 171, p. 250.
- [Ruff RUF069](https://docs.astral.sh/ruff/rules/float-equality-comparison/)
