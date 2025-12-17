# inconsistent-array-declaration (S261)
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

## Automatic Fix
The automatic fix for this moves the variable declaration to a new
statement, and is unsafe as it may clobber comments.

You can use `check.inconsistent-dimension.prefer-attribute` to control
whether to put a `dimension` attribute on the new declaration or not.

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

## Options
- [`check.inconsistent-dimensions.prefer-attribute`][check.inconsistent-dimensions.prefer-attribute]


[check.inconsistent-dimensions.prefer-attribute]: ../settings.md#check_inconsistent-dimensions_prefer-attribute

