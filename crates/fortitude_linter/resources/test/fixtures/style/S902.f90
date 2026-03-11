module m
    implicit none (type, external)
    private

    type :: vector
        integer :: x, y, z
    contains
        procedure :: s5_bound
        procedure :: f5_bound
        procedure :: s6_bound
        procedure :: f6_bound
    end type vector

    ! Never raise in interfaces
    interface
        subroutine s4_interface(a, b, c, d)
            implicit none (type, external)
            integer, intent(in) :: a, b, c, d
        end subroutine s4_interface

        integer function f4_interface(a, b, c, d)
            implicit none (type, external)
            integer, intent(in) :: a, b, c, d
        end function f4_interface

        subroutine s5_interface(a, b, c, d, e)
            implicit none (type, external)
            integer, intent(in) :: a, b, c, d, e
        end subroutine s5_interface

        integer function f5_interface(a, b, c, d, e)
            implicit none (type, external)
            integer, intent(in) :: a, b, c, d, e
        end function f5_interface

    end interface

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
    subroutine s4(a, b, c, d)
        integer, intent(in) :: a, b, c, d
        print *, a, b, c, d
    end subroutine s4

    ! Should not raise
    integer function f4(a, b, c, d) result(f)
        integer, intent(in) :: a, b, c, d
        f = a + b + c + d
    end function f4

    ! Should raise
    subroutine s5(a, b, c, d, e)
        integer, intent(in) :: a, b, c, d, e
        print *, a, b, c, d, e
    end subroutine s5

    ! Should raise
    integer function f5(a, b, c, d, e) result(f)
        integer, intent(in) :: a, b, c, d, e
        f = a + b + c + d + e
    end function f5

    ! Should not raise, since the first argument is 'this' and is likely a type-bound procedure
    subroutine s5_bound(tHiS, a, b, c, d)
        class(vector), intent(in) :: tHiS
        integer, intent(in) :: a, b, c, d
        print *, tHiS%x, tHiS%y, tHiS%z, a, b, c, d
    end subroutine s5_bound

    ! Should not raise, since the first argument is 'self' and is likely a type-bound procedure
    integer function f5_bound(Self, a, b, c, d) result(f)
        class(vector), intent(in) :: Self
        integer, intent(in) :: a, b, c, d
        f = Self%x + Self%y + Self%z + a + b + c + d
    end function f5_bound

    ! Should raise
    subroutine s6_bound(this, a, b, c, d, e)
        class(vector), intent(in) :: this
        integer, intent(in) :: a, b, c, d, e
        print *, this%x, this%y, this%z, a, b, c, d, e
    end subroutine s6_bound

    ! Should raise
    integer function f6_bound(self, a, b, c, d, e) result(f)
        class(vector), intent(in) :: self
        integer, intent(in) :: a, b, c, d, e
        f =  self%x + self%y + self%z + a + b + c + d + e
    end function f6_bound

end module m
