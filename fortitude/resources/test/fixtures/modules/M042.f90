module mod1

contains
    subroutine test1()
    end subroutine test1
end module mod1

program main
    use :: mod1, only: test1

end program main
