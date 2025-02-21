integer(8) function add_if(x, y, z)
  integer :: w
  integer(kind=2), intent(in) :: x
  integer(i32), intent(in) :: y
  logical(kind=4), intent(in) :: z

  if (x) then
    add_if = x + y
  else
    add_if = x
  end if
end function add_if

subroutine complex_mul(x, y)
  real(8), intent(in) :: x
  complex(4), intent(inout) :: y
  real :: z = 0.5
  y = y * x
end subroutine complex_mul

complex(real64) function complex_add(x, y)
  real(real64), intent(in) :: x
  complex(kind=4), intent(in) :: y
  complex_add = y + x
end function complex_add
