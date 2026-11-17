# missing-blank-line-between-procedures (S107)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks that consecutive procedure definitions are separated by a blank line.

## Why is this bad?
A blank line between procedures makes it easier to distinguish where one
procedure ends and the next begins.

## Example
```f90
module my_module
  implicit none
contains
  subroutine foo()
    print *, "foo"
  end subroutine foo
  subroutine bar()
    print *, "bar"
  end subroutine bar
end module my_module
```

Use instead:
```f90
module my_module
  implicit none
contains
  subroutine foo()
    print *, "foo"
  end subroutine foo

  subroutine bar()
    print *, "bar"
  end subroutine bar
end module my_module
```
