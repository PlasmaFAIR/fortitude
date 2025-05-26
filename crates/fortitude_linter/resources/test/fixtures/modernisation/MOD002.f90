program p

  integer, parameter :: dp = kind(0.d0) ! Okay: Permissible in a kind statement
  integer, parameter :: cdp = kind((0.0d0, 0.0d0)) ! Okay: Permissible in a kind statement

  real(dp), parameter :: a = 0.0 ! Okay: default real kind
  real(dp), parameter :: b = 1.34e-4 ! Okay: default real kind
  real(dp), parameter :: c = .25E1 ! Okay: default real kind
  real(dp), parameter :: d = 1.23456d1
  real(dp), parameter :: e = 1.23456D-32
  real(dp), parameter :: f = 3d11
  real(dp), parameter :: g = .23456D-45
  real(dp), parameter :: h = 23456.D21

  ! Okay if in a type cast
  print *, real(1.0d0), int(2.d1)
end program p
