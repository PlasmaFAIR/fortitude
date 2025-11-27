# superfluous-else-stop (S255)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for `else` statements with a `stop` statement in the preceeding
`if` block

## Why is this bad?
The `else` statement is not needed as the `stop` statement will always
exit the parent function. Removing the `else` will reduce nesting and make
the code more readable.

## Example
```f90
integer function max(a, b):
  integer, intent(in) :: a, b
  if (a > b) then
    max = a
    stop
  else
    max = b
  end if
end function max
```

Use instead:
```f90
integer function max(a, b):
  integer, intent(in) :: a, b
  if (a > b) then
    max = a
    stop
  end if
  max = b
end function foo
```
