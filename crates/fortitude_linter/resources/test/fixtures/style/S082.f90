module m
  ;
end module m


program p;
  ;
  implicit none;
  integer :: i;
  integer :: j;
  i = 1;; j = 2;
  i = i + j; write (*, *) i;;
end program p;
