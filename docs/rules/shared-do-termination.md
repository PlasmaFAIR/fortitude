# shared-do-termination (OB092)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for `do` loops that share termination labels.

## Why is this bad?
Labelled `do` loops that share statements to mark their end are particularly
confusing and bugprone, and were deleted in the Fortran 2018 standard.

## Example
```f90
    do 10 i = 1, 10
      do 10 j = 1, 10
        foo(j, i) = i * j
10  continue
```

Use instead:
```f90
   do i = 1, 10
     do j = 1, 10
       foo(j, i) = i * j
     end do
   end do
```
