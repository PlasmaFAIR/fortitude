program test
  use, intrinsic :: iso_fortran_env, only: dp => real64
  implicit none
  real(dp) :: x, y, a, b

  ! Should be flagged
  y = x ** 0.5
  y = x**0.5_dp
  y = x ** 5e-1
  y = x ** 5.0d-1
  y = x ** (0.5)
  y = (a + b) ** 0.5
  y = 2.0 * x ** 0.5 + 1.0

  ! Should not be flagged
  y = sqrt(x)
  y = x ** 2
  y = x ** 2.0
  y = x ** 5
  y = x ** 0.25
  y = 0.5 ** x
  y = x ** (1 / 2)
  y = x * 0.5
end program test
