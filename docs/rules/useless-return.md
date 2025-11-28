# useless-return (S251)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for unnecessary `return` statements

## Why is this bad?
Unlike many other languages, Fortran's `return` statement is only used to
return early from procedures, and not to return values. If a `return`
statement is the last executable statement in a procedure, it can safely be
removed.

## Example
```f90
integer function capped_add(a, b)
  integer, intent(in) :: a, b
  if ((a + b) > 10) then
    capped_add = 10
    return
  end if
  capped_add = a + b
  return   ! This `return` statement does nothing
end function capped_add
```

Use instead:
```f90
integer function capped_add(a, b)
  integer, intent(in) :: a, b
  if ((a + b) > 10) then
    capped_add = 10
    return
  end if
  capped_add = a + b
end function capped_add
```
