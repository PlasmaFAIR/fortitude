module mymod
  implicit none
  use, intrinsic:: iso_fortran_env, only:int64, real64 ! This should add a space before the double colon and not change the single colon
  private
  integer ::i ! This should add a space after the double colon
  integer :: j ! ::in comments should be unchanged
  integer :: k ! This should be unchanged

  character::x, y, z! This should add spaces before and after the double colon

  allocate(character(10) :: x) ! This should be unchanged
  allocate(character(10):: y) ! This should add a space before the double colon
  allocate(character(10) ::z) ! This should add a space after the double colon

end module mymod
