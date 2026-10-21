module test_user_defined
implicit none

contains

  subroutine system(x)
    integer, intent(in) :: x
  end subroutine system

  subroutine test_system()
    call system(5)  ! should not trigger
  end subroutine test_system
end module test_user_defined

program main
implicit none
integer :: status
call system("dir")  ! non-standard
call execute_command_line("dir")
status = system("ls") ! non-standard, can't auto-fix
end program main
