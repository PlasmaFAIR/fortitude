! This function should trigger for missing enclosing module
integer function double(x)
  integer, intent(in) :: x
  double = 2 * x
end function

subroutine print_val(y)
  real, intent(in) :: y
  write (*,*) y
end subroutine print_val

module my_module

  ! Should not raise
  use, intrinsic :: iso_fortran_env, only: dp => real64

  ! Should raise for lack of only clause
  use, intrinsic :: iso_c_binding

  implicit none
  private

  ! Should not raise error
  real(dp), parameter :: pi = 3.14159265358979_dp

  complex(dp), parameter :: imag = (0.0_8, 1.0_8)

  ! Should raise errors for missing suffix
  real(dp), parameter :: pi_32 = 3.14159265358979
  real(dp), parameter :: pi_short = 3.1415

  ! Should raise syntax error
  real(dp), parameter :: mistake = 2e

  ! Should not raise error for maximum line length
  character(*), parameter :: long_string = "https://verylongurl.com/page/another_page/yet_another_page/wow"

  ! Should report error
  !int, parameter :: bad_type = 1

  ! Should not raise error for maximum line length
  character(*), parameter :: long_comment = "short string" ! The origins of this string date back to the 18th century...

  ! Should raise error for non-standard 'type*N'
  logical*4, parameter :: true = .true.

  ! Should raise error for non-standard length
  character*20 :: foo

  ! TODO should raise error for outdated 'character*(*)'
  character*(*), parameter :: hello = "hello world"

  interface
    ! This function should raise an error for missing implicit none, one for using
    ! a number literal kind in the signature, and one for a number literal kind in the
    ! variable list.
    integer(8) function interface_func(x)
      integer(kind=8), intent(in) :: x
    end function

    ! This function shouldn't raise anything
    real function interface_func2(x)
      implicit none
      real, intent(in) :: x
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
  implicit integer(A)
  parameter(N = 1)
end module

! Should trigger for missing implicit none
program myprog
  write(*,*) "Hello world!"
end program
