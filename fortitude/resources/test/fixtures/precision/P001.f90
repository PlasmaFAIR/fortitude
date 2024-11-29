program test
  use, intrinsic :: iso_fortran_env, dp => real64

  integer, parameter :: sp = kind(0.0) ! Okay: Permissible in a kind statement
  integer, parameter :: csp = kind((0.0, 0.0)) ! Okay: Permissible in a kind statement

  real(dp), parameter :: a = 0.0 ! Okay: No loss of precision
  real(dp), parameter :: b = -17745.0 ! Okay: No loss of precision
  real(dp), parameter :: c = .25 ! Okay: No loss of precision
  real(dp), parameter :: d = 0.000244140625 ! Okay: No loss of precision (=2^-12)
  real(sp), parameter :: e = 1.0e10 ! Okay: No loss of precision, e exponent
  real(sp), parameter :: f = -2E10 ! Okay: No loss of precision, E exponent
  real(sp), parameter :: g = 1.23456 ! Bad: Loss of precision
  real(sp), parameter :: h = 1.23456e1 ! Bad: Loss of precision, e exponent
  real(sp), parameter :: i = 1.23456E1 ! Bad: Loss of precision, E exponent
  real(dp), parameter :: j = -1.23456_dp ! Okay: Kind suffix
  real(sp), parameter :: k = 1.23456_sp ! Okay: Loss of precision, but we're explicit
  real(dp), parameter :: l = 1.23456d1 ! Okay: Ignore d exponent
  real(dp), parameter :: m = 1.23456D3 ! Okay: Ignore D exponent
  real(dp), parameter :: n = 2e39 ! Bad: Magnitude doesn't fit in single precision
  real(dp), parameter :: o = -(+(-(((3.141592654))))) ! Bad: Loss of precision, horrible declaration
  
  complex(dp), parameter :: ca = (0.0, 0.0) ! Okay: No loss of precision
  complex(dp), parameter :: cb = (-17745.0, 16429.0) ! Okay: No loss of precision
  complex(dp), parameter :: cc = (.25, -0.5) ! Okay: No loss of precision
  complex(dp), parameter :: cd = (0.000244140625, 0.0) ! Okay: No loss of precision (=2^-12)
  complex(sp), parameter :: ce = (1.0e10, 2.0e5) ! Okay: No loss of precision, e exponent
  complex(sp), parameter :: cf = (2E10, -4.0e-5) ! Okay: No loss of precision, E exponent
  complex(sp), parameter :: cg = (1.23456, -0.0) ! Bad: Loss of precision
  complex(sp), parameter :: ch = (0.0, 1.23456e1) ! Bad: Loss of precision, e exponent
  complex(sp), parameter :: ci = (1.23456E1, 0.0) ! Bad: Loss of precision, E exponent
  complex(dp), parameter :: cj = (-1.23456_dp, 0.2_dp) ! Okay: Kind suffix
  complex(sp), parameter :: ck = (1.23456_sp, 3.14159265_sp)! Okay: Loss of precision, but we're explicit
  complex(dp), parameter :: cl = (1.23456d1, 37d2) ! Okay: Ignore d exponent
  complex(dp), parameter :: cm = (1.23456D3, 37D2) ! Okay: Ignore D exponent
  complex(dp), parameter :: cn = (2e39, 0e0) ! Bad: Magnitude doesn't fit in single precision
  real(dp), parameter :: co = -(+(-(-3.141592654, +5.182647))) ! Bad: Loss of precision, horrible declaration

  real(dp) :: p, q, x, y, z
  complex(dp) :: cp, cq, cx, cy, cz

  x = sqrt(2.0) ! Bad: Loss of precision
  y = real(1.0, kind=dp) ! Okay: Type cast with no loss of precision
  z = real(1.0 + 1.0, kind=dp) ! Bad: Type cast from expression, possible l.o.p
  p = real(-5.0, kind=dp) ! Okay: Type cast with no loss of precision
  q = real(1.23456, kind=dp) ! Bad: Type cast with loss of precision
  
  cx = sqrt((2.0, 0.0)) ! Bad: Loss of precision
  cy = cmplx((1.0, 1.0), kind=dp) ! Okay: Type cast with no loss of precision
  cz = cmplx((1.0, 0.0) + (0.0, 1.0), kind=dp) ! Bad: Type cast from expression, possible l.o.p
  cp = cmplx((5.0, -0.0625), kind=dp) ! Okay: Type cast with no loss of precision
  cq = cmplx((-1.23456, 3.141292654), kind=dp) ! Bad: Type cast with loss of precision
end program test
