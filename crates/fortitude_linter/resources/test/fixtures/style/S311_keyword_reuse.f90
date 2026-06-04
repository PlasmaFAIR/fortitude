module module

    implicit none (type, external)
    private

    real :: private
    real :: public
    real :: contains
    real, allocatable :: allocatable
    real, pointer :: pointer
    real, target :: target
    integer, parameter :: parameter = 3

    public :: public  ! Shouldn't trigger here, only on the declaration above

    type type
        integer :: extends
    end type type

contains

    subroutine sub()
        integer :: i
        integer :: j
        do: do i = 1, 10
            cycle: do j = 1, 10
                print *, i, j
                if (j == 5) then
                    cycle cycle
                end if
                if (i == 6) then
                    exit if
                end if
            end do cycle
        end do if
    end subroutine sub

end module module

program program

    implicit none (type, external)


    integer :: integer
    real :: real
    character :: character
    logical :: logical
    complex :: complex

    block: block
        integer :: allocate, deallocate, write
        print *, allocate, deallocate, write
    end block block

    block
        integer :: namelist
    end block

contains

    subroutine subroutine(print, in, out, inout)
        integer, intent(inout) :: print
        integer, intent(in) :: in
        integer, intent(out) :: out
        integer, intent(inout) :: inout
        print *, print, in, inout
        out = in + inout
        inout = in + 1
    end subroutine subroutine

    integer function function(pure,  elemental, recursive) result(result)
        integer, intent(in) :: pure
        integer, intent(in) :: elemental
        integer, intent(in) :: recursive
        result = pure + elemental + recursive
    end function function

end program program
