module m
  implicit none (type, external)

contains

  subroutine s(x)
    integer, intent(inout) :: x
    integer :: i

    return  ! Error
    ! Ignore this comment

    x = 2
    do i=1,10
      if (i == 5) then
        cycle  ! Error
        ! Ignore this comment
        call some_routine()
        stop  ! Error
        print *, "hello world"
        return  ! Okay
      else if (i == 7) then
        error stop  ! Error
        if (i == 7) then
          write (*, *) "foo bar"
        end if 
        exit  ! Error
        x = 5
        exit  ! Okay
      end if
    end do


    return  ! Pointless, but okay
    ! Ignore this comment
  end subroutine s

end module m
