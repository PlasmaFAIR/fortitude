module my_module
  parameter(N = 1)
end module my_module

module safe_fix
  use, intrinsic :: iso_fortran_env, only: int32
  integer(int32) :: foo
contains
  integer function double(x)
    integer, intent(in) :: x
    double = 2 * x
  end function double
end module safe_fix

program my_program
  use, intrinsic :: iso_fortran_env, only: int32
  ! Fix should be applied after next line
  use safe_fix
  ! Fix should be applied before this line
  write(*,*) 42
end program my_program

subroutine external_sub(x)
  print*, x
end subroutine external_sub
