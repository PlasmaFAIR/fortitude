program test
  use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64

  real(sp), parameter :: x1 = 1.234567_4
  real(dp), parameter :: x2 = 1.234567_dp
  real(dp), parameter :: x3 = 1.789d3
  real(dp), parameter :: x4 = 9.876_8
  real(sp), parameter :: x5 = 2.468_sp
end program
