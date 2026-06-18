program whole_array_indexing
  implicit none

  real, allocatable :: x(:)
  real, allocatable :: y(:, :)
  real :: z(10)
  type box
    real, allocatable :: values(:)
  end type box
  type(box) :: item

  x = x(:)
  y = y(:, :)
  item%values = item%values(:)

  z = z(1:)
  z = z(:5)
  z = z(::2)
  z = z(1:5)
  z = z(1)
end program whole_array_indexing
