module mod1

contains
    subroutine test1()
    end subroutine test1
end module mod1


module mod2
    use :: mod1, only: test1

contains
    subroutine test2()
    end subroutine test2
end module mod2
