program test
  implicit none (external, type)
  ! these are ok
  real, dimension(1) :: x, y(2), z(3, 4)
  ! these are bad
  real :: a, b(5), c, d(2) = [1, 2]
  ! shouldn't complain if all arrays
  real :: e(2), f(2)
end program test
