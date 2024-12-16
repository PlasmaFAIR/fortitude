program test
  implicit none (type, external)
  integer :: foo = 4
  select case(foo)
  case(1)
    print*, "one"
  case(2)
    print*, "two"
  end select
  
  select case(foo)
  case(1)
    print*, "one"
  case(2)
    print*, "two"
  case default
    print*, "not one or two"
  end select
end program test
