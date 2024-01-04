! This function should trigger the linter on line 2
integer function double(x)
  integer, intent(in) :: x
  double = 2 * x
end function

module my_module

  implicit none
  private

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
end module

! This function should trigger the linter on line 28
subroutine triple(x)
  integer, intent(inout) :: x
  x = x * 3
end subroutine

interface
  integer function interface_func(x)
    integer, intent(in) :: x
  end function
end interface
