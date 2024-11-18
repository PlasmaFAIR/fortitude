integer function foo(a, b, c)
  use mod
  integer :: a, c(2), f
  integer, dimension(:), intent(in) :: b
end function foo

subroutine bar(d, e, f)
  integer, pointer :: d
  integer, allocatable :: e(:, :)
  type(integer(kind=int64)), intent(inout) :: f
  integer :: g
end subroutine bar
