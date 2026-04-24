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
  integer, parameter :: x = 2
  write(*,*) x
end program my_program

subroutine external_sub(x)
  implicit none
  integer, intent(in) :: x
  print*, x
end subroutine external_sub

module my_module_type
  implicit none (type)
contains
  integer function double(x)
    integer, intent(in) :: x
    double = 2 * x
  end function double
end module my_module_type

subroutine external_sub_type(x)
  implicit none (type)
  integer, intent(in) :: x
  print*, x
end subroutine external_sub_type
