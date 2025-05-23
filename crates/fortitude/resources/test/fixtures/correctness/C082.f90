module test_pointer_init
  integer, pointer :: global => null()  ! Ok at module level

  type :: my_type
    integer, pointer :: component => null()  ! Ok in types
  end type my_type
contains

  subroutine init_decl1()
    integer, pointer :: foo => null()
  end subroutine init_decl1

  subroutine init_decl2()
    integer, pointer :: foo(:) => null()  ! Not ok
    integer, pointer, dimension(:) :: foo1 => null()  ! Not ok
    integer, pointer, save :: bar => null()  ! Ok with explicit save
  end subroutine init_decl2

  subroutine init_decl3()
    integer, pointer :: foo, bar(:) => null(), quazz(:), zapp => null()
  end subroutine init_decl3

end module test_pointer_init
