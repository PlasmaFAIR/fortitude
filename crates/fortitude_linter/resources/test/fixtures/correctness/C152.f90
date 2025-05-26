program p
  implicit none (type, external)
  logical, parameter :: condition = .false.
  integer :: i

  ! Inline if statements on a single line are permitted.
  if (condition) print *, "Hello world!"

  ! Should never trigger on `if then` blocks
  if (condition) then
    print *, "Hello"
  end if

  if (condition &
      .and. i > 0) &
  then
    print *, "Hello world!"
  end if

  if (condition &
      .and. i > 0) then; print *, &
      "Hello world!"; end if

  ! Raise an error if the body is on a new line.
  if (condition) &
      print *, "Hello"
      print *, "World!"

  ! Misleading semicolons: the second statement in the body should be placed
  ! after the `end if` in the fix.
  if (condition) &
      print *, "Hello"; print *, "World!"

  ! Permit body split across multiple lines.
  if (condition) print *, &
    "Hello world!"

  ! Permit multi-line conditions
  if (condition &
      .and. i > 0) print *, "Hello world!"

  if (condition &
      .and. i > 0) print *, &
                   "Hello world!"

  ! ... but not if the body starts on a new line
  if (condition &
      .and. i > 0) &
        print *, "Hello world!"

  ! Some cases might result in weird indentation.
  do i = 1, 3; if (i == 2) &
    print *, "foo"; end do

  ! We don't handle misleading semicolons here.
  if (condition) print *, "foo"; print *, "bar"; &
    print *, "baz"


end program p
