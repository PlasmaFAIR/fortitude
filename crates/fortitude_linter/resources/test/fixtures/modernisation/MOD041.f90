program test
  implicit none
  integer :: X
  real :: y
  dimension y(2)
  parameter(x = 1, Y=[2, 3])

  block
    integer :: x, y
    allocatable x
    allocatable y(:)
  end block

  block
    integer :: a, b, c
    parameter(a=1, b=2)
  end block

  block
    integer :: x
    allocatable x (:, :)
  end block

  block
    integer :: x
    allocatable x
    dimension x(:, :)
  end block
end program test
