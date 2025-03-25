module my_mod
    implicit none

contains
    ! This is good
    function my_func(b, c) result(a)
        integer :: b, c

        a = b+c
    end function my_func


    ! While this is technically correct, as a style rule we want the
    ! result() clause on here
    function length(b, c)
        real :: b, c

        length = sqrt( b**2 + c**2 )
    end function my_func
end module my_mod