# trailing-backslash (B011)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks if a backslash is the last character on a line

## Why is this bad?
When compilers use the C preprocessor to pre-process Fortran files
the \ character is treated as a line continuation character by the C preprocessor,
potentially causing lines to be merged into one.

## Example
When this Fortran program is passed through the C preprocessor,
```f90
program t
    implicit none
    real :: A

    ! Just a comment \
    A = 2.0
    print *, A
 end
```
it will end up with the variable assignment A placed onto the comment line,
```f90
program t
   implicit none
   real :: A

   ! Just a comment    A = 2.0

   print *, A
end
```
which causes the assignment to not be compiled.