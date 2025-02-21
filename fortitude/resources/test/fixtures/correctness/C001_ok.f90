module my_module
  implicit none
contains
  integer function double(x)
    integer, intent(in) :: x
    double = 2 * x
  end function double
end module my_module

program my_program
  implicit none
  integer, paramter :: x = 2
  write(*,*) x
end program my_program
