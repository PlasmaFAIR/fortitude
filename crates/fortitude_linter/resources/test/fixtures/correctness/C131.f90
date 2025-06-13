module good_module
  ! This is good
  private
  public :: foo
  integer :: foo
end module good_module

module ok_module
  ! This is ok
  public
end module ok_module

module ok_module2
  ! This is ok too
  public
  private :: foo
  integer :: foo
end module ok_module2

module bad_module
  ! This is bad
end module bad_module

module only_some_private_module
  ! This is bad
  private :: foo
  public :: bar
  integer :: foo
  integer :: bar
end module only_some_private_module

program test
  ! Obviously fine
end program test
