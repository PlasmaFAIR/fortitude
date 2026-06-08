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
        end type my_type

  contains

  subroutine do_something()
          integer :: i
  i = i + 1 &
  + 2 &
    + 3
  end subroutine

subroutine do_something_else()
integer :: i
    i = i + 1
  end subroutine do_something_else
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
