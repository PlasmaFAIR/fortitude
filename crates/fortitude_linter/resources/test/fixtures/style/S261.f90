program test
  implicit none (external, type)
  ! y and z are inconsistent with decl
  real, dimension(1) :: x, y(2), z(3, 4)
  ! these are ok
  real :: a, b(5), c(6, 7)
  real, dimension(1) :: alpha = [0], beta(2) = [1, 2] ! more complicated
end program test
