double precision function double(x)
  double precision, intent(in) :: x
  double = 2 * x
end function double

subroutine triple(x)
  double precision, intent(inout) :: x
  x = 3 * x
end subroutine triple

function complex_mul(x, y)
  double precision, intent(in) :: x
  double complex, intent(in) :: y
  double complex :: complex_mul
  complex_mul = x * y
end function complex_mul
