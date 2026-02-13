PROGRAM test
    implicit none
    integer x(1)
    x(1) = 0.0
    IF(x(1)) 30, 20, 10
10  PRINT *, 'first case'
    goto 40
20  PRINT *, 'second case'
    goto 40
30  PRINT *, 'third case'
40  CONTINUE

    if (x(1) < 0) then
      print*, "third case"
    else if (x(1) > 0) then
      print*, "first case"
    else
      print*, "second case"
    end if
END PROGRAM test
