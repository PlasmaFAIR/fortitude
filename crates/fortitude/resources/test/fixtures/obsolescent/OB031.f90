subroutine test()
    use, intrinsic :: iso_fortran_env, dp => real64
    real(kind=dp) :: x, y

    y = ASIN(x)
    y = DSIN(x)
end subroutine test

subroutine test1()
    use, intrinsic :: iso_fortran_env, dp => real64
    real(kind=dp) :: x, y

    y = ASIN(x) + DSIN(x)
end subroutine test1

subroutine test2()
    use, intrinsic :: iso_fortran_env, dp => real64
    real(kind=dp) :: x, y

    y = dsin(x) + dcos(x)
end subroutine test2
