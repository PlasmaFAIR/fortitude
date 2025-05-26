module my_module
  implicit none

  interface
    integer function interfunc(x)
      implicit none
      integer, intent(in) :: x
    end function interfunc
  end interface

contains
  integer function myfunc(x)
    integer, intent(in) :: x
    myfunc = x * 2
  end function myfunc
  subroutine mysub(x)
    integer, intent(inout) :: x
    x = x * 2
  end subroutine mysub
end module my_module

program my_program
  implicit none

  write(*,*) 42

contains
  integer function myfunc2(x)
    integer, intent(in) :: x
    myfunc2 = x * 2
  end function myfunc2
  subroutine mysub2(x)
    integer, intent(inout) :: x
    x = x * 2
  end subroutine mysub2
end program my_program
