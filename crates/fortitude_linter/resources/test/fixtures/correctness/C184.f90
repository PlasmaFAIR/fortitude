program test
    implicit none
    real :: x, y
    integer :: i

    x = 0.1
    y = 0.2
    i = 1

    if (x == y) error stop
    if (x /= y) error stop
    if (x .eq. y) error stop
    if (i == 1) error stop
    if (1.0 == 2.0) error stop
end program test
