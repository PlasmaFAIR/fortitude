program test
  if (0 .gt. 1) error stop
  if (1 .le. 0) error stop
  if (a.eq.b.and.a.ne.b) error stop
  if (1 == 2) error stop  ! OK
  if (2 /= 2) error stop  ! OK
end program test
