! This function should trigger for missing enclosing module
integer function double(x)
  integer, intent(in) :: x
  double = 2 * x
end function

module my_module

  use, intrinsic :: iso_fortran_env, only: dp => real64

  implicit none
  private

  real(dp), parameter :: pi = 3.14159265358979_dp

  ! This function should raise an error for missing implicit none, one for using
  ! a number literal kind in the signature, and one for a number literal kind in the
  ! variable list.
  interface
    integer(8) function interface_func(x)
      integer(kind=8), intent(in) :: x
    end function
  end interface

contains

  ! Should not trigger linter
  integer function double(x)
    integer, intent(in) :: x
    double = 2 * x
  end function

  ! Should not trigger linter
  subroutine triple(x)
    integer, intent(inout) :: x
    x = x * 3
  end subroutine

  ! Should trigger for superfluous implicit none
  integer function quad(x)
    implicit none
    integer, intent(in) :: x
    quad = 4 * x
  end function

  ! Should trigger for use of 'double precision'
  double precision function double_prec(x)
    double precision, intent(in) :: x
    double_prec = 2 * x
  end function
end module

! This function should trigger for missing enclosing module
subroutine triple(x)
  integer, intent(inout) :: x
  x = x * 3
end subroutine

! Should trigger for missing implicit none
module implicit_module
  parameter(N = 1)
end module

! Should trigger for missing implicit none
program myprog
  write(*,*) "Hello world!"
end program
