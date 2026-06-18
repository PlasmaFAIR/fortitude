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

        function minterface_i(x)
          integer :: x
          print *, x
        end function minterface_i

    function minterface_r(x)
          real :: x
  print *, x
          end function minterface_r

  subroutine line_continuation()
          integer :: i
  i = i + 1 &
  + 2 &
    + 3
  end subroutine

subroutine if_statements()
integer :: i
    i = i + 1
if (i == 2) then; i = 3; end if;
if (i == 4) then
            i = 2
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
  end program mprog
