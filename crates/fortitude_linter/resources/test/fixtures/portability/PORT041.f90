program test_pass_program

    integer :: y = add_one(5)
    integer :: z = 10

    if (y == 6) stop  ! all OK.

    do i=1,10
        if (i > y) stop  ! all OK.
    end do

    call minus_one(z)

    stop  ! all OK.

contains

    integer function add_one(value)
        integer, intent(in) :: value
        add_one = value + 1
        return  ! all OK.
    end function add_one

    subroutine minus_one(value)
        integer, intent(inout) :: value
        value = value - 1
        return  ! all OK.
    end subroutine minus_one

end program test_pass_program


module test_pass_module
contains

    SUBROUTINE multiply_by_two(value)
        integer, intent(inout) :: value
        value = value * 2
        return !  all OK.
    end subroutine multiply_by_two

end module test_pass_module

! ****************************************** !

program test_fail

    integer :: y = add_one(5)
    integer :: z = 10

    if (y == 6) return  ! not OK.

    do i=1,10
        if (i > y) return  ! not OK.
    end do

    call minus_one(z)

    return  ! not OK.

contains

    integer function add_one(inp)
        integer, intent(in) :: inp
        add_one = inp + 1
        return  ! all OK.
    end function add_one

    subroutine minus_one(value)
        integer, intent(inout) :: value
        value = value - 1
        return  ! all OK.
    end subroutine minus_one

end program test_fail
