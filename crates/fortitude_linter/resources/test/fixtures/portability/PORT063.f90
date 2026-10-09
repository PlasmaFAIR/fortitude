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
call exit(123456)  ! non-standard, 6 digits: no fix below F2008
call EXIT(ierr)  ! non-standard
call exit(status=2)  ! non-standard
call abort  ! non-standard
call abort(1)  ! non-standard
call abort(123456)  ! non-standard
call abort("message")  ! non-standard
call abort(ierr)  ! non-standard
call abort(1, 2)  ! non-standard, more than one argument
do
  exit  ! loop exit, should not trigger
end do
end program main
