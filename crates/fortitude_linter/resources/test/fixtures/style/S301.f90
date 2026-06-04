program test_program_superfluous_while_true

    implicit none(external, type)

    integer :: x = 0
    logical :: some_logical = .true.

contains

    ! PASS 1 -> standard.
    subroutine pass_1()
        do
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine pass_1

    ! PASS 2 -> do loop name.
    subroutine pass_2()
        some_name: do
            x = x + 1
            if (x > 10) exit some_name
        end do some_name
    end subroutine pass_2

    ! PASS 3 -> comment.
    subroutine pass_3()
        do ! comment
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine pass_3

    ! PASS 4 -> with FALSE.
    subroutine pass_4()
        do while (.false.)
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine pass_4

    ! PASS 5 -> with variable.
    subroutine pass_5()
        do while (some_logical)
            x = x + 1
            if (x > 10) some_logical = .false.
        end do
    end subroutine pass_5

    ! PASS 6 -> with expression.
    subroutine pass_6()
        do while (x <= 10)
            x = x + 1
        end do
    end subroutine pass_6

    ! PASS 7 -> with TRUE and expression.
    subroutine pass_7()
        do while (.true. .and. x <= 10)
            x = x + 1
        end do
    end subroutine pass_7

    ! PASS 8 -> with variable equivalent to TRUE.
    subroutine pass_8()
        do while (some_logical .eqv. .true.)
            x = x + 1
            if (x > 10) some_logical = .false.
        end do
    end subroutine pass_8

    ! PASS 9 -> with parenthesised TRUE and expression.
    subroutine pass_9()
        do while ((.true.) .and. x <= 10)
            x = x + 1
        end do
    end subroutine pass_9

    ! PASS 10 -> with parenthesised TRUE at the end with expressions.
    subroutine pass_10()
        do while ((x <= 10) .and. x > 2 .and. (.true.))
            x = x + 1
        end do
    end subroutine pass_10

    ! FAIL 1 -> standard.
    subroutine fail_1()
        do while (.true.)
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine fail_1

    ! FAIL 2 -> case insensitive.
    subroutine fail_2()
        do while (.TRUE.)
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine fail_2

    ! FAIL 3 -> extra whitespace.
    subroutine fail_3()
        do while (  .true.  )
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine fail_3

    ! FAIL 4 -> do loop name.
    subroutine fail_4()
        a_name: do while (.true.)
            x = x + 1
            if (x > 10) exit
        end do a_name
    end subroutine fail_4

    ! FAIL 5 -> with extra parentheses.
    subroutine fail_5()
        do while ((.true.))
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine fail_5

    ! FAIL 6 -> with lots of extra parentheses.
    subroutine fail_6()
        do while (((((((.true.)))))))
            x = x + 1
            if (x > 10) exit
        end do
    end subroutine fail_6

    ! FAIL 7 -> line continuation in logical expression.
    subroutine fail_7()
        do while (&
                .true.&
                )
            x = x + 1
            if (x > 10) some_logical = .false.
        end do
    end subroutine fail_7

    ! FAIL 8 -> line continuation between "while" and logical expression.
    subroutine fail_8()
        do while &
                (.true.)
            x = x + 1
            if (x > 10) some_logical = .false.
        end do
    end subroutine fail_8

    ! ** auto-fix not currently implemented **
    ! FAIL X -> line continuation inbetween "do" and "while".
    ! subroutine fail_X()
    !    do &
    !            while (.true.)
    !        x = x + 1
    !        if (x > 10) exit
    !    end do
    !end subroutine fail_X

end program test_program_superfluous_while_true
