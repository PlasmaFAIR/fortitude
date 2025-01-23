module test
    use :: iso_fortran_env

    type other
        integer :: hi
    end type other

    type mytype
        real :: val1
        integer :: val2

        real :: val3 = 4.0

        integer :: i1, i2, i3

        real(kind=real64), pointer :: pReal1

        integer, pointer :: pInt1 => null()

        integer, pointer :: pI1, pI2

        integer(kind=int32), pointer :: pI3 => null(), pI4

        type(other), pointer :: pVal4 => null()
    end type mytype
end module test