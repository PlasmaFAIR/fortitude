program myprog
  implicit none
  integer :: a(3)
  a = [1, 2, 3] ! This should be unchanged
  call mysub( a) ! This should remove the space after the (
contains
  subroutine mysub( a) ! This should remove the space after the (
    implicit none
    integer, intent( in) :: a( 3) ! This should remove the space after both the brackets
    integer, dimension(2) :: b ! This should be unchanged
    integer, dimension(   2) :: c = [6, 7] ! This should remove the spaces after the (
    b = [ 4, & ! This should remove the space after the [
          5 ] ! This should remove the space before the ]
    write( *,* ) a, b, c ! This should remove the space before the ) and after the (
  end subroutine mysub
  subroutine myothersub( & ! This should be allowed for long parameter lists
    a &
  ) ! This should be unchanged
    implicit none
    integer :: a
    a = (1 + 1 &
    & ) ! This should be unchanged
  end subroutine myothersub
  subroutine emptyparantesessub( ) ! This should remove the space between the brackets
  end subroutine emptyparantesessub
end program myprog
