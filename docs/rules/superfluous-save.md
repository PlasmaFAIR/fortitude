# superfluous-save (S281)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for unnecessary `save` statements and qualifiers

## Why is this bad?
Since Fortran 2008, module variables are implicitly saved. Save statements
and attributes can safely be removed.

## Example
```f90
module example
    integer, save :: a
end module example
```
or
```f90
module example
    save

    integer :: a
end module example
```

Use instead:
```f90
module example
    integer :: a
end module example
```
