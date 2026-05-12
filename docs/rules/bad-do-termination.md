# bad-do-termination (OB093)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for `do` loops that don't end with `end do` or `continue`.

## Why is this bad?
Until the Fortran 2018 standard, labelled `do` loops were allowed to end in
any executable "action" statement. This makes the code more bugprone and
challenging to understand.

## Example
```f90
    do 10 i = 1, 10
10  foo(i) = i
```

Use instead:
```f90
   do i = 1, 10
     foo(i) = i
   end do
```
