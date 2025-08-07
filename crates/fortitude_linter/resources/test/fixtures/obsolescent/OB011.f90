subroutine s1()
  common /mydata/ i, j
  integer(4) :: i, j
  i = 1
end subroutine s1

function f()
  common /mydata/ x
  real(8) :: x
  real(8) :: f
  x = 0.5
  f = x
end function f

subroutine s2()
  common i, j ! unnamed common block
  integer :: i, j
  write (*, *) i, j
end subroutine s2

subroutine s3()
  common /c1/ i, j, k /c2/ x, y, z /c1/ p, q , r ! Combined names common block
  integer :: i, j, k, p, q, r
  real :: x, y, z
end subroutine s3
