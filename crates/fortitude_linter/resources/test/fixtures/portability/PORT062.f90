program test_rand
  implicit none
  integer,parameter :: seed = 86456
  call srand(seed)
  print *, rand(), rand(), rand(), rand()
  print *, rand(seed), rand(), rand(), rand()
end program test_rand
