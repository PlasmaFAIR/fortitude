#define SOME_MACRO 1 \
                   2
program t
    implicit none
    real :: A

    ! Just a comment \
    A = 2.0
    print *, A

    ! this is fine \
    ! if this is a comment
end program t
