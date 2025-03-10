program cases

  implicit none

  integer, parameter :: MAX_LEN = 64

contains

  subroutine char_input(a, b, c, d, e, f, g, h, i, j, k, l, m, n)
    use, intrinsic :: iso_c_binding, only: c_char

    ! assumed size
    character * ( * ), intent(in) :: a
    character*(*), intent(in) :: b, c*9
    character*(:), allocatable :: d

    ! sized with a number literal
    character*5 e, f
    CHARACTer  *  10 g
    chAracTer* 3 h, i*7

    ! sized with an integer expression
    character*(MAX_LEN), intent(in) :: j
    character * (2* (MAX_LEN) ) k

    ! these are ok
    character(*, c_char) :: l
    character(len=*, kind=4) :: m

    ! this should raise a syntax error and otherwise be ignored
    ! allow(syntax-error)
    character*MAX_LEN n

  end subroutine char_input

end program cases
