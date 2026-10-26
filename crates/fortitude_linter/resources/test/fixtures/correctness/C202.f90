module default_public
  ! Default accessibility is public, so all mutable variables are exposed
  implicit none
  integer :: counter = 0                ! Bad
  real :: x, y                          ! Bad (x and y)
  real, allocatable :: values(:)        ! Bad
  character(len=8) :: label = "label"   ! Bad
  integer, save :: saved = 1            ! Bad
  integer, pointer :: ptr => null()     ! Bad

  integer, parameter :: n = 3           ! Fine: constant
  integer, protected :: prot = 1        ! Fine: can't be modified outside module
  integer, private :: hidden = 1        ! Fine: explicitly private
  integer :: private_later              ! Fine: made private below
  private :: private_later
contains
  subroutine local()
    integer :: scratch = 0              ! Fine: not module scope
    scratch = scratch + 1
  end subroutine local
end module default_public

module bare_private
  ! Everything is private unless made public
  implicit none (type, external)
  private
  public :: exposed, shout, listed_a, listed_b

  integer :: internal = 0               ! Fine: private by default
  integer :: exposed = 0                ! Bad: public statement
  integer :: Shout                      ! Bad: names are case-insensitive
  integer :: listed_a, listed_b         ! Bad (listed_a and listed_b)
  integer, public :: attr_public        ! Bad: public attribute
  integer, public, protected :: ro = 1  ! Fine: protected
  integer, public, parameter :: pi = 3  ! Fine: constant
end module bare_private

module explicit_public
  implicit none
  public                                ! Bare `public` is also the default
  private :: nope
  integer :: nope                       ! Fine: private statement
  integer :: yes                        ! Bad
end module explicit_public

module with_type
  implicit none
  type :: point
    real :: x                           ! Fine: derived type component
    real :: y = 0.0                     ! Fine: derived type component
  end type point
  type(point) :: origin                 ! Bad
  type(point), parameter :: unit = point(1.0, 1.0)  ! Fine: constant
end module with_type

submodule (with_type) with_type_impl
  integer :: sub_counter = 0            ! Bad
end submodule with_type_impl

program test
  implicit none
  integer :: fine = 0                   ! Fine: not a module
end program test
