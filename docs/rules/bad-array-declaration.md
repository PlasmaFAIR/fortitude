# bad-array-declaration (S263)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for variable array declarations that either do or do not use the
`dimension` attribute.

## Why is this bad?
Array variables in Fortran can be declared using either the `dimension`
attribute, or with an "array-spec" (shape) in parentheses:

```f90
! With an attribute
integer, dimension(2) :: x
! With a shape in brackets
integer :: x(2)
```

The two forms are exactly equivalent, but some projects prefer to only use
form over the other for consistency.

## Options
- [`check.inconsistent-dimensions.prefer-attribute`][check.inconsistent-dimensions.prefer-attribute]


[check.inconsistent-dimensions.prefer-attribute]: ../settings.md#check_inconsistent-dimensions_prefer-attribute

