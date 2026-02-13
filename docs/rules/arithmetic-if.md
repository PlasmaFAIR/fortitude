# arithmetic-if (OB081)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for arithmetic `if` statements.

## Why is this bad?
The arithmetic `if` statement is used to jump between one of three statement
labels depending on whether the condition is below, above, or equal to
zero. However, this is incompatible with the IEEE 754 standard on floating
point numbers (due to the comparison between `real`s), and the use of
statment labels can hinder optimisation, as well as making the code harder
to read and maintain.

## Example
```f90
    IF(x(1)) 10, 20, 30
10  PRINT *, 'first case'
    GOTO 40
20  PRINT *, 'second case'
    GOTO 40
30  PRINT *, 'third case'
40  CONTINUE
```

Use instead:
```f90
if (x(1) < 0) then
  print*, "first case"
else if (x(1) > 0) then
  print*, "third case"
else
  print*, "second case"
end if
```
