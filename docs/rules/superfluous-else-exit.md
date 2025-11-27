# superfluous-else-exit (S254)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for `else` statements with a `exit` statement in the preceeding
`if` block

## Why is this bad?
The `else` statement is not needed as the `exit` statement will always
exit the enclosing loop. Removing the `else` will reduce nesting and make
the code more readable.

## Example
```f90
integer function foo(a, b):
  integer, intent(in) :: a, b
  integer :: i
  do i = 1, a
    if (b > i) then
      exit
    else
      foo = b
    end if
  end do
end function foo
```

Use instead:
```f90
integer function foo(a, b):
  integer, intent(in) :: a, b
  integer :: i
  do i = 1, a
    if (b > i) then
      exit
    end if
    foo = b
  end do
end function foo
```
