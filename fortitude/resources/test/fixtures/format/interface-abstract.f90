abstract interface
  subroutine ex(a,b,c)
    implicit none
    real :: a,b(10,2)
    integer :: c
  end subroutine ex
  function why(t,f)
    implicit none
    logical,intent(in) :: t,f
    logical :: why
  end function why
end interface
