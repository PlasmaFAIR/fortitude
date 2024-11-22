module mod_test
  implicit none
  interface
    subroutine sub
    end subroutine sub
  end interface
contains

  integer function foo(a, b, c, p)
    use mod
    integer :: a, c(2), f
    integer, dimension(:), intent(in) :: b
    procedure(sub) :: p         ! must not have `intent`
  end function foo

  subroutine bar(d, e, f)
    integer, pointer :: d
    integer, allocatable :: e(:, :)
    type(integer(kind=int64)), intent(inout) :: f
    integer :: g
  end subroutine bar
end module mod_test
