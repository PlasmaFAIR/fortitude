program test
  integer :: a(3) = (/1, 2, 3/)
  integer :: b(3) = (/ &
       1, &
       2, &
       3 &
       /)
  if (.true.) a = (/4, 5, 6/)
  b(1:3) = (/ &
       4, &
       5, &
       6 &
       /)
end program test
