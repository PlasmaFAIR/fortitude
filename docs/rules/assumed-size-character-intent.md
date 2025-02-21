# assumed-size-character-intent (C072)
This rule is turned on by default.

## What does it do?
Checks `character` dummy arguments have `intent(in)` only

## Why is this bad?
Character dummy arguments with an assumed size should only have `intent(in)`, as
this can cause data loss with `intent([in]out)`. For example:

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