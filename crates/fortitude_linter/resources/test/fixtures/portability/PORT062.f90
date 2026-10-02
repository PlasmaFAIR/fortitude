program test_rand
  implicit none
  integer,parameter :: seed = 86456
  call srand(seed)
  print *, rand(), rand()
  print *, rand(seed)
end program test_rand
