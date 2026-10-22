module test_user_defined
  use, intrinsic :: iso_fortran_env, only: dp => real64
  implicit none

  contains

  subroutine srand(seed)
    integer, intent(in) :: seed
    integer, allocatable :: seeds(:)
    integer :: n

    call random_seed(size = n)
    allocate(seeds(n))
    seeds(1) = seed
    call random_seed(put = seeds)
  end

  function rand() result(x)
    real(dp) :: x
    call random_number(x)
  end

  subroutine use_user_defined
    call srand(1234)
    print *, rand()
  end

end module test_user_defined

module test_use_import
  use test_user_defined, only: srand, rand
  implicit none

  contains

  subroutine use_imported
    call srand(1234)
    print *, rand()
  end

end module test_use_import

module test_false_positives
  use, intrinsic :: iso_fortran_env, only: dp => real64
  implicit none

  contains

  subroutine test_array_assignment()
    real(dp) :: rand(10)
    real(dp) :: srand(5)

    ! Should not trigger as rand and srand are array variables
    rand(:) = 0.0_dp
    srand(:) = rand(1:5)
  end subroutine test_array_assignment

  subroutine test_variable()
    real(dp) :: rand(5, 5)
    real(dp) :: srand(10)
    ! Should not trigger as `rand` and `srand` here are variables
    call random_number(rand)
    call random_number(srand)
  end subroutine test_variable

end module test_false_positives

program test_rand
  implicit none
  integer,parameter :: seed = 86456
  call srand(seed)
  print *, rand(), rand()
  print *, rand(seed)
end program test_rand
