#define MAX_LEN 64

program cases

  implicit none

contains

  subroutine char_input(a, b, c, d, e, f, g, h, i, j, k)
    use, intrinsic :: iso_c_binding, only: c_char

    ! assumed size
    character * ( * ), intent(in) :: a
    character*(*), intent(in) :: b, c*9

    ! sized with a number literal
    character*5 d, e
    CHARACTer  *  10 f
    chAracTer* 3 g, h*7

    ! should also work with macros
    ! FIXME the version with parentheses works, but the other
    ! raises a syntax error
    !character*MAX_LEN, intent(in) :: j
    character*(MAX_LEN), intent(in) :: i

    ! these are ok
    character(*, c_char) :: j
    character(len=*, kind=4) :: k

  end subroutine char_input

end program cases
