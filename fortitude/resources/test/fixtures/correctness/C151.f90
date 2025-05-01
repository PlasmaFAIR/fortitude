program p
  implicit none (type, external)
  logical, parameter :: condition = .false.

  if (condition) print *, "Hello"; print *, "World!"

  if (condition) print * , &
    "Hello"; print *, "World!"

  ! The following is bad practice, but shouldn't trigger
  if (condition) then
    print *, "Hello"
  end if; print *, "World!"

  ! The following are also allowed
  if (condition) print *, "Hello";

  if (condition) print *, "Hello";
  print *, "World!"

  if (condition) print *, &
    "Hello";
  print *, "World!"

  ! Comments are fine
  if (condition) print *, "Hello"; ! comment

  if (condition) print *, &
    "Hello"; ! comment
end program p
