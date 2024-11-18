program cases
contains
  subroutine char_input(a, b, c, d, e, f)
    character * ( * ), intent(in) :: a
    character*(*), intent(in) :: b
    character*(len=*), intent(in) :: c
    character*(3), intent(in) :: d
    character*(MAX_LEN), intent(in) :: e
    ! these are ok
    character(*, kind) :: f
    character(len=*, kind=4) :: g
  end subroutine char_input
end program cases
