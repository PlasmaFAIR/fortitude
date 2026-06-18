module my_mod

  use, intrinsic :: iso_fortran_env, only: real32

  implicit none (type, external)
  private

  real, allocatable :: x(:)

contains

  subroutine initialise(n)
    integer, intent(in) :: n
    real, allocatable :: x(:)  ! This is a bug!
    real :: real32 ! This is a bug!

    allocate(x(n))

  end subroutine initialise

  subroutine selection_sort(arr)
    use some_mod, only: x => y  ! This is a bug!
    real, intent(inout) :: arr(:)
    integer :: i

    do i = 1, size(arr)
      call helper(arr(i:size(arr)))
    end do

  contains

    !! Finds minimum element of an array, swaps it with the start
    subroutine helper(arr)
      real, intent(inout) :: arr(:) ! Allowed: is a dummy arg
      real :: min_val
      integer :: min_idx
      integer :: i  ! Allowed: is a loop variable

      min_val = arr(1)
      min_idx = 1
      do i = 1, size(arr)
        if (arr(i) < min_val) then
          min_val = arr(i)
          min_idx = i
        end if
      end do
      arr(min_idx) = arr(1)
      arr(1) = min_val

    end subroutine helper

  end subroutine selection_sort

end module my_mod
