program test
  type foo
  contains
    procedure(bar_impl), deferred :: bar
  end type
  abstract interface
    subroutine bar_impl
    end subroutine
  end interface
end program
