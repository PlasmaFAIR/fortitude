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

  ! For multiple inline if statements, each should be moved to its own line.
  ! To confirm that the fixes work when combined, see the test
  ! `c151_fix_multiple_inline_if` in `rules/correctness/mod.rs`
  if (condition) print *, "foo"; if(.true.) print *, "bar"; if(.false.) print *, "baz";
end program p
