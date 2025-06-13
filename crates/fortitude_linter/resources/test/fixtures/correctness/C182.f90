program p
  implicit none
  real(8), allocatable :: x(:), y(:)
  integer :: status

  ! no stat params
  allocate (x(10), y(10))
  deallocate (x, y)

  ! stat params, but one each
  allocate (x(10), stat=status)
  allocate (y(10), stat=status)
  deallocate (x, stat=status)
  deallocate (y, stat=status)

  ! stat params, combined in one statement
  allocate (x(10), y(10), stat=status)
  deallocate (x, y, stat=status)

end program p
