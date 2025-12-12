# out-of-line-attribute (MOD041)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for variable attributes which are specified separately to the
variable declaration.

## Why is this bad?
Using separate attribute specification statements (or "out-of-line
attributes") makes the code harder to read by splitting up the important
information about a variable. Instead, give attributes in-line with the
variable declaration. This way, readers only need to look in one place.

## Example
```f90
integer :: nx
real :: x_grid
parameter (nx = 42)
dimension x_grid(nx)
```

Use instead:
```f90
integer, parameter :: nx = 42
real, dimension(nx) :: x_grid
```
