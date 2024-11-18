module my_module
  implicit none
  interface
    integer function myfunc(x)
      integer, intent(in) :: x
    end function myfunc
  end interface
end module my_module

program my_program
  implicit none
  interface
    subroutine myfunc2(x)
      integer, intent(inout) :: x
    end subroutine myfunc2
  end interface
  write(*,*) 42
end program my_program
