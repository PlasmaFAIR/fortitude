integer function f(x) result(y)
  integer, intent(in) :: x
  y = x
endfunction f

module m
  implicit none

  interface
    function f(x)
      integer, intent(in) :: x
    end function f
    module integer function submodule_f1(x)
      integer, intent(in) :: x
    end function submodule_f1
    module integer function submodule_f2(x)
      integer, intent(in) :: x
    end function submodule_f2
    module subroutine procedure_1()
    end subroutine procedure_1
    module subroutine procedure_2()
    end subroutine procedure_2
  end interface

  type :: point_2d
    real :: x, y
  end type point_2d

  type, extends(point_2d) :: point_3d
    real :: z
  endtype point_3d

  enum, bind(c)
    enumerator :: a = 1
  end enum

  enum, bind(c)
    enumerator :: b = 2
  EndEnum

contains
  integer function f1(x, y)
    integer, intent(inoUt) :: x
    integer, intent(in out) :: y
    select case (x)
      case(1)
        print *, x
      case default
        print *, y
    end select
    selectcase (y)
      case(1)
        f1 = y
      case default
        f1 = x
    endselect
  end function f1

  integer function f2(x)
    integer, intent(in) :: x
    block
      integer :: j
      j = x
    end block
    block_name: block
      integer :: k
    endblock block_name
    print *, "x"
    f2 = x
  endfunction f2

  subroutine s1()
    integer, parameter :: N = 3
    integer :: matrix(N, N)
    integer :: i, j
    associate(A => matrix)
      forall(i = 1:N)
        A(i, i) = 1.0
      end forall
    end associate
    named_associate: associate(A => matrix)
      forall(i = 1:N, j = 1:N, i /= j)
        A(i, j) = 0.0
      endforall
    endassociate named_associate
    print *, A
  end subroutine s1

  subroutine s2(p2d, p3d)
    type(point_2d), target :: p2d
    type(point_3d), target :: p3d
    class(point_2d), pointer :: ptr
    ptr => p3d
    select type(ptr)
      class is (point_2d) ! should trigger for either
        print *, ptr%x, ptr%y
    end select
    selecttype(ptr)
      type is(point_3d) ! only trigger for 3d
        print *, ptr%x, ptr%y, ptr%z
    endselect
  endsubroutine s2

end module m

module m2
  implicit none
contains
  function coarray_stuff() result(total)
    integer :: this, images, team_id, total
    total = 0
    this = this_image()
    images = num_images()
    form team(1, team_id)
    change team(team_id)
      critical
        total = total + this
      end critical
      sync all
    end team
    form team(1, team_id)
    change team(team_id)
      critical
        total = total + this
      endcritical
      sync all
    endteam
  end function coarray_stuff
endmodule m2

submodule (m) s1
  implicit none
contains
  module procedure procedure_1
  endprocedure procedure_1
  integer function submodule_f1(x)
    integer, intent(in) :: x
    submodule_f1 = x
  end function submodule_f1
end submodule s1

submodule (m) s2
  implicit none
contains
  module procedure procedure_2
  end procedure procedure_2
  integer function submodule_f2(x)
    integer, intent(in) :: x
    submodule_f2 = x
  end function submodule_f2
endSubmodule s2

program p
  use :: m, only: s1, s2, point_2d, point_3d
  implicit none
  integer, parameter :: n = 10
  double precision :: A(n)
  double complex :: b
  DoublePrecision :: c
  DoubleComplex :: d
  integer :: i, j, k
  type(point_2d) :: p2d
  type(point_3d) :: p3d
  ! Ignore all of the following
  character(len=10) :: elsewhere, endif, endmodule, selectcase
  interface
    function f(x)
      integer, intent(in) :: x
    end function f
  endinterface

  do i = 1, n
    A(i) = real(i)
  end do

  do_name: do i = 1, n
    A(i) = A(i) + 1
  enddo do_name

  where (A > 5.0)
    A = A * 2.0
  eLseWHere
    A = A + 10.0
  endwheRe

  where (A > 15.0)
    A = A + 3.0
  ELSE where
    A = A + 4.0
  end where

  if (A(1) == 11.0) then
    print *, "foo"
  eLsEIf (A(1) == 12.0) then
    print *, "bar"
  else if (A(2) == 12.0) then
    print *, "baz"
  else
    print *, "helloworld"
  end if

  if (A(1) == 11.0) then
    print *, "foo"
  EndIf

  call s1()

  p2d%x = 1.0
  p2d%y = 2.0
  p3d%x = 3.0
  p3d%y = 4.0
  p3d%z = 5.0
  call s2(p2d, p3d)

  i = 1
  10 continue
  i = i + 1
  if (i < 10) goTo 10
  if (i < 20) gO To 10
  if (i < 30) go  & ! helpful comment!
    to 10

contains
  integer function fff(x)
    integer, intent(in & !helpful comment
      out) :: x
    fff = x
  end function fff
endprogram p
