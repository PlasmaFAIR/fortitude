  !> my module
module mmod
  #if USE_MPI==1
  use mpi
#endif
implicit none

interface
    module function interfaced_function(i) result(x)
          integer, intent(in) :: i
      end function interfaced_function
    end interface

interface minterface
          module procedure minterface_i,minterface_r
  end interface minterface

!> my type
type :: my_type
    integer :: i
  real :: y
contains
  procedure :: toString
        end type my_type

  contains

      integer function get_x() result(x)
      x = 1
      end function get_x

      integer function get_x_continued_line() &
    result(x)
      x = 1
      end function get_x

        function minterface_i(x)
          integer :: x
          print *, x
        end function minterface_i

    function minterface_r(x)
          real :: x
  print *, x
          end function minterface_r

      function wrapped_function( &
        x &
      )
        integer, intent(in) :: x

          print *, x
      end function wrapped_function

  subroutine line_continuation()
          integer :: i
  i = i + 1 &
  + 2 &
    + 3
  end subroutine

subroutine if_statements()
integer :: i
    i = i + 1
if (i == 1) i = 2
  if (i == 2) then; i = 3; end if;
if (i == 4) then
            i = 2
else if (i == 2) then
i = 4
      else
    i = 1
            end if

        named_if: if (i == 1) then
    i = i + 1
    end if
  end subroutine if_statements
    function my_func()
    integer :: i
  end function
end module mmod

submodule (mmod) msubmodule
contains
                module function interfaced_function(i) result(x)
        integer, intent(in) :: i
        x = i
      end function interfaced_function
end submodule msubmodule


    !> my program
program mprog
  use mmod
    implicit none

            print "Semicolon ; in string"
            print "Semicolon ; in string with ; not in string"; print "hello"
            print "semicolon in a quote within a quote ';'"
            print 'same as above but reversed quotes ";"'

call do_something()

block
    real :: x = 3.142
      print*, x
    y = x
inner: block
    real :: y = 12.1
      print*, y
end block inner
  end block

            contains

      subroutine select_cases
        integer :: i
        select case (i)
      case (1)
          i = 2
            case (2)
        i = 1
            end select
          i = 3

  end subroutine select_cases

  function do_construct
    integer :: i, j, x

      do i = 1, 10
  do j = i, 10
! Zero indented comment after control flow
    x = i * j
  end do
      end do

          named_do: do i = 1, 10
        print *, i
          end do
  end function do_construct

  subroutine associates
    integer :: i
  associate(x => i)
  print *, x
    end associate

    named_associate: associate(x => i)
  print *, x
    end associate named_associate
  end subroutine associates

subroutine labels(x)
integer, intent(inout) :: x
20        x = 3
x = 1;20   x = 4
end subroutine labels
  end program mprog
