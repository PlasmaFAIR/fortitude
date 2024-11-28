program test
  use, intrinsic :: iso_fortran_env, dp => real64

  integer, parameter :: sp = kind(0.0) ! Okay: Permissible in a kind statement

  real(dp), parameter :: a = 0.0 ! Okay: No loss of precision
  real(dp), parameter :: b = 17745.0 ! Okay: No loss of precision
  real(sp), parameter :: c = 1.0e10 ! Okay: No loss of precision, e exponent
  real(sp), parameter :: d = 2.0E10 ! Okay: No loss of precision, E exponent
  real(sp), parameter :: e = 1.23456 ! Bad: Loss of precision
  real(sp), parameter :: f = 1.23456e1 ! Bad: Loss of precision, e exponent
  real(sp), parameter :: g = 1.23456E1 ! Bad: Loss of precision, E exponent
  real(dp), parameter :: h = 1.23456_dp ! Okay: Kind suffix
  real(sp), parameter :: i = 1.23456_sp ! Okay: Loss of precision, but we're explicit
  real(dp), parameter :: j = 1.23456d1 ! Okay: Ignore d exponent
  real(dp), parameter :: k = 1.23456D3 ! Okay: Ignore D exponent

  real(dp) :: p, q, x, y, z

  x = sqrt(2.0) ! Bad: Loss of precision
  y = real(1.0, kind=dp) ! Okay: Type cast with no loss of precision
  z = real(1.0 + 1.0, kind=dp) ! Bad: Type cast from expression, possible l.o.p
  p = real(5.0, kind=dp) ! Okay: Type cast with no loss of precision
  q = real(1.23456, kind=dp) ! Bad: Type cast with loss of precision
end program test
