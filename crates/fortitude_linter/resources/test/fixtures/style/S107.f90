module procedures
  implicit none
contains
  subroutine foo()
    print *, "foo"
  end subroutine foo ! end of foo
  function bar() result(value)
    integer :: value
    value = 1
  end function bar

  subroutine baz()
    print *, "baz"
  end subroutine baz
end module procedures

subroutine external_one()
  print *, "one"
end subroutine external_one
subroutine external_two()
  print *, "two"
end subroutine external_two
