module my_module
  implicit none
  interface
    integer function myfunc(x)
      implicit none
      integer, intent(in) :: x
    end function myfunc
  end interface
end module my_module

program my_program
  implicit none
  interface
    subroutine mysub(x)
      implicit none
      integer, intent(inout) :: x
    end subroutine mysub
  end interface
  write(*,*) 42
end program my_program
