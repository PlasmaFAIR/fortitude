# non-standard-file-extension (F001)
This rule is turned on by default.

## What it does
Checks for use of standard file extensions.

## Why is it bad?
The standard file extensions for modern (free-form) Fortran are '.f90' or  '.F90'.
Forms that reference later Fortran standards such as '.f08' or '.F95' may be rejected
by some compilers and build tools.