module test_user_defined
implicit none

contains

  subroutine exit(x)
    integer, intent(in) :: x
  end subroutine exit

  subroutine test_exit()
    call exit(5)  ! should not trigger
  end subroutine test_exit
end module test_user_defined

program main
implicit none
integer :: ierr
ierr = 1
call exit  ! non-standard
call exit()  ! non-standard
call exit(1)  ! non-standard
call EXIT(ierr)  ! non-standard
call exit(status=2)  ! non-standard
call abort  ! non-standard
do
  exit  ! loop exit, should not trigger
end do
end program main
