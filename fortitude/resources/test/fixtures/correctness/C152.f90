program p
  implicit none (type, external)
  logical, parameter :: condition = .false.
  integer :: i

  if (condition) &
      print *, "Hello"
      print *, "World!"

  if (condition) &
      print *, "Hello"; print *, "World!"

  if (condition) print *, &
    "Hello world!"

  if (condition) print *, &
     & "Hello"


  ! Inline if statements on a single line are permitted.
  if (condition) print *, "Hello world!"

  ! We don't handle misleading semicolons here.
  if (condition) print *, "foo"; print *, "bar"; &
    print *, "baz"

  ! Some cases might result in weird indentation.
  do i = 1, 3; if (i == 2) &
    print *, "foo"; end do
end program p
