# shadowed-variable (C201)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for variables that shadow those in a higher scope.

## Why is this bad?
Shadowing variables can lead to confusion, as it can be unclear which
variable is being referenced in a given context. Shadowing may be
unintentional, which is a common source of bugs.

There are contexts in which shadowing may be acceptable. For instance, if a
procedure dummy argument has the same name as a module variable, this
indicates that the programmer likely intends to use the dummy argument to
reference the module variable.  If you want to also capture dummy arguments,
the setting `check.shadowed-variables.strict` can be used to toggle this
behaviour.

It is also very common to reuse loop variables such as `i`, `j`, and `k` or
error flags such as `err` in different scopes. By default, Fortitude will
ignore variables with common names. The setting
`check.shadowed-variables.allow` can be used to modify which variables are
allowed to be shadowed.

## Examples

```f90
module my_mod

  implicit none (type, external)
  private

  real, allocatable :: x(:)

contains

  subroutine initialise(n)
    integer, intent(in) :: n
    real, allocatable :: x(:)  ! This is a bug!

    allocate(x(n))

  end subroutine initialise

  subroutine selection_sort(arr)
    real, intent(inout) :: arr(:)
    integer :: i

    do i = 1, size(arr)
      call helper(arr(i:size(arr)))
    end do

  contains

    !! Finds minimum element of an array, swaps it with the start
    subroutine helper(arr)
      real, intent(inout) :: arr(:) ! Allowed: is a dummy arg
      real :: min_val
      integer :: min_idx
      integer :: i  ! Allowed: is a common loop variable

      min_val = arr(1)
      min_idx = 1
      do i = 1, size(arr)
        if (arr(i) < min_val) then
          min_val = arr(i)
          min_idx = i
        end if
      end do
      arr(min_idx) = arr(1)
      arr(1) = min_val

    end subroutine helper

  end subroutine selection_sort

end module my_mod
```

## Options
- [`check.shadowed-variables.allow`][check.shadowed-variables.allow]
- [`check.shadowed-variables.strict`][check.shadowed-variables.strict]


[check.shadowed-variables.allow]: ../settings.md#check_shadowed-variables_allow
[check.shadowed-variables.strict]: ../settings.md#check_shadowed-variables_strict

