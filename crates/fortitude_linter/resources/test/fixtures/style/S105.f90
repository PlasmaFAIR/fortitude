  !> my module
module mmod
  #if USE_MPI==1
  use mpi
#endif
implicit none

!> my type
type :: my_type
    integer :: i
  real :: y
contains
  procedure :: toString
        end type my_type

  contains

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
  end program mprog
