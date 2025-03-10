program cases

  implicit none

  integer, parameter :: MAX_LEN = 64

contains

  subroutine char_input(a, b, c, d, e, f, g, h, i, j, k, l, m)
    use, intrinsic :: iso_c_binding, only: c_char

    ! assumed size
    character * ( * ), intent(in) :: a
    character*(*), intent(in) :: b, c*9

    ! sized with a number literal
    character*5 d, e
    CHARACTer  *  10 f
    chAracTer* 3 g, h*7

    ! sized with an integer expression
    character*(MAX_LEN), intent(in) :: i
    character * (2* (MAX_LEN) ) j

    ! these are ok
    character(*, c_char) :: k
    character(len=*, kind=4) :: l

    ! this should raise a syntax error and otherwise be ignored
    ! allow(syntax-error)
    character*MAX_LEN m

  end subroutine char_input

end program cases
