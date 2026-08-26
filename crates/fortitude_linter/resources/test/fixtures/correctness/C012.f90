program test
  implicit none (type, external)

  ! This should be flagged by C012
  select type(pet)
  type is (dog_t)
    print *, "Dog"
  class is (animal_t)
    print *, "Animal"
  end select

  ! This should pass C012
  select type(pet)
  type is (dog_t)
    print *, "Dog"
  class is (animal_t)
    print *, "Animal"
  class default
    print *, "Unknown pet type"
  end select
end program test
