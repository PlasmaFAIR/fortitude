program test
  type, private, abstract :: abstract_type
  end type abstract_type
  type, public, extends(abstract_type) :: concrete_type
  end type concrete_type
end program
