module test
  implicit none
  integer, parameter :: NUM_POINTS = 54
  integer :: A(221), B(4, 221), C(1:100), D(1, 2:3, 33:44, 5, NUM_POINTS)
  integer, dimension(57) :: E
  integer, dimension(57, 64) :: F
  integer, dimension(NUM_POINTS) :: G
  integer :: H(NUM_POINTS)
  integer, dimension(-NUM_POINTS:NUM_POINTS) :: I
  integer, dimension(0:NUM_POINTS) :: J
contains
  subroutine foo(L, M)
    integer, dimension(8:9, 10, 11:12), intent(in) :: L
    integer, intent(out) :: M(57)
  end subroutine foo
end module test
