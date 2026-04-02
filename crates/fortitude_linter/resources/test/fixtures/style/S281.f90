module test
  save

  integer :: a

  contains
    subroutine should_have_save()
      integer, save :: b = 0
    end subroutine should_have_save
end module test

module test
  integer, save :: a
end module test
