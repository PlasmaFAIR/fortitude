program test
  type foo
    procedure(some_interface), pointer :: bar => null()
    procedure(some_interface), pointer, nopass :: quux
  end type foo
end program test
