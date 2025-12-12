# inconsistent-dimension-declaration (S261)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for variable declarations that have both the `dimension` attribute
and an inline array specification.

## Why is this bad?
Having both methods of declaring an array in one statement may be confusing
for the reader who may expect that all variables in the declaration have the
same shape as given by the `dimension` attribute. Prefer to declare
variables with different shapes to the `dimension` attribute on different
lines.

## Example
```f90
! y and z are inconsistent with the `dimension` attribute
real, dimension(5) :: x, y(2), z(3, 4)
```

Use instead:
```f90
real, dimension(5) :: x
real :: y(2)
real :: z(3, 4)
```
