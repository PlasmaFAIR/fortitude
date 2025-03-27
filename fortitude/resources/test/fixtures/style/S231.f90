program p
    implicit none
    integer, parameter :: n = 10
    real :: A(n)
    integer :: i

    do i = 1, n
        A(i) = real(i)
    end do

    where (A > 5.0)
        A = A * 2.0
    eLseWHere
        A = A + 10.0
    end where

    where (A > 15.0)
        A = A + 3.0
    ELSE where
        A = A + 4.0
    end where

    if (A(1) == 11.0) then
      print *, "foo"
    eLsEIf (A(1) == 12.0) then
      print *, "bar"
    else if (A(2) == 12.0) then
      print *, "baz"
    else
      print *, "helloworld"
    end if

end program p
