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
  call use_array(x(:))

  x(:) = x
  item%values(:) = item%values
  z = z(1:)
  z = z(:5)
  z = z(::2)
  z = z(1:5)
  z = z(1)
contains
  subroutine use_array(array)
    real, intent(in) :: array(:)
  end subroutine use_array
end program whole_array_indexing
