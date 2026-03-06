module m
    implicit none (type, external)
    private
contains
    ! Should not raise
    subroutine s0()
        print *, "hello"
    end subroutine s0

    ! Should not raise
    integer function f0() result(f)
        f = 0
    end function f0

    ! Should not raise
    subroutine s1(a)
        integer, intent(in) :: a
        print *, a
    end subroutine s1

    ! Should not raise
    integer function f1(a) result(f)
        integer, intent(in) :: a
        f = a
    end function f1

    ! Should not raise
    subroutine s5(a, b, c, d, e)
        integer, intent(in) :: a, b, c, d, e
        print *, a, b, c, d, e
    end subroutine s5

    ! Should not raise
    integer function f5(a, b, c, d, e) result(f)
        integer, intent(in) :: a, b, c, d, e
        f = a + b + c + d + e
    end function f5

    ! Should raise
    subroutine s6(a, b, c, d, e, f)
        integer, intent(in) :: a, b, c, d, e, f
        print *, a, b, c, d, e, f
    end subroutine s6

    ! Should raise
    integer function f6(a, b, c, d, e, f) result(g)
        integer, intent(in) :: a, b, c, d, e, f
        g = a + b + c + d + e + f
    end function f6
end module m
