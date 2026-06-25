module test_unary_following_arithmetic

    implicit none(type, external)

    interface operator(.plus.)
        module procedure plus
    end interface

    interface operator(.minus.)
        module procedure minus
    end interface

    interface operator(.negate.)
        module procedure negate
    end interface

contains

    ! user-defined binary function (math_expression)
    integer function plus(lhs, rhs)
        implicit none(type, external)
        integer, intent(in) :: lhs, rhs
        plus = lhs + rhs
    end function plus

    ! user-defined binary function (math_expression)
    integer function minus(lhs, rhs)
        implicit none(type, external)
        integer, intent(in) :: lhs, rhs
        minus = lhs - rhs
    end function minus

    ! user-defined unary function (unary_expression)
    integer function negate(i)
        implicit none(type, external)
        integer, intent(in) :: i
        negate = -i
    end function negate

end module test_unary_following_arithmetic

program main
    use test_unary_following_arithmetic

    implicit none(type, external)

    integer :: i

    ! All ok.
    i = 2 + 3
    i = 2 - 3
    i = 2 * 3
    i = 2 / 3
    i = -2
    i = +3
    i = 2 .plus. 3
    i = .negate.3
    i = 3 + .negate.2  ! User-defined operators are ok.
    i = 3 .minus. .negate.2  ! User-defined operators are ok.
    i = 10 ** 2
    i = 10 ** (+2)
    i = 10 ** (-2)
    i = 10 ** (-2) * 3
    i = 10 ** (-+3) / 2
    i = .negate.10 * 2 + 3
    i = 10 ** (-2) &
            &* 3
    i = 10 ** (-&
            &2) * 3
    i = 10 ** &
            &(-2) * 3
    i = 10 &
            &** (-2) * 3
    i = 10 ** (-2) * &
            &3
    i = 10 ** (-2) * 3 ! some comment.

    ! All fail.
    i = 2 + +3
    i = 2 - - 3
    i = 2 * -3
    i = 2 / -(3)
    i = 10 ** +3
    i = 10 ** -2
    i = 10 ** -2 * 3
    i = 10 ** -(2) / 3
    i = 10 &
            &** -2 * 3
    i = 10 ** &
            &-2
    i = 10 ** -&
            &2
    i = 10 ** -2 ! some comment.
    i = 10   **    -      2
    i = 10   **    -      2  *   3
    i = 10 ** -2 * 3 ** -7

end program main
