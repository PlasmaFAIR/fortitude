program test
  use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64

  real(sp), parameter :: x1 = 1.234567
  real(dp), parameter :: x2 = 1.234567_dp
  real(dp), parameter :: x3 = 1.789d3 ! rule should ignore d exponentiation
  real(dp), parameter :: x4 = 9.876
  real(sp), parameter :: x5 = 2.468_sp
  real(sp), parameter :: x6 = 2.
  real(sp), parameter :: x7 = .0
  real(sp), parameter :: x8 = 1E2
  real(sp), parameter :: x9 = .1e2
  real(sp), parameter :: y1 = 1.E2
  real(sp), parameter :: y2 = 1.2e3
end program test
