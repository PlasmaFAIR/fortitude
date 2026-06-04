# superfluous-while-true (S301)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for `while` statements that only evaluate the boolean literal `.true.` in `do`
statements.

## Why is this bad?
The statement loop is superfluous, as it will always execute.

## Example
```f90
do while (.true.)
  x = x + 1
  if (x > 10) exit
end do
```

Use instead:
```f90
do
  x = x + 1
  if (x > 10) exit
end do
```
