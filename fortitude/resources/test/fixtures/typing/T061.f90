program test
  implicit none (type, external)

  interface
    integer function f(foo, bar)
      implicit none
      integer, intent(in) :: foo
      integer, intent(in):: bar
    end function f
  end interface

  external :: g

end program test
