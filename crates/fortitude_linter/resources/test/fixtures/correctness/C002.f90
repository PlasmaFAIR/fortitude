module my_module
  implicit none
  interface
    integer function myfunc(x)
      integer, intent(in) :: x
    end function myfunc

    subroutine mysub(x)
      implicit none (external)
      integer, intent(in) :: x
    end subroutine mysub
  end interface
end module my_module

program my_program
  implicit none
  interface
    real function myfunc2(x)
      implicit real (a,z)
    end function myfunc2

    subroutine mysub2(x)
      integer, intent(inout) :: x
    end subroutine mysub2
  end interface
  write(*,*) 42
end program my_program

module interface_with_import
  use, intrinsic :: iso_fortran_env, only : wp => real64
  implicit none (type, external)
  interface
     subroutine sub
       import :: wp
     end subroutine sub
  end interface
end module interface_with_import
