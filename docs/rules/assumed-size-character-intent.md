# assumed-size-character-intent (C072)
This rule is turned on by default.

## What does it do?
Checks `character` dummy arguments with an assumed-size length have
`intent(in)` only.

## Why is this bad?
Character dummy arguments whose length is assumed size should only have
`intent(in)`, as this can cause data loss with `intent([in]out)`. For
example:

```f90
program example
  character(len=3) :: short_text
  call set_text(short_text)
  print*, short_text
contains
  subroutine set_text(text)
    character(*), intent(out) :: text
    text = "longer than 3 characters"
  end subroutine set_text
end program
```

Here, `short_text` will only contain the truncated "lon".

To handle dynamically setting `character` sizes, use `allocatable` instead:

```f90
program example
  character(len=:), allocatable :: allocatable_text
  call set_text(allocatable_text)
  print*, allocatable_text
contains
  subroutine set_text(text)
    character(len=:), allocatable, intent(out) :: text
    text = "longer than 3 characters"
  end subroutine set_text
end program
```

Allocatable dummy arguments were not introduced until Fortran 2003, so this
rule is deactivated when targeting earlier standards. When doing so, it is
recommended to always verify that the `character` dummy arguments have the
correct size to avoid data loss:

```f90
  ! Fortran 95 example
  subroutine set_text(text)
    character(len=*), intent(out) :: text
    if (len(text) < 12) stop 1
    text = "hello world!"
  end subroutine set_text
```

## User derived type IO procedures
The standard mandates assumed-size length with `intent(inout)` for the
`iomsg` argument of user defined IO procedures for derived types, although
it doesn't specify a minimum length. Unfortunately, Fortitude is currently
unable to detect this use. You can use [`allow` (suppression)
comments](https://fortitude.readthedocs.io/en/latest/linter/#error-suppression)
to disable this rule for those uses only.
