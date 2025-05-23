integer*8 function add_if(x, y, z)
  integer(kind=2), intent(in) :: x
  integer *4, intent(in) :: y
  logical*   4, intent(in) :: z
  real    * &
       8 :: t

  if (x == 2) then
    add_if = x + y
  else
    add_if = x
  end if
end function add_if

subroutine complex_mul(x, real)
  real * 4, intent(in) :: x
  complex  *  8, intent(inout) :: real
  ! This would be a false positive with purely regexp based linting
  real = real * 8
end subroutine complex_mul
