integer function double(x)
  integer, intent(in) :: x
  double = 2 * x
end function double

subroutine triple(x)
  integer, intent(inout) :: x
  x = 3 * x
end subroutine triple
