module test
  integer :: global = 0  ! Ok at module level

  type :: my_type
    integer :: component = 1  ! Ok in types
  end type my_type
contains

  subroutine init_decl1()
    integer :: foo = 1
  end subroutine init_decl1

  subroutine init_decl2()
    integer, save :: foo = 1  ! Ok with explicit save
    integer, parameter :: bar = 2  ! Ok as parameter
  end subroutine init_decl2

  subroutine init_decl3()
    integer :: foo, bar = 1, quazz, zapp = 2
  end subroutine init_decl3

end module test
