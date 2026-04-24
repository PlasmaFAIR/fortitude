module my_module
  implicit none (type)
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
      implicit none (type)
      integer, intent(inout) :: x
    end subroutine mysub
  end interface
  write(*,*) 42
end program my_program
