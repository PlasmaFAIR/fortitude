# unreachable-statement (C191)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for `return`, `exit`, `cycle`, and `stop` statements that result in
unreachable code.

## Why is this bad?
Unreachable code can never be executed, and is almost certainly a mistake.

## Example
```f90
subroutine example(x)
  integer, intent(inout) :: x
  x = x + 1
  return
  print *, x  ! This statement is unreachable
end subroutine example
```
