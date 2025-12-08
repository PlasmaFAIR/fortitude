program test
  implicit none
  integer :: X
  real :: y
  dimension y(2)
  parameter(x = 1, Y=[2, 3])

  block
    integer :: x
    allocatable x(:)
  end block
end program test
