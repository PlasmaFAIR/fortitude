# labelled-do-loop (OB091)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for uses of the obsolescent labelled `do` statements.

## Why is this bad?
These statements were made completely redundant with the introduction of
construct names. Construct names are clearer and easier to understand, while
not allowing arbitrary `goto` statements and other confusing

The Fortran 2018 standard made these statements obsolescent,

## Example
```f90
    do 10 i = 1, 10
      foo(i) = i
10  continue
```

Use instead:
```f90
   do i = 1, 10
     foo(i) = i
   end do
```
