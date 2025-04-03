module test
  abstract interface
    subroutine do_nothing
    end subroutine do_nothing
  end interface
contains
  subroutine apply(f, g)
    procedure(do_nothing) :: f
    procedure(do_nothing), optional :: g
    call f()
  end subroutine apply
end module test
