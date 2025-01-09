program test
  use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64, qp => real128, int8, int16, int32, int64

  integer(int8), parameter :: i1 = 1_1
  integer(int16), parameter :: i2 = -1_2
  integer(int32), parameter :: i3 = 2_4
  integer(int64), parameter :: i4 = -2_8
  integer(int8), parameter :: i5 = 1_int8
  integer(int16), parameter :: i6 = -1_int16
  integer(int32), parameter :: i7 = 2_int32
  integer(int64), parameter :: i8 = -2_int64

  real(sp), parameter :: x1 = 1.234567_4
  real(dp), parameter :: x2 = 1.234567_dp
  real(dp), parameter :: x3 = 1.789d3
  real(dp), parameter :: x4 = 9.876_8
  real(sp), parameter :: x5 = 2.468e-1_sp
  real(qp), parameter :: x6 = 9.876_16
  real(qp), parameter :: x7 = 9.876e12_sp
  real(qp), parameter :: x8 = 9.876e-12_16
end program
