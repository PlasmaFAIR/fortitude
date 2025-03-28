program p
    implicit none
    integer, parameter :: n = 10
    double precision :: A(n)
    double complex :: b
    DoublePrecision :: c
    DoubleComplex :: d
    integer :: i, j, k
    ! Ignore all of the following
    character(len=10) :: elsewhere, endif, endmodule, selectcase

    do i = 1, n
        A(i) = real(i)
    enddo

    where (A > 5.0)
        A = A * 2.0
    eLseWHere
        A = A + 10.0
    endwheRe

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

    if (A(1) == 11.0) then
      print *, "foo"
    EndIf

endprogram p
