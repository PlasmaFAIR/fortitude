# mixed-scalar-array-declaration (S262)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for variable declarations that mix declarations of both scalars and
arrays.

## Why is this bad?
Mixing declarations of scalars and arrays in one statement may mislead the
reader into thinking all variables are scalar. Prefer to declare arrays in
separate statements to scalars.

## Automatic Fix
The automatic fix for this moves the variable declaration to a new
statement, and is unsafe as it may clobber comments.

You can use `check.inconsistent-dimension.prefer-attribute` to control
whether to put a `dimension` attribute on the new declaration or not.

## Example
```f90
! only y is an array here
real :: x, y(2), z
```

Use instead:
```f90
real :: x, z
real :: y(2)
```

## Options
- [`check.inconsistent-dimensions.prefer-attribute`][check.inconsistent-dimensions.prefer-attribute]


[check.inconsistent-dimensions.prefer-attribute]: ../settings.md#check_inconsistent-dimensions_prefer-attribute

