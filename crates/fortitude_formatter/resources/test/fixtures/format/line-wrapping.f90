program test_wrap
    implicit none

    real :: x, y, z, result
    character(len=:), allocatable :: msg
    real, dimension(:), allocatable :: arr

    type :: my_type
        integer :: field1
        character(len=:), allocatable :: field2
        integer, dimension(:), allocatable :: field3
        real :: field4
    end type my_type

    type(my_type) :: obj

    !--------------------------------------------------------------
    ! 1. Long arithmetic with mixed operators
    !--------------------------------------------------------------
    x = 1.0 + 2.0 * 3.0 - 4.0 / 5.0 + 6.0**2 - 7.0*8.0 + 9.0/10.0 - 11.0**3 + 12.0*13.0 - 14.0/15.0 + 16.0**4

    !--------------------------------------------------------------
    ! 2. Deeply nested parentheses
    !--------------------------------------------------------------
    y = (((((((x + x) * (x - x)) / (x + x)) ** (x - x)) + (x * x)) - (x / (x + x))) ** (x - x)) + x

    !--------------------------------------------------------------
    ! 3. Long IF condition
    !--------------------------------------------------------------
    if (x > y .and. y < z .or. (x == z .and. y /= x) .or. x >= y .and. z <= x .or. (y > z .and. z < x .and. x == y)) then
        call dummy()
    end if

    !--------------------------------------------------------------
    ! 4. String literal concatenation
    !--------------------------------------------------------------
    msg = "This is a very long string literal that might exceed the line length limit in fixed form or free form" // &
          " and continues with more text to test whether the wrapping works correctly when concatenating multiple string pieces" // &
          " and still more to make it ridiculously long"

    !--------------------------------------------------------------
    ! 5. Array constructor with continuation
    !--------------------------------------------------------------
    arr = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, &
           11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0]

    !--------------------------------------------------------------
    ! 6. Function call with many arguments
    !--------------------------------------------------------------
    call long_procedure_with_many_arguments(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11, arg12, arg13, arg14, arg15)

    !--------------------------------------------------------------
    ! 7. Nested function calls
    !--------------------------------------------------------------
    z = func1(func2(func3(func4(x + y, y * z, z / x), x - y, y ** z), x + y, y * z), z / x, x - y, y + z)

    !--------------------------------------------------------------
    ! 8. Derived type initialization
    !--------------------------------------------------------------
    obj = my_type(field1=123456789, field2="a long string here", field3=[1,2,3,4,5,6,7,8,9,10], field4=func(x,y,z,x,y,z))

    !--------------------------------------------------------------
    ! 9. Assignment with long expression mixing everything
    !--------------------------------------------------------------
    result = (x*y + y/z - x**y) * (y + z - x*y) / (x + y*z - x/z + y*z - x**y + z - x + y*z - z)

    !--------------------------------------------------------------
    ! 10. SELECT CASE with long selector expression
    !--------------------------------------------------------------
    select case (x + y * (z + x - y) / (y*x - z) ** (x + 2.0))
    case (1)
        call handle_case1()
    case default
        call handle_default()
    end select

contains

    subroutine long_procedure_with_many_arguments(a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11,a12,a13,a14,a15)
        integer, intent(in), optional :: a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11,a12,a13,a14,a15
    end subroutine long_procedure_with_many_arguments

end program test_wrap
